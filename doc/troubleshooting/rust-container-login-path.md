# Rust 1.97 container: a login Bash shell removes `rustup` from `PATH`

Status:
 **diagnosed and worked around**.
 This affected disposable parser-foundation cross-target checks,
 not a product build or an installed host toolchain.

## Symptom

In the locally inspected `docker.io/library/rust:1.97-bookworm` image,
 a container command ending in `bash -lc 'rustup target add ... && cargo check ...'` exited 127 with:

```text
bash: line 1: rustup: command not found
```

The same image ran `rustup --version` directly and resolved it under non-login `bash -c`.
 The failure was not evidence that the image omitted Rustup.

## Root cause

The image used in the successful checks had local image ID
 `a0635962c16d5f26400703edd4317175cf9531f3285d632613f9da227f9c71d1`.
 Its inspected image configuration sets
 `PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`.
 The checked upstream source at `rust-lang/docker-rust` commit
 `c2b9aae5dfe895debfc0127f5eb794f98f8baef9`
 also places Cargo's binary directory first (`Dockerfile-debian.template:6-9`):

```dockerfile
# Dockerfile-debian.template:6-9
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=%%RUST-VERSION-VALUE%%
```

That source checkout is corroboration of the image recipe,
 not a hash match to the exact built 1.97 image.
 The decisive installed image evidence is its configuration and `/etc/profile`.
 A login Bash reads the container's `/etc/profile`,
 whose lines 5 through 9 set a fresh root-user path and export it:

```bash
# /etc/profile:5-9 inside docker.io/library/rust:1.97-bookworm
PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
# ...
export PATH
```

The replacement omits `/usr/local/cargo/bin`,
 where a direct non-login invocation found `/usr/local/cargo/bin/rustup`.
 It is the selected `bash -l` startup mode,
 not a missing target installer,
 that changes the executable lookup.

## Verification

- `mise run audit:rust-image-toolchains` under `~/temp/agent` printed
   `rustup 1.29.0` and `rustc 1.97.1` from the image.
- `mise run audit:rust-image-paths` printed
   `non-login PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`,
   then `/usr/local/cargo/bin/rustup`,
   then `login PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`,
   and `/etc/profile:5-9` path assignments.
- The initial `mise run check:windows` used `bash -lc` and failed with the quoted diagnostic.
   Replacing the image digest by the local `1.97-bookworm` tag while retaining `bash -lc`
   reproduced the same failure.
- Changing only the shell to `bash -c` let `rustup target add x86_64-pc-windows-gnu`
   and `cargo check --locked --target x86_64-pc-windows-gnu` both finish with status 0.
   The analogous `aarch64-apple-darwin` cross-check also finished with status 0.
   These compile/type checks do not test native Windows or macOS runtime behavior.

A reader can compare both path modes without modifying a host installation:

```bash
# doc/troubleshooting/rust-container-login-path.md
podman run --memory=2g --cpus=2 --rm docker.io/library/rust:1.97-bookworm bash -c 'command -v rustup'
podman run --memory=2g --cpus=2 --rm docker.io/library/rust:1.97-bookworm bash -lc 'rustup --version'
```

The first command should print `/usr/local/cargo/bin/rustup`;
 the second reproduces the `bash: line 1: rustup: command not found` failure in this inspected image.

## Verified workarounds

- Use non-login `bash -c` when chaining `rustup target add` with Cargo inside this image.
   Both target cross-checks succeeded in the bounded container after this change.
   Tradeoff:
   commands that actually require login-shell profile initialization must supply
that environment by another reviewed route;
   `bash -c` intentionally does not read login profiles.
- Invoke `rustup` directly without a shell when only one command is needed.
   The direct `rustup --version` probe succeeded.
   Tradeoff:
   this cannot sequence target installation and a following Cargo check in one ephemeral container invocation.

## What does not work

- Replacing the pinned image reference with `docker.io/library/rust:1.97-bookworm`
   while keeping `bash -lc`:
   the same `rustup: command not found` diagnostic recurred.
- Treating a direct `rustup --version` success as proof a login shell will find the tool:
   the measured paths differ.

## Upstream filing decision

No matching entry under this repository's `.out-of-scope/` exempts this container-path investigation.
 A search of `rust-lang/docker-rust` issues and pull requests for login-shell Rustup `PATH`
 found no duplicate.
 No upstream report or patch was sent.

1. **Upstream fault**:
   no.
   The image config includes the required tool path;
   this harness chose a login shell that reset the path under `/etc/profile`.
2. **Upstream fixability**:
   the image could override Debian login-profile behavior,
   but no change is needed to use Rustup through its documented image environment.
3. **Supported use case**:
   direct and non-login invocations work;
   the inspected Dockerfile template does not promise that a login shell retains the Dockerfile's `ENV PATH`.
4. **Contribution welcome**:
   not material to this diagnosis because the failure comes from the harness's shell invocation,
   not a defect to submit upstream.
5. **Likely upstream fix**:
   no relevant requested fix or maintainer commitment was found;
   likelihood is not inferred from silence.
6. **Prototype**:
   no upstream patch was attempted because the first constraint fails.
   The consumer-side shell change was executed and verified instead.

There is **nothing to file upstream** about this harness invocation.
