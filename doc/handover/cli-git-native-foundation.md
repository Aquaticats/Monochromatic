# cli-git native foundation

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/git/executable` is now `package/git-executable/core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/jsonc-edit` is now `package/jsonc-edit/core-rust`.

## Authority and scope

This records a scoped delegation of the Rust cli-git rewrite:
configuration,
real-Git resolution and forwarding,
the thin executable,
diagnostics and the management-command skeleton,
a mutation runner,
and a fuzz sidecar.
The accepted approach is
[`cli-git-rust-implementation.md`](../planning/cli-git-rust-implementation.md);
the behavior inventory is
[`cli-git-rust-behavior-ledger.md`](../planning/cli-git-rust-behavior-ledger.md).
Transactions,
hooks,
locks,
replay,
recovery,
worktree copy,
auto-push,
the Git command parser,
and the policy rule cores are outside this delegation.

The native executable is a development artifact.
It is not installed,
does not shadow `git`,
and stops every repository-changing command.
Nothing here brings the wrapper close to cutover.

## Gate results

The gate is `mise run //package/git-policy/cli:native:test:container`:
a mount-free,
network-disabled container bounded to 2 GiB,
2 CPUs,
and 128 PIDs,
built on the audited Git 2.56.0 image
`6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`.
It runs `cargo test --offline --locked --all-targets`,
then Clippy with warnings denied.

### Slice 1, configuration

Commit `df25471a9`.
The gate passed 51 unit tests and Clippy before the commit
(9 tests existed before this delegation).
The run's evidence directory was not retained by name;
the later slice 2 run covers the same sources.

### Registry default correction

Commit `72056d25d`.
Verified together with slice 2 in the run recorded there;
it was not gated alone.

### Slice 2, real Git and forwarding

Commit `414ab5ff7`.
The gate passed 114 unit tests,
16 binary-level tests,
and Clippy.
Test image
`606ad8d2e58639ca1729aa18cdc5f77223d389eb1f4150b87620e27f3a4d9d77`;
evidence `package/git-policy/cli/target/verification/native-SyV2Pq`.

### Slice 3, diagnostics and management commands

Commit `5b25df929`.
The gate passed 134 unit tests,
21 binary-level tests,
and Clippy.
Test image
`515ceb783017f1184decb7c12c34ab838a30e1a7a17d5507f22ff3a9730e31c4`;
evidence `package/git-policy/cli/target/verification/native-HpTEoj`.

### Test file split

Commit `1bc84492a` moved controls out of four test files that exceeded 300 code lines.
No control changed:
the gate passed the same 134 unit tests and 21 binary-level tests,
and Clippy.
Test image
`9424eaeacf326c05863fe00e5bb87bcac3bd08ac452fc68646bd9c865b70b0c8`;
evidence `package/git-policy/cli/target/verification/native-YF1758`.

### Controls added after the first mutation campaign

Commit `6fb148913`.
The gate passed 144 unit tests,
21 binary-level tests,
and Clippy.
Test image
`ef4b26e1b68250ee7f3f628b018cd5b7d1867c1af5b6b38e0acfc585ef53ffc7`;
evidence `package/git-policy/cli/target/verification/native-zJK04f`.

### Registry lookup and fixture writer

Commit `cba0702fe` replaced an indexed loop in `policy_descriptor` with a `for` loop.
The gate passed 144 unit tests,
21 binary-level tests,
and Clippy.
Test image
`11f544e29a261417dd283a9e4ebac0988acf4c1118dc43486c0c063213da472c`;
evidence `package/git-policy/cli/target/verification/native-1Kehi5`.

Commit `de15ea3ea` changed how tests write the executables they run;
"Intermittent test failure" gives the reason.
The gate passed the same 144 unit tests and 21 binary-level tests,
and Clippy.
Test image
`15bd4395ea94c2f2cde8cc7957baf037ee7d0c7054c91633ff318172b8bd1a8a`;
evidence `package/git-policy/cli/target/verification/native-8EIyYT`.

### Final tree

The gate was run again on commit `f15351ad5`,
whose wrapper sources equal those of `de15ea3ea`:
144 unit tests,
21 binary-level tests,
and Clippy passed.
The test image ID,
`15bd4395ea94c2f2cde8cc7957baf037ee7d0c7054c91633ff318172b8bd1a8a`,
is the one built for `de15ea3ea`,
which confirms the inputs are identical.
Evidence `package/git-policy/cli/target/verification/native-vWfYMM`.

### Script inspection bound

Commit `57eee2061` adds one control;
these are the final wrapper sources of this delegation.
The gate passed 145 unit tests,
21 binary-level tests,
and Clippy.
Test image
`16f09e939ecefa1a500520d1b4ed0d9745e1f373305101cbb02743dcb1fc212a`;
evidence `package/git-policy/cli/target/verification/native-dtC4QT`.

### Runners under the Oxlint configuration

Commits `6bc2f15f3` and `9b99e49f3` rewrote `bin/test-native-container.mjs` and `bin/mutate-native-container.mjs`
so that they pass the repository Oxlint configuration with no errors and no warnings.
`bin/native-verification-process.mjs` now holds the shared `runCommand`,
`createContainer`,
and `NativeVerificationError`;
`bin/native-planted-controls.mjs` holds the planted-control phase,
split out because the mutation runner otherwise exceeds the 300-line `max-lines` limit.
Commands,
argument arrays,
container bounds,
the `GIT_POLICY_NATIVE_IMAGE_TAG` override,
and evidence files are unchanged.

- `spawnSync` became `spawn`.
  Captured output keeps a bound per stream that stops the command and fails the step.
  The bound is now 64 MiB for both runners;
  the test runner's `maxBuffer` was 16 MiB,
  which its captured commands (`podman image inspect` and `rustc --print sysroot`) never approach.
- `try`/`finally` became `await using` over `mkdtempDisposable` and over a container handle
  whose disposal runs `podman rm --force`,
  in the same order as before:
  the container first,
  then the build context.
- The image-ID,
  container-ID,
  and `--file` scope regexes became linear scans.
  A scratch comparison against the former regexes over 400017 generated inputs found no difference.
- The test runner copies its source snapshot concurrently
  and waits for every copy to settle before it can throw,
  so cleanup never races an in-flight copy.
- Planted controls still run one container at a time,
  through an async generator consumed with `for await`.
- Each file carries `/// <reference types="node" />`.
  `bin` is outside the package `tsconfig.json` include list,
  so type-aware Oxlint checks these files in an inferred program without Node types;
  without the reference,
  `process` and every `node:` import resolve to the error type.

The gate on the former scripts and on the rewrite,
over the same source fingerprint,
both passed 322 unit tests,
21 binary-level tests,
and Clippy,
and both built test image `e357067e614b11aa4eaaf8eb2d6f8ef519828a2325d8d99d751dc836a69b3cc5`.
Evidence `package/git-policy/cli/target/verification/native-NU20Ff` (former)
and `package/git-policy/cli/target/verification/native-lZbioV` (rewrite);
their `manifest.json` and `passed.json` are byte-identical.

Failure controls,
all leaving no temporary directory or container behind:

- A failing test planted in a scratch copy of the crate made the test runner exit 1
  with `podman run failed (status 101, signal null)` and no `passed.json`.
- A missing image tag made the mutation runner exit 1
  naming `podman image` and Podman's `image not known`.
- `createContainer` removed its container when its scope ended by a thrown error and by a normal return.

`native:mutation:scoped -- --file src/native/rule_commit_only_message.rs` on the rewrite's gate image
noticed all five planted controls,
one container at a time,
then caught both mutants of that file and exited 0.
Evidence `package/git-policy/cli/target/verification/native-mutation-LISN18`
has the same files and report entries as `native-mutation-2mwlb7` from the former runner,
the same key order in `manifest.json` and `exit.json`,
and a `planted-controls.json` equal to the former one;
the campaign container and every planted container were removed.

Evidence directories live under the ignored `target` directory of this worktree and are not committed.

## Configuration

### Shape

`cli-git.config.jsonc` at the worktree top level is one JSONC object.
It is read through `package/rust-module/jsonc-edit`
and is the closest direct translation of the TypeScript `policies` map without plugins and trust.

```jsonc
// cli-git.config.jsonc
{
  "policies": {
    "markdown/autofix": ["warn", { "rules": ["lfs-image-url"], "exclude": ["package/ssg/"] }],
    "mono/forbidden-root-context": "error",
    "mono/dependent-version-bump": "error",
    "security/forbidden-strings": ["error", { "builtinRules": true }],
  },
  "hooks": { "concurrentCommits": false },
  "indexLock": { "unprovenOwnerTimeoutMs": 1000 },
  "landing": { "reserveAfterLostRaces": 1 },
}
```

The `policies` block shown is the translation of this repository's `cli-git.config.ts`
under the rule the human decided on 2026-10-05
(see "Defaults with and without a configuration file"):
it names all four optional policies,
including `mono/dependent-version-bump`,
which the TypeScript configuration never listed and the incumbent ran at `error` anyway.
The three concurrency sections show their defaults and may be omitted.
`config_parse_acceptance_tests.rs` pins both sides:
`repository_translation_names_all_four_optional_policies` loads that block,
and `literal_translation_silently_stops_the_unlisted_policy` shows that a word-for-word translation,
which names only three,
leaves `mono/dependent-version-bump` off.

A setting is a severity (`"off"`,
`"warn"`,
`"error"`)
or `[severity, options]` for the two policies that have options.
Unknown keys,
unknown policy IDs,
keys defined twice (also through JSON escapes),
`null`,
and wrong types are errors naming the key path,
for example `policies.markdown/autofix[1].rules[0]`.
The retired keys `plugins`,
`trust`,
`executable`,
and `command` get their own explanation.
Nothing in the file is executed,
imported,
or resolved as a program path.

### Registry

`policy_registry.rs` holds the shipped policies in execution order:
`require-root`,
`linked-worktree-only`,
`branch-worktree-only`,
`add-explicit`,
`final-newline`,
`markdown/autofix`,
`mono/forbidden-root-context`,
`mono/dependent-version-bump`,
`security/forbidden-strings`.
Each default severity equals the incumbent definition's `defaultSeverity`,
pinned by `every_identity_resolves_to_its_declared_row`:
`final-newline` and `markdown/autofix` are `warn`,
every other policy is `error`.

### Configuration choices open to veto

#### Policy names

The four formerly plugin-provided policies keep this repository's namespaced IDs
(`markdown/`,
`mono/`,
`security/`).
Those are the IDs in today's `policies` map and in emitted JSONL `policyId` fields.
Alternatives are the plugin package names (`markdown-lint/autofix`,
`repository/...`,
`forbidden-strings/forbidden-strings`)
or flat names.

#### Defaults with and without a configuration file

Decided by the human on 2026-10-05,
no longer open to veto:
optional policies run only when listed.
The five built-ins run everywhere at their default severity.
Each of the four policies that plugins used to provide
(`markdown/autofix`,
`mono/forbidden-root-context`,
`mono/dependent-version-bump`,
`security/forbidden-strings`)
is off unless `cli-git.config.jsonc` names it,
whether or not a file exists.
An empty `{}` file and no file therefore behave identically,
and `CliGitConfig::defaults()` is the one default set
(`CliGitConfig::unconfigured()` is removed;
the registry field is `off_unless_listed`).

This replaces what this delegation first shipped,
where a present file ran all nine policies at their incumbent defaults
and only a missing file left the four off.

Cutover item:
this repository's translated configuration must name all four,
as the block under "Shape" does.
The incumbent ran `mono/dependent-version-bump` at `error` without the root configuration naming it;
left unlisted,
it silently stops.

A listing with options but no severity still gets the policy's incumbent default severity:
for the two policies that take options,
an options object alone is a third accepted setting form,
so `"markdown/autofix": { "exclude": ["package/ssg/"] }` runs that policy at `warn`
and `"security/forbidden-strings": {}` runs the scanner at `error`.
The two optional policies without options are always listed with a severity word.
The registry records each policy's incumbent default severity,
pinned by `every_identity_resolves_to_its_declared_row`;
`options_alone_list_a_policy_at_its_default_severity` pins the new form.

#### Options

`security/forbidden-strings` keeps `builtinRules`.
`markdown/autofix` keeps `rules` and `exclude`.
`rules` accepts only `lfs-image-url`,
must not be empty,
and must not repeat a rule.
A bare severity on an option-bearing policy uses default options;
the incumbent required an options object there.
Whether `FORBIDDEN_STRINGS_RULES` stays the way to name the rules file is not decided here.

#### Configuration root

Decided by the human on 2026-10-05,
no longer open to veto:
the root is the top level Git reports,
for configuration lookup and for the require-root policy alike.

The root is what `git rev-parse --show-toplevel` reports under the caller's global options.
For a linked worktree that is the linked worktree's own top level,
as with the incumbent.
It honors `--git-dir`,
`--work-tree`,
and their environment forms,
which matches `SPEC.md` ("canonical real-Git toplevel").
The incumbent instead walks up from the effective directory to the nearest valid Git marker
and ignores those options;
that difference is intentional.
A bare repository,
the inside of `.git`,
and a non-repository read no file and use the defaults.

#### Legacy files

Decided by the human on 2026-10-05,
no longer open to veto:
the legacy notice appears only in `git cli-git check`.

`cli-git.config.mjs` or `cli-git.config.ts` without a JSONC file is a migration error on every command,
unchanged.
Beside a JSONC file,
the JSONC file is authoritative,
and ordinary commands and `git cli-git fix` print nothing about the legacy file;
only `git cli-git check` reports each one,
as a `configuration-warning` event on standard output.
This lets both wrappers keep their configuration during the rollback window without a notice on every command.
This delegation first shipped the notice on every configuration-loading command.

#### File handling

The file must be a regular file,
not a symbolic link,
at most 1 MiB,
and UTF-8.
A leading byte order mark is accepted.
`$schema` is rejected like any other unknown key.

## Real Git and forwarding

### Resolution and self-exclusion

`real_git.rs` builds candidates from `PATH`
(and `PATHEXT` on Windows),
promotes conventional locations that `PATH` exposes,
and selects the first candidate `real_git_candidate.rs` classifies as real Git.
A candidate is a wrapper when it is this executable by device and inode or canonical path,
when it has the same length and bytes as this executable,
or when it is a script carrying a TypeScript-wrapper marker.
The incumbent rule that every ELF,
PE,
or Mach-O file is real Git is not inherited without those checks.

`binary_resolution_tests.rs` starts the built executable through a `git`-named symbolic link,
a chained link,
a hard link,
copies earlier and later on `PATH`,
and a linked directory,
with repeated and relative `PATH` entries,
and observes real Git's output each time.
Real Git sits at a non-conventional location in those fixtures,
because a conventional location is promoted first and would hide whether earlier entries were skipped.
`time_bound_detects_a_forwarding_loop` is the positive control:
a self-executing `git` is killed at the time bound.

### Forward-target marker, open to veto

A different wrapper build cannot be recognized by identity or content.
Every real-Git child therefore receives `CLI_GIT_NATIVE_FORWARD_TARGET` naming the selected executable.
A wrapper that finds its own executable named there stops with exit status 2.
The variable is inherited by hooks and every Git child;
a hook-started wrapper sees real Git's path there,
which never matches it.
Limitation:
a different wrapper build ahead of a non-conventional real Git stops every command instead of reaching Git.
The byte-identical copy check is also beyond the stated minimum and is open to veto.

### Forwarding

Commands that need no policy configuration replace the wrapper process with real Git (`exec`) on Unix.
The caller then observes Git's own process:
argument bytes,
streams,
exit status,
and signals.
`binary_forwarding_tests.rs` compares the wrapper with `/usr/bin/git` byte for byte
for non-UTF-8 arguments and paths,
standard input,
binary output,
exit statuses 0,
1,
128,
and 129,
and observes that the wrapper's process ID becomes Git before `SIGTERM`,
`SIGINT`,
and `SIGHUP` end it.

`run_real_git` is the second form,
for callers that must act after Git returns.
It waits for a child with inherited streams and reports `128 + signal` for a signaled child,
which is Git's own `run-command.c` convention;
the ledger lists the incumbent's exit status 1 as a defect.
It installs no signal handler,
as the incumbent did not:
a signal sent to the wrapper alone does not reach Git in that form.
No third-party crate was needed.

### Child environment

`child_environment.rs` appends `core.lockfilePid=true` through the numbered `GIT_CONFIG_*` variables.
`GIT_CONFIG_COUNT` is parsed as Git 2.56.0 `config.c` does (`strtoul`,
then the `*endp` and `INT_MAX` checks),
verified against real Git for accepted and rejected spellings.
The incumbent accepted digits only,
so a count such as `" 2"` lost the injection.

### Worktree identity

`worktree_identity.rs` runs one query:

```sh
git <global options> rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir --show-toplevel
```

Git applies `-C` chains through symbolic links itself;
the incumbent's lexical `-C` resolution is not ported.
Output with a line count other than the expected one is rejected,
so a path containing a line feed fails closed instead of being misparsed.

## Diagnostics and management commands

### Events

`diagnostics.rs` renders JSON Lines events with the incumbent's field order
(`schemaVersion`,
`sequence`,
`type`,
`code`,
`message`),
so a consumer of the incumbent's events parses them unchanged.
`json_string` escapes as `JSON.stringify` does;
`diagnostics_tests.rs` covers quotes,
backslashes,
the short escapes,
the lowest and highest remaining control characters,
and text that tries to end its JSON string or its line.
The `config_schema` fuzz target decodes every rendered rejection back to its message.

A rejected configuration is one `engine-failure` event with code `config-invalid` and exit status 2:
on standard error for a wrapped Git command,
on standard output for `git cli-git check` and `git cli-git fix`.
A legacy file beside the JSONC file is one `configuration-warning` event
with code `legacy-config-ignored` and a `path` field,
on standard output of `git cli-git check` only
(decided 2026-10-05;
see "Legacy files").

### Management namespace

`git cli-git` is answered by the wrapper and never reaches Git,
also when Git global options such as `-C <dir>` precede it.
The word `cli-git` in any other position is an ordinary argument.

```text
Usage: git cli-git check (--all | -- <pathspec>...) [--policy <id>]...
       git cli-git fix (--all | -- <pathspec>...) [--policy <id>]...
       git cli-git --help
```

`check` and `fix` require exactly one scope.
A malformed invocation prints the usage or the specific refusal on standard error and exits 2.
`--policy` accepts shipped policy IDs only;
an unknown ID is a `config-invalid` event.
A well-formed `check` or `fix` resolves real Git,
validates the selected worktree's configuration,
and then stops with exit status 2 and the notice that policy execution is not implemented;
`check`,
and only `check`,
first reports legacy files
(decided 2026-10-05;
see "Legacy files").
`--help` and `-h` print help on standard output with exit status 0,
without resolving Git or reading a repository.

`trust`,
`untrust`,
and `status` still parse.
Each prints that it is retired,
that JSONC configuration is data and runs no repository-supplied code,
and that existing trust records are neither changed nor read.

### Diagnostics choices open to veto

#### Retired trust commands

They exit 0,
with the explanation on standard error,
or on standard output for `--help`.
The caller asked to approve or revoke code execution;
no code execution exists,
so nothing failed.
The alternative is exit status 2,
so that a script still calling `git cli-git trust --yes` notices.
The ledger leaves both the status and the stream open.

#### Exit status 2 for wrapper failures

A missing real Git,
a malformed management invocation,
an invalid configuration,
a repository that cannot be inspected,
and the unimplemented policy stage all exit 2.
The incumbent exited 1 for some of these,
which the ledger lists as a defect.

#### Configuration warning event

`configuration-warning` is a new event type that `SPEC.md` does not define.
The incumbent's `warn-unsafe` warning names a policy and a trigger;
a stale legacy file has neither,
so the event carries `path`.
The alternative is prose on standard error.
A path that is not UTF-8 is rendered with replacement characters in `message` and `path`.

#### Failure codes

`EngineFailureCode` keeps the incumbent's spellings for configuration,
content,
patch,
fix-loop,
transaction,
and index-lock failures.
The plugin and trust codes are dropped.
Which code reports a failing built-in policy is left to the policy engine.

#### Policy selection

`--policy` is checked against the registry before Git is resolved,
so a mistyped ID fails the same way in and outside a repository.
Selecting a shipped policy that the configuration turns off is not rejected here;
what that should do belongs to policy execution.

## What the executable does today

`entry.rs` decides;
`main.rs` only collects arguments and environment.

- A wrapper named by the forward-target marker stops.
- `git cli-git` is handled as described under "Management namespace".
- Real Git is resolved;
  failure exits 2 with the resolver's counts.
  The incumbent exited 1 there;
  2 follows the stream and exit contract for a wrapper failure and is open to veto.
- Inspection commands,
  native queries,
  option errors,
  and a bare `git` are forwarded without reading configuration.
- Every other command resolves the worktree,
  loads and validates configuration,
  and stops with exit 2 and a notice that policy execution is not implemented.
  Forwarding it unguarded would drop enforcement silently.

## Mutation testing

### Runner

Commit `07ca5b9bb`.
`mise run //package/git-policy/cli:native:mutation` runs the gate and then `bin/mutate-native-container.mjs`.
`mise run //package/git-policy/cli:native:mutation:scoped -- --file 'src/native/<name>.rs'`
runs the runner alone over the named files against the last gate image.
The runner builds on the gate's image,
so a campaign is bound to the exact source snapshot the gate tested.
It runs cargo-mutants 27.1.0,
copied from the host Cargo home with its SHA-256 recorded in the manifest,
in a mount-free,
network-disabled container bounded to 2 GiB,
2 CPUs,
and 128 PIDs,
with a 300-second build bound and a 90-second test bound per mutant.
It reads no cargo-mutants configuration (`--no-config`) and excludes no mutant.
It exits nonzero when any mutant is missed or times out.

### Planted controls

Before a campaign the runner removes five guards,
one at a time,
and requires a named control to fail for each.
A campaign whose planted removal goes unnoticed is rejected.

- Self-exclusion by identity and content:
  noticed by `resolution::wrapper_never_selects_itself_or_a_copy_of_itself`.
- The forward-target marker check:
  noticed by `resolution::different_wrapper_build_stops_instead_of_looping`.
- The stop before the policy stage:
  noticed by `policy::repository_changing_commands_are_not_run`.
- Unknown top-level key rejection:
  noticed by `config_parse::tests::unknown_and_retired_top_level_keys_are_named`.
- The legacy migration diagnostic:
  noticed by `config_file::tests::legacy_configuration_alone_requires_migration`.

The first runner version reported the three binary-level controls as not noticed:
Cargo stopped after the unit tests failed and never ran them.
The planted run now passes `--no-fail-fast`,
and all five are noticed in every run recorded here.

### First full campaign

Sources as of commit `07ca5b9bb`,
gate image `7fbc07a48a6e3d6287d73fedff1cd2bc98e7ef9e97fec46a150cb6043bc2eb8e`.
Of 518 mutants,
422 were caught,
56 did not compile,
23 were missed,
and 17 timed out.
The runner exited nonzero,
as it must.
Evidence `package/git-policy/cli/target/verification/native-mutation-q6xDu5`.
This campaign ran before the fix described under "Intermittent test failure",
and its caught count is inflated;
"Full campaign on the fixed tree" replaces it.

### Survivors killed by new controls

Of the 23 missed mutants,
13 showed behavior no control observed.
Commit `6fb148913` adds 10 controls for them.

- `config_loading.rs`,
  9 mutants in `branch` and `tag` classification:
  whether `--format` and `--sort` imply a listing,
  which short letters are presentation for which command,
  whether a lone `-` is a name,
  and the content of the explicit long mutation list.
  Controls `only_commit_filters_imply_listing`,
  `short_letters_are_judged_per_command`,
  and `mutating_long_forms_are_listed_per_command`.
- `config_error.rs`,
  the `Display` implementation:
  `display_prints_exactly_the_message`.
- `config_schema.rs`,
  `markdown_rule_name`,
  2 mutants:
  `markdown_rule_names_are_exact_in_both_directions`.
- `escape_hatch.rs`,
  the separator bound:
  `separator_is_never_removed`.

A scoped campaign over those four files on the sources of commit `6fb148913`
(gate image `ef4b26e1b68250ee7f3f628b018cd5b7d1867c1af5b6b38e0acfc585ef53ffc7`)
tested 101 mutants:
88 caught,
10 did not compile,
3 timed out,
none missed.
Evidence `package/git-policy/cli/target/verification/native-mutation-wEnLMl`.

Through `classify_config_loading` alone the `mutating_long` mutant cannot be observed:
a long option on no list already requires configuration,
so the explicit mutation list is a second guard.
The new control therefore pins the private list directly.

### Survivor removed with its code

The first full campaign missed `<` to `<=` in the indexed loop of `policy_descriptor`.
Every `PolicyId` has a row,
and both forms stop the program on a missing one,
so no control could tell them apart.
Commit `cba0702fe` uses a `for` loop,
which has no bound to mutate.
`mise run //package/git-policy/cli:native:mutation -- --file src/native/policy_registry.rs`,
the chained gate and runner,
then tested 19 mutants on the sources of commit `de15ea3ea`:
15 caught,
3 did not compile,
1 timed out,
none missed.
Evidence `package/git-policy/cli/target/verification/native-mutation-o095Rt`;
gate evidence `native-1NQvVP`.

### Survivors left

#### Code not compiled on Linux

The `#[cfg(not(unix))]` variants of
`forwarding::outcome_of`,
`forwarding::replace_process_with_real_git`,
`git_metadata::path_from_git_bytes`,
`real_git_candidate::is_executable`,
and `real_git_candidate::same_inode` hold 8 missed mutants.
The Linux gate does not compile those bodies,
so no control in it can observe them.
They stay open until a Windows gate exists.

#### Equivalent mutants

- `real_git_candidate.rs`,
  `read_up_to`,
  `filled < limit` to `filled <= limit`:
  the one extra iteration reads into an empty buffer,
  which returns zero bytes and ends the loop the same way.
- `real_git_candidate.rs`,
  `classify_candidate`,
  `MAX_SCRIPT_INSPECTION_BYTES + 1 - header.len()` to `+ header.len()`:
  the script read may take 8 more bytes,
  but any script longer than the bound is still rejected by the length check that follows,
  and a shorter one is read in full either way.

`script_inspection_bound_is_exact` pins the bound from both sides.

#### Timeouts

In the full campaign on the fixed tree 17 mutants timed out.
16 change how a loop advances
(`+=` to `*=` or `-=`,
`-=` to `/=`)
in `child_environment.rs`,
`config_loading.rs`,
`escape_hatch.rs`,
`global_arguments.rs`,
`management_arguments.rs`,
`policy_registry.rs`,
`real_git.rs`,
and `real_git_candidate.rs`,
so that the loop never finishes.
The 17th,
`filled += count` to `filled *= count` in `read_up_to`,
keeps every read at the start of the buffer,
so classifying a large executable reads all of it in 4-byte pieces;
`other_executables_are_real_git` was still running at the bound.
In the scoped rerun that same mutant finished inside the bound and was caught.
The test run is stopped at the 90-second bound,
and cargo-mutants reports that as a timeout,
neither caught nor missed.
The mutation is detected,
but only by the bound.

### Intermittent test failure

The first chained run over `policy_registry.rs` stopped at its unmutated baseline:
`entry::stop_tests::uninterpretable_identity_output_stops` failed with "Text file busy".
Evidence `package/git-policy/cli/target/verification/native-mutation-LugJHy`.
The test process had written a fixture script itself.
A child forked by another test thread at that moment inherits the script open for writing
until it starts its own program,
and running the script inside that window fails.
Commit `de15ea3ea` has fixture scripts written by a child `tee` and wrapper copies made by a child `cp`,
so the test process never holds an executable open for writing.

The rate was measured by repeating the unit and binary-level tests 400 times in one bounded container
(2 GiB,
2 CPUs,
256 PIDs,
no network)
with 16 test threads,
more than the gate's 2,
to widen the window.
Before the fix
(test image `11f544e29a261417dd283a9e4ebac0988acf4c1118dc43486c0c063213da472c`)
74 of 400 runs failed,
68 of them with "Text file busy",
spread over 11 tests,
binary-level controls among them.
The other 6 failing runs were not kept,
so their cause is not recorded.
After the fix
(test image `15bd4395ea94c2f2cde8cc7957baf037ee7d0c7054c91633ff318172b8bd1a8a`)
0 of 400 runs failed.
The rate at the gate's 2 threads was not measured.

The first full campaign ran before the fix,
and its result was inflated by this failure:
two mutants it counted as caught were missed on the fixed tree,
and neither can be caught by a correct control run
(one is equivalent,
the other changes a value no control pinned).
A third,
the `read_up_to` mutant described under "Timeouts",
runs close to the 90-second bound,
so its result varies with timing rather than with this failure.

### Full campaign on the fixed tree

Sources of commit `de15ea3ea`,
gate image `15bd4395ea94c2f2cde8cc7957baf037ee7d0c7054c91633ff318172b8bd1a8a`.
Of 513 mutants
(5 fewer than before:
comparing the two `mutants.json` lists shows that only the `<` and `+=` mutants
of the removed loop in `policy_descriptor` are gone,
and every other difference is a shifted line number),
429 were caught,
56 did not compile,
11 were missed,
and 17 timed out.
The runner exited nonzero,
as it must.
Evidence `package/git-policy/cli/target/verification/native-mutation-3CvHMl`.

The expected result was 9 missed:
the 8 in code not compiled on Linux and the `read_up_to` equivalent mutant.
The 2 others had been counted as caught in the first campaign:

- `64 * 1024` to `64 + 1024` in `MAX_SCRIPT_INSPECTION_BYTES`.
  The bound control is written in terms of the constant,
  so any value passed it.
  Commit `57eee2061` adds `script_inspection_bound_matches_the_incumbent`:
  a 64 KiB launcher with its marker in the last bytes must be recognised,
  matching the incumbent bound in `package/git/executable/src/self-shim.ts`.
- The `classify_candidate` read limit,
  an equivalent mutant listed under "Survivors left".

The third,
the `read_up_to` timeout,
is described under "Timeouts".

A chained gate and scoped campaign over `real_git_candidate.rs` on the sources of commit `57eee2061`
tested 64 mutants:
55 caught,
2 did not compile,
1 timed out,
and 6 missed,
which are the 4 not compiled on Linux and the 2 equivalent mutants.
Evidence `package/git-policy/cli/target/verification/native-mutation-2mwlb7`.

After these runs the missed mutants are exactly the 8 in code not compiled on Linux and the 2 equivalent mutants.

## Fuzzing

### Sidecar

Commits `789eb07b4` and `744f82ede`.
`package/git-policy/cli.fuzz` (crate `git-policy-cli-fuzz`) holds three AddressSanitizer targets over pure functions;
none starts Git or touches a repository.

- `global_arguments` checks `global_layout`:
  arguments unchanged,
  repeatable result,
  a boundary token of the kind the outcome claims,
  a global prefix that names no command on its own,
  and no effect from anything after a decided boundary.
- `config_loading` checks `classify_config_loading`:
  only native queries,
  option errors,
  and an independently restated inspection list skip configuration,
  and any `branch` or `tag` invocation given a mutating flag in first position requires it.
- `config_schema` builds a valid `cli-git.config.jsonc` together with the settings it must parse to,
  compares the parser's result with that expectation,
  restates accepted documents canonically,
  and decodes every rejection's `config-invalid` line back to its message.
  UTF-8 inputs are also parsed as written.

Each target feeds its bytes both raw (split at NUL for arguments) and through a structured generator.
The invariants live in the helper library,
so the generator controls call exactly what the fuzzer calls.

### Controls

`mise run //package/git-policy/cli.fuzz:test` passes 7 generator controls;
they count that the generators reach every layout outcome,
both loading decisions,
and non-default accepted configurations.
`mise run //package/git-policy/cli.fuzz:test:planted` plants five defects one at a time in a temporary copy
and requires a control to fail for each:
a mutating `branch` letter accepted as presentation,
a bare `git` skipping configuration,
an unconsumed global option value,
`warn` read as `error`,
and a rejected `landing` section.
All five were noticed;
evidence `package/git-policy/cli.fuzz/target/verification/planted-ZR7zvL`.
A first attempt removed `d` from the `branch` mutation letters only and was not noticed,
correctly:
an unlisted letter already requires configuration,
so that edit changes no behavior.

### Smoke campaign

`mise run //package/git-policy/cli.fuzz:smoke` builds with the nightly compiler mounted read-only
(`rustc 1.100.0-nightly (1303417c4 2026-09-21)`,
cargo-fuzz 0.13.2),
runs the controls and Clippy in the container,
then fuzzes each target for 30 seconds with no host mounts,
no network,
2 GiB,
2 CPUs,
128 PIDs,
and a 4,096-byte input limit.
A target passes only with exit status 0 and at least one executed unit.
Base image `62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884`.

The run on the committed sources reported
905,203 executions for `global_arguments`,
376,745 for `config_loading`,
and 19,954 for `config_schema`,
each with exit status 0 and no artifact;
evidence `package/git-policy/cli.fuzz/target/verification/campaign-WmBqbc`.
An earlier run,
before the token tables moved to their own module,
reported 676,836,
295,227,
and 29,590;
evidence `campaign-j0BpUV`.

On the final tree
(commit `1d14906ad`,
after the registry and fixture changes in the subject)
the smoke campaign reported 975,367,
359,412,
and 49,392 executions,
each with exit status 0 and no artifact;
evidence `campaign-y7YxHb`.
`test:planted` again noticed all five planted defects;
evidence `planted-qPq6go`.

### Limits

These are 30-second smoke runs,
not a long campaign,
and coverage was not measured.
Real-Git resolution,
forwarding,
configuration file reading,
and the management grammar have no fuzz target.
The sidecar depends on `libfuzzer-sys` 0.4,
as the existing fuzz sidecars do.
`file-enforcer` was not run for the new package.

## Module map

The policy engine can build on these modules of `package/git-policy/cli/src/native`.
Every public item has rustdoc.

- `policy_registry`:
  `PolicyId`,
  `Severity`,
  `PolicyDescriptor`,
  `POLICY_REGISTRY`,
  `policy_by_name`,
  `policy_descriptor`,
  `severity_from_name`,
  `severity_name`.
- `config_schema`:
  `CliGitConfig`,
  `PolicyConfig` with `setting(id)`,
  the option and concurrency records,
  `unlisted_severity(descriptor)`,
  and the `defaults()` constructor
  (`unconfigured()` was removed on 2026-10-05;
  see "Defaults with and without a configuration file").
- `config_parse`:
  `parse_config(source)`.
- `config_file`:
  `load_repository_config(root)` returning `LoadedConfig`.
- `config_error`:
  `ConfigError`.
- `invocation_config`:
  `load_invocation_config(real_git, global_prefix, overlay)`,
  `config_invalid_event`,
  `legacy_warning_events`.
- `real_git` and `real_git_candidate`:
  `ResolutionInputs`,
  `process_resolution_inputs`,
  `resolve_real_git`,
  `classify_candidate`.
- `child_environment`:
  `child_environment_overlay`,
  `lockfile_pid_overlay`,
  `parse_config_count`.
- `forwarding`:
  `git_command`,
  `run_real_git` returning `ChildOutcome`,
  `exit_code`,
  `replace_process_with_real_git`.
- `git_metadata`:
  `run_metadata_git`,
  `strip_git_line`,
  `path_from_git_bytes`.
- `worktree_identity`:
  `WorktreeIdentity`,
  `resolve_worktree_identity`,
  `worktree_root`.
- `effective_target`:
  `EffectiveTarget`,
  `classify_effective_target`,
  `default_allowed_worktree_dirs`.
- `escape_hatch`:
  `strip_escape_hatch`.
- `diagnostics`:
  `EngineFailureCode`,
  `render_engine_failure`,
  `render_configuration_warning`,
  `json_string`.
- `action`:
  `Action`,
  `failure`,
  `ENGINE_FAILURE_EXIT_CODE`.
- `management_arguments` and `management`:
  `parse_management_arguments`,
  `plan_management`.
- `entry`:
  `plan_invocation`,
  which returns an `Action` without side effects,
  and `run_process`,
  which performs it.

Policy execution replaces the stop notice in `entry::plan_invocation` and `management::plan_management`;
both already hold the validated `LoadedConfig` at that point.

## Dependency needs

No third-party crate was added to the wrapper,
and its `Cargo.lock` is unchanged.
Needs that the standard library does not cover,
recorded instead of solved:

- Forwarding a signal from a waiting wrapper to its Git child needs signal handling,
  which the standard library does not offer.
  A crate such as `signal-hook` or direct `libc` calls would be a technology decision.
- Executable detection by `access(2)`,
  which honors access control lists and the caller's identity,
  needs `libc`.
  Mode bits are used instead.
- The verification tools are cargo-mutants 27.1.0 and cargo-fuzz 0.13.2 from the host Cargo home.

## Not ported

- Policy execution of any kind.
- Startup transaction recovery,
  which the incumbent runs before almost every command including `status`.
- The built-in policies and fixed transforms the incumbent runs on inspection commands without configuration,
  for example `require-root` and the status-hints transform.
- Post-command output:
  the version banner and the status note.
  Both need the waiting form of forwarding.
  The banner lists behaviors this executable does not implement.
- Index-writer coordination and worktree copy around forwarding.
- Alias resolution (`src/forwarded-command.ts`) and the built-in command table.
- Generic `--no-enforce-<policy-id>` controls (`src/policy-engine/controls.ts`).
  `escape_hatch.rs` ports only the per-command stripping of one token.
- Windows and macOS behavior is written but not run:
  only Linux is verified.
  Executable detection uses mode bits,
  not `access(2)`.

## Cutover items

- `package/git/executable/src/self-shim.ts` treats every native executable as real Git,
  so the TypeScript resolver would select the native wrapper.
  It needs native-wrapper recognition at cutover;
  this delegation does not edit TypeScript.
- The Cargo binary is named `cli-git-native`,
  not `git`,
  so a `target` directory on `PATH` cannot shadow Git.
  Installation wiring gives it the `git` name.
- The root `cli-git.config.jsonc` translation is shown under "Shape" and is not created here.
- `SPEC.md` still describes plugins,
  trust,
  and executable configuration.
  It needs the JSONC schema,
  the retired trust commands,
  and the `configuration-warning` event if those choices stand;
  this delegation does not edit it.

## Failed or skipped

- The full mutation campaign ends nonzero,
  because the survivors listed under "Survivors left" remain.
  No mutant was excluded to change that.
- The last full campaign ran on the sources of `de15ea3ea`.
  Commit `57eee2061` only adds a control,
  verified by the scoped campaign over `real_git_candidate.rs`;
  no full campaign ran on it.
- `native:mutation`,
  the task that chains the gate and the runner,
  was run only with a one-file scope.
  Both full campaigns used the gate and `native:mutation:scoped` as separate commands.
- Slice 1 was gated before its commit,
  but that run's evidence directory was not recorded by name.
- The registry default correction was gated only together with slice 2.
- Windows and macOS code paths were never compiled or run.
- Fuzzing was limited to 30-second smoke runs.
- `file-enforcer` and the repository-wide lint were not run;
  the 300-code-line limit was checked with a line count,
  not with the linter.
- The container scripts are not clean under the package's Oxlint configuration.
  `mise run //package/git-policy/cli:lint:oxlint:paths` over the two scripts in `bin` reports 11 errors:
  3 on `bin/test-native-container.mjs`,
  which predates this delegation,
  and 8 on `bin/mutate-native-container.mjs`
  (synchronous process calls,
  regular expressions,
  `try` with `finally`,
  and arrow functions).
  They were left as written,
  in the style of the linter package's container scripts.
  The fuzz sidecar's two scripts were not linted;
  that package has no Oxlint task.

## Next action

The policy engine:
replace the stop notice in `entry::plan_invocation` and `management::plan_management` with policy execution,
starting from the validated `LoadedConfig`.
Decide the choices listed as open to veto before that code depends on them.
