# mise 2026.9.5: `llvm-tools-preview` causes duplicate Rust installs before a task

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so this record is historical for the Mise-era toolchain it investigated.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`).

## Symptom

A task unrelated to Rust starts with two Rust installation attempts:

```console
$ mise run lint:markdown -- README.md
mise by @jdx – installing 1 tool
mise rust@nightly-2026-09-12 info: component llvm-tools is up to date
mise rust@nightly-2026-09-12 ... unchanged
mise ✓ rust@nightly-2026-09-12
mise by @jdx – installing 1 tool
mise rust@nightly-2026-09-12 info: component llvm-tools is up to date
mise rust@nightly-2026-09-12 ... unchanged
mise ✓ rust@nightly-2026-09-12
[//:lint:markdown] $ node package/cli/markdown-lint/src/cli.ts ...
```

The same installation attempts recur on every `mise run` invocation.
Rustup does not download or replace the toolchain in this case:
its own output says every component is up to date and the toolchain is unchanged.
Mise nevertheless reports an install because each installation-decision pass
classifies one requested component as missing and invokes the Rust installer.

The parenthesized `(timeout 20s)` shown by the Pi command transcript is the Bash
execution deadline for that invocation.
It is not a Mise setting or a line emitted by Mise.

## Root cause

The behavior is the combination of a component-name mismatch and two task-startup
installation passes.

### The project requests Rust for every root task

The generated root config declares a floating nightly with a canonical Rust
package name (`mise.toml:94`):

```toml
rust = { version = "nightly", components = "clippy,rust-src,llvm-tools-preview" }
```

The same config enables automatic installation (`mise.toml:221`):

```toml
auto_install = true
```

`lint:markdown` runs Node,
 but Mise provisions the active toolset before task
execution rather than limiting provisioning to executables referenced by the task.
The first pass is in `src/cli/run.rs:807-808` at Mise tag `v2026.9.12`:

```rust
let previewed_tools = if !self.skip_tools {
    let (installed, missing) = ts.install_missing_versions(&mut config, &opts).await?;
```

After task preparation,
 `src/cli/run.rs:895-899` performs the task-specific pass:

```rust
// Step 3: Install tools needed by tasks
if !self.skip_tools {
    self.install_task_tools(&mut config, &tasks, &previewed_tools)
        .await?;
}
```

That second pass collects the full config hierarchy owning each task.
`src/task/task_tool_installer.rs:75-103` says:

```rust
for task in tasks {
    requests.extend(task.tool_args()?.into_iter().filter_map(|tool| tool.tvr));

    // Task execution combines task-level tools with the config hierarchy
    // that owns the task.
    let config_root = task
        .cf(config)
        .map(|task_cf| task_cf.config_root())
        .or_else(|| task.config_root.clone());
    if let Some(config_root) = config_root {
        let config_root = canonicalize_path(&config_root);
        if seen_config_roots.insert(config_root.clone()) {
            requests.extend(
                self.collect_tools_from_dir(&config_root, &task.name)
                    .await?,
            );
        }
    }
}
```

Two passes normally do not produce two installs because the first pass makes the
second pass satisfied.
The component-name mismatch prevents that convergence.

### Rustup accepts one component name but lists another

Rustup accepts both `llvm-tools-preview` and its public alias `llvm-tools`.
The Rust distribution manifest for 2026-09-12 records the relation
(`channel-rust-nightly.toml:27460-27461`,
<https://static.rust-lang.org/dist/2026-09-12/channel-rust-nightly.toml>):

```toml
[renames.llvm-tools]
to = "llvm-tools-preview"
```

Rustup 1.29.1 resolves toolchain-install component names through the manifest.
`src/dist/mod.rs:1152-1170` constructs each requested component and applies any
manifest rename before installation:

```rust
for component in components {
    let mut component =
        Component::new(component.to_string(), Some(toolchain.target.clone()), false);
    if let Some(renamed) = m.rename_component(&component) {
        component = renamed;
    }
```

A request for the alias `llvm-tools` is therefore converted to the package
`llvm-tools-preview`.
A request already using `llvm-tools-preview` names that package directly.

Rustup's list output takes each status name (`src/dist/manifest.rs:490-495`):

```rust
res.push(ComponentStatus {
    component: component.clone(),
    name: self.name(component),
    installed,
    available: component_target_pkg.available(),
});
```

`Manifest::name` uses the pre-rename name.
`src/dist/manifest.rs:91-98` performs the reverse lookup:

```rust
pub(crate) fn short_name<'a>(&'a self, component: &'a Component) -> &'a str {
    if let Some(from) = self.reverse_renames.get(&component.pkg) {
        from
    } else {
        &component.pkg
    }
}
```

`Manifest::name` then appends the target (`src/dist/manifest.rs:69-78`):

```rust
pub(crate) fn name(&self, component: &Component) -> String {
    let pkg = self.short_name(component);
    if let Some(t) = &component.target {
        format!("{pkg}-{t}")
    } else {
        pkg.to_owned()
    }
}
```

That is why the same installed package has these two externally visible names:

```console
$ rustup component add llvm-tools-preview --toolchain nightly-2026-09-12
info: component llvm-tools is up to date
$ rustup component list --installed --toolchain nightly-2026-09-12 | grep llvm-tools
llvm-tools-x86_64-unknown-linux-gnu
```

### Mise compares the names without applying the rename

Mise 2026.9.5 asks rustup for the installed list and keeps each output line
verbatim (`src/plugins/core/rust.rs:211-251`):

```rust
Ok(Some(
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect(),
))
```

Its matcher accepts the requested text exactly or with a host-triple suffix
(`src/plugins/core/rust.rs:1093-1101`).
The matcher entered the project in merged PR #10876,
 commit
`251906b9833b6e9d0b4d2918cc73d6db8d5fded1`:

```rust
fn rustup_component_installed(installed: &BTreeSet<String>, component: &str) -> bool {
    installed.iter().any(|item| {
        item == component
            || item
                .strip_prefix(component)
                .and_then(|suffix| suffix.strip_prefix('-'))
                .is_some_and(rustup_component_suffix_is_host_triple)
    })
}
```

For requested `llvm-tools-preview`,
 neither branch matches installed
`llvm-tools-x86_64-unknown-linux-gnu`.
The satisfaction check therefore logs the precise false negative:

```text
DEBUG rust@nightly-2026-09-12 missing rustup component(s): llvm-tools-preview
```

Mise calls `rustup toolchain install ... --component llvm-tools-preview`.
Rustup resolves it to the already-installed `llvm-tools`,
 reports `unchanged`,
and returns successfully.
Mise then runs its second task-install pass,
 makes the same comparison,
 and
launches the same no-op install again.

This is not caused by the global `rust = "latest"` declaration.
The project declaration has higher precedence and `mise ls rust --json` reports
`nightly-2026-09-12` as active with `requested_version: "nightly"` and the root
`mise.toml` as its source.
The dated version comes from `mise.lock:1087-1089`:

```toml
[[tools.rust]]
version = "nightly-2026-09-12"
backend = "core:rust"
```

Mise tag `v2026.9.12` and current `main` commit
`e9b3c2164dd1cf168eebbc2165b4cf306d2ac0b3` retain the complete matcher shown
in the Root cause section.
Source inspection therefore indicates that updating from 2026.9.5 to 2026.9.12
will not remove this behavior;
 the 2026.9.12 executable was not installed for a
second runtime reproduction.

## Verification

Versions and source revisions:

- Mise executable:
   `2026.9.5 linux-x64 (2026-09-10)`.
- Mise release source:
   tag `v2026.9.12`,
   commit
  `1698dd8ff8308b6e39fee8ce1537ddf93f246a2a`.
- Mise current main checked on 2026-09-21:
   commit
  `e9b3c2164dd1cf168eebbc2165b4cf306d2ac0b3`.
- Rustup executable and source:
   `1.29.1`,
   commit
  `d95a37b6ab92cc1e455d1576039333c97ca3e2c5`.
- Toolchain:
   `nightly-2026-09-12-x86_64-unknown-linux-gnu`.

### Failing catalog

The repository command performs both no-op installs before starting Markdown
lint:

```bash
mise run lint:markdown -- README.md
```

A state-disposable fixture with only the preview spelling reproduces the same
two passes.
The fixture runtime is
[mise-rust-preview-component-reinstall.fixture.mjs](mise-rust-preview-component-reinstall.fixture.mjs).
Its private Mise data directory contains the normal Rust install-marker symlink
to a fixture Cargo bin directory.
The executable named `rustup` returns `default` for `show profile`,
 returns
`llvm-tools-x86_64-unknown-linux-gnu` plus the other required components for
`component list --installed`,
 logs `toolchain install`,
 and performs no
installation.
The fixture's `rustc` executable returns a fixed version string.

Create the state from an empty private directory:

```bash
F="$(mktemp --directory "$HOME/temp/agent/mise-rust-component.XXXXXXXX")"
mkdir --parents "$F"/{cargo/bin,rustup,data/installs/rust,home,preview,alias}
cp doc/troubleshooting/mise-rust-preview-component-reinstall.fixture.mjs \
  "$F/fake-runtime.mjs"
chmod 755 "$F/fake-runtime.mjs"
ln --symbolic "$(command -v node)" "$F/cargo/bin/node"
ln --symbolic "$F/fake-runtime.mjs" "$F/cargo/bin/rustup"
ln --symbolic "$F/fake-runtime.mjs" "$F/cargo/bin/rustc"
ln --symbolic "$F/fake-runtime.mjs" "$F/cargo/bin/cargo"
printf 'version = "12"\ndefault_toolchain = "none"\nprofile = "default"\n' \
  > "$F/rustup/settings.toml"
ln --symbolic "$F/cargo/bin" \
  "$F/data/installs/rust/nightly-2026-09-12"
```

Save this config as `"$F/preview/probe.toml"`:

```toml
# probe.toml
[tools]
rust = { version = "nightly-2026-09-12", components = "clippy,rust-src,llvm-tools-preview" }

[settings]
auto_install = true

[tasks.probe]
run = "true"
```

```console
$ env HOME="$F/home" \
    MISE_DATA_DIR="$F/data" \
    MISE_CACHE_DIR="$F/cache" \
    MISE_STATE_DIR="$F/state" \
    CARGO_HOME="$F/cargo" \
    RUSTUP_HOME="$F/rustup" \
    FAKE_RUST_LOG="$F/preview.log" \
    MISE_OVERRIDE_CONFIG_FILENAMES=probe.toml \
    MISE_OFFLINE=1 \
    mise --verbose --cd "$F/preview" run probe
DEBUG rust@nightly-2026-09-12 missing rustup component(s): llvm-tools-preview
INFO  rust@nightly-2026-09-12 [1/3] install
...
DEBUG rust@nightly-2026-09-12 missing rustup component(s): llvm-tools-preview
INFO  rust@nightly-2026-09-12 [1/3] install
...
[probe] $ true
```

`mise run --dry-run lint:markdown -- README.md` also reports
`rust@nightly-2026-09-12 ⇢ would install`,
 proving the false missing state
without invoking rustup's installer.

### Clean catalog

Copy `preview/probe.toml` to `alias/probe.toml` and change only the component
spelling to rustup's listed alias.
The alias fixture performs no installation:

```toml
rust = { version = "nightly-2026-09-12", components = "clippy,rust-src,llvm-tools" }
```

```console
$ env HOME="$F/home" \
    MISE_DATA_DIR="$F/data" \
    MISE_CACHE_DIR="$F/cache" \
    MISE_STATE_DIR="$F/state" \
    CARGO_HOME="$F/cargo" \
    RUSTUP_HOME="$F/rustup" \
    FAKE_RUST_LOG="$F/alias.log" \
    MISE_OVERRIDE_CONFIG_FILENAMES=probe.toml \
    MISE_OFFLINE=1 \
    mise --verbose --cd "$F/alias" run probe
DEBUG $ .../cargo/bin/rustup show profile
DEBUG $ .../cargo/bin/rustup component list --installed --toolchain nightly-2026-09-12
[probe] $ true
```

Skipping task provisioning also starts Markdown lint without a Rust install:

```console
$ mise run --skip-tools lint:markdown -- README.md
[//:lint:markdown] $ node package/cli/markdown-lint/src/cli.ts ...
```

## Verified workarounds

### Use rustup's listed alias in the project config

Change the managed source `mise.no-env.toml` from `llvm-tools-preview` to
`llvm-tools`,
 then regenerate `mise.toml` with the repository's file-enforcer
workflow.
Do not edit generated `mise.toml` directly.
`file-enforcer.config.ts:719-721` owns that generated destination:

```ts
await overwrite({
  dest: './mise.toml',
  content: `# Generated from mise.no-env.toml by file-enforcer.
${await cat(['./mise.no-env.toml',],)}
${envSection}`,
},);
```

Use this component declaration in `mise.no-env.toml`:

```toml
rust = { version = "nightly", components = "clippy,rust-src,llvm-tools" }
```

The state-disposable A/B probe verified that this is the only changed input and that the
alias form produces no install attempt.
Rustup's manifest maps `llvm-tools` to the same `llvm-tools-preview` package,
 so
the installed tools do not change.

Tradeoff:
this works around Mise's comparison bug by depending on rustup's pre-rename
alias.
It does not repair Mise for another project that declares the canonical
`llvm-tools-preview` name.

### Skip provisioning for a task invocation

```bash
mise run --skip-tools lint:markdown -- \
  doc/planning/pi-advisor-prior-review-context.md \
  doc/troubleshooting/pi-advisor-long-session-provider-failure.md
```

The repository command was verified with `README.md` and emitted no Rust install
output.

Tradeoff:
`--skip-tools` bypasses provisioning for every configured tool,
 not only Rust.
It is safe only when all executables required by the task are already available.
It is a one-invocation bypass,
 not a fix for the project configuration.

## What does not work

- Re-running the same task.
  The install is idempotent in rustup,
   but Mise's comparison remains false,
   so
  every run retries.
- Running `rustup component add llvm-tools-preview` first.
  The component is already installed;
   rustup says `llvm-tools is up to date`
  and continues listing it under `llvm-tools-<host>`.
- Updating only from Mise 2026.9.5 to 2026.9.12.
  The release source and current main still contain the same matcher.
- Treating the two banners as two full downloads.
  Rustup reports the toolchain `unchanged`;
   these are two no-op reconciliation
  attempts.
- Raising the command timeout.
  A longer deadline does not remove the repeated pre-task work.
- Removing the Rust lock entry.
  That can change which nightly is selected,
   but rustup uses the same rename for
  the component,
   so the comparison defect remains.
- Running the upstream matcher test with `--no-default-features`.
  Mise's `mise-sigstore` crate imports TUF-gated symbols,
   so that feature set does
  not compile and cannot test this matcher.
- Building the all-features test in the base `rust:1.95` container.
  That image lacked CMake.
  Later all-features attempts established additional requirements for libclang
  resource headers and OpenSSL development files.
  The verified harness uses a Fedora 44 image containing those dependencies.
- Treating the failed `--no-default-features` attempt as evidence about the
  all-features build.
  That separate feature set requested a system Lua 5.1 package and then failed
  because `mise-sigstore` imports TUF-gated symbols.
  The all-features harness built its bundled Lua source instead.
- Limiting the Mise crate compilation to 4 GiB RAM.
  Rustc was killed with signal 9 while compiling the root crate.
  The verified harness uses an 8 GiB limit and one Cargo build job.

## Prototype

A minimal upstream fix recognizes rustup's `llvm-tools-preview` canonical name
as the `llvm-tools` display name before applying Mise's existing host-suffix
matching.
The prototype also adds the missing unit-test case.
The two-hunk diff against Mise tag `v2026.9.12` is recorded in
[mise-rust-preview-component-reinstall.patch](mise-rust-preview-component-reinstall.patch).

The source-level test runs through Mise's `test` task in a disposable Fedora 44
container limited to 8 GiB RAM and 2 CPUs,
 with one Cargo build job.
The container has no ambient credentials and mounts only the disposable clone,
a Cargo cache,
 the selected Rust toolchain,
 and the Mise executable.

The container recipe and focused task config are recorded as
[mise-rust-preview-component-reinstall.Containerfile](mise-rust-preview-component-reinstall.Containerfile)
and
[mise-rust-preview-component-reinstall.test.toml](mise-rust-preview-component-reinstall.test.toml).
Build the test image and define the disposable inputs:

```bash
podman build --memory=2g \
  --cpu-period=100000 --cpu-quota=200000 --jobs=1 \
  --tag localhost/mise-rust-test:fedora44 \
  --file doc/troubleshooting/mise-rust-preview-component-reinstall.Containerfile \
  doc/troubleshooting
mkdir --parents "$HOME/temp/agent"
chmod 700 "$HOME/temp/agent"
PROTO="$(mktemp --directory "$HOME/temp/agent/mise-prototype.XXXXXXXX")"
gh repo clone jdx/mise "$PROTO" -- --branch v2026.9.12 --depth 1
git -C "$PROTO" remote set-url --push origin DISABLED
TEST_CONFIG="$PWD/doc/troubleshooting/mise-rust-preview-component-reinstall.test.toml"
RED_PATCH="$PWD/doc/troubleshooting/mise-rust-preview-component-reinstall.red.patch"
FIX_PATCH="$PWD/doc/troubleshooting/mise-rust-preview-component-reinstall.patch"
CARGO_CACHE="$HOME/temp/agent/mise-cargo-cache"
MISE_BIN="$(command -v mise)"
TOOLCHAIN="$HOME/.rustup/toolchains/nightly-2026-09-12-x86_64-unknown-linux-gnu"
USER_ID="$(id -u):$(id -g)"
mkdir --parents "$CARGO_CACHE"
```

The same bounded command ran before and after the matcher change:

```bash
podman run --memory=8g --cpus=2 --rm \
  --security-opt label=disable \
  --userns=keep-id --user "$USER_ID" \
  --volume "$PROTO:/work" \
  --volume "$TEST_CONFIG:/work/.prototype-mise.toml:ro" \
  --volume "$CARGO_CACHE:/cargo" \
  --volume "$MISE_BIN:/usr/local/bin/mise:ro" \
  --volume "$TOOLCHAIN:/opt/rust:ro" \
  --workdir /work \
  --env CARGO_HOME=/cargo \
  --env CARGO_BUILD_JOBS=1 \
  --env MISE_OVERRIDE_CONFIG_FILENAMES=.prototype-mise.toml \
  --env PATH=/opt/rust/bin:/usr/local/bin:/usr/bin:/bin \
  localhost/mise-rust-test:fedora44 \
  mise run test
```

Apply the test-only
[mise-rust-preview-component-reinstall.red.patch](mise-rust-preview-component-reinstall.red.patch),
then run the bounded command:

```bash
git -C "$PROTO" apply --unidiff-zero "$RED_PATCH"
```

The red run contained the new assertions but not the matcher fix:

```text
assertion failed: rustup_component_installed(&installed, "llvm-tools-preview")
test result: FAILED. 0 passed; 1 failed; 4529 filtered out
```

Restore the tag and apply the complete fix patch,
 then repeat the bounded command:

```bash
git -C "$PROTO" apply --reverse --unidiff-zero "$RED_PATCH"
git -C "$PROTO" apply --unidiff-zero "$FIX_PATCH"
```

The green run used the recorded complete patch:

```text
test result: ok. 1 passed; 0 failed; 4529 filtered out
```

`git diff --check` also passed.
The regression test separately preserves matching when rustup returns the
canonical `llvm-tools-preview-<host>` form.
It rejects an empty installed set,
 an unrelated installed component,
 and a
misleading `llvm-tools-extra-<host>` prefix for the new alias path.
Existing negative assertions continue rejecting `rustfmt`,
 `rust`,
 and `llvm`.

The prototype deliberately maps only the observed rustup rename.
A generic rule that strips `-preview` from every component was rejected because
the inspected Rustup manifest does not establish that every future component
ending in that suffix will have a corresponding pre-rename alias.
The prototype covers the bare `llvm-tools-preview` spelling used by this
project.
It does not claim support for a raw target-qualified request such as
`llvm-tools-preview-x86_64-unknown-linux-gnu`,
 which was not part of the
reproduction.

## Upstream filing decision

A content search of `.out-of-scope/` for `mise`,
 `rust`,
 `rustup`,
`llvm-tools`,
 and `component` found no exemption matching this incident.
Completed GitHub issue and pull-request searches for `llvm-tools-preview`,
`rust component repeatedly installs`,
 and the exact debug message found no
matching report.
Web and Radius searches of Mise Discussions found related component reports but
no report for this alias mismatch.
Related [Discussion #10327][discussion-10327] and merged pull requests
[#10876][pr-10876] and [#12771][pr-12771] concern reconciling genuinely
missing components and toolchains.
They do not cover a component that is installed but listed under rustup's alias.

The six filing constraints all pass:

1. **Is it really upstream's fault?**
   Yes.
   Rustup accepts both names and lists the pre-rename alias by design.
   Mise compares that output against the unresolved requested name and creates
   the false missing state.
2. **Can upstream fix it?**
   Yes.
   The prototype changes one matcher and extends one existing unit test.
3. **Are they supporting this use case?**
   Yes.
   `docs/lang/rust.md:157-168` documents the `components` option and promises:

   > If the Rust toolchain is already installed,
   > `mise install` will still add any missing configured components.

   Merged PRs #10876 and #12771 added and refined component reconciliation
   specifically for task and install flows.
4. **Would the repository welcome the contribution?**
   Yes,
    with process requirements.
   `docs/contributing.md:6-16` welcomes AI-assisted responses when the poster is
   connected to the question and says to review and verify the response before
   posting.
   `docs/contributing.md:20-38` asks contributors to settle non-obvious direction
   before investing in a pull request.
   `README.md:184-191` routes bug reports to the Troubleshooting and Bug Reports
   Discussion category;
    `.github/ISSUE_TEMPLATE/config.yml:1-5` disables blank
   issues and points to Discussions.
   The draft discloses AI assistance and asks whether the scoped patch fits the
   current design before a PR.
5. **Will they likely fix it?**
   Yes,
    based on current evidence.
   The maintainer merged the directly related reconciliation fixes #10876 and
   #12771.
   On 2026-05-13,
    the maintainer rejected PR #9839 as
   ["too hacky"][pr-9839-comment] and said the bug was not important to resolve.
   The maintainer later merged #10876 on 2026-07-08 and #12771 on 2026-09-04,
   implementing the install-satisfaction capability that #9839 proposed.
   Those later merges are stronger evidence for current direction.
   No current documentation or maintainer response marks alias-aware
   reconciliation as a non-goal.
6. **Has a minimal compatible fix been prototyped?**
   Yes.
   The patch extends the existing matcher and test rather than introducing a new
   reconciliation path.
   The red and green focused test results are recorded in the Prototype section.

Decision:
keep a fileable GitHub Discussion draft,
 but do not post it automatically.
The repository explicitly uses Discussions for bug intake.

## GitHub Discussion draft

~~~md
Title: Rust `llvm-tools-preview` component is reinstalled on every `mise run`

Category: Troubleshooting and Bug Reports

Mise treats an installed Rust `llvm-tools-preview` component as missing on every
task run.
I reproduced this with Mise 2026.9.5 and confirmed the matcher is unchanged in
v2026.9.12 and current main (`e9b3c2164dd1cf168eebbc2165b4cf306d2ac0b3`).

Minimal config:

```toml
[tools]
rust = { version = "nightly-2026-09-12", components = "llvm-tools-preview" }

[settings]
auto_install = true

[tasks.probe]
run = "true"
```

Observed behavior:

```text
DEBUG rust@nightly-2026-09-12 missing rustup component(s): llvm-tools-preview
INFO  rust@nightly-2026-09-12 [1/3] install
info: component llvm-tools is up to date
...
DEBUG rust@nightly-2026-09-12 missing rustup component(s): llvm-tools-preview
INFO  rust@nightly-2026-09-12 [1/3] install
info: component llvm-tools is up to date
```

Expected behavior:
Mise should recognize the installed component and start the task without either
no-op install.

Root cause:
`rustup component list --installed` prints
`llvm-tools-<host>` because the Rust manifest maps the pre-rename alias
`llvm-tools` to the package `llvm-tools-preview`.
Mise's `rustup_component_installed` in `src/plugins/core/rust.rs` only matches the
requested name exactly or with a host suffix.
It therefore cannot match requested `llvm-tools-preview` against listed
`llvm-tools-x86_64-unknown-linux-gnu`.

Changing only the requested spelling to `llvm-tools` makes the task idempotent.
The disposable A/B fixture logged two `rustup toolchain install` calls for
`llvm-tools-preview` and none for `llvm-tools`.

Suggested fix:
normalize the observed `llvm-tools-preview` rename before the existing
host-suffix match.
I prototyped this as a one-function change plus focused unit coverage against
v2026.9.12.
The focused matcher test fails before the source change and passes afterward.

Would a pull request with this scoped rename mapping and regression test fit the
current design,
 or should Mise derive Rustup aliases through a more general mechanism?

AI-assisted disclosure:
An AI coding assistant helped trace the Mise and rustup source, construct the
state-disposable A/B reproduction, and prototype the patch.
~~~

[discussion-10327]: https://github.com/jdx/mise/discussions/10327
[pr-9839-comment]: https://github.com/jdx/mise/pull/9839#discussion_r3235550356
[pr-10876]: https://github.com/jdx/mise/pull/10876
[pr-12771]: https://github.com/jdx/mise/pull/12771
