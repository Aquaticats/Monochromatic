# cli-git 0.2.8 `linked-worktree-only` rejects `mise plugins update`'s forced checkout of plugin clones

## Symptom

`mise self-update` (mise 2026.10.20,
`linux-x64`) reports the binary current and then fails its plugin pass
(observed 2026-10-27 from the host paste and on rerun):

```text
mise ERROR Failed to update plugins: cmake, vfox-cmake, vfox-jdx-vfox-cmake
```

```text
mise WARN  Failed to update plugins: command ["/var/home/user/.local/bin/mise", "plugins", "update"] exited with code 1
```

`mise plugins update` reproduces the failure alone.
Each plugin reports a `git failed: Io(...)` block wrapping the `git` argv mise built,
and each block ends in one finding line emitted by the `git` wrapper:

```text
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward","policyId":"linked-worktree-only","severity":"error","code":"linked-worktree-only/linked-worktree-required","message":"cli-git: git checkout with paths, --force, --patch, --ours or --theirs is rejected in the main git worktree: ...","fix":"none"}
```

The message text continues with the recorded alternatives
(`git diff -- <path>`,
`git show HEAD:<path>`,
`git worktree add <path> <branch>`,
and the `--no-enforce-worktree` hatch).
The rejected argv mise constructs is
`git -c ... -C "$HOME/.local/share/mise/plugins/<name>" ... checkout --force main`,
one invocation per installed plugin.

Triggers:
any `mise self-update` or `mise plugins update` that has at least one installed plugin,
while `git` resolves to a `cli-git-native` build older than 0.2.9.
With no plugins installed the plugin pass is a no-op and both commands exit 0.
The same rejection shape appears for any tool that runs a discarding form
(`checkout --force`,
`reset --hard`,
`stash`,
`clean -fd`)
in a main worktree outside an allowlisted tool cache.

## Root cause

The failure is a version gap in this repository's own `git` wrapper,
not a mise defect.

### Step 1: `mise self-update` updates installed plugins

The installed binary's own help states the default
(mise 2026.10.20,
`mise self-update --help`):

```text
Installed plugins are updated too unless you pass `--no-plugins`.
```

### Step 2: the updater force-checks-out `main` in each plugin clone

The argv quoted in `Symptom` runs once per plugin clone under
`$HOME/.local/share/mise/plugins/`.

### Step 3: the policy rejects the form on pre-0.2.9 wrappers

`git` resolves to the cli-git wrapper:
`.cache/cli-git-bin/git` inside this checkout
(the `_.path` entry file-enforcer keeps first,
activated by `mise activate bash` in `~/.bashrc`),
and `~/.local/bin/git` elsewhere.
The wrapper's `linked-worktree-only` policy rejects discarding forms in a main worktree.
The finding code is
`package/git-policy/cli/src/native/rule_linked_worktree.rs:77`:

```rust
// package/git-policy/cli/src/native/rule_linked_worktree.rs
pub const LINKED_WORKTREE_REQUIRED_CODE: &str = "linked-worktree-required";
```

### Step 4: the exposed wrapper predates the 0.2.9 exemption

The wrapper exempts tool caches,
including mise's data directory,
so plugin clones may be force-checked-out.
The resolution and matching are
`package/git-policy/cli/src/native/effective_target.rs:124`,
`:168`,
`:199`,
and `:230`:

```rust
// package/git-policy/cli/src/native/effective_target.rs
pub fn mise_data_dir(environment: &[(OsString, OsString)], home: Option<&Path>) -> Option<PathBuf> {
    // MISE_DATA_DIR, then XDG_DATA_HOME/mise, then HOME/.local/share/mise
```

```rust
// package/git-policy/cli/src/native/effective_target.rs
pub fn is_allowed_worktree_dir(candidate: &Path, allowed_dirs: &[PathBuf]) -> bool {
    // each allowed directory is canonicalized before the segment-wise starts_with
```

The behavior is pinned by
`package/git-policy/cli/src/native/binary_builtin_tests.rs:234`
(`a_mise_plugin_clone_is_exempt_from_the_linked_worktree_rule`).
That exemption landed in commit `197ae38ff5` (2026-10-25) and first shipped in
`monochromatic-git-policy-cli` 0.2.9.
The wrapper that runs here is older:
`.cache/cli-git-bin/git` and `.cache/toolchain-bin/cli-git-native` both link to
`~/.cargo/bin/cli-git-native`
(9,517,816 bytes,
installed 2026-10-20),
and the `cargo:monochromatic-git-policy-cli` row
(`toolchain.lock.json:733`) pins `0.2.8`,
which predates the exemption.
`~/.local/bin/git` points at the old Mise install of 0.2.9
(9,537,632 bytes),
which carries it,
but the checkout exposure precedes `~/.local/bin` on `PATH`.

The plugins themselves were pre-migration leftovers:
the root `cmake = "latest"` declaration in `[tools]`
(with the vfox cmake plugin set it pulled in)
was removed at the toolchain cutover (`a5cdc3158d`),
and nothing post-migration references them.

### Earlier readings that were wrong

The handover entry of 2026-10-25
(`doc/handover/cli-git-rust-implementation.md`,
`User-boundary check of the exemption`)
concluded the exemption worked on this host because
`git -C ~/.local/share/mise/plugins/cmake checkout --force main`
printed `Already on 'main'`.
That probe ran the 0.2.9 wrapper explicitly,
never reran `mise plugins update`,
and therefore never exercised the wrapper `mise` resolves inside this checkout.
The differential probe in `Verification` separates the two binaries on the same command form.

The path-spelling hypothesis:
this host is rpm-ostree (`/home` is a symlink to `/var/home`),
mise passes `-C /home/user/.local/share/mise/plugins/cmake`,
and `git rev-parse` reports `/var/home/user/...`,
so a naive prefix match could miss.
Disproved:
`is_allowed_worktree_dir` canonicalizes the allowlist side
and the 0.2.9 wrapper passes a fixture reached through a symlinked home spelling
(`Verification`).

## Verification

Versions under test:
the exposed wrapper `~/.cargo/bin/cli-git-native`
(release line 0.2.8,
linked from `.cache/cli-git-bin/git`,
9,517,816 bytes,
installed 2026-10-20),
and `~/.local/bin/git`
which points at
`~/.local/share/mise/installs/cargo-monochromatic-git-policy-cli/0.2.9/bin/cli-git-native`
(crates.io 0.2.9,
published by workflow run `40245672686`,
9,537,632 bytes).
Triggering tool:
mise 2026.10.20 `linux-x64`.

Harness:
one disposable plugin-shaped clone,
then the same discarding form through each wrapper
(`~/temp/agent/guard-probe`,
scratch root per `AGENTS.md`):

```bash
# doc/troubleshooting/mise-plugin-update-linked-worktree-guard.md
mkdir -p "$HOME/temp/agent/guard-probe/home/.local/share/mise/plugins/probe"
/usr/bin/git -C "$HOME/temp/agent/guard-probe/home/.local/share/mise/plugins/probe" init --quiet
/usr/bin/git -C "$HOME/temp/agent/guard-probe/home/.local/share/mise/plugins/probe" \
  -c user.email=probe@example.com -c user.name=probe \
  commit --allow-empty --quiet -m x
```

```bash
# doc/troubleshooting/mise-plugin-update-linked-worktree-guard.md
HOME="$HOME/temp/agent/guard-probe/home" \
  "$REPO/.cache/cli-git-bin/git" \
  -C "$HOME/temp/agent/guard-probe/home/.local/share/mise/plugins/probe" \
  checkout --force HEAD
```

### Catalog that fails

- the exposed wrapper (0.2.8 line) on that fixture:
  the `linked-worktree-only/linked-worktree-required` finding,
  exit 1,
  file content unchanged.
- `mise plugins update` inside this checkout before the workaround
  (2026-10-27):
  the host paste reproduced byte for byte,
  exit 1.

### Catalog that works

- `~/.local/bin/git` (0.2.9) on the same fixture and command:
  silent success,
  exit 0.
- the 0.2.9 wrapper on the same fixture reached through a symlinked home
  (`alias/` pointing at the fixture home):
  silent success,
  exit 0.
- `mise plugins update` and `mise self-update --yes` inside this checkout after the plugin removal
  (2026-10-27):
  `mise is already up to date`,
  exit 0.

## Verified workarounds

### Remove the unused plugins

`mise plugins uninstall --yes cmake vfox-cmake vfox-jdx-vfox-cmake`,
applied here 2026-10-27.
`mise self-update --yes` then exits 0 with `mise is already up to date`.
Tradeoff:
a rollback to Mise-owned tools
(the recorded rollback path in `doc/troubleshooting/mpm-toolchain-migration.md`)
reinstalls plugins on demand,
and any worktree still declaring a plugin-backed tool
(the pre-cutover `tr-*` worktrees carry `cmake = "latest"`)
resurrects them,
which brings this failure back while the exposed wrapper is pre-0.2.9.

### Pass `--no-plugins` to the self-update

`mise self-update --no-plugins` keeps the plugins and skips their update.
Tradeoff:
plugins never update,
and a later `mise plugins update` still errors.

### Move the exposed wrapper to 0.2.9 or newer

Through the repository tools path
(`mise run toolchain:update`,
then `mise run toolchain:sync`).
Tradeoff:
the update flow re-resolves every floating request
(16 tools at the 2026-10-27 `--check`),
so the lock diff carries more than this one row.

### Disable the policy per plugin clone

An untracked `cli-git.config.jsonc` at the clone root:

```json
{ "policies": { "linked-worktree-only": "off" } }
```

Verified on the fixture:
the 0.2.8-line wrapper then forwards `checkout --force HEAD`,
exit 0.
Tradeoff:
it turns the guard off in exactly the clone where a forced checkout can destroy
a human's local plugin edits,
and plugin reinstalls discard the file.

## What does not work

- The `--no-enforce-worktree` hatch:
  mise constructs the `git` argv itself,
  so a tool cannot pass the flag
  (also recorded in `doc/handover/cli-git-rust-implementation.md`).
- Any `CLI_GIT_*` bypass:
  `src/native` exposes no environment switch for this policy.
- Testing the command outside this checkout and concluding it works:
  that resolves `git` to `~/.local/bin/git` (0.2.9),
  which is a different binary than the exposed one
  (the earlier false-green reading).
- Editing mise's plugin updater expectations:
  the forced checkout is its documented clone management,
  and `--no-plugins` is the offered opt-out.

## Upstream filing artifact

Nothing to add upstream.
The audit walks the six constraints
(`doc/troubleshooting/resharp.md` states the gate).

### Upstream filing decision

1. Is it really upstream's fault?
   No.
   The rejection is this repository's wrapper policy on a pre-0.2.9 build,
   and mise behaves as its help documents
   (plugin updates default on,
   `--no-plugins` opts out).
2. Can upstream fix it?
   Not reached.
   For the record:
   mise could skip or soften its forced checkout,
   but nothing in the failure requires that.
3. Are they supporting this use case?
   Not reached.
   mise documents plugin updating and the opt-out;
   a PATH-level git policy wrapper is outside its supported surface.
4. Would the repo welcome our contribution?
   Not reached;
   no filing to welcome.
5. Will they likely fix it?
   Not reached.
6. Have we prototyped a minimal fix?
   Not applicable to upstream.
   The in-repo fix already shipped as 0.2.9
   (commit `197ae38ff5`,
   binary test `a_mise_plugin_clone_is_exempt_from_the_linked_worktree_rule`),
   and this incident is the exposed binary lagging that release.

### Duplicate search

`gh search issues --repo jdx/mise`
ran three queries on 2026-10-27:

- `plugin update checkout force`:
  0 hits.
- `plugins update`:
  5 hits
  (13913 `clone.defaultRemoteName`,
  3517 asdf plugin updates,
  3267 self-update output,
  3315 asdf Rust plugin removal,
  128 plugin autoupdating),
  none about a git wrapper or a forced checkout.
- `checkout --force`:
  5 hits,
  none about plugin clones.

`.out-of-scope/` was checked for a mise or git-policy exemption;
none matches.
The artifact is therefore the explicit "nothing to add" note above,
with no issue or comment draft.

## Related records

- `doc/troubleshooting/mpm-toolchain-migration.md`:
  the migration workarounds,
  the rollback path,
  and the post-cutover leftover-plugin entry.
- `doc/handover/cli-git-rust-implementation.md`:
  the 2026-10-25 exemption decision and release,
  and the 2026-10-27 correction of its false-green boundary check.
- `doc/planning/cli-git-native-cutover.md`:
  how the wrapper reaches `PATH`,
  including the `~/.local/bin/git` link of issue 1796.
- `README.md`:
  the live toolchain provisioning commands.
