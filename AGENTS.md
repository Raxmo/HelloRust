# Agent Instructions for Packard Script Language

## Workflow

### Before Starting Work
1. **Read context** - Check specifications.md and IMPLEMENTATION.md to understand the design
2. **Check current branch** - `git status` and `git branch` to see where you are
3. **Ask before major changes** - If unsure about approach, ask first rather than implementing broadly

### Making Changes

#### Documentation Changes
- Update specifications.md or IMPLEMENTATION.md directly
- Commit with message: `docs: [what changed]`
- Commit and push after each logical section

#### Implementation Changes
1. Create a feature branch: `git checkout -b feature/[short-name]`
2. Work incrementally - commit frequently (every 1-2 features)
3. Each commit should be small, focused, and buildable
4. Push regularly: `git push origin feature/[short-name]`
5. When done, create PR or merge to main

### Commits

Use conventional format:
- `feat: [description]` - New feature
- `fix: [description]` - Bug fix
- `docs: [description]` - Documentation
- `refactor: [description]` - Code reorganization
- `test: [description]` - Tests

Example: `feat: implement variable path resolution for nested attributes`

### Branches

- **main** - Stable, merged code
- **specifications** - Spec work (merge to main when complete)
- **implementation-doc** - IMPLEMENTATION.md details (merge to main when complete)
- **feature/*** - Feature branches (review, test, merge to main)

## Code Style

- Keep modules focused and separate (lexer, parser, validator, executor, types, error)
- Use descriptive function and variable names
- Comment complex logic
- Keep functions under 50 lines when possible
- Test as you go (add unit tests to new code)

## Operation Registry Pattern

**For adding new operations:**

1. **Define the handler** - Create a `handle_[operation](&mut self, rtag: &Value) -> Result<Value, String>` method
2. **Register in HANDLERS** - Add entry to the lazy_static HANDLERS HashMap in evaluator_v2.rs:
   ```rust
   map.insert("[operation_name]", Evaluator::handle_[operation] as Handler);
   ```
3. **Validation is automatic** - The validate() function checks all ltag operations against HANDLERS keys

This centralizes operation dispatch and validation. Any new operation is automatically validated at load time. No need to update match statements or validation logic separately.

**Example adding "assert" operation:**
```rust
// In HANDLERS HashMap:
map.insert("assert", Evaluator::handle_assert as Handler);

// Define the handler:
fn handle_assert(&mut self, rtag: &Value) -> Result<Value, String> {
    // Implementation
    Ok(Value::Flag(true))
}
```

## Testing and Validation

### Before Running Test Scripts
- Review the test script syntax with the user BEFORE attempting to execute it
- Output comprehensive parse trees and evaluation traces to files for review
- Use trace files to debug, not interactive trial-and-error

### Test Artifacts
- Always output parse tree logs to files (e.g., `parse_tree.log`, `eval_trace.log`)
- Include full context in logs (token positions, tag structures, evaluation paths)
- Let user review logs before proceeding with further iterations

## Asking Before Starting

Ask about:
- Major architectural changes
- Approach to a complex problem
- Whether to refactor existing code
- How to handle edge cases
- Design of new modules or subsystems
- Test script syntax or semantics

Don't ask about:
- Fixing obvious bugs
- Small improvements or cleanups
- Adding tests
- Documentation formatting

## Pushing and Merging

After finishing a branch:
1. Ensure tests pass: `cargo test`
2. Ensure code builds: `cargo build`
3. Push the branch: `git push origin [branch-name]`
4. Merge to main: `git merge [branch-name]`
5. Delete the branch: `git branch -d [branch-name]`
6. Continue work in new feature branch or on main

## Current Status

As of last handoff:
- specifications.md: Complete and ready to merge to main
- IMPLEMENTATION.md: Complete design details, ready to merge to main
- Code: Initial pipeline (lexer, parser, validator, executor) working
- Next: Variable storage paths, property access, define scoping

See todo.md for current implementation status.
