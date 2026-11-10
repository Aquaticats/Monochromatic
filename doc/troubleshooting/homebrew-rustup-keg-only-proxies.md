# Homebrew `rustup` 1.29 puts cargo/rustc proxies in a keg-only bin, so a bare `cargo build` fails to find `rustc`

On macOS,
 `brew install rustup` then `rustup default <toolchain>` leaves `cargo`
and `rustc` off the default PATH.
 Running the toolchain through `rustup run
<toolchain> cargo build` then fails because cargo cannot find `rustc`.
 The fix is
to add the keg's bin directory to PATH,
 exactly as the formula's own caveat says.

## Symptom

After `brew install rustup` and `rustup default nightly` (a toolchain is
installed under `~/.rustup/toolchains/`),
 invoking cargo through rustup fails at
the very first step:

```text
$ rustup run nightly cargo build
error: could not execute process `rustc -vV` (never executed)

Caused by:
  No such file or directory (os error 2)
```

Symptoms that look related and mislead the diagnosis:

- `rustup run nightly cargo --version` succeeds (it never needs `rustc`),
   which
  makes the toolchain look fully working.
- `~/.cargo/bin` does not exist at all (the directory the official installer
  creates).
- `which cargo` and `which rustc` return nothing on a default PATH;
   only
  `rustup` resolves (a single symlink in `/opt/homebrew/bin`).

## Root cause

Two facts combine:
 Homebrew installs the rustup proxies into a keg-only
directory,
 and `rustup run` resolves `rustc` only through `~/.cargo/bin`.

### 1. Homebrew's `rustup` formula is keg-only and ships the proxies in its own bin

`brew info rustup` on the affected machine:

```text
==> rustup: stable 1.29.0 (bottled), HEAD [keg-only]
Old Names: rustup-init
rustup is keg-only, which means it was not symlinked into /opt/homebrew,
because it conflicts with rust.
```

The formula (`homebrew-core` `Formula/r/rustup.rb`,
 read locally with
`brew cat rustup`) states in its caveats that it does not run the self-installer
and that you must add its bin directory to PATH yourself:

```ruby
# Formula/r/rustup.rb:63
  def caveats
    <<~EOS
      To use rustup, ensure you have "$(brew --prefix rustup)/bin" in your $PATH:
        #{Formatter.url("https://rust-lang.github.io/rustup/installation/already-installed-rust.html")}

      This formula does not provide `rustup-init`.
    EOS
  end
```

Because it does not run `rustup-init` self-install,
 `~/.cargo/bin` is never
created.
 Instead the proxies live in the keg bin,
 `$(brew --prefix rustup)/bin`
(`/opt/homebrew/opt/rustup/bin`).
 The formula's own test block exercises them
there:

```ruby
# Formula/r/rustup.rb:72
  test do
    ENV["CARGO_HOME"] = testpath/".cargo"
    ENV["RUSTUP_HOME"] = testpath/".rustup"
    ENV.prepend_path "PATH", bin
    assert_match "stable", shell_output("#{bin}/rustup default")
    assert_match "stable", shell_output("#{bin}/rustc --version 2>&1")
    system bin/"cargo", "new", "--bin", "./app"
```

Listing the keg bin confirms the full proxy set is present there (not in
`~/.cargo/bin`,
 not in `/opt/homebrew/bin`):

```text
$ ls "$(brew --prefix rustup)/bin"
cargo  cargo-clippy  cargo-fmt  cargo-miri  clippy-driver  rls
rust-analyzer  rust-gdb  rust-gdbgui  rust-lldb  rustc  rustdoc  rustfmt  rustup
```

Keg-only means this directory is NOT linked into `/opt/homebrew/bin`,
 so it is
absent from PATH unless you add it.
 Only `rustup` itself is linked:
`/opt/homebrew/bin/rustup -> ../Cellar/rustup/1.29.0_2/bin/rustup`.

### 2. `rustup run` resolves `rustc` only via `~/.cargo/bin`, with no fallback

When you call `rustup run <toolchain> cargo build`,
 rustup builds the child
process environment and prepends `CARGO_HOME/bin` (default `~/.cargo/bin`) to
PATH so that cargo's later call to `rustc` hits a proxy.
 rustup source
(`rust-lang/rustup`,
 commit `40dcace8cd16a309fa4079b7f5a06397563c00f3`,
`src/toolchain.rs:230`):

```rust
// Prepend CARGO_HOME/bin to the PATH variable so that we're sure to run
// cargo/rustc via the proxy bins. There is no fallback case for if the
// proxy bins don't exist. We'll just be running whatever happens to
// be on the PATH.
let mut path_entries = vec![];
if let Ok(cargo_home) = self.cfg.process.cargo_home() {
    path_entries.push(cargo_home.join("bin"));
}
```

The "There is no fallback case for if the proxy bins don't exist" comment is the
whole bug surface here.
 rustup launches the toolchain's `cargo` directly,
 but
cargo then spawns `rustc` by name through PATH.
 The only PATH entry rustup added
for that is `~/.cargo/bin`,
 which Homebrew never created,
 and the keg bin is not
on PATH either,
 so `rustc` is nowhere and cargo reports "could not execute
process `rustc`".

This is intended behavior on both sides,
 not a defect:
 rustup deliberately routes
through `CARGO_HOME/bin`,
 and Homebrew deliberately ships keg-only and tells you
to add its bin to PATH.
 The failure only appears when you skip the caveat and try
to drive cargo through `rustup run` instead.

## Verification

Versions under test:

- Homebrew `rustup` 1.29.0_2 bottle (formula `homebrew-core` `Formula/r/rustup.rb`).
- Default toolchain `nightly-aarch64-apple-darwin`,
   `cargo 1.98.0-nightly`.
- rustup source trace:
   `rust-lang/rustup` @ `40dcace8cd16a309fa4079b7f5a06397563c00f3`.
- macOS 26.5.1 (25F80),
   Apple Silicon,
   Command Line Tools only (no `/opt/homebrew/bin/cargo`).

Fails:

```text
$ rustup run nightly cargo build
error: could not execute process `rustc -vV` (never executed)
Caused by: No such file or directory (os error 2)
```

Works (add the keg bin to PATH,
 then a bare cargo):

```text
$ export PATH="$(brew --prefix rustup)/bin:$PATH"
$ which cargo   # -> /opt/homebrew/opt/rustup/bin/cargo
$ which rustc   # -> /opt/homebrew/opt/rustup/bin/rustc
$ cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.81s
```

## Verified workarounds

### Add the keg bin to PATH (canonical, what the caveat says)

```bash
echo 'export PATH="$(brew --prefix rustup)/bin:$PATH"' >> ~/.zshrc
# new shell, or: export PATH="$(brew --prefix rustup)/bin:$PATH"
rustup default stable   # or nightly
cargo build             # bare cargo now resolves, and so does rustc
```

Tradeoff:
 none functionally.
 `cargo`,
 `rustc`,
 and `cargo +toolchain` overrides
all work because they run through the proxies.
 The only cost is the one-time PATH
edit the keg-only formula requires.

### Prepend the toolchain bin directly (single-toolchain shortcut)

```bash
export PATH="$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH"
cargo build
```

Tradeoff:
 this bypasses the rustup proxies,
 so per-directory `rust-toolchain.toml`
pins and `cargo +nightly` / `+stable` overrides are ignored;
 you always get the
one hardcoded toolchain.
 Fine for a machine with a single toolchain (used for the
one-off music-player build verification),
 wrong for multi-toolchain work.

### Use the official rustup-init instead of Homebrew

Install via `https://rustup.rs` (`rustup-init`),
 which populates `~/.cargo/bin`
with the proxies and appends it to PATH through `~/.cargo/env`.

Tradeoff:
 you now have a rustup outside Homebrew's management (no `brew upgrade`,
and a later `brew install rustup` could collide).
 Pick one installer,
 not both.

## What does not work

- `rustup run <toolchain> cargo build`:
   fails with "could not execute process
  `rustc`" for the reason traced above (`src/toolchain.rs:230`);
   rustup only adds
  `~/.cargo/bin` to PATH,
   which Homebrew never created.
- Adding only `/opt/homebrew/bin` to PATH:
   that holds the single `rustup` symlink,
  not `cargo`/`rustc` (keg-only formula),
   so cargo is still missing.
- Expecting `rustup default <toolchain>` to create `~/.cargo/bin`:
   the Homebrew
  formula does not run the self-installer,
   so the proxy directory is never made.

## Upstream filing decision

No `.out-of-scope/` exemption matches Homebrew or rustup;
 checked
`.out-of-scope/` and found none.
 The 6-constraint check,
 walked for the audit
trail:

1.  Is it really upstream's fault?
     No. Both behaviors are intended and documented:
    rustup routes through `CARGO_HOME/bin` by design (`src/toolchain.rs:230`,
    with the tradeoff explained against PR 3178 / issue 3825),
     and the Homebrew
    formula is keg-only and states the required PATH edit in its caveat
    (`Formula/r/rustup.rb:65`).
     The failure is using `rustup run` while skipping
    the caveat,
     not a bug in either tool.
2.  Can upstream fix it?
     Not applicable;
     there is nothing broken to fix.
3.  Supported use case?
     The supported path (add the keg bin to PATH) is documented
    in the caveat and works.
4.  Would the repo welcome it?
     Not reached.
5.  Will they fix it?
     Not reached;
     nothing to fix.
6.  Prototyped a minimal fix?
     Not applicable.

Decision:
 do not file.
 This is a usage/PATH issue with a documented fix,
 recorded
here so the next macOS build session does not re-derive it.
 No draft issue is kept.
