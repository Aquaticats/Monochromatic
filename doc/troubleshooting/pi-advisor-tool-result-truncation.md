# Pi Advisor with SDK 1.0.0: source reads lose their tails in review context

Note 2026-11-09T16:51:44-05:00:
`package/pi-plugin/advisor/src/context.ts` is now `package/agent-harness-shared/advisor/src/context.ts`,
moved 2026-10-20 in `093f32832a`.

## Symptom

The primary agent read complete source files,
but Advisor reported omitted characters and refused exact-source admission.
Re-reading the same files in moderately sized chunks did not resolve the omissions.
Chunks whose complete tool-result text fit within 2,000 JavaScript string code units did.

This is separate from provider timeouts and whole-conversation truncation.
The observed review holds concerned missing source evidence,
not a discovered defect in the diagnostic being reviewed.

## Root cause

The current repository's Advisor uses a summarization serializer for review context.
`package/pi-plugin/advisor/src/context.ts:143` calls:

```typescript
// package/pi-plugin/advisor/src/context.ts
const serialized = serializeConversation(convertToLlm(messages,),);
```

`package/pi-plugin/advisor/src/context.ts` moved on 2026-10-20 to
`package/agent-harness-shared/advisor/src/context.ts` in `093f32832a`
(refactor(agent-harness-shared-advisor):
move the advisor core files).

In the installed `@earendil-works/pi-coding-agent@1.0.0`,
`dist/core/compaction/utils.js:82` fixes a per-tool-result limit:

```javascript
// @earendil-works/pi-coding-agent/dist/core/compaction/utils.js
const TOOL_RESULT_MAX_CHARS = 2000;
```

`dist/core/compaction/utils.js:87` defines `truncateForSummary`.
Its return at line 91 retains only the prefix:

```javascript
// @earendil-works/pi-coding-agent/dist/core/compaction/utils.js
return `${text.slice(0, maxChars)}\n\n[... ${truncatedChars} more characters truncated]`;
```

`dist/core/compaction/utils.js:137` applies it to each tool result:

```javascript
// @earendil-works/pi-coding-agent/dist/core/compaction/utils.js
parts.push(`[Tool result]: ${truncateForSummary(content, TOOL_RESULT_MAX_CHARS)}`);
```

The serializer's source comment explicitly targets summarization,
where it says full tool content is unnecessary.
That assumption does not supply complete evidence for an exact-source review.
Reading a complete file at the primary agent boundary therefore does not prove the reviewer received it.

The installed helper was reproduced directly.
The live session's precise loaded package identity was not independently captured;
this source trace establishes the current checkout's call path and installed helper behavior.

## Verification

The private qualification repository retains `contract/diagnostic/advisor-context/`.
Its synthetic `mise --no-env --no-hooks run check` probe exited 0 as `proc_5995`.
It imported the installed SDK helper without creating an agent session or calling a provider.

The inspected helper's SHA-256 is:

```text
f7b64982eba185f03051cc4baedf52fdb5747bb2104d2398aec3426d02a5aa80
```

Passing patterns:

- A tool result containing exactly 2,000 `a` characters is preserved.
- Splitting a 2,013-character string into separate tool results preserves both portions.

Failing pattern:

- One tool result containing 2,000 `a` characters followed by `TAIL_SENTINEL`
  loses the sentinel and gains `[... 13 more characters truncated]`.

The live workaround likewise resolved the source-review holds:
reading the unchanged files through EOF in shorter results allowed admission of the original frozen diagnostic.
No preparation,
controls,
freeze,
or model experiment was replayed to repair the evidence transport.

## Verified workaround

Split source reads so each complete tool result stays below the serializer limit,
including tool-added annotations.
The successful source review used 5 to 20 lines per read,
with shorter chunks around long lines.
Line counts alone are not a size guarantee.

Tradeoffs:
more tool results and repeated context consume space.
Compaction or a separate overall context limit can still remove evidence.
Confirm that the reviewer received the entire required source,
rather than treating a successful primary read as that confirmation.

## What does not work

- Asserting that a complete primary read proves complete reviewer visibility.
- Repeating large reads unchanged.
- Using chunks that still exceed the per-result limit.
- Re-running a paid diagnostic when only its review transport is incomplete.

## Upstream filing decision

No upstream issue or patch is filed.
This is a mismatch between a repo-owned review consumer and a documented summarization-oriented helper.

1.  Upstream fault is not established:
    the helper implements its stated summarization behavior.
2.  A remedy is possible at the consumer boundary,
    but no installed implementation change was made.
3.  Exact-source reviewing is not the inspected helper's stated use case.
4.  Upstream contribution receptiveness was not investigated because upstream responsibility is not established.
5.  Upstream willingness was not investigated;
    no claim about maintainer intent follows.
6.  The chunking workaround was verified,
    but no serializer replacement was prototyped or adopted.

There is no supported upstream filing to draft from this evidence.
A future local serializer proposal belongs in `doc/planning/` and must preserve review context without importing
summarization's per-result omission into an exact-source claim.
