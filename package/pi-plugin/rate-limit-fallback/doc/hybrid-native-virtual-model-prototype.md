# Hybrid native virtual model prototype: findings

**Status: unapproved, non-final.** Written by an agent on 2026-11-19, in the worktree
`.claude/worktrees/pi-rate-limit-fallback-hybrid-prototype`, branch
`feat/pi-plugin-rate-limit-fallback-hybrid-prototype-native-virtual-model-routes-to-the-pool`.
The answers to the short review, and the new captures they cite, were written the same day in
`.claude/worktrees/pi-qwen-review-rate-limit-fallback-hybrid-prototype-at-7328d0ca9d` on the same
branch. Nothing here is adopted: the owner decides whether a rebuild follows.

This page answers one question from the owner's decision of 2026-11-19
(`doc/handover/questions-for-the-owner.md`, "Virtual provider point 4", third answer, "Hybrid,
prototype first"): does a native Pi virtual model whose router sends every request to this package's
pool provider keep the package's in-request member fallback and regain Pi's context-overflow
compact-and-retry?

Each item below names the session that showed it, the command that started it, the result, and the
capture. Captures are folders under
`${HOME}/temp/agent/rate-limit-fallback-hybrid-prototype-2026-11-19/scratch/`; each holds the Pi event
stream verbatim (`records.jsonl`, or `turn-N.jsonl` for a multi-turn run), `stderr.log` with the
prototype's evidence lines, and this package's own log for the same session
(`package-decision-log.jsonl`, or `package-decision-log-turn-N.jsonl` where one turn is one process).
Every cited run used `pi` 1.5.2 and plain CLI options; no script drove it.

## What the prototype is

The prototype is a Pi extension that registers, into the session that loads it:

- `rate-limit-fallback/hybrid-writer`, a native virtual model whose router resolves
  `rate-limit-fallback/writer` from the catalog every request;
- `rate-limit-fallback/hybrid-probe`, the same router over a probe pool whose members one session
  chooses through `PI_HYBRID_PROBE_MEMBERS`;
- `rate-limit-fallback/hybrid-native-probe`, a router straight to one stand-in member with no pool,
  the native-only control arm, chosen through `PI_HYBRID_NATIVE_TARGET`;
- `rate-limit-fallback-probe/probe`, the probe pool itself: this package's `dispatchRequest`, over a
  member list the session names;
- `hybrid-probe-member/*`, stand-in members with no network behind them.

Files, all in `package/pi-plugin/rate-limit-fallback/src/hybrid-prototype/`:

- `hybrid-extension.ts`: the extension entry, the routers' ids, and the environment reads. Its module
  doc is the prototype's top doc;
- `prototype-routing-evidence.ts`: the read-only router views and the evidence lines;
- `prototype-pool-provider.ts`: the probe pool provider over `dispatchRequest`;
- `stand-in-member-provider.ts`: the stand-in members and their behaviors;
- `captured-refusal-text.ts`: every refusal body, each with its capture or source;
- `prototype-evidence.ts`: the stderr evidence line and the limit projection.

The prototype touches shipped code in one place: `src/dispatch-request.ts` takes an optional `pools`
table, defaulting to `POOLS`, so the prototype's probe pool can reuse the package's exact dispatch
loop. Its pool id check is `Object.hasOwn(pools, id)`, so a key inherited from `Object.prototype`
(`constructor`, `toString`, `__proto__`) is refused with `PoolDispatchError` instead of reaching the
member loop. Beside that, two changes ship for the package's own tests and its lint cap: the
caller-options projection moved unchanged into `src/member-attempt-options.ts`, to keep
`src/dispatch-request.ts` under the repository's 300-line cap (rule `MXL`), and `src/index.ts`
re-exports `dispatchRequest`, `PoolDispatchError`, `RoutingSession` and `Member` as `@internal` so a
built-artifact test can reach the pool table.

Sessions load the prototype by `--no-extensions --extension <path>`, which is the only per-session
option Pi offers for this; the installed package in `~/.pi/agent/settings.json` stays untouched and
unloaded. `--no-extensions` also means Pi's other provider extensions do not load, so a session sees
only builtin providers plus whatever else the command names explicitly. Item 1's second run names the
NeuralWatt provider extension for that reason.

## Item 1: a router to the pool answers a real turn

Command, with `${EXT}` naming `src/hybrid-prototype/hybrid-extension.ts` in this worktree and `${NW}`
naming `package/pi-plugin/neuralwatt/src/index.ts`:

```sh
# package/pi-plugin/rate-limit-fallback/doc/hybrid-native-virtual-model-prototype.md
pi --mode json --no-extensions --extension "${EXT}" --extension "${NW}" \
  --model rate-limit-fallback/hybrid-writer --no-tools --no-context-files --no-skills \
  --no-prompt-templates --no-mcp --no-session "Reply with the single word ready." \
  > records.jsonl 2> stderr.log
```

Result: the selection stays `rate-limit-fallback/hybrid-writer`, and the assistant message records
`provider: "neuralwatt", model: "deepseek-v4.1-flash"`, `stopReason: "stop"`, content `ready`, usage
`input: 131, output: 3, cacheRead: 512`. This package's own log for the same session reads
`[dispatchRequest] writer: dispatching to neuralwatt/deepseek-v4.1-flash`. So the virtual model routed,
the pool dispatched, the first writer member answered, and the event stream names that member.

Capture: `cli-item1-hybrid-real-writer/`.

A second run of the same command without `--extension "${NW}"` shows the same router answering through
the only member still registered, `opencode-go/deepseek-v4.1-flash` (usage `input: 646, output: 3`),
with the package log naming two registry misses before it:
`writer: missing registry model neuralwatt/deepseek-v4.1-flash` and the flex lane. That run is also
where item 3's admission shift is visible.
Capture: `cli-item1-hybrid-real-writer-opencode-only/`.

**Verdict: passed.**

## Item 2: a forced overflow recovers under the hybrid, and the direct selection does not

Forcing a real member's overflow was not affordable. The cheapest real member window in the pools is
Synthetic's 524.3 thousand tokens, and Pi compacts before sending whenever the routed model's window is
too small, so a real overflow costs hundreds of thousands of input tokens on a subscription plan. The
probe therefore forces the overflow on a stand-in member whose declared window is the writer pool's
`opencode-go/deepseek-v4.1-flash` member's (`contextWindow: 1000000` in the installed catalog,
`@earendil-works/pi-ai` `dist/providers/data/opencode-go.json`) while its enforced window is smaller,
the way a provider reserves output room. The overflow body is the one Pi's own detector documents for
an OpenAI-compatible member (see "Fixture provenance").

Three prompts, in one session per arm: two quiet padded turns, then a padded third turn that pushes the
request past the stand-in's enforced window.

```sh
# package/pi-plugin/rate-limit-fallback/doc/hybrid-native-virtual-model-prototype.md
# One run folder for both arms, so both system prompts carry the same cwd section and the two
# transcripts hold identical request sizes at every turn. One process per prompt, three prompts,
# one session id per arm; prompt-turn-1.txt to prompt-turn-3.txt sit in the run folder.
export PI_HYBRID_PROBE_MEMBERS='[{"provider":"hybrid-probe-member","id":"small-window"},
{"provider":"hybrid-probe-member","id":"answers"}]'
for n in 1 2 3; do
  # arm A, hybrid:
  pi --mode json --no-extensions --extension "${EXT}" --model rate-limit-fallback/hybrid-probe \
    --no-tools --no-context-files --no-skills --no-prompt-templates --no-mcp \
    --session-dir "$PWD/hybrid/sessions" --session-id cli-hybrid-overflow \
    "$(cat "prompt-turn-${n}.txt")" > "hybrid/turn-${n}.jsonl" 2>> hybrid/stderr.log
  # arm B, control, the same pool provider selected directly:
  pi --mode json --no-extensions --extension "${EXT}" --model rate-limit-fallback-probe/probe \
    --no-tools --no-context-files --no-skills --no-prompt-templates --no-mcp \
    --session-dir "$PWD/control/sessions" --session-id cli-control-overflow \
    "$(cat "prompt-turn-${n}.txt")" > "control/turn-${n}.jsonl" 2>> control/stderr.log
done
```

Arm A, the hybrid, produced this event order: the third request fails with
`400: {"error":{"message":"Input length (148179) exceeds model's maximum context length (140000)."…`,
then `entry_appended` (a `context_edit` with `replacement: null`, Pi omitting the failed attempt),
then `compaction_start` with `reason: "overflow"`, `compaction_end` with `willRetry: true`,
`tokensBefore: 36764` and `estimatedTokensAfter: 23281`, then a fresh run whose router evidence line
reads `reason: "retry"`, `failedModel: "hybrid-probe-member/small-window"`,
`failedStopReason: "error"`. The compaction's own summary request is the `reason: "direct"` line
before it, 55405 characters. That retried request carries 93985 characters, under the stand-in's
140000, and the member answers `stop`. Four assistant messages: two answers, the overflow, the
recovered answer.

Arm B, the control, ran from the same folder and sent the same three requests: 56172, 100514 and
148179 characters, the last refusing with the same body. It shows no `entry_appended`, no
`compaction_start`, no retry and no fourth message: the turn ends on the member's error. Three
assistant messages. The only difference between the arms is the selected model.

Capture: `item2-samecwd/hybrid/` and `item2-samecwd/control/`.

The padding had to be shaped for Pi, not for the probe. Earlier two-turn attempts, kept as the first
two sessions in `item2-hybrid-overflow/`, overflowed and then compacted nothing: 115663 then 180106
characters, and 86963 then 161345, each ending on the 400 with no summary request and no
`reason: "retry"` route line. When the first turn alone exceeds Pi's `keepRecentTokens` budget,
`prepareCompaction` has nothing to summarize and returns nothing, so the failed turn cannot retry
(`@earendil-works/pi-coding-agent` `dist/core/compaction/compaction.js:642-643`, with
`findProjectedCutPoint` at line 628 placing the cut at the first user cut point of the oversized
turn). Compaction runs in Pi before the pool is involved, so that is a transcript property, not a
difference between the arms; `item2-control-direct-pool/` holds only its own three-turn run. Three
turns with a mid-transcript cut point avoid it.

**Verdict: passed, both arms as required.** The probe tells them apart: under the hybrid Pi compacts
and retries and the turn completes; with the pool selected directly Pi does neither.

## Item 3: a member refusal still moves on inside one request

Arm: the hybrid over a probe pool whose first member refuses and whose second answers.

```sh
# package/pi-plugin/rate-limit-fallback/doc/hybrid-native-virtual-model-prototype.md
PI_HYBRID_PROBE_MEMBERS='[{"provider":"hybrid-probe-member","id":"refuses-neuralwatt-429"},
{"provider":"hybrid-probe-member","id":"answers"}]' \
pi --mode json --no-extensions --extension "${EXT}" --model rate-limit-fallback/hybrid-probe \
  --no-tools --no-context-files --no-skills --no-prompt-templates --no-mcp --no-session \
  "Reply with the single word ready." > records.jsonl 2> stderr.log
```

The first member's refusal is NeuralWatt's captured overage 429 (see "Fixture provenance"). Pi's
own classifier calls that text **non**-retryable, because the body names `billing`
(`@earendil-works/pi-ai` `dist/utils/retry.js:18`; `isRetryableAssistantError` tests that list at
line 212 before the retry patterns), so the stream's silence cannot show where the refusal was
consumed. This package's own log does: it reads
`dispatching to hybrid-probe-member/refuses-neuralwatt-429`, then
`returned HTTP 429 before output`, then `dispatching to hybrid-probe-member/answers`, all inside the
one `dispatchRequest`.

The session shows one request to each member, one assistant message from `hybrid-probe-member/answers`
with `stopReason: "stop"`, no `auto_retry_start`, no `compaction_start`, and one `agent_settled`. The
refusal never reached Pi: the pool consumed it and handed the same transcript to the next member.

Capture: `cli-item3-hybrid-member-refusal/`.

The same shift is visible with the real pool in item 1's second run, but as admission rather than
refusal: the two NeuralWatt lanes were absent from the catalog and the pool moved to the third member
before any request went out.

**Verdict: passed on a stand-in member carrying a captured refusal.** Not shown with a real member's
refusal: provoking one needs either a burst past NeuralWatt's concurrent budget or an exhausted
subscription window, and this run kept real requests to the four sessions item 1 and the cost section
name.

## Item 4: the limits Pi reports and enforces before and after the first reply

Measured through Pi's own `getContextUsage()` on session start, on every turn end and on settle, plus
the request options the pool and the member received.

Before the first reply, Pi reports the virtual model's declared pair for the selection,
`contextWindow: 200000` and `maxTokens: 8192` (`get_state` reports the same pair), and enforces the
declared window: `_limitsModel` returns the virtual model until a reply names the member that answered
(`@earendil-works/pi-coding-agent` `dist/core/agent-session.js:365-367`).

After the first reply, the pair Pi applies changes to the member that answered. In the real run that
is `contextWindow: 1048560` (NeuralWatt's reported window) with 646 tokens counted. In the probe run
it is `contextWindow: 1000000`, the stand-in's declared window.

The request's own `maxTokens` does not come from the declared pair: Pi sizes it from the routed model
and clamps it to that model's limit (`@earendil-works/pi-coding-agent` `dist/core/model-runtime.js:516`).
The real writer pool model carries `maxTokens: 384000`; the probe pool model carries `16384`, which is
exactly what the pool and the stand-in received on the wire, while the router had declared 8192.

Capture: `cli-item1-hybrid-real-writer/stderr.log` (context-usage lines, `contextWindow: 200000` then
`1048560`), `smoke-stand-in-json/stderr.log` (a one-turn probe with the whole pair sequence and the
`maxTokens` the member received), `probe-settings/records.jsonl` (an RPC `get_state`, which reports the
declared pair for the selection).

**Verdict: passed.** A rebuild should declare limits that make sense as a floor, not as a promise: Pi
uses the answering member's window from the next request on, and the declared `maxTokens` never reaches
a member.

## Item 5: a refusal Pi calls non-retryable

Two arms on the probe pool, one native-only arm, and one negative result.

Quota exhaustion, the Synthetic subscription refusal (captured), which Pi's classifier calls
non-retryable because the text carries `billing` (`@earendil-works/pi-ai` `dist/utils/retry.js:18`):
under the hybrid the pool moves to the next member
inside the same request and the turn completes, with no `auto_retry_start` and no compaction. The
command is item 3's with `id: "refuses-quota"` as the first member.
Capture: `cli-item5-hybrid-quota-refusal/`.

Native-only control, the same refusal returning from the member the virtual model routes straight to,
with no pool in between: the turn ends on
`429 "You've exceeded your subscription rate limits. …"`, with no retry and no second member. This is
the behaviour the hybrid is meant to keep away from. Capture: `cli-item5-native-only-quota-refusal/`, started with
`PI_HYBRID_NATIVE_TARGET=hybrid-probe-member/refuses-quota` and
`--model rate-limit-fallback/hybrid-native-probe`.

Cloudflare 523, whose own page heads it "Error 523: origin is unreachable" and whose text matches none
of Pi's retry patterns (`@earendil-works/pi-ai` `dist/utils/retry.js:29-112`): under the hybrid the
pool does not move on, because the current code hands off only a
429 before output or a provider-gated policy stop. The turn ends after one member request, with no
`auto_retry_start`. That is the gap the owner's point 1 decision ("Move on" for 503 and Cloudflare 52x)
closes after the rebuild, not something this run built. Capture: `cli-item5-hybrid-523-refusal/`.

A policy stop could not be shown: the package's move needs the member to be the `openai` provider, and
no pool member is, which the owner's point 5 decision keeps.

**Verdict: passed for quota exhaustion, failed for Cloudflare 52x under the current code (as expected:
the decided change is not built), policy stop not shown.**

## Fixture provenance

- NeuralWatt's overage 429 and Synthetic's subscription 429 are captures: `doc/captures.md` records both,
  with the run folder, date and length, and the tests already replay them.
- Cloudflare 523's text is Cloudflare's own page heading, read 2026-11-19 from
  `https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-523/`.
- Cloudflare 522's text is that page's sibling heading, read the same day from
  `https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-522/`:
  "Error 522: connection timed out".
- The overflow body is neither a capture nor a member's own documentation: Pi's overflow detector lists
  the OpenAI-compatible shape with example values in
  `@earendil-works/pi-ai` `dist/utils/overflow.js`. The counts in the probe's body come from the request
  the stand-in refused, and the wrapping is the installed Completions adapter's `<status>: <body>`
  (`dist/utils/error-body.js:139-145`). A rebuild that wants a captured overflow needs one real
  overflow, at the cost named in item 2.
- The probe's stand-in members answer a real OpenAI-shaped stream and refuse without any network, so
  this prototype adds no stand-in HTTP server and no invented wire bytes beyond the texts above.

## Cost of the real requests

Four sessions reached a real member, one prompt each, all against the real writer pool: item 1's two
cited runs (`neuralwatt/deepseek-v4.1-flash`, `input: 131, output: 3, cacheRead: 512`, 0.00002913 US
dollars by the catalog's rates; and `opencode-go/deepseek-v4.1-flash`, `input: 646, output: 3`,
0.0000987 US dollars), plus the two runs an earlier driver started while the commands were being
settled (`input: 629, output: 3` each, 0.00009615 US dollars). Both providers are subscription plans,
so the real cost is quota, not the dollar figures. Every other session in this page reached no
network: its members were stand-ins, so those arms cost nothing.

## What a rebuild would need

- A router per pool, or one router whose state picks the pool, with declared `contextWindow` and
  `maxTokens` that are a floor across members. Item 4 shows Pi replaces them after the first answer.
- The in-request fallback kept where it is: item 3 shows Pi never learns of a refusal the pool consumes
  before output, and item 2's control shows what the direct selection loses.
- Point 1's "Move on" for 503 and Cloudflare 52x, or a 523 ends the turn as it does today (item 5).
- A decision about the member catalog: `--no-extensions` sessions have no member providers, so a
  hybrid install must load the member extensions, and the shipped settings package must not also
  register the pool provider.
- One captured overflow, if the rebuild's tests are to stand on a capture rather than on Pi's detector
  source.

## Checks run

All five tasks were run after the last edit to the package's code, from this worktree, through
`mise run //package/pi-plugin/rate-limit-fallback:<task>`.

- `build`: passes.
- `lint:types`: passes, with no diagnostics.
- `lint:oxlint`: passes, with no errors and no warnings. The four `ModelRegistry.find` false positives
  the first run reported are gone: both evidence call sites now bind the lookup before calling it, the
  shape `src/index.ts` already used, so the array-callback rules no longer read the two strings as a
  callback reference. The `eslint(max-lines)` error is gone too: `src/dispatch-request.ts` moved the
  caller-options projection into `src/member-attempt-options.ts`, which is what rule `MXL` asks for.
- `lint`: passes, the aggregate over the two lint tasks.
- `test:unit`: passes, including the new `dispatch request pool tables` suite, which routes through a
  caller-supplied table, refuses an id the table lacks, and refuses `constructor`, `toString` and
  `__proto__` with `PoolDispatchError` instead of reaching the member loop.

## Where this brief's fact list is wrong

- "under a native virtual model alone a policy stop, quota exhaustion and Cloudflare 521, 522, 523, 525
  and 526 would end the turn": measured, a 522 whose body is Cloudflare's own "Error 522: connection
  timed out" does not end the turn, because Pi's retry classifier matches `timed? out`
  (`@earendil-works/pi-ai` `dist/utils/retry.js:76`, case-insensitive; `isRetryableAssistantError` at
  line 208 tries the non-retryable limit list first). One prompt on the branch's
  `refuses-cloudflare-522` stand-in member made nine requests to that member and then failed: eight
  `auto_retry_start` events, `auto_retry_end` with `success: false` and `attempt: 8`, then
  `agent_settled`. Capture: `item5-hybrid-522-refusal-branch/`; its `package-decision-log.jsonl`
  records nine `dispatching to hybrid-probe-member/refuses-cloudflare-522` lines. The earlier
  `item5-hybrid-522-refusal/` capture came from a local member definition that never landed on the
  branch, and its body left out the page's own "Error 522:" prefix; this run uses the branch's member
  and the heading verbatim. 523 matches no retry pattern and ends the turn after one attempt.
- "Pi retries only errors its classifier calls retryable" is right, but the classifier reads text, not
  status, so a 52x's outcome depends on the words its body carries.
