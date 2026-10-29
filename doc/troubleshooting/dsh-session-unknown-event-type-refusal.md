# DSH Harness 0.2.1-rc.4 refuses to load session history when a plugin appends a non-ignorable unknown event

## Symptom

Opening a session in the DSH Web GUI shows a banner instead of history
 (locale key `chat.loadError`
 in `@deepseek-ai/dsh-client-ui-chat@0.2.1-rc.4/lib/client.js:5991`):

```text
Failed to load history: failed to observe session "session-61a73d26-2266-44fa-bf73-a7a6edd855ac":
session "session-61a73d26-2266-44fa-bf73-a7a6edd855ac" contains event type "advisor/operation" (seq 550)
unknown to this harness and not marked ignorable; refusing to interpret the log — it was likely written
by a newer harness (raw log: ~/.dsh/sessions/--var-home-user-Monochromatic--/session-61a73d26-.../
session.v4.jsonl.zstd) (gateway/internal)
```

Trigger patterns and their variants:

1. Any stored `session.v4.jsonl.zstd` containing an event whose `type` is outside the
   released vocabulary `KNOWN_SESSION_EVENT_TYPES` and whose envelope lacks `ignorable: true`.
   The refusal names the first offending event by type and seq and quotes the raw-log path.
   Concretely,
   this repo's `@monochromatic-dev/dsh-plugin-advisor` wrote one `advisor/operation` event
   for every FAILED Advisor operation,
   so one failed review sealed its whole session.
2. A different variant,
   not covered here,
   is the corruption refusal
   `stored session "..." failed validation: Error: session event "user/message" sourceEventSeqs
   must densely contain non-negative safe integers`.
   See "Adjacent refusals not covered here".

## Root cause

### The sink wrote outside the runtime event vocabulary

The Advisor failure-accounting sink appended a custom event type through `Session.append`:

```ts
// package/dsh-plugin/advisor/src/session-event.ts before commit b45b149b5b
session
  .append(
  'advisor/operation',
  operation,
  );
```

A type-level augmentation of `SessionEventMap` in the same file made that call compile.
That augmentation is compile-time only;
 the runtime vocabulary is generated inside the
harness repository and cannot be extended from outside it
 (`@deepseek-ai/dsh-session@0.2.1-rc.4/lib/types/known-event-types.js:6`):

```js
// node_modules/@deepseek-ai/dsh-session/lib/types/known-event-types.js:6
* Every `SessionEventMap` member declared in this repository — the event
* vocabulary this build understands. ... Downstream (out-of-repo) plugin events are outside this list by
* construction. The persisted `SessionEvent.ignorable` marker is the
* compatibility mechanism; event-name registration was rejected because
* it does not classify omission safety and would make reads
* composition-dependent.
```

### `Session.append` cannot mark an event ignorable

The append path builds the envelope with only surface metadata
 (`@deepseek-ai/dsh-session@0.2.1-rc.4/lib/index.js:1517`):

```js
// node_modules/@deepseek-ai/dsh-session/lib/index.js:1527
const event = deepFreeze({
  type,
  seq: SessionSeq(this.log.length),
  time: Date.now(),
  data: dataSnapshot,
  ...surfaceMetadataSnapshot
});
```

There is no `ignorable` option,
 so every `Session.append` event is a REQUIRED event.

### The persistence read path refuses unknown required events

`validateStoredEvents` runs before any backend interprets a stored log
 (`@deepseek-ai/dsh-session-persistence@0.2.1-rc.4/lib/index.js:184`,
 with an identical bundled copy in
 `@deepseek-ai/dsh-session-persistence-jsonl@0.2.1-rc.4/lib/worker.cjs:7337`):

```js
// node_modules/@deepseek-ai/dsh-session-persistence/lib/index.js:184
if (!KNOWN_SESSION_EVENT_TYPES.has(event.type) && event.ignorable !== true) throw unsupported(
  `session "${meta.id}" contains event type "${event.type}" (seq ${event.seq}) unknown to this harness and not marked ignorable; refusing to interpret the log — it was likely written by a newer harness`,
  location);
```

`@deepseek-ai/dsh-session-query@0.2.1-rc.4/lib/index.js:523` wraps that message as
 `failed to observe session "..."`,
 and the gateway reports it to the GUI as `chat.loadError`.
 The same write also broke every session tool that observes the log
 (`session_event_search`,
 `session_event_read`,
 `session_trace`).

### The sanctioned seam marks plugin records ignorable

`appendPluginRecord` is the only writer of `ignorable` events
 (`@deepseek-ai/dsh-session@0.2.1-rc.4/lib/index.js:1543`):

```js
// node_modules/@deepseek-ai/dsh-session/lib/index.js:1550
#appendRecord(type, data) {
  ...
  const record = deepFreeze({
    type,
    seq: SessionSeq(this.log.length),
    time: Date.now(),
    data: dataSnapshot,
    ignorable: true
  });
  assertPluginRecordEvent(record);
```

It requires the `plugin:` type grammar
 (`plugin:[a-z0-9][a-z0-9._-]*(?:\/[a-z0-9][a-z0-9._-]*)*`),
declares payloads through the `PluginRecordMap` interface,
 and readers recover them with `pluginRecordOf`.

### Earlier reading that was wrong

The refusal text says the log was "likely written by a newer harness".
 It was not.
 The same 0.2.1-rc.4 harness wrote the event through this repo's plugin,
 which used the compile-time `SessionEventMap` augmentation instead of the runtime
 `PluginRecordMap` seam.
 Writer identity is not observable at read time,
 so the message cannot
distinguish "newer harness" from "plugin wrote outside its contract";
 treat the wording as a hint,
 not a diagnosis.

## Verification

Versions under test:
 `@deepseek-ai/dsh@0.2.1-rc.4` web profile (the running gateway),
 `@deepseek-ai/dsh-session@0.2.1-rc.4`,
 `@deepseek-ai/dsh-session-persistence@0.2.1-rc.4`,
 and this repo's `@monochromatic-dev/dsh-plugin-advisor` at commit `b45b149b5b`.
 Harness packages install under
 `~/.local/share/pnpm/global/v11/<hash>/node_modules/.pnpm/`.

Runnable harness (the durable one,
 in this repo):

```sh
# drives Session.create + appendAdvisorOperationRecord + validateStoredEvents
mise run //package/dsh-plugin/advisor:buildAndTest
```

The regression test `package/dsh-plugin/advisor/src/session-event.unit.test.ts`
 commits one operation record to a real `Session`,
 then runs `validateStoredEvents` over the committed log.
 At commit `8134d32186` (before the fix) it goes red with the exact user-visible message
 `session "session-advisor-operation-record-test" contains event type "advisor/operation" (seq 0)
 unknown to this harness and not marked ignorable`;
 after `b45b149b5b` it is green.

Raw-log catalog,
 measured on 2026-10-29 with
 `zstd -dc <log> | node <validator harness>` against `validateStoredEvents`:

1. Failing pattern (before repair):
   `~/.dsh/sessions/--var-home-user-Monochromatic--/session-61a73d26-2266-44fa-bf73-a7a6edd855ac/
   session.v4.jsonl.zstd`,
   one `advisor/operation` event at seq 550,
   refused with the verbatim symptom message.
   A scan of all 264 stored logs found exactly this one log with an unknown-type offender.
2. Working patterns (same scan):
   259 of 264 logs validate cleanly,
   including live logs of the same workspace,
   which shows the refusal tracks the offending event and not the store,
   the workspace,
   or the compression format.
3. After the envelope repair below,
   the repaired log validates cleanly with its 662 events and the ledger payload intact.

## Verified workarounds

1. **Write plugin-owned records through `appendPluginRecord`.**
   The landed fix (`package/dsh-plugin/advisor/src/session-event.ts:61`) calls

   ```ts
   appendPluginRecord(session, 'plugin:advisor/operation', operation);
   ```

   with the payload declared in `PluginRecordMap`.
   Tradeoffs:
   the record is ignorable by contract,
   so a migration or a reader that lacks the plugin may drop it
   while still interpreting the rest of the log correctly;
   the owning plugin reads it back with `pluginRecordOf`,
   which only matches `plugin:` types.
   This is the intended durability class for failure accounting.

2. **Repair an already-sealed log by renaming the envelope.**
   For each event line with `type` `advisor/operation` and no `ignorable` marker,
   rewrite the envelope to the plugin-record form,
   keeping `seq`,
   `time`,
   and `data` byte-for-byte:

   ```js
   // envelope mapping applied to the offending JSONL line
   JSON.stringify({
     type: 'plugin:advisor/operation',
     seq: record.seq,
     time: record.time,
     data: record.data,
     ignorable: true,
   })
   ```

   Recompress with `zstd`,
   validate the candidate with `validateStoredEvents` BEFORE it replaces the original,
   and keep a backup of the original.
   This mirrors the harness's own v3-to-v4 normalization for unknown ignorable events
   (`@deepseek-ai/dsh-session-format-v3-to-v4@0.2.1-rc.4/lib/index.js:1302`:

   `type: `plugin:${event.type}`, ignorable: true`).
   Tradeoffs:
   readers that specifically look for the old bare type name no longer match;
   the data payload is preserved and becomes readable through `pluginRecordOf`;
   the operation ledger itself remains failure-only,
   so completed operations still carry their diagnostics only in the tool JSON value.

## What does not work

1. **Augmenting `SessionEventMap` from a plugin.**
   It type-checks the write and nothing else.
   The runtime vocabulary is generated from the harness repository's own declarations
   and intentionally rejects registration,
   because a name alone cannot say whether omission is safe.
2. **Passing an `ignorable` flag to `Session.append`.**
   The method takes only surface metadata options
   (`sourceEventSeqs`,
   `surfaceOp`),
   so every appended event is required.
3. **Marking the event ignorable after the fact inside the log without renaming.**
   A bare `advisor/operation` event with `ignorable: true` is retained and skipped,
   but `pluginRecordOf` cannot read it back because the name fails the `plugin:` grammar,
   so the owner loses its own record.
4. **GUI-level verification from an agent.**
   The web launch token is random per process
   (`processLaunchToken` at
   `@deepseek-ai/dsh-client-connection@0.2.1-rc.4/lib/index.js:294`),
   printed only to the launching terminal,
   and the API operator cookie requires that token in `/connect`
   (`BrowserAuth.authorizeConnect`).
   The exported `DSH_WEB_URL` carries no token.
   This is a product boundary,
   not a fault;
   the human holding the terminal confirms the banner is gone by reopening the session.

## Adjacent refusals not covered here

A full-store scan on 2026-10-29 found five more logs the persistence read path refuses,
 all with a different error class
 (`SessionPersistenceCorruptionError`):
 `stored session "..." failed validation: Error: session event "user/message" sourceEventSeqs
 must densely contain non-negative safe integers`.
 Affected sessions:
 `session-19191ea7-dd70-4791-8a04-5351044f12b3`,
 `session-466cd7a6-75a0-4348-b412-8c1f2cba24b0`,
 `session-4781b58c-63fd-430f-94ec-472c10d056d1`,
 `session-4eeb1d3e-87ca-41fb-b5b7-5c73bca5d6cb`,
 and `session-ad883975-19fe-43db-a3d2-71c4024a8e26` (live at scan time).
 Separate incident,
 undiagnosed;
 do not treat the envelope repair in this doc as a remedy for it.

## Upstream filing decision

Upstream is `deepseek-ai/deepseek-harness`,
 the source of the `@deepseek-ai/dsh-*` packages.

Out-of-scope check:
 `.out-of-scope/` holds no entry for this tool or bug class
 (candidates read:
 `bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
 `codex-harness.md`,
 `jsr.md`,
 `lightningcss.md`,
 `low-impact-typescript-formatting.md`,
 `module-es-monolith.md`,
 `pi-gpt55-long-context.md`,
 `terminal-title-fork-parity-tests.md`,
 `typescript-project-references.md`).

Duplicate search on 2026-10-29:
 `gh search issues --repo deepseek-ai/deepseek-harness "ignorable"` and
 `"refusing to interpret"` returned nothing,
 and the repository has `has_issues: false`,
 so no tracker thread can exist.

Candidate kept for the record:
 the refusal wording attributes the cause only to newer harnesses
 and does not point plugin authors at the `ignorable`/`PluginRecordMap` contract.

1. **Is it really upstream's fault?**
   Partial.
   The refusal itself is correct,
   documented behavior;
   the incident is this repo's plugin writing outside its contract.
   Only the message wording is upstream's,
   and its causal claim is incomplete.
2. **Can upstream fix it?**
   Yes for the wording,
   a message-text change in `validateStoredEvents`.
3. **Are they supporting this use case?**
   Yes.
   `PluginRecordMap` and the `ignorable` mechanism exist specifically for
   downstream plugin records.
4. **Would the repo welcome our contribution?**
   No for patches.
   `CONTRIBUTING.md` states
   `We are sorry that we cannot accept external pull requests at the moment`
   and routes bug reports to GitHub Discussions.
5. **Will they likely fix it?**
   Unknown.
   No tracker thread exists,
   and the project is early stage.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   No.
   Nothing in this incident requires an upstream code change,
   so no prototype was made in their repository.

Decision:
 **do not file**.
 The incident is ours and is fixed here;
 the wording observation is not worth a public report on its own,
 constraints 1 and 4 are partial or failing,
 and constraint 6 is unmet.
 Draft kept as an auditable record,
 **do not file as-is**:

~~~md
### Title

Session-log refusal message misattributes unknown plugin events to "a newer harness"

### Body

`validateStoredEvents` in `@deepseek-ai/dsh-session-persistence` refuses a log carrying an
event type outside `KNOWN_SESSION_EVENT_TYPES` without `ignorable: true`, with the message:

> ... unknown to this harness and not marked ignorable; refusing to interpret the log —
> it was likely written by a newer harness

A downstream plugin that extends `SessionEventMap` at type level and writes through
`Session.append` produces exactly this failure on the SAME harness version, so the
"newer harness" hint misleads plugin authors toward version skew and away from the real
contract (`ignorable` + `appendPluginRecord`/`PluginRecordMap`).

Reproduction: any plugin appending `session.append('plugin-owned/type', data)` on 0.2.1-rc.4,
then reopening the session.

Suggested fix: extend the message in `validateStoredEvents` (and its bundled copy in
`dsh-session-persistence-jsonl/lib/worker.cjs`) to name both causes: a newer harness OR a
plugin record written without `appendPluginRecord`, pointing at `SessionEvent.ignorable`.
~~~
