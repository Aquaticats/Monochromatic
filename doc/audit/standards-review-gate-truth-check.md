# Standards-review gate truth-check

## Purpose

The shadow log `doc/audit/standards-review-gate-shadow.md` records what independent reviews NAMED.
The pre-registered rule scores naming only.

This truth-check audits two questions so the graduation call rests on more than naming rates:

- Whether each named superlinear cost is a real defect in the code as it stood at the named commit.
- Whether a sample of quiet landings (verdict `no`) hid superlinear cost the review missed.

Method per named-cost entry:

- Read the diff (`git show <sha> --stat`,
  `git show <sha> -- <path>`) and the cited file at the commit
  (`git show <sha>:<path>`).
- Judge the claimed shape (nested scan over unbounded data,
  repeated full walk per item,
  accumulator growth per item,
  per-entry call into a full walk) and the growth it implies.
- Classify `real` (the named superlinear cost exists at that commit;
  quote the loop or call shape and say what it
  grows with),
  `not real` (bounded input,
  the walk is amortized linear,
  or the claim misreads the code),
  or
  `unclear` (the evidence does not settle it;
  say exactly what is missing).

Growth counts only along unbounded dimensions.
A walk whose length a platform bound fixes,
such as the `cwd` to root chain bounded by the path limit,
is bounded input,
so repeated walks of it stay linear in the call count.

Each entry also checks the current working tree for the same helper.
When the current code differs and is linear the entry says "fixed forward" and names the commit
that `git log -S` or `git log -- <path>` found.

Quiet sample rule:
the 8 verdict `no` entries with the largest diff line counts in `Input size:`,
ties broken by shadow-log entry order.
That rule picks `7da722af96f5` (662),
`b7c1773fd104` (634),
`6fb171e256fa` (400),
`3659c961a1f9` (216),
`47c152f71a39` (176),
`166bd64f1a1e` (174),
`d488e9e0de17` (155),
and `c42e90ea0240` (149).
The two `c42e90ea0240` runs tie at 149 diff lines;
shadow-log entry order puts the 179.5 s run in the sample
and the 217.7 s rerun in the ninth slot.

This audit is git reads plus this one markdown write.
No synthetic host load was run.

## Summary

Named-cost cluster,
16 verdict `yes` entries:
real 14,
not real 2,
unclear 0.

Quiet sample,
8 verdict `no` entries:
clean 8,
missed 0.

Real misses:
none.

Entries classified `not real`:

- `2972f3c6a06f`:
  `findNodeModulesUp` walks the `cwd` to root chain per call,
  but that chain is path-limit bounded,
  so `r` calls cost `O(r)`,
  not the named `O(r x d)` superlinear traversal.
- `c1367ce73bad`:
  same helper copied into `wg/cli-quicker` and `slice-reading`;
  same bounded chain,
  per-allocation walks stay linear in the allocation count.

The 14 real named costs cluster in a few helpers:

- The cost-dismissal shadow log rescan (`readRecordedKeys` reading and splitting the whole growing log per event)
  in 5 entries.
- The `p-map` `fillWhenSettled` done branch (scan of `unfilledIndexes` per settled pull) in 3 entries.
- The pi extension `foldMessage` accumulator rebuild and per-event full history scan in 1 entry.
- The `two-size-ratio` argument spread into `Math.min` and `Math.max` in 2 entries.
- Per-shard re-staging of `sourceFileList`,
  the suppression directive tail parser,
  and the filing-watch poller
  in 1 entry each.

## Named-cost cluster

### `2972f3c6a06f` production tools keep working directories under `node_modules/.monochromatic` (not real)

- Named cost:
  `scratchParent` in all six copies searches the ancestor chain through `findNodeModulesUp` per
  invocation,
  twice per `runOneShard`,
  so `r` operations at ancestry depth `d` cost `O(r x d)` instead of one
  linear traversal of the shared ancestry.
- At the commit,
  the call shape exists as named:

```ts
// git show 2972f3c6a06f:package/mutation-test/cli/src/host/run-one-shard.ts
const manifestDir = await mkdtemp(join(await scratchParent(), 'mutation-manifest-'),);
const reportDir = await mkdtemp(join(await scratchParent(), 'mutation-report-'),);
```

- `scratchParent` itself walks with `findNodeModulesUp({ cwd: process.cwd(), stat, dirname, join },)`,
  one `stat` per ancestor level until root.
- Growth:
  the walk length is the component count of `process.cwd()`,
  which the platform path limit bounds
  (`PATH_MAX` on Linux),
  so each call is bounded work and `r` calls total `O(r)`.
  Nothing in the workload grows the chain,
  so no unbounded dimension makes the product superlinear.
- Current tree:
  unchanged.
  `package/mutation-test/cli/src/scratch-parent.ts` still calls `findNodeModulesUp` per call,
  `package/mutation-test/cli/src/host/run-one-shard.ts` still calls `scratchParent` twice per shard.
- Classification:
  not real.
  The repeated probe exists,
  but the named superlinear cost does not:
  bounded input,
  and the cumulative cost stays linear in the operation count.

### `d4988f4c9b57` two-size ratio linear-growth assertion, 332.7 s run (real)

- Named cost:
  `measureTwoSizeRatio` spreads the unbounded round arrays into `Math.min` and `Math.max`,
  so argument-stack space grows with caller-controlled `repeats` and can pass the engine argument limit.
- At the commit:

```ts
// git show d4988f4c9b57:package/test-harness/core-expect/src/two-size-ratio.ts
bestMs: Math.min(...baseRounds,),
worstMs: Math.max(...baseRounds,),
```

- `baseRounds` and `doubledRounds` hold one entry per measured round,
  and `repeats` is a caller parameter with
  no cap,
  so the spread grows with `repeats` and breaks the `O(1)` stack requirement of `RG2`.
- The same findings text also names `Theta(repeats x size)` measurement cost when `repeats` scales with `size`.
  That term is the budget the harness exists to spend;
  the fix-forward commit documents
  "`repeats` must stay independent of size" instead of restructuring the round loop.
- Current tree:
  fixed forward by `d488e9e0de17` (`fix(test-harness-core-expect): fold min and max,
  drop the O(n) proof claim`),
  whose body cites this shadow entry.
  `package/test-harness/core-expect/src/two-size-ratio.ts:80` and `:101` now fold with
  `smallest(baseRounds,)` and `largest(baseRounds,)` at `:289` to `:301`.
- Classification:
  real.
  The spread exists at the commit and grows with unbounded `repeats`.

### `d4988f4c9b57` two-size ratio linear-growth assertion, 158.1 s run (real)

- Named cost:
  the same `Math.min` and `Math.max` spread of round arrays into argument lists proportional to
  caller-controlled `repeats`.
- At the commit the same two spread pairs sit in `measureTwoSizeRatio`;
  this second run of the same commit
  names the same shape with the same growth.
- Current tree:
  fixed forward by `d488e9e0de17`,
  same folds as the `332.7 s` run entry.
- Classification:
  real.
  Same evidence as the `d4988f4c9b57` 332.7 s run entry.

### `8b4374358e26` copy only git-listed source files into the shard work tree (real)

- Named cost:
  every shard launch UTF-8 encodes and writes the entire unbounded `sourceFileList`,
  including reshard launches,
  so `S` launches over a list of `B` bytes cost `O(S x B)` list staging.
- At the commit,
  `orchestrateRun` computes the list once (`const sourceFileList = await listSourceFiles(...)`)
  but `runOneShard` re-stages it per launch:

```ts
// git show 8b4374358e26:package/mutation-test/cli/src/host/run-one-shard.ts
await writeFile(
  join(manifestDir, SOURCE_FILE_LIST_NAME,),
  options.sourceFileList,
  'utf8',
);
```

- Growth:
  staged bytes grow as shard launches times list bytes.
  Shard launches are unbounded (resharding on taint;
  shard count scales with mutant count) and list bytes are
  unbounded (repository size),
  so this is the named per-entry call into a full walk.
- Caveat kept in view:
  each shard work tree copies the same listed sources anyway,
  so the staging repeats work
  the design already pays per shard;
  a single staged file plus hard links would remove the repeated term.
- Current tree:
  still present.
  `package/mutation-test/cli/src/host/run-one-shard.ts:99` to `:104` still writes `options.sourceFileList`
  per launch.
- Classification:
  real.
  The per-launch full write exists at the commit and grows with both dimensions.

### `c1367ce73bad` `wg-quicker` and OCR working directories leave `/tmp` (not real)

- Named cost:
  `scratchParent` in both new modules calls `findNodeModulesUp` at every allocation,
  so `r` allocations at depth `d` cost `O(rd)` with no bound,
  cache,
  or amortization in the diff.
- At the commit the call shape matches `2972f3c6a06f`:
  `scratchDirectory` and `makeTempDir` each call
  `scratchParent`,
  and `scratchParent` walks ancestors from `process.cwd()` one `stat` per level.
- Growth:
  same as `2972f3c6a06f`.
  The chain is path-limit bounded,
  so per-allocation work is bounded and
  `r` allocations total `O(r)`.
- Current tree:
  unchanged,
  `package/wg/cli-quicker/src/scratch-parent.ts` and
  `package/translation-repair/slice-reading/src/scratch-parent.ts` still walk per call.
- Classification:
  not real.
  Bounded input;
  the named `O(rd)` superlinear growth is not there.

### `ee637f1b3b78` a Claude Code Stop hook shadow-records size dismissals in thinking (real)

- Named cost:
  `thinkingDismissalHandler` to `appendShadowRecord` to `readRecordedKeys` reads,
  splits,
  and filters
  the entire growing shadow log before selecting its last 512 lines,
  so repeated stops traverse prior records
  and cumulative work is quadratic.
- At the commit:

```ts
// git show ee637f1b3b78:package/claude-code-plugin/source/src/handler/thinking-dismissal/shadow-record.ts
raw = await readFile(logPath, 'utf8',);
const lines = raw.split('\n',).filter(function isNonEmpty(line,) { ... },);
for (const line of lines.slice(-SEEN_LOG_LINES,)) {
```

- `appendShadowRecord` calls `readRecordedKeys` before every append,
  and the handler appends one record per stop
  even with zero hits (`const foundHits = hits.length;` then `await appendShadowRecord(...)`,
  `index.ts` at the commit).
- Growth:
  `N` events over a log of `Theta(N)` bytes cost `Theta(N^2)` cumulative read and split work;
  the 512-line slice bounds only parsing,
  never the read or split.
- Current tree:
  still present.
  `package/claude-code-plugin/source/src/handler/cost-dismissal/shadow-record.ts:179`,
  `:190`,
  and `:194`
  keep the whole-file `readFile`,
  whole-content `split('\n',)`,
  and `slice(-SEEN_LOG_LINES,)`.
- Classification:
  real.
  Repeated full walk per item over unbounded,
  growing data.

### `a1649fa56f6c` the thinking-dismissal hook records at `PreToolUse`, not `Stop` (real)

- Named cost:
  every tool call reads and splits the entire accumulated `shadow.jsonl` before selecting the last
  512 lines,
  so `N` tool calls cost `Theta(N^2)` cumulative log processing.
- At the commit,
  `shadow-record.ts` holds the same `readFile` plus `split('\n',)` plus `slice(-SEEN_LOG_LINES,)`
  shape,
  and the handler now runs per tool call.
- Growth:
  same as `ee637f1b3b78`,
  with the event rate raised from stops to tool calls.
- Current tree:
  still present in `cost-dismissal/shadow-record.ts` at the same lines.
- Classification:
  real.

### `8fe47e652253` the thinking-dismissal hook observes at three hook events (real)

- Named cost:
  every event reads,
  splits,
  and filters the entire append-only shadow log before selecting its
  last 512 lines,
  so `n` fixed-size events incur `Theta(n^2)` cumulative log scanning.
- At the commit the same `readRecordedKeys` body serves all three events,
  and each event appends a record.
- Growth:
  `Theta(n^2)` in event count;
  the bounded transcript tail does not bound this helper.
- Current tree:
  still present.
- Classification:
  real.

### `0f1b7e349ee3` streaming map coverage and suppression proofs (real)

- Named cost:
  the annotated `fillWhenSettled` loop scans all of `unfilledIndexes` per `done` outcome;
  with unbounded concurrency and outstanding pulls reporting `done` in descending index order,
  lower-index entries survive repeated scans and total work is quadratic.
- The review marks the cost as belonging to the annotated implementation,
  not to the comment.
  At the commit (`p-map-iterable.ts` around `:397`):

```ts
// git show 0f1b7e349ee3:package/upstream-fork/p-map/src/p-map-iterable.ts
if (outcome.kind === 'done')
  for (const unfilledIndex of unfilledIndexes)
    if (unfilledIndex > position) {
      unfilledIndexes.delete(unfilledIndex,);
      fillSlot({ kind: 'done', },);
    }
```

- `unfilledIndexes` holds the positions of started pulls whose slots are still held (added at spawn).
  Each `done` removes only its own position plus higher ones,
  so `n` completions settling from the highest
  position down scan `n`,
  then `n-1`,
  and so on:
  `Theta(n^2)` for input-sized concurrency and backpressure.
  Ascending settle order stays linear,
  so this is a worst-case permitted order,
  not the average.
- Current tree:
  still present,
  `package/upstream-fork/p-map/src/p-map-iterable.ts:397`.
- Classification:
  real.
  Nested scan over unbounded concurrent pulls.

### `af7dc97f2544` the dismissal detector measures overall dismissals (real)

- Named cost:
  every event calls `appendShadowRecord`,
  whose `readRecordedKeys` reads and splits the entire growing
  shadow log before the 512-row slice,
  `Theta(N^2)` across `N` constant-size events.
- At the commit the same `shadow-record.ts` body underpins the renamed detector.
- Growth:
  `Theta(N^2)` in event count,
  unchanged from `ee637f1b3b78`.
- Current tree:
  still present.
- Classification:
  real.

### `efa90b57bd63` the thinking-dismissal shadow covers pi sessions (real)

- Named costs:
  `foldMessage` copies all previous session hits per matching message (`Theta(M^2)`);
  `append` scans the complete session hit history per event (`Theta(T^2)`);
  `seed` reads and splits the entire growing log per extension load.
- At the commit:

```ts
// git show efa90b57bd63:package/pi-plugin/thinking-dismissal/src/index.ts
for (const unit of units) found.push(...findSizeDismissal(unit,),);
hitsBySession.set(sessionId, [...(hitsBySession.get(sessionId,) ?? []), ...found],);
```

- `record` passes `hits: hitsBySession.get(sessionId,) ?? []` (the whole history) into `append`,
  whose deduplication loop walks every entry per event.
  `seed` loops `for (const line of raw.split('\n',))` over the whole log,
  no tail bound,
  once per extension load,
  so `L` loads over a log that grows one record per run cost `Theta(L^2)`.
- Current tree:
  still present.
  `package/pi-plugin/cost-dismissal/src/index.ts:557`,
  `:591`,
  and `:596` keep the full-history pass,
  the `push(...findSizeDismissal(unit,))` spread,
  and the array rebuild.
- Classification:
  real.
  Accumulator growth per item plus per-event full walk.

### `84c602b6a7d2` abort listener and stop tests, narrowed suppressions (real)

- Named cost:
  the pre-existing `done` branch of `fillWhenSettled` scans every remaining `unfilledIndexes` entry
  while retaining earlier pending positions,
  quadratic when concurrency and backpressure scale with input.
- At the commit the loop sits at `p-map-iterable.ts:397`,
  identical to `0f1b7e349ee3`;
  the review again marks it
  as pre-existing in the annotated region.
- Growth:
  `Theta(n^2)` in settled pulls for descending settle order,
  same analysis as `0f1b7e349ee3`.
- Current tree:
  still present at `:397`.
- Classification:
  real.

### `094f868e2b9b` the thinking-dismissal family becomes cost-dismissal (real)

- Named costs:
  `readRecordedKeys` and `appendShadowRecord` re-read,
  split,
  and filter the whole append-only shadow
  log per hook (`Theta(H^2)`);
  `sweepLogDirectory` sorts all retained log-file candidates before retention limits
  apply;
  `findNodeModulesUp` recurses the linear parent chain (minor,
  `ITR`).
- At the commit,
  `shadow-record.ts` keeps the whole-log `readFile` and `split('\n',)` per hook,
  and the newly
  added bundle `package/claude-code-plugin/cost-dismissal/bundle/node/index.mjs` carries `sweepLogDirectory`:

```js
// bundled logger-core file sink in the new bundle at 094f868e2b9b
// package/claude-code-plugin/cost-dismissal/bundle/node/index.mjs
candidates: [...await collectDayFiles({directory, days: keptDays}), ...keptRootFiles].toSorted(...)
```

- Growth:
  the hook-side rescan is `Theta(H^2)` in hook count,
  the decisive named cost.
  The retention sort is `O(n log n)` in retained log files,
  and `selectDeletions` applies `maxBytes` and
  `maxFiles` only after collection and sort,
  so the sorted input is the whole retained set.
  The `findNodeModulesUp` recursion is a linear-spine style violation,
  linear work,
  not superlinear.
- Current tree:
  the rescan is still present in `cost-dismissal/shadow-record.ts`;
  `sweepLogDirectory` with the same sort order now lives in `package/logger/core/src/sink/file-retention-sweep.ts`.
- Classification:
  real.

### `8667f7ac7cef` suppression directives name the exact replacement they hide (real)

- Named costs:
  `parseDirectiveTail` re-splits the whole directive tail and searches from index zero for every
  entry,
  `O(kn)` for `k` entries in an `n`-character tail;
  `foldEntries` rebuilds the replacement set per
  restricted entry,
  `O(k^2)` copied elements.
- At the commit:

```ts
// git show 8667f7ac7cef:package/mutation-test/cli/src/engine/suppression.ts
const end = position.text.split('',).findIndex(function isContent(char, index): boolean {
  return (index >= position.index) && (!isSpace(char,),);
},);
```

- `skipSpaces`,
  `readFamilyName`,
  and `readBareReplacement` each call `position.text.split('',)`,
  an `O(n)` split of the whole tail per call,
  then `findIndex` with an index guard that still visits
  every earlier character.
  `parseDirectiveTail` calls these per entry with an advancing cursor,
  so fixed-size entries give
  `Theta(k^2)` total.
- `foldEntries` does `new Set([...(replacements.get(entry.family,) ?? []), entry.replacement],)`,
  copying the accumulated set on every addition.
- Current tree:
  still present.
  `package/mutation-test/cli/src/engine/suppression.ts:174`,
  `:230`,
  and `:395` keep the per-call `split('',)`,
  and `:627` keeps the set rebuild.
- Classification:
  real.
  Repeated full walk per item and accumulator growth per item.

### `e0a89e56e15f` the filing-watch poller for troubleshooting docs (real)

- Named cost:
  `readComments` and the module polling loop fetch,
  parse,
  and filter the entire comment history on
  every poll,
  so one non-triggering comment per poll makes cumulative work `1 + 2 + ... + n`.
- At the commit:

```ts
// git show e0a89e56e15f:.agents/skills/troubleshooting-doc/poll-issue.ts
const readComments = (): readonly Comment[] => JSON.parse(`[${gh(`repos/${repo}/issues/${issue}/comments`,)
  .replaceAll('][', ',',)...}]`,) as Comment[];
while ((Date.now() - started) / 1000 < deadlineSeconds) {
  const comments = readComments();
```

- Growth:
  `Theta(p^2)` in poll count,
  since the comment count grows with the polls and each poll re-walks all
  of it;
  the `gh --paginate` fetch repeats the whole history too.
- Current tree:
  the helper moved into `.agents/skills/troubleshooting-doc/SKILL.md` at commit `6efb1bef76`
  (`docs(troubleshooting-doc): carry the filing-watch poller inside the skill, not as a linted repo script`),
  carrying the same `readComments` body (`SKILL.md:807`) and the same per-poll re-read (`SKILL.md:822`).
  That is a relocation,
  not a fix;
  the named cost persists where the skill runs it.
- Classification:
  real.
  Repeated full walk per poll over unbounded history.

### `c46c066470cf` narrow the p-map suppression comments (real)

- Named cost:
  in unordered mode with input-sized concurrency and backpressure,
  already-started pulls reporting
  `done` in descending position make successive calls scan the remaining lower `unfilledIndexes` without
  removing them,
  `Theta(n^2)` across a batch of `n` completions;
  the review notes the issue is pre-existing.
- At the commit the loop sits at `p-map-iterable.ts:395` to `:401`,
  the same body quoted at `0f1b7e349ee3`.
- Growth:
  `Theta(n^2)` in settled pulls,
  worst-case permitted settle order.
- Current tree:
  still present at `:397`.
- Classification:
  real.

## Quiet sample

### `7da722af96f5` standalone scripts keep scratch under `node_modules/.monochromatic` (clean)

- Subject:
  `fix(*): standalone container and browser-test scripts keep scratch under node_modules/.monochromatic`
  (Refs #1810).
  Input size:
  662 diff lines,
  21 code files.
- Changed code:
  each `.mjs` script drops `tmpdir()`,
  adds a `scratchBase` constant under
  `import.meta.dirname`,
  one `mkdirSync` at load,
  and feeds it to `mkdtemp`.

```js
// package/music-player/design/cover-picker-revisit-test.mjs at 7da722af96f5
const scratchBase = join(import.meta.dirname, 'node_modules', '.monochromatic', 'music-player-design');
makeScratchBase(scratchBase, { recursive: true });
```

- Judgment:
  clean.
  Per-script work is constant;
  no ancestor walk,
  no loop over unbounded data,
  and the
  remaining diff lines are context.
  The review verdict `no` is a correct quiet.

### `b7c1773fd104` cover the source file list and rsync arguments (clean)

- Subject:
  `test(mutation-test/cli): cover the source file list and rsync arguments, document the work-tree copy`.
  Input size:
  634 diff lines,
  3 code files.
- Changed code:
  `listSourceFiles` names its `result.stdout.split('\0',)` result before counting entries
  (one linear split,
  `package/mutation-test/cli/src/host/source-file-list.ts:76`),
  plus two test files whose
  fixtures map over bounded constant lists (`GIT_MARKER_DIRECTORIES`,
  `RSYNC_EXCLUDES`).
- Judgment:
  clean.
  No unbounded scan;
  the tests write bounded fixture trees.
  Verdict `no` is correct quiet.

### `6fb171e256fa` select p-memoize cases by file stem (clean)

- Subject:
  `test(upstream-fork-p-memoize): select the shared-generation registry, clear thenable, and decorator
  receiver cases by file stem`.
  Input size:
  400 diff lines,
  3 code files.
- Changed code:
  three test files,
  366 insertions of sequential async cases with no `for`,
  `while`,
  or
  repeated-scan constructs (searched with `rg -n 'for \(|while \(|\.repeat\(|flatMap'` over each file at the
  commit,
  zero matches).
- Judgment:
  clean.
  Verdict `no` is correct quiet.

### `3659c961a1f9` hand cargo binstall a GitHub token (clean)

- Subject:
  `fix(file-enforcer/core): hand cargo binstall a GitHub token so an exhausted API quota cannot silently
  trigger a source build`.
  Input size:
  216 diff lines,
  5 code files.
- Changed code:
  `resolveGithubToken` reads `env.GITHUB_TOKEN` before the process cache and stops caching the
  explicit value;
  `installCargoBinstall` spreads `toolchainPathEnv(...)` and adds `GITHUB_TOKEN`.
  Both are constant-time per call;
  the env object spread is bounded by environment size.
- Judgment:
  clean.
  Verdict `no` is correct quiet.

### `47c152f71a39` adopt `monochromatic-git-policy-cli` 0.4.0 (clean)

- Subject:
  `chore(toolchain): adopt monochromatic-git-policy-cli 0.4.0 as toolchain:update resolves it`.
  Input size:
  176 diff lines,
  2 code files.
- Changed code:
  version strings,
  artifact URLs,
  and checksums in `plan.tools.data.ts` and
  `plan.artifacts.data.ts`,
  plus `toolchain.lock.json` and manager manifests.
  Literal data only.
- Judgment:
  clean.
  No processing code changes,
  so no cost shape to carry.
  Verdict `no` is correct quiet.

### `166bd64f1a1e` add `no-system-tmpdir` (clean)

- Subject:
  `feat(oxlint-plugin/no-restricted-syntax): add no-system-tmpdir, which bans the system temporary
  directory` (Refs #1810).
  Input size:
  174 diff lines,
  4 code files.
- Changed code:
  `package/oxlint-plugin/no-restricted-syntax/src/rule/no-system-tmpdir.ts` builds one
  `ImportDeclaration` visitor;
  the inner `for (const specifier of node.specifiers)` walks one declaration's
  specifiers in the lint pass.
- Judgment:
  clean.
  One pass over the AST,
  linear in input size;
  no nested rescan.
  Verdict `no` is correct quiet.

### `d488e9e0de17` fold min and max, drop the `O(n)` proof claim (clean)

- Subject:
  `fix(test-harness-core-expect): fold min and max, drop the O(n) proof claim`.
  Input size:
  155 diff lines,
  1 code file.
- Changed code:
  `smallest` and `largest` fold the round arrays one element at a time
  (`for (const value of values)` in `package/test-harness/core-expect/src/two-size-ratio.ts:80` and `:101`),
  replacing the spread pairs,
  and the docs stop claiming an `O(n)` proof.
- Judgment:
  clean.
  The replacement folds are linear with constant stack,
  and nothing else in the diff loops.
  Verdict `no` is correct quiet.
  This commit is also the fix-forward for the named cost in both `d4988f4c9b57`
  entries.

### `c42e90ea0240` keep CR-separated rows, normalize both version columns (clean)

- Subject:
  `fix(file-enforcer-core): keep CR-separated rows, normalize both version columns`.
  Input size:
  149 diff lines,
  2 code files.
  This is the 179.5 s run;
  the 217.7 s rerun ties at 149 and sits
  ninth by shadow-log entry order.
- Changed code:
  `splitRows` in `package/file-enforcer/core/src/toolchain/providers.parse.ts:99` walks the
  provider output once (`for (let index = 0; index < text.length; index += 1)` with `charAt` and `slice`),
  and both parsers call it instead of `split('\n',)`.
  `withoutVersionPrefix` is a prefix strip;
  `findColumnGaps` is a single pass per row.
- Judgment:
  clean.
  One linear pass with constant stack,
  exactly the `ITR` shape.
  Verdict `no` is correct quiet.
