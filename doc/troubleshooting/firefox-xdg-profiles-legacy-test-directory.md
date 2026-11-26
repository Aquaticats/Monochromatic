# Firefox 147+ appears to reset profiles when a test creates the legacy profile directory

## Status

The stray legacy directory was renamed without deleting it. The original XDG profile indexes
remained byte-identical, and the owner confirmed that regular Firefox, Developer Edition, and ESR
are back. No AppArmor policy or cache change was required. This is an evidence record, not approval
of unrelated browser behavior.

## Symptom

After package upgrades and application restarts, regular Firefox, Developer Edition, and ESR all
appeared to have fresh profiles. Their previous bookmarks, extensions, settings, and sessions were
not visible.

The new profile root was `/home/user/.mozilla/firefox`. It contained newly created profiles with
timestamps around 01:55 to 01:57 on 2026-11-26:

- `4n9kn1ri.default`
- `7h1cgehv.dev-edition-default`
- `d36y734g.default-esr`
- `rw5e2u5a.default-release`

The previous profiles were still intact under `/home/user/.config/mozilla/firefox`:

- `47ezuqf1.default-release`
- `oty15h01.dev-edition-default`
- `iscpx5pg.default-esr`

The original location also contained its earlier crash-recovery backups. No Refresh Firefox
operation was needed or performed.

## Root cause

Firefox 147 and later supports XDG profile roots for new Linux setups, while retaining compatibility
with the legacy `~/.mozilla/firefox` layout. The compatibility decision is based on existence of the
legacy path, not on whether it contains the user's real profiles.

Firefox 161's `nsXREDirProvider::LegacyHomeExists()` checks the old application path in
`toolkit/xre/nsXREDirProvider.cpp:1380-1425`:

```cpp
// check old config ~/.mozilla
nsresult rv = NS_NewNativeLocalFile(homeDir, getter_AddRefs(localDir));
// ...
rv = AppendFromAppData(localDir, true);
// ...
rv = localDir->Exists(&exists);
```

Source: [Firefox 161 `nsXREDirProvider.cpp`, legacy existence check](https://github.com/mozilla-firefox/firefox/blob/9c70d5d48c866afc2e581d6b7ff6de98a1371d99/toolkit/xre/nsXREDirProvider.cpp#L1380-L1425).

`GetLegacyOrXDGHomePath()` then selects the legacy parent whenever that path exists, in the same
file at lines 1499-1520:

```cpp
bool exists = LegacyHomeExists(getter_AddRefs(parentDir));
if (exists || IsForceLegacyHome() || aForceLegacy) {
  if (exists) {
    MaybeRecordXdgTelemetry(legacyOrXDGHomeTelemetry::legacyExists);
  }
  parentDir.forget(aFile);
  return NS_OK;
}
```

Source: [Firefox 161 `nsXREDirProvider.cpp`, legacy selection](https://github.com/mozilla-firefox/firefox/blob/9c70d5d48c866afc2e581d6b7ff6de98a1371d99/toolkit/xre/nsXREDirProvider.cpp#L1499-L1520).

Mozilla tracks the intended XDG behavior in [Bug 259356](https://bugzilla.mozilla.org/show_bug.cgi?id=259356).

On 2026-11-23, the AppArmor rollout created `/home/user/.mozilla/firefox` and two explicitly named
test profiles. The user's real browsers were already using the XDG location and continued running
with those profiles. After upgrades on 2026-11-25 and subsequent restarts, all three channels saw
the test-created legacy directory and created fresh profiles there.

This was a profile-root switch caused by test state, not deletion of browser data and not a Firefox
profile reset. The relevant evidence is path selection, not an AppArmor AVC. A read-only ZFS snapshot
from before the new launches contained only the two AppArmor test profiles and no legacy
`profiles.ini`, which also disproved the hypothesis that the launches deleted an older legacy index.

## Verification

Installed packages were regular Firefox `161.0-1`, Developer Edition `162.0b1-1.1`, and ESR
`153.8.0-1`. The Firefox 161 release source commit was
`9c70d5d48c866afc2e581d6b7ff6de98a1371d99`. The quoted source was fetched directly from that
commit through GitHub's contents API.

Before recovery, the original XDG index hashes were:

```text
a99ad2a14ca8141993371b88a92fe9e1eeb68a7ae5ec8d67f6c83adec8961dbd  profiles.ini
f1fd1cd3a4324b1495a45d1638520b300ea773c9f9334515ff6b4b150a291894  installs.ini
```

### Patterns that fail

- Create even a test-only `/home/user/.mozilla/firefox` while browsers use the XDG root, then restart
  them. The existence check selects the legacy root, and the previous XDG profiles are not selected.
- Look only at the fresh legacy `profiles.ini`. It accurately describes the new profiles but says
  nothing about the intact XDG profiles.
- Treat the simultaneous channel switch as an AppArmor file denial. The payload here is the profile
  root selection, and no profile-data deletion or reset operation occurred.

### Patterns that work

- Keep the real XDG root at `/home/user/.config/mozilla/firefox` and remove the stray legacy path
  from Firefox's lookup by renaming it non-destructively.
- Preserve the complete test/new directory rather than deleting it. This retains any changes made
  during the apparently fresh sessions and allows later inspection.
- Verify the real applications afterward. The owner reported that all three channels were back, and
  their processes had open descriptors under the original XDG profile tree. Regular Firefox and ESR
  were enforcing as `firefox`; Developer Edition was unconfined under the existing `fsp no` coverage
  boundary, which is separate from profile selection.

## Verified workarounds

### Preserve and rename the stray legacy directory

First close all Firefox channels and confirm that the XDG profiles and index files exist. Then rename
only the legacy `firefox` directory:

```sh
mv --no-clobber -- \
  /home/user/.mozilla/firefox \
  /home/user/.mozilla/firefox-apparmor-tests-and-empty-profiles-2026-11-26
```

This leaves `/home/user/.mozilla/native-messaging-hosts` untouched. It does not copy, rewrite, or
delete any original XDG profile. After the rename, `/home/user/.mozilla/firefox` was absent, the
preserved directory and its `profiles.ini` remained present, and both original XDG index hashes were
unchanged.

Launch each channel normally and inspect bookmarks, extensions, settings, and session restoration.
Do not choose Refresh Firefox. If a channel reports a stale profile lock, diagnose that separately
instead of deleting profile data.

The tradeoff is that the old test profiles and any changes made in the new sessions remain under a
nonstandard backup name. They are intentionally not merged into the real profiles. Merging browser
databases would be a separate, riskier operation.

### Keep future browser verification in a disposable home

Browser tests that create profiles must use a temporary `HOME` and browser-profile root. They must
not create `/home/user/.mozilla/firefox` in the real home, even for a named disposable profile. The
existence of that directory changes subsequent real-application behavior.

A disposable fixture should verify both arms:

- With no legacy directory, Firefox selects the fixture's XDG profile root.
- With a fixture legacy directory present, Firefox selects the fixture's legacy root.

Both arms must remain inside the disposable home. No such fixture is needed to confirm the owner's
completed recovery again.

## What does not work

- Adding broader AppArmor read rules: the original profiles were already readable in their real
  applications after restoring the intended lookup path. This was not the measured failure.
- Deleting the new legacy directory: unnecessary and destructive to any changes made during the new
  sessions. Renaming preserves the evidence and data.
- Deleting all of `/home/user/.mozilla`: broader than necessary and would affect native-messaging
  integration. Only the `firefox` child controls this profile-root decision.
- Copying the XDG profiles into the new legacy profiles: duplicates browser state and risks mixing
  incompatible live databases. The original profiles never needed to move.
- Running Refresh Firefox: would create another fresh profile rather than repair the path selection.

## Upstream filing decision

1.  **Is it really upstream's fault?** No. The installed source implements the documented legacy
    compatibility decision. My AppArmor test created the path that triggered it.
2.  **Can upstream fix it?** The behavior is intentional and tracked in Bug 259356. No Firefox code
    change is needed to restore this installation.
3.  **Are they supporting this use case?** Mozilla supports both new XDG setups and existing legacy
    layouts. It does not promise correct selection when a test creates a competing real profile root.
4.  **Would the repo welcome our contribution?** Not evaluated because there is no new upstream
    defect to report. No contribution-policy claim is made.
5.  **Will they likely fix it?** No new fix request is recommended for this locally triggered state.
6.  **Have we prototyped a compatible fix?** The non-destructive rename restored all three real
    channels without modifying Firefox or its profiles.

No upstream issue or additive comment is recommended. The durable prevention is to keep browser
profile creation inside disposable homes during verification.
