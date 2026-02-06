# LLM Tool Test - TODO List

> **Note**: This file tracks remaining work after the implementation plan completion.

## Completed (as of latest update)

- ✅ Remove snapshot mechanism and qipu-specific code
- ✅ Add target tool configuration to scenarios
- ✅ Implement generic gate system
- ✅ Implement scripts system (post scripts, script gates, evaluators)
- ✅ Update README with generic gates
- ✅ Update config example with target section
- ✅ Create AGENTS.md for the project
- ✅ Retire SPLIT_PLAN.md

## Phase 5: Documentation (Remaining)

### 5.6 Create example scenario with scripts
- [ ] Create `fixtures/example_basic/` directory with:
  - [ ] AGENTS.md - documentation for a simple tool
  - [ ] README.md - scenario context
  - [ ] scripts/ - example post scripts, script gates, evaluators
- [ ] Create `fixtures/example_basic.yaml` scenario file
- [ ] Use a real tool (e.g., `git` or mock CLI) for the example

### 5.7 Update specs if implementation deviates
- [ ] Review `specs/evaluation.md` against actual implementation
- [ ] Review `specs/scenarios.md` against actual implementation
- [ ] Review `specs/scripts.md` against actual implementation
- [ ] Update any field names, default values, or edge case behaviors

## Phase 6: Cleanup & Consistency

### 6.1 Remove dead code
- [x] Run `cargo clippy` and address warnings
  - Note: One minor warning about unused `succeeded()` method in script_runner.rs
- [x] Delete qipu references in source code comments
- [x] Remove `src/eval_tests_doctor.rs` (done)
- [x] Remove `src/eval_tests_gates.rs` (done)
- [ ] Review `src/eval_tests_score.rs` - may need updates

### 6.2 Update evaluation report format
- [ ] `src/transcript/writer.rs` - Update `write_evaluation()`:
  - [x] Remove notes/links counts (already done in Phase 1)
  - [ ] Make composite score conditional (only if scenario configures weights)
  - [x] Add evaluator summaries section (done in Phase 4)
  - [x] Update links section - remove store snapshot link (done in Phase 1)

### 6.3 Update `print_result_summary`
- [ ] `src/output.rs` - Remove notes/links lines from summary output
- [ ] Make composite score conditional (only if configured)

### 6.4 Final verification
- [ ] `cargo build` - no errors
- [ ] `cargo test` - all tests pass
- [ ] `cargo clippy -- -D warnings` - no warnings (or justify any remaining)
- [ ] `cargo fmt --check` - formatting clean

## Known Issues (Post-Implementation)

### Minor
- Unused `succeeded()` method in `ScriptResult` (script_runner.rs:27) - either use it or remove it

### Future Enhancements (not part of current plan)
- Budget enforcement (`--max-usd` flag) - CLI accepts it but no enforcement
- Cost estimation completion for adapters
- HTML report generation option
- JUnit XML output for CI
- Parallel scenario execution
- Scenario validation command
- Interactive scenario debugger

## Documentation Ideas

- Add CONTRIBUTING.md guide
- Add troubleshooting guide for common issues
- Add CI/CD integration examples
- Create video walkthrough of scenario creation
