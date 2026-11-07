# toml_edit 0.25.15: index assignment, removal, and `Table::insert` silently drop or move comments

`toml_edit` round-trips an unedited document byte for byte,
but the plain edit calls
(index assignment `doc["a"]["b"] = value(...)` as the crate README shows,
`Array::remove`,
`Table::remove`,
and `Table::insert`)
silently delete comments or leave a comment next to a different element.
No error is raised.

This document covers that one cluster:
comments stored as `Decor` trivia on the node that owns them,
and edit calls that replace or remove that node together with its trivia.
It was found by the differential edit harness in
[the Rust structured-edit research](../planning/monorepo-manager-route-research/rust-structured-edits.md)
("Probe results by candidate",
"TOML"),
where `toml_edit` used as its README shows passed 12 of 21 cases
and a repository wrapper over the same crate passed 21 of 21.
Status (2026-09-17):
behavior unchanged at `toml-rs/toml` HEAD `3b81b06`
(`git diff --stat v0.25.15 origin/HEAD -- crates/toml_edit/src crates/toml_parser/src` printed nothing).

## Symptom

Every variant is silent:
the output parses,
values are correct,
and only comments or formatting change.

- Same-line comment deleted by index assignment.
   `doc["foo"] = value("new")` on `foo = "bar"  # trailing` writes `foo = "new"`.
  On a Cargo plan,
   `doc["dependencies"]["clap"] = value(...)` deletes `# clap trailing`
  while keeping `# comment above clap`:

  ```text
  -- input
  [dependencies]
  # comment above clap
  clap = { version = "3", features = ["derive"] }  # clap trailing
  serde = "1"
  -- output
  [dependencies]
  # comment above clap
  clap = { version = "4", features = ["derive"] }
  serde = "1"
  ```

  The same happens for dotted keys (`a.b = 1  # dotted`)
  and for a key inside an array-of-tables element (`doc["bin"][1]["name"] = value("c")` drops `# b`).
- Comment moved onto the wrong element by `Array::remove`.
   Removing `"b"` deletes the comment on the line of `"a"` and moves the comment of `"b"` onto `"a"`:

  ```text
  -- input
  arr = [
    "a", # first
    "b", # second
    "c",
  ]
  -- output
  arr = [
    "a", # second
    "c",
  ]
  ```

- Element lines joined by value assignment into an array slot.
   `*array.get_mut(1).unwrap() = Value::from("z")` on the same array writes `"a", "z", # second`:
   `# first` is deleted and `"z"` moves onto the line of `"a"`.
- Blank-line-separated comment deleted by `Table::remove`.
   Removing `foo` also deletes `# file header`,
   which a blank line separates from `foo`:

  ```text
  -- input
  # file header

  # about foo
  foo = 1  # foo trailing
  # about bar
  bar = 2
  -- output
  # about bar
  bar = 2
  ```

- Key formatting reset by `Table::insert` or `InlineTable::insert` on an occupied key.
   The leading comment lines and the spacing around the key are dropped,
   even when the caller copies the old value decor onto the new value:

  ```text
  -- input
  [dependencies]
  # comment above clap
  clap   = "3"  # clap trailing
  serde = "1"
  -- output of table.insert("clap", value("4"))
  [dependencies]
  clap = "4"
  serde = "1"
  -- output of table.insert("clap", Item::Value(new value carrying the old value decor))
  [dependencies]
  clap = "4"  # clap trailing
  serde = "1"
  ```

  `InlineTable::insert("version", ...)` on `{ version   = "3", ... }` writes `version = "4"`.

## Root cause

The parser stores every comment,
blank line,
and space as raw trivia (`Decor` prefix and suffix) on a neighboring node:
the prefix of the next key or array element,
or the suffix of the value on the same line.
The edit calls replace or remove whole nodes,
so the trivia goes with them.
None of the calls consults comment ownership.
All citations are `toml-rs/toml` at tag `v0.25.15` (commit `8e1d5a8`),
identical to the published crate source
(`diff --recursive --brief crates/toml_edit/src` against the Cargo registry copy printed nothing).

### Step 1: where the document parser puts comments

For a key-value line,
all trivia collected since the previous item becomes the key's leaf-decor prefix,
and the whitespace plus comment after the value up to the newline becomes the value's decor suffix
(`crates/toml_edit/src/parser/document.rs:61-111`,
excerpt):

```rust
// crates/toml_edit/src/parser/document.rs:61-111 (excerpt)
EventKind::SimpleKey => {
    let key_prefix = state.take_trailing();
    // ...
    key.leaf_decor.set_prefix(key_prefix);
    key.leaf_decor.set_suffix(key_suffix);
    // ...
    let mut value = value(input, source, errors);
    let value_suffix = ws_comment_newline(input)
        .map(|s| RawString::with_span(s.start()..s.end()))
        .unwrap_or_default();
    let decor = value.decor_mut();
    decor.set_prefix(value_prefix);
    decor.set_suffix(value_suffix);
```

The trailing trivia accumulates every whitespace,
comment,
and newline event,
blank lines included,
with no split at a blank line
(`crates/toml_edit/src/parser/document.rs:113-115`,
`:271-274`,
`:465-470`):

```rust
// crates/toml_edit/src/parser/document.rs:113-115
EventKind::Whitespace | EventKind::Comment | EventKind::Newline => {
    state.capture_trailing(event);
}
// crates/toml_edit/src/parser/document.rs:271-274
fn capture_trailing(&mut self, event: &toml_parser::parser::Event) {
    let decor = self.current_trailing.get_or_insert(event.span());
    *decor = decor.append(event.span());
}
```

So in the `# file header` input,
the prefix of key `foo` is `"# file header\n\n# about foo\n"`,
and `"  # foo trailing"` is the suffix of value `1`.

Inside an array,
trivia after a comma is collected as the prefix of the next value,
because `finish_value` runs at the comma and clears `current_value`
(`crates/toml_edit/src/parser/array.rs:65-68`,
`:99-117`):

```rust
// crates/toml_edit/src/parser/array.rs:65-68
EventKind::ValueSep => {
    state.finish_value(event, &mut result);
    state.sep_value(event);
}
// crates/toml_edit/src/parser/array.rs:99-110
fn whitespace(&mut self, event: &toml_parser::parser::Event) {
    let decor = if self.is_prefix() {
        self.current_prefix.get_or_insert(event.span())
    } else {
        self.current_suffix.get_or_insert(event.span())
    };
    *decor = decor.append(event.span());
}

fn is_prefix(&self) -> bool {
    self.current_value.is_none()
}
```

So `# first`,
written on the line of `"a"`,
is the prefix of `"b"`,
and `# second` is the prefix of `"c"`.
Upstream snapshots this ownership in its own test `assign_whitespace`
(`crates/toml_edit/tests/testsuite/edit.rs:134`):
the value `'value' # key comment` gets suffix `" # key comment"` (`:261`),
and the element after `'one' , # one comment` gets prefix `" # one comment\n   "` (`:449`).
The ownership model is deliberate and tested,
not an accident of one code path.

### Step 2: index assignment replaces the value and its decor

`IndexMut` for `Item` returns the existing slot (`crates/toml_edit/src/index.rs:45-58`),
and `value()` builds an `Item::Value` with no decor (`crates/toml_edit/src/item.rs:431-433`):

```rust
// crates/toml_edit/src/index.rs:51-52
match *v {
    Item::Table(ref mut t) => Some(t.entry(self).or_insert(Item::None)),
// crates/toml_edit/src/item.rs:431-433
pub fn value<V: Into<Value>>(v: V) -> Item {
    Item::Value(v.into())
}
```

Assignment through that slot replaces the whole `Value`,
including the decor suffix holding the same-line comment.
The encoder then uses the default value decor `(" ", "")`
(`crates/toml_edit/src/encode.rs:333-335`,
`crates/toml_edit/src/value.rs:361`).
The key and its leaf decor are untouched,
which is why `# comment above clap` survives index assignment.
The crate documents that formatting lives on the value
(`crates/toml_edit/src/item.rs:407-409`):

```rust
/// Since formatting is part of a `Value`,
/// the right hand side of the
/// assignment needs to be decorated with a space before the value.
/// The `value` function does just that.
```

The same replacement explains the joined array lines:
`Array::get_mut` hands out the slot (`crates/toml_edit/src/array.rs:167-169`),
and an element without decor is encoded with `(" ", "")` after the separator
(`crates/toml_edit/src/encode.rs:100-108`),
so the replaced element loses the `" # first\n  "` prefix that held the comment and the line break.

### Step 3: removal takes the node's trivia with it

`Array::remove` removes the element and its decor;
`Table::remove` shift-removes the key and its leaf decor
(`crates/toml_edit/src/array.rs:305-311`,
`crates/toml_edit/src/table.rs:461-464`):

```rust
// crates/toml_edit/src/array.rs:305-311
pub fn remove(&mut self, index: usize) -> Value {
    let removed = self.values.remove(index);
    match removed {
        Item::Value(v) => v,
        x => panic!("non-value item {x:?} in an array"),
    }
}
// crates/toml_edit/src/table.rs:461-464
/// Removes an item given the key.
pub fn remove(&mut self, key: &str) -> Option<Item> {
    self.items.shift_remove(key)
}
```

Removing `"b"` therefore deletes `# first` (prefix of `"b"`)
and leaves `# second` (prefix of `"c"`) printed after the comma of `"a"`.
Removing `foo` deletes its whole prefix,
`# file header` included.

### Step 4: `insert` on an occupied key resets the key on purpose

`Table::insert` calls `Key::fmt` on the existing key before swapping the item
(`crates/toml_edit/src/table.rs:428-443`),
and `Key::fmt` clears the representation and both decors
(`crates/toml_edit/src/key.rs:144-149`):

```rust
// crates/toml_edit/src/table.rs:428-443
/// Inserts a key-value pair into the map.
pub fn insert(&mut self, key: &str, item: Item) -> Option<Item> {
    use indexmap::map::MutableEntryKey;
    let key = Key::new(key);
    match self.items.entry(key.clone()) {
        indexmap::map::Entry::Occupied(mut entry) => {
            entry.key_mut().fmt();
            let old = std::mem::replace(entry.get_mut(), item);
            Some(old)
        }
        indexmap::map::Entry::Vacant(entry) => {
            entry.insert(item);
            None
        }
    }
}
// crates/toml_edit/src/key.rs:144-149
/// Auto formats the key.
pub fn fmt(&mut self) {
    self.repr = None;
    self.leaf_decor.clear();
    self.dotted_decor.clear();
}
```

`InlineTable::insert` does the same (`crates/toml_edit/src/inline_table.rs:395-411`).
The leading comment lives in the key's leaf-decor prefix (Step 1),
so it is cleared;
the same-line comment lives in the replaced item's decor,
so it is lost independently of the key reset.
`insert_formatted` instead stores the caller's key (`crates/toml_edit/src/table.rs:445-459`).

This is intended behavior.
Commit `9bca304` ("fix(edit):
Preserve new key's formatting when inserting",
2024-09-24,
first released in `v0.22.22`) added the reset to restore 0.22.20 behavior after
[toml-rs/toml#787][issue-787],
where the maintainer wrote
"In light of `insert_formatted`,
I'm tempted to keep `insert` not preserving the existing formatting"
and,
on changing it later,
"Likely based on feedback and,
at this point,
through a breaking change".
The doc comment on both `insert` methods still says only "Inserts a key-value pair into the map."

### Documented limitations versus bugs

The crate README "Limitations" (`crates/toml_edit/README.md:39-44`,
mirrored in `crates/toml_edit/src/lib.rs:64-69`)
lists only two things it does not preserve:
the order of dotted keys ([toml-rs/toml#163][issue-163])
and the lack of a trailing newline ([toml-rs/toml#1158][issue-1158]).
Both reproduce in the harness (`readme-limitation-*`),
and neither is about comments.
None of the symptoms in this document is listed there,
while the README opens with `preserving comments, spaces *and relative order* of items` (`README.md:8-10`).

Classification of each symptom against upstream intent:

- Index assignment dropping the same-line comment:
   intended value-replacement semantics,
   documented indirectly (`item.rs:407-409`;
   "By default,
  values are created with default formatting",
   `lib.rs:38`).
  The missing piece is an API that carries formatting over,
   which the maintainer opened as [toml-rs/toml#231][issue-231] in 2021 and is still open.
  `Array::replace` already does this for array elements (`crates/toml_edit/src/array.rs:242-266`).
- `Array::remove` moving a comment and `Table::remove` deleting a blank-line-separated comment:
   consequences of the tested ownership model (Step 1),
   undocumented in rustdoc.
  Upstream declined comment-association heuristics in [toml-rs/toml#791][issue-791]
  (same `remove` report for tables)
  and [toml-rs/toml#818][issue-818].
- `insert` resetting the key:
   intentional (#787,
   `9bca304`),
   with a documentation gap on `Table::insert` and `InlineTable::insert`.

No symptom is a defect against the crate's stated design.
The user-facing gap is the README promise of comment preservation without an edit caveat,
plus the `insert` doc comments.

### Corrected readings

- The research note
   "`Table::insert` on an occupied key calls `entry.key_mut().fmt()` ...
   the reset that deleted a comment"
   (`rust-structured-edits.md`,
  "Probe results by candidate")
   merged two mechanisms.
  The key reset deletes leading comment lines and key spacing;
   the same-line comment is deleted by the item replacement.
  The harness case `table-insert-occupied-key-copied-value-decor` separates them:
   copying the old value decor keeps `# clap trailing` but still drops `# comment above clap` and `clap   =`.
- The Cargo plan case (`t18`) loses `# clap trailing` through index assignment,
   not through `Table::insert`;
   index assignment never touches the key,
   so `# comment above clap` survives there.
- The `insert` key reset is not a regression in 0.25.x:
   it has been the behavior since `v0.22.22` and was chosen deliberately in #787.

## Verification

Version under test:
 `toml_edit` `0.25.15+spec-1.1.0`,
 crates.io checksum `1340ea94a5856333492c9064b02c778b191dd2c853778d9609debdcdfea3a614`
 (crates.io API and the harness `Cargo.lock` agree),
 published 2026-09-11,
 git tag `v0.25.15` (`8e1d5a85c361ac012957441bb4788ae82f5dc9c8`,
 from the crate's `.cargo_vcs_info.json`).
Built and run on 2026-09-17 inside `docker.io/library/rust:slim`
(`rustc 1.98.1 (48a229cea 2026-09-01)`)
with no repository mount and no credentials.

Harness layout:
 `Cargo.toml`,
 `src/bin/reproduce.rs` (this section),
 and `src/bin/wrapper.rs` ("Verified workarounds").

```toml
# ~/temp/agent/toml-edit-quirks/Cargo.toml
[package]
name = "toml-edit-quirks"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
toml_edit = "=0.25.15"

[workspace]
```

```rust
// ~/temp/agent/toml-edit-quirks/src/bin/reproduce.rs
//! toml_edit 0.25.15 edits written the way the crate README shows,
//! plus the crate APIs that keep decor.
use toml_edit::{DocumentMut, InlineTable, Item, Value, value};

const CARGO: &str = "[dependencies]\n# comment above clap\n\
    clap = { version = \"3\", features = [\"derive\"] }  # clap trailing\nserde = \"1\"\n";
const SPACED: &str = "[dependencies]\n# comment above clap\nclap   = \"3\"  # clap trailing\nserde = \"1\"\n";
const ARRAY: &str = "arr = [\n  \"a\", # first\n  \"b\", # second\n  \"c\",\n]\n";
const HEADER: &str = "# file header\n\n# about foo\nfoo = 1  # foo trailing\n# about bar\nbar = 2\n";

fn parse(source: &str) -> DocumentMut {
    source.parse().expect("case source is valid TOML")
}

/// PASS when every `keep` needle survives and every `gone` needle is absent.
fn report(name: &str, source: &str, doc: &DocumentMut, keep: &[&str], gone: &[&str]) {
    let out = doc.to_string();
    let lost: Vec<_> = keep.iter().filter(|n| !out.contains(**n)).collect();
    let stale: Vec<_> = gone.iter().filter(|n| out.contains(**n)).collect();
    let verdict = if lost.is_empty() && stale.is_empty() { "PASS" } else { "FAIL" };
    println!("== {verdict} {name}\n-- input\n{source}-- output\n{out}-- missing {lost:?}; not removed {stale:?}\n");
}

fn round_trip(name: &str, source: &str) {
    let out = parse(source).to_string();
    let verdict = if out == source { "PASS" } else { "FAIL" };
    println!("== {verdict} {name}\n-- input  {source:?}\n-- output {out:?}\n");
}

fn clap4() -> Value {
    let mut table = InlineTable::new();
    table.insert("version", Value::from("4"));
    table.insert("features", Value::Array(["derive"].into_iter().collect()));
    Value::InlineTable(table)
}

fn main() {
    let src = "foo = \"bar\"  # trailing\n";
    let mut doc = parse(src);
    doc["foo"] = value("new");
    report("index-assign-trailing-comment", src, &doc, &["# trailing"], &[]);

    let mut doc = parse(CARGO);
    doc["dependencies"]["clap"] = value(clap4());
    report("index-assign-cargo-clap", CARGO, &doc, &["# comment above clap", "# clap trailing"], &[]);

    let src = "a.b = 1  # dotted\na.c = 2\n";
    let mut doc = parse(src);
    doc["a"]["b"] = value(5);
    report("index-assign-dotted-key", src, &doc, &["# dotted"], &[]);

    let src = "[[bin]]\nname = \"a\"\n\n# second bin\n[[bin]]\nname = \"b\"  # b\n";
    let mut doc = parse(src);
    doc["bin"][1]["name"] = value("c");
    report("index-assign-array-of-tables-element", src, &doc, &["# second bin", "# b"], &[]);

    let mut doc = parse(ARRAY);
    *doc["arr"].as_array_mut().unwrap().get_mut(1).unwrap() = Value::from("z");
    report("value-assign-array-element", ARRAY, &doc, &["\"a\", # first\n"], &[]);

    let mut doc = parse(ARRAY);
    doc["arr"].as_array_mut().unwrap().remove(1);
    report("array-remove-middle-element", ARRAY, &doc, &["\"a\", # first"], &["# second"]);

    let mut doc = parse(HEADER);
    doc.as_table_mut().remove("foo");
    report("table-remove-key-after-blank-line", HEADER, &doc, &["# file header", "# about bar"],
        &["# about foo", "# foo trailing"]);

    let keep_all = ["# comment above clap", "clap   =", "# clap trailing"];
    let mut doc = parse(SPACED);
    doc["dependencies"].as_table_mut().unwrap().insert("clap", value("4"));
    report("table-insert-occupied-key", SPACED, &doc, &keep_all, &[]);

    let mut doc = parse(SPACED);
    let table = doc["dependencies"].as_table_mut().unwrap();
    let mut next = Value::from("4");
    *next.decor_mut() = table.get("clap").and_then(Item::as_value).unwrap().decor().clone();
    table.insert("clap", Item::Value(next));
    report("table-insert-occupied-key-copied-value-decor", SPACED, &doc, &keep_all, &[]);

    let src = "clap = { version   = \"3\", features = [\"derive\"] }  # c\n";
    let mut doc = parse(src);
    doc["clap"].as_inline_table_mut().unwrap().insert("version", Value::from("4"));
    report("inline-table-insert-occupied-key", src, &doc, &["version   =", "# c"], &[]);

    println!("# Limitations the crate README lists\n");
    round_trip("readme-limitation-missing-final-newline", "a = 1");
    round_trip("readme-limitation-dotted-key-order", "a.b = 1\nc = 2\na.d = 3\n");

    println!("# TOML 1.1 input, unedited\n");
    let src = "clap = { version = \"3\", features = [ \"derive\", ], }\nt = {\n  a = 1,\n}\n";
    round_trip("toml-1.1-inline-tables", src);

    println!("# Crate APIs that already keep formatting\n");
    let mut doc = parse(ARRAY);
    doc["arr"].as_array_mut().unwrap().replace(1, "z");
    report("array-replace", ARRAY, &doc, &["\"a\", # first\n"], &[]);

    let mut doc = parse(SPACED);
    let table = doc["dependencies"].as_table_mut().unwrap();
    let old_key = table.get_key_value("clap").map(|(key, _)| key.clone()).unwrap();
    let mut next = Value::from("4");
    *next.decor_mut() = table.get("clap").and_then(Item::as_value).unwrap().decor().clone();
    table.insert_formatted(&old_key, Item::Value(next));
    report("insert-formatted-old-key-copied-value-decor", SPACED, &doc, &keep_all, &[]);
}
```

Run it in a bounded container that sees only the harness directory:

```bash
# Build and run both harness binaries; third-party code runs only inside the container.
podman run --rm --memory=2g --cpus=2 \
  --volume "${HOME}/temp/agent/toml-edit-quirks:/work:Z" --workdir /work \
  docker.io/library/rust:slim \
  sh -c 'cargo build --jobs 4 --quiet && target/debug/reproduce && target/debug/wrapper'
```

The same symptoms appear in the research harness cases
`t01`,
`t03`,
`t07`,
`t10`,
`t14`,
`t16`,
and `t18`
([research doc](../planning/monorepo-manager-route-research/rust-structured-edits.md),
"Probe results by candidate").

### Patterns that keep every comment

- `Array::replace(index, value)`:
   output keeps `"a", # first` and `"z", # second` (`array-replace`).
- `Table::insert_formatted(&old_key, item)` with the old key cloned from `get_key_value`
   and the old value decor copied onto the new value:
   output `# comment above clap` / `clap   = "4"  # clap trailing`
   (`insert-formatted-old-key-copied-value-decor`).
- Unedited TOML 1.1 inline tables with trailing commas and newlines round-trip byte for byte
   (`toml-1.1-inline-tables`).
- Every consumer-side wrapper case in "Verified workarounds".

### Patterns that lose a same-line comment (value decor replaced)

- `doc["foo"] = value("new")`:
   `# trailing` missing (`index-assign-trailing-comment`).
- `doc["dependencies"]["clap"] = value(inline_table)`:
   `# clap trailing` missing,
   `# comment above clap` kept (`index-assign-cargo-clap`).
- `doc["a"]["b"] = value(5)` on a dotted key:
   `# dotted` missing (`index-assign-dotted-key`).
- `doc["bin"][1]["name"] = value("c")`:
   `# b` missing,
   `# second bin` kept (`index-assign-array-of-tables-element`).

### Patterns that move a comment or join lines (array element decor)

- `array.remove(1)`:
   output `"a", # second`;
   `# first` missing and `# second` not removed (`array-remove-middle-element`).
- `*array.get_mut(1).unwrap() = Value::from("z")`:
   output `"a", "z", # second`;
   `# first` missing (`value-assign-array-element`).

### Patterns that delete a detached comment (key leaf-decor prefix)

- `doc.as_table_mut().remove("foo")`:
   output `# about bar` / `bar = 2`;
   `# file header` missing (`table-remove-key-after-blank-line`).

### Patterns that reset key formatting (`Key::fmt`)

- `table.insert("clap", value("4"))`:
   output `clap = "4"`;
   `# comment above clap`,
   `clap   =`,
   and `# clap trailing` missing (`table-insert-occupied-key`).
- `table.insert("clap", Item::Value(value_with_old_decor))`:
   output `clap = "4"  # clap trailing`;
   `# comment above clap` and `clap   =` missing (`table-insert-occupied-key-copied-value-decor`).
- `inline_table.insert("version", Value::from("4"))`:
   output `clap = { version = "4", features = ["derive"] }  # c`;
   `version   =` missing (`inline-table-insert-occupied-key`).

### Documented limitations (README "Limitations")

- `"a = 1"` round-trips as `"a = 1\n"` (`readme-limitation-missing-final-newline`).
- `"a.b = 1\nc = 2\na.d = 3\n"` round-trips as `"a.b = 1\na.d = 3\nc = 2\n"` (`readme-limitation-dotted-key-order`).

## Verified workarounds

All workarounds live at our boundary as consumer-side helpers;
none edits `toml_edit`.
The research wrapper (`rust-structured-edits.md`,
"Probe results by candidate",
226 plus 69 lines of Rust) passed all 21 harness cases,
and additionally creates deep paths as dotted keys,
appends to inline tables and multi-line arrays using sibling formatting,
renames by rebuilding the table with the old key decor,
and restores CRLF when every source line ended in CRLF.
The reduced helpers in `src/bin/wrapper.rs` cover this document's symptoms,
verified by the same container command:

```rust
// ~/temp/agent/toml-edit-quirks/src/bin/wrapper.rs
//! Consumer-side toml_edit 0.25.15 helpers that avoid the comment-losing calls.
use toml_edit::{Array, DocumentMut, InlineTable, Table, Value};

const CARGO: &str = "[dependencies]\n# comment above clap\n\
    clap = { version = \"3\", features = [\"derive\"] }  # clap trailing\nserde = \"1\"\n";
const SPACED: &str = "[dependencies]\n# comment above clap\nclap   = \"3\"  # clap trailing\nserde = \"1\"\n";
const ARRAY: &str = "arr = [\n  \"a\", # first\n  \"b\", # second\n  \"c\",\n]\n";
const HEADER: &str = "# file header\n\n# about foo\nfoo = 1  # foo trailing\n# about bar\nbar = 2\n";

fn parse(source: &str) -> DocumentMut {
    source.parse().expect("case source is valid TOML")
}

fn report(name: &str, source: &str, doc: &DocumentMut, keep: &[&str], gone: &[&str]) {
    let out = doc.to_string();
    let lost: Vec<_> = keep.iter().filter(|n| !out.contains(**n)).collect();
    let stale: Vec<_> = gone.iter().filter(|n| out.contains(**n)).collect();
    let verdict = if lost.is_empty() && stale.is_empty() { "PASS" } else { "FAIL" };
    println!("== {verdict} {name}\n-- input\n{source}-- output\n{out}-- missing {lost:?}; not removed {stale:?}\n");
}

fn raw(part: Option<&toml_edit::RawString>) -> String {
    part.and_then(|r| r.as_str()).unwrap_or_default().to_owned()
}

/// Replaces an existing value in place,
/// carrying its decor (leading space,
/// same-line comment) over.
/// The key and its decor are never touched,
/// so `Table::insert` is never called on an occupied key.
fn replace_value(slot: &mut Value, mut next: Value) {
    *next.decor_mut() = slot.decor().clone();
    *slot = next;
}

/// Removes an array element and hands its prefix,
/// which holds the previous element's
/// same-line comment,
/// to the next element (or to the array trailing when it was last).
fn remove_element(array: &mut Array, index: usize) {
    let prefix = raw(array.remove(index).decor().prefix());
    match array.get_mut(index) {
        Some(next) => {
            next.decor_mut().set_prefix(prefix);
        }
        None => {
            if let Some(cut) = prefix.rfind('\n') {
                array.set_trailing(&prefix[..=cut]);
            }
        }
    }
}

/// Removes a key;
/// comments a blank line separates from it move onto the next key.
fn remove_key(table: &mut Table, key: &str) {
    let prefix = table.get_key_value(key).map(|(k, _)| raw(k.leaf_decor().prefix())).unwrap_or_default();
    let detached = prefix.rfind("\n\n").map(|cut| prefix[..cut + 2].to_owned()).unwrap_or_default();
    let order: Vec<String> = table.iter().map(|(k, _)| k.to_owned()).collect();
    let next = order.iter().position(|k| k == key).and_then(|i| order.get(i + 1)).cloned();
    table.remove(key);
    if let (false, Some(next)) = (detached.is_empty(), next) {
        if let Some(mut next_key) = table.key_mut(&next) {
            let old = raw(next_key.leaf_decor().prefix());
            next_key.leaf_decor_mut().set_prefix(format!("{detached}{old}"));
        }
    }
}

fn clap4() -> Value {
    let mut table = InlineTable::new();
    table.insert("version", Value::from("4"));
    table.insert("features", Value::Array(["derive"].into_iter().collect()));
    Value::InlineTable(table)
}

fn main() {
    let src = "foo = \"bar\"  # trailing\n";
    let mut doc = parse(src);
    replace_value(doc["foo"].as_value_mut().unwrap(), Value::from("new"));
    report("wrapper-replace-trailing-comment", src, &doc, &["# trailing", "\"new\""], &[]);

    let mut doc = parse(CARGO);
    replace_value(doc["dependencies"]["clap"].as_value_mut().unwrap(), clap4());
    report("wrapper-replace-cargo-clap", CARGO, &doc,
        &["# comment above clap", "# clap trailing", "version = \"4\""], &[]);

    let mut doc = parse(SPACED);
    replace_value(doc["dependencies"]["clap"].as_value_mut().unwrap(), Value::from("4"));
    report("wrapper-replace-spaced-key", SPACED, &doc,
        &["# comment above clap", "clap   = \"4\"", "# clap trailing"], &[]);

    let mut doc = parse(ARRAY);
    remove_element(doc["arr"].as_array_mut().unwrap(), 1);
    report("wrapper-array-remove-middle-element", ARRAY, &doc, &["\"a\", # first\n  \"c\","], &["# second"]);

    let src = "arr = [\n  \"a\", # first\n  \"b\", # second\n  \"c\", # third\n]\n";
    let mut doc = parse(src);
    remove_element(doc["arr"].as_array_mut().unwrap(), 2);
    report("wrapper-array-remove-last-element", src, &doc, &["\"a\", # first", "\"b\", # second\n]"],
        &["# third", "\"c\""]);

    let mut doc = parse(HEADER);
    remove_key(doc.as_table_mut(), "foo");
    report("wrapper-remove-key-after-blank-line", HEADER, &doc, &["# file header\n\n# about bar"],
        &["# about foo", "# foo trailing"]);

    println!("# Known wrapper gaps\n");
    let src = "a = 1\n# note about the end of the table\n\nfoo = 2\n";
    let mut doc = parse(src);
    remove_key(doc.as_table_mut(), "foo");
    report("gap-remove-last-key-drops-detached-comment", src, &doc, &["# note about the end of the table"], &[]);

    let src = "# file header\r\n\r\n# about foo\r\nfoo = 1\r\nbar = 2\r\n";
    let mut doc = parse(src);
    remove_key(doc.as_table_mut(), "foo");
    report("gap-crlf-blank-line-not-detected", src, &doc, &["# file header"], &["# about foo"]);

    let src = "arr = [\n  \"a\",\n  # about b\n  \"b\",\n  \"c\",\n]\n";
    let mut doc = parse(src);
    remove_element(doc["arr"].as_array_mut().unwrap(), 1);
    report("gap-own-line-comment-moves-to-next-element", src, &doc, &["\"c\""], &["# about b"]);

    let src = "arr = [\n  \"a\", # first\n  \"b\", # second\n  # closing note\n]\n";
    let mut doc = parse(src);
    remove_element(doc["arr"].as_array_mut().unwrap(), 1);
    report("gap-remove-last-element-replaces-closing-comment", src, &doc, &["# first", "# closing note"],
        &["# second"]);
}
```

Result on 2026-09-17:
 every `wrapper-*` case prints `PASS`;
 every `gap-*` case prints `FAIL` by construction,
 documenting the tradeoffs.

### Replace values in place instead of assigning or inserting

`replace_value` copies the old value decor onto the new value and writes through the existing slot,
so neither the value suffix nor the key decor changes.
Cargo's own manifest editor uses the same pattern
(`rust-lang/cargo` `src/workspace/editor/dependency.rs:676-685`,
fetched 2026-09-17 at the file's latest commit `c74cbfe`):

```rust
// rust-lang/cargo src/workspace/editor/dependency.rs:676-685
fn overwrite_value(
    table: &mut dyn toml_edit::TableLike,
    key: &str,
    value: impl Into<toml_edit::Value>,
) {
    let mut value = value.into();
    let existing = table.entry(key).or_insert_with(|| Default::default());
    if let Some(existing_value) = existing.as_value() {
        *value.decor_mut() = existing_value.decor().clone();
    }
```

Verified:
 `wrapper-replace-trailing-comment`,
 `wrapper-replace-cargo-clap` (output `clap = { version = "4", features = ["derive"] }  # clap trailing`),
 `wrapper-replace-spaced-key` (output `clap   = "4"  # clap trailing`).

Tradeoffs:

- Only for existing values.
   Replacing a standard table with a value,
   or a value with a table,
   has no decor to carry and needs its own rule.
- The copied prefix is the old leading whitespace,
   usually `" "`;
   a caller that wants canonical spacing must set it explicitly.
- The new value's inner formatting is crate default (`["derive"]`,
  `{ version = "4", ... }`),
   not the old inner layout;
   a multi-line inline array replaced wholesale becomes single-line.
- Presence must be checked with `get` before `get_mut`:
   `Item::get_mut` and `IndexMut` with a string key insert an `Item::None` placeholder for a missing key
   (`crates/toml_edit/src/index.rs:45-58`).

For array elements,
 `Array::replace` is the crate API with the same behavior
 (`crates/toml_edit/src/array.rs:257-266`);
 verified by `array-replace`.

### Use `insert_formatted` with the old key when an entry must be replaced through the table

When a table-level replacement is unavoidable,
clone the old `Key` from `get_key_value`,
copy the old value decor onto the new value,
and call `insert_formatted`.
Verified by `insert-formatted-old-key-copied-value-decor`.

Tradeoff:
 the caller must remember both copies;
 forgetting the key copy silently reintroduces the `Key::fmt` reset,
 so wrapping it in one helper is safer than calling it inline.

### Hand a removed element's prefix to the next element

`remove_element` gives the removed element's prefix,
which holds the previous element's same-line comment,
to the next element;
when the removed element was last,
the prefix up to its final newline becomes the array trailing.
Verified by `wrapper-array-remove-middle-element` (output `"a", # first` / `"c",`)
and `wrapper-array-remove-last-element` (output `"a", # first` / `"b", # second` / `]`).

Tradeoffs:

- It assumes a comment after a comma belongs to the element on that line.
   A comment on its own line between elements,
   meant as a leading comment for the removed element,
   is kept and handed to the next element instead of being removed
   (`gap-own-line-comment-moves-to-next-element`).
- Removing the last element overwrites the existing array trailing
   (`remove_element` calls `Array::set_trailing`),
   which held the removed element's own same-line comment (`# third`);
   that is the intended removal,
   but a comment line stored there before the closing `]` is deleted too
   (`gap-remove-last-element-replaces-closing-comment` loses `# closing note`).

### Keep blank-line-separated comments when removing a key

`remove_key` splits the removed key's prefix at the last blank line
and moves the detached part onto the next key.
Verified by `wrapper-remove-key-after-blank-line` (output `# file header`,
blank line,
`# about bar`).

Tradeoffs:

- When the removed key is the last in its table there is no next key,
   so the detached comment is dropped (`gap-remove-last-key-drops-detached-comment`).
   A complete helper would move it to the next table's decor prefix or the document trailing.
- The split looks for `"\n\n"`,
   so a CRLF blank line (`"\r\n\r\n"`) or a whitespace-only line is not recognized
   and the detached comment is deleted (`gap-crlf-blank-line-not-detected`).
- Comment attachment is a policy choice:
   this helper keeps comments separated by a blank line and removes directly attached ones,
   matching the research definition D6,
   while `module-toml-edit` keeps attached comments (research case `t10`).

## What does not work

- **Copying the old value decor and then calling `Table::insert`.**
  The item decor survives,
   but `Key::fmt` still clears the key's leaf decor,
   deleting leading comments and key spacing (`table-insert-occupied-key-copied-value-decor`).
- **Assigning into `Array::get_mut`.**
  The new element has no decor,
   so the line break and the comment stored in the old prefix disappear
   and the element joins the previous line (`value-assign-array-element`).
   Use `Array::replace`.
- **Renaming by `remove` then `insert`.**
  The key moves to the end of the table and its leading comment is deleted
   (research case `t15`,
  output `next = 2` / `new = 1  # trail` from `# above` / `old = 1  # trail` / `next = 2`).
   `toml_edit` has no rename API;
   the research wrapper rebuilds the table order with `remove_entry` and `insert_formatted`.
- **Calling `Table::fmt` or `InlineTable::fmt` to tidy up after an edit.**
  Upstream reports that `fmt` drops comments between items ([toml-rs/toml#818][issue-818]),
   and the maintainer called it "fairly naive in what it does".
- **Waiting for upstream to attach comments by paragraph.**
  The maintainer declined that heuristic in [toml-rs/toml#791][issue-791]:
   "At this time,
  I think I'd rather avoid heuristics as we can easily get things wrong.
   You are welcome to explore these ideas in helpers built on top".

## Upstream filing artifact (do not file)

### Upstream filing decision

`.out-of-scope/` was checked first:
 `rg --ignore-case 'toml|taplo' .out-of-scope/` matched only `bun-install.md` and `cargo-workspace.md`,
 which mention `mise.toml` and `Cargo.toml` files,
 not `toml_edit`,
 so no exemption applies.

Walking the six constraints:

1.  **Is it really upstream's fault?**
    Mostly no.
    Every symptom follows from the deliberate,
    tested comment-ownership model (Step 1)
    and from documented value-replacement semantics (Step 2);
    the `insert` key reset is intentional (#787,
    `9bca304`).
    The upstream-owned part is wording:
    the README promises comment preservation without an edit caveat,
    and the `Table::insert` and `InlineTable::insert` doc comments
    (`table.rs:428`,
    `inline_table.rs:395`) do not mention the key reset.
2.  **Can upstream fix it?**
    Yes.
    Documentation is a small change;
    a formatting-carrying insert is tracked in #231,
    and `Array::replace` already shows the pattern.
    Comment association on removal is technically possible.
3.  **Are they supporting this use case?**
    Partly.
    The README value proposition is preserving comments,
    and the crate is "primarily tailored for cargo-edit needs";
    but the maintainer directs comment-association logic to "helpers built on top" (#791),
    and Cargo carries decor over in its own editor (`overwrite_value`).
    The supported path is consumer-side helpers,
    which is what the workarounds do.
4.  **Would the repo welcome our contribution?**
    No.
    `AI_POLICY.md` at `toml-rs/toml` HEAD (added 2026-08-12,
    commits `d016feb`,
    `516da22`,
    `8b090d2`) says
    "**AI should not be used to generate comments when communicating with maintainers.**",
    "If you are opening an issue,
    we expect you to have personally reproduced the problem and to describe it in your own words",
    "Using AI as tools for coding requires receiving permission on the relevant issue before a PR is posted",
    and routes documentation changes through the same communication rule.
    This investigation,
    its reproduction,
    and every draft here are AI-produced,
    so we cannot truthfully provide the human-authored filing the policy requires.
    Also checked:
    `CONTRIBUTING.md` (welcomes issues and asks for discussion before pull requests),
    `.github/PULL_REQUEST_TEMPLATE.md` ("LLM involvement:"
    field),
    no issue templates (`gh api repos/toml-rs/toml/contents/.github/ISSUE_TEMPLATE` returned 404).
5.  **Will they likely fix it?**
    Split by symptom.
    Removal-side comment association:
    no,
    stated in #791 (declined,
    closed) and #818
    ("There is ambiguity about what what a comment is associated with and there is likely no right answer").
    Formatting-carrying insert:
    no movement since #231 opened on 2021-10-05;
    related #888 (2025-05-02,
    open) notes "This also requires manual work to maintain the `Decor` when changing a `Value`".
    `insert` key reset:
    "Likely based on feedback and,
    at this point,
    through a breaking change" (#787).
    Commits since 2026-02 touching `table.rs`,
    `array.rs`,
    `index.rs`,
    `parser/document.rs`,
    or `parser/array.rs` are parser fixes,
    refactors,
    a span fix,
    style,
    and performance work.
    [toml-rs/toml#1123][pr-1123] (merged 2026-03-23,
    `encode.rs` and `repr.rs`) emits a key's comment before the header bracket
    when an inline table carrying a comment is replaced by a table,
    a narrow fix for #691 rather than a change to comment ownership.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No.
    The auto-prototype step is not triggered:
    it requires constraints 1 to 5 to hold or sorta-hold,
    and constraints 1 and 4 fail for every symptom (and 5 fails for the removal symptoms).
    A documentation patch would also fall under the policy's human-authorship rule.

Decision:
 file nothing,
 comment nothing.
The consumer-side helpers solve the problem at our boundary.
A future filing requires a human who personally reproduces the behavior and writes the report in their own words;
the "Draft (do not file as-is)" subsection is an auditable record,
not text to paste.

### Duplicate search

Searched on 2026-09-17 with `gh search issues --repo toml-rs/toml -- <term>`
and `gh search prs --repo toml-rs/toml -- <term>`
(no state filter,
so open and closed both returned).
Issue terms:
 `comment`,
 `decor`,
 `remove comment`,
 `trailing comment`,
 `preserve formatting`,
 `array remove`,
 `insert formatting`,
 `lost comment`,
 `comments removed`,
 `remove key`,
 `IndexMut`,
 `value decor`,
 `header comment`,
 `insert key`,
 `insert_formatted`,
 `Table::insert`,
 `array element comment`,
 `remove element`,
 `comments attached`,
 `comment association`,
 `Array::remove`.
Pull request terms:
 `decor`,
 `insert_formatted`,
 `preserve decor`,
 `comment`,
 `Array::remove`.
Threads read in full:

- [toml-rs/toml#231][issue-231] (open,
  2021-10-05,
  maintainer-authored):
   "Allow inserting where we carry over the original formatting";
   covers index assignment dropping the same-line comment.
   No comments.
- [toml-rs/toml#791][issue-791] (closed,
  declined,
  2024-09-25):
   removing a table deletes the file header comment and misplaces commented-out entries
   because all trivia lands in the next node's prefix;
   covers `Table::remove` deleting a blank-line-separated comment.
- [toml-rs/toml#818][issue-818] (open,
  2024-12-10):
   `fmt` drops comments;
   maintainer stance on comment association.
- [toml-rs/toml#787][issue-787] (closed,
  2024-09-24):
   origin of the `insert` key reset.
- [toml-rs/toml#888][issue-888] (open,
  2025-05-02):
   "Consider splitting `Decor` out of types".
- [toml-rs/toml#1043][pr-1043] (closed,
  2025-09-19):
   maintainer WIP "fix(edit):
  Auto-adjust decor" for #267,
   closed with "This is getting a bit messy and we may want to re-think how we encode to make this easier".

No thread reports `Array::remove` moving a same-line comment onto the previous element,
but #791 and #818 already state the maintainer position on comment association,
so a new report would not change the outcome.

Nothing to add to #231,
#791,
#818,
or #787:
our reproductions restate behavior those threads already describe,
the decor-carrying pattern we would point to already ships as `Array::replace` in the crate
and as `overwrite_value` in Cargo's editor,
and the policy bars AI-written comments.
Post nothing.

### Draft (do not file as-is)

Kept only as the record of the one upstream-owned gap (constraint 1).
Blocked by constraint 4:
 a human must reproduce it personally and rewrite it in their own words before any use.

Title:
 Document that `Table::insert` and `InlineTable::insert` reset the existing key's formatting

Labels:
 `A-edit`,
 `A-docs`

~~~md
## Description

On an occupied key, `Table::insert` and `InlineTable::insert` call `Key::fmt` on the existing
key before replacing the item (`crates/toml_edit/src/table.rs:428-443`,
`crates/toml_edit/src/inline_table.rs:395-411`, `crates/toml_edit/src/key.rs:144-149`).
That clears the key's leaf decor, so comment lines directly preceding the key and the spacing
around it are removed. The doc comment on both methods says only "Inserts a key-value pair
into the map."

This is intentional since #787 (commit 9bca304), but the difference from `insert_formatted`
and from in-place mutation is only discoverable from source.

## Reproduction (toml_edit 0.25.15)

```rust
use toml_edit::{DocumentMut, value};

let src = "[dependencies]\n# comment above clap\nclap   = \"3\"  # clap trailing\nserde = \"1\"\n";
let mut doc: DocumentMut = src.parse().unwrap();
doc["dependencies"].as_table_mut().unwrap().insert("clap", value("4"));
assert_eq!(doc.to_string(), "[dependencies]\nclap = \"4\"\nserde = \"1\"\n");
```

## Suggested fix

Extend the doc comments at `crates/toml_edit/src/table.rs:428` and
`crates/toml_edit/src/inline_table.rs:395` to say that an existing key's formatting
(including leading comments) is reset and the old item's decor is dropped, and point to
`insert_formatted` with the existing `Key`, or to mutating the value through `get_mut`,
for format-preserving updates.
~~~

[issue-163]: https://github.com/toml-rs/toml/issues/163
[issue-231]: https://github.com/toml-rs/toml/issues/231
[issue-787]: https://github.com/toml-rs/toml/issues/787
[issue-791]: https://github.com/toml-rs/toml/issues/791
[issue-818]: https://github.com/toml-rs/toml/issues/818
[issue-888]: https://github.com/toml-rs/toml/issues/888
[issue-1158]: https://github.com/toml-rs/toml/issues/1158
[pr-1043]: https://github.com/toml-rs/toml/pull/1043
[pr-1123]: https://github.com/toml-rs/toml/pull/1123
