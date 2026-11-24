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
- The existing KeePassXC 2.8.0 process repeatedly failed to read `/home/user/Plain/main.kdbx`, and
  its file picker later could not enter `/home/user/Plain` itself.

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
Real database reads are proven. A later AVC named the directory `/home/user/Plain/` itself, so the
picker also needs directory-only read access. A save and rename at that path has not been exercised
against a disposable database, so this remains an explicit verification limit.

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

Auto-push then reached a different missing profile: managed Git declares `@{bin}/gh px`, but no
packaged `gh` target exists. The AVCs were `profile transition not found` for exec and a read denial
on `/usr/bin/gh`.

GitHub CLI 2.105.0's [keyring
wrapper](https://github.com/cli/cli/blob/e1219499a578b69e3437adaa31cd60614e23c802/internal/keyring/keyring.go#L36-L47)
uses the system keyring through its `Get` function:

```go
// internal/keyring/keyring.go:36-47
func Get(service, user string) (string, error) {
    ch := make(chan struct {
        val string
        err error
    }, 1)
    go func() {
        defer close(ch)
        val, err := keyring.Get(service, user)
        ch <- struct {
            val string
            err error
        }{val, err}
    }()
```

Its [Git
client](https://github.com/cli/cli/blob/e1219499a578b69e3437adaa31cd60614e23c802/git/client.go#L80-L93)
executes Git with the caller's standard streams:

```go
// git/client.go:80-93
cmd := exec.CommandContext(ctx, c.GitPath, args...)
cmd.Stderr = c.Stderr
cmd.Stdin = c.Stdin
cmd.Stdout = c.Stdout
return &Command{cmd}, nil
```

After keyring access and the real push reached the repository, Git denied this repository's own
`.git/hooks/pre-push`. The hook is a POSIX shell script that runs `git lfs pre-push "$@"`. The exact
own-hook inherit-exec rule is separate from third-party hook and build-output policy.

The follow-up auto-push also produced `/proc/154971/cgroup`, with `comm="git-lfs"` and `ouid=0`. Git
LFS 3.8.1 is built with Go 1.27.3-X:nodwarf5. The Go runtime source cited in
`Go retains the current cgroup quota descriptor` resolves the current cgroup from
`/proc/self/cgroup` before opening its CPU limit. That root-owned proc file requires no `owner`
qualifier. The local rule covers only this proc read; any subsequent cgroup-file denial requires its
own evidence.

After the proc repair, Git LFS produced an owner-matched `/proc/<pid>/mountinfo` denial. It also
requested execution of this repository's own cached
`.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`. The
active `git` command resolves through this exact owner-managed wrapper. Only that binary gets
inherit-exec permission; a rebuilt wrapper needs a new review, and third-party build output gets no
general grant.

### Loopback SSH forwarding and picker state

The SSH profile's stock `abstractions/network/out/ssh` grants outbound port 22 access, not local
bind. Reverse forwarding repeatedly failed with `operation="bind"` and loopback source addresses.
The upstream [loopback TCP
abstraction](https://github.com/roddhjav/apparmor.d/blob/5c34009a6ea104831c9a91a578f9255dbb289fe2/apparmor.d/abstractions/network/lo/tcp#L11-L12)
shows the appropriate address-scoped rule shape:

```text
# abstractions/network/lo/tcp:11-12
network (accept bind listen send receive) inet stream ip=127.0.0.1,
network (connect send receive) inet stream peer=(ip=127.0.0.1),
```

KeePassXC's file picker also exercised directory traversal, KIO bookmark and service-menu state,
anonymous sockets, its own proc mount table, thread names, and its XDG state directory. These are
separate from the selected database permission. The installed AppArmor manual states that a trailing
slash matches directories; recursive directory-only rules do not grant file contents.

GitHub CLI's own `px` transition to Git worked, but it then needed the same exact owner-managed
policy-wrapper exec path already granted to Git. Helium's existing KeePassXC integration needed only
its native-messaging JSON in `google-chrome/NativeMessagingHosts`.

## Verification

### Versions and evidence scope

The measured host runs AppArmor 4.1.8-2.1, Sanoid 2.3.0-2, Capture::Tiny 0.50-4, File::Temp from
Perl 5.44.0-1.1, Cachy-Update 4.4.7-1, pacman-contrib 1.13.1-1.2, Go 1.27.4-X:nodwarf5, Incus
7.6.0-1.1, KeePassXC 2.8.0-4.1, Git 2.56.1-1.1, Git LFS 3.8.1-1.1 with Go 1.27.3-X:nodwarf5, GitHub
CLI 2.105.0-1, and the custom confined Helium package.

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
- The first auto-push failed because `/usr/bin/gh` had no transition target. After a custom `gh`
  profile succeeded at `gh auth status`, the real push failed on this repository's own `pre-push`
  hook. The follow-up auto-push also recorded Git LFS's `/proc/154971/cgroup` denial, followed by
  `mountinfo` and exact policy-wrapper exec denials. These were new workflow stages, not repeats of
  the signing-buffer incident.

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
- KeePassXC remained enforcing and produced audited successful opens of its existing database. After
  the later directory denial, PID 4823 opened `/home/user/Plain/` with paired `SYSCALL success=yes`
  and a returned directory descriptor.
- Fresh Helium rendered Example Domain with renderer seccomp filtering and no unexpected unconfined
  helper. Audited events proved its OS-release read, existing crash-report reads, settings
  read/write opens, and lock-file operations. It did not create a new crash dump.
- After the picker/state and loopback rules, the credential helper, real Git push, and `man setpci`
  succeeded, and no new denials appeared after 2026-11-23T20:47:39Z. The selected database directory
  is traversable without making unrelated file contents readable by that rule alone.
- Signed commits `02f5e836951eaa990e1c6e3f88e5017b51e39d0b`,
  `778f7c5f9bbc5a0cacbdc7ac480bc31f4f360e3a`, `d189b6f6d0355cd41c377d1fdfc4b68aff043668`, and
  `2a6969039952edf5564c8f384f03ea9ad4aa7737` reached `main`. `gh auth status` read the existing
  keyring credential through the enforcing custom profile, and `git ls-remote` returned the same
  commit hashes. The push also exercised the exact own Git LFS hook.

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
audit owner "@{HOME}/Plain/" r,
audit owner "@{HOME}/Plain/main.kdbx*" rwl,
```

The picker also uses these measured state and process boundaries:

```text
# /etc/apparmor.d/local/keepassxc
owner @{HOME}/**/ r,
owner @{user_state_dirs}/keepassxc/ rw,
owner @{user_state_dirs}/keepassxc/* rwl,
owner @{user_share_dirs}/user-places.xbel* rwl,
owner @{user_config_dirs}/kservicemenurc{,.*} r,
owner @{run}/user/@{uid}/#@{int} rw,
@{PROC}/@{pid}/mountinfo r,
@{PROC}/@{pid}/mounts r,
@{PROC}/@{pid}/task/@{pids}/comm w,
@{run}/udev/data/b{8:0,259:1} r,
```

Tradeoff: recursive directory traversal exposes names and metadata throughout the home directory but
not file contents. The state, socket, proc, and observed udev metadata grants are separate and do
not make the selected database or other personal files generally readable. KeePassXC save behavior
at the selected database path remains unexercised.

### Loopback SSH forwarding

```text
# /etc/apparmor.d/local/ssh
network (bind listen accept send receive) tcp ip=127.0.0.1,
network (bind listen accept send receive) tcp ip=::1,
```

Tradeoff: this permits SSH listeners and accepted streams on any loopback TCP port, not arbitrary
non-loopback listeners. Outbound SSH remains governed by the stock outbound abstraction. This is
needed for dynamic reverse-forward ports; granting only one historical port would be too narrow.

### Exact wrapper, native messaging, and small service boundaries

`/etc/apparmor.d/local/gh` grants inherit-exec only to the exact current owner-managed
`monochromatic-git-policy-cli` path already measured for Git. `/etc/apparmor.d/local/helium` adds
read-only access to the existing KeePassXC native-messaging JSON.

Other measured local rules grant `basename` to `man//groff`, logind's exact `+module:uhid` read, the
portal's own `memory.pressure`, and the nested profile-sync-daemon Chromium mountpoints. DMI reads
and `iw`'s explicit `audit deny capability sys_module` remain unchanged. Portal pressure writes and
new profile-sync-daemon mount operations are not claimed exercised until their real services repeat
them.

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
owner @{HOME}/Monochromatic/.git/hooks/pre-push rix,
@{PROC}/@{pid}/cgroup r,
owner @{PROC}/@{pid}/mountinfo r,
owner @{HOME}/Monochromatic/.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli rix,
```

The signer has the same file boundary in its own profile:

```text
# /etc/apparmor.d/local/ssh-keygen
owner @{tmp}/.git_signing_buffer_tmp@{rand6}{,.sig} rw,
```

The custom `/etc/apparmor.d/gh` profile supplies the existing `px` target without an unconfined
helper:

```text
# /etc/apparmor.d/gh
abi <abi/4.0>,
include <tunables/global>
@{exec_path} = @{bin}/gh
@{att} = ""
profile gh /{,usr/}bin/gh {
  include <abstractions/base-strict>
  include <abstractions/consoles>
  include <abstractions/golang-strict>
  include <abstractions/nameservice-strict>
  include <abstractions/ssl_certs>
  include <abstractions/bus-session>
  network inet dgram,
  network inet6 dgram,
  network inet stream,
  network inet6 stream,
  network netlink raw,
  @{exec_path} mr,
  @{bin}/git px,
  /etc/gitconfig r,
  /etc/os-release r,
  owner @{HOME}/.gitconfig* r,
  owner @{user_config_dirs}/git/{,*} r,
  owner @{user_config_dirs}/gh/{,**} rw,
  include if exists <local/gh>
}
```

Tradeoff: `gh` receives its own config, keyring, network, TLS, and Git subprocess access. It has no
blanket home or temporary-directory grant. Git can mutate and lock the private source-audit
workspace, but only this repository's measured `pre-push` receives new inherit-exec permission.
Third-party hooks and build output receive none. Agents and unprofiled development programs remain
unconfined outside these specific profile boundaries.

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
- Read/write access to every file under `/home/user/Plain` is unnecessary for the observed database
  and directory picker. Directory-only read access is separate from the selected database-file
  grant.
- Blanket `/home/user/** rw` is unnecessary for a picker. `owner @{HOME}/**/ r` grants directory
  traversal only; file permissions still need their own measured rules.
- Disabling commit signing, granting all of `/tmp`, or using `pux` would hide the Git buffer and
  exec mismatches. The exact buffer and `.sig` names plus the existing enforcing `ssh-keygen`
  profile are sufficient; the separate literal-key temporary file is not needed for this configured
  key file. Leaving the signer unlisted produced a denial, not an unconfined fallback.
- Leaving `gh` unprofiled broke the managed `px` transition. Disabling the credential helper or
  repository hooks would avoid the workload rather than repair it. The custom enforcing `gh` profile
  and exact own-hook rule address the measured stages separately.
- Extending the updater rule to six random characters would accommodate a harness error, not the
  installed wrapper's five-character `mktemp` call.
- Unit status, profile names, empty denial queries, CDP readiness, and returned process handles are
  not substitutes for workload behavior, process labels, kernel maps, namespace state, or successful
  syscall evidence.

## The Git wrapper's own tests, gate and helpers under enforcement

Recorded 2026-11-23T17:10:00-05:00 by the session that works on the Git wrapper,
on the owner's word at its return:
"Add rules to unblock whatever you need to do as you go this session."
While the session weighed a profile of the tests' own,
the owner wrote:
"No,
please do rules for everything instead of a dedicated profile that agents would have to remember to use."
Not final:
the whole container gate on `main` that checks these rules is still running.

### Symptoms

- `git init` of the wrapper's test fixtures failed with "Permission denied",
  both in host test runs (fixtures under the shared cargo build folder `~/.cargo/build/<hash>/.../test-fixtures/`)
  and inside the container gate (fixtures under `/work/package/git-policy/cli/target/`).
  The AVCs were `operation="mkdir" profile="git" requested_mask="c"`,
  inside the container with `fsuid=100999`.
- `gh issue view` failed with the wrapper's "Could not find a real Git executable after examining 66 PATH candidates",
  every candidate,
  `/usr/bin/git` included,
  reading "missing,
  not an executable file,
  or unreadable".
- `gh issue create --body-file <draft under ~/temp/agent>` failed with "permission denied" on the draft.
- After the first rules,
  the fixtures' hooks and editor scripts were denied running `/usr/bin/sleep`,
  `git` was denied `ptrace read` of unconfined processes,
  `ssh-keygen` was denied the signing buffer under a Pi run's temporary folder,
  and Git LFS was denied its cgroup's `cpu.max` under the account's user manager.
- The audit log rotated all five of its 8 MiB files within minutes during a gate,
  and `auditctl -s` showed `lost 68735` with `backlog_limit 64`:
  denials were being dropped before they were written.

### Root cause

- AppArmor attaches a profile by the path of the program run,
  and a rootless Podman container's processes are unconfined,
  so the container's own `/usr/bin/git` attaches to the host's `git` profile.
  That profile grants the development paths the rollout measured,
  not the paths the wrapper's tests make their fixtures in.
- The custom `gh` profile let the policy wrapper inherit `gh`'s narrow confinement,
  which cannot read Git,
  the repository,
  the wrapper's own `/proc` files,
  its caller's environment or its user unit.
- The wrapper reads its caller's environment (`/proc/<pid>/environ`),
  which the kernel checks as a ptrace read of the caller.
- The Pi driver points `TMPDIR` into each run's folder,
  so Git writes its signing buffer there and not in `/tmp`.
- `fuse-overlayfs` is in complain mode,
  so every access its profile does not name is logged as `ALLOWED`:
  11,029 of 11,052 AppArmor records in the two newest audit files were its,
  all inside rootless Podman's layer storage.
  With a backlog of 64,
  bursts like that drop records.

### Rules added

All of them are in the `local/` includes,
loaded with `apparmor_parser --replace`,
which also wrote each profile's entry in the early-policy cache.
Backups of each local file as it was are under
`~/temp/agent/apparmor-rules-for-the-wrapper-work-2026-11-23/keep/`.

```text
# /etc/apparmor.d/local/git, first batch
owner @{HOME}/.cargo/build{,/**} rwlk -> @{HOME}/.cargo/build{,/**},
owner @{HOME}/.cargo/build/** mix,
owner /work{,/**} rwlk -> /work{,/**},
owner /work/** mix,
owner @{PROC}/@{pid}/stat r,
owner @{PROC}/@{pid}/fd/ r,
owner @{PROC}/@{pid}/environ r,
owner @{run}/user/@{uid}/systemd/user/monochromatic-git-policy-cli@.service r,
# /etc/apparmor.d/local/git, second batch
ptrace (read) peer=unconfined,
ptrace (read) peer=gh,
@{bin}/{sleep,cat,touch,mkdir,rm,rmdir,mv,cp,ls,echo,printf,true,false,test,head,tail,wc,sort,uniq,grep,sed,awk,date,kill,stat,sync,id,basename,dirname,readlink,realpath,tr,cut,tee,chmod,ln,mktemp,uname,od,diff,cmp,seq,expr,dd} rix,
@{sys}/fs/cgroup/user.slice/user-@{uid}.slice/user@@{uid}.service/**/cpu.max r,
# /etc/apparmor.d/local/git, third batch
owner @{PROC}/@{pid}/mounts r,
owner @{PROC}/@{pid}/io r,
@{PROC}/loadavg r,
@{sys}/fs/cgroup/{,**/}{cpu,memory,pids}.{max,current} r,
# /etc/apparmor.d/local/git, fourth batch
owner @{PROC}/@{pid}/task/ r,
/etc/machine-id r,
@{sys}/fs/cgroup/{,**/}memory.stat r,
@{bin}/env rix,
```

```text
# /etc/apparmor.d/local/ssh-keygen, fourth batch: signing tests make keys and signatures in their fixtures
owner @{HOME}/.cargo/build/** rw,
owner /work/** rw,
# /etc/apparmor.d/local/gh, fourth batch: its Go runtime's cpu.max
@{sys}/fs/cgroup/user.slice/user-@{uid}.slice/user@@{uid}.service/**/cpu.max r,
```

```text
# /etc/apparmor.d/local/systemd-run (new): a scope runs the account's own programs as the profile runs system ones
owner @{HOME}/** pux,
```

```text
# /etc/apparmor.d/local/gh: the wrapper's line changed from rix, and one rule added
owner @{HOME}/Monochromatic/.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli px -> git,
owner @{HOME}/temp/agent/** r,
```

```text
# /etc/apparmor.d/local/ssh-keygen
owner @{HOME}/temp/agent/**/.git_signing_buffer_tmp@{rand6}{,.sig} rw,
```

```text
# /etc/apparmor.d/local/fuse-overlayfs (new); the profile stays in complain mode
@{user_share_dirs}/containers/storage/overlay{,/**} rwlk,
```

The fifth batch came from the denials of the second whole gate on `main`,
each line with the count that gate logged:

```text
# /etc/apparmor.d/local/git, fifth batch: each thread's stat (38), the container's init (1), the cgroup's swap figures (11)
owner @{PROC}/@{pid}/task/@{tid}/stat r,
@{PROC}/1/stat r,
@{sys}/fs/cgroup/{,**/}memory.swap.{current,max} r,
# /etc/apparmor.d/local/ssh-keygen, fifth batch: Git's signature verification buffer (2)
owner @{tmp}/.git_vtag_tmp@{rand6} r,
owner @{HOME}/temp/agent/**/.git_vtag_tmp@{rand6} r,
# /etc/apparmor.d/local/sshd (new): the ssh terminal tests' own sshd reads its fixture under /work (3)
owner /work/** rwlk,
# /etc/apparmor.d/local/gh, fifth batch: its HTTP cache (55 creates, 55 reads)
owner @{user_cache_dirs}/gh/{,**} rwlk,
# /etc/apparmor.d/local/fusermount, fifth batch: Buildah unmounts its build context's overlay (2)
umount /var/tmp/buildah-context-*/overlay/*/merge/,
```

The sixth batch came from host tests,
and the seventh from the whole gate on `main` after the evening's merges (run `native-RxZa1T`):

```text
# /etc/apparmor.d/local/git, sixth batch: hooks of fixtures under the scratch folder (72 exec denials, host lock tests)
owner @{HOME}/temp/agent/**/hooks/* mix,
# /etc/apparmor.d/local/git_ssh (new), sixth batch: the ssh terminal tests' passphrase program (6 exec denials)
@{bin}/false rix,
# /etc/apparmor.d/local/ssh and local/git_ssh, seventh batch: the ssh terminal tests' key and known_hosts in their fixture
# (client_key read 10, client_key.pub read 4, known_hosts create 2, under ssh and under git//ssh alike)
owner /work/** rwk,
# /etc/apparmor.d/local/sshd-session (new), eighth batch: OpenSSH's per-connection sshd-session, its own profile, reads the
# client's authorized_keys in the ssh terminal fixtures (denied once for each of the two ssh_terminal controls)
owner /work/** r,
# /etc/apparmor.d/local/git, eighth batch: the wrapper, run as git, reads the cgroup root's controller list (3 denials)
@{sys}/fs/cgroup/cgroup.controllers r,
```

Seen in the same gate and not granted:
`capability dac_override` for `git` and `git-receive-pack` (nine denials in one second)
and for `umount` (three),
since no failing test needs either and the capability lets a program pass every file permission check;
and three reads by the browser `helium`,
which is not this work's.

The whole gate on `main` at `6f2b9ef708`,
after the eighth batch and the stale lock controls' fix
(run `native-JsS2Gw`,
2026-11-24T00:50:45Z to 01:09:53Z),
gave `10085 tests run: 10083 passed (7 slow), 2 failed, 37 skipped`.
The two are the cause-H pair,
which fail on a leftover scratch folder and failed on the Btrfs host too
(`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`, "Cause H"),
so no test of the gate fails under this enforcement.
The denials in that window were `umount`'s `dac_override` (three)
and six reads by `helium`.

`local/git_ssh` belongs to the `ssh` child of the `git` profile,
so it is loaded with `/etc/apparmor.d/git`.
Each batch after the fourth is appended,
and its profiles loaded,
by `append-a-batch-of-local-rules-and-load.ts` in
`~/temp/agent/apparmor-rules-for-the-wrapper-work-2026-11-23/`,
which keeps each local file as it was beside the batch in that folder's `keep/`.

On this machine a denial is in `/var/log/audit/audit.log` and nowhere else a session first looks:
on 2026-11-23,
for the 21 minutes of a whole gate,
`journalctl _TRANSPORT=audit` and `journalctl _TRANSPORT=kernel` held no `apparmor="DENIED"` line,
and `sudo ausearch --start 19:20:14 --end 19:41:19 --raw` returned 582 records with no AppArmor record among them,
while the log file itself held 20 denials in that window.
`list-denials-in-a-window.ts` in the same folder reads the file from standard input
(`sudo --non-interactive cat /var/log/audit/audit.log | node list-denials-in-a-window.ts <from> <to>`,
local times of today)
and prints each denial and a count per profile,
operation,
name and mask.

### Tradeoffs

- Git may create,
  lock,
  link and run files anywhere in the shared cargo build folder,
  which holds every build's output,
  not only test fixtures.
  `/work` does not exist on the host,
  and only root could make it.
- The listed tools run under Git's confinement,
  so they gain no file access Git does not have.
- Git,
  and the wrapper under it,
  may read the environment of any unconfined process of the account and of `gh`,
  which includes credentials those processes hold.
  The owner's threat model for this machine assumes it is not compromised.
- Under `gh`,
  the wrapper now runs under the `git` profile it runs under everywhere else,
  not under `gh`'s.
  `gh` may read every file the account owns under `~/temp/agent`.
- `fuse-overlayfs` may read and write any file in the layer storage,
  whoever owns it,
  which is its job;
  those accesses are no longer logged.
- `systemd-run --scope` may run any program the account owns under its home folder,
  under that program's own profile or unconfined,
  as the stock profile already lets it run any system program.
  Before the rule,
  a scope of mise's `node`,
  cargo's binaries or the repository's `node_modules/.bin` tools was refused.

### The audit backlog

The kernel's audit backlog limit was its default of 64:
no rule under `/etc/audit` set it,
and `audit-rules.service`,
which loads `/etc/audit/rules.d` with `augenrules --load`,
is disabled.
The owner chose "A:
8192,
kept (Recommended)" on 2026-11-23.
`auditctl -b 8192` set it for the running boot
(`backlog_limit 8192`,
with `lost 69301` as the count before),
and `/etc/audit/rules.d/10-backlog-limit.rules` holds `-b 8192`.
That file is read at boot only once `audit-rules.service` is enabled.
The owner enabled it on 2026-11-23
(`sudo systemctl enable audit-rules.service`,
which linked it into `multi-user.target.wants`),
after the coordinating session had left it to the owner,
and said:
"I didn't need to run it myself but I just ran it as a courtesy."
The rule against acting on the system service manager in the skill `pi-as-subagent` is for the agents a session briefs;
a change the owner has authorised the session to make,
as the rules of this page,
the session makes itself.

### Verification so far

- After the first batch:
  `git init` under the cargo build folder succeeded;
  `gh issue view 6347` printed its title through the wrapper;
  a container fixture under `/work` ran `git init`,
  a commit and its own `post-commit` hook,
  which wrote its file.
- After the `gh` read rule:
  `gh issue create --body-file` filed issue 6396.
- After the `fuse-overlayfs` rule:
  no `fuse-overlayfs` record was logged during the running gate,
  and the denials that followed could be read.
  The kernel still dropped 484 records in the next minutes,
  with the backlog limit unchanged at 64.
- After the second batch:
  no `ssh-keygen` denial was logged.
- After the `systemd-run` rule:
  `systemd-run --user --scope` ran the repository's `node_modules/.bin/mvm --help`,
  which printed its usage.
- The first whole gate on `main` (run `native-pl935G`) began before the second batch
  and was stopped after 47 minutes,
  once its one remaining test,
  `landing::service_work::push_retry_tests::a_stopped_push_is_still_tried_again_after_the_service_restarts`,
  had run for over 36 minutes;
  whole gates took about 29 minutes before enforcement.
  It had about 42 distinct failures,
  most of them before the batches that answer them were loaded.
  It served to find denials,
  not to judge the rules.
- Held,
  not granted:
  two `capability dac_override` denials for `umount` and one for `git`,
  apparently root inside a container.
  A grant to the `git` profile would also let root's `git` on the host pass file permissions,
  so it waits until a failing test is tied to it.
  The second gate logged four more for `umount`,
  and none for `git`.
- The second whole gate on `main`
  (unit `apparmor-second-gate-on-main-20261123`,
  at `a467a1d22c`)
  ran with all four batches loaded,
  in 21 minutes:
  `Summary [979.117s] 9914 tests run: 9904 passed (8 slow), 10 failed, 36 skipped`.
  Beside the cause-H pair of
  `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
  it failed
  `service_native::an_ssh_signed_commit_lands_through_the_service_and_verifies`,
  the three `ssh_terminal` controls (about 10 s each,
  the denied `sshd_config` read),
  three `stale_ref_lock` controls,
  and `worktree::worktree_clone_conversion_size_read::tests::a_process_that_ends_with_a_running_read_leaves_no_child_behind`.
  The fifth batch answers the `sshd` and verification-buffer denials.
  No denial,
  and no signal record at all,
  was logged for the `stale_ref_lock` controls:
  in `a_fetch_after_a_killed_fetch_keeps_the_locks_and_names_them`
  the wrapper's fetch after the killed one exited 0
  (`left: Some(0)` at `binary_stale_ref_lock_tests.rs:197`),
  so their cause is not shown.
  The lost count stayed at 69,301 through the gate,
  so with the backlog at 8192 the kernel dropped no record.
- The whole gate on `main` at `20c4bfa018`
  (run `native-RxZa1T`,
  after the evening's eleven merges and the sixth batch):
  `Summary [938.002s] 9957 tests run: 9949 passed (7 slow), 8 failed, 37 skipped`.
  The signed-commit control and the unknown-host-key `ssh_terminal` control pass since the fifth batch.
  The eight are the cause-H pair,
  the size read's child control,
  two `ssh_terminal` controls,
  whose denials the seventh batch answers,
  and the three `stale_ref_lock` controls,
  for which the gate's window logged no denial at all;
  the lost count stayed at 69,301.
  Those three are not a denial and not a rule:
  their own control kills the wrong process,
  measured on 2026-11-23 (section "The three stale lock controls killed the `timeout` and not the Git").
- Not yet shown:
  a whole container gate on `main` with no AppArmor denial.

### The three stale lock controls killed the `timeout` and not the Git

Recorded by the round that ran them alone,
at 2026-11-23T19:32:36-05:00 by the fix commit's time
(`fix/git-policy-cli-stale-ref-lock-controls-under-apparmor`, fix at `0767a62bbb`).
Not approved and not final:
no whole container gate has run with the fix.

`a_fetch_after_a_killed_fetch_keeps_the_locks_and_names_them`,
`a_fetch_after_a_killed_fetch_with_no_pid_file_keeps_the_lock_and_gives_the_command` and
`another_command_gets_no_notice_and_clears_nothing`
failed at `binary_stale_ref_lock_tests.rs:197`, `:217` and `:250`
because the lock they meant to leave was not there at all:
the wrapper's fetch after it exited 0 and printed nothing.

`hold_fetch` started its held fetch through `direct()`,
which puts `/usr/bin/timeout --signal=KILL <bound>` in front of the real Git
(`binary_support_native_git_command.rs`),
so `Held::kill` sent `SIGKILL` to that `timeout`
and the Git behind it kept running, still waiting on its `reference-transaction` hook.
Ending the hook then made that Git abort the prepared transaction
(`fatal: in 'prepared' phase, update aborted by the reference-transaction hook`)
and remove the reference lock, the packed-refs lock and the `packed-refs.new` staging file,
so the walk of `ref_lock_candidates` found no candidate,
the wrapper printed no notice, and the fetch it ran succeeded.

That abort is the difference this profile makes,
measured inside the gate's own image on 2026-11-23:
a Git under this profile runs it and removes all three files,
while the same binary at a path no profile names ends without it and leaves them.
No read was refused:
the decision reads the fixture's own files, which `local/git` grants under `~/.cargo/build` and `/work`,
and the gate's window logged no denial.

So the fix is in the control and not in a rule:
`hold_fetch` and `hold_transaction` start their Git with `direct_held()`,
the Git alone, which exists for a control that holds the child.
Red before and green after, on the host and in the gate image.
The unconfined Git's silent ending is not explained:
it is recorded here as measured, not as understood.

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
