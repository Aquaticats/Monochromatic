# Node 26.13.1 SEA key-helper fails at login after the simdjson SONAME changes

## Status at 2026-11-23T06:00:23-05:00

Codex diagnosed the CachyOS boot and verified the recovery in an isolated fixture.
Owner review and a live Ctrl+F4 check remain open.

## Symptom

The key-helper daemon launched by KDE autostart failed at `05:44:17 EST` on November 23.
`journalctl --boot --identifier=key-helper` reported:

```text
/home/user/Monochromatic/package/kwin/key-helper/key-helper-service: error while loading shared libraries: libsimdjson.so.36: cannot open shared object file: No such file or directory
```

KWin reported that the `key-helper-kwin` script was loaded.
The live session bus reported `b false` for `NameHasOwner` on `org.monochromatic.KeyHelper`.
The Helium shortcut callback calls `SendKeys` on that service to send Ctrl+W.

`ydotoold` reported `failed to open uinput device: Permission denied` at `05:44:08`, `05:44:10` and `05:44:12 EST`.
The installed retry drop-in got `ydotoold` running at `05:44:14 EST`, before key-helper launched.
The earlier [ydotool startup incident](ydotool-user-unit-starts-before-uinput-access.md) describes the drop-in.

## Root cause

`/var/log/pacman.log:2504-2505` records the November 21 upgrade:

```text
[2026-11-21T10:37:09-0500] [ALPM] upgraded simdjson (2:5.1.1-1.1 -> 2:5.2.0-1.1)
[2026-11-21T10:37:09-0500] [ALPM] upgraded nodejs (26.13.1-2 -> 26.13.1-3)
```

`readelf --dynamic` shows `DT_NEEDED` for `libsimdjson.so.36` in the November 17 helper binary
and in `/usr/bin/node` extracted from the cached CachyOS `nodejs 26.13.1-2` package.
The cached old `simdjson` package contains `.36`.
The installed `nodejs 26.13.1-3` needs `.37`, and the installed `simdjson 2:5.2.0-1.1` supplies `.37`.

The key-helper `sea` task invokes the build runtime at `package/kwin/key-helper/mise.toml:67`:

```ts
// package/kwin/key-helper/mise.toml:67
execFileSync(process.execPath, ['--build-sea', scratchConfigPath,], { stdio: 'inherit', },)
```

The config leaves the executable path unset.
Node v26.13.1 reads the running Node ELF and injects the bundled app into it.
The resulting binary retains the base executable's dynamic library dependencies.
The relevant excerpts are from `src/node_sea_bin.cc:418-459`. [Node source][node-sea-build]

```cpp
// src/node_sea_bin.cc:418-421, 437-438 and 458-459
  SeaConfig config = opt_config.value();
  if (config.executable_path.empty()) {
    config.executable_path = Environment::GetExecPath(args);
  }
  // ...
  std::vector<uint8_t> exe_data;
  r = ReadFileSync(config.executable_path.c_str(), &exe_data);
  // ...
  InjectOutput out = InjectResource(
      exe_data, kSEAResourceName, sea_blob_u8, kMachoSegmentName);
```

KDE's generated user unit pipes the helper's output to `systemd-cat`.
The shell pipeline returned the logger's status, so systemd reported `Result=success` and `ExecMainStatus=0`
for an inactive unit while the executable itself exited `127`.
The ELF loader stopped the process before it could register the D-Bus service.

## Verification

Run these checks from the repository root on the affected machine:

```sh
readelf --dynamic package/kwin/key-helper/key-helper-service
ldd package/kwin/key-helper/key-helper-service
pacman --query --list simdjson
journalctl --boot --identifier=key-helper --no-pager
busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus \
  NameHasOwner s org.monochromatic.KeyHelper
```

The private bus test ran the actual shortcut callback with a simulated Helium window and the actual helper executable.
A test `ydotool` recorded the arguments. The fixture had an empty `/dev/input` and no access to the host's input devices.

- Fails: the installed SEA on current libraries exited `127` with the journal error and never acquired the bus name.
- Works: the same installed SEA with the real cached `.36` library supplied only to the fixture acquired the name
  and handled the callback.
- Works: repackaging the existing JS bundle with the `sea` task under current Node produced a SEA that resolved `.37`,
  acquired the name, and handled the callback. Both passing cases emitted `key 29:1 17:1 17:0 29:0`.

## Verified workaround

Repackage the existing JS bundle against current Node. The `sea` task replaces the on-disk artifact atomically.
It embeds the existing bundle as-is. Use the package's `binary` task instead if the bundle also needs rebuilding.
On this installation the current-session start command targets the unit generated from the existing desktop entry.
The repackaging was verified on the private bus. Starting the unit and pressing the shortcut on the desktop remain open.

```sh
mise run //package/kwin/key-helper:sea
systemctl --user start 'app-key\x2dhelper\x2dservice@autostart.service'
```

## What does not work

The working `ydotoold` service and the loaded KWin script still left the shortcut without a D-Bus destination.
The generated unit's success status also missed the loader failure.
Running the same executable directly showed exit `127`. Piping it through `cat` reproduced the masked status `0`.

## Upstream filing decision

Node's v26.13.1 implementation embeds the app into a supplied executable, and the CachyOS upgrade refreshed the
packaged executable and library together. The stale executable was generated and installed locally.
The Node issue searches for `"build-sea" "shared libraries"` and `"build-sea" "simdjson"` returned no matches.
No `.out-of-scope/` entry matched this incident. There is no upstream issue or comment to file.

- Upstream fault: the observed SEA behavior follows the selected build runtime.
- Upstream remedy: the stale artifact is owned by the local build and can be regenerated there.
- Supported use case: building a SEA from the current executable is implemented in Node's source.
- Contribution policy and maintainer interest: those checks were unnecessary once the incident traced to the local artifact.
- Prototype: the local rebuild passed with `.37`. An upstream patch was unnecessary.

[node-sea-build]: https://github.com/nodejs/node/blob/e9c277643b0249c50f6e6de5413be8a88ea11b6a/src/node_sea_bin.cc#L418-L459
