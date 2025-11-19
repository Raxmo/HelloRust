# HelloRust Project Workflow

## Overview
This project uses a branch-based task tracking system where `todo.md` maintains a list of pending items only.

## todo.md Format
- Timestamp at the top: `[YYYY.MM.DD-HH:MM]`
- Sub-items for the current active item listed below
- Delimiter: `[PENDING]`
- Pending items listed below the delimiter
- Completed items are **deleted** after each git push

## Workflow

### Starting Work on an Item
1. Create a branch named after the item: `git checkout -b item-name`
2. Move any sub-items for that item from `[PENDING]` to above the `[PENDING]` delimiter in todo.md
3. Delete the item from `[PENDING]` (the item is now inferred by the branch name)
4. Commit and push todo.md changes to the feature branch
5. Continue work on the feature branch, committing as needed

### Completing an Item
1. Make final commit and push to the feature branch
2. Merge the branch back to main: `git checkout main && git merge item-name`
3. Delete the item from `todo.md`
4. Push the updated todo.md to main
5. Delete the feature branch: `git branch -d item-name`

### Sub-items
If an item has sub-tasks:
- Create sub-branches off the main feature branch: `git checkout -b item-name/sub-item-name`
- Track sub-items in a section at the bottom of `todo.md` under the current active item
- Merge sub-branches back to the parent feature branch when complete
- Delete sub-items from the todo after sub-branch merge

## Branch Naming
- Use lowercase with hyphens: `feature-name`, `item-name`
- For sub-items: `parent-item/sub-item-name`
