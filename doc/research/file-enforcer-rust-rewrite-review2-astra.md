# Second outside review of the Rust file-enforcer plan: openai/gpt-6-astra

Review of [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md) after the root round, 2026-10-29,
run through `pi --print --no-tools --thinking xhigh` with the plan, four research reports and two decisions attached.
Findings are unverified model output until checked against the repository.

## Ranked findings

### 1. High, destructive writes: mirror ownership lacks a filesystem-containment contract

From `doc/planning/file-enforcer-rust-rewrite.md`, “Skill mirror owns its targets”:

> Target contents equal `.agents/skills`; any other entry is drift, which `check` lists and `write` removes.

The same file records the first review’s unresolved concern:

> the symlink-containment finding is about what the mirror rule must specify.

A destination root or descendant can become a symlink between runs. Recursive traversal through it could delete files outside the owned mirror. A source symlink can also pull outside content into a generated copy. Owning the destination’s directory entries does not authorize following those entries into other directories.

Smallest fix: specify that destination symlinks are inspected and replaced or removed without following them; reject symlinked destination ancestors. Define source-symlink behavior explicitly, with rejection as the smallest safe implementation. Validate containment again at mutation time, not only while planning.

### 2. High, concurrency: the server does not prevent overwriting concurrent human or tool edits

From `doc/planning/file-enforcer-rust-rewrite.md`, “Current design”:

> one auto-started server per worktree serializes all writes

And “Adopted from the outside reviews”:

> each run plans every rule first against a virtual view of planned outputs

These serialize file-enforcer requests and order their proposed transformations. They do not serialize editors, `mpm-shadow`, catalog update tools, Git checkout, or other processes.

For example, file-enforcer reads a `Cargo.toml`, an editor adds an unrelated dependency, and file-enforcer publishes its previously planned replacement. The edit disappears even though every server request was serialized. Likewise, a lock update during manifest planning can leave generated manifests describing an obsolete lock.

Smallest fix: record the observed content, modes, identities, and relevant directory membership used by each plan. Revalidate them before applying it, then discard and replan on changes. Explicitly document the remaining race with noncooperating writers rather than claiming that server serialization eliminates it.

### 3. High, concurrency: worktree-scoped servers do not serialize shared user outputs

From `doc/planning/file-enforcer-rust-rewrite.md`, “Server follow-ups”:

> Scope: the user chose "One per worktree".

From “Out-of-repo rules”:

> A per-user JSONC config under `~/.config` holds rules that write outside the repo

Two worktrees can run `file-enforcer user write` concurrently through different servers while both modify the same JetBrains XML files. Each server can read the original content and replace the other server’s result. The worktree boundary does not match these outputs’ ownership boundary.

Smallest fix: keep repository servers worktree-scoped, but route user operations through coordination keyed by the canonical per-user config or shared output scope. Specify how `user write` operates when invoked outside any worktree. External IDE writes still need the conflict detection described in finding 2.

### 4. High, lock correctness: offline projection can certify an invalid resolution

From `doc/planning/file-enforcer-rust-rewrite.md`, “Toolchain lock: plan and lock, mpm-shadow writes the lock”:

> an offline `mpm-shadow lock` re-projects plan edits without network.

> file-enforcer `check` verifies that the lock's plan fields equal the plan

Suppose a tool’s request changes from a pin at version `1.0` to `2.0`. Copying the new request into the lock while preserving version `1.0` makes the specified equality check pass, but sync still installs the wrong version. Changing `packageId`, source repository, route, artifact selector, or supported platforms can similarly preserve an unrelated version or checksum.

Smallest fix: define which plan changes preserve an existing resolution. Offline `lock` must reject additions or resolution-affecting changes it cannot satisfy from existing verified data, without changing the lock. Both commands need the same semantic validator, including pin satisfaction and required artifact coverage, not merely declaration-field equality.

### 5. High, concurrency: one lock-writing program is not one lock-writing process

From `doc/planning/file-enforcer-rust-rewrite.md`, “Current design”:

> the only writer of `toolchain.lock.json`

The file-enforcer server does not coordinate independent `mpm-shadow update` and `mpm-shadow lock` processes. Two updates can resolve against different plan snapshots and publish in reverse order. An offline projection can also overwrite a newly resolved lock with a projection based on the previous one. This applies to the global scope as well.

Smallest fix: give mpm-shadow per-scope transaction coordination. A writer must validate that both the plan and lock still match its starting revision before publishing, and atomically replace the complete lock. Define whether a conflicting operation waits, replans, or fails; do not allow silent last-writer-wins behavior.

### 6. High, server lifecycle: startup, stale-socket recovery, and idle shutdown lack an ownership protocol

From `doc/planning/file-enforcer-rust-rewrite.md`, “Server follow-ups”:

> A client that finds no socket starts the server in the background and waits for its socket

> the server exits after an idle period with no clients and no watch subscriber.

Concurrent clients can both observe no socket and start servers. A crashed server can leave a socket pathname behind, so neither client takes the “no socket” branch. Conversely, treating a failed connection as proof of staleness can unlink a live server’s socket during startup or shutdown, allowing another server to become reachable while the first remains alive.

Smallest fix: specify an atomic startup election, readiness handshake, bounded startup wait, and verified stale-owner recovery. A losing starter must connect to the winner. Socket cleanup must verify ownership of the particular socket instance. Clients racing idle shutdown must reconnect without accidentally replaying an already accepted write.

### 7. High, server lifecycle: a pathname hash does not identify a worktree incarnation

From `doc/planning/file-enforcer-rust-rewrite.md`, “Server follow-ups”:

> The socket name derives from a hash of the worktree root

A watch subscriber can keep the server alive while its worktree is removed. If another worktree is created at the same pathname, its client finds the old server. That server may retain the previous config, filesystem handles, or pending plan and apply them to the replacement tree.

Different pathname aliases for the same checkout can produce the opposite problem: separate servers writing the same files.

Smallest fix: canonicalize discovery and bind each server to a verified worktree incarnation, not just its pathname. Check that identity during connection and before mutation. Invalidate pending work and terminate when the identity disappears or changes. Include removal, recreation, rename, and pathname-alias cases in lifecycle tests.

### 8. High, server lifecycle: an upgraded client can silently use an older implementation

From `doc/planning/file-enforcer-rust-rewrite.md`, “Server follow-ups”:

> the server exits after an idle period with no clients and no watch subscriber.

A long-lived watch subscriber prevents idle exit. After `mpm-shadow sync` installs a newer file-enforcer, new CLI invocations can continue talking to the old server. A successful `check` then means “the previous binary considers this clean,” potentially using an obsolete decoder, rule implementation, or bundled browser database.

Smallest fix: require a handshake containing protocol version, implementation/build identity, and worktree identity. A mismatch must cause a coordinated drain and restart or a precise refusal. Merely putting the version in the socket name is insufficient because it permits old and new servers to write concurrently.

### 9. High, bootstrap and lifecycle: headless execution and detached-process completion are undefined

From `doc/planning/file-enforcer-rust-rewrite.md`, “Concurrency: server-client architecture”:

> a filesystem Unix socket under `$XDG_RUNTIME_DIR` with owner-only permissions

And “Server follow-ups”:

> starts the server in the background and waits for its socket

The design has no startup behavior for an unset, unusable, or excessively long `$XDG_RUNTIME_DIR`. Those are conditions a CI or container invocation must handle explicitly. Backgrounding also does not specify whether the server inherits the client’s output pipes, survives job cancellation, or retains a removed fixture as its working directory.

A client disconnect during a write introduces another ambiguity: the caller cannot tell whether the operation completed, and blindly retrying can overlap its previous request.

Smallest fix: define runtime-directory provisioning or a verified private fallback, readiness deadlines, detached descriptor handling, and request-completion semantics. Provide deterministic job-scoped shutdown for CI and container use. The clean-container gate must execute commands with captured streams and exercise cancellation, not merely fetch binaries.

### 10. High, failure recovery: whole-run validation does not cover failure during application

From `doc/planning/file-enforcer-rust-rewrite.md`, “Adopted from the outside reviews”:

> every catalog and manifest problem is collected before any file is replaced, so a failing run writes nothing.

From “Adopted defaults”:

> Writes: temp file then rename

Validation can prevent writes on planning errors. It cannot prevent a partial application when the first rename succeeds and a later rename, deletion, or chmod fails. A server crash can leave the catalog and manifests from different generations. Atomic replacement of individual files is not atomic replacement of the run.

Smallest fix: narrow the no-write guarantee to validation failures and define partial-application recovery explicitly. Report completed and pending operations when possible, discard stale plans after restart, and make the next write reconcile from current inputs. Specify temporary-file modes before publication and safe cleanup of abandoned staging files. Do not imply rollback or whole-run crash atomicity unless those mechanisms are added.

### 11. High, ordering and output ownership: the lock-to-manifest-to-sync sequence remains incomplete

From `doc/planning/file-enforcer-rust-rewrite.md`, “Toolchain lock: plan and lock, mpm-shadow writes the lock”:

> `mpm-shadow update` resolves and writes the whole lock

From “Current design”:

> `mpm.packages.toml` and `mpm.managers.toml` from the lock

The research documents the existing consumer in `doc/research/file-enforcer-rust-rewrite-toolchain-split.md`, “mpm.packages.toml and mpm.managers.toml”:

> `mpm --rustup restore mpm.packages.toml`

After update changes the lock, the manifests remain stale until file-enforcer runs. A subsequent sync can therefore consume the old rustup manifest unless its new behavior is explicitly changed.

The global case remains additionally unassigned: `update --global` changes the global lock, while the live output list names only the Harper policy under `user write`. No adopted rule or command sequence renders the global manifests.

Smallest fix: define the complete repository and global update sequences, with manifest rendering after successful lock publication. Make sync reject stale manifests or derive its operational inputs directly from the lock without writing those manifests. Explicitly install the global rendering rules and connect them to the global task entry points.

### 12. High, CI ordering: checking every output before regeneration fails on a clean checkout

From `doc/planning/file-enforcer-rust-rewrite.md`, “Adopted from the outside reviews”:

> CI checks managed files before any step regenerates them

From “Adopted from the request and repository rules”:

> `check` renders every output in memory and compares, needing no prior write

`doc/research/file-enforcer-rust-rewrite-replacements.md` establishes that the skill mirrors and resolved browserslist file are ignored. The new bin fragment is also ignored by design.

A clean checkout lacks these required outputs. An all-output check reports drift even when every committed generated file is correct. Simply moving the existing CI check earlier therefore does not produce a passing clean-checkout gate.

Smallest fix: distinguish committed outputs from generated local state in the rule contract. CI must check committed outputs before regeneration, then materialize local outputs and verify the complete environment. Preserve all-output checking as the default if desired, but give CI an explicit, tested scope rather than inferring it from whatever happens to exist.

### 13. High, check safety and behavior: the expected-directory contract allows mutation of actual outputs and incomplete diffs

From `doc/planning/file-enforcer-rust-rewrite.md`, “New capabilities”:

> `file-enforcer check --expected-dir <dir>` writes each drifted file's rendered content under `<dir>` at its repo-relative path

With `<dir>` equal to the repository root, the described operation writes expected content over actual files during `check`. An overlapping directory or symlink can cause the same problem indirectly. Reusing a nonempty export directory can also retain stale files from a previous check.

Separately, an expected tree containing only drifted files cannot be compared directly against the whole repository with `diff -ru`: unchanged repository files appear absent from the expected tree. Deletions and mode-only drift have no rendered content to export.

Smallest fix: require a fresh, disjoint export directory and validate resolved containment. Define drift operations explicitly as create, replace, remove, or mode change. Supply a correct per-event diff recipe or paired sparse actual/expected trees; keep mode changes in structured events rather than claiming content diffs represent them.

### 14. Medium, required outputs: catalog mutation and selective license pruning remain outside the consolidated contract

From `doc/planning/file-enforcer-rust-rewrite.md`, “Current design”:

> `[workspace.dependencies]` through the dependency-catalog Rust library

But `doc/research/file-enforcer-rust-rewrite-inventory.md`, “Fan-out over packages”, requires:

> literal versions left by `cargo add` are adopted into the catalog as `>=` floors

> unused catalog entries pruned

Those operations write `rust-catalog.toml`, which the consolidated write-output list omits. Implementing only catalog-to-manifest rendering leaves these workflows uncovered.

The same inventory requires license copies to:

> delete known ids that are no longer required

The replacement research also identifies hand-kept license files. “Every package’s `LICENSES/` copies” does not specify whether obsolete generated texts are removed or unrelated texts preserved.

Smallest fix: add `rust-catalog.toml` to the write and validation contract, including its ordering before manifest rendering. State the exact generated license-file ownership set and selective pruning behavior. Both need fixtures covering deletion as well as creation.

### 15. Medium, required consumer behavior: the new rules list loses the demonstrated path base unless each entry is rooted

From `doc/planning/file-enforcer-rust-rewrite.md`, “Rules file: the scanner reads several files”:

> `FORBIDDEN_STRINGS_RULES` variable takes the same form

From “mise.toml: committed conf.d files”:

> `FORBIDDEN_STRINGS_RULES` becomes the comma-separated list of the two appendix files

The verified old value in `doc/research/file-enforcer-rust-rewrite-mise-confd.md` is:

> `FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"`

If the new value is the literal relative list shown in the owner’s syntax example, running a scanner task from a package directory looks for appendix files there rather than at the repository root. The conf.d equivalence probes exercised the old absolute value, not this new list.

Smallest fix: root each environment-list entry independently with `{{config_root}}`. Define JSONC-array paths as relative to the cli-git config location, independently of invocation cwd. Add a package-directory invocation using the final values, including a missing optional local file.

### 16. Medium, scanner correctness: optional-file and concatenation semantics are not specified

From `doc/planning/file-enforcer-rust-rewrite.md`, “Rules file: the scanner reads several files”:

> The local file is optional; the scanner joins the texts in memory before compiling.

The list representation carries no optionality marker. An implementation must know which missing input is allowed; treating every missing entry as optional can silently disable mandatory rules. Treating every read failure as absence can hide permission or decoding errors.

Literal concatenation also changes meaning when a file lacks a final newline. For example, a trailing comment in the first file can consume the first rule line from the second.

Smallest fix: define the optional-local convention precisely and tolerate only absence for that entry. Reject other read failures and malformed list entries. Insert an explicit newline boundary between texts, retain source-path and line mapping for diagnostics, and test duplicate sections across files. The exact combined text must remain the cache input.

The local file’s previously enforced 0600 mode also needs an owner after seed generation disappears. Assign validation or repair explicitly rather than silently dropping it.

### 17. Medium, cutover gates: publication checks do not bind all changed consumers to compatible binaries

From `doc/planning/file-enforcer-rust-rewrite.md`, “Adopted from the outside reviews”:

> the new `mpm-shadow` and `file-enforcer` are published and fetched by a clean container before the switch

But “Rules file: the scanner reads several files” also changes:

> `forbidden-strings --rules` takes one comma-separated list

> cli-git's config names the files as a JSONC array.

A checkout can obtain the new file-enforcer and mpm-shadow while still installing an older scanner or git-policy binary. The former interprets the comma-separated value as one filename; the latter has not acquired the array-based config contract. The stated two-binary gate does not exclude that combination.

The IDE migration has a related acquisition boundary. The split research says its launcher runs with:

> no network, project read-only, cleared environment, 10 s limit

Changing the entry point to `mpm-shadow bin-paths` does not specify how that sandbox locates the executable before the exposure farm exists.

Smallest fix: gate the actual switch lock against compatible scanner and git-policy releases and execute their new configurations. Exercise the IDE launcher with the sandbox’s real restrictions, a provisioned toolchain, and a missing toolchain. Specify a deterministic executable locator and an actionable missing-bootstrap error.

### 18. Medium, conf.d application: generic temporary files may themselves become live configuration

From `doc/planning/file-enforcer-rust-rewrite.md`, “Adopted defaults”:

> Writes: temp file then rename

From `doc/research/file-enforcer-rust-rewrite-mise-confd.md`, “Which files load”:

> Every pattern loads; there is no stop at the first match.

> Hidden files in conf.d are skipped

A staging file such as `10-bin.local.pending.toml` inside conf.d matches the loader before publication. Mise can read incomplete TOML or load both the temporary and final PATH fragments. A crash can leave that temporary file active indefinitely.

Smallest fix: require hidden staging names that cannot be discovered as config, on the same filesystem as the destination. Apply final permissions before rename. Add a test that invokes mise while staging exists and after an interrupted write; the final-file atomic-rename probe does not exercise temporary-name discovery.

### 19. Medium, unverified update semantics: `mpm outdated` is relative to installed state, not necessarily locked state

From `doc/planning/file-enforcer-rust-rewrite.md`, “Update: mpm where it can, ported lookups elsewhere”:

> `mpm outdated` sees only tools installed on the machine running it

The adopted gates are:

> `mpm outdated` probes for rollback, unchanged, and pinned cases.

A further case is essential: the lock contains `1.0`, but the machine already has registry-latest `2.0`. An installed-tool outdated report can contain no row, yet the lock should move to `2.0`. Reading “not reported outdated” as “the lock is current” produces a false unchanged result.

The opposite divergence, an installed version behind or ahead of the lock, also needs defined handling. The first-version direct lookup does not cover these cases because a lock entry already exists.

Smallest fix: extend the probe matrix to independently vary installed, locked, and registry-selected versions, including absent and failed manager results. Specify how the updater obtains a complete lock-relative result. If the report supplies insufficient evidence, use an authoritative lookup or fail explicitly; do not infer unchanged from an absent row.

### 20. Medium, command semantics: report-only update has no rule preventing auto-sync side effects

From `doc/planning/file-enforcer-rust-rewrite.md`, “Update: mpm where it can, ported lookups elsewhere”:

> `update` syncs the missing tools, then retries the lookup once

From “Adopted from the outside reviews”:

> `mpm-shadow update --check` keeps the report-only mode.

On a machine missing an already-locked npm or PyPI tool, applying the general update recovery path during `--check` installs software and can run postinstall commands. That is not a report-only operation, even if no lockfile is written.

Smallest fix: explicitly exclude `--check` from auto-sync. It must obtain the necessary information without provisioning or return a diagnostic explaining the missing prerequisite. Test that the check leaves installation locations, exposure links, postinstall targets, and lockfiles untouched.

## Contradictions

none.

The apparent conflicts about the resolution file, generated root `mise.toml`, scanner concatenation output, and schema self-enforcement are superseded by the consolidated design and later owner decisions.
