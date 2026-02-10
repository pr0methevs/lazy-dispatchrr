# Module 07 — Strings & String Manipulation

## Learning Objectives

- Understand `String` vs `&str` and when to use each
- Master string formatting with `format!`
- Split, join, trim, and transform strings
- Work with `String::from_utf8_lossy` for external data

---

## 1. `String` vs `&str`

| Type | Ownership | Mutability | Where It Lives |
|------|-----------|-----------|---------------|
| `String` | Owned | Mutable | Heap |
| `&str` | Borrowed | Immutable | Anywhere (usually stack/static) |

```rust
let owned: String = String::from("hello");     // Heap-allocated, owned
let borrowed: &str = "hello";                   // String literal, borrowed
let slice: &str = &owned;                       // Borrow from a String
let new_owned: String = borrowed.to_string();   // Convert &str → String
```

### When to Use Which

- **Function parameters:** Use `&str` (accepts both `String` and `&str`)
- **Struct fields:** Use `String` (the struct owns the data)
- **Return values:** Usually `String` (caller gets ownership)

```rust
// Good — accepts both String and &str
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

// Struct fields are owned Strings
pub struct Repo {
    pub name: String,        // Not &str — the struct owns this data
}
```

> **In our app:** Every struct field that stores text uses `String`. 
> Function parameters that just read text use `&str`.

## 2. String Formatting

### `format!` — Returns a String

```rust
let name = "owner/app";
let count = 5;

let msg = format!("Loaded {} branches for '{}'", count, name);
// "Loaded 5 branches for 'owner/app'"

// Multi-line
let msg = format!(
    "Run #{} | status: {} | conclusion: {}",
    run_id, status, conclusion
);
```

### `println!` and `eprintln!`

```rust
println!("To stdout: {}", value);
eprintln!("To stderr: {}", error);
```

### Format Specifiers

```rust
println!("{:?}", my_struct);     // Debug format
println!("{:#?}", my_struct);    // Pretty debug
println!("{:.2}", 3.14159);      // "3.14" (2 decimal places)
println!("{:>10}", "right");     // "     right" (right-align, width 10)
```

> **In our app:** `format!` is used constantly to build:
> - Error messages: `format!("Error adding repo: {}", e)`
> - Display strings: `format!("Loaded {} branches and {} workflows for '{}'", ...)`
> - API paths: `format!("repos/{}/contents/.github/workflows/{}", repo_name, filename)`
> - CLI arguments: `format!("{}={}", field.name, field.value)`

## 3. String Operations

### Splitting

```rust
let repo = "owner/my-app";

// Split into parts
let parts: Vec<&str> = repo.split('/').collect();
// ["owner", "my-app"]

// Split with limit
let parts: Vec<&str> = repo.splitn(2, '/').collect();
// ["owner", "my-app"] — at most 2 parts
```

> **In our app:** Repo names are "owner/name" format, split with `splitn`:
> ```rust
> let parts: Vec<&str> = repo_name.splitn(2, '/').collect();
> let (owner, name) = (parts[0], parts[1]);
> ```

### Joining

```rust
let parts = vec!["workflow", "run", "deploy.yml"];
let joined = parts.join(" ");
// "workflow run deploy.yml"

let inputs = vec!["env=prod", "version=1.0"];
let display = inputs.join(", ");
// "env=prod, version=1.0"
```

> **In our app:** Used for building CLI command previews:
> ```rust
> let preview = format!("gh {}", args.join(" "));
> ```

### Lines

```rust
let text = "line 1\nline 2\nline 3";
let lines: Vec<&str> = text.lines().collect();
// ["line 1", "line 2", "line 3"]

// Truncate to last N lines
let last_200: String = lines[lines.len().saturating_sub(200)..].join("\n");
```

> **In our app:** Workflow logs are truncated to the last 200 lines:
> ```rust
> let lines: Vec<&str> = full_log.lines().collect();
> let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };
> lines[start..].join("\n")
> ```

### Replacing and Trimming

```rust
let messy = "  hello\n  world\n  ";
let clean = messy.trim();              // "hello\n  world"
let no_newlines = messy.replace('\n', "").replace('\r', "");
```

> **In our app:** Base64 content from GitHub has extra newlines:
> ```rust
> let b64_content = String::from_utf8_lossy(&output.stdout)
>     .replace('\n', "")
>     .replace('\r', "");
> ```

### Checking Content

```rust
let s = "feature/auth";
s.is_empty();                    // false
s.starts_with("feature/");      // true
s.ends_with(".yml");             // false
s.contains("auth");              // true
```

## 4. `String` Mutation

```rust
let mut s = String::new();

s.push('H');                     // Append a char
s.push_str("ello");              // Append a string slice
s.pop();                         // Remove last char → Option<char>

s.clear();                       // Empty the string
```

> **In our app:** User input in popups uses `push` and `pop`:
> ```rust
> KeyCode::Char(c) => {
>     state.ui.add_repo_owner.push(c);
> }
> KeyCode::Backspace => {
>     state.ui.add_repo_owner.pop();
> }
> ```

## 5. Converting External Data

### `String::from_utf8_lossy`

When you get bytes from an external process, they might not be valid UTF-8:

```rust
let output = std::process::Command::new("gh")
    .args(["repo", "list"])
    .output()?;

// Lossy: replaces invalid UTF-8 with the replacement character (�)
let stdout = String::from_utf8_lossy(&output.stdout);
let stderr = String::from_utf8_lossy(&output.stderr);
```

> **In our app:** Every `Command` output is converted with `from_utf8_lossy`:
> ```rust
> let stderr = String::from_utf8_lossy(&output.stderr);
> return Err(format!("gh cli error: {}", stderr.trim()).into());
> ```

### `.to_string()` vs `String::from()` vs `.into()`

```rust
// All equivalent for creating a String from &str:
let s1 = "hello".to_string();
let s2 = String::from("hello");
let s3: String = "hello".into();
```

## 6. Repeat Patterns

```rust
let separator = "─".repeat(60);
// "────────────────────────────────────────────────────────────"
```

> **In our app:** Used in the log output display:
> ```rust
> "─".repeat(60)
> ```

---

## Exercises

1. **String building:** Write a function `build_command_preview(repo: &str, branch: &str, workflow: &str, inputs: &[(String, String)]) -> String` 
   that builds a string like: `"gh workflow run deploy.yml --repo owner/app --ref main -f env=prod -f version=1.0"`

2. **Split and validate:** Write `parse_repo_name(full: &str) -> Result<(&str, &str), String>` 
   that splits "owner/name" and returns an error if the format is wrong.

3. **Truncate logs:** Write a function `truncate_log(log: &str, max_lines: usize) -> String` 
   that keeps only the last `max_lines` lines.

4. **User input simulation:** Create a `String`, simulate typing by pushing chars one at a time, 
   then simulate backspace by popping. Print after each operation.

5. **Connect to the app:** Find every `format!()` call in `src/app.rs`. Categorize them as:
   - Error messages
   - Success messages
   - API/CLI arguments
   - Display strings

---

## Key Takeaways

- Use `&str` for function params, `String` for owned data in structs
- `format!()` is your main tool for building strings
- `splitn()` for parsing delimited strings (like "owner/repo")
- `.join()` for combining vectors back into strings
- `String::from_utf8_lossy()` for external process output
- `.push()` / `.pop()` for character-by-character input handling

---

**Next:** [Module 08 — Serialization with Serde →](./08-serde-serialization.md)
