# `std::fs::remove_dir_all` (Rust nightly 1.102.0, commit `ac8310ba6`) empties a directory another process renamed out of the tree it was handed, and still answers `Ok`

## Symptom

A recursive removal is given the path of a directory tree.
Another process,
in the middle of that removal,
renames one subdirectory of
the tree out to a path of its own,
which is what an atomic claim does.
The removal goes on unlinking that subdirectory's children through the
directory descriptor it already opened,
so the renamed directory ends up
empty,
and the removal reports success:

```text
remove_dir_all returned Ok
race: claimed commit ref survives when the removal opened the member first: False
race: entries left below the claimed directory: 0
```

Anything the moved directory's new owner writes after the last unlink stays,
so the directory is not reliably empty:
it holds whichever entries came after
the removal looked at that directory,
and nothing that was there before.

In this repository the shape is a commit that claims one finished shadow out
of the worktree's pool with one `rename`
(`package/git-policy/cli/src/native/shadow/shadow_pool.rs`,
`take`),
writes the commit's ref store into the claimed directory,
and then
has the wrapper's own removal of that pool run over the same tree
(`package/git-policy/cli/src/native/worktree/worktree_removal_clears_the_pool.rs`).
The commit is then read back from the shadow with
`git rev-parse --verify HEAD^{commit}`,
which fails,
so the wrapper reports a
failed commit while native Git succeeded.

## Root cause

`std::fs::remove_dir_all` on Linux uses the `openat` family:
it opens the directory it is deleting,
lists the open descriptor's entries,
and unlinks every entry through that same descriptor.

`library/std/src/sys/fs/unix.rs:2561-2578`,
from the toolchain this was
measured on (`~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/sys/fs/unix.rs`):

<!-- monochromatic-lint-disable markdown/semantic-line-breaks -- verbatim upstream source excerpt -->

```rust
fn remove_dir_all_recursive(parent_fd: Option<RawFd>, path: &CStr) -> io::Result<()> {
    // try opening as directory
    let fd = match openat_nofollow_dironly(parent_fd, path) {
        Err(err) if matches!(err.raw_os_error(), Some(libc::ENOTDIR | libc::ELOOP)) => {
            ...
        }
        result => result?,
    };
    // open the directory passing ownership of the fd
    let (dir, fd) = fdreaddir(fd)?;
```

and `unix.rs:2597-2603`,
which deletes children by name *below that descriptor*:

```rust
let result: io::Result<()> = try {
    match is_dir(&child) {
        Some(true) => {
            remove_dir_all_recursive(Some(fd), &child.name)?;
        }
        Some(false) => {
            cvt(unsafe { unlinkat(fd, child.name.as_ptr(), 0) })?;
        }
```

<!-- monochromatic-lint-enable markdown/semantic-line-breaks -- end verbatim upstream source excerpt -->

A descriptor names the directory object,
not its path,
so after
`rename(pool/member-x, claim/shadow)` the recursion still lists
`member-x`'s children from the descriptor and still unlinks them,
now inside
the directory that has become `claim/shadow`.
The final `unlinkat(..., AT_REMOVEDIR)` (`unix.rs:2619-2622`) then fails with
`ENOENT` for the entry that is no longer in the tree,
which `ignore_notfound`
swallows,
so the whole call answers `Ok`.

`library/std/src/fs.rs:3620-3622` states the contract that makes this a caller
problem,
not a crash:

<!-- monochromatic-lint-disable markdown/semantic-line-breaks -- verbatim upstream source excerpt -->

```rust
/// This function may return [`io::ErrorKind::DirectoryNotEmpty`] if the directory is concurrently
/// written into, which typically indicates some contents were removed but not all.
/// [`io::ErrorKind::NotFound`] is only returned if no removal occurs.
```

<!-- monochromatic-lint-enable markdown/semantic-line-breaks -- end verbatim upstream source excerpt -->

The removal is documented as best effort under concurrent writers and as
protecting only against symlink TOCTOU races;
nothing says a directory renamed
out of the tree is spared.

No hosted permalink is asserted for these two quotes:
the nightly the source
came from is not published at that commit hash,
so a link could not be
verified.
The quotes are from the installed toolchain's own copy of the file.

## Verification

Version under test:
`rustc 1.102.0-nightly (ac8310ba6 2026-11-09)`,
the toolchain this crate
builds with,
on btrfs.

Harness,
run in a throwaway folder under `${HOME}/temp/agent`:

```bash
# Save this block as run.sh in an empty scratch folder and run it with bash run.sh.
# The remover: exactly the call the wrapper's pool removal makes.
cat > remove_tree_with_std.rs <<'RS'
use std::path::PathBuf;

fn main() {
    let pool = std::env::args_os().nth(1).map(PathBuf::from).expect("pool argument");
    match std::fs::remove_dir_all(&pool) {
        Ok(()) => println!("remove_dir_all returned Ok"),
        Err(error) => {
            eprintln!("remove_dir_all returned Err: {error}");
            std::process::exit(1);
        }
    }
}
RS

# The pause: after the directory `PAUSE_OPEN_PATH` was opened, and before any child is read or removed.
cat > pause_after_member_open.c <<'C'
#define _GNU_SOURCE
#include <dlfcn.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef int (*openat64_fn)(int, const char *, int, ...);

int openat64(int directory, const char *path, int flags, ...) {
    static openat64_fn real_openat64;
    if (real_openat64 == NULL) {
        real_openat64 = (openat64_fn)dlsym(RTLD_NEXT, "openat64");
        if (real_openat64 == NULL) {
            _exit(90);
        }
    }
    mode_t mode = 0;
    if ((flags & O_CREAT) != 0) {
        va_list arguments;
        va_start(arguments, flags);
        mode = (mode_t)va_arg(arguments, int);
        va_end(arguments);
    }
    int fd = real_openat64(directory, path, flags, mode);
    if (fd < 0) {
        return fd;
    }
    const char *target = getenv("PAUSE_OPEN_PATH");
    if (target == NULL) {
        return fd;
    }
    char descriptor_path[64];
    snprintf(descriptor_path, sizeof(descriptor_path), "/proc/self/fd/%d", fd);
    char opened_path[PATH_MAX];
    ssize_t length = readlink(descriptor_path, opened_path, sizeof(opened_path) - 1);
    if (length < 0) {
        return fd;
    }
    opened_path[length] = '\0';
    if (strcmp(opened_path, target) != 0) {
        return fd;
    }
    const char *ready = getenv("PAUSE_READY_FILE");
    const char *release = getenv("PAUSE_RELEASE_FILE");
    if (ready == NULL || release == NULL) {
        _exit(91);
    }
    int marker = open(ready, O_WRONLY | O_CREAT | O_EXCL, 0600);
    if (marker >= 0) {
        close(marker);
    }
    for (int attempt = 0; attempt < 1000; ++attempt) {
        if (access(release, F_OK) == 0) {
            return fd;
        }
        usleep(10000);
    }
    _exit(92);
}
C

gcc -shared -fPIC -o pause_after_member_open.so pause_after_member_open.c -ldl
rustc remove_tree_with_std.rs -o remove_tree_with_std

# One finished member of a shadow pool, as the wrapper fills it.
OID=0123456789abcdef0123456789abcdef01234567
plant() {
  rm -rf "$1"
  mkdir -p "$1/member-ready/objects/info" "$1/member-ready/refs/heads"
  printf 'ref: refs/heads/main\n' > "$1/member-ready/HEAD"
  printf '[core]\n\tbare = true\n' > "$1/member-ready/config"
}

# Working case: the claim comes before the removal starts.
plant work-control/pool
mv work-control/pool/member-ready work-control/claimed
mkdir -p work-control/claimed/refs/heads
printf '%s\n' "$OID" > work-control/claimed/refs/heads/main
./remove_tree_with_std work-control/pool
printf 'control: claimed commit ref survives when the claim precedes the removal: %s\n' \
  "$(test -f work-control/claimed/refs/heads/main && echo True || echo False)"

# Failing case: the claim lands after the removal opened the member and before it unlinked its children.
# The paths the shim compares are the ones the kernel reports, so they carry no symlinked ancestor.
plant work-raced/pool
real=$(pwd -P)
PAUSE_OPEN_PATH="$real/work-raced/pool/member-ready" \
PAUSE_READY_FILE="$real/work-raced/opened" \
PAUSE_RELEASE_FILE="$real/work-raced/resume" \
LD_PRELOAD="$real/pause_after_member_open.so" \
./remove_tree_with_std work-raced/pool &
remover=$!
while [ ! -e work-raced/opened ]; do sleep 0.005; done
mv work-raced/pool/member-ready work-raced/claimed
mkdir -p work-raced/claimed/refs/heads
printf '%s\n' "$OID" > work-raced/claimed/refs/heads/main
touch work-raced/resume
wait "$remover"
printf 'race: claimed commit ref survives when the removal opened the member first: %s\n' \
  "$(test -f work-raced/claimed/refs/heads/main && echo True || echo False)"
printf 'race: entries left below the claimed directory: %s\n' \
  "$(find work-raced/claimed -mindepth 1 | wc -l)"
```

Output on 2026-11-11,
the block above run as written:

```text
remove_dir_all returned Ok
control: claimed commit ref survives when the claim precedes the removal: True
remove_dir_all returned Ok
race: claimed commit ref survives when the removal opened the member first: False
race: entries left below the claimed directory: 0
```

The pause is a legal scheduling preemption between two syscalls of one
function:
the removal opened the member at `unix.rs:2563`,
and the claim
renamed it before the removal's `unlinkat` at `unix.rs:2603`.

Patterns that work cleanly:

- A claim that completes before the removal is called:
  the claimed directory
  keeps its entries and the removal answers `Ok` (the control above).
- A tree whose root is renamed before the removal starts:
  the removal is
  handed a name no other process reads,
  so no claim can land inside the tree
  it deletes.

Patterns that fail:

- A claim between the removal's open of a subdirectory and its unlink of that
  subdirectory's children,
  which is the raced case above.
- The same claim with a second thread instead of a paused child:
  the window is
  microseconds wide,
  so a test that aims at it without a pause passes and
  fails run to run.

## Verified workarounds

1. Take the tree out of the name its users read,
   then remove it under that
   new name.
   One `rename(pool, pool-folder/removing-<random>)` before anything
   is deleted:

   ```rust
   let taken: PathBuf = folder.join(format!("removing-{}", random_uuid()?));
   match std::fs::rename(pool, taken.as_path()) {
       Ok(()) => { /* remove(taken) from here */ }
       Err(error) if error.kind() == std::io::ErrorKind::NotFound => return, // nothing to remove
       Err(error) => return Err(error),
   }
   ```

   Tradeoff:
   a removal that fails leaves the tree under `removing-<random>`,
   where no reader looks,
   so the failure has to be logged with that path (the
   wrapper does).
   A reader that lists the pool folder sees a name it must
   ignore.
   This is what
   `package/git-policy/cli/src/native/worktree/worktree_removal_clears_the_pool.rs`
   does,
   and the same shape is already used for a scratch tree that is about
   to be swept (`package/git-policy/cli/src/native/foundation/scratch_location.rs`)
   and for a log file that is being rotated
   (`package/git-policy/cli/src/native/foundation/log_lines_appended_without_a_lock.rs`).

2. Give the removal one subdirectory at a time,
   each renamed to a private name
   first (`rename(pool/member-x, pool/removing-x)`,
   then remove
   `removing-x`).
   The pool's own name stays readable while its entries go.

   Tradeoff:
   only the subdirectories are protected,
   not the tree root,
   so a
   reader that writes into the root while it is removed (a fill publishing a
   member) still loses that member,
   which is acceptable only when a lost
   member is not a lost commit.

## What does not work

- Removing the pool by its own name,
  which is what
  `worktree_removal_clears_the_pool.rs` did before this:
  a claim under that
  name moves a member out of the tree the removal was given,
  and the removal
  keeps unlinking its children.
- Comparing identities after the claim (the wrapper's `refusal` check):
  the
  emptied directory is empty after the removal,
  so the check either passes and
  the commit read-back fails,
  or the check fails and the commit makes another
  shadow while the first one's ref store is already gone.
- A lock around the pool (a claim waits for the removal):
  the owner's rule is
  "Agents should never wait on each other on commits or merges"
  (`AGENTS.md`,
  `NLK`),
  and the claim must not wait at all.
- `std::fs::remove_dir_all` on the pool folder with a `chmod 0500` on the
  pool:
  the removal empties the member's children and then fails to unlink the
  empty member,
  which leaves a claimable,
  emptied member in the pool.

## Upstream filing decision

Nothing is filed,
and no draft is kept:
the behavior is what the platform and
the standard library's contract allow,
and the fault is the caller's.

1. Is it really upstream's fault?
   No. `remove_dir_all` is documented as best
   effort under concurrent writers (`fs.rs:3620-3622`) and protects only
   against symlink TOCTOU races.
   A directory renamed out of the tree keeps its
   descriptor,
   and no unlink can be made atomic with a rename of the
   directory's own name.
2. Can upstream fix it?
   Not without taking a lock on every path component,
   which would make removal block writers and change the API's contract.
3. Are they supporting this use case?
   No:
   a caller that needs its entries
   preserved during removal must keep them out of the tree,
   and no
   documentation or example suggests `remove_dir_all` offers that.
4. Would the repo welcome our contribution?
   Not applicable:
   there is nothing
   to contribute,
   since the fix would be a new API rather than a defect fix.
5. Will they likely fix it?
   No signal either way,
   and the documented caveat
   reads as intended behavior.
6. Have we prototyped a minimal fix compatible with their architecture?
   No
   upstream fix is compatible;
   the fix belongs at the caller,
   and the caller's
   version is under "Verified workarounds".
