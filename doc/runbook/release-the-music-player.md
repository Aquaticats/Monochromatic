# Release the music player

Use this to publish a GitHub pre-release of the music player (`package/music-player`),
to rehearse one with a dry run,
to recover when a release run fails,
and to rotate or restore the Android signing key.
The design and its reasons are in `doc/decision/music-player-releases.md`.
The workflow is `.github/workflows/music-player-release.yml`.

## What a release is

One release holds one file per platform and a checksum file:

- `music-player-<version>-windows-x64.exe` and `music-player-<version>-windows-arm64.exe` (linked by Visual Studio)
- `music-player-<version>-windows-x64-gnu.exe` (built with MinGW-w64 and GNU ld,
   no Microsoft toolchain)
- `music-player-<version>-macos-arm64.dmg` (there is no Intel Mac build)
- `music-player-<version>-linux-x64.tar.gz` and `music-player-<version>-linux-arm64.tar.gz` (glibc)
- `music-player-<version>-linux-musl-x64.tar.gz` and `music-player-<version>-linux-musl-arm64.tar.gz` (Alpine, musl)
- `music-player-<version>-android-arm64-v8a.apk` and `music-player-<version>-android-x86_64.apk`
  (one APK per ABI, no universal APK; phones, tablets and foldables take the `arm64-v8a` one)
- `music-player-<version>-desktop-third-party-licenses.txt` and
  `music-player-<version>-android-third-party-licenses.txt`
  (license notices;
  see [License notices](#license-notices))
- `SHA256SUMS`

The tag is `music-player-v<version>`.
Every release is flagged as a pre-release.
Every file is attested with GitHub build provenance.

Releases are immutable in this repository (a repository setting).
After a release is published,
its tag and its files can never be replaced or added to,
so a bad release is fixed only by releasing a higher version.
This is why the first real release of a new version is a deliberate act and not an experiment.

## The version

The product has one version.
Its single source is the `version` line of the `[package]` table in
`package/music-player/desktop-app/Cargo.toml`.
The Android app reads it at build time:
`versionName` is the version,
and `versionCode` is `major * 10000 + minor * 100 + patch` (version `0.1.0` is code `100`, `1.2.3` is code `10203`).

The rules the workflow enforces, and fails loudly on:

- The version is plain `MAJOR.MINOR.PATCH`, with minor and patch at most 99.
  A suffix such as `-beta.1` is refused, because it would give two versions the same Android code.
- `package/music-player/android-app/rust/Cargo.toml` carries the same version.
  Bump both files together; the run fails on a mismatch.
- On a push, the new version is higher than the old one.

## Cut a release

1.  Pick the new version and edit both manifests by hand:
    the `version` line in `package/music-player/desktop-app/Cargo.toml`
    and the one in `package/music-player/android-app/rust/Cargo.toml`.
    Both files are managed by `file-enforcer`, but it does not own `version`:
    `buildCargoManifestPlan` in `file-enforcer.config.ts` lists edition, license, publish, lints, shared dependencies,
    repository, readme, homepage and profiles, and in a throwaway checkout with both versions bumped
    `node file-enforcer.config.ts` left the bumps in place.
    Run it after the edit anyway, as for any manifest change, and commit whatever it rewrites.
2.  Refresh both lock files without touching any dependency,
    because the release builds with `--locked` and fails on a stale `Cargo.lock`:
    run `cargo update --workspace` in `package/music-player/desktop-app`
    and in `package/music-player/android-app/rust`.
    Each lock file changes by exactly one line (the crate's own `version` in its `[[package]]` entry;
    checked offline on both crates, and `cargo metadata --locked --offline` then resolves).
3.  Commit the four files and merge to `main`.
    A push to `main` that changes either manifest starts the workflow.
    The detect job compares the old and new version and starts the build only if the version changed.
4.  Watch the run:
    `gh run list --workflow music-player-release.yml` and `gh run watch <run id>`.
    A full run is the desktop builds (the two Alpine builds are the slowest, because they are never cached)
    in parallel with the Android build,
    then a checksum job and the release job.
5.  Open the release page `https://github.com/Aquaticat/Monochromatic/releases/tag/music-player-v<version>`
    and check that every file listed above and `SHA256SUMS` are attached and that it is marked as a pre-release.
6.  Verify one file as a user would (see the section on verifying attestations).

If the tag already exists when the run starts,
the run skips the release and succeeds with a notice.
It never overwrites a tag.

## The first release of an existing version

The version `0.1.0` was in `main` before the workflow existed,
so no push will ever release it:
a push releases only when the version changes.
Release it by dispatching a real run from `main`,
after a dry run has passed on `main`:

```bash
# Rehearsal first: builds and signs everything, creates nothing.
gh workflow run music-player-release.yml --ref main --field dry-run=true

# The real release. Immutable: do this only after the dry run is green and you have read its output.
gh workflow run music-player-release.yml --ref main --field dry-run=false
```

## Dry run

A dry run builds, packages and signs every file,
runs the same checks as a real run,
uploads the files as workflow artifacts that expire after one day,
and prints in the `release` job log the exact `gh release create` command,
the tag, the asset list and the release notes it would use.
It skips attestation and creates no tag and no release.

Start one with `gh workflow run music-player-release.yml --ref <branch> --field dry-run=true`
(the workflow file must exist on `main` before `gh workflow run` can find it).
On any ref other than `main` the Android app is signed with a throwaway key made for that run,
and the run refuses `dry-run=false`.
On `main` a dry run signs with the real release key, so the signing path is proven without publishing.

Download a dry run's files with `gh run download <run id> --dir <new empty directory>`.
Each `asset-*` artifact becomes a subdirectory.

Check the Alpine files of a dry run in a second new empty directory:

```bash
# Extract the x64 Alpine archive, then print the program interpreter of its binary.
tar --extract --gzip --file <download dir>/asset-linux-musl-x64/music-player-<version>-linux-musl-x64.tar.gz \
  --directory <extract dir>
readelf --program-headers <extract dir>/music-player-<version>-linux-musl-x64/bin/music-player
```

The interpreter line must read `/lib/ld-musl-x86_64.so.1`,
and `file` on the arm64 binary must say `ARM aarch64`.
The job has already started both binaries with `--version` in an Alpine image that holds only the packages of
`INSTALL.txt`.
To repeat that by hand,
mount the binary into `alpine:3.24`, run the `apk add` line of `INSTALL.txt` first,
and bound the container with `podman run --memory=2g --cpus=2 --rm`.

Check the Windows files of a dry run the same way,
 in a new empty directory.
The three `.exe` files print their imports in the log of the step "Package (Windows executable)",
and the step "Check the linker rustc ran (Windows)" prints the linker program:
 `link.exe` for the two MSVC files,
`x86_64-w64-mingw32-gcc` (GNU ld) for `windows-x64-gnu.exe`.
To repeat the import check on a download,
 parse the import tables with a PE reader such as the Python `pefile` package
in a throwaway virtual environment (`objdump -p` from mingw-w64 also works).
The MSVC files must import no `vcruntime*` or `msvcp*` DLL;
the GNU file imports the Windows C runtime (`api-ms-win-crt-*`) and system DLLs,
and must import none of `libgcc_s*`,
 `libwinpthread*` and `libstdc++*`.
The same step prints the `subsystem` value of each file and fails unless it is 2 (Windows GUI),
because a console-subsystem file (3) opens a console window on a double-click;
`objdump -p` prints the same field as `Subsystem`.

Check the license notices of a dry run:

- The `notices` job log lists,
  for every desktop target,
  how many third-party crates it links and that all are covered,
  then the size of the file.
- The `notices` job log prints cargo-about's ten warnings `LicenseRef-Slint-Royalty-free-2.0 has no license file`.
  They are expected: the script adds the Slint license text itself.
  The desktop file has a heading starting `Slint (10 crates): Slint Royalty-free Desktop`.
- Every desktop job's step "Smoke test the binary (--version)" compares `--licenses` with that file byte for byte.
- The Linux and Alpine archives must hold `THIRD-PARTY-LICENSES.txt` next to `LICENSE`
  (`tar --list --file <archive>`),
  and the disk image must hold it at its root and in `Music Player.app/Contents/Resources`.
- The APK must hold `assets/THIRD-PARTY-LICENSES.txt`
  (`unzip -l <apk> assets/THIRD-PARTY-LICENSES.txt`),
  equal to `music-player-<version>-android-third-party-licenses.txt`.

## License notices

Every release carries the license notices of everything built into its programs
(decision:
`doc/decision/music-player-third-party-licenses.md`).
The `notices` job writes the desktop file with `mise run //package/music-player/desktop-app:notices`;
the Android job writes the APK's file with `mise run //package/music-player/android-app:notices`.
Both run locally the same way,
with network for `cargo fetch` and the repository's toolchain
(`mise run toolchain:install` installs `cargo:cargo-about` from `toolchain.lock.json`).

When one of them fails,
nothing is released.
Read the message:

- cargo-about's `failed to satisfy license requirements` names a crate whose license is not on the allow-list of
  `package/music-player/desktop-app/about.toml` or `package/music-player/android-app/rust/about.toml`.
  Read the crate's license.
  Add it to the `accepted` list only when it can be combined with the LGPL version 3 (both apps;
  the desktop programs use Slint under its Royalty-free license, so they are not GPL works),
  and write the reason in the comment above the list and in the decision document;
  otherwise replace or drop the dependency.
- `does not match the checksum recorded in this script` or `does not ship ... with the vendored license text`
  means a Slint release changed its Royalty-free license file.
  Read the new text, and in particular its section 2 (attribution):
  the Licenses page of the desktop Settings must keep showing the `AboutSlint` widget,
  and the release notes template in `.github/workflows/music-player-release.yml` must keep the Slint attribution badge
  (the `#MadeWithSlint` image line near the top; section 2(b)).
  Reread the badge's file name and license in `logo/README.md` of the Slint repository,
  and move the commit in the image URL to the current Slint `master` head.
  Then replace `desktop-app/notices/slint-royalty-free-2.0.md` and the SHA-256 in `bin/third-party-notices.mjs`.
  A new license identifier (a version other than `LicenseRef-Slint-Royalty-free-2.0`) fails the gate first;
  see `doc/troubleshooting/cargo-about-licenseref-text-is-dropped.md` for why the script supplies the text.
- cargo-about naming `cfg_block` (or another crate with a `clarify` entry) means the crate's license file changed:
  read the new file and update the license and the SHA-256 in that entry.
- `links crates the notice does not cover` means `cargo tree` found a linked crate that cargo-about did not list;
  compare the two for that target before changing anything.
- `licenseeAndroidRelease` failing names an Android library whose license is not in the `licensee` block of
  `package/music-player/android-app/app/build.gradle.kts`.
  Only Apache-2.0 is allowed,
  because the notices script carries only its text;
  another license needs its text added to `desktop-app/bin/third-party-notices.mjs` as well.
- A desktop build failing in `build.rs` with `MUSIC_PLAYER_REQUIRE_THIRD_PARTY_LICENSES is true`,
  or the Android build failing in `thirdPartyLicensesAsset`,
  means the notices file did not reach the build.

## When a job fails

Before the release job started,
nothing is published.
Read the failure with `gh run view <run id> --log-failed`,
fix the cause on a branch,
merge,
and start a new run (a dispatch of the same version, or the next push).
`gh run rerun <run id> --failed` also works for a transient failure of a build job,
unless the failed job had already uploaded its file:
the upload then conflicts with the earlier one and a fresh dispatch is needed.

The release job can fail after it started.
Look at `gh release list --repo Aquaticat/Monochromatic`:

- No entry,
  or a draft entry:
  nothing was published.
  Delete a draft with `gh release delete music-player-v<version> --yes`,
  check that `git ls-remote --tags origin music-player-v<version>` prints nothing,
  and dispatch again.
- A published entry with missing or wrong files:
  it cannot be repaired,
  because the release is immutable and its tag is reserved even if you delete the release.
  Release the next patch version instead,
  and state in that release's notes that the previous version is broken.

## Caches

The build jobs restore cargo, target and Gradle caches (keys that start with `music-player-`),
so a run after a small change takes minutes and not tens of minutes.
Only runs on `main` and manual dispatches save them.
A changed lock file starts from the previous entry and rebuilds only what changed.
The entries together are about 4 GB, against the repository's 10 GB cap.
The two Alpine release jobs (`build-desktop-musl`) and the Windows GNU job cache no cargo registry or target directory:
every run builds them from a clean registry and target directory
(14 to 20 minutes for the Alpine jobs in the two dry runs measured,
 which makes them the longest jobs of a release run;
the Windows GNU time is in `doc/decision/music-player-releases.md`),
and the repository's cap is not spent on them.
The Windows GNU job caches only the MSYS2 package downloads (about 70 MB,
 key `msys2-pkg-v1-...`),
under the key that the `test-windows` job of the same workflow also uses.
The `notices` job caches only the cargo registry downloads (key `music-player-notices-v1-...`),
because cargo-about and `cargo tree` compile nothing.

```bash
# List the entries with their sizes.
gh cache list --repo Aquaticat/Monochromatic --key music-player- --limit 50

# Delete one entry, for example when it is suspected to be bad.
gh cache delete <key> --repo Aquaticat/Monochromatic
```

To invalidate every entry at once,
change `v1` in the `key` and `restore-keys` of the cache steps of the workflow.
The log of each build job prints a notice titled `cache` with `exact_hit` and the matched key.

## The mise version pin

Two steps of the workflow install mise with `jdx/mise-action`
(the macOS step "Install node (runs the repository's bundle:macos task)" and the Android step
"Set up mise (skip auto-install of all tools)").
Both set `version: 2026.10.9`.
The pin exists because,
 without it,
 `minimum_release_age` made every job list mise releases through the GitHub API
with the workflow token,
 and one `403 Forbidden` from that call failed the job with no retry
(run 38688512043,
 attempt 1,
 two jobs;
 the rerun passed).
With `version` set the action skips the call and ignores `minimum_release_age`,
 so the workflow does not set it.
The evidence and the source lines are in `doc/troubleshooting/mise-action-version-ahead-of-release.md`.

Who owns it:
the maintainer who owns `.github/workflows/music-player-release.yml` (today the project owner).
No bot bumps action inputs in this repository,
 so nothing updates the pin unless a person does.
An old pin keeps working as long as GitHub serves that release's assets;
it is a maintenance item,
 not a deadline.

When to update it:
when a release run needs a newer mise (a `mise.toml` change that an older mise rejects),
or at the latest when you next touch the workflow after a few months.
(`mise.lock` named the tool pins here until 2026-10-26;
the pin record moved to `toolchain.lock.json`
with `doc/decision/mpm-toolchain-adoption.md`.)

How to update it:

1.  Pick a published stable release with
    `gh release list --repo jdx/mise --exclude-drafts --exclude-pre-releases --limit 5`,
    and pick one at least a day old (the repository's release-age rule for npm packages is one day).
    Check its assets with `gh release view <tag> --repo jdx/mise --json assets --jq '.assets[].name'`:
    `mise-<tag>-linux-x64.tar.zst`,
     `mise-<tag>-macos-arm64.tar.zst` and `SHASUMS256.txt` must be listed.
2.  Replace the version in both steps,
     without the leading `v`.
    Do not add `minimum_release_age` back:
     the action ignores it while `version` is set.
3.  Run a dry run (see "Dry run").
    In both setup steps' logs the line `Selected mise` must be absent,
    and `mise --version` must print the new version.

## The Android signing key

The release key signs every Android release.
Android installs an update only if it is signed by the same key as the installed app,
so the key must never be lost or replaced casually.

Where it lives:

- Four repository secrets in `Aquaticat/Monochromatic`:
  `MUSIC_PLAYER_KEYSTORE_BASE64` (the PKCS12 keystore, base64),
  `MUSIC_PLAYER_KEYSTORE_PASSWORD`,
  `MUSIC_PLAYER_KEY_ALIAS` (`music-player-release`),
  and `MUSIC_PLAYER_KEY_PASSWORD`.
  A PKCS12 keystore uses one password for the store and the key, so the two password secrets hold the same value.
  Secrets cannot be read back from GitHub.
- A backup outside GitHub, encrypted with sops to the age identity at `~/.config/mise/age.txt`:
  `music-player-release.p12.sops.json` (the keystore) and `credentials.txt.sops.json` (its alias and password),
  in `~/.local/share/music-player-release-keys` on the maintainer's primary machine
  (directory mode 700, files mode 600; made on 2026-10-14, encrypted on 2026-10-15 with the plaintext deleted).
  Restore the keystore with
  `SOPS_AGE_KEY_FILE=~/.config/mise/age.txt sops --decrypt --input-type json --output-type binary <file>.sops.json`
  and write the output to the file name without the `.sops.json` suffix.
  The backup is only as safe as that age identity:
  keep a copy of `age.txt` outside the machine, or the signing key is lost with the machine.
- The public certificate fingerprint is written in the workflow as `EXPECTED_ANDROID_CERT_SHA256`.
  A real release fails if the APK carries any other certificate.
  Print the fingerprint of a keystore with `keytool -list -v -keystore <file>`.

If the key is lost,
every installed copy of the app can no longer be updated:
users must uninstall (losing the app's settings and caches) and install the new build.
Restore from the backup first;
only if the backup is gone,
create a new key:

1.  Generate a keystore with the command below.
    Give the passwords with `-storepass:file` and `-keypass:file` pointing at a mode 600 file,
    never on the command line.

    ```bash
    keytool -genkeypair -alias music-player-release -keyalg RSA -keysize 4096 -validity 36500 \
      -dname 'CN=Music Player release, O=Monochromatic' -storetype PKCS12 -keystore <file> \
      -storepass:file <password file> -keypass:file <password file>
    ```
2.  Set the four secrets with `gh secret set <name> --repo Aquaticat/Monochromatic < <file>`
    (the keystore secret is the base64 of the file).
3.  Replace `EXPECTED_ANDROID_CERT_SHA256` in the workflow with the new certificate fingerprint.
4.  Back up the new keystore and passwords as described above.
5.  State in the next release's notes that the signing key changed and that the app must be reinstalled.

If the key may have leaked,
rotate it the same way at once and delete the old secrets.

## Verifying attestations

Each file carries build provenance that binds its bytes to this workflow run and commit:

```bash
# Verify one downloaded file; fails if the bytes were changed after the workflow built them.
gh attestation verify music-player-<version>-linux-x64.tar.gz --repo Aquaticats/Monochromatic

# Releases up to 0.1.4 were attested under the earlier owner name: use --owner Aquaticat for those
# (checked on 2026-10-18: 0.1.4 verifies with --owner Aquaticat only, 0.1.5 with --repo Aquaticats/Monochromatic).

# Compare the whole set with the checksum file (verify SHA256SUMS itself the same way).
sha256sum --check --ignore-missing SHA256SUMS
```

Dry runs are not attested.

## Platforms that are not tested

The project has run only the Linux x86_64 build and the Android app on emulators.
The Windows and macOS programs and the Linux arm64 build are built by CI and started with `--version` there,
and nothing more.
The Alpine builds are started with `--version` in an Alpine container in the release run;
the `test-alpine` job of the same workflow (`.github/workflows/music-player-release.yml`) runs the unit tests and a
headless window and sound check in Alpine containers on both architectures, on debug builds and not on the released
files, and the release waits on it.
No real Alpine machine has run them.
No build has run on a real Android device.
A bug report from one of these platforms is the first evidence about it.
