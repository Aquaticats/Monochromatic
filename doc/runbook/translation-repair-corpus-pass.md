# Running a translation-repair corpus pass

A pass reads the pinned corpus,
sends slices to two model providers,
writes one settled artifact per entry,
and publishes a mirrored tree of fixed English pages.
It costs real money and real subscription allowance,
and a long one runs for hours.

This document is the procedure.
`package/translation-repair/README.md` is the reference for what each knob means,
and `doc/handover/translation-repair-run-continuity.md` is the record of how the supervisors
and their guards came to be the way they are.
Neither is ordered,
and neither can be followed from top to bottom by someone who has not run one before.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Running a translation-repair corpus pass: checks after the run exits](translation-repair-corpus-pass-checks-after-the-run-exits.md):
  The checks to make once a pass has exited,
  in order,
  each with its command and what it must print.

## What this proves

That a person who is not the agent which built the pipeline can start a pass,
tell a healthy one from a sick one while it runs,
and afterwards decide whether its output is fit to ship,
using only this document and access they can obtain.

Concurrent targeted passes require separate run roots,
logs,
and publication roots.
Same immutable build may back more than one pass when no rebuild follows.
Use separate worktree whenever source differs,
build it before launching its own process,
and leave every worktree backing active process unchanged until terminal state.
Shared provider capacity makes concurrent elapsed times unsuitable as matched performance comparison.
Record pipeline digest,
corpus commit,
and fixture provenance for each pass.

Production `corpus-pass` has no pull-request input flag.
It reads corpus clone and commit from `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` and `TRANSLATION_REPAIR_CORPUS_COMMIT`
(family doc `package/translation-repair/doc/configuration.md`)
and prints `CORPUS PIN OVERRIDDEN` at launch when either is set.
Pull-request validation points them at exact pull-request commit in isolated corpus clone or minimal Git fixture.
Pull-request 386 procedure of 2026-08-29 predates both variables (`b0a79eb66`,
2026-09-01);
its uncommitted throwaway fork read a `TRANSLATION_REPAIR_CORPUS_DIR` of its own,
which production never reads.
Run `--plan --only <entry>` before spending quota.
Provenance must name throwaway source changes,
pull-request head,
fixture location,
pipeline commit,
pipeline digest,
run root,
and log without copying credentials or corpus wording.

## Why this is a runbook rather than a script

Two prerequisites cannot be bridged,
and they are the reason this is written for a person.

-   The decryption identity at `~/.config/mise/age.txt` is a private key.
    An agent cannot create it,
    recover it,
    or read it out of anywhere else.
    Without it every credential in `.env.local.json` stays encrypted
    and the run reaches its first model call and throws.

-   Budget is bought,
    not arranged.
    Charm Hyper is a prepaid balance that does not refill on a schedule:
    it read `0` continuously through the whole of 2026-08-24,
    before,
    during and after a pass,
    and reached `10000` on 2026-08-25 only because credits were purchased.
    Synthetic is a subscription allowance that does refill on its own schedule,
    which the account owner can sometimes reset,
    but not reliably and not on demand.

Everything else here is scripted,
and is given as the exact command to paste.

## Setup

Status:
TODO | DONE

1.  Confirm the decryption identity exists.

    ```sh
    ls -l ~/.config/mise/age.txt
    ```

    Expected:
    one line naming the file.
    If it is absent,
    stop.
    Nothing further in this runbook can work,
    and the key cannot be regenerated from anything in the repository.

2.  Confirm configured credential names in encrypted store,
    without printing their values.

    ```sh
    cd -- /var/home/user/worktrees/translation-repair
    grep --only-matching '"TRANSLATION_REPAIR_[A-Z_]*"' .env.local.json | sort --unique
    ```

    Fully configured store prints these lines (the Exa lookup key and the Amazon Bedrock key may appear beside them):

    ```text
    "TRANSLATION_REPAIR_CHARM_HYPER_API_KEY"
    "TRANSLATION_REPAIR_NEURALWATT_API_KEY"
    "TRANSLATION_REPAIR_OPENROUTER_API_KEY"
    "TRANSLATION_REPAIR_SYNTHETIC_API_KEY"
    ```

    Key NAMES are plaintext in a sops-encrypted JSON file and values are not,
    so this reads the names and cannot disclose a secret.
    Never read a running process's environment to check this:
    `/proc/<pid>/environ` prints key values in full.

    At least one name must exist.
    One configured provider is valid normal operation;
    an absent provider is marked dry and its seats are unavailable.
    Every name absent is launch refusal.
    The Charm Hyper name carries `CHARM` in the middle.
    A variable missing it configures a run without Hyper rather than silently failing.
    Routing spends in the order NeuralWatt,
    Synthetic,
    Amazon Bedrock,
    Charm Hyper,
    OpenRouter
    (`doc/decision/translation-repair-neuralwatt-provider.md`,
    `doc/decision/translation-repair-openrouter-fallback.md`);
    NeuralWatt's name is `TRANSLATION_REPAIR_NEURALWATT_API_KEY`,
    and it carries 550 Wh of free energy a day on two models that the pipeline meters itself;
    a worktree whose store predates the OpenRouter name can still exercise it by launching the worktree's
    bundle with the main repository as the mise config root,
    where the bundle is `package/translation-repair/corpus-pass/dist/final/node/corpus-pass.mjs`:

    ```sh
    cd /var/home/user/Monochromatic
    TRANSLATION_REPAIR_STARTED_BY=corpus-pass mise exec -- node <worktree>/<bundle> --only <id>
    ```

    Put `env --unset=TRANSLATION_REPAIR_CHARM_HYPER_API_KEY` between `--` and `node` to leave Hyper out of that run.
    The `TRANSLATION_REPAIR_STARTED_BY=corpus-pass` before `mise exec` is what the command's own task would set:
    since 2026-10-06 a built command refuses to run in a process that holds provider keys
    unless that variable names it
    (`package/translation-repair/adjudication/src/task-runner-guard.ts`),
    so a start that skips the task names the command by hand.
    Check which names a root injects before choosing,
    printing presence and never a value:
    `mise exec -- node -e "console.log(process.env.TRANSLATION_REPAIR_OPENROUTER_API_KEY === undefined ? 'absent' : 'present')"`.

3.  Prepare the scratch root that will hold the log,
    the pid file and the run directory.

    ```sh
    mkdir --parents "${HOME}/temp/agent"
    chmod 700 "${HOME}/temp/agent"
    ```

    Expected:
    no output.
    The mode matters:
    trust checks reject group and other permission bits.

4.  Check the setup for no quota at all.

    ```sh
    mise run //package/translation-repair/corpus-pass:corpus-pass -- --plan
    ```

    Expected:
    a line beginning `PLAN ok tip=`,
    carrying `pipeline=`,
    `client=constructed`,
    `pending=` and `first=`.
    It reads the corpus,
    builds the pending list,
    constructs the client,
    and returns without opening a stream.
    A live invocation took 1.88 seconds.

    `client=constructed` is the credential check.
    A missing Synthetic key fails here rather than an hour into a run.

5.  Read both meters before committing to a launch.

    ```sh
    mise run //package/translation-repair/run:budget-sample > ~/temp/agent/pre-launch.log 2>&1
    cat ~/temp/agent/pre-launch.log
    ```

    Expected:
    one line containing `METERS`,
    for example:

    ```text
    METERS synthetic=wet hyper=dry syntheticWeekly=97% syntheticFiveHour=48/50 syntheticThrottled=no hyperBalance=0
    ```

    Capture both streams as shown:
    the reading is at `info`
    and a meter that could not be reached warns at `warn`.

    A provider reading `dry` does not block ordinary launch.
    Validation or performance arm requiring particular providers must add
    `-- --require-providers synthetic,hyper` (any subset of `synthetic`,
    `hyper`,
    `openrouter`);
    that mode refuses before model calls unless every named key and meter is wet.
    The budget layer refuses each model no reachable provider can take,
    the stage records a lost voice,
    and the run continues on whoever answered.
    It does change what the run's output means:
    any quality figure measured while a provider was dry
    rests on whoever was awake,
    which can be five of the ten seats.

6.  Re-read the completion caps against the runs since the last launch.

    ```sh
    mise run //package/translation-repair/run:cap-census -- ~/temp/agent > ~/temp/agent/cap-census.log 2>&1
    rg '^cap-census:|^[a-z].*: card cap|^  [A-Z]{4}' ~/temp/agent/cap-census.log
    ```

    Expected:
    a `cap-census:` summary line,
    then per seat its card cap and the rule's reading,
    and any flags under it.
    It reads ids and numbers only,
    and spends no quota.
    `PLACEHOLDER WITH A DISTRIBUTION` means a card still names the pooled 99th percentile
    though the rule can now read its own calls (ledger P10):
    write the measured cap on the card.
    `RULE READS ... AGAINST THE CARD'S ...` and `CUTS OVER ONE PERCENT` are for reading,
    not for acting on at once:
    a capped call cannot show a longer answer,
    and the log scope differs from the 2026-09-09 table,
    so read each provider's cut columns (with content,
    with none,
    unpaired) in the full log before moving a cap,
    as `completion-cap.ts` records for the 2026-09-28 reading.

## Steps

Status:
TODO | DONE

1.  Decide where the run writes,
    and make it a throwaway directory unless this run's output is meant
    to join the pool that later draws treat as natural accumulation.

    ```sh
    RUNDIR="${HOME}/temp/agent/corpus-pass-$(date +%Y%m%d)"
    mkdir --parents "${RUNDIR}"
    ```

    Expected:
    no output.
    Leaving `TRANSLATION_REPAIR_RUNS_DIR` unset writes into
    `node_modules/.monochromatic/translation-repair-runs` under the worktree root,
    which is the pooled location.
    Any hand-picked run,
    meaning one carrying `--only`,
    wants a throwaway.

2.  Launch it detached,
    with its output captured.

    ```sh
    cd -- /var/home/user/worktrees/translation-repair
    TRANSLATION_REPAIR_RUNS_DIR="${RUNDIR}" nohup setsid \
      mise run //package/translation-repair/corpus-pass:corpus-pass \
      > "${RUNDIR}.log" 2>&1 < /dev/null &
    ```

    Expected:
    the shell prints a job number and a pid,
    and returns immediately.

    The repository's `CLAUDE.md` asks that long runs start as their own systemd user unit,
    out of the cgroup every session's shell commands share,
    which `systemd-oomd` may kill whole.
    The same launch as a unit:

    ```sh
    systemd-run --user --unit="corpus-pass-$(basename -- "${RUNDIR}")" --collect \
      --property=MemoryMax=12G \
      --working-directory=/var/home/user/worktrees/translation-repair \
      --setenv=TRANSLATION_REPAIR_RUNS_DIR="${RUNDIR}" \
      --property=StandardOutput=truncate:"${RUNDIR}.log" \
      "$(command -v mise)" run //package/translation-repair/corpus-pass:corpus-pass
    ```

    Give the unit no `StandardError` property:
    stderr then inherits stdout's file through the same descriptor.
    `StandardError=append:` on the same file opens a second descriptor,
    and stdout,
    writing at its own offset,
    overwrites every line stderr wrote,
    every `warn` among them;
    the TianqiChen66622 launch lost all of them that way
    (the package's audit ledger,
    M147).
    Right after the launch,
    read how systemd wired the unit's two streams:

    ```sh
    systemctl --user show --property=StandardOutput --property=StandardError "corpus-pass-$(basename -- "${RUNDIR}")"
    ```

    Expected:
    `StandardOutput=truncate` and `StandardError=inherit`.
    `StandardError=append` is the form that loses stderr;
    stop the unit and launch again.
    (Measured 2026-10-11 on two throwaway units,
    one launched each way.)
    Under a unit,
    stop the pass with `systemctl --user stop corpus-pass-<name>`,
    where the steps that follow say to kill it by pid.

    A `--setenv=PATH=` on the unit decides which build tools the task's build runs.
    A session's shell started in the main checkout carries that checkout's `node_modules/.bin` first,
    and its `rolldown` need not be the version the worktree's lockfile installs.
    On 2026-10-19 `CuspariaKLSY16` was launched with the shell's whole `PATH`:
    its build ran the main checkout's `rolldown v1.2.14`,
    not the worktree's `1.2.13`,
    and its bundle wrapped CommonJS modules differently,
    so the pipeline digest moved with no family source changed.
    Pass no `PATH`,
    or one with every main-checkout entry removed and the worktree's `node_modules/.bin` first.
    From `0fa3120174` on,
    a family build refuses such a bundler itself,
    and the task stops at its build with
    `This build is running rolldown <x>, but this checkout installs rolldown <y>.`;
    a checkout older than that commit still needs the check that follows.
    Before trusting a launch meant to run on a known build,
    read the bundler version off the build lines and the digest off `START`:

    ```sh
    grep --only-matching --max-count 1 'rolldown v[0-9.]*' "${RUNDIR}.log"
    grep '^START ' "${RUNDIR}.log"
    ```

    Expected:
    the version `node_modules/.bin/rolldown --version` prints in the worktree,
    and,
    for a rotation,
    the same `pipeline=` as the rotation's first entry.
    A different digest is a different build:
    stop the unit before it spends.

    Launch-time dials,
    each printed above the work so the log says which run this was:
    `TRANSLATION_REPAIR_SLICE_OVERLAP=<n>` (prints `OVERLAP <entry> value=<n> source=...` per entry),
    `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=<ms>` (prints `STRAGGLER GRACE OVERRIDDEN by ...`),
    `TRANSLATION_REPAIR_WRITER_GRACE_MS=<ms>` (prints `WRITER GRACE OVERRIDDEN by ...`;
    the editor,
    refiner,
    translate and consolidate rounds alone wait this long on a straggler after quorum,
    the other rounds keep the round window).
    Unset,
    the writer rounds wait their built-in 180000 ms and the launch prints `WRITER GRACE built in`,
    and the pass keeps four slices in flight (`OVERLAP <entry> value=4 source=fallback`);
    since 2026-09-06 the plain launch is the production launch,
    and a dial is for a measured arm.
    A value the dial cannot read refuses the launch in one line naming the variable,
    at zero quota,
    which `-- --plan` shows without spending anything.

    ALWAYS KILL AND RELAUNCH,
    the owner's rule of 2026-09-06:
    if source changes after this launch,
    kill the pass by its pid,
    build,
    and launch the same entry again into a fresh `TRANSLATION_REPAIR_RUNS_DIR` on the new build.
    A pass that finishes on a superseded build stamps a digest that no longer describes the files on disk,
    and its page is not readiness evidence.
    When a fix is already known before the launch,
    land it first and launch once.

    For a measured arm that needs named providers,
    insert `-- --require-providers synthetic,hyper` (or any subset of `synthetic`,
    `hyper`,
    `openrouter`)
    after `corpus-pass`.
    Log must contain `REQUIRED-PROVIDERS synthetic,hyper status=wet` before model traffic.
    This flag is wired only to `corpus-pass`;
    probes and calibrations do not support it and cannot serve as required-provider timing arms.

    THE PID THE SHELL PRINTS IS NOT THE RUN.
    It is a wrapper,
    and the work sits two levels below it.
    Measured on a live calibration,
    the tree was
    a `bash` wrapper holding 2792 KB of resident memory,
    a `mise run` child,
    and a `node dist/final/node/<script>.mjs` grandchild holding 126340 KB,
    which is the process doing the work.
    Do not record the printed pid as the run's identity.

3.  Confirm the run is alive by what it is,
    rather than by a recorded number.

    Paste this function once per shell.
    It reads each process's argument vector out of `/proc` directly.

    ```sh
    running() {
      for d in /proc/[0-9]*; do
        [ -r "$d/cmdline" ] || continue
        mapfile -d '' -t argv < "$d/cmdline" 2>/dev/null || continue
        [ "${#argv[@]}" -gt 0 ] || continue
        [ "$(basename -- "${argv[0]}")" = node ] || continue
        for a in "${argv[@]:1}"; do
          [ "$(basename -- "$a")" = "$1" ] \
            && printf 'alive pid=%s %s\n' "${d#/proc/}" "$(ps -o etime= -p "${d#/proc/}" | tr -d ' ')"
        done
      done
    }
    running corpus-pass.mjs
    ```

    Expected:
    one line beginning `alive pid=`.

    DO NOT REPLACE THIS WITH A `pgrep` PATTERN.
    Two separate flag interactions have each left this exact check silently blind,
    and both failed by returning false,
    which reads precisely like a clear field.
    A pattern is matched against the whole flattened command line,
    so it also matches the shell that merely carries the pattern text,
    and `--exact` then fails against any run carrying `--only`.
    `doc/handover/translation-repair-run-continuity.md` records the measurements.

    Matching the file NAME rather than its path is deliberate,
    and it is what survives the file move that broke this once already.
    It also means a run of the same script under any other path counts as running.
    That asymmetry is chosen:
    reporting a run that is not there makes you wait,
    while missing one that is there launches a second pass
    into the same runs directory and the same slice cache.

4.  Run nothing else through `mise` until the pass is finished.

    Every pass and probe task declares `depends = ["build"]`,
    so invoking any of them rewrites `dist/final/node` underneath the running pass.
    A pass computes its pipeline digest once at startup and stamps it into every artifact,
    so a rebuild that changes any output file leaves the run recording a digest
    that no longer describes what is on disk.
    That is the whole of the reason,
    and it is enough on its own.

    NOT BECAUSE THE PROCESS WOULD LOAD A MIX OF OLD AND NEW MODULES,
    which this runbook used to claim and which was measured false on 2026-08-25.
    Every one of the 176 chunks is reached by a static import resolved at startup,
    the only dynamic imports in the whole bundle are `node:fs/promises` and `node:path`
    inside the logger,
    and no child process the bundle spawns runs a file under `dist`.
    A running pass never reads that directory again.
    The claim is corrected rather than deleted because a wrong mechanism invites
    someone to disprove it and conclude the rule is safe to break.

    A rebuild with no source change is byte-identical and harmless,
    which is exactly why this is easy to get away with and worth stating anyway.

    THE RULE IS ABOUT WHAT THE RUN WRITES,
    not about whether the process would notice.
    A measurement CLI that settles no artifact stamps no digest,
    and `editor-calibrate` is one:
    its output is its log.
    Its bundle was rebuilt eight times under a live 40-slice calibration on 2026-08-25,
    twice with deliberately broken source for a guard proof,
    and the run kept producing rounds throughout.
    Three facts were checked on the running process first,
    rather than assumed:
    one process with zero children,
    no descriptor open under `translation-repair/dist`,
    and zero occurrences of `import(` in its entry bundle.

    That is a licence for a calibration and NOT for a corpus pass.
    A pass stamps its digest into every artifact it settles,
    so a rebuild leaves the run recording a digest that no longer describes what is on disk,
    and the pool partitions generations by exactly that field.

    To read something while a pass is in flight,
    invoke the built entry point directly
    and skip the task that would rebuild it,
    naming the command as its task would,
    since a built command refuses to run in a shell that holds provider keys unless that variable names it:

    ```sh
    TRANSLATION_REPAIR_STARTED_BY=meter-report node dist/final/node/meter-report.mjs "${RUNDIR}.log"
    ```

    To exercise a CHANGE while a pass is in flight,
    run the source rather than the bundle:

    ```sh
    node --experimental-strip-types --disable-warning=ExperimentalWarning src/hyper-client.ts
    ```

    Node runs the package's TypeScript directly,
    resolving its `.ts` extension imports,
    so a probe that injects a fake transport can exercise a provider path end to end
    with no key,
    no network and no build.
    The refusal report at every entry point,
    the rule for which classes may repeat their message
    and the verify wiring on cut streams were each verified this way while a pass held the bundle.
    Two limits:
    a suite that imports `../dist/final/node/index.mjs` still needs the build,
    and a probe run this way exercises the source rather than the artifact,
    so it verifies the change and not the packaging.

    `mise run //package/translation-repair/<package>:lint:types` and `:lint:oxlint`
    carry no build dependency and are safe at any time.

## What to check

Status:
TODO | DONE

Watch these while it runs.
Each is the exact string the log carries.

-   `[error]` at any point.
    A healthy long run carries none.
    Count them rather than reading for them:

    ```sh
    grep --count '\[error\]' "${RUNDIR}.log"
    ```

    Expected:
    `0`.

-   `METERS`,
    once per reading,
    roughly once a minute while the run is spending.
    `hyper=dry` or `synthetic=dry` is ordinary and not a failure.
    A state of `unreadable` means the meter endpoint could not be reached;
    it still routes as spendable,
    because a monitoring failure must not become an outage.

-   `ONLY`,
    only when the invocation was restricted.
    Seeing it on a run you meant to be unrestricted means the pass is doing less than you asked.

-   `SPEND CEILING OVERRIDDEN`,
    only when `TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD` was set.
    It names the USD this run may spend on any one of OpenRouter,
    DeepSeek and Xiaomi before it stops starting entries;
    the built-in is 20.

-   `SPEND CEILING reached`,
    when the run has spent that allowance on one of those providers (the line names which).
    The pass starts no further entry and the entries already running finish;
    `DONE processed=` then reads short of `pending=` for money rather than for time.
    Raise the ceiling for a deliberately larger run.

-   `CAP OVERRIDDEN`,
    only when `TRANSLATION_REPAIR_HARD_CAP_MINUTES` was set.
    It names the per-entry ceiling actually in force.

-   `CAP TOO TIGHT`,
    which is a warning and not a refusal.
    It means the ceiling is at or below one model exchange,
    currently 360000 milliseconds,
    so every attempt is cut before any exchange returns,
    nothing caches,
    and the queue drops each entry as stalled on its second try.

-   `REATTEMPT <id> queued`,
    which is healthy only after resumable operational error or hard cap.
    Entry that bought more cache records goes to back of queue.
    Quality rejection and `status=INCOMPLETE` must never produce this line.

-   `TALLY <id> status=INCOMPLETE`,
    which means stage-local work remains.
    It is neither success,
    quality verdict,
    nor publication evidence.
    Cache stays,
    but whole entry does not restart in same invocation.

-   `TALLY <id> status=PARKED returnsInMs=<n> returnsAt=<time>`,
    which means the pass paused because NeuralWatt's energy returns at that time of day,
    written as an ISO 8601 time with `Z` for UTC,
    and the entry goes back to the front of the queue to run again after the wait.
    It is neither failure,
    nor success,
    nor a verdict on the entry,
    so count it with neither `status=ERROR` nor `status=INCOMPLETE`.
    The `error=` text is the failure that named the return.
    The operator does nothing while the pass waits,
    and the log ends the wait one of two ways.
    When the pass resumes the entry,
    the gate logs `the pass resumes` and the entry's next `TALLY` line is the one that counts.
    When the pass budget (the soft time budget or a spend ceiling) ends during or after the wait,
    the log shows `PARKED`,
    `the pass resumes`,
    the budget stop line
    and `NOT RESUMED <id>`,
    with no later `TALLY` line for that entry:
    it was not attempted again in this invocation and stays resumable,
    so run the pass again.
    A `PARKED` line with neither ending means the run was stopped during the wait,
    and the entry is likewise left resumable.

-   `STALLED <id>`,
    which is the earned-re-attempt rule refusing.
    The entry finished an attempt with no more cache records than it started with,
    so it is dropped for this invocation.
    A few are ordinary.
    Every entry stalling is a defect.

When the run has exited,
check its output rather than its log.

Filed in:

- [Running a translation-repair corpus pass: checks after the run exits](translation-repair-corpus-pass-checks-after-the-run-exits.md)

### Three exit codes every command in this runbook can leave

Each command has its own verdicts in `1` through `3`,
listed with it.
Above those sit three that mean the same thing whichever command printed them,
because they come from the reporter every command is wrapped in.

-   `4`.
    A file it needed would not read,
    and it stopped there.
    The first line names the file and the failure,
    never the file's text.
    Re-run the pass to rewrite the file,
    or name a run directory that has it.
-   `5`.
    A fault in the command rather than in the run.
    The first line names the class,
    and stack frames follow.
    There is no message:
    a fault's message can carry text from whatever it
    choked on,
    and a run directory holds unlicensed corpus wording,
    so the reporter drops the message and keeps the frames.
    This one is a bug report.
-   `6`.
    The command declined in its own words,
    and the words are the report.
    A usage line,
    an unset key,
    a control that did not hold.
    Nothing broke and nothing was half-read.

So `6` means read the line,
`4` means fix the input,
`5` means file a bug.

## Restore

Status:
TODO | DONE

To stop a run in progress,
kill the worker by the pid that `running` reported,
not the pid the launching shell printed.

```sh
running corpus-pass.mjs
kill <pid>
```

Expected:
a later `running corpus-pass.mjs` prints nothing.

A stopped run is resumable and is not wasted.
Run root `prompt-payloads/` contains raw model payloads keyed by canonical prompt digest.
It may contain corpus/model wording:
never quote,
commit,
or share it.
Across interrupted invocations,
these payloads reconstruct correction state without provider resend.
Corrupted payload refuses before network rather than being ignored.
Write failure or corruption retains failed claim for rest of invocation;
entry cannot recover until operator repairs/removes disposable record and relaunches.
This is fail-closed handling,
not quality verdict.

Every stage caches,
so a relaunch into the same `TRANSLATION_REPAIR_RUNS_DIR`
republishes what was already bought rather than re-buying it,
provided no source file changed in between.

When a source file DID change,
the rule is always kill and relaunch:
stop the pass by pid as shown,
build,
and relaunch the entry into a fresh runs dir on the new build.
The old runs dir holds what the old digest bought and is not resumed;
a page finished on a superseded build is not readiness evidence,
whatever it cost.

If a resume supervisor is driving the pass,
stopping the run and stopping the supervisor are two different operations.

-   To stop the supervisor while a pass is running,
    kill it by pid.
    Killing the supervisor does not kill the pass,
    because the pass is a child that outlives it.
    That is what makes swapping supervisors mid-run safe.

-   To stop the run,
    kill the pass by pid.
    The supervisor then sees the field clear,
    and a stop file placed at that point prevents the next launch:

    ```sh
    touch ~/temp/agent/resume-supervisor.stop
    ```

    The stop file is checked on every poll of the supervisor's wait loop,
    so it takes effect within a minute while the supervisor is waiting.
    IT DOES NOTHING WHILE A PASS IS RUNNING.
    The supervisor spends that time awaiting its child and reaches no stop check
    until the pass exits,
    so a stop file placed during a run
    sits unread for as long as the run lasts,
    which can be twelve hours.

To discard a run entirely,
remove its directory and its log.
Do this only for a throwaway run.

```sh
rm --recursive --force -- "${RUNDIR}" "${RUNDIR}.log"
```

Expected:
no output.
Nothing in the repository is touched by a pass,
so there is no working tree state to restore.
