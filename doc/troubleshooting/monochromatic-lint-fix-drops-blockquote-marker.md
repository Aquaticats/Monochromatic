# `monochromatic-lint --fix` drops the blockquote marker when it splits a quoted line

## Symptom

Running `mise run //:format:monochromatic-lint -- <path>` over a Markdown file whose blockquote carries
more than one clause leaves the continuation lines without their `>` marker:

```markdown
> Moved 2026-10-29 by the package categorization and `113633b259`
> paths and names below this note are historical.
```

becomes

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
When the split line belongs to a blockquote,
the rule writes the new line without the `>` prefix it inherited.

Probed on 2026-11-08 with three fixtures under `doc/handover/`:
a one-line blockquote,
a hand-wrapped blockquote,
and a blockquote with a comma inside a sentence.
The last two gained unmarked continuation lines;
the same text with one clause per line and no internal comma came back unchanged.

## Workaround

Write the quoted text one clause per line and keep commas,
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

The workaround constrains the prose;
the defect belongs in the linter,
whose blockquote branch has to re-emit the marker on every line it splits.
Until that lands,
any handover note or quoted passage that carries a multi-clause line can be mangled by one `--fix` run.

## Verification

Write the probe fixtures listed under Root cause and run the fixer over them;
a fixed linter leaves every `>` marker in place.
