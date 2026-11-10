# rustup proxies auto-install a `+toolchain` into whatever `RUSTUP_HOME` is in effect, so one unprefixed step of a chained remote command writes 1.4 GB to the machine's default home

## Symptom

A measurement run that is supposed to keep every byte off a machine's internal
disk puts a full Rust toolchain there anyway,
 silently,
 with exit status 0 and no
warning.

The remote command that did it,
 as recorded in the run's build log:

```bash
RUSTUP_HOME=/Volumes/MacData/.../rustup CARGO_HOME=/Volumes/MacData/.../cargo-home \
  CARGO_TARGET_DIR=... RUSTFLAGS="" /Users/user/.cargo/bin/cargo +nightly-2026-09-12 --version \
  && /Users/user/.cargo/bin/rustc +nightly-2026-09-12 --print cfg --target aarch64-apple-darwin
```

The environment prefix binds to the first command only.
 The `rustc` proxy after
`&&` runs with the login shell's environment,
 resolves `+nightly-2026-09-12`
against the machine's own `~/.rustup`,
 does not find it,
 installs it with the
default profile,
 and prints the answer that was asked for.
 Nothing in the output
says an install happened unless the caller reads past the requested output,
 and a
`| head -40` on the same command removes even that.

What is left behind:

```bash
du -sh ~/.rustup/toolchains/*
# 1.4G  /Users/user/.rustup/toolchains/nightly-2026-09-12-aarch64-apple-darwin
# 1.8G  /Users/user/.rustup/toolchains/nightly-aarch64-apple-darwin

du -sh /Volumes/MacData/.../rustup
# 451M   (the minimal-profile toolchain that was installed on purpose)
```

The intended toolchain is 451 MiB with `--profile minimal`;
 the accidental one is
1.4 GB because the auto-install uses the machine's configured `profile`,
 which
was `default`.

## Root cause

Two behaviours combine,
 and neither is a bug.

First,
 `RUSTUP_HOME` is an ordinary environment variable,
 and a shell environment
prefix applies to one simple command.
 `A=1 foo && bar` runs `bar` without `A`.
This is POSIX shell,
 not a rustup behaviour,
 but it is what makes the second
behaviour reachable by accident.

Second,
 a rustup proxy (`~/.cargo/bin/rustc`,
`~/.cargo/bin/cargo`,
and the rest)
invoked with a `+toolchain` argument installs that toolchain when it is missing,
rather than failing.
 Verified directly,
 with a throwaway `RUSTUP_HOME` so the
observation costs nothing permanent:

```bash
# Auto-install disabled: the proxy refuses and says so.
RUSTUP_HOME=/Volumes/MacData/.../rustup-throwaway RUSTUP_AUTO_INSTALL=0 \
  ~/.cargo/bin/rustc +nightly-2026-09-12 --version
# error: toolchain 'nightly-2026-09-12-aarch64-apple-darwin' is not installed
# help: run `rustup toolchain install nightly-2026-09-12-aarch64-apple-darwin` to install it

# Default behaviour on an empty home: it installs, then answers, and exits 0.
RUSTUP_HOME=/Volumes/MacData/.../rustup-throwaway \
  ~/.cargo/bin/rustc +nightly-2026-09-12 --version
# info: syncing channel updates for nightly-2026-09-12-aarch64-apple-darwin
# info: latest update on 2026-09-12 for version 1.100.0-nightly (0fc141305 2026-09-11)
# info: downloading 6 components
# rustc 1.100.0-nightly (0fc141305 2026-09-11)
```

The three `info:` lines go to stderr,
 the answer to stdout,
 so a caller that
captures stdout,
 or pipes the combined stream through `head`,
 sees only
`rustc 1.100.0-nightly ...` and concludes nothing happened.

The install honours the settings of the `RUSTUP_HOME` it lands in,
 including
`profile`:

```toml
# ~/.rustup/settings.toml on the affected machine
version = "12"
default_toolchain = "nightly-aarch64-apple-darwin"
profile = "default"

[overrides]
```

which is why the accidental install is 1.4 GB and the deliberate one,
 made with
`rustup toolchain install ... --profile minimal` into the scratch home,
 is
451 MiB.

Note what the auto-install does not touch:
 `default_toolchain` stayed
`nightly-aarch64-apple-darwin` throughout.
 Only a toolchain directory appeared.

## Verification

- Machine:
   `MacBookAir10,1`,
   Apple M1,
   macOS 27.0 build `26A428`,
   Darwin 27.0.0.
- rustup proxies at `/Users/user/.cargo/bin`,
   machine default toolchain
  `nightly-aarch64-apple-darwin` (rustc 1.99.0-nightly).
- Toolchain installed by accident and on purpose:
  `nightly-2026-09-12-aarch64-apple-darwin`,
   rustc 1.100.0-nightly (0fc141305 2026-09-11).

Commands that write to the default `RUSTUP_HOME` when the toolchain is absent:

```bash
~/.cargo/bin/rustc +<toolchain> --version
~/.cargo/bin/cargo +<toolchain> build
# any rustup proxy with a + argument
```

Commands that do not:

```bash
RUSTUP_HOME=<scratch> ~/.cargo/bin/rustc +<toolchain> --version   # writes to <scratch>
RUSTUP_AUTO_INSTALL=0 ~/.cargo/bin/rustc +<toolchain> --version   # errors, writes nothing
~/.cargo/bin/rustc --version                                      # no + argument, no resolution
```

The distinction that matters for a chained command:

```bash
# WRONG: only the first command sees RUSTUP_HOME.
RUSTUP_HOME=<scratch> cargo +tc --version && rustc +tc --print cfg

# RIGHT: every rustup proxy carries its own prefix.
RUSTUP_HOME=<scratch> cargo +tc --version && RUSTUP_HOME=<scratch> rustc +tc --print cfg

# ALSO RIGHT: export once for the whole remote shell.
env RUSTUP_HOME=<scratch> sh -c 'cargo +tc --version && rustc +tc --print cfg'
```

## Verified workarounds

- Repeat the environment prefix on every rustup proxy in a chained command.
   This
  is what the affected build driver now does,
   with a comment naming the rule it
  protects.
   Tradeoff:
   verbose,
   and easy to forget again on the next chain;
   it
  protects only the commands a human remembered to prefix.
- Set `RUSTUP_AUTO_INSTALL=0` for the whole run.
   A missing toolchain then fails
  loudly instead of installing,
   which turns the silent 1.4 GB into an error
  message naming the toolchain.
   Tradeoff:
   any step that legitimately relies on
  auto-install has to be replaced with an explicit
  `rustup toolchain install`;
   in a measurement run that is an improvement,
   because
  the install should be deliberate and profiled anyway.
- Install deliberately and minimally first:
  `RUSTUP_HOME=<scratch> rustup toolchain install <tc> --profile minimal --no-self-update`.
   The
  scratch home then has the toolchain,
   so nothing auto-installs even if a later
  command loses its prefix,
   as long as that command also has the prefix.
   Tradeoff:
  it does not protect an unprefixed command at all,
   so pair it with one of the
  other two.

Cleanup,
 if it has already happened:

```bash
rustup toolchain uninstall <toolchain>      # uses the default RUSTUP_HOME
rustup toolchain list                       # confirm only the machine's own remain
cat ~/.rustup/settings.toml                 # confirm default_toolchain unchanged
```

On the affected machine this restored `~/.rustup` to 1.8 GB and one toolchain,
with `default_toolchain = "nightly-aarch64-apple-darwin"` unchanged,
 which is the
state it was in before the run.

## What does not work

- Assuming a read-only-looking probe is read-only.
   `rustc +tc --print cfg` reads a
  target specification and writes nothing,
   except that resolving `+tc` can install
  a toolchain first.
   The write is in the resolution,
   not in the subcommand.
- Reading the command's own output to notice it.
   The install notice is on stderr
  and the requested answer on stdout,
   so `cmd 2>/dev/null`,
   `cmd | head -40`,
   and
  any stdout capture hide it.
   In the affected run the command ended in
  `| head -40`,
   which is why it went unnoticed until a `du` of the internal disk.
- Checking `df` afterwards to confirm the cleanup.
   APFS reclaims lazily,
   so `df`
  showed the same free space before and after the uninstall;
   `du -sh ~/.rustup` and
  `ls ~/.rustup/toolchains` show the change immediately.

## Upstream filing decision

Nothing to file,
 and nothing was filed or commented.
Walking the 6-constraint check stops at the first constraint:

1.  **Is it really upstream's fault?
    ** No.
     Auto-installing a `+toolchain` is
    rustup's documented convenience,
     it is switchable with `RUSTUP_AUTO_INSTALL=0`,
    and it writes to exactly the `RUSTUP_HOME` it was given.
     The environment
    prefix binding to one command is POSIX shell.
     Neither piece is a defect,
     and
    the combination is a caller mistake.
     Constraints 2 through 6 are not reached.

The one thing that would have helped is a louder notice,
 and rustup already emits
one;
 it lands on stderr,
 which the caller discarded.
