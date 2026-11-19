# pnpm 12.21.0 starts Node.js on every invocation when its own install script is skipped

Investigation record pending owner review. Codex checked the Linux npm package with Node.js 26.13.1.
Other platforms and full workspace installs remain untested.

## Symptom

In an interactive terminal, the npm-installed `pnpm` prints this warning on standard error and still runs:

```text
pnpm is running through Node.js because the script that installs its native binary was skipped.
Reinstall pnpm with its build scripts allowed to run the binary directly.
```

## Root cause

The npm package starts with a shell launcher. Its install script replaces that launcher with the platform executable.
The Unix branch in `pnpm/npm/pnpm/install.js:69` installs the same executable at each command name.
[Install script][install]

```js
for (const name of BIN_NAMES) {
  placeBinary(nativeBinary, path.join(wrapperDir, name), 0o755)
}
```

The placeholder requires `node` on `PATH` and prints the warning when stderr is a terminal.
It then calls `exec node "${self%/*}/bin/pnpm.mjs" "$@"`.
The checks and handoff are in `pnpm/npm/pnpm/pnpm:17` and `pnpm/npm/pnpm/pnpm:99`.
[Launcher checks][launcher] · [Handoff][handoff]

```sh
if ! command -v node > /dev/null 2>&1; then
  echo "pnpm's native binary was not installed, and there is no Node.js to run pnpm without it." >&2
  echo "Reinstall pnpm with build scripts enabled (allow-list pnpm's build under pnpm or Bun, or drop --ignore-scripts)." >&2
  exit 1
fi

if [ -t 2 ]; then
  echo "pnpm is running through Node.js because the script that installs its native binary was skipped." >&2
  echo "Reinstall pnpm with its build scripts allowed to run the binary directly." >&2
fi
```

The Node.js wrapper starts the native executable as a subprocess and forwards the arguments and standard streams.
It uses the executable from an installed platform package when available. If both that executable and the cached
download are absent, the wrapper tries to download the matching version into the package directory.
The subprocess call and binary selection are in `pnpm/npm/pnpm/bin/pnpm.mjs:50` and
`pnpm/npm/pnpm/bin/pnpm.mjs:59`. [Subprocess][spawn] · [Binary selection][select]

```js
const result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit' })
```

```js
const installed = resolveInstalledBinary()
if (installed != null) {
  return installed
}
```

```js
await downloadPnpmExecutable({ version, destPath: DOWNLOADED_BINARY, ...options })
```

## Verification

The installed package reported pnpm `12.21.0`, matching upstream tag `v12.21.0` at commit
`b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600`. The GitHub contents API confirmed the linked source against
the tag checkout. This session lacked `agent-browser` for rendering the selections.

Run these probes from the directory of the npm-installed pnpm package while the launcher is in place.

```sh
# npm-installed pnpm package
script --quiet --return --command './pnpm --version' /dev/null
env PATH=/nonexistent /bin/sh ./pnpm --version
env PATH=/nonexistent ./node_modules/@pnpm/exe.linux-x64/pnpm --version
```

- The interactive launcher printed the warning followed by `12.21.0` and exited successfully.
- With Node.js absent from `PATH`, the shell launcher printed the missing-Node.js error and exited with status 1.
  The Linux x64 platform binary printed `12.21.0` under the same restricted `PATH`.
- Python `subprocess.run([path, '--version'], ...)` raised `ENOEXEC` for the shell launcher and succeeded for
  the direct binary. The shell launcher has no shebang and relies on the caller retrying it through a shell.
- An offline `install --offline --lockfile-only --ignore-scripts` for a local `file:` dependency produced
  byte-identical lockfiles in two disposable projects, one per entry point.

In eight alternating pairs, the wrapper took 5.79 times as long as direct invocation for `--version`
(4.94 to 6.69 times across pairs). Each pair used the same native executable under Bash and checked output and
exit status. Direct calls took 5.60 to 6.39 ms, and wrapped calls took 30.44 to 40.57 ms.
The measurement covers startup. The Node.js subprocess parent also stays alive while the native command runs.

## Verified workaround

Running `node ./install.js` in a disposable copy of the npm package with its platform dependency already installed
replaced the launcher with a hard link to the native executable. The resulting command returned `12.21.0` in a
terminal with no warning and no Node.js on `PATH`. Allowing pnpm's own lifecycle scripts during installation
lets it perform this replacement. A setting that broadly enables dependency scripts also permits other packages
to execute their hooks. Monochromatic's toolchain lock already selects the standalone pnpm release archive.
[Toolchain lock](../../toolchain.lock.json)

## Launch failures

Removing Node.js from `PATH` makes the placeholder abort, and launching it with Python without a shell produces
`ENOEXEC`. Redirecting stderr makes the warning disappear while the shell still starts Node.js.

## Upstream filing decision

The fallback is intentional and covered by a test for installation with scripts disabled. Changing its startup
behavior would be an upstream design change. This report assessed the observed effects. It leaves contribution
policy, maintainer interest, and a patch outside the investigation. [Upstream test][test]

[install]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/npm/pnpm/install.js#L69-L73
[launcher]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/npm/pnpm/pnpm#L17-L26
[handoff]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/npm/pnpm/pnpm#L99
[spawn]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/npm/pnpm/bin/pnpm.mjs#L42-L57
[select]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/npm/pnpm/bin/pnpm.mjs#L59-L86
[test]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/npm/pnpm/test/install.test.mjs#L79-L96
