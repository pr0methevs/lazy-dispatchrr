# Module 18 — Fuzzy Search

## Learning Objectives

- Integrate the `fuzzy-matcher` crate for fuzzy matching
- Implement a live search UX with filtered indices
- Handle search activation, typing, confirmation, and cancellation
- Show search state in panel titles

---

## 1. Fuzzy Matching Basics

Fuzzy matching finds strings that approximately match a query, even with typos 
or partial input.

```bash
cargo add fuzzy-matcher
```

```rust
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

let matcher = SkimMatcherV2::default();

// Returns Some(score) if it matches, None if it doesn't
let score = matcher.fuzzy_match("deploy-production.yml", "deploy");
// Some(high_score)

let score = matcher.fuzzy_match("deploy-production.yml", "dprod");
// Some(medium_score) — still matches!

let score = matcher.fuzzy_match("deploy-production.yml", "xyz");
// None — no match
```

## 2. The Filtered Index Pattern (Revisited)

Instead of filtering the data directly, we filter the **indices**:

```rust
// Original data (never modified by search)
self.data.repos = vec![repo_a, repo_b, repo_c, repo_d, repo_e];

// No search → show all indices
self.ui.filtered_repo_indices = vec![0, 1, 2, 3, 4];

// Search "dep" → only matching indices, sorted by score
self.ui.filtered_repo_indices = vec![2, 4];  // repo_c and repo_e matched
```

### Why This Approach?

- **Original data is preserved** — cancel search restores everything
- **Indices map to real data** — `selected_repo_real_index()` always works
- **Score-based ordering** — best matches appear first
- **Works with ListState** — the UI list shows filtered items in order

## 3. The Search Filter Method

```rust
pub fn update_search_filter(&mut self) {
    let matcher = SkimMatcherV2::default();
    let query = &self.ui.search_query;

    match self.ui.focus {
        Focus::Repo => {
            if query.is_empty() {
                // Empty query → show all
                self.ui.filtered_repo_indices = (0..self.data.repos.len()).collect();
            } else {
                // Score each item, keep matches, sort by score
                let mut scored: Vec<(usize, i64)> = self.data.repos
                    .iter()
                    .enumerate()
                    .filter_map(|(i, r)| {
                        matcher.fuzzy_match(&r.name, query)
                            .map(|score| (i, score))
                    })
                    .collect();
                scored.sort_by(|a, b| b.1.cmp(&a.1));  // Descending by score
                self.ui.filtered_repo_indices = scored.into_iter()
                    .map(|(i, _)| i)
                    .collect();
            }
            // Reset selection to first match
            self.ui.repos_state.select(
                if self.ui.filtered_repo_indices.is_empty() { None } else { Some(0) }
            );
        }
        Focus::Branches => { /* same pattern */ }
        Focus::Workflows => { /* same pattern */ }
        _ => {}
    }
}
```

### Breaking It Down

1. **`enumerate()`** — gives `(index, item)` pairs
2. **`filter_map()`** — filters out non-matches, transforms matches to `(index, score)`
3. **`sort_by()`** — best scores first
4. **`.map(|(i, _)| i)`** — discard scores, keep only indices
5. **Reset selection** — put cursor on the first match

## 4. Search Activation and Handling

### Activate Search

```rust
KeyCode::Char('/') => {
    if matches!(state.ui.focus, Focus::Repo | Focus::Branches | Focus::Workflows) {
        state.ui.search_active = true;
        state.ui.search_query.clear();
    }
}
```

### Handle Search Input

```rust
if state.ui.search_active {
    match key.code {
        KeyCode::Esc => {
            // Cancel — restore full lists
            state.reset_search();
        }
        KeyCode::Enter => {
            // Confirm — keep filter, exit search mode
            state.ui.search_active = false;
        }
        KeyCode::Backspace => {
            state.ui.search_query.pop();
            state.update_search_filter();
        }
        KeyCode::Char(c) => {
            state.ui.search_query.push(c);
            state.update_search_filter();
        }
        KeyCode::Up => {
            // Navigate while searching
            select_previous(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len());
        }
        KeyCode::Down => {
            select_next(&mut state.ui.repos_state, state.ui.filtered_repo_indices.len());
        }
        _ => {}
    }
    continue;
}
```

### Cancel Search

```rust
pub fn reset_search(&mut self) {
    self.ui.search_active = false;
    self.ui.search_query.clear();
    // Restore all indices
    self.ui.filtered_repo_indices = (0..self.data.repos.len()).collect();
    self.ui.filtered_branch_indices = (0..self.data.branches.len()).collect();
    self.ui.filtered_workflow_indices = (0..self.data.workflows.len()).collect();
}
```

## 5. Visual Search Feedback

The panel title shows the search state:

```rust
let title = if state.ui.search_active && matches!(state.ui.focus, Focus::Repo) {
    // Active search: show query with cursor
    format!("Repos /{}█", state.ui.search_query)
} else if state.ui.filtered_repo_indices.len() < state.data.repos.len() {
    // Confirmed filter: show count
    format!("Repos [{}/{}]", state.ui.filtered_repo_indices.len(), state.data.repos.len())
} else {
    // No filter: normal title
    "Repos".to_string()
};
```

This gives three visual states:
- **Normal:** `Repos`
- **Searching:** `Repos /dep█` (with cursor)
- **Filtered:** `Repos [3/10]` (filter active but not typing)

## 6. Search UX Flow

```
1. User presses '/' → search_active = true, query = ""
2. User types "dep" → query = "dep", filter updates live
3. User can use ↑/↓ to navigate filtered results
4. User presses Enter → search_active = false, filter stays
5. OR user presses Esc → search cancelled, full list restored
```

## 7. Navigation During Search

While searching, the user can still navigate the filtered list:

```rust
// In search mode, Up/Down navigate filtered results
KeyCode::Up => {
    match state.ui.focus {
        Focus::Repo => select_previous(
            &mut state.ui.repos_state,
            state.ui.filtered_repo_indices.len()  // Uses filtered count!
        ),
        // ...
    }
}
```

---

## Exercises

1. **Fuzzy matching basics:** Use `SkimMatcherV2` to score a list of 10 repo names 
   against a query. Print matches sorted by score.

2. **Filtered indices:** Implement `update_search_filter()` for one list (repos).
   Test with sample data and queries.

3. **Search mode:** Add `/` to activate search mode. Handle typing, backspace, 
   Enter (confirm), and Esc (cancel).

4. **Dynamic titles:** Update panel titles to show search state:
   - Normal: `"Repos"`
   - Searching: `"Repos /que█"`
   - Filtered: `"Repos [3/10]"`

5. **Full integration:** Connect search to navigation — j/k should work on the 
   filtered list while searching.

6. **Connect to the app:** Read `update_search_filter()` in `src/app.rs` and the 
   search handling in `src/event.rs`. Trace a complete search:
   - Press '/'
   - Type "dep"
   - Press Down twice
   - Press Enter
   - What does `selected_repo_real_index()` return?

---

## Key Takeaways

- `fuzzy-matcher` with `SkimMatcherV2` for approximate string matching
- Filter **indices**, not data — preserves original data and enables easy cancel
- Sort by score descending — best matches first
- Live update: every character triggers `update_search_filter()`
- Three states: normal → searching (typing) → filtered (confirmed)
- Panel titles reflect search state for clear UX feedback
- Navigation uses filtered length — respects search results

---

**Next:** [Module 19 — Workflow Dispatch & Replays →](./19-dispatch-and-replays.md)
