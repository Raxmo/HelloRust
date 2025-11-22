# Script Language Specification

## Overview
Tag-based script format for interactive text-based visual novel engine. All syntax uses recursive `[a: b]` bracket notation where both `a` and `b` can be tags or values. Instruction tags execute sequentially to form tag-lists.

## Foundational Principles

### Binary Structure
All tags are strictly binary: `[designator: argument]`. No exceptions.

### Designator as Control Plane
The designator (left side of `:`) is the control plane of the tag. It dictates:
1. **How** the argument is evaluated
2. **What type** the complete tag resolves to
3. **What happens** during resolution (execution, type checking, transformation, etc.)

Examples:
- `[value: 100]` — designator `value` says "treat argument as integer literal", tag resolves to integer
- `[null: expression]` — designator `null` says "check if expression exists", tag resolves to flag
- `[[set: prop]: value]` — designator `set` says "assign value to prop", argument is evaluated as assignment operand
- `[character: alice]` — designator `character` says "create/reference container", tag resolves to character type

### Recursive Nesting and Logic Mutation
Because tags are binary and recursively nestable, the same nested structure can be interpreted completely differently depending on what designator wraps it. Logic and functionality are frequently mutated during resolution.

Examples:
- `[value: [[[character: alice]: [attribute: gold]]]]` — outer `value` designator treats inner resolution as type assertion (must be integer)
- `[null: [[[character: alice]: [attribute: sword]]]]` — outer `null` designator wraps inner resolution in existence check (returns flag)
- `[[if: condition]: [[set: prop]: value]]` — outer `if` designator branches execution based on condition; inner `set` becomes conditional
- `[[set: [[[character: alice]: [attribute: mood]]: [label: happy]]]` — inner property access is transformed by outer `set` into an assignment target

This recursive composition without fixed syntax makes the language highly expressive: the same nested tags behave differently depending on context.

## Tag Categories

Tags are organized by their designator semantics. Each designator name defines what kind of operation or value is being created.

### Value Designators
Primitive values that resolve to concrete data:
- `[value: integer]` - Integer literal
- `[label: string]` - String literal
- `[flag: on|off]` - Flag literal
- `[item: label]` - Item/empty attribute
- `[null: expression]` - Existence check (returns flag)
- `[character: name]` - Character reference or container
- `[attribute: name]` - Attribute reference or container
- `[input: prompt]` - Blocking text input (argument is prompt text, resolves to captured string)

### Action Designators
Tags that execute operations affecting control flow:
- `[[if: resolved-tag]: tag-list]` - Conditional execution
- `[[option: label]: tag-list]` - Player choice
- `[goto: resolved-tag]` - Jump to section

### Transformative Designators
Tags that transform their arguments into actionable targets:
- `[[set: target]: value]` - Assignment (transforms target into settable)
- `[[define: target]: tag-list]` - Definition (transforms target into initializable)

### Narrative Designators
Tags that attribute content:
- `[[as: label]: text]` - Display narrative text attributed to label
- `[cmt: text]` - Comment (inline documentation)

### Structural Designators
Tags that mark locations in the narrative:
- `[section: label]` - Define section anchor
- `[chapter: label]` - Define chapter anchor

### Resolved Tags and Composition
Tags can be composed to form expressions:
- `[character: alice]` combined with `[attribute: mood]` via property access
- Property access chains: `[[[character: alice]: [attribute: mood]]: [label: happy]]`
- Operators: `[[attribute: a] + [attribute: b]]`

All composed tags must ultimately resolve to data through complete bracket composition.

### Tag Lists
Tag lists are implicit sequences of instruction tags with no special syntax. Multiple instruction tags in sequence form a tag list:

```
[[if: condition]: [[set: ...] [option: ...] [goto: ...]]]
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                  This sequence is the tag list argument
```

Tag lists appear as arguments to instruction tags that require multiple sequential actions.

**Script as Top-Level Tag List:**

An entire script is implicitly wrapped in a tag-list. Tags execute sequentially:
```
[section: home]
[[as: Mark]: Welcome to my home]
[[if: condition]: [option: Go to shop: [goto: [section: shop]]]]
[goto: [section: end]]
```

Is evaluated as:
```
[
  [section: home]
  [[as: Mark]: Welcome to my home]
  [[if: condition]: [option: Go to shop: [goto: [section: shop]]]]
  [goto: [section: end]]
]
```

## Parsing Model

The language uses an evaluate-as-encountered parsing model where all tags conform to the structure `[a: b]`:
- `a` (lvalue) must be an instruction name (e.g., `if`, `set`, `as`), never a resolved tag
- `b` (rvalue) can be a value, a resolved tag, or a tag-list

This distinction prevents ambiguity: resolved tags are always rvalues (right of `:`), never lvalues (left of `:`).

**Parsing Strategy (Evaluate-as-Encountered):**

Brackets are parsed sequentially, creating new nodes at `[` and evaluating at `]`. Each node evaluates its arguments before executing:

```
[[if: [[attribute: day] > [value: 3]]]: [[[set: [[[attribute: books]: [attribute: count]]: [value: 3]]]] [goto: [section: school]]]]

Parse Order:
1. [value: 3] → resolves to integer 3
2. [attribute: day] → resolves to attribute reference
3. [[attribute: day] > [value: 3]] → binary comparison, resolves to flag
4. [[[attribute: books]: [attribute: count]]: [value: 3]] → property access with type, resolves to data
5. [[set: property]: value] → set instruction executes
6. [section: school] → section anchor reference resolves
7. [goto: ...] → goto instruction executes
8. Multiple instructions form tag-list
9. [[if: condition]: tag-list] → root instruction executes with condition and tag-list
```

**Evaluation Principles:**

The evaluate-as-encountered model means:
- All `[a: b]` patterns parse uniformly
- Parser creates a new node at each `[`
- Parser collects tokens until it finds the next `:` (which separates `a` from `b`)
- Arguments (`b`) are fully evaluated before the node executes
- Data tags evaluate immediately to values
- Resolved tags compose through property access and operators
- Instruction tags execute with fully evaluated arguments
- Tag-lists are implicit (consecutive bracketed expressions)

**Operator Parsing (Whitespace-Delimited):**

Operators are identified by whitespace boundaries and do not require brackets. Three patterns:

1. **Binary operators** (between two operands):
   ```
   [[attribute: a] + [attribute: b]]
   [[attribute: x] > [value: 10]]
   [[attribute: count] == [value: 5]]
   ```

2. **Prefix operators** (before operand):
   ```
   [++ [attribute: counter]]
   [-- [attribute: x]]
   [not [flag: condition]]
   ```

3. **Postfix operators** (after operand):
   ```
   [[attribute: counter] ++]
   [[attribute: x] --]
   ```

Rules for operator detection:
- Operators must be surrounded by whitespace (or bracket boundaries)
- Parser recognizes operator symbols: `+ - * / ++ -- and or xor not nand nor xnor > < >= <= == !=`
- After `]`, if next non-whitespace is an operator, the current node continues (not closed)
- Operands are complete bracketed expressions
- Operators are resolved with their operands to produce a value or flag

## Designator and Argument Semantics

All tags follow the structure `[designator: argument]` where a **designator** describes what operation or value is being invoked, and an **argument** is what that designator acts upon.

**Core Asymmetry Rule:** Any tag or value that can be an argument can NEVER be a designator. However, a designator (when it forms a complete tag) can become an argument.

### Designator Types

Designators fall into five semantic categories:

**1. Value Designators** (`value`, `label`, `flag`, `character`, `attribute`, `item`, `null`)
- Describe or reference values
- Cannot be used as arguments (only complete tags with these can be)
- Examples:
  - `[value: 100]` - value designator with integer argument
  - `[character: alice]` - character designator with label argument

**2. Action Designators** (`if`, `option`, `goto`)
- Execute operations that branch or jump control flow
- The designator name itself can ONLY appear in a complete tag
- When the complete tag is used as an argument, it resolves to its meaning
- Examples:
  - `[[if: condition]: tag-list]` - `if` designator (cannot use bare `if` as argument)
  - `[goto: [section: shop]]` - `goto` designator with section reference

**3. Transformative Designators** (`set`, `define`)
- Transform their argument into something actionable
- `[set: property]` transforms the property into "something that can be assigned to"
- `[define: entity]` transforms the entity into "something that can be initialized"
- These are special: the complete tag `[set: property]` can be used as an argument because it resolves to a settable target
- Examples:
  - `[[set: [[[character: alice]: [attribute: mood]]: [label: happy]]]` - set designator
  - `[[define: [character: alice]]: tag-list]` - define designator

**4. Structural Designators** (`section`, `chapter`)
- Mark locations in the narrative
- Cannot use bare names; only complete tags
- Examples:
  - `[section: home]` - section designator with label
  - `[goto: [section: shop]]` - section tag as argument

**5. Narrative Designators** (`as`, `cmt`)
- Tag narrative content with attribution or comments
- Examples:
  - `[[as: Alice]: I am here.]` - as designator with label
  - `[cmt: This is a comment]` - comment designator

### Argument Types

Arguments can be:
- **Literal values** (not bracketed): `100`, `hello`, `on`
- **Complete designator tags** (fully bracketed and resolvable): `[value: 100]`, `[section: shop]`, `[if: condition]`
- **Tag-lists** (sequences of instructions): multiple instructions in sequence

Arguments CANNOT be:
- Bare designator names: `value`, `if`, `goto`, `character`, etc.

### Examples

Valid argument usage:
- ✅ `[[set: target]: [value: 100]]` - literal argument
- ✅ `[[if: condition]: [[set: target]: [value: 100]]]` - complete action tag as argument
- ✅ `[goto: [section: shop]]` - complete structural tag as argument
- ✅ `[[define: [character: alice]]: [tag-list]]` - complete designator tag as argument

Invalid argument usage:
- ❌ `[if: ...]` - bare `if` cannot be an argument
- ❌ `[goto: shop]` - bare `shop` is not a designator tag; must be `[section: shop]`
- ❌ `[[character: alice]: value]` - value designator cannot be a designator itself

## Tag Syntax

### Instruction Tags (Full List)
- `[[if: resolved-tag]: tag-list]` - Conditional execution
- `[[set: resolved-tag]: resolved-tag]` - Assignment
- `[[option: label]: tag-list]` - Display option; execute tags on selection
- `[goto: resolved-tag]` - Jump to section
- `[[define: resolved-tag]: tag-list]` - Define variables/characters

### Flow Control Tags (Full List)
- `[section: label]` - Define section anchor (display label, then narrative text)
- `[chapter: label]` - Define chapter anchor (special section)
- `[cmt: text]` - Comment (inline documentation)

### Data Access Tags
- `[character: name]` - Reference or define character (container type)
- `[attribute: name]` - Reference or define attribute (container type)

### Type System
Characters and attributes are distinct types but behave identically:
- Both are containers that can nest other attributes
- Both support property access
- `[character: alice]` is type Character
- `[attribute: wealth]` is type Attribute
- Distinction is semantic; behavior is identical

### Literal Tags
Literal tags serve dual purposes: creating literals and enforcing/checking types.

**As Literals:**
- `[value: 100]` - Integer literal
- `[label: hello]` - String literal
- `[flag: on|off]` - Flag literal

**As Type Enforcers/Checkers:**
- `[value: expression]` - Asserts expression resolves to integer; throws error on mismatch
- `[label: expression]` - Asserts expression resolves to string; throws error on mismatch
- `[flag: expression]` - Asserts expression resolves to flag; throws error on mismatch
- `[null: expression]` - Checks if attribute exists; returns `on` if missing, `off` if exists

Examples:
- `[value: [[[character: bob]: [attribute: gold]]]]` - If gold is not an integer, error.
- `[null: [[[character: player]: [attribute: inventory]]: [attribute: sword]]]` - Returns `on` if sword doesn't exist, `off` if it does.

### Operators
Arithmetic: `+ - * / ++ --`
Comparison: `> < >= <= == !=`
Logical: `and or xor not nand nor xnor`

### Expressions
```
expression => [tag op tag] | [op tag] | [tag op]
```
Expressions return either `flag` (boolean/flag) or `value` (integer).

## Property Access

Property access is strict and fully typed. Every level must specify its type:
```
[label: [[[character: alice]: [attribute: mood]]: [label]]]
[value: [[[character: alice]: [attribute: inventory]]: [value]]]
[value: [[[attribute: wealth]: [attribute: current]]: [value]]]
```

- Inner brackets resolve to a container (character or attribute)
- Outer brackets access that container's property with explicit type wrapper
- All properties must be typed; no implicit access
- Nesting is unlimited

## Instruction Sequences

Instructions execute sequentially in tag-lists. Example:
```
[[if: [[attribute: wealth]: [attribute: current]] > [value: 100]]: [[[option: Buy item]: [goto: [section: shop]]]]]
[[set: [[[attribute: visited]: [attribute: flag]]: [flag: on]]]]
[goto: [section: next]]
```

- Instructions within tag-lists execute in order
- Tag-lists are implicit (no special delimiters)
- All tags resolve before control flow jumps

## Narrative Text

All narrative text must be explicitly tagged with `[[as: label]: text]`. The label attributes the text to a speaker or context:

```
[section: home]
[[as: Narrator]: Alice's home is cozy.]
[[as: Alice]: I have [value: [[[character: alice]: [attribute: gold]]]] gold coins.
[[as: Title]: The Shopping District]
```

Data tags within text are resolved and displayed:
- `[[[character: alice]: [attribute: name]]: [label]]` - Display character's name (string)
- `[[[attribute: gold]: [attribute: amount]]: [value]]` - Display attribute value (integer)
- `[[[character: bob]: [attribute: wealthy]]: [flag]]` - Display attribute value (flag)
- `[label: hello]` - Display string literal
- `[value: 42]` - Display integer literal
- `[flag: on]` - Display flag literal

Instruction tags (e.g., `[goto:]`) in narrative text are invalid.

## Definitions

Characters and attributes are defined via `[define:]` instructions:
```
[[define: [character: alice]]: [[[set: [[[character: alice]: [attribute: name]]: [label: Alice]]]] [[[set: [[[character: alice]: [attribute: mood]]: [label: happy]]]]]]]

[[define: [attribute: wealth]]: [[[set: [[[attribute: wealth]: [attribute: current]]: [value: 0]]]]]]
```

- Definitions can occur anywhere but must precede first use
- Definitions are tag-lists containing initialization instructions
- Property initialization uses fully typed access syntax

## Execution Model

1. Script begins at first `[section:]` or `[chapter:]` found
2. Section label is displayed
3. Narrative text following section is printed, with data tags resolved inline
4. Instructions in tag-lists execute sequentially (left-to-right)
5. All data tags and resolved tags evaluate before instruction execution
6. `[goto:]` instructions jump to target section immediately
7. Control flow continues from target section

## Notes

- Whitespace is syntactically important only for operator detection (operators require whitespace boundaries like ` + `, ` > `). All other whitespace is ignored during parsing. Semantic trimming occurs for labels and narrative text (leading/trailing whitespace removed).
- Chapters are structurally identical to sections (label + narrative)
- Tag-lists are implicit (no special syntax); consecutive instructions form a list
- `[if:]` without matching condition skips the entire instruction (no `[else:]`)
- Variables must be defined before first use
- Nested property access is unlimited: `[[[[object]: prop1]: prop2]: prop3]`
- Type enforcement is optional but recommended: wrap expressions with `[value:]`, `[label:]`, `[flag:]` to assert correct types
- `[null: expression]` checks if attribute exists; useful for optional attributes: inventory items, quest flags, etc.

## Examples

Setting a property:
```
[[set: [[[character: alice]: [attribute: mood]]: [label: happy]]]]
```

Setting a nested attribute:
```
[[set: [[[attribute: wealth]: [attribute: current]]: [value: 100]]]]
```

Incrementing:
```
[[set: [[[attribute: counter]: [attribute: value]]: [value: [++ [value: [[[attribute: counter]: [attribute: value]]]]]]]]]
```

Binary operations:
```
[[set: [[[attribute: temples]: [attribute: value]]: [value: [[value: [[[attribute: gold]: [attribute: value]]]] + [value: [[[attribute: faith]: [attribute: value]]]]]]]]]
```

Character dialogue with interpolation:
```
[section: meeting]
[[as: Alice]: I have [value: [[[character: alice]: [attribute: gold]]]] gold pieces.]
```

Dynamic inventory management:
```
[[if: [null: [[[character: player]: [attribute: inventory]]: [attribute: sword]]]]: [[[option: You don't have a sword]: [goto: [section: no_sword]]]]]
[[if: [not: [null: [[[character: player]: [attribute: inventory]]: [attribute: sword]]]]: [[[option: Use sword]: [goto: [section: use_sword]]]]]

[[define: [character: player]]: [[[set: [[[character: player]: [attribute: inventory]]: [attribute: sword]]: [value: 1]]]]]
```

Player name input:
```
[[set: [[[character: player]: [attribute: name]]: [label]]: [input: Enter your character name]]]
```

Input with conditional:
```
[[if: [label: [input: Type 'yes' or 'no']] == [label: yes]]: [[[option: Confirmed]: [goto: [section: confirmed]]]]]]
```
