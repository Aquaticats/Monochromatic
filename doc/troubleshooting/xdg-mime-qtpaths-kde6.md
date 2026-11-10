# xdg-utils 1.2.1 on KDE 6: Browsers installation emits `qtpaths` not found while defaults still register

## Symptom

Running the Browsers universal Linux installer under Plasma 6 prints one warning for each URL scheme it registers:

```text
/usr/bin/xdg-mime: line 885: qtpaths: command not found
/usr/bin/xdg-mime: line 885: qtpaths: command not found
Browsers has been installed. Enjoy!
```

The installer asks `xdg-mime` to register both `x-scheme-handler/https` and
`x-scheme-handler/http`,
 which accounts for the repeated warning.
The warning does not mean that Qt 6 is absent.
The observed Fedora 44 host has `/usr/bin/qtpaths6`,
 but not an unversioned `qtpaths` command.

## Root cause

The observed system package is Fedora `xdg-utils 1.2.1-5.fc44`.
Its source corresponds to upstream tag `v1.2.1`,
 commit
`356c380ad6fecc9ce6bea1f6a77986ba67402c80`.

### The KDE helper calls an unavailable executable

On KDE 5 or newer,
 `make_default_kde` obtains the configuration directory by invoking the unversioned
`qtpaths` command.
Upstream `xdg-utils` v1.2.1,
 `scripts/xdg-mime.in:167-171`:

```sh
vendor="$1"
mimetype="$2"
if [ "${KDE_SESSION_VERSION:-0}" -gt 4 ] ; then
    default_dir="$(qtpaths --writable-path ConfigLocation)"
    default_file="$default_dir/mimeapps.list"
```

The host reports `KDE_SESSION_VERSION=6` and has only `/usr/bin/qtpaths6`.
The command substitution therefore emits `qtpaths: command not found` and leaves `default_dir` empty.

### The generic helper still performs the required registration

`xdg-mime default` invokes both the KDE helper and the generic helper for every desktop except LXQt.
Upstream v1.2.1,
 `scripts/xdg-mime.in:666-674`:

```sh
case "$DE" in
    lxqt)
    make_default_lxqt "$filename" "$mimetype"
    ;;

    *)
    make_default_kde "$filename" "$mimetype"
    make_default_generic "$filename" "$mimetype"
    ;;
esac
```

The generic helper writes the default to `XDG_CONFIG_HOME/mimeapps.list`,
 falling back to
`$HOME/.config/mimeapps.list`.
Upstream v1.2.1,
 `scripts/xdg-mime.in:292-308`:

```sh
make_default_generic()
{
    # $1 is vendor-name.desktop
    # $2 is mime/type
    # Add $2=$1 to XDG_CONFIG_HOME/mimeapps.list
    xdg_config_home="$XDG_CONFIG_HOME"
    [ -n "$xdg_config_home" ] || xdg_config_home="$HOME/.config"
    default_file="$xdg_config_home/mimeapps.list"
    # ...
    [ -f "$out_file" ] || touch "$out_file"
```

`qtpaths6 --writable-path ConfigLocation` returns `/home/user/.config` on the observed host.
The KDE and generic helpers therefore target the same `mimeapps.list` file for the default application.
The KDE helper would also add an `[Added Associations]` entry,
 but that addition is unnecessary here:
`software.Browsers.desktop` already declares both URL schemes in its `MimeType` key,
 and
`update-desktop-database` registered both associations.

The [MIME Applications specification][mime-associations] defines `[Added Associations]` as associations
added as if the desktop file listed the type itself.
The installed desktop file already lists the types:

```ini
MimeType=x-scheme-handler/http;x-scheme-handler/https;
```

### Upstream removed the dependency

Upstream commit `e6a6e4f1fbcb029bac0cb8eecdeb2879694e1ba8`,
 committed 2025-02-07,
 fixes issue
[xdg-utils#258][upstream-issue].
Current `scripts/xdg-mime.in:183-187` uses a shell helper instead of `qtpaths`:

```sh
vendor="$1"
mimetype="$2"
if [ "${KDE_SESSION_VERSION:-0}" -gt 4 ] ; then
    default_dir="$(get_xdg_config_home)"
    default_file="$default_dir/mimeapps.list"
```

Current `scripts/xdg-utils-common.in:599-606` resolves the XDG configuration directory directly:

```sh
get_xdg_config_home()
{
    # Only use XDG_CONFIG_HOME if it is an absolute path
    case "$XDG_CONFIG_HOME" in
        /*) printf "%s\n" "$XDG_CONFIG_HOME" ;;
        *) printf "%s\n" "$HOME/.config" ;;
    esac
}
```

Fedora 44 still packages the older v1.2.1 implementation without that commit.
The Fedora Rawhide spec for `1.2.1-6.fc45` also lists no patch for this change,
 so a newer Fedora rebuild of the
same upstream release is not evidence that the warning is fixed.

## Verification

Verified 2026-08-14 with:

- Fedora `xdg-utils 1.2.1-5.fc44`.
- `xdg-mime 1.2.1`.
- Plasma with `KDE_SESSION_VERSION=6`.
- Browsers desktop entry at `~/.local/share/applications/software.Browsers.desktop`.
- Upstream `xdg-utils` v1.2.1 commit `356c380ad6fecc9ce6bea1f6a77986ba67402c80`.
- Upstream fix commit `e6a6e4f1fbcb029bac0cb8eecdeb2879694e1ba8`.

A disposable-home reproduction produced the warning,
 returned status zero,
 wrote the default,
 and queried it
successfully:

```sh
fixture_root="$(mktemp --directory)"
mkdir --parents "$fixture_root/config" "$fixture_root/data/applications"
printf '%s\n' \
  '[Desktop Entry]' \
  'Type=Application' \
  'Name=Fixture' \
  'Exec=/usr/bin/true' \
  'MimeType=x-scheme-handler/xdg-mime-diagnostic;' \
  > "$fixture_root/data/applications/fixture.desktop"

env \
  HOME="$fixture_root/home" \
  XDG_CONFIG_HOME="$fixture_root/config" \
  XDG_DATA_HOME="$fixture_root/data" \
  XDG_CURRENT_DESKTOP=KDE \
  KDE_SESSION_VERSION=6 \
  xdg-mime default fixture.desktop x-scheme-handler/xdg-mime-diagnostic

env \
  HOME="$fixture_root/home" \
  XDG_CONFIG_HOME="$fixture_root/config" \
  XDG_DATA_HOME="$fixture_root/data" \
  XDG_CURRENT_DESKTOP=KDE \
  KDE_SESSION_VERSION=6 \
  xdg-mime query default x-scheme-handler/xdg-mime-diagnostic
```

Observed output:

```text
/usr/bin/xdg-mime: line 885: qtpaths: command not found
fixture.desktop
```

The set command returned zero,
 and the generated `mimeapps.list` contained:

```ini
[Default Applications]
x-scheme-handler/xdg-mime-diagnostic=fixture.desktop
```

### Operations that warn

- `xdg-mime default ...` with `KDE_SESSION_VERSION` greater than 4 and no unversioned `qtpaths` in `PATH`.
- One invocation with multiple MIME types,
   which emits one warning per type.
- Future installers that use this packaged `xdg-mime` path,
   even when registration succeeds.

### Operations that work cleanly after the warning

- `xdg-mime query default x-scheme-handler/http` returns `software.Browsers.desktop`.
- `xdg-mime query default x-scheme-handler/https` returns `software.Browsers.desktop`.
- `gio mime` reports Browsers as default,
   registered,
   and recommended for both URL schemes.
- `desktop-file-validate ~/.local/share/applications/software.Browsers.desktop` succeeds.
- Browsers starts and logs both scheme defaults as `software.Browsers.desktop`.

## Verified workarounds

### Ignore the warning after checking the resulting defaults

```sh
xdg-mime query default x-scheme-handler/http
xdg-mime query default x-scheme-handler/https
```

Both commands should print:

```text
software.Browsers.desktop
```

Tradeoff:
 future `xdg-mime default` callers can print the same warning until the packaged script includes the
upstream fix.
An installer that incorrectly treats any stderr as failure could report a false failure even though
`xdg-mime` returns zero.

### Use `qtpaths6` as a temporary compatibility command

A disposable fixture verified that placing a `qtpaths` symlink to `/usr/bin/qtpaths6` first in `PATH` suppresses
the warning and lets both helpers complete:

```sh
mkdir --parents "$fixture_root/bin"
ln --symbolic /usr/bin/qtpaths6 "$fixture_root/bin/qtpaths"
PATH="$fixture_root/bin:$PATH" xdg-mime default fixture.desktop x-scheme-handler/example
```

Tradeoff:
 a persistent user-level symlink changes command resolution for every caller and can become stale after
packaging changes.
It is unnecessary when the generic registration already succeeds,
 so ignoring the warning is preferred.

### Use a packaged version containing the upstream fix

A distribution package based on upstream commit `e6a6e4f1` or newer removes the `qtpaths` dependency.

Tradeoff:
 Fedora's v1.2.1 rebuilds do not currently contain that commit.
Replacing `/usr/bin/xdg-mime` manually would bypass package management and is not recommended.

## What does not work

- Treating the warning alone as proof that MIME registration failed.
  The reproduced command returned zero and the query found the new default.
- Installing more Qt 6 components merely to obtain `qtpaths`.
  The host already has `qtpaths6`;
   the packaged script asks for a different executable name.
- Looking only for `[Added Associations]` in `mimeapps.list`.
  Browsers already declares both URL schemes in its desktop entry,
   and the observed MIME database reports it as
  registered and recommended.
- Assuming a higher Fedora package release number contains the upstream fix.
  Fedora Rawhide `1.2.1-6.fc45` is a rebuild of the same upstream release and its spec contains no relevant patch.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers `xdg-utils`,
 MIME registration,
 or KDE command naming.

The exact diagnostic and cause already exist as [xdg-utils#258][upstream-issue].
Upstream commit `e6a6e4f1` closes that issue by removing the `qtpaths` dependency.
The NixOS tracker also contains the matching downstream report
[NixOS/nixpkgs#338263][nix-issue].
No new upstream issue or comment would add reproduction,
 root-cause,
 fix,
 or affected-version information.

The required filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
     Yes for xdg-utils v1.2.1.
    It invokes an executable that Qt packaging does not guarantee in `PATH`.
2.  **Can upstream fix it?**
     Yes.
    Commit `e6a6e4f1` resolves the configuration directory in shell.
3.  **Are they supporting this use case?**
     Yes.
    The project describes `xdg-mime` as its MIME association and default-application utility.
4.  **Would the repo welcome our contribution?**
     Yes.
    The repository has a public issue template and accepted the existing fix.
    No contribution or AI-assistance prohibition appears in `README.md` or `.gitlab/issue_templates/Default.md`.
5.  **Will they likely fix it?**
     Yes,
     and the fix is already on the upstream branch.
6.  **Have we prototyped a compatible minimal fix?**
     Yes.
    The landed commit is the minimal compatible implementation,
     and the disposable compatibility-command probe
    independently confirms that the failing call is the only source of the warning.

### Nothing to add

Do not file a new issue or comment.
The existing upstream issue and merged fix contain the complete actionable information.
The remaining Fedora release lag is not a new xdg-utils defect,
 and the observed Browsers installation completed
its intended registration.

[mime-associations]: https://specifications.freedesktop.org/mime-apps-spec/latest/associations.html
[nix-issue]: https://github.com/NixOS/nixpkgs/issues/338263
[upstream-issue]: https://gitlab.freedesktop.org/xdg/xdg-utils/-/issues/258
