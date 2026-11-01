# Android Package Manager 37 rejects a debug APK update signed with a different certificate

## Symptom

An Android 17 API 37 disposable Fold already contained the music-player
debug APK.
After a debug-only Compose prototype was rebuilt in a separate capped
container,
`adb -s emulator-5580 install -r app-debug.apk` returned:

```text
INSTALL_FAILED_UPDATE_INCOMPATIBLE: Existing package dev.monochromatic.musicplayer signatures do not match newer version; ignoring!
```

This is the Android package installer's diagnostic,
not a Kotlin compilation error.
The previous `mise run prototype:build` completed successfully.

## Root cause

The installed SDK source at
`/home/user/Android/Sdk/sources/android-37.0/android/content/pm/PackageManager.java:2225-2233`
defines the reported code for a same-name package whose old and new
signatures differ while the old app data remains:

```java
/**
 * Installation return code: this is passed in the {@link PackageInstaller#EXTRA_LEGACY_STATUS}
 * if a previously installed package of the same name has a different signature than the new
 * package (and the old package's data was not removed).
 */
public static final int INSTALL_FAILED_UPDATE_INCOMPATIBLE = -7;
```

`apksigner verify --print-certs` measured different SHA-256 certificate
digests for this exact pair:

- Installed disposable APK:
  `96e9aaf5882375eba61a577f2cb77265f26f78457da630f9bb875aa28c0c998c`.
- Newly built debug APK:
  `08b8db3abbff7d391f781d23ddd95f0ebcd8b83f7b7d934a2e7a6eceedbc7942`.

Their different certificates explain the update rejection.
The distinct build environment may have generated a different debug key,
but its keystore provenance was not traced;
do not state that as an established cause.

## Verification

Both packages used the same app identifier.
The old APK was pulled **from the disposable AVD** and checked with the
installed Android SDK's `apksigner`;
the new APK was checked in the prototype worktree.
The new artifact's SHA-256 file digest was
`e4bfec8e8eb98187a0afc06ef11d587fd3c87623fe45a37da7b0ffd862c6461e`.
The original `install -r` failed with the reported signature code.
On the same disposable AVD,
uninstalling only `dev.monochromatic.musicplayer` and installing the new
APK succeeded;
`pm path dev.monochromatic.musicplayer` returned its new installed path.
This positive control tests the package boundary,
not just a local APK signature command.

## Verified workaround

On a **disposable** test AVD only,
remove the old debug app before installing the newly signed one:

```sh
adb -s emulator-5580 uninstall dev.monochromatic.musicplayer
adb -s emulator-5580 install /path/to/app-debug.apk
adb -s emulator-5580 shell pm path dev.monochromatic.musicplayer
```

This discards the app's local data and UI state;
it does **not** reset Gboard's data or change the original AVD.
For stateful testing,
keep a stable signing certificate instead of using this destructive
workaround.
That alternative was not tested here.

## What does not work

- Repeating `adb install -r` with these two certificates:
  replacement preserves the old package data and fails the measured
  signature check.
- Treating `BUILD SUCCESSFUL` as proof of installability:
  the installer enforces a separate compatibility boundary.
- Uninstalling from the original AVD:
  not attempted or authorized;
  all app-data removal occurred on `emulator-5580`.

## Container-path mistake during a separate first-run study

A later first-run study had no installed debug package.
Its initial unconditional uninstall returned
`Failure [DELETE_FAILED_INTERNAL_ERROR]`;
package enumeration and package dump then confirmed absence.
No underlying Package Manager cause was established from that diagnostic.
The driver now tests presence before attempting removal.

The next attempt ran ADB inside the owned emulator container but passed a
host-only APK path:

```text
adb: failed to stat /home/user/temp/agent/first-run-access-native-private/captured-debug.apk: No such file or directory
```

This was a local file-path mistake,
not `INSTALL_FAILED_UPDATE_INCOMPATIBLE` or a proved guest install failure.
The command had no accepted native frames.
The container's mounts included the AVD and read-only SDK,
not the host's private capture directory.
The correction copies only the retained debug APK into the owned
container's `/tmp`,
checks its exact SHA-256 against the retained host artifact,
then gives that container-visible path to ADB.
The ensuing install and installed-package hash are separately checked by
the capture driver;
launching the copy command alone establishes neither install nor rendering.

This path correction changes no certificate,
real library or original AVD.
No new upstream issue or source patch is implied by either local driver
mistake.

## Upstream filing decision

No upstream report is filed or drafted.
No `.out-of-scope/` entry exempts this Android symptom,
but the SDK's explicitly documented certificate check matched the
measured certificates:

1. There is no evidence of an upstream fault.
2. Android could change the rule,
   but changing a security check is not a justified fix for distinct debug keys.
3. App updates are supported when their signatures are compatible;
   that condition was not met.
4. No upstream contribution path was assessed because there is no defect to patch.
5. No maintainer action is expected from this local fixture mismatch.
6. No upstream patch exists;
   the disposable uninstall/install workaround was verified instead.

Nothing is ready to send upstream.
