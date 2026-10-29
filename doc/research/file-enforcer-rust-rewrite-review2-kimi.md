# Second outside review of the Rust file-enforcer plan: opencode-go/kimi-k3.1-pro

Review of [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md) after the root round, 2026-10-29,
run through `pi --print --no-tools --thinking xhigh` with the plan, four research reports and two decisions attached (re-run after a connection error).
Findings are unverified model output until checked against the repository.

## 1. Mirror writes can escape through symlinks

**Category:** Required outputs and failure modes.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`, "Skill mirror owns its targets":

> "Target contents equal `.agents/skills`;
> any other entry is drift,
> which `check` lists and `write` removes."

**Failure:** If `.claude/skills/example` is a symlink to a directory outside the worktree, an implementation that follows it while copying or pruning can overwrite or delete unrelated user files. Owning the mirror directory does not imply ownership of a symlink's target.

**Smallest fix:** Specify no-follow semantics using `symlink_metadata`: replace or remove the symlink itself, never traverse it. Apply the same rule to source entries and enforce canonical-path containment for every write and deletion. Add adversarial symlink tests before cutover.

## 2. The new CI gate fails on a clean checkout

**Category:** Contradiction.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "`check` renders every output in memory and compares,
> needing no prior write"

> "CI checks managed files before any step regenerates them"

The outputs include `.claude/skills`, `.factory/skills`, `.config/mise/conf.d/10-bin.local.toml`, and `.browserslistrc.resolved.local.json`. `doc/research/file-enforcer-rust-rewrite-replacements.md` records these as gitignored.

**Failure:** A clean container lacks all four generated outputs. A check covering every output reports drift before any regeneration, so the adopted CI gate cannot pass.

**Smallest fix:** Define a committed-output check scope for CI, while retaining full local checks for machine-local outputs. Exercise that exact command in the clean-container cutover gate.

## 3. Lock updates have no required manifest-rendering step

**Category:** Ordering and silently stale outputs.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "`mpm-shadow update` resolves and writes the whole lock;
> an offline `mpm-shadow lock` re-projects plan edits without network."

> "file-enforcer renders,
> from committed inputs only,
> `mpm.packages.toml` and `mpm.managers.toml` from the lock"

**Failure:** `mpm-shadow update` or `lock` can complete successfully with a new lock while both committed mpm manifests still describe the old lock. The old updater rendered the lock and manifests together; the replacement does not say which command restores that invariant.

**Smallest fix:** Make every successful `update` or `lock` invoke the file-enforcer rendering path after writing the lock. Alternatively, emit an explicit incomplete-run result and require `file-enforcer write`, but then `write` must refuse to proceed while plan and lock differ.

## 4. Global lock and manifest generation still has no owner

**Category:** Required output left uncovered.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md` gives every toolchain command a `--global` form, but its file-enforcer output list names only:

> "through `user write`,
> the JetBrains Harper policy."

The toolchain split also says file-enforcer renders:

> "from committed inputs only"

**Failure:** The global plan and lock live under `~/.config`, not in committed inputs. After `mpm-shadow update --global` or `lock --global`, no specified command regenerates or checks the global `mpm.packages.toml` and `mpm.managers.toml`.

**Smallest fix:** Explicitly assign the global plan, lock, and manifests to `user write` and `user check`, including their paths and modes, and make global update or lock run that rendering step.

## 5. The forbidden-strings cutover omits the two implementations that must change

**Category:** Consumer silently broken at cutover.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "`forbidden-strings --rules` takes one comma-separated list"

> "cli-git's config names the files as a JSONC array."

But the one-switch build list contains only:

> "the Rust file-enforcer,
> the dependency-catalog Rust library,
> the mpm-shadow subcommands,
> and the plan conversion."

`doc/research/file-enforcer-rust-rewrite-replacements.md` records that the scanner currently accepts one path and that cli-git's option and schema also require changes.

**Failure:** At the switch, mise supplies a comma-separated value to a scanner that still treats it as one filename, while cli-git rejects its new array configuration. The listed cutover gates do not require either consumer to be updated.

**Smallest fix:** Add the scanner multi-input change and cli-git array/schema change to the pre-switch implementation list. Gate cutover on real invocations using both final configurations, with the local file present and absent.

## 6. Server startup and stale-socket handling are not serialized

**Category:** Server start race.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`, "Server follow-ups":

> "A client that finds no socket starts the server in the background and waits for its socket"

**Failure:** Concurrent CI clients can all observe no socket and spawn servers. Without a separate election, losers can fail startup or clients can receive transient errors. A naive stale-socket cleanup is worse: one client can unlink another server's live socket after a failed probe, allowing a second server to bind and defeating write serialization.

**Smallest fix:** Add an owner-only startup mutex distinct from the socket. Only its holder may probe, remove a confirmed stale socket, and spawn the server; other clients wait. Define connect-refused, accept-timeout, and retry behavior.

## 7. The server does not coordinate with the other toolchain writer

**Category:** Behavior that cannot achieve its stated effect.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "One server process serializes all writes"

> "`toolchain.plan.jsonc` (people) and `toolchain.lock.json` (mpm-shadow only)."

**Failure:** `file-enforcer write` can read the lock while `mpm-shadow update` replaces it. File-enforcer can then write manifests rendered from the old lock after the new lock has landed. Both commands succeed, leaving a contradictory committed state despite the serialization claim.

**Smallest fix:** Put lock mutation and lock-dependent rendering under the same per-scope coordination. At minimum, use atomic lock writes plus a shared mutex and a lock-hash check before committing rendered outputs.

## 8. A running server can execute an older rule engine than its client

**Category:** Server lifecycle.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "one auto-started server per worktree serializes all writes over an owner-only Unix socket
> (newline-delimited JSON-RPC 2.0) and exits when idle"

**Failure:** A watch subscriber can keep a server alive across a file-enforcer upgrade. The upgraded client then connects to the old server, which performs writes and checks using old rule behavior. JSON-RPC 2.0 does not negotiate application protocol or engine compatibility.

**Smallest fix:** Add a handshake carrying RPC protocol version and the server binary's build or release identity. A mismatch must stop and replace the server before any request is processed.

## 9. Per-user writes are not serialized across worktrees

**Category:** Server scope contradiction.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "One server process serializes all writes,
> and CLI invocations are clients"

> "Scope:
> the user chose 'One per worktree'."

The commands also include `user write`.

**Failure:** Two invocations in different worktrees use different servers but can edit the same JetBrains XML files, and potentially the same global toolchain manifests. Read-modify-write updates can lose each other's changes even with temporary files and rename.

**Smallest fix:** Give `user` commands a separate server scope keyed by the per-user configuration or output root. Retain per-worktree scope only for repository commands.

## 10. `update --check` can mutate the machine despite being report-only

**Category:** Contradiction.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "Require sync first,
> and auto-sync on such failures and retry."

> "`mpm-shadow update --check` keeps the report-only mode."

**Failure:** On a machine missing a locked npm or PyPI tool, `mpm outdated` fails to see it. The auto-sync rule then installs the tool during `update --check`, changing host state in a command documented as report-only.

**Smallest fix:** Disable auto-sync under `--check`. Report the unavailable lookup with a hint naming the required sync command; perform no installs or lock writes.

## 11. `sync` can silently ignore a newer plan

**Category:** Toolchain failure mode.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "an offline `mpm-shadow lock` re-projects plan edits without network"

> "file-enforcer `check` verifies that the lock's plan fields equal the plan"

No equivalent check is assigned to `sync`.

**Failure:** A user edits `toolchain.plan.jsonc` and runs `mpm-shadow sync`. Sync reads the old lock and installs old routes or options successfully. The drift is discovered only if the user separately runs file-enforcer's check.

**Smallest fix:** Make `sync` and `status` validate plan-lock parity before operating. On drift, stop with a JSON error naming `mpm-shadow lock` for declaration-only edits or `mpm-shadow update` when resolution is required.

## 12. A crash can leave a partially applied run with no recovery contract

**Category:** Crash mid-write.

**Passages:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "every catalog and manifest problem is collected before any file is replaced,
> so a failing run writes nothing."

> "Writes:
> temp file then rename,
> only when content differs"

**Failure:** Whole-run validation prevents writes only for validation failures. If the server dies after some renames, earlier outputs contain the new plan and later outputs contain the old one. The design does not specify temporary-file cleanup, client reporting, or automatic recovery.

**Smallest fix:** State that atomicity is per file, not per run; require the next write to reconverge and the next check to detect the partial run. Clean abandoned temporary files on startup. If run-level atomicity is intended, a journal or staged commit protocol is required instead.

## 13. Offline projection can produce a lock that passes the specified check but is unusable

**Category:** Behavior that cannot achieve its stated effect.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "file-enforcer `check` verifies that the lock's plan fields equal the plan,
> and never writes the lock."

**Failure:** If a plan entry changes route or request, offline projection can copy declaration fields while retaining an incompatible old version or artifact set. For example, changing an entry to the archive route can leave no suitable locked artifact. Plan-field equality still passes, and sync later fails or installs the wrong resolution.

**Smallest fix:** Make offline lock and file-enforcer check validate resolution compatibility: request versus version, route-required fields, platform artifacts, and hashes. Reject edits needing network resolution and name `mpm-shadow update`.

## 14. The non-archive bootstrap path still requires an unprovided Rust toolchain

**Category:** Bootstrap.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "`cargo install monochromatic-mpm-shadow` is the other documented path,
> and the only one on platforms without a prebuilt archive."

`doc/research/file-enforcer-rust-rewrite-toolchain-split.md` records that compiling needs rustup, which the sync itself installs.

**Failure:** A fresh machine without a prebuilt archive and without Cargo cannot execute the only documented bootstrap command. Saying that a runbook covers people's machines does not supply the missing rung.

**Smallest fix:** Make the runbook explicitly install and verify rustup before `cargo install`, or provide an archive for every supported bootstrap platform.

## 15. A watch server can outlive its worktree and be reused after path recreation

**Category:** Server lifecycle.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "the server exits after an idle period with no clients and no watch subscriber."

**Failure:** Deleting a worktree while a watch subscriber remains connected need not make the server idle. It can retain deleted paths and watchers indefinitely. If the same path is later recreated, its hash selects the same socket and a new client can reach the stale server.

**Smallest fix:** Monitor worktree-root existence and shut down when it disappears. Remove subscriptions on disconnect, validate root identity during handshake, and include root liveness in stale-server cleanup.

## 16. The virtual-view ordering rule does not define composition versus conflict

**Category:** Ordering.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "transformations of one file compose in a fixed order;
> cycles and conflicting writers fail."

**Failure:** The per-crate key rule and catalog rule both transform `Cargo.toml`, so they must compose. Two whole-file rules targeting the same path must conflict. The design gives neither the ordering source nor the distinction between these cases, so deterministic implementation and validation are unspecified.

**Smallest fix:** Define configuration order or an explicit dependency graph, and classify compatible transformations by rule kind. Reserve conflicting-writer errors for outputs without a declared composition path.

## 17. The schema golden test does not prevent decoder drift

**Category:** Behavior that cannot achieve its stated effect.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "A cargo test in `core-rust` asserts it equals the `schemars` output
> and that the hand-written decoder accepts and rejects the same sample configs."

**Failure:** Adding a field to the Rust schema types while forgetting the decoder only requires regenerating the schema and leaving existing samples unchanged. Both assertions can still pass, while the IDE accepts configurations that the CLI rejects.

**Smallest fix:** Keep the hand-written decoder, but require a coverage-complete differential corpus: every rule variant, field, requiredness case, and unknown-field case must have valid and invalid samples, with a test proving every schema variant is exercised.

## 18. Multi-file rules lack path-resolution and joining contracts

**Category:** Consumer behavior left unspecified.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "`FORBIDDEN_STRINGS_RULES` becomes the comma-separated list of the two appendix files"

> "the scanner joins the texts in memory before compiling."

**Failure:** Bare relative paths fail when a task or hook runs with a package directory as its working directory. Direct concatenation can also merge the final line of one file with the first line of the next when the first lacks a trailing newline, changing comments or section parsing.

**Smallest fix:** Generate each mise entry with `{{config_root}}`, define cli-git array paths relative to its config file, and join in the stated order with an explicit newline separator. Test invocation from a package subdirectory.

## 19. `--expected-dir` can show stale drift from earlier runs

**Category:** Failure mode.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "`file-enforcer check --expected-dir <dir>` writes each drifted file's rendered content under `<dir>`"

**Failure:** If the first run writes `A` and a later run fixes `A` but finds drift in `B`, the old `A` remains in the directory. Standard diff tools then report a file that is no longer drifting.

**Smallest fix:** Require the directory to be absent or empty, or clear only a tool-owned expected tree before each run. Record and enforce that ownership before deleting anything.

## 20. Socket placement assumes an unverified CI and container environment

**Category:** Decision resting on an unverified claim.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`, "Concurrency: server-client architecture":

> "a filesystem Unix socket under `$XDG_RUNTIME_DIR` with owner-only permissions"

**Failure:** A CI or container invocation with `XDG_RUNTIME_DIR` unset has no defined socket location, so every client command can fail before checking or writing anything.

**Smallest fix:** Define an owner-only fallback directory and permission checks, or explicitly require the variable with a diagnostic. Add a clean-container probe with the variable unset to the cutover gates.

## 21. Discovery does not cover glob roots absent when watching starts

**Category:** Watch lifecycle.

**Passage:** `doc/planning/file-enforcer-rust-rewrite.md`:

> "rules declare selectors (globs,
> fan-out roots),
> and the server rediscovers matches when files appear or disappear."

**Failure:** A watcher cannot be established directly on a nonexistent glob root. If a configured fan-out root is created later, no watched directory need report its creation, so the promised rediscovery never occurs.

**Smallest fix:** Specify watching the nearest existing ancestor and rerunning selector discovery on relevant directory creation, deletion, and rename events. Include a test where the entire fan-out root begins absent.
