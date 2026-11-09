# pi 0.70.6 `/compact` on small sessions produces a blank "(none)" summary because `prepareCompaction` returns an empty `messagesToSummarize` and `compact()` does not guard the non-split branch

Upstream:
[earendil-works/pi-coding-agent](https://github.com/earendil-works/pi-coding-agent),
`src/core/compaction/compaction.ts`.
 Date 2026-04-28.

Note 2026-11-09T16:56:27-05:00:
these citations in this record name paths that no longer exist.
`package/pi-plugin/morph-compact/` is now `package-paused/pi-plugin/morph-compact`,
moved 2026-10-29 in `64046360d2`.
`package/pi-plugin/morph-compact` is now `package-paused/pi-plugin/morph-compact`,
moved 2026-10-29 in `64046360d2`.

## Symptom

Running `/compact` on a small pi session,
 or auto-compaction
firing when the entire conversation fits within
`keepRecentTokens`,
 produces a blank summary with "(none)" in
every section and writes a useless compaction entry to the
session file:

```text
[compaction]

Compacted from 10,857 tokens

 Goal

 (none - no conversation provided)

 Constraints & Preferences

 - (none)

 Progress

 ### Done

 - (none)

 ...
```

The entry is written with `fromHook: false` and empty
`details` (`readFiles: []`,
 `modifiedFiles: []`),
 polluting the
session with a useless compaction record.

Morph Compact extensions also fail silently in this scenario
because `messagesToSummarize` is empty,
 causing the extension
to fall through to pi's default summariser.

## Root cause

Two bugs in pi's compaction pipeline:

### Bug A: `prepareCompaction` returns a preparation with empty `messagesToSummarize`

Source:
 `src/core/compaction/compaction.ts`,
`prepareCompaction()` function.

When the total session tokens are below `keepRecentTokens`
(default 20,000),
 `findCutPoint()` sets
`firstKeptEntryIndex` to 0 (keep everything from the
beginning).
 This makes `historyEnd` equal to
`firstKeptEntryIndex` (0),
 so the loop that populates
`messagesToSummarize` never executes:

```ts
// src/core/compaction/compaction.ts, prepareCompaction()
const historyEnd = cutPoint.isSplitTurn
  ? cutPoint.turnStartIndex
  : cutPoint.firstKeptEntryIndex;

// This loop never executes when historyEnd === boundaryStart === 0
const messagesToSummarize: AgentMessage[] = [];
for (let i = boundaryStart; i < historyEnd; i++) {
  const msg = getMessageFromEntryForCompaction(pathEntries[i],);
  if (msg)
    messagesToSummarize.push(msg,);
}
```

`prepareCompaction` returns a valid `CompactionPreparation`
object with `messagesToSummarize: []` and
`tokensBefore > 0`.
 The caller in `agent-session.ts` does not
check whether `messagesToSummarize` is empty before
proceeding with compaction.

Compare with the early-return guard in `agent-session.ts`
line 1256:

```ts
const preparation = prepareCompaction(pathEntries, settings,);
if (!preparation) {
  // Check why we can't compact
  const lastEntry = pathEntries[pathEntries.length - 1];
  if (lastEntry?.type === 'compaction')
    throw new Error('Already compacted',);
  throw new Error('Nothing to compact (session too small)',);
}
```

The guard catches `undefined` but not a preparation with
empty messages.

### Bug B: `compact()` calls `generateSummary([])` without an empty guard

Source:
 `src/core/compaction/compaction.ts`,
 `compact()`
function,
 line 574.

When `messagesToSummarize` is empty and the compaction is
not a split turn,
 `compact()` unconditionally calls
`generateSummary(messagesToSummarize, ...)` with an empty
array.
 The summariser serialises an empty conversation
(`<conversation>\n\n</conversation>`) and asks the LLM to
summarise it,
 producing the "(none)" template.

The split-turn branch already has a guard for this case at
line 565:

```ts
// Line 565: split-turn branch has a guard
messagesToSummarize.length > 0
  ? generateSummary(messagesToSummarize, ...)
  : Promise.resolve("No prior history."),

// Line 574: non-split branch has NO guard
summary = await generateSummary(messagesToSummarize, ...);
```

The non-split path should have the same `"No prior history."`
fallback,
 or better yet,
 compaction should not proceed at all
when there is nothing to summarise.

## Verification

Version under test:

- pi 0.70.6
- Upstream source examined:
   `earendil-works/pi-coding-agent`,
  `src/core/compaction/compaction.ts`

Reproduce:

```bash
# Start a fresh pi session with a large system prompt (AGENTS.md, etc.)
pi
# Exchange a single message
> Hello?
< Hey there!

# Manually trigger compaction
/compact

# Result: blank "(none)" summary written to the session file
```

Auto-compaction also triggers this when
`contextTokens > contextWindow - reserveTokens` but the
entire session fits within `keepRecentTokens` (20K).
 This
can happen with models that have small context windows,
 or
sessions where system-prompt tokens dominate but
conversation tokens are minimal.

## Verified workaround

The Morph Compact extension
(`package/pi-plugin/morph-compact/`) returns `{ cancel: true }`
from its `session_before_compact` handler when
`messagesToSummarize` is empty and no `previousSummary`
exists:

```ts
// package/pi-plugin/morph-compact/src/index.ts
const hasMessages = messagesToSummarize.length > 0
  || turnPrefixMessages.length > 0;
if (!hasMessages && previousSummary === undefined) {
  ctx.ui.notify(
    'Morph Compact: nothing to compact (session too small)',
    'warning',
  );
  return { cancel: true, };
}
```

`package/pi-plugin/morph-compact` moved on 2026-10-29 to `package-paused/pi-plugin/morph-compact`
in `64046360d2` (refactor(*):
pause pi-plugin/morph-compact by moving it to package-paused (Refs #1810)).

For manual `/compact`,
 pi throws "Compaction cancelled".
 For
auto-compaction,
 pi silently skips the compaction cycle.
Both are correct behaviours when there is nothing meaningful
to compact.

Tradeoff:
 requires the Morph Compact extension to be
installed and active.
 Without the extension,
 the workaround
is to avoid running `/compact` on sessions that are too
small to compact.

## What does not work

- Returning `undefined` from a `session_before_compact`
  extension handler:
   falls through to pi's default
  summariser,
   which hits the same bug.
- Setting `compressionRatio` to any value in Morph Compact:
  the issue is that `messagesToSummarize` is empty **before**
  Morph is ever called.
- Manually deleting the useless compaction entry after the
  fact:
   removes the entry but does not prevent the next
  compaction from creating another.

## Why we would file this upstream

All 5 constraints hold:

1. **Is it really upstream's fault?
   ** Yes;
    both
   `prepareCompaction` and `compact()` should reject empty
   message lists.
2. **Can upstream fix it?
   ** Yes;
    two small patches sketched
   below;
    either alone would resolve the symptom.
3. **Are they supporting this use case?
   ** Yes;
    small
   sessions are explicitly in scope for compaction.
4. **Will they likely fix it?
   ** Plausible;
    pi is actively
   maintained.
5. **Have we prototyped a minimal fix?
   ** Yes;
    both patches
   below are minimal and exercise the same code paths the
   workaround in Morph Compact already validates.

Decision:
 worth filing.

### Suggested upstream fix

Option A:
 `prepareCompaction` returns `undefined` when
`messagesToSummarize` is empty and no `previousSummary`
exists,
 matching the caller's expectation that `undefined`
means "nothing to compact":

```ts
// At the end of prepareCompaction(), before the return:
if (messagesToSummarize.length === 0
  && turnPrefixMessages.length === 0
  && !previousSummary)
{
  return undefined;
}
```

Option B:
 `compact()` guards against empty
`messagesToSummarize` in the non-split-turn path,
 mirroring
the split-turn guard:

```ts
// In compact(), the non-split else branch:
else {
  summary = messagesToSummarize.length > 0 || previousSummary
    ? await generateSummary(messagesToSummarize, ...)
    : "No prior history.";
}
```

Option A is preferred because it prevents the empty
compaction entry from being written at all;
 Option B fixes
the summary content but still writes the entry.

## Draft upstream issue (kept as reference; revise before filing)

```md
**Title**: Compaction produces empty "(none)" summary when session fits within keepRecentTokens

**Labels**: bug, compaction

**Description**:

### Reproduction

1. Start a fresh pi session.
2. Exchange a single message (e.g. "Hello?" / "Hey there!").
3. Run `/compact`.

### Expected behaviour

Compaction should either refuse ("Nothing to compact") or produce a meaningful summary.

### Actual behaviour

Compaction writes a blank entry with "(none)" in every section.

### Root cause

1. `prepareCompaction()` returns a `CompactionPreparation` with `messagesToSummarize: []` when the entire session fits within `keepRecentTokens` (20K). The caller in `agent-session.ts` only checks for `undefined`, not empty messages.

2. `compact()` calls `generateSummary([])` without checking whether `messagesToSummarize` is empty. The split-turn branch at `src/core/compaction/compaction.ts:565` already has this guard (returns `"No prior history."`); the non-split branch at line 574 does not.

### Suggested fix

Option A: `prepareCompaction` returns `undefined` when `messagesToSummarize` is empty and no `previousSummary` exists.

Option B: `compact()` guards the non-split-turn branch with the same `messagesToSummarize.length > 0` check used in the split-turn branch.

### Environment

- pi: 0.70.6
- OS: linux
```
