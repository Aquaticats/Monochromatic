# typesafe-i18n 5.27.1 parser regex `REGEX_BRACKETS_SPLIT` catastrophically backtracks on translations containing 3+ levels of nested `{}` literals; V8 hangs, JavaScriptCore returns instantly

## Symptom

A page that uses [`typesafe-i18n`][typesafe-i18n] hangs
indefinitely when rendering a translation whose value
contains nested `{}` patterns (e.g. an embedded JSON schema
literal).
 The hang is silent:
 no console error,
 no JS
exception,
 and no network activity until the regex engine
times out.
 Bun (JavaScriptCore) handles the same input in
microseconds,
 so back-end smoke tests pass while browser e2e
tests time out.

Concrete trigger in `paper2vn`:
 the chapter-generation prompt
was stored as a translation:

```ts
chapterInstruction:
  'Split the paper into 3 to 8 chapters that follow the paper\'s logical structure. ' +
  'Return JSON with shape `{ "title": string, "chapters": [{ "title": string, "summary": string, ' +
  '"dialogue": [{ "text": string, "pose": "neutral" | "thinking" | "happy" }] }] }`. ' +
  'Each chapter should have 3 to 6 dialogue beats; each beat is one to three sentences.',
```

Calling `LL().chapterInstruction()` from the browser pinned a
CPU at 100% and never returned within a 4-minute test
timeout.

## Root cause

`typesafe-i18n` parses every translation lazily on first
access using a regex defined in
`package/parser/src/basic.mts:117` (compiled into
`dist/i18n.object.js`):

```js
const REGEX_BRACKETS_SPLIT = /(\{(?:[^{}]+|\{(?:[^{}]+)*\})*\})/g;
```

Source path in node_modules:
`node_modules/.pnpm/typesafe-i18n@5.27.1_typescript@6.0.2/node_modules/typesafe-i18n/dist/i18n.object.js:58`.

(Earlier readings cited `parser/src/parse-rule.mts` and bundle
line 60;
 both wrong.
 The actual source-of-truth is
`package/parser/src/basic.mts:117`,
 bundled to `dist/i18n.object.js:58`,
verified against `codingcommons/typesafe-i18n` HEAD at tag
`5.27.1` commit `462f7118`.
)

The pattern is a textbook
[catastrophic-backtracking][redos] case.
 The outer group
allows arbitrary alternation between `[^{}]+` (non-brace
text) and a nested `\{(?:[^{}]+)*\}` (one level of nesting).
Both branches share `[^{}]+` cost and the nested
`(?:[^{}]+)*` introduces a second unbounded quantifier;
 on
input that exceeds the regex's one-nested-level capacity,
the engine has exponentially many ways to split each
non-brace run between the two layers,
 and explores all of
them before declaring no match.
 See Verification for the
exact trigger (depth alone is not sufficient;
 depth ×
non-brace-text-length per level is).

V8 (Chrome) handles the regex via a backtracking interpreter
and pins for minutes.
 JavaScriptCore (Bun,
 Safari) and
SpiderMonkey (Firefox 140 on this box,
 observationally)
either short-circuit early or use a different scheduling
strategy that hides the worst case.
 The bug is
engine-dependent,
 which is why a Bun smoke test for the same
code path passes in 60 seconds while a Playwright run in
headless Chromium hangs forever.

This is a bug in the parser regex,
 not in the consumer's
translation file.
 The translation contains a literal `{` /
`}` pattern that is meaningful to the consumer (a JSON
schema) but indistinguishable from `typesafe-i18n`'s
parameter-interpolation syntax (`{key}`).
 The regex tries to
disambiguate by accepting any `{}`-bounded section,
 and that
is where it fails.

## Verification

Version under test:
 `typesafe-i18n` 5.27.1 (`codingcommons`
HEAD `462f7118`).

Minimal repro (Node 26.1.0 / V8 14.6.202.34):

```ts
const REGEX = /(\{(?:[^{}]+|\{(?:[^{}]+)*\})*\})/g;
const chapterInstruction =
  "Split the paper into 3 to 8 chapters that follow the paper's logical structure. "
  + 'Return JSON with shape `{ "title": string, "chapters": [{ "title": string, "summary": string, '
  + '"dialogue": [{ "text": string, "pose": "neutral" | "thinking" | "happy" }] }] }`. '
  + 'Each chapter should have 3 to 6 dialogue beats; each beat is one to three sentences.';
console.time('split',);
chapterInstruction.split(REGEX,);
console.timeEnd('split',);
```

Run under Node / V8:
 `split` does not return within a 30s
timeout (exit 124).
 Run under Bun 1.
x or SpiderMonkey:
sub-millisecond.

The trigger is **not** depth alone.
 A compact 3-level input
like `{ "a": [{ "b": [{ "c": "x" }] }] }` (36 chars,
mostly braces) completes in 0.28 ms even under V8 14.6:
there are too few non-brace chars per level for the
catastrophic combinatorial path to explode.
 The trigger is
the product of (depth that exceeds the regex's one-nested-
level capacity) and (length of non-brace text between
braces):
 more text-per-level means more ways the engine can
split a failing attempt,
 so the explosion shows up.
`chapterInstruction` has 3+ levels and dozens of non-brace
chars between most braces,
 which is enough.

## Verified workaround: `rawString` for static, parameter-free translations

`paper2vn`'s `persona`,
 `chapterInstruction`,
 and
`askInstruction` are locale-aware system prompts that need no
parameter interpolation.
 Bypass the parser entirely for those
keys via `rawString` in `i18n/runtime.ts`:

```ts
import { loadedLocales, } from './i18n-util.ts';

type StringKey = {
  [K in keyof Translations]: Translations[K] extends string ? K : never;
}[keyof Translations];

export function rawString(key: StringKey,): string {
  const locale = resolveLocale();
  const value = loadedLocales[locale][key];
  if (typeof value !== 'string')
    throw new Error(`[i18n] expected string for ${key}, got ${typeof value}`,);
  return value;
}
```

Call sites in `client/dialogue/generator.ts` and
`client/dialogue/ask.ts` swap from `ll.chapterInstruction()`
to `rawString('chapterInstruction')`.
 The lookup is a plain
object index;
 no regex evaluation runs.

Tradeoff:
 only works because the affected keys hold static
strings.
 Translation keys that need parameters (`{name}`,
plurals,
 formatters) must keep using `LL()`.
 A comment in
the source points at this doc so the next reader knows why
the bypass exists.

Even with the upstream regex patch below applied,
 the
`rawString` workaround is still required for translations
containing literal `{}`.
 The patch turns the V8 hang into a
terminating but semantically-wrong parse (the parser
splits at the deepest `{...}` block it can match and feeds
that substring through `parseArgumentPart`,
 producing
garbage `BasicArgumentPart` objects).
 Both behaviours misread
the translation;
 the patch only fixes the hang.

## Alternatives we considered and rejected

- Escape every `{` in the translation as `\{`:
   mechanical
  but voluminous;
   the JSON schema has dozens of braces,
   and
  every locale needs the same edits.
   A single missed escape
  silently re-introduces the hang.
   Tradeoff:
   ongoing
  maintenance burden.
- **Move the schema part out of i18n into a constant in
  `generator.ts`**:
   loses locale-aware schema phrasing (the
  Chinese prompt has the schema in the same sentence as the
  count guidance).
   The split would either duplicate the
  natural-language part across locales or interleave i18n
  strings with hardcoded chunks.
   Tradeoff:
   harms translation
  consistency.
- Pre-compute every translation eagerly at boot:
  triggers the same regex on the same input;
   does not help.
- Switch to a different i18n library:
   disproportionate
  to one bug in one prompt;
   `typesafe-i18n` is otherwise
  fine.

## What does not work

- Using `\{` and `\}` only at the outermost level:
   the regex
  still descends into the content and backtracks on inner
  braces.
- Replacing the JSON schema with `<json>...</json>`
  placeholders and substituting at call time:
   the
  placeholders have no `{}`,
   so the parser is fine,
   but the
  LLM sees a degraded prompt and starts producing
  schema-violating output (we observed this in early
  experiments).
- Disabling minification on the client bundle:
   confirmed by
  building with `compress: false`;
   the hang persists,
   ruling
  out a minifier-induced regex change.

## Decision: keep `typesafe-i18n`

We evaluated dropping `typesafe-i18n` entirely.
 Decision:
**keep it**,
 retain the `rawString` workaround.

### Why keep it

- The bug is fully contained.
   Only three translations have
  nested `{}` (`persona`,
   `chapterInstruction`,
  `askInstruction`) and all three already bypass the
  parser.
   Every other key (UI labels,
   error messages,
   status
  text) goes through the normal `LL()` path and works.
- The project is alive enough.
   Original author Ivan Hofer
  (1995-2023) is memorialised in the README;
   the repo was
  transferred to the `codingcommons` organisation
  (`https://github.com/codingcommons/typesafe-i18n`) and the
  new maintainers shipped `5.27.1` on 2026-02-11 after a
  ~2.5-year dormant gap.
   npm downloads roughly doubled
  year-over-year to ~50k/week.
- Zero runtime dependencies,
   zero CVEs / advisories ever
  filed (GitHub Advisory DB,
   Snyk,
   Socket,
   all clean as of
  2026-04-29).
   The supply-chain surface is the smallest of
  any candidate evaluated.
- Migration cost is meaningful:
   every `ll.foo()` call site
  across `screens/*`,
   the type-generation pipeline,
   and three
  locale files would need to change,
   plus full e2e
  re-validation.
   Not justified for one contained parser bug.

### Risks we accept

- No bug-fix commits have landed at `codingcommons` since the
  revival:
   most activity is CI/release plumbing
  (release-please,
   Node 24,
   npm provenance).
   Even if we file
  the ReDoS upstream (the draft below is ready),
   do not
  assume a quick fix.
- The bug is latent and unreported.
   No public CVE exists.
  Anyone introducing a new translation with three or more
  levels of nested `{}` **and** substantial non-brace text
  between braces (long English prose plus an embedded JSON
  schema,
   e.g. an LLM-prompt translation) reproduces the V8
  hang silently.
   The `rawString` workaround comment and this
  doc are the only guards.
- Effective maintainer bus factor is one (Ivan Hofer authored
  1231 of ~1300 commits;
   current top contributor
  `benjaminstrasser` has 6).
   If the codingcommons revival
  stalls,
   the next decision will be forced.

### Top alternatives (for the file-this-when-it-stalls case)

1.  **`i18next`** with default `{{ }}` syntax.
     Interpolation
    regex is `/{{(.+?)}}/g`:
     non-greedy,
     no nested-brace
    alternation,
     and does not match single `{` or `}` at all.
    Empirically 0 matches in 0.16 ms on our JSON-with-3-levels
    input.
     Literal `{` needs no escaping.
     Type safety via
    `i18next-resources-for-ts` (less first-class than
    `typesafe-i18n`,
     but workable).
     Two historical XSS CVEs
    (CVE-2017-16008,
     CVE-2017-16010),
     both fixed.
    Lowest-friction migration.
2.  **`@lingui/core`**.
     Translations compiled to a token array
    at build time;
     the runtime never regex-scans `{...}`.
    Best security posture of any candidate
    (Crowdin-backed,
     248 contributors,
     SLSA provenance
    attestations on npm,
     smallest gzip ~2 kB).
     Downside:
     ICU
    MessageFormat requires single-quote escape for literal `{`
    / `}`:
     same escaping problem at compile time instead of
    runtime.
3.  **`@inlang/paraglide-js`**.
     Only runtime that needs zero
    escaping for `{`:
     translations are compiled to JS template
    literals (e.g. `` `Hello ${name}` ``),
     so the literal text
    never round-trips through any parser.
     Downside:
     heavier
    transitive build deps (`@inlang/sdk`,
     `@lix-js/sdk`,
    `kysely`,
     `sqlite-wasm-kysely`),
     effectively two-maintainer.

### Order if we ever switch

`paraglide-js` for technical fit (no escaping at all),
 or
`i18next` for migration ease.
 `@lingui/core` is the
security-first pick but inherits the same literal-brace
problem in a different shape.

## Why we would file this upstream

All 5 constraints hold:

1.  **Is it really upstream's fault?
    ** Yes;
     the regex is
    pathological for an input class the library is meant to
    accept.
2.  **Can upstream fix it?
    ** Yes;
     the catastrophic-
    backtracking shape is eliminated by reshaping the regex
    itself so that the inner alternation no longer has a
    shared left-factor with a nested quantifier.
     One-line
    change to `package/parser/src/basic.mts:117`.
3.  **Are they supporting this use case?
    ** Yes;
     the library
    accepts arbitrary translation strings.
4.  **Will they likely fix it?
    ** Codingcommons revival is
    focused on plumbing;
     no bug-fix commits since the revival.
    Possible but not assured.
5.  **Have we prototyped a minimal fix?
    ** Yes.
     One-line
    regex change at `package/parser/src/basic.mts:117`.
    Preserves matching contract:
     45/45 parser tests,
     122/122
    runtime tests,
     100/100 generator tests pass on a fresh
    `codingcommons/typesafe-i18n` clone at tag `5.27.1`
    (commit `462f7118`).
     Pre-patch:
     `chapterInstruction`
    reproducer does not return within 30 s under Node 26 /
    V8 14.6 (exit 124).
     Post-patch:
     0.055 ms on the same
    input.
     Patch inline below.

```diff
--- a/packages/parser/src/basic.mts
+++ b/packages/parser/src/basic.mts
@@ -114,7 +114,7 @@ export const parseCases = (text: string): Record<string, string> =>

 // --------------------------------------------------------------------------------------------------------------------

-const REGEX_BRACKETS_SPLIT = /(\{(?:[^{}]+|\{(?:[^{}]+)*\})*\})/g
+const REGEX_BRACKETS_SPLIT = /(\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\})/g

 export const removeOuterBrackets = (text: string) => text.substring(1, text.length - 1)
```

The new shape is `\{ text (block text)* \}`:
 a single outer
brace pair containing alternating non-brace text and
non-recursive nested `{...}` blocks.
 Each iteration of the
inner `*` consumes either a maximal run of non-brace chars
or one literal `{[^{}]*}` block,
 so the engine has at most
one way to match each iteration.
 No shared left-factor,
 no
catastrophic explosion.
 Behaviour on legitimate parameter,
plural,
 and switch-case syntax is identical (verified by the
test suites above).

Decision:
 worth filing;
 draft below is ready,
 with the patch
attached.

## Draft upstream issue (kept as reference; revise before filing)

````md
**Title**: parser regex catastrophically backtracks on translations containing nested `{}` literals (engine-dependent: V8 hangs)

**Labels**: bug, parser

### Description

`REGEX_BRACKETS_SPLIT` at `package/parser/src/basic.mts:117` (bundled to `dist/i18n.object.js:58`) exhibits catastrophic backtracking on translation values that contain literal `{}` interleaved with non-brace text past one level of nesting. The hang is engine-dependent: JavaScriptCore (Bun, Safari) returns sub-millisecond; V8 (Chrome, Edge, Node) pins a CPU and does not return within practical timeouts.

The pattern is `/(\{(?:[^{}]+|\{(?:[^{}]+)*\})*\})/g`. The two alternatives in the inner non-capturing group share `[^{}]+`, and the nested `(?:[^{}]+)*` introduces a second layer of unbounded quantification; together they let the engine explore exponentially many splittings of non-brace text when no overall match exists.

Depth alone is not the trigger. A compact `{ "a": [{ "b": [{ "c": "x" }] }] }` (mostly braces) completes sub-millisecond even under V8. A real-world prompt with the same nesting depth but dozens of non-brace chars per level (an embedded JSON schema literal in an LLM-prompt translation) hangs.

### Reproduction

```js
// Standalone — no library install needed.
const REGEX = /(\{(?:[^{}]+|\{(?:[^{}]+)*\})*\})/g;
const input =
  "Split the paper into 3 to 8 chapters that follow the paper's logical structure. "
  + 'Return JSON with shape `{ "title": string, "chapters": [{ "title": string, "summary": string, '
  + '"dialogue": [{ "text": string, "pose": "neutral" | "thinking" | "happy" }] }] }`. '
  + 'Each chapter should have 3 to 6 dialogue beats; each beat is one to three sentences.';
console.time('split',);
input.split(REGEX,); // hangs in V8, returns sub-millisecond in JSC
console.timeEnd('split',);
```

Run under Node 26 / V8 14.6: `split` does not return within 30 s (`timeout 30 node ...` exits 124). Run under Bun: returns immediately.

### Why this matters

LLM-app boilerplate often stores model instructions as translations so the prompt can be locale-aware. Embedded JSON schema literals (`{ "title": string, "chapters": [{ ... }] }`) are common in those prompts. The hang has no console output, no exception, and no network activity, so it presents to consumers as "the page froze" with no diagnostic.

### Suggested fix

Reshape the regex so the inner alternation no longer has a shared left-factor with a nested quantifier:

```diff
--- a/packages/parser/src/basic.mts
+++ b/packages/parser/src/basic.mts
@@ -114,7 +114,7 @@ export const parseCases = (text: string): Record<string, string> =>

 // --------------------------------------------------------------------------------------------------------------------

-const REGEX_BRACKETS_SPLIT = /(\{(?:[^{}]+|\{(?:[^{}]+)*\})*\})/g
+const REGEX_BRACKETS_SPLIT = /(\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\})/g

 export const removeOuterBrackets = (text: string) => text.substring(1, text.length - 1)
```

The new shape is `\{ text (block text)* \}`: a single outer brace pair containing alternating non-brace text and non-recursive nested `{...}` blocks. Each iteration of the inner `*` consumes either a maximal `[^{}]*` run or one literal `{[^{}]*}` block; the engine has at most one way to match each iteration, so no catastrophic explosion is possible.

Verified on the cloned repo at tag `5.27.1` (commit `462f7118`):

- Parser tests: 45/45 pass.
- Runtime tests: 122/122 pass.
- Generator tests: 100/100 pass.
- Reproducer above: pre-patch hangs >30 s under Node 26 / V8 14.6 (exit 124); post-patch 0.055 ms.

### Workaround (for consumers stuck on 5.27.1)

Read the raw string from `loadedLocales[locale][key]` directly when the translation contains literal braces and does not need parameter interpolation. This bypasses the parser entirely.
````

[typesafe-i18n]: https://github.com/ivanhofer/typesafe-i18n
[redos]: https://en.wikipedia.org/wiki/ReDoS
