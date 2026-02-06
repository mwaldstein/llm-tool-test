# LLM Tool Test - TODO List

> **Note**: Tasks have been manually tracked in this TODO file.

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

- [ ] Generalize tool-specific code
  - [ ] Review `src/eval_helpers.rs` - contains tool-specific functions
    - `get_tool_path()` - hardcoded binary paths
    - `run_tool_json()` - tool-specific command runner
  - [ ] Consider making target CLI tool configurable
  - [ ] Add documentation for adapting to other CLI tools

- [ ] Review and update configuration
  - [ ] Update `llm-tool-test-config.example.toml` if needed
  - [ ] Document all configuration options
  - [ ] Consider if any tool-specific config should be generalized

- [ ] Review test scenarios and fixtures
  - [ ] Check if scenarios are tool-specific
  - [ ] Document how to create scenarios for other tools
  - [ ] Consider including example generic scenarios

## Repository Setup

- [ ] Initialize git repository in llm-tool-test folder
- [ ] Create `.gitignore` file
- [ ] Preserve git history (optional)
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

### Specs
- [x] Copy and adapt `specs/llm-user-validation.md`
- [x] Copy and adapt `specs/distribution.md`
  - [ ] Generalize from tool-specific to generic CLI tool testing
  - [ ] Update architecture section (already says "separate binary")
  - [ ] Update all command examples to generic examples
  - [ ] Update fixture examples to be tool-agnostic
  - [ ] Keep as implementable specification (not just documentation)
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

- [ ] Test with existing scenarios
- [ ] Create test scenario for a different CLI tool

### Missing Test Coverage

From `specs/README.md` - llm-user-validation.md test coverage gaps:
- [ ] Transcript report generation tests
- [ ] Event logging tests
- [ ] Human review workflow tests
- [ ] CLI commands integration tests
- [ ] LLM judge evaluation tests
- [ ] Link parsing tests

## Future Improvements

### Open Tasks

- [ ] **Rich event log format**
  - Design structured event logging for test runs
  - Consider JSON Lines format for events
  - Events: spawn, output, tool_call, tool_result, complete, etc.

- [ ] **Transcript redaction for secrets**
  - Redact API keys and tokens before writing reports
  - Redact passwords, email addresses, file paths with usernames
  - Mark raw transcript as sensitive

- [ ] **Results database (SQLite)**
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

## Known Issues to Address

### P1: Correctness Bugs

- [ ] Budget enforcement not implemented
  - CLI accepts `--timeout-secs` but no `--max-usd` budget limit
  - Budget warning exists but doesn't enforce limits
  - Need to add per-run and session budget controls

- [ ] Cost estimation incomplete
  - OpenCode adapter parses token usage but returns `None` for cost
  - Need cost calculation from token usage using config pricing
  - Other adapters (claude-code, mock) need cost implementation

### General Issues

- [ ] Some hardcoded paths assume specific project structure
- [ ] Doctor gate check (`doctor_passes`) is tool-specific
- [ ] Need to verify all gate types work with generic CLI tools

## Questions to Resolve

- [ ] What CLI tool should be used in example scenarios?
- [ ] Should we publish to crates.io?
- [ ] What is the target user (CLI tool developers vs end users)?
