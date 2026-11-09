# mise 2026.7.0 rejects `mise run` in a detached worktree whose copied config path is untrusted

## Symptom

A guard-failure proof creates a detached Git worktree from a repository whose root `mise.toml` is already trusted.
Running a package task in that disposable worktree fails before the task starts:

```text
mise ERROR error parsing config file: <disposable-worktree>/mise.toml
mise ERROR Config files in <disposable-worktree>/mise.toml are not trusted.
mise ERROR Trust them with `mise trust`.
```

`<disposable-worktree>` is a privacy-normalized substitution for the private temporary path.
The original repository remains trusted and runs the same task.
The detached worktree does not inherit trust merely because it has the same commit and config content.

## Root cause

Verified version:
`mise 2026.7.0 linux-x64 (2026-07-02)`.
The corresponding source tag is `v2026.7.0`,
commit `857b73f6a6b39a3bc90c44119a1e86ee11bd7273`.

`src/errors.rs:24-28` defines the exact terminal diagnostic:

```rust
#[error(
    "Config files in {} are not trusted.\nTrust them with `mise trust`. See https://mise.en.dev/cli/trust.html for more information.",
    display_path(.0)
)]
UntrustedConfig(PathBuf),
```

`src/config/config_file/mod.rs:322-343` checks the copied config path and its newly derived trust root.
When neither path is trusted and no attended prompt accepts trust,
it returns `UntrustedConfig` before task parsing proceeds:

```rust
let config_root = config_trust_root(path);
// ...
if is_path_trusted(path) || cmd == "trust" || cfg!(test) {
    return Ok(());
}
// ...
Err(UntrustedConfig(path.into()))?
```

`src/config/config_file/mod.rs:347-374` canonicalizes the new path and compares it with configured trusted roots:

```rust
let canonicalized_path = match path.canonicalize() {
    Ok(p) => p,
    Err(err) => {
        debug!("trust canonicalize: {err}");
        return false;
    }
};
// ...
for p in settings.trusted_config_paths() {
    if canonicalized_path.starts_with(p) {
        add_trusted(canonicalized_path.to_path_buf());
        return true;
    }
}
```

The copied worktree has a different canonical path,
so trust attached to the original repository path does not match it.
This is expected path-based security behavior,
not evidence that Git worktree copying changed `mise.toml`.

`settings.toml:2635-2644` defines the process environment bridge:

```toml
[trusted_config_paths]
default = []
# ...
env = "MISE_TRUSTED_CONFIG_PATHS"
global_only = true
parse_env = "list_by_os_path_separator"
```

The setting accepts path-separator-delimited roots.
`src/config/config_file/mod.rs:410-415` shows that a matching configured root adds the canonical path only to the
current process's in-memory set:

```rust
static IS_TRUSTED: Lazy<Mutex<HashSet<PathBuf>>> = Lazy::new(|| Mutex::new(HashSet::new()));

fn add_trusted(path: PathBuf) {
    IS_TRUSTED.lock().unwrap().insert(path);
}
```

The earlier hypothesis that copied ignored dependencies caused the first failure was incomplete.
A separate probe with dependencies available still emitted the untrusted-config diagnostic before `rolldown` launched.
After process-scoped trust was supplied,
a probe without dependency links advanced past trust and then failed with
`rolldown: command not found`.
That second diagnostic proved trust and dependency availability are separate preconditions.

## Verification

Access date:
2026-09-01.

The failing probe used a detached worktree at the Candidate M implementation commit:

```bash
git -C /path/to/repository worktree add --detach /private/disposable/worktree HEAD
cd /private/disposable/worktree
mise run //package/translation-repair/<package>:build:js:node
```

It produced the quoted untrusted-config diagnostic and exited nonzero before build execution.

The positive probe linked the already-installed dependency directories into the disposable worktree and scoped trust to
that exact root:

```bash
MISE_TRUSTED_CONFIG_PATHS=/private/disposable/worktree \
  mise run //package/translation-repair/<package>:build:js:node
```

The build completed with exit code `0`.
The environment override was supplied only to the command invocation and made no persistent trust-store change.
Descendant processes can inherit it unless the caller explicitly scrubs their environment.

A repository-wrapper positive control created a worktree without `--no-worktree-copy` and reported 340 copied ignored
entries.
The guarded probe added `--no-worktree-copy` and measured zero bytes in both NUL-delimited pre-link inventories:

```bash
git ls-files --others --exclude-standard -z
git ls-files --others --ignored --exclude-standard -z
```

### Working catalog

- Original repository path already trusted by the user.
- Fresh detached worktree with dependencies available and
  `MISE_TRUSTED_CONFIG_PATHS` set to that exact worktree root for the child process.
- Multiple config files under that exact temporary root,
  because `trusted_config_paths` intentionally covers descendants.

### Failing catalog

- Fresh detached worktree without process-scoped trust:
  untrusted-config failure before the task starts.
- Fresh detached worktree with trust but without installed dependency links:
  task starts and then emits `rolldown: command not found`.
- Reusing trust for the original repository path:
  copied worktree remains untrusted because its canonical path differs.

## Verified workarounds

### Scope trust to the fresh disposable worktree process

Set `MISE_TRUSTED_CONFIG_PATHS` only in the command environment passed to `mise`,
with its value equal to the freshly created worktree root.
Before setting it,
create the worktree through this repository's `--no-worktree-copy` wrapper flag,
then require both NUL-delimited Git inventories to be empty:
ordinary untracked paths and ignored paths.
Add explicit dependency links only after that check.
Preserve command status,
signal,
and launch-error identity separately so interruption cannot masquerade as a failed guard test.

Tradeoff:
every config below that temporary root is trusted for that child process.
The root must therefore be freshly created from an already trusted commit,
private to the current user,
created without automatic ignored-state synchronization,
proved free of every untracked and ignored path,
and removed after the run.
Descendants can inherit the variable,
so launch no unrelated command from that environment and scrub it before invoking untrusted children.
Never set this bridge to `/` or another broad shared ancestor.

### Link existing dependencies into the disposable worktree

Link only the repository's already-installed dependency directories when Git worktree setup did not copy them.
Do not install dependencies or run package-manager mutation inside the proof fixture.

Tradeoff:
the proof shares dependency identity with the original repository rather than proving a clean install.
Ordinary symlinks do not enforce read-only access,
so only verified build and test commands that do not install or mutate dependencies may run in the fixture.
It tests source mutations and built behavior,
not dependency reproducibility.

## Enforcing the shared-dependency write boundary on 2026-09-11

The reviewed-fidelity guard proof used the same installed mise version and source revision.
It created a tracked-only worktree from the main worktree,
verified both empty NUL-delimited inventories,
and added explicit dependency links.
The fixture's `node_modules` parents were real directories,
so newly created caches did not automatically redirect into the source worktree.
No dependency installation was requested.

Current `mise exec --help` and `mise run --help` expose `--allow-write`,
`--allow-env`,
`--deny-net` and `--no-deps`.
The deciding source is `src/sandbox/landlock.rs`:

```rust
// mise v2026.7.0, src/sandbox/landlock.rs
} else if deny_write {
    ruleset = add_read_rule(ruleset, "/", read_access)?;
    ruleset = add_read_rule(ruleset, "/tmp", full_access)?;
    ruleset = add_read_rule(ruleset, "/dev", full_access)?;
    for path in &config.allow_write {
        ruleset = add_path_rule(ruleset, path, full_access)?;
    }
}
```

`src/sandbox/mod.rs` makes `allow_write` imply denied writes elsewhere,
canonicalizes configured paths,
and applies Linux restrictions before child execution.
`src/sandbox/seccomp.rs` blocks creation of `AF_INET` and `AF_INET6` sockets,
while allowing Unix sockets.
Only `--deny-net` was used;
no per-host allow-list behavior is inferred from this proof.

The source-backed launch also set process-local `MISE_AUTO_INSTALL=false`
and `MISE_TASK_RUN_AUTO_INSTALL=false`.
`settings.toml` binds those settings to the installation gates.
`src/cli/run.rs:400` and `src/cli/exec.rs:172` show that `--no-deps` skips automatic dependency preparation.
The allow-env list retained task plumbing,
`usage_*` arguments and the owned `TMPDIR`,
not provider credentials or the encrypted-environment cache key.

The actual probe verified:

- An unsandboxed owned control file was writable,
  and an unsandboxed loopback listener could bind.
- A file inside the allowed fixture remained writable under sandboxing.
- A write to the owned control outside the fixture was refused.
- The same outward write through a symlink inside the fixture was refused.
- The control file's contents remained unchanged.
- An INET listener failed with `EPERM` under sandboxing.
- Credential variables were absent from the filtered child environment.
- The real package build and targeted compiled-artifact tests completed under those same restrictions.

Evidence is in `~/temp/agent/fidelity-guard-sandbox-20260911.out`,
`fidelity-guard-sandbox-control-20260911.json`
and `fidelity-guard-baseline-20260911.out`.
Guard-removal builds and tests subsequently ran through this boundary too.
These were finite source-guard experiments,
not resource stress tests.

Tradeoffs:
`/tmp` and `/dev` remain writable,
reads are not restricted by this write-only policy,
Unix sockets remain available,
and there are no CPU or memory limits.
This is a verified shared-dependency write boundary and INET-creation restriction,
not a general confidential or resource-limited sandbox.
Use the repository's bounded container machinery for hazardous or resource-heavy mutations.

A separate caller error used package task `format`,
which did not exist in the `mise.toml` of `package/module/translation-repair`
(split on 2026-10-18 into the packages under `package/translation-repair`).
Mise emitted `no task //package/module/translation-repair:format found` before any formatter ran;
the same mistake now names `//package/translation-repair/<package>:format`.
The declared task `format:oxlint` was then invoked successfully.
That task-name error was not a trust or sandbox failure.

## What does not work

- Assuming trust follows equal `mise.toml` content to another canonical path.
- Running `mise run` before the new worktree has process-scoped trust.
- Treating `rolldown: command not found` as another trust failure.
  It occurs only after trust succeeds and means dependencies are absent.
- Calling `mise trust` from the disposable proof harness.
  The command is a valid remediation for a deliberately persistent,
  user-controlled worktree,
  but its persistent trust-state tradeoff is wrong for an ephemeral proof fixture.
- Setting a broad trusted root or disabling trust globally.
  That weakens the boundary beyond the one fresh fixture.
- Treating any nonzero or signal-terminated child as guard detection.
  Only a normal successful rebuild followed by a normal nonzero targeted-test exit qualifies.

## Upstream filing decision

`.out-of-scope/` contains no mise-specific exemption.
GitHub issue searches for `detached worktree trust config`,
`MISE_TRUSTED_CONFIG_PATHS`,
and `worktree trust` returned no results because the `jdx/mise` repository has disabled issues.
No upstream issue or comment should be filed because the observed behavior matches documented trust semantics.

1.  Upstream fault:
    **no**.
    A copied config at a new canonical path is intentionally untrusted.
2.  Upstream can fix:
    **not applicable**.
    Automatically inheriting trust by equal content would weaken the path boundary.
3.  Supported use case:
    **yes**.
    Current documentation explicitly provides `MISE_TRUSTED_CONFIG_PATHS` for trusted roots.
4.  Contribution welcome:
    **not investigated further** because there is no upstream defect to report.
5.  Likely upstream fix:
    **not applicable** because no behavior change is requested.
6.  Minimal compatible prototype:
    **implemented at the consumer boundary** by disabling ignored-state worktree copying,
    requiring empty exact NUL inventories,
    then passing exact temporary root through command-scoped,
    non-persistent environment.

## Upstream filing artifact

Nothing to file or comment upstream.
The local workaround uses mise's documented configuration surface and preserves the intended trust boundary.
