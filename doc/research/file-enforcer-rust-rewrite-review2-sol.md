# Second outside review of the Rust file-enforcer plan: openai/gpt-6.1-sol

Review of [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md) after the root round, 2026-10-29,
run through `pi --print --no-tools --thinking xhigh` with the plan, four research reports and two decisions attached.
Findings are unverified model output until checked against the repository.

## Contradictions

none

## Findings, ranked most severe first

### 1. Mirror ownership lacks a filesystem containment rule

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Target contents equal `.agents/skills`; any other entry is drift, which `check` lists and `write` removes.”

> “the default `write` and `check` touch only repo files.”

Owning a destination does not establish where filesystem operations resolve. If `.claude/skills` is a symlink to the user's personal skills directory, copying or pruning through that path can modify files outside the worktree. A symlink inside either tree can create the same problem at a descendant path. The first review explicitly identified symlink containment, but the adopted fixes do not specify it.

Smallest fix: define a no-follow policy for mirror roots, ancestors, and entries. Replace or reject destination symlinks without traversing them; reject source links escaping the permitted source root. Validate containment before any deletion, and test root and descendant symlinks in disposable fixtures.

### 2. `check --expected-dir` can overwrite the files it is checking

Category: behaviors that cannot achieve their stated effect.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`file-enforcer check --expected-dir <dir>` writes each drifted file's rendered content under `<dir>` at its repo-relative path”

With `--expected-dir .` at the repository root, the documented mapping writes expected content directly over drifted real outputs. A supposedly diagnostic check then repairs or overwrites the worktree. A destination containing symlinked descendants can redirect these writes elsewhere.

Smallest fix: require a dedicated, empty destination; reject canonical overlap with the worktree's managed paths, inputs, and per-user targets. Use no-follow traversal within the destination. Reject unsafe destinations before producing any files.

### 3. Offline `lock` can make an incompatible resolution appear valid

Category: behaviors that cannot achieve their stated effect.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “an offline `mpm-shadow lock` re-projects plan edits without network.”

> “file-enforcer `check` verifies that the lock's plan fields equal the plan”

Suppose a tool is locked at version `1.0.0`, and its plan request changes to pinned `2.0.0`. Re-projecting the request while retaining the old resolution makes declaration equality pass, despite the lock installing the wrong pin. Changing package identity, route, or artifact-selection options can retain an unrelated URL and checksum with the same result.

Smallest fix: define which declaration changes preserve resolution validity. Offline `lock` must reject unresolved additions and incompatible edits rather than copying declarations over stale resolutions. Share that validation with `update`, `sync`, and file-enforcer's lock check.

### 4. The server does not cover the other writers or shared destinations

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “one auto-started server per worktree serializes all writes”

> “`toolchain.lock.json` (mpm-shadow only).”

> “A per-user JSONC config under `~/.config` holds rules that write outside the repo”

The listed server clients are file-enforcer commands, not mpm-shadow. Two `mpm-shadow update` or `lock` processes can therefore overwrite each other's results. File-enforcer can render manifests while mpm-shadow replaces their source lock. Separately, servers for different worktrees can concurrently edit the same JetBrains files or global toolchain manifests.

“One writer” names an executable, not a mutually exclusive process.

Smallest fix: define mutation coordination by actual scope. Serialize mpm-shadow's lock and installation mutations per toolchain scope, render from a stable lock revision, and serialize shared user targets by their canonical user-config scope rather than by worktree.

### 5. The CI ordering gate conflicts with checking ignored generated outputs

Category: required outputs and consumers.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “CI checks managed files before any step regenerates them”

> “`check` renders every output in memory and compares, needing no prior write”

`doc/research/file-enforcer-rust-rewrite-replacements.md`:

> “the skill mirrors are per-clone state, never committed.”

> “`.cache/`, `*.local.*` … are also ignored”

A clean checkout lacks the skill mirrors, browserslist output, and generated bin fragment. A check covering every output must report their absence as drift. Regenerating first would restore the original problem of concealing committed drift. Publishing and fetching the binaries does not create these outputs.

Smallest fix: distinguish committed-output verification from local-state completeness. CI should first check committed outputs and the plan/lock relationship without mutation, then materialize local outputs and verify those separately. Make this distinction explicit in the command or rule-selection contract.

### 6. Server startup has no election or stale-socket protocol

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “A client that finds no socket starts the server in the background and waits for its socket”

Two simultaneous clients can both take this branch. Exclusive socket binding alone can select one server, but the losing starter's behavior is unspecified. Stale-socket recovery is also unspecified: blindly unlinking after a failed connection can remove a newly started server's socket and permit another server to bind, leaving two writers alive.

Idle exit creates another race when a client connects as shutdown begins.

Smallest fix: specify an election and recovery state machine. Losers connect to the winner; only the established endpoint owner removes its socket; stale cleanup is coordinated with startup. Define readiness, draining, bounded startup waits, and request-ID handling for reconnects so a write is not blindly replayed after an ambiguous disconnect.

### 7. A watch subscriber can keep an obsolete server active indefinitely

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “the server exits after an idle period with no clients and no watch subscriber.”

The design has no client/server version handshake. A watch subscriber can keep the old executable running while toolchain sync installs a newer file-enforcer. New CLI invocations then connect to old rule implementations, decoder behavior, or protocol handling. A successful check could certify output using the wrong engine.

Smallest fix: handshake on protocol version and engine/build identity before accepting a request. On mismatch, drain the old server and restart the selected executable, with defined watch reconnection behavior. Merely adding the version to the socket name would allow concurrent servers and would not preserve serialization.

### 8. A path hash is insufficient worktree identity

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “The socket name derives from a hash of the worktree root”

> “the server watches inputs, outputs, and glob roots while a watch client is attached”

The design does not say whether the root is canonicalized. Different spellings of the same directory could create separate servers. Conversely, removing a worktree and creating another at the same pathname produces the same pathname hash. A surviving watch subscriber prevents idle exit, so the old server can remain associated with a replacement checkout.

Smallest fix: canonicalize discovery and bind the server to directory identity and Git worktree identity, not pathname alone. Verify that identity before requests and commits; terminate subscriptions and the server when the original root disappears or is replaced.

### 9. Serialization does not protect hand-written content from stale read-modify-write

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “each run plans every rule against a virtual view of planned outputs, validates the whole run, then writes”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “Splice: comments, key order, unowned keys must survive.”

An editor can add a hand-written dependency to `Cargo.toml` after the server reads it but before the planned replacement is published. Replacing the file from the earlier snapshot loses the edit. Serializing file-enforcer requests does not serialize editors, catalog tools, or mpm-shadow.

Smallest fix: record the identities and contents used for planning, then detect observed input or target changes before publication and abort or re-plan. Explicitly define the remaining race with non-participating writers rather than claiming the server eliminates it. Schedule checks against a coherent snapshot, not across a partially published write run.

### 10. CI and Docker need an explicit non-login server lifecycle

Category: ordering, bootstrap, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “a filesystem Unix socket under `$XDG_RUNTIME_DIR` with owner-only permissions”

> “starts the server in the background and waits for its socket”

There is no specified behavior when `XDG_RUNTIME_DIR` is absent, inaccessible, or unsuitable, as can occur in non-login CI and container executions. Nor does the plan specify detaching the server's inherited standard streams. A background server retaining a runner's output pipes can keep the step open.

A Docker build step can also terminate the server while leaving a socket pathname in the layer for the next step.

Smallest fix: define a secure non-login runtime-directory setup and wire it into CI and Docker. Redirect daemon standard streams, bound readiness waits, and define shutdown at job/build-step boundaries. Exercise successive Docker steps and CI commands, including stale endpoints and interrupted startup.

### 11. The update-to-render sequence remains unowned

Category: ordering and bootstrap.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`mpm-shadow update` resolves and writes the whole lock”

> “file-enforcer renders … `mpm.packages.toml` and `mpm.managers.toml` from the lock”

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`:

> “`mpm.managers.toml` … baking `--version` into `binstall` arguments”

Updating or offline-locking changes the lock but leaves both manifests stale until another command renders them. A subsequent restore or documented manager invocation can use old versions. This also affects bootstrap if sync consumes a manifest whose source lock was just updated, while file-enforcer is not installed yet.

Smallest fix: specify the workflow for both `update` and offline `lock`, including failure between lock publication and rendering. Repoint task wrappers accordingly, preserve `update --check` as report-only, and ensure bootstrap sync either consumes the lock directly or rejects stale manifests without requiring an uninstalled renderer.

### 12. Global manifests and global lock checks still lack an explicit owner

Category: required outputs and consumers.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “Toolchain (mpm-shadow): … each with `--global`”

> “and, through `user write`, the JetBrains Harper policy.”

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`:

> “holds `plan.tools.data.ts`, `plan.artifacts.data.ts`, `toolchain.lock.json`, `mpm.packages.toml`, `mpm.managers.toml`”

The live design explicitly assigns global mpm-shadow commands, but does not assign global manifest rendering or plan/lock checking. Repository commands deliberately exclude home-directory targets, and the consolidated per-user output names only Harper. A global update can therefore leave the existing global manifests permanently stale.

Smallest fix: explicitly include global plan/lock validation and both global manifest projections in the per-user rule configuration, with named write/check entry points and workflow integration. Define how these commands operate when invoked outside any worktree.

### 13. The Rust catalog's writable output and adoption behavior are missing from the consolidated contract

Category: required outputs and consumers.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`[workspace.dependencies]` through the dependency-catalog Rust library”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “literal versions left by `cargo add` are adopted into the catalog as `>=` floors”

> “unused catalog entries pruned; internal path-dependency versions rewritten”

The consolidated write inventory names dependency-table projection, but not writing `rust-catalog.toml` or retaining these adoption and pruning operations. Implementing only catalog-to-manifest rendering leaves a newly added literal dependency without the catalog entry needed to render it.

Smallest fix: explicitly assign the complete retained catalog transformation to the Rust library, including its writable paths, conflict behavior, and internal dependency rewriting. Include those mutations in virtual planning and whole-run validation. Any deliberately removed operation needs an explicit retirement rather than omission.

### 14. Package license copies have no retained pruning contract

Category: required outputs and consumers.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “every package's `LICENSES/` copies”

`doc/research/file-enforcer-rust-rewrite-inventory.md`:

> “delete known ids that are no longer required”

`doc/research/file-enforcer-rust-rewrite-replacements.md`:

> “MIT/Apache/OFL/markdownlint copies in 14 packages are hand-kept.”

“Keep copies” does not specify stale-file detection or deletion ownership. After a manifest license changes, a copy-only implementation can retain obsolete GPL/LGPL texts while reporting clean. Treating the whole directory as an owned mirror instead can delete hand-kept licenses.

Smallest fix: retain an explicit set of tool-owned license filenames, compute required and obsolete members, and preserve everything outside that set. Test license transitions and mixed generated/hand-kept directories.

### 15. Whole-run validation does not define partial-publication recovery

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “every catalog and manifest problem is collected before any file is replaced, so a failing run writes nothing.”

> “temp file then rename”

Validation can prevent writes on validation failure. It cannot make multiple renames transactional. A crash, disk error, permission failure, or deletion failure after some replacements leaves a partially applied run. Mirror pruning makes this more than a question of incomplete file contents.

The temporary-file publication contract also needs to prevent mise from discovering a temporary TOML fragment and prevent sensitive data from briefly receiving broader permissions.

Smallest fix: restrict the no-write guarantee to pre-publication validation failures. Stage content and modes before publication, use temporary names excluded from discovery, and define partial-commit reporting and deterministic next-run recovery. Add interruption tests between replacements, mode changes, and deletions.

### 16. A worktree-scoped server has no specified per-request config context

Category: ordering, lifecycle, and failure modes.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “global `--config` defaulting to `file-enforcer.config.jsonc` at the repo root”

> “One per worktree”

A server started for one config can subsequently receive a request using another config in the same worktree. The design does not say that the request carries its canonical config path, that the config is re-read for every run, or how subscriptions for different configs coexist. Outside watch mode, the server is not watching for config changes at all.

Smallest fix: make resolved config identity and command options explicit request data. Re-read the requested config for every run, namespace subscriptions by config, and do not inherit request semantics from the starter's working directory or environment.

### 17. Optional rules-file loading has no failure policy

Category: behaviors that cannot achieve their stated effect.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`forbidden-strings --rules` takes one comma-separated list”

> “The local file is optional”

The comma-separated syntax does not express which entry is optional. Ignoring every missing entry can silently disable required rules after a typo; requiring every entry breaks clean clones without the local appendix. An existing but unreadable local file must also not be treated as an absent optional file.

Smallest fix: define the optional-entry convention consistently for CLI arguments, environment values, and cli-git's array. Ignore only the designated optional file's absence; fail on missing required files, unreadable files, malformed rules, and empty list components. Define the result when no required rules source is loaded.

### 18. The new rules-list environment value needs both paths anchored

Category: required outputs and consumers.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “`FORBIDDEN_STRINGS_RULES` becomes the comma-separated list of the two appendix files”

`doc/research/file-enforcer-rust-rewrite-mise-confd.md`:

> `FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"`

The verified conf.d fixture uses one absolute, root-anchored path. Replacing it with the literal requested list, `forbidden-strings.append.txt,forbidden-strings.append.local.txt`, makes file loading depend on the scanner's working directory. Package tasks can then fail to find the required appendix or read a same-named file in the wrong directory.

Smallest fix: specify root anchoring for **each** environment-list component and define cli-git array paths relative to its config location. Extend the consumer gate to invoke the scanner from the repository root and a package directory using the actual new environment value.

### 19. Joining rules texts needs a boundary and diagnostic contract

Category: behaviors that cannot achieve their stated effect.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “the scanner joins the texts in memory before compiling.”

`doc/research/file-enforcer-rust-rewrite-replacements.md`:

> “Section names must be unique across the concatenation.”

The design does not specify separators or cross-file section handling. If the first file lacks a final newline, direct concatenation can merge its last rule with the next file's header or first rule. Compilation offsets also cease to identify the original input file unless the loader retains a source map.

Smallest fix: define a newline boundary between files, explicit duplicate-section behavior, and combined-offset-to-source-file mapping. Test missing final newlines, malformed second files, duplicate sections, and cache invalidation through both scanner and cli-git entry points.

### 20. A drift-only expected tree does not support the advertised directory diff

Category: behaviors that cannot achieve their stated effect.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “writes each drifted file's rendered content under `<dir>` at its repo-relative path”

> “so `diff -ru`, `git diff --no-index`, or any diff viewer renders it”

Comparing the worktree with a tree containing only drifted expected files reports unchanged files as deletions. An unwanted mirror entry has no expected content to write, so deletion drift is not represented by the stated output mapping. Reusing a destination can additionally retain files from a previous check and misrepresent the current result.

Smallest fix: produce matching actual and expected comparison trees restricted to the drift set, with deletion represented by absence from the expected tree. Keep metadata-only drift in events because content diff tools do not establish mode correctness. Require fresh destinations and provide a tested command against these trees.

### 21. The IDE migration still assumes an executable the sandbox can discover

Category: decisions resting on unverified claims.

`doc/planning/file-enforcer-rust-rewrite.md`:

> “the IDE launcher … move[s] to `mpm-shadow bin-paths`”

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md`:

> “no network, project read-only, cleared environment, 10 s limit”

The research establishes the old source-path invocation and sandbox restrictions, not a runnable replacement location. A command-name substitution can fail with the cleared environment or an unprovisioned toolchain. Invoking the existing `mpm` shadow link is not equivalent: argv[0] selects pass-through mode.

Smallest fix: specify how the launcher finds the real `mpm-shadow` executable by an explicit path, without relying on ambient PATH or invoking the `mpm` alias. Gate cutover on the actual sandboxed invocation, including provisioned and missing-toolchain fixtures, output parsing, and an actionable failure that does not attempt installation inside the sandbox.
