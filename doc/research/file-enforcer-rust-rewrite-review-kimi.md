# Outside review of the Rust file-enforcer plan: opencode-go/kimi-k3.1-pro

Review of [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md) as of 2026-10-29,
run through `pi --print --no-tools --thinking xhigh` with the planning doc, the four research reports and two decisions attached (second run; the first returned only a preamble).
Findings are unverified model output until checked against the repository.

## 1. Contradictions between decisions and research

### High: the committed-input rule has no defined global-scope counterpart

- **Passages:** “file-enforcer renders, from committed inputs only” (`doc/planning/file-enforcer-rust-rewrite.md`, “Toolchain: split between file-enforcer and the mpm wrapper”), but the global plan, lock, and manifests live in “`${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`” (`doc/decision/mpm-toolchain-adoption.md`, “Machine-global scope”).
- **Failure:** The repository rule cannot render the global lock or manifests from committed inputs because those inputs are intentionally uncommitted. The separate `user` commands are specified for out-of-repo rules, but no decision assigns global toolchain rendering to them. A first `mpm-shadow sync --global` can therefore lack a current lock or current mpm manifests.
- **Smallest fix:** Qualify “committed inputs only” as repository-scope-only and explicitly assign global plan-plus-resolution-to-lock and lock-to-manifest rendering to `file-enforcer user write|check`, with exact paths and ordering before `mpm-shadow sync --global`.

### Medium: “exact” check and removal of CI workarounds conflict with accepted machine-dependent output

- **Passages:** “Every input and output path is known from the config without running anything, so `check` is exact” and “CI's `git diff --exit-code` workaround ... can go” (`doc/planning/file-enforcer-rust-rewrite.md`), versus “issue #335 records the list changing between checkouts after filtered or stale installs, so `check` reports `mise.toml` drift on such a machine” (same file, “Root mise.toml PATH”).
- **Failure:** Two machines with identical committed inputs can receive different `check` verdicts. A filtered install can fail CI or a contributor check despite no committed-file error.
- **Smallest fix:** Narrow the exactness claim to reproducibility from committed inputs plus declared host state. Give environment-dependent drift a distinct diagnostic and remediation, and state that CI’s mise check requires the canonical full `pnpm install`.

## 2. Required outputs and consumers left uncovered

### High: cutover deletes the IDE’s hard-coded `bin-paths` entry point

- **Passages:** The consumer has “`ENTRY_POINT = "package/file-enforcer/core/src/toolchain/bin-paths.ts"`” and runs it as Node in a cleared-environment sandbox (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, “`bin-paths` JSON and the exposure directory”). Cutover only says it “switches tasks, CI, and `Dockerfile`, and deletes `package/file-enforcer/core`” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** After cutover, the IDE invokes a deleted TypeScript file. Language-toolchain discovery fails even if `mpm-shadow bin-paths` itself works.
- **Smallest fix:** Add `package/ide/app/src/language/toolchain.rs` and `package/ide/app/bin/inspect-language-toolchain.ts` to the cutover checklist. Specify how the cleared-environment sandbox locates mpm-shadow before toolchain sync, and exercise both provisioned and unprovisioned states in the real IDE sandbox.

### High: strict JSON is preserved, but the actual jq contract is not

- **Passages:** The design guarantees only that “`toolchain.lock.json` stays strict JSON for its `jq` readers” (`doc/planning/file-enforcer-rust-rewrite.md`). Research identifies the frozen surface as “the tool-id keys ... and `version`/`artifacts`/`options.androidPackages`” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, “Consumers and contracts”).
- **Failure:** A valid strict-JSON lock that renames tool IDs, nests declaration and resolution data differently, or omits `options.androidPackages` breaks Dockerfiles, CI setup, release workflows, and TypeScript parsers.
- **Smallest fix:** Record the existing key contract explicitly and add golden consumer tests that execute the current jq queries and parsers against the Rust-rendered lock before cutover.

### High: postinstall commands disappear from the split

- **Passages:** mpm-shadow receives “the `.cache/toolchain-bin` exposure farm and postinstall links” (`doc/planning/file-enforcer-rust-rewrite.md`). The existing exposure phase also includes “postinstall commands (`:294`, `search-shadow link .cache/search-shadow-bin`)” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, R6).
- **Failure:** Links alone do not create every exposed directory. In particular, `.cache/search-shadow-bin` can remain absent, so the generated mise PATH names a directory that sync never populates.
- **Smallest fix:** Include structured postinstall commands, without shell interpolation, in mpm-shadow sync’s required behavior and test the `search-shadow` case in a disposable home.

### High: repository-only comparison can miss global-only routes

- **Passage:** “Global lock uses also `cargo-git` 1 and `go-install` 1” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, R4).
- **Failure:** The planned old-versus-new comparison on this repository exercises only the repository routes. A Rust sync without `cargo-git` or `go-install` can pass that comparison, then fail to provision the converted global toolchain after the TypeScript implementation is deleted.
- **Smallest fix:** Require a route-and-option parity matrix covering both locks, including `cargo-git`, `go-install`, components, targets, features, OS filters, and postinstall behavior. Verify global sync in a disposable home before cutover.

### Medium: two non-CLI consumers are absent from cutover

- **Passages:** The inventory lists “`package/test-fixture/file-enforcer-perf` (benchmark of the old tool; retires)” (`doc/research/file-enforcer-rust-rewrite-inventory.md`, §2.3). The accepted decision also requires that “the `prefer-readonly-parameter-type` read moves to a new fixture package” (`doc/decision/monorepo-manager-all-rust.md`, “Consequences”).
- **Failure:** Deleting only `package/file-enforcer/core` leaves the performance fixture importing a deleted package and the lint fixture reading deleted source, causing unrelated lint or test failures at cutover.
- **Smallest fix:** Delete the performance fixture and create the replacement TOML-edit fixture in the cutover commit.

### Medium: the existing update `--check` contract has no explicit disposition

- **Passages:** Existing mise callers use “the argument grammar ... `--global` and `--check`” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, “Mise tasks”), while the design assigns only unqualified “`sync`, `status`, and `update`” to mpm-shadow (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** Repointed update tasks or automation retaining `--check` will be rejected by clap; silently dropping it also loses the existing non-mutating update report.
- **Smallest fix:** Either specify `mpm-shadow update --check [--global]`, including its lock/render behavior, or explicitly record its removal and migrate every caller.

### Medium: required output modes are not part of the write contract

- **Passages:** The rules file and seed require “mode 0600” (`doc/research/file-enforcer-rust-rewrite-inventory.md`, §1.1), while the adopted write default is only “temp file then rename, only when content differs” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** A default-mode temporary file can replace a 0600 output with a more permissive mode, and an unchanged-content write may never repair an incorrect existing mode.
- **Smallest fix:** Add an optional output-mode property to rules, preserve or enforce it independently of content changes, and test both forbidden-strings outputs and the 0600 global plan.

## 3. Adopted defaults that need correction or confirmation

### Medium: watch mode ignores edits to generated outputs

- **Passages:** The default reruns only “the rules whose declared inputs changed” (`doc/planning/file-enforcer-rust-rewrite.md`), while the existing development loop “warns when a generated file is edited” (`doc/research/file-enforcer-rust-rewrite-inventory.md`, §2.1).
- **Failure:** Editing a generated file produces no event on a declared input, so watch can leave the edit in place indefinitely without warning or regeneration.
- **Smallest fix:** Watch declared outputs as well. Suppress events caused by the tool’s own writes, then warn and rerun the owning rule when an output changes externally.

### Medium: “no lock” permits stale successful concurrent writes

- **Passage:** “Writes: temp file then rename, only when content differs; no lock and no cache” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** A manual `write` and `watch` can render different input snapshots concurrently. Atomic rename prevents malformed files, but the older snapshot can rename last; both commands can report success while outputs do not represent the latest inputs.
- **Smallest fix:** Add a narrowly scoped per-config writer exclusion, or coordinate watch and manual writes through one writer. This does not require restoring the old staleness manifest or cross-process cache.

### Low: the only global-plan backup is agent scratch

- **Passage:** “The old file is backed up under `${HOME}/temp/agent` first” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** Cleaning agent scratch removes the only rollback copy of a hand-edited, commented, 0600 file after the original has been replaced.
- **Smallest fix:** Keep a 0600 backup beside the original, or in a documented durable backup location, until the converted global toolchain has been verified.

## 4. Ordering and bootstrap problems

### Critical: no initial resolution file is created

- **Passages:** “`mpm-shadow update` writes only `toolchain.resolution.json`” and file-enforcer renders the lock from plan plus resolution (`doc/planning/file-enforcer-rust-rewrite.md`, “Lock writer”). Cutover includes only “the plan conversion”; the global converter likewise writes only `toolchain.plan.jsonc`.
- **Failure:** At the first Rust render, declaration data exists but the committed versions, URLs, and SHA-256 values have no source. Rendering either fails or produces an incomplete lock. Re-resolving everything to recover would change pins instead of converting them.
- **Smallest fix:** Make conversion split each existing lock into declaration and resolution files, preserving every tool ID, version, artifact URL, hash, and option. Require the new renderer to reproduce the old lock before cutover, for both repository and global scopes.

### Critical: cutover can reference binaries that have not been released

- **Passages:** “Everything is built first” precedes one switch (`doc/planning/file-enforcer-rust-rewrite.md`), while distribution is “published to crates.io and installed by `mpm-shadow sync`.” Current mpm-shadow command mode supports “`mpm-shadow prune`, `--help`, `--version` only” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, §1).
- **Failure:** Building locally does not make a new sync-capable mpm-shadow archive or file-enforcer crate available to CI. Bootstrap can fetch the old prune-only binary, or request an unpublished version and fail. `prepare:pnpm` can also invoke file-enforcer before sync has installed it.
- **Smallest fix:** Add a pre-cutover release gate: publish the new mpm-shadow and file-enforcer, seed their locked artifacts, then prove a clean container can fetch mpm-shadow, sync file-enforcer and the other tools, and run `prepare:pnpm`. Only then switch callers and delete TypeScript.

### High: update success does not imply an updated lock

- **Passages:** “`mpm-shadow update` writes only `toolchain.resolution.json`” and file-enforcer is “the lock's only writer” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** A caller running `mpm-shadow update` followed by `mpm-shadow sync` still syncs the old committed lock. The update appears successful, but nothing changes, and file-enforcer check reports drift until an undocumented third command runs.
- **Smallest fix:** Define one update pipeline and task: write resolution, render the lock and manifests through file-enforcer, then report. Either invoke the library from mpm-shadow or make the sequenced file-enforcer invocation explicit and test update-then-sync.

### High: conversion of existing global installations has no per-machine ordering

- **Passage:** “At the switch, a one-off converter writes `toolchain.plan.jsonc` beside it ... and replaces the `.data.ts`” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** A repository commit cannot itself convert files in every user’s home. After the TypeScript loaders are deleted, an unconverted machine has no readable global plan. Conversion must also precede global lock rendering and sync.
- **Smallest fix:** Ship an idempotent migration command or explicit runbook that runs on each machine: back up, split plan and resolution, render and compare global outputs, verify global sync in a disposable scope, then replace the TypeScript files.

### High: `cargo install` is not a complete rung-zero path

- **Passages:** “`cargo install monochromatic-mpm-shadow` ... [is] the only one on platforms without a prebuilt archive” (`doc/planning/file-enforcer-rust-rewrite.md`). Research notes that “compiling needs rustup, which the sync installs” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, “Bootstrap order”).
- **Failure:** A fresh macOS or Windows machine without Rust cannot execute the documented fallback, and the tool that would install Rust is the tool being installed.
- **Smallest fix:** Document ambient rustup/cargo installation as rung zero for that path, or publish archives for every supported bootstrap platform. Verify a clean-machine path rather than only compiling the crate.

### Medium: the bootstrap token requirement is omitted

- **Passage:** “A token is an effective rung-0 input for containers and CI” (`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, “Bootstrap order”).
- **Failure:** A clean container without `.env.local` or a GitHub token can download mpm-shadow successfully and then fail during sync on GitHub-backed routes or API quota.
- **Smallest fix:** State the token requirement and supported token sources in the Dockerfile, CI setup, and fresh-machine runbook; test the no-token failure diagnostic.

## 5. Features whose stated behavior cannot achieve the intended effect

### High: update cannot onboard a new npm or PyPI tool under locked sync semantics

- **Passages:** “`mpm outdated` sees only tools installed on the machine running it,” and the remedy is “Require sync first, and auto-sync on such failures and retry” (`doc/planning/file-enforcer-rust-rewrite.md`, “Update”).
- **Failure:** Adding a new npm/PyPI plan entry leaves it uninstalled and without a resolved version. Sync cannot install its locked version because none exists; `mpm outdated` still cannot see it after that sync attempt, so the single retry fails.
- **Smallest fix:** Retain direct npm/PyPI registry resolution for entries with no installed and locked instance, or define a separate initial-resolution operation. Test adding a previously unknown tool, not only updating installed tools.

### High: the generated schema does not constrain the hand-written decoder

- **Passages:** “The config is decoded by hand per field” and “The schema is derived from the Rust config types with `schemars`” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** The decoder and schema can drift independently. Editors can accept a config the CLI rejects, or reject one the CLI accepts; regenerating and committing the schema only detects type changes, not decoder divergence. The `$schema` property itself can also be rejected if the decoder does not explicitly allow it.
- **Smallest fix:** Make one definition authoritative. Either decode through the typed representation used for schema generation or run schema validation before hand extraction, with conformance tests for unknown fields, wrong types, and `$schema`.

### Medium: catalog failures can leave partial output

- **Passages:** The required catalog behavior is “All problems collected first, then written, so a failure writes nothing” (`doc/research/file-enforcer-rust-rewrite-inventory.md`, §1.2). The design specifies per-file temp-and-rename writes but no all-rule validation phase.
- **Failure:** Earlier Cargo manifests or catalog edits can be renamed before a later conflict or parse error aborts the run, leaving a mixed state despite a nonzero exit.
- **Smallest fix:** Render and validate every affected catalog and manifest before committing any write. At minimum, make the complete `rust-catalog` rule a two-phase operation and test a conflict spanning multiple crates.

## 6. Decisions resting on unverified claims

### High: browserslist-rs support is assumed rather than demonstrated

- **Passage:** “A `browserslist` rule kind resolves `.browserslistrc`'s production section with `browserslist-rs`” (`doc/planning/file-enforcer-rust-rewrite.md`). The attached crate research does not evaluate browserslist-rs, and the inventory says the output needs “the browserslist query engine plus caniuse data.”
- **Failure:** The crate may parse basic queries yet differ in environment selection, config discovery, stats handling, or query semantics. Accepting different browser-data versions does not cover unsupported or differently interpreted syntax.
- **Smallest fix:** Make the required vet a differential test against the repository’s real `.browserslistrc`, covering production selection and disabled external config/stats discovery, and validate the resulting array through `package/config/rolldown/src/browserslist-targets.ts`.

### High: the npm/PyPI update path assumes sufficient mpm output

- **Passage:** “npm and PyPI tools: `mpm outdated --output-format json`” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** The design does not establish that the JSON identifies every relevant planned tool unambiguously and carries the requested-track result needed by the resolution schema. Scope overlap, missing tools, and manager-specific output can produce incomplete or misattributed resolutions.
- **Smallest fix:** Probe the installed mpm version with repository and global fixtures covering installed, missing, newly added, pinned, and latest-track tools. Assert the exact JSON fields required before removing native npm/PyPI resolvers.

### Medium: the non-archive platform path assumes buildable and operable sync

- **Passage:** “`cargo install monochromatic-mpm-shadow` ... [is] the only one on platforms without a prebuilt archive” (`doc/planning/file-enforcer-rust-rewrite.md`), while the accepted mpm decision records that “macOS and Windows provisioning parity ... is not exercised” (`doc/decision/mpm-toolchain-adoption.md`, “Evidence limits”).
- **Failure:** The crate may compile while individual providers, path handling, archive extraction, or exposure operations fail on the target host, leaving no usable bootstrap route.
- **Smallest fix:** Run clean-home sync smoke tests on each platform presented as supported, or label those platforms unsupported until prebuilt archives and provisioning tests exist.

### Low: JetBrains `$schema` behavior remains unknown

- **Passage:** “Unverified: JetBrains support for `$schema` in `.jsonc` files” (`doc/planning/file-enforcer-rust-rewrite.md`).
- **Failure:** The committed `$schema` key may provide no completion or validation in JetBrains, so the editor-facing benefit of the generated schema is absent there.
- **Smallest fix:** Test the current JetBrains JSON/JSONC editor with the actual filename. If unsupported, document a manual schema mapping or SchemaStore registration rather than relying on `$schema`.
