# Handover: kopia to pCloud backup on the Pixel 6

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Goal:
 an unattended,
 scheduled,
 reboot-surviving backup of the user's Pixel 6 to pCloud,
 using
kopia,
 without staging a second copy of the data on the device.

This file is the in-repo successor to a handover that used to live in a Seafile-synced notes
folder.
 That copy was reverted externally and its session edits were lost,
 so the working copy was
relocated here on 2026-06-07.
 The stack vet and its rationale live next to this file at
`doc/decision/kotlin-android-kopia-pcloud-stack.md`,
 with raw per-technology reports under
`doc/decision/kotlin-android-kopia-pcloud-vet-report/`.

## Current direction (decided 2026-06-07)

Build a native Android app that runs the stock kopia binary against a local,
 in-app S3 endpoint
which translates S3 calls to pCloud's native HTTP API,
 streaming bytes through.
 No kopia fork,
 no
VM,
 no rclone,
 no WebDAV.

- UI and app shell:
   native (plain) Jetpack Compose,
   Kotlin.
   Owner-selected after a four-stack vet
  that built and ran every candidate on the device.
- Local S3 server (the "facade" that lies to kopia):
   Ktor CIO,
   in-process.
- pCloud client inside the facade:
   OkHttp 5.3.2 / Retrofit 3.0.0 (Retrofit pulls stable OkHttp
  4.12.0),
   streaming.
- Background:
   WorkManager plus a dataSync foreground service runs the kopia snapshot.
- kopia:
   the stock arm64 binary,
   bundled and exec'd (see the hard-constraints list).

## Why this shape (how we got here)

- The target is pCloud specifically.
   pCloud offers a native HTTP API and a fragile WebDAV gateway,
  nothing else (no S3,
   B2,
   SFTP,
   GCS).
- kopia has no plugin system:
   backends are compiled into the single binary,
   so a "pCloud backend"
  would be a kopia fork.
   The owner ruled out forking kopia.
- pCloud's WebDAV gateway is eventually consistent and breaks kopia's storage contract.
   That is
  what broke kopia's `webdav` backend and its `rclone` backend (kopia's rclone backend runs
  `rclone serve webdav` internally,
   so it is WebDAV too).
- pCloud's native HTTP API,
   by contrast,
   is strongly consistent (verified,
   see the probe section).
- rclone is out entirely (owner:
   its VFS cache is unreliable in practice and it has too many rough
  edges).
- Conclusion:
   do not modify kopia and do not depend on pCloud's WebDAV.
   Instead run a local server
  that speaks a stock kopia backend protocol (S3) and translates to pCloud's native API.
   kopia is
  unmodified;
   it just thinks it is talking to S3 on localhost.
   Because that server is local and
  synchronous,
   it owns read-after-write itself.

## The S3 facade design (grounded in kopia repo/blob/s3)

Read against the real kopia S3 backend at `repo/blob/s3/s3_storage.go` (cloned this session).
 The
facade is small:

- Multipart is disabled by kopia (`PutObject DisableMultipart: true`,
   s3_storage.
  go:
  176,
   with a
  comment that kopia already splits into small blobs).
   The facade implements only single-shot PUT.
- TLS off:
   kopia uses `Secure = !DoNotUseTLS` (line 398).
   Configure the repo with `DoNotUseTLS` so
  kopia talks plain HTTP to localhost;
   the facade serves HTTP,
   no certs.
- Auth ignored:
   minio signs SigV4 (line 353) but on localhost the facade trusts the caller and
  accepts any signature and any access key.
- Region static from config (line 399);
   set Region so minio does no `GetBucketLocation`.
   The
  connect path does not call bucket-exists;
   just pre-create the folder.
- ETag:
   compute MD5 of the PUT bytes in the facade and return it as the ETag.
   Do not rely on
  pCloud's md5 (EU accounts return sha1 and sha256,
   not md5).
   kopia sends Content-MD5
  (`SendContentMd5: true`,
   line 181),
   so the facade can verify against its own MD5.
- Keys are flat:
   `getObjectNameString = Prefix + blobID` (lines 249 to 251);
   blobIDs contain no
  slash.
   With an empty prefix each object is one file named exactly the key.
   Map bucket to one
  pCloud folder,
   object to a file in it,
   addressed with pCloud path addressing
  (`path=/<bucket>/<key>`);
   no fileid cache.

The whole operation set kopia calls,
 mapped to pCloud native methods:

- GET object,
   full or Range (GetBlob,
   lines 40 to 85;
   `length==0` is an existence probe;
  `length<0` is full) maps to `getfilelink?path=/<bucket>/<key>` then a ranged GET to the returned
  host,
   streamed back.
   Must return exactly the requested bytes (kopia calls `EnsureLengthExactly`,
  line 84).
- HEAD object (GetMetadata via StatObject,
   lines 114 to 131) maps to `checksumfile` for size and
  timestamp.
- PUT object,
   single-shot,
   may be 0 bytes (PutBlob,
   lines 133 to 219;
   empty-stream case at 197)
  maps to `uploadfile` (bucket folder,
   `filename=key`,
   `nopartial=1`).
   Overwrite is in place.
- DELETE object,
   idempotent (DeleteBlob,
   lines 221 to 228;
   404 counts as success) maps to
  `deletefile`;
   return 204 even if absent.
- LIST objects v2 by prefix,
   flat,
   paginated (ListBlobs,
   lines 253 to 286;
   only `Prefix`,
   no
  delimiter) maps to one `listfolder` of the bucket folder,
   filtered by prefix client-side.
- PutObjectRetention (ExtendBlobRetention,
   lines 230 to 247) is a no-op success stub;
   only called
  for object-lock repos.
- Versioning and point-in-time (`s3_versioned.go`) are ignored unless configured.

kopia repo config to use:
 endpoint `127.0.0.1:<port>`,
 a fixed region,
 bucket is the folder name,
empty prefix,
 `DoNotUseTLS=true`,
 dummy access key and secret.

## pCloud native-API verification (probe results, 2026-06-07)

Verified against the owner's real pCloud account (EU region,
 `eapi.pcloud.com`) with a small Bun
probe (`/tmp/agent/pcloud-settle-probe.ts` and `pcloud-openq-verify.ts`).
 The auth token was read
in memory from a snapshot of pcloudcc's `~/.pcloud/data.db`,
 never printed or persisted;
 all writes
went to a throwaway folder that was deleted afterward.

Consistency (zero eventual-consistency misses):

- 20 sequential 64 KiB blobs:
   read-after-write first-attempt success 20 of 20 (latency to readable
  about 205 to 278 ms,
   just the HTTP round-trip,
   not a settle delay);
   listfolder visibility 20 of
  20 (about 100 ms).
- 12 parallel uploads:
   all 12 visible on the immediate next listfolder call (101 ms).

Open questions,
 all resolved in favor of the simple design:

- Overwrite:
   `uploadfile` replaces in place (same fileid before and after,
   one listing entry,
   new
  content and size win).
   So S3 PUT maps to a plain uploadfile,
   no delete-first.
   Path addressing
  (`path=/<bucket>/<key>`) confirmed working.
- Ranges:
   content hosts honor arbitrary byte ranges exactly,
   HTTP 206,
   including mid-file,
   tail,
  sub-range,
   and open-ended (`bytes=N-`).
   GET with Range maps directly.
- listfolder scale:
   returns all entries in one call,
   no cap or pagination through 1000-plus files;
  latency flat (about 135 to 262 ms,
   round-trip bound,
   not count bound),
   response grows linearly
  (about 0.35 KiB per file,
   about 347 KiB at 1000).
   Sharding is optional,
   only worth it at tens of
  thousands of blobs.

Caveat:
 the sample is small and one-shot on one region.
 Before shipping,
 run a larger soak and an
interrupted-upload atomicity test.
 Also,
 a shipped app should use proper pCloud OAuth,
 not the
scraped pcloudcc session token.

## The Android app

### Architecture

- Bundle the stock kopia arm64 binary as `jniLibs/arm64-v8a/libkopia.so` with
  `extractNativeLibs=true`,
   and exec it from `applicationInfo.nativeLibraryDir` via `ProcessBuilder`
  (the app data dir is noexec since API 29;
   nativeLibraryDir is the exec-allowed path).
   Verified
  on-device by the stack vet;
   language-neutral.
- The S3 facade is in-process (Ktor CIO) on `127.0.0.1:<port>`,
   implementing the handlers above and
  translating to pCloud's native API via OkHttp.
- First run:
   OAuth to a bearer token and region,
   start the facade,
   ensure the bucket folder exists,
  then `libkopia.so repository create s3 --bucket=<folder> --endpoint=127.0.0.1:<port>
  --disable-tls --access-key=<dummy> --secret-access-key=<dummy> --region=<fixed>` with
  `KOPIA_PASSWORD` and config and cache dirs under `filesDir`;
   otherwise `repository connect s3`.
- Per backup:
   WorkManager periodic promotes to a dataSync foreground service,
   starts the facade,
  runs `libkopia.so snapshot create <paths>`,
   occasionally `kopia maintenance run`,
   then stops the
  facade and records the result.

### Hard constraints (from the vet, apply to any stack)

- The OS is GrapheneOS.
   INTERNET is a revocable per-app permission (default off),
   so the app must
  request Network access.
   Dynamic code loading is restricted;
   this is what disqualified Slint (its
  Android backend loads a Java helper via `InMemoryDexClassLoader`,
   raising a SecurityException at
  startup).
- Foreground-service time cap (API 35-plus):
   about 6 hours of dataSync per 24 hours,
   after which
  `onTimeout` fires.
   A kopia snapshot can exceed that,
   so the backup must be chunked,
   resumable,
   and
  onTimeout-checkpointed,
   and the bulk transfer should run as a user-initiated data transfer job,
  not a bare dataSync service.
- 16 KB page alignment:
   native shared objects need it on Android 15 and 16.
   Link the kopia `.so`
  (and any native `.so`) with `-Wl,-z,max-page-size=16384`.

### Verified component picks (Kotlin path)

- Local S3 server:
   Ktor CIO served HTTP 200 in-app on the device.
   Rejected:
   NanoHTTPD (dormant,
  blocking),
   Spring Boot (heavy),
   `java.net.http` (no Ktor engine path).
- pCloud client:
   OkHttp 5.3.2 / Retrofit 3.0.0,
   streaming verified.
- Testing:
   `createAndroidComposeRule` in-process plus Maestro black-box (set
  `Modifier.semantics { testTagsAsResourceId = true }`);
   Jazzer for JVM fuzzing;
   Kotest for
  property tests.
   Pin androidx.
  test 3.7.0,
   runner 1.7.0,
   core 1.7.0 on Android 15 and 16.
- Secrets:
   DataStore plus Tink plus Android Keystore.
   Do not use EncryptedSharedPreferences
  (androidx security-crypto was deprecated April 2025).
   Plain DataStore for non-secret prefs.

### UI, data model, and flows (paper design)

Screens:

- Onboarding:
   connect pCloud via OAuth in a Custom Tab (show account and region);
   grant storage
  (MANAGE_EXTERNAL_STORAGE for whole `/sdcard`,
   or SAF folder picks);
   set the kopia repo password
  (Keystore;
   warn that losing it makes backups unrecoverable);
   initialize the repo (pick a pCloud
  folder as the bucket).
- Main and status:
   pCloud and repo state;
   source paths;
   schedule with constraints;
   last run and
  next run;
   a "Back up now" button.
- Settings:
   change password,
   retention and maintenance,
   excludes (`.kopiaignore`),
   notifications,
  disconnect pCloud,
   export and import config.
- Notifications:
   the foreground-service notification during a backup,
   plus completion and failure.

Data model:

- Keystore-encrypted:
   pCloud token,
   hostname (`eapi`) and account;
   kopia repo password;
   dummy S3
  creds (random per install).
- Plain prefs:
   bucket folder,
   repo prefix,
   source paths,
   excludes,
   schedule and constraints,
  last-run metadata (timestamp,
   exit code,
   summary,
   bytes,
   snapshot id).
- kopia config and cache under `filesDir/kopia/`.
   Only the kopia binary needs the exec-allowed
  nativeLibraryDir;
   its config and cache are ordinary files in the noexec data dir,
   which is fine.

OAuth:
 prefer the token flow (`response_type=token`) to avoid embedding a client secret (verify
pCloud supports it;
 the fallback is the code flow with `oauth2_token` plus an embedded secret,
 not
truly secret but acceptable for a personal app).
 Register an app at pCloud for the client id;
 the
`redirect_uri` is a custom-scheme intent filter;
 capture the token and hostname;
 store in Keystore.
pCloud tokens are long-lived,
 so a 401 means re-auth.

Phasing:
 P1 (OAuth,
 storage,
 repo init,
 manual "Back up now" via the foreground service,
 status);
P2 (WorkManager periodic plus constraints,
 notifications,
 maintenance,
 the 6-hour-cap chunking);
P3 (restore and browse via kopia's web UI in a WebView,
 retention UI,
 multi-source and excludes).

## Device facts (verified on-device)

- Model Pixel 6,
   codename oriole.
   adb over USB,
   serial 1C171FDF600KWW.
- GrapheneOS,
   Android 16,
   SDK 36,
   ABI arm64-v8a.
   Secure lock is a password.
- Consequence of the password lock plus credential-encrypted storage:
   anything that reads `/sdcard`
  or the Keystore (the whole backup) can only run after the first manual unlock following a reboot.
  An app does not escape this;
   it gives cleaner scheduling,
   not hands-off pre-unlock backup.
- AVF Linux Terminal VM present (`com.android.virtualization.terminal`),
   hypervisor kvm.
  arm-nvhe,
  `/dev/kvm` present,
   protected_vm unsupported.
- Termux suite installed:
   com.
  termux,
   com.
  termux.
  boot,
   com.
  termux.
  api,
   com.
  termux.
  widget,
  com.
  termux.
  styling,
   com.
  termux.
  window.
   com.
  termux has SYSTEM_ALERT_WINDOW allowed and shares
  uid 10160 with com.
  termux.
  boot.

## VM autostart (solved, now a fallback)

The chosen app path does not use the AVF VM,
 but the VM-replication fallback (option A:
 run the
proven host pcloudcc plus kopia setup inside the AVF Debian VM) had one open blocker,
 the VM not
auto-starting on reboot.
 That is now solved and recorded here in case the app path stalls.

- The Terminal app has no headless boot path of its own:
   only its `.MainActivity` boots the VM (its
  only BOOT_COMPLETED receivers are WorkManager bookkeeping).
   The VM runs under a foreground service
  that MainActivity starts;
   backgrounding the activity does not kill the VM (crosvm_debian stays
  resident at about 700 MB after pressing HOME).
- Trigger:
   a Termux:
  Boot script `~/.termux/boot/30-start-vm.sh` that wake-locks,
   runs
  `am start -n com.android.virtualization.terminal/.MainActivity`,
   then after a 2-second delay runs
  `am start -a android.intent.action.MAIN -c android.intent.category.HOME` so the Terminal UI does
  not linger,
   then wake-unlocks.
   It uses Termux's own `am`
  (`/data/data/com.termux/files/usr/bin/am`);
   `/system/bin/am` is the fallback.
   Installer saved at
  `/tmp/agent/sv.sh`.
- Background launch is allowed because com.
  termux holds SYSTEM_ALERT_WINDOW (the BAL exemption);
   the
  boot script inherits it via the shared uid.
   To regrant:
  `adb shell appops set com.termux SYSTEM_ALERT_WINDOW allow`.
- Verified:
   rebooted,
   unlocked once,
   did not open the Terminal app;
   the script fired and
  crosvm_debian came up on its own.
   The HOME-bounce variant was foreground-verified (screen ended on
  the launcher,
   VM alive).
- Residual:
   a short (about 2 second) Terminal flash on each boot is structural,
   the VM service only
  starts while MainActivity is briefly foreground.
   Accepted by the owner.

## Host reference setup (bazzite), the proven design

The Linux host already runs kopia to pCloud and it works;
 it is the reference the app reproduces in
miniature.
 Do not copy credentials out of these files into shared notes.

- pcloudcc at `/usr/local/bin/pcloudcc`,
   run as a systemd user service
  (`~/.config/systemd/user/pcloud.service`,
   Type=forking,
  `ExecStart=/usr/local/bin/pcloudcc -u <account> -d -m /mnt/pcloud`).
   DB and cache at `~/.pcloud`,
  a symlink to `/var/mnt/encrypted/pcloud` (a LUKS volume).
- kopia 0.23.0 with a plain filesystem repo inside the mount
  (`~/.config/kopia/repository.config`,
   `storage.type=filesystem`,
   `path=/mnt/pcloud/rclone`;
   the
  "rclone" folder name is a leftover).
   Password in
  `~/.config/kopia/repository.config.kopia-password` (do not echo it).
- Schedule:
   `~/.config/systemd/user/kopia-snapshot-all.{service,timer}`,
   hourly.
- Why it works:
   pcloudcc's pfs is a caching filesystem with a local SQLite db,
   so it supplies
  read-after-write locally while uploading to pCloud asynchronously.
   The app reproduces that
  "local consistent store over pCloud's native API" idea without FUSE.

## What is already proven on-device (earlier testing)

- kopia did a full S3 round-trip from bare Termux against a Garage S3 server on the host bridged
  with `adb reverse tcp:3900` (snapshot and restore byte-identical).
   So kopia's S3 backend works on
  the phone,
   which is the foundation for the facade.
- AVF VM path:
   kopia did a filesystem-repo round-trip in the guest (byte-identical).
- Bare-Termux gotchas,
   both fixed:
   missing CA trust store (`pkg install ca-certificates`,
  `export SSL_CERT_FILE=$PREFIX/etc/tls/cert.pem`);
   kopia "cannot determine current user"
  (`export USER=termux`).
   DNS works in bare Termux.
- proot and termux-chroot are broken on this kernel (`ptrace(TRACEME): Operation not permitted`);
  not needed since bare Termux works.

## Current device state

Installed and intentionally kept:

- Termux:
   kopia at `$PREFIX/bin/kopia` (v0.23.0 arm64),
   ca-certificates installed.
- VM guest:
   kopia at `~/bin/kopia`.
- Doze battery-optimization whitelist entries:
   com.
  termux,
   com.
  termux.
  boot.
- Termux storage set up:
   `~/storage` points to `/sdcard`.
- Termux:
  Boot launcher `~/.termux/boot/30-start-vm.sh` (the VM autostart fallback).

Changed and not yet reverted:

- `screen_off_timeout` set to 1800000 (30 min).
   `svc power stayon` already reverted to false.

## Host-side artifacts (under /tmp/agent, ephemeral)

- Probe and driver scripts:
   `pcloud-settle-probe.ts`,
   `pcloud-openq-verify.ts`,
   `sv.sh`,
   plus the
  older Termux and Garage test scripts.
   No real user credentials are stored here.
- Source clones:
   `kopia-src-20260607` (the S3 backend read for the facade design),
  `rclone-src-20260607`,
   `pcloudcc-console-client-20260607`.

## How to drive the phone (method notes)

- adb over USB,
   serial 1C171FDF600KWW.
   Neither Termux nor the VM guest can be shelled directly via
  adb;
   drive them through the app UI (`adb shell input text '...'` with spaces as `%s`,
  `adb shell input keyevent 66` for Enter,
   `adb exec-out screencap -p` for screenshots).
- File and output bridge:
   push a script to `/sdcard/Download`,
   run it in the app with
  `bash ~/storage/downloads/<script>.sh`,
   write output to `~/storage/downloads`,
   then `adb pull`.
- A host server reachable from Termux:
   `adb reverse tcp:PORT tcp:PORT`.
   adb reverse does not reach
  the VM guest (separate network namespace).

## What is left to do

- Build the app per the phasing above,
   starting with the host prototype of the facade plus a real
  kopia snapshot and restore against pCloud,
   then the P1 Android MVP.
- Before relying on it:
   a larger pCloud native-API soak and an interrupted-upload atomicity test;
  switch from the scraped pcloudcc token to proper pCloud OAuth.
- Investigate why the previous Seafile-hosted handover got reverted (owner has since removed
  Seafile;
   cause still open).
- Optionally revert `screen_off_timeout`.
