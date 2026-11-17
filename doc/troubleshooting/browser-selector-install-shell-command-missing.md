# Browser Selector v0.2.0 reports a successful install and Bash cannot find the command

Non-final troubleshooting note. Codex reproduced the shell failure, checked both URL defaults,
and verified URL dispatch with a disposable KDE profile. The live chooser window was untested.

## Symptom

`./target/release/browser-selector --install` prints `Browser Selector now opens http and https links.`
The next invocation, `browser-selector 'https://example.com'`, prints
`bash: command not found: browser-selector` in the reported shell.
The same shell reported `bash: command not found: open`, `bash: command not found: gtk-open`,
and `bash: command not found: kio-open` for those command names.

## Root cause

The installer reads the current executable path and writes a desktop entry under
`~/.local/share/applications/`. [`src/install.rs:62`][write-launcher]

```rust
// src/install.rs:62
pub fn install(config: &mut Config) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let applications = data_home().join("applications");
    let icons = data_home().join("icons/hicolor");
    std::fs::create_dir_all(&applications).map_err(|e| e.to_string())?;
    std::fs::write(applications.join(desktop_filename()), desktop_entry(&executable)).map_err(|e| e.to_string())?;
```

The desktop entry's `Exec=` value names the executable at its absolute path, followed by
`%u` for the URL. [`src/install.rs:30`][desktop-exec]

The installer sets the default desktop entry for each URL scheme and saves the prior handlers.
The complete install function performs desktop registration and writes metadata and icons.
Bash resolves names through its own command lookup, so it can still fail to find the binary.
[`src/install.rs:62`][write-launcher] [`src/install.rs:78`][set-defaults]

```rust
// src/install.rs:84
for mime in SCHEMES {
    run(&["xdg-mime", "default", &desktop_filename(), mime])?;
}
config.save().map_err(|e| e.to_string())
```

The upstream README explains the binary path dependency and shows bare commands in its usage section.
Those commands assume the executable is stored in a directory on `PATH`.
[`README.md:25`][readme-install] [`README.md:41`][readme-usage]

## Verification

On 2026-11-17, the v0.2.0 binary built from commit
`164028b43c664b4aa72015ae0432c9205be1c692` reproduced the error after `--install` in a disposable
home with `PATH=/usr/bin`. A fresh Bash exited with status `127` and printed
`bash: line 1: browser-selector: command not found`.
The same install set both URL associations to `de.bmevers.BrowserSelector.desktop`.

From the checkout, these read-only commands check both lookup paths.

```bash
# Run from the browser-chooser checkout.
command -v browser-selector open gtk-open kio-open xdg-open
./target/release/browser-selector --help
xdg-mime query default x-scheme-handler/http
xdg-mime query default x-scheme-handler/https
rg '^Exec=' "$HOME/.local/share/applications/de.bmevers.BrowserSelector.desktop"
```

On the affected KDE session, `/usr/bin/xdg-open` was present, the other queried names were absent,
and both defaults matched the desktop ID. The entry pointed at the release binary in the checkout.
In the disposable KDE profile, `xdg-open` 1.2.1 delivered the exact HTTPS URL to a recording
executable in the registered desktop entry. The recorder ran asynchronously.
The commit-pinned selections were checked against the local clone and GitHub source API.
`agent-browser` was unavailable in the session.

## Verified workarounds

- Run `xdg-open 'https://example.com'` to exercise the desktop URL association.
- Run `./target/release/browser-selector 'https://example.com'` from the checkout to invoke the
  binary directly. The `--help` invocation verified this executable path from Bash.
- For a command available across directories, copy the binary into a directory on `PATH`
  and repeat registration from that copy. The disposable install verified both lookups after this change.

```bash
# From the browser-chooser checkout, with $HOME/.local/bin on PATH.
install -D --mode=755 ./target/release/browser-selector "$HOME/.local/bin/browser-selector"
"$HOME/.local/bin/browser-selector" --install
```

Keep the registered binary at that path. Copying it alone leaves the existing desktop entry pointing
at the checkout, where removing build output can break URL dispatch.

## Upstream filing decision

GitHub issue and pull request searches for `"command not found" OR "xdg-open" OR "PATH"`
found no matches on 2026-11-17. The installer performed the documented registration.
Upstream could clarify the usage example by showing a `PATH` installation or an explicit path.
The README covers desktop registration and assumes a shell-visible binary for the usage example.
The verified remedy is at the caller boundary. Contribution policy and maintainer acceptance were
outside this diagnosis, and no upstream patch was prototyped. There is no upstream filing artifact.

[write-launcher]: https://github.com/lava/browser-chooser/blob/164028b43c664b4aa72015ae0432c9205be1c692/src/install.rs#L62-L88
[desktop-exec]: https://github.com/lava/browser-chooser/blob/164028b43c664b4aa72015ae0432c9205be1c692/src/install.rs#L30-L38
[set-defaults]: https://github.com/lava/browser-chooser/blob/164028b43c664b4aa72015ae0432c9205be1c692/src/install.rs#L78-L87
[readme-install]: https://github.com/lava/browser-chooser/blob/164028b43c664b4aa72015ae0432c9205be1c692/README.md#L25-L29
[readme-usage]: https://github.com/lava/browser-chooser/blob/164028b43c664b4aa72015ae0432c9205be1c692/README.md#L41-L47
