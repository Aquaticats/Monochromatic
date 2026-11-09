# `monochromatic-lint --fix` drops the blockquote marker when it splits a quoted line

Note 2026-11-09T16:56:27-05:00:
`package/cli/forbidden-strings` is now `package/forbidden-strings/cli`,
moved 2026-10-29 in `113633b259`.

## Symptom

Running `mise run //:format:monochromatic-lint -- <path>` over a Markdown file whose blockquote carries
more than one clause left the continuation lines without their `>` marker:

```markdown
> Moved 2026-10-29 by the package categorization and `113633b259`
> paths and names below this note are historical.
```

became

```markdown
> Moved 2026-10-29 by the package categorization
and `113633b259`
> paths and names below this note are historical.
```

The rendered page is unchanged,
because CommonMark reads those lines as lazy blockquote continuation,
but the raw text reads as if the quote ends mid-sentence,
and a later edit that adds or removes a line can break the pairing.

## Root cause

`monochromatic-lint --fix` applies the semantic-line-breaks rule,
which puts one clause on each line,
splitting after a comma,
a semicolon,
a full stop,
or a colon.
When the rule breaks a line,
it writes the new line with the prefix of the paragraph's own opening line,
and `continuation_prefix` in
`package/linter/monochromatic-lint/src/markdown_prose_context.rs`
copied that prefix while blanking every marker character,
so a break inside a blockquote produced a line with no `>`.

Commit `1507b0a483` chose that on purpose:
a line without `>` is a lazy paragraph continuation
(CommonMark 0.31.2,
section 5.1 "Block quotes",
rule 2 "Laziness"),
so the rendered page kept the quotation,
and the whitespace-token property stopped seeing an extra `>` token.
The loss is real in the raw text:
a reader of the source sees the quotation end mid-sentence,
and the next edit to that paragraph decides where the quote resumes.

Probed on 2026-11-08 with three fixtures under `doc/handover/`:
a one-line blockquote,
a hand-wrapped blockquote,
and a blockquote with a comma inside a sentence.
The last two gained unmarked continuation lines;
the same text with one clause per line and no internal comma came back unchanged.

## Fix

Commit `6fc7d68f8e` makes `continuation_prefix` copy `>` and its spacing byte for byte,
and blank only the other marker characters.
Every line the fixer inserts inside a quotation therefore opens at the paragraph's own depth:
one quote,
two or more nested quotes,
a quote inside a list item,
a list inside a quote,
and an ordered item holding a quote all keep their chain.
Nested depths come from the paragraph's opening line,
so a break in a lazy line of a depth-two quote is written back at depth two.

A list marker is still replaced with a space:
writing a list marker such as `-` or `1.` or `[ ]` again would open a new item,
and starting at the item's content column is what continues the current one.
The task marker case aligns to the paragraph's own column,
which is the column after `- [ ] `.

The fixer only changes the line it splits.
An existing lazy line the author wrote keeps its bytes,
so a document that already mixes marked and lazy lines keeps that mix.

The whitespace-token property in
`package/linter/monochromatic-lint/src/run_command_whitespace_fix_tests.rs`
now sets aside line-leading quote markers,
the way it already set aside Rust doc-comment markers,
because a marker is container structure rather than a token a reader sees.
The byte-exact tests and the parsed-document comparisons carry the depth requirement instead.

No lint rule's severity or scope changed;
only the bytes the line-break fixer writes for a continuation line changed.

## Workaround

On a linter older than the release that carries `6fc7d68f8e`,
write the quoted text one clause per line and keep commas,
semicolons and full stops out of the middle of a line,
so the rule has nothing to split.
Repeating `>` on every line then survives the fixer:

```markdown
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli` (2026-10-29 `113633b259`)
> and its tasks are `//package/forbidden-strings/cli:*` today.
```

Keep a date and its commit in one span,
`(2026-10-29 \`113633b259\`)`,
because a break inside the parentheses makes the fixer insert a space before the closing bracket.

## Limits

The workaround constrains the prose and applies only to an older linter.
With the fix any handover note or quoted passage that carries a multi-clause line keeps its markers,
and no version of the fixer re-marks a lazy line the author wrote.
The whitespace-token property alone cannot catch a lost marker,
because a lazy line belongs to the same paragraph;
the byte-exact quotation tests and the paragraph-placement comparison are the guards for that.

## Verification

Write the probe fixtures listed under Root cause and run the fixer over them;
a fixed linter leaves every `>` marker in place.

The property check over a real tree takes every tracked Markdown,
MDX and Rust file in that tree through `--fix` on standard input.
It compares the token sequence,
and it parses every Markdown host before and after to compare each paragraph's container chain and words,
every block node in preorder with its depth,
and the exact source of every code and raw HTML block.
It skips nothing:
a processing failure,
an unreadable file or a host that stops parsing fails the measurement.
The package task `test:property` runs it:

```sh
MONOCHROMATIC_LINT_PROPERTY_ROOT=<copy of a repository tree> \
  mise run //package/linter/monochromatic-lint:test:property
```

Run on 2026-11-08 over a copy of this repository it measured 6825 files,
69 of them changed by the fix,
11830859 tokens compared,
3101 Markdown hosts parsed,
12629 code and raw HTML blocks compared,
and reported no difference in any comparison.
A separate one-off comparison of the same two copies,
using a token split that sets aside quote markers and a pandoc CommonMark render,
found the token sequence equal in all 69 changed files
and the HTML identical for all of them because every changed file here is Markdown.

Removal condition:
delete the Workaround and Limits sections once the `monochromatic-lint` release that contains
`6fc7d68f8e` is published.
The package owner runs that release.
This record stays until it lands.
