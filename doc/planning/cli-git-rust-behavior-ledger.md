# cli-git Rust behavior ledger

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Every responsibility the incumbent TypeScript Git policy wrapper carries,
who consumes it,
whether the Rust rewrite keeps it,
and its proposed owner and proof in the native implementation.
This is the first step of the accepted sequence in
[`cli-git-rust-implementation.md`](cli-git-rust-implementation.md)
("Implementation sequence and verification").
It is an inventory:
it changes no accepted decision and proposes no design beyond owner module names.

## Pinned incumbent revision

The TypeScript wrapper this ledger inventories, and the four authoring packages
(`package/git-policy/{api,forbidden-strings,markdown-lint,repository}`), are deleted from the working tree by the
retirement commits.
The last commit that contains them is `f4fc1fa213ece65672e4314983a75266837892f9`.
Every `src/...ts:NN` and `SPEC.md:NN` citation below resolves in that commit
(`git show f4fc1fa213ece65672e4314983a75266837892f9:package/git-policy/cli/src/<path>`),
including the `SPEC.md` sections that the retirement rewrites.

## Source and conventions

Source:
a read-only survey on 2026-10-05.
Line numbers are as of commit `cd54f8b64`.
The incumbent TypeScript under `package/git-policy/cli/src` (outside `src/native`)
and `package/git-policy/cli/SPEC.md` last changed in `cf2bf70d2` (2026-09-26).
The native Rust files are cited as of `df25471a9`,
which landed during the survey and added the configuration and registry modules.
`doc/handover/cli-git-rust-implementation.md` also changed during the survey,
so it is cited by section name.

Path convention:
`SPEC.md`,
`README.md`,
and paths starting with `src/`,
`e2e/`,
`perf/`,
or `bin/` are relative to `package/git-policy/cli/`.
Every other path is repository-relative.
A bare line number or range refers to the file of the nearest preceding citation that carries a line number.

Planning citations:
"implementation plan" is `doc/planning/cli-git-rust-implementation.md`,
"rewrite scope" is `doc/planning/cli-git-rust-rewrite.md`,
and the accepted decision is `doc/decision/cli-git-rust-rewrite.md`.

Each responsibility records these points:

- Behavior:
  what the incumbent does,
  with the implementing source.
- Spec:
  the `SPEC.md` lines that state it,
  or that no section states it.
- Consumers:
  who depends on it.
- Status:
  `retained` or `retired`.
  A retired entry cites the planning text that retires it.
- Rust owner:
  a module under `package/git-policy/cli/src/native/`.
  At `df25471a9` these modules exist:
  `global_arguments.rs`,
  `config_loading.rs`,
  `config_error.rs`,
  `config_file.rs`,
  `config_parse.rs`,
  `config_policies.rs`,
  `config_schema.rs`,
  `config_values.rs`,
  and `policy_registry.rs`.
  Every other name is marked "proposed" and is for the owning delegate to confirm.
- Consumer-level test:
  what is run and what is observed.
  "Standard fixture" means the built native `git` executable,
  first on `PATH`,
  in a disposable repository with a local bare remote,
  inside the bounded Git 2.56.0 container image
  (implementation plan lines 326 to 338).
- Native state:
  `absent`,
  `in progress`,
  or `implemented`.
  "In progress" means the `Resumption 2026-10-05` section of `doc/handover/cli-git-rust-implementation.md`
  delegates the area and no module for it exists at `df25471a9`.
  "Implemented" means a library module with unit tests exists.
  No native executable exists at `df25471a9` (`src/native/lib.rs:1`),
  so no consumer-level test in this ledger has run.

## Measured size

Measured with `tokei` 15.0.0 over `package/git-policy/cli/src` excluding `native`;
code lines only.

- Source outside tests and built-artifact fixtures:
  312 files,
  40,559 lines.
- Built-artifact fixtures under `src/trust/fixture`:
  53 files,
  9,379 lines.
- Unit tests (`*.unit.test.ts`):
  110 files,
  24,102 lines.
- Per source directory,
  tests excluded:
  `policy-engine` 135 files and 18,582 lines;
  `trust` 42 files and 5,939 lines;
  `worktree-copy` 25 files and 3,755 lines;
  top-level files 33 and 3,587 lines;
  `optional` 18 files and 2,211 lines;
  `parser` 20 files and 1,837 lines;
  `index-lock` 11 files and 1,645 lines;
  `shadow-repository` 8 files and 1,176 lines;
  `hook-dispatch` 4 files and 542 lines;
  `rule` 6 files and 453 lines;
  `owner-lock` 2 files and 386 lines;
  `api` 6 files and 317 lines;
  `maintenance` 2 files and 129 lines.
- Inside `policy-engine`,
  by file-name family:
  `commit-transaction-*` without recovery 36 files and 4,798 lines;
  `commit-landing-*` 14 files and 2,301 lines;
  `commit-transaction-recovery-*` 12 files and 1,848 lines;
  `commit-replay-*` 10 files and 1,772 lines;
  `manual-push-*` 8 files and 1,215 lines;
  `commit-capture-order-*` 8 files and 1,109 lines.
- Harnesses:
  `e2e` 48 source files and 7,782 lines plus 7 test files and 1,050 lines;
  `perf` 26 files and 3,498 lines.
- Sibling packages:
  `package/git-executable/core` 7 source files and 409 lines;
  `package/git-policy/repository` 9 and 1,119;
  `package/git-policy/forbidden-strings` 6 and 633;
  `package/git-policy/markdown-lint` 5 and 600;
  `package/git-policy/api` 6 and 305.

## Tally

Each level-3 heading is one responsibility,
counted with `rg --multiline --count-matches` over the `Status` and `Native state` points of this file.

- Responsibilities:
  112 in total.
- Status:
  92 retained,
  20 retired.
  The retired entries are executable configuration discovery,
  plugin registration,
  the 10 trust entries,
  the 5 authoring entries,
  older-Git degradation,
  and the TypeScript unit suites and built-artifact fixtures.
- Native state at `df25471a9`:
  8 implemented as library code without an executable,
  15 in progress,
  88 absent,
  and 1 entry (the unit suites) that is itself verification.

## Entry and dispatch

### Executable entry

- Behavior:
  `src/index.ts:1` carries the Node shebang;
  `src/index.ts:40-41` runs `runCliGit` only when Node executes the file as the program entry.
  `package/git-policy/cli/package.json:17-19` maps the `git` bin name to `dist/final/node/index.mjs`.
  The installed shim `node_modules/.bin/git` is a package-manager shell script that starts Node on that file.
- Spec:
  `SPEC.md:48-60`.
- Consumers:
  every `git` invocation whose `PATH` resolves the shim first.
  Root `mise.toml:1252-1253` puts `node_modules/.bin` on `PATH`;
  root `package.json:36` depends on the package.
  Agent tooling reaches it the same way:
  `AGENTS.md:1338` (rule CLG) names its guards.
- Status:
  retained.
  The implementation plan lines 425 to 426 require the installed launcher to resolve directly to the native executable,
  not to a Node process that starts it.
- Rust owner:
  a thin binary entry beside `lib.rs`
  (implementation plan lines 29 to 35).
- Consumer-level test:
  in the standard fixture,
  run `git --version` through `PATH` and inspect the process tree;
  observe native Git's version text and no Node process.
- Native state:
  in progress.

### Startup order

- Behavior:
  `runCliGit` (`src/bin.ts:109-465`) runs these steps in order:
  install the lock PID environment (`110`);
  detect the management namespace (`139-148`);
  resolve real Git (`169`,
  `187`);
  resolve worktree identity when classification skips configuration (`191-197`);
  recover dead commit transactions unless a preparation lease is inherited (`199-204`);
  load configuration (`208-210`);
  run the commit transaction (`248-257`);
  run the pre-forward engine when no transaction applied (`261-272`);
  run the manual-push gate (`314-346`);
  forward to real Git when no commit landed (`357-369`);
  run the post-commit lifecycle and auto-push (`385-423`);
  print post-command output (`425-428`).
- Spec:
  `SPEC.md:1991-2010`,
  `3497-3503`.
- Consumers:
  every wrapper invocation.
- Status:
  retained.
- Rust owner:
  `wrapper_main.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  plant a dead-owner transaction directory and an invalid configuration,
  then run `git add -- a.txt`;
  observe that the transaction directory is recovered
  before the configuration failure is reported with exit `2`.
- Native state:
  absent.

### Global option parsing

- Behavior:
  `parseGlobalOptions` (`src/parse-global-options.ts:203-210`) walks the tokens before the subcommand.
  It applies each `-C <path>` to an effective working directory (`68-83`,
  `133-155`),
  skips one value after `-c`,
  `-C`,
  `--git-dir`,
  `--work-tree`,
  `--namespace`,
  `--super-prefix`,
  and `--attr-source` (`12-20`),
  treats every other dash-led token as a flag (`166-174`),
  and marks `--version`,
  `-v`,
  `--help`,
  and `-h` as short-circuit forms (`25-30`).
  It returns `effectiveCwd`,
  `subcommandIndex`,
  and `willShortCircuit` (`35-48`).
- Spec:
  `SPEC.md:779-797`.
- Consumers:
  25 non-test source files import it
  (`rg --files-with-matches "parse-global-options" src --glob '!**/*.unit.test.ts' --glob '!src/native/**'`,
  minus the file itself).
- Status:
  retained.
  The native table differs from the incumbent table;
  see "Spec and code disagreements" and "Incumbent defects and stale comments".
- Rust owner:
  `global_arguments.rs` (exists).
- Consumer-level test:
  in the standard fixture,
  run each global option form,
  including a missing value,
  an unknown option,
  and a non-UTF-8 `-C` path,
  through the native `git` and through `/usr/bin/git`;
  observe identical exit status and output.
- Native state:
  implemented as a library function (`src/native/global_arguments.rs:83-157`).
  Evidence:
  5 unit tests in `src/native/global_arguments_tests.rs`,
  part of the 9 tests and Clippy pass of `mise run //package/git-policy/cli:native:test:container`
  recorded in the `Resumption 2026-10-05` section of `doc/handover/cli-git-rust-implementation.md`.
  No executable consumes it yet,
  so the consumer-level test has not run.

### Configuration-loading fast path

- Behavior:
  `classifyConfigLoading` (`src/trust/command-classification.ts:267-304`) returns `skip-config`
  for 27 inspection commands (`17-45`)
  and for `branch` and `tag` forms without a mutating flag that list or take no positional (`49-118`,
  `178-251`,
  `299-303`).
  Every other command,
  and an absent subcommand,
  returns `load-config`.
  `resolveRuntimeConfig` (`src/trust/runtime-config.ts:36-61`) consults it before discovery,
  and `src/bin.ts:191-197` reuses the worktree identity only on the skip path.
  Short-circuit forms never load configuration (`src/bin.ts:208-210`).
- Spec:
  `SPEC.md:787-797`.
- Consumers:
  every wrapped inspection command;
  the `read-only` and `no-config` scenarios of `perf/lifecycle-latency-contracts.ts:219-231`.
- Status:
  retained.
  The implementation plan lines 76 to 77 keep the fast path:
  operations that need no policy configuration must not initialize the policy engines.
- Rust owner:
  `config_loading.rs` (exists).
- Consumer-level test:
  in the standard fixture,
  write an invalid `cli-git.config.jsonc`,
  then run `git status` and `git add -- a.txt`;
  observe that `status` succeeds and `add` exits `2` with a `config-invalid` event.
- Native state:
  implemented as a library function (`src/native/config_loading.rs:210-229`).
  Evidence:
  4 unit tests in `src/native/config_loading_tests.rs`,
  in the same recorded 9-test and Clippy pass.
  No executable consumes it yet.

### Wrapper controls and escape hatches

- Behavior:
  `parsePolicyControls` (`src/policy-engine/controls.ts:132-226`) removes wrapper-only tokens
  before Git's `--` separator and outside option values:
  `--cli-git-keep-going` (`12`),
  one `--no-enforce-<policy-id>` per registered policy (`69-71`,
  `142-147`),
  and the aliases `--no-enforce-worktree`,
  `--no-enforce-worktree-branch`,
  and `--no-enforce-bulk-add` (`37-50`).
  It returns the forwardable arguments,
  the keep-going flag,
  and the escaped policy IDs.
  `stripEscapeHatch` (`src/escape-hatch.ts:81-135`) removes one flag-position token
  for the per-command checks.
  `--no-enforce-only` belongs to the commit-only transform (`src/parser/commit.ts:25`),
  and `--no-worktree-copy` to worktree copy (`src/escape-hatch.ts:15`).
- Spec:
  `SPEC.md:1246-1252`,
  `4005-4008`.
- Consumers:
  agents and humans bypassing one policy for one invocation;
  `AGENTS.md:1338` (rule CLG) restricts when agents may use them.
- Status:
  retained.
- Rust owner:
  `wrapper_controls.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git add --no-enforce-add-explicit .`,
  `git add --no-enforce-bulk-add .`,
  and `git commit -m --no-enforce-only -- a.txt`;
  observe that the first two stage,
  the wrapper token never reaches Git,
  and the third commits with the literal message `--no-enforce-only`.
- Native state:
  absent.

### Post-command output

- Behavior:
  `printPostCommandOutput` (`src/post-command-output.ts:38-103`) prints one line to stdout
  after a successful version request (`88-94`)
  and one human note after `git status`
  unless the output is machine-readable or the caller set `advice.statusHints` (`72-87`,
  `95-102`).
- Spec:
  no `SPEC.md` section states it.
- Consumers:
  humans reading `git status`;
  any script that parses `git --version` output sees the added line.
- Status:
  retained.
  The implementation plan does not name it;
  see "Open questions".
- Rust owner:
  `post_command_output.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git status`,
  `git status --porcelain`,
  and `git -c advice.statusHints=true status`;
  observe the note only in the first.
- Native state:
  absent.

## Real Git and child processes

### Real-Git resolution and self-exclusion

- Behavior:
  `resolveRealGit` (`package/git-executable/core/src/resolve-real-git.ts:360-414`) builds absolute candidates
  from `PATH` and,
  on Windows,
  `PATHEXT` (`78-96`,
  `155-226`),
  promotes common platform locations that `PATH` exposes
  (`package/git-executable/core/src/platform-paths.ts:10-22`,
  `44-98`),
  and returns the first executable candidate that is not a wrapper shim
  (`package/git-executable/core/src/resolve-real-git.ts:290-338`).
  `isGitPolicySelfShim` (`package/git-executable/core/src/self-shim.ts:185-246`) returns `false`
  for any ELF,
  PE,
  or Mach-O header (`34-45`,
  `212-218`),
  and otherwise searches at most 64 KiB of script text for four markers
  naming the package or its bundled entry (`139-164`).
  Successful results are cached for the process (`package/git-executable/core/src/resolution-cache.ts`).
- Spec:
  `SPEC.md:29-38`;
  `README.md:21-38`.
- Consumers:
  the wrapper (`src/bin.ts:33`,
  `169`,
  `187`;
  `src/effective-target.ts:6`;
  `src/policy-engine/branch-worktree-check.ts`;
  `src/policy-engine/direct-check-facts.ts`;
  `src/policy-engine/direct-fix.ts`;
  `src/rule/commit-index-check.ts`;
  `src/rule/commit-sequencer-check.ts`;
  `src/maintenance/hk-config-cleanup-command.ts`)
  and `package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts`.
- Status:
  retained.
  The implementation plan lines 84 to 91 port resolution,
  wrapper-self exclusion,
  and platform lookup into Rust,
  and state that the TypeScript resolver used by other consumers
  needs native-wrapper recognition at cutover.
  The incumbent classifies every native executable as real Git,
  so an unchanged TypeScript resolver would select a native wrapper.
- Rust owner:
  `git_resolution.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  place the native wrapper under two `PATH` directories ahead of real Git and run `git rev-parse --git-dir`;
  observe one real-Git process and no recursion.
  Repeat with no real Git on `PATH`;
  observe a diagnostic and a nonzero exit.
- Native state:
  in progress.

### Alias resolution and built-in command table

- Behavior:
  `resolveForwardedCommand` (`src/forwarded-command.ts:261-397`) resolves the command a forwarded invocation runs.
  Names in `GIT_BUILTIN_COMMANDS` (`src/git-builtin-commands.ts:17`,
  taken from Git 2.55.0) are never treated as aliases.
  Other names are looked up as `alias.<name>` and `alias.<name>.command` (`src/forwarded-command.ts:219-259`),
  split with Git's `split_cmdline` quoting rules (`112-217`),
  and expanded up to 16 times (`28`).
  A value starting with `!` is a shell alias and stays unresolved.
- Spec:
  `SPEC.md:910-918`,
  `3472-3473`.
- Consumers:
  index-writer coordination and worktree-copy applicability.
- Status:
  retained.
- Rust owner:
  `alias_resolution.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  define `alias.wta = worktree add` and a chained alias,
  then run `git wta ../topic` from a linked worktree;
  observe that ignored files are copied into the new worktree.
- Native state:
  absent.

### Worktree identity resolution

- Behavior:
  `resolveGitWorktreeIdentity` (`src/git-worktree-identity.ts:298-374`) replays the caller's global options
  through `git rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir --show-cdup`
  (`158-228`),
  canonicalizes the paths,
  and classifies the target as `outside-worktree`,
  `bare-repository`,
  `main-worktree`,
  or `linked-worktree` (`25-97`,
  `365-373`).
- Spec:
  `SPEC.md:903-908`.
- Consumers:
  linked-worktree enforcement (`src/effective-target.ts:62-86`),
  worktree copy (`src/worktree-copy/git-observer.ts:93`),
  and startup recovery (`src/bin.ts:191-204`).
- Status:
  retained.
- Rust owner:
  `worktree_identity.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run a guarded command with `--git-dir` and `--work-tree` naming a main worktree from inside a linked one;
  observe the main-worktree rejection.
- Native state:
  absent.

### Child environment and lock PID injection

- Behavior:
  `installGitChildEnvironment` (`src/git-child-environment.ts:191-194`) appends `core.lockfilePid=true`
  through `GIT_CONFIG_COUNT`,
  `GIT_CONFIG_KEY_<n>`,
  and `GIT_CONFIG_VALUE_<n>` (`103-157`).
  It preserves existing entries,
  adds nothing when the last numbered `core.lockfilePid` entry already reads as true,
  and adds nothing when `GIT_CONFIG_COUNT` is malformed (`74-83`,
  `119-122`).
  Private preparation removes `GIT_DIR`,
  `GIT_WORK_TREE`,
  `GIT_COMMON_DIR`,
  and `GIT_OBJECT_DIRECTORY` from the native commit's environment
  (`src/policy-engine/commit-preparation-native.ts:96-101`).
  Leases travel in `CLI_GIT_PREPARATION_LEASE`,
  `CLI_GIT_LANDING_LEASE`,
  and `CLI_GIT_WORKTREE_COPY_LEASE`
  (`src/hook-dispatch/hook-dispatch-plan.ts:57`;
  `src/index-lock/landing-lease.ts:26`;
  `src/worktree-copy/journal-lock.ts:63`).
- Spec:
  `SPEC.md:3382-3398`,
  `2209-2213`,
  `2455-2458`,
  `3487-3493`.
- Consumers:
  every forwarded and spawned Git;
  foreign `index.lock` evidence reads the PID files this setting produces.
- Status:
  retained.
- Rust owner:
  `git_child_environment.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run a wrapped `git add` held in a hook with `GIT_CONFIG_COUNT=1` already set;
  observe the caller's entry intact,
  a second entry for `core.lockfilePid`,
  and a PID file beside `index.lock`.
- Native state:
  in progress.

### Forwarding, standard streams, signals, and exit codes

- Behavior:
  `forwardToRealGit` (`src/forward-real-git.ts:41-91`) resolves the command,
  takes index-writer coordination,
  and runs real Git through `runGitWithWorktreeCopy`.
  Real Git is spawned with an argument array,
  no shell,
  and inherited standard streams (`src/worktree-copy/lifecycle.ts:75-106`).
  The exit mapping is in `src/bin.ts:431-464`:
  a failed real Git exits with its code,
  or `1` when it has none (`454-456`);
  a policy decision exits `1` or `2` (`441-442`);
  recovery and unproven-lock errors emit one engine-failure event and exit `2` (`432-440`);
  a worktree-copy failure exits `2`,
  or Git's code,
  or `1` (`443-453`);
  any other error prints its message and exits `1` (`457-460`).
  The wrapper source installs no signal handler:
  `rg "process\.on\(|process\.once\(|\.on\(['\"]SIG" src --glob '!src/native/**' --glob '!**/*.unit.test.ts'`
  returns nothing,
  with and without `--hidden --no-ignore`;
  without the test exclusion it matches two unit-test files only.
  A Git child ended by a signal therefore yields exit `1`.
- Spec:
  `SPEC.md:1577-1622`,
  `1066-1073`.
- Consumers:
  every forwarded command;
  callers that branch on Git's exit status.
- Status:
  retained.
  The implementation plan lines 84 to 91 port child environment,
  signals,
  and exit handling,
  and lines 47 to 49 require argument bytes and path representations to be preserved.
  The incumbent holds arguments as JavaScript strings (`src/bin.ts:114-115`).
- Rust owner:
  `git_forwarding.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run a forwarded command that exits `3`,
  one killed by `SIGTERM`,
  and one with a non-UTF-8 pathspec;
  observe the wrapper's exit status and that Git received the exact bytes.
- Native state:
  in progress.

## Configuration

### Executable configuration discovery

- Behavior:
  `discoverConfig` (`src/trust/config-discovery.ts:142-178`) finds the repository root from the effective directory
  and returns `cli-git.config.mjs`,
  or else `cli-git.config.ts`,
  rejecting symbolic links and non-regular files (`src/trust/config-discovery.ts:68-90`).
  The root configuration is `cli-git.config.ts`,
  which imports plugins from `@monochromatic-dev/git-policy-cli/ts` (`cli-git.config.ts:6-12`).
- Spec:
  `SPEC.md:19-27`,
  `718-747`.
- Consumers:
  this repository's root configuration.
- Status:
  retired.
  The implementation plan lines 71 to 75 delete executable-config discovery;
  lines 440 to 443 replace it with an explicit migration diagnostic for legacy `.ts` or `.mjs` files
  and a hand translation of this repository's configuration.
  The rewrite scope lines 110 to 115 record the user's selection of JSONC.
- Rust owner:
  none for discovery of executable files;
  `config_file.rs` owns the migration diagnostic,
  listed under "Responsibilities the plan adds".
- Consumer-level test:
  in the standard fixture,
  leave only `cli-git.config.ts` in the repository root and run `git add -- a.txt`;
  observe the migration diagnostic and that no TypeScript ran.
- Native state:
  implemented for the diagnostic as a library function:
  `load_repository_config` (`src/native/config_file.rs:260-310`) returns the `migration_required` error (`124-136`)
  when only a legacy file exists.
  Evidence:
  13 unit tests in `src/native/config_file_tests.rs`;
  no gate run for `df25471a9` is recorded in a handover document.

### Configuration value validation

- Behavior:
  `validateConfig` (`src/trust/config-validation.ts:396-544`) rejects unknown top-level keys (`401-407`),
  validates concurrency keys,
  registers built-ins and plugin policies,
  resolves each policy ID,
  applies defaults,
  parses option values through each policy's schema,
  and resolves declared inputs.
  `runPolicyEngine` rejects an unknown configured or selected policy ID
  with `config-invalid` (`src/policy-engine/engine.ts:207-241`).
  A severity is `off`,
  `warn`,
  or `error` (`src/api/policy-types.ts:21`).
- Spec:
  `SPEC.md:718-747`.
- Consumers:
  the root configuration's `policies` map (`cli-git.config.ts:27-58`).
- Status:
  retained for values:
  severities,
  policy IDs,
  per-policy options,
  and concurrency keys.
  The implementation plan lines 55 to 61 require the same errors from a typed JSONC document.
  The plugin and trust keys are retired;
  see "Plugin registration" and "Trust consent protocol".
- Rust owner:
  `config_file.rs`,
  `config_parse.rs`,
  `config_policies.rs`,
  `config_values.rs`,
  `config_schema.rs`,
  and `config_error.rs` (exist).
- Consumer-level test:
  in the standard fixture,
  run `git add -- a.txt` once per invalid document:
  an unknown key,
  an unknown policy ID,
  a duplicate key,
  an invalid severity,
  and an invalid option value;
  observe exit `2` and one `config-invalid` event each,
  before any policy runs.
- Native state:
  implemented as library modules.
  `load_repository_config` (`src/native/config_file.rs:260-310`) reads only `cli-git.config.jsonc`
  at a given repository root (`41`),
  refuses more than 1 MiB (`61`),
  and `parse_config` (`src/native/config_parse.rs:154-198`) accepts the top-level keys `policies`,
  `hooks`,
  `indexLock`,
  and `landing` (`39`).
  Typed options exist for two policies:
  `builtin_rules` for forbidden-strings,
  and `rules` and `exclude` for Markdown autofix (`src/native/config_schema.rs:75-93`).
  Evidence:
  38 unit tests in `config_file_tests.rs`,
  `config_parse_tests.rs`,
  and `config_values_tests.rs`;
  no gate run for `df25471a9` is recorded in a handover document.
  Nothing emits a `config-invalid` event yet.

### Concurrency configuration

- Behavior:
  `validateConcurrencyConfig` (`src/trust/config-validation-concurrency.ts:167-205`) accepts
  `hooks.concurrentCommits` (default `false`),
  `indexLock.unprovenOwnerTimeoutMs` (default `1000`),
  and `landing.reserveAfterLostRaces` (default `1`),
  with the defaults at `60-67`.
  Startup recovery and config-free forwarding use the defaults (`src/bin.ts:361-366`).
- Spec:
  `SPEC.md:749-775`.
- Consumers:
  the hook lock,
  the foreign `index.lock` wait,
  and the starvation reservation.
- Status:
  retained.
  The implementation plan line 57 names "existing concurrency controls" as typed settings.
- Rust owner:
  `config_parse.rs` and `config_schema.rs` (exist).
- Consumer-level test:
  in the standard fixture,
  set each key to a valid value,
  zero,
  a negative number,
  a fraction,
  a string,
  and an unknown nested key;
  observe acceptance or a `config-invalid` exit `2` as `SPEC.md:770-774` lists.
- Native state:
  implemented as library code:
  `apply_concurrency` (`src/native/config_parse.rs:85-150`)
  and the defaults `1000` and `1` (`src/native/config_schema.rs:26-29`,
  `122-167`).
  Evidence:
  the unit tests named under "Configuration value validation";
  no recorded gate run.

### Plugin registration

- Behavior:
  `validateConfig` registers each namespace of `plugins` and resolves plugin policy IDs
  as `<namespace>/<policy-name>` (`src/trust/config-validation.ts:411-420`).
  The root configuration registers the namespaces `markdown`,
  `mono`,
  and `security` (`cli-git.config.ts:22-26`).
- Spec:
  `SPEC.md:336-356`,
  `725-728`.
- Consumers:
  the root configuration.
- Status:
  retired.
  The implementation plan lines 63 to 67 state that there are no imports,
  callbacks,
  plugin package names,
  command arrays,
  or executable-path overrides,
  and lines 103 to 106 fix the registry to shipped policies.
  The rewrite scope lines 113 to 115 record the user's rejection of a generic external-policy extension.
- Rust owner:
  `config_parse.rs` (exists),
  for the rejection.
- Consumer-level test:
  in the standard fixture,
  write a JSONC document with a `plugins` key;
  observe a `config-invalid` exit `2` naming the retired key.
- Native state:
  implemented as library code:
  `unknown_top_level_key` (`src/native/config_parse.rs:49-63`) returns a specific message
  for `plugins` and for `trust`.
  The native registry keeps this repository's namespaced names as fixed policy names:
  `markdown/autofix`,
  `mono/forbidden-root-context`,
  `mono/dependent-version-bump`,
  and `security/forbidden-strings` (`src/native/policy_registry.rs:95-160`).

## Trust subsystem

Every entry in this section is retired by the implementation plan lines 71 to 75
("Delete executable-config discovery,
bundling,
stored-code evaluation,
consent,
and trust-registry management")
and by the rewrite scope lines 115 and 121 to 124
(the user wants the executable-config trust subsystem removed
and no replacement trust registry or global policy layer).
Retained modules still import helpers from `src/trust/`:
17 files import `src/trust/registry-io.ts`
(`rg --files-with-matches "trust/registry-io" src --glob '!src/trust/**' --glob '!**/*.unit.test.ts'`),
and `src/bin.ts:35`,
`44` and `src/policy-engine/commit-transaction.ts:22` import classification and concurrency helpers.
The implementation plan lines 78 to 80 move those helpers by responsibility.

### Trust candidate capture

- Behavior:
  `captureTrustCandidate` (`src/trust/candidate.ts:77-163`) opens the configuration without following links
  and resolves filesystem identity,
  through `/proc/<pid>/fd/<fd>` on Linux (`53`).
- Spec:
  `SPEC.md:1752-1762`.
- Consumers:
  trust commands and strict loading.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  covered by the retired-command test under "Trust management commands".
- Native state:
  absent.

### MJS self-containment validation

- Behavior:
  `validateMjs` (`src/trust/mjs-validator.ts:58-197`) parses module syntax without execution
  and rejects imports other than Node built-ins.
- Spec:
  `SPEC.md:1736-1750`.
- Consumers:
  MJS trust.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  none;
  no JavaScript is parsed or executed by the native wrapper.
- Native state:
  absent.

### TypeScript bundling and source capture

- Behavior:
  `buildTypeScriptCandidate` (`src/trust/typescript-builder.ts:95-243`) bundles the configuration with Rolldown
  into one chunk;
  `sourceCapturePlugin` (`src/trust/typescript-source-capture.ts:204-344`) records the tracked source graph.
- Spec:
  `SPEC.md:1917-1966`.
- Consumers:
  TypeScript trust for the root `cli-git.config.ts`.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  none.
- Native state:
  absent.

### Stored-code evaluation

- Behavior:
  `loadStrictMjs` and `loadStrictTypeScript` (`src/trust/config-loader.ts:180-350`) compare live bytes
  with stored snapshots and import the stored executable snapshot (`107-170`).
- Spec:
  `SPEC.md:1764-1780`.
- Consumers:
  every configuration-loading wrapper command.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  in the standard fixture,
  run a configuration-loading command with `strace` following children;
  observe that no Node or other interpreter starts for configuration.
- Native state:
  absent.

### Relaxed trust mode

- Behavior:
  `loadRelaxedConfig` (`src/trust/relaxed-loader.ts:272-303`) rebuilds or re-snapshots a trusted configuration
  when `CLI_GIT_NO_PARANOID` names it (`src/trust/trust-service.ts:182`;
  `src/trust/relaxed-paths.ts:138-166`).
- Spec:
  `SPEC.md:1881-1915`.
  The spec does not name the environment variable.
- Consumers:
  developers who set the variable.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  in the standard fixture,
  set `CLI_GIT_NO_PARANOID` and run a wrapped command;
  observe no change in behavior.
- Native state:
  absent.

### Trust consent protocol

- Behavior:
  `trustConfig` (`src/trust/trust-service.ts:66-176`) discloses the candidate on stderr
  and accepts an interactive affirmative response or `--yes`;
  `trust.children` adds a second stage for recursive authority.
- Spec:
  `SPEC.md:1782-1807`.
- Consumers:
  `git cli-git trust`.
- Status:
  retired.
  The implementation plan lines 68 to 69 add that repository settings can disable policies
  or change permitted severities without consent,
  and that no other approval mechanism replaces the prompts.
- Rust owner:
  none.
- Consumer-level test:
  in the standard fixture,
  change `cli-git.config.jsonc` to disable a policy and run the affected command;
  observe that the change takes effect with no prompt and no trust failure.
- Native state:
  absent.

### Trust registry storage

- Behavior:
  records live under an account-derived root (`src/trust/account-root.ts:35-65`)
  at reversible base64url paths (`src/trust/registry-path.ts:41-219`),
  with private modes on POSIX and protected ACLs on Windows (`src/trust/registry-io.ts:25-29`,
  `130-247`),
  and atomic prepared-record installation (`src/trust/registry-record-preparation.ts:168-303`).
- Spec:
  `SPEC.md:1624-1734`.
- Consumers:
  trust loading and management.
- Status:
  retired.
  The implementation plan lines 444 to 445 leave old trust records inert rather than deleting them.
- Rust owner:
  none.
- Consumer-level test:
  in the standard fixture,
  seed a trust registry under a disposable home and run wrapped commands;
  observe the registry bytes unchanged.
- Native state:
  absent.

### Recursive enrollment, revocation, and provenance journals

- Behavior:
  `autoEnrollRecursiveConfig` (`src/trust/recursive-enrollment.ts:272-316`) enrolls descendants of a recursive root;
  `revokeRecursiveTrust` (`src/trust/recursive-revocation.ts:168-262`) removes inherited records;
  provenance journals are applied and recovered in `src/trust/registry-transaction.ts:135-291`
  under the recursive-operation lock (`src/trust/registry-recursive-lock.ts:159-259`).
- Spec:
  `SPEC.md:1809-1879`,
  `3339-3356`.
- Consumers:
  trust loading in every wrapped command.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  none beyond the inert-registry test under "Trust registry storage".
- Native state:
  absent.

### Trust management commands

- Behavior:
  `runTrustManagement` (`src/trust/management-runtime.ts:177-294`) implements `trust`,
  `untrust`,
  and `status`,
  each writing one JSON object to stdout.
- Spec:
  `SPEC.md:799-885`.
- Consumers:
  humans and agents enrolling a repository;
  `.github/workflows/cli-git-trust.yml:189-232` (job `trust`) runs the trust suites on Linux,
  macOS,
  and Windows.
- Status:
  retired.
  The implementation plan line 444 states that retired trust commands explain
  that JSONC no longer requires code-execution approval.
- Rust owner:
  `management_retired.rs` (proposed) for the explanation.
- Consumer-level test:
  in the standard fixture,
  run `git cli-git trust`,
  `git cli-git trust --yes`,
  `git cli-git untrust`,
  and `git cli-git status`;
  observe the explanation and that no registry file is read or written.
  The exit status and stream are undetermined;
  see "Open questions".
- Native state:
  in progress
  (the handover lists a management-command skeleton).

### Trust warnings

- Behavior:
  trust warnings are JSON objects on stderr with the codes at `src/trust/types.ts:166-189`.
- Spec:
  `SPEC.md:1309-1325`.
- Consumers:
  none measured outside the package.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  none.
- Native state:
  absent.

## Authoring surfaces

Every entry in this section is retired by the implementation plan lines 33 to 35
(no permanent TypeScript bridge and no general plugin runtime),
lines 63 to 67,
and lines 295 to 297
(retired implementations and authoring surfaces are deleted only after their consumers have moved).

### Authoring API

- Behavior:
  `definePolicy`,
  `definePlugin`,
  `defineConfig`,
  and `definePolicyOptions` return their argument (`src/api/authoring.ts:37-116`);
  the types are in `src/api/config-types.ts`,
  `src/api/policy-types.ts`,
  `src/api/context-types.ts`,
  and `src/api/policy-input-types.ts`.
  The canonical copy is `package/git-policy/api/src`.
- Spec:
  `SPEC.md:77-398`,
  `628-716`.
- Consumers:
  `cli-git.config.ts:6-12`;
  `package/git-policy/repository`,
  `package/git-policy/forbidden-strings`,
  and `package/git-policy/markdown-lint`,
  which define policies with it.
- Status:
  retired.
- Rust owner:
  none.
- Consumer-level test:
  after cutover,
  `rg "git-policy-cli/ts|git-policy-api" . --glob '!doc/**'` returns no importer.
- Native state:
  absent.

### Package exports and single MJS artifact

- Behavior:
  `package/git-policy/cli/package.json:8-16` exports the built artifact and `./ts`;
  `src/authoring.ts:6-99` re-exports the authoring API,
  the optional plugins,
  and final-newline helpers.
  `package/git-policy/cli/rolldown.node.config.ts` builds one static bundle.
- Spec:
  `SPEC.md:48-75`,
  `4254-4283`.
- Consumers:
  `package/config/pnpr/config.yaml:76-80` lists the five `git-policy-*` packages for the private registry;
  `.changeset/config.json:21` names the wrapper package;
  the mise tasks `build`,
  `pack:npm`,
  and `test:built:trust` (`package/git-policy/cli/mise.toml:25-131`).
- Status:
  retired.
  The implementation plan lines 29 to 32 keep the TypeScript executable available
  until the Rust replacement passes the consumer contracts.
- Rust owner:
  none;
  native packaging is implementation sequence step 7.
- Consumer-level test:
  install the native package into a disposable project;
  observe that `PATH` resolves its `git` first and that no JavaScript artifact is installed.
- Native state:
  absent.

### Optional-policy mirrors

- Behavior:
  `file-enforcer.config.ts:2260-2322` copies the sources of `package/git-policy/repository`,
  `package/git-policy/forbidden-strings`,
  and `package/git-policy/markdown-lint` into `src/optional/`,
  rewriting their API imports,
  so the single artifact bundles them.
- Spec:
  `SPEC.md:62-75`.
- Consumers:
  the wrapper build;
  `package/file-enforcer/core/src/generated-policy-contracts.unit.test.ts`.
- Status:
  retired.
  The implementation plan line 435 lists file-enforcer-owned scanner and build integration
  among the things updated together at cutover.
- Rust owner:
  none.
- Consumer-level test:
  after cutover,
  running file-enforcer produces no file under `package/git-policy/cli/src/optional/`.
- Native state:
  absent.

### Internal test exports

- Behavior:
  `src/index.ts:11-37` exports `internalTestExports` and manual-push helpers
  so built-artifact tests reach internals (`src/internal-test-exports.ts:202-236`).
- Spec:
  no `SPEC.md` section states it.
- Consumers:
  unit tests under `src/` that exercise the built artifact
  (`rg --files-with-matches "internalTestExports|internal-test-exports" package` lists files under `src/` only).
- Status:
  retired with the JavaScript artifact.
  The behaviors those tests cover keep their own entries in this ledger.
- Rust owner:
  none.
- Consumer-level test:
  none.
- Native state:
  absent.

### Supported Node runtime contract

- Behavior:
  `package/git-policy/cli/package.json:30-32` declares `^24.11.0 || ^26.0.0`;
  `src/runtime-contract.host-evidence.ts` imports the built artifact and runs representative commands
  at each floor.
- Spec:
  `README.md:6-19`.
- Consumers:
  `.github/workflows/cli-git-trust.yml:34-187`
  (jobs `runtime-policy`,
  `latest-lts-policy`,
  and `supported-runtime`);
  the mise task `verify:supported-runtime` (`package/git-policy/cli/mise.toml:84-86`).
- Status:
  retired with the Node executable.
  The rewrite scope lines 104 to 106 state that eliminating an external Node installation
  is not a hard requirement,
  and the implementation plan lines 425 to 426 remove Node from the launch path.
- Rust owner:
  none.
- Consumer-level test:
  none.
- Native state:
  absent.

## Management commands

### Namespace dispatch, grammar, and help

- Behavior:
  the wrapper owns an invocation only when the parsed subcommand is exactly `cli-git` (`src/bin.ts:147-148`).
  `parseManagementArgs` (`src/management-parser.ts:175-265`) accepts `--help` or `-h`,
  `trust [--yes]`,
  `untrust`,
  `status`,
  and `check` or `fix` with repeatable `--policy <id>`,
  `--all`,
  and pathspecs.
  `runManagementCommand` (`src/management.ts:136-316`) prints usage to stderr and exits `2` on a refusal (`149-152`),
  prints help to stdout and exits `0` (`153-156`),
  and requires exactly one scope for `check` and `fix`:
  `--all`,
  or a non-empty pathspec list after `--` (`171-198`).
  Help returns before real-Git resolution and recovery (`src/bin.ts:159-174`).
- Spec:
  `SPEC.md:796-833`.
- Consumers:
  humans and agents running `git cli-git`;
  `.github/workflows/final-newline.yml:38-39` runs a fixture that calls the direct check.
- Status:
  retained for the namespace,
  help,
  `check`,
  and `fix`.
  The implementation plan lines 107 to 112 preserve `check` and `fix`,
  and line 438 lists the surviving management-command documentation.
- Rust owner:
  `management_arguments.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git cli-git --help`,
  `git cli-git check`,
  `git cli-git check --all -- a.txt`,
  `git cli-git check a.txt`,
  and `git add cli-git`;
  observe help on stdout with exit `0`,
  exit `2` for the three malformed scopes,
  and ordinary staging of a file named `cli-git`.
- Native state:
  in progress
  (management-command skeleton).

### Direct check

- Behavior:
  `git cli-git check` loads configuration with `forceLoad` (`src/management.ts:99-118`),
  projects the selected worktree bytes into a private index
  (`src/policy-engine/direct-check-facts.ts:66-111`),
  runs the engine with the `direct-check` trigger and the optional `--policy` filter (`src/management.ts:291-307`),
  writes events to stdout,
  and returns the engine's exit code (`311-315`).
  `--all` becomes the pathspec `:/` (`234-236`).
- Spec:
  `SPEC.md:1209-1213`.
- Consumers:
  `src/trust/fixture/final-newline-workflow.ts`,
  run by `.github/workflows/final-newline.yml:38-39`;
  agents checking policies without committing.
- Status:
  retained.
- Rust owner:
  `management_check.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  write a worktree file that violates one policy and differs from its staged copy,
  then run `git cli-git check --policy final-newline -- a.txt`;
  observe one `finding` event on stdout for the worktree bytes,
  exit `0` or `1` by severity,
  and unchanged index and worktree bytes.
- Native state:
  absent.

### Direct fix

- Behavior:
  `runDirectFix` (`src/policy-engine/direct-fix.ts:238-272`) holds the landing lock (`181`),
  snapshots the real index,
  converges patches on private candidate state
  (`src/policy-engine/direct-fix-convergence.ts:103-309`,
  at most 8 changed passes,
  `33`),
  replaces only changed worktree files (`src/policy-engine/direct-fix-install.ts:230-339`),
  verifies that every real index blob is unchanged,
  and emits findings plus one `fix-summary`.
- Spec:
  `SPEC.md:1215-1227`,
  `837`.
- Consumers:
  humans and agents applying fixes;
  the `dependent-version-bump` policy declares the `direct-fix` trigger for this command
  (`src/optional/repository-policy/dependent-version-bump-policy.ts:253-258`).
- Status:
  retained.
- Rust owner:
  `management_fix.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  stage one version of a file,
  leave a different fixable version in the worktree,
  and run `git cli-git fix -- a.txt` while a concurrent wrapped commit lands;
  observe the fixed worktree bytes,
  a byte-identical real index,
  one `fix-summary` on stdout,
  and exit `0`.
- Native state:
  absent.

## Policy engine

### Registry, order, and triggers

- Behavior:
  `BUILT_IN_POLICIES` (`src/policy-engine/built-ins.ts:21-27`) fixes the built-in order:
  `require-root`,
  `linked-worktree-only`,
  `branch-worktree-only`,
  `add-explicit`,
  `final-newline`.
  `runPolicyEngine` (`src/policy-engine/engine.ts:157-420`) runs built-ins (`279-311`),
  then fixed transforms for the `pre-forward` trigger only (`315-326`),
  then plugin policies in registration order (`354-375`).
  A policy runs only for the triggers it declares (`src/policy-engine/policy-stage.ts:135-137`);
  the trigger set is `pre-forward`,
  `post-commit`,
  `manual-push`,
  `direct-check`,
  and `direct-fix` (`src/api/policy-types.ts:31-36`).
- Spec:
  `SPEC.md:1229-1244`,
  `3992-3999`.
- Consumers:
  every lifecycle.
- Status:
  retained as a fixed typed registry of shipped policies with stable ordering
  (implementation plan lines 103 to 106).
- Rust owner:
  `policy_registry.rs` (exists) for the table;
  `policy_engine.rs` (proposed) for execution.
- Consumer-level test:
  in the standard fixture,
  make one command violate two policies and pass `--cli-git-keep-going`;
  observe the findings in registry order with increasing `sequence`.
- Native state:
  implemented for the table only:
  `POLICY_REGISTRY` (`src/native/policy_registry.rs:95-160`) lists nine policies in the incumbent order,
  built-ins first.
  It records no triggers.
  Evidence:
  4 unit tests in `src/native/policy_registry_tests.rs`;
  no recorded gate run.
  Execution is absent.

### Severities, defaults, and unsafe warnings

- Behavior:
  the effective severity is the configured value or the policy default (`src/policy-engine/policy-stage.ts:145`);
  `off` skips the policy (`146-147`).
  A `warn` setting on a policy that is not warn-safe emits a `configuration-warning` event,
  even when the check is clean (`151`,
  `209-214`).
  Defaults:
  `final-newline` is `warn`;
  the other four built-ins are `error`
  (`src/policy-engine/final-newline-policy.ts:88`;
  `src/policy-engine/require-root-policy.ts:24`;
  `src/policy-engine/linked-worktree-policy.ts:21`;
  `src/policy-engine/branch-worktree-policy.ts:21`;
  `src/policy-engine/add-explicit-policy.ts:21`).
  `branch-worktree-only` and `final-newline` are warn-safe.
- Spec:
  `SPEC.md:1352-1366`,
  `4000-4004`.
- Consumers:
  the root configuration sets three severities (`cli-git.config.ts:33-57`).
- Status:
  retained.
  The implementation plan lines 68 to 69 let repository settings disable policies
  or change permitted severities without consent.
- Rust owner:
  `policy_registry.rs` and `config_policies.rs` (exist).
- Consumer-level test:
  in the standard fixture,
  set `add-explicit` to `warn` and run `git add .`;
  observe a `finding` with severity `warn`,
  a `configuration-warning` with code `warn-unsafe`,
  and that Git staged the files.
- Native state:
  implemented as data:
  each `PolicyDescriptor` holds `default_severity` and `warn_safe`
  (`src/native/policy_registry.rs:69-160`).
  The five built-ins carry the incumbent defaults.
  The four optional policies default to `Off` there,
  while the incumbent runs a registered plugin's policy at the policy's own default
  (`src/policy-engine/policy-stage.ts:145`);
  see "Open questions".
  No event is emitted yet.

### Stage execution and stopping

- Behavior:
  `runPolicyStage` (`src/policy-engine/policy-stage.ts:103-257`) runs one policy at a time.
  Without `--cli-git-keep-going`,
  the first error finding stops the stage (`223-231`).
  A thrown check emits `plugin-threw` and stops (`162-177`);
  an invalid finding emits `policy-incomplete` and stops (`183-184`,
  `233-248`).
  A proposed patch ends the stage so convergence can restart (`215-222`).
  The direct-check filter and escaped IDs skip policies (`138-141`).
- Spec:
  `SPEC.md:387-398`,
  `1246-1252`.
- Consumers:
  every lifecycle.
- Status:
  retained.
  `plugin-threw` loses its plugin meaning with a fixed registry;
  see "Open questions".
- Rust owner:
  `policy_engine.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run a command that violates the first and third policies,
  with and without `--cli-git-keep-going`;
  observe one finding and exit `1` without the flag,
  and both findings and exit `1` with it.
- Native state:
  absent.

### Finding validation

- Behavior:
  `findingsAreValid` (`src/policy-engine/policy-stage.ts:60-71`) requires a non-empty `code` and `message`.
  `createFindingEvent` prefixes the code with the policy ID (`src/policy-engine/events.ts:410`).
- Spec:
  `SPEC.md:562-574`.
  The spec also requires kebab-case codes and an in-range byte location;
  the engine checks neither
  (see "Spec and code disagreements").
- Consumers:
  JSONL consumers.
- Status:
  retained as structured findings from typed policies
  (implementation plan lines 103 to 106).
- Rust owner:
  `policy_findings.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  trigger a finding that carries a location;
  observe `byteStart` and `byteEnd` inside the candidate's byte length
  and a code of the form `<policy-id>/<code>`.
- Native state:
  absent.

### Patch validation and application

- Behavior:
  `validatePolicyPatch` (`src/policy-engine/commit-transaction-patch.ts:122-209`) accepts one textual unified diff
  for exactly the declared path,
  with one `index` header naming the candidate's blob revision and mode `100644` or `100755` (`62-106`),
  and rejects rename,
  copy,
  mode,
  and binary directives (`21-32`,
  `203-207`) and paths with traversal or line breaks (`134-143`).
  `applyPolicyPatches` (`src/policy-engine/apply-policy-patches.ts:247-342`) applies patches in order
  through `git apply --cached --3way` on the private index
  (`src/policy-engine/commit-transaction-git.ts:248-291`).
  The built-in and optional policies build patches with
  `createFinalNewlinePatch` (`src/policy-engine/final-newline-patch.ts:70-129`)
  and `createFullContentPatch` (`src/optional/markdown-lint/full-content-patch.ts:114-171`).
- Spec:
  `SPEC.md:576-626`.
- Consumers:
  commit transactions and direct fix.
- Status:
  retained.
  The patch is no longer supplied by untrusted plugin code,
  but the candidate edit path stays
  (implementation plan lines 103 to 112).
- Rust owner:
  `policy_patches.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit a file whose fix conflicts with a concurrent edit of the same lines;
  observe exit `2` with `patch-conflict`,
  no patch bytes in the event,
  and unchanged ref,
  index,
  and worktree bytes.
- Native state:
  absent.

### Bounded fix passes

- Behavior:
  `convergeCommitPolicies` (`src/policy-engine/commit-transaction-convergence.ts:110-276`) restarts the whole order
  after each changed candidate state,
  with at most 8 changed passes (`32`).
  Candidate states are serialized to private files and compared byte for byte
  (`src/policy-engine/commit-transaction-candidate-snapshot.ts:82-247`).
  A repeated non-adjacent state is `fix-cycle`;
  a changed last pass is `fix-pass-limit`
  (`src/policy-engine/commit-transaction-results.ts:128-157`,
  `233-248`).
  Direct fix uses the same limit (`src/policy-engine/direct-fix-convergence.ts:33`).
- Spec:
  `SPEC.md:1254-1277`.
- Consumers:
  commit transactions,
  revalidation after replay,
  and direct fix.
- Status:
  retained
  (implementation plan line 106,
  "bounded fixing passes").
- Rust owner:
  `policy_convergence.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit a file that needs two different policy fixes;
  observe one `fix-summary` with `passes` equal to the changed passes,
  the committed bytes fully fixed,
  and no finding for the corrected problems.
- Native state:
  absent.

### JSONL events

- Behavior:
  events are compact JSON objects,
  one per line,
  with `schemaVersion: 1` (`src/policy-engine/events.ts:23`,
  `588-598`).
  Types:
  `finding` (`33`),
  `engine-failure` with 15 codes (`88-103`,
  `113`),
  `commit-landed` (`156`),
  `core-finding` (`195`),
  `configuration-warning` (`250`),
  `fix-summary` (`289`),
  and the concurrency events `landing-race-lost`,
  `landing-reserved`,
  `commit-replayed`,
  and `replay-headers-dropped` (`src/policy-engine/events-concurrency.ts:33-178`).
  `withFixSummary` appends the summary with sorted unique paths (`src/policy-engine/fix-summary.ts:28-100`).
- Spec:
  `SPEC.md:1279-1575`.
- Consumers:
  agents and scripts reading wrapper stderr or management stdout;
  the end-to-end suite checks exit codes against events
  (`e2e/jsonl-event-fixture.ts`).
- Status:
  retained
  (implementation plan lines 107 to 112).
  The trust-only codes `config-untrusted`,
  `config-changed`,
  `trust-consent-unavailable`,
  and `trust-failed` lose their emitters.
- Rust owner:
  `policy_events.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  drive one event of each retained type and parse stderr line by line;
  observe field sets equal to `SPEC.md:1296-1575`,
  no JSON `null`,
  and LF termination.
- Native state:
  absent.

### Stream routing and exit codes

- Behavior:
  wrapper policy events go to stderr (`src/bin.ts:219-224`,
  `277-280`,
  `336-339`,
  `412-415`,
  `433-438`);
  `check` and `fix` events go to stdout (`src/management.ts:219-224`,
  `261-265`,
  `311-314`).
  When Git does not run,
  exit `0` means clean or warnings only,
  `1` means error findings,
  and `2` means an engine failure (`src/policy-engine/engine.ts:308`,
  `347`,
  `392`,
  `417`).
  A landed commit followed by a blocked post-commit gate exits `2` (`src/bin.ts:416-417`).
- Spec:
  `SPEC.md:1577-1622`.
- Consumers:
  every caller that branches on the exit status or separates Git output from events.
- Status:
  retained
  (implementation plan lines 107 to 112).
- Rust owner:
  `policy_events.rs` and `wrapper_main.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  capture stdout and stderr separately for a blocked wrapped command and for `git cli-git check`;
  observe events only on stderr for the first and only on stdout for the second,
  with the exit codes of `SPEC.md:1588-1622`.
- Native state:
  absent.

### Candidate facts

- Behavior:
  each lifecycle builds lazy Git facts for the policy context:
  `git add` predicts the staged delta in a private index
  (`src/policy-engine/add-policy-facts.ts:116-261`;
  `src/policy-engine/add-staged-delta.ts:38-216`);
  commit transactions read the private commit index
  (`src/policy-engine/commit-transaction-candidates.ts:316-367`)
  and tracked files (`src/policy-engine/commit-transaction-tracked-files.ts:166-294`);
  post-commit reads the landed commit's delta
  (`src/policy-engine/post-commit-facts.ts:130-198`);
  manual push reads newly published content
  (`src/policy-engine/manual-push-candidates.ts:330-426`);
  direct commands read selected worktree bytes
  (`src/policy-engine/direct-check-facts.ts:66-111`).
  Blob bytes load through one `git cat-file --batch` process (`src/policy-engine/blob-batch.ts:205-270`).
- Spec:
  `SPEC.md:336-385`,
  `597-605`.
- Consumers:
  every content policy.
- Status:
  retained.
  The implementation plan lines 93 to 99 require immutable candidate versions,
  lazy batched Git facts,
  candidate bytes rather than live worktree bytes,
  and no Git process per file.
- Rust owner:
  `candidate_facts.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  stage clean bytes,
  then write a forbidden string to the worktree copy only,
  and commit the staged path with `--no-only`;
  observe a clean commit,
  and with `strace` a fixed number of Git processes for 1 and for 100 candidate files.
- Native state:
  absent.

### Policy read sets and input fingerprints

- Behavior:
  inside a commit transaction each policy run records what it read:
  the candidate list,
  the paths whose bytes it loaded,
  tracked-file requests,
  and the `headOid`,
  `landedCommitOid`,
  and `pushUpdates` values (`src/policy-engine/policy-read-set.ts:126-159`,
  `400-494`).
  Declared external inputs are fingerprinted:
  worktree pathspecs,
  executables,
  revisions,
  and environment variables
  (`src/policy-engine/policy-input-fingerprint.ts:367-448`;
  `src/policy-engine/policy-input-executable.ts:215-261`).
  After a replay,
  `decideRerun` (`src/policy-engine/policy-read-tracking.ts:253-310`) reuses a recorded run
  when it proposed no patch,
  its inputs hold,
  and its reads replay to the same identities
  (`src/policy-engine/policy-read-validation.ts:254-293`).
  Unrestricted policies always re-run.
- Spec:
  `SPEC.md:400-560`.
- Consumers:
  revalidation after replay.
- Status:
  retained
  (implementation plan lines 215 to 219 list policy read sets;
  lines 143 to 146 require a replay that changes a declared input to invalidate the policy result).
  The authoring-facing `inputs` declaration and its schema
  (`src/trust/policy-inputs-schema.ts:163-330`) retire with the authoring API;
  the shipped policies' declarations become fixed data.
- Rust owner:
  `policy_read_set.rs` and `policy_inputs.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  force a lost landing race with a disjoint winning commit,
  then with a winning commit that changes the scanner rules file;
  observe through debug logs or a counting scanner fixture that the context-only policy is reused in the first case
  and that the scanner re-runs in the second.
- Native state:
  absent.

### Pre-forward lifecycle

- Behavior:
  `runPreForwardPolicyEngine` (`src/policy-engine/pre-forward-engine.ts:34-77`) strips wrapper controls,
  builds add candidate facts when the command is `git add`,
  and runs the engine.
  A blocking result prevents forwarding (`src/bin.ts:273-286`);
  the transformed arguments replace the raw ones (`291`).
- Spec:
  `SPEC.md:889-899`.
- Consumers:
  every wrapped command that loads configuration and is not a commit transaction.
- Status:
  retained.
- Rust owner:
  `lifecycle_pre_forward.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git add -- CONTEXT.md` at the repository root;
  observe a `forbidden-root-context` finding on stderr,
  exit `1`,
  and an unchanged index.
- Native state:
  absent.

### Post-commit lifecycle

- Behavior:
  `runPostCommitLifecycle` (`src/policy-engine/post-commit-lifecycle.ts:101-195`) uses the landed commit ID
  passed from the transaction,
  runs `post-commit` policies against the landed delta,
  and on a blocking result appends `commit-landed` with outcome `post-commit-blocked` (`160-171`).
  A setup failure emits `content-unavailable` plus `commit-landed` (`173-194`).
  Only a clean or warning-only result reaches auto-push (`src/bin.ts:416-422`).
- Spec:
  `SPEC.md:1075-1092`,
  `1558-1575`.
- Consumers:
  auto-push,
  which this gate guards.
- Status:
  retained.
- Rust owner:
  `lifecycle_post_commit.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  land a commit whose content passes pre-forward only because of a one-time escape flag;
  observe the commit present locally,
  the causal finding,
  a final `commit-landed` event,
  exit `2`,
  and no push to the bare remote.
- Native state:
  absent.

### Manual-push lifecycle

- Behavior:
  `runManualPushGate` (`src/policy-engine/manual-push-gate.ts:45-87`) applies to `git push`
  that is not a dry run and has an enabled `manual-push` policy.
  `probeManualPushUpdates` (`src/policy-engine/manual-push-probe.ts:184-208`) runs a private
  `--dry-run` push whose generated `pre-push` hook records Git's update records
  (`src/policy-engine/manual-push-hook.ts:344-405`),
  and validates remote values with `git ls-remote --refs`.
  The published set comes from remote-tracking refs and prior values
  (`src/policy-engine/manual-push-published.ts:105-304`);
  candidates are each newly published commit's delta,
  or the final tree when nothing is known to be published
  (`src/policy-engine/manual-push-descriptors.ts:204-392`;
  `src/policy-engine/manual-push-candidates.ts:330-426`),
  with at most 4 concurrent per-update processes (`37`).
- Spec:
  `SPEC.md:1169-1207`,
  `4020-4041`.
- Consumers:
  `forbidden-strings`,
  `final-newline`,
  and Markdown autofix declare the `manual-push` trigger;
  `perf/manual-push-latency-benchmark.ts` measures it.
- Status:
  retained
  (implementation plan lines 107 to 109 name manual-push checks).
  The generated `pre-push` hook is a Node program today;
  its native form is undetermined
  (see "Open questions").
- Rust owner:
  `lifecycle_manual_push.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  push a branch whose newest commit adds a forbidden string,
  first to a remote that already has its parent and then to an empty remote;
  observe a blocked push with exit `1` in both cases,
  a scan of only the newest commit's delta in the first,
  and a bounded process count for a 1,000-commit history.
- Native state:
  absent.

## Built-in policies and fixed transforms

### Git command region parsers

- Behavior:
  `parseArgv` (`src/parser/argv.ts:240-415`) is a shared region parser over declared flags and value options.
  `expandAbbreviations` (`src/abbrev.ts:52-114`) lists the unambiguous long-option abbreviations Git accepts.
  Per-command parsers produce the facts the rules use:
  `parseAddRegion` (`src/parser/add.ts:258-313`),
  `parseCommitRegion` (`src/parser/commit.ts:212-291`) with `normaliseCommitArgs`
  (`src/parser/commit-normalise.ts:165-288`),
  `parsePushRegion` (`src/parser/push.ts:64-96`),
  `parseStatusPreRegion` and `parseStatusPostRegion` (`src/parser/status.ts:83-157`),
  `parseStashRegion` (`src/parser/stash.ts:96-128`),
  `parseCleanRegion` (`src/parser/clean.ts:144-232`) with `scanCleanOptionOrder`
  (`src/parser/clean-option-order.ts:430-438`),
  `parseResetRegion` (`src/parser/reset.ts:127-197`),
  and `parseBranchCreationRegion` (`src/parser/branch-create.ts:108-217`).
- Spec:
  `SPEC.md:777-797`;
  `doc/handover/cli-git-cac-migration.md` records why the package owns its parser.
- Consumers:
  every built-in policy and fixed transform,
  the commit transaction,
  and the manual-push gate.
- Status:
  retained.
  The implementation plan lines 47 to 49 require Git-specific token classification
  rather than an unrelated option grammar.
- Rust owner:
  `command_*.rs`,
  one module per command,
  owned by the command-parser delegate.
- Consumer-level test:
  in the standard fixture,
  run each documented abbreviation and clustered short form through the native wrapper and through `/usr/bin/git`
  for the same repository state;
  observe that the wrapper's decision matches what Git then does
  (for example `git commit --am` amends and `git clean --dry` deletes nothing).
- Native state:
  in progress
  (queued delegate;
  no `command_*.rs` file at `40436cd01`).

### require-root

- Behavior:
  `requireRoot` (`src/rule/require-root.ts:130-191`) rejects a command whose effective directory
  is inside a repository but not at its root.
  It exempts `init`,
  `clone`,
  `version`,
  and `help` (`24-29`),
  `config` with `--global`,
  `--system`,
  `--list`,
  or `-l` (`34-39`,
  `159-170`),
  and a directory outside any repository (`175-179`).
  Policy metadata:
  default `error`,
  warn-unsafe,
  triggers `pre-forward` and `direct-check` (`src/policy-engine/require-root-policy.ts:22-31`).
- Spec:
  `SPEC.md:3992-4008`.
- Consumers:
  every wrapped command.
- Status:
  retained
  (implementation plan lines 116 to 120).
- Rust owner:
  `rule_require_root.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git status` from a subdirectory,
  from the root,
  with `-C <root>`,
  and outside any repository;
  observe exit `1` with a `require-root` finding only in the first case.
- Native state:
  in progress
  (queued rule-core delegate).

### linked-worktree-only

- Behavior:
  `checkLinkedWorktree` (`src/policy-engine/linked-worktree-check.ts:237-308`) rejects every `git stash`,
  `git clean` that is neither a dry run nor interactive,
  and `git reset` with a destructive mode (`138-175`),
  unless the target is a linked worktree or an allowlisted tool cache (`298-307`).
  Messages are in `src/policy-engine/linked-worktree-messages.ts:52-109`.
  Policy metadata:
  default `error`,
  warn-unsafe,
  trigger `pre-forward` (`src/policy-engine/linked-worktree-policy.ts:19-25`).
- Spec:
  `SPEC.md:3992-4008`.
- Consumers:
  agents and humans in the main worktree;
  `e2e/README.md:525-528` records that it blocks lint-staged's `git stash` in a main worktree.
- Status:
  retained
  (implementation plan lines 116 to 120).
- Rust owner:
  `rule_linked_worktree.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git stash`,
  `git clean -fd`,
  `git clean -nd`,
  and `git reset --hard` in a main worktree and in a linked worktree;
  observe rejection with exit `1` only for the three destructive forms in the main worktree.
- Native state:
  in progress
  (queued rule-core delegate).

### Allowed worktree directories

- Behavior:
  `classifyEffectiveTarget` (`src/effective-target.ts:62-86`) returns `allowlisted`
  when the repository's Git directory lies under a baked-in tool cache.
  The only entry is uv's cache:
  `UV_CACHE_DIR`,
  or `XDG_CACHE_HOME/uv`,
  or `<home>/.cache/uv` (`src/allowed-worktree-dirs.ts:60-102`).
  Membership resolves each root through `realpath` and tests segment-aware containment (`144-164`,
  `247-281`).
- Spec:
  no `SPEC.md` section states it.
- Consumers:
  uv,
  whose internal `git reset --hard` in its cache would otherwise be rejected (`src/allowed-worktree-dirs.ts:43-48`).
- Status:
  retained.
  The implementation plan does not name it;
  it is part of the linked-worktree policy's accepted behavior.
- Rust owner:
  `allowed_worktree_directories.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  create a repository under a disposable `UV_CACHE_DIR` and one beside it with the same prefix,
  then run `git reset --hard` in each;
  observe success in the first and rejection in the second.
- Native state:
  absent.

### branch-worktree-only

- Behavior:
  `checkBranchWorktree` (`src/policy-engine/branch-worktree-check.ts:101-188`) rejects branch creation
  in the current worktree:
  explicit forms of `branch`,
  `checkout`,
  and `switch` (`145-146`),
  and the remote-tracking guess of `git switch <name>` and `git checkout <name>`,
  probed against real Git (`src/policy-engine/branch-worktree-remote-guess.ts:158-227`).
  Policy metadata:
  default `error`,
  warn-safe,
  trigger `pre-forward` (`src/policy-engine/branch-worktree-policy.ts:19-25`).
- Spec:
  `SPEC.md:3992-4008`.
- Consumers:
  agents and humans;
  the status note names the allowed form `git worktree add -b` (`src/post-command-output.ts:96-101`).
- Status:
  retained
  (implementation plan lines 116 to 120).
- Rust owner:
  `rule_branch_worktree.rs` (proposed).
- Consumer-level test:
  in the standard fixture with one remote branch `topic` and no local one,
  run `git switch -c x`,
  `git switch topic`,
  `git branch --list`,
  and `git worktree add -b y ../y`;
  observe rejection of the first two only.
- Native state:
  in progress
  (queued rule-core delegate).

### add-explicit

- Behavior:
  `checkAddExplicit` (`src/policy-engine/add-explicit-check.ts:77-137`) rejects `git add`
  with the bulk tokens `.`,
  `*`,
  `-A`,
  or `-u` (`src/parser/add.ts:23-41`),
  naming the matched tokens (`src/policy-engine/add-explicit-check.ts:121-133`).
  Policy metadata:
  default `error`,
  warn-unsafe,
  trigger `pre-forward` (`src/policy-engine/add-explicit-policy.ts:19-25`).
- Spec:
  `SPEC.md:3992-4008`.
- Consumers:
  agents and humans;
  `AGENTS.md:1338` (rule CLG).
- Status:
  retained
  (implementation plan lines 116 to 120).
- Rust owner:
  `rule_add_explicit.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git add .`,
  `git add -A`,
  `git add --pathspec-from-file -A` with a pathspec file named `-A`,
  and `git add -- a.txt`;
  observe rejection of the first two and staging by the last two.
- Native state:
  in progress
  (queued rule-core delegate).

### final-newline

- Behavior:
  `finalNewlinePolicy` (`src/policy-engine/final-newline-policy.ts:86-119`) checks each regular
  or executable candidate.
  `normalizeFinalNewline` (`src/policy-engine/final-newline-normalize.ts:108-133`) leaves empty,
  NUL-containing,
  and non-UTF-8 bytes unchanged,
  and otherwise requires exactly one terminal LF.
  `isFinalNewlineExcluded` (`72-93`) preserves five path families.
  Where patches apply,
  the finding carries a patch (`src/policy-engine/final-newline-policy.ts:60-74`).
  Policy metadata:
  default `warn`,
  warn-safe,
  all five triggers,
  no external inputs (`86-98`).
- Spec:
  `SPEC.md:4061-4087`.
- Consumers:
  every commit in this repository;
  `.github/workflows/final-newline.yml`.
- Status:
  retained
  (implementation plan lines 116 to 120).
- Rust owner:
  `rule_final_newline.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit a text file without a terminal LF,
  one with three,
  a CRLF file,
  a binary file,
  an empty file,
  and a file under `x/dist/final/node/`;
  observe exact committed bytes:
  the first three end in one LF with interior bytes untouched,
  and the others are unchanged.
- Native state:
  in progress
  (queued rule-core delegate).

### atomic-push transform

- Behavior:
  `atomicPush` (`src/rule/atomic-push.ts:48-89`) inserts `--atomic` after `push`
  unless the caller passed `--atomic` or `--no-atomic`.
- Spec:
  `SPEC.md:1242-1244`.
- Consumers:
  every wrapped `git push`.
- Status:
  retained
  (implementation plan lines 121 to 123).
- Rust owner:
  `rule_atomic_push.rs` (proposed).
- Consumer-level test:
  in the standard fixture with a remote hook that rejects one of two refs,
  run `git push origin a b` and `git push --no-atomic origin a b`;
  observe that the remote takes neither ref in the first case and one in the second.
- Native state:
  in progress
  (queued rule-core delegate).

### commit-only transform

- Behavior:
  `commitOnly` (`src/rule/commit-only.ts:222-418`) inserts `-o` after `commit`.
  It rejects `-a` or `--all` (`all-flag`,
  `302-306`),
  a pathless commit outside a merge,
  cherry-pick,
  or revert conclusion (`pathspec-required`,
  `319-343`),
  and a pathless `--amend` or `--allow-empty` while the index differs from `HEAD`
  (`staged-changes-ignored`,
  `363-393`).
  It skips insertion for an explicit `-o`,
  `--only`,
  or `--no-only` (`346-349`),
  for include,
  interactive,
  or patch selection (`351-357`),
  and for `--no-enforce-only` (`284-299`).
  The checks query real Git (`src/rule/commit-index-check.ts:65-110`;
  `src/rule/commit-sequencer-check.ts:108-176`).
  Rejections are `core-finding` events (`src/policy-engine/fixed-transforms.ts:123-133`).
- Spec:
  `SPEC.md:1368-1396`,
  `2516-2529`.
- Consumers:
  every wrapped `git commit`;
  `AGENTS.md:1338` (rule CLG) and rule CPN.
- Status:
  retained
  (implementation plan lines 121 to 123).
- Rust owner:
  `rule_commit_only.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  stage `b.txt`,
  then run `git commit -m x -- a.txt`,
  `git commit -m x`,
  `git commit -a -m x`,
  and `git commit --amend --no-edit`;
  observe that the first commits only `a.txt` and leaves `b.txt` staged,
  and that the other three exit `1` with the three `commit-only/*` codes.
- Native state:
  in progress
  (queued rule-core delegate).

### status-hints-off transform

- Behavior:
  `statusHintsOff` (`src/rule/status-hints-off.ts:105-137`) inserts `-c advice.statusHints=false`
  before `status` unless the caller set that key with a global `-c` (`56-71`).
- Spec:
  `SPEC.md:1242-1244`.
- Consumers:
  every wrapped `git status`.
- Status:
  retained
  (implementation plan lines 121 to 123).
- Rust owner:
  `rule_status_hints.rs` (proposed).
- Consumer-level test:
  in the standard fixture with one unstaged change,
  run `git status` and `git -c advice.statusHints=true status`;
  observe Git's `use "git add"` hint only in the second.
- Native state:
  in progress
  (queued rule-core delegate).

## Optional policies

### forbidden-strings

- Behavior:
  `forbiddenStringsPolicy` (`src/optional/forbidden-strings/index.ts:112-156`,
  canonical source `package/git-policy/forbidden-strings/src/index.ts`) scans candidate bytes.
  `materializeCandidates` writes candidate bytes to temporary files with at most 64 concurrent lanes
  (`src/optional/forbidden-strings/materialize-candidates.ts:21`,
  `97-236`);
  `scanCandidates` starts the scanner executable with an argument array
  (`src/optional/forbidden-strings/scan-candidates.ts:273-387`);
  `parseScannerOutput` turns exit `1` output into redacted findings
  (`src/optional/forbidden-strings/scanner-output.ts:323-361`).
  Options:
  `executable` (default `forbidden-strings`) and `builtinRules` (default `true`)
  (`src/optional/forbidden-strings/index.ts:40-49`).
  Declared inputs:
  the executable,
  `FORBIDDEN_STRINGS_RULES`,
  and the rules file (`78-101`).
  Policy metadata:
  default `error`,
  warn-unsafe,
  triggers `pre-forward`,
  `post-commit`,
  `manual-push`,
  and `direct-check` (`116-124`).
- Spec:
  `SPEC.md:4016-4059`.
- Consumers:
  the root configuration (`cli-git.config.ts:45-57`),
  which points `executable` at the Rust scanner under `package/forbidden-strings/cli/target/release/`;
  root `mise.toml:1321-1327` sets `FORBIDDEN_STRINGS_RULES`;
  `.github/workflows/forbidden-strings.yml` runs the standalone scanner independently.
- Status:
  retained
  (implementation plan lines 124 and 128 to 152).
  The scanner is linked in-process through its library interface;
  the temporary content files,
  the scanner child process,
  and stderr parsing are removed (lines 148 to 152).
  The `executable` option retires:
  the rewrite scope lines 126 to 131 forbid arbitrary program selection through JSONC.
- Rust owner:
  `policy_forbidden_strings.rs` (proposed),
  over the `Scanner` interface in `package/forbidden-strings/cli/src`.
- Consumer-level test:
  in the standard fixture,
  commit a file with a planted built-in-rule match and a clean control,
  with a different string in the worktree copy than in the staged copy;
  observe exit `1`,
  a redacted finding for the staged bytes only,
  no matched bytes in any output,
  and with `strace` no scanner child process and no temporary content file.
- Native state:
  absent in the wrapper.
  The scanner's embedding interface is implemented and verified
  (`doc/handover/scanner-native-verification.md`).

### forbidden-root-context

- Behavior:
  `forbiddenRootContext` (`src/optional/repository-policy/index.ts:114-148`,
  canonical source `package/git-policy/repository/src/index.ts`) reports a non-deleted candidate
  at exactly `CONTEXT.md` (`97-103`).
  Policy metadata:
  default `error`,
  warn-safe,
  triggers `pre-forward` and `direct-check`,
  no external inputs (`115-123`).
- Spec:
  `SPEC.md:4010-4014`.
- Consumers:
  the root configuration (`cli-git.config.ts:44`);
  `AGENTS.md` rule SK3 states the no-context-file convention it enforces.
- Status:
  retained
  (implementation plan line 125).
- Rust owner:
  `policy_forbidden_root_context.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit a new root `CONTEXT.md`,
  a nested `a/CONTEXT.md`,
  and a deletion of a root `CONTEXT.md`;
  observe rejection of the first only.
- Native state:
  absent.

### Dependent-version propagation

- Behavior:
  `dependentVersionBump` (`src/optional/repository-policy/dependent-version-bump-policy.ts:247-271`,
  canonical source `package/git-policy/repository/src`) runs for commits that modify a workspace manifest
  (`179-191`).
  It plans patch-level bumps for publishable dependents
  (`src/optional/repository-policy/dependent-bump-workflow.ts:297-391`;
  `src/optional/repository-policy/dependent-version-bump.ts:153-353`),
  reading manifests through `trackedFiles`,
  publishable names from `package/config/pnpr/config.yaml`
  (`src/optional/repository-policy/publishable-names.ts:11`,
  `39-82`),
  and source imports (`src/optional/repository-policy/source-imports.ts:39-267`).
  Each stale dependent is a finding with a full-content patch for a tracked file that is not a candidate,
  so the fix adds the path to the commit
  (`src/optional/repository-policy/dependent-version-bump-policy.ts:204-226`).
  Policy metadata:
  default `error`,
  warn-unsafe,
  triggers `pre-forward`,
  `direct-check`,
  and `direct-fix` (`247-258`).
  The root configuration registers the plugin but does not list this policy (`cli-git.config.ts:22-58`),
  so it runs at its default severity `error`.
  The native registry gives it the default `Off` (`src/native/policy_registry.rs:146-152`),
  so a translated configuration that omits it would stop the policy;
  see "Open questions".
- Spec:
  `SPEC.md:531-532`,
  `607-623`,
  `2531-2559`;
  `doc/planning/cli-git-policy-added-paths.md`.
- Consumers:
  every commit that bumps a workspace package version.
  The standalone task `//package/git-policy/repository:bump:dependents`,
  called by the root `changeset:version` task (`mise.toml:1193-1197`),
  shares the planning code (`package/git-policy/repository/src/bump-dependents-worktree.ts`).
- Status:
  retained
  (implementation plan line 125).
- Rust owner:
  `policy_dependent_version.rs` (proposed).
- Consumer-level test:
  in the standard fixture with three workspace manifests where `b` depends on `a`,
  bump `a` and commit only `a`'s manifest;
  observe that the landed commit also bumps `b`,
  that `b`'s worktree and index entries match the commit,
  and that a dirty `b` manifest instead yields `patch-conflict` with exit `2`.
- Native state:
  absent.

### Markdown autofix

- Behavior:
  `markdownLintPolicy` (`src/optional/markdown-lint/index.ts:93-141`,
  canonical source `package/git-policy/markdown-lint/src`) sends each Markdown candidate
  through the configured command in stdin fix mode
  (`src/optional/markdown-lint/rewrite-candidates.ts:594-632`),
  reports `markdown-autofix` with a full-content patch or `markdown-violation` (`34-39`),
  and treats exit `2` or any other status as a failure.
  Options:
  `command` (default `node package/cli/markdown-lint/src/cli.ts`),
  `rules` (default `lfs-image-url`),
  and `exclude` (`src/optional/markdown-lint/index.ts:46-81`).
  Policy metadata:
  default `warn`,
  warn-safe,
  all five triggers,
  unrestricted inputs (`97-109`).
- Spec:
  `SPEC.md:547-552`.
- Consumers:
  the root configuration (`cli-git.config.ts:33-43`),
  which runs only `lfs-image-url` and excludes `package/ssg/`.
- Status:
  retained as current commit-time Markdown normalization
  (implementation plan line 126).
  Lines 181 to 189 replace the configured command with the coordinated installation's native linter,
  called with `--config`,
  `--stdin`,
  `--stdin-filename`,
  and `--fix`;
  repository JSONC cannot choose the executable.
  The `command` option retires
  (rewrite scope lines 126 to 131).
- Rust owner:
  `policy_markdown.rs` (proposed),
  calling the binary of `package/linter/monochromatic-lint`.
- Consumer-level test:
  in the standard fixture with `.lfsconfig` and an LFS-tracked image,
  commit a Markdown file that links the image,
  including an astral-character and a BOM variant;
  observe the landed bytes with the rewritten object URL,
  one `fix-summary`,
  and an untouched file under an excluded path.
- Native state:
  absent in the wrapper.
  The linter rule port is pending:
  the handover work queue lists native LFS URL normalization as open.

## Commit transactions

Every entry in this section is retained by the implementation plan lines 198 to 237,
which port the transaction state machine and durable protocol explicitly
and list what to preserve.
One entry,
"Older-Git degradation",
is retired.
Native state is absent for every entry:
the handover work queue lists transactions,
hooks,
locks,
replay,
recovery,
worktree copy,
and auto-push as not started.

### Scope and applicability

- Behavior:
  `runCommitTransaction` (`src/policy-engine/commit-transaction.ts:87-402`) handles every `git commit`
  that is not a dry run (`109-116`);
  dry runs and short-circuit forms are forwarded (`src/bin.ts:248-250`).
  The mode is explicit-path or index
  (`src/policy-engine/commit-transaction-journal-states.ts:50`).
  `runCommitTransactionBoundary` (`src/policy-engine/commit-transaction-boundary.ts:34-65`) turns an unexpected error
  into `transaction-failed`,
  or `index-lock-unproven-owner`,
  with exit `2`,
  and rethrows native Git failures so their exit code is preserved.
  No configuration key or environment variable disables the transaction.
- Spec:
  `SPEC.md:1970-2010`.
- Consumers:
  every commit made through the wrapper,
  by humans and by concurrent agents in one worktree.
- Status:
  retained.
- Rust owner:
  `transaction.rs` (proposed) with typed states for capture,
  private preparation,
  policy convergence,
  landing,
  replay,
  and post-landing completion
  (implementation plan lines 200 to 206).
- Consumer-level test:
  in the standard fixture,
  run one explicit-path commit,
  one `--no-only` commit,
  one `commit -a --no-enforce-only`,
  and one `git commit --dry-run`;
  observe one landed commit for each of the first three with native parents and messages,
  no transaction directory afterward,
  and plain forwarding for the dry run.
- Native state:
  absent.

### Invocation capture

- Behavior:
  `captureInvocationLayout` (`src/policy-engine/commit-transaction-capture.ts:237-379`) records
  the symbolic `HEAD` target,
  the compare-and-swap target ref,
  the conclusion kind,
  the repository root,
  the common Git directory,
  the real index path,
  the ref storage format,
  and the reflog nonce;
  `captureInvocationBase` (`381-421`) reads the preparation base afterward.
  Conclusion detection and ref helpers are in
  `src/policy-engine/commit-transaction-capture-refs.ts:86-246`.
  A caller-set `GIT_INDEX_FILE`,
  `GIT_DIR`,
  `GIT_WORK_TREE`,
  `--git-dir`,
  and `--work-tree` are honored.
- Spec:
  `SPEC.md:2012-2062`.
- Consumers:
  every later transaction phase and recovery.
- Status:
  retained.
- Rust owner:
  `transaction_capture.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  start a commit held in `pre-commit`,
  move the branch with a second commit,
  and release the first;
  observe that the first replays onto the second rather than re-reading live `HEAD`,
  and that a detached `HEAD` commit lands by compare-and-swap on `HEAD`.
- Native state:
  absent.

### Transaction registry and journal

- Behavior:
  transactions live under `<git-dir>/cli-git-transactions/<uuid>/`
  (`src/policy-engine/commit-transaction-registry.ts:45`,
  `101`).
  A directory is built under a `.pending` name and published by rename after `owner.json` is complete (`60`,
  `259-336`);
  removal renames to `.retired` first (`65`,
  `338-451`).
  State files are created exclusively and never rewritten
  (`src/policy-engine/commit-transaction-journal-states.ts:378-416`):
  `preparing.json`,
  `prepared.json`,
  `index-lock-<n>.json`,
  `landing-<n>.json`,
  `ref-updated.json`,
  and the `index-installed` marker (`30-45`,
  `312-374`),
  at journal schema version 2 (`25`).
  Records are parsed strictly (`src/policy-engine/commit-transaction-journal-parse.ts:140-330`).
  `owner.json` holds the PID,
  birth identity,
  schema version 2,
  and invocation start time (`src/policy-engine/commit-transaction-owner.ts:28`,
  `41-65`,
  `85-120`).
- Spec:
  `SPEC.md:2071-2146`.
- Consumers:
  recovery,
  the starvation reservation,
  capture-order pruning.
- Status:
  retained.
  The implementation plan lines 228 to 233 keep the existing journal and lock formats where practical
  during the first release,
  with cross-version fixture verification.
- Rust owner:
  `transaction_registry.rs` and `transaction_journal.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  hold a commit in `pre-commit` and list the registry;
  observe directory mode `0700`,
  file mode `0600`,
  one published UUID directory with `owner.json` and `preparing.json`.
  Then feed journal files written by the incumbent to the native recovery;
  observe that each state is read without loss.
- Native state:
  absent.

### Private index and selection modes

- Behavior:
  `initializeCommitIndex` (`src/policy-engine/commit-transaction-index.ts:80-176`) builds `<tx>/commit.index`:
  from the preparation base plus the selected worktree paths for explicit-path commits,
  or a copy of the real index for index commits.
  `resolvePrivateCommitArgs` (`src/policy-engine/commit-transaction-selection.ts:149-193`) drops pathspecs,
  `--git-dir`,
  `--work-tree`,
  and the internal `--only`;
  pathspec files,
  including stdin and NUL forms,
  are materialized (`258-287`);
  interactive and patch selection run native Git once against the private index (`195-256`)
  and stay read-only for automatic fixes.
  Index copies keep the source index timestamps
  (`src/policy-engine/index-file-timestamps.ts:30-86`).
- Spec:
  `SPEC.md:2064-2069`,
  `2497-2529`.
- Consumers:
  preparation and landing.
- Status:
  retained
  (implementation plan line 210,
  "private indexes",
  and line 221,
  "real index and worktree isolation").
- Rust owner:
  `transaction_private_index.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit with partial staging and an unstaged tail,
  with `--pathspec-from-file=-` and `--pathspec-file-nul`,
  and with a racily clean same-size edit;
  observe exact committed,
  indexed,
  and worktree bytes,
  and that `git status` still shows the same-size edit.
- Native state:
  absent.

### Shadow repository

- Behavior:
  `createShadowRepository` (`src/shadow-repository/shadow-repository.ts:75-191`) creates
  `<git-common-dir>/cli-git/shadow/<transaction-id>`
  (`src/policy-engine/commit-transaction-capture.ts:477-490`):
  a private object store whose `info/alternates` names the real store,
  a snapshot of every real ref (`src/shadow-repository/shadow-refs.ts:158-397`,
  one `packed-refs` file for the files backend,
  one `update-ref --stdin` transaction for reftable),
  a generated `config` that includes the real one and pins `core.worktree`,
  `gc.auto`,
  and `maintenance.auto` (`src/shadow-repository/shadow-config.ts:63-222`),
  and links to every other common-directory entry (`src/shadow-repository/shadow-links.ts:53-322`).
  On Windows,
  shared directories are junctions and shared files are copies (`96-125`).
  Conclusion state is copied in (`src/shadow-repository/shadow-conclusion-state.ts:55-176`).
- Spec:
  `SPEC.md:2148-2406`.
- Consumers:
  native preparation,
  replay,
  hooks that read branch state.
- Status:
  retained
  (implementation plan line 210).
- Rust owner:
  `shadow_repository.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  hold a commit in `pre-commit` and from the hook print the branch name,
  `@{upstream}`,
  an `includeIf "onbranch:"` value,
  and `git rev-parse --git-dir`;
  observe the real branch facts and the shadow path.
  Run `git gc --prune=now` in the real repository meanwhile;
  observe the commit lands whole,
  `git worktree list` never shows the shadow,
  and no shadow remains afterward.
  Repeat in LFS,
  submodule,
  sparse-checkout,
  reftable,
  and SHA-256 repositories.
- Native state:
  absent.

### Hook dispatch and hook lock

- Behavior:
  `writeHookShim` (`src/hook-dispatch/hook-shim-writer.ts:59-130`) writes `<tx>/hooks/dispatch.mjs`,
  `plan.json`,
  and executable entries for `pre-commit`,
  `prepare-commit-msg`,
  and `commit-msg` whose first line is `#!<process.execPath>`
  (`src/hook-dispatch/hook-dispatch-plan.ts:33-49`).
  `computeHookDispatchPlan` (`295-385`) records the repository's `core.hooksPath`,
  user-disabled events,
  the caller's config parameters in Git's quoting (`131-285`),
  the real Git path,
  the worktree root,
  the preparation lease,
  and the hook lock.
  The generated program (`src/hook-dispatch/hook-dispatch-program.ts:27-238`) restores those parameters,
  exports an absolute `GIT_WORK_TREE`,
  takes the hook lock at `<git-common-dir>/cli-git/hook.lock` with its own liveness check (`40-173`),
  runs `git hook run --ignore-missing <event>`,
  and propagates the status (`175-221`).
  `post-commit` has no entry and never runs during preparation.
  A nested wrapper with a valid preparation lease skips recovery and the hook lock
  (`src/hook-dispatch/preparation-lease.ts:40-127`).
- Spec:
  `SPEC.md:2408-2478`.
- Consumers:
  repository hooks,
  hookdir and config-based.
- Status:
  retained
  (implementation plan lines 211 to 214:
  hook,
  editor,
  signing,
  and branch identity behavior).
  The shim is a Node program today;
  its native form is undetermined
  (see "Open questions").
- Rust owner:
  `hook_dispatch.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  install a hookdir `pre-commit` and a `hook.<name>.command` hook,
  each appending to a log;
  commit once and observe each ran once,
  that a hook in a subdirectory sees the right top level and an absolute `GIT_WORK_TREE`,
  that two concurrent preparations serialize their hooks by default
  and overlap with `hooks.concurrentCommits: true`,
  and that an open message editor does not hold the hook lock.
- Native state:
  absent.

### Native preparation

- Behavior:
  `runNativePreparation` (`src/policy-engine/commit-preparation-native.ts:315-374`) runs native `git commit`
  with inherited standard streams against the shadow repository,
  `GIT_INDEX_FILE` naming the private index,
  `core.hooksPath` naming the shim,
  and each commit event's `hook.<event>.enabled=false`.
  Git owns hooks,
  the editor,
  templates,
  message cleanup,
  and signing.
  A failure raises `NativeCommitFailedError` with Git's exit code (`106-145`);
  the prepared commit is read from the shadow `HEAD` (`409-447`).
- Spec:
  `SPEC.md:2193-2237`.
- Consumers:
  every commit transaction.
- Status:
  retained
  (implementation plan lines 39 to 46 keep native Git responsible for hooks,
  signing,
  editors,
  and sequencer behavior).
- Rust owner:
  `transaction_preparation.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit with an SSH signing key,
  with `GIT_EDITOR` set to a script that edits the message,
  and with a failing `commit-msg` hook;
  observe a verifiable signature,
  the edited message,
  and for the failure Git's exit code with ref,
  index,
  and worktree bytes unchanged and no shadow repository left.
- Native state:
  absent.

### Preparation policy convergence, added paths, and hook-staged changes

- Behavior:
  policies run against the private index and patches converge
  (`src/policy-engine/commit-transaction-convergence.ts:110-276`).
  A patch for a tracked file that is not a candidate adds the path when `HEAD`,
  the captured real index,
  the private index,
  and the worktree all hold the same blob
  (`src/policy-engine/commit-transaction-added-paths.ts:248-361`);
  otherwise the result is `patch-conflict` (`111-149`).
  After landing,
  each added path's worktree copy is replaced only while it still holds the original bytes (`401-501`).
  Hook-staged changes are found by diffing the settled tree against the committed tree
  (`src/policy-engine/commit-hook-changes.ts:161-245`).
  A settled tree equal to the base yields `commit-normalization/no-change`
  (`src/policy-engine/commit-transaction-no-change.ts:76-209`).
- Spec:
  `SPEC.md:2480-2502`,
  `2531-2588`.
- Consumers:
  `final-newline`,
  Markdown autofix,
  and dependent-version propagation;
  lint-staged-style hooks.
- Status:
  retained.
- Rust owner:
  `transaction_convergence.rs` and `transaction_added_paths.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit with a `pre-commit` hook that reformats and re-stages a selected file
  and stages one extra path;
  observe that the landed tree is what the hook staged,
  that the real index and worktree are reconciled,
  and that a worktree edit made after the hook ran is kept with a warning.
- Native state:
  absent.

### Landing critical section

- Behavior:
  `landTransaction` (`src/policy-engine/commit-landing.ts:198-453`) takes the landing lock,
  or the reserved landing lock (`236-249`),
  then the real `index.lock` (`250`),
  checks the symbolic `HEAD` target,
  reads the target ref,
  migrates objects,
  computes the post-index,
  writes `landing-<n>.json`,
  and advances the target with
  `git update-ref -m <reflog message> <target> <new> <old>`
  (`src/policy-engine/commit-landing-support.ts:192-229`).
  The reflog message is `commit (cli-git <nonce>): <subject>` (`125-190`).
  It then writes `ref-updated.json`,
  removes the pack's `.keep`,
  installs the post-index through an owner-preserving hard link
  (`src/policy-engine/commit-transaction-install-link.ts:28-64`),
  writes `index-installed`,
  and reproduces conclusion-state cleanup.
  `landWithReplay` (`src/policy-engine/commit-landing-loop.ts:107-255`) loops on lost races.
  Every landing-lock acquisition first recovers dead landings
  (`src/policy-engine/commit-landing-lock.ts:105-248`).
  Core findings `head-moved` and `branch-switched` come from
  `src/policy-engine/commit-landing-findings.ts:111-149`.
- Spec:
  `SPEC.md:2590-2675`,
  `3002-3012`.
- Consumers:
  every commit transaction;
  `reference-transaction` hooks see one update of the branch ref.
- Status:
  retained
  (implementation plan lines 215 to 219:
  compare-and-swap landing).
- Rust owner:
  `transaction_landing.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  start 8 commits to disjoint paths together;
  observe 8 landed commits,
  each exactly once with its captured bytes,
  linear native parents,
  the nonce entry exactly once in the branch reflog and in the `HEAD` reflog,
  and a clean `git fsck`.
  Switch branches between preparation and landing;
  observe `concurrent-commit/branch-switched` with exit `1` and unchanged state.
- Native state:
  absent.

### Real index at landing

- Behavior:
  `computeLandingPostIndex` (`src/policy-engine/commit-landing-index.ts:415-528`) computes the post-index
  against the then-current real index.
  Explicit-path commits reset the committed paths to the landed tree.
  Index commits reuse the private index only when no replay happened and the real index is byte-identical
  to the captured one;
  otherwise each landed path is taken only while its real index entry still equals the captured entry.
- Spec:
  `SPEC.md:2677-2704`.
- Consumers:
  concurrent stagers in the same worktree.
- Status:
  retained
  (implementation plan line 221).
- Rust owner:
  `transaction_landing_index.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  hold commit A in `pre-commit`,
  stage path `p` with a second process,
  and release A;
  observe that `p` is still staged after A lands
  and that the index never stages a revert of landed content.
- Native state:
  absent.

### Object migration and pending-object protection

- Behavior:
  `migrateShadowObjects` (`src/policy-engine/commit-landing-objects.ts:85-165`) pipes
  `git pack-objects --revs --local --stdout` from the shadow
  into `git index-pack --stdin --keep=<message>` in the real repository,
  with the keep message `cli-git <transaction-id>` (`57-59`).
  `removePackKeep` (`167-235`) and `removeTransactionKeeps` (`237-287`) remove `.keep` files
  after the compare-and-swap,
  after a failed one,
  and during recovery.
- Spec:
  `SPEC.md:2706-2749`.
- Consumers:
  landing and recovery.
- Status:
  retained
  (implementation plan line 220:
  protection of pending objects).
- Rust owner:
  `transaction_object_migration.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  pause a landing after migration with the phase marker,
  run `git prune --expire=now` and `git repack -a -d`,
  then release;
  observe that the commit lands whole and that no `.keep` with the transaction's message remains.
- Native state:
  absent.

### Replay

- Behavior:
  `replayOrFail` (`src/policy-engine/commit-landing-replay-step.ts:313-343`) replays a prepared commit
  after a lost race.
  `mergeReplayTree` (`src/policy-engine/commit-replay.ts:178-271`) runs
  `git merge-tree --write-tree --name-only -z --merge-base=<base> <current> <prepared>` in the shadow.
  `writeReplayedCommit` (`353-440`) rewrites an unsigned commit's raw object,
  replacing only the `tree` and `parent` lines
  (`src/policy-engine/commit-replay-object.ts:176-271`),
  and rebuilds a signed commit with `git commit-tree -S`,
  dropping custom headers and emitting `replay-headers-dropped` (`273-352`).
  Signing options come from the invocation (`src/policy-engine/commit-replay-options.ts:198-224`).
  A conflict yields `concurrent-commit/replay-conflict` with the conflicting paths,
  the winning commit (`src/policy-engine/commit-replay.ts:273-312`),
  and the prepared commit,
  which is migrated without `.keep` so it can be cherry-picked.
- Spec:
  `SPEC.md:2751-2844`.
- Consumers:
  concurrent agents editing one file.
- Status:
  retained
  (implementation plan lines 215 to 219).
- Rust owner:
  `transaction_replay.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  land two commits started together that edit non-overlapping hunks of one file,
  then two that edit the same lines;
  observe a clean replay with `landing-race-lost` and `commit-replayed` events in the first case,
  and in the second `concurrent-commit/replay-conflict` with exit `1`,
  unchanged ref,
  index,
  and worktree,
  and a prepared commit that `git cherry-pick` accepts.
  Repeat with a non-UTF-8 `i18n.commitEncoding` and with a signed commit carrying a custom header.
- Native state:
  absent.

### Subsumption

- Behavior:
  `subsumeLandedChanges` (`src/policy-engine/commit-replay-subsumption.ts:318-442`) decides,
  for each path both sides changed (`src/policy-engine/commit-replay-shared-paths.ts:281-332`),
  whether the prepared bytes already contain the landed change.
  Text paths are tested by strict reverse application
  (`src/policy-engine/commit-replay-reverse-apply.ts:331-360`,
  3 context lines) and by the one-sided extension rule
  (`src/policy-engine/commit-replay-containment.ts:247-298`);
  a NUL byte in the first 8,000 bytes marks a blob binary
  (`src/policy-engine/commit-replay-subsumption-text.ts:40`,
  `50`,
  `253-373`).
  Subsumed paths go into a synthetic merge base.
- Spec:
  `SPEC.md:2846-2942`.
- Consumers:
  replay.
- Status:
  retained
  (the implementation plan line 217 names replay;
  `doc/decision/cli-git-concurrent-commits.md` section "Serial landing" holds the owner decision).
- Rust owner:
  `replay_subsumption.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run the cases of `SPEC.md:3716-3727` on real repositories;
  observe each stated outcome,
  and that the strict reverse check agrees with `git apply --reverse --check` on seeded inputs.
- Native state:
  absent.

### Revalidation after replay

- Behavior:
  `revalidateReplay` (`src/policy-engine/commit-revalidation.ts:205-421`) reads the merged tree into
  `<tx>/replay-<r>/commit.index`,
  moves the shadow `HEAD` target,
  fingerprints declared inputs,
  re-runs policies against the paths the replayed tree changes,
  and re-runs `pre-commit` when the tree differs from the last approved one and `--no-verify` is absent
  (`src/policy-engine/commit-replay-hook.ts:72-127`).
  A failing re-run exits `1` without an event.
  Worktree completions are retargeted to the replayed blobs
  (`src/policy-engine/commit-landing-replay-records.ts:26-90`).
- Spec:
  `SPEC.md:2944-3000`.
- Consumers:
  replay.
- Status:
  retained.
- Rust owner:
  `transaction_revalidation.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  make the winning commit introduce content that a policy must fix in the replayed tree;
  observe the fix in the landed commit,
  a `fix-summary` after `commit-replayed`,
  one `pre-commit` re-run,
  none under `--no-verify`,
  and nothing landed when the re-run fails.
- Native state:
  absent.

### Amend, conclusions, and sequencer state

- Behavior:
  amend,
  merge,
  cherry-pick,
  and revert conclusions fail with `concurrent-commit/head-moved` when the target moved.
  Conclusion state (`MERGE_HEAD`,
  `MERGE_MSG`,
  `MERGE_MODE`,
  `SQUASH_MSG`,
  `AUTO_MERGE`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `MERGE_RR`,
  `sequencer/`;
  `src/shadow-repository/shadow-conclusion-names.ts:10-55`) is copied into the shadow,
  with reftable pseudorefs read and written through Git
  (`src/shadow-repository/shadow-conclusion-files.ts:136-246`).
  `reproduceConclusionCleanup` (`src/shadow-repository/shadow-conclusion-cleanup.ts:72-261`) removes,
  in the owning worktree,
  each entry native Git removed from the shadow,
  only while it still holds the copied bytes.
- Spec:
  `SPEC.md:3002-3069`.
- Consumers:
  merge,
  cherry-pick,
  and revert workflows.
- Status:
  retained.
- Rust owner:
  `transaction_conclusion.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  conclude a merge,
  a cherry-pick,
  and a revert through the wrapper and through `/usr/bin/git` in twin repositories,
  in files and reftable formats;
  observe byte-identical commits and identical Git-directory state afterward,
  including mid-sequence.
- Native state:
  absent.

### Capture order

- Behavior:
  each capture takes the per-worktree capture lock and a sequence number
  (`src/policy-engine/commit-capture-order-store.ts:57-72`,
  `302-395`),
  and records `captured.json` with the stamp and worktree-captured paths
  (`src/policy-engine/commit-capture-order-capture.ts:134-286`;
  `src/policy-engine/commit-capture-order-journal.ts:33`,
  `245-347`).
  A landing writes `landed/<oid>.json`
  (`src/policy-engine/commit-capture-order-records.ts:61-71`,
  `248-310`,
  `453-476`).
  Replay decides each shared path by comparing sequence numbers
  (`src/policy-engine/commit-capture-order-decision.ts:94-137`;
  `src/policy-engine/commit-capture-order-replay.ts:60-154`)
  over the first-parent history (`src/policy-engine/commit-capture-order-history.ts:123-172`).
  Records are pruned when no published transaction can need them
  (`src/policy-engine/commit-capture-order-prune.ts:150-217`).
- Spec:
  `SPEC.md:3071-3217`.
- Consumers:
  replay of commits captured from one worktree.
- Status:
  retained
  (implementation plan lines 215 to 219:
  capture ordering).
- Rust owner:
  `capture_order.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  capture commit A,
  then commit B rewriting the same line,
  and land B first;
  observe that A keeps B's bytes for that path and lands its other paths.
  Reverse the capture order;
  observe that the later capture's bytes land.
  After the last transaction ends,
  observe no file under `cli-git-captures/landed/`.
- Native state:
  absent.

### Starvation reservation

- Behavior:
  after `landing.reserveAfterLostRaces` lost races a transaction writes `reservation-request`
  and waits for `reservation.lock`
  (`src/policy-engine/commit-landing-reservation.ts:68`,
  `187-332`).
  Requests are granted oldest invocation first,
  ties by transaction ID (`145-185`).
  Other transactions wait before landing while a live reservation exists (`334-365`).
  A granted reservation emits `landing-reserved`.
- Spec:
  `SPEC.md:3265-3307`.
- Consumers:
  concurrent commits under contention.
- Status:
  retained
  (implementation plan lines 215 to 219:
  starvation reservations).
- Rust owner:
  `landing_reservation.rs` (proposed).
- Consumer-level test:
  in the standard fixture with the default of 1,
  make one transaction lose a race while others keep landing;
  observe `landing-reserved` after its `landing-race-lost`,
  at most 2 lost races for the holder,
  and release of the reservation when the holder lands,
  conflicts,
  or is killed.
- Native state:
  absent.

### Post-landing completion

- Behavior:
  after both locks are released,
  `concludeCommitTransaction` (`src/policy-engine/commit-transaction-conclusion.ts:168-431`) completes
  added-path worktree copies (`393`),
  removes the shadow repository and the transaction directory,
  runs automatic maintenance (`406`),
  and runs `post-commit` once (`411`).
  `runAutoMaintenance` (`src/policy-engine/commit-landing-auto-maintenance.ts:135-267`) mirrors Git's decision
  from `maintenance.auto`,
  `gc.auto`,
  and the detach settings,
  runs `git maintenance run --auto --quiet`,
  and ignores its status.
  `runPostCommitHook` (`src/policy-engine/commit-landing-post-commit-hook.ts:98-160`) runs
  `git hook run post-commit` in the real worktree under the hook lock,
  with `GIT_INDEX_FILE`,
  `GIT_AUTHOR_*`,
  and `GIT_EDITOR=:`.
- Spec:
  `SPEC.md:3219-3263`.
- Consumers:
  `post-commit` hooks;
  later Git commands,
  whose speed depends on the pack count.
- Status:
  retained
  (implementation plan line 220:
  post-landing automatic maintenance).
- Rust owner:
  `transaction_post_landing.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  land 30 commits in sequence with a counting `post-commit` hook;
  observe 30 hook runs with the native environment,
  a bounded pack count,
  and no background-maintenance text in the JSONL stream.
- Native state:
  absent.

### Recovery

- Behavior:
  `recoverCommitTransaction` (`src/policy-engine/commit-transaction-recovery.ts:127-191`) runs at startup,
  before configuration loading.
  `recoverRegisteredTransactions` (`src/policy-engine/commit-transaction-recovery-scan.ts:194-282`)
  inspects every published directory (`src/policy-engine/commit-transaction-recovery-inspect.ts:118-166`).
  A live owner is skipped.
  A dead owner without a landing record loses its `.keep` files,
  reservation,
  shadow repository,
  and directory.
  A dead owner with a landing record is recovered under the landing lock
  (`src/policy-engine/commit-transaction-recovery-landing.ts:185-353`):
  an unlanded attempt is discarded,
  an interrupted index install is completed from the recorded post-index
  (`src/policy-engine/commit-transaction-recovery-files.ts:162-238`),
  and a completed install is recognized
  (`src/policy-engine/commit-transaction-recovery-completion.ts:98-232`).
  Before `ref-updated.json` exists,
  a landing counts only when the target reflog holds the nonce entry
  (`src/policy-engine/commit-transaction-recovery-reflog.ts:90-157`).
  Real `index.lock` files are removed only by recorded device and inode
  (`src/policy-engine/commit-transaction-recovery-evidence.ts:212-270`;
  `src/policy-engine/commit-transaction-recovery-validation.ts:328-371`).
  Malformed state fails closed with `CommitTransactionRecoveryError` (`27-49`).
  The legacy single-journal directory `cli-git-transaction` is still recovered
  (`src/policy-engine/commit-transaction-registry.ts:50`;
  `src/policy-engine/commit-transaction-recovery-journaled.ts:60`,
  `252-430`).
- Spec:
  `SPEC.md:3497-3549`,
  `2145-2146`.
- Consumers:
  every wrapper invocation after a crash;
  `README.md` section "Commit recovery".
- Status:
  retained
  (implementation plan lines 225 and 228 to 233:
  durable recovery after interruption;
  old transaction state is never silently discarded,
  and an unreadable state fails with an actionable recovery diagnostic).
  Whether the native wrapper recovers the legacy single-journal directory is undetermined
  (see "Open questions").
- Rust owner:
  `transaction_recovery.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  kill the wrapper with `SIGKILL` at each phase marker and each hook point,
  then run `git status`;
  observe after recovery that the commit either landed whole or not at all,
  that the real index matches,
  and that no shadow repository,
  transaction directory,
  lock,
  `.keep`,
  or landed-capture record remains.
  Plant a published directory without a valid owner record;
  observe exit `2`,
  the path named,
  and its contents preserved.
- Native state:
  absent.

### Test phase markers

- Behavior:
  `CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill[:<directory>]` makes the wrapper kill itself at a phase,
  and `<phase>:pause:<directory>` writes a marker and waits for a release file
  (`src/policy-engine/commit-transaction-test-phase.ts:37`,
  `109-234`).
  The phases are listed at `47-58`.
  A malformed value fails the invocation (`83-107`).
- Spec:
  `SPEC.md:3954-3972`.
- Consumers:
  the container end-to-end suite (`e2e/scenario-phase-kill-fixture.ts`,
  `e2e/scenario-reservation-fixture.ts`,
  `e2e/scenario-subsumption-fixture.ts`)
  and the concurrent-commit benchmark (`perf/concurrent-commit-latency-pause.ts`).
- Status:
  retained.
  The implementation plan lines 289 to 291 reuse the existing end-to-end scenarios against the Rust executable,
  and those scenarios reach landing phases only through these markers.
  Whether a release build carries them is undetermined
  (see "Open questions").
- Rust owner:
  `transaction_test_phase.rs` (proposed).
- Consumer-level test:
  the end-to-end `sigkill-phase-*` scenarios pass against the native executable.
- Native state:
  absent.

### Older-Git degradation

- Behavior:
  the wrapper has no version gate.
  `replayPlumbingAvailable` (`src/policy-engine/replay-plumbing.ts:118-146`) probes
  `git merge-tree --write-tree --merge-base` after a first lost race
  and falls back to `concurrent-commit/head-moved`;
  automatic maintenance retries without `--detach` on exit `129`
  (`src/policy-engine/commit-landing-auto-maintenance.ts:45`);
  a Git without `core.lockfilePid` yields no PID evidence.
- Spec:
  `SPEC.md:3551-3618`.
- Consumers:
  the end-to-end matrix on Git 2.39.5 and 2.40.0
  (`package/git-policy/cli/mise.toml:296-358`;
  `e2e/git-version-fixture.ts`;
  `e2e/scenario-replay-degradation-fixture.ts`).
- Status:
  retired.
  The implementation plan lines 22 to 25 support the latest stable Git release only
  and forbid porting older-Git compatibility branches or keeping an old-version test matrix;
  lines 235 to 237 report missing required Git behavior as an unsupported-environment failure.
  The rewrite scope lines 105 to 106 state that legacy Git degradation paths are not a parity requirement.
- Rust owner:
  none for degradation;
  the unsupported-environment failure is listed under "Responsibilities the plan adds".
- Consumer-level test:
  run the native wrapper against a Git older than the supported release in a container;
  observe an unsupported-environment failure and no fallback algorithm.
- Native state:
  absent.

## Locks

Every entry in this section is retained by the implementation plan lines 222 to 224
(process-birth-aware locks,
foreign-lock evidence,
and cancellation).
Native state is absent for every entry.

### Owner locks with process-birth identity

- Behavior:
  `acquireOwnerLock` (`src/owner-lock/owner-lock.ts:499-535`) publishes a lock directory by rename
  after writing `owner.json` exclusively and syncing it (`150-165`,
  `205-225`).
  The record holds schema version 1,
  a random token,
  the owner PID,
  the owner's birth identity,
  and for the reservation a transaction ID (`src/owner-lock/owner-lock-record.ts:24-63`).
  An owner is alive only while its PID names a process with the recorded birth identity (`165-171`).
  A dead owner's lock is retired by renaming it aside before deletion (`src/owner-lock/owner-lock.ts:468-497`).
  A waiter polls every 20 ms (`69`) without a time limit while the owner lives.
  Birth identity comes from `/proc/<pid>/stat` on Linux,
  where states `Z` and `X` count as exited,
  from `ps -o lstart=` on macOS,
  and from a PowerShell `Get-Process` start time on Windows
  (`src/policy-engine/commit-transaction-process-identity.ts:12-70`,
  `132-159`).
  The lock paths are the landing lock,
  the reservation lock,
  the hook lock,
  the capture lock,
  and one push lock per branch.
- Spec:
  `SPEC.md:3309-3380`.
- Consumers:
  landing,
  reservation,
  hook dispatch,
  capture order,
  auto-push,
  direct fix,
  index-writer coordination.
- Status:
  retained.
- Rust owner:
  `owner_lock.rs` and `process_identity.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  kill a lock holder,
  leave a zombie holder under a PID 1 that does not reap,
  and plant a lock whose PID was reused by a younger process;
  observe that the next acquirer retires each lock,
  and that a live holder makes the acquirer wait until it exits.
- Native state:
  absent.

### Real index lock

- Behavior:
  `acquireRealIndexLock` (`src/policy-engine/commit-landing-index-lock.ts:214-352`) creates `<index>.lock` exclusively,
  writes Git's lock PID file format,
  and journals the lock's device and inode in `index-lock-<n>.json` right after creation.
- Spec:
  `SPEC.md:2597-2602`,
  `3378-3380`.
- Consumers:
  landing and recovery.
- Status:
  retained.
- Rust owner:
  `index_lock.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  pause a landing inside the critical section and run `/usr/bin/git add` directly;
  observe that native Git reports the lock held and names the wrapper's PID from the PID file.
- Native state:
  absent.

### Foreign index-lock evidence and waiting

- Behavior:
  `gatherIndexLockEvidence` (`src/index-lock/index-lock-evidence.ts:415-561`) reads the lock's device,
  inode,
  and ctime,
  the Git PID file (`165-315`),
  and open holders by device and inode:
  `/proc/<pid>/fd` on Linux (`src/index-lock/index-lock-holders-linux.ts:286-359`,
  16 concurrent readers),
  `lsof` on macOS (`src/index-lock/index-lock-holders-darwin.ts:335-372`),
  and a Restart Manager query through PowerShell on Windows
  (`src/index-lock/index-lock-holders-win32.ts:31-187`).
  `classifyIndexLock` (`src/index-lock/index-lock-evidence.ts:366-413`) returns proven alive,
  dead,
  or evidence-free.
  Process start times use a 20 ms resolution on Linux and 1 s on macOS
  (`src/index-lock/process-start-time.ts:29-54`,
  `339-376`).
  `waitForIndexLock` (`src/index-lock/index-lock-wait.ts:270-366`) waits without limit for a proven-alive owner,
  with polls capped at 100 ms or 500 ms (`71-77`),
  and otherwise backs off quadratically with jitter (`49-64`,
  `164-204`) up to `indexLock.unprovenOwnerTimeoutMs`,
  then raises `IndexLockUnprovenOwnerError` (`82-129`),
  leaving the lock in place.
- Spec:
  `SPEC.md:3400-3448`.
- Consumers:
  landing and forwarded index writers.
- Status:
  retained.
- Rust owner:
  `index_lock_evidence.rs` and `index_lock_wait.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  hold `index.lock` with a native `git commit` waiting in its editor,
  with a PID file naming an exited process,
  with no PID file,
  and with a child that holds the lock file open;
  observe an unbounded wait with one stderr line naming the holder in the first and last cases,
  and `index-lock-unproven-owner` with exit `2` after the configured budget in the others,
  with the lock left in place.
- Native state:
  absent.

### Index-writer coordination and landing lease

- Behavior:
  `isIndexWriter` (`src/index-lock/index-writer-commands.ts:196-235`) classifies the resolved command:
  15 commands always write the index (`33-49`),
  and `restore --staged`,
  `reset` except `--soft`,
  and `apply --cached` or `--index` write it by option (`68-97`).
  `coordinateIndexWriter` (`src/index-lock/index-writer-coordination.ts:85-173`) takes the landing lock
  for a writer against the real index,
  pre-waits for a foreign `index.lock`,
  and hands the forwarded Git a `CLI_GIT_LANDING_LEASE`
  (`src/index-lock/landing-lease.ts:26-110`).
  A nested invocation whose lease names the same held lock proceeds without it.
- Spec:
  `SPEC.md:3450-3495`.
- Consumers:
  every forwarded index writer;
  hooks and `rebase --exec` commands that call the wrapper again.
- Status:
  retained.
- Rust owner:
  `index_writer_coordination.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git add -- a.txt` while a landing is paused inside its critical section;
  observe that `add` waits and then succeeds.
  Run `git rebase --exec 'git add -- b.txt'` through the wrapper;
  observe no self-deadlock.
- Native state:
  absent.

### Lock order

- Behavior:
  the order is reservation check,
  landing lock,
  then real `index.lock` (`src/policy-engine/commit-landing.ts:236-250`).
  No process takes the hook lock or a push lock while holding the landing lock,
  and none takes another lock while holding the capture lock
  (`src/policy-engine/commit-capture-order-store.ts:328-395`).
  The wrapper never deletes a foreign lock.
- Spec:
  `SPEC.md:3372-3380`.
- Consumers:
  every lock user.
- Status:
  retained.
- Rust owner:
  the transaction and lock modules together;
  no separate module.
- Consumer-level test:
  the end-to-end reservation,
  index-writer,
  and hooked scenarios finish without a stuck process under a bounded timeout,
  on every seed.
- Native state:
  absent.

## Durable formats

### Journal and lock formats

- Behavior:
  the wrapper writes these durable formats:
  transaction journal records at schema version 2
  (`src/policy-engine/commit-transaction-journal-states.ts:25`);
  transaction `owner.json` at schema version 2 (`src/policy-engine/commit-transaction-owner.ts:28`);
  owner-lock `owner.json` at schema version 1 (`src/owner-lock/owner-lock-record.ts:29`);
  `captured.json`,
  `worktree-id`,
  `sequence`,
  and landed-capture records at schema version 1
  (`src/policy-engine/commit-capture-order-store.ts:57-72`;
  `src/policy-engine/commit-capture-order-records.ts:71`);
  `last-pushed` records at schema version 1 (`src/auto-push-record.ts:33`);
  worktree-copy journals under `cli-git-worktree-copy/v1` (`src/worktree-copy/journal.ts:29-34`)
  with an `install-log.jsonl` (`src/worktree-copy/install-log.ts:39`);
  the worktree-copy settlement owner record (`src/worktree-copy/journal-lock-owner.ts:46`,
  `66-86`);
  candidate snapshots as length-prefixed binary files
  (`src/policy-engine/commit-transaction-candidate-snapshot.ts:22-26`,
  `82-151`);
  the hook plan `plan.json`;
  the reflog message `commit (cli-git <nonce>): <subject>`;
  and the pack keep message `cli-git <transaction-id>`.
  Private directories are mode `0700` and files `0600`,
  and reads refuse symbolic links.
- Spec:
  `SPEC.md:2083-2146`,
  `1000-1014`,
  `1097-1120`,
  `3088-3159`.
- Consumers:
  recovery in every later invocation,
  including one made by a different wrapper version.
- Status:
  retained
  (implementation plan lines 228 to 233).
- Rust owner:
  each format belongs to the module that owns its lifecycle;
  a `durable_formats` test fixture set proves compatibility.
- Consumer-level test:
  write each record with the incumbent in a disposable repository,
  stop it at a phase marker,
  and run the native wrapper;
  observe correct recovery from every incumbent-written state.
  Feed each parser truncated,
  oversized,
  and wrong-schema records;
  observe a fail-closed diagnostic and preserved bytes.
- Native state:
  absent.

## Linked-worktree ignored-state copy

Every entry in this section is retained by the implementation plan lines 239 to 249,
which port worktree copy as its own lifecycle module
and preserve the main-worktree bypass
and the distinction between settlement locks and landing locks.
Native state is absent for every entry.

### Applicability and main-worktree bypass

- Behavior:
  `runGitWithWorktreeCopy` (`src/worktree-copy/lifecycle.ts:229-425`) forwards without synchronization
  when the invocation is a short-circuit form (`278-286`),
  targets a main worktree or no repository (`295-303`),
  inherits a valid `CLI_GIT_WORKTREE_COPY_LEASE` (`307-324`),
  or carries `--no-worktree-copy` in flag position (`259-277`).
  A command that neither creates nor moves worktrees only recovers pending journals (`325-337`).
  `git worktree add` and `git worktree move`,
  after alias resolution,
  are applicable sources (`src/forwarded-command.ts:399-412`).
- Spec:
  `SPEC.md:901-958`.
- Consumers:
  agents that create linked worktrees from a linked worktree;
  `AGENTS.md` rule IWT uses `git worktree add`.
- Status:
  retained.
- Rust owner:
  `worktree_copy.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git worktree add` from the main worktree,
  from a linked worktree,
  through an alias,
  and with `--no-worktree-copy`;
  observe a copy only for the linked source without the flag,
  and that the flag never reaches Git.
- Native state:
  absent.

### Created-worktree detection

- Behavior:
  `observeWorktreeRepository` (`src/worktree-copy/git-observer.ts:93-147`) captures the administrative identity set
  under the common directory before Git runs,
  and `findCreatedWorktrees` (`src/worktree-copy/git-registry.ts:247-306`) compares it afterward,
  so a worktree that Git registered before failing still counts.
- Spec:
  `SPEC.md:924-937`.
- Consumers:
  worktree copy.
- Status:
  retained.
- Rust owner:
  `worktree_copy_registry.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run `git worktree add` with a `post-checkout` hook that fails;
  observe that the retained worktree still receives the ignored files and that Git's exit status is preserved.
- Native state:
  absent.

### Ignored-state selection

- Behavior:
  `readIgnoredRoots` (`src/worktree-copy/ignored-paths.ts:354-410`) asks Git for ignored paths
  under the standard exclusion stack,
  excludes registered worktrees nested under the source,
  and reserves the private stage prefix `.cli-git-worktree-copy-` (`src/worktree-copy/snapshot.ts:37`).
  Repository paths are validated before use (`src/worktree-copy/ignored-paths.ts:34-108`).
  A bare repository contributes an empty set.
- Spec:
  `SPEC.md:939-962`.
- Consumers:
  worktree copy.
- Status:
  retained
  (implementation plan line 242:
  ignored-state selection).
- Rust owner:
  `worktree_copy_selection.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  ignore files through a nested `.gitignore`,
  `info/exclude`,
  and `core.excludesFile`,
  including a symbolic link,
  a FIFO,
  and a nested registered worktree;
  observe that files and links are copied,
  the nested worktree is skipped,
  and the FIFO produces a copy failure without being opened.
- Native state:
  absent.

### Staging with copy-on-write

- Behavior:
  `stageIgnoredSnapshot` (`src/worktree-copy/snapshot.ts:225-374`) copies the selection into a mode-`0700`
  sibling directory of the destination,
  with `COPYFILE_EXCL | COPYFILE_FICLONE` (`47`),
  which falls back to a full copy.
  A parent-first manifest is built (`src/worktree-copy/entry-manifest.ts:127-267`),
  modes are applied,
  and the source and stage manifests must match exactly
  (`src/worktree-copy/entry-compare.ts:251-312`).
- Spec:
  `SPEC.md:960-978`.
- Consumers:
  worktree copy.
- Status:
  retained
  (implementation plan line 243:
  copy-on-write requests).
- Rust owner:
  `worktree_copy_stage.rs` (proposed).
- Consumer-level test:
  in the standard fixture on a reflink-capable filesystem and on one without reflinks,
  copy a tree with executable files and symbolic links;
  observe identical bytes,
  modes,
  and link targets,
  and that a source file changed during staging blocks the destination.
- Native state:
  absent.

### Installation, existing-entry checks, and rollback

- Behavior:
  `installSnapshot` (`src/worktree-copy/install.ts:247-316`) preflights every existing destination entry
  and accepts only an exact match (`src/worktree-copy/entry-compare.ts:183-249`).
  It creates absent entries exclusively,
  installing a regular file as a hard link to its staged copy and falling back to an exclusive copy
  (`src/worktree-copy/install-entry.ts:41`,
  `233-321`),
  in batches of 512 entries (`src/worktree-copy/install.ts:31`).
  `rollbackCreated` (`src/worktree-copy/install-rollback.ts:80-171`) removes only transaction-owned paths
  that still match the stage.
- Spec:
  `SPEC.md:980-998`.
- Consumers:
  worktree copy.
- Status:
  retained
  (implementation plan line 244:
  exact existing-entry checks).
- Rust owner:
  `worktree_copy_install.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  pre-create one identical and one differing file at destination paths;
  observe that the identical file is accepted,
  the differing one is never overwritten,
  the run exits `2`,
  and rollback leaves the Git-created worktree and every unowned path in place.
- Native state:
  absent.

### Journals and install log

- Behavior:
  each destination has a journal under `<git-common-dir>/cli-git-worktree-copy/v1`
  (`src/worktree-copy/journal.ts:74-120`,
  `317-404`),
  written through a private no-follow temporary file,
  file sync,
  rename,
  and directory sync where supported (`208-315`).
  Phases are `staged`,
  `installing`,
  and `complete` (`src/worktree-copy/model.ts:213`).
  Intents and created identities are appended to `install-log.jsonl` inside the stage,
  one synced line per batch (`src/worktree-copy/install-log.ts:283-385`;
  `src/worktree-copy/transaction-journal.ts:55-198`).
  Journal values are validated (`src/worktree-copy/journal-validation.ts:170-272`).
- Spec:
  `SPEC.md:1000-1014`.
- Consumers:
  worktree-copy recovery.
- Status:
  retained
  (implementation plan line 245:
  journals).
- Rust owner:
  `worktree_copy_journal.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  copy 10,000 ignored files and count journal rewrites and log lines;
  observe one journal write per phase and a log that grows by batch,
  not by file.
- Native state:
  absent.

### Settlement lock

- Behavior:
  `acquireWorktreeCopyLock` (`src/worktree-copy/journal-lock.ts:362-457`) serializes installation and recovery
  per common directory.
  It waits without limit for an owner proven alive,
  after one stderr line naming the PID and the lock;
  an unreadable owner record gets a 1000 ms backoff budget (`53`) and then a failure
  that leaves the lock in place.
  Owner records and liveness are in `src/worktree-copy/journal-lock-owner.ts:161-420`.
  The forwarded Git receives the lease token so nested wrapper invocations forward without the lock
  (`src/worktree-copy/journal-lock.ts:323-360`).
- Spec:
  `SPEC.md:1015-1029`,
  `3334-3338`.
- Consumers:
  worktree copy;
  hooks that call the wrapper during `git worktree add`.
- Status:
  retained
  (implementation plan lines 248 to 249).
- Rust owner:
  `worktree_copy_lock.rs` (proposed),
  or the shared `owner_lock.rs`.
- Consumer-level test:
  in the standard fixture,
  start two `git worktree add` commands together from one linked worktree,
  and run concurrent commits in that worktree meanwhile;
  observe both copies complete,
  one waiter line,
  and no commit waiting on the settlement lock.
- Native state:
  absent.

### Crash recovery

- Behavior:
  `recoverPendingWorktreeCopies` (`src/worktree-copy/pending-recovery.ts:66-91`) checks for pending journals
  without the lock and takes it only to recover.
  `recoverWorktreeCopyTransactions` (`src/worktree-copy/transaction-recovery.ts:173-200`) validates journal paths
  and the private stage (`src/worktree-copy/journal-filesystem.ts:178-249`;
  `src/worktree-copy/private-path.ts:44-73`),
  discards a transaction whose destination is no longer a linked registration or whose stage is gone,
  resumes installation otherwise (`src/worktree-copy/transaction-install.ts:83-280`),
  and resumes completed cleanup (`src/worktree-copy/journal-cleanup.ts:63-109`).
  Malformed or unsafe state fails closed without deleting it.
- Spec:
  `SPEC.md:1031-1064`.
- Consumers:
  every later linked-worktree or bare-repository invocation.
- Status:
  retained
  (implementation plan lines 246 to 247:
  containment validation and crash recovery).
- Rust owner:
  `worktree_copy_recovery.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  kill the wrapper during staging,
  during installation,
  and during cleanup,
  then run `git status` in a linked worktree;
  observe that each transaction reaches an end with one notice line.
  Plant a journal whose paths escape the common directory;
  observe a fail-closed diagnostic and nothing deleted outside the validated stage.
- Native state:
  absent.

### Summary line and exit codes

- Behavior:
  after all destinations settle the wrapper writes one summary line to stderr
  (`src/worktree-copy/lifecycle.ts:153-173`,
  `394-395`).
  A copy-only failure exits `2`;
  when Git also failed,
  Git's status is kept,
  or `1` after a signal (`src/bin.ts:443-453`;
  `src/worktree-copy/errors.ts:30-97`).
- Spec:
  `SPEC.md:1066-1073`.
- Consumers:
  humans and agents creating worktrees.
- Status:
  retained.
- Rust owner:
  `worktree_copy.rs` (proposed).
- Consumer-level test:
  covered by the applicability,
  detection,
  and installation tests,
  each asserting the exit status and the stderr line.
- Native state:
  absent.

## Auto-push

Every entry in this section is retained by the implementation plan line 226
(current auto-push completion and failure reporting).
Native state is absent for every entry.

### Trigger, skips, and push arguments

- Behavior:
  `autoPush` (`src/auto-push.ts:392-497`) runs after every landed commit whose post-commit gate passed
  (`src/bin.ts:416-422`).
  It skips silently without an upstream and without an `origin` remote (`src/auto-push.ts:413-431`),
  skips with a note on a detached `HEAD` (`436-448`),
  pushes plainly when an upstream exists,
  and otherwise runs `git push --set-upstream origin HEAD` (`35-47`,
  `454-456`).
  The push runs real Git directly,
  so the wrapper's manual-push lifecycle does not run for it.
  No configuration key disables auto-push.
- Spec:
  `SPEC.md:1094-1167`.
- Consumers:
  every commit in this repository
  (`AGENTS.md` rule APG);
  agents that must not push twice.
- Status:
  retained.
- Rust owner:
  `auto_push.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit on a branch with an upstream,
  on a new branch,
  on a detached `HEAD`,
  and in a repository without remotes;
  observe the bare remote's refs and tracking configuration in the first two cases
  and no coordination file in the last two.
- Native state:
  absent.

### Branch keys

- Behavior:
  `encodeBranchKey` (`src/auto-push-branch-key.ts:89-205`) writes each byte of the ref name outside lowercase letters,
  digits,
  `-`,
  `_`,
  and `.` as uppercase `%XX`;
  `decodeBranchKey` (`207-220`) reverses it.
- Spec:
  `SPEC.md:1111-1120`.
- Consumers:
  push lock and record paths under `<git-common-dir>/cli-git/push/`.
- Status:
  retained.
- Rust owner:
  `auto_push_branch_key.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  commit on branches named `Feature/A` and `feature/a`;
  observe two distinct key files on a case-insensitive filesystem,
  each decoding to its ref name.
- Native state:
  absent.

### Last-pushed records

- Behavior:
  `writeLastPushedRecord` (`src/auto-push-record.ts:202-235`) publishes by rename a record of the last attempt:
  the resolved tip,
  the outcome,
  the lock token and PID,
  the exit code,
  and a failed attempt's complete output (`58-101`).
  An absent or unreadable record reads as no attempt (`152-200`).
- Spec:
  `SPEC.md:1097-1110`.
- Consumers:
  single-flight joins.
- Status:
  retained.
- Rust owner:
  `auto_push_record.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  corrupt the record between two commits;
  observe that the second commit pushes anyway and rewrites a valid record.
- Native state:
  absent.

### Single flight and joins

- Behavior:
  `runSingleFlightPush` (`src/auto-push-single-flight.ts:296-414`) finishes without the lock
  when the landed commit is an ancestor of the recorded successful tip (`226-294`),
  otherwise waits for the per-branch push lock (`346`),
  re-reads the record,
  joins a covering success or a covering failure recorded after its first read,
  or pushes and records the attempt.
  A dead pusher's lock is retired through owner liveness.
- Spec:
  `SPEC.md:1122-1167`.
- Consumers:
  concurrent landings on one branch.
- Status:
  retained.
- Rust owner:
  `auto_push_single_flight.rs` (proposed).
- Consumer-level test:
  in the standard fixture with a slow `pre-push` hook,
  land 8 commits together;
  observe fewer pushes than commits,
  every landed commit on the remote,
  and after killing one pusher a takeover push by a waiting joiner.
- Native state:
  absent.

### Push output and failure reporting

- Behavior:
  `filterPushOutput` (`src/auto-push.ts:105-121`) surfaces only `remote:` lines after a clean push
  and the complete output after a failure.
  Every affected invocation,
  owner or joiner,
  prints the failure and the note at `130`,
  and the commit command still exits `0`.
  A coordination failure is reported the same way (`483-496`).
- Spec:
  `SPEC.md:1156-1164`,
  `1572-1575`,
  `1603`.
- Consumers:
  humans and agents reading commit output.
- Status:
  retained.
- Rust owner:
  `auto_push.rs` (proposed).
- Consumer-level test:
  in the standard fixture with a remote that rejects the push,
  commit from two processes together;
  observe the complete rejection text and the local-commit note from both,
  exit `0` from both,
  and the commits present locally only.
- Native state:
  absent.

## Logging and diagnostics

### Tagged debug logging

- Behavior:
  production code logs through tagged loggers from `@monochromatic-dev/logger-core`,
  rooted at the tag `cli-git` (`src/bin.ts:54`).
  96 non-test source files call `tagged(` 192 times,
  and 90 files hold 215 `debug`,
  `info`,
  `warn`,
  or `error` calls
  (`rg --count-matches` over `src`,
  excluding tests,
  fixtures,
  and `native`).
  The console sink prints debug records when `MONOCHROMATIC_VERBOSE=true`
  or when `process.argv` contains `--verbose`
  (`package/logger/core/src/sink/console.ts:55-77`).
- Spec:
  `SPEC.md:1586` (debug logs must not corrupt the selected JSONL stream),
  `1478`.
- Consumers:
  developers and agents diagnosing a wrapper decision.
- Status:
  retained.
  The implementation plan line 260 lists diagnostics in the first native slice.
- Rust owner:
  `diagnostics.rs` (proposed).
- Consumer-level test:
  in the standard fixture,
  run a blocked command with verbose diagnostics enabled and capture stderr;
  observe that every line is either one complete JSONL event or one diagnostic line,
  never an interleaved fragment,
  and that stdout holds only Git's output.
- Native state:
  absent.

### Human diagnostics

- Behavior:
  expected rejections carry a message that names the input,
  the reason,
  and each way forward,
  for example `src/rule/require-root.ts:182-186`,
  `src/rule/commit-only.ts:38-51`,
  `108-115`,
  and `src/policy-engine/add-explicit-check.ts:124-132`.
  Human lines outside JSONL go to stderr through `console`:
  management usage (`src/management.ts:150`,
  `172`,
  `190`,
  `196`),
  auto-push notes (`src/auto-push.ts:85-86`,
  `130`,
  `488-489`),
  worktree-copy summaries and failures (`src/worktree-copy/lifecycle.ts:172`;
  `src/bin.ts:444`,
  `451`),
  lock waiter lines (`src/index-lock/index-lock-evidence.ts:588-604`),
  and uncaught error messages (`src/bin.ts:458`).
  Raw `console` calls appear in 9 non-test source files,
  two of them host-evidence programs
  (`rg --count-matches 'console\.(log|error|warn|info)\(' src`,
  excluding tests,
  fixtures,
  and `native`).
- Spec:
  `SPEC.md:1016-1018`,
  `1066-1067`,
  `3430`,
  `1156-1161`.
- Consumers:
  humans and agents acting on a rejection.
- Status:
  retained.
- Rust owner:
  `diagnostics.rs` (proposed),
  with message text owned by each rule module.
- Consumer-level test:
  in the standard fixture,
  trigger each rejection and compare the message with the accepted text fixture;
  observe that each names the affected input and every remedy.
- Native state:
  absent.

## Platform-specific behavior

The implementation plan lines 300 to 303 keep the existing Linux,
macOS,
and Windows consumer coverage,
and line 338 keeps native macOS and Windows jobs for their filesystem and process contracts.
Trust-only platform code (`src/trust/registry-io.ts`,
`src/trust/account-root.ts`,
`src/trust/candidate.ts`) retires with the trust subsystem.

### Linux

- Behavior:
  process-birth identity and exited-state detection read `/proc/<pid>/stat`
  (`src/policy-engine/commit-transaction-process-identity.ts:12-70`,
  `133-134`);
  process start time uses start ticks and `/proc/uptime` (`src/index-lock/process-start-time.ts:29`,
  `355-360`);
  open lock holders are found through `/proc/<pid>/fd` (`src/index-lock/index-lock-evidence.ts:324-328`);
  the hook dispatcher repeats the liveness check (`src/hook-dispatch/hook-dispatch-program.ts:69-84`);
  common Git paths are `/usr/bin/git` and `/usr/local/bin/git`
  (`package/git-executable/core/src/platform-paths.ts:10-13`).
- Spec:
  `SPEC.md:3358-3370`,
  `3406-3424`.
- Consumers:
  the development host,
  every container harness,
  and every CI job that runs the wrapper.
- Status:
  retained.
- Rust owner:
  `process_identity.rs` and `index_lock_evidence.rs` (proposed).
- Consumer-level test:
  the standard fixture runs on Linux;
  the zombie-owner and open-descriptor cases under "Locks" are the platform cases.
- Native state:
  absent.

### macOS

- Behavior:
  birth identity and start time come from `ps -o lstart= -p <pid>`
  (`src/policy-engine/commit-transaction-process-identity.ts:135-145`;
  `src/index-lock/process-start-time.ts:361-362`;
  `src/hook-dispatch/hook-dispatch-program.ts:85-88`);
  open holders come from `lsof` (`src/index-lock/index-lock-holders-darwin.ts:335-372`),
  spawned only while a foreign lock is present;
  common Git paths add `/opt/homebrew/bin/git` and `/opt/local/bin/git`
  (`package/git-executable/core/src/platform-paths.ts:18-22`).
- Spec:
  `SPEC.md:3411-3424`.
- Consumers:
  macOS developers;
  the `trust` job of `.github/workflows/cli-git-trust.yml:189-232` is the only macOS CI job,
  and it runs trust and filesystem-identity tests only.
- Status:
  retained.
- Rust owner:
  `process_identity.rs` and `index_lock_evidence.rs` (proposed).
- Consumer-level test:
  on a macOS host,
  run the owner-lock,
  foreign `index.lock`,
  and concurrent-commit cases with the native `git`;
  observe the same outcomes as on Linux.
- Native state:
  absent.

### Windows

- Behavior:
  real-Git lookup follows `PATHEXT` and compares candidates case-insensitively
  (`package/git-executable/core/src/resolve-real-git.ts:31`,
  `78-123`),
  with Program Files and `LOCALAPPDATA` install roots
  (`package/git-executable/core/src/platform-paths.ts:27`,
  `51-92`)
  and backslash shim markers (`package/git-executable/core/src/self-shim.ts:149-154`).
  Birth identity and start time come from a PowerShell `Get-Process` call
  (`src/policy-engine/commit-transaction-process-identity.ts:146-157`;
  `src/index-lock/process-start-time.ts:363-364`);
  open holders come from a Restart Manager query run through PowerShell
  (`src/index-lock/index-lock-holders-win32.ts:26-187`),
  never a `DELETE`-access probe.
  The shadow repository shares directories as junctions and copies shared files
  (`src/shadow-repository/shadow-links.ts:96-125`).
  Directory sync is skipped (`src/worktree-copy/journal.ts:217-219`;
  `src/worktree-copy/install-log.ts:255-257`).
  The hook shim's shebang form is pending verification on Windows (`SPEC.md:2460-2462`).
- Spec:
  `SPEC.md:3417-3423`,
  `2460-2462`;
  `doc/decision/cli-git-concurrent-commits.md:475-478`.
- Consumers:
  Windows developers;
  the `trust` job of `.github/workflows/cli-git-trust.yml:189-232` is the only Windows CI job.
- Status:
  retained.
- Rust owner:
  `git_resolution.rs`,
  `process_identity.rs`,
  `index_lock_evidence.rs`,
  `shadow_repository.rs`,
  and `hook_dispatch.rs` (proposed).
- Consumer-level test:
  on a Windows host,
  run an ordinary commit with a hookdir hook,
  two concurrent commits,
  a foreign `index.lock` held by Git for Windows,
  and a `git worktree add` from a linked worktree;
  observe the outcomes of the Linux cases.
- Native state:
  absent.

## Harnesses and verification

### TypeScript unit suites

- Behavior:
  110 `*.unit.test.ts` files under `src`,
  run by the `test:unit` task (`package/git-policy/cli/mise.toml:81-82`).
  Many exercise the built artifact through the internal test exports.
- Spec:
  `SPEC.md:3620-3907` lists the required disposable fixtures.
- Consumers:
  local development;
  `.github/workflows/cli-git-trust.yml:180-184` runs two of the files at each supported Node floor,
  and lines 221 to 228 run three trust files on three operating systems.
- Status:
  retired with the TypeScript implementation
  (implementation plan lines 295 to 297).
  The fixture catalog of `SPEC.md:3620-3907` stays the list the native tests must cover
  (lines 304 to 316).
- Rust owner:
  each native module's tests,
  and the container suite for consumer behavior.
- Consumer-level test:
  not applicable;
  this entry is itself verification.
- Native state:
  51 `#[test]` functions exist under `src/native/` at `df25471a9`.

### Built-artifact fixtures

- Behavior:
  53 files under `src/trust/fixture`,
  consumer programs and their helpers,
  install the packed tarball in a disposable project and drive the built `git` shim.
  `test:built:trust` runs `src/trust/fixture/built-trust-consumer.ts` in a bounded `podman` container
  (`package/git-policy/cli/mise.toml:100-131`).
- Spec:
  `SPEC.md:4254-4283`.
- Consumers:
  local release verification.
  No CI workflow runs the task
  (`rg "built:trust|e2e|shadow|worktree-copy|concurrent" .github --hidden --no-ignore` matches nothing for them).
- Status:
  retired with the JavaScript artifact
  (implementation plan lines 295 to 297).
  Their non-trust cases feed the native container tests.
- Rust owner:
  the native container suite.
- Consumer-level test:
  not applicable.
- Native state:
  absent.

### Container end-to-end suite

- Behavior:
  `e2e/` holds a seeded concurrent-commit suite:
  a commit-shape trace mined from this repository's history (`e2e/commit-shape-trace.json`),
  a scenario catalog (`e2e/scenario-catalog-fixture.ts`),
  and invariant checks (`e2e/invariant-fixture.ts`,
  `e2e/leftover-fixture.ts`).
  The image builds Git 2.39.5,
  2.40.0,
  and 2.55.0 (`e2e/concurrent-commits.Containerfile`).
  Tasks:
  `e2e:concurrent:trace`,
  `test:e2e:concurrent:image`,
  and `test:e2e:concurrent` (`package/git-policy/cli/mise.toml:273-358`),
  the last with 2 GiB,
  2 CPUs,
  and no network.
  `e2e/README.md:445-460` records 38 scenarios per Git version and three passing seeds.
- Spec:
  `SPEC.md:3909-3988`.
- Consumers:
  local verification before a merge.
  No CI workflow runs it.
- Status:
  retained.
  The implementation plan lines 289 to 291 reuse the existing scenarios against the Rust executable,
  adapting the driver;
  line 300 replaces the multi-version Git fixture with the exact latest stable release.
- Rust owner:
  the adapted driver under `e2e/`,
  outside `src/native/`.
- Consumer-level test:
  run every seed-1,
  seed-2,
  and seed-3 scenario against the native executable on Git 2.56.0;
  observe every invariant of `SPEC.md:3973-3988`.
- Native state:
  absent.

### Performance harnesses

- Behavior:
  `perf:lifecycle-latency` measures paired wrapper and direct-Git samples per scenario
  and enforces budgets (`package/git-policy/cli/mise.toml:137-200`;
  `perf/lifecycle-latency-contracts.ts:212-231`,
  with a 2,000 ms ceiling).
  `perf:concurrent-commits` measures throughput,
  lock holds,
  and lost races (`package/git-policy/cli/mise.toml:202-271`).
  `perf/manual-push-latency-benchmark.ts` measures the manual-push gate.
  Recorded results are the JSON files under `perf/`.
- Spec:
  `SPEC.md:4089-4252`.
- Consumers:
  `.github/workflows/cli-git-performance.yml:70-114` runs the lifecycle benchmark when the package version changes
  and uploads the raw samples.
- Status:
  retained as an acceptance check
  (implementation plan lines 318 to 322).
  The trust scenarios `strict-mjs`,
  `strict-typescript`,
  and `relaxed-rebuild` retire with the trust subsystem.
  The rewrite scope lines 91 to 96 state that no numeric acceptance budget is settled for the new design.
- Rust owner:
  the harness under `perf/`,
  outside `src/native/`.
- Consumer-level test:
  run the lifecycle scenarios against the release native artifact with a positive control per scenario;
  observe wrapper-added medians and p95 beside the direct-Git baseline,
  with local processing reported apart from push latency.
- Native state:
  absent.

### CI workflows

- Behavior:
  `.github/workflows/cli-git-trust.yml` checks the Node runtime policy,
  builds and tests at each Node floor,
  and runs trust suites on Linux,
  macOS,
  and Windows.
  `.github/workflows/cli-git-performance.yml` runs the lifecycle benchmark on a version bump.
  `.github/workflows/final-newline.yml:38-39` runs `src/trust/fixture/final-newline-workflow.ts`,
  an isolated direct check of the final-newline policy.
  `.github/workflows/forbidden-strings.yml` runs the released scanner on changed files,
  independent of the wrapper.
- Spec:
  `SPEC.md:4059`,
  `4279-4281`.
- Consumers:
  pull requests and pushes to `main`.
- Status:
  retained for the final-newline check,
  the performance job,
  and the independent scanner job;
  the Node-runtime and trust jobs retire with their subjects.
  The implementation plan line 436 lists CI among the things updated together at cutover.
- Rust owner:
  none;
  workflow files.
- Consumer-level test:
  after cutover,
  each retained workflow passes on a pull request that uses the native executable.
- Native state:
  absent.

## Maintenance utilities

### hk Git-config cleanup

- Behavior:
  `cleanupHkGitConfig` (`src/maintenance/hk-config-cleanup.ts:56-157`) removes only keys beginning with `hook.hk-`
  from explicitly chosen Git configuration scopes.
  The command entry is `src/maintenance/hk-config-cleanup-command.ts`;
  it resolves real Git through `@monochromatic-dev/git-executable-core`.
- Spec:
  no `SPEC.md` section states it;
  `doc/runbook/remove-retired-hk-git-config.md` is the procedure.
- Consumers:
  the root task `cleanup:hk-git-config` (`mise.toml:655-661`)
  and the fixture task `test:hk-config-cleanup` (`package/git-policy/cli/mise.toml:133-135`).
  The npm package excludes `src/maintenance` (`package/git-policy/cli/package.json:24`).
- Status:
  retained as a repository task outside the wrapper executable.
  The implementation plan does not name it;
  see "Open questions".
- Rust owner:
  none proposed.
- Consumer-level test:
  the existing disposable-scope fixture,
  run against whichever implementation remains.
- Native state:
  absent.

## Consumers

One list of who reaches the wrapper,
by route.

- Installed-bin wiring:
  root `package.json:36` depends on `@monochromatic-dev/git-policy-cli`;
  the package manager writes `node_modules/.bin/git`,
  a shell shim that starts Node on `dist/final/node/index.mjs`;
  root `mise.toml:1252-1253` puts `node_modules/.bin` first on `PATH`,
  and line 1269 adds the package's own `node_modules/.bin`.
- Repository configuration:
  `cli-git.config.ts`.
- Root mise tasks:
  `cleanup:hk-git-config` (`mise.toml:655-661`);
  `changeset:version`,
  which calls `//package/git-policy/repository:bump:dependents` (`mise.toml:1193-1197`);
  the `FORBIDDEN_STRINGS_RULES` environment entry (`mise.toml:1321-1327`).
- Package mise tasks:
  `package/git-policy/cli/mise.toml`
  (`run`,
  `build`,
  `lint*`,
  `test:unit`,
  `verify:supported-runtime`,
  `pack:npm`,
  `test:built:trust`,
  `test:hk-config-cleanup`,
  `perf:*`,
  `e2e:*`,
  `test:e2e:*`,
  and the native tasks at lines 1 to 19).
- file-enforcer:
  the optional-policy mirrors (`file-enforcer.config.ts:2260-2322`)
  and the generated scanner rules file (`2258`).
- CI workflows:
  `cli-git-trust.yml`,
  `cli-git-performance.yml`,
  `final-newline.yml`.
- Hooks:
  repository Git hooks run under the wrapper's dispatcher during commits.
  No tracked file registers a Git hook
  (the hk configuration was removed,
  `doc/handover/cli-git-policies-platform.md` section "Retirement checkpoint on 2026-07-11";
  `git config --get core.hooksPath` is unset in this checkout).
  This checkout's untracked `.git/hooks` holds the four Git LFS hooks,
  `post-checkout`,
  `post-commit`,
  `post-merge`,
  and `pre-push`,
  each running `git lfs <event>` through `PATH`.
  Each therefore re-enters the wrapper as a nested invocation of the external command `lfs`,
  which the classifier treats as configuration-loading (`src/trust/command-classification.ts:280-281`).
- Other packages:
  `package/git-executable/core` (the resolver the wrapper uses and that must recognize it);
  `package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts` (uses that resolver);
  `package/git-policy/api`,
  `package/git-policy/repository`,
  `package/git-policy/forbidden-strings`,
  and `package/git-policy/markdown-lint` (policy sources mirrored into the wrapper);
  `package/forbidden-strings/cli` (the scanner);
  `package/cli/markdown-lint` (the Markdown command);
  `package/config/pnpr/config.yaml:76-80` (private-registry publishing list);
  `.changeset/config.json:21`.
- Agent tooling:
  agent shells reach the wrapper through `PATH`.
  `AGENTS.md` rules CLG,
  CPN,
  APG,
  APQ,
  GCE,
  and GCA describe working with its guards and auto-push.
  `rg --hidden --no-ignore --glob '!**/worktrees/**' "cli-git|git-policy" .claude` matches nothing,
  so no agent hook or skill there names the wrapper.

## Responsibilities the plan adds

These have no incumbent behavior and are not counted in the status tally.

- JSONC configuration loading from repository-root `cli-git.config.jsonc`
  through `package/jsonc-edit/core-rust` (implementation plan lines 55 to 61).
  Native state:
  implemented as library modules (`src/native/config_file.rs`,
  `src/native/config_parse.rs`).
- Legacy configuration migration diagnostic (lines 440 to 443).
  Native state:
  implemented as library code.
  `load_repository_config` fails when only a legacy file exists (`src/native/config_file.rs:273-277`)
  and returns the legacy paths for a notice when both exist (`146-153`,
  `298-303`).
- Retired trust-command explanation (line 444).
  Native state:
  in progress.
- Unsupported-environment failure for a Git without required behavior (lines 22 to 25 and 235 to 237).
  Native state:
  absent.
- In-process scanner linkage with the standalone scanner routed through the same core (lines 128 to 152).
  Native state:
  the scanner side is implemented (`doc/handover/scanner-native-verification.md`);
  the wrapper side is absent.
- First-party linter selection and a one-rule temporary JSONC configuration for the Markdown policy
  (lines 181 to 189).
  Native state:
  absent.
- Native launcher,
  installed-bin wiring,
  and native-wrapper recognition in the TypeScript resolver (lines 425 to 434).
  Native state:
  absent.
- Container,
  mutation,
  and fuzz gates,
  with a sibling fuzz package (lines 324 to 419).
  Native state:
  the bounded container runner exists
  (`bin/test-native-container.mjs`,
  `bin/build-git-test-image.mjs`,
  `package/git-policy/cli/mise.toml:1-19`);
  the mutation runner and fuzz package are in progress.
- Rollback:
  the previous executable stays available until native installation and recovery checks pass (line 446).

## Spec and code disagreements

Both sides are recorded;
none is resolved here.
Items marked "by reading" were derived from the cited source lines
and were not observed at the consumer boundary in this survey.

- Built-in order and IDs:
  `SPEC.md:1237-1241` and the `BuiltInPolicyId` type at `SPEC.md:233-237` list four built-ins;
  `SPEC.md:3995-3999`,
  `src/policy-engine/built-ins.ts:21-27`,
  and `src/api/config-types.ts:59-64` list five,
  with `final-newline` last.
- Patch target revision:
  `SPEC.md:582` makes `revision` being `ABSENT_GIT_VALUE` a condition of a valid patch,
  and `SPEC.md:373` calls a candidate with that value mutable.
  The code gives commit candidates their blob ID as `revision`
  (`src/policy-engine/commit-transaction-candidates.ts:185`),
  throws when a patch target's revision is the absent symbol
  (`src/policy-engine/final-newline-policy.ts:62-63`),
  and requires the patch's `index` header to name that revision
  (`src/policy-engine/commit-transaction-patch.ts:103-105`,
  `164`).
- Post-commit candidates:
  `SPEC.md:1085` says `candidates()` enumerates the landed commit's complete recursive tree.
  The code supplies only the paths the landed commit changed,
  through `git diff-tree --root --no-commit-id -r -z -m`
  (`src/policy-engine/post-commit-facts.ts:119-154`).
  `doc/handover/cli-git-policies-platform.md`,
  section "Release-readiness checkpoint on 2026-07-11",
  records the restriction to the landed delta as intended.
- Event sequence (by reading):
  `SPEC.md:1306` says `sequence` starts at `0` for each invocation and increases by one in emission order.
  Each engine run numbers from `0` (`src/policy-engine/engine.ts:298`),
  and one invocation can write several batches:
  pre-forward (`src/bin.ts:277-280`),
  manual push (`336-339`;
  `src/policy-engine/manual-push-lifecycle.ts:236`),
  and post-commit (`src/bin.ts:412-415`;
  `src/policy-engine/post-commit-lifecycle.ts:140-153`,
  `177-190`).
  A commit with a pre-forward event and a post-commit event therefore repeats `sequence` `0`.
- Canonical command facts (by reading):
  `SPEC.md:784-785` says `effectiveCwd` is canonicalized and `repositoryRoot` is the canonical real-Git top level.
  `parseGlobalOptions` resolves `-C` lexically without `realpath` (`src/parse-global-options.ts:68-83`),
  and the engine uses `effectiveCwd` as `repositoryRoot` when no lifecycle supplies one
  (`src/policy-engine/engine.ts:115`).
- Finding validation:
  `SPEC.md:564-574` requires kebab-case codes and a location inside the candidate's byte length.
  `findingsAreValid` checks only non-empty `code` and `message` (`src/policy-engine/policy-stage.ts:60-71`),
  and `rg "byteStart|byteEnd" src` outside tests matches only the type declaration and the event copy
  (`src/api/policy-types.ts:183-193`;
  `src/policy-engine/events.ts:417-418`).
- Policy cancellation:
  `SPEC.md:383-385` says cancellation uses `context.signal`.
  The engine passes the signal of a fresh `AbortController` that nothing aborts
  (`src/policy-engine/engine.ts:119`),
  and the wrapper installs no signal handler.
- Management usage text:
  `SPEC.md:808-811` requires exactly one of `--all` or `-- <pathspec>...`.
  The usage text prints both as optional (`src/management-parser.ts:22-28`);
  the enforcement matches the spec (`src/management.ts:189-192`).
- Shipped plugin exports:
  `SPEC.md:71-73` names `repositoryPolicyPlugin` and,
  conditionally on issue #354,
  `forbiddenStringsPlugin`.
  `src/authoring.ts:28-52` exports both unconditionally and also `markdownLintPlugin`.
- Recovery failure code:
  `SPEC.md:3526-3528` says malformed transaction state fails closed and names no code.
  `src/bin.ts:432-438` reports it as `content-unavailable`,
  while `SPEC.md:1457` also defines `transaction-failed`.
- Planning text against the spec:
  `doc/planning/cli-git-concurrent-commits.md:347-353` says a private ref under `refs/cli-git/`
  protects pending commits.
  `SPEC.md:2184-2191` and `doc/decision/cli-git-concurrent-commits.md:482-483` say no such ref exists
  and the shadow object store protects them.
- Windows verification:
  `doc/decision/cli-git-concurrent-commits.md:475-478` says Windows verification of the shadow links runs in CI.
  The only Windows job runs trust and filesystem-identity tests (`.github/workflows/cli-git-trust.yml:189-232`).
- Incumbent against native global options:
  the incumbent skips a value after `--super-prefix` and has no entry for `--config-env` or `--shallow-file`
  (`src/parse-global-options.ts:12-20`),
  treats every unknown dash-led token as a flag (`166-174`),
  and short-circuits only on `--version`,
  `-v`,
  `--help`,
  and `-h` (`25-30`).
  The native table has `--config-env` and `--shallow-file`,
  no `--super-prefix` (`src/native/global_arguments.rs:37-46`),
  reports an unknown option as `InvalidOption` (`149-152`),
  and also treats `--html-path`,
  `--man-path`,
  `--info-path`,
  `--list-cmds=`,
  and a bare `--exec-path` as queries (`95-120`).
  By reading,
  the incumbent takes the value of a separated `--config-env <name>=<var>` as the subcommand.
- Incumbent against native classification:
  the incumbent's mutating `branch` long flags omit `--delete-merged`,
  `--set-upstream`,
  and `--create-reflog`,
  and its short letters omit `u` and `t`
  (`src/trust/command-classification.ts:49-69`);
  the native table includes them (`src/native/config_loading.rs:54-68`,
  `179`).
  The incumbent ignores unknown long options when classifying `branch` and `tag`
  (`src/trust/command-classification.ts:222-238`);
  the native classifier loads configuration for them (`src/native/config_loading.rs:167-168`).

## Incumbent defects and stale comments

Recorded for the port,
not fixed.

- Stale parser-library comments:
  43 mentions of "optique" remain in 14 files under `src`
  (`rg --count-matches --ignore-case optique src --glob '!src/native/**'`),
  for example `src/escape-hatch.ts:55-58`,
  `src/rule/atomic-push.ts:28`,
  and `src/rule/commit-only.ts:181`.
  The package no longer depends on it (`package/git-policy/cli/package.json:50-59`),
  and `src/management-parser.ts:2` says the grammar replaced that facade.
- `README.md:1185-1190` tells authors to add rules to a `RULES` array in `src/index.ts`;
  `src/index.ts:1-41` has no such array.
- The version banner lists eight behaviors by hand and omits `final-newline`
  (`src/post-command-output.ts:89-92`).
- `src/policy-engine/engine.ts:2` and `41` still call the module a first slice with a built-in-only schema,
  and the message at `235` says "Unknown built-in policy ID" for any unknown ID,
  plugin IDs included.
- Recursion over linear input:
  `walkGlobalOptions` (`src/parse-global-options.ts:101-181`)
  and `filterFlagEscapeHatch` (`src/escape-hatch.ts:178-227`) recurse once per argument.
- Three pinned Git versions describe one grammar:
  classification fixtures target Git 2.54.0 (`src/trust/command-classification.ts:4`),
  the built-in command table comes from Git 2.55.0 (`src/git-builtin-commands.ts:2`),
  and the native tables target Git 2.56.0 (`src/native/global_arguments.rs:1`).
- Newline-delimited parsing of `git rev-parse` output:
  `parseIdentityMetadata` splits on LF (`src/git-worktree-identity.ts:248-254`),
  so by reading a Git directory path that contains LF is misparsed.
- Duplicate constants:
  `ref-updated.json` and `index-installed` are declared in both
  `src/policy-engine/commit-transaction-journal.ts:19-23`
  and `src/policy-engine/commit-transaction-journal-states.ts:40-45`;
  `journal.json` in both `src/policy-engine/commit-transaction-recovery-journaled.ts:60`
  and `src/policy-engine/commit-transaction-recovery-landing.ts:81`.
- Verbose logging is tied to a Git argument:
  the logger enables verbose output when `process.argv` contains `--verbose`
  (`package/logger/core/src/sink/console.ts:69-77`),
  so by reading `git commit --verbose` also turns on wrapper debug lines on stderr.
- A Git child ended by a signal exits `1` (`src/bin.ts:454-456`),
  not a status that names the signal.
- The tool-cache allowlist has no platform branch
  (`src/allowed-worktree-dirs.ts:60-88`):
  it derives one cache location from `UV_CACHE_DIR`,
  `XDG_CACHE_HOME`,
  or `<home>/.cache`.
- `SPEC.md:2460-2462` leaves the hook shim's shebang form on Windows,
  and for an executable path with spaces,
  pending verification.
- `doc/handover/cli-git-concurrent-commits.md` keeps a "Next actions" list (lines 341 to 364)
  of steps its own "Landed" section (lines 5 to 52) reports as done.
- Retained code depends on the trust directory:
  17 non-trust files import `src/trust/registry-io.ts` for private-file helpers,
  and `src/bin.ts:35`,
  `44` import classification and concurrency defaults from `src/trust/`.
- No CI workflow runs the transaction,
  lock,
  worktree-copy,
  or end-to-end suites,
  and none runs any retained wrapper behavior on macOS or Windows
  (`rg "e2e|built:trust|test:unit|shadow|worktree-copy|concurrent" .github --hidden --no-ignore`
  matches only the two-file `test:unit` step of the Linux `supported-runtime` job
  and unrelated fuzz workflows).

## Open questions

The planning documents do not determine these.
Each names the text that stops short.

- Fixed policy names and optional-policy defaults.
  The implementation plan lines 55 to 61 require unknown policy IDs to be errors
  but name no IDs or defaults.
  The native registry keeps `markdown/`,
  `mono/`,
  and `security/` prefixes and defaults the four optional policies to `Off`
  (`src/native/policy_registry.rs:95-160`).
  The incumbent runs `mono/dependent-version-bump` at `error` without listing it.
  Should the optional policies default to their incumbent severities,
  and are these the accepted names?
- Policy option surface.
  The rewrite scope lines 126 to 131 forbid `executable` and `command`.
  Nothing states which other options stay.
  The native schema keeps `builtinRules`,
  `rules`,
  and `exclude` (`src/native/config_schema.rs:75-93`).
  Does `FORBIDDEN_STRINGS_RULES` remain the way to name the rules file?
- Configuration root.
  The plan says "repository-root" (line 55).
  The incumbent uses the nearest ancestor of the effective directory that holds a Git marker
  (`src/trust/config-discovery.ts:110-131`),
  which is a linked worktree's own root and ignores `--git-dir` and `--work-tree`.
  Which root does the native wrapper pass to `load_repository_config`,
  and what happens in a bare repository?
- Legacy configuration during the rollback window.
  The plan line 446 keeps the previous executable available,
  which needs `cli-git.config.ts`,
  while lines 440 to 443 require a migration diagnostic.
  The native loader reports a legacy file beside a JSONC file on every load
  (`src/native/config_file.rs:146-153`).
  Is a notice on every configuration-loading command intended while both wrappers are installed?
- Retired trust commands.
  The plan line 444 says they explain the retirement.
  It does not give the exit status,
  the stream,
  or whether `git cli-git status` keeps its name for another purpose.
- Signals.
  The plan lines 84 to 91 port "signals" without stating the contract.
  The incumbent has no handler and maps a signaled Git to exit `1`.
  Should the native wrapper forward signals to the child,
  and which exit status reports a signaled child?
- Event sequence numbering.
  The spec and the code disagree (see "Spec and code disagreements").
  The plan lines 315 to 316 make the accepted behavior the oracle.
  Does the native wrapper number events once per invocation?
- Non-UTF-8 paths in events and journals.
  The plan lines 47 to 49 preserve argument bytes and path representations.
  JSONL events carry paths as JSON strings (`SPEC.md:1337`),
  and `captured.json` stores Latin-1 decoded path bytes (`SPEC.md:3138-3139`).
  The encoding of a non-UTF-8 path in an event is not stated.
- Engine failure codes that lose their source.
  `plugin-threw`,
  `policy-incomplete`,
  `config-untrusted`,
  `config-changed`,
  `trust-consent-unavailable`,
  and `trust-failed` (`src/policy-engine/events.ts:88-103`) describe plugins and trust.
  Which code reports a failed built-in policy,
  such as a scanner error?
- Hook dispatcher and generated hooks without Node.
  The shim and the manual-push probe hook are generated Node programs
  (`src/hook-dispatch/hook-dispatch-program.ts:27`;
  `src/policy-engine/manual-push-hook.ts:344-405`).
  The plan lines 425 to 426 remove Node from the launch path.
  The native form of these hook entries,
  and its Windows form,
  is not stated.
- Lock and journal interoperability across versions.
  The plan lines 228 to 233 keep formats where practical with cross-version fixtures.
  Birth-identity strings are built from `ps` and PowerShell output
  (`src/policy-engine/commit-transaction-process-identity.ts:135-157`).
  Must the native wrapper produce identical strings so each version judges the other's locks correctly?
- Legacy single-journal directory.
  The incumbent still recovers `cli-git-transaction` (`SPEC.md:2145-2146`).
  The plan forbids silently discarding old state (lines 230 to 233)
  but does not say whether this older format is ported or reported.
- Unsupported Git detection.
  The plan lines 22 to 25 support the latest stable release and forbid a network query at run time,
  and lines 235 to 237 require an unsupported-environment failure.
  When is the check made,
  and is it a version comparison or a behavior probe?
- Post-command output.
  The plan does not mention the version banner or the status note.
  Does the native wrapper keep adding a line to `git --version` output?
- Tool-cache allowlist.
  It is compiled in and Linux-shaped.
  The plan does not say whether it stays compiled in,
  moves to JSONC,
  or gains other platforms.
- Test phase markers.
  The reused end-to-end scenarios need `CLI_GIT_TEST_ONLY_PHASE_SIGNAL`.
  The plan does not say whether the release executable carries the markers
  or a separate test build does.
- macOS and Windows coverage.
  The plan lines 300 to 303 keep existing coverage,
  which today is trust-only on those systems.
  Which native tests run there?
- TypeScript utilities beside the wrapper.
  `cleanup:hk-git-config` and `//package/git-policy/repository:bump:dependents` are TypeScript tasks.
  The second shares planning code with the dependent-version policy.
  Do they stay TypeScript,
  leaving two implementations of the bump plan?
- Verbose diagnostics switch.
  The incumbent switch is `MONOCHROMATIC_VERBOSE` or a `--verbose` argument.
  The native switch is not stated.
- Performance acceptance.
  The rewrite scope lines 91 to 96 say no numeric budget is settled,
  while `SPEC.md:4038` and `perf/lifecycle-latency-contracts.ts:212` hold a 2,000 ms ceiling.
  Which numbers gate the native artifact?
- Markdown policy without the linter.
  The plan lines 181 to 189 select the coordinated installation's linter.
  The behavior when that binary is missing is not stated.

## Related documents

- `doc/decision/cli-git-rust-rewrite.md`:
  the accepted decision.
- `doc/planning/cli-git-rust-rewrite.md`:
  scope,
  settled interview requirements,
  and the installed-wrapper baseline.
- `doc/planning/cli-git-rust-implementation.md`:
  module responsibilities,
  sequence,
  verification gates,
  and cutover.
- `doc/handover/cli-git-rust-implementation.md`:
  execution state and delegations.
- `doc/decision/cli-git-policies-platform.md` and `doc/handover/cli-git-policies-platform.md`:
  the policy platform and its checkpoints.
- `doc/decision/cli-git-concurrent-commits.md`,
  `doc/planning/cli-git-concurrent-commits.md`,
  and `doc/handover/cli-git-concurrent-commits.md`:
  concurrent commits.
- `doc/planning/cli-git-policy-added-paths.md`:
  policies that add paths to a commit.
- `doc/planning/cli-git-noninteractive-trust-ux.md`:
  trust diagnostics,
  retired with the trust subsystem.
- `doc/handover/cli-git-cac-migration.md`:
  why the package owns its argument parser.
- `package/git-policy/cli/doc/concurrent-commits-implementation-plan.md`:
  the incumbent code map.
- `doc/planning/unified-linter-coverage-ledger.md`:
  the linter ledger this document is modeled on.
