# zfs/request-to-openzfs-kernel-device.fuzz

The libFuzzer sidecar of
`package/zfs/request-to-openzfs-kernel-device`
(rule `SBS`:
the sidecar sits beside its subject and is named after it).
It fuzzes the adapted native nvlist codec:
whatever bytes arrive,
`NvList::unpack` must either refuse them or produce a list whose own packing
decodes and re-packs to the same bytes.

## How it runs

The only path is the container campaign,
because the repository forbids
mounting the host's Rust sysroot into a container
(`doc/handover/cli-git-rust-implementation.md` records the Clippy E0514 that
mixing them caused) and libFuzzer needs a nightly compiler.

```bash
# build the instrumented target; the container is removed, so this proves the compile
mise run //package/zfs/request-to-openzfs-kernel-device.fuzz:build

# build, then fuzz for 30 seconds in a mount-free, network-free, bounded container
mise run //package/zfs/request-to-openzfs-kernel-device.fuzz:smoke
```

`bin/container.ts` builds an image from the wrapper's gate base image,
adds pacman's `cargo-fuzz` and the repository's nightly toolchain from its
published archive,
fetches the two crates' locked dependencies at image-build time,
and then runs the target with `--network none`,
no home folder,
only the corpus and artifacts folders of the run's scratch mounted,
no `/dev/zfs` and explicit CPU,
memory and PID bounds.
It prints the `rustc`,
`cargo` and `cargo-fuzz` versions it used.
The nightly is the one `toolchain.lock.json`'s `rust` row names
(`nightly-2026-11-16` when this was written),
and `bin/container.ts` builds the published archive's URL from that version,
so a lock bump moves the fuzz image with it.
It is unpacked rather than mounted from the host,
because a container never mounts the host's Rust sysroot.

## The target

There is one target,
`nvlist_codec`,
named for the code it exercises.
Its corpus and artifacts directories are gitignored,
and `bin/container.ts` mounts both from
`node_modules/.monochromatic/zfs-binding-fuzz/` into the run,
so a crash's input at `artifacts/nvlist_codec/` and the growing corpus
outlive the container that found it;
a finding is filed with that input attached.
