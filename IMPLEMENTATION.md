# Packard Implementation Guide

## Overview

Packard uses a three-phase compilation model:

1. **Parsing Phase:** Load the entire script and build an Abstract Syntax Tree (AST). Tags have uniform structure, so AST construction is simple.
2. **Validation Phase:** Perform complete static analysis on the AST—syntax validation, symbol resolution with scope awareness, type checking, and error detection.
3. **Execution Phase:** Traverse the validated AST sequentially, evaluating tags as they're encountered (JIT-style).

This approach provides full validation upfront (catching syntax errors, undefined symbols, type mismatches) while keeping execution simple and efficient.

## Parsing Phase: Building the AST

### Tokenization

The tokenizer converts raw script text into tokens:
- Opening bracket `[` — tag start
- Closing bracket `]` — tag end
- Colon `:` — LTag/RTag separator
- Arrow `->` — property accessor
- Identifiers, numbers, strings — primitive values
- Whitespace and comments — stripped

### AST Construction

The parser consumes tokens and builds an AST. Since tags follow a simple `[ltag: rtag]` pattern, AST nodes are straightforward:

- **TagNode:** Represents a tag with an LTag, RTag, and optional content (for define blocks)
- **PrimitiveNode:** Represents literal values (numbers, text, flags)
- **PropertyAccessNode:** Represents `->` chains
- **TagListNode:** Represents sequences of tags within blocks

The AST is a tree representation of the entire script, preserving nesting and structure.

## Validation Phase (Complete Static Analysis)

The validator performs a full tree walk of the AST, building symbol tables and checking types.

### Syntax Validation

Validate that the AST structure is valid:
- All tags follow `[ltag: rtag]` pattern (or are FTags)
- All brackets are matched
- Exceptions (`->`, operators, tag lists) are used correctly

### Symbol Resolution

Build a symbol table with scope awareness:
- **Global scope:** Characters defined at script level
- **Local scopes:** Attributes/containers defined within `define` blocks, scoped to their parent
- For each symbol reference, verify it exists in the appropriate scope
- Catch undefined symbol errors before execution

### Type Checking

For all accesses, verify type consistency:
- When accessing `[character: alice] -> [attribute: hp]`, verify alice exists and is a character, and hp exists as an attribute
- When reading with type wrapper `[number: [attribute: hp]]`, verify hp is numeric
- Type coercions with `[[become: type]: value]` are validated for compatibility

### Scope Analysis

Analyze `define` block scopes:
- Track nesting depth and parent contexts
- Verify implicit property access will resolve correctly within define blocks
- Ensure attribute/container declarations don't conflict within their scope

On validation failure, reject the script with detailed error messages (location, what failed, expected vs. found).

## Execution Phase: AST Traversal (JIT)

After validation, execution traverses the AST sequentially.

### Execution Context

Maintain runtime state:
- **Variable store:** Values of all attributes and items
- **Scope stack:** Current define block contexts (for implicit property access)
- **Execution pointer:** Current position in AST

### Sequential Evaluation

Evaluate nodes in depth-first order as they're traversed:
- Primitive nodes resolve to their values
- Tag nodes invoke their LTag operation with the RTag as argument
- Property access nodes (`->`) resolve the chain left-to-right
- Tag lists execute their children sequentially

### Tag Operations

When executing a tag `[ltag: rtag]`:
1. Resolve the RTag to a value
2. Invoke the LTag operation with that value
3. Return the result (used if the tag is itself an RTag)

### Define Block Execution

When entering a `define` block:
1. Push a new scope onto the scope stack
2. Execute initialization statements within implicit context
3. On `set` within define, store attributes in the target declaration
4. Pop scope when exiting the block

### Property Access Resolution

Property access `[character: alice] -> [attribute: hp]` resolves by:
1. Look up `alice` in the variable store
2. Access the `hp` attribute within alice
3. Return the value

Within `define` blocks, implicit access skips the left part of the chain based on the define target.

## Runtime State Management

### Variable Storage

Store variable values in a map indexed by their full path:
- Global attributes: `global.alice.hp`
- Nested containers: `global.alice.bag.sword`

### Scope Stack

Track define block contexts:
- Each scope records the target declaration (e.g., `[character: alice]`)
- Implicit property access resolves relative to the current scope
- On error, scope information helps with error reporting

### Error Handling at Runtime

If a runtime error occurs (e.g., accessing undefined attribute during execution):
- Report the error with context (line, scope, attribute)
- Halt execution

## Navigation and Branching

### Section and Chapter Handling

Sections and chapters are structural tags that mark navigation points:
- `[section: location]` — a named location in the narrative
- Navigation can jump to sections via `[goto: location]`

### Conditional Evaluation

When executing `[[if: condition]: tag-list]`:
1. Evaluate the condition (a flag or boolean expression)
2. If true, execute the tag-list
3. If false, skip the tag-list

### Player Choice and Branching

Choices present options to the player:
- Each option is a branch in the script
- Player selection determines which branch executes next

## Performance Considerations

### Memory Usage

- AST size is proportional to script size; simple tag structure keeps overhead low
- Variable store is map-based; access is O(1) for simple paths, O(n) for deep nesting

### Optimization Strategies

- Cache frequently accessed variables
- Short-circuit boolean evaluation in conditionals
- Pre-compute immutable property accesses
