# @monochromatic-dev/mutation-test-cli-cargo-mutants

Product-neutral tooling for `cargo-mutants` campaigns on any Rust product in this repository.
It holds the launcher that runs several campaigns at once and the slot limiter that keeps the host from being
overrun by them.
Anything specific to one product (its mise task,
its image names,
its source layout) is an argument here,
never a constant.

The sibling `package/mutation-test/cli` mutates TypeScript with its own engine and shares nothing with this package.
The inventory of other cargo-mutants scripts and the plan to fold their neutral parts in is
`doc/planning/shared-cargo-mutants-package.md`.

## Launcher

```bash
# mise task: any Rust product, with its own campaign task
mise run //package/mutation-test/cli-cargo-mutants:campaign:parallel -- \
  --task //package/some/cli:mutation:scoped --tag-env SOME_IMAGE_TAG \
  --image-repository localhost/some-test --from-tag development \
  --scope-prefix src/ <label> <file.rs>...
```

The launcher starts one capped `systemd-run --user` unit per file and returns at once.
Each unit runs `mise run <task> -- --timeout <seconds> <scope flag> <scope prefix><file>` through the slot limiter
(plus `--build-timeout <seconds>` and one `--examine <regex>` per pattern when given,
see the options),
with the image tag in the variable named by `--tag-env`,
so no two campaigns share an image.
Before the first unit starts,
the launcher makes sure every unit's image exists (see "Image tags").
Output goes to `<log dir>/<log prefix>-<label>-<stem>.log.<attempt>`.
The launcher prints one line per unit and a `systemctl --user is-active` command that waits for the started ones.

### Exit status and errors

The launcher exits 0 only when every unit started.
When `systemd-run` refuses a unit,
the launcher still tries every other unit,
prints that unit's line with the `systemd-run` error,
then a summary that names each failed file and unit,
and exits 1.
The units that started keep running:
they are independent,
and a stopped one resumes when launched again.
The wait command lists only the started units,
and is left out when none started.
To start the failed files,
launch only them under another label:
a tag belongs to a file's position in the list,
so the same label would reuse the tags of the running units.
(Before 2026-11-07 a refused unit left the exit status 0.)

A launch that is refused before any unit starts
(a wrong option,
a missing image tag,
two files with one unit name)
prints only the message of the `CargoMutantsToolingError` to standard error and exits 1.
Any other error is a defect of the launcher or a failure of the host,
and keeps its stack trace
(`runLauncherMain` in `src/launcher-main.ts`).
An option the argument parser itself rejects (an unknown flag,
a flag without its value) is one of those:
it ends with the parser's `TypeError` and a stack trace.

### Image tags

A unit runs the image `<repository>:<tag>`,
with the repository from `--image-repository` and the tag built from `--tag-template`
(`{label}{index}` by default,
one tag per file).
A unit whose image does not exist dies at once with "image not known",
so the launcher decides about every tag before it starts any unit
(`ensureUnitImages` in `src/unit-images.ts`).
On 2026-11-07 it did not:
it started 21 units whose tags did not exist,
every unit died at once,
and the launcher exited 0 and printed "started" for each.

- With `--from-tag <tag>`,
  the image `<repository>:<tag>` (the gate image) is the source.
  The launcher reads its full ID,
  runs `podman tag <full ID> <repository>:<unit tag>` for every unit,
  and then reads every unit tag back:
  a tag that does not name the source's ID stops the launch,
  naming the tag,
  both IDs and the `podman tag` command that makes it.
  A missing source image stops the launch too.
  `podman tag` moves a tag that already exists,
  so relaunching a label after a new gate points that label's tags at the new image
  (a resumed campaign then has another image ID,
  so another campaign key,
  see "Where the outcomes live").
  The launcher prints a line naming each tag that held another image,
  with both IDs,
  and leaves the old image in storage.
- Without `--from-tag`,
  every unit tag must already resolve to a full image ID.
  A missing one stops the launch before anything starts and exits nonzero,
  naming each missing tag with the command that makes it,
  `podman tag <repository>:<gate tag> <repository>:<unit tag>`.
- The launcher's only writes to Podman are `podman tag` calls.
  It reads with `podman image inspect`,
  in the environment a unit gets (so it looks in the same container storage),
  and never removes an image or a tag.
- The launch is refused before `systemd-run` is called,
  so no unit starts when it is refused.

### Reruns

A unit reruns its campaign,
at most `--attempts` times,
only while the attempt's log contains the retry marker
(`FAILED   Unmutated baseline`,
with three spaces):
the baseline run of the unmutated source failed,
which a loaded host
can cause.
Any other outcome,
including missed mutants,
ends the unit after one run.

A killed unit (`systemctl --user stop <unit>`),
a reboot or an out-of-memory stop does not lose the campaign:
start the same launcher command again and each unit resumes its campaign,
scoring only the mutants that no earlier
run scored (see "Resuming a campaign").
A rerun with the same label appends to the same log files `<log prefix>-<label>-<stem>.log.<attempt>`,
and a different
label starts new ones;
the resume does not depend on the label.

### Options

Run the script without arguments for the same list with defaults.

- `--task <task>`,
  `--tag-env <name>` and `--image-repository <repository>` are required:
  the campaign task,
  the environment variable through which it reads its image tag,
  and the repository of the image a unit runs (`<repository>:<tag>`),
  which the launcher needs to make or check each unit's tag (see "Image tags").
- `--from-tag <tag>` names the gate image,
  the tag in `--image-repository` that every unit's tag is made from with `podman tag` before any unit starts
  (see "Image tags").
  Without it every unit tag must already exist.
- `--tag-template <text>` builds the tag from `{label}` and `{index}`;
  `{index}` is required (default `{label}{index}`).
- `--scope-prefix <text>` and `--scope-flag=<flag>` turn each file into the task's scope argument
  (default no prefix and `--file`).
  The scope flag needs the equals sign,
  because a value that starts with hyphens is read as another option.
- `--setenv <NAME=value>`,
  repeatable,
  adds environment to every unit.
- `--timeout <seconds>` is the per-mutant test timeout forwarded to the task (default 900).
- `--build-timeout <seconds>` is the per-mutant build timeout,
  cargo-mutants' own `--build-timeout` ("maximum run time
  for cargo build command"),
  forwarded to the task only when given;
  otherwise the task's default applies
  (`defaultBuildTimeoutSeconds`,
  300 seconds,
  in `src/campaign-files.ts`).
  A `TIMEOUT ... in 300s build` line of a campaign log means the build of that one mutant used the whole build timeout,
  which a heavily loaded host can cause;
  rerun that mutant with a larger `--build-timeout`.
  Each product's scoped task accepts it up to its maximum (3600 seconds unless the product names another).
- `--examine <regex>`,
  repeatable,
  is a mutant-name pattern forwarded to every unit's task,
  so only the mutants whose
  names match are tried in every listed file;
  the pattern may hold the plain characters the scoped task accepts
  (names,
  `|`,
  `:`,
  `.`,
  `*`,
  `?`,
  parentheses,
  backslash,
  space and hyphen) and is checked here by the same function.
  It reruns one named mutant,
  for example a timed-out one:
  `-- --timeout 2400 --build-timeout 1200 --examine read_regular_file m047 private_storage.rs`.
- `--memory <size>` and `--cpus <count>` are the container limits.
  They are forwarded to the task only when given;
  otherwise nothing is forwarded and the task's defaults apply.
  The process limit (`--pids-limit`) is not a launcher option;
  the task's default is 512 processes
  (`defaultPidsLimit` in `src/container-limits.ts`,
  128 before 2026-11-01).
  Every started container also gets the lowest disk weight (`--blkio-weight=10` and `--cgroup-conf=io.weight=10`),
  which no option raises.
- `--passes <1|2>` is forwarded to the task only when given.
  `--passes 2` asks a product that offers it to run each file's campaign in two passes (see "Two passes").
- `--unit-memory-max <value>`,
  `--unit-cpu-quota <value>` and `--unit-tasks-max <count>` cap the whole systemd unit
  (the task,
  the container client and the build),
  not the container;
  they are separate from `--memory`,
  `--cpus`
  and `--pids-limit` (defaults `12G`,
  `500%` and 256 tasks).
  The task limit counts threads as well as processes and must be a whole number;
  `defaultUnitTasksMax` in
  `src/constants.ts` records the measurement it comes from,
  and a start that fails against it reports a bare `EAGAIN`,
  which is the sign to raise it.
  Every unit also gets `IOWeight=10`,
  the highest the owner's rule in `CLAUDE.md` (RXS) allows;
  no option raises it,
  and the container,
  which sits outside the unit,
  has its own lowest weight (see the `--memory` and `--cpus` entry).
- `--unit-prefix`,
  `--log-prefix`,
  `--log-dir`,
  `--tmp-dir` and `--root` place names and files;
  the working tree comes from `--root`,
  then `MUT_ROOT`,
  then the checkout that holds this package.
  Logs and `TMPDIR` default under `<home>/temp/agent`.
- `--attempts <count>` and `--retry-marker <text>` tune the rerun rule.
- `--slot-count <count>` and `--lock-dir <dir>` (or `GATE_SLOT_COUNT` and `GATE_SLOT_DIRECTORY`) are forwarded to the
  limiter.

Every file name reaches the shell body as one single-quoted word,
so spaces,
quotes,
`$`,
`;`,
backticks and newlines
stay literal.
Two files that collapse to one unit name make the launch fail before anything starts.

## Slot limiter

```bash
mise run //package/mutation-test/cli-cargo-mutants:campaign:gate-slot -- mise run //package/some/cli:test:container
```

The limiter holds no lock.
Each slot is a series of numbered claim files,
`slot-1.owner.<generation>` to `slot-N.owner.<generation>`,
in the lock directory,
made as `package/git-policy/cli/src/native/lock/single_owner_claim.rs` makes its claims (see "Single writer").
A gate takes the lowest free slot at once,
with one exclusive `link` of a new generation,
and runs the command inside it at
nice 19 with inherited stdio;
the command's exit status becomes the limiter's.
A slot is free when its newest generation is released,
or names a process that no longer exists with that start time and boot,
so a gate that was killed leaves nothing for a person to clean up.
The gate waits only while all slots are held by live processes.
It writes one notice to standard error naming the directory and each holder (`slot 2: process 41233`),
waits for a change in
the lock directory (a release renames a file there,
so the gate wakes at once),
and writes `still waiting` notices at the
notice interval,
10 seconds by default (`--notice-interval-ms`).
A holder that dies makes no event,
so the wait also ends after `--poll-interval-ms` (15 seconds by default) and looks again.
The gate process holds its claim for as long as the command runs and passes `SIGINT`,
`SIGTERM` and `SIGHUP` on to the command.
A gate killed by `SIGKILL` cannot,
so its slot reads as free while its command may still run.
`systemctl --user stop` of the unit that runs the gate ends the command as well,
and that is the way gates are normally stopped.
The default is six slots in `<home>/temp/agent/gate-slots`.
Every launcher on the host must share that directory for the limit to hold,
so change it only on purpose.
The limit exists because the host runs under a shared CPU cap;
the slot count follows the user's CPU hold,
documented at `defaultSlotCount` in `src/constants.ts`.

### Gates of the earlier release

Until this release the limiter held `flock` locks on `slot-N.lock` files for as long as the command ran.
A gate that started before the change may still run,
and a gate that waits polls `flock --nonblock` on `slot-N.lock` every
few seconds without ever looking at a claim file,
as does any checkout that still has the earlier source.
So a gate of this release holds that kernel lock too,
for exactly as long as its claim (`src/legacy-flock.ts`):
it asks
`flock` once,
without waiting,
for the lock on the slot's file (creating the file as a poll of the earlier release does) after
it has taken the claim,
keeps the file open while the command runs,
and gives the lock back just before the claim.
A waiting gate of the earlier release then finds the slot held and keeps waiting.
Each time a gate looks at a slot it first asks,
without waiting,
whether the lock is held:
a held lock with a live claim in the
slot is named in the notice as that process,
and a held lock without one as a gate of the earlier release.
The lock files are never removed:
a probe that removes a file races with a gate of the earlier release that has just created
and locked a new one at the same path,
and would delete the file that gate holds.
A directory or a pipe where a slot file belongs is no lock file of the earlier release,
which makes only plain files:
the gate gives its claim back and stops with an error naming the path (`src/legacy-hold-or-give-back.ts`),
and the person moves the path away,
instead of the slot counting as held by an earlier release for good.
A gate that reads the interim source (claims without the lock) holds only a claim,
and an earlier-release gate can still take
its slot;
that source existed for hours,
and the limit is whole again once no such gate runs.

## Who runs a campaign

A session's main agent starts every campaign,
waits on it and reads its results itself.
It never hands a campaign,
or the wait for one,
to a subagent.
The owner's reason,
2026-11-01:
"Because of how a few Claude Code quirks work together,
long-running mutation runs when handed to subagents would burn
abnormally amount of compute,
so they'd have to be done by the main agent."
And the mechanism:
"subagent context would be auto dumped into main agent's context (expensive!)
if the subagent doesn't have any
foreground work.
Foreground bash is limited to max 3 min by Claude Code and the tuner doesn't work."
The record and the workaround are in
[`doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md`](../../../doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md).

## Resuming a campaign

Every campaign run through `runCampaign` is resumable.
A kill,
a reboot,
an out-of-memory stop or a stopped unit loses no scored mutant,
and running the same command again
continues where the campaign stopped.
Nothing needs to be passed to resume:
run the same task with the same options on the same tested image.

### Where the outcomes live

The campaign's evidence directory is `<evidence root>/<prefix><key>`,
for example
`target/verification/native-mutation-<key>/`.
The key is the first twelve hexadecimal digits of a SHA-256 over the tested image ID and the exact command,
and over `CampaignDefinition.keyParts` when a product gives any,
so the same campaign finds the same directory,
and another snapshot,
scope,
option or product condition starts a clean one
(`campaign.json` holds the full identity and refuses a directory that belongs to another).
A product puts there whatever its kept outcomes depend on:
the native wrapper adds the digest of its planted controls and the identity of the code that judges them,
so a changed control table or runner rescores everything instead of keeping outcomes scored under the old ones.
A product with no `keyParts` keeps its keys.
Every attempt directory the runner makes also gets `attempt-protocol.json`,
which says the runner writes `container-started.json` before every container start,
so a reader may take a missing marker beside that attempt as proof that no container started in it.
The record belongs to the attempt,
not to the campaign:
an older runner can resume a directory a newer one began,
and its attempts carry no record and may have started a container without leaving a marker;
for such an attempt a missing manifest is unknown,
not harmless.
Each run of the container is an attempt with its own `attempt-NNN/` directory inside it,
bind-mounted at the
container's report path.
cargo-mutants appends every scored mutant to `caught.txt`,
`missed.txt`,
`timeout.txt` or `unviable.txt` as it goes,
so those lines are on the host the moment they are written and survive anything that stops the container.
The container is created with `--userns=keep-id:uid=<image uid>,gid=<image gid>` read from the campaign image's
configured user (root for an image with no `USER`),
so it still runs as the image's user while the files it writes
belong to the host user and can be deleted without `podman unshare`;
a user given by name gets no mapping.
The mount carries the `:z` label because the host runs SELinux.

### How a rerun skips scored mutants

The runner reads the lists of every earlier attempt and writes all four outcomes into the new attempt's
`mutants.out/previously_caught.txt`;
the command always carries cargo-mutants' `--iterate`,
which reads that file
before it rotates `mutants.out` and leaves out every mutant named there.
`--iterate` alone would not be enough:
cargo-mutants 27.1.0 documents (`book/src/iterate.md`) and implements
(`load_previously_caught` in `src/output.rs`) that it skips only mutants that were caught or unviable,
so a missed or
timed-out mutant would be scored again,
and the single `mutants.out` it keeps is rotated to `mutants.out.old` and
deleted by the next run,
so a second kill would lose the first attempt's missed list.
Seeding the missed and timed-out names into the file `--iterate` already reads,
and keeping one directory per
attempt,
gives "never rescore a scored mutant" with only what cargo-mutants supports.

### What a finished campaign leaves

`<evidence>/mutants.out/` holds `caught.txt`,
`missed.txt`,
`timeout.txt` and `unviable.txt` as the union over all
attempts,
an `outcomes.json` whose counts come from those lists,
and the `log`,
`diff` and `mutants.json` copied from
the attempts,
as before.
`exit.json` carries the status of the whole campaign (a timeout in an earlier attempt still ends the campaign with
status 3 although the last attempt,
handed nothing to score,
exits 0),
plus `attempt` and `skippedMutants`.

### Single writer

The right to run a campaign is a numbered claim file in its evidence directory,
held by the campaign process itself
(`src/campaign-lock.ts`,
`src/owner-claim.ts`),
made as `package/git-policy/cli/src/native/lock/single_owner_claim.rs` makes its claims.
`campaign.owner.<generation>` is written whole into a private file and published with one exclusive `link`,
so exactly one
process creates generation `n + 1`;
the newest generation is the claim,
and it names its owner by process id,
birth (start
time from `/proc/<pid>/stat` field 22 plus the boot id) and PID namespace (`src/process-identity.ts`).
A second run of the same campaign is refused at once with the owner's process id and claim file named,
and never waits.
A run that ended normally rewrites its claim as released.
A run that was stopped by `systemctl --user stop`,
lost its host in a reboot or was killed leaves its claim naming a process
that no longer exists with that birth,
and the next run takes the next generation;
there is nothing to clean up by hand.
The newest generation file stays as a tombstone,
so a run that listed the directory earlier cannot create a number that was
used.
An owner in another PID namespace is never judged by the age of its claim file,
so a gate that runs for days keeps its slot.
Its claim file names the id the owner has inside its own namespace;
a process of a namespace nested in the reader's shows that
id on the `NSpid` line of its `/proc/<pid>/status`,
and with the start time in the claim it names the owner exactly
(`src/foreign-process-scan.ts`).
The owner is gone when it began in an earlier boot,
when a process of its namespace has its id with another start time,
or when
its namespace is seen,
every process could be read,
and none has its id.
A namespace this process cannot see into shows nothing,
and nothing is not proof of death,
so such an owner (and one whose
record or reader names no namespace) counts as alive;
a holder that crashed there is freed by removing the newest claim file of its slot in the lock directory the notice names
(`<slot claim prefix>.<generation>`).
The claim knows its owner only by process id,
start time and boot.
The kernel lock this replaced was exact about whether the holder still lived;
the claim is exact for a process that was killed
or lost to a reboot,
but a campaign process that handed its work to another process and exited would read as finished.
A campaign that the earlier release started holds a kernel lock on the evidence directory instead of a claim.
The run takes its claim first and then asks `flock` once,
without waiting,
for that lock (`src/legacy-flock.ts`);
a held lock gives its claim back and refuses the run,
naming the earlier release.
A lock that is free is kept for as long as the run holds its claim,
so a campaign of the earlier release that starts on the
same directory meanwhile is refused by the lock it takes without waiting.
The claim is needed although the parallel launcher names each campaign's systemd unit,
because a run started by hand,
or by another launcher under another label for the same file,
names the same directory,
and the runner removes an orphaned container of the campaign's name before it creates its own.
A container left by a killed run keeps its name (`cargo-mutants-<key>`) and is removed before the next one is
created.
On `SIGTERM`,
`SIGINT` or `SIGHUP` the runner stops the container (the container is supervised outside the
runner,
so ending the runner need not end it),
composes the partial report and fails;
`systemctl --user stop <unit>` takes this path.

### Planted controls

A product's `beforeBuild` hook is always called and receives `scoredMutants`,
the number of mutants earlier attempts
scored.
The native wrapper does not take it as proof:
mutants scored by an earlier attempt say nothing about a control added or a limit lowered since.
A product can record a pass of its controls in the campaign's evidence and check it in the same hook.
The native wrapper does (see "What a mutation campaign costs per mutant" in `package/git-policy/cli/README.md`),
keyed by the full image ID,
the controls,
the code that judges them and the limits.

### Starting over

Delete the campaign's directory under the evidence root (its files belong to the host user).
A source change gives a new tested image ID and so a new directory without any deletion.

### Limits

A mutant that was in flight when the container died is not in any list and is scored again.
The baseline run of the unmutated source repeats on every attempt,
because it also checks the environment after a
reboot,
unless the product proved that its gate passed on the same image (see "Skipping the baseline").
Mutants match by file,
line,
column and description,
as in cargo-mutants' own `--iterate`.

## Skipping the baseline

cargo-mutants runs the whole test suite on the unmutated tree before it tries a mutant (`--baseline run`),
so that a
failing suite is not mistaken for caught mutants.
When a gate has already run that suite on the same image,
the baseline repeats work:
about 265 seconds of unit time
per file for the native wrapper (`doc/planning/mutation-campaign-per-mutant-cost.md`).

A product opts in by giving `CampaignDefinition.baselineProof`,
a function from the full ID of the tested image to a
short description of the proof,
as the only element of a list,
or to an empty list when it has none.
With a proof the container runs `cargo mutants --baseline skip`,
which is cargo-mutants' own switch (`--baseline`,
values `run` and `skip`;
with an explicit `--timeout` and `--build-timeout`,
which every campaign passes,
it has no
fallback timeout).
Without a proof the baseline runs as before.
The proof must never be a guess,
and it must cover the campaign's execution envelope:
a product that cannot show that
its gate ran with at least the campaign's memory and process limits answers with no proof.
The baseline of 2026-11-01 failed under 128 processes where the gate's tests pass under more,
so a campaign that runs
with fewer processes than its gate would count the same fork failures as caught mutants.
`memoryBytes` compares memory sizes given in different units.

The campaign key and `campaign.json` hash the command with `--baseline run`,
so a campaign that gains the proof between
two runs finds the same directory and resumes.
`skipBaselineArguments` makes the container's command,
the Containerfile and the created container carry the skip,
and
the manifest records the proof as `baselineProof`.
The launcher's retry rule keys on the text a failed baseline prints,
so it never fires for a skipped one.

The git-policy proof is a gate directory in the evidence root whose `manifest.json` names the image ID and whose
`passed.json` says the tests and Clippy passed;
see `package/git-policy/cli/bin/native-gate-proof.ts`.
(The `git-policy-baseline-skip` golden is a different thing:
`NATIVE_MUTATION_BASELINE_SKIP` names one test to leave
out of the baseline and of every mutant run.)

## Two passes

A campaign can run its mutants in two passes,
to avoid running every test for every mutant.
A product opts in by giving `CampaignDefinition.passOne`,
the campaign command with a nextest filterset added
(`selectTestsArguments`),
and selects the mode with `--passes 2` on its task,
which `parseTimeoutAndExamine` reads for a
product whose rules set `acceptPasses`.

A two-pass campaign starts only with a baseline proof (see "Skipping the baseline").
Pass 1 runs a filtered baseline at best,
and a campaign whose pass 1 catches everything never starts pass 2,
so without
a proof nothing would run the whole suite on the unmutated tree.
Without one `runCampaign` stops before it builds anything and names the product's `testImageHint` (for the native
wrapper,
the gate task).

Pass 1 runs the filtered tests.
Pass 2 runs the whole suite,
but only on the mutants pass 1 missed or timed out:
pass 1's caught and unviable mutants
are written to pass 2's `mutants.out/previously_caught.txt`,
the file `--iterate` already reads,
so cargo-mutants leaves
them out.
Nothing is scored twice and no kill is lost:
a mutant any subset of the tests catches is caught,
and a mutant the subset
misses meets the whole suite.

### Where each pass lives

The campaign's key hashes the tested image,
`CampaignDefinition.command` and `CampaignDefinition.keyParts`,
so it is the key of
the one-pass campaign over the same files and conditions.
Pass 1 is an ordinary resumable campaign in the `pass-1` directory of the evidence directory
(`<evidence>/pass-1/attempt-NNN/`,
its own `manifest.json`,
`mutants.out` and `exit.json`);
pass 2 is the ordinary
campaign of the evidence directory itself (`<evidence>/attempt-NNN/`).
The pass-1 test selection is deliberately not part of the key:
it comes from the source tree on the host,
which can
change while the tested image stays,
and a mutant caught by any set of tests is caught.
Containers are named `cargo-mutants-<key>-pass-1` and `cargo-mutants-<key>`.
One claim (`campaign.owner.<generation>` in the evidence directory) covers both passes.

### Composition and status

`<evidence>/mutants.out` is the composed report.
Its four lists are pass 1's with every mutant pass 2 scored taken from pass 2 (`overlayScoredNames`),
its
`outcomes.json`
holds pass 1's entries for the mutants pass 2 did not score and pass 2's entries for the rest,
and `log/` and `diff/`
are pass 1's overwritten by pass 2's for the same mutant.
`total_mutants` and the counts come from the lists,
as for a one-pass campaign.
A pass-2 container that has not finished leaves pass 1's missed and timed-out mutants in the lists,
so a partial report
still names every survivor.
The exit status is cumulative as before (a timeout wins over missed mutants),
and `exit.json` adds a `passes` record:
whether pass 1 finished,
how many mutants it scored and missed,
and whether pass 2 ran.

### Resuming

- A kill in pass 1 resumes pass 1 as any campaign resumes:
  the next run seeds the mutants it had scored and scores the
  rest,
  then goes on to pass 2.
- Pass 1 counts as finished when its last `pass-1/exit.json` records a finishing status (0,
  2 or 3) and its attempts
  scored every mutant the first attempt listed in `mutants.json`.
  A finished pass 1 starts no container on a later run.
  An exit that looks finished but scored fewer mutants than listed is recorded as status 1 and stops the campaign.
- Pass 2 runs only when some mutant pass 1 missed or timed out is not yet in pass 2's lists.
  A kill in pass 2 resumes it with the same seed plus what pass 2 had scored,
  so each survivor is scored once.
- Rerunning a finished campaign scores nothing and starts no container.
- A campaign directory keeps its mode:
  `campaign.json` gains `"passes": 2` for a two-pass campaign,
  and a run in the
  other mode is refused before anything starts,
  with a message that says which mode the directory holds.
  The two modes share the evidence directory,
  and a one-pass run would score again what pass 1 caught,
  so switching
  needs the directory moved away (a one-pass directory written before this field existed counts as one-pass).

### Pass 1 of the native wrapper

The native wrapper selects pass 1 from its own sources (`package/git-policy/cli/bin/native-pass-one.ts`,
which uses
`testScope` and `scopeFilterset` from this package):

- It reads `Cargo.toml` for the library and the declared `[[test]]` targets,
  follows `mod` declarations and
  `#[path]` attributes from each target's root into a module tree,
  and scans every file's tokens (comments,
  strings and
  character literals are skipped) for paths that start with `crate::`,
  `super::` or the library's name.
- A top-level module imports another when any file below it has such a path whose first segment is the other module's
  name,
  or opens a `use` group that lists it.
  The scan over-approximates (an item with a module's name also counts),
  which only makes pass 1 larger.
- Reaches without a path:
  a glob of the crate root (`use crate::*`,
  or `use super::*` in a module directly below the
  root) lists every identifier of the file;
  an item the root re-exports with `use` counts as the module it comes from
  (and a glob re-export,
  as any of that module's items);
  `include!` counts as reaching every module,
  because the
  included file is not read.
- Declarations inside an inline module (`mod outer { mod inner; }`) are found below that module's directory and belong
  to the top-level module `outer`.
  A `#[cfg_attr(condition, path = "...")]` is followed to every file that exists,
  beside the file the name gives.
  Brackets and braces are matched by token kind,
  so a string such as `"["` is not structure.
  What an inline module's own body refers to is attributed to the file's root,
  not to the inline module.
- Pass 1 holds the mutated file's module and every module that reaches it through any chain of importers.
  One level of importers is not enough:
  for `repo_read_query` the killing tests of three measured mutants
  (`candidate_known_content`,
  `commit_convergence`,
  `config_snapshot`) sit in modules that import it only through
  `repo_read_hook`.
  A test target such as `native_candidates` joins when any of its files refers to a module in scope;
  `native_binary`
  is excluded by name (`slowTestTargets`) because it is slow and exercises everything.
- The filterset is `(kind(lib) & test(/^(?:a|b|c)::/)) | binary(=native_candidates)`:
  an anchored regular expression
  over the module path at the start of the test name,
  which also selects the test files that hang under a module
  (`a::tests`,
  `a::worktree_tests`).
  It reaches nextest as one `--cargo-test-arg=--filterset=<expression>` argument,
  and `--cargo-test-arg=--no-tests=pass`
  keeps an empty selection from failing the pass (nextest 0.9.148 in the image has both;
  checked against the nextest
  documentation and a scratch crate with 0.9.150).
- `--passes 2` refuses a glob `--file`,
  no `--file`,
  and a file the library does not declare.

Without a `Cargo.toml` `[[test]]` table a test target Cargo discovers by itself (`tests/*.rs`) is not seen by the scan,
and only the library is selected;
pass 2 still runs those targets.

The tests drive the whole flow with stand-in `podman` and cargo-mutants programs (`src/fake-programs.ts`,
`src/fake-mutants-simulation.ts`):
`src/campaign-resume.unit.test.ts` covers a fresh campaign,
a campaign killed
partway and run again,
and a finished campaign run again.

## Campaign building blocks

Every Rust product that runs cargo-mutants in containers uses these through `./ts`,
and keeps only an adapter script
that supplies its image names,
scopes,
limits defaults and manifest:

- `excludedMutantKinds` and `exclusionArguments`:
  the two mutant kinds no campaign tries,
  one repository-wide
  constant;
  a product adds exclusions on top and never removes one.
- `parseContainerLimitOptions` and the argument builders for `--memory`,
  `--cpus`,
  `--pids-limit` and the lowest disk
  weight (`lowestDiskWeight`,
  `diskWeightArguments`,
  both from `@monochromatic-dev/container-limit-core`;
  `runLimitArguments` adds it to every started container,
  and
  `podman build` has no such option,
  so the build arguments carry none).
- `parseTimeoutAndExamine`,
  `examinePatternOf`,
  `parseFileScopes` and `baselineSkipArguments`:
  the campaign options
  (`--timeout`,
  `--build-timeout`,
  `--examine`,
  `--file`) and their messages.
  A product passes the parsed `buildTimeoutSeconds` to `campaignCommand`,
  which hands it to cargo-mutants.
- `campaignFailureMessage`:
  the failure text of a campaign container;
  the timeout exit status names both timeout
  options,
  because cargo-mutants gives a build timeout and a test timeout the same status.
- `campaignCommand`,
  `campaignContainerfile`,
  `evidenceJson` and `exitRecord`:
  the command,
  the Containerfile that
  layers cargo-mutants over a tested image,
  and the evidence files.
- `runCampaign`:
  the whole container flow (inspect the tested image,
  find or open the campaign's evidence directory and
  its attempt,
  build with `--iidfile`,
  create from the image ID with the command given and the attempt's report
  directory bind-mounted,
  read the command back,
  start,
  compose the report of all attempts,
  write the evidence,
  remove
  the container);
  see "Resuming a campaign".
- `campaignKey`,
  `prepareAttempt`,
  `readScoredNames`,
  `seedScoredNames`,
  `composeReport` and `cumulativeStatus`:
  the
  resume bookkeeping that `runCampaign` uses,
  exported for adapters that report on a campaign's directory.
- `skipBaselineArguments` and `CampaignDefinition.baselineProof`:
  the baseline skip (see "Skipping the baseline").
- `selectTestsArguments`,
  `CampaignDefinition.passOne`,
  `runTwoPasses`,
  `openCampaign`,
  `openAttempt`,
  `mergeScoredNames`,
  `overlayScoredNames`,
  `plannedMutantCount` and `composeReport({ evidence, earlier })`:
  the two
  passes and their composition (see "Two passes").
- `testScope`,
  `scopeFilterset`,
  `parseCargoTargets`,
  `scanTargetModules`,
  `tokenizeRust`,
  `moduleDeclarations` and
  `referencedNames`:
  the Rust source scan that finds the tests reaching a mutated file,
  with no Cargo run.
- `listMutants`,
  `checkScopeCoverage`,
  `takeShard` and `takeMutantRegex`:
  the host-side listing and the proof that a
  product's scopes hold every mutant.

The adapters are `package/git-policy/cli/bin/mutate-native-container.ts`,
`package/linter/monochromatic-lint/bin/mutate-container.ts` and
`package/linter/monochromatic-lint/bin/mutate-processors.ts`,
and
`package/forbidden-strings/core/bin/mutate-container.ts`.
The scanner keeps its own snapshot flow and uses the exclusion constant.

Tests pin the adapters' outcomes against a stand-in `podman` and `cargo` (`src/golden`);
the campaigns themselves are run by a person,
never by a test.

## Source layout

- `bin/mutation-parallel.ts` and `bin/gate-slot.ts` only read the process state and spawn;
  they hold no logic.
- The claim files:
  `src/process-identity.ts` (process id,
  birth,
  PID namespace),
  `src/claim-record.ts` (the text of a claim
  and the names of its generations),
  `src/claim-files.ts` (the exclusive `link` publication and the atomic release),
  `src/claim-liveness.ts` (whether an owner is alive) and `src/owner-claim.ts` (take a claim,
  read the live owner).
  `src/campaign-lock.ts` uses one claim per evidence directory,
  and `src/gate-slot-run.ts` one claim series per slot.
- `src/legacy-flock.ts` probes,
  and while a claim is held keeps,
  the kernel locks the earlier release took;
  it is the only
  place that runs `flock`,
  and it can be deleted with its callers' calls once no gate or campaign of that release,
  nor any
  checkout of its source,
  can still run.
- `startUnit` (`src/start-units.ts`) starts one unit and reports it;
  a product launcher can use it,
  with `planUnits`,
  to start its own extra units beside the file units
  (the native wrapper starts its planted-controls unit that way).
- `src/launch-units.ts` starts the planned units together and reports which started,
  which failed and the exit status;
  `src/launcher-main.ts` prints only the message of a tooling error.
  The stand-in `systemd-run` of the tests (`src/fake-programs.ts`) fails for the unit named in `FAKE_SYSTEMD_FAIL_UNIT`.
- `src/unit-images.ts` makes or checks every unit's image tag before any unit starts (see "Image tags");
  the stand-in `podman` of the tests (`src/fake-programs.ts`) keeps an image registry in `images.json` of its fixture
  for it.
- `src/` holds the pure builders (unit name,
  log path,
  image tag,
  task arguments,
  shell body,
  `systemd-run` arguments,
  slot arguments) and their option parsers,
  exported through `./ts` for product adapters.
- Tests import the sources directly:
  the package has no build output,
  because its two executables run as TypeScript.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/mutation-test/cli-cargo-mutants:<task>`.

Products give `campaign:parallel` its values in a task of their own,
for example `native:mutation:parallel` in the git-policy package.
