# Dolphin 26.08.3 file drags fail in Helium 0.22.4.1 when the synced tree is unreadable

## Status

The owner selected read-only access to all of `~/Plain`. The exact confined payload read,
outside-tree negative control, owner's real Dolphin-to-Helium drag, and complete cache rebuild
pass. The owner confirmed the UI result. This is an evidence record, not separate approval of the
whole AppArmor rollout.

## Symptom

Dragging a file from Dolphin into Helium does not insert or open it. The retained kernel audit
identifies the exact payload and receiving thread:

```text
apparmor="DENIED" operation="open" class="file" profile="helium"
name="/home/user/Plain/Text/low/君とふたり永遠の花園で_完全版.md"
comm="ThreadPoolForeg" requested_mask="r" denied_mask="r"
```

The real drag produced repeated denials for this same file. It is a regular file owned by UID 1000,
with mode `0644` and size 14953 bytes. Its SHA-256 outside confinement is
`cd3141a5da1fbed3568b7937c7701ebe78186f0cf60cdfadee64bfd17c24510c`.

## Root cause

Dolphin's `KFileItemModel::createMimeData()` collects each selected item's URL and most-local URL,
then passes both lists to `KUrlMimeData::setUrls()` in
`src/kitemviews/kfileitemmodel.cpp:526-561`:

```cpp
urls << item.url();

bool isLocal;
mostLocalUrls << item.mostLocalUrl(&isLocal);
// ...
KUrlMimeData::setUrls(urls, mostLocalUrls, data);
```

Source: [Dolphin `kfileitemmodel.cpp`, URL MIME construction](https://github.com/KDE/dolphin/blob/2fdee206603fd3a21928984e9a3d404e82ae9890/src/kitemviews/kfileitemmodel.cpp#L526-L561).

`KItemListController::startDragging()` obtains that MIME data, offers a portal export, attaches the
data to `QDrag`, and starts the drag in `src/kitemviews/kitemlistcontroller.cpp:1423-1452`:

```cpp
QMimeData *data = m_model->createMimeData(selectedItems);
// ...
KUrlMimeData::exportUrlsToPortal(data);
// ...
QDrag *drag = new QDrag(QApplication::activeWindow());
drag->setMimeData(data);
// ...
drag->exec(Qt::MoveAction | Qt::CopyAction | Qt::LinkAction, Qt::CopyAction);
```

Source: [Dolphin `kitemlistcontroller.cpp`, drag construction](https://github.com/KDE/dolphin/blob/2fdee206603fd3a21928984e9a3d404e82ae9890/src/kitemviews/kitemlistcontroller.cpp#L1423-L1452).

KCoreAddons' `KUrlMimeData::exportUrlsToPortal()` adds an
`application/vnd.portal.filetransfer` transfer ID in `src/lib/io/kurlmimedata.cpp:282-390`:

```cpp
mimeData->setData(QStringLiteral("application/vnd.portal.filetransfer"), QFile::encodeName(transferId));
setSourceId(mimeData);
return true;
```

Source: [KCoreAddons `kurlmimedata.cpp`, portal transfer offer](https://github.com/KDE/kcoreaddons/blob/19b95cd96ca7d81e392661b463dbf55b3c7a9e79/src/lib/io/kurlmimedata.cpp#L282-L390).

That portal offer does not remove the original URL list. The measured Helium worker opened the
original `/home/user/Plain/...` path, and AppArmor denied that open. The Chromium abstraction's
`user-read-strict` rules cover configured XDG and sync directories, but this installation defines
`XDG_SYNC_DIR` as `Sync`, not `Plain`. No Wayland, D-Bus, portal, executable, or browser-sandbox
change is needed for the demonstrated file denial.

## Verification

Installed packages are Dolphin `26.08.3-1.1`, KCoreAddons `6.32.0-1.1`, and Helium
`0.22.4.1-1`. Dolphin tag `v26.08.3` resolves to commit
`2fdee206603fd3a21928984e9a3d404e82ae9890`; KCoreAddons tag `v6.32.0` resolves to
`19b95cd96ca7d81e392661b463dbf55b3c7a9e79`. Helium release `0.22.4` resolves to
`41a32da3ed31e0c4a550b7f317f5ab095deee1c9`.

The following command exercises the exact payload read under the real enforcing Helium profile:

```sh
aa-exec --profile helium -- /usr/bin/sha256sum \
  '/home/user/Plain/Text/low/君とふたり永遠の花園で_完全版.md'
```

### Patterns that fail

Before the local rule, the exact command returned `Permission denied` and exited 1. Repeated real
Dolphin drag attempts emitted matching `DENIED` records. An unrelated hidden-file control also
returned `Permission denied`:

```sh
aa-exec --profile helium -- /usr/bin/sha256sum /home/user/.bash_profile
```

### Patterns that work

After adding the rule and replacing only Helium's profile, the exact payload command exited 0 and
returned:

```text
cd3141a5da1fbed3568b7937c7701ebe78186f0cf60cdfadee64bfd17c24510c  /home/user/Plain/Text/low/君とふたり永遠の花園で_完全版.md
```

This matches the independent unconfined hash. The `.bash_profile` negative control remained denied,
so the change did not grant general home access. Source excerpts were checked directly through
GitHub's contents API; no browser automation or new windows were used.

The owner then retried the real Dolphin-to-Helium drag and reported, "Working well now." No new
Helium denial for `/home/user/Plain` appeared after the rule change. This user-boundary result is
separate from the confined file-read evidence.

`verified-selected-dolphin-plain-drag-fix-2026-11-25T17-30-36-0500.json` records all 809 expected
cache artifacts, exact live feature bytes, a Helium cache hit, selection, reload, and rollback
preservation.

## Verified workarounds

### Permit read-only access to the owner-approved synced tree

The owner explicitly selected all of `~/Plain`, including its KeePassXC database files, over narrower
`Text` and `Music` or single-directory grants. Add to `/etc/apparmor.d/local/helium`:

```apparmor
# Dolphin supplies original file URLs for dragged files in the owner-approved synced tree.
owner @{HOME}/Plain/{,**} r,
```

Then reload the affected profile:

```sh
sudo apparmor_parser --replace --write-cache /etc/apparmor.d/helium
```

The tradeoff is that enforcing Helium can read any user-owned file or directory under `Plain` if it
obtains its path, including `main.kdbx` and its conflict copies. AppArmor cannot express "only while
this file is being dragged" with this pathname rule. The grant provides no write, delete, lock,
execute, capability, or broader-home permission. Files outside `Plain` retain their existing policy.

The pre-change local policy is retained privately at
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/helium-local.before-dolphin-plain-drag-read`
(scratch,
deleted on 2026-11-27;
not kept).

## What does not work

- Treating this as a generic Wayland drag failure: the exact receiving thread and file open are in
  the AVC, and the same path fails under `aa-exec` without any GUI action.
- Granting all of `/home/user`: unnecessary for the owner-selected synced tree and contrary to the
  existing confinement boundary.
- Adding write, lock, or executable permissions: the measured receiver operation requires read only.
- Assuming the portal transfer offer replaces every original file URL: Helium's measured open used
  the original `Plain` path.
- Treating parser exit 0 as proof: acceptance required the exact confined hash and a still-denied
  outside-tree control. A complete real drag remains a separate UI-boundary check.

## Upstream filing decision

On 2026-11-25, issue and pull-request searches for Dolphin, AppArmor, Helium, and drag terms under
`KDE`, `roddhjav`, and `imputnet` returned no matching records.

1.  **Is it really upstream's fault?** No. Dolphin exports the selected URL and offers a portal
    transfer; the locally authored Helium policy omitted the selected synced tree.
2.  **Can upstream fix it?** No Dolphin, KCoreAddons, or Helium code change is required for this
    measured policy failure.
3.  **Are they supporting this use case?** Dolphin explicitly constructs file-URL drags and portal
    transfer offers. The local consumer policy must authorize the path the receiver opens.
4.  **Would the repo welcome our contribution?** Not evaluated because there is no upstream defect
    to report. No contribution-policy claim is made.
5.  **Will they likely fix it?** No upstream fix request is recommended for this local omission.
6.  **Have we prototyped a compatible fix?** The local read-only rule passed the exact confined
    payload reproduction and preserved the outside-tree denial. No upstream source prototype is
    needed.

No upstream issue or additive comment is recommended. This record is not a claim that every drag
source or every directory outside `Plain` is authorized.
