# cli-git ZFS runtime layout checker design

**Non-final, unapproved proposal by an agent.**
This file designs one open item of the accepted ZFS worktree design,
"The runtime layout checker" in
[Open items the decisions leave][design-open-items],
which blocks step 6 of [the implementation plan][plan-file],
"The start-time compatibility gate and the start probe".
It was written on 2026-11-19 by a Pi coding agent session
on branch `docs/planning-cli-git-zfs-runtime-layout-checker-design`,
made from `main` at `73eaad0846`.
It read the sources named under [Sources read](#sources-read),
ran the read-only host probes named under
[What was run, and what was not](#what-was-run-and-what-was-not),
and changed no pool, dataset, mount or unit, and wrote no file in the repository except this one.
It ran no guest, so every behaviour outside the probes is a source reading,
and the measurements that would make the choices safe are
under [Owed measurements](#owed-measurements), unrun.
It decides nothing: the choices that are the owner's are
under [Questions for the owner](#questions-for-the-owner),
and the parts a writer can settle are
under [What a writer can settle without asking](#what-a-writer-can-settle-without-asking).
Nothing here is implemented, released or approved.

## Status and sources

### What this file is

The [decision record][design-record] accepted the route
"with its open items still open", and this is one of them.
The accepted design says the runtime gate verifies
"the field layout at runtime against the approved release and request contract",
answering "Runtime layout check (Recommended)"
([ABI verification][design-project-decisions]).
This file proposes how that check is built:
what it reads, what it compares, how it refuses,
which dependency parses the kernel's BTF,
and what must be measured before either the check or its dependency is trusted.
It adds no decision the accepted design did not make,
and it does not reopen the accepted answers.

The plan says the design "picks the profile dependency,
which goes to the owner with that design"
([Open items and the step that needs each][plan-open-items]).
The candidate work and the ranked options are
under [The profile-checking dependency](#the-profile-checking-dependency);
the pick is not made here.

### Sources read

The design records, read at `73eaad0846`:

- [The accepted design record][design-record].
- [The design proposal](cli-git-zfs-worktree-design-proposal.md), in the sections its open item
  and the plan's step 6 cite: "Requirements and module ownership",
  "Binding crate and compatibility", "ABI gate and tests",
  "What doctor checks and prints", "Project decisions" (the "ABI verification" and
  "Kernel-release readiness" items), "Open items the decisions leave",
  "VM measurements preceding acceptance", "Acceptance tests the build still owes"
  and "Decided behaviours with no test yet", plus "Dataset and path layout",
  "Add and Git registration", "ZED reconciliation and mount safety",
  "Removal and recovery", and "Ladder and fallback" for the facts this file uses.
- [The implementation plan](cli-git-zfs-worktree-implementation-plan.md), in its "How to read a step",
  "Dependencies and their state", "Wrapper code the design reuses", step 1, step 2, step 6,
  "What can start now", "Open items and the step that needs each",
  "Gaps the design's open-item list does not name",
  "Where the design is silent", and "What this plan did not do".
- [The measurements record](cli-git-zfs-measurements-owed-before-acceptance.md), in its fixture,
  items 1 and 2, its item summaries, its limits, and the guest logs named beside each fact.

OpenZFS, at tag `zfs-2.4.4`, commit `71a9f9578616`:
`META`, `config/zfs-meta.m4`, `include/sys/fs/zfs.h`,
`include/sys/zfs_ioctl.h`, `include/os/linux/spl/sys/sysmacros.h`,
`module/os/linux/zfs/zfs_ioctl_os.c`, `module/zfs/zfs_ioctl.c`,
`lib/libzfs/os/linux/libzfs_util_os.c`.

The Linux kernel, at tag `v6.8`, files fetched from the tagged tree:
`include/uapi/linux/btf.h`, `include/linux/btf.h`, `kernel/bpf/btf.c`,
`kernel/bpf/sysfs_btf.c`, `fs/sysfs/file.c`, `lib/Kconfig.debug`,
`scripts/Makefile.modfinal`, `Documentation/bpf/btf.rst`.
The current `include/uapi/linux/btf.h` on the default branch,
fetched 2026-11-19, for the header's later fields.

The wrapper's code the plan's "Wrapper code the design reuses" lists,
read for the pattern this design follows:
`package/git-policy/cli/SPEC.md` ("Compatibility and degradation",
"Single executable and shipped policies", "Doctor"),
`package/git-policy/cli/src/native/invocation/git_version.rs`,
`package/git-policy/cli/src/native/health/doctor_filesystem.rs`,
`package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
`package/git-policy/cli/src/native/service/wait_ceilings.rs`,
`package/git-policy/cli/Cargo.toml`, `toolchain.lock.json`.

The prior vet, [A Rust crate for OpenZFS][vet],
for the 2.2.2 and 2.4.4 struct difference, the request numbers and the wrapper's release model.

The governing method for [The profile-checking dependency](#the-profile-checking-dependency):
[`choosing-technology`][skill]
at commit `45d42d9b23d2f2752f9c512683b45905e13afe99`,
SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`,
the same revision the OpenZFS vet recorded.

Candidate sources, cloned under
`/home/user/temp/agent/zfs-runtime-layout-checker-design-2026-11-19/scratch/`
with `git remote set-url --push origin DISABLED` before any other command:

- `retis-org/btf-rs` at tag `v2.0.1`, commit `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`;
  the default branch's `btf-rs/src/cbtf.rs` at `cdf9ea46`, read through GitHub's API.
- `alessandrogario/btfparse` at `3788c4410a6cfd64ff57aa5bca20b2ee4eed3cd5`, tag `1.3.9`.
- `aya-rs/aya` at `44a6709164d2ac0c19c7845101e7b033e25b3f3f` (`aya-obj` 0.6.0).
- `arcjustin/btf` at `3b57cdab2ea80d13c3b5abd400bb2e4bfc0d6292` (crate `btf` 0.5.1).
- `anakryiko/btfdump` at `e5951d4b2e2e7ce6c36001c1b48c1c239c9759dc` (crate `btfdump` 0.0.5).
- `gidotencate/padlock` at `6a7d045f2c6b0cb64549ddebe5825411e25178fa` (`padlock-dwarf` 0.82.0).
- `libbpf/libbpf-rs` at `6d771ee9835fcf517dbcf1d29e25a97480425d45` (crate `libbpf-rs` 0.27.2).

Registry evidence: the crates.io API for those crates' metadata and versions,
and three searches recorded under
[Discovery record](#discovery-record).

### What was run, and what was not

Run on this host, read only:

- `stat`, `cat` and `od` on `/sys/module/zfs/version`,
  `/sys/kernel/btf/vmlinux` and `/sys/kernel/btf/zfs`, plus `uname --kernel-release`.
- `pahole --version` and `pahole --btf_base /sys/kernel/btf/vmlinux -C ... /sys/kernel/btf/zfs`
  for `zfs_cmd`, `zfs_ioc`, `dmu_objset_stats` and `zfs_cmd_t`.
- `bpftool version` and `bpftool btf dump file /sys/kernel/btf/zfs format raw`
  (read through `grep`, so the dump's own exit was `SIGPIPE`, not a parse result).
- Copies of the two sysfs BTF files into the scratch folder,
  then `pahole` and `bpftool` on the copies, with and without an explicit base,
  to record what a file without its base does.
- A scratch Rust program depending on `btf-rs` 2.0.1, built in the scratch folder,
  that parses `/sys/kernel/btf/vmlinux` with the mmap backend and
  `/sys/kernel/btf/zfs` as a split file with the cached backend,
  then prints the `zfs_cmd`, `dmu_objset_stats` and `zfs_ioc` types.
- The same program on truncated prefixes of the module file,
  to see whether the reader refuses or stalls.
- A small Node script that reads the two BTF headers and checks
  that the section lengths sum to each file's size.

Not run:

- No VM, no container, no ZFS command that changes anything,
  no `/dev/zfs` open, no pool, dataset, mount or unit change.
- No measurement of the checker's start cost, memory, or behaviour on a rebuilt module.
- No test of the built binding, the gate, the handler or the wrapper.
  None exists yet.

## The open item and the decisions that bound it

### The open item

The proposal's item, in its words:

> The runtime layout checker. The answer's words: "Runtime layout check". Comparing the loaded module's field layout
> with the approved release and request contract at runtime needs a parser, a set of accepted values, diagnostics
> and tests, and none of them is designed here, so the dependency that parser needs is chosen with it. The vet's BTF
> field comparison this design keeps is a test-time check, not this start-time one
> ([ABI gate and tests][design-abi-gate]).

The plan marks it "step 6, blocking, writer design work;
its design picks the profile dependency, which goes to the owner with that design"
([Open items and the step that needs each][plan-open-items]),
and lists the design among the three that can start beside steps 1 to 5
([What can start now][plan-start]).

### The owner's answer and the decisions around it

The answer, in the record's words
([ABI verification][design-project-decisions]):

> ABI verification. Decided: "Runtime layout check (Recommended)": verification of the field layout at runtime
> against the approved release and request contract, re-ranked after the kernel-release readiness answer. Exact
> approved kernel/module BTF profiles were the alternative the project did not select, and they would have
> disabled clones after a rebuild. Runtime checking can accept more builds but adds a parser and a larger
> compatibility test surface, and that checker has no design yet
> ([Open items the decisions leave][design-open-items]). Neither verifier was exercised by
> [the fixture][vm-fixture]; the test plus runtime gate decision stands.

These decisions bound what the checker may do:

- The runtime gate itself is decided. At start, before the first ioctl, both binaries
  "must prove compatibility with the loaded module and target ABI, including module release, architecture,
  endianness, request contract and kernel/module BTF layout. A missing device, missing required BTF or mismatch
  disables raw requests, including reads; no force flag. Unverified architectures, including `linux-arm64` until
  separately tested, do not qualify."
  On refusal "the wrapper uses the applicable same-dataset reflink or ordinary Git route for an unused slot",
  reports what failed, and keeps registrations and recovery state
  ([ABI gate and tests][design-abi-gate]).
- The checker verifies the field layout at runtime against the approved release and request contract.
  It is not an allowlist of kernel releases: the project chose "Probe at start" for that, and
  "The runtime module gate of "A: Test plus runtime gate" stays"
  ([Kernel-release readiness][design-project-decisions]).
- The reader lives in the binding. The project chose "Our own small binding (Recommended)",
  with `package/zfs/request-to-openzfs-kernel-device`, and both binaries "use the same binding and compatibility
  gate, even for reads". The binding "owns the `/dev/zfs` request format", and the verification method is decided
  while "the profile-checking dependency that checker needs is chosen with its design, which has none yet"
  ([Binding crate and compatibility][design-binding]).
- The approved release is OpenZFS `zfs-2.4.4`; the request numbers and struct layout are the ones in its headers
  ([Requests and input ownership](cli-git-zfs-worktree-design-proposal.md#requests-and-input-ownership)).
- The test-time BTF comparison the design keeps is separate: it compares the binding's compiled layout with the
  running module's BTF and the pinned headers in a test
  ([ABI gate and tests][design-abi-gate]).
  The runtime checker is the start-time one, and it runs where the test does not.
- `doctor` must report an ABI refusal as "the loaded module and target plus the supported compatibility evidence,
  not a bypass" ([What doctor checks and prints][design-doctor]).
- A failed gate "does not authorize an unrelated mount or cleanup";
  registrations and durable recovery state stay
  ([ABI gate and tests][design-abi-gate]).
- The build owes a start-time test: "Run it against a module whose field layout differs from the approved release
  and assert raw requests are disabled with no force flag"
  ([Decided behaviours with no test yet][design-decided]).

### What the decisions leave this design to settle

The decisions do not name:

- the reader and its dependency;
- how the module release string is matched to the approved release;
- which parts of the layout are compared, and how a missing or unreadable file is distinguished;
- the accepted set of values (the compiled expectations) and how they stay in step with the binding;
- what a refusal says, and how `doctor` shows it without becoming a second bypass;
- where the checker reads, in which order, and what it does if the module reloads between reads;
- how the check is cached in a process, and whether any cache is written to disk;
- the tests, fixtures and fuzz target, and the measurements that qualify them.

## Facts

Each fact names its source. A source line is a line in the file at the version named;
a measurement is in the measurements record or in a log the record names;
a probe is a command this session ran, listed under
[What was run, and what was not](#what-was-run-and-what-was-not).

### The module release string

- OpenZFS's `META` at `zfs-2.4.4` reads `Version:       2.4.4` (`META:4`) and `Release:       1` (`META:5`).
- The Linux module publishes that pair as its version:
  `MODULE_VERSION(ZFS_META_VERSION "-" ZFS_META_RELEASE)` at
  `module/os/linux/zfs/zfs_ioctl_os.c:386`, so the sysfs attribute reads `2.4.4-1`.
- A build from a git checkout overwrites the release with `git describe`'s suffix,
  with `tr - _`, when a matching tag exists (`config/zfs-meta.m4:73-88`).
  A tarball build keeps the `META` value, and a distribution may set its own:
  the guest's packaged module reported `2.2.2-0ubuntu9.5`
  (`~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/guest-prepare.log:469`),
  while the tarball build the same record used reported `2.4.4-1`
  (`item-1-exact.log:8-11`).
- libzfs reads the attribute with `fopen(ZFS_SYSFS_DIR "/version", "re")` and one `getline`,
  stripping the newline (`lib/libzfs/os/linux/libzfs_util_os.c:224-241`).
  The attribute is a page-sized sysfs file: on this host and in the guest it is 4096 bytes,
  mode `0444`.
- Probe: `/sys/module/zfs/version` on this host reads `2.4.4-1`,
  the host kernel is `7.3.5-1-cachyos`, and the account can read the file.

### The BTF files the kernel exposes

- `/sys/kernel/btf/vmlinux` exists when the kernel was built with BTF:
  the bin attribute is created with mode `0444` and size `__stop_BTF - __start_BTF`,
  and not at all when that size is zero (`kernel/bpf/sysfs_btf.c:13-43`, Linux `v6.8`).
- `/sys/kernel/btf/<module>` exists only for a module that carries BTF and only with
  `CONFIG_DEBUG_INFO_BTF_MODULES`: the notifier is compiled under that symbol
  (`kernel/bpf/btf.c:7390`), it skips a module whose `btf_data_size` is zero (`:7418-7422`),
  parses the module BTF (`:7435`), and registers a `0444` bin attribute of that size
  (`:7466-7478`). Unloading the module removes it (`:7503-7506`).
  The symbol is `def_bool y` under `DEBUG_INFO_BTF && MODULES && PAHOLE_HAS_SPLIT_BTF`
  (`lib/Kconfig.debug:410-414`).
- `CONFIG_MODULE_ALLOW_BTF_MISMATCH` can load a module whose BTF does not match vmlinux
  "without BTF" where the default rejects the module (`lib/Kconfig.debug:416-424`).
  A module that fails its BTF check in that mode registers no file
  (`kernel/bpf/btf.c:7435-7451`).
- Probe on this host: both files exist, mode `0444`;
  `/sys/kernel/btf/vmlinux` is 6,579,400 bytes and `/sys/kernel/btf/zfs` is 570,234 bytes.
  The account reads both.
- A module BTF is validated when the kernel loads it, so a file the kernel exposes
  has passed the kernel's own structural check for that build.

### A module BTF needs the vmlinux base

- A module BTF is split BTF. `btf_parse_module` sets its base to the vmlinux BTF,
  its first type id to `base_btf->nr_types`, and its first string offset to
  `base_btf->hdr.str_len` (`kernel/bpf/btf.c:5985-5987`).
  Name lookup walks to the base when an offset is smaller than that start
  (`kernel/bpf/btf.c:748-772`), so a module's own names are encoded past the base's
  string length and its type ids are past the base's type count.
- Probe: `pahole --btf_base <vmlinux> -C zfs_cmd <module>` works,
  and `pahole -C zfs_cmd /sys/kernel/btf/zfs` works on the sysfs path without an explicit base.
  Once the module file is copied out of `/sys/kernel/btf`,
  `pahole -C zfs_cmd ./zfs-module.btf` fails with
  `libbpf: Malformed BTF string section, did you forget to provide base BTF?`,
  and `bpftool btf dump file ./zfs-module.btf` fails with
  `failed to load BTF ...: Invalid argument`. With `--base-btf ./vmlinux.btf`,
  `bpftool btf dump file ./zfs-module.btf --base-btf ./vmlinux.btf format raw` works.
  A reader of a module file therefore needs both files, and its API must accept a base.
- Probe through `btf-rs` 2.0.1: the base reports 151,181 types;
  the module occupies the range 151,182 to 164,322 (13,141 types),
  and resolving `zfs_cmd` in the split object returns its struct.

### The BTF header grew after 6.8

- Linux `v6.8`'s header is 24 bytes: magic `0xeB9F`, version 1, flags, `hdr_len`,
  then `type_off`, `type_len`, `str_off`, `str_len`
  (`include/uapi/linux/btf.h:8-22`). Its `info` word packs 16 bits of `vlen` and
  5 bits of kind (`:34-57`).
- The current default branch's header is 32 bytes: it adds `layout_off` and `layout_len`,
  and a layout section describes each kind's element sizes (`include/uapi/linux/btf.h:12-34`).
  Its `info` word packs 24 bits of `vlen` and 7 bits of kind (`:51,70-72`).
- Probe on this host: both files report `hdr_len` 32 and version 1.
  `/sys/kernel/btf/vmlinux` reports type length 3,789,716, string offset 3,789,796,
  string length 2,789,572, layout offset 3,789,716 and layout length 80,
  and its sections sum to the file size.
  `/sys/kernel/btf/zfs` reports type length 329,244, string offset 329,244,
  string length 240,958 and zero layout offset and length, and its sections sum to the file size.
  A reader that assumes `hdr_len` is 24 misreads both files.
- The kernel's own header check accepts a longer `hdr_len` only as far as it understands,
  requires extra bytes to be zero in `v6.8` (`kernel/bpf/btf.c:5249-5294`),
  and the section check requires contiguous, non-overlapping sections that end at the file size
  (`:5195-5245`).
- The header's shape follows the tool that emitted the BTF, not the kernel that loaded it:
  the current tool writes the 32-byte header this host's files carry, and an older one wrote 24.
  The checker accepts both and does not infer a kernel release from the header.

### What the BTF carries

- `struct btf_type` carries a name offset and an `info` word; `struct`, `union`, `enum`,
  `enum64` carry `vlen` entries after it, and `struct`/`union` members carry a name offset,
  a type id and a bit offset (with bitfield size when the kind flag is set)
  (`include/uapi/linux/btf.h:23-150`, Linux `v6.8`).
- Probe with `pahole` and with `btf-rs` 2.0.1 on this host's module:
  `struct zfs_cmd` is 13,744 bytes with 32 members,
  including `zc_name` at bit 0 with size 4096, `zc_nvlist_src` at bit 32,768,
  `zc_share` at bit 101,376, `zc_objset_stats` at bit 101,632,
  `zc_begin_record` at bit 103,936, an anonymous union member at bit 106,368,
  `zc_cleanup_fd` at bit 109,312 and `zc_zoneid` at bit 109,888.
  The union's members are `zc_inject_record` at 13,296 with size 368 and an anonymous struct
  holding `zc_pad1[352]`, `zc_defer_destroy` at 13,648, `zc_flags` at 13,652 and
  `zc_action_handle` at 13,656.
  `dmu_objset_stats` is 288 bytes with `dds_creation_txg` at bit 64,
  `dds_guid` at bit 128, `dds_type` at bit 192, `dds_is_snapshot` at bit 224 and
  `dds_origin` at bit 248 with size 256.
  `enum zfs_ioc` has 104 values, including `ZFS_IOC_FIRST = 23040`,
  `ZFS_IOC_OBJSET_STATS = 23058`, `ZFS_IOC_DESTROY_SNAPS = 23099`,
  `ZFS_IOC_CLONE = 23106`, `ZFS_IOC_GET_BOOKMARKS = 23108`,
  `ZFS_IOC_CHANNEL_PROGRAM = 23112` and `ZFS_IOC_LAST = 23177`.
- The anonymous union and struct members have an empty BTF name,
  so a reader cannot find `zc_defer_destroy` by walking named members alone:
  it must descend into the anonymous member by kind and position.
  The probe reaches the union and its anonymous struct through `btf-rs` and reads
  `zc_defer_destroy` at 2,816 bits into the union, which is 109,184 bits from the struct's start.
- BTF records no alignment and no field name for an anonymous member.
  Size and member offsets are the layout facts it carries.

### The whole struct is copied per request

- On Linux, each ZFS ioctl copies the whole `zfs_cmd_t` in and out:
  `ddi_copyin(..., sizeof (zfs_cmd_t), 0)` and
  `ddi_copyout(zc, ..., sizeof (zfs_cmd_t), 0)`
  (`module/os/linux/zfs/zfs_ioctl_os.c:139-150`), and the request number is an index
  into a table sized `ZFS_IOC_LAST - ZFS_IOC_FIRST`
  (`module/zfs/zfs_ioctl.c:7442-7449`).
- The struct is declared at `include/sys/zfs_ioctl.h:509-562`, and the module asserts its size
  is `(MAXPATHLEN*3)+MAXNAMELEN+1200` (`:565-575`).
  On Linux `MAXPATHLEN` is 4096 and `MAXNAMELEN` is 256
  (`include/os/linux/spl/sys/sysmacros.h:68-69`), which is 13,744 bytes.
- The request numbers are `enum zfs_ioc` at `include/sys/fs/zfs.h:1474-1590`;
  `ZFS_IOC_FIRST` is `('Z' << 8)` on Linux (`:1480-1481`).
- The 2.2.2 to 2.4.4 change kept the struct size and moved the defer fields into a union with a
  `zinject_record_t` that grew from 352 to 368 bytes
  ([the vet][vet], "Layout against OpenZFS 2.4.4",
  lines 430-446). A check that compares the total size and the named offsets alone
  accepts that pair; a check that also compares member type sizes refuses it.

### When the module carries no BTF

- The kernel's module build generates module BTF with
  `pahole -J ... --btf_base vmlinux $@` and skips it, with a printed reason,
  when `vmlinux` is not in the build directory
  (`scripts/Makefile.modfinal:37-43`); the rule's prerequisites are guarded by
  `CONFIG_DEBUG_INFO_BTF_MODULES`, `KBUILD_BUILTIN` and `vmlinux` (`:54-57`).
- The design's own guest built OpenZFS 2.4.4 from the release tarball against
  `linux-headers-6.8.0-147-generic`, and its log records
  `BTF [M] .../module/zfs.ko` followed by
  `Skipping BTF generation for .../module/zfs.ko due to unavailability of vmlinux`
  (`~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/guest-openzfs-build.log:834-838`).
  That guest's module reported `2.4.4-1` (`item-1-exact.log:8-11`), and the module that log
  built carries no BTF; whether the guest still exposed a file from another module was not
  recorded.
- Whether the same guest's packaged `2.2.2-0ubuntu9.5` module carried BTF was not recorded.
  A distribution package may carry it, and an out-of-tree build without a `vmlinux`
  in its build directory does not.
- The consequence for the accepted design's own fixture:
  the tarball module the measurements record's guest built carries no BTF,
  so a runtime layout checker that requires it disables raw requests there,
  and the build's acceptance arms need a guest whose loaded module carries BTF.

### What the readers did on this host

- `pahole` 1.32 with `--btf_base` prints `zfs_cmd` and `zfs_ioc` from the sysfs path,
  and prints `zfs_cmd` from a copied module file only when given the base.
- `bpftool` 7.8.0 on libbpf 1.8 prints the module's raw type listing,
  numbering module types from the base count (`[151182]` and up), with the base file
  required once the module file leaves `/sys/kernel/btf`.
- `btf-rs` 2.0.1, built in the scratch folder, parses the pair and returns the type,
  name and member data listed under [What the BTF carries](#what-the-btf-carries).
  With the base through the mmap backend and the module through `from_split_file`,
  it resolves a module type by name.
- The same program on truncated module prefixes refuses each one with
  `Could not parse string: data provided does not contain a nul`;
  it neither panics nor stalls.
- The published `btf-rs` 2.0.1 tree does not contain the widened `vlen` and kind masks
  that its default branch carries at `cdf9ea46` (2026-04-21):
  the tag's `btf-rs/src/cbtf.rs:269-275` still masks `vlen` with `0xffff` and kind with `0x1f`,
  while the branch's file uses `0xffffff` and `0x7f`.
  GitHub's comparison of `cdf9ea46` with `v2.0.1` reports the histories diverged.
  Today's BTF kinds and member counts fit the old masks, and the probe parses both files,
  but the release is behind its own branch on this point.

### The wrapper's parts this design uses

- The wrapper's compatibility precedent warns and continues for an old Git,
  and stops only when the version cannot be read, with an `engine-failure` event,
  code `content-unavailable` and exit `2` (`package/git-policy/cli/SPEC.md:18651-18680`).
  The ZFS gate is fail-closed instead, so it uses the same reporting shape but refuses.
- The version check caches its answer in a `OnceLock`
  (`package/git-policy/cli/src/native/invocation/git_version.rs:23-60`);
  the checker follows that shape per process.
- The doctor's filesystem section gathers facts apart from the wording and never changes
  the exit status (`package/git-policy/cli/src/native/health/doctor_filesystem.rs:1-30`).
  The checker's `doctor` output follows that split.
- The wrapper's no-crossing walk exists for trees the account can change
  (`package/git-policy/cli/src/native/subvolume/fs_beneath.rs:1-30`).
  The checker reads `/sys/module/zfs/version` and `/sys/kernel/btf/*`, which root controls,
  so it uses plain absolute opens with `O_NOFOLLOW` and does not use that walk.
- The wrapper's license is `LGPL-3.0-or-later` (`package/git-policy/cli/Cargo.toml:6`),
  and its release artifacts are `x86_64` and `aarch64` GNU archives
  (`toolchain.lock.json:614-653`).

## Assumptions

- The layout compiled into the binding is the layout in the pinned `zfs-2.4.4` headers.
  Step 2's test is what proves it, and that test does not exist yet.
- A module whose layout matches the compiled expectations behaves as the approved release for
  the requests the binding sends. The checker proves layout and request numbers, not code.
- The account and the root handler can read `/sys/module/zfs/version` and `/sys/kernel/btf/*`
  in the deployments the design describes. A masked `/sys` is a refusal, not a pass.
- A module BTF that the kernel exposes is structurally valid, because the kernel parsed it at load
  (`kernel/bpf/btf.c:7418-7478`). The checker still refuses a short read or a header that does not
  add up.

## Limits

- The checker cannot prove nvlist key names, property names or handler behaviour.
  A kernel that expects another key for the same request fails at the ioctl,
  and no layout fact shows it.
- It cannot tell a patched module from the approved one when the patch keeps the same layout.
- It proves architecture only as the compiled target and an allowlist of tested targets;
  BTF records no architecture. `linux-arm64` stays refused until a guest qualifies it.
- It cannot run without module BTF. On a build without it, the managed route is unavailable,
  and the fallback is the design's.
- Alignment is not in BTF. Equality of size, member offsets and member sizes is the proof,
  and it is the same evidence the kernel's copy needs.
- The checker's expectations are only as good as the binding's struct.
  A wrong expectation refuses a good module; a missing expectation passes an unproven field.
- The 32-byte header path is proven on this host's files by three readers,
  not in a guest, and not on a kernel built with an older header.
- The header's widened `vlen` and kind fields are newer than `v6.8`;
  a reader built on the 5-bit masks cannot represent a future file that uses them.
  The values in today's files fit, as the probe shows.

## What the checker must prove

The gate's five proofs, and what each one means here:

- Module release. The loaded module identifies the approved release.
  Which strings qualify is [Q2](#q2-which-module-release-strings-qualify).
- Architecture. The binary runs on a target the project has qualified for the managed route.
  Today that is `x86_64`; `linux-arm64` is refused.
- Endianness. The module's BTF byte order matches the target's.
- Request contract. Every request number the binding can send has the value the approved
  `zfs_ioc` enum gives it, and the range's endpoints match.
- Kernel and module BTF layout. The loaded module's `zfs_cmd` has the size, member offsets,
  member sizes and nested shapes the binding compiled, and the base types its members use have
  the sizes the binding assumes.

The gate runs before the first ioctl in both binaries. A missing device, a missing file,
an unreadable file, a short read, a parse failure or any mismatch disables raw requests,
including reads, and the command takes the design's fallback for an unused slot.
There is no force flag and no environment override.
The refusal keeps registrations and recovery state.

## Ways to meet it

Each way is a different way to prove the five items under
[What the checker must prove](#what-the-checker-must-prove).
Costs are ranges of new source and test lines, not schedules.
No way was built or measured.

### Way 1: parse in process and compare the declared layout

Read the module's BTF (with its vmlinux base) in process, and compare it with the expectations
compiled into the binding: the struct size, every member's offset and size, the nested shapes the
codec uses, the base type sizes and the request numbers.

Pros:

- The whole proof runs in the wrapper's address space: no external program, no root PATH surface,
  and a refusal is a value the caller renders.
- The comparison is exact where the kernel's copy is exact: size, every offset and every size.
  It refuses the 2.2.2 shape even though its total size and named offsets would pass.
- Both binaries share it, as the design requires.
- The refused facts are structured, so `doctor` and the caller print the same evidence.

Cons:

- It adds a parser or a parser dependency to the binding, and a pre-scan that the parser's input
  bounds need.
- The binding grows an expectation table that must stay in step with its struct and its request
  list; an exhaustiveness test is the guard.
- The check costs a read of the base BTF, which is larger than the module's.

Cost to build and test:

- With a reader dependency: the dependency row, a bounded loader and pre-scan, the expectation
  table, the comparison and the diagnostics, about 350 to 550 lines of source;
  about 300 to 450 lines of tests; one fuzz target for the loader and pre-scan.
- The same shape with an in-house reader adds the reader and its model,
  about 700 to 1,000 lines of source and 400 to 600 lines of tests.

What it needs measured: items 1, 2, 3, 4, 6, 7, 9 and 10 under
[Owed measurements](#owed-measurements): a pass, the true-positive refusal on the 2.2.2 shape,
a missing module BTF, the header variants, the reload race and the start cost.

### Way 2: parse in process and compare a narrow set

The same in-process reader, but the expectations cover only the struct size, every direct
member's bit offset, and the offsets (not the sizes or the anonymous shapes) of the nested fields
the codec reads.

Pros:

- A smaller table and fewer refusal paths than Way 1.
- It catches a moved field the codec uses and a changed total size.

Cons:

- It accepts the 2.2.2 shape: its total size and named offsets match 2.4.4, while
  `zinject_record` is 352 rather than 368 bytes and the anonymous union does not exist there.
  The kernel still copies 13,744 bytes, so the accepted shape is not the approved one.
- A nested type change that moves no direct member passes, which is the class of change
  the runtime check exists to catch.

Cost to build and test:

- About 250 to 400 lines of source and 200 to 300 lines of tests with a reader dependency.
- The same reader and loader cost as Way 1, so the saving is small against the loss.

What it needs measured: item 2, where the 2.2.2 shape must pass for this way to be described
honestly (and does, because the offsets match), plus items 1 and 10. That result is the reason
this way is ranked second, not a reason to trust it.

### Way 3: start an external tool

Spawn `pahole` or `bpftool` at start, parse the type listing it prints, and compare it.

Pros:

- The parser is the tool's: `pahole` is the command the design already names for the test-time
  comparison, and `bpftool` carries libbpf's reader.
- No new Rust dependency in the binding.

Cons:

- The product gains a runtime program it must find, verify and version.
  A missing tool would look like an ABI refusal unless the checker adds its own step for it.
- The root handler would execute an external binary; its path and library resolution become
  part of the privileged boundary the design otherwise keeps small.
- Text output is not a machine interface: `pahole` prints cacheline comments and a human layout,
  `bpftool`'s raw dump needs a type-id resolver built in the wrapper anyway.
- The tool's version drifts with the distribution, and the tool must be present in every
  deployment, including the smallest.

Cost to build and test:

- A spawn wrapper and output parser per tool, about 200 to 300 lines of source and
  250 to 400 lines of tests.
- A packaging and installation requirement in the host and guest fixtures, and a version matrix
  per tool release in the VM.
- A path-verification test for the root handler's invocation.

What it needs measured: items 1, 2 and 10, plus item 13 for the tool's output on a compatible and
a mismatched module and its absence path, and item 14 for the root handler's verified path.

### Way 4: compare a BTF digest

Hash the module BTF (or a selected part of it) and compare with a value baked into the release.

Pros:

- No parser and no per-member table; a single comparison.
- A rebuilt module with any difference at all is refused.

Cons:

- It is the alternative the project did not select: "Exact approved kernel/module BTF profiles were
  the alternative the project did not select, and they would have disabled clones after a rebuild"
  ([ABI verification][design-project-decisions]).
  A digest refuses every rebuild, including one that keeps the approved layout.
- It proves nothing about which field changed, so the refusal cannot name the field.

Cost to build and test:

- A digest table for each approved build and the release step that fills it, about 50 lines of
  source, plus item 15. The digest must be regenerated for every approved build, which is the
  maintenance the answer rejected.

This way is bounded out by the accepted answer. It is listed so the boundary is visible.
What it needs measured: item 15, where a rebuild that keeps the approved layout must be shown
refused, which is the cost the answer rejected. No reason to build it otherwise.

### Way 5: prove the size with a request

Issue one harmless request whose name is known to fail, with a user buffer larger than any
candidate struct, and read the error: a kernel whose struct fits the buffer copies the request in
and out and returns the name error.

Pros:

- No BTF reader at all, and it proves the end-to-end copy on the running kernel.
- Small: one request and one error interpretation.

Cons:

- It proves only that the kernel's struct fits the buffer it was handed and that dispatch
  reached a handler. A moved field, a changed member size, a renumbered request or a kernel
  struct larger than the buffer passes or corrupts, and no error names the difference.
- It needs `/dev/zfs`, so it cannot run where the device is missing, and its request choice must be
  side-effect free on every pool state.
- It overlaps the start probe, which the design asks for separately for kernel releases.

Cost to build and test:

- About 50 lines of source and a device arm in the guest.
- It cannot replace the layout check; it is at most a control beside Way 1.

What it needs measured: item 16, where a moved field of the same size must be shown passing it,
and the buffer bound it can claim. A pass alone says nothing.

### Way 6: keep the release string check only

Refuse when the module version is not the approved string, and do not check the layout.

Pros:

- No parser, no BTF dependency, the smallest gate.

Cons:

- It is what the design had before the answer, and the answer added the layout check.
- A distro or patched build that keeps `2.4.4` and changes a field passes, and the kernel copies
  the wrong shape.

Cost to build and test:

- The version read and the release rule, about 20 lines of source, plus a test of the rule.
  It is the gate the design had before the answer.

This way is bounded out by the accepted answer.
What it needs measured: item 16, where a patched module that keeps the version and moves a field
must be shown passing it. No reason to build it otherwise.

### Ranking

Ranking among the ways the decision leaves open: Way 1 > Way 2 > Way 3 > Way 5.

- Way 1 beats Way 2 because the kernel copies the whole struct and the 2.2.2 to 2.4.4 change
  kept the total size and named offsets while changing `zinject_record` from 352 to 368 bytes.
  Way 2 accepts that shape; Way 1 refuses it for one more comparison per member.
- Way 2 beats Way 3 because Way 2 keeps the proof in process, with no runtime program and no
  root PATH surface, and the tool's text formats need a resolver in the wrapper anyway.
- Way 3 beats Way 5 because Way 3 proves field offsets and member sizes; Way 5 proves only the
  copied size and dispatch, and it needs the device.

Way 4 and Way 6 are bounded out by the accepted answer, so they are not ranked.
Way 4 would refuse every rebuild, and Way 6 is the gate the answer replaced.

The reader inside Way 1 is the owner's pick;
the ranked reader options are
under [The profile-checking dependency](#the-profile-checking-dependency).

## The shape this proposal designs

This section proposes the shape of Way 1. It is a proposal, not a decision.
The reader is a seam: either a dependency or an in-house reader can fill it,
and the rest of the shape stays the same.

### The seam and the placement

The binding package owns the request format, the `zfs_cmd` representation and the codec
([Binding crate and compatibility][design-binding]),
so the binding owns:

- the bounded file loader and the pre-scan;
- the expectations derived from its `#[repr(C)]` type and its request list;
- the comparison and its refusal value.

The wrapper and the handler own:

- the three paths, the read order and the per-process cache;
- the target allowlist (architecture) and the module release rule the owner picks;
- the words the caller and `doctor` print.

Both binaries call the same entry point, as the design requires for reads as well as writes.

### The read order and the reload race

The gate, for a caller that needs the device:

1.  Check that `/dev/zfs` exists and that the caller can open it.
    A missing or unopenable device refuses with that step and starts no read.
2.  Read `/sys/module/zfs/version` once, bound to one page, and strip one trailing newline.
3.  Read the two BTF files, each opened with `O_RDONLY | O_CLOEXEC | O_NOFOLLOW`,
    each read to a bounded size, each checked against its own header and file size.
4.  Read the version file again. If it differs, retry the whole check once,
    then refuse with "module changed while checking".

`doctor` runs the same facts without step 1, so it can report what a caller would meet.

The second version read closes the window in which the module unloads or is replaced between
the first read and the BTF reads. It does not close the window during a single read;
the header and size checks refuse a short or inconsistent file.

### The bounded pre-scan

Before any parser sees the bytes, the loader checks:

- the magic is `0xeB9F` in one byte order, the version is 1, and the flags are zero;
- `hdr_len` is at least the 24 bytes of the oldest header and at most the read bound;
- the type, string and, if present, layout sections are in the header's order, contiguous
  with no gap or overlap, and end at the read length;
- the string section ends the file, as the kernel requires (`kernel/bpf/btf.c:5160-5178`);
- every type entry's length is computable with the widest encoding
  (24-bit `vlen`, 7-bit kind), and any value the chosen reader cannot represent is a refusal,
  not a misread.

The last check matters for the reader chosen at `2.0.1`: its released masks are the older
5-bit and 16-bit ones, so a future file that uses a wider value must be refused before it is
parsed. The check is a linear walk that skips each type by its entry length, not a second parser.

### The expectations

The expectations come from the binding's own type, so one change moves both sides:

- `size_of::<ZfsCmd>()`, and `offset_of!` for every field the Rust type can name;
- the union and anonymous members, whose Rust side has named fields and whose BTF side has an
  empty name: a path step names the kind and the expected bit offset, then the inner names;
- the sizes of the nested types the codec reads, including `zinject_record` (368),
  `dmu_objset_stats` (288) and the fields listed under
  [What the BTF carries](#what-the-btf-carries);
- the base integer sizes the members use;
- every request constant in the binding's request list, plus `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.

A path grammar, as it would live in the binding:

```text
# package/zfs/request-to-openzfs-kernel-device/src/layout/expectations.rs
zfs_cmd
  zc_name              [0, 4096]          # [bits from outer start, bytes]
  zc_nvlist_src        [32768, 8]
  ...
  <anon-union @ 106368>
    zc_inject_record   [106368, 368]
    <anon-struct @ 106368>
      zc_pad1          [106368, 352]
      zc_defer_destroy [109184, 4]
      zc_flags         [109216, 4]
      zc_action_handle [109248, 8]
  ...
```

The numbers in the table are bits from the start of the outer struct, with a byte size beside
each entry. BTF gives a member's offset relative to its container, so the comparison sums the
container offsets along the path: `zc_defer_destroy` is 2,816 bits into the union at 106,368,
which is 109,184 bits from the struct's start. A hand-written table of literals would drift;
the table is generated from the Rust type at compile time where the language allows it,
and the few members it cannot express (the anonymous containers and union members) carry a
literal with a test that recomputes them from the type's size and the neighbours.

An exhaustiveness test fails when the struct gains a field with no path, or the request list gains
a constant with no expected value, so a new field cannot silently go unchecked.

### The comparison

For each expected entry, resolve the BTF type and compare:

- the struct's size with `size_of::<ZfsCmd>()`, in bytes;
- each member's bit offset with the expected bit offset, and the member type's size with the
  expected byte size;
- an anonymous container by its kind and bit offset, then its inner members by name and offset;
- each base type size the member types resolve to;
- each request constant with its `zfs_ioc` enumerator, by name and value;
- `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.

The comparison stops at the first mismatch and reports its path, expected and actual values.
It also reports a missing type, a missing enumerator, and a member the expectations do not cover
when the table's count check fails.

Size comparison is not decoration: the 2.2.2 module has the same total size and the same named
offsets for the defer fields, and only `zinject_record`'s size separates it from 2.4.4
([the vet][vet], lines 430-446).
A checker that compares offsets only would pass it.

### The refusal

One value carries the outcome, with the step that failed, the file it read, and the detail:

```text
# package/zfs/request-to-openzfs-kernel-device/src/layout/refusal.rs
Refused {
  step: Device | Version | BaseBtf | ModuleBtf | Header | Type | Member | Enum | BaseType,
  file: Option<PathBuf>,
  detail: String,            # the field path, the expected and actual values, or the read error
}
```

The caller prints the loaded module's version string, the target architecture, the files it read
and the failing step, and takes the design's fallback for an unused slot.
It never offers a bypass. `doctor` prints the same facts, with the healthy case as an explicit
"compatible" line naming the module and the file sizes, and does not change its exit status.

### The cache

The result is cached per process in a `OnceLock`, following
`git_version.rs`. Before each use, the cache's identity is rechecked against the version text and
the two files' identity (device, inode, size and modification time); a change re-runs the check.

No verdict is written to disk. A disk cache of a compatibility verdict would have to be bound to
the boot, the module instance and the binary's build, and a forged one in the account's own cache
would enable raw requests on a mismatched module. If the measured start cost is too high,
[Q3](#q3-may-a-verification-be-cached-across-processes) puts the alternatives to the owner.

### What the shape does not cover

- The nvlist keys and the handler's behaviour.
- A patched module with the approved layout.
- A base BTF whose primitive sizes are checked but whose other types the checker does not read.
- A file the kernel exposes that is internally inconsistent in a way its own load check accepted;
  the pre-scan refuses only what it can see.

## The profile-checking dependency

This section follows
[`choosing-technology`][skill]:
hard constraints, a candidate ledger with a source audit per candidate, hard-gate outcomes,
a scored comparison of the finalists, and a sensitivity pass.
The pick is the owner's, so this section ends with ranked options and no choice.

The skill's report artifact (`doc/audit/tech-...-vet-YYYY-MM-DD.md`) and its
pre-discovery check-in are not done, because this brief permits edits to this file only and
allows no questions. Both are owed before adoption; see
[The vet artifacts this brief could not write](#the-vet-artifacts-this-brief-could-not-write).

### Method and hard constraints

Base category for every candidate: inspectable open-source local technology.
Overlays: high-trust execution (the root handler runs the same check),
native boundary (a parser of kernel-produced bytes), and multi-platform claim
(the wrapper's two GNU release targets).

Hard constraints, each with the design sentence it comes from:

- H1. Parses split BTF from a module file with its vmlinux base, as
  [A module BTF needs the vmlinux base](#a-module-btf-needs-the-vmlinux-base) shows.
  The design's start-time layout proof needs module types whose ids and names cross into the base.
- H2. License compatible with `LGPL-3.0-or-later` (the wrapper's license;
  the binding is a new crate in the same repository).
- H3. Builds for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` with no C build step
  and no load-time library from the distribution, because the wrapper's release is two GNU
  archives that must start on a machine without OpenZFS
  (`toolchain.lock.json:614-653`;
  [the vet][vet], "Where the wrapper ships and what it links today").
- H4. Reads a file the caller names, with no BPF syscall, no root requirement and no network.
- H5. Exposes named struct members with their bit offsets and type sizes, and enum members with
  their values, at the depth the anonymous union needs.
- H6. Source is inspectable and its provenance maps to the published artifact.

This screening applies H1 to H6 before any score. H5 is the reason a low-level `object`-style
ELF reader is not a candidate by itself: it exposes a section's bytes, not BTF types.
H3 constrains what the binding builds and links, and H2 what it may include; a spawned tool adds
no link, so the tools meet H3 and carry the runtime-program cost in C2 instead. Dropping H3 would
re-admit `libbpf-rs` and a `dlopen` of `libbpf`, which is a choice for the owner, not a fact.

### Candidate ledger

#### btf-rs 2.0.1

- Discovery: crates.io query `btf`, first result by recent downloads among parser crates.
- Version and revision: 2.0.1, tag `v2.0.1`, commit `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`,
  published 2026-11-12. License `LGPL-2.1-or-later`.
- Dependencies: `btf-rs-derive`, `byteorder`, `fallible-iterator`, `memmap2`, `thiserror`;
  optional `elf`, `regex`, `bzip2`, `flate2`, `liblzma` and `zstd` for features this use does not
  enable. The default feature set is empty. No C build.
- Source audit: 2,749 lines under `btf-rs/src`.
  Split support is explicit: `Btf::from_split_file` and `Btf::from_split_bytes`
  (`btf-rs/src/btf.rs:66-107`) take a base and reject a base that is itself split.
  The cached backend keys module strings past the base's string length and numbers types from the
  base's count (`btf-rs/src/section.rs:190-265`), and name lookup falls back to the base
  (`btf-rs/src/btf.rs:190-200`).
  The mmap backend is lazy and checks the string section against the file length
  (`btf-rs/src/section.rs:348-390`), but its `iter_over_names` compares an offset against the
  module's own string length (`:405-417`), which is wrong for a split file; split files do not use
  that backend, because both split constructors use `from_reader`.
  Kinds cover `ENUM64`, `DECL_TAG` and `TYPE_TAG` (`btf-rs/src/btf.rs:303-355`), and members carry
  bit offsets and bitfield sizes (`:583-598`).
  The header reader is 24 bytes and section offsets use `hdr_len`
  (`btf-rs/src/cbtf.rs:182-191`; `section.rs:182,206`), so a 32-byte header parses.
- Tests and CI: an integration test with checked-in BTF assets
  (`btf-rs/tests/integration_test.rs`), and one CI workflow running format, Clippy,
  documentation, six feature builds and the tests (`.github/workflows/ci.yaml:1-40`).
  No fuzz target.
- Maintenance: releases `v2.0.0` (2026-04-16) and `v2.0.1` (2026-11-12);
  commits through 2026-10-29; one merged fix for an out-of-bounds mmap name read
  (PR 42, merged 2026-10-29, included in 2.0.1); two open items, PR 37
  "Add kind layout support" and issue 1 "Add some BTF building capabilities";
  4 forks, a small maintainer group.
  The 2.0.1 tree lacks the widened `vlen` and kind masks that the default branch carries at
  `cdf9ea46`; the branches diverged, and the release lags its own main on that point.
- Probe: parses this host's pair, resolves `zfs_cmd` by name, walks the anonymous union to
  `zc_defer_destroy`, and returns 104 `zfs_ioc` values; refuses truncated inputs with an error.
- Gates: H1 pass (split constructors plus the probe), H2 pass (LGPL-2.1-or-later can be used
  under LGPL-3.0; the wrapper is already LGPL-3.0-or-later), H3 pass on source (pure Rust, default
  features off; the aarch64 build is unrun), H4 pass, H5 pass, H6 pass.
- Finalist.

#### btfparse 1.3.9

- Discovery: crates.io query `btf`.
- Version and revision: 1.3.9, commit `3788c4410a6cfd64ff57aa5bca20b2ee4eed3cd5`, 2026-01-08.
  License `Apache-2.0`.
- Dependencies: none. Edition 2024. 5,876 lines under `src/`.
- Source audit: one `TypeInformation::new` over one `Readable`, numbering types from 1
  (`src/btf/type_information.rs:435-490`); no base file, no split id or string offset.
  Its `id_of` and `offset_of` API is the closest to what the check wants
  (`:491-520`, `:607-761`), but it cannot represent a module file whose ids and names cross into
  the base.
- Tests and CI: unit tests embedded in each type module, with byte fixtures; five workflows
  (`check`, `clippy`, `format`, `main`, `posix_build`); no fuzz target.
- Gates: H1 fail. The rest were not scored.
- Exit.

#### aya-obj 0.6.0

- Discovery: crates.io query `btf`, by downloads.
- Version and revision: 0.6.0, `aya-rs/aya` at `44a6709164d2ac0c19c7845101e7b033e25b3f3f`.
  License `MIT OR Apache-2.0`.
- Dependencies: `bytes`, `log`, `object` (ELF), `thiserror`.
- Source audit: the BTF reader parses one object's `.BTF` section and has no base
  (`aya-obj/src/btf/btf.rs:340-365`; the `base` at `:355` is a byte offset, not a base BTF).
  It is written for eBPF object loading and relocations.
- Gates: H1 fail. Its `object` dependency is a fit concern, not a gate. Exit.

#### btf 0.5.1

- Discovery: crates.io query `btf`.
- Version and revision: 0.5.1, `arcjustin/btf` at `3b57cdab2ea80d13c3b5abd400bb2e4bfc0d6292`,
  last commit 2023-02-08. License `MIT`.
- Dependencies: `byteorder`, `thiserror` 1.
- Source audit: one blob parser with a 24-byte header and no base
  (`src/btf.rs:15-81`); no split support.
- Maintenance: no commit since 2023.
- Gates: H1 fail; the maintenance signal is noted but the gate is enough. Exit.

#### btfdump 0.0.5

- Discovery: crates.io query `btf`.
- Version and revision: 0.0.5, `anakryiko/btfdump` at `e5951d4b2e2e7ce6c36001c1b48c1c239c9759dc`.
  License `BSD-2-Clause`.
- Dependencies: `clap`, `object`, `memmap2`, `scroll`, `scroll_derive`, `regex`, `bitflags`.
- Source audit: a tool first, a library second; it dumps one BTF file and has no split base
  (`src/types.rs:1089-1120`, `src/lib.rs`). Its README names only an ELF file or a raw BTF file,
  not a module pair.
- Gates: H1 fail as a library; as a tool it is Way 3 with an extra package. Exit.

#### padlock-dwarf 0.82.0

- Discovery: crates.io query `btf`.
- Version and revision: 0.82.0, `gidotencate/padlock` at `6a7d045f2c6b0cb64549ddebe5825411e25178fa`.
  License `MIT OR Apache-2.0`.
- Dependencies: `padlock-core`, `gimli`, `object`, `pdb`, `miniz_oxide`.
- Source audit: a 794-line BTF reader inside a DWARF/PDB struct-layout analyzer,
  single blob, no base (`crates/padlock-dwarf/src/btf.rs:114-152`).
  The analyzer's output is a whole-program struct list, not the named type model the check needs.
- Gates: H1 fail; H5 fail for fit. Exit.

#### libbpf-rs 0.27.2 and libbpf through FFI

- Discovery: crates.io query `libbpf`, and repository precedent.
- Version and revision: 0.27.2, `libbpf/libbpf-rs` at
  `6d771ee9835fcf517dbcf1d29e25a97480425d45`. License `LGPL-2.1-only OR BSD-2-Clause`.
- Source audit: the safe wrapper around `libbpf-sys`; its default feature is
  `libbpf-sys/vendored-libbpf`, and the manifest says it "requires other necessary libs to be
  present on the system" (`libbpf-rs/Cargo.toml:20-26`).
- Gates: H2 fail for the `LGPL-2.1-only` arm, and H3 fail (C build and system libraries).
  A `dlopen` of the system's `libbpf` avoids the link but keeps the native provenance and adds a
  privileged library load in the handler, so H3 and H6 fail there too. Exit.

#### bpftool 7.8.0

- Discovery: this host (`/usr/bin/bpftool`), and the libbpf reader it carries.
- Version: 7.8.0 on libbpf 1.8; license of the tool is the kernel's (GPL-2.0 with BSD-2 parts),
  and it runs as a separate process.
- Source audit: the tool's `btf dump` reads one `BTF_SRC` with an optional `--base-btf`,
  and formats `raw` or `c` (`bpftool btf help dump`).
  Probe: on the sysfs path it resolves the base itself; on a copied file it needs `--base-btf`.
- Gates: H1 pass, H2 pass for a separate process, H3 not applicable as a library but the product
  needs the program at run time, H4 pass, H5 pass, H6 pass.
- Finalist as a Way 3 tool.

#### pahole 1.32

- Discovery: this host, and the design's test-time inspection command
  ("ABI gate and tests").
- Version: 1.32; license GPL-2.0 for the dwarves package, run as a separate process.
- Source audit: the documented `--btf_base` option and `-C` type printer
  (`pahole --help`); probe output for `zfs_cmd`, `zfs_ioc` and `dmu_objset_stats`.
  On a copied module it needs `--btf_base`; on the sysfs path it finds the base.
- Gates: H1 pass, H2 pass as a separate process, H3 as for `bpftool`, H4 pass, H5 pass
  (the printer gives member offsets and sizes), H6 pass.
- Finalist as a Way 3 tool.

#### Writing the reader in-house

- Discovery: repository precedent ("Our own small binding", the binding's own unsafe boundary),
  and the skill's existing-tools rule.
- Source: none; the model and the walker would follow `include/uapi/linux/btf.h` and the
  kernel's validation (`kernel/bpf/btf.c:5195-5325`).
- License: the repository's.
- Gates: H1 to H6 pass by construction, but the skill's rule is that
  "A custom implementation is eligible only when every existing tool fails a named hard constraint."
  `btf-rs` passes, so this candidate is eligible only as a fallback if the owner rejects the
  dependency on a named ground.
- Conditional finalist, scored for comparison.

#### Not candidates

- `zfsdev-rs`'s layout code: it hard-codes a layout and does not parse BTF, so it cannot serve a
  runtime check. The design already reuses its codec and layout code for the binding.
- `object`, `goblin`, `scroll` and `libelf`: they expose an ELF section or a byte stream, not BTF
  types, so they are building blocks of the in-house reader, not candidates.
- `trailofbits/btfparse` (C++): discovery found it; a C++ build fails H3.

### Discovery record

- crates.io `https://crates.io/api/v1/crates?q=btf&per_page=50`:
  50 rows returned of 81 total; new candidates: `btf`, `btf-rs`, `btfparse`, `btfdump`,
  `aya-obj`, `padlock-dwarf`; screening survivors: `btf-rs`, `btfparse`, `btfdump`, `aya-obj`.
- crates.io `?q=split%20btf&per_page=30`: 17 rows of 17; no new candidate beyond `btf-rs`
  and `aya-obj`.
- crates.io `?q=btf%20kernel&per_page=30`: 30 rows of 50; no new parser candidate.
- GitHub repository search `BTF parser in:name,description language:rust`: 7 rows;
  new: `luzhixing12345/btfparser` (a C function-declaration parser, not a Rust BTF library),
  `genksome/kxray` (an Android offsets tool); no new survivor.
- GitHub code search `from_split_bytes language:rust`: only `retis-org/btf-rs` held split BTF
  code; `likewhatevs/ktstr` has a vmlinux-only struct-field resolver in a larger tool, with no
  module base, and the other hits were unrelated `split_*` helpers.
- Web search `Rust crate parse BTF split BTF module vmlinux base BTF parser library`:
  44 result lines, 10 shown by the provider; new candidates: `bpf-compatible-rs` (builds
  BPF-compatible binaries, not a BTF parser) and `btf-derive` (macros for the `btf` crate);
  no new parser survivor. The search also surfaced the `btf-rs` README's statement that
  `Backend::Mmap` is only supported for base BTF files.
- Repository sources: the OpenZFS vet, the design's `pahole` command, the repo's `zfsdev`
  reuse. These are the incumbent and prior-report class.
- Saturation: the parser class is saturated for crates and GitHub at the recorded queries;
  the tool class is the two programs named. A later search could add a new crate, so the
  ledger is dated by this file's commit.

### Scored comparison

Criteria and weights, frozen before the ratings:

- C1. Bounded, fail-closed handling of untrusted bytes, weight 5.
- C2. Release and deployment fit: no runtime program, no C build, both GNU targets, weight 5.
- C3. Privilege surface in the root handler, weight 4 (higher is smaller).
- C4. Diagnostic precision: member-level expected and actual values, weight 4.
- C5. Maintenance and upstream responsiveness, weight 3.
- C6. Integration and test cost, weight 3 (higher is cheaper).
- C7. Start cost, weight 2, low confidence because it is unmeasured.

Ratings, 0 to 4, with confidence:

- `btf-rs` 2.0.1: C1 3 high, C2 4 high, C3 4 high, C4 4 high, C5 3 medium, C6 4 medium,
  C7 2.5 low (range 2 to 3).
- In-house reader: C1 4 high, C2 4 high, C3 4 high, C4 4 high, C5 3 medium, C6 1 medium,
  C7 2.5 low (range 2 to 3).
- `bpftool` 7.8.0: C1 3 medium, C2 1 high, C3 1 high, C4 2 medium, C5 4 high, C6 3 medium,
  C7 1.5 low (range 1 to 2).
- `pahole` 1.32: C1 3 medium, C2 1 high, C3 1 high, C4 3 medium, C5 4 high, C6 3 medium,
  C7 1.5 low (range 1 to 2).

Evidence for the ratings:

- C1: `btf-rs` refuses truncated input and its input is kernel-validated, but it has no section
  check of its own and its released masks are old, so the pre-scan is ours (3).
  The in-house reader can enforce every bound in one place (4).
  The tools refuse through their own parser and exit status, and the wrapper must bound the file
  before handing it over (3).
- C2: both libraries are pure Rust with no default features (4);
  both tools add a program the product must find and version (1).
- C3: in-process code keeps the privileged handler to its own binary (4);
  a root-spawned tool adds a path and library resolution to that boundary (1).
- C4: `btf-rs` and an in-house model expose names, bit offsets and sizes (4);
  `pahole`'s `-C` printer maps directly to the compared members (3);
  `bpftool`'s raw dump needs a type-id resolver in the wrapper (2).
- C5: `btf-rs` is active but small, with two open items and a release branch behind its main (3);
  the tools follow their projects' release cadence (4); in-house depends on this repository's own
  review (3).
- C6: `btf-rs` needs a loader, a pre-scan, a table and tests (4); in-house needs the reader too (1);
  the tools need a spawn wrapper and output parsing plus packaging (3).
- C7: unmeasured; `btf-rs` parses the base's type offsets and the module's types;
  in-house can read lazily; the tools pay a process start and a full dump.

Arithmetic:

- `btf-rs`: 5*3 + 5*4 + 4*4 + 4*4 + 3*3 + 3*4 + 2*2.5 = 93 of 104, 89.4.
- In-house: 5*4 + 5*4 + 4*4 + 4*4 + 3*3 + 3*1 + 2*2.5 = 89 of 104, 85.6.
- `pahole`: 5*3 + 5*1 + 4*1 + 4*3 + 3*4 + 3*3 + 2*1.5 = 60 of 104, 57.7.
- `bpftool`: 5*3 + 5*1 + 4*1 + 4*2 + 3*4 + 3*3 + 2*1.5 = 56 of 104, 53.8.

Low-signal endpoint totals:

- `btf-rs`: 92 to 94 (88.5 to 90.4).
- In-house: 88 to 90 (84.6 to 86.5).
- `pahole`: 59 to 61 (56.7 to 58.7).
- `bpftool`: 55 to 57 (52.9 to 54.8).

### Sensitivity

One input at a time, as the skill requires:

- Raising or lowering every weight from 1 to 5 leaves `btf-rs` first in every run except one:
  with C6 (integration cost) at weight 1, the in-house reader leads, 90.6 to 88.5.
  The controlling preference is therefore how much integration and test cost weighs against the
  parser's bound, and that is the owner's.
- Moving the medium and low ratings one step each, and the low-signal endpoints, flips only the
  third and fourth places: `pahole` and `bpftool` trade places when C4 or C7 moves.
  `btf-rs` stays first and the in-house reader stays second in every such run.
- Because one adjacent pair (`btf-rs` and in-house) flips on a weight the owner sets, and the
  tools' pair is a near-tie, this section gives a conditional ranking and recommends none,
  as the skill's sensitivity outcome requires.

### Ranking and pros and cons

Ranking: `btf-rs` 2.0.1 > in-house reader > `pahole` 1.32 > `bpftool` 7.8.0,
with the first pair conditional on C6's weight and the last pair conditional on C4 and C7.

- `btf-rs` beats the in-house reader because it already holds the split model, the type kinds and
  the tests that the check needs, and its two named residuals (the missing section check and the
  old masks in the release) are closed by the pre-scan this proposal already requires.
  The in-house reader buys a stronger bound at the cost of a parser, its fuzz target and its
  review; the skill's existing-tools rule keeps it second while `btf-rs` passes.
  If the owner weighs integration cost at 1, the arithmetic puts in-house first.
- The in-house reader beats `pahole` because a root-spawned program and a text format cost more
  than the parser it replaces, and the check must run in both binaries, including the handler.
- `pahole` beats `bpftool` because its `-C` output carries the compared members directly, while
  `bpftool`'s raw dump needs a type-id resolver in the wrapper; the pair is close, and the
  sensitivity run under [Sensitivity](#sensitivity) shows it flipping on the medium ratings.

Pros and cons follow each ledger entry. No candidate is chosen here.

### The vet artifacts this brief could not write

The skill asks for a vet report under `doc/audit/` when an evaluation crosses its threshold,
with a fingerprint, a lock, the full evidence records and the pre-discovery check-in.
This brief allows edits to this file only and allows no questions, so:

- the ledger, gates, source audits, arithmetic and sensitivity are here instead;
- the `doc/audit/tech-...-vet-YYYY-MM-DD.md` report is owed before any dependency is adopted;
- the finalist validation is incomplete: `btf-rs` was built and probed, not run through its own
  test suite, and the two tools were exercised by hand, not through a repeatable harness. The
  skill's rule that an unvalidated finalist cannot be recommended is why this section recommends
  none; the ranking is an input to the owner's pick, not a recommendation;
- the check-in that lets the owner delegate discovery to a model of their choosing is
  [Q4](#q4-is-this-discovery-the-one-the-vet-should-use);
- no candidate is adopted and no crate is added to a manifest.

## Owed measurements

None of these ran. Each runs in a throwaway VM with a file-backed pool, never this host's pool and
never with the host's `/dev/zfs`. Each names its control, because a pass alone does not show the
checker can fail.

1.  A compatible module passes. Build or install a module 2.4.4 with BTF in the guest, run the
    built gate, and assert it reports compatible and that a real read request then runs.
    Control: the same binary on a module without BTF must refuse, so a pass is not a vacuous pass.
2.  A changed layout refuses, with the 2.2.2 shape as the true positive. If the guest's packaged
    2.2.2 module carries BTF, run the gate on it and assert it refuses at `zinject_record`'s
    size (352 against 368) while the struct's total size and the defer offsets match.
    Control: the 2.4.4 module passes the same binary, and the refusal names the field and both
    values. If the packaged module has no BTF, patch `zfs_cmd` in the 2.4.4 source, rebuild, and
    use the patched module instead, with the unpatched one as the control.
3.  A missing module BTF refuses and falls back. Use the tarball build path that skipped BTF
    (`guest-openzfs-build.log:834-838`) or hide the base file from the checker's namespace,
    and assert raw requests are disabled, that the fallback is offered for an unused slot,
    and that registrations and recovery state survive.
    Control: the same guest after providing the base and rebuilding with BTF passes.
4.  A missing or unreadable file refuses. Hide `/sys/kernel/btf` from a mount namespace or an
    unreadable copy, and assert the refusal names the file and the step.
    Control: the same binary in the same guest with the file readable passes.
5.  A short read refuses. A test seam fed a truncated module and base pair must refuse with the
    section or header step, not stall; the unit test and the fuzz target carry this, and the VM
    arm repeats it through the product's loader only if a seam exists.
    Control: the untruncated pair passes.
6.  The header variants both pass. A module whose BTF an older tool emitted with the 24-byte
    header, and a module whose BTF reports `hdr_len` 32 (the host's kernel is
    `7.3.5-1-cachyos`, unqualified for the product until step 23), must both parse and compare.
    The design's `6.8.0-147-generic` guest emitted no module BTF for its tarball build, so this
    arm needs a build route that emits one, with the 24-byte header as its own case.
    Control: a file with `hdr_len` 16 or with non-contiguous sections must refuse.
7.  The request contract is checked. Renumber one `zfs_ioc` value in a patched module and assert
    the refusal names the request and both values.
    Control: the unpatched module passes, and the patched module still has the same struct layout,
    so the refusal can only come from the request comparison.
8.  Architecture and endianness refuse. On an `aarch64` guest (unqualified) the gate must refuse
    before any ioctl; on a big-endian BTF fixture the endianness step must refuse.
    Control: the `x86_64` little-endian guest passes.
9.  A module reload between reads is caught. Unload and load a different module between the gate's
    first and second version reads, through a test seam or a scripted guest, and assert the
    checker retries once and then refuses rather than passing on a mixed pair.
    Control: a single load passes.
10. Start cost and memory. Measure the built gate's wall time and peak memory on the approved
    module, interleaved with a same-binary control that does the same command without the ZFS
    path, and report the ratio and run-to-run spread, per the repository's timing rule.
    Also measure the base parse and the module parse separately.
    Control: the base-free case, and the no-op path, in the same round.
11. The guest fixture for step 6. The tarball module the design's guest built carries no BTF.
    Repeat item 1 with a packaged module that carries BTF or with a tarball build given a
    `vmlinux` in its build directory, and record which route produced the BTF and its release
    string.
12. The refused state in `doctor`. With a mismatched module loaded, assert `doctor` names the
    loaded module, the target and the file, keeps its exit status, and does not print a bypass;
    then with a compatible module assert it prints the compatible line.
    Control: both states in the same guest.
13. A tool-based checker's output and absence (Way 3). With the tool's packaged version, run the
    tool arm on a compatible and on a mismatched module, and assert the refusal names the member
    and both values. Then remove the tool from the guest and assert the refusal names the missing
    program and its version, not the ABI. Repeat with a second tool version.
    Control: the same module with the tool present and matching passes.
14. The root handler's tool path (Way 3). As root, run the handler with a writable `PATH`
    containing a decoy `pahole` or `bpftool`, and assert the handler uses a verified absolute path
    or refuses, never the decoy.
    Control: the verified tool at its absolute path passes, and the decoy is never executed.
15. The digest way's cost (Way 4). Rebuild the same 2.4.4 source with a different compiler or flag
    set, record whether the module BTF bytes differ, and assert the digest refuses while the full
    layout check passes. If the bytes are identical, use a patched source change that keeps the
    layout.
    Control: the full check on the same two modules passes both.
16. The blind spots of Ways 2, 5 and 6. Use a module whose `zinject_record` is 352 rather than 368
    and a module with one same-size field moved, and assert the narrow comparison, the request
    probe and the release-string-only check accept both while the full comparison refuses them.
    Control: the full comparison on the unmodified 2.4.4 module passes.

## Questions for the owner

Each question is ready for the question tool. Options are ranked, best first, with the reason for
the adjacent order. The pick is the owner's.

### Q1: which reader parses the module BTF

Recommended first: `btf-rs` 2.0.1, because no hard gate fails on the source evidence read and it holds the split
model and tests; this proposal's pre-scan closes its two residuals.
The label is the question's ranking; the vet section records why it makes no adoption
recommendation until its finalist validation is complete.

Options:

1.  `btf-rs` 2.0.1 (Recommended). Pros: split BTF and the type kinds this check needs; pure Rust;
    a small dependency tree with default features off; a documented API and tests; the probe
    parses this host's files and walks the anonymous union. Cons: `LGPL-2.1-or-later` adds a
    copyleft-family dependency; no section validation in the crate, so the binding keeps the
    pre-scan; its release's masks lag its main branch; small maintainer group.
    Cost: about 350 to 550 lines of source and 300 to 450 of tests, plus one fuzz target.
2.  An in-house reader in the binding. Pros: exact bounds and refusal semantics in one place;
    no new dependency or license; can read lazily. Cons: a parser of kernel bytes is ours to
    review, fuzz and maintain; the skill's existing-tools rule keeps it behind `btf-rs` while
    `btf-rs` passes; about 700 to 1,000 lines of source and 400 to 600 of tests.
    It ranks first only if the owner weighs integration cost at 1 or rejects the dependency on a
    named ground.
3.  `pahole` 1.32 as a start-time program. Pros: the command the design already names for the
    test-time comparison; `-C` prints the compared members; no new Rust dependency.
    Cons: a runtime package; a root-spawned program in the handler; a text format; three
    invocations for the struct, the nested type and the enum.
4.  `bpftool` 7.8.0 as a start-time program. Pros: libbpf's reader; a documented base option and a
    raw type listing. Cons: the same runtime-program and root-path costs; the raw listing needs a
    type-id resolver in the wrapper before it can compare members; the tool's version tracks the
    kernel's tools, not the distribution's.

Not offered: `btfparse` (no split base), `aya-obj` (no base, ELF-oriented), `btf` 0.5.1
(unmaintained, no base), `btfdump` (no base, tool first), `padlock-dwarf` (no base, wrong fit),
`libbpf-rs` and `libbpf` (C build or privileged library load), and a BTF digest (the answer the
project did not select).

### Q2: which module release strings qualify

Recommended first: accept the `2.4.4` version with any release suffix, because the layout check is
the substantive proof and the answer's stated reason for rejecting exact profiles was that a
rebuild must not disable clones.

Options:

1.  Version `2.4.4`, any release suffix (Recommended). Pros: the tarball `2.4.4-1`, a git build's
    `2.4.4-<suffix>` and a distribution's `2.4.4-<distro>` all reach the layout check; a rebuild
    with the approved layout keeps working, which is the answer's stated reason for the runtime
    check. Cons: the release string no longer identifies the build; a patched module with the
    approved layout passes, which is outside the check's proof but still a patched module;
    `doctor` must print the full string so a person sees what is loaded.
2.  Exact `2.4.4-1`. Pros: literal match with the approved `META` pair; simple; the guest's
    tarball build and this host's module both pass. Cons: a distribution build of 2.4.4 that sets
    its own release, in the shape the guest's packaged module shows (`0ubuntu9.5`, though that one
    is 2.2.2), would fail, and a future security rebuild with a suffix would disable the managed
    route until the owner approves it, which is the rejected exact-profile cost in another form.
3.  An owner-maintained allowlist of full release strings. Pros: exact identity for each approved
    build. Cons: a list to maintain; an unlisted rebuild still disables the route; it buys little
    over option 1, because the layout comparison is what detects ABI drift.

### Q3: may a verification be cached across processes

Recommended first: no disk cache now, because the start cost is unmeasured and a cached verdict
is a new forgeable or stale input.

Options:

1.  Per process only, no disk cache (Recommended). Pros: no stale verdict, no new state, no new
    privilege question; the cache identity is rechecked in the process. Cons: every ZFS command
    pays the parse; if the measurement shows the cost is too high, this answer changes.
2.  A per-boot cache bound to the boot id, the module version, both BTF files' device, inode,
    size and modification time, and the binary's own build id. Pros: the parse cost is paid once
    per boot. Cons: a forged cache in an account's own directory could enable raw requests on a
    mismatched module; the file must be root-owned for the handler and validated before use;
    the binding still needs the identity check that makes it safe.
3.  A cache of the parsed base BTF in memory only. Pros: no verdict cache. Cons: it helps only a
    long-lived process, and the wrapper's gate runs in short-lived ones.

This question changes only if [Owed measurements](#owed-measurements) item 10 shows a cost the
owner will not pay. The writer records the measurement result beside this question.

### Q4: is this discovery the one the vet should use

Recommended first: accept this ledger and its source classes, because the parser class saturated
at the recorded queries and the one split-capable crate was found and probed.

Options:

1.  Accept the ledger (Recommended). Pros: the candidates are named with versions and sources,
    the hard gates and a probe decided the finalists, and the class has no other split-capable
    crate in the recorded sources. Cons: the check-in the skill asks for was not made, and a later
    crate or fork could change the ledger.
2.  Delegate a fresh discovery to a model the owner names. Pros: satisfies the skill's check-in
    and may find a newer crate. Cons: cost, and the ledger here is already dated and sourced;
    a fresh run would start from this file.

## What a writer can settle without asking

- The checker lives in the binding (`package/zfs/request-to-openzfs-kernel-device`),
  because the design gives the binding the request format and the `zfs_cmd` representation and
  requires both binaries to share the gate. The wrapper module supplies the paths, the target
  allowlist, the release rule and the wording.
- The gate runs after the device check and before the first ioctl; `doctor` runs the facts without
  the device check.
- The read order and the second version read, with one retry, as
  [The read order and the reload race](#the-read-order-and-the-reload-race) states.
- The file bounds: a page for the version, 64 MiB for the base BTF and 16 MiB for the module BTF.
  Both caps are at least ten times the sizes measured here (6,579,400 and 570,234 bytes), so an
  ordinary debug kernel fits; a file that exceeds its cap is a refusal until the constant and its
  measurement are raised.
- The pre-scan rules, including the widest-encoding walk and the refusal of a value the chosen
  reader cannot represent.
- The compared set: the struct size, every direct member's offset and size, the anonymous
  containers and their members, the `dmu_objset_stats` fields the codec reads, the base integer
  sizes, and every request constant plus `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.
- The expectations come from the Rust `#[repr(C)]` type where `offset_of!` reaches, with literals
  for the anonymous containers and union members, guarded by an exhaustiveness test.
- The refusal value, its steps, and the words the caller and `doctor` print, including the
  loaded module, the target, the file and the first mismatch, and no bypass.
- The per-process `OnceLock` and the identity recheck; no disk cache until Q3 says otherwise.
- The architecture allowlist: `x86_64` only until an `aarch64` guest qualifies the managed route.
- Endianness from the BTF magic against the target.
- The test placement and fixtures: device-free unit tests with small generated BTF blobs, the
  exhaustiveness tests, the diagnostic tests, the `doctor` rendering test, and the fuzz target for
  the loader and pre-scan.
- The refusal's interaction with the design's fallback for an unused slot and the failure path for
  a bound clone; the design decides the behaviour, the writer writes the words.
- The check's placement in step 6, beside the start probe, and the step-2 test remaining the
  proof that the compiled layout matches the pinned headers.
- No external program, no C build, no `zfs` invocation, no `/dev/zfs` read for the layout check
  itself, and no force flag or environment override.

## What this proposal does not do

- It ran no guest, no built product and no measurement in
  [Owed measurements](#owed-measurements); every one is unrun.
- It changed no file except this one.
- It made no choice that belongs to the owner, and no decision beyond the accepted design.
- It does not design the start probe or the staging folder, the other two blocking designs.
- It contains no timing claim and no forecast.

## Links

[design-open-items]: cli-git-zfs-worktree-design-proposal.md#open-items-the-decisions-leave
[design-project-decisions]: cli-git-zfs-worktree-design-proposal.md#project-decisions
[design-binding]: cli-git-zfs-worktree-design-proposal.md#binding-crate-and-compatibility
[design-record]: ../decision/cli-git-zfs-worktree-design.md
[vet]: ../audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md
[skill]: ../../.agents/skills/choosing-technology/SKILL.md
[design-abi-gate]: cli-git-zfs-worktree-design-proposal.md#abi-gate-and-tests
[design-doctor]: cli-git-zfs-worktree-design-proposal.md#what-doctor-checks-and-prints
[design-decided]: cli-git-zfs-worktree-design-proposal.md#decided-behaviours-with-no-test-yet
[plan-file]: cli-git-zfs-worktree-implementation-plan.md
[plan-open-items]: cli-git-zfs-worktree-implementation-plan.md#open-items-and-the-step-that-needs-each
[plan-start]: cli-git-zfs-worktree-implementation-plan.md#what-can-start-now
[vm-fixture]: cli-git-zfs-worktree-design-proposal.md#vm-measurements-preceding-acceptance
