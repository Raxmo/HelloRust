# Packard Script Language Specification

## Overview

Packard is a script language designed for interactive fiction writers, not necessarily for programmers. It uses intuitive tag-based syntax to structure narratives, conditionals, and player choices without much programming knowledge.

## Core Concepts

### The Tag Structure

Every tag in Packard follows a simple, consistent pattern: **[left: right]**

This uniformity makes the language learnable and predictable. Once you understand the pattern, you can build sophisticated stories from simple, repeatable pieces.

**Exceptions:** Tag lists (sequences of instructions), operators (like `+`, `>`, `and`), and the property accessor (`->`) don't follow the `[left: right]` pattern. These are special cases, but they're easy to recognize and understand in context.

### Tag Roles: FTags, LTags, RTags, and CTags

Every tag in Packard has a fixed, static role determined by its structure, not where it appears.

**FTags (Full Tags)**
FTags are complete, self-contained tags that resolve to completion. They require no further context. They are the only tags that can stand alone.

Examples: `[goto: location]`, `[[if: condition]: tag-list]`, `[section: home]`, `[chapter: intro]`

**RTags (Right Tags)**
RTags are tags that appear on the right side of a colon. They can be primitive values or composite tags, but they always require an LTag to give them meaning. RTags never stand alone.

- Primitive RTags: `100`, `on`, `Alice`, `home`
- Composite RTags: `[value: 100]`, `[text: Alice]`, `[character: alice]`, `[container: inventory]`, `[attribute: name]`

**LTags (Left Tags)**
LTags are tags that appear on the left side of a colon. They determine what operation or type is being invoked. LTags never stand alone.

- Primitive LTags: `if`, `set`, `define`, `add`, `remove`, `goto`, `section`, `value`, `number`, `text`, `flag`, `item`, `character`, `attribute`, `container`
- Composite LTags: `[if: condition]`, `[null: expression]`

**CTags (Composite Tags)**
CTags are any tags with the structure `[ltag: rtag]`. All FTags are CTags. RTags and LTags may also be CTags if they contain the bracket-colon structure.

**Type Designators**
Eight LTags serve as type or declaration designators: `value`, `number`, `text`, `flag`, `item`, `character`, `attribute`, `container`. When combined with an RTag, they form composite RTags. For example, `[number: 100]` and `[text: Alice]` are composite RTags.

**Key Principle**
A tag's role is determined by its structure and purpose, not by its context. `[value: 100]` is always an RTag. `[goto: home]` is always an FTag. This static assignment makes the language predictable.

### Property Access with the Accessor Operator

To access properties of characters and containers, use the `->` operator:

```
[[character: alice] -> [attribute: name]]
[[[character: alice] -> [container: bag]] -> [attribute: sword]]
```

The `->` operator chains property access left-to-right, making it intuitive for writers. Each step drills deeper into the structure.

### Nesting and Composition

Tags can contain other tags. This lets you build sophisticated logic by stacking simple pieces.

Example: A conditional that sets a value based on a condition, which itself depends on a character's container property.

The language handles all the nesting—you just focus on the story.

### Everything is Data

Characters, attributes, items, and containers are all treated as data you can read, write, and compose. This uniformity makes the language predictable and learnable.

## Type System

Packard has four primitive types that form the foundation of all data:

### Number
Integers representing quantities, counts, and numerical values.

Examples: `100`, `-5`, `0`, `9001`

### Text
Strings representing names, dialogue, descriptions, and any text content.

Examples: `Alice`, `The forest is dark`, `East Gate`

### Flag
Boolean values representing true/false states, on/off switches, and existence checks.

Values: `on` (true) or `off` (false)

Examples: `on`, `off`

### Item
A type representing existence without data. Used for inventory, optional attributes, or quest markers—where you only need to know if something exists, not what value it holds.

Examples: A sword you either have or don't; a location you've visited or haven't

### Value Types vs. Structural Types

The four primitives (Number, Text, Flag, Item) are **value types**—they represent actual data values. They behave independently and can be assigned, compared, and manipulated.

Attribute, Container, and Character are **structural types**—they are declarations that organize and hold primitive values. They do not represent data themselves; instead, they define how data is organized within a script. You cannot assign a value directly to a Character; you must first declare its attributes and containers, then populate those with primitive values.

### Structural Types

**Attribute** - A singular value of one of the primitive types (Number, Text, Flag, or Item). Attributes are leaf values and cannot contain other attributes or items.

Examples: A character's name (`[attribute: name]` with value `[text: Alice]`), health (`[attribute: hp]` with `[number: 100]`), or visited flag (`[attribute: seen]` with `[flag: on]`)

**Container** - A structure that holds multiple attributes, items, or other containers. Containers organize and group related data.

Examples: An inventory container `[container: bag]` holding sword items, a stats container `[container: stats]` holding number attributes like health and mana

**Character** - A special container type representing an entity (player, NPC, object) in the story. Characters can hold attributes, items, and containers.

Examples: The player character `[character: player]`, an NPC `[character: merchant]`, a treasure chest `[character: chest]`

## Tag Behavior

### Resolution and Evaluation Flow

Tags are evaluated recursively from the innermost expressions outward. RTags are resolved to values before being passed to LTags, which then perform operations or transformations.

### Implicit Property Access in Define Blocks

Within `define` blocks, properties are accessed without requiring the full `->` chain. The implicit context allows nested definitions to reference parent containers and attributes directly.

### Property Accessor Chaining

The `->` operator chains property access left-to-right. Each step returns an RTag representing the accessed property, which can be further chained or passed to an LTag.

### Scope and Context

Properties and attributes defined within a `define` block belong to their target declaration. Nested `define` blocks create child contexts within their parent, but changes persist after the block closes.

### Type Behavior

Attributes hold typed values (Number, Text, Flag). Items represent existence without value. Type mismatches are handled through assertions and casting (see type system).

## Operators

### Arithmetic Operators

- `+` (Addition)
- `-` (Subtraction)
- `*` (Multiplication)
- `/` (Division)
- `+=` (Add and Assign)
- `-=` (Subtract and Assign)
- `*=` (Multiply and Assign)
- `/=` (Divide and Assign)
- `++` (Increment)
- `--` (Decrement)

### Comparison Operators

- `>` (Greater Than)
- `<` (Less Than)
- `=` (Equal To)
- `!=` (Not Equal To)
- `>=` (Greater Than or Equal)
- `<=` (Less Than or Equal)

### Logical Operators

- `and` (Logical AND)
- `or` (Logical OR)
- `not` (Logical NOT)
- `xor` (Exclusive OR)
- `nor` (Logical NOR)
- `nand` (Logical NAND)
- `xnor` (Exclusive NOR)

## Tag Reference

### `define`

**Type:** LTag (Operation)

**Syntax:** `[[define: declaration-target]: tag-list]`

**Description:** Creates a scoped initialization context for characters, containers, or attributes. Within a define block, property access is implicit, and Items accept text labels as initialization values.

**Examples:**
```
[[define: [character: alice]:
    [[set: [attribute: name]]: [text: Alice]]
    [[define: [container: bag]:
        [[set: [attribute: sword]]: [item:]]
        [[set: [attribute: capacity]]: [number: 100]]
    ]]
]]
```

**Key behaviors:**
- Properties are accessed implicitly within the block (no need for full `->` paths)
- All attributes and items use `set` for uniform modification semantics
- Multiple attributes and nested containers can be defined together

### `add`

**Type:** LTag (Operation)

**Syntax:** `[[add: property-accessor]: value-rtag]`

**Description:** Inserts new attributes or items into existing containers after initial definition.

**Examples:**
```
[[set: [[character: alice] -> [container: bag]] -> [attribute: shield]]: [item]]
```

**Key behaviors:**
- Uses property accessor targeting via `->` to specify insertion point
- Always routes through `set` to maintain uniform modification semantics
- Items are represented as `[item]` without labels or values

### `remove`

**Type:** LTag (Operation)

**Syntax:** `[[remove: property-accessor]]`

**Description:** Deletes attributes or items from existing containers.

**Examples:**
```
[[remove: [[character: alice] -> [container: bag]] -> [attribute: sword]]]
```

**Key behaviors:**
- Takes only a property accessor as its RTag
- No value required—the target itself is deleted
- Removes existence (for items) or the entire attribute and its value

### `become`

**Type:** LTag (Type Coercion)

**Syntax:** `[become: [type: rtag]]`

**Description:** Explicitly coerces a value to a different type. The compiler validates that the conversion is semantically valid at compile time.

**Examples:**
```
[become: [text: [attribute: hp]]]
[become: [number: [text: 100]]]
```

**Key behaviors:**
- Asserts the source type is known and valid
- Validates the target type conversion at compile time
- Returns the coerced value as the target type
- Invalid conversions (e.g., arbitrary text to number) are compile-time errors

## Examples

### Defining a Character with Items

Within a `define` block, Item RTags accept text labels for initialization. This allows you to declare items without needing data values.

```
[[define: [character: alice]:
    [[set: [attribute: name]]: [text: Alice]]
    [[define: [container: bag]]:
        [item: book]
        [[set: [attribute: capacity]]: [number: 100]]
        [[set: [attribute: usage]]: [number: 1]]
    ]
]
```

**Breaking this down:**
- `[[define: [character: alice]:` — Begin defining character alice
- `[[set: [attribute: name]]: [text: Alice]]` — Direct attribute assignment (set takes an accessor LTag and a value RTag)
- `[[define: [container: bag]]:` — Define bag as a container that can hold attributes and items
- `[item: book]` — Item takes text and builds a complete FTag within the define context
- `[[set: [attribute: capacity]]: [number: 100]]` — Add an attribute to the container with a number value
- `[[set: [attribute: usage]]: [number: 1]]` — Add another attribute to the container
- `]` — End the bag container define
- `]` — End alice character define

**Key insights:**
- Within define, property access is implicit (no need for full paths)
- Items accept text labels as RTags only in define blocks
- Outside of define, Items work as existence checks without data
### Post-Initialization with `add`

After initial definition, use `add` to insert new items into existing containers:

```
[[set: [[character: alice] -> [container: bag]] -> [attribute: sword]]: [item]]
```

This adds a sword item to alice's bag. The item's existence is represented by `[item]` with no label or value—consistent with how all data modifications use `set`.

### Post-Initialization with `remove`

Use `remove` to delete attributes or items from existing containers:

```
[[remove: [[character: alice] -> [container: bag]] -> [attribute: sword]]]
```

This removes the sword item from alice's bag. The `remove` LTag takes a property accessor as its RTag and requires no value—the attribute itself is the target.

## Script Execution Flow

### Execution Order of Nested Tags

### Tag Evaluation and Resolution

### Control Flow with Conditionals

### Results and Side Effects

### Script Entry and Exit Points

## Compilation and Semantics

Packard performs complete static analysis at script load time. This means syntax errors, type mismatches, undefined references, and failed assertions are all discovered when the script is opened—before any execution begins. Writers receive immediate feedback on script integrity.

### Parsing and Grammar

The parser validates that all tags follow the `[left: right]` structure and that exceptions (tag lists, operators, `->`) are used correctly. Malformed tags are rejected immediately with clear error messages indicating location and issue.

Example errors caught:
- Missing colons: `[if condition]` (should be `[if: condition]`)
- Mismatched brackets: `[character: alice`
- Invalid nesting: `[[set: [attribute: name]: [text: Alice]]: [number: 100]]` (nested too deep)

### Type Resolution and Inference

The compiler traces property access chains to determine types. When `[character: alice] -> [attribute: hp]` is accessed, the compiler verifies:
- That `alice` is declared as a character
- That `alice` has a `bag` container
- That `bag` has an `hp` attribute
- What type `hp` holds (Number, Text, Flag, or Item)

Type mismatches are reported at load time.

### Scope and Symbol Resolution

Symbols (character, container, attribute names) are resolved within their scopes. `define` blocks create local symbol tables. References to undefined symbols are caught at load time.

Example errors caught:
- `[character: unknown]` when `unknown` was never declared
- `[[character: alice] -> [container: undefined]]` when alice has no such container

### Semantic Validation

Beyond syntax and symbols, the compiler validates semantic rules:
- Type assertions on reads: `[number: [attribute: hp]]` asserts hp is numeric at compile time
- Type coercion with `[become: [type: value]]` is validated—conversions between types are checked. For example, `[become: [text: [attribute: hp]]]` coerces hp to text, with an assertion that the conversion is valid.

### Error Handling and Assertions

Failed assertions are compilation errors. If a script asserts a type or condition that cannot be proven true from the declarations, the compiler rejects the script with a detailed error message showing:
- What assertion failed
- Where in the script
- What type or value was expected vs. what was found