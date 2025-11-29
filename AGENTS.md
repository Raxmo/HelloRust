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

## Asking Before Starting

Ask about:
- Major architectural changes
- Approach to a complex problem
- Whether to refactor existing code
- How to handle edge cases
- Design of new modules or subsystems

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
