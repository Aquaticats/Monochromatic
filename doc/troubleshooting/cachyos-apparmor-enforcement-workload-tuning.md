# AppArmor 4.1.8 default enforcement breaks local snapshot, updater, browser, password-store, development, and container paths

## Status

Current-boot repairs are measured. The final enforcement cold boot and owner review remain open.
This is a local deployment record, not an upstream bug report or a claim that every profile is
enforcing.

The rollout uses the broad `apparmor.d` collection at commit
`5c34009a6ea104831c9a91a578f9255dbb289fe2`, with `default enforce`, `include default`, `reload yes`,
and `fsp no`. Package complain and unconfined exceptions remain intentional.

## Symptom

Default enforcement exposed independent workload failures:

- Sanoid 2.3.0 exited with status 2 after snapshot creation. Its journal said
  `Timed out waiting for subprocesses to start at /usr/share/perl5/vendor_perl/Capture/Tiny.pm line 271.`
- `checkupdates` 1.13.1 could not create the `local` symlink selected by Cachy-Update 4.4.7. The
  parent user service could still exit 0 with an empty or misleading package comparison.
- `aa-install` and `aa-mode` were denied their transient scope's `cpu.max` file during Go runtime
  initialization.
- Incus 7.6.0 could launch no test instance because `apparmor_parser` was denied its generated
  cache's `.features` file.
- Git could not create source-audit directories beneath `/home/user/temp/agent`.
- Helium was denied `/etc/os-release` and initially used Chromium's configuration and cache
  directory names. Its crashpad child was denied the real `net.imput.helium/Crash Reports` paths.
- The existing KeePassXC 2.8.0 process repeatedly failed to read `/home/user/Plain/main.kdbx`.

These are separate AVCs and workloads. A successful parent unit, a loaded profile name, or an empty
later denial query does not prove that any of them succeeded.

## Root cause

### Sanoid uses two different temporary filename shapes

[Sanoid's snapshot
path](https://github.com/jimsalterjrs/sanoid/blob/8d4abf14b2bf8d6b98fc8e163fa9895486e8cdeb/sanoid#L664-L674)
calls `tee_stderr` around the real `zfs snapshot` operation:

```perl
# sanoid:666-673
my $stderr;
my $exit;
($stderr, $exit) = tee_stderr {
    if ($recursiveFlag) {
        system($zfs, "snapshot", "-r", "$snap");
    } else {
        system($zfs, "snapshot", "$snap");
    }
};
```

Capture::Tiny 0.50 creates a readiness marker before waiting for its tee child. The [installed
`lib/Capture/Tiny.pm:202-217`](https://fastapi.metacpan.org/v1/source/DAGOLDEN/Capture-Tiny-0.50/lib/Capture/Tiny.pm#:~:text=scalar(%20tmpnam()%20)%20.%20%24%24)
uses this expression:

```perl
# lib/Capture/Tiny.pm:217
$stash->{flag_files}{$which} = scalar( tmpnam() ) . $$;
```

The exact installed module was compared byte for byte with the CPAN 0.50 archive, not with the newer
0.51 default branch. Both copies had SHA-256
`229bdb082963bbb29629da2ef61bf151fd0f4705800e45672127ed11162581a8`.

The installed File::Temp 0.2312 defines ten temporary characters, including underscore, at
`File/Temp.pm:207-223`. Its [`tmpnam`
implementation](https://github.com/Perl-Toolchain-Gang/File-Temp/blob/bb40d5b960961c5c37503461e6ca0d6759d37675/lib/File/Temp.pm#L1963-L1978)
applies that template:

```perl
# File/Temp.pm:1963-1975
sub tmpnam {
  my $tmpdir = _wrap_file_spec_tmpdir();
  croak "Error temporary directory is not writable"
    if $tmpdir eq '';
  my $template = File::Spec->catfile($tmpdir, TEMPXXX);
  if (wantarray() ) {
    return mkstemp($template);
  } else {
    return mktemp($template);
  }
}
```

The failed AVC `/tmp/2y2FMN44FE102311` contains ten File::Temp characters followed by confined
parent PID `102311`, with `fsuid=0` and `ouid=0`. The first narrow rule covered only the
ten-character file used by `tempfile`, not the readiness marker with its PID suffix. Repeated
scheduled runs reproduced that same shape.

### Cachy-Update overrides the checkupdates database location

[`checkupdates`](https://github.com/archlinux/pacman-contrib/blob/75d4a70517c649155959d28260198630fcb8fe51/src/checkupdates.sh.in#L132-L134)
normally uses `${TMPDIR:-/tmp}/checkup-db-${UID}/`:

```bash
# src/checkupdates.sh.in:132-134
if [[ -z $CHECKUPDATES_DB ]]; then
    CHECKUPDATES_DB="${TMPDIR:-/tmp}/checkup-db-${UID}/"
fi
```

The stock profile covers that default location.

Cachy-Update's
[`common.sh:24-32`](https://github.com/Antiz96/arch-update/blob/1010a4ea6b5597a1d7dbc938dbe9d6a5af7cc7dd/src/lib/common.sh#L24-L32)
instead sets a runtime prefix:

```bash
# src/lib/common.sh:24-32
statedir="${XDG_STATE_HOME:-${HOME}/.local/state}/${name}"
tmpdir="${XDG_RUNTIME_DIR:-/run/user/$UID}/${name}"
mkdir -p "${statedir}" "${tmpdir}" || exit 16
checkupdates_db_tmpdir_prefix="${tmpdir}/checkupdates-"
```

Its
[`check.sh:18-26`](https://github.com/Antiz96/arch-update/blob/1010a4ea6b5597a1d7dbc938dbe9d6a5af7cc7dd/src/lib/check.sh#L18-L26)
adds five `mktemp` characters and exports the override:

```bash
# src/lib/check.sh:18-26
checkupdates_db_tmpdir=$(mktemp -d "${checkupdates_db_tmpdir_prefix}XXXXX")
if [ -z "${no_version}" ]; then
    CHECKUPDATES_DB="${checkupdates_db_tmpdir}" checkupdates > "${statedir}/last_updates_check_packages"
else
    CHECKUPDATES_DB="${checkupdates_db_tmpdir}" checkupdates | awk '{print $1}' > "${statedir}/last_updates_check_packages"
fi
```

The installed `check.sh` matches tag `v4.4.7` byte for byte. Installed `common.sh` differs only in
CachyOS tray-icon names; the allocation lines match.

### Go retains the current cgroup quota descriptor

[Go's runtime
documentation](https://github.com/golang/go/blob/go1.27.4/src/runtime/cgroup_linux.go#L11-L16) says
startup opens the current cgroup's limit files and keeps them open for the process lifetime:

```go
// src/runtime/cgroup_linux.go:11-16
// cgroup-aware GOMAXPROCS default
//
// At startup (defaultGOMAXPROCSInit), we read /proc/self/cgroup and /proc/self/mountinfo
// to find our current CPU cgroup and open its limit file(s), which remain open
// for the entire process lifetime. We periodically read the current limit by
// rereading the limit file(s) from the beginning.
```

The cgroup-v2 branch of
[`OpenCPU`](https://github.com/golang/go/blob/go1.27.4/src/internal/runtime/cgroup/cgroup_linux.go#L93-L106)
opens `cpu.max` read-only and stores the descriptor:

```go
// src/internal/runtime/cgroup/cgroup_linux.go:93-106
case 2:
    n2 := copy(base[n:], v2MaxFile)
    path := base[:n+n2]
    maxFD, errno := linux.Open(&path[0], linux.O_RDONLY|linux.O_CLOEXEC, 0)
    if errno != 0 {
        return CPU{}, errSyscallFailed
    }
    c := CPU{version: 2, quotaFD: maxFD, periodFD: -1}
    return c, nil
```

The packaged `golang-strict` abstraction covered `system.slice/*.service/cpu.max`, but not the
observed transient `system.slice/*.scope/cpu.max`. This is a descriptor-lifetime behavior, so
command success or missing AVCs is weaker evidence than inspecting the real process's open
descriptor.

### Incus generates a separate parser profile and cache tree

Incus's
[`InstanceLoad`](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/apparmor/instance.go#L61-L77)
creates the namespace, generates the instance profile, then loads it:

```go
// internal/server/apparmor/instance.go:61-77
func InstanceLoad(sysOS *sys.OS, inst instance, extraBinaries []string) error {
    if inst.Type() == instancetype.Container {
        err := createNamespace(sysOS, InstanceNamespaceName(inst))
        if err != nil {
            return err
        }
    }
    err := instanceProfileGenerate(sysOS, inst, extraBinaries)
    if err != nil {
        return err
    }
    err = loadProfile(sysOS, instanceProfileFilename(inst))
    if err != nil {
        return err
    }
    return nil
}
```

Its
[`loadProfile`](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/apparmor/apparmor.go#L65-L76)
invokes `apparmor_parser` with the generated cache and profile directories. The measured AVC named
`/var/lib/incus/security/apparmor/cache/48da56f3.0/.features`, not the system early-policy cache.

The host-init label includes the generated outer profile and the named namespace.
[`driver_lxc.go:1085-1090`](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/instance/drivers/driver_lxc.go#L1085-L1090)
appends `//&:<instance namespace>:` when stacking is available. Consequently, an inner guest
`unconfined` label is not sufficient evidence of a missing outer profile.

### KeePassXC's selected database is outside the stock password-store path

The stock KeePassXC profile grants `*.kdbx*` beneath `@{user_passwordstore_dirs}`. This host leaves
that variable at `.password-store`, while the selected existing database is
`/home/user/Plain/main.kdbx`.

[KeePassXC 2.8.0's
`performSave`](https://github.com/keepassxreboot/keepassxc/blob/1c2f23ef5914c0f442a00533def1689857afb5a7/src/core/Database.cpp#L358-L395)
uses `QSaveFile` for atomic saves:

```cpp
// src/core/Database.cpp:358-382
bool Database::performSave(const QString& filePath, SaveAction action, const QString& backupFilePath, QString* error)
{
    if (!backupFilePath.isNull()) {
        backupDatabase(filePath, backupFilePath);
    }
    QFileInfo info(filePath);
    auto createTime = info.exists() ? info.birthTime() : QDateTime::currentDateTime();
    switch (action) {
    case Atomic: {
        QSaveFile saveFile(filePath);
        if (saveFile.open(QIODevice::WriteOnly)) {
            HashingStream hashingStream(&saveFile, QCryptographicHash::Md5, kFileBlockToHashSizeBytes);
            if (!hashingStream.open(QIODevice::WriteOnly)) {
                return false;
            }
            if (!writeDatabase(&hashingStream, error)) {
                return false;
            }
            saveFile.setFileTime(createTime, QFile::FileBirthTime);
            if (saveFile.commit()) {
                m_fileBlockHash = hashingStream.hashingResult();
                return true;
            }
        }
```

The current local rule covers the database basename and same-basename save or temporary suffixes.
Real database reads are proven. A save and rename at that path has not been exercised against a
disposable database, so this remains an explicit verification limit.

### Helium and Git require deployment-specific boundaries

The custom Helium profile originally inherited Chromium's configuration and cache directory
variables. That did not match this package's real `.config/net.imput.helium` and
`.cache/net.imput.helium` paths. `/etc/os-release` is root-owned, so an `owner` qualifier would not
match its `ouid=0` read.

Git's source-audit denials were confined to the private source-audit workspace. A later signed
commit also failed on `/tmp/.git_signing_buffer_tmpM40r71`.

[Git 2.56.1's SSH signing
path](https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/gpg-interface.c#L1092-L1112)
creates that exact six-character buffer, writes it, and passes it to the signer:

```c
// gpg-interface.c:1092-1112
buffer_file = mks_tempfile_t(".git_signing_buffer_tmpXXXXXX");
if (!buffer_file) {
    error_errno(_("could not create temporary file"));
    goto out;
}
if (write_in_full(buffer_file->fd, buffer->buf, buffer->len) < 0 ||
    close_tempfile_gently(buffer_file) < 0) {
    error_errno(_("failed writing ssh signing key buffer to '%s'"),
                buffer_file->filename.buf);
    goto out;
}
strvec_pushl(&signer.args, use_format->program,
             "-Y", "sign", "-n", "git", "-f", ssh_signing_key_file,
             NULL);
```

At `gpg-interface.c:1128-1130`, Git appends `.sig` and reads that signature file. It later removes
both temporary paths. This does not justify access to every personal directory, arbitrary temporary
files, or execution of repository hooks and build outputs.

Granting only the buffer exposed the next AVC:
`operation="exec" profile="git" name="/usr/bin/ssh-keygen" requested_mask="x"`. An unlisted
executable inside this confined Git profile does not automatically receive an unconfined fallback.
The narrow repair transitions only `ssh-keygen` to its existing enforcing profile and grants the
same buffer and `.sig` paths there.

## Verification

### Versions and evidence scope

The measured host runs AppArmor 4.1.8-2.1, Sanoid 2.3.0-2, Capture::Tiny 0.50-4, File::Temp from
Perl 5.44.0-1.1, Cachy-Update 4.4.7-1, pacman-contrib 1.13.1-1.2, Go 1.27.4-X:nodwarf5, Incus
7.6.0-1.1, KeePassXC 2.8.0-4.1, Git 2.56.1-1.1, and the custom confined Helium package.

Evidence is under `/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/`. Reports marked
`ownerAcceptance: false` are not final approval.

### Failing catalog

- An isolated `AppArmorProfile=sanoid` `tee_stderr` fixture timed out when only the ten-character
  File::Temp rule was installed. The AVC contained ten random characters followed by the parent PID.
- A ten-character-only Sanoid `tempfile` fixture was insufficient even though create, write, read,
  and delete succeeded for that different operation.
- The unchanged native `arch-update.service` denied `checkupdates` creation of
  `/run/user/1000/arch-update/checkupdates-1fHIo/local`.
- A harness-generated six-character `checkupdates-cMs5PD` directory remained denied after the
  five-character rule was installed. This was an incorrect fixture, not a reason to broaden the
  rule.
- An `ALLOWED`-only matcher missed explicitly audited successful KeePassXC operations. The kernel
  emitted `AUDIT` with paired `SYSCALL success=yes` instead.
- The first Incus namespace-assertion run completed its workload and cleanup, then failed the report
  writer's no-clobber guard. That was not a guest-confinement failure.
- A signed documentation commit initially failed with `fatal: failed to write commit object`. The
  corresponding AVC named `/tmp/.git_signing_buffer_tmpM40r71`; this was not a repository ownership
  problem. After the buffer rule, the next attempt failed with
  `fatal: cannot exec 'ssh-keygen': Permission denied`, with a separate exec AVC. The two denials
  required separate evidence.

### Passing catalog

- The corrected Sanoid tee fixture returned exact captured stderr. The already-scheduled 14:00 EST
  `sanoid.service` exited 0, and direct `zfs list -t snapshot` queries confirmed all three reported
  snapshots.
- `verify-sanoid-narrow-tempfile.ts sanoid-narrow-tempfile-readback` proved nonempty readback and
  deletion while the same confined PID's `/proc/<pid>/attr/current` read remained denied.
- `verify-checkupdates-runtime-database.ts` used native five-character `mktemp` allocation,
  confirmed raw target `/var/lib/pacman//local`, canonical target `/var/lib/pacman/local`, nonempty
  synchronized databases, package comparisons, and fixture removal.
- `verify-go-cpu-quota-descriptor.ts` observed each real enforcing helper with the expected scope's
  `cpu.max` open. Descriptor contents matched direct quota reads.
- The strengthened Incus workload proved the exact outer enforcing profile, namespace presence
  during the workload, nonzero host UID/GID mappings, payload readback, and namespace and profile
  removal afterward.
- KeePassXC PID 4210 remained enforcing and produced audited successful opens of its existing
  database.
- Fresh Helium rendered Example Domain with renderer seccomp filtering and no unexpected unconfined
  helper. Audited events proved its OS-release read, existing crash-report reads, settings
  read/write opens, and lock-file operations. It did not create a new crash dump.

## Verified workarounds

Install only the measured local rules and reload the affected profile. Rebuild the complete
early-policy cache after all local edits.

### Sanoid temporary files

```text
# /etc/apparmor.d/local/sanoid
owner /tmp/[0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z] rw,
owner /tmp/[0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z]@{pid} rw,
```

Tradeoff: these owner-matched flat paths cover the installed File::Temp character set and
Capture::Tiny PID suffix. They do not cover other temporary directories or arbitrary filenames.

### Cachy-Update database location

```text
# /etc/apparmor.d/local/checkupdates
owner @{run}/user/@{uid}/arch-update/checkupdates-@{rand4}@{c}/ w,
audit owner @{run}/user/@{uid}/arch-update/checkupdates-@{rand4}@{c}/local w,
```

Tradeoff: the five-character shape is specific to the installed wrapper. Pacman remains complain, so
this is not proof of enforcing pacman confinement. Remove the audit qualifier after final evidence
collection if routine audit volume is unwanted.

### Go quota descriptors

```text
# /etc/apparmor.d/local/aa-install and /etc/apparmor.d/local/aa-mode
owner @{sys}/fs/cgroup/system.slice/*.scope/cpu.max r,
```

Tradeoff: each helper can read any flat scope's quota beneath `system.slice`, not only its current
scope. The rule does not cover nested scopes or other cgroup files. Keep it profile-local instead of
broadening every Go application through the shared abstraction.

### Incus parser paths

```text
# /etc/apparmor.d/local/apparmor_parser
owner /var/lib/incus/security/apparmor/profiles{,/**} r,
owner /var/lib/incus/security/apparmor/cache{,/**} rw,
```

Tradeoff: the parser can write Incus's generated cache tree. This does not unconfine the Incus
controller or remove the guest's outer profile. The strengthened workload must prove namespace
creation and cleanup independently.

### KeePassXC selected database

```text
# /etc/apparmor.d/local/keepassxc
audit owner "@{HOME}/Plain/main.kdbx*" rwl,
```

Tradeoff: this covers one selected database basename and same-basename suffixes, not the containing
directory. It does not prove backup paths or an actual save. Do not replace it with broad
`/home/user/Plain` access.

### Helium and Git

Helium uses the real `net.imput.helium` configuration and cache variables. Its local OS-release rule
is `audit /etc/os-release r,` with no `owner` qualifier. The crashpad local include audits the
existing path and permissions without adding access.

Git's local rules are:

```text
# /etc/apparmor.d/local/git
owner @{HOME}/temp/agent{,/**} rwlk -> @{HOME}/temp/agent{,/**},
@{bin}/ssh-keygen px,
owner @{tmp}/.git_signing_buffer_tmp@{rand6}{,.sig} rw,
```

The signer has the same file boundary in its own profile:

```text
# /etc/apparmor.d/local/ssh-keygen
owner @{tmp}/.git_signing_buffer_tmp@{rand6}{,.sig} rw,
```

Tradeoff: Git can mutate and lock the private source-audit workspace, but receives no new execution
permission for hooks or build output. Agents and unprofiled development programs remain unconfined
outside this specific Git profile boundary.

## What does not work

- A universal `/tmp/*` Sanoid grant works around the immediate denial but discards the
  source-derived filename boundary.
- The ten-character-only Sanoid rule misses the readiness marker's PID suffix and leaves scheduled
  snapshots failing.
- Raising `PERL_CAPTURE_TINY_TIMEOUT` only changes the wait. It does not grant the readiness marker
  and is not a repair.
- Running extra Sanoid snapshot cycles solely for testing is unnecessary and changes real backup
  state. Use the isolated tee fixture and observe the configured timer.
- Broad `/home/user/temp` Helium access hides an incorrect disposable-profile location. Keep browser
  state in its permitted native configuration and cache scope.
- `owner /etc/os-release r` does not match root-owned OS metadata.
- `--no-sandbox`, `--disable-setuid-sandbox`, privileged containers, `apparmor=unconfined`, and
  manually supplied fake session runtime paths bypass the workload instead of repairing it.
- A global Go abstraction grant exposes every Go profile to scope quota files. The measured helpers
  need only profile-local reads.
- Broad KeePassXC access to all of `/home/user/Plain` is unnecessary for the observed single
  database.
- Disabling commit signing, granting all of `/tmp`, or using `pux` would hide the Git buffer and
  exec mismatches. The exact buffer and `.sig` names plus the existing enforcing `ssh-keygen`
  profile are sufficient; the separate literal-key temporary file is not needed for this configured
  key file. Leaving the signer unlisted produced a denial, not an unconfined fallback.
- Extending the updater rule to six random characters would accommodate a harness error, not the
  installed wrapper's five-character `mktemp` call.
- Unit status, profile names, empty denial queries, CDP readiness, and returned process handles are
  not substitutes for workload behavior, process labels, kernel maps, namespace state, or successful
  syscall evidence.

## Upstream filing artifact

### Upstream filing decision

Nothing to add upstream. These incidents are local policy mismatches for this host's configured
paths and the custom Helium profile.

- Upstream fault: not established. The applications expose the paths and behavior used here; the
  local profile did not cover them.
- Upstream fix capability: not required for the verified local configuration changes.
- Supported use case: the actual native services, wrapper, browser launcher, password database,
  source workspace, and unprivileged container were exercised locally.
- Contribution policy: no filing is proposed. Incus also prohibits direct LLM-filed issues, and
  these local caller and policy incidents are not upstream-filing artifacts.
- Likelihood of an upstream fix: not assessed because no upstream defect is claimed.
- Minimal compatible prototype: the installed local rules and their passing fixtures are the
  consumer-side configuration changes. No upstream patch is offered.

Do not file a report from this document. Reassess only if a reproducible defect remains after the
application and profile agree on the intended path and behavior.
