# AUR `opensnitch-ebpf-module-git` 1.8.0.r224: clang 23 aborts `llc -march=bpf` on host-target IR, so the build needs LLVM 22

Status at 2026-11-18T13:08:50-05:00:
agent-written from a live switch to the AUR `opensnitch-git` packages on a CachyOS desktop;
not reviewed or approved by the owner.
The workaround lives in a local commit of the AUR recipe and is not pushed anywhere,
and nothing has been filed upstream.
The facts here were checked on this host on 2026-11-18 unless a section says otherwise.

The host runs kernel `7.3.5-1-cachyos`,
`clang` and `llvm` `23.1.5-2`,
and `clang22` and `llvm22` `22.1.8`.
The AUR packages are `opensnitch-git` and `opensnitch-ebpf-module-git`,
both `1.8.0.r224.a1353848-1`.
That `pkgver` counts the commits since the tag `v1.8.0`
and ends at upstream commit `a1353848ba1b660320e90cefea782c3fba272c00`
(committed 2026-07-27T01:33:38+02:00),
whose daemon reports `1.9.0` ([daemon/core/version.go:6][version-go]).

Source citations are pinned to that commit,
to `b404c4c6316760fa7bc415509d3f8d747f7dc9cc` (the commit the annotated tag `v1.8.0` points to),
and to the LLVM commits named in "Root cause".

## Symptom

Building the AUR `opensnitch-ebpf-module-git` with clang 23 stops at the `llc` step.
The first object that reaches it aborts `make`
(the Makefile line number is that of the Makefile after the AUR recipe's patch):

```text
llc -march=bpf -mcpu=generic -filetype=obj -o opensnitch.o opensnitch.bc
LLVM ERROR: Can't create a MachineFunction using a Module with a Target-incompatible DataLayout attached
  Target DataLayout: e-m:e-p:64:64-i64:64-i128:128-n32:64-S128
  Module DataLayout: e-m:e-p270:32:32-p271:32:32-p272:64:64-p:64:64-i64:64-i128:128-n32:64-S128
...
make: *** [Makefile:104: opensnitch.o] Aborted (core dumped)
```

The same build with LLVM 22 first in `PATH` succeeds (see "Verification").

On a current kernel more failures sit behind that one.
Which one shows depends on the source and the flags:

1.  The `v1.8.0` tag with kernel 6.19 or newer headers fails in the compile step with
    `error: no member named 'slock' in 'socket_lock_t'`.
2.  Adding `-fms-extensions` moves the stop to
    `asm/ftrace.h:9:3: error: Compiler does not support fentry?`.
3.  Adding `-DCC_USING_FENTRY` as well reaches `llc`,
    and the `LLVM ERROR` abort follows.

## Root cause

### The Makefile gives host-target IR to the BPF backend

`ebpf_prog/Makefile` names `clang` and `llc`
([ebpf_prog/Makefile:18-19][mk-master-cc]):

```make
# ebpf_prog/Makefile at a1353848ba1b660320e90cefea782c3fba272c00
CC = clang
LLC ?= llc
```

The compile flags end in `-emit-llvm`,
and the pattern rules turn each source into IR and then into a BPF object
([ebpf_prog/Makefile:93-101][mk-master-rules]):

```make
# ebpf_prog/Makefile at a1353848ba1b660320e90cefea782c3fba272c00
	-g -O2 -emit-llvm

all: $(BIN)

%.bc: %.c
	$(CC) $(CFLAGS) -c $<

%.o: %.bc
	$(LLC) -march=bpf -mcpu=generic -filetype=obj -o $@ $<
```

No flag in that Makefile sets a target triple:
the only match for `target` is `-D__TARGET_ARCH_$(ARCH)`.
So clang writes IR for the host triple,
with the x86-64 data layout,
and `llc -march=bpf` then builds a BPF target machine whose data layout differs.
The two layouts are the `Module DataLayout` and `Target DataLayout` lines in "Symptom".
The `v1.8.0` Makefile has the same two rules
([ebpf_prog/Makefile:62-70][mk-v180-rules]).

LLVM's reply on the LLVM report names exactly this,
after looking at the OpenSnitch Makefile:
"it seems to be calling `clang` without any `--target`,
so this is producing IR for the host triple.
And then that gets feds to `llc -march=bpf`"
(the reply's spelling),
followed by "If that's indeed what it does,
then that is certainly not supposed to work"
([llvm-project#228348 reply][llvm-228348-reply]).
That reply is conditional on the Makefile reading,
which the lines here confirm.
The first reply on that report had called it "user error (using incorrect data layout for the target)"
([first reply][llvm-228348-first]),
before the Makefile was looked at.
A commenter on the OpenSnitch report reached the same reading:
"LLVM's diagnosis is correct:
`ebpf_prog/Makefile` compiles with clang for the host (`-emit-llvm`),
then runs `llc -march=bpf` on the result (line 101)"
([opensnitch#1686 comment][issue-1686-diagnosis]).
That commenter has no association with the repository (`author_association` is `NONE`).

### Why LLVM 22 accepted it and LLVM 23 does not

BPF has no data-layout override,
so the target machine's default check is exact equality
([llvm/include/llvm/Target/TargetMachine.h:215-217][llvm-tm-compat]):

```cpp
// llvm/include/llvm/Target/TargetMachine.h at llvmorg-23.1.5
  bool isCompatibleDataLayout(const DataLayout &Candidate) const {
    return DL == Candidate;
  }
```

A search of `BPFTargetMachine.h` at the same commit finds no `compatible` override
([llvm/lib/Target/BPF/BPFTargetMachine.h][llvm-bpf-tm]).

In LLVM 22.1.8 the caller of that check is an `assert`
([llvm/lib/CodeGen/MachineFunction.cpp:242-244][llvm22-check]):

```cpp
// llvm/lib/CodeGen/MachineFunction.cpp at llvmorg-22.1.8
  assert(Target.isCompatibleDataLayout(getDataLayout()) &&
         "Can't create a MachineFunction using a Module with a "
         "Target-incompatible DataLayout attached\n");
```

The `llc` of `llvm22` on this host reports `Optimized build.` and not a build with assertions,
so the assert is compiled out and the mismatch passes.
In LLVM 23.1.5 the same condition is a fatal error that always runs
([llvm/lib/CodeGen/MachineFunction.cpp:249-256][llvm23-check]):

```cpp
// llvm/lib/CodeGen/MachineFunction.cpp at llvmorg-23.1.5
  if (!Target.isCompatibleDataLayout(getDataLayout())) {
    report_fatal_error(
        formatv("Can't create a MachineFunction using a Module with a "
                "Target-incompatible DataLayout attached\n  Target "
                "DataLayout: {0}\n  Module DataLayout: {1}\n",
                Target.createDataLayout().getStringRepresentation(),
                getDataLayout().getStringRepresentation()));
  }
```

The tag `llvmorg-23.1.6` (commit `5b10d2d74929eae78912771222c9855af97496f4`)
and `main` at commit `acfb7759cf64f4e81fbbbdaccf2d40aa22c6b42b` keep the same fatal error
([23.1.6:249-256][llvm23-6-check],
[main:237-244][llvm-main-check]).
The reporter of the LLVM issue says the same sources built before the distribution moved from LLVM 22 to 23
([llvm-project#228348][llvm-228348]).
Which LLVM commit turned the assert into a fatal error was not looked up.

### More breaks sit behind it

Kernel 6.19 headers use anonymous tagged structs,
which clang accepts only with `-fms-extensions`.
Master's Makefile adds that flag for such kernels
([ebpf_prog/Makefile:56-59][mk-master-flags]):

```make
# ebpf_prog/Makefile at a1353848ba1b660320e90cefea782c3fba272c00
ifeq ($(KERNEL_6_19_CHECK),1)
	EXTRA_FLAGS += -Wno-microsoft-anon-tag -fms-extensions
endif
```

That comes from upstream commit [`614537c9`][commit-614537c9]
("fix kernel 6.19 build,
add some hardening options",
committed 2026-02-27T08:42:27+01:00),
merged as [PR 1554][pr-1554] on 2026-03-01.
`git merge-base --is-ancestor` reports `614537c9` as not in `v1.8.0` and in `a1353848`,
and `v1.8.0` is the latest release,
so the release lacks the flag.
[opensnitch#1537][issue-1537] tracks the report and is open.
The Arch packaging merge request opened on 2026-11-17 states the same:
"Upstream fixed this in evilsocket/opensnitch@614537c9 (2026-02-27),
which isn't in a release yet (the latest release is v1.8.0)"
([merge request 4][arch-mr-4-614]).

Kernel 7.2 and 7.3.5 headers also need `-DCC_USING_FENTRY`.
Neither Makefile defines it (`git grep CC_USING_FENTRY` over `ebpf_prog` at `a1353848` finds nothing).
[opensnitch#1656][issue-1656] reports `Compiler does not support fentry?` on 7.2 headers
([the error][issue-1656-fentry]),
and was closed with a pointer to a fix in the reporter's own distribution.
The AUR recipe's `0001-Build-with-Archlinux.patch` adds `-DCC_USING_FENTRY` to the compile flags
([the patch][aur-patch]).

### A tempting fix that is wrong: `--target=bpf` alone

A commenter on the OpenSnitch report proposed "compile with `clang --target=bpf` and drop the `llc` step"
([comment][issue-1686-target]).
The reporter tried it and replied "clang still crashes"
([comment][issue-1686-quicktest]),
and the commenter conceded that "`--target=bpf` alone isn't enough"
([comment][issue-1686-notenough]).
The test in "Verification" shows why:
the Makefile includes the x86 kernel headers,
which contain x86 inline assembly and word-size checks that a BPF target rejects,
and it does so with clang 22 as well as clang 23.
The fix LLVM points at therefore needs a source rework,
for example BPF CO-RE with a `vmlinux.h` and no architecture headers,
and not a flag change.
A contributor's pull request took that route,
and its author wrote:
"`-emit-llvm` - was for two-step build (clang -> LLVM IR -> ll -> BPF),
replaced by -target bpf direct compilation"
([PR 1422 comment][pr-1422-comment]).
It was closed unmerged on 2026-08-20.

## Verification

### Versions under test

- Host:
  CachyOS,
  kernel `7.3.5-1-cachyos`,
  headers from `linux-cachyos-headers 7.3.5-1`.
- Toolchains:
  `clang` and `llc` `23.1.5` (`/usr/bin`),
  and `clang` and `llc` `22.1.8` (`/usr/lib/llvm22/bin`).
- Sources:
  upstream `a1353848ba1b660320e90cefea782c3fba272c00` with the AUR recipe's `0001` patch,
  and `b404c4c6316760fa7bc415509d3f8d747f7dc9cc` (`v1.8.0`).
- Every build ran from a `git archive` copy in a fresh temporary directory,
  with output to that directory only.

### Harness

The smallest reproduction needs no OpenSnitch source:
one function through host-target IR and then the BPF backend.

```bash
# minimal.sh: run in an empty directory; prefix PATH to pick the toolchain
printf 'int add_one(int x) { return x + 1; }\n' > t.c
clang -O2 -emit-llvm -c t.c -o t.bc && llc -march=bpf -mcpu=generic -filetype=obj -o t.o t.bc
echo "exit status: $?"
```

With `clang` and `llc` 23.1.5 that prints the `LLVM ERROR` line from "Symptom" and `exit status: 134`.
With `PATH=/usr/lib/llvm22/bin:$PATH` it prints `exit status: 0`.

The OpenSnitch builds use this script
(`OPENSNITCH_CLONE` is a clone of `evilsocket/opensnitch`,
`AUR_PATCH` is the recipe's patch file):

```bash
# matrix.sh <ref> <apply-aur-patch: 0 or 1> <extra-flags> <llvm-bin-dir or ->
# Builds the eBPF objects from a git archive copy of a clone, in a fresh temporary directory.
set -u
ulimit -c 0
ref=$1; aurpatch=$2; flags=$3; llvmbin=$4
clone=${OPENSNITCH_CLONE:?path of a clone of evilsocket/opensnitch}
aur_patch=${AUR_PATCH:?path of 0001-Build-with-Archlinux.patch from the AUR recipe}
work=$(mktemp --directory)
git -C "$clone" archive "$ref" ebpf_prog | tar -x -C "$work"
if [ "$aurpatch" = 1 ]; then patch -s -d "$work/ebpf_prog" Makefile < "$aur_patch" || exit 9; fi
kdir=/usr/lib/modules/$(uname -r)/build
[ "$llvmbin" = - ] || export PATH="$llvmbin:$PATH"
echo "## $ref aurpatch=$aurpatch flags='$flags' $(clang --version | head -1)"
if [ -n "$flags" ]; then
  make -C "$work/ebpf_prog" all KERNEL_DIR="$kdir" KERNEL_HEADERS="$kdir" ARCH=x86 EXTRA_FLAGS="$flags" >"$work/out.txt" 2>&1
else
  make -C "$work/ebpf_prog" all KERNEL_DIR="$kdir" KERNEL_HEADERS="$kdir" ARCH=x86 >"$work/out.txt" 2>&1
fi
echo "make exit status: $?"
grep -E 'error:|LLVM ERROR|DataLayout|Aborted' "$work/out.txt" | cut -c1-200 | sort | uniq -c | sort -rn | head -6
ls "$work/ebpf_prog"/*.o 2>/dev/null | xargs -r -n1 basename | tr '\n' ' '; echo
```

The script's path variables replace the literal paths of the original runs.
A run of it on 2026-11-18 reproduced a pass and a fail,
shown here with the build-environment lines left out:

```text
## a1353848ba1b660320e90cefea782c3fba272c00 aurpatch=1 flags='' clang version 22.1.8
make exit status: 0
opensnitch-dns.o opensnitch.o opensnitch-procs.o

## b404c4c6316760fa7bc415509d3f8d747f7dc9cc aurpatch=0 flags='-fms-extensions -Wno-microsoft-anon-tag -DCC_USING_FENTRY' clang version 23.1.5
make exit status: 2
      1   Target DataLayout: e-m:e-p:64:64-i64:64-i128:128-n32:64-S128
      1   Module DataLayout: e-m:e-p270:32:32-p271:32:32-p272:64:64-p:64:64-i64:64-i128:128-n32:64-S128
      1 make: *** [Makefile:70: opensnitch.o] Aborted (core dumped)
      1 LLVM ERROR: Can't create a MachineFunction using a Module with a Target-incompatible DataLayout attached
```

### Failing catalog, by variant

#### Host-target IR rejected by the clang 23 `llc`

- The minimal file from the harness,
  with `clang` and `llc` 23.1.5:
  `exit status: 134`.
- `a1353848` with the AUR patch,
  default `PATH`:
  abort at `opensnitch.o`,
  `Makefile:104`.
- `v1.8.0` with `-fms-extensions -Wno-microsoft-anon-tag -DCC_USING_FENTRY`:
  abort at `opensnitch.o`,
  `Makefile:70`.

#### Anonymous tagged structs without `-fms-extensions`

`v1.8.0` with no extra flags,
clang 23.1.5,
exit status 2:

```text
<kdir>/include/net/sock.h:1706:38: error: no member named 'slock' in 'socket_lock_t'
<kdir>/include/net/sock.h:1813:21: error: no member named 'owned' in 'socket_lock_t'
```

`<kdir>` is `/usr/lib/modules/7.3.5-1-cachyos/build`.

#### Missing `CC_USING_FENTRY`

`v1.8.0` with `-fms-extensions -Wno-microsoft-anon-tag` only,
clang 23.1.5,
exit status 2:

```text
<kdir>/arch/x86/include/asm/ftrace.h:9:3: error: Compiler does not support fentry?
make: *** [Makefile:67: opensnitch.bc] Error 1
```

#### Direct `--target=bpf` against the x86 headers

A copy of `a1353848` with the AUR patch,
built with `make -k all CC='clang --target=bpf'` and the kernel variables as in the script.
Both toolchains fail in the compile step with the same kinds of error:

```text
<kdir>/include/asm-generic/bitsperlong.h:27:16: error: static assertion failed due to requirement 'sizeof(long) * 8 == 32': Inconsistent word size. Check asm/bitsperlong.h
<kdir>/arch/x86/include/asm/div64.h:110:23: error: invalid output constraint '+a' in asm
```

Other lines in the same output include `invalid output constraint '=a' in asm` and `'=@ccc'`.
The reporter's "clang still crashes" in the OpenSnitch thread is a different symptom from these compile errors,
and nothing here reproduces a crash with `--target=bpf`.

### Working catalog

- The minimal file with `PATH=/usr/lib/llvm22/bin:$PATH`:
  `exit status: 0`.
- `a1353848` with the AUR patch and LLVM 22 first in `PATH`:
  `make exit status: 0`,
  objects `opensnitch.o`,
  `opensnitch-procs.o` and `opensnitch-dns.o`.
- `v1.8.0` with `-fms-extensions -Wno-microsoft-anon-tag -DCC_USING_FENTRY` and LLVM 22 first in `PATH`:
  `make exit status: 0`,
  the same objects.
- Section check with `llvm-readelf -S`:
  18 `kprobe`,
  `kretprobe`,
  `tracepoint`,
  `uprobe` and `uretprobe` sections in each rebuilt tree.
  The sorted section names of the installed objects are identical to those of the objects in the cached package
  `opensnitch-1.7.2-4.1-x86_64_v4.pkg.tar.zst` (18 names each,
  `diff` prints nothing).

### Installed module at run time

The installed module is the one the AUR recipe built with LLVM 22 on 2026-11-18 (`build.log` ends at 12:34:41).
The cutover session found these under the running 1.9.0 daemon:

- `bpftool prog list` shows 17 attached programs,
  among them `kprobe__tcp_v4_connect`,
  `tracepoint__syscalls_sys_enter_execve` and `uprobe__getaddrinfo`.
- The daemon logged no eBPF warning.

This write-up repeated only the second check:
a search of `/var/log/opensnitchd.log` for `WAR` or `ERR` lines that mention eBPF finds none
(the log is at debug level and carries `[eBPF exec event]` lines).
It did not repeat the `bpftool` listing,
which needs root.
The objects have 18 probe sections and the listing has 17 programs;
which section has no attached program was not identified.

### Not verified

- A reboot on the AUR packages.
- A newer clang 23.x.
  The `llvmorg-23.1.6` and `main` sources keep the fatal error (see "Root cause"),
  but no 23.1.6 build was run,
  and the local pacman sync databases (`extra` and the CachyOS repository)
  still list `23.1.5` at the time of writing.
- Loading the `v1.8.0` objects.
  Only the objects of the master recipe were loaded.
- Which of the 18 probe sections is the one without an attached program.
- `make CC=... LLC=...` with Arch's LLVM 22 names (see "Verified workarounds").
- The Arch GitLab and AUR links in a browser.
  Both hosts answer an automated browser with an Anubis access-denied page,
  and no way around it was tried.
  Their quoted text was checked over plain HTTP instead:
  the merge request's API JSON,
  the AUR page HTML,
  and the AUR cgit raw patch at the pinned commit.
  The two `#:~:text=` links into the merge request were therefore never loaded.
- The highlight of the `#:~:text=` link into opensnitch#1656.
  `agent-browser` confirmed that the quoted text is on the rendered page,
  not that the browser highlights it.

## Verified workarounds

### LLVM 22 first in `PATH`

```bash
# build step, run in the ebpf_prog directory of a checkout
kdir="/usr/lib/modules/$(uname -r)/build"
PATH="/usr/lib/llvm22/bin:${PATH}" KERNEL_DIR="${kdir}" KERNEL_HEADERS="${kdir}" ARCH=x86 make all
```

The Arch packaging merge request for the `opensnitch` package takes the same approach:
"This MR builds the eBPF step with LLVM 22"
([merge request 4][arch-mr-4-llvm22]).
It was opened on 2026-11-17 and is still open,
with `-fms-extensions -Wno-microsoft-anon-tag` passed through `EXTRA_FLAGS`.

Tradeoffs:

- It needs `clang22` and `llvm22`,
  which Arch ships beside LLVM 23 for now.
  The build breaks again when those packages go away,
  and then the Makefile needs the source rework.
- The objects come from an older toolchain than the distribution's.
  Their probe sections match those of the 1.7.2 package,
  and the master objects load.
- `PATH` is changed for that one command only,
  so nothing else on the system sees LLVM 22.

The OpenSnitch report's suggestion is `make CC=clang-22 LLC=llc-22`
([comment][issue-1686-diagnosis]),
which its author did not test with clang 23.
On this host `llc-22` is linked into `/usr/bin` by `llvm22`,
but `clang-22` is not on `PATH`,
so that spelling does not work as written on Arch.
`CC` is a plain assignment in the Makefile,
and a command-line `CC=` overrides it
(the `--target=bpf` test in "Verification" relied on that).
The `PATH` form is the one verified here.

### The AUR recipe change

The change is local commit `b809be8` in `/home/user/aur/opensnitch-ebpf-module-git`
(comments omitted here):

```diff
--- a/PKGBUILD
+++ b/PKGBUILD
@@ makedepends @@
-makedepends=('git' 'clang' 'llvm' 'libelf'
-             'bc' 'rsync' 'linux-headers' 'coreutils')
+makedepends=('git' 'clang22' 'llvm22' 'libelf'
+             'bc' 'rsync' 'linux-cachyos-headers' 'coreutils')
@@ build() @@
-  KERNEL_DIR="/usr/lib/modules/$(uname -r)/build" KERNEL_HEADERS="${KERNEL_DIR}" ARCH="${arch}" make all
+  local kdir="/usr/lib/modules/$(uname -r)/build"
+  PATH="/usr/lib/llvm22/bin:${PATH}" KERNEL_DIR="${kdir}" KERNEL_HEADERS="${kdir}" ARCH="${arch}" make all
```

Tradeoffs:

- `linux-cachyos-headers` provides `LINUX-HEADERS` and not `linux-headers`.
  With `linux-headers` left in `makedepends`,
  `makepkg -s` would install Arch's headers for a kernel that is not installed.
  The new dependency ties the recipe to the CachyOS kernel;
  another kernel needs its own headers package there.
- The module builds against `/usr/lib/modules/$(uname -r)/build`,
  the running kernel's headers.
  It cannot be built while that directory is missing,
  for example after a kernel upgrade without a reboot.
  See [the kernel-upgrade doc][doc-kernel-modules] for that state and its workaround.
  The module also needs a rebuild after each kernel update.
- The `local kdir` variable is cosmetic.
  The original `KERNEL_DIR=x KERNEL_HEADERS="${KERNEL_DIR}" make` is fine,
  because bash expands prefix assignments from left to right
  (see "What does not work").

### Building the `v1.8.0` tag

The tag needs all three flags besides LLVM 22:

```bash
# in the ebpf_prog directory of a v1.8.0 checkout
kdir="/usr/lib/modules/$(uname -r)/build"
PATH="/usr/lib/llvm22/bin:${PATH}" KERNEL_DIR="${kdir}" KERNEL_HEADERS="${kdir}" ARCH=x86 \
  EXTRA_FLAGS="-fms-extensions -Wno-microsoft-anon-tag -DCC_USING_FENTRY" make all
```

Tradeoffs:

- Setting `EXTRA_FLAGS` on the command line replaces the Makefile's own `EXTRA_FLAGS`.
  For `v1.8.0` that value is empty on x86-64,
  so nothing is lost there.
- `-DCC_USING_FENTRY` is a manual claim that the compiler supports fentry,
  which the kernel build normally decides.
  The OpenSnitch programs are kprobe,
  tracepoint and uprobe programs,
  and the objects have the same sections as the 1.7.2 objects,
  but the `v1.8.0` objects were not loaded.
- `ARCH=x86` is set by hand here.
  The section "The recipe's `ARCH` is the first element of the `arch` array"
  covers what the recipe's own `ARCH="${arch}"` does.

### The recipe's `ARCH` is the first element of the `arch` array

The recipe passes `ARCH="${arch}"`,
and `arch` is the PKGBUILD array `('i686' 'x86_64' 'armv6h' 'armv7h' 'aarch64')`.
A bash check printed `i686` for `echo "${arch}"`,
and the Makefile maps `i686` to `x86` ([ebpf_prog/Makefile:26][mk-master-i686]).
That is right on x86-64 by accident,
and it would select the x86 include paths on the other architectures the recipe lists.
It is unrelated to the clang 23 failure and was not changed.

## What does not work

- `--target=bpf` alone,
  or in place of the `llc` step:
  fails on the x86 headers with clang 22 and 23 (see "Verification").
- `-fms-extensions` alone on `v1.8.0`:
  the build stops at `Compiler does not support fentry?`.
- Waiting for a clang 23 point release:
  `llvmorg-23.1.6` and `main` keep the unconditional fatal error.
  No 23.1.6 build was run.
- Keeping `linux-headers` in `makedepends` on CachyOS (see "The AUR recipe change").
- Treating `KERNEL_DIR=x KERNEL_HEADERS="${KERNEL_DIR}" make` as a bug.
  The bash check shows both variables set in the command's environment:

  ```text
  B=x
  A=x
  ```

  It ran `A=x B="${A}" env | grep -E "^(A|B)="` after `unset A B`.
- The `--target=bpf` direction with a `vmlinux.h`:
  [PR 1422][pr-1422] closed unmerged.
  The last comment on it,
  from a collaborator (`author_association` `COLLABORATOR`),
  lists kernels where the CO-RE objects failed
  and says "we lose support for older kernels,
  but in return we gain better compatibility with newer ones"
  ([comment][pr-1422-last]).
  A collaborator comment of 2025-09-11 says
  "Unless the PR adds new interesting functionality or solves an issue,
  I'm usually reluctant to change things just because"
  ([comment][pr-1422-reluctant]).

## Other gaps in the AUR `opensnitch-git` package

### `tasks.json` is not installed

The recipe creates `/etc/opensnitchd/tasks` with `install -d` and installs no file into it.
The daemon then warns `[tasks] LoadTaskFile ... no such file` (seen during the cutover).
Upstream's own RPM spec installs `daemon/data/tasks/tasks.json` with mode 600
([utils/packaging/daemon/rpm/opensnitch.spec:85-89][spec-tasks]),
and that file holds `{"tasks": []}`
([daemon/data/tasks/tasks.json][tasks-json]).
It was installed by hand here with mode 600,
and the file on the host is 20 bytes.

### The eBPF module is an optdepend and the config asks for it

The recipe lists `opensnitch-ebpf-module-git` under `optdepends`
and leaves its `-process-monitor-method` edits commented out,
with a note that `default-config.json` "should use proc unless changed".
Upstream's `default-config.json` already says `"ProcMonitorMethod": "ebpf"`
([daemon/data/default-config.json:21][default-config]).
An install without the module therefore keeps a config that asks for it.
What the daemon does in that state was not tested here.

### Replacing the repo package turns edited configs into `.pacsave` files

Replacing the repo package `opensnitch 1.7.2-4.1` with `opensnitch-git`
moved the modified `default-config.json` and `system-fw.json` to `.pacsave` files
and installed the stock copies of the new package.
The stock `system-fw.json` changed shape.
Version 1.7.2 uses the tables `filter`,
`mangle` and `nat`
([daemon/system-fw.json:42-43][sysfw-172]),
with chains named `input`,
`forward`,
`output` and so on.
Master uses one `inet opensnitch` table
with chains `filter_input`,
`filter_forward`,
`mangle_output` and so on
([daemon/data/system-fw.json:42-43][sysfw-master]).

The daemon hardcodes the new names for its own rules.
The DNS rule goes on `opensnitch`/`filter_input`
and the interception rule on `mangle_output`
([daemon/firewall/nftables/chains.go:115-137][chains-go];
[daemon/firewall/nftables/exprs/enums.go:5-7][enums-go]):

```go
// daemon/firewall/nftables/exprs/enums.go
	TABLE_OPENSNITCH     = "opensnitch"
	CHAIN_FILTER_INPUT   = "filter_input"
	CHAIN_MANGLE_OUTPUT  = "mangle_output"
```

A rule file written for the 1.7.2 table and chain names must be remapped.
The remaining differences in `daemon/rule/` were read only in part:
`FindFirstMatch` still walks the active rules in sorted name order and returns the first match,
now over an atomically swapped snapshot
([daemon/rule/loader.go:520-541 at master][findfirst-master];
[daemon/rule/loader.go:496-515 at 1.7.2][findfirst-172]).
The operator code gained a `range` type,
and type `network` is allowed only with operand `dest.network`
([daemon/rule/operator.go:185-188][operator-network];
[1.7.2 had no operand check there][operator-network-172]).
The list-matching code was refactored and was not audited.

## How the switch was done

The cutover from the repo package to the AUR packages used this order:

1.  ufw was enabled as cover while OpenSnitch's own firewall was replaced.
2.  One pacman transaction installed both AUR packages,
    with the prompts answered by `yes |`.
    `pacman.log` shows `opensnitch (1.7.2-4.1)` removed at 2026-11-18T12:38:18-0500,
    with `opensnitch-git` and `opensnitch-ebpf-module-git` installed in the same second.
3.  The configs were regenerated from the new stock file,
    keyed on chain hook and type,
    and written in place with `tee`.
4.  The unit override was derived again.
    The new `ExecStart` is `/usr/bin/opensnitchd`,
    with no `-rules-path`.
5.  The daemon and the UI were restarted and ufw was disabled again.

Checks afterwards (2026-11-18):

- The `git://` reject rule refused a connection in 6 ms.
- A netns and veth inbound test passed:
  the allowed port connected and the closed port was dropped and logged.
- IPv6 works.
- The 1.9.0 UI connected to the daemon.

The system-firewall behaviours recorded for 1.7.2 are in [that version's doc][doc-sysfw-quirks].
They were not rechecked on master here.

## Upstream filing decision

Nothing was filed.
The decision here is a record for a future session.
The task that produced this doc was documentation only,
so no fix was prototyped.

### Out-of-scope list and existing reports

- `.out-of-scope/` was read on 2026-11-18.
  No entry covers OpenSnitch,
  LLVM,
  BPF or eBPF
  (the only match for `clang` is an unrelated Rust-workspace entry).
- Duplicate search on 2026-11-18T12:46:18-05:00 over `evilsocket/opensnitch`.
  `gh search issues` used the terms `clang 23`,
  `llc bpf`,
  `DataLayout`,
  `fentry`,
  `fms-extensions`,
  `ebpf build clang` and `CO-RE vmlinux`.
  `gh search prs` used `ebpf_prog`,
  `clang`,
  `llc`,
  `fentry` and `kernel 6.19`.
  Both matched titles and bodies in open and closed state.
  The matching report is [opensnitch#1686][issue-1686] (open,
  opened 2026-10-05).
  Related:
  [#1537][issue-1537] (open),
  [#1656][issue-1656] (closed),
  [PR 1422][pr-1422] (closed unmerged) and [PR 1554][pr-1554] (merged).
  #1686 was read in full.
  #1537,
  #1656 and PR 1422 were read with their long logs and comments truncated.
- Work in progress,
  checked at 2026-11-18T12:55:34-05:00:
  no open pull request touches `ebpf_prog` (every open pull request was listed with its files),
  #1686 has no assignee and no claim of work,
  the last commit on `ebpf_prog/Makefile` is `614537c9` of 2026-02-27,
  and no branch name matches the symptom.
  The Arch packaging [merge request 4][arch-mr-4] (opened 2026-11-17T20:48:40Z,
  open)
  carries the LLVM 22 and `-fms-extensions` changes for the `opensnitch` package.
  Its JSON reports `user_notes_count` 0,
  but its notes endpoint answered `401 Unauthorized` without a login,
  so comments on it were not read.
- The AUR pages were fetched on 2026-11-18 and only the first page of comments on each was scanned.
  That of [opensnitch-ebpf-module-git][aur-ebpf] reports the same LLVM error
  and the fentry error on 7.2.2 headers;
  that of [opensnitch-git][aur-git] mentioned neither.
  The snippets seen add nothing beyond the reports linked in this doc.
- Trust and blocklist search,
  against commit `a1353848` and on 2026-11-18:
  `.github` holds `FUNDING.yml`,
  `ISSUE_TEMPLATE` and the workflows
  `build_ebpf_modules.yml`,
  `generic_validations.yml` and `go.yml`;
  the root tree has no `CONTRIBUTING`,
  `VOUCHED` or similar file;
  `gh search code --repo evilsocket/opensnitch vouch` returned nothing;
  the README mentions contributors only as a link.
  No AI-assistance policy was found.
  Check the live repository again on the day of posting.

### The six constraints

1.  Is it upstream's fault?
    Yes for the Makefile:
    it builds IR for the host and gives it to a BPF-only backend,
    and LLVM said that is "certainly not supposed to work" if the Makefile does so,
    which it does.
    The kernel-header breaks come from kernel changes,
    which upstream partly fixed on master (`614537c9`)
    and which the fentry define still lacks.
2.  Can upstream fix it?
    Yes.
    The fix is a source rework (CO-RE and `vmlinux.h`),
    which [PR 1422][pr-1422] shows can be done;
    the x86 headers cannot be compiled for the BPF target as they are.
3.  Is the use case supported?
    Yes.
    The Makefile is the shipped build,
    the repository's [workflow][ci-workflow] builds the modules on `ubuntu-22.04`
    with the distribution's `clang` and `llvm`,
    and `614537c9` added support for newer kernels.
4.  Would the repository welcome the contribution?
    No rule against it was found:
    no contributing file,
    no pull-request template,
    no trust list,
    no AI policy.
    The issue form [.github/ISSUE_TEMPLATE/bug_report.md][issue-template] asks for version,
    OS,
    window manager and kernel.
    Maintainers merge outside pull requests ([PR 1554][pr-1554] has `author_association` `CONTRIBUTOR`),
    and a collaborator was reluctant about a rework of this Makefile ([comment][pr-1422-reluctant]).
5.  Will they likely fix it?
    Unknown,
    which does not fail the constraint.
    [#1686][issue-1686] has had no maintainer comment since 2026-10-05,
    and every comment on it has `author_association` `NONE`.
6.  Is there a prototype?
    No.
    A prototype of a Makefile rework was not attempted in this documentation-only task.

The draft comment therefore stays "do not file as-is".

### Readiness check

No item was completed for filing:

1.  Latest release and other maintained lines:
    the reproduction ran on `v1.8.0` and `a1353848` with clang 23.1.5.
    Other lines and clang 23.1.6 were not run.
2.  Prior art:
    the threads and pull requests named in this section were read in part.
    Merged and closed pull requests were not searched for the LLVM symbols.
3.  Intake:
    the issue form was read.
    The draft comment needs no form.
4.  Automation:
    `generic_validations.yml` and `go.yml` were not read.
5.  Day-of re-check of the searches,
    the work-in-progress check and the trust search:
    not run.
6.  The draft's reproduction run as written by the poster:
    not run.
7.  Reproduction repository with a CI workflow:
    not run.
8.  Verdict:
    no recommendation to file.

### Risk to the poster

Filing is the poster's decision.
Some repositories run trust or blocklist systems that close or lock reports from unvouched accounts,
and a denouncement can follow a person to other projects.
An agent-written report that a maintainer finds low-effort is the case those systems exist for,
and a reproduction does not remove that risk.
No such system was found in this repository on 2026-11-18,
and no recommendation to file is made.

### Draft comment on opensnitch#1686

Content absent from the thread:
the LLVM-side cause,
why `--target=bpf` fails whichever clang is used,
and a verified build on current headers.
The LLVM issue gets no comment:
an LLVM contributor has already said the usage is not supposed to work,
and the finding here concerns OpenSnitch's Makefile.
A comment on the Arch merge request is not drafted:
its description lists `-fms-extensions` and LLVM 22 and not `-DCC_USING_FENTRY`,
and whether the `1.7.2` sources it packages need that define was not tested.

The draft is marked do not file as-is:
it is an unreviewed agent draft,
and no prototype of a fix backs it.

~~~md
The crash is not specific to clang 23.1.1.
In LLVM 22.1.8 the data-layout check in `MachineFunction::init` is an `assert`
(MachineFunction.cpp:242-244), which a release `llc` compiles out.
In 23.1.5, 23.1.6 and `main` it is an unconditional `report_fatal_error`
(MachineFunction.cpp:249-256 at 23.1.5).
A one-function file shows it without OpenSnitch:
`clang -O2 -emit-llvm -c t.c -o t.bc && llc -march=bpf -mcpu=generic -filetype=obj -o t.o t.bc`
exits 134 with clang/llc 23.1.5 and 0 with 22.1.8.

`--target=bpf` alone does not work on x86 headers with either clang 22.1.8 or 23.1.5.
The compile step fails on kernel 7.3.5 headers with
`bitsperlong.h:27:16: error: static assertion failed ... Inconsistent word size`
and `asm/div64.h:110:23: error: invalid output constraint '+a' in asm`.
So the Makefile needs a source change (CO-RE with a `vmlinux.h`, as in #1422), not a flag.

Workaround checked on CachyOS, kernel 7.3.5-1-cachyos:
put `/usr/lib/llvm22/bin` first in `PATH` for `make all`.
Master `a1353848` (with `-DCC_USING_FENTRY` from the AUR patch) builds `opensnitch.o`, `opensnitch-procs.o`
and `opensnitch-dns.o`, with the same 18 probe sections as the 1.7.2 package's objects.
v1.8.0 builds the same way with
`EXTRA_FLAGS="-fms-extensions -Wno-microsoft-anon-tag -DCC_USING_FENTRY"`.
The master objects load under a 1.9.0 daemon.
`clang-22` is not on `PATH` on Arch, so the `CC=clang-22` suggestion in this thread needs `/usr/lib/llvm22/bin/clang`.
~~~

The comment makes no claim about the `v1.8.0` objects at run time.
A comment has no form fields,
so the poster has none to answer.

[version-go]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/core/version.go#L6
[mk-master-cc]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/ebpf_prog/Makefile#L18-L19
[mk-master-rules]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/ebpf_prog/Makefile#L93-L101
[mk-master-flags]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/ebpf_prog/Makefile#L56-L59
[mk-master-i686]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/ebpf_prog/Makefile#L26
[mk-v180-rules]: https://github.com/evilsocket/opensnitch/blob/b404c4c6316760fa7bc415509d3f8d747f7dc9cc/ebpf_prog/Makefile#L62-L70
[llvm-tm-compat]: https://github.com/llvm/llvm-project/blob/392dfff97d29b59d77e5cac6d4b6ef2585fe7117/llvm/include/llvm/Target/TargetMachine.h#L215-L217
[llvm-bpf-tm]: https://github.com/llvm/llvm-project/blob/392dfff97d29b59d77e5cac6d4b6ef2585fe7117/llvm/lib/Target/BPF/BPFTargetMachine.h
[llvm22-check]: https://github.com/llvm/llvm-project/blob/ca7933e47d3a3451d81e72ac174dcb5aa28b59d1/llvm/lib/CodeGen/MachineFunction.cpp#L242-L244
[llvm23-check]: https://github.com/llvm/llvm-project/blob/392dfff97d29b59d77e5cac6d4b6ef2585fe7117/llvm/lib/CodeGen/MachineFunction.cpp#L249-L256
[llvm23-6-check]: https://github.com/llvm/llvm-project/blob/5b10d2d74929eae78912771222c9855af97496f4/llvm/lib/CodeGen/MachineFunction.cpp#L249-L256
[llvm-main-check]: https://github.com/llvm/llvm-project/blob/acfb7759cf64f4e81fbbbdaccf2d40aa22c6b42b/llvm/lib/CodeGen/MachineFunction.cpp#L237-L244
[llvm-228348]: https://github.com/llvm/llvm-project/issues/228348
[llvm-228348-reply]: https://github.com/llvm/llvm-project/issues/228348#issuecomment-5979013383
[llvm-228348-first]: https://github.com/llvm/llvm-project/issues/228348#issuecomment-5978942790
[issue-1686]: https://github.com/evilsocket/opensnitch/issues/1686
[issue-1686-diagnosis]: https://github.com/evilsocket/opensnitch/issues/1686#issuecomment-6060890468
[issue-1686-target]: https://github.com/evilsocket/opensnitch/issues/1686#issuecomment-6060890468
[issue-1686-quicktest]: https://github.com/evilsocket/opensnitch/issues/1686#issuecomment-6061215036
[issue-1686-notenough]: https://github.com/evilsocket/opensnitch/issues/1686#issuecomment-6061636849
[issue-1537]: https://github.com/evilsocket/opensnitch/issues/1537
[issue-1656]: https://github.com/evilsocket/opensnitch/issues/1656
[issue-1656-fentry]: https://github.com/evilsocket/opensnitch/issues/1656#:~:text=Compiler%20does%20not%20support%20fentry%3F
[pr-1422]: https://github.com/evilsocket/opensnitch/pull/1422
[pr-1422-comment]: https://github.com/evilsocket/opensnitch/pull/1422#issuecomment-3844919893
[pr-1422-last]: https://github.com/evilsocket/opensnitch/pull/1422#issuecomment-3874536187
[pr-1422-reluctant]: https://github.com/evilsocket/opensnitch/pull/1422#issuecomment-3282740922
[pr-1554]: https://github.com/evilsocket/opensnitch/pull/1554
[commit-614537c9]: https://github.com/evilsocket/opensnitch/commit/614537c92ec82f54f76a45fb406ad2fb6e6fa618
[arch-mr-4]: https://gitlab.archlinux.org/archlinux/packaging/packages/opensnitch/-/merge_requests/4
[arch-mr-4-614]: https://gitlab.archlinux.org/archlinux/packaging/packages/opensnitch/-/merge_requests/4#:~:text=Upstream%20fixed%20this%20in%20evilsocket%2Fopensnitch%40614537c9
[arch-mr-4-llvm22]: https://gitlab.archlinux.org/archlinux/packaging/packages/opensnitch/-/merge_requests/4#:~:text=This%20MR%20builds%20the%20eBPF%20step%20with%20LLVM%2022
[aur-patch]: https://aur.archlinux.org/cgit/aur.git/tree/0001-Build-with-Archlinux.patch?h=opensnitch-ebpf-module-git&id=b67f55db48c86e790bcd37185dd7ebc18c882bf3
[aur-ebpf]: https://aur.archlinux.org/packages/opensnitch-ebpf-module-git
[aur-git]: https://aur.archlinux.org/packages/opensnitch-git
[spec-tasks]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/utils/packaging/daemon/rpm/opensnitch.spec#L85-L89
[tasks-json]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/data/tasks/tasks.json
[default-config]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/data/default-config.json#L21
[sysfw-172]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/system-fw.json#L42-L43
[sysfw-master]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/data/system-fw.json#L42-L43
[chains-go]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/firewall/nftables/chains.go#L115-L137
[enums-go]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/firewall/nftables/exprs/enums.go#L5-L7
[findfirst-master]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/rule/loader.go#L520-L541
[findfirst-172]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/loader.go#L496-L515
[operator-network]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/rule/operator.go#L185-L188
[operator-network-172]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/operator.go#L156-L185
[ci-workflow]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/.github/workflows/build_ebpf_modules.yml#L34-L45
[issue-template]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/.github/ISSUE_TEMPLATE/bug_report.md
[doc-kernel-modules]: kernel-upgrade-without-reboot-removes-running-kernel-modules.md
[doc-sysfw-quirks]: opensnitch-1-7-2-system-firewall-quirks.md
