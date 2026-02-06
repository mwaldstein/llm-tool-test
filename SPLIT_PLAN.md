# LLM Tool Test - Standalone Project

A testing framework for evaluating LLM coding agents against structured test scenarios.

## Overview

This tool is a standalone project for testing CLI tools with LLM agents.

## Split Plan

### Phase 1: Initial Setup (Current)
1. ✅ Create standalone project structure
2. ✅ Set up crate files
3. ✅ Configure as independent Cargo project

### Phase 2: Dependencies Review
- Verify all dependencies in Cargo.toml work standalone
- No external dependencies on other projects (verified: clean separation)
- Standard Rust ecosystem dependencies only

### Phase 3: Configuration & Documentation
- Move config examples to project root
- Update README for standalone context
- Add CHANGELOG documenting split
- **Specs**: Adapt `specs/llm-user-validation.md`
  - This spec defines the testing harness architecture
  - Generalize references to "target CLI tool"

### Phase 4: Repository Independence
- Initialize as separate git repository
- Preserve git history (optional)
- Update package metadata (author, repository, keywords)

## File Structure

```
llm-tool-test/
├── Cargo.toml                    # Standalone project config
├── README.md                     # Standalone documentation
├── CHANGELOG.md
├── llm-tool-test-config.example.toml
├── specs/                        # Specifications
│   └── llm-user-validation.md    # Testing harness architecture spec
├── src/
│   ├── main.rs
│   ├── lib.rs (if applicable)
│   ├── cli.rs
│   ├── commands.rs
│   ├── config.rs
│   └── ... (all source files)
└── tests/
    ├── cli.rs
    └── support.rs
```

## Notes

- **No external dependencies**: The tool is completely self-contained
- **General purpose**: Tool is designed to test any CLI tool
