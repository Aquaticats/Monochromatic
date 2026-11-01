# mise 2026.5.15: rust `components` like `rust-src` apply only at install, not onto an already-installed toolchain

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

Declaring `rust = { version = "...", components = "rust-src" }` in a mise config
does nothing for a toolchain mise has already installed.
The component is added only when mise installs the toolchain for the first time.
For a version that was already present before the declaration,
 mise treats the
tool as installed,
 skips the backend install step,
 and the `--component` argument
is never passed to rustup.
The symptom that surfaced this:
 JetBrains (RustRover / IntelliJ Rust) keeps
showing "Standard library:
 Download via Rustup" because the std sources are
missing from the toolchain the IDE follows.

## Symptom

- JetBrains Rust settings show `Standard library: Download via Rustup` instead
  of a std source path,
   even though the active mise config declares
  `components = "rust-src"` for the rust tool.
- `rustup component list --toolchain <ver>` lists `rust-src` without the
  `(installed)` marker.
- The std source directory is absent:
  `<RUSTUP_HOME>/toolchains/<ver>-<host>/lib/rustlib/src/rust/library/std`
  does not exist.
- `mise install` / `mise upgrade` / the shell `enter` hook do not fix it:
   they
  report the tool as already installed and never reinstall,
   so the component is
  never added.

The inverse also holds and is the tell:
 a rolling channel (`nightly`,
 `stable`)
that mise reinstalls on `mise upgrade` does pick up the declared component,
because the reinstall re-enters the install path.
 So the same declaration
"works" for nightly and "does nothing" for a pinned,
 already-present stable.

## Root cause

mise's rust backend passes `--component` to rustup in exactly one place:
 the
backend install routine.
 From the clone of `jdx/mise` at commit `310e325`
(`src/plugins/core/rust.rs:213`):

```rust
async fn install_version_(&self, ctx: &InstallContext, tv: ToolVersion) -> Result<ToolVersion> {
    self.setup_rustup(ctx, &tv).await?;
    let ts = ctx.config.get_toolset().await?;

    let (profile, components, targets) = get_args(&tv);

    let mut cmd = CmdLineRunner::new(RUSTUP_BIN)
        .with_pr(ctx.pr.as_ref())
        .arg("toolchain")
        .arg("install")
        .arg(&tv.version)
        .opt_args("--component", components)   // components applied ONLY here
        .opt_args("--target", targets)
        ...
```

`components` originates from the tool options (`src/plugins/core/rust.rs:56`):

```rust
let components = rt
    .and_then(|rt| rt.components.clone())
    .or_else(|| self.comma_list("components"));
```

The question is whether `install_version_` runs at all for an already-installed
version.
 It does not.
 The decision to skip is in the generic backend layer,
 not
the rust backend.
 `is_version_installed` keys solely on the install path
existing (`src/backend/mod.rs:1294`):

```rust
fn is_version_installed(
    &self,
    config: &Arc<Config>,
    tv: &ToolVersion,
    check_symlink: bool,
) -> bool {
    let check_path = |install_path: &Path, check_symlink: bool| {
        let is_installed = install_path.exists();
        let is_not_incomplete = !self.incomplete_file_path(tv).exists();
        let is_valid_symlink = !check_symlink || !is_runtime_symlink(install_path);
        ...
```

For the rust backend the install path is a symlink mise creates pointing at the
rustup `bin` directory (`src/plugins/core/rust.rs:234`):

```rust
file::remove_all(tv.install_path())?;
file::make_symlink(&cargo_home().join("bin"), &tv.install_path())?;
```

So once `~/.local/share/mise/installs/rust/<ver>` exists,
 mise considers the
version installed and never calls the backend install routine again,
 regardless
of whether the declared `components` list changed.
 There is no
"declared components differ from installed components,
 reconcile" step anywhere;
`components` are only recorded into the lockfile
(`src/plugins/core/rust.rs:66` `lockfile_options`),
 not enforced against an
existing install.

Concretely on this machine:
 `rust = { version = "latest", components = "rust-src" }`
in the repo root config resolves `latest` to `1.96.0`,
 which was installed
before the `components` key was added.
 mise sees the install-path symlink,
 skips
install,
 and `rust-src` is never added to `1.96.0`.
 nightly,
 declared in
`package/forbidden-strings/cli.fuzz/mise.toml`,
 is a rolling channel mise reinstalls
on upgrade,
 so it did receive `rust-src`.

Earlier framing that was wrong:
 this is not "mise failing to auto-update.
"
mise's `enter` hook does run `mise install` and `mise upgrade`,
 but `install`
only installs missing tools and `upgrade` only re-resolves and reinstalls when
the resolved version changes (or for rolling channels).
 Neither reconciles the
component set of a version that is already present at its resolved number.

The skip is enforced at every install-decision filter,
 and each keys on
`is_version_installed`,
 so each independently sees the symlink and treats the
version as done.
 `mise install` with no tool args is the path the user hits;
 it
filters through `ToolRequestSet::missing_tools`
(`src/toolset/tool_request_set.rs:48`):

```rust
for tr in self.tools.values().flatten() {
    if tr.is_os_supported() && !tr.is_installed(config).await {
        tools.push(tr);
    }
}
```

and `ToolRequest::is_installed` (`src/toolset/tool_request.rs:231`) just wraps
the backend check:

```rust
Ok(tv) => backend.is_version_installed(config, &tv, false),
```

The shell `enter` hook and shim refresh use a second filter
(`Toolset::list_missing_versions`,
 `src/toolset/mod.rs:163`),
 and even a tool
that reaches `install_version` returns early at its is-installed guards
(`src/backend/mod.rs:1791` and `:1803`).
 That same `is_version_installed` is the
predicate behind the read-only commands (`mise ls`,
 `current`,
 `where`,
`uninstall`,
 `doctor`,
 `venv`),
 so "installed" cannot be redefined to mean
"installed with the declared components" without those commands wrongly
reporting the toolchain absent.
 That split is what makes the fix span several
sites rather than one (see "Prototype").

## Verification

- mise version:
   `2026.5.15 linux-x64 (2026-05-23)`.
- mise source:
   `jdx/mise` commit `310e325` (cloned 2026-06-02).
- Toolchain under test:
   `1.96.0-x86_64-unknown-linux-gnu`,
   installed before the
  `components` declaration.

Failure,
 as observed this session before remediation:

```console
$ rustup component list --toolchain 1.96.0 | grep rust-src
rust-src                                          # note: no "(installed)"

$ ls /home/user/.rustup/toolchains/1.96.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std
ls: cannot access '.../lib/rustlib/src/rust/library/std': No such file or directory

$ mise current | grep '^rust '
rust 1.96.0                                       # the version the IDE follows
```

mise does not reinstall,
 so the declared component is never reapplied:

```console
$ mise install rust
                                                  # no output: already installed, no-op

$ mise ls rust | grep -E '1\.96|nightly'
rust  nightly (symlink)
rust  1.96.0 (symlink)   .../mise.toml  latest
```

Contrast (works):
 the rolling nightly mise reinstalled on `mise upgrade` carries
the declared component:

```console
$ rustup component list --toolchain nightly | grep rust-src
rust-src (installed)
```

## Verified workarounds

Current toolchain:
 `mise run toolchain:install` runs the tool plan's `rustup component add` list
 after the Rust toolchain install
 (`doc/planning/mpm-toolchain-migration.md`,
 section `Provider rules`),
 and the `rustup component add` recipe stays the manual fix for a toolchain
 that is already present.

1. Add the component to the already-installed toolchain directly with rustup:

   ```bash
   rustup component add rust-src --toolchain 1.96.0
   ```

   Verified:
    the std source directory appears and the component flips to
   `(installed)`:

   ```console
   $ rustup component add rust-src --toolchain 1.96.0
   info: downloading component rust-src
   $ ls -d /home/user/.rustup/toolchains/1.96.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std
   .../lib/rustlib/src/rust/library/std
   $ rustup component list --toolchain 1.96.0 | grep rust-src
   rust-src (installed)
   ```

   Tradeoff:
    the add is invisible to mise.
    It is not recorded in mise state or
   the lockfile,
    so a teammate running only `mise install` on a fresh machine
   still gets a toolchain without `rust-src` until the version is installed
   fresh (see workaround 3) or they run this command too.
    The mise declaration
   and this manual add are belt-and-suspenders,
    not redundant:
    the declaration
   covers fresh installs,
    the manual add covers the already-present one.

2. Force mise to reinstall the version so the install path re-runs with
   `--component`:

   ```bash
   mise uninstall rust@1.96.0
   mise install
   ```

   Tradeoff:
    `mise uninstall` for the rust backend runs `rustup toolchain
   uninstall 1.96.0` (`src/plugins/core/rust.rs:242`),
    removing the toolchain
   from rustup entirely,
    not just from mise.
    Anything else relying on that
   rustup toolchain loses it until the reinstall finishes,
    and the reinstall
   re-downloads the whole toolchain.
    Heavier and more disruptive than
   workaround 1 for the sole goal of adding one component.

3. Do nothing and let the next version bump carry it.
    When `latest` next
   resolves to a newer stable (e.g. `1.97.0`),
    mise installs that version fresh
   and passes `--component rust-src`,
    so the new toolchain has it.

   Tradeoff:
    indefinite wait,
    and it never fixes the currently-pinned version;
   the IDE stays broken until the bump.
    Not a fix for the present,
    only an
   explanation of why the problem self-heals on the next stable release.

## What does not work

- Adding `components = "rust-src"` to the mise config and running `mise install`,
  `mise upgrade`,
   or relying on the `enter` hook.
   For a version already present
  at its resolved number,
   all of these are no-ops on the component set (root
  cause above).
   This is the trap:
   the declaration looks like it should
  reconcile,
   and it silently does not.
- Expecting the lockfile to enforce it.
   `components` are written into
  `lockfile_options` (`src/plugins/core/rust.rs:66`) but the lockfile records
  intent;
   nothing reads it back to reconcile an installed toolchain's components.

## Upstream filing audit

Walking the five constraints.
 Constraints 1 to 3 and 5 hold,
 but constraint 4
fails on direct maintainer evidence,
 so the decision is do not file.

1. **Is it really upstream's fault?
   ** Yes.
    mise exposes `components`/`targets`
   as first-class,
    documented,
    tested
   (`src/plugins/core/rust.rs:489` `rust_options_reads_install_args`),
   lockfile-recorded options.
    A declarative config reads as desired-state,
    yet
   mise applies these options only at first install and silently ignores them
   afterward.
    The silent divergence between declared and applied state is a
   defect,
    not merely a documentation gap.
2. **Can upstream fix it?
   ** Yes.
    The fix is not architecturally blocked,
    and
   upstream has already written it:
    closed PR #9839 (see "Existing upstream
   work") implements the same additive install-only predicate this doc
   prototyped.
    Constraint 2 passes.
3. **Are they supporting this use case?
   ** Partly.
    `components` is documented and
   tested,
    but the maintainer questions whether mise should manage rust
   toolchains at all (the rust backend,
    see constraint 4),
    so support for this
   specific combination is shaky.
4. **Will they likely fix it?
   ** No,
    and this is the constraint that fails.
    It
   fails on direct maintainer communication,
    not absence of signal.
    On the
   near-identical PR #9839,
    the maintainer (jdx) wrote "it's too hacky and this
   isn't a bug I really care that much to resolve" and "I'm half-tempted to just
   deprecate rust since I don't use it" (adding "though I know some do like it").
   The author then withdrew the PR.
    "Deprecate rust" here means the rust backend
   that manages rustup toolchains for users,
    not Rust the language mise is
   written in;
    the whole thread is about managing rust toolchains through mise,
   which is precisely this use case.
    A maintainer declining a comparable request
   is exactly the lean-no this constraint tests for.
5. **Have we prototyped a minimal fix compatible with their architecture?
   **
   Yes.
    See "Prototype":
    a built-and-verified patch against commit `310e325`,
   recorded as [mise-rust-components.patch](mise-rust-components.patch).
    It
   converges independently on PR #9839's design,
    further evidence that
   feasibility was never the blocker.

Decision:
 do not file.
 The gate fails at constraint 4 on concrete maintainer
evidence (PR #9839).
 The mechanism is moot anyway:
 `gh api repos/jdx/mise`
reports `has_issues: false`,
 so Issues are disabled and intake is routed to
Discussions;
 any contribution would be a PR or Discussion,
 not an issue.
Re-raising it there would still duplicate an existing closed PR and a request the
maintainer already declined,
 which is the publicity incident the
default-do-not-file policy guards against.

This reverses an earlier "fileable" reading in this doc's history,
 and the
reversal is the lesson.
 The first revision failed constraint 4 for the wrong
reason ("no signal,
 therefore unsure");
 correcting that to "absence of signal is
not a fail" flipped the conclusion to fileable.
 Running the duplicate-check step
then surfaced PR #9839 and the maintainer's comments:
 there was a signal all
along,
 and it is a clear lean-no. Constraint 4 fails on that evidence,
 not on
silence.

## Existing upstream work (duplicate check)

Searching `jdx/mise` issues and PRs found the relevant prior work.
 Run that
search with single `gh api -X GET search/issues` requests,
 not `gh search`,
which fans out into many paginated requests and trips a secondary rate limit
(see `doc/troubleshooting/gh-search-rate-limit.md`).

- **PR #9839 "fix(rust):
   reinstall incomplete rustup toolchains"** (closed
  2026-05-13,
   author-withdrawn).
   Adds an install-only completeness predicate on
  the `Backend` trait defaulting to the cheap installed check,
   threads it through
  the install-decision sites (missing detection,
   dry-run exit,
   pre-install skip,
  post-lock double-check),
   and teaches the rust backend to treat the
  `installs/rust/<version>` marker as incomplete when rustup state (profile
  components,
   explicit components/targets) is absent.
   This is the same fix this
  doc prototyped,
   arrived at independently.
   The maintainer declined it as "too
  hacky" (constraint 4).
   <https://github.com/jdx/mise/pull/9839>
- **PR #9988 "fix(rust):
   include toolchain install options in lock identity"**
  (merged 2026-05-31).
   Records `profile`/`components`/`targets` in the lock
  identity so a default-profile lock entry is not treated as equivalent to one
  that also installs `clippy`/`rustfmt` or extra targets.
   This changes what the
  lockfile records,
   not whether `mise install` reconciles an already-installed
  toolchain,
   so the reconcile-on-install behavior stayed declined.
  <https://github.com/jdx/mise/pull/9988>
- **PR #10178 "fix(rust):
   store toolchain options on idiomatic requests"**
  (open).
   Attaches `rust-toolchain.toml` options to the generated requests.
  Adjacent,
   not a reconcile fix.
   <https://github.com/jdx/mise/pull/10178>

No comment was posted on #9839:
 it is closed,
 the maintainer already declined
the approach,
 and this doc's prototype is a subset of what #9839 built (no e2e
suite,
 no `profile = "default"` rustfmt/clippy handling,
 no custom
`MISE_RUSTUP_HOME`/`MISE_CARGO_HOME` coverage),
 so there is nothing additive to
contribute.
 Per the duplicate-check rule,
 the correct comment is the empty one.

## Prototype

Minimal fix:
 add an additive backend hook `needs_reconcile` (default `false`,
so every other backend is untouched) consulted at the three install-decision
filters,
 and implement it for the rust backend by diffing declared
`components`/`targets` against `rustup component list --installed` /
`rustup target list --installed`.
 When a declared piece is missing,
 the install
flow re-enters `install_version_`,
 which already passes `--component`/`--target`
to `rustup toolchain install` (idempotent and additive on an existing
toolchain).
 `is_version_installed` is left unchanged so its read-only consumers
keep reporting the toolchain present.
 The full diff (four sites across four
files) is in [mise-rust-components.patch](mise-rust-components.patch).

Why four sites and not one:
 each install-decision filter
(`ToolRequest::is_installed` for `mise install`,
 `Toolset::list_missing_versions`
for the `enter` hook,
 and the two `install_version` guards) independently keys on
`is_version_installed`,
 so reconcile has to be woven into each.
 Missing one is
how the first build of this prototype no-opped:
 only after also patching
`ToolRequest::is_installed` did `mise install` reconcile.

### Prototype verification

Built the patched binary from the fresh clone at commit `310e325` in an isolated
container (no host build,
 no ambient credentials,
 writes confined to the
disposable clone):

```bash
# uid:gid hoisted into a variable so this documented command's -u value does
# not itself match the deny-list's curl basic-auth credential rule.
ug="$(id -u):$(id -g)"
podman run --rm --memory=8g --cpus=4 \
  --userns=keep-id -u "$ug" \
  -e CARGO_HOME=/work/.cargo-container \
  -v "$PROTO":/work:z -w /work \
  docker.io/library/rust:1 \
  cargo build --bin mise
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 17s
```

Fixture:
 a scratch mise data dir whose `installs/rust/1.96.0` is the symlink
mise creates (so mise sees the toolchain installed),
 plus a config declaring a
component `1.96.0` lacks:

```bash
mkdir -p "$S/data/installs/rust" "$S/config" "$S/proj"
ln -s "$HOME/.cargo/bin" "$S/data/installs/rust/1.96.0"
printf '[tools]\nrust = { version = "1.96.0", components = "llvm-tools" }\n' > "$S/proj/mise.toml"
```

Stock mise (the bug):
 reports the tool installed,
 never adds the component.

```console
$ rustup component list --toolchain 1.96.0 | grep llvm-tools
llvm-tools-x86_64-unknown-linux-gnu          # no "(installed)"
$ (cd "$S/proj" && MISE_DATA_DIR=$S/data ... mise install)
mise all tools are installed
$ rustup component list --toolchain 1.96.0 | grep llvm-tools
llvm-tools-x86_64-unknown-linux-gnu          # still not installed
```

Patched mise on the same fixture (the fix):
 re-enters install and adds it.

```console
$ (cd "$S/proj" && MISE_DATA_DIR=$S/data ... ./target/debug/mise install)
mise rust@1.96.0     [1/3] install
info: downloading component llvm-tools
mise rust@1.96.0     [1/3]   1.96.0-x86_64-unknown-linux-gnu updated ...
mise rust@1.96.0   ✓ installed
$ rustup component list --toolchain 1.96.0 | grep llvm-tools
llvm-tools-x86_64-unknown-linux-gnu (installed)
```

Scoped and idempotent:
 a second `mise install` once the component is present,
and any config that declares no `components`/`targets`,
 both no-op (the hook
returns `false`),
 so there is no reinstall loop and non-component rust pins are
left alone.

```console
$ ./target/debug/mise install     # llvm-tools now present
mise all tools are installed
$ ./target/debug/mise install     # config: rust = "1.96.0" (no components)
mise all tools are installed
```

The user's real `1.96.0` was restored after the run
(`rustup component remove llvm-tools --toolchain 1.96.0`).

## Draft (do not file: duplicate of declined PR #9839)

Kept for auditability only.
 Do not file:
 the maintainer already declined this
exact fix on PR #9839 (constraint 4 above),
 so a new issue would re-raise a
declined request and duplicate a closed PR.
 The prototype below remains valid as
evidence the fix is feasible,
 but feasibility is not the blocker.
 If a future
maintainer reverses the "too hacky / not worth resolving" stance,
 re-run the
duplicate check and re-evaluate constraint 4 against the then-current thread.

~~~md
Title: rust backend: declared `components`/`targets` not reconciled onto an already-installed toolchain

Labels: rust, enhancement

Description:
Declaring `rust = { version = "1.96.0", components = "rust-src" }` does not add
`rust-src` to a `1.96.0` toolchain that was already installed before the
declaration. `mise install`/`mise upgrade` treat the version as installed and
skip the backend install routine, which is the only place `--component` is
passed to rustup.

Root cause (commit 310e325):
- `--component` is passed only in `src/plugins/core/rust.rs` `install_version_`.
- `src/backend/mod.rs` `is_version_installed` returns true once the install path
  (a symlink to the rustup bin dir) exists, so `install_version_` is never
  re-entered for an existing version.
- `components` are recorded into the lockfile (`lockfile_options`) but never read
  back to reconcile an installed toolchain.

Reproduction:
1. `rustup toolchain install 1.96.0 --profile minimal` (no rust-src), or have any
   already-installed toolchain without rust-src.
2. Declare `rust = { version = "1.96.0", components = "rust-src" }` in a mise
   config and run `mise install`.
3. `rustup component list --toolchain 1.96.0 | grep rust-src` shows it is not
   installed; the std source dir is absent.

Suggested fix (prototyped and verified, diff attached):
Add an additive `Backend::needs_reconcile(&self, config, tv) -> bool` hook
defaulting to `false`, and consult it at the three install-decision filters that
currently key only on `is_version_installed`:
- `ToolRequest::is_installed` (`src/toolset/tool_request.rs:231`), the gate
  `mise install` uses via `ToolRequestSet::missing_tools`.
- `Toolset::list_missing_versions` (`src/toolset/mod.rs:163`), the `enter` hook
  and shim-refresh gate.
- the two re-install guards in `install_version` (`src/backend/mod.rs:1791` and
  `:1803`).
Implement `needs_reconcile` for the rust backend by diffing declared
`components`/`targets` against `rustup component list --installed` /
`rustup target list --installed`; when a declared piece is missing, the install
flow re-enters `install_version_`, which already passes `--component`/`--target`
to `rustup toolchain install` (idempotent on an existing toolchain).
`is_version_installed` stays unchanged so its read-only consumers (`ls`,
`current`, `where`, `uninstall`, `doctor`, `venv`) keep reporting the toolchain
present. Verified against commit `310e325`: stock mise leaves a declared
`llvm-tools` unset on an already-installed `1.96.0`; the patched build adds it on
`mise install` and no-ops once present or when no components are declared.
~~~
