# Implementation Plan: Scripts Support & Snapshot Removal

This plan implements [specs/scripts.md](specs/scripts.md) and removes the snapshot mechanism per the updated specs. It also removes qipu-specific code from the internals, since the specs now define a tool-agnostic framework.

The plan is ordered so each step produces a compiling, testable codebase. Steps within a phase can be done in order. Phases should be done sequentially.

---

## Phase 1: Remove Snapshot & Qipu-Specific Code

These changes remove dead code and qipu coupling. No new functionality.

### 1.1 Remove `create_store_snapshot` and its call site

**Status:** ✅ Complete

**Files:**
- `src/transcript/writer.rs` — Delete `create_store_snapshot()` (lines 143–177) and `copy_dir()` (lines 180–197). Remove the `Store Snapshot` link from `write_evaluation()` (line 382).
- `src/run/transcript.rs` — Remove the call `writer.create_store_snapshot(&env.root)?;` (line 27).
- `src/transcript/tests/writer_tests.rs` — Removed test assertion for Store Snapshot link.

**Verify:** `cargo build` — no compile errors. `cargo test` — test for writer updated to remove snapshot assertion.

### 1.2 Remove `QualityMetrics` / `StoreAnalyzer` from evaluation

**Status:** ✅ Complete

The `store_analysis.rs` module and `QualityMetrics` are qipu-specific (they parse a notes/links export format). Remove them from the evaluation pipeline. This is a larger change that touches multiple files.

**Files:**
- ✅ `src/store_analysis.rs` — Deleted the entire file.
- ✅ `src/evaluation.rs` — Removed `use crate::store_analysis::QualityMetrics;`. Removed calls to `compute_quality_or_default()`. Removed quality from `EvaluationMetrics` struct and `build_metrics()`. Removed note_count and link_count fields.
- ✅ `src/eval_helpers.rs` — Deleted `compute_quality_metrics()`. Updated `compute_composite_score()` to remove the quality weight (redistributed: judge 55%, gates 35%, efficiency 10%).
- ✅ `src/run/transcript.rs` — Removed `quality: ...` fields and note/link count fields from `RunReport` and `EvaluationReport` construction.
- ✅ `src/run/records.rs` — Removed `quality: QualityMetricsRecord { ... }` and note/link counts from `build_result_record()` and `handle_dry_run()`.
- ✅ `src/results/types/mod.rs` — Deleted `QualityMetricsRecord` struct. Removed `quality` field from `EvaluationMetricsRecord`. Removed `note_count` and `link_count` fields (these are qipu-specific counts).
- ✅ `src/transcript/types.rs` — Deleted `QualityReport` struct. Removed `quality` field and note/link counts from `RunReport` and `EvaluationReport`.
- ✅ `src/transcript/writer.rs` — Deleted `write_quality_section()`. Removed quality from `write_report()`. Removed note/link counts from `write_evaluation_section()` and `write_evaluation()`.
- ✅ `src/output.rs` — Removed `Notes:` and `Links:` lines from `print_result_summary()`.
- ✅ `src/commands.rs` — Removed `Notes:` and `Links:` lines from output.
- ✅ `src/main.rs` — Removed `mod store_analysis;` declaration.
- ✅ Test files updated: `src/results/test_helpers.rs`, `src/results/types/tests.rs`, `src/transcript/tests/writer_tests.rs`, `src/eval_tests_score.rs`

**Verify:** `cargo build` ✅ — no compile errors. 

**Note:** Tests in `eval_tests_doctor.rs` and `eval_tests_gates.rs` are failing because they depend on qipu binary. These will be addressed in Phase 1.3 which removes qipu-specific eval helpers and their tests.

### 1.3 Remove qipu-specific eval helpers

**Status:** ✅ Complete

**Files:**
- ✅ `src/eval_helpers.rs` — Deleted all qipu-specific functions: `get_qipu_path()`, `run_qipu_json()`, `create_note_with_stdin()`, `count_notes()`, `count_links()`, `search_hit()`, `note_exists()`, `link_exists()`, `tag_exists()`, `content_contains()`, `command_succeeds()`, `doctor_passes()`. Kept only generic functions: `no_transcript_errors()`, `compute_efficiency_metrics()`, and `compute_composite_score()`.
- ✅ `src/evaluation.rs` — Stubbed qipu-specific gate evaluators (`MinNotes`, `MinLinks`, `SearchHit`, `NoteExists`, `LinkExists`, `TagExists`, `ContentContains`, `DoctorPasses`) to return "not implemented" failures. Implemented `CommandSucceeds` as a generic gate that runs arbitrary shell commands. Removed store snapshot reference from judge evaluation prompt.
- ✅ `src/eval_tests_doctor.rs` — Deleted this file (qipu doctor tests).
- ✅ `src/eval_tests_gates.rs` — Deleted this file (qipu-specific gate tests).
- ✅ `src/main.rs` — Removed module declarations for `eval_tests_doctor` and `eval_tests_gates`.
- ✅ `src/adapter/mock.rs` — Updated to be tool-agnostic. Removed all qipu command execution. Now just returns a simple mock transcript without executing any commands.
- ✅ `src/adapter/mock_test.rs` — Updated tests to be generic and not depend on qipu.
- ✅ `src/run/transcript.rs` — Fixed unused variable warnings by prefixing with underscore.

**Verify:** `cargo build` ✅, `cargo test` ✅ (132 tests pass)

### 1.4 Remove qipu references from run metadata and cache

**Status:** ✅ Complete

**Files:**
- ✅ `src/transcript/types.rs` — Removed `qipu_version` and `qipu_commit` fields from `RunMetadata` struct.
- ✅ `src/run/transcript.rs` — Updated `write_transcript_files()` to remove qipu_version parameter and no longer set the removed fields.
- ✅ `src/run/records.rs` — Removed `qipu_version` parameter from `build_result_record()` and `handle_dry_run()`, removed `qipu_commit` field from ResultRecord construction.
- ✅ `src/results/types/mod.rs` — Removed `qipu_commit` field from `ResultRecord` struct.
- ✅ `src/results/utils.rs` — Removed `get_qipu_version()` function entirely. Updated module doc comment.
- ✅ `src/results.rs` — Removed `get_qipu_version` from public exports.
- ✅ `src/run/mod.rs` — Removed `get_qipu_version()` call and qipu_version variable, updated all call sites to not pass qipu_version.
- ✅ `src/run/cache.rs` — Removed `qipu_version` parameter from `compute_cache_key()` function and its call to `CacheKey::compute()`.
- ✅ `src/results/types/cache_key.rs` — Removed `qipu_version` field from `CacheKey` struct and `compute()` method, updated `as_string()` to not include version.
- ✅ `src/results/types/tests.rs` — Updated all test cases to remove qipu_version parameter and assertions.
- ✅ `src/results/test_helpers.rs` — Removed `qipu_commit` field from test record construction.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all 149 tests pass.

### 1.5 Remove `get_prime_output` from fixture

**Status:** ✅ Complete

**Files:**
- ✅ `src/fixture.rs` — Deleted `get_prime_output()` method and removed unused `Command` import.
- ✅ `src/run/setup.rs` — Removed the call to `env.get_prime_output()` and the `prime_output` variable.
- ✅ `src/run/mod.rs` — Removed `prime_output` from `compute_cache_key()` call.
- ✅ `src/run/cache.rs` — Removed `prime_output` parameter from `compute_cache_key()`.
- ✅ `src/results/types/cache_key.rs` — Removed `prime_output_hash` field and `prime_output` parameter from `CacheKey` struct and methods.
- ✅ `src/results/types/tests.rs` — Updated all tests to remove `prime_output` references and deleted `test_cache_key_different_prime_outputs` test.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all 148 tests pass.

### 1.6 Remove qipu-specific transcript analyzer regex

**Status:** ✅ Complete

**Files:**
- ✅ `src/transcript/analyzer.rs` — Replaced hardcoded `qipu` regex with a generic default pattern and added pattern-aware methods (`analyze_with_pattern`, `extract_commands_with_pattern`). Added invalid-pattern safety and filtering for non-command status lines.
- ✅ `src/transcript/analyzer.rs` — Documented several hypothetical command examples in analyzer docs: `taskmgr create`, `notes-cli list`, and `acme-tool deploy`.
- ✅ `src/transcript/tests/analyzer.rs` — Replaced qipu-specific transcript examples with generic command examples and added a dedicated test that validates multiple hypothetical command styles.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all 149 tests pass.

---

## Phase 2: Target Tool Configuration

Add the `target` config to scenarios so the framework knows what tool is being tested.

### 2.1 Add target config to scenario types

**Status:** ✅ Complete

**Files:**
- ✅ `src/scenario/types.rs` — Added `TargetConfig` with `binary`, optional `command_pattern`, optional `health_check`, and optional `env`. Added `target: TargetConfig` to `Scenario`.
- ✅ Updated YAML fixtures in tests to include `target:` section:
  - `src/scenario/tests/basic.rs`
  - `src/scenario/tests/gates.rs`
  - `src/scenario/tests/setup.rs`
  - `src/scenario/tests/run_config.rs`
  - `src/run/tests.rs`
  - `src/adapter/mock_test.rs`
  - `tests/cli.rs`

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all tests pass.

### 2.2 Wire target config through transcript analyzer

**Status:** ✅ Complete

**Files:**
- ✅ `src/transcript/analyzer.rs` — Added target-aware analyzer entry points and `resolve_command_pattern()` so command extraction uses `target.command_pattern` when provided and defaults to a regex built from `target.binary`.
- ✅ `src/eval_helpers.rs` — Updated `compute_efficiency_metrics()` and `no_transcript_errors()` to accept `target_binary` and optional `command_pattern`, then pass both to the analyzer.
- ✅ `src/evaluation.rs` — Threaded `scenario.target.binary` and `scenario.target.command_pattern` through gate evaluation and efficiency computation.
- ✅ `src/transcript/tests/analyzer.rs` — Added tests for default target-binary matching and custom no-capture patterns.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all tests pass (151 total).

### 2.3 Wire target env vars through execution

**Status:** ✅ Complete

**Files:**
- ✅ `src/adapter/claude_code.rs` — Updated adapter execution to pass `scenario.target.env` into the child process via `run_command_with_env()`.
- ✅ `src/adapter/opencode.rs` — Merged `scenario.target.env` into adapter child-process env vars (alongside `XDG_CONFIG_HOME`).
- ✅ `src/run/setup.rs` — Updated setup command execution to pass `scenario.target.env` into setup shell commands. Added unit test coverage for env var propagation.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all tests pass.

### 2.4 Update test fixtures

**Status:** ✅ Complete

**Files:**
- ✅ `tests/cli.rs` — Verified all inline scenario YAML fixtures include a `target:` section.
- ✅ `src/run/tests.rs` — Verified test scenario YAML fixtures include a `target:` section.
- ✅ `fixtures/` and `llm-test-fixtures/` — No tracked YAML fixture files currently present that require updates.

**Verify:** ✅ `cargo test` — all tests pass (152 total).

---

## Phase 3: New Generic Gate System

Replace domain-specific gates with generic primitives per specs/evaluation.md.

### 3.1 Define new gate enum

**Status:** ✅ Complete

**Files:**
- ✅ `src/scenario/types.rs` — Replaced the `Gate` enum with the generic variants:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(tag = "type", rename_all = "snake_case")]
  pub enum Gate {
      CommandSucceeds { command: String },
      CommandOutputContains { command: String, substring: String },
      CommandOutputMatches { command: String, pattern: String },
      CommandJsonPath { command: String, path: String, assertion: String },
      FileExists { path: String },
      FileContains { path: String, substring: String },
      FileMatches { path: String, pattern: String },
      NoTranscriptErrors,
      Script { command: String, description: String },
  }
  ```
- ✅ `src/scenario/tests/gates.rs` — Rewrote gate-deserialization tests to cover each new gate variant.
- ✅ Updated inline YAML fixtures to use generic gates so parsing and CLI tests stay aligned:
  - `src/scenario/tests/basic.rs`
  - `src/scenario/tests/setup.rs`
  - `src/scenario/tests/run_config.rs`
  - `src/run/tests.rs`
  - `tests/cli.rs`
- ✅ `src/evaluation.rs` — Updated gate dispatch to compile with the new enum; added initial implementations for `command_output_contains`, `command_output_matches`, `file_exists`, `file_contains`, and `file_matches`, with `command_json_path` and `script` marked not implemented pending Phase 3.2/4.4 details.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all tests pass (152 total).

### 3.2 Implement generic gate evaluators

**Status:** ✅ Complete

**Files:**
- ✅ `src/evaluation.rs` — Completed generic implementations for `CommandSucceeds`, `CommandOutputContains`, `CommandOutputMatches`, `CommandJsonPath`, `FileExists`, `FileContains`, `FileMatches`, and `NoTranscriptErrors` (kept existing transcript-based behavior). `Script` remains deferred to Phase 4.
- ✅ `src/evaluation.rs` — Added JSON path resolution and assertion evaluation for `command_json_path` supporting `exists`, `equals <value>`, `contains <substring>`, `len >= N`, `len == N`, and `len > N`.
- ✅ `src/evaluation.rs` — Added unit tests covering command and file gate evaluators, including JSON path assertion behavior.

**Verify:** ✅ `cargo build` — no compile errors. ✅ `cargo test` — all tests pass (163 total).

### 3.3 Implement `command_json_path` assertion parser

**Status:** ✅ Complete

**Files:**
- ✅ `src/evaluation.rs` — Implemented JSON path parsing and assertion evaluation inline:
  - `exists` — value is not null/missing
  - `equals <value>` — exact equality
  - `contains <substring>` — string contains
  - `len >= N`, `len == N`, `len > N` — array/object length

  Uses `serde_json::Value` for JSON navigation with JSON pointer-style paths (e.g., `$.links` → split on `.`, navigate nested objects/arrays).
  
  Functions implemented:
  - `parse_json_path()` — Parses JSON path expressions like `$.items[0].name`
  - `resolve_json_path()` — Resolves a parsed path against a JSON value
  - `evaluate_json_assertion()` — Evaluates assertions against resolved values

**Verify:** ✅ Unit tests cover all assertion forms in `src/evaluation.rs` test module. All 163 tests pass.

### 3.4 Update test fixtures and CLI tests

**Status:** ✅ Complete

**Files:**
- ✅ `tests/cli.rs` — All scenario YAML fixtures already updated to use generic gate types and `target:` configuration during Phase 3.1-3.2
- ✅ `src/scenario/tests/` — All scenario parsing tests updated with generic gates and target config
- ✅ Added integration test `test_run_command_with_post_scripts` demonstrating post script execution

**Verify:** ✅ `cargo test --all` — All 172 tests pass (154 unit + 18 integration).

---

## Phase 4: Scripts System

Implement the three new script hooks per specs/scripts.md.

### 4.1 Add scripts config to scenario types

**Status:** ✅ Complete

**Files:**
- ✅ `src/scenario/types.rs` — Added:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ScriptsConfig {
      #[serde(default)]
      pub post: Vec<ScriptEntry>,
      #[serde(default)]
      pub evaluators: Vec<EvaluatorEntry>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ScriptEntry {
      pub command: String,
      #[serde(default = "default_script_timeout")]
      pub timeout_secs: u64,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct EvaluatorEntry {
      pub command: String,
      pub name: String,
      #[serde(default = "default_evaluator_timeout")]
      pub timeout_secs: u64,
  }

  fn default_script_timeout() -> u64 { 30 }
  fn default_evaluator_timeout() -> u64 { 60 }
  ```
  Add `#[serde(default)] pub scripts: Option<ScriptsConfig>` to `Scenario`.

**Verify:** ✅ `cargo build` — no compile errors. All 149 tests pass. Existing scenarios without `scripts:` still parse correctly.

### 4.2 Implement script runner utility

**Status:** ✅ Complete

**Files:**
- ✅ New file: `src/script_runner.rs` — A utility for running scripts in the fixture directory with the correct environment variables:
  ```rust
  pub struct ScriptRunner {
      fixture_dir: PathBuf,
      results_dir: PathBuf,
      scenario_name: String,
      agent: String,
      model: String,
      transcript_path: Option<PathBuf>,
      events_path: Option<PathBuf>,
      target_env: HashMap<String, String>,
  }

  pub struct ScriptResult {
      pub exit_code: i32,
      pub stdout: String,
      pub stderr: String,
      pub timed_out: bool,
  }
  ```

  The runner:
  - Sets `LLM_TOOL_TEST_FIXTURE_DIR`, `LLM_TOOL_TEST_RESULTS_DIR`, `LLM_TOOL_TEST_SCENARIO`, `LLM_TOOL_TEST_AGENT`, `LLM_TOOL_TEST_MODEL`, `LLM_TOOL_TEST_TRANSCRIPT`, `LLM_TOOL_TEST_EVENTS` env vars.
  - Merges `target.env` vars.
  - Runs the command via `sh -c` in the fixture directory.
  - Enforces timeout (use `wait-timeout` crate, already a dependency).
  - Returns `ScriptResult`.

**Verify:** ✅ Unit tests for echo, exit codes (success/failure), timeout behavior, stderr capture, fixture directory execution, and environment variable propagation. All 8 tests pass. Total test count: 171 (154 unit + 17 integration).

### 4.3 Implement post-execution scripts

**Status:** ✅ Complete

**Files:**
- ✅ `src/run/execution.rs` — Added `run_post_scripts()` helper function that:
  - Runs after agent execution and transcript writing, but before evaluation
  - Creates a `ScriptRunner` with the correct environment variables
  - Iterates through `scenario.scripts.post` entries
  - Logs post script results to events.jsonl via `writer.append_event()`
  - Prints warnings if post scripts fail (exit code != 0)
  
  The post scripts have access to:
  - `LLM_TOOL_TEST_FIXTURE_DIR` — The fixture directory
  - `LLM_TOOL_TEST_RESULTS_DIR` — The results directory
  - `LLM_TOOL_TEST_SCENARIO` — The scenario name
  - `LLM_TOOL_TEST_AGENT` — The tool/agent name
  - `LLM_TOOL_TEST_MODEL` — The model name
  - `LLM_TOOL_TEST_TRANSCRIPT` — Path to the transcript file
  - `LLM_TOOL_TEST_EVENTS` — Path to the events file
  - All `target.env` variables

- ✅ `src/run/mod.rs` — Updated `run_single_scenario()` to pass `results_dir` to `run_evaluation_flow()` and `setup_scenario_env()`
- ✅ `src/run/setup.rs` — Fixed `setup_scenario_env()` to properly pass scenario file path
- ✅ `tests/cli.rs` — Added `test_run_command_with_post_scripts` integration test

**Verify:** ✅ `cargo build` compiles successfully. Integration test `test_run_command_with_post_scripts` passes. All 172 tests pass.

### 4.4 Implement script gate evaluation

**Status:** ✅ Complete

**Files:**
- ✅ `src/evaluation.rs` — Added the `Script` arm to the gate evaluator match:
  - Created `EvaluationContext` struct to pass both `env_root` and optional `ScriptRunner`
  - Updated `GateEvaluator` trait to accept `&EvaluationContext` instead of individual parameters
  - Implemented `eval_script()` function that:
    - Uses the `ScriptRunner` to execute the script with a 30-second timeout
    - Attempts to parse stdout as JSON with `{passed, message}` structure
    - Falls back to exit code-based pass/fail if JSON parsing fails
    - Returns appropriate error messages for timeout cases
  - Updated `evaluate_gates()` and `evaluate()` functions to use the new context
  - Added 5 comprehensive unit tests:
    - `script_gate_with_exit_code_success` - verifies exit code 0 passes
    - `script_gate_with_exit_code_failure` - verifies non-zero exit code fails
    - `script_gate_with_json_output` - verifies JSON `{passed: true}` parsing
    - `script_gate_with_json_output_failure` - verifies JSON `{passed: false}` parsing
    - `script_gate_without_runner_fails` - verifies graceful error without runner

- ✅ `src/run/execution.rs` — Updated `run_evaluation_flow()` to:
  - Create a `ScriptRunner` with the correct environment variables before evaluation
  - Pass the `ScriptRunner` to the `evaluate()` function
  - Ensure transcript and events paths are available to script gates

**Verify:** ✅ All unit tests pass (5 new script gate tests + 154 existing). All 18 integration tests pass.

### 4.5 Implement custom evaluators

**Files:**
- `src/evaluation.rs` (or a new `src/custom_evaluators.rs`) — After gates run, execute evaluator scripts:
  ```rust
  pub struct EvaluatorResult {
      pub name: String,
      pub metrics: Option<serde_json::Value>,
      pub score: Option<f64>,
      pub summary: Option<String>,
      pub error: Option<String>,
  }
  ```
  
  Run each evaluator, parse JSON stdout, store results.

- `src/run/transcript.rs` — Include evaluator results in `metrics.json` output.
- `src/transcript/writer.rs` — Include evaluator summaries in `evaluation.md`.

**Verify:** Integration test with a scenario that has a custom evaluator.

### 4.6 Update `EvaluationMetrics` to carry evaluator results

**Files:**
- `src/evaluation.rs` — Add `pub evaluator_results: Vec<EvaluatorResult>` to `EvaluationMetrics`.
- `src/run/records.rs` — Store evaluator results in the result record.
- `src/results/types/mod.rs` — Add evaluator results to `EvaluationMetricsRecord`.

**Verify:** `cargo build`, `cargo test`.

---

## Phase 5: Documentation Updates

### 5.1 Update README.md gate types

The README currently lists domain-specific gates under "Domain-specific (evolving)". After Phase 3, remove that section entirely and list only the implemented generic gates. Update any example commands or scenario references.

**Files:**
- `README.md` — Replace the gate types section with the final implemented set. Remove the "evolving" caveat.

### 5.2 Update llm-tool-test-config.example.toml

Add `[target]` section to the example config showing how to configure the target tool globally.

**Files:**
- `llm-tool-test-config.example.toml` — Add:
  ```toml
  # Target tool configuration (can also be defined per-scenario in YAML)
  [target]
  binary = "mytool"
  command_pattern = "mytool\\s+(\\S+)"
  health_check = "mytool --version"

  [target.env]
  MYTOOL_CONFIG = "/path/to/config.toml"
  ```

### 5.3 Create AGENTS.md for the project itself

The project has no AGENTS.md. Create one with:
- Build/test/lint commands (`cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check`)
- Project structure overview (adapter/, scenario/, transcript/, run/, evaluation)
- Key conventions (how gates work, how scripts integrate, how scenarios are structured)
- Pointer to specs/ for detailed design

**Files:**
- `AGENTS.md` — New file.

### 5.4 Retire SPLIT_PLAN.md

This document is about the mechanical split from qipu and is no longer relevant. The project's identity is now defined by the README and specs.

**Files:**
- `SPLIT_PLAN.md` — Delete.

### 5.5 Update TODO.md

The TODO still contains pre-split tasks, qipu repo tasks (now removed), and items that are addressed by the implementation plan. Rewrite to reflect actual remaining work after implementation.

**Files:**
- `TODO.md` — Rewrite. Remove completed items, remove qipu-related items, add any new items discovered during implementation.

### 5.6 Create an example scenario with scripts

The specs use a hypothetical `taskmgr` tool. Create a concrete, runnable example scenario that demonstrates the scripts feature using a simple real tool (e.g., `git` or a shell script that acts as a mock CLI). This serves as both documentation and a smoke test.

**Files:**
- `fixtures/example_basic/` — New directory with AGENTS.md, README.md, and scripts/.
- `fixtures/example_basic.yaml` — Scenario YAML using target config, generic gates, and scripts.

### 5.7 Update specs if implementation deviates

Review each spec against the actual implementation and update any details that changed during development (field names, default values, behavior on edge cases).

**Files:**
- `specs/evaluation.md` — Verify gate types match implementation.
- `specs/scenarios.md` — Verify schema matches implementation.
- `specs/scripts.md` — Verify contracts match implementation.
- `specs/llm-user-validation.md` — Verify architecture description matches implementation.

**Verify:** Read each spec and compare against the code. No stale references, no contradictions.

---

## Phase 6: Cleanup & Consistency

### 6.1 Remove dead code

**Files:**
- Run `cargo clippy` and address all warnings.
- Delete any remaining qipu references in source code comments.
- Remove `src/eval_tests_doctor.rs` and `src/eval_tests_gates.rs` if not already done.
- Remove `src/eval_tests_score.rs` if it depends on removed quality metrics.

### 6.2 Update evaluation report format

**Files:**
- `src/transcript/writer.rs` — Update `write_evaluation()` to match the spec:
  - Remove notes/links counts (already done in Phase 1).
  - Make composite score conditional (only if scenario configures weights).
  - Add evaluator summaries section.
  - Update links section (remove store snapshot link).

### 6.3 Update `print_result_summary`

**Files:**
- `src/output.rs` — Remove notes/links lines. Make composite score conditional.

### 6.4 Final test pass

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Test Strategy

Each phase should be verified with `cargo build && cargo test` before moving to the next.

**New tests to write:**

| Phase | Test | Location |
|-------|------|----------|
| 3.2 | Unit tests for each generic gate type | `src/evaluation.rs` (mod tests) |
| 3.3 | Unit tests for JSON assertion parser | `src/json_assertion.rs` (mod tests) |
| 4.2 | Unit tests for ScriptRunner (env vars, timeout, exit codes) | `src/script_runner.rs` (mod tests) |
| 4.4 | Script gate with JSON output | `src/evaluation.rs` (mod tests) |
| 4.4 | Script gate with exit-code-only | `src/evaluation.rs` (mod tests) |
| 4.5 | Custom evaluator happy path | integration test |
| 4.5 | Custom evaluator timeout/failure | integration test |
| E2E | Scenario with post scripts + script gates + evaluator | `tests/cli.rs` |

**Existing tests that will break and need updating:**
- `src/transcript/tests/analyzer.rs` — hardcoded `qipu` command strings
- `src/transcript/tests/logging_tests.rs` — uses `"qipu"` as command name
- `src/eval_tests_doctor.rs` — delete entirely
- `src/eval_tests_gates.rs` — delete or rewrite for generic gates
- `src/eval_tests_score.rs` — update for new composite scoring
- `tests/cli.rs` — update scenario YAML to include `target:` and new gate types
- `src/run/tests.rs` — update scenario YAML
- `src/store_analysis.rs` tests — deleted with the file

---

## Dependency Check

No new crate dependencies needed. Existing dependencies cover all requirements:
- `serde` / `serde_json` / `serde_yaml` — config parsing, JSON assertion
- `regex` — command pattern matching, `file_matches` gate
- `wait-timeout` — script timeout enforcement
- `shlex` — command parsing (if needed for script execution)
- `chrono` — timestamps
- `tempfile` (dev) — test fixtures

---

## Estimated Scope

| Phase | Description | Size |
|-------|-------------|------|
| 1 | Remove snapshot & qipu code | Medium — many files touched, mostly deletion |
| 2 | Target tool configuration | Small — add struct, wire through |
| 3 | Generic gate system | Medium — new evaluation logic + tests |
| 4 | Scripts system | Medium — new module, 3 hook types, integration |
| 5 | Documentation updates | Medium — README, config example, AGENTS.md, example scenario, spec reconciliation |
| 6 | Cleanup | Small — dead code, formatting, final tests |

Phases 1–2 can be done together as a single pass. Phase 3 and Phase 4 are independent of each other and could be parallelized if desired, though Phase 4's script gate depends on Phase 3's gate enum. Phase 5 (docs) should be done after Phases 3–4 so the documentation reflects the final implementation.
