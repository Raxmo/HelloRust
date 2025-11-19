# Rust Reference Guide

## Enums

Enums are discriminated unions that can hold different data per variant.

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

let msg = Message::Move { x: 10, y: 20 };
```

### Common Enums

**Option<T>** - represents Some(value) or None:
```rust
enum Option<T> {
    Some(T),
    None,
}

let x: Option<i32> = Some(5);
let y: Option<i32> = None;
```

**Result<T, E>** - represents Ok(value) or Err(error):
```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let result: Result<i32, String> = Ok(42);
```

---

## Pattern Matching (match)

Match is a control flow construct that evaluates a value against patterns.

### Basic Syntax
```rust
match value {
    pattern1 => code1,
    pattern2 => code2,
    _ => default_code,
}
```

### Pattern Types

**Literal values:**
```rust
match number {
    1 => println!("One"),
    2 => println!("Two"),
    _ => println!("Other"),
}
```

**Ranges:**
```rust
match score {
    0..=50 => println!("F"),
    51..=70 => println!("C"),
    71..=90 => println!("B"),
    91..=100 => println!("A"),
    _ => println!("Invalid"),
}
```

**Multiple patterns (OR):**
```rust
match day {
    "Saturday" | "Sunday" => println!("Weekend"),
    _ => println!("Weekday"),
}
```

**Binding variables:**
```rust
match point {
    (0, y) => println!("On y-axis at {}", y),
    (x, y) => println!("Point at {}, {}", x, y),
}
```

**Destructuring enums:**
```rust
match message {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to {}, {}", x, y),
    Message::Write(text) => println!("Text: {}", text),
    Message::ChangeColor(r, g, b) => println!("Color: {}, {}, {}", r, g, b),
}
```

**Guards (additional conditions):**
```rust
match number {
    n if n % 2 == 0 => println!("Even"),
    n if n > 10 => println!("Odd and greater than 10"),
    _ => println!("Odd"),
}
```

### Match Expressions Return Values
```rust
let result = match number {
    1 => "one",
    2 => "two",
    _ => "other",
};
```

### Exhaustiveness
The compiler enforces handling all patterns or using `_` for catch-all.

---

## Traits

Traits define shared behavior that types can implement. Think of them as interfaces.

### Defining a Trait
```rust
trait Animal {
    fn make_sound(&self) -> String;
    fn move_around(&self);
}
```

### Implementing a Trait
```rust
struct Dog;

impl Animal for Dog {
    fn make_sound(&self) -> String {
        String::from("Woof!")
    }
    fn move_around(&self) {
        println!("Dog runs");
    }
}
```

### Default Implementations
```rust
trait Animal {
    fn make_sound(&self) -> String;
    fn describe(&self) {
        println!("This is an animal");
    }
}
```

### Trait Bounds
Constrain generics to types implementing specific traits:
```rust
fn describe<T: Animal>(animal: T) {
    println!("{}", animal.make_sound());
}
```

### Trait Objects
Use `dyn` to work with any type implementing a trait:
```rust
fn animal_sound(animal: &dyn Animal) {
    println!("{}", animal.make_sound());
}
```

---

## Ownership & Borrowing (Basics)

### Ownership Rules
1. Each value has one owner
2. When owner goes out of scope, value is dropped
3. Transfer ownership with moves

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 moved to s2, s1 no longer valid
// println!("{}", s1);  // ERROR: s1 moved
println!("{}", s2);  // OK
```

### Borrowing (References)
Borrow a value without taking ownership using `&`:

```rust
let s1 = String::from("hello");
let s2 = &s1;  // Borrow s1
println!("{}", s1);  // OK, s1 still owns value
println!("{}", s2);  // OK, s2 borrows it
```

### Mutable Borrowing
Mutably borrow using `&mut`:

```rust
let mut s = String::from("hello");
let s2 = &mut s;  // Mutable borrow
s2.push_str(" world");
println!("{}", s);  // OK
```

### Borrowing Rules
- **Many immutable references** OR **one mutable reference** at a time
- Never both simultaneously
- References must be valid

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;  // OK - multiple immutable
let r3 = &mut s;  // ERROR - can't mix mutable and immutable
```

---

## Common Patterns

### Using Option
```rust
let x: Option<i32> = Some(5);

match x {
    Some(value) => println!("Value: {}", value),
    None => println!("No value"),
}

// Using if let
if let Some(value) = x {
    println!("Value: {}", value);
}
```

### Using Result
```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}

match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}
```

### if let
Simpler match for single pattern:
```rust
if let Some(value) = option {
    println!("Value: {}", value);
}
```

---

## Key Differences from C++

| Aspect | C++ | Rust |
|--------|-----|------|
| Enums | Named constants | Discriminated unions with data |
| Pattern matching | switch/case | match with exhaustiveness |
| Memory safety | Manual, error-prone | Enforced by compiler |
| Error handling | Exceptions or codes | Result type (explicit) |
| Inheritance | Yes, class-based | No, trait-based composition |
| References | Raw pointers | Safe references with rules |

