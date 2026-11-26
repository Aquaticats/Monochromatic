# IVPN UI 3.15.15 AUR build stops in `prepare()` when the builder's `~/.npmrc` sets `prefix`, via nvm 0.40.8

Diagnosis recorded on `2026-11-26T01:41:13-05:00`.
The fix was prototyped in a scratch copy of the recipe; nothing was posted upstream.
This record is unapproved.

## Symptom

On CachyOS, the Shelly UI ran `shelly install aur ivpn-ui ivpn --ui-mode` through `pkexec`.
`ivpn` built and installed.
`ivpn-ui` `3.15.15-1` failed in `prepare()`, right after nvm downloaded node v22.12.0.
The per-build log (`~/.cache/Shelly/ivpn-ui/ivpn-ui-<id>.log`) ends:

```text
# Shelly per-build log
[stderr] Your user’s .npmrc file (${HOME}/.npmrc)
[stderr] has a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.
[stderr] Run `nvm use --delete-prefix v22.12.0` to unset it.
[error] ivpn-ui: prepare: StepFailed
```

The UI reported `Could not build "ivpn-ui". The build failed during prepare.`
The literal `${HOME}` in the message is nvm's own path sanitizing, not an unexpanded variable.
The host's `~/.npmrc` holds one line, `prefix=/home/user/.local`.

## Root cause

### The recipe loads nvm with the builder's real home

The recipe at AUR commit `d18e66dec2b72b366a3718a7085de7f2f4d6e6e7` points `NVM_DIR` into `$srcdir`,
but leaves `HOME` alone, then installs node in `prepare()`. [AUR recipe][aur-recipe]

```bash
# PKGBUILD:26-43
_ensure_local_nvm() {
    # https://wiki.archlinux.org/title/Node.js_package_guidelines

    # let's be sure we are starting clean
    which nvm >/dev/null 2>&1 && nvm deactivate && nvm unload
    export NVM_DIR="${srcdir}/.nvm"
    ...
    source /usr/share/nvm/init-nvm.sh || [[ $? != 1 ]]
}

prepare() {
  # https://wiki.archlinux.org/title/Node.js_package_guidelines
  _ensure_local_nvm
  nvm install 22.12.0
}
```

### nvm refuses a `prefix` line in `${HOME}/.npmrc`

After installing a version,
nvm runs `nvm_die_on_prefix`.
Its user-npmrc check takes the path from `HOME`,
not from npm's `userconfig` setting,
and returns `10` unless asked to delete the line. [nvm user npmrc check][nvm-user-npmrc]

```bash
# nvm.sh:3194-3206 (v0.40.8)
  local NVM_NPM_USER_NPMRC
  NVM_NPM_USER_NPMRC="${HOME}/.npmrc"
  if nvm_npmrc_bad_news_bears "${NVM_NPM_USER_NPMRC}"; then
    if [ "_${NVM_DELETE_PREFIX}" = "_1" ]; then
      npm config --loglevel=warn delete prefix --userconfig="${NVM_NPM_USER_NPMRC}"
      npm config --loglevel=warn delete globalconfig --userconfig="${NVM_NPM_USER_NPMRC}"
    else
      nvm_err "Your user’s .npmrc file ($(nvm_sanitize_path "${NVM_NPM_USER_NPMRC}"))"
      nvm_err 'has a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.'
      nvm_err "Run \`${NVM_COMMAND}\` to unset it."
      return 10
    fi
  fi
```

The test is a plain line match on the file. [nvm npmrc test][nvm-bad-news]

```bash
# nvm.sh:3091-3098 (v0.40.8)
nvm_npmrc_bad_news_bears() {
  local NVM_NPMRC
  NVM_NPMRC="${1-}"
  if [ -n "${NVM_NPMRC}" ] && [ -f "${NVM_NPMRC}" ] && nvm_grep -Ee '^(prefix|globalconfig) *=' <"${NVM_NPMRC}" >/dev/null; then
    return 0
  fi
  return 1
}
```

The same function also rejects an `NPM_CONFIG_PREFIX` environment variable that points outside `NVM_DIR`
(`nvm.sh:3138-3152`).

### Shelly keeps the invoking user's `HOME` and will not let configuration override it

Shelly 3.4.1 builds its child environment from the inherited one and adds only `[build.env]` assignments and `PATH`.
[Shelly build environment][shelly-env]

```zig
// Shelly.PackageManager/src/aur/builder/builder.zig:275-289
var environment = try self.environ.createMap(self.allocator);
defer environment.deinit();
for (self.shellybuild_config.build.env) |assignment| {
    ...
    try environment.put(assignment.name, assignment.value);
}
try environment.put("PATH", path);
```

`HOME` is on the reserved list,
so `shellybuild.conf` cannot point it elsewhere. [Shelly reserved variables][shelly-reserved]

```zig
// Shelly.PackageManager/src/aur/shellybuild.zig:423-428
if (containsString(&.{
    "PATH",     "HOME",         "USER",              "LOGNAME", "SHELL",    "PWD",       "OLDPWD",
    ...
}, name)) return error.ReservedBuildEnvironmentVariable;
```

This is not a Shelly defect:
`makepkg` does not change `HOME` either
(its only uses read `$HOME/.makepkg.conf` and the XDG config folder, `/usr/share/makepkg/util/config.sh:53-58`).
A `makepkg` build of the same recipe was not run.

## Verification

Versions:
AUR `ivpn-ui` at `d18e66dec2b72b366a3718a7085de7f2f4d6e6e7` (`pkgver=3.15.15`, `pkgrel=1`),
Arch `nvm` `0.40.8-1` (upstream tag `v0.40.8`, commit `a885b885fef16fac4bc544188fb25e9e37ae83e8`),
CachyOS `shelly` `3.4.1-1` (tag `v3.4.1`, commit `852e56e4c4db255eb93b4d6feca12a86aa9e9485`).
`nvm` (a make dependency) was installed for each run and removed afterwards.

Harness, run as the regular user in a fresh `git clone https://aur.archlinux.org/ivpn-ui.git`,
with a minimal environment so the session's variables do not leak into the build:

```bash
# In the cloned recipe; HOME is the variable under test
env --ignore-environment HOME=<home under test> USER=user LOGNAME=user \
  PATH=/usr/local/sbin:/usr/local/bin:/usr/bin LANG=C.UTF-8 TERM=dumb \
  shelly build --reviewed --no-confirm --nosign PKGBUILD
```

A probe recipe whose `prepare()` printed `$HOME` confirmed that a user-level `shelly build` passes `HOME` through.

### Fails

- Unmodified recipe, `HOME=/home/user` (holds `prefix=/home/user/.local`):
  the nvm message from "Symptom", `Technical details: StepFailed`, exit `1`.
- The Shelly UI run (`pkexec`, building as the invoking user): the same message.

### Works

- Unmodified recipe, `HOME` set to an empty scratch folder:
  `Now using node v22.12.0`, `Built ivpn-ui: .../ivpn-ui-3.15.15-1-x86_64.pkg.tar.zst`, exit `0`, in 26 seconds.
- Recipe with the patch in "Recipe fix", `HOME=/home/user`:
  `Now using node v22.12.0 (npm v10.9.0)`, `Built ivpn-ui`, exit `0`.

The archive from the scratch-home build was installed with `sudo shelly install standard <archive>`.
`pacman -Qi ivpn-ui` then reported `3.15.15-1`, `Explicitly installed`.

## Verified workarounds

### Build as the user with a scratch home, then install the archive with Shelly

```bash
# In a reviewed clone of the AUR recipe; nvm and npm must be installed (make dependencies)
mkdir --parents ~/temp/agent/ivpn-ui-build-home
env --ignore-environment HOME="$HOME/temp/agent/ivpn-ui-build-home" USER="$USER" LOGNAME="$USER" \
  PATH=/usr/local/sbin:/usr/local/bin:/usr/bin LANG=C.UTF-8 TERM=dumb \
  shelly build --reviewed --no-confirm --nosign PKGBUILD
sudo shelly install standard ./ivpn-ui-3.15.15-1-x86_64.pkg.tar.zst
```

Tradeoffs:
the build cannot see anything else in the real home
(npm and Electron caches start empty, so every build downloads them again);
`shelly upgrade aur` and the Shelly UI still build with the real home,
so each `ivpn-ui` release needs this procedure again.
Shelly still lists the package among AUR packages,
because it discovers them from pacman's foreign inventory
(see `doc/troubleshooting/shelly-browsers-bin-package-root-check.md`, "Upgrade behavior").

### Recipe fix (prototype)

Point `HOME` into `$srcdir` inside the recipe's own helper:

```diff
--- a/PKGBUILD
+++ b/PKGBUILD
@@ -29,6 +29,9 @@ _ensure_local_nvm() {
     # let's be sure we are starting clean
     which nvm >/dev/null 2>&1 && nvm deactivate && nvm unload
     export NVM_DIR="${srcdir}/.nvm"
+    # nvm refuses a prefix= or globalconfig= line in ${HOME}/.npmrc; keep the builder's npm configuration out of the build
+    export HOME="${srcdir}/.home"
+    mkdir -p "${HOME}"
 
     # The init script returns 3 if version specified
     # in ./.nvrc is not (yet) installed in $NVM_DIR
```

Verified by the "Works" and "Fails" catalogs:
the same environment with `HOME=/home/user` fails before the patch and builds after it.
Tradeoff:
`npm install` and electron-builder then cache under `$srcdir/.home`,
so a clean build downloads them again,
and any registry credentials in the builder's `~/.npmrc` are no longer used.
Upgrades work unchanged only once the recipe itself carries this.

### Remove the `prefix` line from `~/.npmrc`

Source-derived and not run:
with no `prefix` or `globalconfig` line,
`nvm_npmrc_bad_news_bears` returns `1` and the check passes,
so `shelly install aur ivpn-ui` and later upgrades build unchanged.
Tradeoff:
`npm install --global` no longer installs into `~/.local`.
On `2026-11-26T01:41:13-05:00`,
`~/.local/lib/node_modules` was empty,
so nothing currently installed depends on the setting.
This is the owner's npm configuration and was not changed.

## What does not work

- A `[build.env]` entry in `~/.config/shelly/shellybuild.conf`:
  `HOME` is reserved,
  and `NPM_CONFIG_USERCONFIG` does not move the file nvm checks.
- Moving the setting to `NPM_CONFIG_PREFIX`:
  nvm rejects that variable too when it points outside `NVM_DIR`.
- `nvm use --delete-prefix`, as the message suggests:
  it deletes the line from the owner's `~/.npmrc`.
- `shelly install aur --isolated ivpn-ui` (not run):
  Shelly's isolated builds provision global `depends` as build inputs and accept only repository packages,
  and `ivpn>=3.15.15` exists only in the AUR
  (Shelly `docs/isolated-builds.md`, "Isolated plans require repository artifacts for every build dependency").
- Building from an agent session without clearing the environment:
  the session's `NODE_OPTIONS=--permission-audit` reaches node v22.12.0,
  which exits with `node: --permission-audit is not allowed in NODE_OPTIONS` (build step code `9`),
  and the session's API keys would be visible to every npm install script.
  The minimal environment in the harness avoids both.

## Upstream filing decision

Target:
the AUR package `ivpn-ui`,
whose recipe is not in `ivpn/desktop-app`
(that repository's top level holds `cli`, `daemon`, `docs`, `snap` and `ui`, and no `PKGBUILD`).
No `.out-of-scope/` entry covers IVPN, nvm or Shelly.

Searches on `2026-11-26T01:41:13-05:00`:
the AUR comment page (`?PP=250`) had four matches for `npmrc`, `prefix` or `nvm`,
all about the `nvm` dependency listing or nvm defaults, none about this failure;
`gh search issues --repo ivpn/desktop-app` for `npmrc`, `nvm prefix` and `PKGBUILD nvm` returned nothing.
No work in progress on the recipe was found.

1.  **Is it really upstream's fault?**
     Yes, at the recipe level:
    nvm's refusal is deliberate,
     and a `prefix` line is an ordinary npm setup,
     but the recipe lets the builder's home configuration decide whether it builds.
2.  **Can upstream fix it?**
     Yes; the prototype is three lines.
3.  **Are they supporting this use case?**
     Soft yes:
    the package is meant to build for AUR users,
     whose home folders vary;
     no statement either way was found.
4.  **Would the repo welcome our contribution?**
     AUR comments are the package's intake,
     and a comment thanking the maintainer for an earlier fix shows they respond.
    The AUR's rules on AI-assisted comments were not read on `2026-11-26T01:41:13-05:00`;
     the AUR site also refused `agent-browser` (Anubis "Access Denied"),
     so this constraint is unchecked.
5.  **Will they likely fix it?**
     Plausible; no sign of refusal.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     Yes; see "Recipe fix (prototype)".

Readiness on `2026-11-26T01:41:13-05:00`:
reproduction re-run on the only release line the AUR carries (`3.15.15-1`);
prior-art search done as listed;
AUR comment policy, a clean-runner reproduction repository and a same-day trust-system check not done.
So there is no recommendation.
A comment posted from the owner's AUR account puts that account in front of the maintainer;
filing is the owner's decision.

~~~md
ivpn-ui 3.15.15-1 fails in prepare() when the builder's ~/.npmrc contains a prefix= (or globalconfig=) line:

    Your user’s .npmrc file (${HOME}/.npmrc)
    has a `globalconfig` and/or a `prefix` setting, which are incompatible with nvm.
    Run `nvm use --delete-prefix v22.12.0` to unset it.

nvm 0.40.8 checks "${HOME}/.npmrc" after `nvm install` (nvm.sh lines 3194-3206 at v0.40.8) and returns 10.
_ensure_local_nvm moves NVM_DIR into $srcdir but keeps the builder's HOME, so the result depends on the
builder's dotfiles. A prefix= line is a common way to install npm globals without root.

Setting HOME inside the helper fixes it; with this change the package builds with the same ~/.npmrc:

    export NVM_DIR="${srcdir}/.nvm"
    export HOME="${srcdir}/.home"
    mkdir -p "${HOME}"

npm and electron-builder then cache under $srcdir, so a clean build downloads them again.
~~~

[aur-recipe]: https://aur.archlinux.org/cgit/aur.git/tree/PKGBUILD?h=ivpn-ui&id=d18e66dec2b72b366a3718a7085de7f2f4d6e6e7#n26
[nvm-user-npmrc]: https://github.com/nvm-sh/nvm/blob/a885b885fef16fac4bc544188fb25e9e37ae83e8/nvm.sh#L3194-L3206
[nvm-bad-news]: https://github.com/nvm-sh/nvm/blob/a885b885fef16fac4bc544188fb25e9e37ae83e8/nvm.sh#L3091-L3098
[shelly-env]: https://github.com/Seafoam-Labs/Shelly-ALPM/blob/852e56e4c4db255eb93b4d6feca12a86aa9e9485/Shelly.PackageManager/src/aur/builder/builder.zig#L263-L292
[shelly-reserved]: https://github.com/Seafoam-Labs/Shelly-ALPM/blob/852e56e4c4db255eb93b4d6feca12a86aa9e9485/Shelly.PackageManager/src/aur/shellybuild.zig#L415-L432

The `nvm` and Shelly links were opened in `agent-browser` on `2026-11-26T01:41:13-05:00` and land on the quoted lines.
The AUR link could not be opened there (Anubis "Access Denied");
its lines were checked with `git show d18e66dec2b72b366a3718a7085de7f2f4d6e6e7:PKGBUILD` in a clone.
