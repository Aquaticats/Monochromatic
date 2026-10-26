# Installing llamacpp-rocm globally with mise

This runbook installs the lemonade-sdk `llamacpp-rocm` prebuild,
a llama.
cpp distribution with the AMD ROCm runtime libraries bundled in,
as a global mise tool through `~/.config/mise/config.toml`.
The default target here is the `gfx110X` GPU family (RDNA3:
 RX 7600/7700/7800/7900 class);
the asset name is the only thing that changes for another target.

What this proves:
 after the procedure,
 `llama-server` runs from the global tool install
and resolves its co-located ROCm `.so` libraries on its own,
so the bundle is usable outside the Monochromatic checkout.

Bridges tried,
 so this is not an unconsidered handoff:
the whole install is scripted through the mise CLI,
there is no UI to click and nothing is done by hand inside the install.
Two non-obvious obstacles are baked into the config rather than left for the operator:

- The monorepo `mise.toml` sets `disable_backends` (it lists `github` among others),
  so a `mise install` run with the working directory inside the checkout fails with
  `backend github is disabled by disable_backends`.
  The install must run from a directory outside the checkout,
   where that setting does not apply.
  Toolchain migration note:
   the monorepo's `disable_backends` retires with the `[tools]` removal
   (`doc/decision/mpm-toolchain-adoption.md`, 2026-10-26),
   so this obstacle holds only while the repository config still sets it.
- The release `.zip` stores every entry as mode `0644` (no executable bit),
  and mise preserves the archive's stored modes when it extracts,
  so the launchers land non-executable.
  A tool-level `postinstall` hook restores the executable bit on the launchers only,
  leaving the `.so` libraries non-executable;
  the launchers find those libraries through their `$ORIGIN` runpath.

## Setup

Status:
TODO

Prerequisites for a fresh machine:

- Linux on x86_64.
- mise installed and activated in the shell.
- About 3 GB of free disk:
   the download is roughly 600 MB and the extracted install is about 2.3 GB.
- Network access to `github.com` and `release-assets.githubusercontent.com`.
- For actual inference,
   an AMD GPU in the `gfx110X` family with a working ROCm driver stack.
   The install and the verification below do not need the GPU;
   they exercise only startup and dynamic linking.
- A working directory that is **not** inside the Monochromatic checkout,
   because the checkout's `mise.toml` disables the `github` backend
   (the `disable_backends` setting retires with the toolchain migration,
   `doc/decision/mpm-toolchain-adoption.md`, 2026-10-26).

## Steps

Status:
TODO

1.  Move to a directory outside the checkout and confirm the `github` backend is enabled there.

    ```sh
    cd ~
    mise settings get disable_backends
    ```

    Expected:
     the command prints `[]`.
    If it prints a list containing `github`,
     the working directory is still inside the monorepo;
     move further out and rerun.

2.  Add the tool entry to the global config `~/.config/mise/config.toml` under its `[tools]` table.
    Use exactly this entry (single-quote the `postinstall` value so the shell expands
    `$MISE_TOOL_INSTALL_PATH` at hook time,
     and the inner double quotes survive intact):

    ```toml
    # ~/.config/mise/config.toml
    "github:lemonade-sdk/llamacpp-rocm" = { version = "latest", asset_pattern = "llama-*-ubuntu-rocm-gfx110X-x64.zip", postinstall = 'chmod +x "$MISE_TOOL_INSTALL_PATH"/llama* "$MISE_TOOL_INSTALL_PATH"/rpc-server' }
    ```

    Expected:
     `mise settings get disable_backends` still prints `[]` from `~`,
     and the file saves without a TOML parse error.

3.  Install the tool from the same outside-the-checkout directory.

    ```sh
    cd ~
    mise install "github:lemonade-sdk/llamacpp-rocm"
    ```

    Expected:
     the final two lines are
     `mise github:lemonade-sdk/llamacpp-rocm@<tag> ✓ running custom postinstall hook`
     and
     `mise github:lemonade-sdk/llamacpp-rocm@<tag> ✓ installed`,
     where `<tag>` is the resolved release (for example `b1292`).
    The download line names `llama-b1292-ubuntu-rocm-gfx110X-x64.zip`,
     confirming the `gfx110X` Linux asset was selected.

## What to check

Status:
TODO

1.  Resolve the install path.

    ```sh
    mise where "github:lemonade-sdk/llamacpp-rocm"
    ```

    Expected:
     a path under `~/.local/share/mise/installs/github-lemonade-sdk-llamacpp-rocm/<tag>`.

2.  Confirm the launcher is executable and a bundled library is not.

    ```sh
    P="$(mise where 'github:lemonade-sdk/llamacpp-rocm')"
    ls -l "$P"/llama-server "$P"/libggml-hip.so
    ```

    Expected:
     `llama-server` shows `-rwxr-xr-x` and `libggml-hip.so` shows `-rw-r--r--`.

3.  Run the server binary through mise and confirm it links its bundled libraries.

    ```sh
    mise exec "github:lemonade-sdk/llamacpp-rocm" -- llama-server --version
    ```

    Expected:
     a first line starting with `version:` (for example `version: 1 (a66d505)`),
     a second line starting with `built with Clang`,
     and exit code 0.
    A failure here prints `error while loading shared libraries`,
     which would mean the `$ORIGIN` runpath did not resolve the `.so` files.

4.  Confirm mise exposes the command name.

    ```sh
    mise which llama-server
    ```

    Expected:
     a path ending in `/llama-server` under the install tree.
    Inside the Monochromatic checkout this command instead resolves to the repo's own
     `llama.cpp` tool (the CPU build),
     because the `github` backend is disabled there;
     that is expected and not a fault.

## Restore

Status:
TODO

1.  Uninstall the tool.

    ```sh
    cd ~
    mise uninstall "github:lemonade-sdk/llamacpp-rocm"
    ```

    Expected:
     mise reports the version removed,
     and `mise where "github:lemonade-sdk/llamacpp-rocm"` then exits non-zero.

2.  Remove the tool entry and its comment block from `~/.config/mise/config.toml`,
    then confirm the config still parses.

    ```sh
    mise ls --global
    ```

    Expected:
     the listing no longer includes `github:lemonade-sdk/llamacpp-rocm`,
     and the command exits 0.

## Targeting a different GPU or pinning a release

To build for another AMD target,
 change `gfx110X` in `asset_pattern` to one of the published variants:
 `gfx103X`,
 `gfx110X`,
 `gfx1150`,
 `gfx1151`,
 `gfx120X`,
 `gfx908`,
 `gfx90a`.
List the current assets with
 `gh release view --repo lemonade-sdk/llamacpp-rocm --json assets --jq '.assets[].name'`.

`version = "latest"` tracks the newest release,
 so `mise upgrade "github:lemonade-sdk/llamacpp-rocm"` moves to it.
To freeze a build,
 replace `latest` with the exact tag (for example `version = "b1292"`).
