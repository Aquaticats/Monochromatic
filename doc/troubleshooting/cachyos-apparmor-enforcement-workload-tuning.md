# AppArmor 4.1.8 default enforcement breaks local snapshot, updater, browser, password-store, development, and container paths

## Status

Current-boot repairs are measured.
The final enforcement cold boot and owner review remain open.
This is a local deployment record,
not an upstream bug report or a claim that every profile is
enforcing.

The rollout uses the broad `apparmor.d` collection at commit
`5c34009a6ea104831c9a91a578f9255dbb289fe2`,
with `default enforce`,
`include default`,
`reload yes`,
and `fsp no`.
Package complain and unconfined exceptions remain intentional.

## Symptom

Default enforcement exposed independent workload failures:

- Sanoid 2.3.0 exited with status 2 after snapshot creation.
  Its journal said
  `Timed out waiting for subprocesses to start at /usr/share/perl5/vendor_perl/Capture/Tiny.pm line 271.`
- `checkupdates` 1.13.1 could not create the `local` symlink selected by Cachy-Update 4.4.7.
  The
  parent user service could still exit 0 with an empty or misleading package comparison.
- `aa-install` and `aa-mode` were denied their transient scope's `cpu.max` file during Go runtime
  initialization.
- Incus 7.6.0 could launch no test instance because `apparmor_parser` was denied its generated
  cache's `.features` file.
- Git could not create source-audit directories beneath `/home/user/temp/agent`.
- Helium was denied `/etc/os-release` and initially used Chromium's configuration and cache
  directory names.
  Its crashpad child was denied the real `net.imput.helium/Crash Reports` paths.
- The existing KeePassXC 2.8.0 process repeatedly failed to read `/home/user/Plain/main.kdbx`,
  and
  its file picker later could not enter `/home/user/Plain` itself.

These are separate AVCs and workloads.
A successful parent unit,
a loaded profile name,
or an empty
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

Capture::Tiny 0.50 creates a readiness marker before waiting for its tee child.
The [installed
`lib/Capture/Tiny.pm:202-217`](https://fastapi.metacpan.org/v1/source/DAGOLDEN/Capture-Tiny-0.50/lib/Capture/Tiny.pm#:~:text=scalar(%20tmpnam()%20)%20.%20%24%24)
uses this expression:

```perl
# lib/Capture/Tiny.pm:217
$stash->{flag_files}{$which} = scalar( tmpnam() ) . $$;
```

The exact installed module was compared byte for byte with the CPAN 0.50 archive,
not with the newer
0.51 default branch.
Both copies had SHA-256
`229bdb082963bbb29629da2ef61bf151fd0f4705800e45672127ed11162581a8`.

The installed File::Temp 0.2312 defines ten temporary characters,
including underscore,
at
`File/Temp.pm:207-223`.
Its [`tmpnam`
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
parent PID `102311`,
with `fsuid=0` and `ouid=0`.
The first narrow rule covered only the
ten-character file used by `tempfile`,
not the readiness marker with its PID suffix.
Repeated
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

The installed `check.sh` matches tag `v4.4.7` byte for byte.
Installed `common.sh` differs only in
CachyOS tray-icon names;
the allocation lines match.

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

The packaged `golang-strict` abstraction covered `system.slice/*.service/cpu.max`,
but not the
observed transient `system.slice/*.scope/cpu.max`.
This is a descriptor-lifetime behavior,
so
command success or missing AVCs is weaker evidence than inspecting the real process's open
descriptor.

### Incus generates a separate parser profile and cache tree

Incus's
[`InstanceLoad`](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/apparmor/instance.go#L61-L77)
creates the namespace,
generates the instance profile,
then loads it:

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
invokes `apparmor_parser` with the generated cache and profile directories.
The measured AVC named
`/var/lib/incus/security/apparmor/cache/48da56f3.0/.features`,
not the system early-policy cache.

The host-init label includes the generated outer profile and the named namespace.
[`driver_lxc.go:1085-1090`](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/instance/drivers/driver_lxc.go#L1085-L1090)
appends `//&:<instance namespace>:` when stacking is available.
Consequently,
an inner guest
`unconfined` label is not sufficient evidence of a missing outer profile.

### KeePassXC's selected database is outside the stock password-store path

The stock KeePassXC profile grants `*.kdbx*` beneath `@{user_passwordstore_dirs}`.
This host leaves
that variable at `.password-store`,
while the selected existing database is
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
Real database reads are proven.
A later AVC named the directory `/home/user/Plain/` itself,
so the
picker also needs directory-only read access.
A save and rename at that path has not been exercised
against a disposable database,
so this remains an explicit verification limit.

### Helium and Git require deployment-specific boundaries

The custom Helium profile originally inherited Chromium's configuration and cache directory
variables.
That did not match this package's real `.config/net.imput.helium` and
`.cache/net.imput.helium` paths.
`/etc/os-release` is root-owned,
so an `owner` qualifier would not
match its `ouid=0` read.

Git's source-audit denials were confined to the private source-audit workspace.
A later signed
commit also failed on `/tmp/.git_signing_buffer_tmpM40r71`.

[Git 2.56.1's SSH signing
path](https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/gpg-interface.c#L1092-L1112)
creates that exact six-character buffer,
writes it,
and passes it to the signer:

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

At `gpg-interface.c:1128-1130`,
Git appends `.sig` and reads that signature file.
It later removes
both temporary paths.
This does not justify access to every personal directory,
arbitrary temporary
files,
or execution of repository hooks and build outputs.

Granting only the buffer exposed the next AVC:
`operation="exec" profile="git" name="/usr/bin/ssh-keygen" requested_mask="x"`.
An unlisted
executable inside this confined Git profile does not automatically receive an unconfined fallback.
The narrow repair transitions only `ssh-keygen` to its existing enforcing profile and grants the
same buffer and `.sig` paths there.

Auto-push then reached a different missing profile:
managed Git declares `@{bin}/gh px`,
but no
packaged `gh` target exists.
The AVCs were `profile transition not found` for exec and a read denial
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

After keyring access and the real push reached the repository,
Git denied this repository's own
`.git/hooks/pre-push`.
The hook is a POSIX shell script that runs `git lfs pre-push "$@"`.
The exact
own-hook inherit-exec rule is separate from third-party hook and build-output policy.

The follow-up auto-push also produced `/proc/154971/cgroup`,
with `comm="git-lfs"` and `ouid=0`.
Git
LFS 3.8.1 is built with Go 1.27.3-X:nodwarf5.
The Go runtime source cited in
`Go retains the current cgroup quota descriptor` resolves the current cgroup from
`/proc/self/cgroup` before opening its CPU limit.
That root-owned proc file requires no `owner`
qualifier.
The local rule covers only this proc read;
any subsequent cgroup-file denial requires its
own evidence.

After the proc repair,
Git LFS produced an owner-matched `/proc/<pid>/mountinfo` denial.
It also
requested execution of this repository's own cached
`.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`.
The
active `git` command resolves through this exact owner-managed wrapper.
Only that binary gets
inherit-exec permission;
a rebuilt wrapper needs a new review,
and third-party build output gets no
general grant.

### Loopback SSH forwarding and picker state

The SSH profile's stock `abstractions/network/out/ssh` grants outbound port 22 access,
not local
bind.
Reverse forwarding repeatedly failed with `operation="bind"` and loopback source addresses.
The upstream [loopback TCP
abstraction](https://github.com/roddhjav/apparmor.d/blob/5c34009a6ea104831c9a91a578f9255dbb289fe2/apparmor.d/abstractions/network/lo/tcp#L11-L12)
shows the appropriate address-scoped rule shape:

```text
# abstractions/network/lo/tcp:11-12
network (accept bind listen send receive) inet stream ip=127.0.0.1,
network (connect send receive) inet stream peer=(ip=127.0.0.1),
```

KeePassXC's file picker also exercised directory traversal,
KIO bookmark and service-menu state,
anonymous sockets,
its own proc mount table,
thread names,
and its XDG state directory.
These are
separate from the selected database permission.
The installed AppArmor manual states that a trailing
slash matches directories;
recursive directory-only rules do not grant file contents.

GitHub CLI's own `px` transition to Git worked,
but it then needed the same exact owner-managed
policy-wrapper exec path already granted to Git.
Helium's existing KeePassXC integration needed only
its native-messaging JSON in `google-chrome/NativeMessagingHosts`.

## Verification

### Versions and evidence scope

The measured host runs AppArmor 4.1.8-2.1,
Sanoid 2.3.0-2,
Capture::Tiny 0.50-4,
File::Temp from
Perl 5.44.0-1.1,
Cachy-Update 4.4.7-1,
pacman-contrib 1.13.1-1.2,
Go 1.27.4-X:nodwarf5,
Incus
7.6.0-1.1,
KeePassXC 2.8.0-4.1,
Git 2.56.1-1.1,
Git LFS 3.8.1-1.1 with Go 1.27.3-X:nodwarf5,
GitHub
CLI 2.105.0-1,
and the custom confined Helium package.

Evidence is under `/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/`.
Reports marked
`ownerAcceptance: false` are not final approval.

### Failing catalog

- An isolated `AppArmorProfile=sanoid` `tee_stderr` fixture timed out when only the ten-character
  File::Temp rule was installed.
  The AVC contained ten random characters followed by the parent PID.
- A ten-character-only Sanoid `tempfile` fixture was insufficient even though create,
  write,
  read,
  and delete succeeded for that different operation.
- The unchanged native `arch-update.service` denied `checkupdates` creation of
  `/run/user/1000/arch-update/checkupdates-1fHIo/local`.
- A harness-generated six-character `checkupdates-cMs5PD` directory remained denied after the
  five-character rule was installed.
  This was an incorrect fixture,
  not a reason to broaden the
  rule.
- An `ALLOWED`-only matcher missed explicitly audited successful KeePassXC operations.
  The kernel
  emitted `AUDIT` with paired `SYSCALL success=yes` instead.
- The first Incus namespace-assertion run completed its workload and cleanup,
  then failed the report
  writer's no-clobber guard.
  That was not a guest-confinement failure.
- A signed documentation commit initially failed with `fatal: failed to write commit object`.
  The
  corresponding AVC named `/tmp/.git_signing_buffer_tmpM40r71`;
  this was not a repository ownership
  problem.
  After the buffer rule,
  the next attempt failed with
  `fatal: cannot exec 'ssh-keygen': Permission denied`,
  with a separate exec AVC.
  The two denials
  required separate evidence.
- The first auto-push failed because `/usr/bin/gh` had no transition target.
  After a custom `gh`
  profile succeeded at `gh auth status`,
  the real push failed on this repository's own `pre-push`
  hook.
  The follow-up auto-push also recorded Git LFS's `/proc/154971/cgroup` denial,
  followed by
  `mountinfo` and exact policy-wrapper exec denials.
  These were new workflow stages,
  not repeats of
  the signing-buffer incident.

### Passing catalog

- The corrected Sanoid tee fixture returned exact captured stderr.
  The already-scheduled 14:00 EST
  `sanoid.service` exited 0,
  and direct `zfs list -t snapshot` queries confirmed all three reported
  snapshots.
- `verify-sanoid-narrow-tempfile.ts sanoid-narrow-tempfile-readback` proved nonempty readback and
  deletion while the same confined PID's `/proc/<pid>/attr/current` read remained denied.
- `verify-checkupdates-runtime-database.ts` used native five-character `mktemp` allocation,
  confirmed raw target `/var/lib/pacman//local`,
  canonical target `/var/lib/pacman/local`,
  nonempty
  synchronized databases,
  package comparisons,
  and fixture removal.
- `verify-go-cpu-quota-descriptor.ts` observed each real enforcing helper with the expected scope's
  `cpu.max` open.
  Descriptor contents matched direct quota reads.
- The strengthened Incus workload proved the exact outer enforcing profile,
  namespace presence
  during the workload,
  nonzero host UID/GID mappings,
  payload readback,
  and namespace and profile
  removal afterward.
- KeePassXC remained enforcing and produced audited successful opens of its existing database.
  After
  the later directory denial,
  PID 4823 opened `/home/user/Plain/` with paired `SYSCALL success=yes`
  and a returned directory descriptor.
- Fresh Helium rendered Example Domain with renderer seccomp filtering and no unexpected unconfined
  helper.
  Audited events proved its OS-release read,
  existing crash-report reads,
  settings
  read/write opens,
  and lock-file operations.
  It did not create a new crash dump.
- After the picker/state and loopback rules,
  the credential helper,
  real Git push,
  and `man setpci`
  succeeded,
  and no new denials appeared after 2026-11-23T20:47:39Z.
  The selected database directory
  is traversable without making unrelated file contents readable by that rule alone.
- Signed commits `02f5e836951eaa990e1c6e3f88e5017b51e39d0b`,
  `778f7c5f9bbc5a0cacbdc7ac480bc31f4f360e3a`,
  `d189b6f6d0355cd41c377d1fdfc4b68aff043668`,
  and
  `2a6969039952edf5564c8f384f03ea9ad4aa7737` reached `main`.
  `gh auth status` read the existing
  keyring credential through the enforcing custom profile,
  and `git ls-remote` returned the same
  commit hashes.
  The push also exercised the exact own Git LFS hook.

## Verified workarounds

Install only the measured local rules and reload the affected profile.
Rebuild the complete
early-policy cache after all local edits.

### systemd-oomd reading the cgroup tree

During the whole gate of 2026-11-24T01:57:53Z to 02:51:53Z,
with six guests and Rust builds beside it,
the audit log held 224 denials for each of eleven cgroup folders,
all `systemd-oomd` opening a folder under `/sys/fs/cgroup/` for read
(`list-denials-in-a-window.ts` over that window).
Its journal repeated "Considered 0 cgroups for killing,
top candidates were:",
and `oomctl` listed only `/system.slice` and `.../app.slice` with "Swap Usage:
0B"
while 47.5 GiB of swap was in use and 1,964 MiB of memory was available.
So under enforcement the daemon meant to end a runaway cgroup before memory runs out could choose none.
On 2026-11-22,
before enforcement,
it read the cgroups
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"Nothing shed load").

```text
# /etc/apparmor.d/local/systemd-oomd
  /sys/fs/cgroup/ r,
  /sys/fs/cgroup/** r,
```

The owner answered "Add the read rule now (Recommended)" on 2026-11-24
(`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`);
the lines were appended with a comment naming that answer,
and `sudo apparmor_parser --replace /etc/apparmor.d/systemd-oomd` reloaded the profile and wrote its early-policy cache.
Afterwards `oomctl` listed the account's app cgroups one by one.
Tradeoff:
the daemon may read every cgroup file,
which is what it is for;
it gains no write.

### Sanoid temporary files

```text
# /etc/apparmor.d/local/sanoid
owner /tmp/[0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z] rw,
owner /tmp/[0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z][0-9A-Z_a-z]@{pid} rw,
```

Tradeoff:
these owner-matched flat paths cover the installed File::Temp character set and
Capture::Tiny PID suffix.
They do not cover other temporary directories or arbitrary filenames.

### Cachy-Update database location

```text
# /etc/apparmor.d/local/checkupdates
owner @{run}/user/@{uid}/arch-update/checkupdates-@{rand4}@{c}/ w,
audit owner @{run}/user/@{uid}/arch-update/checkupdates-@{rand4}@{c}/local w,
```

Tradeoff:
the five-character shape is specific to the installed wrapper.
Pacman remains complain,
so
this is not proof of enforcing pacman confinement.
Remove the audit qualifier after final evidence
collection if routine audit volume is unwanted.

### Go quota descriptors

```text
# /etc/apparmor.d/local/aa-install and /etc/apparmor.d/local/aa-mode
owner @{sys}/fs/cgroup/system.slice/*.scope/cpu.max r,
```

Tradeoff:
each helper can read any flat scope's quota beneath `system.slice`,
not only its current
scope.
The rule does not cover nested scopes or other cgroup files.
Keep it profile-local instead of
broadening every Go application through the shared abstraction.

### Incus parser paths

```text
# /etc/apparmor.d/local/apparmor_parser
owner /var/lib/incus/security/apparmor/profiles{,/**} r,
owner /var/lib/incus/security/apparmor/cache{,/**} rw,
```

Tradeoff:
the parser can write Incus's generated cache tree.
This does not unconfine the Incus
controller or remove the guest's outer profile.
The strengthened workload must prove namespace
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

Tradeoff:
recursive directory traversal exposes names and metadata throughout the home directory but
not file contents.
The state,
socket,
proc,
and observed udev metadata grants are separate and do
not make the selected database or other personal files generally readable.
KeePassXC save behavior
at the selected database path remains unexercised.

### Loopback SSH forwarding

```text
# /etc/apparmor.d/local/ssh
network (bind listen accept send receive) tcp ip=127.0.0.1,
network (bind listen accept send receive) tcp ip=::1,
```

Tradeoff:
this permits SSH listeners and accepted streams on any loopback TCP port,
not arbitrary
non-loopback listeners.
Outbound SSH remains governed by the stock outbound abstraction.
This is
needed for dynamic reverse-forward ports;
granting only one historical port would be too narrow.

### Exact wrapper, native messaging, and small service boundaries

`/etc/apparmor.d/local/gh` grants inherit-exec only to the exact current owner-managed
`monochromatic-git-policy-cli` path already measured for Git.
`/etc/apparmor.d/local/helium` adds
read-only access to the existing KeePassXC native-messaging JSON.

Other measured local rules grant `basename` to `man//groff`,
logind's exact `+module:uhid` read,
the
portal's own `memory.pressure`,
and the nested profile-sync-daemon Chromium mountpoints.
DMI reads
and `iw`'s explicit `audit deny capability sys_module` remain unchanged.
Portal pressure writes and
new profile-sync-daemon mount operations are not claimed exercised until their real services repeat
them.

### The wrapper link a toolchain install repoints

The rules name one exact wrapper build
(`.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`),
and `.cache/monochromatic-git-policy-cli-bin/git` links to it.
A `mise run toolchain:install` in the main checkout rewrites that link to the locked launcher
`.cache/toolchain-launchers/monochromatic-git-policy-cli`,
which the `gh` profile's inherit-exec rule does not name,
so every `gh` call then fails with
`fork/exec .../monochromatic-git-policy-cli-bin/git: permission denied`
while `git` itself keeps working.
The fix is to point the link back at the build the rules name,
which the owner asked for on 2026-11-25:
"Repoint the git link too".
Any later toolchain install in the main checkout undoes that the same way,
and it takes a fresh review of the new binary's path to point the rules at a newer build at all
(`doc/handover/full-system-policy-handover.md`,
"a rebuilt wrapper requires a fresh review").
So the session's `git` is the build of `ee0cf2268b` from 2026-11-22,
whose removals still keep the commit service's journals,
a defect fixed after that build;
the fixed build's `service stop-units-of-removed-worktrees` clears them after each batch.
A toolchain install outside the main checkout repoints only that worktree's own link.

### Helium and Git

Helium uses the real `net.imput.helium` configuration and cache variables.
Its local OS-release rule
is `audit /etc/os-release r,` with no `owner` qualifier.
The crashpad local include audits the
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

Tradeoff:
`gh` receives its own config,
keyring,
network,
TLS,
and Git subprocess access.
It has no
blanket home or temporary-directory grant.
Git can mutate and lock the private source-audit
workspace,
but only this repository's measured `pre-push` receives new inherit-exec permission.
Third-party hooks and build output receive none.
Agents and unprofiled development programs remain
unconfined outside these specific profile boundaries.

## What does not work

- A universal `/tmp/*` Sanoid grant works around the immediate denial but discards the
  source-derived filename boundary.
- The ten-character-only Sanoid rule misses the readiness marker's PID suffix and leaves scheduled
  snapshots failing.
- Raising `PERL_CAPTURE_TINY_TIMEOUT` only changes the wait.
  It does not grant the readiness marker
  and is not a repair.
- Running extra Sanoid snapshot cycles solely for testing is unnecessary and changes real backup
  state.
  Use the isolated tee fixture and observe the configured timer.
- Broad `/home/user/temp` Helium access hides an incorrect disposable-profile location.
  Keep browser
  state in its permitted native configuration and cache scope.
- `owner /etc/os-release r` does not match root-owned OS metadata.
- `--no-sandbox`,
  `--disable-setuid-sandbox`,
  privileged containers,
  `apparmor=unconfined`,
  and
  manually supplied fake session runtime paths bypass the workload instead of repairing it.
- A global Go abstraction grant exposes every Go profile to scope quota files.
  The measured helpers
  need only profile-local reads.
- Read/write access to every file under `/home/user/Plain` is unnecessary for the observed database
  and directory picker.
  Directory-only read access is separate from the selected database-file
  grant.
- Blanket `/home/user/** rw` is unnecessary for a picker.
  `owner @{HOME}/**/ r` grants directory
  traversal only;
  file permissions still need their own measured rules.
- Disabling commit signing,
  granting all of `/tmp`,
  or using `pux` would hide the Git buffer and
  exec mismatches.
  The exact buffer and `.sig` names plus the existing enforcing `ssh-keygen`
  profile are sufficient;
  the separate literal-key temporary file is not needed for this configured
  key file.
  Leaving the signer unlisted produced a denial,
  not an unconfined fallback.
- Leaving `gh` unprofiled broke the managed `px` transition.
  Disabling the credential helper or
  repository hooks would avoid the workload rather than repair it.
  The custom enforcing `gh` profile
  and exact own-hook rule address the measured stages separately.
- Extending the updater rule to six random characters would accommodate a harness error,
  not the
  installed wrapper's five-character `mktemp` call.
- Unit status,
  profile names,
  empty denial queries,
  CDP readiness,
  and returned process handles are
  not substitutes for workload behavior,
  process labels,
  kernel maps,
  namespace state,
  or successful
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
(`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
"Cause H"),
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

Read a window across every rotated file,
once the log rotates:
`audit.log.2`,
`audit.log.1` and `audit.log` together,
because the five 8 MiB files rotate within minutes during a gate and a denial that started a batch
can sit in an earlier file than the one read.
In zsh pass the window's two times as two words,
as `for from to in <from> <to>`,
because an unquoted `$times` reaches the program as one argument and window matching then answers nothing.

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
The ninth whole gate's window is read record by record in
"Ninth batch:
  the gate at 8519c599d0".

### The three stale lock controls killed the `timeout` and not the Git

Recorded by the round that ran them alone,
at 2026-11-23T19:32:36-05:00 by the fix commit's time
(`fix/git-policy-cli-stale-ref-lock-controls-under-apparmor`,
fix at `0767a62bbb`).
Not approved and not final:
no whole container gate has run with the fix.

`a_fetch_after_a_killed_fetch_keeps_the_locks_and_names_them`,
`a_fetch_after_a_killed_fetch_with_no_pid_file_keeps_the_lock_and_gives_the_command` and
`another_command_gets_no_notice_and_clears_nothing`
failed at `binary_stale_ref_lock_tests.rs:197`,
`:217` and `:250`
because the lock they meant to leave was not there at all:
the wrapper's fetch after it exited 0 and printed nothing.

`hold_fetch` started its held fetch through `direct()`,
which puts `/usr/bin/timeout --signal=KILL <bound>` in front of the real Git
(`binary_support_native_git_command.rs`),
so `Held::kill` sent `SIGKILL` to that `timeout`
and the Git behind it kept running,
still waiting on its `reference-transaction` hook.
Ending the hook then made that Git abort the prepared transaction
(`fatal: in 'prepared' phase, update aborted by the reference-transaction hook`)
and remove the reference lock,
the packed-refs lock and the `packed-refs.new` staging file,
so the walk of `ref_lock_candidates` found no candidate,
the wrapper printed no notice,
and the fetch it ran succeeded.

That abort is the difference this profile makes,
measured inside the gate's own image on 2026-11-23:
a Git under this profile runs it and removes all three files,
while the same binary at a path no profile names ends without it and leaves them.
No read was refused:
the decision reads the fixture's own files,
which `local/git` grants under `~/.cargo/build` and `/work`,
and the gate's window logged no denial.

So the fix is in the control and not in a rule:
`hold_fetch` and `hold_transaction` start their Git with `direct_held()`,
the Git alone,
which exists for a control that holds the child.
Red before and green after,
on the host and in the gate image.
The unconfined Git's silent ending is not explained:
it is recorded here as measured,
not as understood.

## Ninth batch: the gate at 8519c599d0

Recorded 2026-11-24T02:50:00-05:00 by the round `apparmor-ninth-batch-for-the-gate-at-8519c599d0-2026-11-24`,
on branch `docs/troubleshooting-apparmor-ninth-batch-for-the-gate-at-8519c599d0`.
Not applied,
not approved and not final:
this section names what made each denial of the window and proposes the smallest rule for it,
or says why none belongs,
and only the coordinating session loads the rules,
because loading one needs the root this round never takes.
Every measurement,
script and log named here is under that round's `keep/`.

### What the window held

The whole container gate on `main` at `8519c599d0`,
run `native-bNr3O6`,
reported `10745 tests run: 10741 passed (7 slow), 4 failed, 41 skipped` in 988.793 s
(`gate-gate-on-main-after-the-merges-of-2026-11-24-night-8519c599d0.txt` in
`~/temp/agent/apparmor-rules-for-the-wrapper-work-2026-11-23/keep/`).
Its four failures are the cause-H pair,
the source guard step 10's two `std::thread::sleep` calls trip
(`add_mount.rs:70`,
`add_route_plan.rs:265`),
and the `statmount` control that answers `ENOSYS` in the container.
No AppArmor refusal is among them.

`podman events` still holds the run's containers,
which is what dates every record below
(`keep/podman-events-in-the-window.txt`).
The suite and the Clippy container were both created and started at 01:30:19,
the Clippy container died and was removed at 01:34:25,
and the suite's death is the one event the log does not hold.
The suite's own 988.793 s therefore ends it at 01:46:48,
and the run's first log line places the image build at 01:18:44 to 01:30:19.
A denial is the gate's only when it falls between 01:30:19 and 01:46:48
and no other container of those minutes made it.

The other containers of the window are the reason most of the list is not the gate's.
The askpass design round
(`~/temp/agent/typed-network-command-next-step-at-a-new-host-key-design-2026-11-24`)
saw throwaway containers at 01:20:00,
01:20:59 and 01:26:36,
and then `cli-git-askpass-probe-2026-11-24` at 01:26:50,
which it removed at 01:47:15.
Its `keep/state-so-far.md` and `keep/probe-transcripts.txt` record,
at their own times,
the host key,
the three agent starts and the askpass refusals named below.
It ran as the container's root
(its client keys lie in `/root/.ssh` and its sshd served `root@127.0.0.1`),
which is why a capability denial in it can bite,
where the gate's own `"user": "1000:1000"`
(`manifest.json` of run `native-bNr3O6`) has no capability at all.

### A capability record names no file

A `capable` record holds no path and no owner,
so the log cannot say which file a capability was asked for,
nor which container the caller was in.
The kernel's AppArmor capability record adds `capname=` alone
([`security/apparmor/capability.c:47-53`](https://github.com/torvalds/linux/blob/e5f0a698b34ed76002dc5cff3804a61c80233a7a/security/apparmor/capability.c#L47-L53)),
and the common fields are the type,
operation,
class,
profile,
pid,
command and error
([`security/apparmor/audit.c:97-141`](https://github.com/torvalds/linux/blob/e5f0a698b34ed76002dc5cff3804a61a/security/apparmor/audit.c#L97-L141)):

```c
// security/apparmor/capability.c:47-53, the whole callback a capability record adds
static void audit_cb(struct audit_buffer *ab, void *va)
{
	struct common_audit_data *sa = va;

	audit_log_format(ab, " capname=");
	audit_log_untrustedstring(ab, capability_names[sa->u.cap]);
}
```

`list-denials-in-a-window.ts` prints an empty name for them,
and nothing was lost when the window was kept:
there is no `fsuid` in such a record to set a container's process beside a host one.
The two capability families at the end of this section are therefore attributed from the other
evidence named with them,
never from the log alone.

### The `ssh-keygen` record: a host key on a bind mount

`01:20:01 ssh-keygen mknod /probe/hostkey c`.
The design round's first container ran at 01:20:00 and died at 01:20:01,
and its `keep/state-so-far.md` names the command and the time:
`ssh-keygen -f /probe/hostkey` on a bind mount,
at 01:19 to 01:20.
No file of this repository names `/probe/hostkey`,
and the gate's own signing controls write under the fixture root and `/tmp`
(`local/ssh-keygen`),
so no test of the gate made this record.
No rule:
it is a probe's path,
not a workload's.

### The six `ssh-agent` records: three starts in one container

`01:32:06`,
`01:32:11` and `01:32:16` hold one `capable setgid` each,
plus `mknod /work/out/agent.sock`,
`mknod /root/agent.sock` and `open / r`.
The state file names two of the three starts and their exact sentences:
`ssh-agent -a /root/agent.sock` failed with
`unix_listener: cannot bind to path /root/agent.sock: Permission denied`,
and `ssh-agent -s` failed with `mkdir_path: open("/"): Permission denied`,
which is the `open` record of the list.
The third start made its socket under the container's own `/work/out`,
which is the round's scratch folder mounted at `/work`.
No test of the gate starts an `ssh-agent`:
`package/git-policy/cli` names it only inside diagnostic sentences
(`lfs_pre_push.rs:678-699`,
`lfs_upload_tests.rs:829-858`),
and `local/ssh-agent` is empty today.
No rule:
the gate runs no agent,
and a rule for one would grant a program no test starts.

### The five askpass programs: two paths of one probe

`01:29:19`,
`01:29:24` and `01:31:06` hold `git//ssh exec /work/stubs/askpass`,
and `01:31:11` holds `git//ssh exec /work/bin/askpass` twice.
`keep/probe-transcripts.txt` of the design round prints both refusals word for word,
under its arms named `askpass`:
`ssh_askpass: exec(/work/stubs/askpass): Permission denied` and
`ssh_askpass: exec(/work/bin/askpass): Permission denied`,
and its state file records the second at 06:31Z with the command that made it.
Neither path exists in this repository:
`git ls-tree --name-only 8519c599d0` has no `stubs` directory,
and the wrapper never names an askpass path of its own,
because the program it gives OpenSSH is the bare name `false`
(`package/git-policy/cli/src/native/invocation/child_environment.rs:497`)
which `ssh_askpass` finds on `PATH` with `execlp`
([`readpass.c:77`](https://github.com/openssh/openssh-portable/blob/V_10_6_P2/readpass.c#L55-L80)).
The gate's own ssh child is `git//ssh`,
and `local/git_ssh` grants it `@{bin}/false rix`,
which is why no exec denial of `false` appears in this window at all.
No rule:
a rule for `/work/stubs/askpass` or `/work/bin/askpass` would grant a probe's program.

### The four programs plain `ssh` ran

`01:30:39 ssh exec /usr/bin/false`,
then `/usr/bin/true`,
`/usr/bin/cat` and `/usr/bin/echo`,
one record each in one second.
The design round's state file names exactly these four programs and the reason:
a person's own `ssh` under `/etc/apparmor.d/ssh` may run none of them,
measured the same way at 06:31Z,
and the round swept them to see which askpass program a plain `ssh` could still start.
One `podman exec` of that container also ended in the same second,
01:30:39
(`keep/podman-events-in-the-window.txt`).
The gate cannot produce these under profile `ssh`:
its plain `ssh` arms run with `env_clear()` and carry no `SSH_ASKPASS` at all
(`package/git-policy/cli/src/native/binary/binary_support.rs:141-161`),
and every `ssh` the wrapper's Git starts inherits the profile `git//ssh`,
where `false` is already granted.
No rule:
the gate runs no plain `ssh` with an askpass program,
and `local/ssh` stays as it is.

### The two `systemd-analyze` folders: a timer install, not a test

`01:22:44` holds two `mkdir` records,
`/tmp/systemd-analyze-XR4rRL/` and `/tmp/systemd-analyze-aGss0w/`,
both before the gate's containers existed.
The only place this repository spawns the tool is the daily-update timer install,
`systemd-analyze --user verify <service> <timer>`
(`package/monorepo-task/core-exec/src/daily-update-timer-install-logic.ts:292-309`),
and nothing under `package/git-policy` starts it,
so no test of the gate made these.
The denial is not harmless:
run by hand on 2026-11-24,
the same command prints `Failed to setup working directory: Permission denied` and exits 1
(`keep/systemd-analyze-verify-under-enforcement.log`).
systemd 262 makes that folder as `mkdtemp_malloc("/tmp/systemd-analyze-XXXXXX", &tempdir)`
([`src/analyze/analyze-verify.c:62-69`](https://github.com/systemd/systemd/blob/8cc40e0c5e9234bf45084751ac53b1fbfe70b492/src/analyze/analyze-verify.c#L62-L69)),
and the stock profile covers only a different name:
`owner @{tmp}/systemd-temporary-*/ rw,`
(`/etc/apparmor.d/systemd-analyze:92`).
The two lines the rule answers are:

```c
// src/analyze/analyze-verify.c:62-69
int verb_verify(int argc, char *argv[], uintptr_t _data, void *userdata) {
        _cleanup_strv_free_ char **filenames = NULL;
        _cleanup_(rm_rf_physical_and_freep) char *tempdir = NULL;
        int r;

        r = mkdtemp_malloc("/tmp/systemd-analyze-XXXXXX", &tempdir);
        if (r < 0)
                return log_error_errno(r, "Failed to setup working directory: %m");
```

The rule for it is proposed below.

### The 26 `git` and the 2 `umount` `dac_override` records

Two capability families are left,
and neither can be tied to a file by the log.

The `git` family holds 26 records in seven groups between 01:36:46 and 01:39:20,
two of them from `git-receive-pack`.
No file record accompanies any of them,
so the profile allowed every path and only the file mode refused it.
The gate's own tests make exactly that shape on purpose:
`binary_commit_tests.rs:1587`
(`a_staging_failure_that_is_not_a_pathspec_matching_nothing_stays_an_engine_failure`)
sets a tracked file to mode 000 and asserts Git's own `Permission denied`,
and the same access asks the capability once and only once.
Measured in the gate's base image as the gate's user,
under `strace`:
`openat(AT_FDCWD, "dir/x.txt", O_RDONLY) = -1 EACCES (Permission denied)`,
one record
(`keep/repro-one-denial-per-refused-open.log`).
A mode-000 file is one such access;
a directory with no search bit and a file without `x` are others,
because the kernel consults the same capability at every refused `inode_permission`
(`generic_permission` in `fs/namei.c` reaches `capable_wrt_inode_uidgid` once the mode bits
refuse),
and `package/git-policy/cli/src` holds a dozen such fixtures.
The design round's root container is the other possible maker,
since its Git holds the capability and a refusal there would be real,
but its transcripts report no refused filesystem path at all,
while these five askpass refusals are the only filesystem errors they hold.

The two `umount` records at 01:46:51 are three seconds from the suite's end,
which is when the gate's mount-namespace control unmounts its tmpfs
(`landing/unreachable_git_dir_proven_gone_tests.rs:400-429`),
and that control runs `umount` inside a user namespace of its own,
where the capability is held,
so a refusal there would be real.
Whether these two are that control's or another session's cannot be told:
a capability record names no file,
and the `umount` profile allows its own operation anywhere (`/etc/apparmor.d/umount:27`)
and no `/work` file is refused in this window at all.
Run as the gate runs it,
the sequence completes anyway:
`mount-exit=0` and `umount-exit=0` inside a user namespace of its own under uid 1000
(`keep/repro-nested-userns-mount-and-umount.log`),
and the control's `Drop` ignores the status in any case.

No rule for either family,
and the reason is measured rather than argued:

- The gate's containers run as `1000:1000`,
  and that user's effective capability set is
  empty:
  `CapEff: 0000000000000000`
  (`keep/repro-capabilities-of-the-gates-user.log`).
  The kernel's own capability check runs before AppArmor's
  (the `capability` LSM is `LSM_ORDER_FIRST` in `security/commoncap.c`,
  and this host's `/sys/kernel/security/lsm` reads
  `capability,landlock,lockdown,yama,apparmor,bpf`),
  so a caller that holds no capability never reaches AppArmor's check:
  every `capable` denial inside the gate is a log line and nothing else.
- A grant is not free outside the gate.
  `local/git` confines this host's Git as well as the gate's,
  and the `umount` profile is the host's own,
  so a `capability dac_override` line in either would let a root process of that program on
  this machine pass every file permission check,
  for a workload no test needs.
  That is why the first batch held the line and why it stays held.

### The `git` `dac_override` records are the any-name fixture's Git (issue 6459)

Recorded 2026-11-25T07:55:39-05:00 by the round
`any-name-fixture-root-git-without-capabilities`,
on branch `fix/git-policy-cli-any-name-fixture-root-git-without-capabilities`.
Not approved and not final.
This corrects the attribution of the two capability families as far as the `git` one goes,
and the ordering the section `## Ninth batch: the gate at 8519c599d0` states for the two capability checks.

The `SYSCALL` record beside each of these denials names the call:
`syscall=86 success=yes` with `/usr/bin/git commit --quiet --allow-empty --message=initial`
in a fixture repository under the run's fixture root,
and its `PATH` record carries `mode=0100444` and `nametype=CREATE`.
System call 86 is `link`,
and Git makes it for a loose object it has just written in its object directory
(`finalize_object_file` links the temporary file into place unless
`core.createObject` is `rename`,
and the fixture repositories set no such value).

This host sets `fs.protected_hardlinks = 1`,
so `may_linkat` asks
`safe_hardlink_source` (`fs/namei.c`),
which calls `inode_permission(idmap, inode, MAY_READ | MAY_WRITE)`.
The file's mode is `0444`,
so the owner's write bit refuses,
and `generic_permission` asks `capable_wrt_inode_uidgid(..., CAP_DAC_OVERRIDE)`.
That reaches `security_capable`,
which walks the capability hooks and returns at the first refusal.
The `capability` LSM is `LSM_ORDER_FIRST` (`security/commoncap.c`),
and this host's `/sys/kernel/security/lsm` reads
`capability,landlock,lockdown,yama,apparmor,bpf`,
so an ordinary process's ask is answered by `cap_capable` and AppArmor is never consulted for it.
That is why no record names an ordinary test's hardlink.

The namespace's root is answered differently,
and that is what makes these records.
`handle_privileged_root` in the same file gives the root of a user namespace
the whole bounding set in permitted and effective at every `exec`,
so a Git started inside the any-name control's inner half holds `CAP_DAC_OVERRIDE`
for an inode whose owner that namespace maps:
`cap_capable` answers 0,
AppArmor's `git` profile is asked next,
refuses,
and logs,
and `may_linkat` then succeeds on its owner check (`inode_owner_or_capable`),
which is why the record says `success=yes`.
Measured inside that half on this host,
an exec'd child starts with `CapPrm: 000001ffffffffff` and `CapEff: 000001ffffffffff`,
and the control's `git` `link()` of each loose object returns 0
(`keep/inner-half-before.trace`).

So the `git` family is the any-name fixture's own Git and no workload outside it,
and the fix belongs to the fixture rather than to a profile.
Once the inner half has mounted its tmpfs it sets `SECURE_NOROOT`
(`src/native/testing/fixture_root_taking_any_name.rs`),
so every program it starts is given no capability of the namespace,
while the inner half keeps its own for the root-owned fixtures it goes on to make.
`cap_capable` then answers the hardlink's ask as it does for an ordinary account,
no profile is consulted,
no record is made,
and the `link()` still returns 0
(`keep/inner-half-after.trace`,
`prctl(PR_SET_SECUREBITS, SECBIT_NOROOT) = 0`).
A run whose fixture root already takes a name that is not UTF-8 starts no namespace,
and the same control runs there as it always did.

The control that holds the fix is
`testing::fixture_root_taking_any_name_of_unit_tests::tests::a_program_started_from_the_any_name_root_holds_no_capability`:
it reads a child's own `/proc/self/status` and requires an empty `CapInh`,
`CapPrm`,
`CapEff` and `CapAmb`.
It was red before the bit,
naming `CapPrm=0x000001ffffffffff`
(`keep/control-red-before-the-fix.log`),
and green after it (`keep/control-green-after-the-fix.log`),
and the guarded controls of `commit::capture_invocation` passed after it
(`keep/module-capture-invocation-after-the-fix.log`).
No rule is added for this family and none is needed:
the ask was the fixture's,
and the fixture now runs as an ordinary account does.

### The tests that would pass with their program refused

No test of the gate passed **because of** a denial in this window:
the only records the gate can have made are capability lines that change nothing for
uid 1000.
Two controls are blind to a refusal all the same,
and the sixth batch's six `exec /usr/bin/false` denials under `git//ssh` are proof that the
blind spot was real before the grant.

`a_typed_command_over_ssh_at_a_terminal_asks_no_passphrase`
(`binary_ssh_terminal_tests.rs:376`) and
`an_unknown_host_key_is_refused_at_a_terminal_and_never_asked` (`:555`)
read only the pseudo-terminal and the status.
A refused askpass program produces the same status and the same OpenSSH sentence,
because an empty answer and no answer are the same refusal,
and the only trace of the difference is OpenSSH's own line,
`ssh_askpass: exec(<program>): Permission denied`,
which it writes on standard error
([`readpass.c:74-80`](https://github.com/openssh/openssh-portable/blob/V_10_6_P2/readpass.c#L74-L80)):

```c
// readpass.c:74-80, the child that runs the askpass program
	if (pid == 0) {
		close(p[0]);
		if (dup2(p[1], STDOUT_FILENO) == -1)
			fatal_f("dup2: %s", strerror(errno));
		if (env_hint != NULL)
			setenv("SSH_ASKPASS_PROMPT", env_hint, 1);
		execlp(askpass, askpass, msg, (char *)NULL);
		fatal_f("exec(%s): %s", askpass, strerror(errno));
	}
```

`run_at` appends that file to the transcript (`:193`),
so the trace is already in hand;
no assertion reads it.
With `local/git_ssh`'s `@{bin}/false rix` removed,
both controls would stay green while
proving nothing about the wrapper's no-answer program.
The fix is one assertion per wrapped arm:
the transcript must not contain `ssh_askpass: exec(`.

The four negative askpass controls are blind in the same way,
without a denial in this
window to blame:
`binary_lfs_server_tests.rs:1179` and `:1217` assert that the recorder's log does not exist,
and `service_push_retry_tests.rs:1263` and `:1293` assert that it holds no prompt.
A refused recorder would satisfy both.
Neither route reaches `ssh` (both push over `file://` or `http://`),
so no refusal was logged
for them here,
and the trap is the assertion,
not the run.
The fix is one positive arm per recorder that runs it through the same fixture and the same
profile path and asserts that it logs its question,
so that an empty log means the rule held rather than that the program never started.

### Proposed rules

One rule,
for the timer install's tool;
none for the gate.

```text
# /etc/apparmor.d/local/systemd-analyze (proposed, not applied)
# systemd-analyze verify makes its working folder with mkdtemp("/tmp/systemd-analyze-XXXXXX") and the stock profile
# covers only @{tmp}/systemd-temporary-*. Measured 2026-11-24: the daily-update timer install's own
# `systemd-analyze --user verify` printed "Failed to setup working directory: Permission denied" and exited 1, and the
# window of 01:22:44 holds the two mkdir refusals of that shape.
owner @{tmp}/systemd-analyze-@{rand6}/ rw,
```

Its tradeoff:
`systemd-analyze` may create,
list and remove a folder of six random characters in `/tmp`
that carries its own name prefix and nothing else,
which is what the stock profile already grants the same tool under another prefix.
It gains no read of another tool's folder and no write outside `/tmp`.
A verify given a `src:dest` argument also copies a unit into that folder
([`analyze-verify.c:45-51`](https://github.com/systemd/systemd/blob/8cc40e0c5e9234bf45084751ac53b1fbfe70b492/src/analyze/analyze-verify.c#L45-L51)),
which the rule above does not cover;
the second line `owner @{tmp}/systemd-analyze-@{rand6}/** rw,` belongs beside it only if a
run needs that form,
and no run of this repository does today.

### Applied, and what the runs showed

Applied by the coordinator on 2026-11-24 at about 02:35-05:00,
with the two read rules a run showed the folder rule alone does not reach:

```text
# /etc/apparmor.d/local/systemd-analyze
owner @{tmp}/systemd-analyze-@{rand6}/ rw,
owner @{HOME}/.config/systemd/user/{,**} r,
owner @{run}/user/@{uid}/systemd/{,**} r,
```

Before any rule,
`systemd-analyze --user verify` of the two repository units printed "Failed to setup working directory:
Permission denied"
and exited 1.
With the folder rule alone,
the working folder was made and the verify then failed reading the user unit search path:
"Failed to open "/home/user/.config/systemd/user",
ignoring:
Permission denied",
the same for `/run/user/1000/systemd/user.control` and `/run/user/1000/systemd/user`,
then "Unit toolchain-daily-update.service not found."
and exit 1;
the audit log held those three `open` denials.
With the two read rules beside it,
the verify of the installed units in `~/.config/systemd/user` (the folder the timer install writes and verifies in)
exited 0.
Two lines remain on every run,
"/usr/lib/systemd/user-environment-generators/30-systemd-environment-d-generator failed with exit status 1,
ignoring."
and the same for `60-flatpak`:
the profile does not let it run the user environment generators,
the verify ignores them,
and no rule is added for them.
The logs are `coordinator-verify-*.log` in
`~/temp/agent/apparmor-ninth-batch-for-the-gate-at-8519c599d0-2026-11-24/keep/`.
Tradeoff of the read rules:
`systemd-analyze` may read the account's user unit files and the user manager's runtime unit folders,
which is what a `--user` verify is for;
it gains no write there.

### What would show the proposal right

After the line is appended and
`apparmor_parser --replace /etc/apparmor.d/systemd-analyze` reloads the profile,
the same two unit files verify with no output and exit 0,
the daily-update timer install's verify step passes,
and a window of the audit log holds no `systemd-analyze` `mkdir` denial.
None of that is shown yet:
this round reads no audit log and edits no profile.

### What the log cannot decide

The two capability families stay partly unattributed,
and no further reading of the window can settle them,
because the record carries neither a path nor an owner.
The cheapest decider is a run of one named control,
the mount one or `binary_commit_tests.rs:1587`,
with the design round's container stopped and the coordinator reading the audit log as the
record appears:
the record then belongs to that control,
and no other process of the machine made it.
It changes no rule either way,
which is why this batch proposes none for `dac_override`.

## The askpass controls read a refusal, and prove the recorder starts

Recorded 2026-11-24T03:23:00-05:00 by the round `askpass-controls-see-a-refused-program-2026-11-24`,
on branch `fix/git-policy-cli-askpass-controls-see-a-refused-program`,
whose one code commit is `2f9d76aaea`.
Not approved and not final:
no whole container gate ran in this round.

The trap the section "The tests that would pass with their program refused" named is answered in the controls
themselves,
and not in a rule:
the sixth batch's `@{bin}/false rix` grant is what kept the two terminal controls green while proving nothing
about the wrapper's no-answer program,
and the four negative askpass controls were blind the same way with no grant to blame.

### What the terminal controls read now

`a_typed_command_over_ssh_at_a_terminal_asks_no_passphrase` and
`an_unknown_host_key_is_refused_at_a_terminal_and_never_asked`
each fail when their transcript holds `ssh_askpass: exec(`
(`REFUSED_ASKPASS_LINE` and `assert_the_no_answer_program_ran`
in `src/native/binary/binary_ssh_terminal_tests.rs`).
The line is in the transcript because `run_at` appends the command's standard error to it,
so no new read of the run was needed.
The file's third control needs no such assertion and says why in its own doc:
its key asks nothing,
so a program that could not start would leave the connection succeeding exactly as no program does,
and that success is what both of its arms assert.

### What the four negative askpass controls read now

`an_unattended_push_without_a_controlling_terminal_never_opens_the_password_dialog` and
`a_push_with_a_controlling_terminal_never_opens_the_password_dialog`
(`src/native/binary/binary_lfs_server_tests.rs`)
and `a_background_push_that_needs_a_login_never_starts_the_hosts_askpass_program` and
`a_background_lfs_upload_never_starts_the_hosts_askpass_program`
(`src/native/landing/service_push_retry_tests.rs`)
keep their assertions that the recorder's log is empty,
and each now runs the same recorder through the same program first and requires its log to hold the question.
The LFS pair runs the fixture's own `bin/git-lfs` link,
at the path the wrapper's search finds and a profile
attaches to,
against the same login-requiring server;
the push control runs real Git at `/usr/bin/git` against its `LoginServer`,
and the LFS-upload control runs the same `git-lfs` stand-in through the same plan with only the rule's empty
`GIT_ASKPASS` replaced by the recorder.
An empty log therefore means the rule held and not that the program never started.

### What the runs showed

Host,
`cargo test --locked --offline --test native_binary -- never_opens_the_password_dialog`
and `cargo test --locked --offline --lib -- never_starts_the_hosts_askpass_program`,
2026-11-24 between 02:52 and 02:56:
all four controls pass with their arms.
With the recorder written mode `0644` so nothing can start it,
the arms fail and say so,
quoting the program's own line
(`fatal: cannot exec '<path>': Permission denied` for Git and for `git-lfs`,
and `/bin/git-lfs: line 4: <path>: Permission denied` for the stand-in);
with the arms removed and the same mode in place,
all four controls pass,
which is the old blindness reproduced.
The patches of each state are `red-demo-lfs-recorder-cannot-start.patch` and
`red-demo-service-recorder-cannot-start.patch` in this round's `keep/`.

Gate image,
built from the committed sources by
`keep/build-the-gate-image-for-a-narrowed-run.ts` under the tag `askpass-refusal-20261124`
(the gate's own build functions,
no suite and no Clippy container),
then run in the gate's own container geometry
(`--memory 2g --cpus 4 --pids-limit 1024`,
the suite's tmpfs fixture root):
`Summary [1.460s] 7 tests run: 7 passed, 10842 skipped`.
The seven are the six controls above and `an_upload_that_needed_a_login_at_a_terminal_is_told_the_same_rule`,
which the filter matched and which passed.
The log is `gate-image-green-run.txt` in this round's `keep/`.
The same image ran every test of the three touched modules,
which is the three `ssh_terminal` controls,
the `lfs_server` controls and the `push_retry_tests` controls,
with `Summary [28.473s] 74 tests run: 74 passed, 10775 skipped`
(`gate-image-touched-modules-run.txt` in this round's `keep/`).

### What the terminal controls cannot show on the host

The three controls of `src/native/binary/binary_ssh_terminal_tests.rs` cannot run on this host at all,
and this is the confinement rather than the controls:
`/etc/apparmor.d/local/sshd` grants `owner /work/** rwlk` and nothing else,
so a throwaway `sshd` reads its configuration only under the gate container's `/work`.
Measured 2026-11-24T02:48:03-05:00:
`sshd -f <round scratch>/sshd-probe/sshd_config -D -e` printed
`<round scratch>/sshd-probe/sshd_config: Permission denied`,
and `cargo test --test native_binary -- at_a_terminal` on the same host failed at
`the throwaway sshd never accepted a connection on port 37481`,
which is before every assertion of either control.
So the red and green of the terminal assertion were shown in the gate image and not on the host.

### Red and green of the terminal assertion, in the gate image

Green:
the six controls' run recorded under "What the runs showed" has both terminal controls passing
with the assertion in place.

Red:
with the demonstration in `red-demo-ssh-false-cannot-start.patch`,
where each wrapped arm's `PATH` holds a link to `/usr/bin/git` and a mode-0600 file named `false`,
both controls fail at the assertion and quote the transcript it read.
The host-key control's transcript holds:

```text
ssh_askpass: exec(false): Permission denied
Host key verification failed.
fatal: Could not read from remote repository.
```

and the passphrase control's holds:

```text
ssh_askpass: exec(false): Permission denied
user@127.0.0.1: Permission denied (publickey).
fatal: Could not read from remote repository.
```

Every other assertion of both arms still holds in that state,
which is the trap itself:
the status is 128,
the key is refused,
and no question is asked,
and only the assertion this round added separates a program that answered nothing from one that never
started.
`Summary [1.334s] 5 tests run: 3 passed, 2 failed, 10844 skipped`
(`gate-image-red-ssh-run.txt` in this round's `keep/`).

With the same `PATH` and the two assertion calls removed
(`old-blindness-ssh-same-state-without-the-assertion.patch`),
the same run passes every one of the five:
`Summary [1.436s] 5 tests run: 5 passed, 10844 skipped`
(`gate-image-old-blindness-run.txt` in this round's `keep/`),
which is the blindness the ninth batch predicted and this round closes.
The `PATH` has to hold a `git` link because the wrapper resolves the real Git from `PATH`:
a `PATH` naming no Git stopped the command before it reached `ssh`,
with `Could not find a real Git executable after examining 1 PATH candidates`,
which was the first attempt at this demonstration and is recorded here rather than kept,
because the run's log was overwritten by the second run of the same file.

## Upstream filing artifact

### Upstream filing decision

Nothing to add upstream.
These incidents are local policy mismatches for this host's configured
paths and the custom Helium profile.

- Upstream fault:
  not established.
  The applications expose the paths and behavior used here;
  the
  local profile did not cover them.
- Upstream fix capability:
  not required for the verified local configuration changes.
- Supported use case:
  the actual native services,
  wrapper,
  browser launcher,
  password database,
  source workspace,
  and unprivileged container were exercised locally.
- Contribution policy:
  no filing is proposed.
  Incus also prohibits direct LLM-filed issues,
  and
  these local caller and policy incidents are not upstream-filing artifacts.
- Likelihood of an upstream fix:
  not assessed because no upstream defect is claimed.
- Minimal compatible prototype:
  the installed local rules and their passing fixtures are the
  consumer-side configuration changes.
  No upstream patch is offered.

Do not file a report from this document.
Reassess only if a reproducible defect remains after the
application and profile agree on the intended path and behavior.

## Tenth batch: git listing the folders on PATH, 2026-11-24

In the window of the whole gate run `native-mEmcZt` on `main` at `adcb708071`
(2026-11-24T14:15:42-05:00 to 14:37:51-05:00),
the `git` profile was denied `open` with read access on four folders,
each twice:
`~/.cargo/bin/`,
`~/.local/share/pnpm/bin/`,
`~/.jbcontext/bin/` and `/opt/rocm/bin/`.
Git opens each folder on `PATH` when it lists the commands it can run,
as `git help --all` does.
No test failed because of it.

Applied in `/etc/apparmor.d/local/git`,
by the owner's standing answer to add rules for whatever the work is denied:

```text
# /etc/apparmor.d/local/git
owner @{HOME}/.cargo/bin/ r,
owner @{HOME}/.local/share/pnpm/bin/ r,
owner @{HOME}/.jbcontext/bin/ r,
/opt/rocm/bin/ r,
```

`apparmor_parser --replace /etc/apparmor.d/git` exited 0,
and `git help --all` afterwards exited 0 with no `git` denial in the kernel log.
The same window's `umount` request for `dac_override` stays refused,
as the ninth batch decided.
(Note of 2026-11-25:
the eleventh batch allows it,
by the owner's "Always widen apparmor.")

## Thirteenth batch: IVPN Desktop 3.15.15, and apparmor_parser reading scratch drafts, 2026-11-26T01:41:13-05:00

Met while installing the AUR packages `ivpn` and `ivpn-ui` `3.15.15` through Shelly
(the build failure itself is in `doc/troubleshooting/ivpn-ui-aur-recipe-nvm-rejects-npmrc-prefix-in-builder-home.md`).
Under the owner's rule recorded under "Eleventh batch",
and the owner's request on 2026-11-26 to make sure both packages "have suitable apparmor rules".
Probes ran in throwaway network namespaces (`sudo unshare --net`),
so no firewall rule, route or tunnel reached the host;
the AVC windows were read from `/var/log/audit/audit.log`.
Probe logs: `~/temp/agent/ivpn-apparmor-netns-probes-2026-11-26/keep/`.

### OpenVPN connections were denied their configuration

The IVPN daemon runs `/usr/sbin/openvpn --config /etc/opt/ivpn/mutable/openvpn.cfg`
(IVPN `daemon/service/service_connect.go:427-433`, `daemon/vpn/openvpn/openvpn.go:435`).
The enforcing `openvpn` profile reads only `/etc/openvpn/**`,
so with a probe configuration at that exact path the measured result was:

```text
apparmor="DENIED" operation="open" class="file" profile="openvpn" name="/etc/opt/ivpn/mutable/openvpn.cfg" requested_mask="r"
Options error: In [CMD-LINE]:1: Error opening configuration file: /etc/opt/ivpn/mutable/openvpn.cfg
```

Every IVPN OpenVPN connection would have stopped there.
The new `/etc/apparmor.d/local/openvpn` grants the configuration,
the proxy credential file,
the bundled `ca.crt` and `ta.key`,
and runs IVPN's `client.up` and `client.down` DNS scripts in a child profile:

```text
# /etc/apparmor.d/local/openvpn (new)
/etc/opt/ivpn/mutable/openvpn.cfg r,
/etc/opt/ivpn/mutable/proxyauth.txt r,
/opt/ivpn/etc/ca.crt r,
/opt/ivpn/etc/ta.key r,
/opt/ivpn/etc/client.{up,down} rcx -> ivpn-client-updown,
profile ivpn-client-updown {
  include <abstractions/attached/base>
  include <abstractions/attached/consoles>
  /opt/ivpn/etc/client.{up,down} r,
  @{sh_path} rix,
  @{bin}/mv rix,
  @{bin}/chmod rix,
  @{bin}/resolvectl rPx,
  /etc/resolv.conf rw,
  /etc/resolv.conf.ivpnsave rw,
  include if exists <local/openvpn_ivpn-client-updown>
}
```

After `apparmor_parser --replace /etc/apparmor.d/openvpn.apparmor.d`,
the same probe read the configuration,
built its TLS context from the CA and key,
and stopped only at the namespace's intended `Network is unreachable`,
with no `openvpn` denial.
The child profile ran both scripts with `-use-resolvconf /usr/bin/resolvectl`
(the branch used when `systemd-resolved` answers),
and `resolvectl` ran under its own profile;
an interface name that does not exist kept host DNS untouched.
Run from `/`, the service's working directory, there was no denial;
run from a folder inside the owner's home,
`dac_read_search` and `dac_override` were denied for traversing it,
which is a property of the probe and was not granted.
The `resolv.conf` rewrite branch, used only without `systemd-resolved`,
and a real tunnel's `up` call were not exercised.

### WireGuard already worked through the existing profiles

IVPN's bundled `/opt/ivpn/wireguard-tools/wg-quick` with IVPN's configuration shape
(`AllowedIPs = 0.0.0.0/0, ::/0`, `SaveConfig = true`)
runs `ip`, `nft -f /dev/fd/63`, `sysctl` and `sync` under their enforcing profiles.
`up` and `down` both exited `0` and removed the interface, rules and nft tables,
with no AVC.
With the configuration under `~/temp/agent`,
`down` failed instead:
the `sync` profile has no `dac_override`,
root needed it to reach a file below the owner's `0700` folders,
and `wg-quick`'s `set -e` then left the interface, rules and nft tables in place.
IVPN keeps the file in `/etc/opt/ivpn/mutable`,
which root reaches without that capability,
so nothing was granted for it.

### New profiles for IVPN's own programs

- `/etc/apparmor.d/ivpn`, enforcing:
  the CLI reads `/etc/opt/ivpn/mutable/port.txt`,
  talks TCP to the daemon,
  and `ivpn exclude` replaces itself with the requested command (`/** pux`).
  `ivpn status`, `ivpn splittun -status` and `ivpn servers` ran with no denial.
  With split tunneling on, `ivpn exclude /usr/bin/env` and the UI's form
  `sh -c '/usr/bin/ivpn exclude  /usr/bin/env'` ran with no denial.
  The first draft used `PUx`:
  the kernel then marked the exec secure and glibc dropped `TMPDIR` from the launched program,
  which a direct start keeps
  (see `doc/troubleshooting/pdeathsig-is-cleared-by-a-confined-exec.md` for the secure-exec rule);
  with `pux` the variable arrived.
- `/etc/apparmor.d/ivpn-ui`, `flags=(unconfined)` with `userns`,
  the form apparmor.d uses for its Electron applications here
  (`obsidian`, `Discord`, `signal-desktop`, `1password`, `balena-etcher`).
  A launch in a nested headless `kwin_wayland --virtual` with a private D-Bus showed every process labelled
  `ivpn-ui (unconfined)` and an established connection to the daemon's port.
- `/etc/apparmor.d/ivpn-service`, in complain mode,
  a documented exception like the package complain profiles.
  It lists `Px` for each child that has its own profile
  (`openvpn`, `ip`, `nft`, `sysctl`, `sync`, `resolvectl`, `ps`, `uname`, `mount`, `umount`, `runuser`),
  so those keep the confinement they get from an unconfined daemon,
  and lets IVPN's scripts and bundled helpers inherit it.
  After `systemctl restart ivpn-service` the daemon ran as `ivpn-service (complain)`.
  Its first start logged `netlink dgram` and its own `mountinfo`;
  both are in the profile now,
  and the next start logged nothing.
  `firewall.sh -enable`, `-status` and `-disable`
  and `wg-quick up` and `down` under `aa-exec --profile ivpn-service` in a namespace logged nothing either,
  while a control read of `/etc/shadow` under the same profile was logged.
  When the owner then opened the UI and turned on inverse split tunneling,
  complain mode logged `splittun.sh` running `/usr/bin/rmdir` on `/sys/fs/cgroup/net_cls/ivpn-exclude/`
  (as `ivpn-service//null-/usr/bin/rmdir`, with `dac_override`)
  and the daemon reading desktop-entry icons such as `/usr/share/hplip/data/images/128x128/hp_logo.png`;
  `rmdir` and `png`, `svg`, `svgz` and `xpm` files under `/usr/{,local/}share` and `/opt` were added.
  The owner then connected and disconnected with WireGuard (with V2Ray) and with OpenVPN,
  with inverse split tunneling on and Firefox ESR started through it;
  complain mode logged nothing further for `ivpn-service`,
  and `openvpn` and `openvpn//ivpn-client-updown` logged no denial.
  The built-in firewall was not used,
  since the owner does not use it;
  it ran under the profile only in the namespace probes.
  On 2026-11-26 at 06:55 UTC the profile was switched to enforce
  (`flags=(complain)` removed, `apparmor_parser --replace`),
  while the owner's tunnel stayed up:
  the daemon and its `v2ray` helper then ran as `ivpn-service (enforce)`,
  the namespace probes (firewall enable, status, disable; `wg-quick` up and down) all exited `0` under it,
  and the next minute of live use held no `ivpn-service` denial.

### The daemon's netlink socket leaks into `ip` and `ps`

Once the owner turned on split tunneling,
the stock `ip` profile logged 1128 denials between 06:40 and 06:48 UTC,
90 to 188 a minute, all of one shape:

```text
apparmor="DENIED" operation="file_inherit" class="net" info="failed type match" profile="ip" comm="ip" family="netlink" sock_type="dgram"
```

`ivpn-service` holds a netlink datagram socket without close-on-exec
(fd 7, `flags=02`, its inode listed in `/proc/<pid>/net/netlink`),
and polls `ip route` (IVPN `daemon/netinfo/netinfo_linux.go:74`);
its `ps -aux` children met the same denial.
`ip` and `ps` still worked with the descriptor revoked,
but the flood fed an audit backlog that already drops records.
This does not depend on the new `ivpn-service` profile:
an unconfined daemon's children attach to `ip` and `ps` the same way.
Applied with `append-a-batch-of-local-rules-and-load.ts`
(label `ivpn-netlink-inherit-and-aa-install-cpu-max-2026-11-26`),
following the owner's rule rather than an apparmor.d-style `deny` silencer:

```text
# /etc/apparmor.d/local/ip, /etc/apparmor.d/local/ps
# and, after the owner's connects at 06:52 UTC met the same denial in them,
# /etc/apparmor.d/local/nft, local/resolvectl, local/sysctl and local/sync
network netlink dgram,
# /etc/apparmor.d/local/aa-install
@{sys}/fs/cgroup/user.slice/user-@{uid}.slice/user@@{uid}.service/app.slice/*/cpu.max r,
```

The next 20 seconds held no `ip`, `ps` or `aa-install` denial.
The `aa-install` line answers five denials of
`.../user@1000.service/app.slice/claude-code-bash/cpu.max`
from this session's Shelly transactions,
whose apparmor.d hook runs `aa-install` in the caller's cgroup;
the file belongs to the user,
so the existing `owner` rule could not match it.
`sudo aa-install --status` afterwards reported 710 profiles up to date with no denial.

### Firefox ESR started through the split tunnel

The owner then started Firefox ESR (`/opt/firefox-esr`) from the IVPN UI:
its process carried `IVPN_STARTED_BY_PARENT=IVPN_UI`, its parent was `ivpn-ui`,
and it ran as `firefox (enforce)`.
`firefox` was denied the root-owned `/etc/os-release` (four reads),
and `firefox-crashreporter`, whose `@{lib_dirs}` names `/opt/firefox` only,
was denied `/opt/firefox-esr/defaults/pref/` and `/opt/firefox-esr/libxul.so`.
`aa-exec --profile firefox -- head --lines=1 /etc/os-release`
and the same read of `libxul.so` under `firefox-crashreporter` reproduced both denials first.

```text
# /etc/apparmor.d/local/firefox (appended)
/etc/os-release r,
# /etc/apparmor.d/local/firefox-crashreporter (new)
/opt/firefox-esr/{,**} r,
```

These were appended by hand and loaded with
`apparmor_parser --replace /etc/apparmor.d/firefox.apparmor.d /etc/apparmor.d/firefox-crashreporter`.
`append-a-batch-of-local-rules-and-load.ts` would have reloaded `/etc/apparmor.d/firefox`,
the `apparmor` package's unconfined-flag stub that `/etc/apparmor.d/disable/firefox` keeps out of the boot load;
`apparmor_parser` loads a file named on its command line regardless of that folder,
so the stub would have replaced the enforcing `firefox` profile.
Afterwards both profiles were still enforcing and both reads succeeded with no denial.

### Tor Browser's pluggable transport and shader cache

Seen in the same window, from Tor Browser started on its own (not through the split tunnel).
Between 06:55 and 07:02 UTC `torbrowser-tor` denied `lyrebird`,
the pluggable transport `tor` runs inside that profile,
362 `inet` and 144 `inet6` datagram sockets, two `inet6` stream sockets and its own `/proc/<pid>/cgroup`;
the stock profile allows only `network inet stream`,
so datagram transports could not run.
`torbrowser` denied 59 locks on `.cache/mesa_shader_cache/<xx>/<hash>.tmp`,
where its rule grants `rw` but not `k`.
The owner then said "Grant tor."
Applied with `append-a-batch-of-local-rules-and-load.ts` (label `tor-browser-lyrebird-and-shader-cache-2026-11-26`):

```text
# /etc/apparmor.d/local/torbrowser-tor (new)
network inet dgram,
network inet6 dgram,
network inet6 stream,
owner @{PROC}/@{pid}/cgroup r,
# /etc/apparmor.d/local/torbrowser (new)
owner @{lib_dirs}/.cache/{,**} k,
```

A first check reported 289 Tor Browser denials in the 90 seconds before loading and none after.
That was weak evidence twice over:
`lyrebird` may simply have been idle in that window,
and the reader took `stdout` from one `rg` over five audit files,
which the agent harness cuts to its start when it is long,
so later lines could be missing.
Re-read from `/var/log/audit/audit.log` alone (it had not rotated since 01:11 local) with the whole output,
the socket denials did stop at the load,
but the owner reported Tor Browser still not connecting.

The stock profile's `deny /etc/resolv.conf r`, `deny /etc/hosts r` and `deny /etc/nsswitch.conf r`
apply to `lyrebird` too, because it runs there through `PluggableTransports/** mix`;
`deny` rules are not logged, and a local allow cannot lift them.
Snowflake has to resolve its front (`app.datapacket.com`) and its STUN servers.
Without `/etc/resolv.conf` a resolver falls back to `127.0.0.1:53`,
and nothing listens there on this host (systemd-resolved is on `127.0.0.53`).
`aa-exec --profile torbrowser-tor -- getent hosts app.datapacket.com` exited `2` with no AVC,
while the same lookup outside the profile returned `185.59.222.177`.

An exact-path `cx` rule for `lyrebird` failed to compile
("profile has merged rule with conflicting x modifiers") against the stock glob.
AppArmor 4.1's `priority=` qualifier, which "will completely override permissions of lower priority rules
where they overlap" (`apparmor.d(5)`), compiled:

```text
# /etc/apparmor.d/local/torbrowser-tor (added)
priority=1 @{lib_dirs}/TorBrowser/Tor/PluggableTransports/lyrebird rcx -> lyrebird,
signal receive set=(term kill) peer=torbrowser,
profile lyrebird {
  include <abstractions/base-strict>
  include <abstractions/nameservice-strict>
  include <abstractions/ssl_certs>
  network inet stream,
  network inet6 stream,
  network inet dgram,
  network inet6 dgram,
  network netlink raw,
  @{lib_dirs}/TorBrowser/Tor/PluggableTransports/lyrebird mr,
  @{lib_dirs}/TorBrowser/Tor/PluggableTransports/{,**} r,
  owner @{data_dirs}/Tor/pt_state/{,**} rw,
  owner @{PROC}/@{pid}/cgroup r,
  owner @{PROC}/@{pid}/mountinfo r,
  @{PROC}/sys/net/core/somaxconn r,
  @{sys}/kernel/mm/transparent_hugepage/hpage_pmd_size r,
  owner @{sys}/fs/cgroup/user.slice/user-@{uid}.slice/user@@{uid}.service/app.slice/*/cpu.max r,
  signal receive peer=torbrowser-tor,
  include if exists <local/torbrowser-tor_lyrebird>
}
# /etc/apparmor.d/local/torbrowser (appended)
owner @{lib_dirs}/.config/pulse/ rw,
owner @{lib_dirs}/.config/pulse/** rwk,
```

`tor` itself keeps the stock denies.
The signal rule answers the browser's `term` and `kill` being denied at 07:06:18 UTC,
and the `pulse` lines its `mkdir` of `.config/pulse` at 07:05:45.
The child ran in complain mode while the owner restarted Tor Browser and connected,
which succeeded at 07:14 UTC with `lyrebird` labelled `torbrowser-tor//lyrebird (complain)`;
it logged only the `cpu.max` read, which was added before switching it to enforce.
Under enforce,
`curl --socks5-hostname 127.0.0.1:9150 https://check.torproject.org/api/ip` returned
`{"IsTor":true,"IP":"204.8.96.168"}`,
and the next 45 seconds held no `torbrowser-tor` denial.

The same reader fault touches this batch's other "no denial" checks;
re-read the same way,
`ivpn-service` (since enforce), `ip` and `ps` (after 06:50:21 UTC)
and `nft`, `resolvectl`, `sysctl` and `sync` (after 06:58:40 UTC) still held none.
The `firefox` denials after 06:58:40 were another session's negative control
(`~/Documents/Restore Firefox/Unrelated_apparmor-negative-control`).
These are apparmor.d's `torbrowser` and `torbrowser-tor`,
not the `apparmor` package's `torbrowser.Browser.firefox` and `torbrowser.Tor.tor`,
whose empty local files were left alone.
`zfs` denying `/sys/module/zfs/version` once was seen and not granted.

### apparmor_parser on drafts in the scratch folder

`sudo apparmor_parser --skip-kernel-load` on profile drafts in `~/temp/agent`
printed `File ... not found, skipping...` and exited `13`;
the denials were `capability dac_read_search` and `capability dac_override`.
The owner then said "Widen apparmor rules so apparmor_parser works."
Applied with `append-a-batch-of-local-rules-and-load.ts` (label `apparmor-parser-scratch-read-2026-11-26`):

```text
# /etc/apparmor.d/local/apparmor_parser
capability dac_read_search,
capability dac_override,
@{HOME}/temp/agent/{,**} r,
```

All three drafts then parsed from the scratch folder with exit `0` and no denial.

## Twelfth batch: Git and pdftotext in ~/Plain, and systemd-analyze on user units, 2026-11-26T00:03:44-05:00

Met while resolving the Garage sync's conflict copies and building `package/garage-file-sync/linux-client`,
read from `/var/log/audit/audit.log` between 2026-11-25T21:00-05:00 and the batch:

- `git` reading `~/Plain/Text/Reference/Legal/.git/HEAD` (5) and `~/Plain/Text/Reference/Health/.git` (4),
  so Git in those repositories printed "not a git repository" or "error opening ... Permission denied";
  the stock profile grants only `@{user_projects_dirs}`.
  Health's Git directory was restored to `~/.local/share/health-records-git` the same night, so it is granted too.
- `git` reading the Claude Code harness's per-command diff folders under `/tmp/claude-1000/bash-edit-diff/` (about 200).
- `systemd-analyze --user verify` reading `~/.config/systemd/user/`, `/run/user/1000/systemd/user{,.control}/`,
  the folders of the unit files it was given (scratch and worktree folders), `~/.config/environment.d/`,
  and running the two user environment generators.
  A writer and two reviewers worked around this by reading units by hand instead.
- `pdftotext` opening `~/Plain/.../carleton-counter-2026-08-26--518a77e7.pdf.conflict1`,
  a PDF whose name does not end in `.pdf`; the session fed it through standard input instead.

Those two workarounds broke the owner's rule recorded under "Eleventh batch",
because the session had not read this file; the owner then said "Add the rules".
Applied with `append-a-batch-of-local-rules-and-load.ts` (labels `twelfth-batch-2026-11-26` and `twelfth-batch-environment-d-2026-11-26`):

```text
# /etc/apparmor.d/local/git
owner @{HOME}/Plain/ r,
owner @{HOME}/Plain/** rwlk -> @{HOME}/Plain/**,
owner @{HOME}/.local/share/health-records-git/ r,
owner @{HOME}/.local/share/health-records-git/** rwlk -> @{HOME}/.local/share/health-records-git/**,
owner /tmp/claude-@{uid}/bash-edit-diff/ r,
owner /tmp/claude-@{uid}/bash-edit-diff/** r,
# /etc/apparmor.d/local/systemd-analyze
owner @{HOME}/.config/systemd/user/ r,
owner @{HOME}/.config/systemd/user/** r,
owner @{run}/user/@{uid}/systemd/ r,
owner @{run}/user/@{uid}/systemd/** r,
owner @{HOME}/temp/agent/** r,
owner @{HOME}/Monochromatic/** r,
/usr/lib/systemd/user-environment-generators/* rix,
owner @{HOME}/.config/environment.d/ r,
owner @{HOME}/.config/environment.d/** r,
# /etc/apparmor.d/local/pdftotext (new)
owner @{HOME}/Plain/** r,
```

`apparmor_parser --replace` loaded `git`, `systemd-analyze` and `pdftotext`.
Afterwards `git log` and `git status` worked in both repositories,
`systemd-analyze --user verify` of the installed `rclone-bisync-garage*` units exited 0 with no message,
and `pdftotext` read a PDF under `~/Plain`; the audit log held no new denial for these profiles.
No exec grant was added under `~/Plain`, so repository hooks there still cannot run.

Seen in the same window and not granted, since this work did not need them:
`gh` running `~/Monochromatic/.cache/toolchain-launchers/monochromatic-git-policy-cli` (3; its `gh api` reads succeeded),
`helium`, `systemd-coredump`, `wl-copy` and `systemd-oomd`.

## Eleventh batch: carving swap from the old NVMe, and the owner's rule to always widen, 2026-11-25

The owner,
on 2026-11-25 while this session worked around a denial instead of widening the profile:
"Always widen apparmor."
So a denial the work meets is answered by a rule that allows it,
written to the profile's `local/` file and reloaded,
never by moving the work somewhere the profile happens to allow.

The denials,
read from `/var/log/audit/audit.log` between 2026-11-25T14:20:00-05:00 and 14:32:00-05:00,
while the session unlocked and shrank the old NVMe's filesystem for a swap partition:

- `pacman-key` and `gpg` opening `/var/cache/pacman/pkg/linux-cachyos-7.3.5-2-x86_64_v4.pkg.tar.zst.sig` for read,
  so `sudo pacman-key --verify` printed `gpg: can't open ... Permission denied` even as root.
- `kmod` (`modinfo`,
  `insmod`) opening `dm-crypt.ko.zst` outside `/usr/lib/modules`.
- `cryptsetup` opening and creating a LUKS header backup under `~/temp/agent/`
  (`Cannot create header backup file`).
- `btrfs` reading `/sys/devices/virtual/block/dm-0/queue/zoned`,
  opening the mount folder of the filesystem it maintains,
  and asking `dac_read_search` and `dac_override`.

Applied,
one `local/` file each,
each under a comment naming its denial and the owner's words:

```text
# /etc/apparmor.d/local/pacman-key, /etc/apparmor.d/local/pacman-key_gpg, /etc/apparmor.d/local/gpg
/var/cache/pacman/pkg/** r,
# /etc/apparmor.d/local/kmod
/**.ko{,.gz,.xz,.zst} r,
# /etc/apparmor.d/local/cryptsetup
@{HOME}/temp/agent/** rwk,
# /etc/apparmor.d/local/btrfs
capability dac_read_search,
capability dac_override,
/sys/devices/virtual/block/*/queue/** r,
/** r,
# /etc/apparmor.d/local/umount
capability dac_override,
```

`apparmor_parser --replace` exited 0 for `pacman-key`,
`gpg`,
`kmod`,
`cryptsetup`,
`btrfs` and `umount`.
Afterwards `sudo pacman-key --verify` on that package printed
`gpg: Good signature from "CachyOS <admin@cachyos.org>" [full]`,
and `btrfs filesystem usage -T` read the mounted filesystem.
The `umount` rule ends the ninth batch's refusal of `dac_override`,
which every gate window since has logged (for example the gate at `7ed5ec0b34`).

Later the same afternoon,
while setting up zram writeback to the old NVMe
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"zram writeback to the old NVMe"):

```text
# /etc/apparmor.d/local/systemd-analyze
/usr/bin/flatpak rix,
/sys/firmware/acpi/fpdt/** r,
/tmp/systemd-analyze-*/ rw,
/tmp/systemd-analyze-*/** rwl,
# /etc/apparmor.d/local/zram-generator
/{etc,run,usr/lib,usr/local/lib}/systemd/zram-generator.conf.d/ r,
/{etc,run,usr/lib,usr/local/lib}/systemd/zram-generator.conf.d/*.conf r,
@{HOME}/temp/agent/** rw,
capability dac_read_search,
capability dac_override,
/dev/dm-* rw,
/dev/mapper/ r,
/dev/mapper/* r,
```

`systemd-analyze verify` had printed a flatpak generator's `Permission denied`
and then failed with "Failed to setup working directory";
afterwards it exited 0 with no denial between 15:15:17 and 15:15:19.
The `zram-generator` profile allowed `/etc/systemd/zram-generator.conf` and none of its drop-in folders,
so a `conf.d` file made the generator fail with `Permission denied (os error 13)`,
which at boot is no zram swap at all;
and when `--setup-device` writes `backing_dev` the kernel opens the device as that process,
denied as `open /dev/dm-0 wr`.
A probe on a throwaway `zram2` read error 13 before the `/dev/dm-*` rule and error 16 (busy) after it.
The home and capability lines serve a dry run of the generator into a scratch folder.
The main `systemd-analyze` profile does include `local/systemd-analyze`,
on the line before its closing brace;
a search capped at two lines first hid it.
