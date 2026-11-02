# @monochromatic-dev/mutation-test-cli-cargo-mutants

Product-neutral tooling for `cargo-mutants` campaigns on any Rust product in this repository.
It holds the launcher that runs several campaigns at once and the slot limiter that keeps the host from being
overrun by them.
Anything specific to one product (its mise task, its image names, its source layout) is an argument here,
never a constant.

The sibling `package/mutation-test/cli` mutates TypeScript with its own engine and shares nothing with this package.
The inventory of other cargo-mutants scripts and the plan to fold their neutral parts in is
`doc/planning/shared-cargo-mutants-package.md`.

## Launcher

```bash
# mise task: any Rust product, with its own campaign task
mise run //package/mutation-test/cli-cargo-mutants:campaign:parallel -- \
  --task //package/some/cli:mutation:scoped --tag-env SOME_IMAGE_TAG \
  --scope-prefix src/ <label> <file.rs>...
```

The launcher starts one capped `systemd-run --user` unit per file and returns at once.
Each unit runs `mise run <task> -- --timeout <seconds> <scope flag> <scope prefix><file>` through the slot limiter
(plus `--build-timeout <seconds>` and one `--examine <regex>` per pattern when given, see the options),
with the image tag in the variable named by `--tag-env`, so no two campaigns share an image.
Output goes to `<log dir>/<log prefix>-<label>-<stem>.log.<attempt>`.
The launcher prints one line per unit and a `systemctl --user is-active` command that waits for them.

A unit reruns its campaign, at most `--attempts` times, only while the attempt's log contains the retry marker
(`FAILED   Unmutated baseline`, with three spaces): the baseline run of the unmutated source failed, which a loaded host
can cause.
Any other outcome, including missed mutants, ends the unit after one run.

A killed unit (`systemctl --user stop <unit>`), a reboot or an out-of-memory stop does not lose the campaign:
start the same launcher command again and each unit resumes its campaign, scoring only the mutants that no earlier
run scored (see "Resuming a campaign").
A rerun with the same label appends to the same log files `<log prefix>-<label>-<stem>.log.<attempt>`, and a different
label starts new ones; the resume does not depend on the label.

### Options

Run the script without arguments for the same list with defaults.

- `--task <task>` and `--tag-env <name>` are required: the campaign task and the environment variable through which it
  reads its image tag.
- `--tag-template <text>` builds the tag from `{label}` and `{index}`; `{index}` is required (default `{label}{index}`).
- `--scope-prefix <text>` and `--scope-flag=<flag>` turn each file into the task's scope argument
  (default no prefix and `--file`).
  The scope flag needs the equals sign, because a value that starts with hyphens is read as another option.
- `--setenv <NAME=value>`, repeatable, adds environment to every unit.
- `--timeout <seconds>` is the per-mutant test timeout forwarded to the task (default 900).
- `--build-timeout <seconds>` is the per-mutant build timeout, cargo-mutants' own `--build-timeout` ("maximum run time
  for cargo build command"), forwarded to the task only when given; otherwise the task's default applies
  (`defaultBuildTimeoutSeconds`, 300 seconds, in `src/campaign-files.ts`).
  A `TIMEOUT ... in 300s build` line of a campaign log means the build of that one mutant used the whole build timeout,
  which a heavily loaded host can cause; rerun that mutant with a larger `--build-timeout`.
  Each product's scoped task accepts it up to its maximum (3600 seconds unless the product names another).
- `--examine <regex>`, repeatable, is a mutant-name pattern forwarded to every unit's task, so only the mutants whose
  names match are tried in every listed file; the pattern may hold the plain characters the scoped task accepts
  (names, `|`, `:`, `.`, `*`, `?`, parentheses, backslash, space and hyphen) and is checked here by the same function.
  It reruns one named mutant, for example a timed-out one:
  `-- --timeout 2400 --build-timeout 1200 --examine read_regular_file m047 private_storage.rs`.
- `--memory <size>` and `--cpus <count>` are the container limits.
  They are forwarded to the task only when given; otherwise nothing is forwarded and the task's defaults apply.
- `--unit-memory-max <value>` and `--unit-cpu-quota <value>` cap the whole systemd unit (the task, the container client
  and the build), not the container; they are separate from `--memory` and `--cpus` (defaults `12G` and `500%`).
- `--unit-prefix`, `--log-prefix`, `--log-dir`, `--tmp-dir` and `--root` place names and files;
  the working tree comes from `--root`, then `MUT_ROOT`, then the checkout that holds this package.
  Logs and `TMPDIR` default under `<home>/temp/agent`.
- `--attempts <count>` and `--retry-marker <text>` tune the rerun rule.
- `--slot-count <count>` and `--lock-dir <dir>` (or `GATE_SLOT_COUNT` and `GATE_SLOT_DIRECTORY`) are forwarded to the
  limiter.

Every file name reaches the shell body as one single-quoted word, so spaces, quotes, `$`, `;`, backticks and newlines
stay literal.
Two files that collapse to one unit name make the launch fail before anything starts.

## Slot limiter

```bash
mise run //package/mutation-test/cli-cargo-mutants:campaign:gate-slot -- mise run //package/some/cli:test:container
```

The limiter tries `slot-1.lock` to `slot-N.lock` under the lock directory with `flock --nonblock`, runs the command
inside the first free slot at nice 19, and sleeps 15 seconds before trying again when every slot is held.
The command inherits stdio and its exit status becomes the limiter's.
The default is six slots in `<home>/temp/agent/gate-slots`.
Every launcher on the host must share that directory for the limit to hold, so change it only on purpose.
The limit exists because the host runs under a shared CPU cap; the slot count follows the user's CPU hold,
documented at `defaultSlotCount` in `src/constants.ts`.

## Resuming a campaign

Every campaign run through `runCampaign` is resumable.
A kill, a reboot, an out-of-memory stop or a stopped unit loses no scored mutant, and running the same command again
continues where the campaign stopped.
Nothing needs to be passed to resume: run the same task with the same options on the same tested image.

### Where the outcomes live

The campaign's evidence directory is `<evidence root>/<prefix><key>`, for example
`target/verification/native-mutation-<key>/`.
The key is the first twelve hexadecimal digits of a SHA-256 over the tested image ID and the exact command, so the
same campaign finds the same directory, and another snapshot, scope or option starts a clean one
(`campaign.json` holds the full identity and refuses a directory that belongs to another).
Each run of the container is an attempt with its own `attempt-NNN/` directory inside it, bind-mounted at the
container's report path.
cargo-mutants appends every scored mutant to `caught.txt`, `missed.txt`, `timeout.txt` or `unviable.txt` as it goes,
so those lines are on the host the moment they are written and survive anything that stops the container.
The container runs with `--userns=keep-id`, so the files belong to the host user, and the mount carries the `:z`
label because the host runs SELinux.

### How a rerun skips scored mutants

The runner reads the lists of every earlier attempt and writes all four outcomes into the new attempt's
`mutants.out/previously_caught.txt`; the command always carries cargo-mutants' `--iterate`, which reads that file
before it rotates `mutants.out` and leaves out every mutant named there.
`--iterate` alone would not be enough: cargo-mutants 27.1.0 documents (`book/src/iterate.md`) and implements
(`load_previously_caught` in `src/output.rs`) that it skips only mutants that were caught or unviable, so a missed or
timed-out mutant would be scored again, and the single `mutants.out` it keeps is rotated to `mutants.out.old` and
deleted by the next run, so a second kill would lose the first attempt's missed list.
Seeding the missed and timed-out names into the file `--iterate` already reads, and keeping one directory per
attempt, gives "never rescore a scored mutant" with only what cargo-mutants supports.

### What a finished campaign leaves

`<evidence>/mutants.out/` holds `caught.txt`, `missed.txt`, `timeout.txt` and `unviable.txt` as the union over all
attempts, an `outcomes.json` whose counts come from those lists, and the `log`, `diff` and `mutants.json` copied from
the attempts, as before.
`exit.json` carries the status of the whole campaign (a timeout in an earlier attempt still ends the campaign with
status 3 although the last attempt, handed nothing to score, exits 0), plus `attempt` and `skippedMutants`.

### Single writer

`campaign.lock` names the running process by process number, boot ID and start time, so a second run of the same
campaign is refused while the first lives, and the lock of a killed run or a rebooted host is taken over.
A container left by a killed run keeps its name (`cargo-mutants-<key>`) and is removed before the next one is
created.
On `SIGTERM`, `SIGINT` or `SIGHUP` the runner stops the container (a rootless container is not in the unit's cgroup,
so stopping the unit would not), composes the partial report and fails.

### Planted controls

The planted controls that a product runs before the build are skipped on a resume, because an earlier attempt scored
mutants only after they passed.

### Starting over

Delete the campaign's directory under the evidence root (its files belong to the host user).
A source change gives a new tested image ID and so a new directory without any deletion.

### Limits

A mutant that was in flight when the container died is not in any list and is scored again.
The baseline run of the unmutated source repeats on every attempt, because it also checks the environment after a
reboot.
Mutants match by file, line, column and description, as in cargo-mutants' own `--iterate`.

The tests drive the whole flow with stand-in `podman` and cargo-mutants programs (`src/fake-programs.ts`,
`src/fake-mutants-simulation.ts`): `src/campaign-resume.unit.test.ts` covers a fresh campaign, a campaign killed
partway and run again, and a finished campaign run again.

## Campaign building blocks

Every Rust product that runs cargo-mutants in containers uses these through `./ts`, and keeps only an adapter script
that supplies its image names, scopes, limits defaults and manifest:

- `excludedMutantKinds` and `exclusionArguments`: the two mutant kinds no campaign tries, one repository-wide
  constant; a product adds exclusions on top and never removes one.
- `parseContainerLimitOptions` and the argument builders for `--memory`, `--cpus` and `--pids-limit`.
- `parseTimeoutAndExamine`, `examinePatternOf`, `parseFileScopes` and `baselineSkipArguments`: the campaign options
  (`--timeout`, `--build-timeout`, `--examine`, `--file`) and their messages.
  A product passes the parsed `buildTimeoutSeconds` to `campaignCommand`, which hands it to cargo-mutants.
- `campaignFailureMessage`: the failure text of a campaign container; the timeout exit status names both timeout
  options, because cargo-mutants gives a build timeout and a test timeout the same status.
- `campaignCommand`, `campaignContainerfile`, `evidenceJson` and `exitRecord`: the command, the Containerfile that
  layers cargo-mutants over a tested image, and the evidence files.
- `runCampaign`: the whole container flow (inspect the tested image, find or open the campaign's evidence directory and
  its attempt, build with `--iidfile`, create from the image ID with the command given and the attempt's report
  directory bind-mounted, read the command back, start, compose the report of all attempts, write the evidence, remove
  the container); see "Resuming a campaign".
- `campaignKey`, `prepareAttempt`, `readScoredNames`, `seedScoredNames`, `composeReport` and `cumulativeStatus`: the
  resume bookkeeping that `runCampaign` uses, exported for adapters that report on a campaign's directory.
- `listMutants`, `checkScopeCoverage`, `takeShard` and `takeMutantRegex`: the host-side listing and the proof that a
  product's scopes hold every mutant.

The adapters are `package/git-policy/cli/bin/mutate-native-container.ts`,
`package/linter/monochromatic-lint/bin/mutate-container.ts` and `bin/mutate-processors.ts`, and
`package/forbidden-strings/core/bin/mutate-container.ts`.
The scanner keeps its own snapshot flow and uses the exclusion constant.

Tests pin the adapters' outcomes against a stand-in `podman` and `cargo` (`src/golden`);
the campaigns themselves are run by a person, never by a test.

## Source layout

- `bin/mutation-parallel.ts` and `bin/gate-slot.ts` only read the process state and spawn; they hold no logic.
- `src/` holds the pure builders (unit name, log path, image tag, task arguments, shell body, `systemd-run` arguments,
  slot arguments) and their option parsers, exported through `./ts` for product adapters.
- Tests import the sources directly: the package has no build output, because its two executables run as TypeScript.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/mutation-test/cli-cargo-mutants:<task>`.

Products give `campaign:parallel` its values in a task of their own,
for example `native:mutation:parallel` in the git-policy package.
