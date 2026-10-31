# Rust file-enforcer cutover inventory

## Status

Preparation only.
Nothing in this document has been performed on the repository:
no consumer was edited,
no task was repointed,
and the TypeScript tool still produces every managed file.
The decision is [`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md)
("Cutover",
"Elsewhere" and "Consequences");
the state is [`doc/handover/file-enforcer-rust-rewrite.md`](../handover/file-enforcer-rust-rewrite.md).

Measured on 2026-10-31 in a worktree at commit `9805cb3cb5`,
which is `main` plus the merged wiring branch (`worktree-agent-a23c3740c67ba36d9`:
the three rule kinds in the cli crate,
the browserslist lock at the repository root)
and the merged core-hardening branch (`worktree-agent-ad604356a8bd46267`:
`file:line:col` config errors and a regenerated schema).

Every line number in this document was measured with `rg --line-number` in that worktree;
the command is stated per section.
Where a line number cited by the decision record has drifted,
the measured one is given and the drift is noted.

## What this document decides and what it leaves open

The document inventories the one-switch commit and proves the toolchain plan conversion on a disposable copy.
It decides nothing the owner has not decided.
Five points need an owner answer before the switch,
each stated in "Open decisions":
the shape of the root `mise.toml`,
how CI and containers obtain the two binaries,
the plan entries that install the binaries,
the IDE launcher's explicit path,
and the `asset` hints of the machine-global plan.

## Preconditions

Each must hold before the switch commit is made.
The order is the order of work,
not of importance.

1.  Version bump and release of `monochromatic-jsonc-edit`.
    The published 0.1.1 (crates.io `max_version` 0.1.1,
    last updated 2026-10-06,
    read with `curl`)
    predates `parse_jsonc_with_spans`,
    which the merged `package/jsonc-edit/core-rust/src/span.rs` adds as a new file
    and which `monochromatic-file-enforcer` now uses.
    `package/file-enforcer/core-rust/Cargo.toml:36` pins `=0.1.1`;
    it must name the bumped version.
    The crate has a release job already (`.github/workflows/cargo-publish.yml:29` path,
    `:865` onward),
    Trusted Publishing.
2.  A version bump of `monochromatic-mpm-shadow`.
    The merged crate still says `version = "0.1.1"` (`package/mpm-shadow/cli/Cargo.toml:3`),
    which is the version crates.io has had since 2026-10-29T14:00Z
    (`curl` of the crates API:
    `max_version` 0.1.1,
    versions 0.1.0 and 0.1.1 created 2026-10-29),
    before the toolchain commands merged on 2026-10-30,
    so the published 0.1.1 is the shadow-only build;
    a fresh build of the merged crate prints `mpm-shadow 0.1.1`.
    The existing release jobs (`.github/workflows/cargo-publish.yml:1304` onward) publish on the version change.
3.  First publication of the five new crates,
    by hand,
    as `doc/runbook/publish-crate-first-time.md` describes.
    Another agent is preparing it.
    The five,
    with their measured versions and dependency order:
    `monochromatic-xml-edit` 0.1.0 (`package/xml-edit/core-rust/Cargo.toml:3`),
    `monochromatic-browserslist` 0.1.0 (`package/browserslist/core-rust/Cargo.toml:3`),
    `monochromatic-dependency-catalog` 0.1.0 (`package/dependency-catalog/core-native/Cargo.toml:3`),
    `monochromatic-file-enforcer` 0.1.0 (`package/file-enforcer/core-rust/Cargo.toml:3`,
    needs the bumped jsonc-edit),
    and `monochromatic-file-enforcer-cli` 0.1.0 (`package/file-enforcer/cli/Cargo.toml:3`),
    which pins the other four by `=0.1.0` (`package/file-enforcer/cli/Cargo.toml:47,54,56,58`).
    None has a dispatch option,
    a path trigger or a release job today:
    `.github/workflows/cargo-publish.yml:10-17` lists eight crates and `:26-33` eight paths,
    and none of the five is among them.
    A binary release job is also needed for `monochromatic-file-enforcer-cli`,
    or `cargo binstall` compiles it
    (`doc/troubleshooting/cargo-binstall-compiles-when-github-api-rate-limited.md`).
4.  The version-keyed cargo-binstall installs in `mpm-shadow sync`.
    The owner decided that mpm-shadow installs cargo-binstall tools under version-keyed directories
    in the toolchain root,
    never into the shared `~/.cargo/bin`,
    with each worktree's tree linking its own lock's version.
    The coordinator relayed this decision to this inventory;
    the decision record's "Toolchain (mpm-shadow)" text and the cited reason,
    `doc/troubleshooting/toolchain-sync-older-lock-downgrades-shared-cargo-installs.md`,
    are not in the tree at commit `9805cb3cb5` (`rg` finds neither),
    so they are expected from the branch that carries them.
    Another agent is building it.
    The first `mpm-shadow sync` of the switch must leave `~/.cargo/bin` alone,
    because other worktrees still on the TypeScript toolchain link to the files there.
    The rehearsal lock diff in "Rehearsal" was made before that branch and must be rerun after it merges.
5.  A clean container fetches the published `file-enforcer` and `mpm-shadow` binaries,
    verifies their SHA-256,
    and runs `file-enforcer check` and `mpm-shadow lock --check` on a copy of the repository.
    The container is bounded (`podman run --memory=2g --cpus=2 --rm`).
    Not done:
    no binary of the new `file-enforcer` is published.
6.  The comparison with the TypeScript tool's output,
    rerun at the switch revision.
    Partial result now:
    see "Comparison with the TypeScript output".
7.  `package/vm-builder/cli` has its private process runner.
    It still imports `exec` from the old package at `package/vm-builder/cli/src/import.ts:9`
    and `package/vm-builder/cli/src/build-and-import.ts:20`,
    and depends on it at `package/vm-builder/cli/package.json:16`.
    The decision lists it under "Elsewhere",
    not in the switch commit's list,
    so it is a third consumer that must be migrated before `package/file-enforcer/core` is deleted.
    Done on the worktree branch `worktree-agent-a12d076e7eb77be7b`:
    `package/vm-builder/cli/src/exec.ts` (direct spawn through `nano-spawn`, no platform-aware form),
    the dependency is gone from `package/vm-builder/cli/package.json` and `pnpm-lock.yaml`,
    and `rg 'file-enforcer' package/vm-builder` finds nothing.
8.  The per-user config for `file-enforcer user write` exists on the owner's machine
    (`<config dir>/monochromatic-dev/file-enforcer/user.config.jsonc`).
    No sample is committed (`rg` finds the path only in the Rust sources and the CLI README);
    the TypeScript policy it replaces is `manageHarperLsp4ij` (`file-enforcer.config.ts:1399-1440`).
    The owner writes it;
    this document does not touch `~/.config`.
9.  The remaining branches (loose ends of the wiring branch,
    the version-keyed installs) are merged,
    and the rehearsal is rerun on the merged `mpm-shadow`.

## Order of operations

1.  Satisfy the preconditions.
2.  In a throwaway worktree,
    run the converter against `plan.tools.data.ts` (appendix),
    run `mpm-shadow update`,
    `lock --check` and `update --check` in a disposable home,
    and show the lock diff to the owner (the review surface the handover names).
3.  The switch commit,
    made from a clean `main` tip,
    contains these edits and no others:
    1.  Write `toolchain.plan.jsonc` (converter output) and `toolchain.lock.json` (rebuilt by `mpm-shadow update`).
        Delete `mpm.packages.toml` and `mpm.managers.toml`
        and the plan data under `package/file-enforcer/core/src/toolchain/`.
    2.  Repoint tasks (section "Tasks").
    3.  Repoint CI (section "CI").
    4.  Repoint the `Dockerfile`,
        the IDE launcher and its tests (sections "Dockerfile" and "IDE launcher").
    5.  Apply the `mise.toml` decision (section "Root mise.toml").
    6.  Migrate or retire the remaining consumers of the old package (section "Consumers of the old package").
    7.  Reword `PNPR_CONFIG_HEADER_LINE` (section "pnpr").
    8.  Fix the doc references (section "Documents").
    9.  Delete `package/file-enforcer/core`,
        then regenerate `pnpm-lock.yaml` with `pnpm install` and review the diff.
    10. Run `file-enforcer write` from the new binary and commit its output as-is
        (`CLAUDE.md` after the `AGENTS.md` edit,
        `package/config/pnpr/config.yaml` after the package removal).
        Output need not be byte-identical to the TypeScript tool's.
4.  After the switch:
    `file-enforcer check`,
    `mpm-shadow lock --check`,
    `mpm-shadow status`,
    the IDE launcher against a real worktree,
    the CI run on the switch commit.
5.  Convert the machine-global plan (section "Global plan conversion").
    This is the owner's step on the owner's machine,
    and it follows the switch because the old tool disappears with it.

## Inventory

### Tasks

Command:
`rg --line-number "file-enforcer|sync:files|toolchain[:/]|prepare:pnpm" mise.no-env.toml mise.toml task.ts`.
`mise.toml` is today generated from `mise.no-env.toml`:
it carries the same text one line later for the first 1319 lines (one added header line),
then the `[env]` block (`mise.toml:1322-1444`).
Lines are given for `mise.no-env.toml`,
the source;
add one for `mise.toml`.
Task names stay,
so no caller changes;
only the `run` bodies change.

- `mise.no-env.toml:626-628`,
  `toolchain:sync`:
  `run = "node package/file-enforcer/core/src/toolchain/sync.ts"` becomes `run = "mpm-shadow sync"`.
- `mise.no-env.toml:630-632`,
  `toolchain:status`:
  `.../status.ts` becomes `mpm-shadow status`.
- `mise.no-env.toml:634-636`,
  `toolchain:update`:
  `.../update.ts` becomes `mpm-shadow update`
  (its description already says `--check` reports without writing;
  `mpm-shadow update --check` does).
- `mise.no-env.toml:638-640`,
  `toolchain:sync:global`:
  `.../sync.ts --global` becomes `mpm-shadow sync --global`.
- `mise.no-env.toml:642-644`,
  `toolchain:status:global`:
  becomes `mpm-shadow status --global`.
- `mise.no-env.toml:646-648`,
  `toolchain:update:global`:
  becomes `mpm-shadow update --global`.
- New task,
  not present today:
  `toolchain:lock:check` running `mpm-shadow lock --check`
  (the toolchain drift gate the decision names for CI beside `file-enforcer check`).
- `mise.no-env.toml:1041-1042`,
  `bootstrap`,
  and `:1112-1113`,
  `prepare:android`:
  `run = ["mise run toolchain:sync"]`;
  unchanged,
  they alias `toolchain:sync`.
- `mise.no-env.toml:1082-1084`,
  `prepare:pnpm:others:files`:
  `description = "Sync file-enforcer managed files"`,
  `run = "node file-enforcer.config.ts"`
  becomes `run = "file-enforcer write"`.
  This is the line through which `mise run prepare` and `prepare:pnpm` run the sync.
- `mise.no-env.toml:1149-1152`,
  `sync:files` (alias `file-enforcer`):
  `run = "node package/file-enforcer/core/src/cli.ts"` becomes `run = "file-enforcer write"`.
- `mise.no-env.toml:1154-1156`,
  `watch:sync:files`:
  `run = "node package/file-enforcer/core/src/cli.ts --watch"` becomes `run = "file-enforcer watch"`.
- New task,
  not present today:
  `check:files` running `file-enforcer check`.
- `mise.no-env.toml:324` (and `mise.toml:325`),
  in the `dispatch_workspace_node` variable:
  `existsSync(join(configRoot, 'mise.no-env.toml'))` decides the monorepo root.
  If `mise.no-env.toml` is renamed or deleted this test is false everywhere and the fallback
  `resolve(configRoot, '..', '..', '..')` is used from the wrong directory for root tasks.
  Change it to test `mise.toml` or `toolchain.plan.jsonc`.
- `mise.no-env.toml:3-4` and `mise.toml:1,4,5`:
  the "generated from" header lines become false and are removed or reworded.
- `task.ts:335-337`,
  `toolchain:sync`:
  `run: 'node package/file-enforcer/core/src/toolchain/sync.ts'`
  becomes `run: 'mpm-shadow sync'`.
- `task.ts:339-341`,
  `toolchain:status`:
  becomes `mpm-shadow status`.
- `task.ts:343-345`,
  `toolchain:update`:
  becomes `mpm-shadow update`.
- `task.ts:347-353`,
  `bootstrap` and `prepare:android`:
  `runTaskCli('toolchain:sync')`;
  unchanged.
- `task.ts:355-361`,
  `prepare:pnpm`:
  the third entry,
  `'node file-enforcer.config.ts'` (`task.ts:360`),
  becomes `'file-enforcer write'`.
  `task.ts` has no `--global` or `sync:files` task;
  CI never needed them.
- `package.json:35`:
  `"@monochromatic-dev/file-enforcer-core": "workspace:*"` is removed
  (the root config imported it at `file-enforcer.config.ts:33`,
  which is deleted).
- `README.md:195-199,219-232,251`:
  `mise run toolchain:sync` and the `node package/file-enforcer/core/src/toolchain/*.ts` commands
  (`README.md:199,222,227,232`) become `mpm-shadow sync`,
  `update`,
  `status` and `bin-paths`;
  `README.md:251` names the deleted directory.
- `AGENTS.md:869` (rule `WC2`):
  "check `file-enforcer.config.ts` before editing or adding one" becomes `file-enforcer.config.jsonc`.
  `CLAUDE.md:918` is generated from it,
  so `file-enforcer write` regenerates it in the same commit.
  `AGENTS.md:549` (`UPN`) and `:767` (`CM7`) name tasks that keep their names.

The `mpm-shadow` and `file-enforcer` programs must be found on `PATH` when a task runs.
Today `node` is.
A fresh clone has neither binary until something installs them:
see "Open decisions",
the bootstrap.

### CI

Command:
`rg --line-number --hidden "prepare:pnpm|prepare:android|sync:files|file-enforcer|toolchain.lock" .github .depot`.

Steps that run or depend on the TypeScript tool:

- `.github/workflows/advisory-core.yml:91`,
  `run: task-run prepare:pnpm`.
  That task runs the sync (`task.ts:360`),
  then the steps at `:92-118` diff the result.
- `.github/workflows/current-baseline.yml:23`,
  `run: task-run prepare:pnpm`.
- `.depot/workflows/current-baseline.yml:26`,
  `run: task-run prepare:pnpm`.
- `.depot/workflows/weekly-native.yml:68`,
  `run: task-run prepare:pnpm`.
- `.depot/workflows/weekly-native.yml:108` and `.github/workflows/music-player-release.yml:1082`,
  `task-run prepare:android`,
  which is `toolchain:sync` (the TypeScript sync today,
  `mpm-shadow sync` after the switch).
  Each runner needs `mpm-shadow` before this step.

Workflows that read `toolchain.lock.json` with `jq` need no edit,
because the lock's frozen surface is unchanged (see "Rehearsal"):
`.github/actions/task-cli-setup/action.yml:42,46`,
`.github/workflows/kotlin-linter-publish.yml:115,124,125`,
`.github/workflows/music-player-release.yml:253-259,358,983,984,1041-1051,1067,1146,1602`,
and `Dockerfile:27-28,36-37`.
The `task-cli-setup` action installs Node and pnpm only (`action.yml:42-52`).

Design the decision asks for ("checking committed outputs before any regeneration,
then local outputs separately"):

1.  Fetch the two binaries (gap:
    no step does;
    see "Open decisions").
2.  `file-enforcer check` before `task-run prepare:pnpm`.
    Measured on a fresh worktree,
    `check` exits 1 with 26 `drift-missing` paths that all belong to four rules whose outputs
    are gitignored:
    `skills-claude` (12),
    `skills-factory` (12),
    `mise-bin-list` (1),
    `browserslist-targets` (1).
    No committed output drifted (`CLAUDE.md`,
    `LICENSE`,
    every `LICENSES/`,
    every `Cargo.toml`,
    `rust-catalog.toml`).
    So the committed-output gate needs a filter on the JSON lines.
    This filter must print nothing:

    ```sh
    # CI step sketch, not yet run in a workflow
    file-enforcer check | jq -c 'select(.type=="drift"
      and (.rule|IN("skills-claude","skills-factory","mise-bin-list","browserslist-targets")|not))'
    ```

    The filter was run against the measured output and selected zero lines;
    it has not been run inside a workflow.
    There is no per-rule option on `check`,
    so the list of local rule ids lives in the workflow.
3.  `mpm-shadow lock --check` (offline,
    exits 1 on plan and lock disagreement or a digest mismatch).
4.  `task-run prepare:pnpm` writes the local outputs,
    then a second `file-enforcer check` over all rules must be clean.
5.  Replace the two `git diff --exit-code` steps at `.github/workflows/advisory-core.yml:92-105` ("Rust catalog drift")
    and `:107-118` ("pnpr config drift") with the checks of this list and
    `task-run //package/config/pnpr:check:config` (`package/config/pnpr/mise.toml:30`,
    `package/config/pnpr/task.ts:25`).
    Their stated reason (the sync has just rewritten the files,
    so only a diff can see the drift) disappears,
    and the comment blocks at `:92-96` and `:107-109` and the messages at `:104` and `:117` name `mise run sync:files`.

### Dockerfile

Command:
`rg --line-number "sync|toolchain|file-enforcer|mise run" Dockerfile`.

- `Dockerfile:44-45`:
  the comment "The workspace dependencies the sync entry point imports."
  and `RUN pnpm install --frozen-lockfile`.
  The sync no longer imports workspace packages,
  but `mise run prepare` at `:51` still needs `node_modules`.
  Keep the install;
  reword the comment.
- `Dockerfile:47-49`:
  the comment and
  `RUN export $(cat .env.local | xargs) && node package/file-enforcer/core/src/toolchain/sync.ts`
  become a step that fetches the `mpm-shadow` archive,
  verifies its SHA-256 from the lock,
  and runs `mpm-shadow sync`.
  The `jq`,
  `curl` and `sha256sum` pattern already used for Node and pnpm at `Dockerfile:27-43` applies,
  but it needs a lock row for `mpm-shadow` (gap;
  see "Open decisions").
- `Dockerfile:51`,
  `RUN mise run prepare`:
  unchanged in text;
  its `prepare:pnpm:others:files` child now runs `file-enforcer write`,
  so `file-enforcer` must be on `PATH` first.

Other container files name only comments that stay true
(`package/vm-builder/cli/Containerfile:11` mentions `mise run toolchain:sync`),
except `package/config/pnpr/Containerfile:14`,
whose comment "Generated by file-enforcer;
baked in ..." should name pnpr's generator.

### IDE launcher

Command:
`rg --line-number "bin-paths|file-enforcer" package/ide`.

The launcher runs `node package/file-enforcer/core/src/toolchain/bin-paths.ts` in the language servers' sandbox.

- `package/ide/app/src/language/toolchain.rs:65`:
  `pub const ENTRY_POINT: &str = "package/file-enforcer/core/src/toolchain/bin-paths.ts";`
  is both the file that marks a toolchain worktree (`entry_point`,
  `:177-195`,
  walks ancestors for `is_file()`)
  and the script `node` runs (`resolve_confined`,
  `:229`,
  finds `node` with `command::find_node`).
  It becomes:
  the worktree marker is `toolchain.plan.jsonc` at the worktree root,
  and the program is `mpm-shadow` found by an explicit path,
  run as `mpm-shadow bin-paths` with the worktree as its directory.
  Candidate path:
  `<worktree>/.cache/toolchain-bin/mpm-shadow` (the exposure link of the plan's `mpm-shadow` bin).
  A fresh worktree has no such link before its first sync,
  which is the "toolchain not provisioned" notice's case.
  `bin-paths` was measured on the rehearsal build to be read-only and offline,
  and prints `exposureDir`,
  `pathDirs` and `bins` as JSON (`{"exposureDir": ".cache/toolchain-bin", "pathDirs": [], "bins": {}}`
  before any sync).
- `package/ide/app/src/language/toolchain.rs:4,15,16`:
  the module comment names the old command.
- `package/ide/app/src/language/toolchain/command.rs:5,53,119-165`:
  the sandbox launch assumes `node` and repository script code
  (`find_node`,
  `find_node_in`,
  the `NODE_OPTIONS` scrub).
  The sandbox must now bind the `mpm-shadow` binary read-only instead.
- `package/ide/app/src/language/toolchain/state.rs:169` and `toolchain.rs:256,305`:
  notice texts built from `ENTRY_POINT`.
- Tests and fixtures that name the old command or create the script file:
  `package/ide/app/src/language/toolchain_tests.rs:6,57,81,159,231,233,258,275`,
  `package/ide/app/src/language/notice_tests.rs:351,385`,
  `package/ide/app/src/language/toolchain/state_tests.rs:58,78,87,110,135`,
  `package/ide/app/src/language/toolchain/parse_tests.rs:115,119` (a captured Node stack trace as the "unreadable"
  sample),
  `package/ide/app/src/language/toolchain/command_tests.rs:19`,
  `package/ide/app/src/test_scratch.rs:44`.
- `package/ide/app/bin/inspect-language-toolchain.ts:41,84,86,288`:
  copies the TypeScript toolchain sources into a scratch worktree (`cpSync(toolchainSources, ...)`) and asserts the
  notice text;
  it must build a scratch worktree with a plan,
  a lock and the binary instead.
- `package/ide/app/README.md:1128,1145,1231`:
  the prose describing the lookup.

### Scanner and cli-git configs

Command:
`rg --line-number --hidden "forbidden-strings\.rules|compile-rules|rulesFile|FORBIDDEN_STRINGS_RULES" cli-git.config.jsonc mise.toml file-enforcer.config.ts`.

Already switched by `ad9fe75d05` (monochromatic-7a),
nothing to do in the switch commit for the configs themselves:

- `cli-git.config.jsonc:24`:
  `"rulesFile": ["forbidden-strings.append.txt", { "path": "forbidden-strings.append.local.txt", "optional": true }]`.
- `mise.toml:1444`:
  `FORBIDDEN_STRINGS_RULES = "{{config_root}}/forbidden-strings.append.txt,{{config_root}}/forbidden-strings.append.local.txt?"`.
- `.github/workflows/forbidden-strings.yml:115,146` build their own rules file under `$RUNNER_TEMP`;
  unrelated.

Stale references to the dropped generator remain in documents and fixtures,
none of them in a config:

- `forbidden-strings.append.txt:6`,
  the comment "file-enforcer to produce the runtime .cache/forbidden-strings.rules.txt".
- `package/forbidden-strings/cli/README.md:174,184,188,190,194`.
- `package/forbidden-strings/cli/mise.toml:115` (comment).
- `package/git-policy/cli/README.md:448,510`.
- `package/git-policy/cli.fuzz/seed/rules_file/repository-default:1` and
  `package/git-policy/cli.fuzz/src/configuration.rs:161`
  (the fuzz seed and sample value `.cache/forbidden-strings.rules.txt`;
  a still-valid path string,
  so a cleanup,
  not a break).
- `doc/runbook/forbidden-strings-out-of-band-rules.md:30,39,45,151`
  (`mise run file-enforcer` concatenates the appendixes;
  it no longer does).

### Consumers of the old package

Command:
`rg --line-number --hidden "file-enforcer-core|file-enforcer/core[^-]" --glob "!doc/**" --glob "!pnpm-lock.yaml" --glob "!package/file-enforcer/core/**" .`.

- The two the decision names:
  `package/test-fixture/file-enforcer-perf` (`package.json:9`;
  the sources at `src/perf.config.ts:21` and `src/perf.bench.test.ts:21`;
  the package retires,
  as `doc/decision/monorepo-manager-all-rust.md` decides) and
  `package/oxlint-plugin/prefer-readonly-parameter-type/src/workspace-source-effect.unit.test.ts:22`
  (reads `../../../file-enforcer/core/src/cargo/apply-plan.ts` as its subject file;
  it moves to a new fixture that really depends on `@monochromatic-dev/toml-edit-core`,
  per `doc/decision/monorepo-manager-all-rust.md:130-141`).
- Not in the decision's list:
  `package/vm-builder/cli` (precondition 7).
- Retiring the perf package leaves these edits:
  `pnpm-workspace.yaml:469` (a comment naming it as the only consumer),
  `mise.toml:1409` (the `node_modules/.bin` entry of the perf package,
  only if the committed `[env]` stays),
  and its entries in `pnpm-lock.yaml`.
- `mise.toml:1377` is the same for `package/file-enforcer/core/node_modules/.bin`.
- Consumers of `mise.no-env.toml` or its generated header:
  `package/monorepo-task/core-exec/src/build-and-test-fixture-config.ts:23` reads `../../../../mise.no-env.toml`,
  and `package/monorepo-task/core-exec/src/build-and-test.unit.test.ts:41` asserts the generated config starts with
  `# Generated from mise.no-env.toml by file-enforcer.\n${canonicalConfig}\n`.
  That test fails the moment the generation stops;
  it needs the new expectation decided in "Root mise.toml".
- Text only (no behavior):
  `package/test-harness/core-matrix/src/discover.ts:32` (a TSDoc example path),
  `slopo.ignore.txt:331` (a comment),
  `package/search-shadow/cli/README.md:57,59`,
  `package/config/pnpr/README.md:64-65`,
  `package/dependency-catalog/cli-rust-update/README.md:35`,
  `package/dependency-catalog/cli-tighten/README.md:154`.
- The container runner that the decision moves to `package/test-harness/core-matrix` is already there
  (`package/test-harness/core-matrix/src/container.ts:354`,
  `runContainer`);
  the old copy `package/file-enforcer/core/src/toolchain/container-runner.ts` has no consumer outside the deleted
  package.

### pnpr

Command:
`rg --line-number "PNPR_CONFIG_HEADER_LINE|Generated by file-enforcer" package/config/pnpr`.

- `package/config/pnpr/src/generated-config-settings.ts:22-23`,
  constant `PNPR_CONFIG_HEADER_LINE`:
  `'# Generated by file-enforcer (generatePnprConfig in file-enforcer.config.ts); edit that source.'`
  (the TSDoc at `:14-15` says the wording waits for the cutover).
  The generated file starts with the same text (`package/config/pnpr/config.yaml:1`).
  Replacement:
  a line naming pnpr's own generator,
  for example `# Generated by @monochromatic-dev/config-pnpr (mise run //package/config/pnpr:generate:config); edit its source.`
  Rewording changes `config.yaml`,
  which redeploys pnpr on Coolify and starts a publish run (handover,
  "Cutover reminders").
- Deleting `package/file-enforcer/core` also changes `package/config/pnpr/config.yaml` regardless of the header:
  `config.yaml:78` lists `'@monochromatic-dev/file-enforcer-core'`
  and `config.yaml:3` says `# Published package count: 169`.
  So one regeneration (`mise run //package/config/pnpr:generate:config`) carries both changes,
  and the redeploy happens once.
- pnpr's own tasks exist (`package/config/pnpr/mise.toml:26,30`;
  `task.ts:21,25`).
  `file-enforcer.config.jsonc` has no pnpr rule,
  by decision.

### Root mise.toml

The decision says `mise.no-env.toml` is renamed to the hand-written `mise.toml`
and the static environment moves to the committed `.config/mise/conf.d/20-env.toml`
(`doc/decision/file-enforcer-rust-rewrite.md`,
"Outputs that stop being generated";
planning:
"mise.toml:
committed conf.d files").
The coordinator's brief for this inventory says instead that the root `mise.toml` keeps its committed content.
These differ in what happens to the 123 lines of `[env]` (`mise.toml:1322-1444`) and to `mise.no-env.toml`.
Measured facts for the choice:

- `.config/mise/conf.d/` does not exist.
  Nothing generates `20-env.toml`;
  the `mise-path-list` rule (`file-enforcer.config.jsonc:202-208`)
  writes only the gitignored `10-bin.local.toml`,
  the existing `package/*/*/node_modules/.bin` directories.
  In the decision's layout the fixed `_.path` prefix
  (`.cache/search-shadow-bin`,
  `.cache/mpm-shadow-bin`,
  `.cache/toolchain-bin`,
  `.cache/cli-git-bin`,
  `node_modules/.bin`),
  the `_.file` sops line (`mise.toml:1329`),
  `SLINT_ENABLE_EXPERIMENTAL_FEATURES` (`:1427`),
  `SLINT_EMIT_DEBUG_INFO` (`:1436`)
  and `FORBIDDEN_STRINGS_RULES` (`:1444`) must be written by hand into `20-env.toml` in the switch commit.
  That is a hand edit of the switch commit,
  not a rule's output.
- In the alternative (keep `mise.toml` as committed),
  `mise.no-env.toml` is deleted,
  `mise.toml` loses its three header lines (`:1,4,5`) and the comment at `:1331-1347`
  that says the list is "committed as file-enforcer writes it",
  and the `_.path` list (`:1348-1417`,
  64 `node_modules/.bin` entries,
  one of them the root's) stays frozen by hand;
  `10-bin.local.toml` would then duplicate it.
  The two stale entries at `:1377` and `:1409` must be deleted by hand.
  `FORBIDDEN_STRINGS_RULES` stays at `:1444` byte for byte.
- Either way these follow:
  `mise.no-env.toml:324` / `mise.toml:325` (root test),
  the two files under `package/monorepo-task/core-exec`,
  `.idea/jsonSchemas.xml`,
  and the five per-package `mise.toml` comments and README links that name `mise.no-env.toml`
  (`package/xml-edit/core-rust.fuzz/mise.toml:2`,
  `package/jsonc-edit/core-rust.fuzz/mise.toml:2`,
  `package/forbidden-strings/core.fuzz/mise.toml:2`,
  `package/forbidden-strings/cli.fuzz/mise.toml:2`,
  `package/forbidden-strings/cli.fuzz/README.md:30,37`,
  `package/linter/monochromatic-lint-closure/README.md:186`,
  `package/linter/kotlin/mise.toml:3`,
  `package/linter/kotlin/README.md:54`,
  `package/music-player/android-app/mise.toml:8,51`).
- The package `mise.toml` files had their task headers normalized by the TypeScript tool
  (`file-enforcer.config.ts:671-682`,
  `MISE_MANIFEST_GLOBS`:
  `package/*/*/mise.toml` and three sibling globs).
  After the switch nobody normalizes them;
  the existing files keep their current headers.

`.idea/jsonSchemas.xml` (command:
`rg --line-number "mise" .idea/jsonSchemas.xml`):
the decision cites `:26,32`;
the measured lines are `:50` (`value='mise.toml'`) and `:56` (`value='mise.no-env.toml'`),
inside the `mise` schema entry that starts at `:30`.
The `mise.no-env.toml` item (`:54-57`) goes in either layout.
`conf.d/20-env.toml` and `10-bin.local.toml` need items if the owner wants the schema applied there.
The file-enforcer schema mapping is already in place (`.idea/jsonSchemas.xml:6-22`).

### Documents

- `doc/runbook/publish-crate-first-time.md:463`:
  `mpm --config mpm.managers.toml --cargo-binstall-monochromatic-mpm-shadow install monochromatic-mpm-shadow`
  runs against a deleted file.
  Replace with the version-keyed `mpm-shadow sync` flow,
  or with a download of the release archive plus `sha256sum --check`,
  whichever the version-keyed branch lands.
  The text at `:77,79,460` (`mise run toolchain:sync`,
  `toolchain:status`) stays valid.
- `doc/decision/mpm-toolchain-adoption.md`:
  the decision cites `:348`;
  the sentence measured there is the "Tradeoff" about a cached version pinned by `Cargo.lock`.
  The command in question is at `:361` (`mpm --config mpm.managers.toml --cargo-binstall-monochromatic-mpm-shadow install ...`),
  introduced by `:360`;
  `:343` also mentions `mpm.managers.toml` accepting `cleanup_cache`.
- `doc/troubleshooting/cargo-binstall-compiles-when-github-api-rate-limited.md:5` and
  `doc/troubleshooting/mpm-toolchain-migration.md:87,229-230` also name the files (historical records;
  add a pointer,
  do not rewrite).
- `doc/runbook/rust-dependencies.md:16,54,65,78,101` (`mise run sync:files` stays valid;
  `:101` names `node package/file-enforcer/core/src/cli.ts --check`),
  `doc/runbook/release-the-music-player.md:61,63` (`buildCargoManifestPlan in file-enforcer.config.ts`,
  `node file-enforcer.config.ts`),
  `doc/runbook/deploy-pnpr-registry.md:257` ("`config.yaml` changes only through `mise run sync:files`" becomes the
  pnpr task).

### Amendments the decision lists under "Consequences"

Both records already carry a status banner for the amendment
(`doc/decision/monorepo-manager-all-rust.md:20-23`,
`doc/decision/mpm-toolchain-adoption.md:8-19`).
The body edits are not made.
Command:
`rg --line-number "^#{2,3} |HCL|mise.toml|file-enforcer" doc/decision/monorepo-manager-all-rust.md doc/decision/mpm-toolchain-adoption.md`.

For `doc/decision/monorepo-manager-all-rust.md`:

- `:98-103`,
  the bullet "Configuration syntax is OpenTofu-shaped HCL":
  state that meow's own configuration stays HCL
  while its file-enforcement configuration is JSONC (`file-enforcer.config.jsonc`),
  read by the linked library.
- `:55-58` already says meow links `monochromatic-file-enforcer`;
  no change.
- `:75-84` already records that nothing generates `mise.toml`;
  once the root `mise.toml` decision is made,
  align the sentence with the layout chosen.
- `:129-133` and `:139-141`:
  the sentences about migrating `sync:files` tasks,
  the perf fixture and the
  `prefer-readonly-parameter-type` read become "done at the Rust file-enforcer cutover" with the commit hash.

For `doc/decision/mpm-toolchain-adoption.md`
(the decision cites `:92-112` and `:201-223`;
the measured ranges are `:101-125` and `:213-239`):

- `:101-107`,
  "Meta Package Manager owns",
  first bullet:
  the restore manifest `mpm.packages.toml` generated from the typed plan
  is deleted;
  remove the bullet or mark it superseded.
- `:213-239`,
  "Tool plan and generated artifacts":
  the policy and lock belong to mpm-shadow
  (`toolchain.plan.jsonc`,
  `toolchain.lock.json`),
  not to `package/file-enforcer/core/src/toolchain/`;
  `mpm.packages.toml` and `mpm.managers.toml` are deleted.
- `:240-265`,
  "Migration" step 4 ("regenerate `mise.toml` with file-enforcer"):
  historical;
  add the pointer.
- `:325,343,360-361` (the `mpm.managers.toml` mentions and the extension install command):
  update with the runbook edit.
- `:266-293` ("Machine-global scope") describes the global files;
  add the pointer for the plan conversion.

### What `file-enforcer.config.ts` generates today, and its owner

Command:
`rg --line-number "overwrite\(|overwriteIfNotExists\(|manageMiseManifests|assertForbiddenRootContextAbsent|manageHarperLsp4ij|mirrorSkills\(" file-enforcer.config.ts`.
The entry point is `file-enforcer.config.ts:2080-2169`.

- `CLAUDE.md` (`:2098`):
  rule `claude-md` (`file-enforcer.config.jsonc:9-18`);
  the preamble is `.config/file-enforcer/claude-preamble.md`.
- `LICENSE` (`:2092`):
  rule `license` (`:20-25`).
- `.claude/skills` and `.factory/skills` (`mirrorSkills`,
  `:1298`):
  rules `skills-claude` and `skills-factory` (`:27-28`);
  the per-target `.agents-mirror-manifest.json` stops being written.
- Per-package `LICENSES/*.txt` (`generatePackageLicenseTexts`,
  `:552`):
  rule `package-licenses` (`:174-189`).
- Per-crate `Cargo.toml` keys,
  `rust-catalog.toml` and the `[workspace.dependencies]` tables (`generateCargoManifests`,
  `:2039`):
  rules `cargo-base`,
  `cargo-dylint`,
  `cargo-profile-scanner`,
  `cargo-profile-overflow`,
  `cargo-profile-linter`,
  `cargo-profile-bench`,
  `cargo-profile-music`,
  `cargo-profile-fuzz` (`:32-162`) and `rust-catalog` (`:165-170`).
- `.browserslistrc.resolved.local.json` (`generateResolvedBrowserslistTargets`,
  `:939`):
  rule `browserslist-targets` (`:192-199`);
  data lock `browserslist-data.lock.json` at the root.
- JetBrains Harper LSP4IJ policy under `~/.config/JetBrains` (`manageHarperLsp4ij`,
  `:1399`):
  `file-enforcer user write` with a `jetbrains-harper` rule in the per-user config (precondition 8).
- `forbidden-strings.append.local.txt` mode (`:891`):
  rule `forbidden-strings-local-mode` (`:30`),
  mode only.
- `.config/mise/conf.d/10-bin.local.toml`:
  new,
  rule `mise-bin-list` (`:202-208`);
  it replaces the `_.path` bin directories
  that `generateMiseToml` (`:694-785`) put into `mise.toml`.
- `package/config/pnpr/config.yaml` (`generatePnprConfigFile`,
  `:2069`):
  new owner pnpr (`generate:config`,
  `check:config`).
- `mise.toml` (`generateMiseToml`,
  `:694`;
  the write is at `:778-785`):
  stops being generated;
  it keeps its committed content (see "Root mise.toml").
- `toolchain.lock.json`:
  new owner mpm-shadow (`update`,
  `lock`).

Stops being generated with no replacement (by decision,
"Outputs that stop being generated"):

- `.cache/forbidden-strings.rules.txt` (`:899-918`),
  the seed of `forbidden-strings.append.local.txt` (`:872-890`)
  and the retired-output removal of `./forbidden-strings.local.txt` (`:923`).
  Consumers read the appendixes directly (section "Scanner and cli-git configs").
- The eager `compile-rules` step (`compileForbiddenStringsRuntimeCache`,
  `:822-841`,
  called at `:920`) and the scanner existence probe (`forbiddenStringsScannerExists`,
  `:796`).
  The scanner compiles on use and repairs its own cache.
- `mpm.packages.toml` and `mpm.managers.toml`,
  in both scopes.
- `.cache/toolchain.env.json` (`package/file-enforcer/core/src/toolchain/exposure.env.ts`):
  dropped as unused (planning,
  "no reader";
  `rg` finds no reader outside the deleted package).
  The `env` blocks stay in the plan and lock.
- The mise task-header quoting (`manageMiseManifests`,
  `:2083-2088`),
  the `CONTEXT.md` guard (`assertForbiddenRootContextAbsent`,
  `:2080`;
  cli-git enforces `mono/forbidden-root-context`,
  `cli-git.config.jsonc:13`),
  the staleness cache,
  the OS package installer and the `exec` export.

Outputs with no owner:
none were found.
Two points are borderline and are listed in "Gaps and unowned outputs":
the Harper policy has no committed per-user config (precondition 8),
and nothing yet installs the `file-enforcer` binary (open decision).

## Open decisions

1.  Root `mise.toml` shape:
    the decision's rename plus `20-env.toml`,
    or keep `mise.toml` as committed.
    Ranking:
    the decision's layout > keep as committed.
    The decision's layout is what the owner accepted and it removes the 63 hand-frozen package `node_modules/.bin`
    entries from a committed file
    that the `[env]` comment itself calls install-state dependent (`mise.toml:1331-1347`);
    keeping the committed file is a smaller diff and the only one that preserves `FORBIDDEN_STRINGS_RULES` where
    `ad9fe75d05` put it,
    but it leaves the same list in two places once `10-bin.local.toml` exists.
2.  How CI and containers obtain `file-enforcer` and `mpm-shadow`.
    Both cargo-binstall entries have `artifacts: {}` in the lock (`jq` shows an empty object for
    `cargo:monochromatic-mpm-shadow`),
    so the `jq`,
    `curl`,
    `sha256sum` pattern of `task-cli-setup` and the `Dockerfile` has no row to read.
    A probe in the rehearsal shows a pinned release tag works as an `archive` entry:
    `{"mpm-shadow": {"request": "github:Aquaticats/Monochromatic@monochromatic-mpm-shadow-v0.1.1", "route": "archive", "bins": ["mpm-shadow"]}}`
    resolved to version 0.1.1 with `linux-x64`,
    `linux-x64-musl`,
    `linux-arm64` and `linux-arm64-musl` rows
    (`monochromatic-mpm-shadow-0.1.1-x86_64-unknown-linux-gnu.tar.gz`,
    SHA-256 from the release).
    But exposure names must be unique across a plan,
    so such an entry replaces `cargo:monochromatic-mpm-shadow`
    (and carries its `mpm` link postinstall) rather than joining it,
    and every release then needs a plan edit and an `update`,
    as `bin` does.
    The same shape needs a release job for `monochromatic-file-enforcer-cli` (precondition 3).
    This couples to the version-keyed install work;
    decide together.
3.  The plan entries that install the binaries:
    `file-enforcer` has no entry at all today;
    `cargo:monochromatic-mpm-shadow` exists but is the pre-switch release.
    An entry for the cli crate is needed in the switch commit's plan,
    and it cannot resolve until the crate is on crates.io
    (the rehearsal `update` queries crates.io).
4.  The IDE launcher's explicit path,
    proposed in the IDE launcher section as `<worktree>/.cache/toolchain-bin/mpm-shadow`.
5.  The machine-global plan's `asset` hints (see "Global plan conversion"):
    the converter cannot know them without reading the file.

## Toolchain plan conversion

### Method

The converter is `convert.ts` in the appendix (a Node script run by `node convert.ts`,
TypeScript stripped by Node).
It reads `ALL_TOOLS` from `plan.tools.data.ts` and writes `toolchain.plan.jsonc`,
following
`package/mpm-shadow/cli/README.md`,
"What the cutover converter must write":

- the `request` string is `<origin>@<range>`:
  `cargo:<crate>@<latest or version>`,
  `npm:<name>@...`,
  `pipx:<name>@...`,
  `conda:<name>@...`,
  `github:<owner>/<repo>@...` (owner and repo read from the lock's own release URLs,
  then checked against the README table),
  `node@`,
  `zig@`,
  `java:temurin@21`,
  `android-sdk@latest`,
  `rust@nightly`;
- `manager` and `packageId` are dropped where the request names the package,
  and kept as `{version}` for `rust`;
- `{version}` is written for `rust` (`packageId`,
  `RUSTUP_TOOLCHAIN`),
  `java` (`JAVA_HOME`) and `android-sdk`
  (`ANDROID_HOME`,
  `ANDROID_SDK_ROOT`);
- `options.asset` is written for `harper-cli` (`harper-cli-*`) and `tombi` (`tombi-cli-*`);
  `assetPattern` is renamed `asset`;
  `cargoGit` becomes `git:<url>@<revision>`;
  every other option and `postinstall` carries over;
- `//` comments that sit directly before a tool object in the data file are kept before its entry.

Checks run on the output:

- 63 tools converted;
  the 33 README rows that list a request all agree with the converter (`readme-check.ts`:
  0 mismatches).
- `mpm-shadow lock --check` accepts the plan and lock (see "Rehearsal").

### Every tool, old entry and new request

One line per tool:
old request,
route,
manager and package id,
then the new request,
route and any added or changed fields.
The `{version}` hazard (a literal would leave `packageId` stale after the first `update`) is why `rust`,
`java` and `android-sdk` are marked.

```text
// repository plan, from package/file-enforcer/core/src/toolchain/plan.tools.data.ts (ALL_TOOLS, 63 entries)
bin: {pinned:0.29.4} archive => "github:marcosnils/bin@0.29.4" route archive; options bootstrap
mpm: {pinned:8.2.0} mpm-uvx uvx:meta-package-manager => "pipx:meta-package-manager@8.2.0" route mpm-uvx; options bootstrap
bun: {pinned:1.3.14} bin-sourced => "github:oven-sh/bun@bun-v1.3.14" route bin-sourced; options binAliases
pnpm: {track:latest} bin-sourced => "github:pnpm/pnpm@latest" route bin-sourced; options binAliases
cargo:fd-find: {track:latest} cargo-binstall cargo:fd-find => "cargo:fd-find@latest" route cargo-binstall
ripgrep: {track:latest} bin-sourced => "github:BurntSushi/ripgrep@latest" route bin-sourced
cargo:fastmod: {track:latest} cargo-binstall cargo:fastmod => "cargo:fastmod@latest" route cargo-binstall
cargo:cargo-fuzz: {pinned:0.13.2} cargo-binstall cargo:cargo-fuzz => "cargo:cargo-fuzz@0.13.2" route cargo-binstall
node: {track:latest} archive => "node@latest" route archive
harper-cli: {track:latest} bin-sourced => "github:Automattic/harper@latest" route bin-sourced; options asset
dprint: {track:latest} bin-sourced => "github:dprint/dprint@latest" route bin-sourced
uv: {track:latest} archive => "github:astral-sh/uv@latest" route archive
hyperfine: {track:latest} bin-sourced => "github:sharkdp/hyperfine@latest" route bin-sourced
caddy: {track:latest} bin-sourced => "github:caddyserver/caddy@latest" route bin-sourced
watchexec: {track:latest} bin-sourced => "github:watchexec/watchexec@latest" route bin-sourced
sops: {track:latest} bin-sourced => "github:getsops/sops@latest" route bin-sourced
age: {track:latest} archive => "github:FiloSottile/age@latest" route archive
rust: {channel:nightly} rustup-toolchain rustup:nightly-2026-10-31 => "rust@nightly" route rustup-toolchain packageId {version}; options rustComponents+rustTargets; env RUSTUP_TOOLCHAIN
cargo-binstall: {track:latest} bin-sourced => "github:cargo-bins/cargo-binstall@latest" route bin-sourced
cargo:timeout-cli: {track:latest} cargo-binstall cargo:timeout-cli => "cargo:timeout-cli@latest" route cargo-binstall
cargo:cargo-dylint: {track:latest} cargo-binstall cargo:cargo-dylint => "cargo:cargo-dylint@latest" route cargo-binstall
cargo:dylint-link: {track:latest} cargo-binstall cargo:dylint-link => "cargo:dylint-link@latest" route cargo-binstall
cargo:cargo-nextest: {track:latest} cargo-binstall cargo:cargo-nextest => "cargo:cargo-nextest@latest" route cargo-binstall
cargo:cargo-about: {track:latest} cargo-binstall cargo:cargo-about => "cargo:cargo-about@latest" route cargo-binstall
cargo:monochromatic-lint: {track:latest} cargo-binstall cargo:monochromatic-lint => "cargo:monochromatic-lint@latest" route cargo-binstall
cargo:monochromatic-git-policy-cli: {track:latest} cargo-binstall cargo:monochromatic-git-policy-cli => "cargo:monochromatic-git-policy-cli@latest" route cargo-binstall; options postinstall; os linux
cargo:monochromatic-search-shadow: {track:latest} cargo-binstall cargo:monochromatic-search-shadow => "cargo:monochromatic-search-shadow@latest" route cargo-binstall; options postinstall; os linux
cargo:monochromatic-mpm-shadow: {track:latest} cargo-binstall cargo:monochromatic-mpm-shadow => "cargo:monochromatic-mpm-shadow@latest" route cargo-binstall; options postinstall; os linux
cargo:monochromatic-nested-wayland-session: {track:latest} cargo-binstall cargo:monochromatic-nested-wayland-session => "cargo:monochromatic-nested-wayland-session@latest" route cargo-binstall
cmake: {track:latest} archive => "github:Kitware/CMake@latest" route archive
cargo:apple-codesign: {track:latest} cargo-binstall cargo:apple-codesign => "cargo:apple-codesign@latest" route cargo-binstall
zig: {pinned:0.15.2} archive => "zig@0.15.2" route archive
cargo:slint-lsp: {track:latest} cargo-binstall cargo:slint-lsp => "cargo:slint-lsp@latest" route cargo-binstall
cargo:slint-viewer: {track:latest} cargo-binstall cargo:slint-viewer => "cargo:slint-viewer@latest" route cargo-binstall
conda:bubblewrap: {track:latest} archive conda:bubblewrap => "conda:bubblewrap@latest" route archive; os linux
taplo: {track:latest} archive => "github:tamasfe/taplo@latest" route archive; options binAliases
tombi: {track:latest} bin-sourced => "github:tombi-toml/tombi@latest" route bin-sourced; options asset
marksman: {track:latest} bin-sourced => "github:artempyanykh/marksman@latest" route bin-sourced
rumdl: {track:latest} bin-sourced => "github:rvben/rumdl@latest" route bin-sourced
github:Feel-ix-343/markdown-oxide: {track:latest} bin-sourced => "github:Feel-ix-343/markdown-oxide@latest" route bin-sourced
zizmor: {track:latest} bin-sourced => "github:zizmorcore/zizmor@latest" route bin-sourced
npm:yaml-language-server: {track:latest} mpm-pnpm pnpm:yaml-language-server => "npm:yaml-language-server@latest" route mpm-pnpm
npm:@actions/languageserver: {track:latest} mpm-pnpm pnpm:@actions/languageserver => "npm:@actions/languageserver@latest" route mpm-pnpm
npm:@ansible/ansible-language-server: {track:latest} mpm-pnpm pnpm:@ansible/ansible-language-server => "npm:@ansible/ansible-language-server@latest" route mpm-pnpm
npm:vscode-langservers-extracted: {track:latest} mpm-pnpm pnpm:vscode-langservers-extracted => "npm:vscode-langservers-extracted@latest" route mpm-pnpm
superhtml: {track:latest} bin-sourced => "github:kristoff-it/superhtml@latest" route bin-sourced
npm:bash-language-server: {track:latest} mpm-pnpm pnpm:bash-language-server => "npm:bash-language-server@latest" route mpm-pnpm
npm:dockerfile-language-server-nodejs: {track:latest} mpm-pnpm pnpm:dockerfile-language-server-nodejs => "npm:dockerfile-language-server-nodejs@latest" route mpm-pnpm
npm:@microsoft/compose-language-service: {track:latest} mpm-pnpm pnpm:@microsoft/compose-language-service => "npm:@microsoft/compose-language-service@latest" route mpm-pnpm
github:docker/docker-language-server: {track:latest} bin-sourced => "github:docker/docker-language-server@latest" route bin-sourced
llama.cpp: {pinned:b9829} archive => "github:ggml-org/llama.cpp@b9829" route archive
opentofu: {track:latest} bin-sourced => "github:opentofu/opentofu@latest" route bin-sourced
hcloud: {track:latest} bin-sourced => "github:hetznercloud/cli@latest" route bin-sourced
npm:typescript-language-server: {track:latest} mpm-pnpm pnpm:typescript-language-server => "npm:typescript-language-server@latest" route mpm-pnpm
cargo:coreutils: {track:latest} cargo-binstall cargo:coreutils => "cargo:coreutils@latest" route cargo-binstall; options cargoFeatures
npm:socket: {track:latest} mpm-pnpm pnpm:socket => "npm:socket@latest" route mpm-pnpm
npm:wrangler: {track:latest} mpm-pnpm pnpm:wrangler => "npm:wrangler@latest" route mpm-pnpm
npm:pagefind: {track:latest} mpm-pnpm pnpm:pagefind => "npm:pagefind@latest" route mpm-pnpm
betterleaks: {track:latest} bin-sourced => "github:betterleaks/betterleaks@latest" route bin-sourced
java: {channel:temurin-21} archive => "java:temurin@21" route archive; env JAVA_HOME
android-sdk: {track:latest} android-sdk => "android-sdk@latest" route android-sdk; options androidPackages; env ANDROID_HOME+ANDROID_SDK_ROOT
cargo:cargo-ndk: {track:latest} cargo-binstall cargo:cargo-ndk => "cargo:cargo-ndk@latest" route cargo-binstall
pipx:slopo: {track:latest} mpm-uvx uvx:slopo => "pipx:slopo@latest" route mpm-uvx
```

### Rehearsal

Procedure (all under `${HOME}/temp/agent/fe-rust/cutover-rehearsal/`,
nothing outside it):

1.  `mise run //package/mpm-shadow/cli:build` (release build,
    41 s,
    run as the transient unit `fe-cutover-build-mpm`,
    since collected).
2.  `node convert.ts <plan.tools.data.ts> repo/toolchain.plan.jsonc`.
3.  `node run.ts <mpm-shadow> repo home update`,
    where `run.ts` starts the binary with a disposable `HOME`,
    `XDG_*`,
    `CARGO_HOME`
    and `RUSTUP_HOME` under the rehearsal directory,
    `PATH=/usr/bin:/bin`,
    `MONOCHROMATIC_WORKTREE_ROOT` set,
    and a GitHub token passed in the child's environment only.
    It refuses a `HOME` outside the rehearsal directory.
    `update` ran as the transient unit `fe-cutover-update` (since collected),
    and wrote the lock.
    Per the README it downloads the Android command-line tools and the AWS CLI once each to hash them;
    that was not separately observed.
4.  `lock --check`,
    `update --check`,
    `lock` and `bin-paths` in the same scope.
5.  `node compare.ts today.lock.json repo/toolchain.lock.json`.

Results:

- `update`:
  wrote `toolchain.lock.json`,
  63 tools;
  every resolved version equals today's (no version moved).
- `lock --check`:
  `toolchain lock is current: 63 tools, digest verified`.
- `update --check`:
  `0 tools would move; nothing was written or installed`.
- `lock`:
  `toolchain lock already current`.
- The frozen `jq` surface is unchanged:
  the selected values `node`,
  `pnpm` and `java` versions and `linux-x64` rows,
  `rust` version,
  `cargo-binstall` and `cargo:cargo-about` versions,
  `android-sdk` `options.androidPackages`,
  and the `rust`,
  `java` and `android-sdk` env records hash identically in both locks.
- Lock shape changes:
  `generatedFrom` is `toolchain.plan.jsonc` (was `package/file-enforcer/core/src/toolchain/plan.ts`);
  a top-level `digest` (`sha256:5e0c2934...`) is added;
  `request` is a string (was an object such as `{"track":"latest"}`);
  `manager` and `packageId` are gone for the 32 tools whose request names the package (they were redundant).
- Artifact rows,
  field by field against today's lock (6 changed,
  2 added,
  6 dropped):
  - changed to musl builds:
    `ripgrep` `linux-arm64-musl`,
    `harper-cli` `linux-arm64-musl` and `linux-x64-musl`,
    `hyperfine` `linux-arm64-musl`,
    `marksman` `linux-arm64-musl` and `linux-x64-musl` (the file name gains `musl`;
    the SHA-256 differs);
  - added:
    `bin` `linux-arm64-musl` and `linux-x64-musl`;
  - dropped:
    `bin` `macos-arm64`,
    `macos-x64`,
    `windows-arm64`,
    `windows-x64`
    and `bun` `linux-x64-baseline` and `linux-x64-musl-baseline` (the Linux-only stopgap,
    `doc/decision/mise-lock-linux-only-stopgap.md`);
  - `options` gains `asset` for `harper-cli` and `tombi` (as converted).
- The handover's recorded rehearsal also listed `superhtml` 0.7.0 to 0.7.1 and `aws` 2.39.2 to 2.39.3 in the global
  lock;
  today's repository lock is already at `superhtml` 0.7.1,
  so no repository version moves on 2026-10-31.
- Not rehearsed:
  `sync` (no installs ran,
  by instruction),
  and any real installation.

The review surface for the owner is `node compare.ts` over the real committed lock and the rebuilt one;
rerun it at the switch revision,
because upstream releases move `latest` entries and the version-keyed install branch may change the lock.

### Global plan conversion

Source of the global entries:
the real file `~/.config/monochromatic-dev/toolchain/plan.tools.data.ts` was **not read**.
A read of that directory was refused by the sandbox,
and the task also forbids touching it.
This section therefore rests on `doc/planning/mpm-global-toolchain-migration.md:80-260` and the README line that the
global plan
"also holds `go: go@latest`,
`aws: aws@latest`,
`rustup: rustup@latest` and `go:github.com/charmbracelet/crush@latest` on the `go-install` route".
The planning record puts the global plan at 59 entries (`doc/planning/file-enforcer-rust-rewrite.md`,
"Global toolchain plan:
converted directly").

To exercise the converter's branches for the forms that only the global plan holds,
a synthetic fixture (`global-shapes.data.ts`,
labeled synthetic in its first comment) holds 15 entries of the documented shapes.
Converted and resolved with `update` in a disposable home,
the results are:

```text
// synthetic global-shape fixture, 15 entries: old request, route => new request
go: {track:latest} archive => "go@latest" route archive
aws: {track:latest} archive => "aws@latest" route archive
rustup: {track:latest} archive => "rustup@latest" route archive
rust: {channel:stable} rustup-toolchain rustup:1.98.1 => "rust@stable" route rustup-toolchain packageId {version}; env RUSTUP_TOOLCHAIN
go:github.com/charmbracelet/crush: {track:latest} go-install go:github.com/charmbracelet/crush => "go:github.com/charmbracelet/crush@latest" route go-install
cargo:https://github.com/slint-ui/slint: {pinned:ref:85e3eb76819762cdcaa732fa87533ff896546bac} cargo-git cargo:slint-lsp => "git:https://github.com/slint-ui/slint@85e3eb76819762cdcaa732fa87533ff896546bac" route cargo-git packageId slint-lsp
npm:typeslayer: {pinned:0.1.32-aquaticat.0319832} mpm-pnpm pnpm:typeslayer => "npm:typeslayer@0.1.32-aquaticat.0319832" route mpm-pnpm
npm:@deepseek-ai/dsh: {channel:next} mpm-pnpm pnpm:@deepseek-ai/dsh => "npm:@deepseek-ai/dsh@next" route mpm-pnpm
npm:@opencode/cli: {track:latest} mpm-pnpm pnpm:@opencode/cli => "npm:@opencode/cli@latest" route mpm-pnpm; options allowBuilds
github:lemonade-sdk/llamacpp-rocm: {track:latest} archive => "github:lemonade-sdk/llamacpp-rocm@latest" route archive; options asset
github:openinterpreter/openinterpreter: {track:latest} bin-sourced => "github:openinterpreter/openinterpreter@latest" route bin-sourced; options asset
cargo:coreutils: {track:latest} cargo-binstall cargo:coreutils => "cargo:coreutils@latest" route cargo-binstall; options cargoFeatures
cargo:tokei: {track:latest} cargo-binstall cargo:tokei => "cargo:tokei@latest" route cargo-binstall
pipx:huggingface_hub: {track:latest} mpm-uvx uvx:huggingface_hub => "pipx:huggingface_hub@latest" route mpm-uvx
cargo:monochromatic-git-policy-cli: {track:latest} cargo-binstall cargo:monochromatic-git-policy-cli => "cargo:monochromatic-git-policy-cli@latest" route cargo-binstall; options postinstall; os linux
```

- Decoded and resolved:
  `go` 1.27.3,
  `aws` 2.39.3,
  `rustup` 1.29.2,
  `rust@stable` 1.99.1,
  `go:` (crush 0.99.0),
  the `git:` request at a pinned revision,
  `npm:typeslayer@0.1.32-aquaticat.0319832`,
  `npm:@deepseek-ai/dsh@next` (0.2.1-rc.5),
  `npm:@opencode/cli` with `allowBuilds`,
  `cargo:tokei`,
  `pipx:huggingface_hub`,
  `cargo:coreutils` with `cargoFeatures`,
  and the absolute `linkDir` postinstall.
- Two entries failed,
  and the failures are the finding:
  `github:lemonade-sdk/llamacpp-rocm` with `asset` `*ubuntu*x64*` ("has no Linux asset"),
  and `github:openinterpreter/openinterpreter` with `asset` `*linux*` ("has 8 equally good assets for linux-arm64").
  The `asset` globs in the fixture are invented stand-ins for Mise's `asset_pattern` and `matching` values.
  The real values must be converted from the real file,
  and each tool must resolve to exactly one asset per platform,
  because the resolver's `asset` glob matches release asset names,
  and the old `asset_pattern` and `matching` values were Mise's ("resolver-side asset selection" in the migration plan),
  not verified to select the same files.
- The converter handles `cargo-git`,
  `go-install`,
  `assetPattern`,
  `allowBuilds`,
  `npm:` pinned and `next` channel,
  `rust@stable`,
  `rustup@latest`,
  `go@latest` and `aws@latest`.
  It has not run on the real file;
  run it first as a dry run into a scratch path.

### Global plan backup and conversion procedure

For the owner to execute on their own machine,
after the switch commit and after the repository `mpm-shadow sync` is verified.
No step is automated and none was run.

1.  Confirm nothing is mid-sync:
    no `mpm-shadow` or old `sync.ts` process,
    and `mpm-shadow status --global` is not being run by a worktree.
2.  Back up beside the originals with owner-only permissions.
    In the directory `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`:
    `umask 077`,
    then `cp --archive --no-clobber` each of `plan.tools.data.ts`,
    `plan.artifacts.data.ts`,
    `toolchain.lock.json`,
    `mpm.packages.toml` and `mpm.managers.toml` to the same name plus `.pre-cutover`.
    Then `chmod 0600` on the five copies,
    `stat --format '%a %n'` to show the modes,
    and `sha256sum` of each original and copy to show they match.
    The original `plan.tools.data.ts` is already mode 0600 and the copy must stay 0600.
3.  Dry-run the converter into a scratch path,
    never into that directory:
    `node convert.ts <global plan.tools.data.ts> ${HOME}/temp/agent/<scratch>/toolchain.plan.jsonc`,
    open the output and check every `asset` value by hand against the release assets of its repository.
4.  Write `toolchain.plan.jsonc` into the global directory with mode 0600 (`umask 077` first),
    keeping the comments.
5.  Rebuild the lock:
    `mpm-shadow update --global`.
    Compare with the backed-up lock using `node compare.ts toolchain.lock.json.pre-cutover toolchain.lock.json`
    and read every version move and musl change,
    as for the repository.
6.  `mpm-shadow lock --check --global`,
    then `mpm-shadow sync --global`,
    then `mpm-shadow status --global`.
7.  Verify the converted global sync:
    `status --global` exits 0;
    run each installed global tool's `--version` (for example `gh`,
    `rg`,
    `go`,
    `aws`);
    check the `git` link in `$HOME/.local/bin` points at the wrapper 0.4.4 or newer.
    On a failure at any step,
    stop and leave the backups.
8.  Only after step 7 passes,
    delete the old `plan.tools.data.ts`,
    `plan.artifacts.data.ts`,
    `mpm.packages.toml` and `mpm.managers.toml`
    from the directory.
    Keep the `.pre-cutover` copies until the owner chooses to remove them.
    Rollback is:
    copy the `.pre-cutover` files back over the originals,
    and check out the commit before the switch for the old `sync.ts --global`.

## Comparison with the TypeScript output

Command,
from the worktree,
with the built binary:
`FILE_ENFORCER_IDLE_SECONDS=3 file-enforcer check --expected-dir <scratch>`.
It writes nothing in the worktree and left no server running.

- Exit 1,
  26 `drift-missing` paths in four rules,
  all with gitignored outputs absent from a fresh worktree:
  `skills-claude` 12,
  `skills-factory` 12,
  `mise-bin-list` 1,
  `browserslist-targets` 1.
- Zero drift in every committed output:
  `CLAUDE.md`,
  `LICENSE`,
  `LICENSES/` under every package,
  `Cargo.toml` key sets,
  `rust-catalog.toml` and the `[workspace.dependencies]` tables.
  The TypeScript tool wrote these files,
  so the Rust tool agrees with it on all committed outputs of this revision.
- Not yet compared:
  the content of the four gitignored outputs against a TypeScript run in a second copy,
  the `jetbrains-harper` rule (compared once in a disposable home by the wiring agent,
  with byte-identical results per its report),
  and the pnpr config (outside the tool).
  The final comparison is a precondition at the switch revision.

## Gaps and unowned outputs

- No lock row and no plan entry gives CI,
  the `Dockerfile` or a fresh clone the `mpm-shadow` or `file-enforcer` binary.
- No release job,
  dispatch option or path trigger exists for the five new crates (`.github/workflows/cargo-publish.yml:10-33`).
- `monochromatic-jsonc-edit` and `monochromatic-mpm-shadow` need version bumps before the new code can be published.
- `package/vm-builder/cli` still depends on the old package and is not in the decision's switch list.
- No per-user config for `file-enforcer user write` is committed;
  the Harper policy is written only after the owner creates it.
- `file-enforcer check` has no per-rule scope,
  so the committed-output gate in CI needs a `jq` filter on four rule ids.
- The `.config/mise/conf.d/20-env.toml` content has no generator and must be written by hand if the decision's layout
  is chosen.
- The machine-global plan was not read;
  its `asset` values and any form this document did not anticipate are unrehearsed.
- No output of the TypeScript tool is without an owner or an explicit "stops being generated" record.

## Appendix: the converter

Scratch copies of the other scripts (`run.ts`,
`compare.ts`,
`map.ts`,
`readme-check.ts`,
`global-shapes.data.ts`)
are in `${HOME}/temp/agent/fe-rust/cutover-rehearsal/`;
the converter is reproduced here because the data file it reads is deleted by the switch commit.

```ts
// One-off rehearsal converter: plan.tools.data.ts (ALL_TOOLS) to toolchain.plan.jsonc.
// usage: node convert.ts <plan.tools.data.ts path> <out toolchain.plan.jsonc>
// Rules come from package/mpm-shadow/cli/README.md, section "What the cutover converter must write".
import { readFileSync, writeFileSync, } from 'node:fs';
import { pathToFileURL, } from 'node:url';

const [dataPath, outPath,] = process.argv.slice(2,);
if (dataPath === undefined || outPath === undefined)
  throw new Error('usage: convert.ts <plan.tools.data.ts> <out>',);

const source = readFileSync(dataPath, 'utf8',);
const { ALL_TOOLS, } = await import(pathToFileURL(dataPath,).href,);

// Comments that sit directly above a tool object, keyed by tool id (the old file keeps them above `{ id: '...'`).
const comments = new Map<string, string[]>();
{
  const lines = source.split('\n',);
  for (let i = 0; i < lines.length; i += 1) {
    const match = /^\s+id: '([^']+)',$/u.exec(lines[i] ?? '',);
    if (match === null || (lines[i - 1] ?? '').trim() !== '{') continue;
    const block: string[] = [];
    let j = i - 2;
    while (j >= 0 && (lines[j] ?? '').trim().startsWith('//',)) {
      block.unshift((lines[j] ?? '').trim(),);
      j -= 1;
    }
    comments.set(match[1] as string, block,);
  }
}

// GitHub owner/repo of a downloaded tool, read from its recorded release URL (checked against the README table).
function githubRepo(artifacts: Record<string, { url: string; }>,): string | undefined {
  for (const row of Object.values(artifacts,)) {
    const found = /^https:\/\/github\.com\/([^/]+\/[^/]+)\/releases\/download\//u.exec(row.url,);
    if (found !== null) return found[1];
  }
  return undefined;
}

const ASSET_HINT: Record<string, string> = { 'harper-cli': 'harper-cli-*', tombi: 'tombi-cli-*', };

// Tags that are pinned and not a plain version: the old request records the bare version, the tag has a prefix.
const TAG_FOR_PINNED: Record<string, string> = { bun: 'bun-v1.3.14', };

const notes: string[] = [];
const out: Record<string, Record<string, unknown>> = {};

for (const tool of ALL_TOOLS as readonly any[]) {
  const id: string = tool.id;
  const range: string = tool.request.pinned ?? tool.request.channel ?? 'latest';
  let request: string;
  const entry: Record<string, unknown> = {};
  const options: Record<string, unknown> = { ...tool.options, };
  const env: Record<string, string> = { ...tool.env, };
  let keepManager = false;
  switch (tool.route) {
    case 'cargo-binstall':
      request = `cargo:${tool.packageId}@${range}`;
      break;
    case 'mpm-pnpm':
      request = `npm:${tool.packageId}@${range}`;
      break;
    case 'mpm-uvx':
      request = `pipx:${tool.packageId}@${range}`;
      break;
    case 'rustup-toolchain':
      request = `rust@${range}`;
      keepManager = true;
      break;
    case 'cargo-git': {
      // The revision rides in `request.pinned` as `ref:<rev>`; the repository URL rides in `options.cargoGit.url`.
      const revision = String(tool.request.pinned ?? '',).replace(/^ref:/u, '',);
      request = `git:${tool.options.cargoGit.url}@${revision}`;
      delete options.cargoGit;
      entry.packageId = tool.packageId;
      break;
    }
    case 'go-install':
      request = `go:${tool.packageId}@${range}`;
      break;
    case 'android-sdk':
      request = `android-sdk@${range}`;
      break;
    default: {
      if (['node', 'zig', 'go', 'aws', 'rustup',].includes(id,)) request = `${id}@${range}`;
      else if (id === 'java') request = `java:temurin@${String(tool.request.channel,).replace('temurin-', '',)}`;
      else if (id.startsWith('conda:')) request = `conda:${tool.packageId}@${range}`;
      else {
        const repo = githubRepo(tool.artifacts,);
        if (repo === undefined) throw new Error(`no GitHub origin for ${id}`,);
        const tag = tool.request.pinned === undefined ? 'latest' : (TAG_FOR_PINNED[id] ?? tool.request.pinned);
        request = `github:${repo}@${tag}`;
      }
    }
  }
  entry.request = request;
  entry.route = tool.route;
  if (keepManager) {
    entry.manager = tool.manager;
    entry.packageId = '{version}';
  }
  entry.bins = tool.bins;
  if (tool.os !== undefined) entry.os = tool.os;
  if (id in ASSET_HINT) options.asset = ASSET_HINT[id];
  if ('assetPattern' in options) {
    options.asset = options.assetPattern;
    delete options.assetPattern;
    notes.push(`${id}: assetPattern renamed asset`,);
  }
  if ('cargoGit' in options) notes.push(`${id}: cargoGit needs the git:<url>@<revision> request by hand`,);
  if (id === 'rust') env.RUSTUP_TOOLCHAIN = '{version}';
  if (id === 'java') env.JAVA_HOME = 'java/{version}';
  if (id === 'android-sdk') {
    env.ANDROID_HOME = 'android-sdk/{version}';
    env.ANDROID_SDK_ROOT = 'android-sdk/{version}';
  }
  if (Object.keys(options,).length > 0) entry.options = options;
  if (Object.keys(env,).length > 0) entry.env = env;
  out[id] = entry;
}

const lines: string[] = [
  '// toolchain.plan.jsonc: the toolchain plan. People edit this file; mpm-shadow writes toolchain.lock.json from it.',
  '// Converted from package/file-enforcer/core/src/toolchain/plan.tools.data.ts by the one-off cutover converter.',
  '{',
  '  "schemaVersion": 1,',
  '  "tools": {',
];
const ids = Object.keys(out,);
ids.forEach((id, index,) => {
  for (const comment of comments.get(id,) ?? []) lines.push(`    ${comment}`,);
  const body = JSON.stringify(out[id], null, 2,).split('\n',).map((line, n,) => n === 0 ? line : `    ${line}`).join('\n',);
  lines.push(`    ${JSON.stringify(id,)}: ${body}${index === ids.length - 1 ? '' : ','}`,);
},);
lines.push('  }', '}', '',);
writeFileSync(outPath, lines.join('\n',),);
console.log(`wrote ${ids.length} tools to ${outPath}`,);
for (const note of notes) console.log(note,);
```
