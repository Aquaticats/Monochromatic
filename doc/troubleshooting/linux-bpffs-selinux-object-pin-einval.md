# Linux 7.1.3 with SELinux returns `EINVAL` when `BPF_OBJ_PIN` creates a bpffs object

## Symptom

On kernel `7.1.3-ogc5.1.fc44.x86_64` with SELinux enforcing,
`wg-quicker-exempt attach` loads its map and programs and creates a cgroup BPF link,
but the first `BPF_OBJ_PIN` fails:

```text
error: attach /sys/fs/cgroup/wgq-strace-probe: Invalid argument (os error 22)
```

The syscall trace identifies the failing boundary:

```text
bpf(BPF_MAP_CREATE, ..., 168) = 5
bpf(BPF_MAP_UPDATE_ELEM, ..., 168) = 0
bpf(BPF_PROG_LOAD, ..., 168) = 6
bpf(BPF_LINK_CREATE, ..., 168) = 7
bpf(BPF_OBJ_PIN, {pathname=".../connect4", bpf_fd=7, ...}, 168) = -1 EINVAL
```

The same `EINVAL` occurs when pinning a map,
 program,
 or link at the bpffs root or in a nested directory.

A separate bpffs rule rejects any path component containing a dot with `EPERM`.
It affected the loader's first hidden staging-directory design and would affect unencoded Ghostty scope names.
This is not the cause of the `BPF_OBJ_PIN` `EINVAL` after names are encoded without dots.

Kernel `7.2.0-ogc6.1.fc44.x86_64` no longer reproduced object-pin failure.
Same disposable attach probe completed with:

```text
attached mark=8888 to /sys/fs/cgroup/wgq-pin-recovery-probe (4 links pinned)
```

First functional-test run on recovered kernel failed for separate harness assumptions:

```text
build debug CLI before functional tests: .../target/debug/build/wg-quicker-exempt/.../wg-quicker-exempt
active keeper absent
keeper state absent
```

## Root cause

The source trace uses upstream Linux commit
`8ba098e6b6ff0db8edf28528d1552be261af30d4` from 2026-07-30.
The running kernel is `7.1.3-ogc5.1.fc44.x86_64`.

`kernel/bpf/inode.c:491-523` routes `BPF_OBJ_PIN` through security initialization and object creation:

```c
static int bpf_obj_do_pin(int path_fd, const char __user *pathname, void *raw,
                          enum bpf_type type)
{
        /* ... */
        ret = security_path_mknod(&path, dentry, mode, 0);
        if (ret)
                goto out;

        switch (type) {
        case BPF_TYPE_PROG:
                ret = vfs_mkobj(dentry, mode, bpf_mkprog, raw);
                break;
        case BPF_TYPE_MAP:
                ret = vfs_mkobj(dentry, mode, bpf_mkmap, raw);
                break;
        case BPF_TYPE_LINK:
                ret = vfs_mkobj(dentry, mode, bpf_mklink, raw);
                break;
```

Each object creation reaches `security_inode_init_security()` at
`kernel/bpf/inode.c:363-380`:

```c
ret = security_inode_init_security(inode, dir, &dentry->d_name,
                                   bpf_fs_initxattrs, NULL);
if (ret && ret != -EOPNOTSUPP) {
        iput(inode);
        return ret;
}
```

The affected SELinux implementation initializes the inode security state before checking whether the mount supports
per-inode labels.
 `security/selinux/hooks.c:2983-2993` shows that order:

```c
if (sbsec->flags & SE_SBINITIALIZED) {
        struct inode_security_struct *isec = selinux_inode(inode);
        isec->sclass = newsclass;
        isec->sid = newsid;
        isec->initialized = LABEL_INITIALIZED;
}

if (!selinux_initialized() ||
    !(sbsec->flags & SBLABEL_MNT))
        return -EOPNOTSUPP;
```

Carlos Llamas reported this regression in the upstream thread and posted a patch on 2026-07-30.
The patch moves the `SBLABEL_MNT` check before the inode security state is initialized.
It names commit `9722955b5430` (`bpf: Add simple xattr support to bpffs`) as the regression source and requests stable
backports.

The earlier hypothesis that pin-directory depth or BPF object type caused `EINVAL` was wrong.
Root-level and nested pins fail,
 and map,
 program,
 and link pins all fail.
The same failure also occurs on a fresh bpffs mounted in a disposable mount namespace.

The independent dot-name failure is explicit in `kernel/bpf/inode.c:414-424`:

```c
/* Dots in names (e.g. "/sys/fs/bpf/foo.bar") are reserved for future
 * extensions. That allows popoulate_bpffs() create special files.
 */
if ((dir->i_mode & S_IALLUGO) &&
    strchr(dentry->d_name.name, '.'))
        return ERR_PTR(-EPERM);
```

`wg-quicker-exempt` therefore encodes canonical cgroup path bytes as chunked hexadecimal components.
That encoding is injective,
 contains no dots,
 and keeps each component within `NAME_MAX`.

### Why the functional harness failed after kernel recovery

`package/wg/cli-quicker-exempt/src/main.rs:72-82` selects real pins when attachment succeeds and enters
keeper fallback only for typed object-pin failure:

```rust
match pin::attach_cgroup(mark, cgroup) {
    Ok(count) => {
        keeper::detach_keeper(cgroup)?;
        println!("attached mark={mark} to {dir} ({count} links pinned)");
    }
    Err(error) if bpf_error::is_pin_object_invalid(&error) => {
        keeper::replace_keeper(mark, cgroup)?;
```

Fallback-specific tests previously relied on host kernel producing that error.
Successful pinning on kernel `7.2.0-ogc6.1.fc44.x86_64` therefore left no keeper state for assertions such as
`package/wg/cli-quicker-exempt/src/pin_tests.rs:609-617`:

```rust
run_cli(&["attach", "8888", cgroup])?;
let output = Command::new(cli_binary()?)
    .args(["attach", "9999", cgroup])
    .env(FORCE_PIN_INVALID_ENV, "1")
    .env("WG_QUICKER_EXEMPT_TEST_FAIL_AFTER_ATTACH", "1")
```

An independent path assumption also failed first.
Current Rust test executable was under `target/debug/build/.../out`,
not directly under `target/debug/deps`.
`package/wg/cli-quicker-exempt/src/pin_tests.rs:54-73` now walks test executable ancestors until it finds actual
`debug/wg-quicker-exempt`:

```rust
for directory in test_binary.ancestors() {
    if !directory.ends_with("debug") {
        continue;
    }
    let binary = directory.join("wg-quicker-exempt");
    if binary.exists() {
        return Ok(binary);
    }
}
```

## Verification

Affected environment:

```text
kernel: 7.1.3-ogc5.1.fc44.x86_64
SELinux: Enforcing
bpffs: /sys/fs/bpf, rw, mode=700
LSM order: lockdown,capability,yama,selinux,bpf,landlock,ipe,ima,evm
selinux-policy: 44.4-1.fc44
```

Recovered comparison:

```text
kernel: 7.2.0-ogc6.1.fc44.x86_64
wg-quicker-exempt attach: 4 links pinned
wg-quicker-exempt detach: completed
```

Reproduce the object-pin failure with a disposable cgroup:

```sh
cgroup=/sys/fs/cgroup/wgq-pin-repro
sudo mkdir -- "$cgroup"
sudo strace --trace=bpf \
  package/wg/cli-quicker-exempt/target/debug/wg-quicker-exempt \
  attach 8888 "$cgroup"
sudo package/wg/cli-quicker-exempt/target/debug/wg-quicker-exempt \
  detach "$cgroup" || true
sudo rmdir -- "$cgroup"
```

Working catalog:

- `BPF_MAP_CREATE` and `BPF_MAP_UPDATE_ELEM` succeed.
- `BPF_PROG_LOAD` succeeds for all four cgroup socket-address hook types.
- `BPF_LINK_CREATE` succeeds.
- Holding unpinned link descriptors marks TCP4,
   TCP6,
   UDP4,
   and UDP6 sockets with `8888`.
- Dropping those descriptors detaches all four programs and subsequent sockets retain mark `0`.
- The detached holder preserves all four marks after the invoking CLI exits.
- Reattach changes the observed mark from `8888` to `9999`,
   and detach returns it to `0`.
- A candidate failure after all four new links attach leaves the prior `8888` holder active.
- State pointing at a wrong process blocks detach and preserves the live holder.
- Parent death before transition state closes candidate stdin and drops all untracked links.
- Recovery rejects a live candidate without a commit marker and preserves the prior holder.
- Recovery adopts a live committed candidate before performing the next replacement.
- A removed cgroup still maps to its lexical state key and detaches cleanly.
- A bpffs directory name without a dot can be created and removed.
- Kernel `7.2.0-ogc6.1.fc44.x86_64` pins all four links and removes them through public CLI.
- Debug functional suite passes all ignored root tests.
- Fallback-specific cases force descriptor fallback while public lifecycle case retains native pin coverage.

Current complete harness is:

```sh
mise run //package/wg/cli-quicker-exempt:buildAndTest
mise run //package/wg/cli-quicker-exempt:test:functional
```

Fallback-specific debug functional invocations inject typed object-pin failure at
`package/wg/cli-quicker-exempt/src/pin.rs:302-306`:

```rust
#[cfg(debug_assertions)]
if std::env::var_os(FORCE_PIN_INVALID_ENV).is_some() {
    let source = io::Error::from_raw_os_error(libc::EINVAL);
    let error = pin_object_invalid(staging_text.to_string(), source);
    return Err(rollback_staging_error(&staging_dir, error));
}
```

This keeps fallback coverage independent of installed kernel while release compilation omits injection.
Public-CLI lifecycle test runs without this flag and retains native pin-path coverage.

Failing `EINVAL` catalog:

- pinning `BPF_TYPE_MAP` directly under the bpffs root;
- pinning `BPF_TYPE_PROG` directly under the bpffs root;
- pinning `BPF_TYPE_LINK` directly under the bpffs root;
- pinning each object type in a nested tool-owned directory;
- pinning on a fresh bpffs mounted in a disposable mount namespace.

Failing `EPERM` catalog:

- creating `.probe` under the bpffs root;
- creating `.probe` in a nested bpffs directory.

## Verified workarounds

### Keep link descriptors open

The privileged test harness retains all four `BPF_LINK_CREATE` descriptors instead of pinning them.
All four protocols receive the requested mark while the descriptors remain open.
Dropping them cleanly detaches the marker.

Tradeoff:
 the links do not survive holder-process exit.
`wg-quicker-exempt` now starts a detached holder through a two-phase handshake,
 persists transition state,
 validates PID,
start time,
 full command,
 mark,
 and cgroup before shutdown,
 and recovers interrupted replacement.
A holder crash safely detaches its links instead of leaving unowned kernel state.

### Inject typed pin failure in debug functional tests

Debug-only environment seam returns same typed object-pin error used by production behavior selection.
This makes descriptor-holder tests deterministic on affected and recovered kernels.

Tradeoff:
test confirms fallback response to exact error boundary,
not whether current kernel naturally emits that error.
Release builds contain no injection branch.

### Encode every bpffs path component without dots

The loader's hexadecimal canonical-path encoding avoids bpffs's reserved-dot rule and slash-replacement collisions.

Tradeoff:
 encoded paths are not human-readable.
The loader must compute them from the canonical cgroup path rather than expecting operators to derive them manually.

## What does not work

- A hidden staging directory fails with `EPERM` because bpffs reserves dots in names.
- A non-hidden staging directory fixes `mkdir`,
   but object pinning still fails with `EINVAL` on the affected kernel.
- Moving the pin to the bpffs root does not avoid `EINVAL`.
- Changing the pinned object from link to program or map does not avoid `EINVAL`.
- Mounting a fresh bpffs in a private mount namespace does not avoid `EINVAL`.
- Adding an explicit SELinux `context=` mount option does not avoid `EINVAL`.
- Relying on current host to emit `EINVAL` makes fallback tests silently take pin path after kernel recovery.
- Assuming test executable lives under `target/debug/deps` fails when current executable is under
  `target/debug/build/.../out`.
- A local VM could not be used for a fixed-kernel comparison because this host lacks `qemu-img`,
   `qemu-system-x86_64`,
   and
  `virsh`.
   The failure occurred before VM startup and does not provide kernel comparison evidence.

Disabling SELinux was not attempted and is not an acceptable production workaround.

## Upstream filing decision

No `.out-of-scope/` entry covers Linux,
 bpffs,
 BPF,
 or SELinux.

GitHub searches across open and closed issues and pull requests in `torvalds/linux` found no tracker item,
 but the Linux
kernel uses mailing-list review and an exact upstream thread already exists:

- [Regression report and source diagnosis](https://lore.kernel.org/all/akWdcp6P0FkNDzBk@google.com/)
- [Proposed fix](https://lists.openwall.net/linux-kernel/2026/07/30/2199)

The filing constraints resolve as follows:

1. **Upstream fault:**
    yes.
    The proposed patch names the regressing commit and corrects SELinux state initialization order.
2. **Upstream can fix it:**
    yes.
    The patch changes `security/selinux/hooks.c` without a userspace API change.
3. **Supported use case:**
    yes.
    `BPF_OBJ_PIN` is a documented UAPI command and bpffs object creation supports maps,
    programs,
   and links in `kernel/bpf/inode.c`.
4. **Contribution policy:**
    yes.
    `Documentation/process/submitting-patches.rst` documents external patch submission and no
   AI-assistance ban was found in the checked process and code-of-conduct documents.
5. **Likely resolution:**
    yes.
    The patch was merged into `selinux/stable-7.2` and mainline as commit
    `28254722a459938d97150d3b0712b81e06d0645e`.
6. **Compatible minimal fix:**
    yes.
    The upstream patch moves one existing guard before inode security initialization and
   includes the affected source location and regression commit.

Do not open a duplicate report.
The patch is merged,
and recovered `7.2.0-ogc6.1.fc44.x86_64` behavior matches it.
There is nothing further to file or comment upstream.
