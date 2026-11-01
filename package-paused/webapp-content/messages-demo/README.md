# messages-demo

## Status: development paused

Active development is paused pending repo-wide work.
 `mise run //packages/webapp-content/messages-demo:lint` currently reports 13 errors
 from `no-restricted-syntax/no-regex` across `src/client/composer/helpers.ts`,
 `src/client/editor/buffer-table.unit.test.ts`, `src/client/outbox.unit.test.ts`, `src/lib/markdown-stream.ts`,
 `src/lib/pagination.ts`, and `src/lib/pagination.unit.test.ts`. The refactor is deferred; resume
 by completing the no-regex sweep documented in `HANDOVER.no-regex.md`.

Demo webapp that lists user-submitted markdown messages with two hard-scale targets:

- Up to **millions of messages** in the corpus
- Up to **1 billion characters (~1 GB) per message**

Every message renders as markdown, served by h3 with heavy SSR and keyset pagination.
 The composer is a custom virtualised editor (no third-party editor library); messages above 1 MB open
 in chunk-paginated edit mode so the editor never holds more than one chunk at a time.

See `/home/user/.claude/plans/build-a-demo-of-composed-quill.md` for the full design.

## Run

```sh
mise run //packages/webapp-content/messages-demo:dev:site
mise run //packages/webapp-content/messages-demo:seed:demo      # 10 000 mixed-size messages
mise run //packages/webapp-content/messages-demo:seed:stress    # one synthetic 1 GB message
```

The server listens on `:3000` by default. Override with `--port=N` or `PORT=N`.
The SQLite database lives at `./data/messages-demo.db`. Override with `--db=PATH` or `DB_PATH=PATH`.

## Demo-grade caveats

- **Identity is cosmetic.** The dropdown selects a `user_id` that is sent in the body of writes.
 There is no authentication; any client can claim any identity. Edit and delete are gated against this trusted identity,
 but a malicious client can spoof it.
- **CSRF is not enforced.** Suitable for a local demo only.
- **Markdown dialect is CommonMark only.** No GFM tables, task lists, or footnotes.
 Reference-style links (`[text][ref]`) render as literal source
 when the definition cannot be resolved within the chunk being rendered. Inline `[text](url)` links
 and autolinks work normally.
- **Edit revisions are capped at 10.** The 11th edit returns `409`; the user is asked to "save
 as new message." This keeps copy-on-write chunk-read latency bounded without a chain-compaction job.
- **JS is required.** No `<noscript>` editor; crawlers see a banner.

## Known visual quirks

- Reference-style links with definitions that fall outside the rendered chunk boundary appear as literal source.
 Use inline links to be safe.
- The first-chunk text preview falls back to "(no text preview)" when the first chunk is a code block or image.

## Storage tiers

The composer probes IndexedDB, OPFS, and `localStorage` at startup and uses whichever pass to enhance UX:

- IDB available -> outbox persists across page refresh
- OPFS or IDB available -> tier-3 chunk navigation hits a local cache
- localStorage available -> identity and last-draft pointer survive reload

If none are available the app still works; a "volatile mode" badge appears in the composer corner
 so the user knows their unsent buffer is in-memory.

## Compile-pipeline metrics overlay

Open the composer with `?debug=1` to mount a live metrics overlay in the bottom-inline-end corner.
The overlay reads from the worker's `metrics` channel and shows:

- compile p50 / p99 (per-block micromark time, ms)
- samples (compile observations folded into the rolling 200-entry buffer)
- put queue max (peak in-flight PUTs in the upload pass)
- wasted puts (chunks recompiled before a previous PUT acked)
- transition (wall-clock time of the most recent tier 2 -\> 3 promotion)

The overlay is a debug surface only. Without `?debug=1` no metrics are collected.

## Verifications

The unit-test suite (`mise run //packages/webapp-content/messages-demo:test`) covers
the in-memory portions of the plan's verification checklist:

- 16b (piece-table normalisation): the `applyToTable` test in
  `editor/buffer-table.unit.test.ts` runs 5 000 single-character inserts and asserts
  the explicit collapse step (`resetTable + materialise`) re-anchors the table to a
  single piece without losing data. The worker's idle-triggered `setTimeout`
  cooldown that drives this in production is not exercised here; only the underlying
  primitive is.
- The compile-pipeline aggregation primitives (`median`, `percentile`) are covered
  by `composer/metrics-stats.unit.test.ts`.

The DOM- and worker-bound checks must be driven through a running browser:

- 16a (DOM-as-source-of-truth invariant): the editor mounts an automatic invariant
  check when `?editor=custom&debug=1` is on the URL. After every applied changeset
  it compares `editor.text` against the worker's `snapshot` reply and logs
  `'editor invariant violated: mirror !== worker snapshot'` to the console on
  mismatch. To verify, drive the editor through a representative scenario
  (insert / delete / paste / IME composition) and confirm the console stays clean.
- 16c (compile-pipeline metrics thresholds): the worker emits per-chunk
  `compileMs`, `putMs`, `maxPutQueueDepth`, and `wastedPuts`; the overlay aggregates
  them into the snapshot rendered in the bottom-inline-end corner. Open
  `?debug=1`, paste a 200 KB body, and read the overlay text to confirm
  `compile p50 < 8 ms`, `compile p99 < 32 ms`, `wasted puts < 10 %` of `samples`,
  and `put queue max <= 4`.

Both 16a and 16c require booting the server and a browser; they are not part of
the unit-test suite (`mise run //packages/webapp-content/messages-demo:test:unit`).
