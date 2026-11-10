# Editor installation issues: VS Code Server fetch failures on non-MS builds, Helix install on Debian, Homebrew gotchas

This file groups three independent editor-install issues that
developers on this workspace have hit.
 Each gets its own canonical
section.

---

## Bug 1: VS Code Server fetch fails when the local VS Code is a non-MS-branded build (VSCodium, OSS)

### Symptom

When opening a workspace in a development container or via Remote-WSL
from a non-Microsoft-branded VS Code (VSCodium,
 OSS builds,
 or
distro-packaged builds without Microsoft branding),
 the automatic
VS Code Server download fails.
 The remote window never finishes
connecting;
 logs show a failure to fetch the server tarball.

### Root cause

VS Code's "download server on first connect" code path uses Microsoft
download endpoints that are gated on the Microsoft branding bit being
set in the client.
 Non-MS builds either omit the branding or use
substituted endpoints that may not serve the matching server build.
The result is a fetch failure with no automatic fallback.

This is a Microsoft licensing choice (the server is licensed only to
official VS Code clients),
 not a defect.
 Non-MS clients are not
entitled to the same download path.

### Verification

Version under test:

- Microsoft VS Code 1.85+ (works)
- VSCodium 1.85+ (fails at server fetch)
- VS Code OSS distro builds (fails at server fetch)

Reproduce:
 open a Remote-WSL workspace from a non-MS build;
 the
connection fails during the "Downloading VS Code Server…" step.

### Verified workaround: download the server tarball manually

Fetch the latest server build directly from Microsoft and place it
where the client expects it:

```bash
curl -L \
  https://update.code.visualstudio.com/latest/server-linux-x64/stable \
  -o vscode-server.tar.gz
mkdir -p ~/.vscode-server/bin
tar -xz -C ~/.vscode-server/bin -f vscode-server.tar.gz
```

Tradeoff:
 the server is technically licensed for use with Microsoft
clients;
 running it under VSCodium may violate the terms of service.
For pure development use this is a workspace decision;
 for any
distribution or commercial deployment,
 switch to Microsoft VS Code or
to an editor with a permissively licensed server (code-server,
openvscode-server).

### What does not work

- Pointing the non-MS client at a third-party mirror:
   the client
  pins the tarball hash;
   mismatched hashes are rejected.
- Building the server from source:
   the Microsoft-licensed bits are
  not in the upstream `microsoft/vscode` repo.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. The branding gate is a
    licensing requirement.
2.  **Can upstream fix it?
    ** They could relax the licensing,
     but
    doing so would undermine the commercial model around the
    Marketplace and proprietary extensions.
3.  **Are they supporting this use case?
    ** No;
     non-MS clients are
    explicitly out of scope.
4.  **Will they likely fix it?
    ** No.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.
 Use Microsoft VS Code,
 or accept the
manual server install with the licence implications named.

---

## Bug 2: Helix install on Debian via PPA fails with `AttributeError: 'NoneType' object has no attribute 'people'`

### Symptom

```bash
sudo add-apt-repository ppa:maveonair/helix-editor
# AttributeError: 'NoneType' object has no attribute 'people'
```

### Root cause

PPAs (Personal Package Archives) are Launchpad-hosted Ubuntu
repositories.
 `add-apt-repository` looks up the PPA owner via the
Launchpad API by Ubuntu series name.
 On Debian,
 the system's series
name does not match any Ubuntu series,
 so the API query returns
`None`,
 and the Python script that prepares the PPA URL dereferences
`None.people` and raises `AttributeError`.

This is a documented limitation of PPAs:
 they are Ubuntu-specific.
Debian and its non-Ubuntu derivatives cannot consume them.
 The
crash is the surface of that limitation;
 the underlying impossibility
is the architectural mismatch.

### Verification

Version under test:

- Debian 12 (bookworm) with `software-properties-common` providing
  `add-apt-repository`
- Ubuntu 24.04+ (works correctly)

Reproduce on Debian:
 the `add-apt-repository ppa:...` invocation
raises the AttributeError verbatim.

### Verified workarounds

#### Homebrew (preferred for this workspace)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
echo 'export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"' >> ~/.profile
source ~/.profile
brew install helix
```

Tradeoff:
 introduces a second package manager.
 Homebrew on Linux is
self-contained under `/home/linuxbrew`,
 so it does not collide with
APT,
 but every developer must bootstrap it once.

#### Pre-built binary from GitHub releases

```bash
curl -LO https://github.com/helix-editor/helix/releases/download/25.01/helix-25.01-x86_64-linux.tar.xz
tar xf helix-25.01-x86_64-linux.tar.xz
sudo mv helix-25.01-x86_64-linux/hx /usr/local/bin/
```

Tradeoff:
 no automatic updates.
 Each release must be downloaded
manually.
 Suitable for pinning to a known-good version.

#### Build from source

```bash
git clone https://github.com/helix-editor/helix
cd helix
cargo install --path helix-term --locked
```

Tradeoff:
 requires a Rust toolchain (`cargo`);
 compile time is
minutes on first build.
 Useful when needing unreleased fixes from
`master`.

### What does not work

- Editing `/etc/apt/sources.list.d/` by hand to add the PPA URL:
   even
  with a correctly synthesised URL,
   the PPA's package metadata is
  built against a specific Ubuntu series and APT will refuse the
  signed-by mismatch on Debian.
- `apt install helix` from Debian's own repos:
   as of bookworm,
   Helix
  is not packaged in stable Debian.
   May change in trixie.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. The PPA system is by design
    Ubuntu-specific.
2.  **Can upstream fix it?
    ** Helix could publish Debian packages,
     but
    that is a packaging request,
     not a bug.
3.  **Are they supporting this use case?
    ** The published install docs
    already list Homebrew and binary releases as alternatives.
4.  **Will they likely fix it?
    ** A Debian repository would be a
    sizeable maintenance commitment;
     unlikely without a packager
    volunteering.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 Use Homebrew,
 prebuilt binary,
 or
source build.

---

## Bug 3: Homebrew PATH override on Linux clobbers system binaries

### Symptom

After following an install guide that writes `export PATH=...` for
Homebrew,
 common commands stop working:

```bash
$ ls
bash: ls: command not found
$ cd /home/user
bash: cd: command not found
```

### Root cause

The bad form **overwrites** the entire PATH instead of prepending:

```bash
# BAD: this is now PATH, system bins are gone
export PATH="/home/linuxbrew/.linuxbrew/bin"
```

The shell's PATH now contains only `/home/linuxbrew/.linuxbrew/bin`;
`/usr/bin`,
 `/bin`,
 and `/usr/local/bin` are not searched,
 so all
system commands fail to resolve.

### Verification

Reproduce in a new shell session:
 run the BAD command and try
`ls`,
 `cd`,
 `cat`.
 All fail.
 Run the corrected form (below) and the
commands resolve again.

### Verified workaround

Always **prepend** to the existing PATH:

```bash
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
```

The trailing `:$PATH` preserves the prior search path so system
binaries continue to resolve.

Tradeoff:
 prepending means Homebrew binaries beat system binaries
when names collide.
 This is usually the goal (Homebrew newer than
system) but can surprise scripts that assumed a specific path.
Append (`PATH=$PATH:...`) only when system binaries should win.

### What does not work

- Setting PATH only inside a function:
   PATH is exported,
   so the loss
  of system bins inside the function is total.
   The wrong assignment
  destroys the search in the same scope.
- Restarting the shell to recover:
   only works because the rc file
  reset PATH from defaults;
   the bad export in `.profile` would
  re-apply on next login and lock the shell out again.
   Edit the rc
  file first.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. PATH is a shell variable;
    the bug is a user-side `export` mistake.
2.  **Can upstream fix it?
    ** Homebrew's install script already prints
    the correct form.
     The wrong form comes from outdated guides,
     not
    Homebrew.
3.  **Are they supporting this use case?
    ** Yes.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.
 The fix is to use the correct
prepend form.

---

## Bug 4: Homebrew installs its own gcc when system gcc is missing

### Symptom

When installing Helix on a fresh Debian:

```bash
$ brew install helix
Warning: The post-install step did not complete successfully
```

Helix runs,
 but it links against Homebrew's libc and libgcc copies
under `/home/linuxbrew/.linuxbrew/...`,
 doubling disk usage and
making the editor sensitive to Homebrew library updates.

### Root cause

Homebrew compiles its formulae against its own toolchain when a
system gcc is not detected.
 The "post-install warning" surfaces the
toolchain mismatch but does not fail the install.
 The result is a
working binary linked to Homebrew's runtime instead of the system's.

### Verification

```bash
# Check what gcc/glibc the Homebrew binary links against
ldd "$(which hx)" | grep -E '(gcc|libc)'

# Homebrew-linked output: /home/linuxbrew/.linuxbrew/...
# System-linked output:   /lib/x86_64-linux-gnu/...
```

### Verified workaround

Install the system gcc **before** installing Helix:

```bash
sudo apt install gcc
brew install helix
```

If Homebrew already installed its own gcc:

```bash
brew uninstall helix
brew uninstall gcc   # auto-removes if nothing else depends
brew install helix   # uses system gcc now
```

Tradeoff:
 the system gcc adds APT-managed packages to the system
inventory.
 Acceptable for a development machine;
 less so for a
container image trying to minimise size.

### What does not work

- Setting `HOMEBREW_USE_SYSTEM_GCC=1` (or similar env hints):
  Homebrew does not expose a per-install "prefer system toolchain"
  flag;
   the detection is automatic at install time.
   The only lever
  is to have the system toolchain present first.
- Removing Homebrew's gcc post-install:
   brew refuses if any installed
  formula depends on it,
   so the workaround requires uninstalling the
  dependent formula first.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** No. Homebrew is correctly
    compiling against the available toolchain.
2.  **Can upstream fix it?
    ** They could prompt for system-gcc
    installation,
     but the current "warn-and-proceed" behaviour is
    reasonable.
3.  **Are they supporting this use case?
    ** Yes;
     Homebrew-on-Linux is
    supported.
4.  **Will they likely fix it?
    ** No.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.
 Install system gcc first.

---

## Verifying Helix works after install

```bash
hx --version  # version string
hx --health   # language server status; warnings about missing runtime
              # directories are normal on a fresh install
```

Helix uses language servers (no plugin system);
 configuration lives
under `~/.config/helix/`.
