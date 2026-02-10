# Module 01 — Rust Fundamentals: Ownership & Types

## Learning Objectives

- Understand Rust's type system and variable bindings
- Master ownership, borrowing, and the borrow checker
- Work with references and lifetimes at a basic level
- Understand mutability rules

---

## 1. Variables and Mutability

In Rust, variables are **immutable by default**:

```rust
fn main() {
    let x = 5;
    // x = 6;  // ERROR: cannot assign twice to immutable variable
    
    let mut y = 5;  // `mut` makes it mutable
    y = 6;          // OK
    println!("x = {}, y = {}", x, y);
}
```

### Shadowing

You can re-declare a variable with `let` — this is called shadowing:

```rust
let x = 5;
let x = x + 1;      // New binding, shadows the old one
let x = x * 2;      // Shadows again
println!("{}", x);   // 12
```

> **In our app:** `AppState::new()` uses shadowing in several places, like
> re-binding `repos` from the config into a `Vec<Repo>`.

## 2. Scalar Types

```rust
let integer: i32 = 42;          // Signed 32-bit integer
let unsigned: u64 = 100;        // Unsigned 64-bit integer  
let float: f64 = 3.14;          // 64-bit float
let boolean: bool = true;       // Boolean
let character: char = '🚀';     // Unicode character (4 bytes)
let unit: () = ();               // Unit type (like void)
```

Common integer types you'll see in our app:

| Type | Use in our app |
|------|---------------|
| `usize` | Vector indices, list lengths |
| `u64` | GitHub run IDs (`last_run_id: Option<u64>`) |
| `u16` | Terminal dimensions (from ratatui) |
| `i64` | Fuzzy match scores |

## 3. Compound Types

### Tuples

```rust
let point: (i32, i32) = (10, 20);
let (x, y) = point;               // Destructuring
println!("x={}, y={}", x, y);
```

> **In our app:** `fetch_repo_details` returns a tuple `(Vec<String>, Vec<String>)` 
> — branches and workflow names.

### Arrays and Slices

```rust
let arr = [1, 2, 3, 4, 5];    // Fixed-size array
let slice = &arr[1..3];        // Slice: [2, 3]
```

## 4. Ownership — The Core Concept

Every value in Rust has exactly **one owner**. When the owner goes out of scope, the value is dropped.

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;          // s1 is MOVED to s2
    // println!("{}", s1); // ERROR: s1 no longer valid
    println!("{}", s2);    // OK
}
```

### Why This Matters

Ownership prevents:
- Double frees (two variables freeing the same memory)
- Use-after-free (accessing memory that was freed)
- Data races (two threads mutating the same data)

All checked **at compile time** — no garbage collector, no runtime cost.

## 5. Borrowing — References

Instead of moving, you can **borrow** a value:

```rust
fn print_length(s: &String) {    // Immutable borrow
    println!("Length: {}", s.len());
}

fn main() {
    let s = String::from("hello");
    print_length(&s);     // Borrow s
    println!("{}", s);    // s is still valid!
}
```

### Mutable Borrowing

```rust
fn add_exclamation(s: &mut String) {
    s.push('!');
}

fn main() {
    let mut s = String::from("hello");
    add_exclamation(&mut s);
    println!("{}", s);    // "hello!"
}
```

### The Rules

1. You can have **any number of immutable references** (`&T`)
2. OR **exactly one mutable reference** (`&mut T`)
3. Never both at the same time

> **In our app:** The event loop borrows `AppState` mutably (`&mut state`),
> and the render function borrows `Frame` mutably (`&mut Frame`). You'll see
> `&self` (immutable borrow) and `&mut self` (mutable borrow) throughout.

## 6. The `Copy` and `Clone` Traits

Simple types that live entirely on the stack implement `Copy` — they're duplicated automatically:

```rust
let x = 5;
let y = x;      // Copy, not move — both valid
println!("{} {}", x, y);
```

For heap-allocated types, you must explicitly `.clone()`:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();    // Deep copy
println!("{} {}", s1, s2);  // Both valid
```

> **In our app:** Many places use `.clone()` on strings — like 
> `repo_name.clone()` when we need to keep the original while also passing 
> it to a function.

## 7. Functions

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b    // No semicolon = this is the return value (expression)
}

fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    let sum = add(3, 4);
    greet("Rustacean");
}
```

### Expression-Based Language

In Rust, almost everything is an expression. The last expression in a block (without `;`) is its value:

```rust
let x = {
    let a = 5;
    let b = 10;
    a + b        // This is the value of the block
};
// x is 15
```

## 8. Type Inference

Rust has powerful type inference — you don't always need to annotate types:

```rust
let x = 42;                    // inferred as i32
let name = "Rust";              // inferred as &str
let numbers = vec![1, 2, 3];   // inferred as Vec<i32>
```

But sometimes you need to help:

```rust
let parsed: i64 = "42".parse().unwrap();  // Turbofish also works:
let parsed = "42".parse::<i64>().unwrap();
```

---

## Exercises

1. **Ownership drill:** Write a function that takes ownership of a `String`, prints it, 
   and then try to use the original variable after the call. Observe the compiler error.
   Fix it using borrowing.

2. **Mutable borrow:** Write a function `append_world(s: &mut String)` that appends " world" 
   to a string. Call it from `main` and print the result.

3. **Tuple returns:** Write a function `split_name(full: &str) -> (&str, &str)` that splits 
   "owner/repo" into two parts. (Hint: use `.splitn()`)

4. **Shadowing exercise:** Create a variable `x` as a string `"42"`, shadow it by parsing 
   it into an integer, then shadow it again by doubling it. Print the final value.

5. **Connect to the app:** Look at `src/app.rs` lines where `AppState::new()` creates 
   repos from config. Identify:
   - Where ownership is transferred
   - Where references (`&`) are used
   - Where `.clone()` is called and why

---

## Key Takeaways

- Variables are immutable by default — use `mut` when you need to change them
- Ownership ensures memory safety at compile time with zero runtime cost
- Borrowing lets you use values without taking ownership
- `&T` = shared/immutable reference, `&mut T` = exclusive/mutable reference
- The compiler is your teacher — read its error messages carefully

---

**Next:** [Module 02 — Structs, Enums & Pattern Matching →](./02-structs-enums-patterns.md)
