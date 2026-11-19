# Cargo Binstall 1.27.0 resolves the wrong crate for Maki or rejects a bare repository URL

Investigation record. Codex reproduced both diagnostics and verified the prebuilt install routes.
Owner review is open.

## Symptoms

- `cargo binstall maki` reports `no binaries specified nor inferred` and exits with status 86.
- `cargo binstall https://github.com/tontinton/maki` reports
  `https://github.com/tontinton/maki is not found` and exits with status 76.
- After a successful install to `~/.cargo/bin/maki`, Bash can report `maki: command not found`
  if `~/.cargo/bin` is missing from the shell's `PATH`.

## Root cause

The positional argument is a crate name with an optional `@version`. The parser stores the entire
argument as the name when there is no `@`. The bare URL therefore becomes the registry lookup key.
See `crates/binstalk/src/ops/resolve/crate_name.rs:30`. [Crate name parser][crate-name]

```rust
// crates/binstalk/src/ops/resolve/crate_name.rs
CrateName {
    name: s.into(),
    version_req: None,
}
```

With no manifest override, the resolver passes that name to its registry client.
See `crates/binstalk/src/ops/resolve.rs:503`. [Registry lookup branch][registry-lookup]

```rust
// crates/binstalk/src/ops/resolve.rs
let (manifest, pubtime) = Box::pin(opts.registry.fetch_crate_matched(
    client,
    &name,
    version_req,
))
.await?;
```

The registry lookup for `maki` succeeds and returns `maki 0.1.0`, a library for automating Cargo releases.
Its published manifest names `https://github.com/foresterre/maki` as its repository.
The archive contains `src/lib.rs` and has no executable target.
The resolver reads `manifest.bin` and raises `UnspecifiedBinaries` when that list is empty.
See `crates/binstalk/src/ops/resolve.rs:539`. [Registry index][index] · [Binary selection][binary-selection]

```rust
// crates/binstalk/src/ops/resolve.rs
if binaries.is_empty() {
    Err(BinstallError::UnspecifiedBinaries)
}
```

The Git manifest override takes the repository URL separately.
It clones that repository and resolves the positional crate name against the clone.
See `crates/bin/src/args.rs:122`. [Git flag declaration][git-flag]

```rust
// crates/bin/src/args.rs
#[clap(
    help_heading = "Overrides",
    long,
    conflicts_with("manifest_path"),
    value_name = "URL"
)]
pub(crate) git: Option<binstalk::registry::GitUrl>,
```

## Verification

Codex tested Cargo Binstall `1.27.0` on x86-64 Linux.
The crates.io archive matched SHA-256 `8aebbe0de91e05d15c89008ca6323d053aa484b86722a4b81b4986187062834c`.
The Maki GitHub release was `v0.7.2`. [Registry index][index] · [Maki release][release]

The following dry runs isolate the registry lookups. Run them from a disposable directory.

```sh
# doc/troubleshooting/cargo-binstall-maki.md
cargo binstall --dry-run --disable-strategies compile,quick-install maki
cargo binstall --dry-run --disable-strategies compile,quick-install https://github.com/tontinton/maki
```

- The crate-name form returned `no binaries specified nor inferred`.
- The bare URL form returned `https://github.com/tontinton/maki is not found`.
- The Git override and Maki's installer each installed an executable in a disposable directory.
  Both executables printed `maki 0.7.2` and had the same SHA-256 digest.

## Verified workarounds

Use the Git override and the release download strategy to keep the installation in Cargo's bin directory.
The manifest comes from the repository's default branch. If its version has no published binary for your
target yet, the command fails at release resolution.

```sh
# doc/troubleshooting/cargo-binstall-maki.md
cargo binstall --git https://github.com/tontinton/maki.git --strategies crate-meta-data maki
maki --version
```

Maki's installer selects the latest published release and the archive for your machine.
On Linux it defaults to `~/.local/bin`, which needs to be on `PATH` for `maki` to resolve.
The verification run set `MAKI_INSTALL_DIR` to a disposable directory. [Maki installation instructions][install]

```sh
# doc/troubleshooting/cargo-binstall-maki.md
curl -fsSL https://maki.sh/install.sh -o maki-install.sh
# Inspect the downloaded script before running it.
sh maki-install.sh
maki --version
```

## Bash cannot find the installed binary

Cargo adds its own bin directory to its external subcommand search when the directory is absent
from `PATH`. That lets `cargo binstall` work in a shell that cannot resolve `maki` by name.
See `src/bin/cargo/main.rs:341`. [Cargo subcommand source][cargo-subcommand-source]

```rust
// src/bin/cargo/main.rs
if !path_dirs.iter().any(|p| p == &home_bin) {
    path_dirs.insert(0, home_bin);
}
```

If Cargo Binstall reported installing to `~/.cargo/bin/maki`, add that directory to the current Bash
session and check which binary runs. [Repository prerequisites](../../README.md#prerequisites)

```sh
# doc/troubleshooting/cargo-binstall-maki.md
export PATH="$HOME/.cargo/bin:$PATH"
command -v maki
maki --version
```

With a clean Bash process and `PATH=/usr/bin:/bin`, Codex reproduced exit status 127 for `maki`.
Under that same `PATH`, `cargo binstall -V` printed `1.27.0`. After the export, `command -v maki`
resolved to the installed binary and `maki --version` printed `maki 0.7.2`.

If `environment.d` already declares the Cargo directory and it disappears at Plasma login, see
[Plasma's PATH overwrite](plasma-overwrites-environment-d-path.md).

## What does not work

- Adding release-download options to a registry lookup still selects the library's manifest.
  Cargo Binstall needs an executable target before it can download an archive.
- Passing the repository as the positional argument sends it to crate-name resolution.
  The URL belongs in `--git`, with `maki` as the positional crate name.

## Upstream filing decision

Nothing to add upstream. The `.out-of-scope/` search found no applicable exemption.
Tracker searches for `git url`, `no binaries specified nor inferred`, and `binstall` found the original
Git override pull request (Cargo Binstall #1162) and related Git revision requests (#1165 and #2358).

1.  The tested failures follow Cargo Binstall's positional argument grammar and the registry's crate contents.
2.  Cargo Binstall already provides a Git manifest override. The plain-name lookup is appropriate for the registry.
3.  Git installs are supported through `--git <URL>` and a positional crate name.
4.  Maki's `CONTRIBUTING.md` asks contributors to search for duplicates and disclose AI use in pull requests.
    The Cargo Binstall checkout had no contribution file or templates matching the searched names.
5.  Cargo Binstall merged Git support in #1162. The linked issue discussions concern Git revision selection.
6.  Both install routes worked through existing interfaces, so this investigation needed no upstream patch.

[crate-name]: https://github.com/cargo-bins/cargo-binstall/blob/5885714d0a40a14b2535c6ffc051fd8e49aa9665/crates/binstalk/src/ops/resolve/crate_name.rs#L27-L41
[registry-lookup]: https://github.com/cargo-bins/cargo-binstall/blob/5885714d0a40a14b2535c6ffc051fd8e49aa9665/crates/binstalk/src/ops/resolve.rs?plain=1#L503-L512
[binary-selection]: https://github.com/cargo-bins/cargo-binstall/blob/5885714d0a40a14b2535c6ffc051fd8e49aa9665/crates/binstalk/src/ops/resolve.rs?plain=1#L539-L569
[git-flag]: https://github.com/cargo-bins/cargo-binstall/blob/5885714d0a40a14b2535c6ffc051fd8e49aa9665/crates/bin/src/args.rs?plain=1#L122-L135
[install]: https://github.com/tontinton/maki/blob/339b61a5518ad9d98cc9cba6f3dbf44e48ce93cc/README.md#L101-L124
[index]: https://index.crates.io/ma/ki/maki#:~:text=%220.1.0%22
[release]: https://github.com/tontinton/maki/releases/tag/v0.7.2#:~:text=v0.7.2
[cargo-subcommand-source]: https://raw.githubusercontent.com/rust-lang/cargo/ede6cefea9731cb595942338824c2f9a610b8189/src/bin/cargo/main.rs#:~:text=path_dirs.insert(0%2C%20home_bin)
