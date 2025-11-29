# Step-by-Step Walkthrough: `test_minimal.psl`

This document traces the execution of a single script file through all stages of the interpreter.

## The Script

```
[[set: [attribute: name]]: [text: Alice]]
```

This is an assignment that sets the `name` attribute to the text "Alice".

---

## STEP 1: Lexer - Tokenization

**Input**: The raw string above

**Process**: The lexer reads character by character and emits tokens

**Output** (18 tokens):
```
0: OpenBracket          [
1: OpenBracket          [
2: Identifier("set")    set
3: Colon                :
4: OpenBracket          [
5: Identifier("attr")   attribute
6: Colon                :
7: Identifier("name")   name
8: CloseBracket         ]
9: CloseBracket         ]
10: Colon               :
11: OpenBracket         [
12: Identifier("text")  text
13: Colon               :
14: Identifier("Alice") Alice
15: CloseBracket        ]
16: CloseBracket        ]
17: Eof                 (end of input)
```

**How it works**: 
- `[` → `OpenBracket`
- alphabetic identifier → `Identifier(String)`
- `:` → `Colon`
- `]` → `CloseBracket`
- End of input → `Eof`

---

## STEP 2: Parser - Building the TagNode Tree

**Input**: Vec<Token> from Step 1

**Process**: The parser reads tokens and builds a nested TagNode structure

The parser implements a state machine for each tag:
1. See `[` → start new tag, begin parsing ltag
2. Collect tokens until `:` → that's the ltag
3. See `:` → switch to parsing rtag
4. Collect tokens until `]` → that's the rtag
5. See `]` → tag is complete, return it

**Trace for our script**:

```
Token stream: [ [ set : [ attribute : name ] ] : [ text : Alice ] ]

Position 0: OpenBracket - Start parsing tag
  State: ParsingLTag
  
Position 1-9: Parse ltag
  Position 1: OpenBracket - Nested tag!
    Recursively parse: [set: [attribute: name]]
    This returns a composite TagNode
  Position 1-9 completed, ltag = [set: [attribute: name]]

Position 10: Colon - Switch to ParsingRTag

Position 11-16: Parse rtag
  Position 11: OpenBracket - Nested tag!
    Recursively parse: [text: Alice]
    This returns a composite TagNode
  Position 11-16 completed, rtag = [text: Alice]

Position 16: CloseBracket - Tag complete!
```

**Output** (the final TagNode tree):

```
Composite {
  ltag: Composite {
    ltag: Primitive(Identifier("set")),
    rtag: Composite {
      ltag: Primitive(Identifier("attribute")),
      rtag: Primitive(Identifier("name"))
    }
  },
  rtag: Composite {
    ltag: Primitive(Identifier("text")),
    rtag: Primitive(Identifier("Alice"))
  }
}
```

**Then the parser wraps it in root**:

```
Composite {
  ltag: Primitive(Keyword("root")),
  rtag: Composite {
    ltag: Primitive(Keyword("list")),
    rtag: Composite {
      ltag: Composite {
        ltag: Primitive(Identifier("set")),
        rtag: Composite {
          ltag: Primitive(Identifier("attribute")),
          rtag: Primitive(Identifier("name"))
        }
      },
      rtag: Composite {
        ltag: Primitive(Identifier("text")),
        rtag: Primitive(Identifier("Alice"))
      }
    }
  }
}
```

This is shown in the output as:
```
Parsed root tag:
  [
  ltag: root
  rtag: [
    ltag: list
    rtag: [
      ltag: [
        ltag: set
        rtag: [
          ltag: attribute
          rtag: name
        ]
      ]
      rtag: [
        ltag: text
        rtag: Alice
      ]
    ]
  ]
]
```

**Key insight**: Every program becomes `[root: [list: ...]]` automatically. This normalizes the input.

---

## STEP 3: Validator - Check All Operations Exist

**Input**: The TagNode tree from Step 2

**Process**: Walk the entire tree and verify that every operation name (the innermost ltag primitive) is either:
- In the HANDLERS registry, OR
- One of the special operations ("define" or "set")

**For our script**, the validator checks:
- "root" - ✓ in HANDLERS
- "list" - ✓ in HANDLERS
- "set" - ✓ special handling
- "attribute" - ✓ in HANDLERS
- "text" - ✓ in HANDLERS

All operations exist, so validation passes.

---

## STEP 4: Evaluator - Execute the Tree

**State at start**:
```
Evaluator {
  store: {},                    // empty global store
  frames: [Frame::new()],       // one frame (global scope)
  eval_counter: 0,
}
```

Now the evaluator recursively calls `evaluate_tag()` on each node, starting from the root.

### Eval Step 1: Evaluate `[root: [list: ...]]`

```
[Eval 1] Composite tag: [ltag: rtag]
[Eval 1] Operation: root
```

Action:
1. Extract operation name from ltag → "root"
2. It's not "define" or "set", so evaluate rtag
3. `evaluate_tag([list: ...])`

### Eval Step 2: Evaluate `[list: [...]]`

```
[Eval 2] Composite tag: [ltag: rtag]
[Eval 2] Operation: list
[Eval 2] Evaluating rtag...
```

Action:
1. Extract operation name → "list"
2. Evaluate rtag (which is our actual tag)
3. `evaluate_tag([[set: ...]: [text: ...]])`

### Eval Step 3: Evaluate `[[set: [attribute: name]]: [text: Alice]]`

```
[Eval 3] Composite tag: [ltag: rtag]
[Eval 3] Operation: set
```

Action:
1. Extract operation name from ltag → "set"
2. This IS a special operation!
3. Call `handle_set_block(ltag, rtag)`
   - ltag = `[set: [attribute: name]]`
   - rtag = `[text: Alice]`

**Inside handle_set_block**:

Step 3a: Extract target from ltag
```
ltag is Composite { ltag: Primitive("set"), rtag: [attribute: name] }
target = [attribute: name]
```

Step 3b: Evaluate target
```
evaluate_tag([attribute: name])
  → Operation is "attribute"
  → Evaluate rtag "name" → Value::Text("name")
  → Call handle_attribute(Value::Text("name"))
    → Checks if "name" exists in any frame: NO
    → Declares "name" in current frame with Value::Item
    → Returns Value::Reference("name")
```

From eval trace:
```
[Eval 4] Composite tag: [ltag: rtag]
[Eval 4] Operation: attribute
[Eval 4] Evaluating rtag...
[Eval 5] Primitive: name => "name"
[Eval 4]   rtag evaluated to: "name"
[Eval 4] Handler result: &name
```

Step 3c: Evaluate the value to assign
```
evaluate_tag([text: Alice])
  → Operation is "text"
  → Evaluate rtag "Alice" → Value::Text("Alice")
  → Call handle_text(Value::Text("Alice"))
    → Returns Value::Text("Alice")
```

From eval trace:
```
[Eval 6] Composite tag: [ltag: rtag]
[Eval 6] Operation: text
[Eval 6] Evaluating rtag...
[Eval 7] Primitive: Alice => "Alice"
[Eval 6]   rtag evaluated to: "Alice"
[Eval 6] Handler result: "Alice"
```

Step 3d: Perform assignment
```
target_value = Value::Reference("name")
value = Value::Text("Alice")

Search frames in reverse:
  - Current frame (index 0) has "name" in attributes
  - Update it: attributes["name"] = Value::Text("Alice")

Return Value::Text("Alice")
```

From eval trace:
```
[Eval 3] Handler result: "Alice"
```

### Eval Step 2 (continued): Back in list handler

```
[Eval 2]   rtag evaluated to: "Alice"
[Eval 2] Handler result: item
```

The list handler gets the value "Alice" from evaluating its rtag, and returns Value::Item (acknowledgement).

### Eval Step 1 (continued): Back in root handler

```
[Eval 1]   rtag evaluated to: item
[Eval 1] Handler result: item
```

The root handler gets Value::Item from list, and returns it.

---

## Final State

**Evaluator state after execution**:
```
Evaluator {
  store: {},                    // not used by this script
  frames: [
    Frame {
      variables: {},
      attributes: { "name": Value::Text("Alice") }
    }
  ],
}
```

**Output**:
```
Evaluation trace written to eval_trace.log
Result: item
Variable store:
  (empty - "name" is an attribute, not in the store)
```

---

## Key Takeaways

1. **Lexer** converts text → tokens (purely syntactic)
2. **Parser** converts tokens → tree (builds nested structure)
3. **Parser wraps everything** in `[root: [list: ...]]` automatically
4. **Validator** walks tree and checks operation names are known
5. **Evaluator** recursively evaluates each node:
   - Primitives → convert to Value
   - Composites → extract operation, dispatch to handler
   - Special ops (set, define) get raw TagNode access
6. **Frame stack** tracks scope; each define pushes a frame
7. **Assignment** (set) searches up the frame stack to find where the variable is declared
