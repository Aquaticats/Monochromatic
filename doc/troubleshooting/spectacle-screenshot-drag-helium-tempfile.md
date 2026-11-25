# Spectacle 6.8.3 screenshot drags fail in Helium because its temporary PNG is unreadable

## Status

The narrow policy change, confined payload read, and complete cache rebuild are verified. The
owner's real drag retry remains open, so this is not a final or approved completion report.

## Symptom

Dragging a screenshot from Spectacle into Helium does not insert or open the image. The retained
kernel audit identifies the exact failing payload rather than a generic Wayland drag failure:

```text
apparmor="DENIED" operation="open" class="file" profile="helium"
name="/tmp/Spectacle.qlbCwq/Screenshot_20261125_131723.png"
comm="ThreadPoolForeg" requested_mask="r" denied_mask="r"
```

The payload existed, was owned by UID 1000, had observed mode `0644`, and was 940591 bytes. Its
SHA-256 was
`d47075e07d3ea4604a00509cc267720e732f6b2c19c0272794992636c9da1ac3` when read outside
confinement.

## Root cause

Spectacle 6.8.3's `ViewerWindow::startDrag()` obtains a temporary file from `ExportManager`, puts
its URL in `QMimeData`, and starts a copy drag in `src/Gui/ViewerWindow.cpp:159-202`:

```cpp
const QUrl tempFile = SpectacleCore::instance()->videoMode()
    ? SpectacleCore::instance()->currentVideo()
    : exportManager->tempSave();
// ...
mimeData->setUrls(QList<QUrl>{tempFile});
// ...
dragHandler->exec(Qt::CopyAction);
```

Source: [Spectacle `ViewerWindow.cpp`, drag construction](https://github.com/KDE/spectacle/blob/1b5f4f8f5563d0f8c4073bf19163d5c31fbb29c1/src/Gui/ViewerWindow.cpp#L159-L202).

`ExportManager::temporaryDir()` creates the source-derived directory pattern in
`src/ExportManager.cpp:186-202`:

```cpp
m_tempDir = std::make_unique<QTemporaryDir>(QDir::tempPath() + u"/Spectacle.XXXXXX"_s);
m_tempDir->setAutoRemove(false);
```

Source: [Spectacle `ExportManager.cpp`, temporary directory](https://github.com/KDE/spectacle/blob/1b5f4f8f5563d0f8c4073bf19163d5c31fbb29c1/src/ExportManager.cpp#L186-L202).

`ExportManager::tempSave()` writes the screenshot there and calls `setPermissions(QFile::ReadUser)`
for the receiving application in `src/ExportManager.cpp:544-577`. The observed payload's later mode
was `0644`, so the policy conclusion relies on the measured read, not an assumed final mode:

```cpp
if (writeImage(&tmpFile, suffix.toLatin1()) && tmpFile.commit()) {
    m_tempFile = QUrl::fromLocalFile(tmpFile.fileName());
    // properly treats this as readonly, also hide from other users
    tmpFile.setPermissions(QFile::ReadUser);
    return m_tempFile;
}
```

Source: [Spectacle `ExportManager.cpp`, read-only export](https://github.com/KDE/spectacle/blob/1b5f4f8f5563d0f8c4073bf19163d5c31fbb29c1/src/ExportManager.cpp#L544-L577).

Helium's Chromium abstraction already permitted its own temporary-file shapes, but not Spectacle's
`Spectacle.<six characters>` shape. The receiving Helium worker therefore could not open the PNG.
No D-Bus, portal, Wayland, or browser-sandbox change is needed for the demonstrated denial.

## Verification

Installed Spectacle is `1:6.8.3-1.1`. The source tag `v6.8.3` resolves to commit
`1b5f4f8f5563d0f8c4073bf19163d5c31fbb29c1`. GitHub's contents API returned the same quoted
expressions at that commit. Visible-browser citation validation was not run because the owner
asked for no more unwanted windows.

This command exercises the exact payload read under the real enforcing Helium profile:

```sh
aa-exec --profile helium -- /usr/bin/sha256sum \
  /tmp/Spectacle.qlbCwq/Screenshot_20261125_131723.png
```

### Patterns that fail

Before the local rule, the same payload produced `Permission denied`, and `sha256sum` exited 1.
Repeated real drag attempts emitted matching `DENIED` records for Helium's `ThreadPoolForeg`
worker.

### Patterns that work

After adding the read-only rule and replacing only the Helium profile, the same command exited 0
and returned:

```text
d47075e07d3ea4604a00509cc267720e732f6b2c19c0272794992636c9da1ac3  /tmp/Spectacle.qlbCwq/Screenshot_20261125_131723.png
```

This matches the independent unconfined hash byte for byte. It proves the exact file read, not
merely parser success or a successful reload.

`verified-selected-spectacle-drag-fix-2026-11-25T13-23-05-0500.json` records all 809 expected
cache artifacts, exact live feature bytes, successful cache hits including Helium, selection,
reload, and rollback preservation. The confined hash check passed again after selection. A
complete browser drag remains the user-boundary check.

## Verified workarounds

### Permit read-only access to Spectacle's exported payloads

Add to `/etc/apparmor.d/local/helium`:

```apparmor
# Spectacle exports dragged screenshots read-only from its six-character temporary directory.
owner @{tmp}/Spectacle.@{rand6}/{,**} r,
```

Then reload the affected profile:

```sh
sudo apparmor_parser --replace --write-cache /etc/apparmor.d/helium
```

The tradeoff is that enforcing Helium can read user-owned files in Spectacle's six-character
temporary directories if it obtains their paths. It cannot write them, and the rule grants no
access to unrelated temporary directories, executable transitions, capabilities, or broader home
access. Read access is sufficient for the measured payload open; the source also requests read-only
permissions for the receiver.

The pre-change local policy is retained privately at
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/helium-local.before-spectacle-drag-read`.

## What does not work

- Granting all of `/tmp`: unnecessary for one source-derived directory shape and much broader than
  the measured workload.
- Adding write or lock permissions: the receiving browser needs to read the exported PNG, not
  modify Spectacle's files.
- Treating every Helium DMI denial as the drag failure: the payload-specific `open` denial is the
  relevant record.
- Disabling Helium enforcement or its Chromium sandbox: the narrow read rule fixes the demonstrated
  boundary without either action.
- Treating parser exit 0 as proof: acceptance required the confined read to return the exact PNG
  hash, and the real drag still needs separate confirmation.

## Upstream filing decision

The `.out-of-scope/` and troubleshooting-document searches found no Spectacle drag exemption or
existing matching record. On 2026-11-25, these issue and pull-request searches returned no matches:

```sh
gh search issues 'Spectacle AppArmor drag' --owner KDE --limit 20
gh search prs 'Spectacle AppArmor drag' --owner KDE --limit 20
gh search issues 'Spectacle chromium' --owner roddhjav --limit 20
gh search prs 'Spectacle chromium' --owner roddhjav --limit 20
```

1.  **Is it really upstream's fault?** No. Spectacle exports the file as designed; the locally
    authored Helium profile omitted that consumer path.
2.  **Can upstream fix it?** No Spectacle code change is needed. The consumer-side policy owns the
    fix.
3.  **Are they supporting this use case?** Spectacle explicitly supports dragging temporary file
    URLs, which is why the policy should permit their reads.
4.  **Would the repo welcome our contribution?** Not evaluated for filing because this is not an
    upstream defect. No contribution-policy claim is made.
5.  **Will they likely fix it?** No upstream fix request is recommended for this local policy
    omission.
6.  **Have we prototyped a compatible fix?** The local policy change passed the exact confined
    payload-read reproduction. No upstream source prototype is needed.

The readiness check did not test other maintained Spectacle release lines, filing automation, or a
clean-runner reproduction repository because there is no upstream defect report to file. Posting
would expose the poster to tracker and trust-system risk without an upstream change to justify it.
The filing decision belongs to the poster; this record recommends no filing and keeps no fileable
issue draft.
