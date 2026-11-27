# Pi run starts of 2026-11-26 that ended without a run

Seven Pi run starts by `package/pi-shared/subagent-driver` ended without a run on 2026-11-26 between 20:28 and 21:31 -05:00,
on a host reported at load average 100 to 235 with a disk stall `some avg10` up to 92 percent (issue 6556).
Each was started again at 2026-11-27T01:20:30 to 01:20:31 and ran.

This record is unapproved and not final: the owner declares completion (rule `SGO`).
It holds what each of the seven starts left, which wait ended it and at what bound, and how each fact was established.

## Where the evidence is

Each start wrote into the run folder of its round under `${HOME}/temp/agent/<round>-2026-11-26/run-1`,
with the retried run beside it in `run-2` and the launcher's own output in `starter.log`.
That scratch expires a day after its last change, so the four journal records below are quoted here whole.

The journal quotes came from `journalctl --user --unit=<unit> --since "2026-11-26 20:00" --until "2026-11-26 22:00" --lines 200`,
read on 2026-11-27, one unit at a time and bounded as the rules file requires.
No run folder held a copy of what the driver printed, which is the defect the last section answers.

## The four starts that ended in 32 seconds with no summary and no standard error

The units were `restate-the-scratch-evidence-cited-by-handover-audit-research-and-package-docs`,
`restate-the-scratch-evidence-cited-by-troubleshooting-records`,
`split-mpm-shadow-cli-files-over-the-line-budget-part-3-of-3` and
`subagent-driver-worktree-launcher-skips-pnpm-install-after-the-add`.

Each run folder held one 61-byte feed line, `{"t":1,"kind":"environment","removedNames":["ICEAUTHORITY"]}`, and nothing else:
no `summary.json`, no `stderr.log`, no session.
The launcher's line for each was `{"unit":"<unit>","runFolder":"<folder>","status":1}` with no `ended` and no `turns`.

Each unit's journal says the same thing, here for the first of them:

```text
Nov 26 20:28:12 ... Started [systemd-run] ... drive-pi-subagent.ts --run-folder=.../run-1 --working-folder=... --provider=rate-limit-fallback --model=writer --wall-seconds=28200 --turn-cap=500.
Nov 26 20:28:28 ... node[2340681]: logger file sink: No ancestor node_modules directory was found; using fallback "/home/user/.local/state/monochromatic/log"
Nov 26 20:28:44 ... node[2340681]: SecretEnvironmentUnreadableError: the .env.local.json of /home/user/Monochromatic/.claude/worktrees/<worktree> holds values of this machine's secret environment and cannot be read, so no value of it could be masked before it reached the model, and no run was started: sops could not decrypt env store /home/user/Monochromatic/.env.local.json (timeout)
```

The wait was therefore the sops decrypt of the checkout's store, run by the store reader in
`@monochromatic-dev/env-local-json-core`, whose bound is `SOPS_TIMEOUT_MS`, 15 000 ms
(`package/env-local-json/core/src/constants.ts`).
The driver's own process spent the first 16 seconds of each unit's 32 seconds before that read began
(the first logger line came at 20:28:28, sixteen seconds after the unit started), so 32.285 to 32.322 seconds of unit
runtime is a loaded host's node start plus the 15 seconds the sops child was allowed.

Nothing of this reached the run folder, and no summary was written, because `supervisePiRun` throws before it starts Pi
and nothing in that path wrote a record: the thrown error goes to the unit's journal and nowhere else.

## The one that ended on a TimeoutError

`split-cargo-shadow-cli-files-over-the-line-budget-part-4-of-4`, run 1, ended `pi-exited-before-prompt` after
`totalMilliseconds` 169 656, with `piStartAttempts` 1 and `piStartStoppedBy` `start-sequence-spent`.

Its feed named the exit and the reason in Pi's own words:
`{"t":168385,"kind":"exit","code":1}` then
`{"t":169653,"kind":"start_exited","attempt":1,"code":1,"reason":"DOMException [TimeoutError]: The operation was aborted due to timeout"}`.
Pi's standard error shows the abort was one of its own timers inside an extension
(`pi-plugin-neuralwatt`'s store read, attempted twice, the second `aborted`), not a bound of the driver's:
the driver's start-up bound is 120 000 ms and Pi left at 168 385 ms.
The wait that refused a second start was `START_SEQUENCE_MILLISECONDS`, one start-up answer, 120 000 ms,
already spent when Pi left.

So this start did name what Pi said, but not the driver's bound that stopped the starts, not how long the start had taken,
and not what to do next.

## The two that ended prompt-rejected after four minutes and a half

`issue-6398-daily-update-regenerates-manifests-and-locks-with-a-catalog-floor` and
`issue-6491-forbidden-strings-embedding-control-runs-on-a-utf8only-home`, run 1 of each,
ended `prompt-rejected` after 272 513 and 272 451 ms with `turns` 0 and `piStartAttempts` 1.

Their feeds put the two start-up answers at their bounds:

```text
{"t":3,"kind":"environment"}
{"t":403,"kind":"secret_values",...}
{"t":120456,"kind":"state"}                       # the get_state answer had 120 000 ms, and came back at ~120 456 ms
{"t":243579,"kind":"commands","names":[]}         # the get_commands answer had 120 000 ms more, and came back at ~243 579 ms
{"t":244717,"kind":"masking_not_armed","handedOver":39,"answered":false,"commands":[]}
{"t":267707,"kind":"problem","problem":"Pi was still running 5000 ms after SIGKILL."}
```

The wait that ended each of them was `START_UP_ANSWER_MILLISECONDS`, 120 000 ms, spent twice: once on `get_state` and once on
`get_commands` (`src/supervise-pi-run.ts` asks them one after the other in `noteStartingState`, each with that bound).
Because the command list never came, `maskingIsArmedIn` had nothing to read, and the run ended as `prompt-rejected`
with `sessionStats` `no answer in 5000 ms` (the teardown) and `feedProblem`
`Error [ERR_STREAM_WRITE_AFTER_END]: write after end`.

What those two summaries did not say: which ask each 120 seconds was spent on, that the bound is 120 000 ms, that the
get_state wait and the get_commands wait were the two that ended it, and that no prompt was ever sent.

## What each start would have had to write

- For the four: a line naming the sops read of the checkout's store, its `SOPS_TIMEOUT_MS` of 15 000 ms, the 15 seconds it had
  waited, and what to do next. A summary could not be written where the run stopped, so the line has to live in the run folder
  under a name of its own, and the launcher has to print it.
- For the TimeoutError start: the bound that refused another start (`START_SEQUENCE_MILLISECONDS`), how long the start took,
  and that Pi left on its own error, not on a driver bound.
- For the two `prompt-rejected` starts: the ask each bound was spent on (`get_state`, then `get_commands`), the bound itself,
  how long each had waited, and that no prompt was sent because the masking could not be confirmed.

## What the driver writes now

Every start that ends without a run leaves `start-ended-without-a-run.txt` in its run folder, prints the same line where the
ending is printed, and, when a summary can be written, carries the same facts as `startThatEndedWithoutARun`.
The launcher prints the line from the folder, which is what makes a start that wrote no summary say how it ended.
The package README's section "A start that ended without a run" states the shape, and issue 6556's first half,
the bound sized from a measurement under load, is still open: this change measures nothing and changes no bound.
