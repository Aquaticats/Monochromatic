# Issue #413: Advisor hedging design interview

## Request and current state

The user requested resolution of GitHub issue #413 with `/grill-me` and `/grilling`,
 explicitly requiring critical examination of its acceptance criteria.
The user completed the interview and explicitly authorized implementation after Q7.
Implement the confirmed policy,
 not the original first-winner acceptance criterion.

Implementation is starting;
 no package changes or package verification runs had occurred when the interview ended.
A network-free installed-provider probe has verified diagnostic and available-usage behavior.
The first round corrected a quality premise.
The second round established bounded collection after the first usable result.
The user accepted a configurable 30-second collection grace capped by the original operation deadline.
The user also explicitly requested just-in-time,
 operation-local exclusion of providers reporting exhausted credits.

## Evidence

`gh issue view 413 --json number,title,body,comments,labels,state,url` returned an open issue
 with no comments.
Its proposed behavior is delayed speculative default-model execution,
 first valid result wins,
 and cancellation of the other call.
Explicit model requests remain exact and hedging starts disabled.

`gh issue list --state all --search 'repo:Aquaticat/Monochromatic is:issue 408 409 411 412'
 --limit 100 --json number,title,state,body` returned all listed prerequisites as open:

- #408:
   operation attempt ledger and aggregate usage.
- #409:
   configured default preference and session-health fallback.
- #411:
   progress rendering derived from the ledger.
- #412:
   failure-aware same-model recovery.

`doc/planning/pi-advisor-recovery-policy.md:3` records acceptance for issue tracking,
 not a final implementation contract for this interview.
Its option F proposes first-valid selection.
Its recommended implementation order puts operation accounting and serial recovery before hedging.

Current source confirms that this is not merely a missing hedge timer:

- `package/pi-plugin/advisor/src/advisor-selection.ts:34` describes highest expected-cost selection;
   the function delegates default ranking to the shared selector.
- `package/pi-plugin/advisor/src/advisor-completion.ts:29` limits the current completion sequence
   to two provider-boundary calls on the same model.
  The second follows a successful terminal response without text.
- `package/pi-plugin/advisor/src/advisor-completion.ts:246` rejects provider errors,
   aborts,
   and unsupported tool use.
  `responseHasText` at line 385 only checks for a text block unequal to the empty string.
  This is not a semantic review-quality assessment.
- `package/pi-plugin/advisor/src/advisor-deadline.ts:88` computes an absolute deadline
   from operation start and configured timeout.
- `package/pi-plugin/advisor/src/tool.ts:118` does not consume its progress callback.
  Line 158 returns details without top-level usage;
   line 372 retains only the returned response's usage in details.
- `package/pi-plugin/advisor/src/config-schemas.ts` has no hedge,
   model-preference,
   or session-health configuration fields.

These are source observations,
 not evidence that concurrent scheduling or cancellation has been verified.
The historical provider failures in
 `doc/troubleshooting/pi-advisor-long-session-provider-failure.md`
 do not establish which winner policy the user wants.

## Acceptance criteria that need scrutiny

- First completion turns model preference into a launch preference.
  The user states that the default reviewer has no guarantee of good quality.
  Do not infer a quality endorsement from default status or require an equivalence tier.
  Transport success and visible text are operational properties,
   not semantic quality guarantees.
- The issue body asks for a different provider,
   while its checklist only prefers one.
  Distinct provider identifiers do not by themselves establish independent infrastructure.
- One speculative branch does not necessarily mean one additional provider request:
   both branches can contain retries.
  Any dispatch bound must include recovery and distinguish plugin calls from adapter-internal retries.
- Canceling a race loser must not count as evidence that its model is unhealthy.
- Caller cancellation and deadline expiry terminate the operation;
   a branch-local provider abort is not the same event.
- Immediate winner delivery and complete loser accounting can conflict.
  Abort delivery,
   terminal settlement,
   available usage,
   and actual provider billing must be distinguished.
  Current provider behavior has not been investigated in this interview.
- Branches may have different model-derived context budgets.
  A first-result policy could also select a result based on less evidence.

## Decision tree

User correction to Q1:

> We do not guarantee the default reviewer is of a good quality.

The agent withdrew its recommendation for an explicit interchangeable-quality tier.
That recommendation treated default status as an endorsement unsupported by the current cost selector.
Do not replace the withdrawn tier with another implicit quality guarantee.

The agent next offered immediate first-result delivery or waiting for all started reviews until the deadline.
The user rejected that binary:

> None of them:
> kill straggler after a while (of already getting at least 1 usable result),
> if not killed,
> return all reviews altogether.

Adopt bounded collection rather than first-result winner selection:

- Hold the first usable result while pending reviewers receive a grace period.
- Return all usable completed reviews together when every started reviewer settles.
- If reviewers remain pending when the grace period expires,
   cancel them and return all usable reviews collected by that cutoff.
- Start the collection clock at the first usable result.
  With the issue's initial reviewer and one fallback,
   another usable result finishes collection immediately.
- Preserve the original operation deadline before any usable result exists.
  This is inherited behavior,
   not a new decision established by the second answer.

The first-result grace is distinct from the launch delay for speculative work.
Do not silently treat the user's straggler grace as a value for the original hedge-launch delay.
Shared-understanding confirmation was received after Q7.

Round 3 asked for collection grace duration and deadline precedence.
The user answered:

> I'll go with whatever you recommend this round.

Accepted policy:

- Default collection grace is 30 seconds,
   configurable.
  This is an initial policy choice,
   not a measured optimum.
- The original operation deadline remains absolute.
- Collection ends at the earlier of grace expiry and original deadline.
- At either cutoff,
   cancel pending calls and return all usable reviews already collected.
- Caller cancellation stops the operation immediately.

Retain delayed launch and one fallback as the issue's working scope,
 rather than expanding to simultaneous or wider review fan-out without need.
Round 4 asked about replacement after failure and provider diversity.
The user selected Q5 A and Q6 A.

Accepted routing policy:

- Before the first usable review,
   replace failed reviewers using untried eligible scoped candidates.
- Keep at most two reviewers running concurrently.
- Bound attempts by the candidate list,
   bounded recovery,
   and the original deadline.
- After the first usable review,
   start no replacements;
   collect only already-running reviewers within the accepted grace.
- Prefer a different registered provider for the alternate.
- When no different-provider candidate is available,
   permit a different model on the same provider.
- Never dispatch on a provider blocked for exhausted credits in this operation.

Q7 A was accepted:
 default calls receive serial failure fallback even when speculative overlap is disabled.
Opt-in controls concurrent speculative work only.
Explicit-model calls remain exact.

The user then ended the grilling session
and authorized implementation unless another consequential design question arose.
No further product decision was identified.

Proposed engineering details for the confirmation summary:

- Keep existing default cost ranking;
   do not silently implement all of #409's preference and session-health policy.
- Use one evidence snapshot with model-specific context budgets.
- Require an explicit launch delay to opt into overlapping reviews.
- Permit bounded existing no-text recovery only before any usable review arrives.
  An already-started logical call may finish authentication and dispatch within its collection grace.
- Accept non-whitespace text only from completed `stop` or `length` responses.
  Label length-limited output;
   do not promise semantic quality.
- Return separate model-labelled reviews without a synthesis model call.
- Record received usage and identify canceled-attempt usage as potentially incomplete.
- Do not extend the deadline while waiting for cancellation acknowledgement.

Recompute the frontier after the user's answers.

## Added requirement: exhausted provider credits

The user reported this Advisor failure during the interview:

```text
advisor Consulting advisor hyper/deepseek-v4-flash-0731 with question
advisor: provider call failed for hyper/deepseek-v4-flash-0731 on attempt 1:
402: {"message":"You're out of credits. Add more at https://hyper.charm.land","type":"billing_error","code":null}
```

The user explicitly requested:

> Also implement:
> JIT,
> If a provider is out of credits,
> block the provider for this call.

Accepted implementation requirement:

- Detect exhausted credits from the actual request outcome,
   not an eager credit-balance preflight.
- Block further dispatches to every model on the affected provider
   within the same Advisor operation,
   including recovery and fallback dispatches.
- Keep other providers eligible.
- Do not persist this provider exclusion into later Advisor calls.
- Preserve exact explicit-model semantics;
   an explicit request does not switch to a different provider or model.
- Keep completed usable reviews and failure diagnostics.
- Add regression coverage using the reported `402` and `billing_error` response shape.

This is user-reported evidence,
 not a claim that this interview reproduced the provider failure.
The installed adapter's terminal-error path was investigated using synthetic responses.
See the verified probe document referenced in Verified integration evidence.
Do not generalize every HTTP 402 or every billing failure to exhausted credits without classification evidence.
No top-up or other billing mutation is authorized or needed.

Track this as an independently verifiable implementation task alongside collection behavior.

## Verified integration evidence

`doc/troubleshooting/pi-advisor-provider-billing.md` records the installed Pi 0.85.1 adapter probe.
The embedded harness was extracted and executed from the repository root.

- The OpenAI-wrapped exhausted-credit response reproduced the reported diagnostic.
- The adapter's `onResponse` callback did not observe HTTP 402.
- An unrelated HTTP 402 remained distinguishable through its message and error type.
- A bare body without the OpenAI error wrapper did not reproduce the user's diagnostic.
- Synthetic successful and aborted responses retained usage already received.

No live provider calls,
 credit lookups,
 or billing mutations occurred.
The result does not establish completeness of usage after remote cancellation.
Both the handover and troubleshooting document passed scoped Markdown lint.

Read-only scratch clones:

- Pi `v0.85.1` at `d981de1229ef899957bbe968bc8dcda02a21f477`,
   `${HOME}/temp/agent/pi-advisor-413-2026-09-09`.
- Hyper provider at `ac3ed634636b9e8eddad3e02e358943a5a829737`,
   `${HOME}/temp/agent/hyper-advisor-413-2026-09-09`.
  Its `src/index.ts` matched the installed 0.3.2 package.
- Exploratory probe:
   `${HOME}/temp/agent/advisor-413-provider-probe.ts`.
  The troubleshooting document contains the durable runnable version.

Pi documentation read completely:
 `README.md`,
 `docs/extensions.md`,
 `docs/custom-provider.md`.
The `examples/extensions/summarize.ts` example was read.
Read relevant TUI and session persistence documentation before implementing rendering/accounting.

## Independent review

Advisor was asked to challenge the first-completion premise and prerequisite scope.
It highlighted quality substitution,
 retry multiplication,
 late accounting,
 and misleading health penalties for canceled losers.
Its issue-number mapping was inconsistent with the fetched issue bodies.
Use the verified mapping in the Evidence section,
 not that part of the Advisor response.

A second independent review checked the user's correction.
It confirmed that absence of a quality guarantee does not establish reviewer equivalence
 or settle first-completion behavior.
It also confirmed that the withdrawn quality-tier recommendation must not remain active.

An explicit `openai-codex/gpt-5.6-sol` review of the current implementation proposal
 timed out after `600000` ms on attempt 1.
It produced no review feedback.
Do not treat that timeout as approval,
 a billing failure,
 or evidence about the proposed changes,
 which remain unimplemented.

## Worktree boundaries

Initial unrelated changes:

- `doc/troubleshooting/module-test-unhandled-rejection.md`
- `mise.lock`
- `doc/handover/module-test-explainer.md`

Leave those changes untouched.

## Implementation progress

Implementation is authorized and underway.
Task #6 (accounting foundations) and task #5 (credit classification and exclusion gates) are complete.
Task #3 (collection integration) is complete.
Task #4 covers final verification evidence and issue closure.

Implemented source includes:

- `operation.ts` and `operation-attempt.ts`:
   serial recovery,
   delayed overlap,
   bounded collection,
   and final dispatch gates.
- `operation-ledger.ts`,
   `operation-types.ts`,
   and `operation-usage.ts`:
   detached records and aggregate available usage.
- `operation-clock.ts`:
   local timeout and caller-cancellation races independent of provider cooperation.
- `provider-credit.ts` and `operation-candidates.ts`:
   credit-specific diagnostics and whole-provider exclusion within one call.
- `run-advisor.ts` and `run-preparation.ts`:
   one evidence snapshot,
   per-model budgets,
   scope revalidation,
   and collected results.
- `operation-failure-accounting.ts`:
   durable failed-operation entries and top-level usage restoration through `tool_result`.
- `rendering-operation.ts`:
   final attempt summaries,
   currently being verified.

Configuration now uses `hedgingEnabled: false` by default,
 an explicitly supplied `hedgeDelayMs` when enabled,
 and `collectionGraceMs: 30000`.
Project disablement is independent of inherited timing values.

The Advisor package's full unit suite and all guarded real-Pi host scenarios passed
 after the final logical-call timing correction.
The shared model-selection package's full unit suite,
 zero-warning Oxlint,
 and TypeScript task also passed.
Existing shared-package tests were changed from source imports to built-artifact imports
 to satisfy the active `test-import/require-eventual-artifact` rule.

Independent Kimi code review identified the missed getter-backed final scope gate.
That was addressed by reusing `readLiveScope`,
 including getter precedence and raw/wrapped model normalization,
 rather than maintaining a duplicate parser.
A new built-Advisor regression changes getter-backed scope during authentication.

Review interpretation and corrections:

- Credit exclusion blocks new provider dispatches,
   not already-running calls that might still produce a usable review.
  Cancelling all same-provider work on one failure could discard such a result.
  A regression now preserves that in-flight review while forbidding further dispatches.
The initial rejection of the preparation-grace finding was withdrawn after re-reading the user's wording.
A logical reviewer call already in authentication is a started call,
 so it receives the same grace as a dispatched provider request.
Only new replacements and retries are barred after first success.
The regression now requires such preparation to finish and return its review inside the grace,
 while still forbidding dispatch after the collection cutoff.
Commit `839490c06` records these changed expectations before the scheduler correction.

Progress payloads carry only a progress discriminator plus rendered metadata.
The full ledger remains on final results and durable failure entries,
 not on partial tool updates where provider diagnostics could expose payload text.

Important commits include:

- `8af348685`:
   coordinator and attempt observer.
- `ce198df27`:
   scoped operation preparation and explicit overlap configuration.
- `d75c079d7`:
   tool progress,
   collected output,
   and failed usage integration.
- `9eaa516a0`:
   deterministic collection/recovery timelines.
- `05207ee4a`:
   shared getter-backed scope revalidation and configuration tests.

## Final verification evidence

`mise run //package/pi-plugin/advisor:verify:host` passed these real host scenarios:

- Serial default recovery after exhausted credits.
- Exact explicit-model failure with overlap enabled.
- Returning collected reviews together.
- Cancelling a straggler and retaining the other review.
- Persisting a manual slash-command review.
- Persisting a failed explicit slash-command operation.

Each host used a disposable home,
 no ambient credentials,
 only scripted faux providers,
 a fixed prompt,
 bounded output,
 and a parent watchdog.
The verifier checks persisted top-level tool usage,
 operation accounting,
 bounded metadata-only updates,
 and empty child stderr.
The scripted primary agent continued after successful and failed Advisor tool results.

Red/green controls exposed and corrected additional integration defects:

- Started authentication initially received no collection grace.
  The changed regression failed before `f1f3ba918`,
   then passed after the scheduler retained preparing calls through the cutoff.
- Authentication initially delayed the hedge clock.
  The regression expected fixture time `50` but observed `150` before `02c9373c3`.
  The hedge now starts from the first logical reviewer call,
   after shared evidence preparation but including authentication.
- Project overlap settings discarded an unrelated global `systemPrompt`.
  The regression observed `undefined` before `6150be885`.
  Global instructions now survive unrelated overrides,
   while an explicit empty project prompt still clears them.

Guard mutation verification used a separate private Git worktree with a passing baseline.
No main-worktree guard was removed.

- Removing the final credit dispatch gate made the direct exclusion assertion fail
   and allowed the fixture's second `hyper` request after its provider was blocked.
- Removing candidate-level provider filtering selected `hyper/first`
   instead of the expected `other/third`.
- Removing the final live-scope gate made the raw,
   wrapped,
   and empty getter-scope rejection regressions fail.
- Restoring the guards,
   rebuilding,
   and rerunning the affected suites passed.
- The main-worktree credit suite also passed while the separate mutation artifact existed,
   checking isolation at the artifact-consumer boundary.
- The mutation worktree had no remaining tracked diff before removal.
  Root-sentinel `find`,
   `git check-ignore`,
   and `git clean --dry-run` checks preceded cleanup.
  No root sentinels existed,
   and the dry runs selected nothing.
  The disposable worktree was removed.

The `cli-git` auto-push for `c01ea5ae8` reported a concurrent ref update.
`git ls-remote --heads origin main` confirmed that exact commit was already remote.
No force push or history rewrite was used.

## Delivery status

The final Advisor lint,
 TypeScript,
 registration,
 unit,
 and real-Pi host checks passed.
The affected shared package's lint,
 TypeScript,
 and unit checks passed.
All task documentation passed scoped Markdown lint.
GitHub #413's title and acceptance criteria now describe the confirmed bounded-collection policy
 and the call-local credit exclusion.

Implementation is complete.
Related preference/session-health and broader recovery work remains separately tracked;
 this resolution does not claim those entire issues are finished.

## Completion

Closing commit `db8b230fb` reached `origin/main`.
Its ancestry in the observed remote head was verified with `git merge-base --is-ancestor`.
`gh issue view 413 --json state,closedAt` reported `CLOSED`
 with closure timestamp `2026-09-10T01:59:35Z`.

No task remains for #413.
Do not reopen settled decisions or revert unrelated concurrent work.
