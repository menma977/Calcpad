# Calcpad

I built Calcpad because I wanted a calculator notepad that lives in the terminal — a plain text file where every line I type is evaluated as I type it, with the result shown in a column on the right. It is written in Rust and similar in spirit to Soulver or the Windows Calculator notebook mode.

Everything you write lives in a plain-text `.cpad` file, so a calculation document is just a text file you can keep, version, and reopen.

```
  No │ Code                              │ Result
──────┼───────────────────────────────────┼──────────────────
   1  │ harga_satuan = 1000               │      = 1.000
   2  │ jumlah_unit  = 15                 │      = 15
   3  │                                   │
   4  │ subtotal = harga_satuan * jumlah  │      = 15.000
   5  │ pajak    = subtotal * 10 / 100    │
   6  │ total    = subtotal + pajak       │      = 16.500
──────┴───────────────────────────────────┴──────────────────
 nota.cpad  |  Line 6:5  |  Esc/Ctrl+C to quit  |  Ctrl+S to save
```

Results update **as you type**: change `jumlah_unit` to `20` and every value below it recalculates, including anything that referenced earlier variables.

> A personal tool, published as-is in case it is useful to someone else —
> not a product, no roadmap, no guaranteed support. See
> [Contributing](#contributing) for forks and pull requests.

---

## Features

| Feature | Description |
|---|---|
| Live evaluation | The whole document is re-evaluated on every keypress; results appear per line. |
| Variables | Assign with `name = expression`; variables are stored as `f64` and reused in later lines. |
| Expression language | Arithmetic, comparison, boolean logic, bitwise ops, and a ternary conditional. |
| `if` / `else` blocks | Multi-line, nestable conditional blocks with `{ ... }` braces. |
| Multiple statements per line | Separate statements with `;`. |
| Fuzzy autocomplete | Suggests previously assigned variable names while you type, with fuzzy scoring. |
| Syntax highlighting | Assignments, numbers, operators, brackets, keywords and variables are colorized. |
| Copy result to clipboard | Click any value in the **Result** column to copy the clean number to the clipboard. |
| Mouse support | Scroll vertically, click to place the cursor, click results to copy. |
| `.cpad` plain-text format | Documents are ordinary UTF-8 text, one statement per line. |

---

## Getting started

### Prerequisites

- Rust 1.85+ (stable) — the crate uses `edition = "2024"`; install via [rustup](https://rustup.rs) if needed.
- A terminal with mouse + alternate-screen support (Linux / macOS / Windows terminals supported through `crossterm`).

### Install

If you want to use it, build it from source — there are no prebuilt binaries:

```bash
git clone https://github.com/menma977/Calcpad.git
cd Calcpad
cargo install --path .
```

The binary ends up in `~/.cargo/bin/calcpad`, runnable from anywhere:

```bash
calcpad                      # fresh empty buffer
calcpad nota                 # opens nota.cpad; .cpad appended if missing
calcpad ~/docs/hitung.cpad   # any path
```

If the file does not exist yet, Calcpad starts with an empty buffer prefilled with that path, so the first `Ctrl+S` saves straight to it.

Without installing, you can also run it directly from the repo:

```bash
cargo run --release -- nota
```

### Exiting

- `Esc` or `Ctrl+C` — quit (the panic hook and loop exit always restore raw mode, the alternate screen and mouse capture).

---

## The script language

A `.cpad` document is a sequence of statements, one per line. There are two statement kinds: **assignments** and **bare expressions**. Execution is strictly sequential from top to bottom — a variable defined on line N is usable from line N onward, and everything is re-computed on every edit, like a spreadsheet without cells.

### Statements

```text
total = 1.000,00            ← assignment: name = expression (value stored)      ! not a valid literal, see Numbers
total = subtotal * 1.1      ← assignment
total                       ← bare expression: evaluated and shown in Result
x = 5; y = x * 2; y         ← several statements on one line, `;`-separated
```

*Whitespace* is insignificant; variable lookup is done on whole-word boundaries (letters, digits and `_` count as word characters), so `total2` never matches the variable `total`.

Variable names may in practice contain any characters that appear before the `=`, even spaces (see [Limitations](#limitations)).

### Operators

Binary operators, listed **from lowest to highest precedence** (matching the parser's precedence groups in `src/services/expression_service.rs:50`):

| Precedence | Operators | Kind |
|---|---|---|
| 1 (lowest) | `\|\|` | logical OR |
| 2 | `&&` | logical AND |
| 3 | `\|` | bitwise OR |
| 4 | `^` | bitwise XOR |
| 5 | `&` | bitwise AND |
| 6 | `==` `!=` | equality comparison |
| 7 | `>=` `<=` `>` `<` | ordering comparison |
| 8 | `<<` `>>` | bit shifts |
| 9 (highest) | `+` `-` `*` `/` `%` | arithmetic |

Additional syntax constructs (evaluated after variable substitution, before the table above):

* `( ... )` — grouping parentheses (nestable).
* `condition ? thenValue : elseValue` — ternary, checked before the binary operators above.

Semantics:

* All values are `f64`. Logic/comparison operators return `1.0` / `0.0`.
* Equality uses an epsilon comparison: `|a - b| < f64::EPSILON` — comparing decimal results does not suffer from binary round-off noise.
* **Truthiness** (for `? :` and `if`): a value is *true* when `|value| ≥ f64::EPSILON` (`0` and values indistinguishable from `0` are false).
* Bitwise operators and shifts convert their operands to `i64` with explicit range checks: operands must be finite and inside `i64`; shift amounts must be `0..=63`. Shifts use wrapping (`wrapping_shl` / `wrapping_shr`).
* Division and modulo by zero are caught and produce an error instead of crashing.

### Conditional blocks — `if` / `else`

Multi-line blocks, nestable, brace-delimited. The condition is an ordinary expression (truthiness rule above); if it cannot be parsed it is treated as *false*.

```text
stok_barang = 15
batas_minimum = 20

if (stok_barang < batas_minimum) {
    status_gudang = 0;
    jumlah_pesanan = batas_minimum - stok_barang
} else {
    status_gudang = 1;
    jumlah_pesanan = 0
}

total_biaya = jumlah_pesanan * 50
```

* One-liners are allowed: `if (cond) { a = 1; b = 2 } else { a = 3; b = 4 }`.
* `else { ... }` may also appear on the line *after* the closing `}` of the `if` block.
* Variables assigned inside a block persist into the rest of the document (there is **no block scope**) — as long as only one branch runs, the variable behaves like normal. A branch that did *not* run does not create the variable.
* Comments (`//`) and string literals (`"..."`) are honored while scanning braces, so braces or `?` inside a comment or literal do not confuse the parser.

### Comments and decorative lines

* A line starting with `//` (after trimming) is a comment: it is neither executed nor displayed in the Result column.
* A line consisting only of `=`, `-`, `_` and `*` characters (e.g. `===================`) is treated as decoration and produces no result. This lets you draw section separators.

### Numbers and formatting

The **Result** column and the status rendering use a European-style number format:

* Thousands are separated with `.` — e.g. `1.234.567`
* Decimals are separated with `,`, **2 digits** when the fractional part is significant — e.g. `3 / 4` → `0,75`
* Values whose fractional part is negligible (below `1e-9`) print as integers.
* Values above `u64::MAX` fall back to scientific notation with 6 decimals (e.g. `1.221632e17` style), since the grouped integer path only supports `u64` magnitude.

Internally (state, substitution) numbers keep the regular Rust `f64` `to_string()` form (`1000000`, `0.75`), so expressions are always written in canonical form — results are formatted only for display.

### Error reporting

Errors appear inline in the Result column (in red), never abort the document:

* `error: cannot parse: <text>` — syntax could not be split into numbers, variables and operators.
* `error: division by zero` / `error: modulo by zero`
* `error: string values are not supported` — assigning `"text"` to a variable (strings exist only for the block scanner to skip over).
* `error: value 1e30 is out of range for bitwise operation ...` / `error: shift amount ... out of range (must be 0..=63)`
* `error: variable 'x' has non-finite value (...)` — prevented state corruption from `inf`/`NaN`.

Because evaluation is per-statement, a failing line does not stop the rest of the document.

### Examples

Simple payroll (from `bebek.cpad`):

```text
hours_worked = 45
base_pay = 20

regular_total = (hours_worked <= 40 ? hours_worked : 40) * base_pay
overtime_total = (hours_worked > 40) * (hours_worked - 40) * (base_pay * 1.5)
final_pay = regular_total + overtime_total
```

Inventory with if/else (from `testing.cpad`):

```text
stok_barang = 50
batas_minimum = 100

if (stok_barang < batas_minimum) {
    status_gudang = 0
    jumlah_pesanan = batas_minimum - stok_barang
} else {
    status_gudang = 1
    jumlah_pesanan = 0
}

total_pendapatan = total_penjualan + komisi_nilai
```

---

## Using the editor

### Keyboard

| Key | Action |
|---|---|
| `Esc`, `Ctrl+C` | Quit |
| `Ctrl+S` | Save (opens the *Save as:* dialog if no file is set yet) |
| `←` `→` `↑` `↓` | Move cursor (wraps from line end to next line start and back) |
| `Home` / `End` | Jump to start / end of the current line |
| `PageUp` / `PageDown` | Move the cursor a terminal page (minus 3 rows of chrome) |
| `Enter` | Insert a new empty line below the current one and move there |
| `Backspace` | Delete the character before the cursor — at the start of a line it merges the line into the previous one |
| `Delete` | Delete the character under the cursor |
| `Tab` / `Enter` (when autocomplete is open) | Accept the selected suggestion |
| `↑` / `↓` (when autocomplete is open) | Move the selection (wraps around) |

Any character insert, deletion or autocomplete confirm triggers a full re-evaluation of the document.

*Save dialog*: type a name, `Enter` to save (`.cpad` appended if missing), `Esc` to cancel. An empty name aborts with a status message.

Status messages (`Saved successfully!`, `Result copied to clipboard!`, save errors, …) are shown in the bottom bar and auto-clear after 3 seconds.

### Mouse

| Action | Behavior |
|---|---|
| Scroll wheel up/down | Scroll the document vertically |
| Left click in the Code panel | Move the (line, column) cursor there |
| Left click in the Result panel | Copy that line's result to the clipboard |

Copied results are *cleaned up for computation*: the display formatting (`.` thousands, `,` decimal) is reversed back into a plain number before it hits the clipboard, so pasting `1.234,56` gives you `1234.56`. Results in scientific notation are copied verbatim. Empty and error results are not copied.

The cursor is kept visible via automatic vertical and horizontal scrolling (a scroll controller tracks the terminal size and the fixed result-column width, 25 % of the window).

---

## Architecture

Calcpad follows an MVC-flavoured layout with services and a repository:

```
main.rs
  └─ controllers/app_controller.rs   terminal setup, panic hook, main loop
       │
       ├─ models/app.rs              App state: lines, results, cursor, scroll,
       │    │                        mode (Editing | SavePrompt), autocomplete,
       │    │                        status message/timer; app_actions.rs = cursor
       │    │                        & text mutations (insert/backspace/delete/…)
       │
       ├─ controllers/
       │    ├─ event_handler.rs      dispatch keyboard vs. mouse events,
       │    │                        Ctrl+C / Esc handling, clipboard copy
       │    ├─ keyboard/             editing_keys (typing, save, nav),
       │    │                        autocomplete_keys, save_prompt_keys,
       │    │                        cursor_keys
       │    └─ scroll_controller.rs  vertical/horizontal scrolling, panel metrics
       │
       ├─ views/
       │    ├─ app_view.rs           3-column layout (No | Code | Result) +
       │    │                        status bar + save dialog + autocomplete popup
       │    └─ thames/               color palette (#d6719e pink, #61afef blue,
       │                             #1e222a background)
       │
       ├─ services/
       │    ├─ calculator_service.rs orchestrator: documents → statements → results,
       │    │                        assignment handling and number formatting
       │    ├─ expression_service.rs recursive expression parser + operator table
       │    ├─ state_service.rs      variable store (HashMap<String, f64>) and
       │    │                        word-boundary-safe variable substitution
       │    └─ syntax_service.rs     syntax highlighting spans for the Code column
       │
       ├─ parsers/block_parser.rs    lines → Statement tree (Line | IfBlock),
       │    │                        brace/string-aware block extraction
       │    │
       └─ repositories/file_manager_repository.rs   load/save plain-text files,
                                                    `.cpad` name normalization
```

### Evaluation pipeline

`CalculatorService::evaluate_document` (src/services/calculator_service.rs:24) is the single entry point — it is invoked on boot and after every edit:

1. **Reset** the variable store (`StateService::clear`).
2. **Parse** the raw lines with `BlockParser::parse` into a `Vec<Statement>`. Empty lines are skipped; `if` lines produce a nested `IfBlock{ condition, true_statements, false_statements }`, everything else becomes a `Line{ index, content }`. Multiple `;`-separated statements on one line each become a `Line` pointing at the same `index`.
3. **Execute** statements in order:
   * `Line` — evaluated to a `String` result that is written into `results[index]` (keeping that line's own slot; empty results leave the slot untouched so a preceding `;`-split statement survives).
   * `IfBlock` — the condition is evaluated (errors → `0.0`), and exactly one branch runs recursively as a nested statement list. Both branches live in the main statement tree, results keep flowing to their original line indices regardless of which branch executes.
4. **Render** — the view pulls `app.results` and colorizes errors in red, values in the primary color, and strips nothing: results for decoration/comment/blank lines stay empty.

Assignment detection happens per `Line`: text up to the first `=` is taken as the variable name (rejected if empty or if it ends with `<`, `>`, `!`, i.e. the `=` is really part of a comparison such as `<=`); the RHS is evaluated and stored in the state. Assignment *and* bare expressions share the same evaluator, so `x = 2 + 3` returns `5` too — the value shows up in the Result column and is stored.

Expression evaluation (src/services/expression_service.rs) proceeds bottom-up:

1. `StateService::replace_variables` substitutes every known variable name with its numeric value, longest name first, only on whole-word boundaries — later variables never clobber the substitution of longer ones (`total2` vs `total`) and identifiers spanning other words are safe.
2. Balanced outer parentheses are unwrapped; plain `f64` literals are parsed directly.
3. Ternary is tried, then the operator precedence groups, scanning each expression from the rightmost operator of the lowest available group and recursing into both halves.
4. Operators are applied with well-defined error cases (division by zero, out-of-range bitwise/shift operands) returned as `Result<f64, String>`.

### Rendering and input

* `app_controller::run` starts the alternate screen, enables raw mode and mouse capture, installs a panic hook that restores the terminal, then loops: `update_scroll` → `app_view::render` → `event::read` → `handle_event`.
* The three panels scroll together vertically; the Code panel additionally scrolls horizontally. Line numbers, Code and Result are aligned by row.
* Autocomplete pops up at the cursor in the Code panel (and flips above it when there is no room below), with its own list state.
* Clipboard access goes through `arboard` (with the `wayland-data-control` feature enabled therefore Wayland-native and X11 fallback both work).

---

## File format

* `.cpad` files are plain UTF-8 text, lines joined with `\n`.
* The CLI argument and the save dialog both auto-append the `.cpad` extension when missing (`normalize_cpad_path`).
* A loaded document is stored as `Vec<String>` (one entry per line) and saved as `lines.join("\n")` — no metadata, no escaping, fully round-trip-safe.

---

## Dependencies

| Crate | Purpose |
|---|---|
| [`ratatui`](https://crates.io/crates/ratatui) 0.30 | Terminal UI widgets, layout, styling |
| [`crossterm`](https://crates.io/crates/crossterm) 0.29 | Raw mode, alternate screen, key/mouse events |
| [`arboard`](https://crates.io/crates/arboard) 3.6 | Cross-platform clipboard (incl. Wayland data control) |
| [`strum`](https://crates.io/crates/strum) 0.26 | Operator enum ↔ symbol string mapping |

---

## Project status and limitations

This project is published in exactly the state I use it myself, so expect the
rough edges of a personal tool:

* **Numbers only** — the language has no strings (assignments with string literals produce `error: string values are not supported`), no lists, no functions (`sin`, `sqrt`, … are not implemented).
* **Variable names are lax** — because an assignment is "text before the first `=`", names like `duit gw` or `kuda lumping` are accepted; references still work as long as the whole name is used contiguously (word-boundary substitution). Stick to `[A-Za-z0-9_]+` names for sanity.
* **No scope, no undo, no search, no copy of code text** — only results can be copied; edits are not undoable; variables live in one flat document-wide map.
* **No persistent configuration** — no theme or key bindings config; colors are compile-time constants in `src/views/thames/color.rs`.
* **Re-evaluation is O(document)** on every keypress (`evaluate_document` clears state and recompiles everything). Completely fine for notebook-sized files; not tuned for huge documents.
* **Ternary and `if` conditions are always re-evaluated eagerly** — both branches of a `? :` ternary are *parsed*, but only the selected branch is *computed*.
* Error messages are user-facing strings, not structured diagnostics; the Result column shows only the first error of a line.

## Contributing

Calcpad is maintained primarily for my own use. That said:

- **Forks** are always welcome — take it and adapt it (MIT license).
- **Pull requests** are accepted, with one rule: create a **new branch** for
  your work first (`feature/whatever` or `fix/whatever`), keep the changes
  small and focused, and explain in the PR why you need the change.
- Bug reports via issues are appreciated; include a minimal `.cpad` snippet
  that reproduces the problem.
- Merging is at the author's discretion; nothing here promises support.

## License

MIT — see [LICENSE](LICENSE).
