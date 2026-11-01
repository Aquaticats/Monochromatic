# Translation repair handover snapshots: the hf-prefix misroute that halved the roster, 2026-08-25 to 2026-08-26

The diagnosis of the defect tracked as 235,
kept for its evidence:
five Charm Hyper models sent to Synthetic and failing on every call,
why nobody noticed,
what it voided,
the three instruments that misled,
and where to pick the work up.

Part of [Translation repair handover snapshots: 2026-08-25 to 2026-08-27](translation-repair-handover-2026-08-25-to-27.md),
which lists every file of this record.

## FIXED: half the roster was sent to a provider that cannot serve it (`#235`, 2026-08-25 to 2026-08-26)

STATUS 2026-08-26:
fixed in `8b289c3ab`,
guards in `e0010019f`,
each guard shown to fail with its fix line removed,
verified live the same day (see "State of the tree" under the audit section,
now in the history);
`doc/troubleshooting/synthetic-hf-prefix-misroute.md` holds the located cause and the fix.
The body below is the diagnosis as it stood while the cause was being located,
kept for its evidence.

This is the most important open item in this file,
and it is unfixed.
It was found while running arm A of the `#213` overlap measurement,
and it invalidates that run.

### What happened

The run was the serial control arm:
`TRANSLATION_REPAIR_SLICE_OVERLAP=1`,
four slices,
the full ten-model roster.
Log at `~/temp/agent/overlap-arm-serial.log`,
887 lines;
run directory `~/temp/agent/overlap-arm-serial`.
It started at 01:54:46 UTC and ended at 02:15:58 UTC,
about twenty-one minutes,
and it exited cleanly:
no refusal line,
no fault line,
all four slices reported.

Every single round it logged heard five of ten voices.
Twenty-five rounds,
`5/10 heard` on all twenty-five,
no exceptions.
(Re-measured 2026-08-26 with `grep -oE 'round: [0-9]+/[0-9]+ heard'`.
The first count here was twenty-two;
it came from a narrower anchor that required a stage prefix,
and every round is `5/10` under either anchor,
so the substance stands.)

The five that were lost are exactly the five Charm Hyper endpoint labels,
the ones whose identifiers do not carry an `hf:` prefix,
and each of them failed on every call it made:

```text
model                                              http400  streams
  qwen3.8-max                                           25       25
  minimax-m3                                            25       25
  gemma-4-26b-a4b-it                                    25       25
  deepseek-v4-pro-0813                                  25       25
  deepseek-v4-flash-0731                                25       25
  hf:zai-org/GLM-5.2                                     0       26
  hf:Qwen/Qwen3.8-27B                                    0       25
  hf:moonshotai/Kimi-K3                                  0       25
  hf:openai/gpt-oss-120b                                 0       25
  hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4      0       25
```

A hundred percent failure rate on five models,
a zero percent failure rate on the other five.
The provider's own words,
which are the giveaway:

```text
SyntheticHttpError: provider API returned HTTP 400:
{"error":"Your model name should start with an hf: prefix; for example: ...
```

`SyntheticHttpError` in that output is the class renamed `ProviderHttpError` on 2026-10-19.

Synthetic was handed five models that only Charm Hyper serves,
and said so,
125 times.

### Why nobody noticed

Quorum is five of ten.
The five surviving models are exactly enough to meet it,
so every round stood,
every slice settled,
and the command exited zero.

The resilience work is what hid this.
`#199` and the multi-provider landing made the pipeline survive a provider going away,
and it did survive:
it produced a complete,
well-formed,
four-slice calibration.
It just produced it from half the roster.
CORRECTED 2026-08-26:
it did not say "nothing above `warn`".
The calibration's closing coverage line (`producer-silence.ts:222`,
printed by
`editor-calibrate.ts:359-367`) named all five dark seats twice,
as
`WROTE NOTHING AT ALL: ... covers 5 of 10 seats`.
What was missing:
per-seat call and failure counts,
the failure class
(the line itself says a budget refusal,
a refused sheet and a timeout all look alike from there),
any such line in the pass or the other 36 CLIs,
and anything louder than stdout prose under exit 0.
Audit finding `calibrate-1` in `doc/audit/translation-repair-package-audit.md`.

That is the defect to fix,
and it has two halves that should not be conflated:

1.  **The routing half.**
    A model that only one provider serves must never be offered to the other one.
    CAUSE LOCATED 2026-08-26,
    and it is NOT `src/budget-routing.ts`.
    That file and `src/roster-reach.ts` were read in full and are correct:
    a bare Charm Hyper label derives `{ onSynthetic: false, onHyper: true }` and routes to Hyper.
    The misroute is the single-provider fallback in `createRunClient`,
    `src/corpus-run/run-config.ts:611-617`:
    when `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY` is unset it returns the bare Synthetic client,
    and that client sends whatever model id it is handed to the wire with no catalog check
    (`src/synthetic-client.ts:265`).
    The log proves the process had no Hyper key:
    `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is not set` appears four times,
    once per slice,
    and no `METERS ` line exists at all,
    because the routing client was never built.
    How the key went missing:
    arm A was launched with `nohup env ... node dist/final/node/editor-calibrate.mjs 4`
    directly from the fork worktree `~/worktrees/tr-overlap`,
    which has no `.env.local.json`,
    so nothing injected secrets;
    the process inherited whatever that shell happened to hold.
    The earlier pointer at `budget-routing.ts` was written after reading sixty lines of it,
    and was wrong.

2.  **The silence half.**
    Losing five of ten voices for an entire run should be loud.
    Right now it is a `warn` per call,
    a `5/10 heard` per round,
    and one closing coverage
    sentence in the two calibration CLIs that names the seats but not the counts or the cause.
    A run that finishes with a model at a hundred percent failure should say so at the end,
    in its own summary,
    where a reader who is not grepping will see it.

### What this costs, and what is now void

The arm A measurement is void as a ten-model measurement.
Do not read wall clock,
concurrency,
or overlap conclusions off it.
Arm B,
the `TRANSLATION_REPAIR_SLICE_OVERLAP=4` arm,
was never launched;
`~/temp/agent/overlap-arm-four.log` does not exist.
Both arms need re-running after the routing fix,
back to back,
and that re-run also pays the recovery rate owed by `#230`,
since it will carry `91f0c8ba5`.

`#213` therefore stays open,
and it is now blocked on this defect rather than on quota.

This also blocks `#219`.
Production readiness cannot be signalled while one provider's absence
silently halves the roster and the run still reports success.

### Three instruments that lied, and how to not be fooled again

Recorded because each one cost real time in this session.

1.  **The run log stamps are UTC,
    and the machine is on EDT.**
    A line reading `02:11:48` was written at `22:11:48` local.
    Reading them as local time makes a live run look like it died twenty hours ago.
    Compare against `date -u`,
    never against `date`.

2.  **Node's `comm` on this machine is `node-MainThread`,
    not `node`.**
    Scanning `/proc/*/comm` for exactly `node` finds nothing and looks like proof of death.
    Match on a `node*` prefix,
    or match the `cmdline` instead.

3.  **`pgrep -f PATTERN` matches the shell that is running it.**
    A waiting loop whose own command line contains the pattern never exits.
    This has now cost three separate incidents,
    including one where a kill loop
    killed its own shell and returned exit 144.
    List PIDs first,
    confirm the kind from `/proc/<pid>/comm`,
    then kill by number
    from a command that does not contain the pattern.

### Exactly where to pick this up

- The cause is `createRunClient` in `package/module/translation-repair/src/corpus-run/run-config.ts` (moved on
  2026-10-18 to `package/translation-repair/run/src/corpus-run/run-config.ts`);
  the fix design (require the Hyper key,
  catalog guard in the Synthetic client,
  process-scoped seat tally reported by `reportingRefusals`,
  `RunConfigError` as a stated refusal)
  is recorded on `#235`,
  advisor-reviewed,
  and held until the `#236` audit closes.
- Launch rule from now on:
  a run starts through `mise run //package/module/translation-repair:<task>` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:<task>`)
  from a worktree that holds `.env.local.json`,
  so sops injects BOTH keys.
  A fork made with `git worktree add` has no secrets file.
  Never `nohup env ... node dist/...`.
- The fix needs a test proven per `GFP`:
  a non-`hf:` model with Synthetic live must never produce a Synthetic call,
  and the test must be shown to fail with the fix removed.
- The overlap prototype itself is built and lint-clean but NOT committed.
  It lives in the fork at `/var/home/user/worktrees/tr-overlap`,
  detached at `166b08e81`,
  and its two files are saved at `~/temp/agent/proto-slice-overlap.ts`
  and `~/temp/agent/proto-editor-calibrate.ts`,
  with a drafted,
  unlanded test at `~/temp/agent/proto-slice-overlap.unit.test.ts`.
  Because the fork is pinned to an older commit,
  recreate it from the new head
  and reapply those two saved files rather than patching the stale checkout.
