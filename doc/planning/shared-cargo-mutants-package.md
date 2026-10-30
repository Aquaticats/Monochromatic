# Shared cargo-mutants package

Status:
 steps 1 to 6 executed 2026-10-29 on a branch awaiting landing, after the user answered the open questions
 (see `Decisions`) and the container-limit work in `package/git-policy/cli/bin` landed.
Step 7 is not started, by design.
`Execution record` says what each step did, what changed on purpose, and what has not been run.

## Purpose

Several Rust products run `cargo-mutants` campaigns through scripts that were written one product at a time.
The user's direction is that all product-neutral cargo-mutants tooling lives in one workspace package,
and that each product keeps only a thin adapter that supplies its names, scopes, and defaults.
The launcher and the slot limiter that began as scratch scripts are the first content of that package.

## The package

The package is `package/mutation-test/cli-cargo-mutants`, named `@monochromatic-dev/mutation-test-cli-cargo-mutants`.
The directory follows the repository rules:
 it sits in the existing `mutation-test` category beside `cli`, `cli.fixture` and `cli-unused-export`,
 uses the `cli-<what>` naming those siblings use,
 and has singular segments and a name that is the path under `package/` with `/` changed to `-`.
It is not a sidecar (`<pkg>.<kind>`), because it has no single subject package;
 it serves every Rust product.
It is separate from `package/mutation-test/cli`, which mutates TypeScript with its own engine and shares no code.

What it holds now:
- the parallel launcher `bin/mutation-parallel.ts`, one capped systemd user unit per source file;
- the slot limiter `bin/gate-slot.ts`, a host-wide cap on concurrent campaign containers;
- pure builders in `src/` for the unit name, log path, image tag, task arguments, retry-loop shell body,
  `systemd-run` arguments and slot arguments, each tested, with shell quoting at the final interpolation.
- the campaign building blocks every product shares: the excluded mutant kinds, the container limit options, the
  timeout, examine, scope and baseline-skip parsing, the campaign command, Containerfile and evidence records, the
  container flow (`runCampaign`), the mutant listing and coverage proof, and the shard and pattern options.

Task name, image-tag variable and template, scope prefix and flag, extra environment, container `--memory` and `--cpus`,
unit `--unit-memory-max` and `--unit-cpu-quota`, attempts, retry marker, locations and slot settings are all options.

## Inventory

The inventory comes from searching `package/`, the repository root files and `doc/troubleshooting/` for
`cargo.mutants`, `cargo-mutants`, `mutants.out`, `mutate-native` and `native:mutation`,
then reading the hits that hold code or configuration.
Documents that only mention the tool (handovers, audits, decisions) are not part of the inventory.
Line counts were measured with `wc --lines` on 2026-10-29.

### git-policy native wrapper

`package/git-policy/cli/bin/mutate-native-container.ts` (411 lines) runs a scoped campaign in bounded containers.
- Product-neutral:
  the campaign flow (inspect the tested image ID, copy the installed `cargo-mutants` in, hash it into the manifest,
  layer a campaign image with `FROM <image id>`, `COPY`, `RUN cargo mutants --version`, `ENV CARGO_NET_OFFLINE`, `CMD`,
  build, create, start attached, copy the report out, write `manifest.json` and `exit.json`),
  the `mutationCommand` argument list,
  the `isContentAddressedId` check,
  the scratch directory convention under `node_modules/.monochromatic`,
  and the `excludedMutantKinds` constant with its 2026-10-05 rationale.
- Product-specific:
  image names `localhost/git-policy-native-test` and `-mutation`,
  the tag variable `GIT_POLICY_NATIVE_IMAGE_TAG`,
  the `NATIVE_MUTATION_SKIP_PLANTED` switch,
  the evidence directory under the package `target/`,
  and the default limits (which the in-flight work makes settable per task).

`package/git-policy/cli/bin/native-mutation-options.ts` (260 lines) parses and validates the wrapper's options.
- Product-neutral:
  `usesOnly`, `isPositiveWholeNumber`, `timeoutOf` bounds, the `--examine` character set,
  the `--timeout` and `--examine` split in `parseTimeout`,
  and the `baselineSkipArguments` mechanism that appends `-- -- --skip <test path>` for nextest.
- Product-specific:
  the `src/native/` prefix and `.rs` suffix check in `isNativeSourceGlob`,
  and the environment variable name `NATIVE_MUTATION_BASELINE_SKIP`.
  Both become arguments.

`package/git-policy/cli/bin/native-planted-controls.ts` plus `-commit.ts`, `-entry.ts` and `-landing.ts`
(857 lines together) plant known guard removals and require a named control to fail.
- Product-neutral:
  the mechanism (copy the snapshot, apply a `from`/`to` text replacement, run the control, require the failure),
  the `PlantedControl` and `PlantedControlResult` shapes, and the evidence it writes.
- Product-specific:
  every control definition, the container source path `/work/package/git-policy/cli`, and the failing test names.
  The three definition files stay in the product; the runner moves.

`package/git-policy/cli/bin/native-verification-process.ts` holds `runCommand`, `createContainer` and
`NativeVerificationError`.
These are generic Podman process helpers.
Whether a shared container helper already exists elsewhere in the repository has to be checked before moving them
(the extension rule: extend an owner that exists);
 the linter and scanner scripts below carry their own copies of the same helper.

`package/git-policy/cli/bin/test-native-container.ts` builds the tested image that the campaign layers on.
It is the gate, not cargo-mutants tooling, and stays in the product.
Its image-tag scheme and bounded limits are the same ones the campaign reuses, so the shared runner takes them as input.

`package/git-policy/cli/mise.toml` tasks `native:mutation`, `native:mutation:scoped` and `native:mutation:list`
 are product tasks and stay as thin adapters.
`native:mutation:list` runs `cargo mutants --list --no-config` with no container and no build.
It passes no exclusion patterns, so it does not exercise the argument builders;
 it only lists the mutants that exist.

### Linter

`package/linter/monochromatic-lint/bin/mutate-container.mjs` (709 lines) runs scoped campaigns.
- Product-neutral:
  the campaign flow (the same as in git-policy, written separately),
  `exclusionArguments`, `skipArguments`, `fileArguments`,
  `listMutants` (host-side `cargo mutants --list`),
  `coverage` (proves that the union of the scope listings equals the unscoped listing, name by name),
  the `--shard k/n` parser,
  the `--re` mutant-name filter,
  the explanation of how libtest arguments reach nextest (a second `--` for `--skip`),
  and the `excludedMutantPatterns` constant, a second copy of the exclusion list.
- Product-specific:
  the scope table (`--rust-style`, `--markdown`, `--processors` and the others),
  the `workspaceTests` list of tests the fast scopes skip,
  the image names `monochromatic-lint-*`,
  the tag variable `MONOCHROMATIC_LINT_IMAGE_TAG`,
  and the per-scope timeouts.
- Defect noted:
  the file is `.mjs`, which the scripts rule forbids.

`package/linter/monochromatic-lint/mise.toml` has an inline `mutation:processors:files` task of about fifty lines
that re-implements the campaign flow a third time,
 with a fourth copy of the exclusion patterns,
 a hard-coded outcome check (`total_mutants` and `caught` both five),
 and `mkdtemp(join(tmpdir(), ...))`, which uses the system temporary directory that the scratch rule bans.
- Product-neutral:
  the flow and the check that reads `mutants.out/outcomes.json`.
- Product-specific:
  the filter, the file list and the expected count of five.
The task `mutation:list:processors` repeats the exclusion patterns on its command line as a fifth copy.

### Forbidden-strings scanner

`package/forbidden-strings/cli/bin/mutate-container.mjs` (212 lines) runs the scanner's campaigns in a snapshot.
- Product-neutral:
  the campaign flow and `listMutants`, which is the same shape as the linter's, and a copy of the exclusion patterns.
- Product-specific:
  the `SCOPES` table (`--startup`, `--pathname`) and the consumer suite it keeps.
- Defect noted:
  `.mjs`.

`package/forbidden-strings/cli/bin/container-snapshot.mjs` holds `run`, `inventory` and `snapshot`,
 the offline mount-free source snapshot.
The snapshot and the process helper are generic;
 the input file set is the scanner's.

`package/forbidden-strings/cli/bin/verify-evidence.mjs` (317 lines) compares retained manifests with the current
compiled inputs and summarizes terminal evidence.
- Product-neutral:
  the hash comparison and the summary.
- Product-specific:
  `currentInputs`, which names the scanner and engine directories.
Classified product-specific for now,
 because only the scanner uses it and the neutral part is small.

`package/forbidden-strings/core/mutants.Containerfile`, the `test:mutation` and `test:mutation:image` tasks in
`package/forbidden-strings/core/mise.toml`, and `package/forbidden-strings/core/.cargo/mutants.toml`
 form a different design.
The task mounts the crate read-write into a container that bakes `cargo install cargo-mutants`,
 runs with `--jobs` scaled to the host,
 and keeps its own memory arithmetic.
That conflicts with the mount-free snapshot design that git-policy, the linter and the scanner converged on,
 and it does not use the slot limiter.
- Product-neutral:
  the idea of one baked tool image, and the exit-status handling.
- Product-specific:
  the equivalent-mutant exclusions with their proofs in `.cargo/mutants.toml`.
  They stay with the crate.
Whether this task migrates to the snapshot design is a decision for the user, listed under open questions.

### Other Rust packages

`package/jsonc-edit/core-rust/.cargo/mutants.toml` holds equivalent-mutant exclusions with proofs anchored on line and
 column.
It has no runner of its own.
It is product-specific configuration and stays.
No other package contains a cargo-mutants script, task or configuration;
 the search found none under `package/` besides the ones named here.

### Repository-level files

`.gitignore` ignores `mutants.out/` and `mutants.out.old/`.
That is neutral and stays where it is.
`doc/troubleshooting/cargo-mutants-timeout-exit-status.md` and `.patch` explain why one timed-out mutant fails a whole
 campaign with status 3.
They are neutral and stay in `doc/`;
 the package README links to them from the section that documents the exclusion constant once it moves.

## Migration plan

Each step is one commit, small enough to review, and touches one concern.
No step runs a campaign from an agent session;
 the user runs the single scoped campaign that closes a runner step, through the slot limiter.

### Step 0, done

The package holds the launcher, the slot limiter and their tests.
The git-policy values the launcher needs began as a preset task in the package and moved into the git-policy package
 as `native:mutation:parallel` in step 6.

### Step 1, after the container-limit work lands

Move `excludedMutantKinds` and its rationale into the package as one constant with a `exclusionArguments` builder,
 export it through `./ts`,
 and replace the five copies (git-policy bin, linter bin, linter inline task, linter list task, scanner bin).
It is the smallest step, it is pure, and it removes the most duplication.
Check: a unit test that the shared builder returns exactly the argument arrays the five copies build today,
 and the linter `mutation:list:processors` output (the one listing task that passes the exclusions)
 is byte-identical before and after.

### Step 2

Move the option parser (`usesOnly`, timeout bounds, examine character set, `--timeout` and `--examine` split,
 scope validation, `baselineSkipArguments`) with `scopePrefix`, `scopeSuffix` and `skipEnvironmentVariable`
 as arguments.
The package already has `usesOnly` and `isPositiveWholeNumber` in `src/character-set.ts`;
 the git-policy copy is deleted rather than extended.
The container `--memory` and `--cpus` options added by the in-flight work move here too,
 so the launcher's forwarded options and the wrapper's accepted options come from one definition.
Check: the existing git-policy option tests, ported to the package, plus `lint:types` in both.

### Step 3

Move the pure parts of the campaign: `mutationCommand`, the Containerfile text writer, the manifest and `exit.json`
 shapes, `isContentAddressedId`.
Check: unit tests of the generated Containerfile and command for the three products' current inputs,
 compared with the text the existing scripts write.

### Step 4

Move the container flow into one `runCampaign` function that takes the tested image name, campaign image name,
 tag variable, limits, command, evidence root and report path.
First check whether an existing shared Podman helper can be extended instead of copying `runCommand` and
 `createContainer`.
Close the step with one scoped campaign over one small file per product, run by the user.

### Step 5

Move the list and coverage tools (`listMutants`, `coverage`, `--shard`, `--re`), parameterized by the scope table.
Check: `mutation:coverage` of the linter still reports no mutant outside the scopes.

### Step 6

Reduce each product to an adapter of a few dozen lines that supplies names, tag variable, limits, scope table and
 mise task, and convert the `.mjs` scripts to `.ts` in the same change.
Move the git-policy preset task from the package into the git-policy `mise.toml`.
Keep every existing task name so commands, handovers and running scratch scripts keep working.
Replace the linter inline task by a call to the shared runner, which also removes its use of the system temporary
 directory.

### Step 7, only if the user asks for it

Move the planted-controls runner (the mechanism, not the definitions) once a second product wants it.
Today only git-policy uses it, so extracting it earlier is speculative.

## Risks

- A runner step changes a script that other sessions are running right now.
  Mitigation: land runner steps only when no campaign is running, and keep the old task names.
- The shared campaign flow could drift from one product's needs, for example the linter's `--jobserver-tasks` and
  `--caught` options.
  Mitigation: the command builder takes extra arguments verbatim from the adapter.
- The forbidden-strings core task uses a different design;
  forcing it into the shared flow without a decision would change its behavior.

## Decisions

The user decided on 2026-10-29:

- The forbidden-strings core `test:mutation` task migrates to the shared mount-free snapshot design and the slot
  limiter.
- The package keeps the name `@monochromatic-dev/mutation-test-cli-cargo-mutants` at
  `package/mutation-test/cli-cargo-mutants`.
- The excluded mutant kinds stay one repository-wide constant owned by the shared package
  (`src/excluded-mutants.ts`); a product may add exclusions on top and never removes one.

## Execution record

Every step was checked against outcomes pinned before any product script changed.
The pins run each product script against a stand-in `podman` and `cargo` (`src/fake-podman-fixture.ts`) and record
 the podman calls, the Containerfile, the manifest and exit records, the output, and the message of every rejected
 option, for a spread of options per product (`src/golden-scenarios.ts`, outcomes in `src/golden`).
A pin was shown to fail once per step by changing the shared code it covers.
No container, systemd unit or mutation campaign was run by the agent.

### Steps 1 to 3

The excluded kinds, the container limit options (moved with their tests), the timeout, examine, scope and baseline-skip
 parsing, the campaign command, the Containerfile text, and the evidence records are in the shared package.
The outcomes pinned before the steps are unchanged, except for the changes below.

### Step 4

`runCampaign` and `runPodman` run the git-policy native wrapper and the linter campaigns.
The linter's flow won: the campaign image is built with `--iidfile`, the container is created from that image ID with
 the command given explicitly, and the command is read back from the created container.
The native wrapper used to create its container from the shared tag and to rely on the image's default command.

### Step 5

`listMutants`, `checkScopeCoverage`, `coverageSummary`, `takeShard` and `takeMutantRegex` are in the shared
 package; the coverage listings still run one after another.

### Step 6

The linter and scanner mutation scripts are TypeScript (`bin/mutate-container.ts`; the linter's scope table is
 `bin/mutation-scopes.ts`).
The linter's inline processors task is `bin/mutate-processors.ts` on the shared runner.
The engine's `test:mutation` runs the shared flow over a snapshot image built by `bin/build-test-image.ts` from
 `test.Containerfile`; its `.cargo/mutants.toml` is untouched and a test pins that none of its fifteen exclusions
 is dropped.
The git-policy parallel task is `native:mutation:parallel` in that package.

### Intentional changes

- Native wrapper: the iidfile build, the explicit create command and the read-back (step 4), so its podman calls
  change; an invalid `NATIVE_MUTATION_BASELINE_SKIP` now fails before any podman call or evidence directory.
- Linter script: accepts `--memory`, `--cpus` and `--pids-limit` (defaults unchanged), also after a scope switch,
  because the limit parser accepts valueless flags.
- Linter processors task: shared Containerfile text, command option order, iidfile build, explicit create command,
  limit options, and scratch under the package instead of the system temporary directory.
- Engine: `test:mutation` no longer mounts the crate, takes the shared options instead of passing extra flags to
  cargo-mutants, no longer takes `--jobs`, and its base image gains `cargo-nextest`.
- Several packages gain a `package.json` (linter, scanner, engine) so they can depend on the shared package.

### Not done and why

- The scanner keeps its own snapshot flow: it bakes its sources into the image that runs the campaign instead of
  layering over a tested image, so it cannot use `runCampaign`; it uses the shared exclusion constant and is
  TypeScript now.
  Its older `.mjs` scripts (snapshot, evidence, guards, documents) are untouched and are not part of this plan.
- The planted-controls runner (step 7) stays in the native wrapper until a second product wants it.
- Nothing past an option error or a stand-in podman has run: the user runs one scoped campaign per product to prove
  parity, and the engine's snapshot image build and campaign are new and have never run against real Podman.
- The package-wide Oxlint run of the linter and scanner reports findings in their older `.mjs` scripts, which this
  work does not touch; the new TypeScript files are linted by path with `lint:scripts:oxlint`.
