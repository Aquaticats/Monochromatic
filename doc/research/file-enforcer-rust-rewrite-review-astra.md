# Outside review of the Rust file-enforcer plan: openai/gpt-6-astra

Review of [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md) as of 2026-10-29,
run through `pi --print --no-tools --thinking xhigh` with the planning doc, the four research reports and two decisions attached.
Findings are unverified model output until checked against the repository.

Document-only review of the supplied files. I did not re-probe the implementation or external tools. Findings are ranked by severity; missing contracts are distinguished from demonstrated contradictions.

## 1. The plan, resolution, lock, and sync workflow has an unresolved cycle

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`mpm-shadow update` writes only `toolchain.resolution.json`”

> “file-enforcer renders `toolchain.lock.json` from `toolchain.plan.jsonc` plus `toolchain.resolution.json`”

> “`update` syncs the missing tools, then retries the lookup once”

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`:

> “mpm-shadow (sync, status, update) needs the lock only (all fields present).”

The last passage is a research inference, but no subsequent decision explicitly replaces it.

**Failure scenario:** Changing a request in the plan leaves the lock carrying the old request. A lock-only updater resolves the wrong intent. Even if update reads the plan, it writes only the resolution, so a subsequent lock-driven sync still installs the old version until file-enforcer runs. Adding a previously unresolved npm tool exposes a further cycle: update requires installation, installation requires a resolved lock entry, and lock generation requires resolution.

**Smallest fix:** Specify the authoritative inputs and command sequence:

- Update reads the current plan, not merely its previous projection.
- Resolution records are validated against the declaration they resolve, especially changed pins, sources, and routes.
- `toolchain:update` runs resolution followed by file-enforcer rendering.
- Sync consumes the completed lock without re-resolution.
- Put the previously unresolved npm/PyPI case to the owner. “Sync first” is executable only when an installable resolution already exists.

## 2. Exact checking requires a staged dependency model that the rule design does not specify

`doc/planning/file-enforcer-rust-rewrite.md`:

> “file-enforcer renders, from committed inputs only, `mpm.packages.toml` and `mpm.managers.toml` from the lock”

> “`check` renders every output in memory and compares, needing no prior write”

The same design generates that lock and assigns several transformations of each `Cargo.toml` to different rule kinds.

**Failure scenario:** A check computes a new lock in memory but renders the manifests from the old on-disk lock. A write publishes the new lock with old manifests and needs another run to converge. Independently rendered Cargo transformations can overwrite one another: generic TOML enforcement and catalog synchronization both produce versions of the same file.

**Smallest fix:** Define evaluation against a staged candidate filesystem: downstream rules read proposed upstream content, transformations of one destination have an explicit order, conflicting writers and cycles fail, and validation finishes before publication. The catalog’s “failure writes nothing” requirement must apply before any catalog-related files are published.

## 3. Removing CI’s diff check can recreate the exact false-green condition being fixed

`doc/planning/file-enforcer-rust-rewrite.md`:

> “so CI's `git diff --exit-code` workaround in `.github/workflows/advisory-core.yml:92-105` can go.”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “runs `task-run prepare:pnpm` and then `git diff --exit-code -- rust-catalog.toml 'package/**/Cargo.toml'`”

> “`--check` after a sync ‘could never fail’”

**Failure scenario:** CI retains `prepare:pnpm`, which writes generated files, then substitutes Rust `check` for `git diff`. Stale committed files are repaired before checking, so CI passes with an unreported dirty tree.

There is also a policy distinction to settle: missing ignored outputs, such as the skill mirrors and browser-target cache, are not the same as stale committed outputs on a fresh checkout.

**Smallest fix:** Specify CI ordering explicitly: dependency provisioning may precede checking, but managed committed files must be checked before any regeneration. Define how CI initializes or excludes local-only outputs without first repairing committed drift. Apply the same ordering to pnpr’s new independent check.

## 4. “Built before cutover” is weaker than the distribution prerequisites require

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Everything is built first”

> “one commit switches tasks, CI, and `Dockerfile`, and deletes `package/file-enforcer/core`.”

> “Distribution: published to crates.io and installed by `mpm-shadow sync`”

**Failure scenario:** The local source builds successfully, but a fresh checkout downloads an older published mpm-shadow without `sync`, or attempts to install file-enforcer before its library dependencies are published. The switch commit cannot bootstrap itself from unpublished local builds.

The fallback is also incomplete:

> “`cargo install monochromatic-mpm-shadow` is the other documented path”

A fresh platform without an archive still needs Cargo and a compatible Rust toolchain before it can run the command whose job includes provisioning Rust.

**Smallest fix:** Make cutover conditional on published, downloadable versions of all required crates and archives, installed in dependency order and recorded in the converted resolution and lock. Exercise the exact fresh-machine, CI, and Docker acquisition paths before switching. Document the independent Rust prerequisite for the source-build fallback and select a bootstrap version compatible with the committed lock, rather than relying on an unspecified registry latest.

## 5. Global conversion does not produce all inputs required by the new architecture

`doc/planning/file-enforcer-rust-rewrite.md`:

> “a one-off converter writes `toolchain.plan.jsonc` beside it, keeping comments, at mode 0600, and replaces the `.data.ts`.”

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`:

> “holds `plan.tools.data.ts`, `plan.artifacts.data.ts`, `toolchain.lock.json`, `mpm.packages.toml`, `mpm.managers.toml`”

**Failure scenario:** Only the declaration file is converted. No global `toolchain.resolution.json` exists, or artifact information remains stranded in `plan.artifacts.data.ts`. Later, `mpm-shadow update --global` changes resolution data but no specified file-enforcer invocation regenerates the global lock and manifests.

The proposed per-user CLI is described principally for JetBrains rules, not as the operational owner of this global rendering workflow.

**Smallest fix:** Convert and back up both old data modules, seed the global resolution from existing versions and artifacts without updating them, and define the global file-enforcer configuration and update/render sequence. Use the accepted XDG locations. Include the global-only `cargo-git` and `go-install` routes in migration verification.

## 6. Preserving `bin-paths` JSON does not migrate its executable consumer

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`mpm-shadow bin-paths` prints the same JSON (`exposureDir`, `pathDirs`, `bins`)”

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`:

> “`ENTRY_POINT = "package/file-enforcer/core/src/toolchain/bin-paths.ts"`”

> “run as `node bin-paths.ts` in the language-server sandbox: no network, project read-only, cleared environment, 10 s limit”

**Failure scenario:** The switch deletes the script while the IDE still launches it through Node. Updating the command name alone is insufficient if the cleared sandbox environment cannot locate the provisioned mpm-shadow binary. The inspection fixture also copies and exercises the old entry point.

**Smallest fix:** Add the IDE launcher, sandbox executable-resolution logic, and `package/ide/app/bin/inspect-language-toolchain.ts` fixture to the cutover checklist. Verify the real sandbox invocation, including an unprovisioned checkout and its actionable diagnostic, not merely JSON compatibility.

## 7. The chosen update mechanism has not been shown to preserve the accepted npm rollback policy

`doc/decision/mpm-toolchain-adoption.md`:

> “`toolchain:update` follows the registry's `latest` dist-tag, so a publisher rollback can move a tool down in the update diff”

`doc/planning/file-enforcer-rust-rewrite.md`:

> “npm and PyPI tools: `mpm outdated --output-format json`.”

**Failure scenario:** An installed version is newer than the version currently named by npm’s `latest` tag. If the selected adapter reports only upgrades, it omits the required downgrade. An omitted result is also ambiguous: up-to-date, uninstalled, unsupported, or failed are not interchangeable outcomes.

The supplied research does not establish the relevant adapter behavior through execution. This is an unverified compatibility claim, not proof that mpm cannot support the policy.

**Smallest fix:** Gate adoption of each adapter on disposable tests for rollback, unchanged versions, missing installations, failed manager queries, and installed versions differing from the lock. Verify the accepted release-age policy as well. Retain a native lookup where the adapter cannot express the required semantics.

## 8. Destructive mirror ownership lacks a filesystem-boundary contract

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Target contents equal `.agents/skills`; any other entry is drift, which `check` lists and `write` removes.”

> “the default `write` and `check` touch only repo files.”

**Failure scenario:** `.claude/skills`, a parent directory, or a destination entry is a symlink into another location. An implementation that follows it while enforcing ownership can delete or overwrite files outside the repository. Lexically repo-relative paths do not establish the promised filesystem boundary.

This is unspecified behavior, not evidence that an implementation already follows symlinks.

**Smallest fix:** Define containment and symlink semantics before implementing deletion. Reject destinations escaping the permitted root, validate parent components, and specify treatment of source and destination symlinks. Test root symlinks, parent symlinks, nested symlinks, and traversal paths. Per-user rules need explicitly permitted destination roots rather than an implicit removal of all confinement.

## 9. Dropping the cache lock does not justify dropping writer coordination

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Writes: temp file then rename, only when content differs; no lock and no cache.”

**Failure scenario:** `watch` renders revision A. A concurrent `write` renders and publishes revision B. The older watch run then renames A over B. Per-file atomic replacement prevents torn files, but not stale replacement or mixed generations across dependent files.

Concurrent edits to partially owned Cargo manifests introduce another lost-update case.

**Smallest fix:** Separate the rejected staleness-manifest lock from output-publication coordination. Specify short-lived writer exclusion and detection of changed source/target snapshots before publication. This need not resurrect a persistent cache or the old lock implementation.

This default should not be confirmed as written.

## 10. Required licensing and seed-file behavior is not assigned to concrete rules

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Rust implements a fixed set of rule kinds (concatenate, mirror a directory, set TOML keys where a table exists, append a block if absent, fan out over packages, plus named built-ins).”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “`forbidden-strings.append.local.txt` = comment-only seed created only if absent, mode 0600”

> “write `<pkg>/LICENSES/<id>.txt` copied from root `LICENSES/`, and delete known ids that are no longer required”

The inventory also explicitly leaves root and package licenses among the keep-or-drop questions.

**Failure scenario:** The configuration migrates the named built-ins but omits license outputs, stale-license removal, or creation of the local forbidden-strings seed. A normal overwrite rule destroys user additions to the seed. Conversely, “only when content differs” leaves a byte-correct sensitive file at an incorrect permission mode.

**Smallest fix:** Add a concrete output-to-rule coverage list before acceptance, including root `LICENSE`, package licenses, licensing derivations, owned-file pruning, create-if-absent semantics, and both forbidden-strings files’ modes. Define `check` and `write` over required metadata as well as bytes. Generic “fan out” and “named built-ins” do not close these requirements.

## 11. Catalog ownership is decided, but its required transformation semantics are not

`doc/planning/file-enforcer-rust-rewrite.md`:

> “A new Rust library crate in `package/dependency-catalog` owns parsing and rendering the catalog.”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “literal versions left by `cargo add` are adopted into the catalog as `>=` floors (conflicts reported, named catalogs for exceptions); unused catalog entries pruned; internal path-dependency versions rewritten”

> “All problems collected first, then written, so a failure writes nothing.”

**Failure scenario:** A library that only parses the catalog and renders `[workspace.dependencies]` can leave literal dependencies unenforced, miss global version conflicts, or fail to update the catalog itself. A per-crate implementation can publish earlier changes before discovering a conflict in a later crate.

**Smallest fix:** Explicitly carry the inventory’s catalog semantics into the rule contract: literal adoption, conflict handling, named catalogs, pruning, internal version rewriting, absent-catalog behavior, and the documented Cargo default-features workaround where applicable. Require global validation before catalog-related writes. No preservation of the old implementation is necessary.

## 12. The toolchain split omits named duties that are neither reassigned nor retired

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Ported to Rust: GitHub releases, nodejs.org, crates.io … go, zig, Adoptium, Android, conda.”

The actual passage’s crates.io parenthesis explains that Cargo lacks `outdated`; it does not assign Rust toolchain resolution.

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md` identifies:

> “github 457, rust 403”

and lists:

> “static.rust-lang.org”

**Failure scenario:** Rust stable/nightly/channel resolution has no stated owner. crates.io resolution covers Cargo packages, not Rust toolchain distributions.

The split also assigns:

> “the `.cache/toolchain-bin` exposure farm and postinstall links”

but the research identifies:

> “postinstall commands (`:294`, `search-shadow link .cache/search-shadow-bin`)”

**Failure scenario:** Porting only links leaves search-shadow’s command-driven setup unperformed.

**Smallest fix:** Add Rust toolchain resolution and postinstall command execution explicitly to mpm-shadow’s responsibilities. Preserve their required inputs, ordering, and failure behavior, or obtain an explicit retirement decision.

## 13. The watch default cannot cover the inventory using only current input-file changes

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Every input and output path is known from the config without running anything”

> “rerunning the rules whose declared inputs changed”

The same document accepts package fan-out, directory mirroring, and discovering existing nonempty `.bin` directories.

`doc/research/file-enforcer-rust-rewrite-inventory.md` also requires watch mode to:

> “warns when a generated file is edited.”

**Failure scenario:** A newly created package is absent from the current input set. An empty `.bin` becomes nonempty without an already watched input file changing. Someone adds an unwanted mirror destination entry or edits an output-only file. An input-file-only watcher misses these changes.

**Smallest fix:** Replace the literal static-path claim with a contract for declared selectors expanded against filesystem state. Watch membership changes, missing-path parents, relevant metadata, and managed destinations as well as existing input files. Define warning versus repair behavior for output edits. Verify that ignored directories required by these rules are not excluded by the walker.

## 14. The TOML editor proposal omits existing evidence that the bare crate fails the required preservation contract

`doc/research/file-enforcer-rust-rewrite-meow-status.md`:

> “TOML = `toml_edit` behind a repository wrapper (21 of 21 cases; bare `toml_edit` 12)”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “Splice: comments, key order, unowned keys must survive.”

`doc/planning/file-enforcer-rust-rewrite.md` only names a future vet for:

> “`toml_edit`”

**Failure scenario:** Implementers treat the crate’s general preservation claim as sufficient and use operations corresponding to the previously failing conformance cases. A first-run reformat does not authorize deleting comments or unowned configuration.

**Smallest fix:** Make the existing wrapper requirements and conformance cases explicit acceptance gates for the new TOML editor layer. Likewise, require the XML implementation to preserve comments and embedded JSON correctly and to reject malformed managed XML. “Formatting may change” must not silently weaken those content-preservation requirements.

## 15. The deletion checklist misses consumers explicitly named by the accepted decision

`doc/decision/monorepo-manager-all-rust.md`:

> “the `file-enforcer-perf` fixture, a `prefer-readonly-parameter-type` fixture read, and the `sync:files` tasks need migration.”

> “the `prefer-readonly-parameter-type` read moves to a new fixture package that really depends on `@monochromatic-dev/toml-edit-core`”

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`package/file-enforcer/core` (TypeScript) is deleted at cutover.”

The plan addresses vm-builder and task switching, but does not schedule the fixture replacement or explicit performance-fixture retirement.

**Failure scenario:** A lint/test fixture continues reading a file in the deleted package, or the old performance fixture remains registered with an unsatisfied dependency.

**Smallest fix:** Add both fixture actions to the switch commit. The accepted decision already supplies the required behavior; this does not need another owner choice.

## 16. Clap’s default output paths are not reconciled with “JSON lines always”

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Every line the tool writes is one JSON object (`schemaVersion`, `sequence`, `type`, `code`, `message`, `hint`)”

`doc/research/file-enforcer-rust-rewrite-crates.md`:

> “Default features: `std color help usage error-context suggestions`.”

**Failure scenario:** An ordinary clap help, version, missing-command, or unknown-flag path prints human-readable text rather than the event envelope. Wrapping only rule execution diagnostics does not satisfy the chosen protocol.

**Smallest fix:** Specify that clap’s display and error outcomes are captured and serialized by the CLI’s event layer. Define stream and exit behavior for help/version separately from errors. Verify both stdout and stderr for every parser exit path.

## 17. A generated schema is not automatically the grammar accepted by a handwritten decoder

`doc/planning/file-enforcer-rust-rewrite.md`:

> “The config is decoded by hand per field”

> “The schema is derived from the Rust config types with `schemars`”

> “`file-enforcer.schema.json` is committed and kept current by a file-enforcer rule, so `check` fails when it is stale”

**Failure scenario:** The handwritten decoder and derived schema disagree about field names, tagged values, defaults, unknown fields, or `$schema` itself. Separately, an installed older binary generates its older schema and cannot detect that repository source types now require a different schema.

**Smallest fix:** Require runtime/schema agreement fixtures and explicitly support the instance’s `$schema` metadata. Define schema checking against the binary built from the relevant source revision, not merely whichever published binary sync installed.

The claim of editor support also remains unverified:

> “Unverified: JetBrains support for `$schema` in `.jsonc` files.”

Verify completion and invalid-value diagnostics in the actual supported JetBrains configuration. If association requires explicit mapping, ship or document that mapping rather than assuming `$schema` suffices.

## 18. Browserslist’s accepted data-version difference does not establish consumer compatibility

`doc/planning/file-enforcer-rust-rewrite.md`:

> “resolves `.browserslistrc`'s production section with `browserslist-rs` … and writes the same JSON array for `package/config/rolldown/src/browserslist-targets.ts`.”

`doc/research/file-enforcer-rust-rewrite-inventory.md` requires:

> “config and stats discovery off”

**Failure scenario:** The chosen crate handles environment selection, query syntax, discovery, or emitted browser identifiers differently from the assumed contract. An array can remain valid JSON while becoming unacceptable to the actual rolldown consumer. The accepted browser-data-version difference does not authorize those differences.

**Smallest fix:** Make the vet execute the repository’s real production queries with discovery disabled, then feed the emitted array through the actual rolldown consumer. Test unknown queries and unsupported target identifiers as failures. Record data-version differences separately from syntax and consumer compatibility.

## 19. Dropping scanner compilation rests on an unexecuted invalidation claim

`doc/planning/file-enforcer-rust-rewrite.md`:

> “forbidden-strings compiles from that text when its cache is missing or stale (`package/forbidden-strings/cli/src/runtime_cache/warning_tests.rs:23`).”

The attached inventory and research explicitly report source-only investigation.

**Failure scenario:** Missing-cache compilation works, but an existing cache is not invalidated by the relevant rules-file change. After removing the explicit compile step, scanner behavior stays stale even though file-enforcer generated the correct text.

**Smallest fix:** Before removing the step, exercise the real scanner with an absent cache, an existing cache followed by a rule change, and malformed changed rules. Use a positive control whose verdict must change. If invalidation fails, fix that contract in forbidden-strings rather than silently restoring scanner execution inside file-enforcer.

## Coverage by requested section

- **Contradictions between decisions or research:** findings 7, 13, and 14; finding 1 identifies an unreconciled research assumption.
- **Uncovered outputs or consumers:** findings 5, 6, 10, 11, 12, and 15.
- **Problematic adopted defaults:** findings 8, 9, 13, and 16.
- **Ordering and bootstrap:** findings 1 through 6, plus 17.
- **Behavior insufficient to achieve its stated effect:** findings 2, 3, 9, 13, 16, and 17.
- **Unverified load-bearing claims:** findings 7, 14, and 16 through 19.
