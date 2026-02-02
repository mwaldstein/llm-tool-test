# LLM Tool Test - TODO List

> **Note**: This project was split from [qipu](https://github.com/mwaldstein/qipu). Qipu uses a beads-based task tracking system (`bd`), but llm-tool-test has not yet been set up with beads. Tasks from the qipu specs have been manually migrated to this TODO file.

## Pre-Split Tasks

- [ ] Update package metadata in `Cargo.toml`
  - [ ] Add proper description and author
  - [ ] Update repository URL (when created)
  - [ ] Add appropriate keywords for crates.io
  - [ ] Verify license field is correct

- [ ] Review dependencies in `Cargo.toml`
  - [ ] Verify all dependencies are still needed
  - [ ] Check for any unused dependencies
  - [ ] Update to latest versions if appropriate

- [ ] Generalize qipu-specific code
  - [ ] Review `src/eval_helpers.rs` - contains qipu-specific functions
    - `get_qipu_path()` - hardcoded qipu binary paths
    - `run_qipu_json()` - qipu-specific command runner
  - [ ] Consider making target CLI tool configurable
  - [ ] Add documentation for adapting to other CLI tools

- [ ] Review and update configuration
  - [ ] Update `llm-tool-test-config.example.toml` if needed
  - [ ] Document all configuration options
  - [ ] Consider if any qipu-specific config should be generalized

- [ ] Review test scenarios and fixtures
  - [ ] Check if scenarios are qipu-specific
  - [ ] Document how to create scenarios for other tools
  - [ ] Consider including example generic scenarios

## Post-Split Tasks (Qipu Repo)

### Workspace & Build Changes
- [ ] Update `Cargo.toml` - remove `"crates/llm-tool-test"` from workspace members
- [ ] Delete `crates/llm-tool-test/` directory (or archive to separate branch)
- [ ] Verify `cargo build` and `cargo test` still work in qipu
- [ ] Check for any build scripts or dev-dependencies referencing llm-tool-test

### Documentation Updates in Qipu

**Files requiring updates:**
- [ ] `README.md` - Update any sections mentioning llm-tool-test
  - Remove references to building with `-p llm-tool-test`
  - Add pointer to standalone llm-tool-test project
- [ ] `AGENTS.md` - Update development guidance for agents
- [ ] `docs/llm-testing.md` - Major rewrite needed
  - Current doc assumes llm-tool-test is part of qipu workspace
  - Update all `cargo build -p llm-tool-test` references
  - Update paths like `crates/llm-tool-test/`
  - Change fixture location references
- [ ] `specs/llm-user-validation.md` - Update or archive
  - **Option A**: Keep in qipu as historical reference, update intro to point to standalone project
  - **Option B**: Move to standalone llm-tool-test repo, replace qipu copy with a pointer
  - Changes needed:
    - Change "separate crate within the workspace" to "standalone project"
    - Update any build/usage instructions
    - Add note about where to find the actual implementation
- [ ] `specs/README.md` - Update any status notes about llm-user-validation
  - Mark as "moved to standalone llm-tool-test project"

### CI/CD Updates
- [ ] Check `.github/workflows/` for any llm-tool-test references
- [ ] Remove llm-tool-test from any test matrices
- [ ] Update any integration tests that depend on llm-tool-test

### Configuration & Fixtures
- [ ] Decide fate of `llm-test-fixtures/` directory in qipu
  - Option A: Keep in qipu as qipu-specific scenarios
  - Option B: Move to standalone llm-tool-test repo as examples
  - Option C: Split (generic examples to llm-tool-test, qipu-specific stay)
- [ ] Update or remove `llm-tool-test-config.toml` from qipu root

### Beads/Issues Cleanup

**Open beads to close/archive in qipu:**
- [ ] `qipu-14h` [P3] [task] - Design: rich event log format for llm-tool-test
- [ ] `qipu-rtj` [P3] [task] - Design: transcript redaction for secrets (llm-tool-test)
- [ ] `qipu-m3o` [P3] [task] - Design: llm-tool-test results.db SQLite (optional)

**Action for each:**
- Add comment explaining llm-tool-test is now a standalone project
- Close with status "moved to standalone llm-tool-test project"
- Do NOT transfer to new repo (llm-tool-test not using beads yet)

### Communication
- [ ] Add migration note to qipu CHANGELOG
- [ ] Create transition guide if users were depending on in-tree llm-tool-test

## Repository Setup

- [ ] Initialize git repository in llm-tool-test folder
- [ ] Create `.gitignore` file
- [ ] Preserve git history from qipu (optional)
  - Consider using `git filter-branch` or `git subtree split`
- [ ] Create GitHub repository (if applicable)
- [ ] Push initial commit

## Documentation Improvements

- [ ] Write CHANGELOG.md documenting the split
- [ ] Add CONTRIBUTING.md guide
- [ ] Add example scenarios for different use cases
- [ ] Document scenario format/specification
- [ ] Add troubleshooting guide for common issues
- [ ] Document how to add support for new LLM tools

### Specs (Migrated from Qipu)
- [ ] Copy and adapt `specs/llm-user-validation.md` from qipu
  - [ ] Generalize from qipu-specific to generic CLI tool testing
  - [ ] Update architecture section (already says "separate binary")
  - [ ] Update all qipu command examples to generic examples
  - [ ] Update fixture examples to be tool-agnostic
  - [ ] Keep as implementable specification (not just documentation)
- [ ] Do NOT copy `specs/llm-context.md` (stays in qipu - it's about qipu's LLM features)
- [ ] Consider creating new specs directory structure for llm-tool-test
  - `specs/architecture.md` - Overall design
  - `specs/adapters.md` - Tool adapter interface
  - `specs/scenarios.md` - Scenario format specification
  - `specs/evaluation.md` - Gates and judging criteria

## Testing & Validation

- [ ] Verify standalone build works
  - [ ] `cargo build`
  - [ ] `cargo test`
  - [ ] `cargo clippy`
  - [ ] `cargo fmt --check`

- [ ] Test with existing qipu scenarios (if keeping for backward compatibility)
- [ ] Create test scenario for a non-qipu CLI tool

### Missing Test Coverage (Migrated from Qipu Specs)

From `specs/README.md` - llm-user-validation.md test coverage gaps:
- [ ] Transcript report generation tests
- [ ] Event logging tests
- [ ] Human review workflow tests
- [ ] CLI commands integration tests
- [ ] LLM judge evaluation tests
- [ ] Link parsing tests

## Future Improvements

### Migrated from Qipu Beads (Open Tasks)

These were tracked in qipu's beads system and should be preserved during the split:

- [ ] **Rich event log format** (was `qipu-14h`)
  - Design structured event logging for test runs
  - Consider JSON Lines format for events
  - Events: spawn, output, tool_call, tool_result, complete, etc.

- [ ] **Transcript redaction for secrets** (was `qipu-rtj`, blocked `qipu-7uul`)
  - Redact API keys and tokens before writing reports
  - Redact passwords, email addresses, file paths with usernames
  - Mark raw transcript as sensitive

- [ ] **Results database (SQLite)** (was `qipu-m3o`)
  - Optional SQLite backend for results tracking
  - Enable trend analysis and querying across runs
  - Current implementation uses JSONL

### General Improvements

- [ ] Add support for additional LLM tools
  - [ ] Add more adapters in `src/adapter/`
  - [ ] Document adapter interface

- [ ] Improve scenario management
  - [ ] Add scenario validation command
  - [ ] Add scenario template generator
  - [ ] Support scenario dependencies/composition

- [ ] Enhance reporting
  - [ ] HTML report generation option
  - [ ] JUnit XML output for CI integration
  - [ ] Trend analysis across multiple runs

- [ ] Performance optimizations
  - [ ] Parallel scenario execution
  - [ ] Better caching strategies
  - [ ] Reduce transcript processing time

- [ ] Developer experience
  - [ ] Add interactive scenario debugger
  - [ ] Improve error messages and diagnostics
  - [ ] Add progress indicators for long runs
  - [ ] Better logging and verbosity controls

- [ ] Integration features
  - [ ] CI/CD integration examples (GitHub Actions, etc.)
  - [ ] Webhook support for notifications
  - [ ] Export to popular test management tools

## Known Issues to Address (Migrated from Qipu Specs)

### P1: Correctness Bugs

- [ ] Budget enforcement not implemented
  - CLI accepts `--timeout-secs` but no `--max-usd` budget limit
  - Budget warning exists but doesn't enforce limits (was in qipu spec at `crates/llm-tool-test/src/run.rs:417-424`)
  - Need to add per-run and session budget controls

- [ ] Cost estimation incomplete
  - OpenCode adapter parses token usage but returns `None` for cost
  - Need cost calculation from token usage using config pricing
  - Other adapters (claude-code, mock) need cost implementation

### General Issues

- [ ] Some hardcoded paths assume qipu project structure
- [ ] Doctor gate check (`doctor_passes`) is qipu-specific
- [ ] Need to verify all gate types work with generic CLI tools

## Questions to Resolve

- [ ] Should we keep backward compatibility with qipu as default behavior?
- [ ] What CLI tool should be used in example scenarios?
- [ ] Should we publish to crates.io?
- [ ] What is the target user (CLI tool developers vs end users)?
