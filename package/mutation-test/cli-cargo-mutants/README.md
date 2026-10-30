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
Each unit runs `mise run <task> -- --timeout <seconds> <scope flag> <scope prefix><file>` through the slot limiter,
with the image tag in the variable named by `--tag-env`, so no two campaigns share an image.
Output goes to `<log dir>/<log prefix>-<label>-<stem>.log.<attempt>`.
The launcher prints one line per unit and a `systemctl --user is-active` command that waits for them.

A unit reruns its campaign, at most `--attempts` times, only while the attempt's log contains the retry marker
(`FAILED   Unmutated baseline`, with three spaces): the baseline run of the unmutated source failed, which a loaded host
can cause.
Any other outcome, including missed mutants, ends the unit after one run.

### Options

Run the script without arguments for the same list with defaults.

- `--task <task>` and `--tag-env <name>` are required: the campaign task and the environment variable through which it
  reads its image tag.
- `--tag-template <text>` builds the tag from `{label}` and `{index}`; `{index}` is required (default `{label}{index}`).
- `--scope-prefix <text>` and `--scope-flag=<flag>` turn each file into the task's scope argument
  (default no prefix and `--file`).
  The scope flag needs the equals sign, because a value that starts with hyphens is read as another option.
- `--setenv <NAME=value>`, repeatable, adds environment to every unit.
- `--timeout <seconds>` is the per-mutant timeout forwarded to the task (default 900).
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

## Campaign building blocks

Every Rust product that runs cargo-mutants in containers uses these through `./ts`, and keeps only an adapter script
that supplies its image names, scopes, limits defaults and manifest:

- `excludedMutantKinds` and `exclusionArguments`: the two mutant kinds no campaign tries, one repository-wide
  constant; a product adds exclusions on top and never removes one.
- `parseContainerLimitOptions` and the argument builders for `--memory`, `--cpus` and `--pids-limit`.
- `parseTimeoutAndExamine`, `parseFileScopes` and `baselineSkipArguments`: the campaign options and their messages.
- `campaignCommand`, `campaignContainerfile`, `evidenceJson` and `exitRecord`: the command, the Containerfile that
  layers cargo-mutants over a tested image, and the evidence files.
- `runCampaign`: the whole container flow (inspect the tested image, build with `--iidfile`, create from the image
  ID with the command given, read the command back, start, copy the report, write the evidence, remove the container).
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

- `campaign:parallel` runs the launcher.
  Products give it their values in a task of their own,
  for example `native:mutation:parallel` in the git-policy package.
- `campaign:gate-slot` runs the limiter.
- `lint`, `lint:types`, `lint:oxlint`, `format:oxlint` and `test:unit` are the usual package checks.
