# Install Firefox ESR and Nightly side by side on Fedora Atomic

## What this proves

This runbook installs Mozilla's current English (US) Firefox ESR and Firefox Nightly tarballs under the current
user's home directory.
It creates independent commands and desktop launchers without layering packages onto the immutable host image:

- `~/.local/opt/firefox-esr` and `~/.local/bin/firefox-esr`
- `~/.local/opt/firefox-nightly` and `~/.local/bin/firefox-nightly`
- `~/.local/share/applications/firefox-esr.desktop`
- `~/.local/share/applications/firefox-nightly.desktop`

The direct shell bridge was tried and worked.
The installation and headless browser verification require no GUI automation.
The final launcher,
 window grouping,
 and profile checks remain visual because they exercise the user's live desktop
session.

The procedure was verified on x86_64 Bazzite 44,
 a Fedora Kinoite derivative.
The same commands select Mozilla's `linux64-aarch64` artifacts on AArch64.
Verification covered both current product redirects,
 both detached OpenPGP signatures,
 archive layout,
 desktop-file
validation,
 executable versions,
 and headless page rendering.

Mozilla publishes the [ESR Linux downloads][firefox-esr-downloads] and
[Nightly Linux downloads][firefox-nightly-downloads] used here.
The desktop entries follow Mozilla's [Linux desktop template][mozilla-desktop-template].
Their filenames and `StartupWMClass` values match each build's `RemotingName`,
 as required for correct Firefox
identity on [Wayland desktops][mozilla-wayland-launcher] and especially [KDE Wayland][mozilla-kde-wayland].
Firefox selects defaults per install directory,
 so these distinct directories receive distinct default profiles,
as described by the [Firefox Profiles Service][firefox-profiles].

[firefox-esr-downloads]: https://www.firefox.com/en-US/download/all/desktop-esr/
[firefox-nightly-downloads]: https://www.firefox.com/en-US/channel/desktop/#nightly
[firefox-profiles]: https://firefox-source-docs.mozilla.org/toolkit/profile/
[mozilla-desktop-template]: https://github.com/mozilla/sumo-kb/blob/main/install-firefox-linux/firefox.desktop
[mozilla-kde-wayland]: https://bugzilla.mozilla.org/show_bug.cgi?id=1826330
[mozilla-wayland-launcher]: https://bugzilla.mozilla.org/show_bug.cgi?id=1751153

## Setup

Status:
TODO | DONE

Prerequisites include HTTPS access to Mozilla's download hosts and free space in the user's home directory for both
archives,
 extracted browsers,
 and any rollback copy retained from an earlier installation.

1.  Confirm that the host is Fedora Atomic or a Fedora-derived immutable desktop and uses a supported architecture.

    ```sh
    grep --fixed-strings --ignore-case 'fedora' /etc/os-release
    uname --machine
    ```

    Expect the first command to print a line such as `ID_LIKE="fedora"`.
    Expect the second command to print exactly `x86_64` or `aarch64`.
    Stop if either expectation fails.

2.  Quit any existing Firefox ESR and Firefox Nightly processes.
    In each browser,
     press **Ctrl+Q**.
    Then run:

    ```sh
    for channel in firefox-esr firefox-nightly; do
      if pgrep --full --list-full -- "$HOME/.local/opt/$channel/"; then
        printf 'STOP: %s is still running\n' "$channel"
      else
        printf '%s is not running\n' "$channel"
      fi
    done
    ```

    Expect exactly:

    ```text
    firefox-esr is not running
    firefox-nightly is not running
    ```

    Do not replace browser program files while either channel is running.

3.  Check every command used by this runbook.

    ```sh
    missing=0
    for command_name in \
      awk cat chmod curl date desktop-file-validate file gpg gpgv grep ln mkdir mktemp mv pgrep readlink rm tar \
      uname update-desktop-database
    do
      if command_path="$(command -v -- "$command_name")"; then
        printf '%s=%s\n' "$command_name" "$command_path"
      else
        printf 'missing=%s\n' "$command_name"
        missing=1
      fi
    done
    test "$missing" -eq 0
    ```

    Expect an absolute path for every command and no `missing=` line.
    Stop if any command is missing.
    This runbook requires the commands to be present in the host image and does not install prerequisite packages.

## Steps

Status:
TODO | DONE

1.  Move any prior side-by-side installation into a timestamped rollback directory.
    Profiles under `~/.mozilla/firefox` are deliberately not moved.

    ```sh
    (
      set -o errexit
      set -o nounset
      set -o pipefail

      state_root="$HOME/.local/state/firefox-side-by-side"
      mkdir --parents "$state_root"
      backup_root="$(
        mktemp --directory \
          --tmpdir="$state_root" \
          "backup-$(date --utc +%Y%m%dT%H%M%SZ).XXXXXXXX"
      )"
      mkdir --parents \
        "$backup_root/opt" \
        "$backup_root/bin" \
        "$backup_root/applications"
      printf '%s\n' "$backup_root" > "$state_root/latest-backup"

      if test -e "$HOME/.local/opt/firefox-esr"; then
        mv -- "$HOME/.local/opt/firefox-esr" "$backup_root/opt/"
      fi
      if test -e "$HOME/.local/opt/firefox-nightly"; then
        mv -- "$HOME/.local/opt/firefox-nightly" "$backup_root/opt/"
      fi
      if test -e "$HOME/.local/bin/firefox-esr" || test -L "$HOME/.local/bin/firefox-esr"; then
        mv -- "$HOME/.local/bin/firefox-esr" "$backup_root/bin/"
      fi
      if test -e "$HOME/.local/bin/firefox-nightly" || test -L "$HOME/.local/bin/firefox-nightly"; then
        mv -- "$HOME/.local/bin/firefox-nightly" "$backup_root/bin/"
      fi
      if test -e "$HOME/.local/share/applications/firefox-esr.desktop"; then
        mv -- "$HOME/.local/share/applications/firefox-esr.desktop" "$backup_root/applications/"
      fi
      if test -e "$HOME/.local/share/applications/firefox-nightly.desktop"; then
        mv -- "$HOME/.local/share/applications/firefox-nightly.desktop" "$backup_root/applications/"
      fi

      printf 'rollback=%s\n' "$backup_root"
    )
    ```

    Expect one `rollback=` line.
    Save that exact path until the new installation passes `What to check`.
    On a fresh machine,
     the rollback directory exists but contains no prior browser files.

2.  Download,
     authenticate,
     and install Firefox ESR.

    ```sh
    (
      set -o errexit
      set -o nounset
      set -o pipefail

      case "$(uname --machine)" in
        x86_64) mozilla_os=linux64 ;;
        aarch64) mozilla_os=linux64-aarch64 ;;
        *) printf 'Unsupported architecture: %s\n' "$(uname --machine)" >&2; exit 1 ;;
      esac

      product=firefox-esr-latest-ssl
      install_dir="$HOME/.local/opt/firefox-esr"
      tmp_dir="$(mktemp --directory)"
      trap 'rm --recursive --force -- "$tmp_dir"' EXIT
      archive="$tmp_dir/firefox-esr.tar.xz"

      resolved_url="$(
        curl --fail --show-error --location \
          --output "$archive" \
          --write-out '%{url_effective}' \
          "https://download.mozilla.org/?product=$product&os=$mozilla_os&lang=en-US"
      )"
      curl --fail --show-error --location \
        --output "$archive.asc" \
        "$resolved_url.asc"
      key_url="${resolved_url%%/linux-*}/KEY"
      curl --fail --show-error --location \
        --output "$tmp_dir/KEY" \
        "$key_url"

      expected_fingerprint=14F26682D0916CDD81E37B6D61B7B526D98F0353
      primary_fingerprints="$(
        gpg --show-keys --with-colons "$tmp_dir/KEY" \
          | awk --field-separator=: '
            $1 == "pub" { primary = 1; next }
            primary && $1 == "fpr" { print $10; primary = 0 }
          '
      )"
      test "$primary_fingerprints" = "$expected_fingerprint"
      printf '%s\n' "$primary_fingerprints"
      gpg --batch --yes --dearmor \
        --output "$tmp_dir/mozilla.gpg" \
        "$tmp_dir/KEY"
      gpgv --keyring "$tmp_dir/mozilla.gpg" "$archive.asc" "$archive"

      tar --extract --xz --file="$archive" --directory="$tmp_dir"
      test -x "$tmp_dir/firefox/firefox"
      grep --fixed-strings --line-regexp \
        'RemotingName=firefox-esr' \
        "$tmp_dir/firefox/application.ini"

      mkdir --parents "$HOME/.local/opt" "$HOME/.local/bin"
      mv -- "$tmp_dir/firefox" "$install_dir"
      ln --symbolic --force --no-target-directory \
        "$install_dir/firefox" \
        "$HOME/.local/bin/firefox-esr"
      "$HOME/.local/bin/firefox-esr" --version
    )
    ```

    Expect the fingerprint `14F26682D0916CDD81E37B6D61B7B526D98F0353`,
    `Good signature from "Mozilla Software Releases <release@mozilla.com>"`,
    `RemotingName=firefox-esr`,
     and a version ending in `esr`.
    Stop and use `Restore` if the fingerprint or signature check fails.

3.  Download,
     authenticate,
     and install Firefox Nightly.

    ```sh
    (
      set -o errexit
      set -o nounset
      set -o pipefail

      case "$(uname --machine)" in
        x86_64) mozilla_os=linux64 ;;
        aarch64) mozilla_os=linux64-aarch64 ;;
        *) printf 'Unsupported architecture: %s\n' "$(uname --machine)" >&2; exit 1 ;;
      esac

      product=firefox-nightly-latest-ssl
      install_dir="$HOME/.local/opt/firefox-nightly"
      tmp_dir="$(mktemp --directory)"
      trap 'rm --recursive --force -- "$tmp_dir"' EXIT
      archive="$tmp_dir/firefox-nightly.tar.xz"

      resolved_url="$(
        curl --fail --show-error --location \
          --output "$archive" \
          --write-out '%{url_effective}' \
          "https://download.mozilla.org/?product=$product&os=$mozilla_os&lang=en-US"
      )"
      curl --fail --show-error --location \
        --output "$archive.asc" \
        "$resolved_url.asc"
      key_url="${resolved_url%/*}/KEY"
      curl --fail --show-error --location \
        --output "$tmp_dir/KEY" \
        "$key_url"

      expected_fingerprint=14F26682D0916CDD81E37B6D61B7B526D98F0353
      primary_fingerprints="$(
        gpg --show-keys --with-colons "$tmp_dir/KEY" \
          | awk --field-separator=: '
            $1 == "pub" { primary = 1; next }
            primary && $1 == "fpr" { print $10; primary = 0 }
          '
      )"
      test "$primary_fingerprints" = "$expected_fingerprint"
      printf '%s\n' "$primary_fingerprints"
      gpg --batch --yes --dearmor \
        --output "$tmp_dir/mozilla.gpg" \
        "$tmp_dir/KEY"
      gpgv --keyring "$tmp_dir/mozilla.gpg" "$archive.asc" "$archive"

      tar --extract --xz --file="$archive" --directory="$tmp_dir"
      test -x "$tmp_dir/firefox/firefox"
      grep --fixed-strings --line-regexp \
        'RemotingName=firefox-nightly' \
        "$tmp_dir/firefox/application.ini"

      mkdir --parents "$HOME/.local/opt" "$HOME/.local/bin"
      mv -- "$tmp_dir/firefox" "$install_dir"
      ln --symbolic --force --no-target-directory \
        "$install_dir/firefox" \
        "$HOME/.local/bin/firefox-nightly"
      "$HOME/.local/bin/firefox-nightly" --version
    )
    ```

    Expect the same Mozilla fingerprint and good-signature string,
    `RemotingName=firefox-nightly`,
     and a version ending in `a1`.
    Stop and use `Restore` if the fingerprint or signature check fails.

4.  Create the Firefox ESR desktop entry.

    ```sh
    mkdir --parents "$HOME/.local/share/applications"
    cat > "$HOME/.local/share/applications/firefox-esr.desktop" <<EOF
    [Desktop Entry]
    Version=1.0
    Name=Firefox ESR
    GenericName=Web Browser
    Comment=Browse the World Wide Web
    Keywords=Internet;WWW;Browser;Web
    Exec=$HOME/.local/opt/firefox-esr/firefox %u
    Icon=$HOME/.local/opt/firefox-esr/browser/chrome/icons/default/default128.png
    Terminal=false
    Type=Application
    Categories=Network;WebBrowser;
    StartupNotify=true
    StartupWMClass=firefox-esr
    EOF
    ```

    Expect `~/.local/share/applications/firefox-esr.desktop` to exist.
    Its filename and `StartupWMClass` both match ESR's `firefox-esr` remoting name.

5.  Create the Firefox Nightly desktop entry.

    ```sh
    cat > "$HOME/.local/share/applications/firefox-nightly.desktop" <<EOF
    [Desktop Entry]
    Version=1.0
    Name=Firefox Nightly
    GenericName=Web Browser
    Comment=Browse the World Wide Web
    Keywords=Internet;WWW;Browser;Web
    Exec=$HOME/.local/opt/firefox-nightly/firefox %u
    Icon=$HOME/.local/opt/firefox-nightly/browser/chrome/icons/default/default128.png
    Terminal=false
    Type=Application
    Categories=Network;WebBrowser;
    StartupNotify=true
    StartupWMClass=firefox-nightly
    EOF
    ```

    Expect `~/.local/share/applications/firefox-nightly.desktop` to exist.
    Its filename and `StartupWMClass` both match Nightly's `firefox-nightly` remoting name.

6.  Validate and register both desktop entries.

    ```sh
    chmod 0644 \
      "$HOME/.local/share/applications/firefox-esr.desktop" \
      "$HOME/.local/share/applications/firefox-nightly.desktop"
    desktop-file-validate \
      "$HOME/.local/share/applications/firefox-esr.desktop" \
      "$HOME/.local/share/applications/firefox-nightly.desktop"
    update-desktop-database "$HOME/.local/share/applications"
    printf 'Desktop entries registered\n'
    ```

    Expect no validator diagnostic and then exactly `Desktop entries registered`.
    This runbook does not change the default browser or MIME-handler selection.

## What to check

Status:
TODO | DONE

1.  Check versions,
     symlinks,
     remoting names,
     and desktop identities.

    ```sh
    "$HOME/.local/bin/firefox-esr" --version
    "$HOME/.local/bin/firefox-nightly" --version
    readlink -- "$HOME/.local/bin/firefox-esr"
    readlink -- "$HOME/.local/bin/firefox-nightly"
    grep --fixed-strings --line-regexp \
      'RemotingName=firefox-esr' \
      "$HOME/.local/opt/firefox-esr/application.ini"
    grep --fixed-strings --line-regexp \
      'RemotingName=firefox-nightly' \
      "$HOME/.local/opt/firefox-nightly/application.ini"
    grep --fixed-strings --line-regexp \
      'StartupWMClass=firefox-esr' \
      "$HOME/.local/share/applications/firefox-esr.desktop"
    grep --fixed-strings --line-regexp \
      'StartupWMClass=firefox-nightly' \
      "$HOME/.local/share/applications/firefox-nightly.desktop"
    ```

    Expect the ESR version to end in `esr` and the Nightly version to end in `a1`.
    Expect the symlink targets to be exactly:

    ```text
    /home/<user>/.local/opt/firefox-esr/firefox
    /home/<user>/.local/opt/firefox-nightly/firefox
    ```

    Expect each `RemotingName` and `StartupWMClass` line to print exactly as requested.

2.  Render a local page through each installed browser using disposable profiles.

    ```sh
    (
      set -o errexit
      set -o nounset
      set -o pipefail

      tmp_dir="$(mktemp --directory)"
      trap 'rm --recursive --force -- "$tmp_dir"' EXIT
      printf '<!doctype html><title>Firefox check</title><h1>Firefox works</h1>\n' > "$tmp_dir/index.html"

      for binary in \
        "$HOME/.local/bin/firefox-esr" \
        "$HOME/.local/bin/firefox-nightly"
      do
        name="${binary##*/}"
        mkdir --parents "$tmp_dir/$name-profile"
        "$binary" \
          --headless \
          --new-instance \
          --profile "$tmp_dir/$name-profile" \
          --screenshot "$tmp_dir/$name.png" \
          "file://$tmp_dir/index.html"
        test -s "$tmp_dir/$name.png"
        file "$tmp_dir/$name.png"
      done
    )
    ```

    Expect `PNG image data` once for `firefox-esr.png` and once for `firefox-nightly.png`.
    The disposable profiles and screenshots disappear when the subshell exits.

3.  Open the desktop application launcher with **Meta** and search for **Firefox ESR**.
    Expect exactly one launcher named `Firefox ESR` with the ESR icon.

4.  Launch **Firefox ESR**.
    Expect an ESR window to open and remain grouped under its own icon in the desktop task manager or dock.

5.  Open the desktop application launcher with **Meta** and search for **Firefox Nightly**.
    Expect exactly one launcher named `Firefox Nightly` with the Nightly icon.

6.  Launch **Firefox Nightly** while ESR remains open.
    Expect a separate Nightly window grouped under the Nightly icon,
     not the ESR icon.

7.  In each browser,
     focus the address bar with **Ctrl+L**,
     enter **`about:support`**,
     and press **Enter**.
    Locate **Profile Directory** in each page.
    Expect different profile directory paths for ESR and Nightly.

Rerunning `Setup` and `Steps` refreshes both channels from Mozilla's current product endpoints and retains the prior
program files in another timestamped rollback directory.

## Restore

Status:
TODO | DONE

1.  Quit Firefox ESR and Firefox Nightly with **Ctrl+Q**.
    Rerun Setup's process check and expect both `is not running` lines.

2.  Print the rollback directory recorded by the latest run.

    ```sh
    state_root="$HOME/.local/state/firefox-side-by-side"
    backup_root="$(cat "$state_root/latest-backup")"
    printf 'rollback=%s\n' "$backup_root"
    find "$backup_root" -mindepth 2 -maxdepth 2 -print
    ```

    Expect the same `rollback=` path printed during Steps.
    A fresh installation prints no paths after that line.
    A replacement installation lists whichever prior program files,
     symlinks,
     and desktop entries existed.

3.  Remove only the program files,
     symlinks,
     and desktop entries created by this runbook,
     then restore files from the
    rollback directory when present.

    ```sh
    (
      set -o errexit
      set -o nounset
      set -o pipefail

      state_root="$HOME/.local/state/firefox-side-by-side"
      backup_root="$(cat "$state_root/latest-backup")"

      rm --recursive --force -- \
        "$HOME/.local/opt/firefox-esr" \
        "$HOME/.local/opt/firefox-nightly"
      rm --force -- \
        "$HOME/.local/bin/firefox-esr" \
        "$HOME/.local/bin/firefox-nightly" \
        "$HOME/.local/share/applications/firefox-esr.desktop" \
        "$HOME/.local/share/applications/firefox-nightly.desktop"

      if test -e "$backup_root/opt/firefox-esr"; then
        mv -- "$backup_root/opt/firefox-esr" "$HOME/.local/opt/"
      fi
      if test -e "$backup_root/opt/firefox-nightly"; then
        mv -- "$backup_root/opt/firefox-nightly" "$HOME/.local/opt/"
      fi
      if test -e "$backup_root/bin/firefox-esr" || test -L "$backup_root/bin/firefox-esr"; then
        mv -- "$backup_root/bin/firefox-esr" "$HOME/.local/bin/"
      fi
      if test -e "$backup_root/bin/firefox-nightly" || test -L "$backup_root/bin/firefox-nightly"; then
        mv -- "$backup_root/bin/firefox-nightly" "$HOME/.local/bin/"
      fi
      if test -e "$backup_root/applications/firefox-esr.desktop"; then
        mv -- "$backup_root/applications/firefox-esr.desktop" "$HOME/.local/share/applications/"
      fi
      if test -e "$backup_root/applications/firefox-nightly.desktop"; then
        mv -- "$backup_root/applications/firefox-nightly.desktop" "$HOME/.local/share/applications/"
      fi

      update-desktop-database "$HOME/.local/share/applications"
      rm --recursive --force -- "$backup_root"
      rm --force -- "$state_root/latest-backup"
      printf 'Rollback complete\n'
    )
    ```

    Expect the final output line to be exactly `Rollback complete`.
    On a fresh installation this uninstalls both channels.
    On a replacement installation it restores the files moved by Steps.

4.  Preserve `~/.mozilla/firefox` unless deleting browser profiles and user data is an independently authorized goal.
    This runbook never removes that directory because it can contain bookmarks,
     history,
     logins,
     extensions,
     and
    profiles owned by other Firefox installations.
