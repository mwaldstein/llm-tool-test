# LLM Tool Test

A framework for verifying that CLI tools work correctly when driven by LLM coding agents. It launches real agents against your tool in isolated scenarios, captures full transcripts, and evaluates the results across multiple quality dimensions.

The transcript is a first-class artifact. When a scenario fails, the transcript shows you exactly where the agent went wrong — whether due to unclear docs, missing guidance, or tool behavior the LLM couldn't navigate.

## Who is this for?

**CLI tool authors** who want confidence that their tool works well when an LLM agent is the operator. Does the agent invoke the right subcommands? Does it recover from errors? Do your `--help` strings actually help?

**Skills/guidance authors** (people writing AGENTS.md files, tool documentation, or system prompts) who want to test whether their documentation effectively guides LLMs through real workflows.

## How it works

1. You define **scenarios** — structured test cases with a prompt, expected outcomes, and evaluation gates.
2. The framework launches a real LLM agent (opencode, claude-code) in an isolated environment with your tool available.
3. The agent works through the prompt. The full interaction is captured as a **transcript**.
4. Results are evaluated on three layers:
   - **Interaction quality** — derived from the transcript (errors, retries, confusion)
   - **Outcome assertions** — configurable gates that check concrete results (files created, commands succeed, expected output)
   - **LLM-as-judge** — rubric-based evaluation of overall quality

Each run produces an `evaluation.md` with pass/fail, metrics, and links to all artifacts.

## Safety

**Required**: Set `LLM_TOOL_TEST_ENABLED=1` before running tests.

```bash
export LLM_TOOL_TEST_ENABLED=1
```

This prevents accidental expensive LLM API calls.

## Basic Commands

### Run Scenarios

```bash
# Run single scenario
llm-tool-test run --scenario capture_basic

# Run all scenarios
llm-tool-test run --all

# Filter by tags or tier
llm-tool-test run --all --tags smoke
llm-tool-test run --all --tier 1

# Dry run (no LLM calls)
llm-tool-test run --scenario capture_basic --dry-run
```

### List Scenarios

```bash
# List all
llm-tool-test scenarios

# Filter
llm-tool-test scenarios --tags capture
llm-tool-test scenarios --tier 0
```

### Show Scenario Details

```bash
llm-tool-test show capture_basic
```

### Clean Artifacts

```bash
# Clean old results (older than 7 days)
llm-tool-test clean --older-than "7d"

# Clean all
llm-tool-test clean
```

## Matrix Runs

Test multiple tools/models in one run:

```bash
llm-tool-test run --all --tools opencode,claude-code --models gpt-4o,claude-sonnet
```

## Interpreting Results

Each run generates an `evaluation.md` with:

**Summary**: Scenario name, tool, model, outcome (Pass/Fail)

**Metrics**:
- Gates Passed: X/N — test criteria satisfied
- Duration: Time taken
- Cost: Estimated API cost
- Composite Score: Available when configured per scenario (0.0-1.0)

**Human Review**: Manual scoring section (you fill in)

**Links**: Transcript, metrics, events

### Gate Types

Tests pass when all gates succeed. Gates are domain-independent assertions that verify outcomes after the LLM tool completes the task:

- `command_succeeds`: Shell command exits successfully (exit code 0)
- `command_output_contains`: Command stdout contains expected substring
- `command_output_matches`: Command stdout matches regex pattern
- `command_json_path`: JSON output contains data matching a path assertion (e.g., `$.items[0].status exists`, `$.count > 5`)
- `file_exists`: File present at expected path in fixture directory
- `file_contains`: File content contains expected substring
- `file_matches`: File content matches regex pattern
- `no_transcript_errors`: No command errors detected in transcript
- `script`: Custom script gate that can return pass/fail via exit code or JSON output (`{"passed": true, "message": "..."}`)

## Typical Workflow

```bash
# 1. Enable safety flag
export LLM_TOOL_TEST_ENABLED=1

# 2. List available scenarios
llm-tool-test scenarios

# 3. Run specific scenario
llm-tool-test run --scenario capture_basic --tool opencode

# 4. Check results
cat llm-tool-test-results/<timestamp>*/evaluation.md

# 5. Review transcript for debugging
cat llm-tool-test-results/<timestamp>*/transcript.raw.txt
```

## Configuration

Optional `llm-tool-test-config.toml` for tool/model configuration and cost tracking:

```toml
[tools.opencode]
name = "opencode"
command = "opencode"
models = ["gpt-4o", "claude-sonnet"]

[profiles.quick]
name = "quick"
tools = ["opencode"]
models = ["gpt-4o"]

[models.gpt-4o]
input_cost_per_1k_tokens = 2.5
output_cost_per_1k_tokens = 10.0
```

Copy `llm-tool-test-config.example.toml` as a starting point.

## Troubleshooting

**"LLM testing is disabled"**: Set `LLM_TOOL_TEST_ENABLED=1`

**Scenario not found**: Check it's in `fixtures/` directory, use `llm-tool-test scenarios` to list

**Gate failures**: Check metrics.json and transcript.raw.txt for details

**Timeout errors**: Increase timeout with `--timeout-secs 600`

**Cache issues**: Disable caching with `--no-cache` or clean old results

**Composite score low**: Review which gates failed in evaluation.md

**Tool not supported**: Available tools: opencode, claude-code. (Note: amp is experimental/de-prioritized)

## Results Location

All test artifacts stored in `llm-tool-test-results/<timestamp>-<tool>-<model>-<scenario>/`

## Installation

```bash
cargo build --release
# Binary will be at target/release/llm-tool-test
```
