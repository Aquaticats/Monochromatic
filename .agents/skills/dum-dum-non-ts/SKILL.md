---
name: dum-dum-non-ts
description: Use when writing or editing source files in non-TypeScript general-purpose languages.
---

# Dum-dum non-TS

Fires when writing or editing source files in non-TypeScript general-purpose
programming languages:
 C,
 C++,
 Rust,
 Go,
 Python,
 Java,
 Kotlin,
 Swift,
 Ruby,
PHP,
 Lua,
 Zig,
 Nim,
 Haskell,
 OCaml,
 Elixir,
 Clojure,
 Scala,
 F#,
 or any other.

The skill encodes two rules (simplest possible constructs;
 comments on every
concept-introducing line) and a comment template (What,
 Why,
 optional Gotcha)
with a TypeScript pseudocode equivalent above every concept-introducing line.

See "Out of scope" at the bottom for file types this skill does not apply to.

## Audience

The human reader knows **only TypeScript**.
 Optimise comments and code shape
for them,
 not for an experienced practitioner of the target language.
 The
compiler does not read the comments;
 the user does.

## Two rules

### Rule 1 — Simplest possible constructs

Write boring,
 beginner-level code.
 Forbidden by default:

- macros,
   metaprogramming,
   reflection,
   code generation
- operator overloading,
   custom conversion operators
- template / generic gymnastics beyond a single type parameter
- point-free chains,
   monad stacks,
   `do`-notation
- decorators,
   mixins,
   multiple inheritance
- comprehensions / generators when their syntax differs from TS
- pattern matching used for control flow that an `if`/`else` chain handles
- "clever" one-liners,
   ternaries longer than `cond ? a : b`

Prefer:
 plain named functions,
 named local variables,
 explicit type
annotations where the language allows them,
 `if`/`else`,
 `for` loops,
 early
`return`s,
 one statement per line.
 **If two ways exist,
 pick the one whose
translation to TS is mechanical.
**

If the task genuinely needs a construct with no clean TS analogue (manual
memory management,
 lifetimes,
 async runtimes,
 FFI,
 channels),
 stop and ask
the user whether to (a) proceed with a verbose comment that admits the
analogy is loose,
 (b) wrap the gnarly bit in a tiny helper and only use the
helper from then on,
 or (c) pick a different language for this piece.

### Rule 2 — Comments on every concept-introducing line

A line is **concept-introducing** if it is any of:

- an import / `#include` / `use` / `require`
- a function,
   method,
   type,
   struct,
   class,
   trait,
   interface,
   or enum
  declaration
- a **wrapper or variant constructor**:
   `Some(...)`,
   `None`,
   `Ok(...)`,
  `Err(...)`,
   `Box::new(...)`,
   `Rc::new(...)`,
   `Arc::new(...)`,
  `Cell::new(...)`,
   `RefCell::new(...)`,
   `Vec::new()`,
   `String::new()`,
  Haskell's `Just x` / `Nothing`,
   Python's `Optional[X]` annotation,
   etc.
  These exist precisely because the language lacks `null` / `throw`.
   Every
  appearance is concept-introducing for a TS reader,
   **including bare
  one-token tails like `Ok(rules)` or `None`**,
   so every appearance carries a
  comment.
   How long that comment is depends on whether this is the concept's
  first appearance in the file;
   see "Full block on a concept's first
  occurrence in each file".
- a **type-conversion or wrapper-unwrap method call**:
   `.to_string()`,
  `.to_owned()`,
   `.into()`,
   `.from(...)`,
   `.clone()`,
   `.as_str()`,
  `.as_bytes()`,
   `.unwrap()`,
   `.unwrap_or(...)`,
   `.unwrap_or_else(...)`,
  `.unwrap_or_default()`,
   `.expect(...)`,
   `.ok()`,
   `.ok_or(...)`,
  `.collect()`,
   `.into_iter()`,
   the `?` propagation operator.
   (Same idea
  in other languages:
   Python's `dict.get(k, default)`,
   Go's `, ok` comma-ok
  pattern,
   Kotlin's `?:` Elvis,
   Swift's `as!` / `as?`.
  )
- a **borrow / reference / deref expression**:
   `&x`,
   `&mut x`,
   `*x`,
  Rust lifetimes `'a`,
   C/C++ pointer arithmetic,
   Go's `&v` / `*v`.
   Even
  if the line looks like a plain function call,
   **every `&` / `&mut`
  argument is concept-introducing** because it states ownership intent
  ("I am only lending this to you").
- a **closure / lambda** with non-TS-arrow syntax:
   Rust `|x| ...` /
  `|&x| ...`,
   Ruby `do |x| ... end`,
   Haskell `\x -> ...`,
   etc.
- a **`match` / pattern-match expression** used to extract from
  `Option` / `Result` / sum types (`match x { Some(v) => ..., None => ... }`,
  `match r { Ok(v) => ..., Err(e) => ... }`).
   Even one-arm `if let
  Some(v) = x` counts.
- an **implicit-return tail expression**:
   a line at the end of a function
  body whose value becomes the return because there's no trailing `;`.
  Examples:
   `Ok(rules)`,
   `Some(escape_literal(trimmed))`,
   `(line, col)`,
  `if cond { a } else { b }`.
   The reader has to know **both** that this is
  a return AND understand the value being constructed.
   Mark it with the
  same comment block as any other concept-introducing line.
- a **statement inside a function body** that uses any language feature
  without a 1:1 TS analogue (pointers,
   lifetimes,
   channels,
   `defer`,
  `with`,
   list comprehensions,
   destructors,
   ownership moves,
   `await` on a
  non-Promise type,
   range-indexing like `s[1..n]`,
   byte literals `b'/'`,
  type-annotated `let` bindings like `let x: Vec<T> = ...`,
   `let _ = ...`
  discard,
   etc.)
- a line containing punctuation a TS dev would not recognise:
   `<<`,
   `::`,
  `&`,
   `*`,
   `?`,
   `!`,
   `<...>` template args,
   `mut`,
   `'a` lifetimes,
  `->` return type arrows in non-TS positions,
   etc.

Lines that **do not** need a comment:
 closing braces,
 blank lines,
 plain
arithmetic whose TS equivalent is character-identical (`x + 1`),
 and
re-assignments to an already-commented variable using only TS-native
operators.

**Full block on a concept's first occurrence in each file.
** The first time a concept appears in a file (`?`,
 `Some(...)`,
`.to_string()`,
 an `&` argument,
 `match r { Ok(_) => ..., Err(_) => ... }`,
…),
 give it the entire What / Why / TypeScript-pseudocode block.

**Later occurrences of that same concept in that same file take one short
line.
** Say what this particular line does and move on.
 Do not restate the
concept itself:
 the reader met it above,
 in this same file.

The scope is **per file**,
 not per package,
 not per session,
 and not per
crate.
 A new file starts over at full density,
 because a reader may open it
first and has no obligation to have read any sibling.

Two concepts are **the same** when the explanation would be the same sentence.
`Ok(...)` and `Err(...)` are one concept.
 `.to_string()` and `.clone()` are
two,
 because one converts a type and the other duplicates ownership.
 When
in doubt,
 treat them as different and write the full block.

What this trades away is repetition of the *explanation*,
 not clarity,
 which
is still the deliverable.
 If a file becomes unreadable from the comment
density,
 **make the file smaller**,
 do not thin out the first-occurrence
blocks.

## Comment template

Every concept-introducing line gets a comment block above it,
 written in
whichever of the language's comment forms fits the line's role.
 Do not assume a
plain line comment:

- A **documentable declaration** (function,
   type,
   field,
   variant,
   module,
  constant,
   import,
   and so on) in a language that has **doc comments** takes the
  block AS a doc comment,
   so the one block is also the item's API documentation:
  in Rust,
   `///` above the item and `//!` at the top of the file;
   the equivalent
  in other languages.
   Write the block once,
   in the doc-comment form.
   Never leave
  it as a plain comment and bolt a separate one-line summary on top,
   and never
  keep both a plain block and a doc-comment summary for the same item.
- **Everything else** (statements and expressions inside a body,
   and languages
  with no doc comments) takes an ordinary comment:
   `//` in C / C++ / Rust / Go /
  Java / Swift / Kotlin,
   `#` in Python / Ruby / Elixir,
   `--` in Haskell / Lua /
  SQL,
   `;` in Lisp / Clojure.
   Where a doc comment on a non-item is an error
  (Rust `///` on a statement),
   this ordinary comment is the only legal choice.

The block's fields are the same whichever comment form carries it:

- **What:
  ** name the construct in plain English.
   No jargon.
   If jargon is
  unavoidable (e.g. "preprocessor",
   "namespace"),
   define it in the same
  sentence.
   Two or three lines is fine — exhaustively name every symbol on
  the line and what each one means.
  - **When the line introduces a type from a family of siblings,
    ** the
    What field must explicitly **list the siblings the reader might have
    expected**.
     Examples:
     `usize` → name `u32` / `u64` / `i32` / `i64` as
    siblings;
     `String` → name `&str` as the sibling;
     `Vec<T>` → name
    `&[T]` and `[T; N]`;
     `Box<T>` → name `Rc<T>` and `Arc<T>`;
     `i32` →
    name `u32` / `i64`.
     Just naming the type is not enough;
     the dum-dum
    reader has no way to know what alternatives even exist.
- **Why:
  ** one sentence on what the **program** gains from this line.
   Not
  "declares X" — "we need this so the next line can do Y".
  - **When the line introduces a type from a family of siblings,
    ** the Why
    field must explicitly **justify this type over the siblings named in
    What**.
     "Why `usize` and not `u64`?
    " "Why `String` and not `&str`?
    "
    "Why `Box<T>` and not `Rc<T>`?
    " One sentence each is enough;
     silence
    is not.
- **Gotcha** *(optional,
   only when warranted):
  * one line warning the reader
  when the construct **looks like** something familiar from TS but behaves
  differently (operator overloading,
   value-vs-reference,
   integer overflow,
  hoisting,
   GIL,
   ownership moves,
   …).
   Skip if there is no trap.
- A blank comment line.
- The literal lead-in `In TS you'd write (pseudocode):` — always TS,
  regardless of which language the surrounding file is in.
   The pseudocode
  block IS the TypeScript translation;
   that's the whole point.
- A fenced `` ```ts `` block (inside the language's comment syntax) holding
  the closest TS translation.
   Comments inside the block may further narrate.

Comments **never** describe the next line in the target language's own
jargon ("declares a `unique_ptr<T>`").
 Translate to TS-thinking:
 "an owning
reference to a heap-allocated `T` — when this variable goes out of scope,
the `T` is freed automatically".

## Worked example — C++ "hello world"

````cpp
// What:     `#include <iostream>` is C++'s "paste this file in here at
//           compile time" directive. `<iostream>` is the standard library
//           header that defines input/output streams. The `<...>` form
//           tells the compiler to look for the file in the standard library
//           directories (vs `"..."` which means "in my project").
// Why:      We need it so the `std::cout` line below is defined. Without
//           this `#include`, the file does not compile.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — TS gives you `console.log` for free.
// ```
#include <iostream>

// What:     `int main() { ... }` declares a function named `main` that
//           takes no arguments and returns an `int` (a 32-bit signed
//           integer). The `{ ... }` is the function body.
// Why:      Every C++ program must have exactly one `main`. The OS calls it
//           when the program starts; whatever it returns becomes the
//           program's exit code.
// Gotcha:   `int` is NOT TS's `number`. It is a fixed-width 32-bit signed
//           integer (range roughly ±2.1 billion). Overflow is undefined
//           behaviour — there is no auto-widening to `bigint`.
//
// In TS you'd write (pseudocode):
// ```ts
// async function main(): Promise<number> {
//   // ...body goes here...
//   return 0;
// }
// ```
int main() {
  // What:     `std::cout` is the standard "character output" stream — an
  //           object that represents the terminal's stdout. `<<` is the
  //           "stream insertion" operator: it pushes the right-hand value
  //           into the left-hand stream and returns the stream, which is
  //           why you can chain more `<<` after it. `"hello"` is a string
  //           literal. `std::endl` is a special value meaning "write a
  //           newline character AND flush the buffer to the terminal".
  // Why:      Print the word `hello` followed by a newline.
  // Gotcha:   `<<` here is OPERATOR OVERLOADING — the same `<<` symbol is
  //           bitwise left-shift on integers. C++ lets a type redefine what
  //           an operator means. TS has no such mechanism, so do not expect
  //           to see `<<` used this way anywhere in TS.
  //
  // In TS you'd write (pseudocode):
  // ```ts
  // console.log("hello");
  // ```
  std::cout << "hello" << std::endl;

  // What:     `return 0;` ends the function and hands the value `0` back to
  //           whoever called it. For `main` specifically, the caller is the
  //           operating system, and `0` is the convention for "program
  //           succeeded".
  // Why:      Tell the OS the program finished without error.
  //
  // In TS you'd write (pseudocode):
  // ```ts
  // return 0;
  // ```
  return 0;
}
````

In this 4-line program the comments outweigh the code roughly 15:1.
 **That
is the intended ratio.
** Other languages follow the same template by
analogy;
 only the comment form changes:
 a plain `//` / `#` / `--` / `;`
comment,
 or the language's doc-comment form when the block sits above a
documentable declaration.
 The lead-in stays `In TS you'd write (pseudocode):`
regardless of source language,
 because the pseudocode block always contains
TypeScript.

If a real file feels too verbose,
 the answer is to make the file smaller
(split into more functions,
 more files),
 **not** to thin out the comments.

## Anti-patterns drawn from real failures

These cases are taken from a real session where the skill was followed
loosely.
 Each shows the **bad** output the agent produced and the **good**
output the skill demands.
 Internalise the pattern,
 not just the specific
construct.

### Plain comment block with a stub doc comment bolted on (the rustdoc case)

Bad — the dum-dum block is written as plain `//` comments,
 then a separate,
name-derived doc comment is bolted on only to satisfy a "every item needs a doc
comment" lint:

````rust monochromatic-lint-skip-rustdoc
// What:     `const CEILING: f32 = ...` is the -1 dBTP linear-amplitude ceiling.
// Why:      Normalised output must never exceed it.
//
// In TS you'd write (pseudocode):
// ```ts
// const CEILING = 0.891;
// ```
/// Ceiling.
const CEILING: f32 = 0.8912509;
````

Good — the block itself is the doc comment (`///`),
 so the one block both
satisfies the lint and documents the item;
 there is no second summary:

````rust
//! A constant whose dum-dum block is its doc comment
/// What:
///     `const CEILING: f32 = ...` is the -1 dBTP linear-amplitude ceiling.
/// Why:
///      Normalised output must never exceed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CEILING = 0.891;
/// ```
const CEILING: f32 = 0.8912509;
````

The `/// Ceiling.` line only restates the name and splits the documentation into
a real block plus a hollow summary.
 For a statement inside a function body (not a
documentable item) the block stays plain `//`,
 because a doc comment there is a
compile error.

### Bare wrapper-constructor tail (the `Ok(rules)` case)

Bad — no comment on the function's tail expression because "it's just
wrapping a value":

```rust monochromatic-lint-skip-rustdoc
fn load_rules(path: &str) -> Result<Vec<Rule>, String> {
    // ... body with comments ...
    Ok(rules)
}
```

Good:

````rust
//! A loader whose tail expression carries its own dum-dum block
/// What:
///     `fn load_rules(path: &str) -> Result<Vec<Rule>, String>`
///     reads the rule file at `path`
///     and builds one `Rule` per entry.
/// Why:
///     The caller gets every rule or one error saying what failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadRules(path: string): Rule[]
/// ```
fn load_rules(path: &str) -> Result<Vec<Rule>, String> {
    // ... body with comments ...
    // What:     `Ok(rules)` constructs the success variant of `Result`,
    //           wrapping our `Vec<Rule>`. No trailing `;` means this is
    //           the function's tail expression — Rust auto-returns it.
    // Why:      Hand the freshly loaded rules to the caller and signal
    //           "no error".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rules;
    // ```
    Ok(rules)
}
````

### Type-conversion on a literal (the `.to_string()` case)

Bad — `.to_string()` slipped in unexplained on a string literal:

```rust monochromatic-lint-skip-rustdoc
return Err("no rules loaded".to_string());
```

Good:

````rust
//! A literal converted to an owned `String` with the conversion explained
// What:     `Err("...".to_string())`. `Err` is the failure variant of
//           `Result`. The literal `"no rules loaded"` has type
//           `&'static str` — a borrowed slice of bytes baked into the
//           binary at compile time. `.to_string()` allocates a fresh,
//           OWNED `String` whose contents are copied from that slice.
// Why:      The function signature is `Result<_, String>`, so the error
//           channel must contain an owned `String`, not a borrowed
//           `&str` — the caller may keep the error past our stack frame.
//
// In TS you'd write (pseudocode):
// ```ts
// throw new Error("no rules loaded");
// ```
return Err("no rules loaded".to_string());
````

### `Some(...)` at function tail (the `Some(escape_literal(trimmed))` case)

Bad — silent because "it's obviously the return":

```rust monochromatic-lint-skip-rustdoc
fn parse_rule_source(line: &str) -> Option<String> {
    // ... body ...
    Some(escape_literal(trimmed))
}
```

Good:

````rust
//! A parser whose `Some` tail carries its own dum-dum block
/// What:
///     `fn parse_rule_source(line: &str) -> Option<String>`
///     turns one line of the rule file into an escaped rule source.
/// Why:
///     `None` means the line holds no rule and the caller skips it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRuleSource(line: string): string | undefined
/// ```
fn parse_rule_source(line: &str) -> Option<String> {
    // ... body ...
    // What:     `Some(escape_literal(trimmed))` constructs the present
    //           variant of `Option`, wrapping the escaped string. Tail
    //           expression, so it's the return value.
    // Why:      The trimmed line is a plain literal; we escape it and
    //           hand it back as "yes, here is a rule source".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return escapeLiteral(trimmed);
    // ```
    Some(escape_literal(trimmed))
}
````

### Unwrap-with-fallback (the `.unwrap_or(false)` case)

Bad — silently appended to a method chain:

```rust monochromatic-lint-skip-rustdoc
if !combined.is_match(&content).unwrap_or(false) {
    return Ok(false);
}
```

Good:

````rust
//! A fallback on a fallible match with the dropped error named
// What:     `combined.is_match(&content)` returns `Result<bool, RegexError>`.
//           `.unwrap_or(false)` extracts the inner `bool` if the result
//           is `Ok`, otherwise substitutes `false` and DROPS the error.
//           `&content` borrows the bytes read-only — we are not
//           transferring ownership to the matcher.
// Why:      Treat any regex-engine failure as "no match" rather than
//           propagating it; a corrupt input shouldn't crash the scan.
// Gotcha:   `.unwrap_or(value)` on `Result` SILENTLY discards the error
//           value. Use only when "no info, fall back" is genuinely
//           correct.
//
// In TS you'd write (pseudocode):
// ```ts
// let matched: boolean;
// try { matched = combined.isMatch(content); } catch { matched = false; }
// if (!matched) return false;
// ```
if !combined.is_match(&content).unwrap_or(false) {
    return Ok(false);
}
````

### Type-choice rationale (the `usize` / `String` case)

Bad — names `usize` and `String` without saying why these and not the
siblings:

```rust monochromatic-lint-skip-rustdoc
// What:     `struct Rule { ... }` with `index` (usize), `src` (String),
//           `regex` (Regex).
// Why:      We keep one Rule per forbidden-string entry.
struct Rule { index: usize, src: String, regex: Regex }
```

Good:

````rust
//! A record type whose field choices are explained on the fields
/// What:
///     `struct Rule { ... }` declares a record type with three OWNED fields
///     and holds one rule per forbidden-string entry.
/// Why:
///     Each field below says why its type beats the siblings a reader might expect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rule = { index: number; src: string; regex: Regex };
/// // TS has no owned/borrowed distinction (everything is GC'd), so the type-choice
/// // question does not arise.
/// ```
struct Rule {
    /// What:
    ///     `index: usize` is the unsigned integer wide enough to address any byte in memory on this platform
    ///     (32 bits on a 32-bit OS and 64 bits on a 64-bit OS).
    ///     The siblings a reader might expect are `u32` and `u64` and `i32` and `i64`.
    /// Why:
    ///     Every std API that takes a "size" or "index" wants `usize`
    ///     and mixing widths forces casts everywhere.
    index: usize,
    /// What:
    ///     `src: String` is a heap-allocated growable UTF-8 buffer that THIS struct owns.
    ///     Its sibling `&str` is a borrowed view that doesn't own its bytes.
    /// Why:
    ///     The struct outlives the function that read the file
    ///     and a borrowed slice would dangle.
    src: String,
    /// What:
    ///     `regex: Regex` is an owned compiled regex object.
    /// Why:
    ///     It is owned for the same outlive reason as `src`.
    regex: Regex,
}
````

## Anti-patterns

- Reaching for an idiomatic-but-opaque construct because it is "the way" in
  that language (Rust iterator chains,
   Python list comprehensions,
   Go
  interface satisfaction by name,
   Kotlin scope functions).
- Dropping the **pseudocode** block because "the code is obvious".
   It is
  not obvious to this user,
   and it is mandatory on every block.
- Skipping **What** on a symbol-heavy line.
   If the line contains `<<`,
  `::`,
   `&`,
   `*`,
   `?`,
   `!`,
   `<...>`,
   `mut`,
   lifetimes,
   or any punctuation a
  TS dev would not recognise,
   name and explain each piece individually.
- Skipping a comment block on a **bare tail expression** (`Ok(rules)`,
  `Some(x)`,
   `(line, col)`,
   `if ... { a } else { b }` at end of fn).
   The
  tail IS the return;
   it always needs the full block.
- Skipping a comment block on a **wrapper constructor or unwrap chain**
  (`Some(...)`,
   `None`,
   `Ok(...)`,
   `Err(...)`,
   `.to_string()`,
  `.unwrap_or(...)`,
   `.ok()`,
   `.clone()`,
   `?`) on its **first**
  appearance in a file because it appeared a few lines up in a different file.
   The first-occurrence rule is scoped per file;
   later occurrences within
  that same file do take the short form.
- Naming a type without **naming its siblings and justifying the choice**.
  `usize` without "(not `u32`/`u64`/`i64`)";
   `String` without "(not
  `&str`)";
   `Box<T>` without "(not `Rc<T>`/`Arc<T>`)".
   The dum-dum reader
  has no way to know a sibling exists.
- Comments that describe the next line in the target language's own jargon
  ("declares a `unique_ptr<T>`").
   Translate to TS-thinking instead.
- Combining several lines under one comment block.
   One concept-introducing
  line ↔ one comment block,
   even if it gets repetitive.
- Removing comments later to "tidy up".
   The comments **are** the
  deliverable.
- Padding **Why** with a restatement of **What**.
   If `Why:` does not say
  what the program gains,
   delete it and write a real one.

## Out of scope

This skill does not apply when:

- the file is `.ts`,
   `.tsx`,
   `.js`,
   `.jsx`,
   `.mjs`,
   `.cjs`
- the file is a declarative or data format:
   `.html`,
   `.css`,
   `.scss`,
  `.json`,
   `.yaml`,
   `.yml`,
   `.toml`,
   `.sql`,
   `Dockerfile`,
   `mise.toml`,
  `package.json`,
   etc.
- the file is generated (auto-produced by a tool,
   marked `DO NOT EDIT`,
  vendored from a third party)
- the change is a single-character or mechanical edit in an existing non-TS
  file (rename,
   typo fix) where adding the comment template would dwarf
  the change.
   In that case,
   match the surrounding comment density instead.
