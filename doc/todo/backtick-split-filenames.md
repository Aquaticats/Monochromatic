# Backtick bare filenames split by semantic line breaks

## Problem

Filenames written bare in prose contain a period,
 and `semantic-line-breaks` used to break after every break-point character whatever followed it,
 so the autofix split them mid-token:
 `AGENTS.md` became `AGENTS.` at the end of a line with `md` starting the next.

**The cause is fixed.**
The rule now breaks only where a written word ends,
 so the character after the break-point character has to be a space,
 a tab,
 a newline,
 or the end of the prose.
A dot inside `AGENTS.md`,
 `crates.io`,
 `Node.js`,
 `checker.TupleType` or `9.0.0-rc.3` is followed by a letter or a digit and is left alone.
`package/cli/markdown-lint/src/semantic-break-points.ts` carries the guard;
 `semantic-line-breaks.unit.test.ts` pins each of those five shapes.

**The artifacts remain.**
Every place the old fixer ran still holds a split token,
 and nothing removes one except editing it.
Backticking is still the style rule (`AGENTS.md` WRP) because an inline code span is skipped by every prose rule,
 not only by this one.

## Scale

Measured 2026-07-31 over tracked Markdown,
 pairing a line ending in `<word>.` with a next line opening on an extension that is never an ordinary English word
 (`md`,
`ts`,
`tsx`,
`mjs`,
`cjs`,
`json`,
`jsonc`,
`toml`,
`yaml`,
`yml`,
`rs`,
`css`,
`svg`,
`tgz`,
`pkl`,
`ini`,
`plist`):

```text
524 artifacts across 111 files
```

`doc/planning/` is already cleared:
 its 27 were rejoined and backticked when the family's line-break debt was cleared.
The largest remaining concentrations are `doc/decision/ios-iphone-x-vet-report/`,
 `doc/audit/em-dash.md` and `doc/philosophy/agents.md`.

A wider extension set (adding `d`,
`test`,
`config`,
`local`,
`log`,
`io`,
`com`,
`dev`,
`net`,
`org`) reaches 927 across 174 files,
 but those extensions are also ordinary words and sentence openings,
 so that count needs reading rather than trusting.

## Detection

```bash
rg --line-number --glob '*.md' --glob '!node_modules' --glob '!CLAUDE.md' '(AGENTS|CLAUDE|README|SKILL)\.$' .
```

That pattern only covers the most common filenames.
Other split tokens turn up by pairing lines ending in `[A-Za-z]\.` with continuation lines starting
with an extension-like fragment:

```bash
rg --line-number --glob '*.md' --glob '!node_modules' --glob '!CLAUDE.md' '^ ?(md|ts|json|toml|txt|yaml|lock)\b' .
```

Inspect matches manually;
 short words like `log` and `ts` also start legitimate prose lines.

## Fix recipe

1.  Join the split token onto one line and wrap it in backticks in the source file.
2.  Run `mise run format:markdown <file>` and confirm the reflow is stable,
     then `mise run lint:markdown <file>`.
3.  When the file feeds a generated output (for example `AGENTS.md` feeds `CLAUDE.md`),
     run `mise run sync:files` and commit both.

A doubly split token needs more than one join.
`doc/planning/extract-refactor-guardrail.md` held `AGENTS.md / CLAUDE.md` spread over three lines,
 so a joiner has to re-examine the line it just produced rather than move on to the next input line.

## Done when

The detection commands return no filename-shaped fragments across tracked Markdown files.

## Reading the diagnostic: the reported position used to be the paragraph

**Fixed,
 and recorded because the failure mode generalises.**
`semantic-line-breaks` used to anchor its diagnostic at the text node,
 so it reported the first line of the paragraph containing the violation,
 with a column that could sit past the end of that line.
The break-point character actually needing a break could be several lines below.

Measured on `doc/planning/prefer-readonly-return-substitution.md`.
The report read `946:1`,
 where line 946 was a short complete sentence with no break-point character except its closing period,
 and the column was one past the line's own length.
The offence was on line 950,
 an ordinary sentence carrying two mid-line commas.

The reason the position misled so convincingly is that it moved when you edited nearby text,
 which reads exactly like feedback about the edit.
Rewording line 946 moved the column;
 backticking a path on the following line moved it again;
 neither touched the cause.

What converged was bisection on the paragraph rather than inspection of the reported line:
 delete the whole added section,
 confirm the file lints clean,
 then re-add it in halves until the report returns.
That found the real line in three rounds after roughly a dozen rounds of editing the reported one.

The rule now anchors each diagnostic at the offset the break goes in,
 so the reported line is the offending line.
The general lesson survives the fix:
 **a diagnostic anchored at a container reports the container,
 and a reader will edit what it names.**
When a rule's violation is one character inside a long node,
 anchoring at the node is a defect in the rule rather than a detail of its output.
