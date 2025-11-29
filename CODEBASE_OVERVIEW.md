# Packard Script Language - Codebase Overview

## High-Level Architecture

The Packard Script Language evaluator follows a clean 4-stage pipeline:

```
Source Code
    ↓
[Lexer] → Tokens
    ↓
[Parser] → TagNode Tree (single root)
    ↓
[Validator] → checks operation names exist
    ↓
[Evaluator] → executes with frame stack for scoping
```

---

## Stage 1: Lexer (src/lexer.rs)

**Purpose**: Convert source text into tokens

**Input**: String source code
**Output**: Vec<Token>

**Token Types**:
- Structural: `[`, `]`, `:`, `,`
- Operators: `+`, `-`, `*`, `/`, `=`, `!=`, `>`, `<`, `>=`, `<=`, `and`, `or`, `not`
- Literals: `Identifier(String)`, `Number(f64)`, `String(String)`, `Keyword(String)`
- Special: `->` (arrow), `Eof`

**Key Methods**:
- `read_identifier()` - reads alphanumeric + `_` + `-`
- `read_number()` - parses integers and floats
- `read_string()` - handles quoted strings with escape sequences
- `skip_line_comment()` - skips `//` comments
- `skip_block_comment()` - skips `/* */` comments

**Note**: Keywords ("on", "off", "and", "or", "not") are distinguished from identifiers at lex time.

---

## Stage 2: Parser (src/streaming_parser.rs)

**Purpose**: Build a TagNode tree from tokens

**Input**: Vec<Token>
**Output**: Single `TagNode` (implicitly wrapped root)

**Core Concept - The Tag Structure**:
```
A tag is: [ltag: rtag]
  - ltag (left tag): can be a primitive or another composite tag
  - rtag (right tag): can be a primitive or another composite tag
  - colon `:` separates them
```

**Parse Algorithm**:
1. Expect `[`
2. Parse ltag (can be primitive or nested `[ltag: rtag]`)
3. Expect `:`
4. Parse rtag (can be primitive or nested `[ltag: rtag]`)
5. Expect `]`

**Special Root Wrapping**:
- All top-level tags are wrapped in an implicit `[root: [list: ...]]` structure
- This makes the evaluator always receive a single root tag
- The list structure nests multiple items as: `[item1, [item2, [item3, ...]]]`

**TagInProgress State Machine**:
- Tracks which side of the tag (ltag or rtag) is being parsed
- Switches from `ParsingLTag` to `ParsingRTag` when `:` is encountered

---

## Stage 3: Type System (src/tag.rs)

**Value Type** (runtime values):
```rust
pub enum Value {
    Number(f64),
    Text(String),
    Flag(bool),          // on/off
    Item,                // placeholder/unit type
    Reference(String),   // points to a variable/attribute name
}
```

**TagNode Type** (compile-time structure):
```rust
pub enum TagNode {
    Composite { ltag: Box<TagNode>, rtag: Box<TagNode> },
    Primitive(Primitive),
}

pub enum Primitive {
    Identifier(String),
    Number(f64),
    String(String),
    Keyword(String),
}
```

**Conversion**:
- `Primitive::to_value()` - converts static primitive to runtime value
- "on" keyword → `Flag(true)`
- "off" keyword → `Flag(false)`
- Identifiers → `Text(string)`
- Numbers → `Number(f64)`
- Strings → `Text(string)`

---

## Stage 4: Evaluator (src/evaluator_v2.rs)

**Purpose**: Execute the TagNode tree with scoping and operation dispatch

**Key Concepts**:

### Handler Registry (lazy_static HashMap)
Operations are dispatched through a centralized registry:
```rust
HANDLERS: HashMap<&'static str, Handler>
```

**Current Handlers**:
- `root` - returns rtag value
- `character` - stores in global store
- `list` - acknowledgement (list nesting handled by parser)
- `text`, `number`, `flag`, `item` - return rtag value
- `attribute` - declares/returns reference to attribute

**Special Operations** (not in HANDLERS):
- `define` - creates new scope block
- `set` - assigns value to reference

### Frame Stack (for scoping)
```rust
pub struct Evaluator {
    frames: Vec<Frame>,  // call stack for scopes
    store: HashMap<String, Value>,  // global store for characters
}

struct Frame {
    variables: HashMap<String, Value>,
    attributes: HashMap<String, Value>,
}
```

**Scope Rules**:
1. Global frame always exists (index 0)
2. `define` blocks push a new frame
3. When exiting a define block, the frame is popped
4. Variable/attribute lookups traverse the frame stack backwards (innermost first)

### Evaluation Flow

**entry: `execute_root(root: &TagNode)`**
1. Calls `validate(root)` - checks all operations are known
2. Calls `evaluate_tag(root)` - executes the tree

**core: `evaluate_tag(tag: &TagNode)`**

For primitives:
- Convert `Primitive` to `Value`
- Return the value

For composites `[ltag: rtag]`:
1. Extract operation name from ltag (walk nested composites to innermost primitive)
2. Dispatch based on operation:
   - **"define"**: call `handle_define_block(rtag)`
   - **"set"**: call `handle_set_block(ltag, rtag)`
   - **other**: evaluate rtag → get value → dispatch to handler registry

### Special Handling

**Define Block** (`handle_define_block`):
```
1. Push new Frame
2. Execute rtag (the content)
3. Pop Frame
4. Return result
```

**Set Block** (`handle_set_block`):
```
1. Extract target from [set: target]
2. Evaluate target → should return Value::Reference(name)
3. Evaluate value to assign
4. Search frames (innermost first) for where name is defined
5. Update that frame's variable/attribute
6. Return assigned value
```

---

## Stage 5: Entry Point (src/main.rs)

**Flow**:
1. Read filename from command line
2. Read file contents
3. Tokenize via `lexer::tokenize()`
4. Parse via `StreamingParser::parse()`
5. Print parsed tree
6. Create Evaluator with log file
7. Execute via `evaluator.execute_root()`
8. Print final store and result

**Logging**:
- All evaluation is traced to `eval_trace.log`
- Each evaluation step gets a unique ID for debugging

---

## Example Execution

**Input Script**:
```
[define: [attribute: x]]
```

**After Lexing**:
```
[ define : [ attribute : x ] ]
```

**After Parsing** (simplified):
```
Composite {
  ltag: Primitive(Keyword("root")),
  rtag: Composite {
    ltag: Primitive(Keyword("list")),
    rtag: Composite {
      ltag: Primitive(Keyword("define")),
      rtag: Composite {
        ltag: Primitive(Keyword("attribute")),
        rtag: Primitive(Identifier("x"))
      }
    }
  }
}
```

**During Evaluation**:
1. Validate: "root", "list", "define", "attribute" all exist in handlers or special handling
2. Evaluate root → evaluate list
3. Evaluate list → evaluate first item
4. Evaluate define → 
   - Push Frame
   - Evaluate [attribute: x]
     - Evaluate "attribute" handler with rtag="x"
     - Handler declares x in current frame with Value::Item
     - Returns Value::Reference("x")
   - Pop Frame
5. Result: Value::Item (the attribute reference)

---

## Key Architecture Decisions

1. **Single Root Tag**: Every program is wrapped in `[root: [list: ...]]` to simplify evaluator logic

2. **Frame Stack**: Scoping is explicit via a Vec of Frame structs pushed/popped on define/set

3. **Handler Registry**: Centralized dispatch prevents scattered match statements; easy to add operations

4. **Reference Values**: Assignment targets are resolved to `Value::Reference(name)` which points to where the variable is stored

5. **Lazy-Static Handlers**: HANDLERS HashMap is computed once at startup; validation just checks the HashMap keys

6. **Special Operation Handling**: "define" and "set" need raw `TagNode` access (not just rtag value), so they're matched before handler dispatch

---

## Current Limitations / Future Work

- No nested attribute resolution (e.g., `obj.attr.nested`)
- No function definitions or calls
- No control flow (if/else, loops)
- set currently only works on existing references (no auto-declaration)
- No type checking beyond runtime
