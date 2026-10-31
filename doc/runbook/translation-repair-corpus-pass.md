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

3.  Prepare the scratch root that will hold the pid file and the run directory,
    which holds the run's records.

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
    SAMPLEDIR="${HOME}/temp/agent/pre-launch-sample"
    TRANSLATION_REPAIR_RUNS_DIR="${SAMPLEDIR}" mise run //package/translation-repair/run:budget-sample
    mise run //package/translation-repair/run:meter-report -- "${SAMPLEDIR}"
    ```

    Expected:
    one reading,
    `meter-report: logs=1 readings=1 unread=0`,
    then the window and each provider's state.
    The sample writes its reading as a record in the run folder it is given,
    `${SAMPLEDIR}/log/<day>/<timestamp>.<uuid>.log.jsonl`,
    the way a pass does,
    and `meter-report` reads the folder.
    The record's message holds the `METERS` line,
    for example:

    ```text
    METERS synthetic=wet hyper=dry syntheticWeekly=97% syntheticFiveHour=48/50 syntheticThrottled=no hyperBalance=0
    ```

    The reading is at `info`
    and a meter that could not be reached warns at `warn`,
    and both are records of the folder.
    A sample taken into a pass's own run folder is read with the pass's readings by `meter-report` on that folder.

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
    mise run //package/translation-repair/run:cap-census -- ~/temp/agent > ~/temp/agent/cap-census.txt 2>&1
    rg '^cap-census:|^[a-z].*: card cap|^  [A-Z]{4}' ~/temp/agent/cap-census.txt
    ```

    The census reads the record files of the run folders under the directory it is named
    (`<run folder>/log/<day>/*.log.jsonl`),
    and takes a pass's by the `START tip=` record its first records hold.
    A `.log` capture of a console is no log to it.

    Expected:
    a `cap-census:` summary line,
    then per seat its card cap and the rule's reading,
    and any flags under it.
    It reads ids and numbers only,
    and spends no quota.
    The summary line counts paths it could not read,
    each named in a `cap-census: cannot read` line
    because a directory it could not list,
    or a link named like a log,
    may hold a run log,
    and counts apart the dangling links under any other name,
    which it passes over unnamed
    since they hold nothing to read.
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
    as its own systemd user unit,
    out of the cgroup every session's shell commands share,
    which `systemd-oomd` may kill whole
    (the repository's `CLAUDE.md` asks for it).
    The pass writes its own records:
    everything it logs and every line it prints is a record in
    `${RUNDIR}/log/<day>/<timestamp>.<uuid>.log.jsonl`,
    one file for each process,
    through the shared logger's file sink,
    which no retention sweep ever touches.
    The file appears with the first record,
    so it is not there at the instant the unit starts.

    ```sh
    systemd-run --user --unit="corpus-pass-$(basename -- "${RUNDIR}")" --collect \
      --property=MemoryMax=12G \
      --working-directory=/var/home/user/worktrees/translation-repair \
      --setenv=TRANSLATION_REPAIR_RUNS_DIR="${RUNDIR}" \
      "$(command -v mise)" run //package/translation-repair/corpus-pass:corpus-pass
    ```

    Expected:
    `Running as unit: corpus-pass-<name>.service`,
    and the shell returns at once.

    Give the unit no `StandardOutput` or `StandardError` property,
    and never redirect the launch to `/dev/null`:
    the console of the pass goes to the unit's journal,
    and the journal is where to read what the records cannot hold
    (`journalctl --user --unit="corpus-pass-$(basename -- "${RUNDIR}")" --output=cat`,
    which keeps the unit's output after `--collect` has removed the unit).
    The records are the evidence of a run;
    the journal is for a refusal before the run log opened,
    which leaves no record because the run log is what had not opened yet:
    the task runner's guard,
    a command line or an environment override that cannot be read,
    a runs directory another pass holds,
    a `MONOCHROMATIC_LOG_DIRECTORY` that names the run folder or a place in it,
    and a run folder the pass cannot write (`${RUNDIR}/log`)
    or that does not answer within 30 seconds.
    Each is stated once on the console,
    the pass exits with a nonzero code (6 for a refusal it states in its own words),
    and the run folder holds nothing of it.
    The journal also holds the build lines of the task (`rolldown v...`).
    Read the records with `run-records`:

    ```sh
    mise run //package/translation-repair/run:run-records -- "${RUNDIR}"
    ```

    It prints one line for each record as the console showed it,
    with control characters written out as `\uXXXX`,
    and `--verbose` adds the debug and trace records the console leaves out.
    Do not read the files with `jq --raw-output '.message'`:
    that prints a message raw,
    so an escape sequence in it reaches the terminal,
    and it fails on a record holding a lone surrogate.
    Before 2026-10-31 this step captured the console into `${RUNDIR}.log`,
    and every report parsed that capture;
    the owner's ruling of 2026-10-31 is that the logger's own `*.log.jsonl` files are the record,
    and a capture beside them means the logger was not used as designed (ledger B1090).
    Stop the pass with `systemctl --user stop corpus-pass-<name>`:
    it records which signal stopped it,
    flushes its records,
    releases the lock of the runs directory and exits
    (a SIGKILL,
    a crash or a power loss flushes nothing,
    and leaves the lock for the next pass to judge stale;
    ledger B1113).
    Where the steps that follow say to kill it by pid,
    the unit's main pid is `systemctl --user show --property=MainPID --value corpus-pass-<name>`.

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
    read the bundler version off the build lines and the digest off `START`.
    The build lines are the unit's console output,
    which reaches the journal and no record,
    and `START` is a record:

    ```sh
    journalctl --user --unit="corpus-pass-$(basename -- "${RUNDIR}")" --output=cat \
      | grep --only-matching --max-count 1 'rolldown v[0-9.]*'
    mise run //package/translation-repair/run:run-records -- "${RUNDIR}" | rg --fixed-strings 'Z] START '
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
    `TRANSLATION_REPAIR_SLICE_OVERLAP=<n>`
    (prints `OVERLAP <entry> at=<time> attempt=<n> value=<n> source=...` per attempt),
    `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=<ms>` (prints `STRAGGLER GRACE OVERRIDDEN by ...`),
    `TRANSLATION_REPAIR_WRITER_GRACE_MS=<ms>` (prints `WRITER GRACE OVERRIDDEN by ...`;
    the editor,
    refiner,
    translate and consolidate rounds alone wait this long on a straggler after quorum,
    the other rounds keep the round window).
    Unset,
    the writer rounds wait their built-in 180000 ms and the launch prints `WRITER GRACE built in`,
    and the pass keeps four slices in flight (`OVERLAP <entry> at=<time> attempt=<n> value=4 source=fallback`);
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
    TRANSLATION_REPAIR_STARTED_BY=meter-report node dist/final/node/meter-report.mjs "${RUNDIR}"
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
Each is the exact string the run's records carry,
as the `message` of a record in `${RUNDIR}/log/<day>/*.log.jsonl`
(`mise run //package/translation-repair/run:run-records -- "${RUNDIR}"` prints them as lines).

-   `[error]` at any point.
    A healthy long run carries none.
    Count them rather than reading for them,
    by the level each record carries:

    ```sh
    mise run //package/translation-repair/run:run-records -- "${RUNDIR}" | rg --count '^\[error\] '
    ```

    Expected:
    no output and exit status 1,
    because `rg --count` prints a count only for an input that matches
    and exits 1 when nothing matches.
    Any number printed is the count of `[error]` records.

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

-   `START ... hard=<ms>ms hard-scope=per-attempt`,
    on every launch.
    The hard ceiling ends one attempt and the next attempt of the same entry begins under a fresh one,
    so an entry that hits it and is re-attempted runs for more than the ceiling in one invocation,
    which is not a ceiling that failed to fire.

-   `at=<time> attempt=<n>`,
    on every line the pass prints for an attempt of an entry
    (`OVERLAP`,
    `TALLY`,
    `INTERRUPTED`,
    `DESTINATIONS`,
    `DEFECTS`,
    `INTERIM`,
    `CLEANUP`,
    `REATTEMPT`,
    `STALLED`,
    `INCOMPLETE`),
    straight after the keyword and the entry
    (and after the status on a `TALLY`,
    or `queued` on a `REATTEMPT`).
    `at=` is the time the line was rendered,
    an ISO 8601 UTC time with milliseconds,
    so a window of the log is found by it
    (a settled `TALLY` is rendered before its page is persisted,
    so it reads a little earlier than the `DESTINATIONS` line printed beside it);
    `attempt=` is the number `attempts.json` holds for the entry,
    so the `TALLY` of an attempt and the `REATTEMPT` after it print the same number.
    A tally ending `error=Timeout: ... exceeded its ...ms deadline`,
    a `REATTEMPT` after it and a later `attempt=` read as one cap ending one attempt,
    and not as a cap that failed to fire.

-   `at=<time>` alone,
    on the lines of the pass itself:
    `START` and `DONE` end with it,
    as do the lines that stop the pass between entries
    (`SOFT budget reached`,
    `SPEND CEILING reached`,
    `STOP every provider is out of budget`).
    They belong to no attempt and carry no `attempt=`.
    The `INCOMPLETE <id> at=<time>: <side> page absent at the pin` line is printed before any attempt exists,
    so it carries `at=` straight after the entry and no `attempt=`,
    and that absent field tells it from the `INCOMPLETE <id> at=<time> attempt=<n>:` line of an attempt.
    The lines the tagged logger writes carry the logger's own time,
    and those inside an attempt hold an `[attempt <n>]` bracket after the entry
    (`[<entry>] [attempt <n>] ...`,
    and `[<entry>] [runDocumentLanes] [repairPreparedDocument] [attempt <n>] ...`
    for a lane's own lines,
    where the function tags come first),
    followed inside a slice by `[<lane> slice <i>]`,
    with the lane `repair`,
    `translate`,
    `consolidation` or `contest`.
    Search for the `[attempt <n>]` bracket and not for its place after the entry
    (`package/translation-repair/doc/configuration.md` states the order).

-   `CAP OVERRIDDEN`,
    only when `TRANSLATION_REPAIR_HARD_CAP_MINUTES` was set.
    It names the ceiling each attempt of an entry runs under actually in force.

-   `CAP TOO TIGHT`,
    which is a warning and not a refusal.
    It means the ceiling is at or below one model exchange,
    currently 360000 milliseconds,
    so every attempt is cut before any exchange returns,
    nothing caches,
    and the queue drops each entry as stalled on its second try.

-   `REATTEMPT <id> queued at=<time> attempt=<n>`,
    which is healthy only after resumable operational error or hard cap.
    Entry that bought more cache records goes to back of queue.
    Quality rejection and `status=INCOMPLETE` must never produce this line.

-   `TALLY <id> status=INCOMPLETE at=<time> attempt=<n>`,
    which means stage-local work remains.
    It is neither success,
    quality verdict,
    nor publication evidence.
    Cache stays,
    but whole entry does not restart in same invocation.

-   `STOP every provider is out of budget ... at=<time>`,
    which means the pass stopped at once because no funded provider serves a writing seat.
    The pass never waits for NeuralWatt's day to start again
    (the owner's answer of 2026-10-29,
    "Never wait,
    relaunch":
    "Stop at once and relaunch later.").
    The cause in the line states the time of day in UTC NeuralWatt's day starts again
    (`at 19:15:08 UTC (in 5 min 1 s)`),
    so launch the pass again after that time.
    The entries already settled keep their pages,
    and the entry in flight stays resumable and ships an interim page (next entry).
    No `TALLY` line says `PARKED`,
    and no `NOT RESUMED` line follows a stop.

-   `INTERIM <id> at=<time> attempt=<n> settled=<n> archive=<n> source=<n> reason=dry-stop`,
    printed just before the `TALLY <id> status=INCOMPLETE` line of an entry the dry stop ended in the middle of its page
    (the owner's answer of 2026-10-29,
    "Ship interim,
    resumable").
    The page of that entry in the published tree is interim:
    `settled` slices carry the wording a consolidation record settled,
    `archive` slices carry the archive's text,
    and `source` slices carry the original under the note
    "This passage has not been translated yet.
    The original text follows."
    No artifact is written,
    so the entry is still pending;
    `interim/<id>.json` in the runs directory records the counts and the digest of the page.
    Launch the pass again after the time the stop line states:
    it repeats the entry on its caches,
    overwrites the page,
    and removes the record when the entry settles.
    A stop before the entry is sliced prints no `INTERIM` line and ships no page.
    `mise run //package/translation-repair/run:verify-published` names the page `INTERIM PAGE` with the counts,
    or `INTERIM PAGE DISAGREES WITH ITS RECORD` when the page on disk is not the one the record was written for.

-   `STALLED <id> at=<time> attempt=<n>`,
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
remove its directory,
which holds its records.
Do this only for a throwaway run.

```sh
rm --recursive --force -- "${RUNDIR}"
```

Expected:
no output.
Nothing in the repository is touched by a pass,
so there is no working tree state to restore.
