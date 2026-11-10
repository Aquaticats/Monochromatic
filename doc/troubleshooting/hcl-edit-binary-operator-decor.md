# hcl-edit 0.9.7: comments and line breaks around binary operators become single spaces on write-back

`hcl-edit` round-trips most HCL byte for byte,
but every comment and every newline that sits next to a binary operator
(`+`,
`-`,
`*`,
`/`,
`%`,
`==`,
`!=`,
`<`,
`<=`,
`>`,
`>=`,
`&&`,
`||`)
is replaced by a single space when the parsed document is printed again.
Nothing is reported:
the output still parses and still evaluates to the same value,
only the author's comment is gone and the line is joined.

Found while designing meow's HCL front end
(`doc/planning/monorepo-manager-from-scratch-design.md`,
"Declarative configuration"),
where the earlier research had already recorded the joined `&&` condition
(`doc/planning/monorepo-manager-route-research/stack-declarative-config.md`,
"HCL in Rust")
without a cause.

Status 2026-09-17:
unchanged at upstream `HEAD` `f7b17594f8d738343508f87dc9a24018e7fbeb8d` (2026-09-06),
reproduced there and in the published crate `hcl-edit` 0.9.7.

## Symptom

Silent in every variant.

- A comment between the operands disappears.

  ```text
  -- input
  x = (1 /* inside operands */ + 2)
  -- output of parse_body followed by to_string
  x = (1 + 2)
  ```

- A comment after the operator disappears.

  ```text
  -- input
  x = (1 + /* after the operator */ 2)
  -- output
  x = (1 + 2)
  ```

- A multi-line condition is joined onto one line,
   and a trailing comment inside it disappears.

  ```text
  -- input
  x = [
    for p in ["x", "y"] : p
    if p != ""
    && p != "z" # trailing comment
    && length(p) > 0
  ]
  -- output
  x = [
    for p in ["x", "y"] : p
    if p != "" && p != "z" && length(p) > 0
  ]
  ```

- A parenthesized multi-line expression is joined.

  ```text
  -- input
  x = (
    true
    || false
  )
  -- output
  x = (
    true || false
  )
  ```

- Input written without spaces keeps its shape:
   `x = 1+2` round-trips unchanged,
   because an empty decor is stored as `RawStringInner::Empty` rather than as a span.

## Root cause

The parser stores whitespace and comments as byte spans into the input
and converts them to owned strings in a second pass called `despan`.
`BinaryOp::despan` converts the node decor and both operand expressions,
but not the decor of the operator itself:

```rust
// hcl-edit-0.9.7/src/expr/operation.rs:77-81
    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.lhs_expr.despan(input);
        self.rhs_expr.despan(input);
    }
```

The operator is a `Decorated<BinaryOperator>`
(`hcl-edit-0.9.7/src/expr/operation.rs:53`),
so it carries a prefix and suffix decor that the parser did fill with spans:

```rust
// hcl-edit-0.9.7/src/parser/expr.rs:316-333, binary_ops
        repeat(
            1..,
            (
                decorated(
                    ws_or_sp(state),
                    binary_operator.map(Decorated::new),
                    ws_or_sp(state),
                ),
                expr_term_with_state(state),
            ),
        )
```

Because `despan` never runs on that decor,
the two `RawString`s stay in the `Spanned` variant after parsing.
Printing a `Spanned` raw string writes the caller's default instead of the source text:

```rust
// hcl-edit-0.9.7/src/raw_string.rs:42-52
    pub(crate) fn encode_with_default(
        &self,
        buf: &mut EncodeState,
        default: &str,
    ) -> std::fmt::Result {
        if let RawStringInner::Spanned(_) = self.0 {
            buf.write_str(default)
        } else {
            buf.write_str(self.as_str())
        }
    }
```

and the encoder for a binary operation passes `BOTH_SPACE_DECOR`,
a single space on each side:

```rust
// hcl-edit-0.9.7/src/encode/expr.rs:243-249
impl Encode for BinaryOp {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.lhs_expr.encode_decorated(buf, NO_DECOR)?;
        self.operator.encode_decorated(buf, BOTH_SPACE_DECOR)?;
        self.rhs_expr.encode_decorated(buf, NO_DECOR)
    }
}
```

So every byte the author wrote around an operator,
comments included,
is replaced by `" "`.
Sibling nodes do not have the bug:
`Traversal::despan` walks its operators
(`hcl-edit-0.9.7/src/expr/traversal.rs:33-39`)
and `ObjectItem::despan` walks its key and value
(`src/expr/object.rs:174-181`).

An earlier reading,
recorded in `stack-declarative-config.md`,
was that the printer deliberately normalizes multi-line conditions.
It is wrong:
the printer has no normalization pass at all,
and the same code path also deletes comments,
which a formatting choice would not do.

## Verification

- Version under test:
   `hcl-edit` 0.9.7 from crates.io,
   archive SHA-256 `8a67cc5751cb5996669b9780cd738d4541a76c2d5f2f10c17af4965c5a03a5f5`,
   and the upstream clone at commit `f7b17594f8d738343508f87dc9a24018e7fbeb8d`.
- Toolchain:
   `nightly-2026-09-12` (`rustc 1.100.0-nightly`).
- Isolation:
   every build and run happened in `podman run --rm --init --memory=2g --cpus=2 --pids-limit=512
   --network=none` with no credentials,
  no home directory,
  and no repository mount.

Harness:

```rust
// ~/temp/agent/hcl-evaluator-2026-09-17/lab/proto-harness/src/main.rs (abbreviated)
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = std::fs::read_to_string(&args[1]).expect("cases");
    // cases are separated by lines of `=== <name>`
    for (name, body) in cases(&source) {
        match hcl_edit::parser::parse_body(&body) {
            Ok(parsed) => {
                let printed = parsed.to_string();
                println!("=== {name} identical={}", printed == body);
                print!("{printed}");
            }
            Err(error) => println!("=== {name} parse error {error}"),
        }
    }
}
```

Input file:

```hcl
# ~/temp/agent/hcl-evaluator-2026-09-17/corpus/operator-cases.txt
=== comment_before_operator
x = (1 /* inside operands */ + 2)
=== comment_after_operator
x = (1 + /* after the operator */ 2)
=== multiline_for_condition
x = [
  for p in ["x", "y"] : p
  if p != ""
  && p != "z" # trailing comment
  && length(p) > 0
]
=== multiline_parenthesis
x = (
  true
  || false
)
=== plain_spacing
x = 1+2
```

Output before the fix
(`data/logs/proto-operator-build-pre.log` in the research scratchpad):

```text
=== comment_before_operator identical=false
=== comment_after_operator identical=false
=== multiline_for_condition identical=false
=== multiline_parenthesis identical=false
=== plain_spacing identical=true
```

Patterns that round-trip cleanly:

- Any expression with no binary operator.
- A binary operation whose operands are written without any whitespace (`1+2`),
   or with exactly one space on each side,
  which is what the default prints anyway.
- Comments anywhere else:
   before an attribute,
  after a value,
  inside arrays and objects,
  and inside heredocs.

Patterns that lose bytes:

- Block comments and line comments on either side of any binary operator.
- Newlines around a binary operator inside brackets,
  parentheses,
  or a `for` condition,
   where HCL allows multi-line expressions.

Corpus scale:
over 2,246 HCL files from the OpenTofu,
HashiCorp HCL,
hcl-rs,
tofu-ls,
and hcl-lang repositories,
this defect changed one file,
because production Terraform rarely puts comments next to operators;
the meow configuration draft written for this research hits it on two of its expressions.

## Verified workarounds

- Compare before writing.
   Parse,
  print,
  and refuse to write when `printed != input` unless the caller asked for a rewrite.
  Tradeoff:
   turns silent data loss into a refusal,
   but blocks legitimate edits of any file that contains a comment beside an operator,
   so it needs the fix or a formatter to make progress.
- Do not use `hcl-edit` as the writer.
   Keep `hcl-edit` for reading (spans and values) and write files through a separate formatter
   or through byte-range splicing of the original text.
  Tradeoff:
   two code paths over one grammar,
   and splices must recompute offsets after each edit.
- Vendor or fork the crate with the patch under "Upstream filing artifact".
  Tradeoff:
   a fork to maintain,
   and a crates.io release of the fork is needed before a crate that depends on it can be published.

## What does not work

- Setting the decor by hand after parsing
   (`op.operator.decor_mut().set_prefix(...)`)
   does not recover the lost text:
   by then the `Spanned` range is still there,
   but the caller has no supported way to read the source slice,
   since `RawString::span` is `pub(crate)`
   (`hcl-edit-0.9.7/src/raw_string.rs:27`).
- Calling `Body::despan` from outside the crate is impossible:
   `despan` is `pub(crate)` on every node type,
   and `parse_body` already calls it
   (`hcl-edit-0.9.7/src/parser/mod.rs:37-41`).
- Avoiding the parser's `to_string` by serializing through `hcl-rs`
   (`hcl::format::to_string`)
   loses every comment in the document,
  not only those beside operators,
   so it is strictly worse.

## Upstream filing artifact

### Upstream filing decision

1.  Is it really upstream's fault?
     Yes.
    The crate's stated purpose is "Parse and modify HCL while preserving comments and whitespace"
     (`hcl-edit` README,
    first line),
     and the maintainer states the crate follows the HCL specifications
     (`martinohmann/hcl-rs` issue #304,
    comment of 2024-04-09).
    Losing comments on an unedited round trip contradicts that purpose.
2.  Can upstream fix it?
     Yes.
    The fix is one call in `BinaryOp::despan`;
     the same call already exists in sibling node types.
3.  Are they supporting this use case?
     Yes.
    `crates/hcl-edit/src/parser/tests.rs` contains `roundtrip_expr` and `roundtrip_body` tests
     whose whole purpose is byte-for-byte round trips,
     including comment cases such as `"{ foo = 1 /*comment*/ }"`.
4.  Would the repo welcome our contribution?
     Yes.
    `CONTRIBUTING.md` invites pull requests,
     asks for an issue first for features and breaking changes,
     asks for regression tests with bug fixes,
     and says nothing about AI assistance;
     `.github` holds no issue or pull request template.
    External bug-fix pull requests were merged recently
     (#549 on 2026-08-09 and #562 on 2026-08-20).
5.  Will they likely fix it?
     Likely.
    No documented non-goal exists,
     the maintainer merged comparable parser fixes within days,
     and there is no won't-fix statement.
    The open issue #566 from 2026-09-04 has no maintainer reply yet,
     which is silence rather than a decline.
6.  Have we prototyped a minimal fix compatible with their architecture?
     Yes,
    see the diff and its verification.

### Duplicate search

Searched `martinohmann/hcl-rs` issues and pull requests,
open and closed,
on 2026-09-17
(`gh search issues` and `gh search prs`,
`--limit 100`,
recorded in
`data/dup-search.jsonl` in the research scratchpad):
`operator comment`,
`operator whitespace`,
`operator newline`,
`binary operator decor`,
`despan`,
`multiline condition`,
`roundtrip comment`,
`round trip`.
No hit describes this behavior.
The closest are #466 and its fix #467,
where a line comment after a binary operator made parsing fail,
which is a different failure mode and is already released;
and #567,
about object keys not surviving `to_string` plus `parse`,
which is a different node type.

### Prototype fix and verification

Applied in a disposable clone of `martinohmann/hcl-rs` at `f7b17594f8d738343508f87dc9a24018e7fbeb8d`
with `git remote set-url --push origin DISABLED`:

```diff
--- a/crates/hcl-edit/src/expr/operation.rs
+++ b/crates/hcl-edit/src/expr/operation.rs
@@
-use crate::{Decor, Decorated, Spanned};
+use crate::{Decor, Decorate, Decorated, Spanned};
@@ impl BinaryOp {
     pub(crate) fn despan(&mut self, input: &str) {
         self.decor.despan(input);
         self.lhs_expr.despan(input);
+        self.operator.decor_mut().despan(input);
         self.rhs_expr.despan(input);
     }
```

plus three inputs added to the existing `roundtrip_expr` test:

```rust
// crates/hcl-edit/src/parser/tests.rs
        "(1 /* before */ + /* after */ 2)",
        "(\n  true\n  || false\n)",
        "[for p in xs : p\n  if p != \"\"\n  && p != \"z\" # trailing\n  && length(p) > 0\n]",
```

Verification,
both runs in the bounded container:

- Without the source change and with the new test inputs:
   `cargo test --offline --locked -p hcl-edit --lib` printed
   `test parser::tests::roundtrip_expr ... FAILED`
   and `test result: FAILED. 13 passed; 1 failed`.
- With the source change:
   `cargo test --offline --locked --workspace` printed `ok` for all 23 test binaries,
   including the crate's specsuite runner,
   with no failures.
- The five-case harness above reported `identical=true` for all five cases after the change.

### Draft issue

~~~md
Title: [bug]: comments and newlines around binary operators are lost on round trip

Labels: bug

### Description of the bug

`parse_body` followed by `to_string` replaces everything the author wrote around a binary
operator with a single space. Comments next to the operator are deleted, and multi-line
conditions are joined onto one line. No error is returned and the result still parses.

```rust
let input = "x = (1 /* keep me */ + 2)\n";
let body = hcl_edit::parser::parse_body(input).unwrap();
assert_eq!(body.to_string(), input); // fails: "x = (1 + 2)\n"
```

More cases, all of which change:

```hcl
x = (1 + /* after the operator */ 2)
y = [
  for p in ["x", "y"] : p
  if p != ""
  && p != "z" # trailing comment
  && length(p) > 0
]
z = (
  true
  || false
)
```

### Root cause

`BinaryOp::despan` (crates/hcl-edit/src/expr/operation.rs) converts the node decor and both
operands but not `self.operator`, which is a `Decorated<BinaryOperator>` whose decor the parser
filled with spans in `binary_ops` (crates/hcl-edit/src/parser/expr.rs). A `RawString` still in the
`Spanned` variant prints `encode_with_default`'s default, and `impl Encode for BinaryOp` passes
`BOTH_SPACE_DECOR`, so the original bytes are replaced by one space on each side.
`Traversal::despan` and `ObjectItem::despan` do walk their sub-nodes, so this looks like an
oversight rather than a design choice.

### Suggested fix

```diff
--- a/crates/hcl-edit/src/expr/operation.rs
+++ b/crates/hcl-edit/src/expr/operation.rs
@@
-use crate::{Decor, Decorated, Spanned};
+use crate::{Decor, Decorate, Decorated, Spanned};
@@ impl BinaryOp {
     pub(crate) fn despan(&mut self, input: &str) {
         self.decor.despan(input);
         self.lhs_expr.despan(input);
+        self.operator.decor_mut().despan(input);
         self.rhs_expr.despan(input);
     }
```

Suggested regression inputs for `parser::tests::roundtrip_expr`:

```rust
"(1 /* before */ + /* after */ 2)",
"(\n  true\n  || false\n)",
"[for p in xs : p\n  if p != \"\"\n  && p != \"z\" # trailing\n  && length(p) > 0\n]",
```

With the three inputs and without the fix, `cargo test -p hcl-edit --lib` fails in
`parser::tests::roundtrip_expr`; with the fix, `cargo test --workspace` passes.

This report was prepared with AI assistance; the reproduction, the source trace, the patch, and
the test runs above were executed and checked before filing.
~~~
