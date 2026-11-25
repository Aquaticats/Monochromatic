# KeePassXC cannot save a database when AppArmor omits Qt's same-directory temporary file

## Status

The measured policy repair, confined create-write-rename-readback fixture, and complete cache
rebuild pass. The owner's real database save retry remains open, so this is not a final or approved
completion report.

## Symptom

The user reported: "KeepassXC writing the database failed: Permission denied."

KeePassXC remained enforcing as `keepassxc (enforce)`. The selected database was
`/home/user/Plain/main.kdbx`, owned by UID 1000 and mode `0644`. Its parent directory was owned by
UID 1000 and mode `0755`.

The retained audit record identified a different path from the database itself:

```text
apparmor="DENIED" operation="open" class="file" profile="keepassxc"
name="/home/user/Plain/#14711592" pid=4823
requested_mask="wr" denied_mask="wr" fsuid=1000 ouid=1000
```

The paired `SYSCALL` record had `syscall=257 success=no exit=-13`, identifying a failed `openat`.
The denied temporary path did not exist afterward.

## Root cause

KeePassXC 2.8.0 defaults to atomic saves. Its `Database::performSave()` constructs a `QSaveFile`,
writes the database, and calls `commit()`, in
[`src/core/Database.cpp:358-389`](https://github.com/keepassxreboot/keepassxc/blob/1c2f23ef5914c0f442a00533def1689857afb5a7/src/core/Database.cpp#L358-L389):

``` cpp
// src/core/Database.cpp:369-383
QSaveFile saveFile(filePath);
if (saveFile.open(QIODevice::WriteOnly)) {
    // Write the database.
    // ...
    if (saveFile.commit()) {
        return true;
    }
}
```

Qt 6.12.0 creates a same-directory temporary file through `QTemporaryFileEngine`, in
[`src/corelib/io/qsavefile.cpp:147-177`](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/src/corelib/io/qsavefile.cpp#L147-L177):

``` cpp
// src/corelib/io/qsavefile.cpp:166-168
// Until commit, ensure the temporary file is not accessible to a third party.
static_cast<QTemporaryFileEngine *>(fileEngine.get())->initialize(finalFileName, 0600);
if (!fileEngine->open(mode | QIODevice::Unbuffered)) {
```

On commit, Qt calls `renameOverwrite()` for the final path, in
[`src/corelib/io/qsavefile.cpp:380-417`](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/src/corelib/io/qsavefile.cpp#L380-L417):

``` cpp
// src/corelib/io/qsavefile.cpp:410-416
if (!fe->renameOverwrite(d->finalFileName)) {
    d->setError(fe->error(), fe->errorString());
    fe->remove();
    return false;
}
```

The stock AppArmor profile already models this sequence for configured password-store directories:

```apparmor
owner @{user_passwordstore_dirs}/ r,
owner @{user_passwordstore_dirs}/*.kdbx* rwl -> @{user_passwordstore_dirs}/#@{int},
owner @{user_passwordstore_dirs}/#@{int} rw,
```

The local rule for the separately selected `~/Plain/main.kdbx` database granted only
`main.kdbx* rwl`. It did not include the `#<integer>` temporary file or the rename target
relationship. Reading and opening the database therefore worked while saving failed.

## Policy change

Preserve the selected-path boundary and mirror only the stock atomic-save relationships:

```apparmor
# /etc/apparmor.d/local/keepassxc
audit owner "@{HOME}/Plain/main.kdbx*" rwl -> "@{HOME}/Plain/#@{int}",
audit owner "@{HOME}/Plain/#@{int}" rw,
```

Reload only KeePassXC:

```console
$ sudo apparmor_parser --replace --write-cache /etc/apparmor.d/keepassxc
Cache miss: /etc/apparmor.d/keepassxc
Wrote cache: /etc/apparmor/earlypolicy/48da56f3.0/keepassxc
```

This does not grant broad home access, arbitrary `~/Plain` file contents, a directory-wide file
permission class, or access to other database locations. The existing directory rule remains
read-only. AppArmor's file and rename rules authorize the demonstrated operations; no additional
directory write rule was needed in the measured sequence.

## Reproduction and verification

### Failing pattern

Before the change, an inline confined fixture attempted to create the exact denied temporary path
with exclusive creation, write a fixed non-secret payload, rename it to
`main.kdbx.apparmor-save-probe-2026-11-25`, and read it back. It failed at the same operation:

```text
Error: EACCES: permission denied, open '/home/user/Plain/#14711592'
confined_atomic_save_exit=1
red_repro_left_no_files=yes
```

The first harness attempt used a script under `~/temp/agent` and failed while reading that script.
That was a harness-location error, not the save reproduction. The inline harness reached the exact
database-directory operation without adding any `~/temp/agent` grant.

### Working pattern

After the policy reload, the same create-write-rename-readback sequence returned:

```text
{
  "temporaryPath": "/home/user/Plain/#14711592",
  "finalPath": "/home/user/Plain/main.kdbx.apparmor-save-probe-2026-11-25",
  "readback": "keepassxc-atomic-save-apparmor-probe\n"
}
independent_readback=keepassxc-atomic-save-apparmor-probe
```

The audit records changed to `AUDIT`, including `mknod`, `open`, `rename_src`, `rename_dest`, and
the final read. Their paired `SYSCALL` records showed `success=yes`: `openat` returned a descriptor,
`rename` returned `0`, and the final `openat` returned a descriptor. Explicitly audited successful
operations need this pairing; an `AUDIT` label alone is not the success assertion.

An intermediate harness revision called Node's `FileHandle.sync()` and encountered
`ERR_ACCESS_DENIED: fsync API is disabled when Permission Model is enabled`. That Node restriction
was separate from AppArmor and from KeePassXC. The fixture was verified by exact ownership, size,
and content before removal. The final harness omitted that unrelated Node operation and exercised
the policy boundary at issue.

Both fixture paths were checked for absence before use. Only the exact fixture with the expected
UID, size, and payload was removed after independent readback. The real `main.kdbx` was not opened
for writing or modified by this verification.

`verified-selected-keepassxc-atomic-save-fix-2026-11-25T16-48-29-0500.json` records all 809 expected
cache artifacts, exact live feature bytes, a KeePassXC cache hit, selection, reload, and rollback
preservation.

## Current configuration boundary

KeePassXC's source defaults to `UseAtomicSaves=true`, `UseDirectWriteSaves=false`, and
`BackupBeforeSave=false`, in
[`src/core/Config.cpp:65-68`](https://github.com/keepassxreboot/keepassxc/blob/1c2f23ef5914c0f442a00533def1689857afb5a7/src/core/Config.cpp#L65-L68).
Neither current settings file overrides those entries. This repair therefore covers the active
atomic-save configuration without adding permissions for disabled backup or alternate save modes.

The fixture proves the exact AppArmor file and rename boundary. It does not claim that the owner's
real GUI save succeeded, that a different database path is authorized, or that optional backups
and non-atomic modes were exercised. The owner must retry the real save for that separate result.

## Upstream disposition

No KeePassXC or Qt filing is warranted for this record. Atomic replacement is the documented and
source-verified behavior, and the stock profile already contains the required pattern. The defect
was the omission in the locally authored selected-database rule.
