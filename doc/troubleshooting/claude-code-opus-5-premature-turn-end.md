# Opus 5 ends turns on announced-but-undone work

Status on 2026-08-06:
Claude Code turns end on a status report that names the turn's own next action without performing it,
leaving the user to type a bare `Continue.` to restart execution.

The failure shape is verified from local transcripts and corroborated by two independent upstream reports.
Prose rules in `AGENTS.md` did not correct it.

The rate is much higher on `claude-opus-5` than on `claude-fable-5` or `claude-opus-4-8` in this corpus,
but that comparison is confounded,
and this document does not claim the model is the established cause.
See the confounds section for what the comparison cannot support.

## Symptom

A turn ends with a well-formed report whose final sentence announces the next action.
Observed final sentences,
 verbatim,
 each immediately followed by the user typing `Continue.`:

- `Next: fixture and test, then the sweep with pre-registered reading criteria.`
- `Next: implement #118, with the directory-candidate guard I traced beforehand.`
- `Next I'll start the editor ensemble and the refinement lane.`
- `Next in the queue is #68 (transitive capture through a sibling call), now unblocked.`
- `Queue: #43 (module-binding store), #44 (unbound projection, iteration binding...)`
- `**Next, unprompted:** #117 stays open ... I'm writing that regression test now:`

The last of those states in the present tense that the work is underway,
then emits `end_turn` without starting it.
Of the Opus 5 cases,
 all but one announce future work in the closing region of the message.

## Measurement

Source:
93 session transcripts under `~/.claude/projects`,
covering 2026-07-05 to 2026-08-06,
474 human-typed turns.
A turn counts as a restart nudge when a human-typed message under 200 characters
asks the agent to resume (`continue`,
 `go on`,
 `keep going`,
 `are you working`,
 `you stopped`,
 and similar).
Model attribution comes from the `message.model` field of the assistant turn preceding the nudge.

Two counting bases appear in this document and they do not agree,
so every figure names which one it uses.
Model-attributed counting keeps only the human turns whose preceding assistant turn ran the named model.
Whole-session counting keeps every human turn in the transcript.
Sessions switch models,
 so the same session yields different totals:
the longest Opus 5 session is 43 turns with 15 nudges model-attributed,
and 50 turns with 16 nudges whole-session.
Figures in this section are model-attributed.
Comparing a figure from one basis against a figure from the other is not valid,
and doing so is the defect retracted in the goal feature section.

The session in which this investigation ran is excluded from every figure.
Its human turns are responses to the investigation rather than instances of the failure,
and it grew while being measured:
figures first recorded as 26 of 76 turns and 5 of 8 sessions
drifted to 27 of 82 and 5 of 9 within the same working session.
Excluding it makes the numbers reproducible.
Any rerun must exclude the session doing the rerunning.

Restart-nudge rate with reasoning effort held constant at `xhigh`,
Wilson 95% intervals:

- `claude-opus-5`:
   27 of 80 turns,
   33.8%,
   CI [24.3,
   44.6]
- `claude-fable-5`:
   4 of 119 turns,
   3.4%,
   CI [1.3,
   8.3]
- `claude-opus-4-8`:
   0 of 12 turns,
   0.0%,
   CI [0.0,
   24.3]

The intervals for Opus 5 and Fable 5 do not overlap.
This is a descriptive difference between two populations,
not an isolated model effect.

Session-level spread,
 which rules out a single unrepresentative session.
A session counts for a model when any human turn in it was attributed to that model,
so a session running several models counts for each and the totals overlap:

- `claude-opus-5`:
   nudges appear in 5 of 8 sessions,
   63%
- `claude-fable-5`:
   nudges appear in 4 of 15 sessions,
   27%
- `claude-opus-4-8`:
   nudges appear in 2 of 15 sessions,
   13%

Session length alone does not explain the rate.
The two longest Fable 5 sessions,
 51 and 50 human turns,
ran at 8% and 0%.
The longest Opus 4.8 session,
 62 turns,
 ran at 6%.
Both long Opus 5 sessions,
 43 and 14 turns,
 ran at 35% and 29% independently.
Session length and task shape are distinct,
and the confounds section shows the long sessions differ in shape,
so this comparison bounds session length as an explanation without isolating the model.

## Confounds on the model comparison

The rate difference is real as a description of what happened.
It does not establish that the model is the cause,
because three differences separate the two populations and all push the same way.

Task shape,
which contaminates the denominator directly.
The metric is nudges divided by human turns,
so a session where the user only types `continue` approaches 100% by construction,
while a session carrying substantive direction approaches 0%
even if the model stops early just as often.
The populations differ exactly this way.
Median length of the non-nudge human turns:

Every model-attributed session of at least 14 turns is listed,
so the comparison is not a selected subset:

- `claude-opus-5`,
   43-turn session:
   33 characters,
   with turns like `finish the migration` and `Let's split the model.`
- `claude-opus-5`,
   15-turn session:
   75 characters
- `claude-opus-4-8`,
   62-turn session:
   58 characters
- `claude-fable-5`,
   50-turn session:
   69 characters
- `claude-fable-5`,
   51-turn session:
   120 characters
- `claude-fable-5`,
   27-turn session:
   155 characters

Session sizes here are model-attributed.
The gap between the bases is large for the second Opus 5 session,
which is 15 turns model-attributed and 82 whole-session,
so these sessions were heavily multi-model
and the medians describe the model-attributed slice rather than the session as a whole.

The Opus 5 sessions were more queue-execution-shaped
and the Fable 5 sessions more design-conversational,
so the denominators are not comparable populations.

Time and project.
The Fable 5 turns span 07-16 to 07-23 and the Opus 5 turns span 07-25 to 08-06.
The windows are disjoint,
across different projects and different Claude Code versions.

Instructions.
`AGENTS.md` rule `PXQ` landed 2026-07-28,
after nearly all Fable 5 turns and before most Opus 5 turns,
so the two populations did not run under identical instructions.
This difference cuts against the observed direction rather than explaining it,
since the stricter instruction applied to the worse-performing population.

What the corpus does support:
the failure shape is real and reproducible,
it is concentrated in queue-shaped work,
and it persisted across every corrective this repository applied.
An unconfounded rate comparison between models
would need the same task shape,
 period,
 and instructions on both arms,
which this corpus does not contain.

## What was ruled out

Automation artifact:
the user configured an automatic `continue` send on 2026-07-29.
No such send appears as a human turn in the corpus.
Every nudge carries `origin.kind` of `human`,
with `promptSource` of `typed` for 22 of the Opus 5 nudges and `queued` for 5,
`queued` being a message the user typed while the agent was busy.
The counted nudges are genuine user typing.

Reasoning effort:
every Opus 5 turn in the corpus ran at `xhigh`,
and Fable 5 ran 119 of its turns at the same `xhigh` setting with a 3.4% rate.
Effort does not separate the two.

Permission mode:
every Opus 5 turn but one ran in `auto` mode,
so mode carries no variance to explain the gap.

Repository instructions:
`AGENTS.md` rule `PXQ`,
which states plainly that a turn must never end on a status report the user answers with `continue`,
was committed 2026-07-28 in `af1c69278`.
Counting only turns whose entire text is some spelling of `continue`,
a stricter set than the restart-nudge definition used elsewhere,
Opus 5 drew 16 of them and 13 came after that commit.
An earlier revision said 15 and 12,
which mixed a snapshot taken before the corpus was pinned
with a nudge set that also admitted `go ahead`.

Reactive automation:
on 2026-07-29 the user reported having configured an automatic `continue` send.
Nudges continued afterward on 07-30 and 08-06.

Hook friction:
upstream issue 84007 hypothesizes that governance gates bias the agent toward safe turn endings.
The same Stop hook,
 `ccsr`,
 was active across every model in this corpus,
including the Fable 5 sessions measured at 3.4%,
so gate friction does not account for the difference here.

## Upstream reports

Two independent reporters,
 neither of them this repository's user:

- `anthropics/claude-code` issue 84007,
   opened 2026-08-05,
  titled `Agent ends turns with 'continuing now' promises instead of continuing`.
  It describes the identical shape,
  including status turns that substitute for work turns
  and read-only turns that stop at the edit boundary.
- `anthropics/claude-code` issue 81133,
   opened 2026-07-25,
  titled `[Bug] Claude Opus 5 stops responding mid-task in auto mode`,
  against version 2.1.219.

A third report,
 issue 69415,
 covers `API Error: Connection closed mid-response`.
That is a distinct failure.
In this corpus it accounts for the `<synthetic>` model rows,
where the transcript records a truncated response rather than a completed turn.

## Stop hook capacity, corrected

The Stop hook is the repository's existing enforcement point,
`ccsr`,
 from `package/claude-code-plugin/stop-reminder`,
whose handler lives in
`package/claude-code-plugin/source/src/handler/stop-reminder/index.ts`.
It already blocks stops for hedging language,
 uncited categorical dismissals,
 and trailing questions.

Measured hook behavior across the corpus:
67 blocking events,
of which 61 turns carried exactly one block and 3 turns carried two.

The cap of one block per turn was self-imposed.
The handler returned `{}` unconditionally when `stop_hook_active` was set.

The three turns carrying two blocks are not evidence that the hook re-armed.
This repository runs two Stop hooks,
 `ccsr` and `cctt`,
 plus goal blocks,
so two blocks in one turn is more readily two hooks than one hook firing twice.

### Measured on a throwaway

Claude Code's own behavior was measured directly rather than inferred,
on two disposable sessions against Claude Code 2.1.220,
each a temporary directory with `--settings` naming a single Stop hook
that logged its input and blocked unconditionally up to a self-imposed cap of 15.

Both runs behaved identically:

- The CLI dispatched the Stop hook 9 times,
   then ended the turn
  despite the ninth response being another block.
- `stop_hook_active` was `false` on the first dispatch and `true` on all 8 after.
  It never cleared.
  Honoring it therefore caps forced continuation at exactly one,
  and ignoring it yields as many continuations as Claude Code allows,
  which is not a fixed number.

A third run,
 of the shipped handler rather than a minimal probe,
ended after 17 dispatches,
 all of them forced continuations.
So 9 is not a Claude Code constant.

A fourth run refines the claim.
Its hook blocked unconditionally up to a self-imposed cap of 30,
and its block reason told the agent to run one shell command before continuing.
It reached 31 dispatches with 31 `Bash` calls across 124 assistant turns,
and stopped only because the probe's own cap fired.

Claude Code does have a cap.
`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` is present in the 2.1.224 binary,
alongside the override message
`A hook blocked the turn from ending`.
An earlier revision of this document concluded no cap existed,
on the strength of the 31-dispatch run alone.

The cap bounds an idle loop,
 not a busy one.
The runs overridden at 9 produced no tool calls and repeated a one-word reply;
the run that reached 31 worked on every continuation.
So a hook that keeps a busy agent blocked is not bounded by the platform,
and must bound itself.
That is the case that costs real money,
which is why the depth guard stays even though the platform cap exists.

The `Stop` input carried exactly these keys:
`background_tasks`,
 `cwd`,
 `hook_event_name`,
 `last_assistant_message`,
`permission_mode`,
 `prompt_id`,
 `session_crons`,
 `session_id`,
`stop_hook_active`,
 `transcript_path`.

### Undocumented Stop fields

Two of those keys are absent from the published hook reference,
and one of them turned out to carry the signal this whole mechanism needed.

`background_tasks` is an array of objects shaped like this,
captured by dumping a live payload on 2.1.224
while a backgrounded shell command ran:

```json
[
  {
    "id": "b6ldjvy2v",
    "type": "shell",
    "status": "running",
    "description": "Sleep for 60 seconds in background",
    "command": "sleep 60"
  }
]
```

A `status` of `running` is the only reliable way a Stop hook can tell
that the session is waiting on something another turn cannot advance.
Without it the hook has no way to distinguish
an agent that stopped early from one that stopped because it is blocked,
which is exactly the confusion that produced the eleven-turn incident.

`session_crons` is an empty array when no scheduled jobs exist;
its populated shape is unmeasured.

Both are now declared in
`package/claude-code-plugin/hook-type/src/events-agent.ts`.
The published reference also lists `effort` among the common fields,
which was absent from every payload observed here.
The published hook reference lists `effort` among the common fields;
no `effort` key was present on any of the 18 logged dispatches.

## Rescue rate, measured

These figures come from a snapshot taken before forced continuation shipped,
and the investigating session contributed no blocks to it,
so they carry none of the drift the measurement section describes.
A rerun today would not reproduce them:
this repository's own hook now emits Stop feedback on every turn.

The corpus contains 94 Stop-hook feedback records on the main branch.
Only 67 carry a `hook_blocking_error` attachment;
the remaining 27 are goal-condition blocks,
 described in the goal feature section.

A block counts as a rescue when the forced continuation issues at least one tool call
before the next stop boundary,
where the boundary is the next human turn,
 the next block,
 or end of session.

Pooling every block hides the difference that matters,
so the figures are given by block type.
An earlier revision of this document reported only the pooled rates,
which overstated the case:
the goal blocks score perfectly on every metric and carry the pooled average.

Across all 94 blocks:
91% issued at least one tool call,
88% issued a state-changing call,
the median continuation ran 9 tool calls,
and 7% were followed by a human restart nudge anyway.

Split by what did the blocking:

- Goal-condition blocks,
   27:
  100% issued a tool call,
  100% issued a state-changing call,
  100% changed task state,
  and none was followed by a nudge.
- `ccsr` text-detector blocks,
   67:
  88% issued a tool call,
  84% issued a state-changing call,
  25% changed task state,
  and 10% were followed by a nudge anyway.

Restricted to `claude-opus-5`,
 49 blocks,
 the same split is sharper:

- Goal-condition blocks,
   27:
   100% on every metric,
   none nudged again.
- `ccsr` text-detector blocks,
   22:
  82% issued a tool call,
  82% issued a state-changing call,
  41% changed task state,
  and 27% were followed by a nudge anyway.

Neither figure is a clean prior for unconditional blocking,
which conditions on neither text nor state and had no instance in the corpus.
The text-detector rate is the more conservative of the two
and the closer analogue in that it also fires without consulting task state.

The one-shot guard is therefore not the main bottleneck,
though it is not irrelevant either.
A single block puts the agent back to work in the large majority of cases,
and on the closest analogue,
 Opus 5 text-detector blocks,
roughly one block in five produced no tool call
and roughly one in four was followed by a nudge regardless.
So bounded progress-rearmed re-blocking would recover something,
but it is second-order next to the larger gap:
no detector fired on this failure at all,
since the hedging and trailing-question detectors do not look for it.

An earlier revision said a single block "reliably" restores work
and concluded re-blocking was not the missing piece.
That sentence was written against the pooled rate
and was left standing when the pooled rate was disaggregated.

## The goal feature already implements state-based blocking

Claude Code's goal feature was active in one session,
where it produced 27 Stop blocks.
Its feedback text keys on exactly the failure shape described here,
for example
`The assistant explicitly states 'Remaining: #105, #109, ...'`.

Per block it performs better than the phrase detectors:
all 27 issued tool calls,
 state-changing calls,
 and task-state changes,
and none was immediately followed by a nudge.

It did not prevent the session from needing nudges.
The session where it ran took 16 restart nudges across its 50 human turns
despite 27 goal blocks.

An earlier revision of this document compared that against
a 26.9% average over the Opus 5 sessions without the feature.
That comparison is retracted.
It grouped sessions by dominant model while counting turns on a different basis per arm,
so the two rates were not measured the same way.
The corpus cannot support a goal-active against goal-absent rate comparison.

What survives is narrower:
the goal feature rescued every stop it caught,
and the one session that ran it still required 16 nudges.
The mechanism rescues each stop it catches
and does not stop the agent from stopping again later.

Any recomputation from here needs care,
because forced continuation now emits Stop feedback
that a text classifier cannot distinguish from a goal block.
Sessions after 2026-08-06 will show apparent goal blocks that are this repository's own hook.

The practical consequence is that building a state-based detector into `ccsr`
would reimplement a mechanism the user already runs,
with evidence that the mechanism does not remove the problem.

## What a hook can and cannot do here

A Stop hook is mechanical in when it fires.
Its payload is not.
Blocking a stop rejects one stop attempt and re-enters the model,
but the `reason` it delivers is still an instruction the model may or may not follow.
It does not execute the announced action.

This matters for the user's stated constraint that adjusting prompts and instructions does not fix the behavior.
Every hook-based option is a better-targeted instruction delivered at a better moment,
not an escape from instruction-following.
Of the candidates recorded here,
only routing work to a different model would act without relying on the model obeying an instruction,
and its effect on this failure is unestablished.

## What was adopted

The user rejected the measurement-first options recorded here:
no model comparison,
no broad adoption of the goal feature,
and not the `loop` skill.
The stated reason is that the goal feature and `loop`
are both model-mediated,
since the model decides whether to invoke them
or whether a stated condition is satisfied,
so they inherit the failure they are meant to correct.
The stated design is to auto-prompt on every stop.

Implemented as unconditional stop-blocking in the existing hook,
`package/claude-code-plugin/source/src/handler/stop-reminder/auto-continue.ts`.
Gating became per-detector:
the three response-quality detectors still run only on the first stop of a chain,
while forced continuation re-arms on every stop.
A trailing question keeps precedence over it.
`MONOCHROMATIC_STOP_AUTO_CONTINUE` originally worked as an opt-out kill switch.

Off by default since 2026-09-25 at the user's request.
The user reports that `claude-opus-5-5` does not end turns on announced-but-undone work,
so the forced-continuation reason had become a distraction.
`MONOCHROMATIC_STOP_AUTO_CONTINUE` is now an opt-in switch:
 `on`,
 `1`,
 `true`,
 or `yes` enables it,
 and any other value or none leaves it off.
The response-quality detectors in the same `ccsr` hook stay active.

### What this does not achieve

The stated goal was that auto-prompting on every stop would eventually finish the work.
Unconditional blocking does not reach that,
and the gap is structural rather than a tuning problem.

A Stop hook extends the current turn.
When the chain ends,
 at the depth limit or because the agent stopped producing work,
the session idles waiting for the user exactly as before.
So the mechanism converts one user turn into many agent turns;
it does not remove the need for the next user turn.
Raising the limit buys more agent turns per user turn
and never removes the last one,
which is why this gap is structural rather than a matter of tuning.

No hook can close that gap.
All 16 hook events fire in reaction to something already happening,
and every output field they carry
(`decision`,
 `reason`,
 `additionalContext`,
 `systemMessage`)
modifies the turn in progress.
None starts a turn.
The `prompt` field in `UserPromptSubmit` is input describing a prompt the user already submitted,
not an output that creates one.

Closing the gap would need a driver outside the hook system,
something that submits a fresh prompt once the chain ends.
The `loop` skill is the in-harness form of that.
It was rejected as model-mediated,
which is accurate about its invocation:
the model decides to start it.
Its subsequent firings are scheduled by the harness rather than chosen by the model,
so the rejection applies to starting the loop and not to sustaining it.

**Decision,
 2026-08-07:
 the gap is accepted and will not be closed.**
The measured problem was 33.8% of Opus 5 turns needing a nudge,
and that cost is now absorbed inside each turn.
What remains is one prompt per chain end,
a much smaller cost than the one this work removed.
The alternatives were weighed and declined:
an external re-prompt driver is the only fully non-model-mediated route
but carries real runaway risk on the user's quota for the largest build of the options,
and raising the depth limit does not close the gap at all,
it only lengthens each chain while weakening the bound
that the platform cap does not supply for a busy agent.
Revisit only if long unattended runs become common enough
that stalling at each chain end outweighs that risk.

### Releasing, not only bounding

The first version had a depth bound and no release condition,
which produced the opposite waste from a runaway.
Reported from a real session:
blocked on a long-running process with nothing else to do,
the hook forced eleven turns that each restated the same blocker,
until Claude Code's cap overrode it.

Three releases now apply,
 each read from state rather than from the response text
and none asking the agent whether it considers itself finished:

- A background task is running,
  so the session waits on something another turn cannot advance.
  `background_tasks` carries this on the `Stop` event
  and was missing from this repository's `StopInput` type until then.
- The previous forced continuation issued no tool call,
  so pushing bought prose rather than work.
  One push is the threshold because a second rarely recovers:
  of the blocks in this corpus whose push produced no tool call
  and were followed by another block,
  only 2 of 9 second pushes produced work,
   22%.
  The sample is small,
  so that argues against pushing twice rather than establishing a rate.
  The accepted cost is that an agent answering a block with another announcement
  is released,
   which is the original failure slipping through;
  the other two releases do not have that weakness
  and cover the cases this one was added for.
- Every tracked task is finished,
  replayed from `TaskCreate` results and `TaskUpdate` calls.
  An absent task list is deliberately not treated as finished,
  since most sessions never create one.

The block reason also routes a decision-blocked agent to `AskUserQuestion`.
That tool waits for the user,
so it is the one exit that gets the agent what stopping was reaching for,
and the hook has nothing to refuse.
This reframes the mechanism from forbidding a stop
to naming the right tool for what the agent wanted.

### Bounding

Termination is shared with Claude Code rather than delegated to it.
The first version shipped without a bound,
on the belief that the CLI enforced one,
and that belief was wrong in the direction that mattered.
`continuation-depth.ts` now counts this hook's own feedback records
since the last human turn and allows the stop at the limit,
default 25,
overridden with `MONOCHROMATIC_STOP_AUTO_CONTINUE_MAX`.

Two implementation notes worth keeping,
both found by exercising the built hook rather than by unit tests:

- Each block writes its reason to the transcript twice,
  once as the feedback record fed back to the model
  and once inside a `hook_blocking_error` attachment.
  Counting both halves the effective limit.
  Verified live at a limit of 4,
   which produced exactly 4 blocks.
- Making the handler asynchronous broke the kill-switch tests,
  which had mutated `process.env` and now raced across the await.
  The policy was split into a pure function taking its inputs as parameters,
  so no test touches process state.

The cost falls on every session,
 not only queue-shaped ones.
A turn that genuinely had nothing left to do
now receives forced continuations up to the depth limit,
including on short question-and-answer turns.
Before the depth guard existed the ceiling was whatever the agent's own idleness produced,
which is what the 17-block figure below measures;
that figure describes the unbounded version and is kept as the record of it.
Verified end to end on 2026-08-06:
the built hook blocked 17 stops in a disposable session
whose entire prompt was `Reply with the single word: ok`,
where the agent visibly cast around for work that did not exist.

### First observation on a real session

The session that built this feature then ran under it,
which is the only observation of unconditional blocking on real work.
Measured from its own transcript through block 14,
at which point counting was deliberately stopped for the reason given below:
14 forced-continuation blocks,
against 3 human turns,
producing 13 commits.

The yield was not uniform,
and the shape of the decline is the useful part.
The first 10 blocks each produced a committed fix.
The finds included two wrong published figures,
a retracted comparison built on mixed counting bases,
an unpinned corpus that was drifting while being measured,
and several conclusion sentences left standing after the numbers beneath them were corrected.
Blocks 11 and 12 still found real defects,
including a retracted claim surviving in source rather than prose.
Block 13 found nothing:
every check it ran confirmed existing work was already correct.
Yield therefore fell to zero once the session's unexamined surface was exhausted,
after roughly a dozen blocks on an unusually deep backlog of unverified assertions.

Read carefully,
 because this cuts both ways.
The mechanism worked far better here than the disposable probe suggested,
and the reason is that this session had genuine unexamined surface:
a long analysis whose figures had been asserted from memory rather than re-derived.
A session without that surface is the `Reply with the single word: ok` case instead,
where the same hook produced 17 blocks and no work at all.
One session is not a rate,
and the agent measuring the benefit is the same agent that produced the defects,
so treat this as an existence proof that the blocks can be productive,
not as an estimate of how often they are.

The practical reading for tuning:
the mechanism converts unexamined surface into commits until that surface is gone,
then costs a full model turn per stop to confirm there is nothing left.
Neither the benefit nor the cost is constant,
and which dominates is a property of the session rather than of the hook.

### Self-referential yield, a hazard worth naming

Block 14 found a real defect:
this section's own figures had drifted,
because the section was written at block 10 and the blocks kept coming.
That defect existed only because the mechanism was running.

This is a failure mode distinct from a false positive.
A false positive produces no work.
A self-referential block produces work that is genuine,
committed,
and caused entirely by the blocking,
so it is indistinguishable from real yield by any measure taken from commits.
An agent recording its own forced-continuation statistics
will therefore find fresh work at every block forever,
and each fix will look justified.

Counting was stopped at block 14 for this reason,
and the figures here are deliberately not maintained.
Anyone evaluating a stop-blocking mechanism on commit counts
should exclude commits that only update the evaluation.

## Remediation options considered

This section is a historical record of what was weighed before the decision,
kept because the reasoning stays useful if the adopted approach is revisited.
None of it is a live recommendation.
Every option here was either rejected by the user or superseded by what was adopted,
and each carries its disposition.

No option here had an established effect on this failure.
The model comparison is confounded,
and the one state-based mechanism actually deployed,
 the goal feature,
rescued every stop it caught while its session still needed 16 nudges.
The ranking below weighed expected value against implementation cost,
not proven results.
It was written before the goal rate comparison was retracted,
so where a bullet leans on that comparison the lean is noted rather than silently repaired.

- **Rejected.**
   Run the goal feature deliberately across several queue-shaped sessions,
  and compare against matched sessions without it.
  Ranked first at the time because it was the cheapest remaining action,
  required no code,
  and tested the state-based approach before anything was built.
  The corpus contains one goal-active session,
  which is too few to estimate an effect,
  and that session is also the most queue-shaped.
  The 32% figure this bullet originally cited belongs to the retracted comparison
  and is not evidence for or against the option.
  Getting three or four matched sessions settles whether state-based blocking helps at all.
- **Rejected.**
   Route long queue-shaped sessions to `claude-fable-5`,
  run as a deliberate comparison rather than adopted as a fix.
  Ranked below the goal experiment because it costs capability on the hard analysis work
  these sessions consist of,
  where the goal experiment costs nothing.
  Ranked above building a detector because it is still only a measurement,
  and the corpus lacks exactly this comparison at matched task shape and period.
- **Superseded** by unconditional blocking,
   which needs no task state at all.
  Build tracked-task-state blocking into `ccsr`,
  with a high-confidence phrase detector only as fallback when task state is unavailable.
  Ranked below both measurements because it is the largest build here
  and would reimplement what the goal feature already does.
  The ranking also cited evidence that the goal feature did not lower the nudge rate;
  that evidence is retracted,
  and what remains is only that its one session still needed 16 nudges.
  Its guidance still applies if unconditional blocking proves too blunt
  and a narrower trigger is wanted,
  since narrowing the trigger is exactly what this option describes.
  State-first ordering is what defends against the main hazard:
  a wording-keyed rule can be satisfied by deleting the announcement instead of doing the work,
  converting an informative stop into a silent one.
  Three mitigations belong in any implementation of this option:
  enforcement keyed on state so deleting the phrase does not help,
  a block reason that explicitly tells the model to keep the status line rather than rephrase it,
  and shadow logging of stops that leave actionable work open with no announcement present,
  which is the only way to detect suppression once it starts.
  Durable task state is cheaper to maintain through `PostToolUse` hooks on the task tools,
  written to a sidecar keyed by `session_id`,
  than to reconstruct by replaying the transcript on every stop.
  The handler is currently synchronous,
  but `HookHandler` in
  `package/claude-code-plugin/source/src/runtime/handler-runtime.ts`
  already admits a promise-returning handler.
  Trailing-question handling must keep precedence,
  since the existing detector treats a trailing question as itself an invalid stop.
  User authorization to pause,
  for a compaction boundary or a genuine blocker,
  needs an exact user-supplied marker consumed from `UserPromptSubmit`,
  never a marker the assistant can print to authorize itself.
- **Rejected.**
   Drive continuation from the harness rather than from the user,
  using the `loop` skill's self-paced mode.
  Ranked last because it treats the symptom:
  it removes the typing cost without changing how often the agent stops,
  and it spends tokens on turns the user would otherwise have judged unnecessary.
  Rejected on the same ground as the goal feature,
  that the model decides whether to invoke it,
  so it inherits the failure it is meant to correct.

Filing upstream is not an option here.
`.out-of-scope/claude-code-upstream-bugs.md` settles it:
this project does not file or track Claude Code bugs as GitHub issues,
and the trigger to revisit is empirical evidence that a report produced a timely fix,
not the severity of any individual defect.
This document is the local record that policy prescribes instead.

## Luna situation check, 2026-10-18

The 5.5 series reduced early stopping and did not eliminate it.
The user asked for the stop hook to be re-enabled with Luna (`openai/gpt-6-luna-decisions`) making the decision.
The forced-continuation reason had become a distraction when it fired on every stop,
so the model reads the final message and the hook acts only when the message calls for it.

### What the transcripts show for the 5.5 series

Source: the 828 turns in this repository's session transcripts that end in `end_turn`,
read by `${HOME}/temp/agent/extract-stops.ts` (scratch, not committed).

- Restart nudges on 5.5 are rare:
   `claude-opus-5-5` 4 of 164,
   `claude-sonnet-5-5` 4 of 274,
   against `claude-opus-5` 35 of 283.
   Most of the eight are the user returning from a CPU swap, so nudges cannot calibrate a threshold for 5.5.
- The failure that remains is ending a turn while background agents or commands are still running.
   Follow-ups on such stops include `It's been 10 minutes`,
   `1.25h passed`,
   and `Are you waiting for something?`.
- On an unbiased sample (every seventh 5.5 stop, 63 stops) Luna labelled 25 as waiting on background work,
   19 as waiting on the user,
   16 as work closed,
   and 3 as announcing the agent's own next step.
   Five of the 25 background waits were followed by an idle complaint;
   the other 20 were followed by new instructions.
- Luna answered 128 of 128 probe requests with HTTP 200 and a median of 413 ms.
   One probe labelled a stop followed by `Are you waiting for something?` as `work_closed` at 0.94,
   so misses will happen.

### Design

One question with six situations (five until 2026-10-19) replaces a shared question, because a three-way split could not separate waiting
on background work from announcing a next step.
The action for each label follows rules this repository already states:

- `announces_own_next_step`: block, with the existing "resume the next item now" reason.
- `leaves_open_work`: block, with a reason to do the listed items now (added 2026-10-19, see "Missed block").
- `waits_on_user`: block, with a reason routing the question through `AskUserQuestion`,
   including requests that contain no question mark.
- `waits_on_background_work`: block only when the Stop event's `background_tasks` has no running task,
   since then nothing can resume the turn.
   With a task running the stop is released and the verdict is logged.
- `pauses_for_handoff` and `work_closed`: release.

A label is acted on at a probability of 0.60 or higher.
That value is interim:
there is no labelled ground truth for 5.5,
so it is revisited from the verdict log.
The response's separate `confidence` field is recorded by the gateway but is not used,
matching `doc/troubleshooting/llmgateway-luna-admission.md`.

State still decides without a model call when the depth limit is reached,
when the last forced continuation issued no tool call,
when a trailing question already owns the stop,
and, since 2026-10-19, when a subagent is running or the session is in plan mode.
Every block reason starts with the sentence the depth guard counts,
so the depth limit keeps bounding the chain.

The earlier release "a background task is running, so another turn cannot help" was retracted as a blanket rule.
`background_tasks` also lists subagents, monitors, and workflows,
and the user's own rule is that waiting subagents rarely wake the main agent.
It is kept only for the narrower case above and for the fallback described next.

### Failure behavior

A missing key, a transport error, a status other than 200, a timeout (5 seconds, no retry),
or a response that breaks the contract all fall back to the state-based behavior the hook had before Luna,
including its background-task release,
and add a `systemMessage`.
Claude Code shows a Stop hook's `systemMessage` as a notice reading `Stop says: ...`,
verified with `claude --print` against a throwaway settings file.

### Credential and switches

The key is `STOP_REMINDER_LLMGATEWAY_API_KEY`,
stored sops-encrypted in `.env.local.json` and decrypted by mise.
The hook reads its own environment first and runs `mise env --json` once, in the session directory, only when the key is absent.
That fallback is a stopgap until a credential mechanism that does not depend on mise exists;
it costs 0.10 to 0.14 seconds when it runs.
Replacing it is tracked in #1773.

- `MONOCHROMATIC_STOP_AUTO_CONTINUE=on` enables forced continuation.
   It is set in `.claude/settings.local.json`.
- `MONOCHROMATIC_STOP_LUNA=off` (also `0`, `false`, `no`) skips the model and keeps the state-based behavior.
- `MONOCHROMATIC_STOP_VERDICT_LOG` overrides the verdict log path,
   `~/.local/state/monochromatic/stop-reminder/verdicts.jsonl` by default.

The log holds one JSON record per consulted stop:
label probabilities,
the action,
the type and status of each background task,
the continuation depth,
and the last 600 characters of the message.
The message tail stays on this machine.

### Hazard found while building it

`console.info` from `@monochromatic-dev/module-logger` writes to stdout,
which a hook reserves for its JSON output.
An `info` line in the handler made the built bundle print `[info] ...` ahead of the JSON,
and the bundle suite failed on it.
Handler code logs at `debug` (stderr) or `warn`.

### Verification

- Unit tests with a fake gateway cover each label, the cutoff boundary, each skip, each fallback cause, and the log.
- The bundle suite drives the built `ccsr` with Luna unreachable and checks the block and the notice.
- Four live requests through the built bundle, with the key supplied only by the `mise env` fallback,
   returned `announces_own_next_step`, `waits_on_background_work`, `waits_on_user`, and `work_closed`
   for four matching messages,
   and the hook blocked the first three.

### False block, 2026-10-19

A stop ending "I'll verify its report and rerun the gates, then land it before verifying the tall-window fix" was blocked
with the resume-now reason while a subagent was running.
The verdict log (`~/.local/state/monochromatic/stop-reminder/verdicts.jsonl`, record at 02:06:12 UTC) shows Luna labelled it
`announces_own_next_step` at 0.95 with `waits_on_background_work` at 0.02.
The next stop, 74 seconds later, restated the blocker and was released at 0.99, so the cost was one wasted continuation.
The cause is that Luna reads only the last reply and the criterion said "with nothing external blocking it", which the
message did not contradict on its face.

Of the three blocks in the log at that point, one was a genuine early stop (0.99, "Next I'm continuing the queue with
the template cleanup"), one was this false block, and one (0.64 `waits_on_user`, a message that only offered a veto)
was borderline.

Fix: the criteria now say an announced step "does not depend on any running agent, command, build, or report" and that
a background wait includes "steps it will take only after they report".
Measured on the 44 logged stops plus the 63-stop unbiased sample (108 messages), run to run identical across six repeats:

- This stop: 0.95 to 0.57 for `announces_own_next_step`, below the 0.60 cutoff, so it is released.
- The genuine early stop: 0.99 to 0.97, still blocked.
- Announce labels overall: 8 to 2; blocks at 0.60 or higher: 3 to 1.
- Rejected: prepending the running task types to the input.
   It also released this stop (0.90 for `waits_on_background_work`) but dropped the genuine early stop to 0.48,
   and it widened what is sent beyond the final reply.

The margin on this stop is 0.03 under the cutoff, so a differently worded variant of the same message could still block.
Precision of the blocking labels is still unmeasured; see #1772.

### Skipping the call while a subagent runs, 2026-10-19

The user proposed skipping Luna whenever a subagent is running.
The verdict log supported it:
37 of 46 consulted stops had a running subagent,
35 of those were already released,
and both blocks among them were wrong
(the false block above, and a 0.64 `waits_on_user` block on a message that only offered a veto).
The one correct block in the log, a 0.99 announced next step, had only a shell task running.
So the skip lost no correct block in that data and removed most of the requests.

The skip covers type `subagent` with status `running` only.
A running shell command, monitor, or workflow still goes to Luna.
The skip is logged as outcome `skipped`, so the log still shows which stops it covered.

The cost is a stop that announces independent work while a subagent runs
(the original failure shape, such as "I'll start n66 when a new defect appears") is no longer blocked.
No such stop appears in the log; the 63-stop unbiased sample was drawn before `background_tasks` was recorded
and cannot say.
Revisit with #1772 if the log shows one.

### Other skip candidates, 2026-10-19

The user asked for more cases where the call can be skipped.
All 459 stops of the 5.5 models in this repository's transcripts were replayed through the production request.
Luna acted (probability 0.60 or higher) on 298: 182 background waits, 44 waits on the user, 61 closed,
6 pauses, and 5 announced next steps.
The blocking labels (announced or waits on the user) fired on 49.
Reading all 49 against the user's next message, most were genuine requests for a decision,
and no state the hook can see separated the false ones from the true ones.

Applied:

- Plan mode (`permission_mode` of `plan`).
   The agent cannot perform what it announces there, so a block could only be wrong.
   The corpus has no plan-mode stop (458 of 459 were `auto`), so this rests on the mode's definition, not on a count.

Measured and rejected:

- Any running shell task.
   The only correct block in the verdict log had a shell command running.
- Short messages.
   Of 37 messages under 200 characters, 29 were background waits, and "Still waiting." with nothing running is the
   failure the check exists to catch.
- Turns with no tool call (25 stops, 2 blocks), turns that used `AskUserQuestion` (87 stops, 5 blocks, 6%
   against 11% overall), and a last tool of `AskUserQuestion` (3 stops, 1 block).
   The rates do not separate and the counts are too small to justify a skip.

Not decided for lack of data:

- Other task types that report back (`monitor`, `workflow`, `teammate`).
   The log has none of these types, and the user's own rule says monitors rarely wake the main agent.
- Sessions with a scheduled wakeup (`session_crons`).

The verdict log now also records `permissionMode` and the count of `sessionCrons` per consulted stop,
so those two can be judged from data later (#1772).

### Missed block, 2026-10-19

A stop that reported a finished minimap and then listed "Still open" items was released.
Among the items: a 12,000 px measurement pass that "needs its own issue", an intermittent test failure,
and unaudited remote branches, all things the agent could act on itself.
The verdict log (record at 16:48:30 UTC, session `55e550ef`) shows `work_closed` at 0.66 and `pauses_for_handoff` at 0.29.
Rerunning the exact message from the session transcript reproduced those numbers, so the miss is not a transcription
artifact.
The cause is that `announces_own_next_step` asks for a stated next action, and this message announced none;
it named work and stopped.
The repository's own rule on this is that the agent should do named work rather than list it.

Three fixes were measured on the 457 to 459 5.5 stops in the corpus (a stop is blocked when a blocking label reaches 0.60):

- Widening the announce criterion to cover listed open work: the missed stop reads 0.94, but blocks go from 49 to 64
   (26 added, 11 removed).
   Rejected: it removed the most existing blocks, mostly waits on the user, because one label then absorbed both.
- A separate label `leaves_open_work`, announce wording unchanged: the missed stop reads 0.94 to 0.99 depending on the
   other criteria wording, blocks go from 49 to 79 (34 added, 4 removed).
   Chosen.
- The new label plus a rewritten `work_closed` ("names nothing further the agent could do itself"): the missed stop
   reads 0.99 and the corpus gives 11 `leaves_open_work` blocks, against 10 without the rewrite.
   Rejected: the rewrite buys one more block and changes a criterion that was not at fault.

Of the 10 corpus stops the new label blocks, 7 were followed by a user message that pushed the agent to keep going or asked why it was idle
("And? Keep working?", "You've named work to do, so just do them", "Please eager commit", "Are your bg tasks and
agents stuck?"), and the other 3 do not say either way.
The 4 blocks it removed sit near the cutoff (0.60 to 0.70 before, 0.37 to 0.56 after).
The three handoff stops in the log ("you can shut down", open items recorded) stay `pauses_for_handoff`
at 0.82 to 0.93, so the new label does not take them.

Side effect, measured and not designed: adding a label moved probability mass inside the distribution, and 24 stops
that read `waits_on_user` at 0.43 to 0.58 now clear the cutoff.
Reading them, most are real waits on the user (a sudo authorization, batch approvals, form pastes), which the block
policy already covers, but the 0.60 cutoff was set under five labels and is now applied to six; see #1772.
The new label's precision against a human label is unmeasured.

### False block on a finished task, 2026-10-21

A stop that said "Type-check, lint, and the unit test are running in the background" was blocked with the reason
that no background task was running.
The verdict log (record at 00:51:37 UTC, session `0eae2765`) shows `waits_on_background_work` at 0.63 and an empty
`backgroundTasks` list.
The transcript explains the empty list.
The type-check task started at 00:51:31, its completion notification entered Claude Code's input queue at 00:51:36.452,
the final message was written at 00:51:36.725, the hook ran at 00:51:37.2, and the notification was delivered at
00:51:39.
The agent was right that it was waiting, and the Stop event no longer listed the task because it had just finished.
This answers part of the "Not yet known" question below: `background_tasks` lists running tasks only, so it is empty
in the moment between a task ending and its notification being delivered.

The transcript records the queue as `queue-operation` records (`enqueue`, `dequeue`, `remove`, `popAll`).
The hook now replays them and treats a queued `<task-notification>` as a wake that is already certain: the stop is
released without a Luna request and logged as skipped with the detail `a task notification is queued`.
Replaying has two measured pitfalls.
Of the 46 transcripts that hold queue records, 31 notifications were never removed, 26 of them with at least 100
lines after; counting them would release every later stop of such a session.
Resetting the replayed queue at each tool result fixed that, because Claude Code absorbs queued input into the next
tool result (6,609 `remove` records with reason `absorbed_mid_turn`).
Before the reset, a stale entry at the head of the queue also made a `dequeue` take the wrong entry.

A delivery record (a user record with origin `task-notification`) also removes the entry it names, because a
`dequeue` takes the head of the queue and a stale prompt ahead of the notification made it take the wrong one.

Measured on the 5,260 assistant records that hold only prose, in the transcripts under 200 MB (four larger files
skipped; the corpus grew while this was measured, so only the rates compare):
201 are flagged, and 188 of them (94%) had a notification delivered within 40 lines after.
That proxy undercounts, since a notification absorbed into the next tool result leaves no delivery record, and
assistant text is often followed by tool calls in the same turn, which is not a stop.
Two of the 13 without a nearby delivery were read: one was prose followed by tool calls, the other was a session-limit
message with three notifications still queued.
The other 11 were not examined.
Against the real false block the built bundle releases it, and the same transcript with the one `enqueue` line removed
reproduces the original block, so the change is what releases it.

Limits:

- The queue records are an internal transcript format of Claude Code 2.1.305.
   If it changes, the check finds nothing and the hook behaves as before, which is the old false block, not a new
   failure.
- Whether the `enqueue` record is already on disk when the hook starts is not verified.
   The timestamps say it was written 0.8 s before the hook ran; the bundle check replayed the file afterward.
- A flagged stop whose notification never arrives is released once; the next stop is judged normally.

### Uncertainty check missed admitted gaps, 2026-10-22

A report about two music-player workflows listed what the agent had not checked ("I didn't check whether the release
job runs the tests", "I didn't read its rules", "I didn't check whether the release can save that cache itself") and
the stop was released with no uncertainty reminder.
Running the saved message through the pre-change code reproduced it: `findUncertainty` returned `NO_MATCH`.

Two separate causes:

- The phrase list held hedge words (`probably`, `likely`, `could be`) and no phrase for an admitted gap.
- `stripQuotedStrings` treats a single quote as a delimiter, so on a line with two contractions it deleted the text
   between them.
   In the report, `workflow's ... didn't` removed 121 characters of the "Release cache" line, hedge included.
   The same bug hides an ordinary hedge: `It doesn't build, probably because the cache isn't warm.` lost `probably`.

Changes: an "admitted gaps" group of phrases in `uncertainty-phrases.ts`, and a fold of in-word apostrophes to
`U+2019` before quote stripping, with `findUncertainty` folding curly apostrophes back so one ASCII entry matches both.
A quote mark after a space or at the edge of the text still opens a quoted string.

Measured on 1,682 stops (assistant text followed by a user record with no tool call between) in the transcripts under
200 MB, four larger files skipped:
the old code flagged 116 (6.9%) and the new code flags 208 (12.4%).
Of the 92 newly flagged, 80 come from the added phrases and 12 from the stripper fix, which revealed `likely`,
`could be`, `may be`, `probably`, `should be`, `i suspect` and `not sure` that contractions had hidden.
No stop that the old code flagged is lost.
The built hook blocks the saved report with `uncertain language ("i didn't check")`, passes a plain
"I haven't deleted anything", and still passes inside a blocked chain.

A first list also held the bare adjectives `untested` and `unverified`.
They accounted for 25 of the 116 new flags, and by my reading of the snippets about half described a thing
("untested merge", "the unverified version job") instead of the agent's own gap, so they were removed.
That reading is not a labelled sample.
A few kept hits are also not gaps ("I did not run anything or change anything else"), and several are gaps the agent
cannot close (a check that needs the user's consent); the reminder lets the agent say so and continue.

Owner decision on the same day: Luna may also be used to detect uncertainty, since a phrase list cannot catch every
wording; the phrase list is the fix for this case.
Not built.
A Luna uncertainty label would need its own corpus calibration (see #1772) and a decision on whether it replaces or
adds to the phrase list.

### More admitted-gap wording, and a stop the phrases cannot reach, 2026-10-25

The owner pasted a Slint IDE report (session `55e550ef`, 24 Oct 23:11 local) that ended with "Still open" items:
"I didn't try to remove it, and I didn't measure the capture-to-file lag", "Unexplained: why a slice list set inside a
frame isn't shown", "Test gap: there is no regression test".
The owner's reply was "Unexplained? Test gap? Just do it."
The verdict log has the stop: Luna read it and gave `work_closed` 0.48, below the 0.60 cutoff, and the phrase list had
no entry for "measure" or "try", so nothing blocked.
The owner also said a second miss happened in the same session around 10:00 the next morning.

What I found for that second one, by reading the transcript and the verdict log (timestamps in the log are UTC, four
hours ahead of local time):

- The 10:29:37 message ("I suspect the pointer rested over no character ... I haven't shown that") was blocked on
  `"i suspect"` (transcript records 64655 to 64657), so that stop worked.
- The next two stops, 10:30:44 and 10:31:04, came inside that blocked chain.
  `stopRemindersDecision` turns the phrase detectors off while `stop_hook_active` is true, and Claude Code never
  clears it for the rest of the turn, so they ran no phrase check at all.
  Luna was still asked: `work_closed` for the first, `waits_on_user` at 0.59 for the second, 0.01 under the cutoff.
- The 10:31 message listed "Popup screenshots: I checked the 14 hover popups from the log only" and "Hover
  limitation ... I left it unfixed".
  No phrase would catch "from the log only", so extending the list cannot cover it.
- I did not confirm that the owner meant the 10:31 message. It is the only 10 o'clock stop whose check was off.

Changes: first-person gaps for `measure`, `try`, `look`, `reproduce`, `rerun` and `re-run` (each in the "didn't",
"did not", "haven't" and "have not" forms), `I haven't shown`, `test gap` and `not viewed`.
Measured on 1,200 stops under 200 MB, against the list as it stood: 169 already flagged; the candidates would flag 67
more if every candidate were added, 39 of those from `still open`, which I left out.
Of the kept phrases about 15 stops are newly flagged, and every snippet I read was the agent's own gap ("I haven't
measured the net effect on collisions", "I haven't looked into it", "I did not rerun scoped mutation").
The bare adjectives `unexplained` and `unmeasured` were left out for the reason `untested` was: the snippets
mostly describe a thing ("an unexplained abnormality"), not the agent's own gap.
The built hook blocks the 23:11 report on `"i didn't measure"`, and passes "I tried the build and it passed".

The chain gate is not the lever for the 10:31 stop, and the paragraph that called it an owner decision is withdrawn.
Luna was asked inside the chain and scored `waits_on_user` at 0.59.
A block on that label routes the agent to `AskUserQuestion`, and the owner's reply to that stop was to do it
themselves.
Phrases cannot reach "from the log only" either.

### Purpose measured against the owner's nudges, 2026-10-25

The purpose is written down, so the question of what the hook is for was not open.
`package/claude-code-plugin/stop-reminder/README.md` says the hook "can optionally refuse a stop whenever pushing
could plausibly help", and the measurement section of this document counts a restart nudge: a human-typed message that
asks the agent to resume.
I read that as: the owner should not have to type the nudge ("Continue.", "You didn't check, so check.", "Just do
it.").
The owner asked for research before deciding Luna's input and block rule, so I measured against that signal.

Method.
Each of the 956 Luna verdicts in the log (9 sessions, 2026-10-18 to 2026-10-25) was matched to its stop in the
transcript, and the next record was classified.
All 956 matched.
The agent carried on by itself after 764 releases, a human replied after 119, a hook block followed 67 (42 Luna blocks
and 25 releases that a phrase blocked), and the transcript ended after 6.
The 88 verdicts before 2026-10-19 17:00 UTC have no `leaves_open_work` score and are a separate stratum.
Luna labelled the 119 human replies in two passes.
The first pass (four classes) called 61 of them nudges, which was too coarse: it included new instructions and
corrections.
The second pass (five classes) called 30 "the agent should have done it".
I read those 30 and kept 14 with plain nudge wording as the strict set; the rest were new instructions or
corrections.
The strict set was chosen by me after seeing the replies, and Luna's labels are a model's reading, not a human label.

Nudge rate by the reason a stop was released (strict set):

- Released because a task was running: 6 of 623 (1.0%).
- Released under the cutoff with top label `work_closed`: 3 of 66 (4.5%).
- Released under the cutoff with top label `waits_on_user`: 3 of 44 (6.8%).
- Released as `work_closed` at or above the cutoff: 1 of 44 (2.3%).
- Released under the cutoff with top label `waits_on_background_work`: 1 of 85 (1.2%).
- Under the cutoff with `announces_own_next_step`, `pauses_for_handoff` or `leaves_open_work`, and deliberate
  pauses: 0 of 27.

No release reason has a rate high enough to block on.
Blocking both under-cutoff strata above would add 110 blocks to catch 6 nudges, and 86 of those 110 stops resumed
without a nudge.

Experiment on the 14 strict-set stops against 30 controls (stops released because a task was running, after which the
agent carried on; every tenth of 567, in time order).
The same input gave the same output twice, so run-to-run spread is zero and the differences below are real for this
sample.

- Final message only (today's input), rerun on the full message text: mean `leaves_open_work` plus
  `announces_own_next_step` is 0.15 on the nudged stops and 0.07 on the controls; one nudged stop and no control
  reaches 0.4; AUC 0.64.
  The first run fed Luna the last 600 characters the verdict log keeps, which cut the message of 16 of the 84 rows used
  across these studies, and gave 0.14 and 0.08 with nothing at 0.4.
- With the assistant text of every earlier stop in the turn: 0.09 and 0.06; at 0.4 it catches 1 nudged stop and 2
  controls.
- With the standing instructions (`PX1`, `PXQ`, `QF1`, `ASK`) as a preamble: 0.20 and 0.15; at 0.4 it catches 2
  nudged stops and 3 controls.
- A direct question ("could the agent continue, or is it rightly stopped?"): AUC 0.47 on the final message and 0.26
  with turn history, where 0.5 is chance.
  History made the controls look more like open work, since their turns are long.

No variant separated the nudged stops from the controls, though the full-message run is a weak separation (AUC 0.64)
where the first run showed none.
The turn-history, standing-instruction and direct-question variants were not rerun on full messages, so their figures
describe a truncated input for the long messages.
The nudged stops read like the stops that resumed on their own, and the information that differs (whether the
running task will actually resume the agent) is not in the text.
Limits: 14 nudged stops is small; the controls cover one release reason only; the 764 count would be inflated if some
human messages lack `origin.kind: human`, and I found none in three samples per session but did not check every
record; the turn history was capped at 1,500 characters per stop and 8,000 in all.

The phrase list, as it stands after `410c4eeb39`, flags 4 of the 14 nudged stops on their full text
(`i haven't checked`, `i didn't measure`, `should be`, `i didn't check`) and 0 of the 30 controls.
On the 600-character tails the log keeps it flagged 3 of 14, so the fourth is a hit the tail cut off.
Across the whole log it flags 31 stops (full text for the 76 recovered, the tail for the rest): 15 already blocked, 10
followed by a human reply, 6 where the agent carried on.
It is the only instrument tested here with no hits on the controls.

Decision state when this section was written: Luna's input and the 0.60 cutoff were unchanged.
Nothing measured supported sending turn history, adding the standing instructions, or lowering the cutoff.
Turn history for repeated stops was adopted later the same day; see the sections below.

### A choice left with the user in prose, 2026-10-25

The owner reported that a stop in this session should have been blocked and the agent nudged to ask.
The stop was a report whose last paragraph said "Veto this if you want turn history tried anyway", after the owner had
said the Luna input and block-rule decisions were theirs and not yet decided.
The verdict log for that stop: Luna `leaves_open_work` 0.30, `waits_on_user` 0.16, released under the cutoff; the
phrase check then blocked it on `i haven't checked`, so the reminder said to search for evidence, not to ask.
The trailing-question detector needs a literal `?` and found none.
The 10:31 stop in the Slint session ("Say so if you want it") is the same shape and was inside a blocked chain.
Luna with the assistant text of every earlier stop in the turn, on the full text of both messages and through the
shipped code, did not catch either: the one-line reply stayed at `work_closed` 0.94 (0.97 with history), and the long report
moved from `work_closed` 0.29 to 0.51 and from a block mass of 0.65 to 0.41.
So turn history is not the fix here; it lowered the report's score.
An earlier version of this paragraph said history left both scores unchanged; that was measured on the 600-character
tail of the report and is retracted.

Measured on 1,222 past stops, not already flagged: `your call` 14, bare `if you want it` 11, `if you'd rather` 9,
`tell me if` 9, `veto` 8, `say if you` 5, `if you want that` 3, `if you would rather` 3, `say so if` 2,
`if you want me to` 2, `up to you` 2.
Reading the snippets, `say so if`, `say if you`, `tell me if you` and `if you'd rather` are offers that wait for an
answer ("Say so if you want it posted", "Tell me if you want me to unmount it").
`your call`, `up to you` and a bare `if you want it` often explain who decides ("whether $33.90 for an emailed copy
beats a second trip is your call"), and `tell me if` alone also matched "the Monitor will tell me if CPU crosses a
limit".

The owner chose the narrow set (`say so if`, `say if you`, `tell me if you`, `if you'd rather`; I added the
`if you would rather` spelling), over adding `veto` or everything measured.
The owner also chose to run this check inside a blocked chain, which the other wording checks do not do.
To keep a block answerable, it fires once per phrase per turn: the transcript since the last human turn is scanned for
this hook's own feedback record carrying the same phrase, and the stop goes through when one exists.
A record that only quotes the reminder (a tool result, an assistant message) does not count.
The reason reuses the trailing-question wording and names `AskUserQuestion`; when a trailing question also matches,
only its reminder is issued.
Code: `parked-decision.ts`; the transcript is now read when `stop_hook_active` is true even with forced continuation
off, because the once-per-phrase scan needs it.

Built bundle check: the real 10:31 message inside a chain with a fresh turn blocks on `say so if`;
the same message passes when the transcript already holds the reminder for that phrase;
a different phrase still blocks;
"The monitor will tell me if CPU crosses the limit" and "Whether to ship is your call" pass.
Not measured: the false-positive rate of the narrow set against a human reading, beyond the snippets above.

Owner decisions in the same exchange: Luna's block rule stays at the top label at 0.60.
On Luna input, the owner asked for better samples of a repeated false positive and a repeated false negative before
deciding whether Luna should receive the earlier stops of the turn.

### Luna with the earlier stops of the turn, on repeated stops, 2026-10-25

Samples came from the verdict log: 142 turns, of which 41 had a hook block followed by later stops (227 stops).
I read the 83 of those stops that Luna blocked again, scored 0.2 or higher for open work, or that a human replied to.

Labelled repeats, with how each label was made:

- Repeated false blocks, 3: stops 17 and 26 of turn `969554a8#50` and stop 31 of `969554a8#49`.
  The log shows running shell tasks at each (7, 7 and 1), Luna labelled them `waits_on_user`, and the table blocked.
  A fourth I had read as a false block (`83e3a58d#167`, stop 15) had no running task, and I did not verify it, so it is
  not in the sample.
- Repeated misses, 7: released stops after a first block in the same turn that the owner's next message answered with a
  nudge, including the 23:11 and 10:31 stops of 2026-10-24 and 2026-10-25.
- Controls, 30: stops after a first block that were released and after which the agent carried on.

Correction: the verdict log keeps only the last 600 characters of a message, and the first run of this study fed Luna
that tail.
That cut the long reports, including the 23:11 and 10:31 stops, so the first figures (0.38 to 0.14 on the misses, and a
"drift across days" finding) described a truncated input.
The figures below are a rerun on the full message text, recovered from the transcripts, through the shipped
`earlierStopTexts` and `situationRequestBodyWithHistory`.

Luna was asked the usual six-label question with the final message alone, and again with the assistant text of every
earlier stop of the turn (1,500 characters per stop, 8,000 in all).
Block mass is the summed probability of `announces_own_next_step`, `leaves_open_work` and `waits_on_user`.

- Repeated false blocks: 0.72 alone, 0.07 with history; all three fall.
  Two move to `waits_on_background_work` at 1.00, the right label with tasks running; one moves to `work_closed`.
- Repeated misses: 0.41 alone, 0.25 with history; five of seven fall, two are unchanged, none rises.
  The 10:31 stop goes from 0.80 to 0.58; the 23:11 report is unchanged (0.42 alone, 0.46 with history).
- Controls (stops after a first block that were released, after which the agent carried on): 0.14 alone, 0.13 with
  history; ten fall, five rise, fifteen are unchanged.

So history removes the repeated false blocks and lowers the block score on five of seven repeated misses, without
moving the 23:11 report.
For the failure the owner reported, a miss late in a chain, it is not a fix and mostly moves the score the wrong way.
Neither group is large enough for a rate: three and seven stops.

A logged score reproduces when the full message is sent.
The 10:31 stop was logged at `waits_on_user` 0.59 and scores 0.59 now; the 23:11 stop was logged at `leaves_open_work`
0.40 and `work_closed` 0.48 and scores the same.
Across the 84 rows the largest difference between a logged label and a rerun averages 0.008 on messages the log kept
whole and 0.026 on those it cut, with a maximum of 0.26 on one short message.
An earlier version of this section said scores drift across days; that came from the truncated input and is retracted.
The verdict log keeps the final message tail and the probabilities but not the text Luna was sent, so any analysis of a
long message must recover the full text from the transcript.
Until 2026-10-25 it also lacked `stop_hook_active`, so a repeated stop could not be told apart from a first stop
without rebuilding the turn from the transcript.

Owner decisions: send the earlier stops to Luna on repeated stops only, and log `stop_hook_active` in every verdict
record.
Built as `turn-history.ts` (the earlier stops, read from the transcript since the last human turn, without the final
message) and `situationRequestBodyWithHistory` (the request).
A first stop sends the final message alone, so the measured lenient shift applies only to repeated stops.
There it removed the repeated false blocks and also lowered the block score on five of seven repeated misses, which the
phrase checks and the parked-decision check remain the catch for.
A replay of the labelled stops through the shipped code found that the exclusion of the final message compared whole
texts, which fails when the hook is handed a message that differs at the edges from the transcript text; it now compares
the last 200 characters.
Checked on the built bundle with the real gateway: a repeated stop and a first stop both returned a verdict, and
the records carry `stopHookActive` true and false.
The request body itself is checked in the unit tests with a stubbed `fetch`, not on the wire.
Not yet known: whether the repeated-stop history changes outcomes in live use; the new field makes that measurable.

### Not yet known

- Agreement between Luna's label and a human label.
   The verdict log exists so it can be measured; calibration is tracked in #1772.
- Whether `background_tasks` is populated at the moments the idle waits happened.
   Hook input is not stored in transcripts.
- The `pauses_for_handoff` label did not appear in the unbiased sample, so it is untested on real stops.
- Whether blocking a background wait with a running task would help.
   That is the case the verdict log is meant to decide.

## Open questions

Whether the behavior is specific to queue-shaped work,
where a task list makes the next action nameable,
is untested.
Session shape was examined only for the sessions of at least 14 turns,
so the claim that the Opus 5 sessions were queue-shaped
covers those and not all 8.

Whether the announcement sentence causes the stop
or merely accompanies it
is likewise untested,
and the distinction decides whether a phrase-keyed detector can work at all.

How many goal-active sessions are needed to settle the state-based question is open,
but one is certainly too few,
and the single available session is the corpus's most queue-shaped.

Why the goal feature rescued every stop it caught
while the session still needed 16 nudges is unexplained.
The candidate readings are that the goal condition allowed stops it judged satisfied,
that it evaluated only at some stops,
or that the agent stopped again promptly after each rescue.
Distinguishing them means reading that session's goal blocks against its nudges in sequence.

Detector precision is unmeasured.
The nudge rate does not estimate it.
Estimating it means running a candidate detector over every stop in the corpus
and labeling the matches by hand,
including matches on turns the user did not nudge.
