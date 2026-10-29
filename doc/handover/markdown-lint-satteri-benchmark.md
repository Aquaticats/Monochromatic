# Markdown-lint to Sätteri: speed benchmark

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/rss` is now `package/rss/app`.

Status:
 complete.
 Recommendation:
 adopt the parser swap (variant 2);
 do not
pursue the plugin rewrite (variant 3).

## Goal

Measure,
 not guess,
 the speed of three ways to run the repo's Markdown/MDX
linting (`@monochromatic-dev/cli-markdown-lint`,
 `package/cli/markdown-lint`),
to decide whether and how to migrate to Sätteri (<https://satteri.bruits.org/>,
version 0.9.4),
 a Rust Markdown/MDX engine with JavaScript plugins:

- Current state:
   the existing linter,
   parsing with `mdast-util-from-markdown`
  plus the micromark GFM/frontmatter/MDX extensions.
- Parser only (variant 2):
   the same rule engine,
   `parse.ts` swapped to Sätteri's
  `markdownToMdast` / `mdxToMdast` (Rust parse,
   then materialization to a
  standard `mdast.Root`).
- Completely rewritten (variant 3):
   the twelve rules re-expressed as Sätteri
  MDAST visitor plugins,
   driven by the manual pipeline (`createMdastHandle` +
  `visitMdastHandle`),
   reading nodes lazily and fusing the twelve per-rule tree
  walks into one dispatch.

## Headline result

The current linter spends almost all its time parsing,
 and Sätteri's Rust parser
removes that cost.
 The parser swap (variant 2) is the clear winner:

- Lint (clean corpus):
   4150 ms to 601 ms,
   a 6.9x speedup.
- Lint (violation-dense corpus):
   5661 ms to 922 ms,
   a 6.1x speedup.
- Autofix (violation-dense corpus):
   17836 ms to 3012 ms,
   a 5.9x speedup.

The full plugin rewrite (variant 3) gives no speed benefit over the parser swap
(it ties on lint and is slower on autofix) and introduces two correctness
hazards,
 so it is not worth its much larger cost and risk.

## Corpus

Fixed snapshot of every `.md`/`.mdx` the linter would process (gitignore-style
exclusions:
 `node_modules`,
 `package-paused`,
 `package-deprecated`,
`.out-of-scope`).

- Files:
   642 (36 MDX),
   about 7.24 MB;
   largest `doc/audit/em-dash.md` (~212 KB).
- Because the real corpus mostly passes lint,
   a mangled copy was generated
  (scratchpad `corpus-mangled/`,
   about 8.74 MB) by appending violation blocks
  scaled to file size,
   covering every fixable rule (MD014,
   MD026,
   MD034,
   MD053,
  MD054,
   no-pipe-tables,
   semantic-line-breaks).
   This exercises diagnostic
  construction and,
   crucially,
   the autofix fixpoint loop,
   which re-parses on
  every pass and is the most parser-bound path.

## Method

- Each variant runs in its own Node process (v26.4.0,
   running `.ts` directly,
   as
  the `node src/cli.ts` task does).
   Warmup passes,
   then measured full-corpus
  passes;
   the minimum pass time is reported (least GC/JIT noise),
   plus
  throughput.
   Total diagnostic count is a checksum to confirm equivalent work.
- Timing covers parse plus rule evaluation only (no file IO,
   no fix writes),
  which is the work that differs across variants.
- Three git worktrees off HEAD `0c8f0d064`,
   node_modules symlinked to main
  (identical dep graph at the same commit):
   `/var/home/user/mono-wt/{current,
  parser-swap,plugin-rewrite}`.
   Sätteri installed standalone in scratchpad
  `satteri-pkg/` (native `satteri-linux-x64-gnu` binary),
   imported by absolute
  path.
   Benchmark scripts and corpora live in the session scratchpad.
- Absolute numbers are machine-specific;
   the ratios are the portable takeaway.

## Results

### Parse stage only (642 files, 7.24 MB)

- Current,
   `fromMarkdown` (micromark):
   3824 ms,
   1.9 MB/s.
- Sätteri `markdownToMdast` (Rust parse plus JS materialization):
   108 ms,
  67 MB/s.
   Same tree as the current parser (identical node checksum).
- Sätteri `createMdastHandle` (Rust parse,
   no materialization):
   65 ms,
   111 MB/s.
- Sätteri parse plus visitor dispatch,
   no rule work (variant 3 floor):
   136 ms.

Reading:
 parsing is 92% of the current linter's lint time (3824 of 4150 ms).
 The
Rust parser is 35x to 59x faster.
 Materialization is cheap (about 43 ms over the
whole corpus).
 Once parsing is fast,
 the JavaScript rule-walking dominates.

### Lint, clean corpus (min ms, throughput)

- Variant 1 current:
   4150 ms,
   1.74 MB/s.
- Variant 2 parser swap:
   601 ms,
   12.0 MB/s.
- Variant 3 plugin rewrite:
   611 ms (see the correctness caveat below).

### Lint, mangled corpus (min ms)

- Variant 1 current:
   5661 ms.
- Variant 2 parser swap:
   922 ms.
- Variant 3 plugin rewrite:
   980 ms.

### Autofix, mangled corpus (min ms)

- Variant 1 current:
   17836 ms.
- Variant 2 parser swap:
   3012 ms.
- Variant 3 plugin rewrite:
   4198 ms,
   and this undercounts (about 22 files per
  pass crash on re-parse and were skipped;
   see the MDX hazard below).

## Why variant 3 does not win

The rule logic (dominated by `semantic-line-breaks` scanning text nodes) is
identical JavaScript work in variants 2 and 3.
 The only structural difference is
that variant 2 materializes the tree and each rule walks it,
 while variant 3
dispatches nodes lazily in one pass.
 But Sätteri's dispatch floor (136 ms) is
already higher than variant 2's parse plus materialization (108 ms),
 so fusing
the twelve walks into one cannot make up the difference.
 Variant 3 ties variant 2
on lint and is slower on autofix.

## Two correctness hazards in variant 3 (the plugin path)

These are properties of Sätteri's manual visitor pipeline,
 discovered by
diffing variant 3's diagnostics against the current linter:

- Byte offsets,
   not character offsets.
   `visitMdastHandle` reports node positions
  as UTF-8 byte offsets,
   whereas the materialized path (`markdownToMdast`) and
  JavaScript strings use UTF-16 code-unit offsets.
   On `doc/audit/em-dash.md`
  (many multibyte `—`),
   a text node the materialized path places at char 1842
  arrives at byte 1858.
   Every `source.slice` and every fix offset is then wrong
  on any non-ASCII document:
   variant 3 diverged on 108 of 642 files.
   A correct
  variant 3 must build a byte-to-character map per file and thread it through
  every offset helper,
   which adds cost and erodes the lazy-dispatch advantage.
- MDX re-parse crash on the table fix.
   The `no-pipe-tables` fix emits a raw
  `<table>` HTML block.
   When the autofix loop re-parses an `.mdx` file through
  Sätteri's MDX parser,
   that block is read as JSX and rejected
  (`mdx-jsx:unexpected-character`).
   The byte-offset corruption makes the emitted
  HTML malformed as well,
   so the re-parse throws.
   The current parser
  (`mdast-util-mdx`) tolerates the same input.

The parser swap (variant 2) has neither hazard:
 it consumes Sätteri's
materialized tree,
 whose positions are correct character offsets,
 and reuses the
current rules and fixer unchanged.

## Parity of variant 2 (parser swap)

With the fix below applied,
 variant 2 produces 5095 diagnostics against the
current linter's 5070,
 differing on a single file.
 Sätteri threw on zero of 642
files.
 The residual difference is an irreducible parser-semantics divergence:
 for
`1. 3.1.1 **Testing**` (an ordered-list item whose content starts with a number),
Sätteri's pulldown-cmark parser folds the `1. ` marker into a text node and
mis-bounds the following `**strong**`,
 while micromark parses a clean list item.
This affects `package/webapp-productivity/rss/TODO.index.md` only (25
diagnostics,
 0.5%).
 Sätteri documents such divergences from the unified
ecosystem;
 a real migration should re-run the rule unit tests against Sätteri's
tree and record the accepted divergences.

## A rule bug found and fixed while benchmarking

Before the fix,
 variant 2 diverged on six files.
 The cause was a latent
parser-dependency in `semantic-line-breaks`:
 it scans a text node's own source
slice for a break-point followed by a newline,
 but when an inline node (such as
inline code) begins on the next line,
 the newline sits just past the node
boundary.
 mdast-util-from-markdown extends the text node's end offset over that
newline;
 Sätteri ends the node tighter.
 So the rule falsely flagged an
already-broken line.

The fix (applied in the `current` and `parser-swap` worktrees) makes the rule
parser-independent by scanning a bounded window of source just past the node:
`followStatus` in `semantic-break-points.ts` gains a `trailing` argument,
 and
`semantic-line-breaks.ts` passes `source.slice(nodeEnd, min(paragraphEnd,
nodeEnd + 256))`.
 It resolved five of the six files,
 left the current parser's
results unchanged (still 5070),
 and cost no measurable time.

**Landed on `main` on 2026-07-31,
 and the 256-character bound is gone.
** The bound was a way to avoid copying the paragraph,
 and inter-node whitespace has no length limit,
 so a break already written past a longer run of spaces read as absent and `--fix` inserted a second one,
 which is a blank line and splits the paragraph.
`trailing` is now a range over the source (`source`,
 `tailEnd`,
 `paragraphEnd`) rather than a copy of it,
 and the scan stops at the first character that is not a space or a tab,
 so it costs the whitespace run rather than the paragraph and needs no bound at all.
Not copying is what removes the bound.

## Recommendation

Adopt variant 2,
 the parser swap:

- Replace the body of `package/cli/markdown-lint/src/parse.ts` with Sätteri's
  `markdownToMdast` / `mdxToMdast` (features `{ gfm: true, frontmatter: true }`);
  keep every rule,
   the `Diagnostic`/`Fix` model,
   the fixpoint fixer,
   the
  reporters,
   and the CLI unchanged.
- Add `satteri` as a dependency (it ships a per-platform native binary with a
  WASI fallback;
   the CLI still runs under Node).
   This reintroduces a platform
  binary that the current pure-TypeScript-under-Node linter avoided;
   note it in
  the package README.
- ~~Land the `semantic-line-breaks` trailing-lookahead fix and update its tests.~~
   Landed 2026-07-31,
   as a source range rather than a bounded slice.
- Re-run the rule unit tests against Sätteri's tree;
   record the one accepted
  list-marker divergence.

Do not pursue variant 3.
 It is a much larger rewrite with no speed benefit over
variant 2 and two correctness hazards (byte offsets,
 MDX table re-parse) that
variant 2 sidesteps entirely.

## Artifacts and reproduction

- Worktrees:
   `/var/home/user/mono-wt/{current,parser-swap,plugin-rewrite}` (off
  HEAD `0c8f0d064`).
   The parser-swap worktree holds the one-function `parse.ts`
  swap;
   the plugin-rewrite worktree holds the visitor driver.
   These are
  throwaway;
   remove with `git worktree remove` when done.
- Scratchpad scripts:
   `bench-core.mjs` (harness),
   `mangle.mjs` (mangled corpus),
  `v1-current.mjs`,
   `v2-parser-swap.mjs`,
   `v3-driver.mjs` plus `v3-bench.mjs`,
  `v3-explore.mjs` (parse-stage breakdown),
   and `satteri-pkg/` (standalone
  Sätteri install).
- The main worktree is unchanged except for this document.
