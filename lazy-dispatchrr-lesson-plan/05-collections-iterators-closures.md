# Module 05 — Collections, Iterators & Closures

## Learning Objectives

- Work with `Vec`, `HashMap`, and their common operations
- Use iterators and their adaptors (`.map()`, `.filter()`, `.collect()`)
- Write and use closures
- Understand iterator chains — a core Rust idiom

---

## 1. `Vec<T>` — Dynamic Arrays

Vectors are the most common collection. You'll use them everywhere.

```rust
let mut repos: Vec<String> = Vec::new();
repos.push("owner/app".to_string());
repos.push("owner/lib".to_string());

// Shorthand with vec! macro
let repos = vec!["owner/app".to_string(), "owner/lib".to_string()];

// Accessing
let first = &repos[0];           // Panics if out of bounds
let maybe = repos.get(0);        // Returns Option<&String>

// Length and emptiness
repos.len();
repos.is_empty();
```

> **In our app:** Everything is stored in `Vec`:
> - `repos: Vec<Repo>`
> - `branches: Vec<String>`
> - `workflows: Vec<Workflow>`
> - `input_fields: Vec<InputField>`
> - `filtered_repo_indices: Vec<usize>`

### Common Vec Operations

```rust
let mut v = vec![1, 2, 3, 4, 5];

v.push(6);                  // Add to end
v.pop();                    // Remove from end → Option<T>
v.remove(1);                // Remove at index
v.insert(0, 0);             // Insert at index
v.contains(&3);             // Check membership
v.iter();                   // Immutable iterator
v.iter_mut();               // Mutable iterator
v.into_iter();              // Consuming iterator (takes ownership)
```

## 2. Iterators

Iterators are Rust's way of processing sequences. They're **lazy** — nothing happens 
until you consume them.

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Manual iteration
for n in &numbers {
    println!("{}", n);
}

// Iterator chain
let doubled: Vec<i32> = numbers.iter()
    .map(|n| n * 2)
    .collect();
// [2, 4, 6, 8, 10]
```

### Key Iterator Adaptors

```rust
let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// map — transform each element
let squares: Vec<i32> = data.iter().map(|x| x * x).collect();

// filter — keep elements matching a predicate
let evens: Vec<&i32> = data.iter().filter(|x| *x % 2 == 0).collect();

// filter_map — filter and transform in one step
let parsed: Vec<i32> = vec!["1", "two", "3"]
    .iter()
    .filter_map(|s| s.parse::<i32>().ok())
    .collect();
// [1, 3]

// enumerate — add index
for (i, val) in data.iter().enumerate() {
    println!("[{}] = {}", i, val);
}

// find — first match
let first_even = data.iter().find(|x| *x % 2 == 0);  // Some(&2)

// position — index of first match
let pos = data.iter().position(|x| *x == 5);  // Some(4)

// any / all
let has_negative = data.iter().any(|x| *x < 0);  // false

// collect into different types
let as_strings: Vec<String> = data.iter().map(|n| n.to_string()).collect();

// join (on strings)
let csv = as_strings.join(", ");  // "1, 2, 3, 4, 5, 6, 7, 8, 9, 10"
```

> **In our app:** Iterator chains are used extensively. Here's a real example
> from `app.rs` that converts config repos into domain repos:
> ```rust
> let repos: Vec<Repo> = config.repos
>     .iter()
>     .map(|rc| Repo {
>         name: rc.name.clone(),
>         branches: vec![],
>         workflows: vec![],
>     })
>     .collect();
> ```

## 3. Closures

Closures are anonymous functions that can capture variables from their environment:

```rust
let multiplier = 3;
let multiply = |x: i32| x * multiplier;  // Captures `multiplier`
println!("{}", multiply(5));  // 15
```

### Closure Syntax

```rust
|params| expression                    // Single expression
|params| { multiple; statements; }     // Block body

// Type annotations (usually inferred)
let add = |a: i32, b: i32| -> i32 { a + b };
```

### Closures with Iterators

This is where closures shine — they're the "function" you pass to `.map()`, `.filter()`, etc.:

```rust
let names = vec!["Alice", "Bob", "Charlie"];

// Closure with filter
let long_names: Vec<&&str> = names.iter()
    .filter(|name| name.len() > 3)
    .collect();

// Closure with map
let upper: Vec<String> = names.iter()
    .map(|name| name.to_uppercase())
    .collect();
```

> **In our app:** Closures appear in almost every iterator chain:
> ```rust
> // From github.rs — extracting branch names from JSON
> let branches: Vec<String> = repository["refs"]["nodes"]
>     .as_array()
>     .map(|nodes| {
>         nodes
>             .iter()
>             .filter_map(|n| n["name"].as_str().map(String::from))
>             .collect()
>     })
>     .unwrap_or_default();
> ```

## 4. Sorting

```rust
let mut scores: Vec<(usize, i64)> = vec![(0, 50), (1, 30), (2, 80)];

// Sort by the second element, descending
scores.sort_by(|a, b| b.1.cmp(&a.1));
// [(2, 80), (0, 50), (1, 30)]
```

> **In our app:** Fuzzy search results are sorted by score:
> ```rust
> scored.sort_by(|a, b| b.1.cmp(&a.1));
> ```

## 5. Ranges and Index Collections

```rust
// Range as an iterator
let indices: Vec<usize> = (0..5).collect();  // [0, 1, 2, 3, 4]

// Useful for creating index maps
let repos_len = 10;
let all_indices: Vec<usize> = (0..repos_len).collect();
```

> **In our app:** Filtered index lists are initialized from ranges:
> ```rust
> let filtered_repo_indices: Vec<usize> = (0..repos.len()).collect();
> ```

## 6. Chaining Complex Operations

Real-world example from our app — building display strings from input fields:

```rust
let inputs_display = self.data.input_fields
    .iter()
    .map(|f| format!("  {} = {}", f.name, f.value))
    .collect::<Vec<_>>()
    .join("\n");
```

Breaking this down:
1. `.iter()` — iterate over input fields
2. `.map(|f| ...)` — transform each field into a formatted string
3. `.collect::<Vec<_>>()` — collect into a `Vec<String>` (turbofish for type hint)
4. `.join("\n")` — join all strings with newlines

## 7. `unwrap_or_default()` and Fallback Patterns

```rust
let maybe_vec: Option<Vec<String>> = None;
let vec = maybe_vec.unwrap_or_default();  // Empty Vec

let maybe_str: Option<&str> = None;
let s = maybe_str.unwrap_or("fallback");
```

> **In our app:** Used when parsing JSON that might be missing fields:
> ```rust
> let branches: Vec<String> = repository["refs"]["nodes"]
>     .as_array()
>     .map(|nodes| { /* ... */ })
>     .unwrap_or_default();  // Empty vec if the field is missing
> ```

---

## Exercises

1. **Vec basics:** Create a `Vec<String>` of repo names. Write functions to:
   - Add a repo
   - Remove a repo by name
   - Find a repo by name (return `Option<&String>`)
   - List all repos (print them)

2. **Iterator chains:** Given a `Vec<String>` of branch names like 
   `["main", "dev", "feature/auth", "feature/ui", "bugfix/crash"]`:
   - Filter to only branches starting with "feature/"
   - Map them to remove the "feature/" prefix
   - Collect into a new `Vec<String>`

3. **Enumerate + filter_map:** Given a `Vec<Repo>` where each repo has a `name`,
   write a function that returns `Vec<usize>` — the indices of repos whose names 
   contain a search query. (This is the core of the filtered indices pattern.)

4. **Sort by score:** Create a `Vec<(usize, i64)>` of (index, score) pairs. Sort them 
   by score descending, then extract just the indices into a `Vec<usize>`.

5. **Connect to the app:** Find 5 different iterator chains in `src/app.rs` and 
   `src/ui.rs`. For each one, write a comment explaining what it does step by step.

---

## Key Takeaways

- `Vec<T>` is Rust's dynamic array — used for almost every collection need
- Iterators are lazy chains: `.iter().map().filter().collect()`
- Closures capture their environment — `|params| body`
- `filter_map` combines filtering and transforming in one step
- `.collect()` transforms an iterator into a collection (needs a type hint)
- `.join()` is great for building display strings from vectors

---

**Next:** [Module 06 — Traits & Implementations →](./06-traits-and-implementations.md)
