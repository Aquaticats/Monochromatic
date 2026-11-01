# marked 18.0.12 and older end a list item at a type 7 HTML line and invert later code spans

## Symptom

No error is printed.
 A Markdown document renders with a stretch of prose wrapped in `<code>` elements and its code tokens rendered as
 plain text,
 starting at some arbitrary point in a long paragraph and running to the paragraph's end.
 The live symptom,
 from a rendered check of `doc/handover/cli-git-rust-implementation.md` with marked 18.0.11:

```html
<!-- marked 18.0.11, one paragraph of the pre-fix handover record -->
trade-offs are recorded in the </code>Force background<code>section of</code>package/claude-code-plugin/bash-output-filter/README.md<code>.
```

`Force background` and `package/claude-code-plugin/bash-output-filter/README.md` are code spans in the source and
 must render as code,
 while the words around them are prose.
 The fragment shows the opposite pairing.

Surface trigger patterns that produce it,
 each combined with an inline code span whose backticks sit on different source lines:

- a lazy continuation line (indented less than the list item's content indent) that begins with `<` and a letter,
  such as `  <merge_oid>` at the start of a nested list item's continuation line
- the same line shape at the top of a list item's second line,
  where the item's own first line already opened a code span
- any type 7 tag spelling matches,
  because the shipped pattern accepts every `<word...>` line;
  closing tags such as `</div>` fail differently and are catalogued in `Verification`

Two failure variants exist:

- marked 18.0.11 ends the list item,
  renders the `<word>` token as raw inline HTML in a new paragraph of the parent item,
  and inverts every later inline code span of that new block
- marked 18.0.12 ends the list item and inverts the spans,
  but escapes the tag as text (`&lt;merge_oid&gt;`) instead of parsing it as raw HTML

## Root cause

The source shape is two defects together:
 a code span hard-wrapped across a line boundary (the `MD8` class in `AGENTS.md`),
 and a continuation line that begins with a type 7 HTML-looking token.
 Neither shape alone triggers the inversion.

marked's list tokenizer collects a list item's lines until one looks like a new block.
 The `html` start test in that loop is the deciding code.
 In the shipped marked 18.0.11 bundle it reads:

```js
// node_modules/.pnpm/marked@18.0.11/node_modules/marked/lib/marked.esm.js:13
htmlBeginRegex: I(l => new RegExp(`^ {0,${l}}<(?:[a-z].*>|!--)`, "i")),
```

That pattern matches any `<` followed by a letter and later a `>` on the line,
 so a type 7 tag like `<merge_oid>` counts as "start of html block" and ends the item:

```ts
// markedjs/marked src/Tokenizer.ts:405-408 at 4b7b5b2d1e05d24c85ac2c2fa96bbfe6952e6770
// End list item if found start of html block
if (htmlBeginRegex.test(nextLine)) {
  break;
}
```

The item's collected raw therefore stops one line early,
 which the token stream shows directly:
 for the failing source the nested list token's raw is `- Stall after a failed attempt.\ntext A stays\n` and the
 `<merge_oid>` line becomes a sibling `paragraph` token of the nested list inside the parent item.

The inversion follows from that split.
 The hard-wrapped code span opened with a backtick before the split and would have closed with the backtick after
 `<merge_oid>`.
 After the split the closing backtick is the first backtick of a fresh block,
 so inline pairing shifts by one:
 the next backtick closes a span that should have opened one,
 every later span flips,
 and the block's odd trailing backtick renders literally.

CommonMark forbids the interruption that produces the split.
 From the spec text quoted by upstream fix PR 4072 (commonmark-spec 0.31.2,
 spec.txt lines 2396 to 2408 and 2449 to 2450):

> All types of HTML blocks except type 7 may interrupt a paragraph.
 Blocks of type 7 may not interrupt a paragraph.

Reference implementations agree.
 micromark 4.0.3 (CommonMark reference) and GitHub's cmark-gfm both keep the line in the list item and form the
 whole code span on the same failing source (measured in `Verification`).

### Earlier readings that were wrong

- A `<word>` line start does not split paragraphs in general.
 The same line inside a plain paragraph (no list) parses as one paragraph with inline HTML under marked,
 and micromark parses the nested-list source with no split.
 Only marked's list-item end test treats it as a block start.
- Explaining 18.0.11 from the markedjs/marked source tree at HEAD is wrong.
 HEAD's `src/rules.ts:103-104` already carries the strict pattern with the comment "type 7 is excluded there",
 while the shipped 18.0.11 bundle carries the permissive pattern shown above.
 The bundle is the code under test.

## Verification

### Versions under test

- marked 18.0.11,
  the workspace resolution via `@earendil-works/pi-tui@1.3.0` (`pnpm-lock.yaml:11466`),
  tarball integrity `sha512-HnslJfsZkRPBDJRHvVtAaWlZHEpSu7u8LgQuJCELjRKuWR+hpq4A7sLq3p8HaI9ypVoXDXxV34CsQJEe1+J5Aw==`
- marked 18.0.12 published 2026-09-07,
  18.0.13 published 2026-09-12,
  18.1.1 published 2026-10-12
- micromark 4.0.3,
  already in this workspace's `node_modules/.pnpm`
- GitHub cmark-gfm,
  through `gh api --method POST /markdown` with `mode: gfm`
- markedjs/marked source at `4b7b5b2d1e05d24c85ac2c2fa96bbfe6952e6770` (18.1.1),
  origin `https://github.com/markedjs/marked.git`,
  cloned with `gh repo clone markedjs/marked "${HOME}/temp/agent/marked-2026-10-22" -- --depth 1`

### Harness

```js
// ~/temp/agent/marked-list-html-line-repro.mjs
// Usage: node marked-list-html-line-repro.mjs <path-to-marked-esm.js>
const marked = await import(process.argv[2]);

const failingSource = [
  '- Outer item text.',
  '  - Stall after a failed attempt.',
  '  0b97c22d5d). Plan: fast-forward when `git diff --name-only <local>',
  '  <merge_oid>` is disjoint from the status paths.',
].join('\n');

const workingSource = failingSource.replace('<merge_oid>`', 'merge_oid`');

for (const [name, source] of [
  ['lazy line starts with <merge_oid>', failingSource],
  ['lazy line starts with plain word', workingSource],
]) {
  const html = marked.marked.parse(source);
  const expected = name.includes('<merge_oid>')
    ? '<code>git diff --name-only &lt;local&gt; &lt;merge_oid&gt;</code>'
    : '<code>git diff --name-only &lt;local&gt; merge_oid</code>';
  console.log('---', name, '---');
  console.log('whole code span in the nested item:', html.includes(expected));
  console.log(html);
}
```

### Patterns that fail

- the harness failing source on marked 18.0.11:
  `whole code span in the nested item: false`.
  The output ends the nested item before the `merge_oid` line and opens a new paragraph in the parent item:

```html
<!-- marked 18.0.11 output tail for the failing source -->
</ul>
<p><merge_oid>` is disjoint from the status paths.</p>
```

- the harness failing source on marked 18.0.12:
  the item still ends and the span still breaks,
  with the tag escaped as text instead of parsed as raw HTML:

```html
<!-- marked 18.0.12 output tail for the failing source -->
</ul>
<p>&lt;merge_oid&gt;` is disjoint from the status paths.</p>
```

- PR 4072's own corpus line shapes on 18.0.11:
  `</div>`,
  `</p>`,
  `</ul>`,
  `<b>`,
  `<span>`,
  `<em x='1'>`,
  `<a href='x'>`,
  `<x-widget>`,
  all listed there as before-and-after divergences from the reference

### Patterns that work

- the harness working source (plain word on the lazy line) on every version tested:
  `whole code span in the nested item: true`
- the harness failing source on marked 18.0.13 and 18.1.1:
  `whole code span in the nested item: true`,
  output `<code>git diff --name-only &lt;local&gt; &lt;merge_oid&gt;</code> is disjoint from the status paths.</li>`
- the harness failing source on micromark 4.0.3 and on GitHub cmark-gfm:
  the nested item keeps the line and the code span forms whole
- a type 7 token that is not the first token of its line:
  no split on any version tested

## Verified workarounds

1. Keep every code span on one source line,
   so no closing backtick can be orphaned by a block split (the `MD8` rule in `AGENTS.md`).
   Two wraps of this class were repaired in `doc/handover/cli-git-rust-implementation.md`
   (the `git diff --name-only <local> <merge_oid>` span and the `doc/runbook/publish-npm-package-first-time.md:57`
   path token,
   which was rendering with a spurious space after `first-`).
   Tradeoffs:
   historical line wrapping changes although the rendered text is preserved,
   and the repair must be repeated wherever else a span is hard-wrapped.
2. Re-indent a lazy continuation line to the list item's content indent so it is not lazy,
   or reword it to avoid a leading `<word` token.
   Tradeoffs:
   re-indenting changes how the nesting reads in raw source,
   and rewording changes historical record text.
3. Upgrade marked to 18.0.13 or newer in consumers that parse Markdown with it.
   Tradeoffs:
   the dependency move belongs to the consumer packages,
   and nothing changes for documents read through GitHub's renderer,
   which never showed the failure.

## What does not work

- `monochromatic-lint`'s `markdown/semantic-line-breaks` rule does not catch the wrap.
   The pre-fix wrap drew zero findings from
   `mise run lint:monochromatic-lint -- doc/handover/cli-git-rust-implementation.md`:
   the rule checks break positions after punctuation,
   not span integrity.
- Treating a `<word>` line start as an unconditional block start.
   Plain paragraphs keep such lines,
   and CommonMark only restricts type 7 from interrupting a paragraph.
- The first harness discriminator,
   `!html.includes('<p><merge_oid>')`,
   mis-called marked 18.0.12 as passing,
   because that version escapes the tag to `&lt;merge_oid&gt;` while still splitting the item.
   The exact code-span assertion in the harness replaces it.
- A one-character upstream patch adding `/?` to the old pattern.
   PR 4072 measured 4,316 regressions on its corpus for that variant,
   because closing type 7 tags would then end list items.

## Upstream filing artifact

Nothing to add upstream.
 The behavior was fixed before this write-up,
 so there is no new-issue draft and no comment draft to keep.

### Upstream filing decision

Checked `.out-of-scope/` first:
 no entry there exempts markdown,
 marked,
 or CommonMark rendering.
 The entries read were bun-install,
 cargo-workspace,
 claude-code-upstream-bugs,
 codex-harness,
 jsr,
 lightningcss,
 low-impact-typescript-formatting,
 module-es-monolith,
 pi-gpt55-long-context,
 terminal-title-fork-parity-tests,
 and typescript-project-references.

Duplicate search before any filing:
 `gh search issues --repo markedjs/marked "lazy continuation HTML"`,
 `"code span newline"`,
 and `"html block interrupt paragraph"` all returned no issues,
 and the source history (`git log -S "type 7 is excluded" -- src/rules.ts`) found the fixing PR 4072
 (merged 2026-09-12,
 commit `c2facac8d455c87034f82639c900242de288aca0`).
 Its body cites the earlier issues 3437 and 3444 for the two halves of the same start-condition test.

1. **Is it really upstream's fault?**
   Yes for marked 18.0.12 and older.
   The list-item end test deviated from CommonMark 0.31.2's type 7 limitation,
   which micromark 4.0.3 and cmark-gfm both implement (measured).
   Our own authoring debt contributed the hard-wrapped code span that turns the split into an inversion.
2. **Can upstream fix it?**
   Yes.
   The minimal change is one regex,
   recorded inline below.
3. **Are they supporting this use case?**
   Yes.
   CommonMark conformance is marked's stated goal,
   and the fix added spec files `html_closing_tag_following_list`,
   `html_inline_tag_in_list`,
   and `html_multiline_tag_following_list` under `test/specs/new`.
4. **Would the repo welcome our contribution?**
   Moot for a filing,
   and checked anyway:
   `.github/ISSUE_TEMPLATE/Bug_report.md`,
   `.github/ISSUE_TEMPLATE/Feature_request.md`,
   `.github/ISSUE_TEMPLATE/Proposal.md`,
   `.github/PULL_REQUEST_TEMPLATE/badges.md`,
   and `README.md` contain no ban on AI-assisted reports,
   and PR 4072 merged through the ordinary pull request flow.
5. **Will they likely fix it?**
   Fixed already,
   released in 18.0.13 on 2026-09-12,
   and present in 18.1.1 (both verified with the harness).
6. **Have we prototyped a minimal fix compatible with their architecture?**
   No separate prototype is kept,
   because the minimal fix already exists upstream and ships in the current release.
   Re-deriving it would duplicate merged code.
   The pre-fix and post-fix sides of the fix are measured here instead:
   18.0.11 and 18.0.12 fail the harness,
   18.0.13 and 18.1.1 pass it.

The minimal fix,
 from `git show c2facac -- src/rules.ts`:

```diff
--- a/src/rules.ts
+++ b/src/rules.ts
@@ -96,7 +96,9 @@ export const other = {
   fencesBeginRegex: cachedIndentRegex((indent: number) => new RegExp(`^ {0,${indent}}(?:\`\`\`|~~~)`)),
   headingBeginRegex: cachedIndentRegex((indent: number) => new RegExp(`^ {0,${indent}}#`)),
-  htmlBeginRegex: cachedIndentRegex((indent: number) => new RegExp(`^ {0,${indent}}<(?:[a-z].*>|!--)`, 'i')),
+  // a list item ends where a paragraph would be interrupted, so this mirrors the
+  // html start conditions in the paragraph rule below; type 7 is excluded there
+  htmlBeginRegex: cachedIndentRegex((indent: number) => new RegExp(`^ {0,${indent}}(?:</?(?:${_tag})(?: +|$|/?>)|<(?:script|pre|style|textarea|!--))`, 'i')),
   blockquoteBeginRegex: cachedIndentRegex((indent: number) => new RegExp(`^ {0,${indent}}>`)),
 };
```

A later commit (`0db74cb`) tightens the script-family half of the same alternation in HEAD
 (subject `fix: match pre, script, style and textarea as whole tag names`).
