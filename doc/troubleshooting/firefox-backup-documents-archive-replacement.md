# Firefox 153.8 ESR and 161 backups fail when AppArmor blocks archive replacement

## Status

The confined rename-readback-delete probe, unrelated-filename negative control, real backups in
both channels, and complete 812-profile cache rebuild pass. The owner confirmed both application
results. This is an evidence record, not approval of unrelated Firefox behavior.

## Symptom

Firefox ESR's Backup settings displayed:

```text
Hmm, there was a problem backing up.
Try again in a few minutes.
```

The configured location was `/home/user/Documents/Restore Firefox`. Regular Firefox showed the same
failure. This was separate from the earlier profile-root incident: the restored profiles were in
use, but new backup archives could not replace older ones.

The ESR audit showed the exact failing operations:

```text
apparmor="DENIED" operation="rename_dest" class="file" profile="firefox"
name="/home/user/Documents/Restore Firefox/FirefoxBackup_default-esr_2c4356f2_20261126-020155.537.html"
requested_mask="wc" denied_mask="wc"

apparmor="DENIED" operation="unlink" class="file" profile="firefox"
name="/home/user/Documents/Restore Firefox/FirefoxBackup_default-esr_2c4356f2_20261118-120635.216.html"
requested_mask="d" denied_mask="d"
```

## Root cause

Firefox ESR 153.8.0's `BackupService.finalizeSingleFileArchive()` constructs the timestamped HTML
filename and moves the completed archive into the configured folder in
`browser/components/backup/BackupService.sys.mjs:2036-2055`:

```javascript
const FILENAME = `${FILENAME_PREFIX}_${archiveDateSuffix}.html`;
let destPath = PathUtils.join(destFolder, FILENAME);

lazy.logConsole.log("Moving single-file archive to ", destPath);
await IOUtils.move(sourcePath, destPath);
```

Source: [Firefox ESR 153.8.0 `BackupService.sys.mjs`, archive move](https://github.com/mozilla-firefox/firefox/blob/7a005ddca7c1e2b14706aca399520a822055164a/browser/components/backup/BackupService.sys.mjs#L2036-L2055).

It then removes older archives with the same application and profile prefix in the same function:

```javascript
if (
  childFileName.startsWith(FILENAME_PREFIX) &&
  childFileName.endsWith(".html")
) {
  // ...
  await IOUtils.remove(childFilePath);
}
```

Source: [Firefox ESR 153.8.0 `BackupService.sys.mjs`, archive rotation](https://github.com/mozilla-firefox/firefox/blob/7a005ddca7c1e2b14706aca399520a822055164a/browser/components/backup/BackupService.sys.mjs#L2057-L2082).

Regular Firefox 161 has the same `IOUtils.move(sourcePath, destPath)` and rotation logic in its
installed-version source. Both channels attach to the shared enforcing `firefox` AppArmor profile.

That profile grants read access to configured user document directories through
`user-read-strict`. Its write-oriented `user-download-strict` rules cover Desktop and Downloads,
not this custom folder beneath Documents. Creating the archive elsewhere succeeded, but the final
rename destination and old-archive unlink were denied. Broader Documents write access is unnecessary.

## Verification

Installed versions were regular Firefox `161.0-1` and ESR `153.8.0-1`. The ESR source commit was
`7a005ddca7c1e2b14706aca399520a822055164a`; the regular Firefox source commit was
`9c70d5d48c866afc2e581d6b7ff6de98a1371d99`. The ESR implementation was also extracted from the
installed `/opt/firefox-esr/browser/omni.ja`, avoiding a default-branch comparison. Regular Firefox's
matching statements were fetched separately from its release commit.

### Patterns that fail

Before the rule, a confined rename between two new probe files in the configured backup folder
failed deterministically:

```text
Error: EACCES: permission denied, rename
'/home/user/Documents/Restore Firefox/FirefoxBackup_apparmor-policy-probe-source.html'
-> '/home/user/Documents/Restore Firefox/FirefoxBackup_apparmor-policy-probe-renamed.html'
```

An unrelated filename in the same directory also remained denied after the repair:

```text
Error: EACCES: permission denied, open
'/home/user/Documents/Restore Firefox/Unrelated_apparmor-negative-control'
```

### Patterns that work

After the rule, the same confined probe completed rename, exact payload readback, and deletion:

```text
confined_rename_readback_delete=passed
unrelated_file_still_denied=yes
probe_cleanup_complete=yes
```

The owner then completed **Backup now** in both channels. The resulting files were present with new
timestamps and sizes:

```text
FirefoxBackup_default-release_e17ee57a_20261126-020653.697.html  18770752 bytes
FirefoxBackup_default-esr_2c4356f2_20261126-020643.773.html      10749540 bytes
```

No backup-related Firefox denial occurred after the loaded rule. No existing backup was used as a
probe or deleted by verification; the application rotated its own archives during the real runs.

`verified-selected-firefox-backup-fix-2026-11-26T02-09-04-0500.json` records all 812 expected
cache artifacts, exact live feature bytes, Firefox and IVPN cache hits, selection, reload, and
rollback preservation.

## Verified workarounds

### Permit archive replacement only in the selected backup folder

Add to `/etc/apparmor.d/local/firefox`:

```apparmor
# Firefox Backup moves new archives and removes rotated copies in the selected folder.
owner @{HOME}/Documents/Restore\ Firefox/ rw,
owner @{HOME}/Documents/Restore\ Firefox/FirefoxBackup_* rwl,
```

Reload the managed Firefox source, not the unconfined compatibility stub:

```sh
sudo apparmor_parser --replace --write-cache /etc/apparmor.d/firefox.apparmor.d
```

The rule lets Firefox read, create, replace, and delete `FirefoxBackup_*` entries in the selected
folder. AppArmor cannot distinguish an intended backup rotation from another invocation using the
same path and prefix. Unrelated filenames and the rest of Documents retain their previous policy.
No execute, capability, broad home, or unrelated-directory grant is added.

The pre-change local policy is retained privately at
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/firefox-local.before-backup-archive-replace`
(scratch,
deleted on 2026-11-27;
not kept).

### Preserve the parser spelling for a path containing a space

The initial quoted form using multi-value `@{user_documents_dirs}` was rejected by the installed
parser before any profile load:

```text
AppArmor parser error for /etc/apparmor.d/firefox.apparmor.d in profile
/etc/apparmor.d/local/firefox at line 9:
Lexer found unexpected character: '"' (0x22) in state: INITIAL
```

The working form uses `@{HOME}` and an escaped literal space, as shown in the rule. The rejected
candidate did not alter the running profile and was not counted as a workaround.

## What does not work

- Treating this as another profile reset: the restored XDG profiles remained active. The failing
  operations were backup archive rename and unlink.
- Granting write access to all of Documents: the measured workload needs only one selected folder
  and the `FirefoxBackup_` prefix.
- Adding execute permission for browser helpers: the backup failure is a file replacement boundary,
  not an executable transition.
- Testing only parser exit 0: acceptance required confined rename, readback, delete, an
  outside-prefix denial, and two real application backups.
- Using an older 809-profile cache inventory after the separate IVPN profiles raised the current
  inventory to 812. Initial cache harness attempts using an obsolete or ambiguous inventory stopped
  before candidate selection or reload; they were not treated as successful cache evidence.

## Upstream filing decision

Searches found Mozilla's intended configurable backup-location work in Bug 1903127 and a separate
macOS Documents-permission discussion, but no matching Linux AppArmor policy omission. Searches for
Firefox Backup, Documents, and AppArmor under `roddhjav` returned no matching issue or pull request.

1.  **Is it really upstream's fault?** No. Firefox deliberately moves and rotates archives in the
    configured folder. The locally authored AppArmor policy omitted that write boundary.
2.  **Can upstream fix it?** No Firefox code change is required for the measured denial.
3.  **Are they supporting this use case?** The source explicitly supports a user-configured backup
    location and archive rotation.
4.  **Would the repo welcome our contribution?** Not evaluated because there is no upstream defect
    to report. No contribution-policy claim is made.
5.  **Will they likely fix it?** No upstream fix request is recommended for this local omission.
6.  **Have we prototyped a compatible fix?** The narrow rule passed the confined replacement probe
    and both real application backups without broadening unrelated Documents access.

No upstream issue or additive comment is recommended. This record does not claim that a different
backup directory is authorized; changing `browser.backup.location` requires corresponding measured
policy review.
