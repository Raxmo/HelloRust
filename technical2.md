# Packard Script Language Specification

## Overview

Packard is a script language designed for interactive fiction writers, not necessarily for programmers. It uses intuitive tag-based syntax to structure narratives, conditionals, and player choices without much programming knowledge.

## Core Concepts

### The Tag Structure

Every tag in Packard follows a simple, consistent pattern: **[left: right]**

This uniformity makes the language learnable and predictable. Once you understand the pattern, you can build sophisticated stories from simple, repeatable pieces.

**Exceptions:** Tag lists (sequences of instructions) and operators (like `+`, `>`, `and`) don't follow the `[left: right]` pattern. These are special cases, but they're easy to recognize and understand in context.

### Tag Roles: FTags, LTags, RTags, and CTags

Every tag in Packard has a fixed, static role determined by its structure, not where it appears.

**FTags (Full Tags)**
FTags are complete, self-contained tags that resolve to completion. They require no further context. They are the only tags that can stand alone.

Examples: `[goto: location]`, `[[if: condition]: tag-list]`, `[section: home]`, `[chapter: intro]`

**RTags (Right Tags)**
RTags are tags that appear on the right side of a colon. They can be primitive values or composite tags, but they always require an LTag to give them meaning. RTags never stand alone.

- Primitive RTags: `100`, `on`, `Alice`, `home`
- Composite RTags: `[value: 100]`, `[label: Name]`, `[character: alice]`

**LTags (Left Tags)**
LTags are tags that appear on the left side of a colon. They determine what operation or type is being invoked. LTags never stand alone.

- Primitive LTags: `if`, `set`, `value`, `character`, `section`
- Composite LTags: `[if: condition]`, `[character: alice]`, `[null: expression]`

**CTags (Composite Tags)**
CTags are any tags with the structure `[ltag: rtag]`. All FTags are CTags. RTags and LTags may also be CTags if they contain the bracket-colon structure.

**Key Principle**
A tag's role is determined by its structure and purpose, not by its context. `[value: 100]` is always an RTag. `[goto: home]` is always an FTag. This static assignment makes the language predictable.

### Nesting and Composition

Tags can contain other tags. This lets you build sophisticated logic by stacking simple pieces.

Example: A conditional that sets a value based on a condition, which itself depends on a character attribute.

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

### Composite Types

**Attribute** - A singular value of one of the primitive types (Number, Text, or Flag). Attributes are leaf values and cannot contain other attributes or items.

Examples: A character's name (Text), health (Number), or visited flag (Flag)

**Container** - A structure that holds multiple attributes, items, or other containers. Containers organize and group related data.

Examples: An inventory container holding sword items, a stats container holding number attributes

**Character** - A special container type representing an entity (player, NPC, object) in the story. Characters can hold attributes, items, and containers.

Examples: The player character, an NPC, a treasure chest

## Tag Behavior

## Operators

## Tag Reference

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
{{ sugested post initialization additions
[[define: [[character: alice]: [attribute: bag]]]:
    [[set: [attribute: straps]]: [number: 2]]
]

}}
## Script Execution Flow
