# Platform probes for the all-Rust monorepo manager

Probes run 2026-09-16 and 2026-09-17 for `doc/planning/monorepo-manager-from-scratch-design.md`
("Platforms and builds",
 "Hashing",
 "Single binary",
 "Cache").
Later decisions changed two premises here:
meow's hash is XXH3-128 rather than `gxhash`,
so the AES-specific startup check became a check for whatever target features a build was compiled with,
and the release matrix now ships separate `x86-64-v3` and `x86-64-v4` builds
of which only `x86-64-v4` and aarch64 block publishing.
The measurements and the static-pie work below are unaffected.
Work directory:
`/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/platforms/`
(called `platforms/` in this file).
Nothing inside the repository was modified.

Every claim carries a label.
"Verified" names the command and output,
 or the file and line,
 that backs it.
"Unverified" marks recall,
 inference,
 or a claim no probe exercised.

## Summary

- Verified:
   all four targets build from the skeleton with gxhash 3.5.0 and run the skeleton's `run` mode to exit 0,
   natively for x86_64 and under QEMU user mode for aarch64.
- Verified:
   sizes are 1,983,504 bytes (`x86_64-unknown-linux-gnu`,
   Fedora 44 host),
   2,098,120 bytes (`x86_64-unknown-linux-musl`),
   1,709,080 bytes (`aarch64-unknown-linux-gnu`,
   Debian bookworm container),
   and 1,744,728 bytes (`aarch64-unknown-linux-musl`).
- Verified:
   both glibc builds need glibc 2.34 or newer;
   each ran on glibc 2.34 (UBI 9) and failed on glibc 2.33 (Fedora 34) with "version `GLIBC_2.34' not found".
- Verified:
   without AES the unmodified skeleton dies with SIGILL on both architectures,
   after positive controls with AES passed on the same QEMU binaries.
- Verified:
   the startup check prototype prints a diagnostic and exits 3 on the same no-AES CPU models,
   and the same binary with the check skipped dies with SIGILL.
- Verified:
   `is_x86_feature_detected!("aes")` and `is_aarch64_feature_detected!("aes")` both returned `true` on CPU models
   without AES in `+aes` builds.
- Verified:
   gxhash issue #111 did not reproduce on aarch64 Linux under QEMU,
   with nightly-2026-09-12 or with the reporter's Rust 1.84.0 (`9fc6b4312`),
   in dev and release-with-debug-assertions profiles,
   while a positive control produced the issue's exact panic message under the same QEMU and compilers.
- Nothing found blocks the four-target matrix.
  Caveats that the design should record are in "Caveats for the four-target matrix".

## Bounds and network use

- Verified:
   every `cargo build` used `--jobs 4`.
- Verified:
   every probe container run used `podman run --rm --init --memory=2g --cpus=2 --security-opt label=disable`
   (`platforms/run-probes.ts`,
   function `podman`),
   and the cross builds,
   sysroot copy,
   and QEMU build used the same flags without `--init`.
  `--security-opt label=disable` avoids relabeling host directories under enforcing SELinux.
- Exception,
   Verified:
   one-line version checks ran as `podman run --rm <image>` without memory or CPU bounds:
   `ldd --version` in `debian:bullseye-slim`,
   `rpm --query glibc` in `ubi9/ubi-minimal` and `fedora:34` (x86_64 and arm64),
   and `uname -m`.
- Verified:
   `podman build --cpuset-cpus=0,1` failed with "controller `cpuset` is not available",
   because the rootless user slice delegates only `cpu io memory pids dmem`
   (`/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/cgroup.controllers`);
   image builds therefore used `--memory=2g --cpu-period=100000 --cpu-quota=200000`.
- Verified:
   the patched QEMU build ran `ninja -j 2` inside a bounded container.
- Verified:
   small single-process QEMU runs of static aarch64 binaries (ids `b1`,
   `e6`,
   `f-arm-*`,
   `g-arm184-*`,
   `h-*`) ran on the host with the host's `qemu-aarch64-static`,
   sequentially,
   with a 120 second kill timeout per run in `platforms/run-probes.ts` (`PROBE_TIMEOUT_MS`).
- Verified:
   direct HTTP fetches used `curl --user-agent 'Mozilla/5.0 (X11; Linux x86_64)'`,
   unauthenticated,
   against `raw.githubusercontent.com` and `api.github.com`.
  `src.fedoraproject.org` answered with an Anubis bot challenge page instead of the `sources` file.
- Unverified:
   `dnf`,
   `apt-get`,
   `rustup`,
   and `podman pull` sent their own default User-Agent strings,
   which were not inspected;
   no credentials or personal identifiers were passed to them.
- Verified (this session's command history):
   `rpm-ostree` was not invoked,
   and no cgroups or systemd units were created.

## Environment facts

- Verified:
   `rustc 1.100.0-nightly (0fc141305 2026-09-11)`,
   LLVM 23.1.1,
   active through `RUSTUP_TOOLCHAIN=nightly-2026-09-12`
   (`rustc --version --verbose`).
- Verified:
   `/usr/bin/qemu-aarch64-static` is present from package `qemu-user-static-aarch64-10.2.2-1.fc44.x86_64`
   (`rpm --query --file`),
   and `/proc/sys/fs/binfmt_misc/qemu-aarch64` registers it with flags `F`.
  This corrects the design's "QEMU user mode is not installed on the development machine" for aarch64.
  `qemu-x86_64` is not on the host;
   x86_64 emulation ran from Fedora 44's `qemu-user-static-x86` inside a container.
- Verified:
   the scratch filesystem is tmpfs with 16G size (`findmnt --target /tmp/claude-1000`),
   so the skeleton's `statfs` magic in these runs is `16914836` (tmpfs),
   not the btrfs magic.

## Probe 1: four-target build matrix

The skeleton was copied to `platforms/skeleton/` (`Cargo.toml`,
 `Cargo.lock`,
 `src/main.rs`) unchanged.
Its release profile is `lto = true`,
 `codegen-units = 1`,
 `strip = true`,
 `panic = "abort"`,
 `opt-level = 3`.

### Target installation

- Verified:
   `rustup target add aarch64-unknown-linux-gnu aarch64-unknown-linux-musl --toolchain nightly-2026-09-12`
   installed both targets (`rustup target list --installed` lists all four).

### Commands and linker setup

`x86_64-unknown-linux-gnu`,
 on the host:

```bash
# platforms/skeleton
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS='-C target-feature=+aes,+sse2' \
  cargo build --offline --release --jobs 4 --target x86_64-unknown-linux-gnu
```

- Verified:
   rustc links through `cc` with `-fuse-ld=lld` and `-B <toolchain>/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld`
   (`cargo rustc ... -- --print link-args`),
   and the binary's `.comment` section records `Linker: LLD 23.1.1`
   (`readelf --string-dump=.comment`).

`x86_64-unknown-linux-musl`,
 on the host:

```bash
# platforms/skeleton
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS='-C target-feature=+aes,+sse2' \
  cargo build --offline --release --jobs 4 --target x86_64-unknown-linux-musl
```

- Verified:
   rustc links through the host `cc` with `-nostartfiles -static-pie` and the toolchain's self-contained `rcrt1.o`
   (`--print link-args`);
   `.comment` has no LLD entry,
   so the host GNU linker did the link.

`aarch64-unknown-linux-musl`,
 on the host,
 with no C cross toolchain:

```bash
# platforms/skeleton
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS='-C target-feature=+aes,+neon' \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
  cargo build --offline --release --jobs 4 --target aarch64-unknown-linux-musl
```

- Verified:
   rustc invokes `rust-lld -flavor ...` with the self-contained `crt1.o`,
   `crti.o`,
   `crtbegin.o`,
   `crtend.o`,
   `crtn.o`,
   and `-static`
   (`--print link-args`).
- Verified:
   the skeleton's dependency tree needs no C compiler for this target,
   because the build succeeded without one.
  Unverified:
   future dependencies with C sources would need an aarch64 musl C compiler.

`aarch64-unknown-linux-gnu`,
 in a Debian bookworm container that supplies only the cross linker and sysroot:

```dockerfile
# platforms/containers/debian-cross/Containerfile
FROM docker.io/library/debian:bookworm-slim
RUN apt-get update \
 && apt-get install --yes --no-install-recommends \
      gcc-aarch64-linux-gnu libc6-dev-arm64-cross binutils-aarch64-linux-gnu \
      gcc libc6-dev file binutils ca-certificates \
 && rm --recursive --force /var/lib/apt/lists/*
```

```bash
# image: podman build --memory=2g --cpu-period=100000 --cpu-quota=200000 --tag localhost/meow-probe-debian-cross:bookworm platforms/containers/debian-cross
podman run --rm --memory=2g --cpus=2 --security-opt label=disable \
  --volume /var/home/user/.rustup:/rustup:ro \
  --volume /var/home/user/.cargo/registry:/cargo/registry:ro \
  --volume platforms/skeleton:/w --workdir /w \
  --env RUSTUP_HOME=/rustup --env CARGO_HOME=/cargo \
  --env PATH=/rustup/toolchains/nightly-2026-09-12-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
  --env CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  --env 'CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS=-C target-feature=+aes,+neon' \
  localhost/meow-probe-debian-cross:bookworm \
  cargo build --offline --release --jobs 4 --target aarch64-unknown-linux-gnu --target-dir /w/target-bookworm
```

- Verified:
   the host's rustup toolchain and cargo registry mounted read-only were enough for an offline build inside the
   container.
- Verified:
   the sysroot is `libc6-arm64-cross 2.36-8cross1` (`dpkg -s`),
   and `.comment` records `GCC: (Debian 12.2.0-14)` with no LLD entry.
- Verified:
   the same container also built `x86_64-unknown-linux-gnu` into `target-bookworm`
   (same command with `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS='-C target-feature=+aes,+sse2'`
and no linker override).

### Sizes and file output

Sizes from `stat --format='%s %n'`,
 file types from `file`:

- Verified,
   `target/x86_64-unknown-linux-gnu/release/skeleton-gxhash`,
   1,983,504 bytes:
   "ELF 64-bit LSB pie executable,
   x86-64,
   version 1 (SYSV),
   dynamically linked,
   interpreter /lib64/ld-linux-x86-64.so.2,
   for GNU/Linux 3.2.0,
   BuildID[sha1]=3a22673f9f37cfe076098b88290aa4dbb4b1d170,
   stripped".
- Verified,
   `target-bookworm/x86_64-unknown-linux-gnu/release/skeleton-gxhash`,
   1,980,536 bytes:
   "ELF 64-bit LSB pie executable,
   x86-64,
   version 1 (SYSV),
   dynamically linked,
   interpreter /lib64/ld-linux-x86-64.so.2,
   for GNU/Linux 3.2.0,
   BuildID[sha1]=f12923a116773f1931ee3346335735d265952ee8,
   stripped".
- Verified,
   `target/x86_64-unknown-linux-musl/release/skeleton-gxhash`,
   2,098,120 bytes:
   "ELF 64-bit LSB pie executable,
   x86-64,
   version 1 (SYSV),
   static-pie linked,
   BuildID[sha1]=e2b2c32702a1a3ad87791f3dd10b45b8fab72f23,
   stripped".
  The size matches the design's recorded 2,098,120 bytes.
- Verified,
   `target-bookworm/aarch64-unknown-linux-gnu/release/skeleton-gxhash`,
   1,709,080 bytes:
   "ELF 64-bit LSB pie executable,
   ARM aarch64,
   version 1 (SYSV),
   dynamically linked,
   interpreter /lib/ld-linux-aarch64.so.1,
   BuildID[sha1]=c1f01df5d70c2143a7cddaf2e1541a46bf6948b1,
   for GNU/Linux 3.7.0,
   stripped".
- Verified,
   `target/aarch64-unknown-linux-musl/release/skeleton-gxhash`,
   1,744,728 bytes:
   "ELF 64-bit LSB executable,
   ARM aarch64,
   version 1 (SYSV),
   statically linked,
   stripped".
- Verified:
   both glibc binaries need `libgcc_s.so.1`,
   `libm.so.6`,
   and `libc.so.6` (`readelf --dynamic`).

### aarch64 musl is static but not position independent

- Verified:
   `readelf --file-header` reports `Type: EXEC` for the aarch64 musl binary,
   with the first `LOAD` at `0x200000`,
   while x86_64 musl is `static-pie linked`.
- Verified:
   `rustc -Z unstable-options --print target-spec-json --target aarch64-unknown-linux-musl` has no
   `static-position-independent-executables` key,
   while the same output for `x86_64-unknown-linux-musl`
   and `aarch64-unknown-linux-gnu` has `"static-position-independent-executables": true`.
- Verified:
   the aarch64 musl link uses `crt1.o`,
   not `rcrt1.o` (`--print link-args`).
- Unverified (inference from the target spec):
   another linker would not change this,
   because rustc picks the non-PIE output kind from the target spec before choosing crt objects.
  The consequence is no ASLR for the aarch64 musl executable image.
  Whether upstream plans to enable static PIE for this target is Unverified;
   one GitHub issue search found nothing specific.

### glibc floor

Versioned symbol needs (`readelf --version-info --wide`,
 `objdump --dynamic-syms`):

- Verified,
   Fedora 44 host build (host glibc `glibc-2.43-8.fc44`,
   `rpm --query glibc`):
   highest strong need `GLIBC_2.34`
   (`__libc_start_main`,
   `pthread_create`,
   `pthread_join`,
   `pthread_key_create`,
   `dlsym`,
   and other pthread symbols),
   plus `GLIBC_2.39` for `pidfd_spawnp` and `pidfd_getpid`,
   which `.gnu.version_r` marks `Flags: WEAK`.
- Verified,
   bookworm builds (glibc 2.36),
   both architectures:
   highest need `GLIBC_2.34`,
   with no weak `GLIBC_2.39` entry.

Runtime floor checks (`platforms/logs/probe-results.jsonl`,
 ids starting `a4`,
 `a5`,
 `a6`):

- Verified:
   x86_64 host build and x86_64 bookworm build both exit 0 on `ubuntu:22.04` (glibc 2.35)
   and `ubi9/ubi-minimal` (glibc 2.34-275.el9_8).
- Verified:
   the host build also prints to stderr on those older systems:
   "/lib64/libc.so.6:
   weak version `GLIBC_2.39' not found (required by .../skeleton-gxhash)".
  The bookworm build prints nothing there.
- Verified:
   both x86_64 builds exit 1 on `fedora:34` (glibc 2.33) with "version `GLIBC_2.34' not found",
   and on `debian:bullseye-slim` (glibc 2.31) with the same message for 2.32,
   2.33,
   and 2.34.
- Verified:
   the aarch64 bookworm build exits 0 in an arm64 `ubi9/ubi-minimal` container (glibc 2.34,
   through binfmt_misc QEMU),
   and exits 1 in an arm64 `fedora:34` container with "version `GLIBC_2.34' not found".
- Minimum glibc for both glibc targets:
   2.34 (Verified by the runs above).
  A release build host with glibc 2.39 or newer adds the loader warning on glibc 2.34 to 2.38
  (Verified on 2.34 and 2.35 only).

## Probe 2: aarch64 binaries under QEMU user mode

Driver:
`platforms/run-probes.ts` creates a fresh fixture directory per run (`platforms/runs/<id>/package/a/b/Cargo.toml`),
runs the binary as `skeleton-gxhash run .` with that directory as the working directory
(so the Unix socket path stays short),
and appends status,
 signal,
 and output to `platforms/logs/probe-results.jsonl`.

- Verified,
   `b1-skeleton-arm-musl-qemu-host`:
   `/usr/bin/qemu-aarch64-static target/aarch64-unknown-linux-musl/release/skeleton-gxhash run .` exited 0.
- Verified,
   `b2-skeleton-arm-gnu-qemu-debian`:
   inside the Debian cross image with the host's `qemu-aarch64-static` mounted,
   `qemu-aarch64-static -L /usr/aarch64-linux-gnu target-bookworm/aarch64-unknown-linux-gnu/release/skeleton-gxhash run .` exited 0.
- Verified:
   both printed the same line as the native x86_64 builds:
   `true 107314281523559204355742562552865021658 0 [a] b = 1 {"dirs":4} 16914836 exit status: 0`,
   so `gxhash128(b"x", 0)` is identical on x86_64 and aarch64.

## Probe 3: unmodified skeleton on CPUs without AES

### CPU models

- Verified:
   no stock QEMU 10.2.2 aarch64 CPU model lacks AES.
  Every model's `ID_AA64ISAR0` has AES field value 2
   (`target/arm/cpu64.c:694`,
   `:756` and `target/arm/tcg/cpu64.c:70`,
   `:224`,
   `:315`,
   `:357`,
   `:429`,
   `:510`,
   `:678`,
   `:757`,
   `:1006`,
   `:1108`,
   `:1218` at tag `v10.2.2`),
   and `-cpu max,aes=off` fails with "Property 'max-arm-cpu.aes' not found".
- Verified:
   QEMU gates the AArch64 AES instructions on that ID field (`target/arm/tcg/translate-a64.c:5453-5456`,
   `TRANS_FEAT(AESE, aa64_aes, ...)`)
   and derives the guest `AT_HWCAP` AES bit from it (`linux-user/aarch64/elfload.c:147`).
- Verified:
   a patched QEMU was built from Fedora's signed source RPM
   (`dnf download --srpm qemu-10.2.2-1.fc44`,
   `rpm --checksig` printed "digests signatures OK",
   tarball sha256 `784b296ff29c1417aa72323abcb2d2ea9ab9771724f577dcd785c3b04f21e176`),
   changing only line 315 of `target/arm/tcg/cpu64.c` in `aarch64_a72_initfn` from `0x00011120` to `0x00010000`,
   which models a Cortex-A72 without the Cryptography Extension (no AES,
   PMULL,
   SHA1,
   SHA2).
  Configure:
   `./configure --target-list=aarch64-linux-user --disable-system --disable-tools --disable-docs --disable-werror`,
   then `ninja -C build -j 2 qemu-aarch64`,
   in `localhost/meow-probe-fedora-qemu:44`
   (`platforms/containers/fedora-qemu/Containerfile`).
  Output:
   `platforms/qemu-out/qemu-aarch64-nocrypto-a72`,
   sha256 `08a84152af75225cccdb217295efd68bd890d38390f9a46eed4061992e069a0d`,
   dynamically linked against Fedora 44 glib,
   so it runs inside that image.
- Verified:
   on x86_64,
   stock `qemu-x86_64-static` 10.2.2 models `Westmere` has AES and `Nehalem` and `qemu64` do not:
   `cpuid(1).ecx` read `0x82982203`,
   `0x80982201`,
   and `0x80002001` in the probe 4 runs (ECX bit 25 set only for Westmere).
  QEMU's decoder refuses AES opcodes without that bit (`target/i386/tcg/decode-new.c.inc:2333-2334`).

### Results

All runs in `platforms/logs/probe-results.jsonl`.
Status 132 is 128 plus signal 4 (SIGILL),
 reported through `podman run --init`.

x86_64,
 stock `qemu-x86_64-static` in the Fedora 44 image:

- Verified,
   positive control:
   `-cpu Westmere` exited 0 for the musl build (`c1-skeleton-x86-musl-Westmere`)
   and the Fedora-host glibc build (`c1-skeleton-x86-gnu-Westmere`).
- Verified:
   `-cpu Nehalem` and `-cpu qemu64` exited 132 for both builds,
   with stderr "qemu:
   uncaught target signal 4 (Illegal instruction) - core dumped"
   and no stdout (`c2-*`,
   `c3-*`).

aarch64,
 patched QEMU in the Fedora 44 image:

- Verified,
   positive control:
   patched QEMU `-cpu cortex-a53` exited 0 for musl and glibc builds (`d1-*`;
   glibc used `-L /p/sysroot-arm64-bookworm`,
   copied from the Debian cross image's `/usr/aarch64-linux-gnu/lib`).
- Verified:
   patched QEMU `-cpu cortex-a72` exited 132 for musl and glibc builds,
   with "qemu:
   uncaught target signal 4 (Illegal instruction) - core dumped" (`d2-*`).
- Verified,
   control for the patch:
   stock `qemu-aarch64-static -cpu cortex-a72` exited 0 for both builds (`d3-*`).

### QEMU as PID 1 hangs instead of dying

- Verified:
   the first batch,
   without `--init`,
   timed out after 120 seconds for every SIGILL case
   (`platforms/logs/probe-results-run1-without-init.jsonl`,
   status 143 from the driver's kill).
- Verified:
   QEMU dies from a guest fatal signal by installing `SIG_DFL`,
   calling `kill(getpid(), host_sig)`,
   then `sigsuspend`
   (`linux-user/signal.c:787-811`).
- Unverified (kernel behavior,
   recall):
   the kernel drops default-action signals a PID namespace's init sends to itself,
   so `sigsuspend` never returns.
  Consistent with that,
   adding `podman run --init` turned every hang into status 132 (Verified).
  CI that runs QEMU user mode as a container's entry command needs an init process.

## Probe 4: startup CPU check prototype

Source:
`platforms/probes/src/bin/cpucheck.rs`,
 crate `platforms/probes/Cargo.toml`
(gxhash `=3.5.0`;
 `rustix` `=1.1.4` with `default-features = false, features = ["param"]` for aarch64 only;
 same release profile as the skeleton).

- Verified:
   the file starts with `#![forbid(unsafe_code)]` and compiles for `x86_64-unknown-linux-musl`
   and `aarch64-unknown-linux-musl` on nightly-2026-09-12,
   and for `x86_64-unknown-linux-gnu` on stable `rustc 1.98.0 (88d9e12ae 2026-08-18)`.
- Verified:
   `core::arch::x86_64::__cpuid` is a safe `pub fn` in the nightly sources
   (`library/stdarch/crates/core_arch/src/x86/cpuid.rs:107`).
- Verified,
   sizes:
   `cpucheck` is 393,848 bytes on x86_64 musl and 354,880 bytes on aarch64 musl.

Behavior:

- x86_64:
   `__cpuid(0).eax` must be at least 1,
   then leaf 1 ECX bit 25 (AES-NI) and EDX bit 26 (SSE2),
   the bits `std_detect` uses (`library/std_detect/src/detect/os/x86.rs:108`,
   `:118`).
- aarch64:
   `rustix::param::linux_hwcap()`,
   then `HWCAP_AES` (bit 3) and NEON computed as std_detect does:
   `fp && asimd && (!fphp || asimdhp)` with bits 0,
   1,
   9,
   10
   (`library/std_detect/src/detect/os/linux/aarch64.rs:144-154`,
   `:332`).
- Missing capabilities print to stderr,
   for example:

  ```text
  error: this CPU lacks AES (Arm Cryptography Extension) (AT_HWCAP bit 3, HWCAP_AES).
  This tool hashes file contents with gxhash, which uses AES (Arm Cryptography Extension) and NEON instructions on aarch64.
  It has no fallback hash implementation, so it cannot run on this CPU and stopped before hashing anything.
  To run it, use a CPU that has these capabilities; in a virtual machine or emulator, expose the host CPU's AES support to the guest.
  ```

  and the process exits 3.
- `cpucheck --skip-check` hashes without checking,
   as a control.

Results (`platforms/logs/probe-results.jsonl`):

- Verified,
   `e1-cpucheck-x86-native`:
   exit 0,
   "check passed",
   `gxhash128=368417c83080600e87bb73a3fa35dae2 gxhash64=87bb73a3fa35dae2`.
- Verified,
   `e2-cpucheck-x86-Westmere`:
   exit 0,
   same hashes.
- Verified,
   `e3-cpucheck-x86-Nehalem` and `e5-cpucheck-x86-qemu64`:
   exit 3 with the x86_64 diagnostic "error:
   this CPU lacks AES-NI (CPUID leaf 1,
   ECX bit 25)."
   and no SIGILL.
- Verified,
   `e4-cpucheck-x86-Nehalem-skip`:
   exit 132,
   "qemu:
   uncaught target signal 4 (Illegal instruction)".
- Verified,
   `e6-cpucheck-arm-host-qemu-max`:
   exit 0,
   `AT_HWCAP=0x1effffffb`,
   same hashes as x86_64.
- Verified,
   `e7-cpucheck-arm-cortex-a53` (patched QEMU):
   exit 0,
   `AT_HWCAP=0x8fb`,
   same hashes.
- Verified,
   `e8-cpucheck-arm-cortex-a72` (patched QEMU):
   exit 3,
   `AT_HWCAP=0x883` (AES,
   PMULL,
   SHA1,
   SHA2 bits clear),
   the aarch64 diagnostic,
   no SIGILL.
- Verified,
   `e9-cpucheck-arm-cortex-a72-skip`:
   exit 132,
   SIGILL.
- Verified,
   compile-time detection:
   on Nehalem and qemu64 the evidence line printed
   `cfg!(target_feature="aes")=true is_x86_feature_detected!("aes")=true` while ECX bit 25 was clear;
   on the patched Cortex-A72 it printed `is_aarch64_feature_detected!("aes")=true` with `AT_HWCAP=0x883`.
  This confirms the design's claim that these macros cannot perform the check in a `+aes` build.
- Verified:
   the check path itself contains no AES instructions reachable before the check,
   because the no-AES runs reached the diagnostic and exited 3.

### Findings for the design

- Verified from source:
   `rustix::param::linux_hwcap` calls `init_auxv()` whenever `AT_HWCAP` or `AT_HWCAP2` is 0
   (`src/backend/linux_raw/param/auxv.rs:66-70` in rustix 1.1.4),
   and `init_auxv` is `init_auxv_impl().unwrap()` (`:268-270`),
   which tries `prctl(PR_GET_AUXV)` (Linux 6.4 and newer) and then `/proc/self/auxv` (`:283-319`).
  Unverified at run time:
   on a kernel older than 6.4 with no readable `/proc`,
   the check would panic,
   and with `panic = "abort"` abort,
   instead of printing a diagnostic.
  The prototype's `hwcap == 0` branch is unreachable for that reason.
- Verified from source:
   rustix's `use-libc-auxv` alternative finds `getauxval` through `dlsym(RTLD_DEFAULT, ...)` (`src/weak.rs:139-144`)
   and returns `(0, 0)` when absent (`src/backend/linux_raw/param/libc_auxv.rs:80-82`).
  Unverified:
   that `dlsym` lookup fails in a static musl binary,
   which would make this feature unsuitable for the musl targets.
- Verified:
   under QEMU user mode,
   `PR_GET_AUXV` falls into `do_prctl`'s default branch and returns `-TARGET_EINVAL` (`linux-user/syscall.c:6752-6755`),
   and QEMU serves the guest auxv for `/proc/self/auxv` (`linux-user/syscall.c:8566`,
   `:8719`),
   so the aarch64 QEMU runs exercised rustix's `/proc/self/auxv` path,
   not `PR_GET_AUXV`.
- Verified from source:
   Rust's aarch64 `aes` target feature means FEAT_AES
   and FEAT_PMULL (`library/std_detect/src/detect/arch/aarch64.rs:123-124`),
   and `pmull` detection is implied by the `aes` target feature (`:117-118`).
  std's runtime `aes` detection requires only the AES bit on most CPUs
  (`library/std_detect/src/detect/os/linux/aarch64.rs:343`)
   but AES and PMULL on the Exynos 9810 path (`:286`).
  Open question for the design:
   whether the check should also require `HWCAP_PMULL` (bit 4).
  gxhash's aarch64 code calls only `vaeseq_u8` and `vaesmcq_u8` among crypto intrinsics (Verified,
   `src/gxhash/platform/arm.rs`).
- Unverified:
   no probe could separate NEON from AES,
   because every QEMU aarch64 model has NEON;
   NEON is part of the `+v8a` baseline in both aarch64 Linux target specs (Verified,
   `"features": "+v8a,+outline-atomics"`).
- Observation,
   Verified:
   `gxhash64` equals the low 64 bits of `gxhash128` for the probe input on both architectures.

## Probe 5: gxhash issue 111

Upstream record,
 fetched unauthenticated from `api.github.com/repos/ogxd/gxhash/issues/111`:

- Verified:
   the issue is open,
   titled "Panic when debug-assertions=true in release profile",
   reported on macOS Apple Silicon with Rust 1.84.0 (`9fc6b4312`) and gxhash 3.4.1,
   and a 2025-03-22 comment says it still happens on 3.5.0.
- Verified:
   the reported panic is "unsafe precondition(s) violated:
   ptr::copy_nonoverlapping requires that both pointer arguments are aligned and non-null
   and the specified memory ranges do not overlap",
   inside `BuildHasher::hash_one`,
   from a `gxhash::HashMap<usize, usize>` reproducer (`pedantic79/gxhash-test`,
   `src/main.rs`).

Probe:
`platforms/probes/src/bin/gx111.rs` (nightly,
 edition 2024) and `platforms/gx111-rust184/src/main.rs` (same code,
 edition 2021,
 Rust 1.84.0).
Each run is one case:
`stack64`,
 `stack128`,
 `heap64`,
 `heap128` hash every length from 0 to 67 through `gxhash64` or `gxhash128` from a stack array or a `Vec`;
`hasher` feeds `GxHasher::write` for lengths 0 to 67 plus `write_usize`;
`hashmap` is the issue's reproducer.
Profiles:
dev (debug-assertions on) and `release-da` (release plus `debug-assertions = true`).
Every run printed `debug_assertions=true`.

- Verified:
   0 of 12 aarch64 musl runs with nightly-2026-09-12 panicked (`f-arm-debug-*`,
   `f-arm-release-da-*`,
   host stock QEMU),
   and 0 of 12 x86_64 runs did.
- Verified:
   0 of 12 aarch64 musl runs with Rust 1.84.0 (`.comment`:
   "rustc version 1.84.0 (9fc6b4312 2025-01-07)") panicked (`g-arm184-*`),
   nor 0 of 6 with `-C target-cpu=apple-m1` (`g-arm184-applem1-*`),
   nor 0 of 6 x86_64 glibc runs with 1.84.0 (`g-x86184-*`).
- Verified,
   positive control:
   `ubcontrol` (a deliberately overlapping `core::ptr::copy_nonoverlapping`) aborted under the same QEMU
   with the issue's exact message on both compilers
   (`h-ubcontrol-arm184-debug`,
   `h-ubcontrol-arm-nightly-debug`,
   signal SIGABRT).
  The UB checks were active,
   so the null result is not a disabled-check artifact.
- Verified from source,
   why current nightly is unlikely to hit this path:
   aarch64 `vld1q_s8` is `crate::ptr::read_unaligned(ptr.cast())`
   (`library/stdarch/crates/core_arch/src/aarch64/neon/generated.rs:10009-10010`),
   nightly's `read_unaligned` calls `read` on an alignment-1 wrapper (`library/core/src/ptr/mod.rs:1837-1857`),
   and only `copy_nonoverlapping` carries the overlap precondition (`library/core/src/ptr/mod.rs:553-568`),
   while Rust 1.84.0's `read_unaligned` still called `copy_nonoverlapping`
   (`library/core/src/ptr/mod.rs:1466` in that toolchain's `rust-src`).
- Unverified:
   why 1.84.0 did not reproduce on Linux.
  The overlap check compares the 16-byte read with the stack destination,
   so it depends on frame layout,
   which differs between `aarch64-apple-darwin` and `aarch64-unknown-linux-musl`;
   QEMU user mode cannot run macOS binaries,
   so the reported platform was not reproduced.
- Verified:
   the read beyond the input still happens in gxhash 3.5.0 (`src/gxhash/platform/mod.rs:18-25`,
   `src/gxhash/platform/arm.rs` `get_partial_unsafe`),
   so the issue is a latent out-of-bounds read hidden by the page check,
   not only a debug-check artifact.

## Probe 6: musl performance

- No timing comparison was made.
  The `elapsedMs` values in `platforms/logs/probe-results.jsonl` are single runs that mix native execution,
   QEMU,
   and container startup,
   and they say nothing about musl against glibc.

## Caveats for the four-target matrix

None of these blocked a build or a run.

- aarch64 musl is non-PIE static (Verified,
   "aarch64 musl is static but not position independent").
- glibc builds require glibc 2.34 (Verified),
   and building on a host with glibc 2.39 or newer adds a visible loader warning on glibc 2.34 to 2.38 (Verified);
   building in an older-glibc container such as Debian bookworm avoids the warning (Verified).
- aarch64 glibc linking needs a cross linker and an aarch64 glibc sysroot;
   Debian's `gcc-aarch64-linux-gnu` and `libc6-dev-arm64-cross` worked (Verified).
  Unverified (recall):
   Fedora's `gcc-aarch64-linux-gnu` ships no userspace glibc sysroot.
- aarch64 musl linked with `rust-lld` alone because the skeleton has no C code (Verified);
   a C dependency would need an aarch64 musl C toolchain (Unverified).
- Negative CPU tests on aarch64 need a patched QEMU,
   and QEMU in containers needs `--init` (Verified).
- The aarch64 CPU check can abort instead of diagnosing
   when neither `PR_GET_AUXV` nor `/proc/self/auxv` works (Verified from source,
   Unverified at run time).
- Unverified:
   CI runner availability for native aarch64 Linux was not checked.

## Changes made outside the scratchpad

- Verified:
   rustup targets `aarch64-unknown-linux-gnu` and `aarch64-unknown-linux-musl` added to `nightly-2026-09-12`.
- Verified:
   rustup toolchain `1.84.0` installed with
   `--profile minimal --target aarch64-unknown-linux-musl --component rust-src`.
- Verified,
   podman images built:
   `localhost/meow-probe-debian-cross:bookworm` (454 MB) and `localhost/meow-probe-fedora-qemu:44` (561 MB).
- Verified,
   podman images pulled:
   `docker.io/library/ubuntu:22.04`,
   `docker.io/library/debian:bullseye-slim`,
   `registry.access.redhat.com/ubi9/ubi-minimal:latest` and `registry.fedoraproject.org/fedora:34`
   (the last two now tagged as their arm64 variants after `podman pull --arch arm64`).
- Verified:
   podman named volume `meow-probe-qemu-src` (1.144 GB) holds the QEMU source RPM,
   source tree,
   and build tree.
- Verified:
   `platforms/` uses 785 MB of tmpfs,
   mostly Cargo target directories.

## Artifacts

- `platforms/run-probes.ts`:
   probe driver (`node run-probes.ts [id-prefix ...]`).
- `platforms/logs/probe-results.jsonl`:
   86 run records with argv,
   status,
   signal,
   and output.
- `platforms/logs/probe-results-run1-without-init.jsonl`:
   the PID 1 hang records.
- `platforms/logs/qemu-patched-build.log`,
   `platforms/qemu-out/configure.log`,
   `platforms/qemu-out/ninja.log`.
- `platforms/probes/src/bin/cpucheck.rs`,
   `platforms/probes/src/bin/gx111.rs`,
   `platforms/probes/src/bin/ubcontrol.rs`.
- `platforms/qemu-src/`:
   QEMU `v10.2.2` source files cited in this file.
- `platforms/upstream/`:
   gxhash issue #111 JSON and the reporter's reproducer.
