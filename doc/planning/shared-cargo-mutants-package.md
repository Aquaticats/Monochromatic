# Shared cargo-mutants package

Status:
 proposed 2026-10-29, not accepted.
The package exists and holds the launcher and the slot limiter;
the migration below is sequenced after the container-limit work in `package/git-policy/cli/bin` lands.

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

The package holds the launcher, the slot limiter, their tests, and the `campaign:parallel:git-policy-native` task that
 supplies the git-policy values.
That task is the temporary home of the git-policy adapter values and moves into the git-policy package in step 6.

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

## Open questions

- Should the forbidden-strings core `test:mutation` task migrate to the mount-free snapshot design and the slot
  limiter, or stay as it is?
  Ranking if it migrates: it removes the only host-mounted mutation run and the separate memory arithmetic,
  but it costs an image rework and loses the `--jobs` parallelism inside one container.
- Is the package name acceptable, or should it take a `core` role name (`cli-cargo-mutants` is a CLI plus a library
  surface exported through `./ts`)?
- Should the exclusion list stay a repository-wide constant, or should each product be able to add to it?
  The user accepted the two exclusions on 2026-10-05 for every runner, which favors one constant.
