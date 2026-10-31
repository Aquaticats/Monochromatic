# Rust file-enforcer rewrite handover

## Purpose

Carry the Rust + clap + JSONC file-enforcer rewrite across the user's reboot
and into implementation.
The design is accepted:
[`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md).
Every question,
answer,
and its history is in
[`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md).
Implementation is tracked in issue #1818.

## State on 2026-10-29

- The decision record is accepted and committed,
  with the dependency approvals (`toml_edit`,
  `browserslist-rs`,
  `schemars`,
  `serde_json`) and the in-repo XML edit crate.
- All three gate probes finished
  ([`doc/research/file-enforcer-rust-rewrite-gate-probes.md`](../research/file-enforcer-rust-rewrite-gate-probes.md)),
  and the user then chose direct registry lookups for every npm and PyPI version.
- The four halted vet drafts are committed whole under `doc/audit/tech-file-enforcer-*-vet-2026-10-29.md`.
  Their scratch (crate downloads,
  clones,
  and extractions) stays under `${HOME}/temp/agent/`.
- No code exists yet.

## Update 2026-10-30
in-repo browserslist and parallel implementation

The oxc look is done
([`doc/research/file-enforcer-rust-rewrite-oxc-browserslist.md`](../research/file-enforcer-rust-rewrite-oxc-browserslist.md)),
and the user replaced both crates:
"Change of plans,
for browserslist,
we retrieve the data and write something conforming to the original browserslist ourselves,
only supporting the queries we use now,
throw on everything non-conforming with \"not implemented\"."
The decision record carries the change.
The user also said "Don't serialize aggressively paralleling work.",
so implementation areas now run as parallel worktree agents (limit five at once),
sharing the brief `${HOME}/temp/agent/fe-rust/brief-common.md`.

State (updated as agents finish;
each finished branch is reviewed and merged into `main`):

- Done and merged:
  pnpr owns `config.yaml` (task 24,
  merge `668311d986`;
  CI advisory drift step `93a95f4259`);
  browserslist subset specification
  (`doc/research/browserslist-subset-semantics.md`,
  `f1778271d5`);
  dependency-catalog Rust library `package/dependency-catalog/core-native` (task 21,
  merges `6aae0afe1c` and `b7798a4da5` for the closure fixes);
  XML edit crate `package/xml-edit/core-rust` (task 26,
  `032366654f`);
  rules list with `?` (task 23,
  `2ebe1dd93a`,
  now owned by session monochromatic-7a);
  in-repo browserslist `package/browserslist/core-rust` (task 28,
  `c60687b1d8`,
  then the Node pre-release filter with node-releases 2.0.62 and
  `doc/troubleshooting/browserslist-node-major-prerelease.md`);
  file-enforcer `core-rust` and `cli` (task 20,
  `71d0ad760b`);
  mpm-shadow toolchain commands (task 22,
  first pass,
  then merge `bca46afa8b`:
  `request` is `<origin>@<range>`,
  the lock is derived from the plan alone,
  every resolver is ported,
  and the crate's lint debt is cleared);
  the linter's Dylint driver skip (`package/linter/monochromatic-lint`,
  after the owner-approved toolchain sync and driver warm on 2026-10-30).
- Merged 2026-10-31 through the core-hardening branch:
  branch `worktree-agent-a23c3740c67ba36d9` wires the `rust-catalog`,
  `browserslist`,
  and `jetbrains-harper` rule kinds in the cli crate and moves the browserslist lock to the root.
  `git merge` had refused because the main checkout held an uncommitted `pnpm-lock.yaml` of unknown authorship
  (13 catalog specifiers raised to their resolved versions);
  the owner chose "Commit it, then merge",
  so it is `add4429c97` on its own.
  monochromatic-c0 answered afterwards that the change was not theirs
  (it was already modified when that session started).
  Its scope deviations to keep in mind:
  `core-rust/src/plan.rs` no longer reports `rule-order` between two composing rules,
  so "rust-catalog after toml-keys" is a config convention;
  `package-licenses` moved after `rust-catalog` in `file-enforcer.config.jsonc`;
  the jetbrains-harper policy constants are hardcoded from the TypeScript tool;
  the browserslist `environment` field selects nothing because sections are refused.
- Merged 2026-10-31:
  branch `worktree-agent-ad604356a8bd46267` hardens the core and contains the wiring branch.
  It adds `parse_jsonc_with_spans` to `monochromatic-jsonc-edit` without a version bump (0.1.1 is published),
  so jsonc-edit needs a bump and a Trusted Publishing release before file-enforcer publishes.
  Every config decode error now reads `file:line:col: /pointer: message`.
  No `user watch` command exists (the decision lists only `user write` and `user check`).
- Running (worktree agents):
  loose ends
  (browserslist `bin/*.ts` oxlint errors with a real rerun of the scripts,
  `XDG_CONFIG_HOME` in JetBrains discovery,
  closure lints in the cli crate and the pre-existing ones in jsonc-edit and browserslist `fetch.rs`,
  clap-derived closures in `args.rs`,
  and the watch test that timed out once under load);
  first-publication prep for the five new crates
  (`cargo-publish.yml` jobs,
  package contents,
  dry runs and container builds of the unpacked crates,
  runbook sections;
  the owner token step is asked only once the dry runs pass);
  the cutover inventory `doc/planning/file-enforcer-rust-cutover.md`
  with a rehearsed toolchain plan conversion in scratch.
- Then:
  the cutover (task 25),
  dropping the rules-file generator (7a's config switch landed in `ad9fe75d05`).

Decided 2026-10-30 and recorded in the decision record:
the HTTP stack (`hyper`,
`hyper-util`,
`tokio`,
`http-body-util`,
`hyper-rustls`),
browserslist data by our own registry fetch with a lock (`flate2` and `tar`),
Firefox ESR from Mozilla product-details,
`jsonschema` for the schema corpus check,
and the browserslist grammar (exact text,
`#` comments and blank lines,
count 1 only).

Cutover reminders from finished agents:

- `package/config/pnpr/config.yaml` still starts with "Generated by file-enforcer (generatePnprConfig in file-enforcer.config.ts); edit that source."
  for byte identity;
  reword it at cutover through the constant `PNPR_CONFIG_HEADER_LINE` in `package/config/pnpr/src/generated-config-settings.ts`,
  knowing a content change redeploys pnpr on Coolify and starts a publish run.
- `file-enforcer.config.ts` renders pnpr's config through pnpr's `renderPnprConfigForWorkspace` but writes through its own `overwrite`,
  so `--check` reports drift instead of writing;
  the Rust engine needs the same split.
- Delegated 2026-10-30: the user wrote "I'm handing the forbidden-strings and git-policy work to another session. If you have more delegate them to it too."
  The forbidden-strings release, the cli-git publish, and their cutover config switch belong to that session;
  monochromatic-7a (the cli-git Rust wrapper session) confirmed it holds them and received the items on 2026-10-31
  (release order, the merged rules-list feature, the config switch);
  this session only drops the rules-file generator, and only after 7a reports the config switch done.
  Gate from 7a (2026-10-31): `cli-git.config.jsonc` cannot change until a wrapper that reads a `rulesFile` array
  (0.4.4, after forbidden-strings 0.6.0) is published, the toolchain lock names it, and it is installed as the production git;
  an older wrapper cannot parse the array. So the cutover keeps the generator until 7a says the switch landed.
  Progress seen on `main` 2026-10-31:
  forbidden-strings 0.6.0 (`5ec9a377e2`) and git-policy/cli 0.4.4 (`7ce408f679`) are released;
  the lock entry,
  install,
  and config switch are still 7a's to report.
  Gate cleared 2026-10-31:
  7a reported the switch done in `ad9fe75d05`
  (`FORBIDDEN_STRINGS_RULES` in `mise.toml`,
  generated from `file-enforcer.config.ts`,
  is now `{{config_root}}/forbidden-strings.append.txt,{{config_root}}/forbidden-strings.append.local.txt?`,
  and `cli-git.config.jsonc` `rulesFile` is the matching array),
  with wrapper 0.4.4 installed as the production git.
  The cutover commit drops the rules-file generator (the eager compile-rules step) and the seeded local appendix step,
  and the root `mise.toml` stops being generated at cutover
  (decision record,
  "Consequences":
  "the root `mise.toml` is no longer generated"),
  so it simply keeps the `ad9fe75d05` value as a committed file.
- mpm-shadow at cutover
  (from `package/mpm-shadow/cli/README.md`):
  the converter writes each tool's `request` as `<origin>@<range>`,
  writes `{version}` for rust `packageId` and `RUSTUP_TOOLCHAIN`,
  `JAVA_HOME`,
  `ANDROID_HOME`,
  and `ANDROID_SDK_ROOT`,
  and gives `harper-cli` and `tombi` an `options.asset` glob
  (`tombi`'s need is untested);
  `assetPattern` and `cargoGit` are now unknown fields.
  There is no adopt step:
  the lock is rebuilt with `update`,
  and the rehearsal against the converted plan changed six tools
  (musl builds for `ripgrep`,
  `harper-cli`,
  `hyperfine`,
  `marksman`,
  and `bin`,
  and `superhtml` 0.7.0 to 0.7.1),
  while the global lock moves `aws` 2.39.2 to 2.39.3.
  Only the four Linux platforms are written
  (`doc/decision/mise-lock-linux-only-stopgap.md`),
  so `bin`'s macOS and Windows rows and `bun`'s baseline rows go away.
  Show that lock diff to the user as the review surface.
- forbidden-strings must be released before cli-git is published:
  cli-git now uses the rules-list API (task 23; see the planning doc, "Cutover dependency").

## Superseded next action
look at oxc-browserslist

The user said the next step is to look at `oxc-browserslist`,
before building on `browserslist-rs`.
Why it matters:
the gate probe measured `browserslist-rs` 0.21.2 on the repository's `.browserslistrc` against npm `browserslist`
and found older data
(14 targets against 18,
`chrome151` against `chrome156`,
a hard-coded `firefox140` ESR against `firefox153`,
Node up to 26.8.0 against 26.12.0).

What is already known,
from
[`doc/audit/tech-file-enforcer-browserslist-engine-vet-2026-10-29.md`](../audit/tech-file-enforcer-browserslist-engine-vet-2026-10-29.md)
(sections "oxc-browserslist 5.0.1" and "E7" to "E9"):

- `oxc-browserslist` 5.0.1 (2026-08-12,
  MIT) is the oxc project's fork of `browserslist-rs`,
  passed screening,
  and its published crate matches its tag byte for byte.
- It has no config-file loading,
  no `BROWSERSLIST*` environment variables,
  and no `extends`,
  so file-enforcer parses `.browserslistrc` itself either way.
- Its data inputs at 5.0.1 (`caniuse-lite 1.0.30001809`,
  `browserslist 4.28.8`) are older than `browserslist-rs` 0.21.2's,
  so the published crate alone does not close the data gap;
  its `cargo codegen` data regeneration has not been run.
- Unread:
  how it resolves `Firefox ESR`,
  `not dead`,
  and the `ChromeAndroid` aliases.
- Single crates.io owner (`Boshen`).

The user picked this candidate and it is already in a repository vet report,
so this is not broad vetting,
and the check-in that `.agents/skills/choosing-technology/SKILL.md` requires before enumerating candidates does not apply.
If looking at it means searching for candidates beyond memory,
the user's picks,
and repository precedent,
it becomes broad:
ask the user who runs that vet first.
The probe scratch for `browserslist-rs` is under `${HOME}/temp/agent/fe-rust/gates/`,
and an oxc clone is at `${HOME}/temp/agent/oxc-browserslist-2026-10-29`.

## After that

Start the implementation areas tracked in issue #1818:

1.  `package/file-enforcer/core-rust` and `package/file-enforcer/cli`.
2.  The dependency-catalog Rust library.
3.  mpm-shadow:
    `sync`,
    `status`,
    `update [--check]`,
    `lock`,
    `lock --check`,
    `bin-paths`,
    `--global`,
    the lock digest,
    and bootstrap.
4.  forbidden-strings and git-policy:
    the comma-separated rules list with a trailing `?` for optional entries.
5.  pnpr generates and checks its own `config.yaml`.
6.  The in-repo XML edit crate.
7.  Conversions and the one-switch cutover.

Open facts for implementation are listed in the decision record's "Consequences":
`schemars` derive without serde `Deserialize` is unproven,
the golden test's JSON Schema validator crate is unnamed,
and tag `v8.2.0` of mpm resolves to `99eb596b` while `doc/decision/mpm-toolchain-adoption.md` cites `f095931a`.

## Lesson recorded from this work

The grilling settled the design first and left its dependencies as vet gates after acceptance;
the user then approved them without vets.
The user asked that grilling enumerate and pick every dependency first,
now `GXD` in `AGENTS.md`,
and that each broad vet check in with the user before enumerating candidates,
now "Check in before broad discovery" in `.agents/skills/choosing-technology/SKILL.md`.

## Task 20 status on 2026-10-30
Built on branch `worktree-agent-a9e487e6d38ec954c`.

`package/file-enforcer/core-rust` (library) and `package/file-enforcer/cli` (binary `file-enforcer`) exist,
with `file-enforcer.config.jsonc`,
`file-enforcer.schema.json`,
the content files under `.config/file-enforcer/`,
and the schema mapping in `.idea/jsonSchemas.xml`.
Run against a throwaway copy of the repository,
the Rust tool reports zero drift for `CLAUDE.md`,
`LICENSE`,
every
`LICENSES/` directory and all 26 `Cargo.toml` key sets,
which the TypeScript tool wrote.
The rule-kind interface for the seams (rust-catalog,
browserslist,
jetbrains-harper) is in
[`package/file-enforcer/core-rust/README.md`](../../package/file-enforcer/core-rust/README.md);
until their builders are registered those kinds warn with `kind-not-wired` and plan nothing.
Not switched:
no task,
CI,
Dockerfile or config uses the new binary.
Next for this area:
register the three builders when their crates land,
then the cutover (task 25).

## Rule kinds wired on 2026-10-31
Branch `worktree-agent-a23c3740c67ba36d9`.

The `file-enforcer` binary now implements `rust-catalog`,
`browserslist` and `jetbrains-harper`
(`package/file-enforcer/cli/src/kinds.rs`;
details in the CLI README,
section "Rule kinds linked here").
The browserslist data lock moved to `browserslist-data.lock.json` at the repository root,
and the rule names it in its required `lock` field.
Against a throwaway copy of this repository the Rust tool reports no drift for `rust-catalog.toml`
and every manifest the globs match (32 files) as the TypeScript tool wrote them,
detects and repairs a changed `[workspace.dependencies]` table,
writes the same `.browserslistrc.resolved.local.json` as npm `browserslist` 4.29.5 on the same `caniuse-lite`,
and writes byte-identical JetBrains settings files to the retired TypeScript policy in a disposable home.
Engine change:
the read-order check skips pairs where both rules compose,
and `package-licenses` moved after `rust-catalog` in `file-enforcer.config.jsonc`
because it reads manifests and does not compose.
Sandbox note:
a Rust binary's direct connection to `registry.npmjs.org` is aborted inside the agent sandbox
(`Software caused connection abort`) while `curl` works,
so the cold-cache fetch was not exercised there;
the cache was seeded with `curl` and verified against the lock.
Open:
the `bin/*.ts` scripts of the browserslist package now belong to a workspace package
and still have oxlint findings
(sync Node APIs,
arrow functions,
try/finally,
missing TSDoc).

## How to respond

Reply in the session that resumes this work;
it reads this file,
then the decision record,
then issue #1818.
