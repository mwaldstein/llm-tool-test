# LLM Tool Test - Standalone Project

A testing framework for evaluating LLM coding agents against structured test scenarios.

## Overview

This tool was originally part of the qipu project but has been split out as a standalone project due to its general-purpose applicability for testing CLI tools with LLM agents.

## Split Plan

### Phase 1: Initial Setup (Current)
1. ✅ Create standalone project structure
2. ✅ Copy llm-tool-test crate files from qipu
3. ✅ Configure as independent Cargo project (not workspace member)

### Phase 2: Dependencies Review
- Verify all dependencies in Cargo.toml work standalone
- No dependencies on qipu-core (verified: clean separation)
- Standard Rust ecosystem dependencies only

### Phase 3: Configuration & Documentation
- Move config examples to project root
- Update README for standalone context
- Add CHANGELOG documenting split
- **Specs**: Copy/adapt `specs/llm-user-validation.md` from qipu
  - This spec defines the testing harness architecture
  - Generalize references from "qipu" to "target CLI tool"
  - Keep `llm-context.md` in qipu (it's about qipu's LLM features, not the test tool)

### Phase 4: Repository Independence
- Initialize as separate git repository
- Preserve git history from qipu (optional, via filter-branch or subtree split)
- Update package metadata (author, repository, keywords)

### Phase 5: Cleanup from Qipu
**Workspace Changes:**
- Remove `crates/llm-tool-test` from qipu workspace Cargo.toml members
- Delete `crates/llm-tool-test/` directory (or archive to branch)
- Verify qipu builds and tests pass without llm-tool-test

**Documentation Updates:**
- Update `README.md` - remove in-tree build references, add pointer to standalone
- Update `AGENTS.md` - update development guidance
- Update `docs/llm-testing.md` - rewrite for standalone usage (major changes needed)
- Update `specs/llm-user-validation.md` - change "workspace crate" to "standalone project"

**Configuration & Fixtures:**
- Decide fate of `llm-test-fixtures/` (keep in qipu, move, or split)
- Handle `llm-tool-test-config.toml` in qipu root

**CI/CD:**
- Check and update `.github/workflows/` for llm-tool-test references

## File Structure

```
llm-tool-test/
├── Cargo.toml                    # Standalone project config
├── README.md                     # Standalone documentation
├── CHANGELOG.md                  # Document split from qipu
├── llm-tool-test-config.example.toml
├── specs/                        # Implementable specifications (migrated from qipu)
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

- **No qipu-core dependencies**: The tool is completely self-contained
- **Existing config**: `llm-tool-test-config.toml` exists in qipu root for testing qipu itself
- **General purpose**: Tool is designed to test any CLI tool, not just qipu
