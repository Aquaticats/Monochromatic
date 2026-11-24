# KeePassXC 2.8.0 with Qt 6.12.0 stalls during Helium native messaging when AppArmor omits `flock` permission

## Symptom

The user reported: "KeepassXC desktop seems to be not responding when I use the browser extension in
Helium?" KWin created `kwin_killer_helper` for `main.kdbx [Locked] - KeePassXC`, confirming that the
GUI was not answering window-manager pings.

The native-messaging proxy was already running as `/usr/bin/keepassxc-proxy`, under the enforcing
`helium` profile. Its Unix socket was connected to KeePassXC, but data was queued in both
directions. KeePassXC remained enforcing as `keepassxc (enforce)`.

A D-Bus introspection call initially appeared to recover while the GUI was still stalled. That was a
false-green test: the all-thread debugger capture showed `QDBusConnection` responding on a worker
thread while the GUI thread remained blocked.

## Root cause

### KeePassXC synchronizes an XDG state file

KeePassXC 2.8.0 selects `XDG_STATE_HOME/keepassxc/keepassxc.ini`, falling back to
`~/.local/state/keepassxc/keepassxc.ini`, in
[`src/core/Config.cpp:607-627`](https://github.com/keepassxreboot/keepassxc/blob/1c2f23ef5914c0f442a00533def1689857afb5a7/src/core/Config.cpp#L607-L627):

``` cpp
// src/core/Config.cpp:607-627
configPath = QStandardPaths::writableLocation(QStandardPaths::GenericConfigLocation) + "/keepassxc";
// ...
localConfigPath = xdgStateHome + "/keepassxc";
// ...
configPath += QString("/keepassxc%1.ini").arg(suffix);
localConfigPath += QString("/keepassxc%1.ini").arg(suffix);
```

The GUI thread's real backtrace was:

```text
nanosleep
QLockFile::tryLock(std::chrono::duration<...>)
QSettings::sync()
QTimer::timerEvent(QTimerEvent*)
QApplicationPrivate::notify_helper(...)
QEventLoop::exec(...)
```

A debugger `catch syscall openat` probe then captured the exact path:

```text
"/home/user/.local/state/keepassxc/keepassxc.ini.lock"
#1 open64()
#2 QLockFile::tryLock(...)
#5 QSettings::sync()
```

### `rwl` does not permit Qt's native lock

The local AppArmor rule granted `rwl`, meaning read, write, and link permission. It did not grant
`k`. Qt 6.12.0 calls `flock(fd, LOCK_EX | LOCK_NB)` in
[`src/corelib/io/qlockfile_unix.cpp:147-176`](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/src/corelib/io/qlockfile_unix.cpp#L147-L176):

``` cpp
// src/corelib/io/qlockfile_unix.cpp:147-176
static NativeLocking setNativeLocks(int fd)
{
    int ret;
    // ...
    QT_EINTR_LOOP(ret, flock(fd, LOCK_EX | LOCK_NB));
    // ...
    case EACCES:
        return NativeLocking::Failed;
}
```

File creation was allowed, but native locking failed. The resulting lock files were empty and the
GUI thread kept retrying.

Qt then attempted stale-lock cleanup with a new `.rmlock` suffix, in
[`src/corelib/io/qlockfile.cpp:378-409`](https://github.com/qt/qtbase/blob/025bdad181de241e81bf853c8a2d7bf3d19261a9/src/corelib/io/qlockfile.cpp#L378-L409):

``` cpp
// src/corelib/io/qlockfile.cpp:378-409
bool QLockFile::tryLock(std::chrono::milliseconds timeout)
{
    // ...
    case LockFailedError:
        if (!d->isLocked && d->isApparentlyStale()) {
            // Ensure two processes don't remove it at the same time
            QLockFile rmlock(d->fileName + ".rmlock"_L1);
            if (rmlock.tryLock()) {
                if (d->isApparentlyStale() && d->removeStaleLock())
                    continue;
            }
        }
}
```

Because each new lock attempt had the same missing `k` permission, the directory accumulated 34
empty artifacts: the base lock plus 33 successive `.rmlock` suffixes. No process held any of them
open.

The base name is 18 bytes. The longest name was 249 bytes, and another suffix would make 256 bytes.
`getconf NAME_MAX /home/user/.local/state/keepassxc` returned 255. At that point no deeper file
could be created, so the denial stream stopped while the GUI remained blocked. An empty current
audit window was therefore not evidence of recovery.

### The native-host manifest was already discoverable

Copying the manifest into Helium's own directory would not fix this stall. Helium 0.22.2 explicitly
falls back to Chrome's user native-messaging directory in
[`scan-chrome-native-messaging-hosts.patch:5-19`](https://github.com/imputnet/helium/blob/f32f2c066787492b9ba112dc1be1ad230b53658a/patches/helium/core/scan-chrome-native-messaging-hosts.patch#L5-L19):

``` diff
 result = FindManifestInDir(chrome::DIR_USER_NATIVE_MESSAGING, host_name);
+result = FindManifestInDir(chrome::DIR_CHROME_USER_NATIVE_MESSAGING, host_name);
 result = FindManifestInDir(chrome::DIR_NATIVE_MESSAGING, host_name);
+result = FindManifestInDir(chrome::DIR_CHROME_NATIVE_MESSAGING, host_name);
```

The existing `google-chrome/NativeMessagingHosts/org.keepassxc.keepassxc_browser.json` manifest was
therefore sufficient. The Chromium abstraction also already allowed proxy execution and both browser
socket paths.

## Verification

The installed versions were KeePassXC 2.8.0-4.1, Helium 0.22.2.1-1, and Qt 6.12.0-5. The source
revisions were KeePassXC `1c2f23ef5914c0f442a00533def1689857afb5a7`, Helium
`f32f2c066787492b9ba112dc1be1ad230b53658a`, and Qt `025bdad181de241e81bf853c8a2d7bf3d19261a9`.

### Failing pattern before the fix

With `owner @{user_state_dirs}/keepassxc/* rwl,`:

- The GUI thread remained in `QLockFile::tryLock()` and `clock_nanosleep`.
- KWin's unresponsive-window helper was present.
- `ss -xpn` showed a connected proxy with 7,680 bytes queued on one side and 1,911 on the other.
- The state directory contained 34 empty lock artifacts and no valid PID contents.
- No process descriptor referred to any of those 34 files.

### Working pattern after the fix

With `owner @{user_state_dirs}/keepassxc/* rwlk,` and the stale artifacts removed:

- The GUI thread returned to `ppoll`, and KWin's helper disappeared.
- Both native socket queues returned to zero.
- `.local/state/keepassxc/keepassxc.ini` was successfully rewritten.
- Reopening the real extension popup kept both queues at zero.
- No `.rmlock` chain returned.
- No new KeePassXC or Helium denial appeared after the second lock-path adjustment.
- KeePassXC and the proxy remained under enforcing profiles.

All four pinned source excerpts were rechecked through GitHub's contents API. The visible-browser
citation check was stopped after repeated launcher failures created unwanted windows; it is not
claimed as completed browser evidence.

The final checks were:

``` sh
sudo cat /proc/4823/task/4823/stack
ss -xpn | rg '317694|308117|keepass|kpxc'
ls -la /home/user/.local/state/keepassxc
```

The relevant results were `__x64_sys_ppoll`, two `ESTAB 0 0` socket rows, and only `keepassxc.ini`
besides the directory entries. PID values and socket inodes are evidence for this run, not stable
values to reuse on another run.

The stale-removal evidence is private at
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/keepassxc-native-messaging-lock-2026-11-24/removed-empty-unheld-stale-locks.json`.

## Verified workarounds

### Permit native locking on the existing state files

``` diff
-owner @{user_state_dirs}/keepassxc/* rwl,
+owner @{user_state_dirs}/keepassxc/* rwlk,
+# Qt also synchronizes the application-name fallback settings lock.
+owner @{user_config_dirs}/KeePassXCrc.lock rwlk,
```

Reload only the affected profile and its cache:

``` sh
sudo apparmor_parser --replace --write-cache /etc/apparmor.d/keepassxc
```

The tradeoff is that KeePassXC can take native advisory locks on its own state files and the exact
fallback settings lock. This grants no new paths, executable transitions, capabilities, or access to
other applications' locks. The fallback rule was added after two measured `mknod` denials for that
exact path; no broader `KeePassXCrc*` grant was added.

### Remove only proven stale artifacts

Removing lock files is safe here only because every matched file was empty, user-owned, matched the
exact base-plus-`.rmlock` pattern, and had no opener in any `/proc/<pid>/fd` directory. The executed
cleanup checked all of those conditions immediately before unlinking. Do not delete a nonempty or
held lock file.

The cleanup report records every removed name and metadata. Its tradeoff is that stale-lock evidence
is removed from the live directory, so the private report must be retained for diagnosis. Adding `k`
without removing these 34 stale files would leave the existing retry chain in place; removing them
without adding `k` would allow the chain to return.

## What does not work

- Checking only D-Bus introspection: Qt's `QDBusConnection` worker can respond while the GUI thread
  is blocked.
- Checking only recent AVCs: the 255-byte filename limit stopped new lock creation and new denials
  before the GUI recovered.
- Copying the native-host manifest into `net.imput.helium/NativeMessagingHosts`: Helium already
  scans the Chrome fallback directory, and the proxy was already running.
- Adding more `r`, `w`, or `l` permissions: none permits `flock()`. The required permission is `k`.
- Deleting a lock based only on age: a live or nonempty lock must not be treated like these verified
  empty, unheld artifacts.
- Disabling either browser or KeePassXC enforcement: unnecessary and contrary to the measured narrow
  fix.

## Upstream filing decision

The `.out-of-scope/` search for KeePassXC, Qt, AppArmor, and lock topics found no exemption. On
2026-11-24, `gh search issues 'QLockFile AppArmor' --owner keepassxreboot --limit 20` and the
matching `gh search prs` query returned no matches.

1.  **Is it really upstream's fault?** No. The local state-file rule omitted `k`; Qt's native
    `flock` call and KeePassXC's settings synchronization are normal behavior.
2.  **Can upstream fix it?** No upstream code change is needed for this incident. The consumer-side
    AppArmor rule fixes the boundary.
3.  **Are they supporting this use case?** KeePassXC supports browser integration and Qt supports
    native locking, but that does not make the local policy omission an upstream defect.
4.  **Would the repo welcome our contribution?** Not evaluated for filing because the first
    constraint fails. No contribution policy claim is made.
5.  **Will they likely fix it?** No fix request is recommended for this local policy error.
6.  **Have we prototyped a compatible fix?** The consumer-side policy change was applied and
    verified against the real GUI and extension. No upstream source prototype is needed.

The readiness check records these limits: maintained release-line testing beyond the installed
versions was not run; upstream intake and automation were not read because no filing is proposed;
open-branch work-in-progress checks were not run; the exact pre/post reproduction was run on the
real application; and no clean-runner workflow was created for this host-policy incident.

There is no recommendation to file. If future evidence identifies an upstream defect, filing remains
the poster's decision and carries account-reputation, trust-list, and blocklist risks that must be
checked again then.

### Upstream artifact: nothing to add

No new issue or additive comment is drafted because the demonstrated cause and complete fix are
local to this host's AppArmor policy. An upstream report would incorrectly assign the locally
authored `rwl` rule to KeePassXC or Qt.
