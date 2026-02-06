# LLM User Validation

**Status: Draft**

## Purpose

llm-tool-test is a framework for validating that a CLI tool works well when used by LLM coding agents. This is **LLM User Validation** — testing the tool from the LLM's perspective rather than from a human user's perspective.

The framework serves two audiences:

1. **CLI tool authors**: "Can an LLM use my tool effectively given my documentation?" — validates that the tool's CLI design, help output, error messages, and documentation are sufficient for an LLM agent to accomplish real tasks.

2. **Guidance authors**: "Does my AGENTS.md or documentation effectively guide LLMs?" — validates that project-level instructions, workflow descriptions, and tool usage patterns produce good outcomes when followed by an LLM agent.

Both audiences share the same core question: if you give an LLM agent your tool's documentation and a task, can it accomplish the task?

## Core Concept

The quality of an LLM-tool interaction reveals:

- Whether the tool's CLI design is LLM-friendly (flag names, subcommand structure, defaults)
- Whether the documentation is clear and complete enough for an LLM to use without guessing
- Whether error messages guide recovery (can the LLM self-correct after a mistake?)
- Whether the tool's output is parseable and actionable (can the LLM extract IDs, status, and results?)

llm-tool-test automates this evaluation. It launches a real LLM coding agent in an isolated workspace, gives it a task that requires using the target CLI tool, captures the full interaction transcript, and evaluates the result against defined criteria.

The framework is tool-agnostic. It does not link against the target tool's code or assume anything about its internals. It treats the target tool as a black box — exactly as an LLM agent would.

---

## Architecture Overview

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Scenarios     │────▶│  llm-tool-test  │────▶│  LLM Agent      │
│   (YAML)        │     │  (harness)      │     │  Adapters        │
└─────────────────┘     └────────┬────────┘     └────────┬────────┘
                                 │                       │
                        ┌────────┴────────┐              │ launches
                        │  Target Tool    │              ▼
                        │  Configuration  │     ┌─────────────────┐
                        └─────────────────┘     │  LLM Agent      │
                                                │  (opencode,     │
                                                │   claude-code)  │
                                                └────────┬────────┘
                                                         │ uses
                                                         ▼
                                                ┌─────────────────┐
                                                │  Target CLI     │
                        ┌─────────────────┐     │  Tool           │
                        │  Transcript     │◀────└─────────────────┘
                        │  Capture        │
                        └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  Evaluator      │
                        │  (3-layer)      │
                        └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  Results &      │
                        │  Artifacts      │
                        └─────────────────┘
```

### Key Components

1. **Scenarios** — YAML files that define tasks and evaluation criteria. See [specs/scenarios.md](scenarios.md).
2. **Target Tool Configuration** — declares what CLI tool is being tested, including its commands and how to inspect its state.
3. **LLM Agent Adapters** — invoke LLM coding agents (opencode, claude-code) that then use the target tool.
4. **Transcript Capture** — records the full agent interaction via PTY.
5. **Evaluator** — three-layer quality measurement. See [specs/evaluation.md](evaluation.md).
6. **Results & Artifacts** — structured output for analysis and review.

### Key Architectural Decisions

1. **Separate binary**: `llm-tool-test` is a standalone tool, not a library or test harness linked into the target tool.

2. **Black-box testing**: The harness treats the target as an external CLI tool. It doesn't link against the tool's library code.

3. **Adapter indirection**: The harness invokes LLM agents, not the target tool. The LLM agent invokes the target tool. This mirrors real-world usage.

4. **Tool-agnostic**: The harness can test any CLI tool with appropriate scenario definitions and target tool configuration.

---

## LLM Agent Adapters

Adapters handle the specifics of launching and communicating with each LLM coding agent. An important distinction: adapters invoke the **LLM coding agent** (opencode, claude-code), not the target CLI tool. The agent then uses the target tool autonomously.

### How Adapters Work

1. The harness prepares an isolated workspace with the scenario's fixture files (AGENTS.md, README, seed data).
2. The adapter launches the LLM agent in that workspace with a task prompt.
3. The agent reads the documentation, uses the target CLI tool to accomplish the task, and exits.
4. The adapter captures the full interaction transcript and extracts cost/token metadata.

### Trait Definition

```rust
pub trait ToolAdapter: Send + Sync {
    /// Check if tool is installed and authenticated.
    fn is_available(&self) -> Result<ToolStatus, AdapterError>;

    /// Check if the tool is available and ready to use.
    fn check_availability(&self) -> anyhow::Result<()>;

    /// Run the tool with the given scenario in the specified working directory.
    /// Returns (output, exit_code, cost_usd, token_usage).
    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, Option<f64>, Option<TokenUsage>)>;
}

pub struct ToolStatus {
    pub available: bool,
    pub authenticated: bool,
}

pub struct TokenUsage {
    pub input: usize,
    pub output: usize,
}
```

### Adapter Responsibilities

- **Execution**: Launch the agent process with appropriate flags and prompt.
- **Transcript capture**: Capture full PTY output from the agent session.
- **Structured event emission**: When possible, adapters should emit structured events (`tool_call`, `tool_result` with exit codes) to `events.jsonl`. This feeds Layer 1 interaction metrics directly. Raw transcript parsing is the fallback when structured events are unavailable.
- **Cost/token tracking**: Parse actual cost and token usage from agent output when available. Do not estimate from character counts.
- **Timeout enforcement**: Kill the agent process if it exceeds the configured timeout.

### Available Adapters

| Adapter | Agent Invocation | Status |
|---------|-----------------|--------|
| opencode | `opencode <prompt>` | Primary |
| claude-code | `claude --prompt <text>` | Primary |

---

## Transcript Capture

Transcripts are the primary artifact of a test run. For CLI tool authors, they reveal how the LLM interprets documentation and error messages. For guidance authors, they show exactly how the LLM follows (or deviates from) instructions.

### PTY-Based Capture

The harness uses pseudo-terminal capture to get the complete interaction including:

- ANSI colors and formatting
- Interactive prompts and responses
- Real-time streaming output
- Tool invocations and results

Fallback to piped stdout/stderr if PTY is unavailable.

### Event Log Format

Structured events extracted from the raw transcript:

```jsonl
{"ts": 1705500000.123, "event": "spawn", "command": "opencode", "args": ["--prompt", "..."]}
{"ts": 1705500001.456, "event": "output", "text": "I'll work on this task...\n"}
{"ts": 1705500005.789, "event": "tool_call", "tool": "bash", "command": "my-tool create \"Main Concept\""}
{"ts": 1705500006.012, "event": "tool_result", "output": "Created item abc123\n", "exit_code": 0}
{"ts": 1705500030.000, "event": "complete", "exit_code": 0, "duration_secs": 30.0}
```

### Artifact Structure

Each run produces:

```
llm-tool-test-results/<timestamp>-<tool>-<model>-<scenario>/
├── transcript.raw.txt      # Complete PTY output
├── events.jsonl            # Structured event log
├── metrics.json            # Run metadata and measurements
├── evaluation.md           # Human-readable summary
└── fixture/                # Working directory, preserved after run
```

### Run Metadata

```json
{
  "scenario_id": "capture_article_basic",
  "scenario_hash": "abc123def456",
  "tool": "opencode",
  "model": "claude-sonnet-4-20250514",
  "target_tool_version": "0.1.0",
  "timestamp": "2025-01-17T12:00:00Z",
  "duration_secs": 45.3,
  "cost_estimate_usd": 0.023,
  "token_usage": {
    "input": 1500,
    "output": 800
  }
}
```

---

## Execution Flow

A single scenario run proceeds through these steps:

### 1. Load Scenario

Parse the YAML scenario file. Resolve the target tool configuration. Validate that all referenced fixtures, rubrics, and commands exist.

### 2. Prepare Isolated Workspace

Copy the scenario's fixture template into a `fixture/` directory inside the results directory. This includes AGENTS.md, README, seed data, and any pre-initialized tool state. The harness creates a fresh copy for each run. Because the fixture lives inside the results directory, any files the agent creates or modifies are automatically preserved as post-run artifacts.

### 3. Run Setup Commands

Execute any `setup` commands defined in the scenario (e.g., initializing the target tool, importing seed data). These run before the LLM agent is launched.

### 4. Launch LLM Agent via Adapter

The adapter launches the LLM coding agent (opencode, claude-code) in the prepared workspace with the scenario's task prompt. The agent autonomously reads documentation and uses the target CLI tool to accomplish the task.

### 5. Capture Transcript

The full PTY output is captured during the agent session. After the agent exits (or is killed by timeout), the raw transcript is written to disk and structured events are extracted.

### 6. Post-Execution Scripts

Run any post-execution scripts defined in `scripts.post`. These handle tool-specific state capture, data export, or normalization — anything that needs to happen after the agent finishes but before evaluation. Failures are logged as warnings but do not block evaluation.

See [specs/scripts.md](scripts.md) for details.

### 7. Evaluate

The evaluator runs:
- **Interaction quality metrics**: command error rate, self-correction, tool usage patterns (always runs)
- **Gates**: built-in gates and script gates, all run regardless of earlier failures
- **Custom evaluators**: scripts that produce additional metrics/scores (see [specs/scripts.md](scripts.md))
- **LLM-as-judge**: qualitative assessment against a rubric (optional, if enabled and gates pass)

See [specs/evaluation.md](evaluation.md) for details.

### 8. Generate Artifacts

Write the transcript, event log, metrics, and evaluation summary to the results directory. The fixture (working directory) is already in the results directory and is preserved as-is.

---

## Cost Management

LLM API calls are expensive. The harness enforces cost controls at multiple levels.

### Budget Enforcement

1. **Per-run limit**: From the scenario's `cost.max_usd` field.
2. **Session limit**: From `--max-usd` flag or `LLM_TOOL_TEST_BUDGET_USD` environment variable.
3. **Estimate before run**: Warn if estimated cost exceeds the limit.
4. **Track actual cost**: Log actual cost (from adapter output) to results for trend analysis.

### Caching

Cache key components:
- Scenario YAML hash
- Prompt content hash
- Target tool version
- Agent tool + model identifier

If cache hit, reuse transcript and evaluation results. Disable with `--no-cache`.

### Dry Run Mode

`--dry-run` shows:
- Scenarios that would run
- Estimated prompt sizes
- Estimated costs
- Cache status (hit/miss)

No LLM API calls are made.

### Environment Variables

```bash
LLM_TOOL_TEST_ENABLED=1          # Must be set to run tests (safety)
LLM_TOOL_TEST_BUDGET_USD=5.00    # Session budget limit
LLM_TOOL_TEST_TOOL=opencode      # Default LLM agent tool
LLM_TOOL_TEST_JUDGE=gpt-4o-mini  # Judge model for LLM-as-judge evaluation
```

---

## Security

### Transcript Redaction

Before writing the human-readable evaluation summary, redact:
- API keys and tokens
- Passwords and secrets
- Email addresses (optional)
- File paths containing usernames

The raw transcript is preserved but marked as sensitive. It should not be committed to version control.

### Gitignore

```gitignore
# LLM test artifacts (volatile, potentially sensitive)
llm-tool-test-results/
```

---

## CLI Interface

### Commands

```bash
# Run scenarios
llm-tool-test run --scenario capture_basic  # Run specific scenario
llm-tool-test run --all                     # Run all scenarios
llm-tool-test run --all --tags capture      # Run by tags
llm-tool-test run --all --tier 1            # Run by tier
llm-tool-test run --tool opencode           # Run with specific agent
llm-tool-test run --max-usd 1.00            # Budget limit
llm-tool-test run --dry-run                 # Show what would run + cost estimate

# Matrix runs
llm-tool-test run --all --tools opencode,claude-code --models gpt-4o,claude-sonnet

# List and inspect scenarios
llm-tool-test scenarios                     # List all scenarios
llm-tool-test scenarios --tags capture      # Filter by tags
llm-tool-test show <scenario-id>            # Show scenario details

# Maintenance
llm-tool-test clean --older-than 7d         # Clean old artifacts
llm-tool-test clean                         # Clean all artifacts
```

---

## Results Tracking

### Storage

Results are stored in an append-only format for trend analysis:

```
llm-tool-test-results/
├── results.jsonl           # Append-only run results
└── results.db              # Optional SQLite for queries
```

### Regression Detection

Compare against baseline runs:
- Score degradation > 15% triggers warning
- Gate failures that previously passed trigger alert
- Cost increases > 50% trigger warning

---

## Not In Scope

- CI integration (too expensive for automated runs)
- Multi-model statistical benchmarking (future)
- Real-time cost tracking via provider APIs
- Interactive test authoring UI

---

## Cross-References

- Scenario format and target tool configuration: see [specs/scenarios.md](scenarios.md)
- Quality measurement and evaluation: see [specs/evaluation.md](evaluation.md)
- Scripts and extension hooks: see [specs/scripts.md](scripts.md)
- Distribution and installation: see [specs/distribution.md](distribution.md)
