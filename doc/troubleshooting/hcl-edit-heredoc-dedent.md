# hcl-edit 0.9.7: an indented heredoc with a mid-line interpolation loses text and changes value

An `<<-` heredoc whose first line starts at column zero and whose later lines are indented
(the shape produced by a template that interpolates in the middle of a line)
comes out of `hcl-edit` with characters cut off the front of every literal line.
The damage is not only to formatting:
the evaluated string changes too,
and the printed document silently carries the changed text.
A second,
milder variant drops the `-` from `<<-` when nothing can be stripped.

Found while designing meow's HCL front end
(`doc/planning/monorepo-manager-from-scratch-design.md`,
"Declarative configuration").

Status 2026-09-17:
unchanged at upstream `HEAD` `f7b17594f8d738343508f87dc9a24018e7fbeb8d` (2026-09-06),
reproduced there and in the published crate `hcl-edit` 0.9.7.

## Symptom

Silent in both variants.

- Text loss and value change.
   The first literal line is deleted from the printed document,
   and the string the document evaluates to changes:

  ```text
  -- input
  x = <<-EOT
  top
      mid ${"i"}
      deep
  EOT
  -- output of parse_body followed by to_string
  x = <<-EOT

      mid ${"i"}
      deep
  EOT
  ```

  The HashiCorp reference implementation evaluates the input to `"top\n    mid i\n    deep\n"`
  and the output to `"\nmid i\ndeep\n"`.
  `hcl-rs` 0.19.8,
  which parses through this crate,
  evaluates even the input to `"\nmid i\ndeep\n"`,
  so a consumer that only evaluates,
  without printing,
  also sees the wrong string.

- Real-world shape.
   The same input shape appears in OpenTofu's own test fixtures.
   In `internal/backend/remote-state/azure/meta-test/ado/main.tf`,
   a YAML pipeline embedded in an `<<-EOT` heredoc with three `${...}` interpolations
   comes back with sixteen characters cut from most lines
   (`pool:` becomes empty,
  `  vmImage: ubuntu-latest` becomes `u-latest`).

- Marker loss.
   A `<<-` heredoc whose lines have no common indentation is printed as `<<`:

  ```text
  -- input
  x = <<-EOT
  one
    two
  EOT
  -- output
  x = <<EOT
  one
    two
  EOT
  ```

  The value is unchanged,
  but the document differs from the author's text and from what the reference formatter emits.

- Tab indentation is replaced by the same number of spaces.
   `<<-EOT` with tab-indented lines prints with one space per stripped tab.
   The value is unchanged,
  because the indentation is stripped at evaluation.

## Root cause

An indented heredoc is normalized at parse time:
the parser strips the common indent from the literal parts and remembers how much it stripped,
then the encoder re-indents by that amount.

```rust
// hcl-edit-0.9.7/src/parser/expr.rs:686-706 (abbreviated)
        let mut heredoc = HeredocTemplate::new(Ident::new_unchecked(delim), template);

        if indented {
            heredoc.dedent();
        }
```

`Template::dedent` folds a minimum over the literal parts of the template:

```rust
// hcl-edit-0.9.7/src/template/mod.rs:404-435 (abbreviated)
    pub(crate) fn dedent(&mut self) -> Option<usize> {
        let mut indent: Option<usize> = None;
        let mut skip_first_line = false;

        for element in &self.elements {
            if let Element::Literal(literal) = element {
                if let Some(leading_ws) = min_leading_whitespace(literal, skip_first_line) {
                    indent = Some(indent.map_or(leading_ws, |indent| indent.min(leading_ws)));
                }
                skip_first_line = !literal.ends_with('\n');
            } else if !skip_first_line {
                // Directive or interpolation at line start always mean that no indent can be
                // stripped.
                return None;
            }
        }
```

The helper it folds over returns `None` for two different facts:
"this literal has no lines to measure" and "one of its lines starts at column zero".

```rust
// hcl-edit-0.9.7/src/util.rs:26-54 (abbreviated)
pub(crate) fn min_leading_whitespace(s: &str, skip_first: bool) -> Option<usize> {
    if s.is_empty() {
        return None;
    }
    ...
        if line_leading_ws == 0 {
            // Fast path: no dedent needed if we encounter a non-empty line which starts with a
            // non-whitespace character.
            return None;
        }
```

`Template::dedent` treats both as "no information",
so a literal containing a column-zero line does not pin the minimum to zero.
A template is split into several literals wherever an interpolation appears,
so in

```hcl
x = <<-EOT
top
    mid ${"i"}
    deep
EOT
```

the first literal (`"top\n    mid "`) returns `None` because of `top`,
and the second literal (`"\n    deep\n"`) returns `Some(4)`.
The fold therefore chooses 4 and dedents every literal by 4,
including the one it had just declined to measure.
The dedent step then removes characters by position,
not by class:

```rust
// hcl-edit-0.9.7/src/util.rs:3-14 (abbreviated)
pub(crate) fn dedent_by(s: &str, n: usize, skip_first: bool) -> Cow<'_, str> {
    for (i, line) in s.lines().enumerate() {
        if i == 0 && skip_first {
            dedented.push_str(line);
        } else if !line.is_empty() {
            dedented.extend(line.chars().skip(n));
        }
```

`"top".chars().skip(4)` is empty,
so `top` is deleted from both the stored template and the value.
With a longer line,
as in the OpenTofu fixture,
the first characters of the content are eaten.

The marker variant has a different immediate cause:
the encoder decides between `<<` and `<<-` from whether an indent was recorded,

```rust
// hcl-edit-0.9.7/src/encode/template.rs:32-50 (abbreviated)
        buf.write_str("<<")?;

        if self.indent().is_some() {
            buf.write_char('-')?;
        }
```

so a `<<-` heredoc that stripped nothing is indistinguishable from a plain `<<` heredoc after parsing.

For comparison,
the reference implementation counts a line that starts with a non-space
as zero spaces and folds it into the minimum,
and it counts grapheme clusters rather than characters:

```go
// hcl/hclsyntax/parser_template.go:710-739 (abbreviated)
	for _, ttok := range parts.Tokens {
		if newline {
			newline = false
			var spaces int
			if lit, ok := ttok.(*templateLiteralToken); ok {
				orig := lit.Val
				trimmed := strings.TrimLeftFunc(orig, unicode.IsSpace)
				if len(trimmed) == 0 && strings.HasSuffix(orig, "\n") {
					spaces = maxInt
				} else {
					spaceBytes := len(lit.Val) - len(trimmed)
					spaces, _ = unicodeutil.GraphemeCount([]byte(orig[:spaceBytes]))
					adjust = append(adjust, lit)
				}
			}
			...
			if spaces < minSpaces {
				minSpaces = spaces
			}
```

An earlier reading of the same round-trip failures,
recorded in `doc/planning/monorepo-manager-route-research/stack-declarative-config.md`,
attributed the changed files to formatting preferences.
That is wrong for this case:
the byte that changes is inside a string value,
and the value changes with it.

## Verification

- Version under test:
   `hcl-edit` 0.9.7 from crates.io,
   archive SHA-256 `8a67cc5751cb5996669b9780cd738d4541a76c2d5f2f10c17af4965c5a03a5f5`,
   and the upstream clone at `f7b17594f8d738343508f87dc9a24018e7fbeb8d`.
- Reference implementation used as the oracle:
   `hashicorp/hcl` at `4932c1452af6` with `zclconf/go-cty` at `a918e1174fcf`,
   driven by a small Go program that parses each case with `hclsyntax.ParseConfig`
   and prints the evaluated attribute values as JSON.
- Toolchain:
   `nightly-2026-09-12`,
  Go 1.27.1.
- Isolation:
   builds and runs in `podman run --rm --init --memory=2g --cpus=2 --pids-limit=512 --network=none`
   with no credentials,
  no home directory,
  and no repository mount.

Input file:

```hcl
# ~/temp/agent/hcl-evaluator-2026-09-17/corpus/heredoc-cases.txt
=== flush_line_then_indented_interpolation
x = <<-EOT
top
    mid ${"i"}
    deep
EOT
=== all_indented_with_interpolation
x = <<-EOT
    one ${"i"}
    two
    EOT
=== indented_no_interpolation
x = <<-EOT
    one
      two
    EOT
=== flush_no_interpolation
x = <<-EOT
one
  two
EOT
=== tab_indent
x = <<-EOT
	tabbed
	lines
	EOT
```

Round trip through `hcl-edit` 0.9.7 (`parse_body` then `to_string`):

```text
=== flush_line_then_indented_interpolation identical=false   (first line deleted)
=== all_indented_with_interpolation identical=true
=== indented_no_interpolation identical=true
=== flush_no_interpolation identical=false                   (<<- printed as <<)
=== tab_indent identical=false                               (tabs printed as spaces)
```

Values,
reference implementation on the input versus on the round-tripped output:

```text
input  : x = "top\n    mid i\n    deep\n"
output : x = "\nmid i\ndeep\n"
```

`hcl-rs` 0.19.8 evaluating the input directly returns `"\nmid i\ndeep\n"`,
matching the damaged output rather than the reference.
The four other cases evaluate identically before and after the round trip.

Patterns that work cleanly:

- Every line of the heredoc indented by at least one space,
  with or without interpolations.
- A plain `<<` heredoc,
  with or without interpolations.
- An `<<-` heredoc with no interpolations at all,
   because then the template has a single literal and the fold sees the column-zero line.

Patterns that fail:

- `<<-` plus an interpolation that starts in the middle of a line,
   plus at least one column-zero literal line and at least one indented literal line:
   characters are cut from the front of lines.
- `<<-` where no common indent exists:
   the marker becomes `<<`.
- `<<-` indented with tabs:
   tabs become spaces,
  one space per tab.

Corpus scale:
across 2,246 HCL files from OpenTofu,
HashiCorp HCL,
hcl-rs,
tofu-ls,
and hcl-lang,
`hcl-edit` printed 2,153 files byte for byte,
27 differed only by CRLF normalization,
and of the five genuine differences four are heredoc cases from this cluster
(one of them the OpenTofu fixture with cut content),
the fifth being the binary-operator defect in
`doc/troubleshooting/hcl-edit-binary-operator-decor.md`.

## Verified workarounds

- Compare before writing.
   Parse,
  print,
  and refuse to write when the printed text differs from the input.
  Tradeoff:
   catches the damage,
  but blocks edits to any file containing such a heredoc.
- Reject `<<-` heredocs that contain interpolations in consumer validation,
   or normalize them to plain `<<` with already-stripped content before storing.
  Tradeoff:
   rejects valid HCL that the reference implementation accepts,
   and rewriting the heredoc changes the author's file.
- Do not trust `hcl-edit` or `hcl-rs` values for indented heredocs:
   read the heredoc's source span and strip the indent with the reference rule
   (minimum over all lines,
  counting a column-zero line as zero).
  Tradeoff:
   the consumer re-implements part of the template semantics.
- Vendor or fork the crate with the patch under "Upstream filing artifact".
  Tradeoff:
   a fork to maintain,
   and publishing a crate that depends on the fork needs the fork on crates.io.

## What does not work

- Passing the heredoc through `hcl-rs`'s formatter instead:
   `hcl::format::to_string` re-emits the damaged template,
   because the damage happened during parsing,
  and it also drops every comment in the document.
- Using `HeredocTemplate::set_indent` after parsing to restore the marker:
   the deleted characters are already gone from the template literals,
   and re-indenting adds spaces rather than the original text.
- Treating the case as a formatting preference and normalizing with a formatter:
   the reference formatter `hclwrite.Format` changes whitespace only outside heredocs,
   and the OpenTofu fixture shows lost content,
  not lost formatting.

## Upstream filing artifact

### Upstream filing decision

1.  Is it really upstream's fault?
     Yes.
    The crate parses `<<-` heredocs itself and states that it follows the HCL specifications
     (`martinohmann/hcl-rs` issue #304,
    maintainer comment of 2024-04-09);
     the specification defines the stripped indent as the minimum over all lines.
2.  Can upstream fix it?
     Yes.
    The fix is two edits:
    report zero instead of `None` for a column-zero line,
     and keep the `<<-` marker when nothing is stripped.
3.  Are they supporting this use case?
     Yes.
    `crates/hcl-edit/src/template/tests.rs::dedent_template` and the crate's specsuite runner
     cover `<<-` heredocs,
    including templates with interpolations.
4.  Would the repo welcome our contribution?
     Yes.
    `CONTRIBUTING.md` invites pull requests,
    asks for a regression test with each fix,
     and says nothing about AI assistance;
     external fixes were merged in 2026-07 and 2026-08 (#549,
    #562).
5.  Will they likely fix it?
     Likely.
    Heredoc indentation has been fixed several times before
     (#8,
    #11,
    #15,
    #157),
     which shows the maintainer treats these as bugs;
     no won't-fix statement exists.
6.  Have we prototyped a minimal fix compatible with their architecture?
     Yes,
    see the diff and its verification.

### Duplicate search

Searched `martinohmann/hcl-rs` issues and pull requests,
open and closed,
on 2026-09-17
(recorded in `data/dup-search.jsonl` in the research scratchpad):
`heredoc indent`,
`heredoc dedent`,
`heredoc interpolation`,
`<<-`,
`strip indent`,
`round trip`.
The only hits are the older merged fixes
(#8 "Fix heredoc handling to match HCL spec",
#11 "strip indent from `<<-` heredocs as defined in the HCL spec",
#15 "correctly handle non-space indent in heredocs",
#157 "automatically dedent indented heredocs",
#512 "prevent heredoc delimiter prefix matching against longer identifier"),
none of which covers a template split by an interpolation.
No open issue describes this behavior.

### Prototype fix and verification

Applied in a disposable clone of `martinohmann/hcl-rs` at `f7b17594f8d738343508f87dc9a24018e7fbeb8d`
with `git remote set-url --push origin DISABLED`:

```diff
--- a/crates/hcl-edit/src/util.rs
+++ b/crates/hcl-edit/src/util.rs
@@ pub(crate) fn min_leading_whitespace(s: &str, skip_first: bool) -> Option<usize> {
         if line_leading_ws == 0 {
-            // Fast path: no dedent needed if we encounter a non-empty line which starts with a
-            // non-whitespace character.
-            return None;
+            // Fast path: no dedent is possible if we encounter a non-empty line which starts with
+            // a non-whitespace character. Report zero rather than `None`, so callers combining
+            // several literals do not mistake this line for "no information".
+            return Some(0);
         }
--- a/crates/hcl-edit/src/parser/expr.rs
+++ b/crates/hcl-edit/src/parser/expr.rs
@@
         if indented {
             heredoc.dedent();
+
+            // Keep the `<<-` marker on output even when no indent could be stripped.
+            if heredoc.indent().is_none() {
+                heredoc.set_indent(0);
+            }
         }
```

plus regression inputs:

```rust
// crates/hcl-edit/src/template/tests.rs, dedent_template
        ("top\n    mid ${i}\n    deep\n", "top\n    mid ${i}\n    deep\n"),
// crates/hcl-edit/src/parser/tests.rs, roundtrip_expr
        "<<-EOT\none\n  two\nEOT",
        "<<-EOT\ntop\n    mid ${i}\n    deep\nEOT",
```

Verification,
both runs in the bounded container:

- Without the source changes and with the new test inputs:
   `cargo test --offline --locked -p hcl-edit --lib` printed
   `test parser::tests::roundtrip_expr ... FAILED`,
   `test template::tests::dedent_template ... FAILED`,
   and `test result: FAILED. 12 passed; 2 failed`.
- With the source changes:
   `cargo test --offline --locked --workspace` printed `ok` for all 23 test binaries with no failures,
   the crate's specsuite runner included.
- The five-case harness then printed `identical=true` for four cases,
   with only the tab case still differing (tabs printed as spaces,
  value unchanged),
   and the reference implementation evaluated every round-tripped case to the same value as its input.

The tab difference is left alone on purpose:
fixing it means storing the stripped prefix instead of a count,
which is a larger change than the defect this report is about.

### Draft issue

~~~md
Title: [bug]: `<<-` heredoc with a mid-line interpolation loses characters and changes its value

Labels: bug

### Description of the bug

When an indented heredoc contains an interpolation in the middle of a line, the dedent step picks
an indent from one literal part and applies it to a literal part that starts at column zero. The
leading characters of those lines are removed, so both the printed document and the evaluated
string change.

```rust
let input = "x = <<-EOT\ntop\n    mid ${\"i\"}\n    deep\nEOT\n";
let body = hcl_edit::parser::parse_body(input).unwrap();
println!("{body}");
// x = <<-EOT
//
//     mid ${"i"}
//     deep
// EOT
```

The reference implementation evaluates the input to `"top\n    mid i\n    deep\n"`; hcl-rs 0.19.8
evaluates the same input to `"\nmid i\ndeep\n"`. A real-world instance is OpenTofu's
`internal/backend/remote-state/azure/meta-test/ado/main.tf`, where a YAML pipeline inside
`<<-EOT` loses sixteen characters per line.

A second, milder case: a `<<-` heredoc with no common indent prints as `<<`.

```rust
let input = "x = <<-EOT\none\n  two\nEOT\n";
// prints "x = <<EOT\none\n  two\nEOT\n"
```

### Root cause

`min_leading_whitespace` (crates/hcl-edit/src/util.rs) returns `None` both for "nothing to
measure" and for "this line starts at column zero". `Template::dedent`
(crates/hcl-edit/src/template/mod.rs) folds a minimum over the literal parts and skips `None`, so
a column-zero line in one literal does not pin the minimum to zero when another literal is
indented. `dedent_by` then removes `n` characters regardless of whether they are whitespace.

For comparison, `hclsyntax`'s `flushHeredocTemplateParts` counts a line that starts with a
non-space as zero and folds it into the minimum.

### Suggested fix

```diff
--- a/crates/hcl-edit/src/util.rs
+++ b/crates/hcl-edit/src/util.rs
@@
         if line_leading_ws == 0 {
-            return None;
+            return Some(0);
         }
--- a/crates/hcl-edit/src/parser/expr.rs
+++ b/crates/hcl-edit/src/parser/expr.rs
@@
         if indented {
             heredoc.dedent();
+
+            // Keep the `<<-` marker on output even when no indent could be stripped.
+            if heredoc.indent().is_none() {
+                heredoc.set_indent(0);
+            }
         }
```

Suggested regression cases:

```rust
// crates/hcl-edit/src/template/tests.rs, dedent_template
("top\n    mid ${i}\n    deep\n", "top\n    mid ${i}\n    deep\n"),
// crates/hcl-edit/src/parser/tests.rs, roundtrip_expr
"<<-EOT\none\n  two\nEOT",
"<<-EOT\ntop\n    mid ${i}\n    deep\nEOT",
```

Without the fix those cases fail; with it, `cargo test --workspace` passes, including the
specsuite runner.

One difference stays after this patch: tab indentation is re-emitted as spaces, because the
heredoc records a count rather than the stripped prefix. That is value-preserving and looks like a
separate change.

This report was prepared with AI assistance; the reproduction, the source trace, the patch, and
the test runs above were executed and checked before filing.
~~~
