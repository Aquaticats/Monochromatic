# Splitting `monochromatic-lint --fix` into whitespace fixes and content fixes

Status:
written 2026-11-07 by the agent that fixed the whitespace defects of `--fix`.
The coordinator picked option A on 2026-11-07 (a coordinator pick,
the owner's veto open),
and the branch `feat/monochromatic-lint-fix-classes-whitespace-and-content` builds it;
see "What option A built".
No rule severity changed.

## Why this exists

The root task `format:monochromatic-lint` (`mise.toml`) runs `monochromatic-lint --fix`,
and agents run the same command before they commit.
The repository-wide lint agent measured on 2026-11-07 that this command rewrites content,
not only whitespace,
and the owner's rules for the situation are:
"Principle 1:
Lint issues need to be taken care of asap.",
"never discard user data"
and "Never loosen lint rules without prior approval."

## How the fixer separates safe fixes from content fixes today

It does not.
`--fix` applies every fix that every enabled rule offers
(`package/linter/monochromatic-lint/src/fix_loop.rs`,
`fix_source`,
collects the `fix` of every finding),
and `--rules` reports only `fixable: true` or `false` per rule,
with no class.
A configuration can only set a severity per rule,
so the only way to run the whitespace fix alone is a scratch configuration that enables that one rule,
which is what the repository-wide lint agent did for its commits.

## What each fixable rule does to tokens

The tokens are the whitespace-separated words of the file.
Measured by reading each rule's edit in `package/linter/monochromatic-lint/src`:

- `markdown/semantic-line-breaks` replaces one space with a line break.
  A continuation line inside a blockquote gains the `>` markers of the paragraph it continues,
  which are container syntax and not a word,
  and a continuation line outside a container gains nothing.
  In a Rust host the continuation line gains the comment marker (`///`,
  `//!`),
  which is comment syntax and not a word.
  Why the markers are written again is recorded in
  `doc/troubleshooting/monochromatic-lint-fix-drops-blockquote-marker.md`.
- `markdown/no-bare-urls` adds `<` and `>` to a URL token,
  or rewrites it as an inline link in MDX.
- `markdown/link-image-style` adds `[]` after a shortcut reference.
- `markdown/no-trailing-punctuation` deletes a `.` or `:` from a heading.
- `markdown/commands-show-output` deletes `$ ` from a fenced block.
- `markdown/fenced-code-language` adds a language word to a fence.
- `markdown/link-image-reference-definitions` deletes definitions that are unused or duplicated.
- `markdown/lfs-image-url` rewrites an image URL.
- `markdown/no-pipe-tables` rewrote a table as HTML until this change removed that fix.

Only the first is a whitespace fix.
The other rules rewrite content.
The repository-wide lint agent's scratch configuration called `markdown/no-bare-urls` whitespace-shaped and
accepted a URL token that gained `<` and `>`;
that is a content fix by the token definition.

## Options

### Option A: a class on every fix and a separate flag for content

Each `Fix` records `whitespace` or `content`.
`--fix` applies only whitespace fixes,
and a new `--fix-content` applies both.
`--rules` lists the class next to `fixable`.

- Pros:
  the safe behavior is the default,
  so a caller that forgot the flag cannot rewrite content;
  the class is declared once,
  beside the rule's edit,
  and a rule cannot change class without a test noticing;
  findings are unchanged,
  so no rule is loosened.
- Cons:
  `format:monochromatic-lint` stops fixing fence languages,
  prompts,
  link styles and trailing punctuation,
  so those findings stay open until someone runs `--fix-content` or fixes them by hand;
  `Fix` and the rule list gain a field.

### Option B: `--fix-type` selecting classes, as ESLint's `--fix-type` does

The same class on every fix,
but the selection is `--fix-type whitespace`,
`--fix-type content` or both,
and plain `--fix` means `whitespace`.

- Pros:
  a caller can run content fixes alone,
  for one reviewed commit of content changes;
  more classes (for example "structural") can be added later without a new flag.
- Cons:
  a larger command-line surface than the owner's two cases need;
  a repeated option grammar to specify and test.

### Option C: a per-rule fix class in the configuration

A rule option such as `"fix": "whitespace" | "content" | "off"`.

- Pros:
  one repository can switch a content fix on for the rules it trusts.
- Cons:
  the configuration then decides whether a rewrite happens,
  and an agent that edits the configuration changes it,
  which is the loosening the owner forbids without approval;
  every rule gains an option and a validation.

### Option D: drop content fixes and keep only reports

Content rules stay as findings and never carry an edit.

- Pros:
  the simplest guarantee,
  and the tool cannot rewrite content at all.
- Cons:
  the mechanical repairs for fence languages,
  prompts and link styles are lost,
  and every such finding costs a hand edit,
  against "Principle 1".

### Option E: keep one `--fix` and use scratch configurations

What the repository-wide lint agent did.

- Pros:
  no change to the tool.
- Cons:
  the default stays unsafe,
  every caller must know which rules are safe,
  and the scratch configuration is exactly the kind of hand-written exception that drifts.

## Ranking

Ranking:
A > B > D > C > E.

- A over B:
  the owner asked for two classes,
  and A encodes exactly those with the least surface;
  B costs more grammar to buy classes nobody has asked for.
- B over D:
  B keeps every existing repair available on request;
  D deletes them to get the same default safety.
- D over C:
  D removes content rewrites for good,
  while C leaves the decision in a configuration that an agent can edit.
- C over E:
  C at least makes the choice explicit and per rule,
  while E leaves the unsafe default in every caller's hands.

## What this change already did

The five whitespace defects of the report were fixed in `package/linter/monochromatic-lint` without this split:

- A URL that the parser read together with a code span's closing backtick is not wrapped any more
  (`markdown/no-bare-urls`,
  no edit when the link text contains a backtick;
  the finding stays).
- `markdown/no-pipe-tables` has no fix.
  Its HTML table broke backticks in cells,
  and repository rule MD5 forbids tables of either kind.
- Fenced listings are never rewritten:
  a rustdoc comment inside a Rust listing in Markdown,
  or inside a doc example,
  is reported and left as written.
  A Rust file's own doc comments are still broken at their markers.
- A line break in a blockquote adds no `>`.
  The new line is a lazy continuation (CommonMark 0.31.2,
  section 5.1 "Block quotes",
  rule 2 "Laziness",
  and the nested example in that section),
  which stays in the paragraph of the quote it continues;
  the fixer never breaks where the next line would start another block,
  the one case laziness excludes.
  A reader who prefers `>` on every line writes it by hand.

## What option A built

- Every `Fix` carries a `FixClass`,
  `whitespace` or `content`,
  set by the rule beside the edit it builds.
  `markdown/semantic-line-breaks` is `whitespace`;
  the seven other fixable Markdown rules are `content`.
- Plain `--fix` applies only `whitespace` fixes.
  `--fix-content` applies both and fixes without `--fix`.
  A fix outside the run's scope is withheld before the fix loop sees it,
  and its finding is still reported.
- `--rules` prints `fixClass` (`whitespace`,
  `content` or `null`) beside `fixable`.
- The in-process entry `run_embedded::fix_source` applies both classes.
  The commit path's `markdown/autofix` runs only `markdown/lfs-image-url`,
  which rewrites an image URL and is therefore a content fix,
  and its configuration names that one rule on purpose.
  Making that entry whitespace-only would switch the policy's rewrite off,
  so it is left as a flagged choice for the owner:
  `package/git-policy/cli/src/native/markdown/markdown_linter.rs` is unchanged.
- The root task `format:monochromatic-lint` already ran plain `--fix`,
  so it now changes no word;
  added line breaks,
  comment markers and blockquote markers are syntax.
  Its description and comment in `mise.toml` say so,
  and content fixes are run by hand with `monochromatic-lint --fix-content <paths>`.
- The property check covers plain `--fix` with every fixable Markdown rule enabled,
  over fixtures and over a real tree.
