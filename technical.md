# Script Language Specification

## Overview
Tag-based script format for interactive text-based visual novel engine. All syntax uses `[tag: arguments]` notation with `||` blocks for grouping tags and options.

## Tag Syntax

### Core Tags
- `[section: label]` - Define a section (display label, then narrative text)
- `[chapter: number: label]` - Define a chapter (special section)
- `[goto: tag]` - Jump to target section
- `[option: label: tag ... tag]` - Display option; execute tags on selection
- `[if: expression: tag ... tag]` - Conditionally execute tags if expression is true
- `[set: target: value]` - Set target to value

### Definition Tags
- `[define: tag ... tag]` - Define variables/characters (can define multiple at once)
- `[character: name]` - Reference or define character
- `[attribute: name]` - Reference or define attribute (variable)
- `[cmt: text]` - Comment (ignored)

### Literal Tags
- `[value: integer]` - Integer literal
- `[label: string]` - String literal
- `[bool: true||false]` - Boolean literal

### Operators
Arithmetic: `+ - * / ++ --`
Comparison: `> < >= <= == !=`
Logical: `and or xor not nand nor xnor`

### Expressions
```
expression => [tag op tag] || [op tag]
```
Expressions return either `bool` (boolean) or `value` (integer).

## Property Access

Properties are accessed compositionally:
```
[[character: alice]: mood]
[[character: alice]: [attribute: gold]]
```

The inner tag resolves to an object type, and the outer bracket accesses that object's property.

## Blocks

Blocks are delimited by `||` and contain tags and options:
```
|
[option: Go home: [goto: [section: home]]]
[if: [[attribute: wealth] > 100]: [option: Buy item: [goto: [section: shop]]]]
|
```

- Blocks cannot be nested.
- Options are only valid inside blocks.
- All tags execute sequentially before jumps.

## Narrative Text

Text outside blocks is implicitly printed:
```
[section: home]
Alice's home is cozy. She has [value: 5] gold coins.
```

Data tags within text are resolved and displayed:
- `[attribute: name]` - Display attribute value
- `[label: text]` - Display string
- `[value: number]` - Display number
- `[[object]: property]` - Display object property

Malformed tags (e.g., `[goto:]`) in text are invalid.

## Definitions

Characters and attributes are defined in definition blocks:
```
|[define: [character: alice]]
[character: [name: Alice]]
[character: [attribute: mood: happy]]|

|[define: [attribute: wealth]]|
```

- Definitions can occur anywhere but must precede first use.
- Definitions can include initial values via `[set:]` inside the block.

## Execution Model

1. Script begins at first `[section:]` or `[chapter:]`.
2. Section label is displayed.
3. Narrative text following section is printed, with data tags resolved inline.
4. When a `||` block is reached, all tags execute in order.
5. All `[if:]` conditions evaluated, `[set:]` operations occur, other tags resolve.
6. All `[goto:]` jumps execute sequentially after block resolution.
7. Control flow continues from target section.

## Notes

- Whitespace is not syntactical; used only as delimiter.
- Chapters are structurally identical to sections (label + narrative).
- No block nesting.
- Multiple tags in `tag ... tag` execute sequentially.
- `[if:]` without matching condition skips that tag (no `[else:]`).
- Variables must be defined before first use.
- Nested property access is unlimited: `[[[[object]: prop1]: prop2]: prop3]`

## Examples

Setting attributes:
```
[set: [attribute: wealth]: 100]
[set: [[character: alice]: mood]: happy]
```

Incrementing:
```
[set: [attribute: wealth]: [++ [attribute: wealth]]]
```

Binary operations:
```
[set: [attribute: total]: [[attribute: a] + [attribute: b]]]
[if: [[attribute: gold] > 50]: [option: Expensive item: [goto: [section: luxury_shop]]]]
```

Character dialogue:
```
[section: meeting]
Alice says: I have [[[character: alice]: attribute]: gold] gold.
```
