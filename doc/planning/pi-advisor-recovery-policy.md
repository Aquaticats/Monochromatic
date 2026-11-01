# Pi Advisor recovery policy

Status:
options A to D accepted for issue tracking;
 option F refined and confirmed during the #413 design interview;
 option E rejected.

## Goal

Increase review completion across variable providers without silently replacing an explicit model,
 imposing a global context cap,
 or giving each retry a fresh deadline.

## Constraints

- The first attempt keeps the model-aware compaction context.
- Explicit model selection can retry the same model but cannot switch models silently.
- Default selection can use configured fallback order and session health.
- Every provider and model attempt shares one `timeoutMs` deadline.
- Caller cancellation remains operation-terminal.
  A provider abort ends that attempt without discarding another usable review.
- Usage and diagnostics include every attempt.

The continued-session Qwen request tokenized to `172314` input tokens.
Adding the configured `16384` output reserve produced `188698`,
 below the documented `262144` context.
That one HTTP 400 does not establish a stable context ceiling.
The same session also had larger successful Luna and Spark requests than later failed requests.

## Option A: Failure-aware same-model recovery

Policy:

- Let Pi's provider adapter retry one transport,
   HTTP 408,
   HTTP 409,
   HTTP 429,
   or HTTP 5xx failure while honoring `Retry-After`.
- Retry one successful no-text response with lower reasoning effort rather than repeating the identical request.
- Retry one recognized context-limit response unchanged because backend routing can vary.
- Return repeated context rejection with request metrics instead of inventing a global cap.
- End at the shared operation deadline.

Pros:

- Preserves explicit model choice and full first-attempt context.
- Directly addresses transient transport failure,
   variable context rejection,
   and reasoning-only responses.
- Uses retry controls already exposed by Pi's `SimpleStreamOptions`.

Cons:

- Can consume another provider request and more of the operation deadline.
- Requires a strict classifier so authentication,
   quota,
   billing,
   and deterministic request errors fail immediately.
- Lower reasoning can change review depth.

## Option B: Health-aware serial fallback for default selection

Policy:

- Replace highest-cost default selection with configured preference order.
- Record each model's last success,
   latency,
   failure class,
   and consecutive failures for the current session.
- Put failed models on a class-specific cooldown.
- Continue to the next healthy model only for default calls and within the original deadline.

Pros:

- Removes manual model switching after a provider failure.
- Avoids immediately choosing a model that just failed.
- Keeps explicit model semantics predictable.

Cons:

- A fallback can differ in quality,
   latency,
   and cost.
- Session health starts empty after a new Pi process.
- Configured ordering needs a clear default and status display.

## Option C: Provider-aware context negotiation

Policy:

- Use exact tokenizers where available and calibrated conservative estimates elsewhere.
- Preserve complete evidence entries,
   the latest compaction summary,
   and latest task evidence.
- Reduce context only after repeated context rejection for that operation or from an explicit configured limit.
- Record the attempted token budget and omitted entries.

Pros:

- Improves budget correctness without a low global character cap.
- Keeps more evidence on providers that accept it.
- Makes any adaptive reduction local and observable.

Cons:

- Tokenizer adapters add model-specific maintenance.
- Provider framing can differ from the public tokenizer.
- Some context errors do not report an enforced limit.

## Option D: Attempt ledger and progress rendering

Policy:

- Record model,
   attempt,
   reasoning level,
   context size,
   start and end times,
   stop reason,
   diagnostic,
   and usage.
- Aggregate top-level tool usage across all attempts.
- Emit bounded `onUpdate` progress for model,
   attempt,
   elapsed time,
   retry reason,
   and fallback state.

Pros:

- Makes long reviews and retry decisions auditable.
- Uses Pi's existing tool update callback and top-level tool-result usage field.
- Supplies the evidence needed to tune policy from observed outcomes.

Cons:

- Does not by itself make a provider succeed.
- Attempt history needs a compact renderer.

## Rejected option E: Task-scoped evidence mode

Decision:
rejected on 2026-08-01 because task-boundary inference is too indeterministic.
No implementation issue should be opened.
Advisor retains Pi's model-aware compaction context instead of guessing which evidence belongs to the current task.

The rejected policy would have sent the latest compaction summary,
 current task evidence,
 relevant diff,
 and recent messages instead of the complete compacted session.
Its possible relevance and input savings do not outweigh the risk of omitting a requirement,
 earlier decision,
 or cross-task dependency.
A trustworthy evidence packet would require provenance that the current session model does not provide.

## Option F: Delayed default reviews with bounded collection

The #413 interview rejected immediate first-result delivery and cancellation.
The default reviewer has no quality guarantee and receives no quality-based privilege.
See `doc/handover/pi-advisor-413-grilling.md` for the confirmed decisions.

Policy:

- Default calls recover serially through untried eligible scoped candidates,
   even when overlap is disabled.
- Explicit model requests remain exact and never overlap or switch models.
- Overlap remains disabled by default.
  Enabling it requires an explicit launch delay.
- The delay starts with the first logical reviewer call,
   including authentication.
- Keep at most two reviewer calls active.
  Before any usable review exists,
   failed calls can be replaced within the original deadline.
- Prefer another provider,
   but allow a different model on the same provider if no cross-provider candidate remains.
- First usable success stops replacements and retries,
   not already-started calls.
  Those calls receive a configurable 30-second default collection grace,
   capped by the original deadline.
- Return all usable completed reviews together when remaining calls settle,
   or cancel stragglers at the cutoff and return what completed.
- Detect exhausted credits from failed requests just in time.
  Block further dispatches on every model of that provider for this operation only.
  Preserve usable results from calls already running on that provider.
- Keep one evidence snapshot,
   per-model budgets,
   attempt records,
   metadata-only progress,
   and aggregate available usage.

Pros:

- Another reviewer can supply a result while one call stalls.
- Completed independent reviews are retained rather than discarded by a first-response race.
- Credit exhaustion does not cause repeated dispatches on that provider within the same call.

Cons:

- Overlap can bill both requests even after cancellation.
- Serial recovery can attempt additional models after failure without overlap being enabled.
- The collected reviewer set depends on completion timing.
- Cancelled or failed usage may remain incomplete.

Configured preference order and session-health cooldowns remain separate option B work.
The #413 implementation retains existing cost ranking rather than claiming to complete that policy.

## Ranking

Ranking:
A > B > C > D > F.

A ranks over B because same-model recovery helps explicit and default calls,
 while B helps only default selection.
B ranks over C because observed provider variability caused manual switching even when requests fit documented windows.
C ranks over D because correct context negotiation can prevent failures,
 while D primarily exposes them.
D ranks over F because attempt evidence
and progress improve every review without the duplicate provider spend of hedging.

## Recommended implementation order

1. Build an internal review-operation module with an attempt ledger and one absolute deadline.
2. Add provider-native transient retry and lower-reasoning no-text retry.
3. Add configured default order and session-health fallback.
4. Add exact-tokenizer adapters and entry-aware context negotiation.
5. Wire aggregate usage and `onUpdate` rendering to the attempt ledger.
6. Keep overlap opt-in and use recorded attempt evidence before tuning launch or collection timing.

Accepted work is tracked by the attempt ledger and usage issue (`#408`),
 health-aware selection (`#409`),
 context negotiation (`#410`),
 progress rendering (`#411`),
 failure-aware same-model recovery (`#412`),
 and opt-in hedging (`#413`).
