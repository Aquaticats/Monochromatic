# Mis-indented TypeScript passes oxlint-stylistic after dprint's typescript-0.95.9 formatter was disabled

## Symptom

Mis-indented TypeScript passes `oxlint` in every package.
Issue [aquaticats/Monochromatic#577](https://github.com/Aquaticats/Monochromatic/issues/577) states it plainly:
"nothing that runs today checks indentation".

Evidence recorded there:

- the translation-repair audit fixed blocks whose indentation disagreed with their nesting while lint stayed clean:
  about 190 lines in `src/benchmark.ts` (`runCriticBenchmark`'s nested blocks,
  fixed in `632ef5fbc`),
  about 60 lines in `src/repair-translation.unit.test.ts` after a template literal,
  and shallower-than-block arguments in `src/stream-idle-guard.unit.test.ts` (fixed in `f7b766fa4`);
- a heuristic scan of that package's 1,666 `.ts` files flagged 35 lines;
  the rest were template-literal ends.

Measured over the whole repository corpus with the model in "The model" (probe in "Verification"):

- 10,057 code lines (0.96% of 1,052,606 code lines) disagree with the model;
  the model matches 99.04% of code
  lines overall;
- 578 flagged lines are off by an odd width (1 or 3 spaces),
  which no 2-space convention can intend;
- concrete failures:
  `package/agent-harness-shared/advisor/rolldown.node.config.ts:16` holds array elements two
  spaces deeper than both the dprint convention and the `item-per-line-fix` render,
  and
  `package/agent-harness-shared/advisor/src/operation-usage.ts:46` starts a function-expression body two spaces
  shallower than its owner line.

The rule that closes this gap now ships as `stylistic/indentation` in `package/oxlint-plugin/stylistic`,
with
check and autofix (see "Implementation").
`package/config/oxlint/src/rule/style.ts` enables it at `warn` (`a0de3e051f`),
and one normalization sweep has landed (`68415aeb53`),
so repository lint now flags and fixes mis-indented
TypeScript.

Related replacement gap recorded in the same issue:
`comma-dangle` ships in the plugin but
`package/config/oxlint/src/rule/style.ts` does not enable it,
while the retired dprint config set
`"trailingCommas": "always"`;
files in one package mix both styles.

## Root cause

`package/config/dprint/index.json` disables the dprint TypeScript formatter,
moving its duties to the plugin:

```jsonc
// package/config/dprint/index.json:39
    // Disabled: to be replaced by our oxlint-stylistic
    //    "https://plugins.dprint.dev/typescript-0.95.9.wasm",
```

The conventions it used to enforce live on as commented settings in the `"typescript"` block
(`package/config/dprint/index.json:105`:
`preferHanging`,
`memberExpression.linePerExpression`,
`binaryExpression.linePerExpression`,
`trailingCommas`,
`useBraces`,
`nextControlFlowPosition`)
plus `lineWidth: 90`,
`indentWidth: 2`,
`useTabs: false` (`package/config/dprint/index.json:32`).

The plugin accepted the handoff in its own package doc
(`package/oxlint-plugin/stylistic/src/index.ts:38`):

```text
 The per-line rules fire when 2 or more items share a source line and
 auto-fix by placing every item on its own line with consistent
 indentation. They are this repository's TypeScript layout authority
 because dprint's TypeScript formatter is disabled; dprint still formats
 non-TypeScript files.
```

No rule in the plugin computed expected indentation before `stylistic/indentation`.
The convention survives only as fixer emission points,
all in the same shape:
indent children of the line
where the construct starts,
plus two spaces.
`package/oxlint-plugin/stylistic/src/utility/item-per-line-fix.ts:188`:

```ts
  const linePrefix = sourceText.slice(
    lineStart,
    openPos,
  );
  const baseIndent = leadingWhitespace(linePrefix,);
  /**
   Two-space continuation indent for items inside the brackets.
   */
  const childIndent = `${baseIndent}  `;
```

and `package/oxlint-plugin/stylistic/src/utility/item-per-line-fix.ts:257` returns the closer to that base indent:

```ts
  const replacement =
    `${bracketPair.open}\n${formattedItems}\n${baseIndent}${bracketPair.close}`;
```

`package/oxlint-plugin/stylistic/src/utility/invocation-depth-fix.ts:292` repeats it:

```ts
  /**
   Head-line indentation; the closing bracket returns to it.
   */
  const baseIndent = baseIndentAt({
    sourceText,
    offset: owner.start,
  },);
  /**
   Operand-line indentation: head-line indent plus two spaces.
   */
  const childIndent = `${baseIndent}  `;
```

`package/oxlint-plugin/stylistic/src/rule/chain-per-line.ts` first emitted one flat `childIndent` for every
continuation line of a chain region,
which contradicted the indent model for a member break whose operand-chain
head renders off the region head line (fixture b8 in `package/test-fixture/oxlint-stylistic/src/invalid/chain-per-line.ts`:
model 4 columns for `.f()`,
flat render 2).
Two rules with different canonical indents for one line oscillate under `--fix`,
the failure mode recorded in
`doc/troubleshooting/oxlint-multi-fix-convergence.md`.
The fix unified chain rendering with the model:
every break carries its owning node's offset and its
continuation line indents two columns past the line where that anchor renders
(`package/oxlint-plugin/stylistic/src/utility/chain-render.ts`),
matching the model's per-axis nesting.

Consequence:
any checker or fixer that computes a different width fights the others and the file oscillates.
A checker built on "anchor line indent plus two spaces,
closers at the bracket base" converges with the
item and invocation fixers by construction,
and the chain fixer now shares the same anchors.

Readings this investigation retracted,
each with the evidence that disproved it:

- Matching dprint's output exactly was the goal.
  Wrong:
  the rule defines its own pragmatic convention and formatter parity is out of scope
  (user correction 2026-10-22).
- A report-only checker is enough.
  Wrong:
  the feature ships with an autofix (user correction 2026-10-22).
- Plain ancestor-plus-two predicts indentation everywhere.
  Wrong:
  the probe's first model systematically predicted `catch`/`else` two spaces deeper than their
  statement lines and chain lines deeper than the fixers emit;
  the emission excerpts above show the true anchors.
- A body brace whose owner ends where its parent ends can adopt the parent's start line.
  Wrong:
  the `owner.end === parent.end` probe variant mis-remapped `ParenthesizedExpression` nodes inside
  `BinaryExpression` (example `package/agent-harness-shared/advisor/src/advisor-client.ts:234`,
  predicted 6,
  produced 4);
  restricting the remap to body kinds owned by a non-body construct fixed it.
- The probe's third-generation counts were final (10,380 mismatches on 1,052,277 code lines).
  Wrong:
  two model corrections moved the count to 10,057 on 1,052,606 code lines (the `owner.end === parent.end`
  retraction above plus source-rooted lines expecting zero);
  `full3.json` and `full4.json` in the scratch probe
  directory hold both.
- The existing fixers all emit anchor-line plus two.
  Wrong as stated:
  `chain-per-line` emitted one flat region indent instead,
  which is why its renderer needed the
  unification described in "Root cause" (commit `310a3fa94c`).
- TSDoc continuation lines explain the gap between probe mismatches and fixer churn.
  Wrong:
  only 11 changed lines in the sweep start with `*`;
  the churn gap is cascade re-indentation,
  measured in
  "Verification".
- The repository's `typescript` dependency can parse TypeScript for tooling.
  Wrong:
  `require('typescript')` resolves to `typescript@7.1.0-beta`'s `lib/version.cjs`,
  where `SyntaxKind`
  is undefined;
  the probe parses with the classic `typescript.js` from the TypeScript v5.9.3 release.

## Verification

Versions under test:
`oxlint` 1.89.0 and `@oxlint/plugins` 1.89.0 from `node_modules`,
the dprint oracle
running dprint 0.64.0 with `typescript-0.95.9.wasm`,
and TypeScript v5.9.3 classic `typescript.js` in the probe.
The plugin API surface needed for check and autofix is present in `node_modules/@oxlint/plugins/index.d.ts:2917`:

```ts
  insertTextBeforeRange(range: Range, text: string): Fix;
  removeRange(range: Range): Fix;
  replaceTextRange(range: Range, text: string): Fix;
```

Harness:
a disposable AST probe at `~/temp/agent/indent-probe-2026-10-22/probe4.mjs`,
run as
`node probe4.mjs corpus.txt > full4.json`,
over every `.ts` file under `package/` excluding test fixtures,
build output,
and `node_modules`:
7,091 files and 1,052,606 code lines after blank,
comment,
and
template-literal-content lines are exempted.
The probe is scratch,
and "The model" below is sufficient to rebuild it.

Clean patterns (the model predicts the corpus):

- suffix keywords (`else`,
  `catch`,
  `finally`,
  do-while `while`) at the owner statement line:
  1,998 of 1,998 match (100%),
  and decl-line closer bases (class,
  interface,
  enum,
  switch):
  436 of 436 (100%);
- closers at the bracket base:
  230,373 of 231,629 opener-line bases match (99.46%) and 13,346 of 13,532
  owner-line bases match (98.63%);
- item start lines at anchor plus two:
  707,161 of 710,114 match (99.58%).

Failing patterns (the model and the corpus disagree;
the autofix rewrites these):

- continuation lines overall match 89,180 of 94,842 (94.03%),
  and the disagreements concentrate in coherent
  alternative styles:
  flat binary and string-concatenation operands (`PlusToken`,
  1,898 lines),
  flat union and intersection bars (`BarToken`,
  1,265 lines),
  and hanging function-expression bodies written
  flat (`ReturnKeyword`,
  736 lines,
  plus their closers);
- deviation widths:
  7,947 lines two spaces shallower than the model,
  1,382 two spaces deeper,
  578 off by an odd width,
  150 off by four;
- mismatch routes:
  8,615 ancestor-route lines,
  1,256 opener-line closers,
  186 owner-line closers;
- corpus-level consequence:
  the rule's autofix normalizes 20,722 to 20,725 leading-whitespace runs in one sweep (two runs),
  which the
  user accepted ("I don't care about huge sweeping changes").

Cross-check with the dprint oracle (scratch `~/temp/agent/indent-dprint-oracle-2026-10-22/`):
200 corpus files formatted by dprint with the repository's conventions change in 195 files,
with 6,785
changed line pairs;
only 228 are genuine content-line indent changes,
1,972 are TSDoc blank-line whitespace
normalizations,
and 4,585 (67.58%) are non-indent rewraps.
On the overlapping package samples (the oracle's first 200 corpus files,
the probe's 50-file advisor-heavy
smoke slice) the two instruments agree on where indent drift lives:
228 per 200 files against 56 per 50 files.

### Implementation verification

The shipped rule and fixer were verified at three levels (suite commands in "Implementation"):

- fixtures:
  `package/test-fixture/oxlint-stylistic/src/invalid/indentation.ts` reports exactly its six expected
  lines,
  `package/test-fixture/oxlint-stylistic/src/valid/indentation.ts` reports none,
  and
  `package/test-fixture/oxlint-stylistic/src/invalid/indentation-mixed.ts` shares one deviating region with the
  argument,
  chain,
  and body fixers;
- unit and autofix tests:
  every engine route and exemption has a case,
  `--fix` tests run the fixer to a fixed
  point twice,
  and the convergence tests assert zero remaining stylistic diagnostics after `fixUntilStable`;
- corpus sweep (scratch `sweep.mjs`,
  disposable mirror of the corpus under `~/temp/agent/indent-probe-2026-10-22/fixwork/`):
  one `oxlint --fix` pass with `stylistic/indentation` alone reaches the fixed point,
  the next pass changes
  nothing,
  20,725 leading-whitespace runs change across 1,793 of 7,091 files (20,722 across 1,792 in the
  first run;
  concurrent sessions edit the corpus),
  zero lines change beyond their
  leading whitespace,
  zero files change line count,
  and zero diagnostics remain;
- whole-set sweep (same script,
  every fixture rule enabled):
  the corpus reaches a fixed point.
  An eight-pass cap reads `stable=false` only because a few files need three cascade layers
  (pass 9 moved 3 files,
  passes 10 and 11 one each,
  pass 12 none);
  `stylistic/indentation` and `stylistic(chain-per-line)` leave zero diagnostics in parseable
  files.
  The remaining 26 chain and 14 indent diagnostics sit inside the 3,716 files the
  `stylistic(comma-dangle)` corruption made unparseable,
  where no fixer applies.

Churn decomposition (probe variant `probe4b.mjs` dumps as-written mismatch ids,
compared against the sweep's
changed-line ids):
9,921 changed lines are direct model disagreements,
10,801 are cascade followers whose
anchor line moved (example region `package/agent-harness-shared/advisor/src/operation-test-fixture.ts:308` to
`:312`,
object properties following their `return` line),
and 138 probe mismatches stayed put because their
resolved width equals their written width.
Two ids drifted between runs because concurrent sessions edit the corpus during measurement.

Guard-proof runs (the tests only mean something once shown to fail):
re-adding the old two-column fallback for
source-rooted lines broke the engine and integration suites,
and flattening the chain renderer to one region
indent broke both `renderCanonical` nesting guards and both chain autofix convergence tests.
Both mutations are kept as `gfp-mutation-1.patch` and `gfp-mutation-2.patch` in the scratch probe directory.

Measurement correction (2026-10-22):
a whole-set corpus run built on a stale `dist` bundle (written seconds before
`chain-render.ts`'s final write of the same minute,
so the bundle still held the pre-`310a3fa94c` flat renderer) reported a two-cycle between
`stylistic(indentation)` and `stylistic(chain-per-line)` on 86 and 43 residual diagnostics across
four corpus files.
Rebuilding puts the renderer back on the model's widths,
and those four files then converge to zero diagnostics in one to three `--fix` passes.
The fresh whole-set sweep reaches a fixed point at pass 12 (the eight-pass cap is depth,
not a
cycle).
Lesson:
rebuild the package before any lint-based measurement;
`mise run //package/oxlint-plugin/stylistic:test:unit` builds first,
ad-hoc oxlint runs do not.

Separate incident found by the same sweep (not this rule):
`stylistic/comma-dangle`'s fixer corrupts `;`-terminated type literals of destructured
parameters;
see `doc/troubleshooting/oxlint-stylistic-comma-dangle-type-literal-fix.md`.

## Verified workarounds

- `stylistic/indentation` in `package/oxlint-plugin/stylistic`,
  enabled per config as in
  `package/test-fixture/oxlint-stylistic/.oxlintrc.fixture.json`
  and in `package/config/oxlint/src/rule/style.ts` at `warn`,
  checks and autofixes the model.
  Tradeoff:
  the one normalization sweep rewrote 21,441 leading-whitespace runs
  in 1,845 of 7,333 candidate files
  (an earlier probe projected 20,722 to 20,725 runs
  in 1,792 to 1,793 of 7,091 candidate files),
  and a whole-set `--fix` run mixes in other rules' fixes
  unless the config also sets `categories: { "correctness": "off" }`,
  see `doc/troubleshooting/oxlint-config-default-categories.md`.
- Probe-guided manual correction:
  run `node probe4.mjs <file-list>` and fix the reported lines by hand.
  Tradeoff:
  no autofix and no lint integration,
  so drift returns;
  the alternative corpus styles listed
  under failing patterns show up as flags even where they are internally consistent.
- Classic `typescript.js` for AST work:
  fetch `typescript.js` from the TypeScript v5.9.3 release when scratch
  tooling needs the parser and `SyntaxKind`.
  Tradeoff:
  a second compiler version runs beside the repository's `typescript@7.1.0-beta`;
  fine for scratch probes,
  not for repository code.
- dprint as an indentation oracle:
  copy corpus files to scratch and run `dprint check` and `dprint fmt` with a
  scratch `dprint.json` carrying the repository's `lineWidth`,
  `indentWidth`,
  `useTabs`,
  and `"typescript"`
  settings,
  then keep only indentation hunks.
  Tradeoff:
  67.58% of dprint's changes are non-indent rewraps,
  so hunk filtering is manual,
  and dprint's
  convention is not the rule's convention.

## What does not work

- Re-enabling the dprint TypeScript plugin:
  it rewrites 195 of 200 sampled files with 4,585 non-indent rewraps
  and 1,972 TSDoc blank-line normalizations.
  That is a whole-file reformat,
  the outcome
  `.out-of-scope/low-impact-typescript-formatting.md` rejects in favor of newline-first structure work.
- The plain ancestor-plus-two model with no closer,
  suffix,
  or template handling:
  systematic errors on
  `catch`,
  `else`,
  chains,
  and template tail lines (see the retracted readings in "Root cause").
- A checker whose fixer uses another width than the existing fixers emit:
  oscillation.
  The chain renderer's
  flat region indent produced exactly that against the indent model (fixture b8 looped under `--fix` until the
  unification in commit `310a3fa94c`).
- `require('typescript')` from the repository for parsing:
  `typescript@7.1.0-beta` exposes only
  `lib/version.cjs`.
- Reproducing eslint's `lib/rules/indent.js` offset tables (2,332 lines,
  fixable whitespace):
  that rule
  encodes per-construct offsets for eslint's ecosystem,
  while this repository needs one uniform convention,
  and the shipped rule computes it in far less code.

## Implementation

The rule `stylistic/indentation` in `package/oxlint-plugin/stylistic`,
check plus autofix,
on the simplest model
the measurements support (the user's direction:
"Make the model as simple as possible.
I don't care about huge
sweeping changes.").
It landed in four commits:
`bee2e27eb7` (engine,
rule,
registration),
`2b4b5cefd1` (fixtures,
exports),
`1e6f2c0db3` (source-rooted zero and engine tests),
`310a3fa94c` (chain-nesting unification,
tests,
mixed
fixture).
Measured against the pre-rule tree the work adds 3,395 lines and removes 165 across 16 files,
inside the
2,000 to 3,400 LoC estimate and about 3 times under the user's 10,000 LoC close-as-not-planned threshold.

### The model

Every code line is judged by one of four routes.
Blank lines,
comment interiors,
template-literal content lines,
and lines whose first content is not a code
token carry no expectation and are exempt.

#### Template exemption

Template-literal content (`TemplateLiteral` spans) is exempt.

#### Ancestor plus two

A line's first token expects the indent of the line where its nearest ancestor node starting on an earlier
line begins,
plus two spaces.
A `BlockStatement` or `TSModuleBlock` ancestor remaps to its owning construct's line when that construct opens
on an earlier line and is not itself a body,
so hanging signatures and conditions anchor at their statement;
nested bare blocks keep their own base.
A line with no earlier-line ancestor (`Program`) expects zero.

#### Closer at the bracket base

A closer (`}`,
`]`,
`)`) expects the indent of the line where its opening bracket begins,
except two brace
remaps:
a `{` opening a body remaps like the ancestor route,
and a `{` whose owning construct starts earlier
and ends with the brace (class,
interface,
enum,
switch bodies) bases at the construct's line.
An unmatched closer self-matches and is exempt.

#### Suffix keyword at the owner line

Suffix keywords (`else`,
`catch`,
`finally`,
do-while `while`) expect their owner statement's line indent.

### Autofix and convergence

The fixer rewrites the leading whitespace of deviating lines only.
Expected widths are absolute:
each expectation is computed from its anchor line's expected width in one forward
pass,
so one `--fix` pass normalizes whole cascades and reaches the fixed point (measured in
"Implementation verification").
The item and invocation fixers emit anchor-line plus two (`item-per-line-fix.ts:195`,
`invocation-depth-fix.ts:293`),
and the chain fixer now indents each break two columns past its anchor line
(`chain-render.ts`),
so all four agree at the fixed point.
`doc/handover/issue-577-indent-rule-investigation.md` carries the queue beyond the rule itself.

## Upstream filing artifact

Nothing to add upstream.
The capability gap is in this repository's plugin,
and `@oxlint/plugins` 1.89.0 already ships the
report-and-fix primitives quoted in "Verification".

`.out-of-scope/` was checked:
`low-impact-typescript-formatting.md` excludes type-parameter,
type-argument,
import-attribute,
enum,
and
quote rules as low-impact polish,
and names block bodies,
lists,
and expression chains as the useful work;
indentation checking is that useful work,
so no exemption applies.

### Upstream filing decision

#### Is it really upstream's fault?

No.
dprint's formatter behavior is by design and this repository disabled the plugin deliberately
(`package/config/dprint/index.json:39`).
oxlint's plugin API supports `Context.report` with `loc` and range fixers,
so no upstream behavior blocks
the rule.

#### Can upstream fix it?

Not applicable:
nothing is broken upstream.
An upstream built-in `indent` rule would be a feature request against the eslint-core port umbrella
(`oxc-project/oxc#479`),
not a bug.

#### Are they supporting this use case?

The plugin API exists for rules like this,
and the repository's own
`.out-of-scope/low-impact-typescript-formatting.md` assigns indentation of block bodies,
lists,
and chains
to this repository's formatter work.
The use case is supported here,
not upstream.

#### Would the repo welcome our contribution?

Not applicable to upstream filing;
the work lands in this repository either way.
`CONTRIBUTING.md` and repository policy here were checked and place no restriction on in-repo lint rules.

#### Will they likely fix it?

Not applicable.
Tracker search on 2026-10-22 for indent issues in `oxc-project/oxc` found no dedicated request for a built-in
TypeScript indent lint rule.
The closest items are formatter-side indent bugs (`oxc-project/oxc#20145`,
`oxc-project/oxc#26598`) and the
jsplugin interplay report `oxc-project/oxc#24385`.

#### Have we prototyped a minimal fix compatible with their architecture?

Yes;
it is the shipped rule.
The probe implements the four-route model against the real `typescript.js` parser and the `@oxlint/plugins`
visitor shape ("Verification"),
and `package/oxlint-plugin/stylistic` now carries the full check,
autofix,
fixtures,
and convergence tests described in "Implementation".
