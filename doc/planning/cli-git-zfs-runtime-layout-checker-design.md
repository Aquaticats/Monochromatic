# cli-git ZFS runtime layout checker design

**Non-final, unapproved proposal by an agent.**
This file designs one open item of the accepted ZFS worktree design,
"The runtime layout checker" in
[Open items the decisions leave][design-open-items],
which blocks step 6 of [the implementation plan][plan-file],
"The start-time compatibility gate and the start probe".
It was written on 2026-11-19T18:10:30-05:00 by a Pi coding agent session
on branch `docs/planning-cli-git-zfs-runtime-layout-checker-design`,
made from `main` at `73eaad0846`, and revised twice as
[Revisions](#revisions) records.
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

### Revisions

- `2026-11-19T18:10:30-05:00`, commit `3ac4d96ae3`: the first version.
- `2026-11-19T18:53:08-05:00`: this revision, answering the independent gate review of the
  first version, kept at
  `/home/user/temp/agent/zfs-runtime-layout-checker-design-gate-review-2026-11-19/keep/review.md`.
  It is a review, not an approval, and the reviewer is independent of this file's author
  (rule `IRW`). This revision applies every finding it raised:
  the pre-scan's section order now accepts this host's own vmlinux (F1);
  the 2.2.2 to 2.4.4 differences are listed in full, and the Way 1 over Way 2 reason,
  owed measurement 2 and the comparison's order rest on them (F2, F3);
  Way 5 is bounded out and Way 2 is stated not to meet the answer (F4);
  the owner questions are rebuilt (F5, F9, F10);
  the scoring, the sensitivity sentence, the license gate and the discovery record are
  corrected (F6, F7, F8);
  the step 2 handoff is written (F11);
  and the low findings on the kernel versions, the file caps, the nested interiors, the
  reader independence, the timestamps, the measurement lists, the controls and the probe
  route are applied (F12 to F19).
  No accepted decision and no plan step changes.
- `2026-11-19T18:58:32-05:00`: a second review pass on this revision, which found five errors in
  it and fixes them here. Owed measurement 2's patch now also keeps the anonymous struct's pad at
  352 bytes, because the pad's own expression would otherwise shrink the struct and break the
  module's size assertion. The rule that would refuse this host's vmlinux is named correctly: the
  header's field order, not the string-last rule. Owed measurement 16 moves its field inside
  `drr_begin`, because the narrow comparison does check direct member offsets. The sensitivity now
  reports the four exact ties beside the eight reversals. And Q2 no longer both adopts and reopens
  the same suffix: it adopts the rule and asks only whether to narrow it for git snapshots.

### Sources read

Logs under `~/temp/agent` that this file cites are the author's session scratch.
Scratch is not durable, so a fact that only a scratch log shows is a lead for a guest
run, not proof, and [Owed measurements](#owed-measurements) re-runs each one.

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
The same files at tag `v7.3`, which is the release this host's kernel is built from
(`7.3.5-1-cachyos`), read through the gate review's tree:
`include/uapi/linux/btf.h`, `kernel/bpf/btf.c`, `kernel/bpf/btf_relocate.c`,
`kernel/bpf/sysfs_btf.c`, `lib/Kconfig.debug`, `scripts/Makefile.modfinal`,
`tools/lib/bpf/btf.c` and `tools/lib/bpf/btf_relocate.c`.
Where the two releases differ, a fact below names both.

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
and the query schedule recorded under
[Discovery record](#discovery-record).

### What was run, and what was not

Run on this host, read only:

- `stat`, `cat`, `od` and `uname --kernel-release` against `/sys/module/zfs/version`,
  `/sys/kernel/btf/vmlinux` and `/sys/kernel/btf/zfs`.
  The `od` read of the first 32 bytes of each BTF file is what the header facts below rest on.
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
  That route is not the product's: the loader reads bounded bytes and calls
  `Btf::from_bytes` and `Btf::from_split_bytes`.
  The gate review re-ran this second route on this host and its claims held, so the facts below
  rest on the route the proposal designs.
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
- The reader lives in the binding, as a proposal the owner may veto. The project chose
  "Our own small binding (Recommended)",
  with `package/zfs/request-to-openzfs-kernel-device`, and both binaries "use the same binding and compatibility
  gate, even for reads". The binding "owns the `/dev/zfs` request format", and the verification method is decided
  while "the profile-checking dependency that checker needs is chosen with its design, which has none yet"
  ([Binding crate and compatibility][design-binding]).
  The record does not decide where the reader lives: both binaries are one cargo package,
  `package/git-policy/cli`, so a reader in the binding and a reader in the wrapper package both satisfy the
  record. Rule `OWK` makes placement across components the named owner's, so
  [The seam and the placement](#the-seam-and-the-placement) states it as a proposal with a veto, and the
  binding's ownership of the request format is the reason proposed.
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
  Those lines are Linux `v6.8`; the same notifier at `v7.3` registers the same `0444` attribute
  (`:8612-8625`) and adds no `.mmap` to it, so a module file is read the same way on both.
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
  `base_btf->hdr.str_len` (`kernel/bpf/btf.c:5985-5987`, Linux `v6.8`; the same three assignments
  sit at `:6573-6575`, `v7.3`).
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
  (`include/uapi/linux/btf.h:8-22`, `v6.8`). Its `info` word packs 16 bits of `vlen` and
  5 bits of kind (`:34-57`).
- The header at `v7.3` is 32 bytes: it adds `layout_off` and `layout_len`,
  and a layout section describes each kind's element sizes
  (`include/uapi/linux/btf.h:12-34`, `v7.3`).
  Its `info` word packs 24 bits of `vlen` and 7 bits of kind (`:51,70-72`).
- Probe on this host: both files report `hdr_len` 32 and version 1.
  `/sys/kernel/btf/vmlinux` reports type length 3,789,716, string offset 3,789,796,
  string length 2,789,572, layout offset 3,789,716 and layout length 80,
  and its sections sum to the file size.
  `/sys/kernel/btf/zfs` reports type length 329,244, string offset 329,244,
  string length 240,958 and zero layout offset and length, and its sections sum to the file size.
  A reader that assumes `hdr_len` is 24 misreads both files.
- The header's field order is type, string, layout, but the sections inside a file are not
  required to follow it. The kernel sorts the sections it knows by their offset and requires
  them to be contiguous, with no gap and no overlap, ending at the read length
  (`kernel/bpf/btf.c:5697-5735`, `v7.3`; `:5196-5247`, `v6.8`).
  libbpf writes the layout section between the types and the strings
  (`tools/lib/bpf/btf.c:360`, `v7.3`), and this host's vmlinux is laid out that way:
  type at 0, layout at 3,789,716, strings at 3,789,796.
- The string section ends the file only on the older header: at `v6.8` the check is
  unconditional (`kernel/bpf/btf.c:5160-5164`), while at `v7.3` it applies only when `hdr_len`
  is below the 32-byte header (`:5625-5629`).
  So on a 32-byte header the string section may sit before the layout section, and a rule that
  required the header's field order, type then string then layout, would refuse this host's
  vmlinux.
- The kernel's own header check accepts a longer `hdr_len` only as far as it understands and
  requires extra bytes to be zero in `v6.8` (`kernel/bpf/btf.c:5249-5294`), which is the
  24-byte header's path.
- The header's shape follows the tool that emitted the BTF, not the kernel that loaded it:
  the current tool writes the 32-byte header this host's files carry, and an older one wrote 24.
  The checker accepts both and does not infer a kernel release from the header.
- `v7.3` adds two things the reader must survive and the checker must not assume away.
  `/sys/kernel/btf/vmlinux` gains a `.mmap` file operation (`kernel/bpf/sysfs_btf.c:19-46`),
  absent at `v6.8`, so a loader must not depend on mmap being refused or allowed.
  And a module may carry a distilled `.BTF.base`; `v7.3`'s `btf_parse_module` parses it,
  relocates the module BTF in place against vmlinux and clears it
  (`kernel/bpf/btf.c:6533-6611`, `:6604-6610`), so the file the kernel exposes is the relocated
  split form and this file's id and string-offset statement holds for it.
  libbpf writes `hdr_len = sizeof(struct btf_header)` (`tools/lib/bpf/btf.c:1253`, `v7.3`).

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
  `dds_guid` at bit 128, `dds_type` at bit 192, `dds_is_snapshot` at bit 224,
  `dds_origin` at bit 248 with size 256 and `dds_flags` at bit 2,296 with size 1.
  `dds_flags` occupies the byte that was tail padding in 2.2.2, so adding it moved nothing
  and left the total at 288 (`include/sys/dmu.h:1032-1042`; 2.2.2 `:951-960`).
  `enum zfs_ioc` has 104 values at 2.4.4, including `ZFS_IOC_FIRST = 23040`,
  `ZFS_IOC_OBJSET_STATS = 23058`, `ZFS_IOC_DESTROY_SNAPS = 23099`,
  `ZFS_IOC_CLONE = 23106`, `ZFS_IOC_GET_BOOKMARKS = 23108`,
  `ZFS_IOC_CHANNEL_PROGRAM = 23112` and `ZFS_IOC_LAST = 23177`.
- 2.4.4 has 104 enumerators where 2.2.2 has 102: 2.2.2 lacks
  `ZFS_IOC_POOL_PREFETCH` (23,128) and `ZFS_IOC_DDT_PRUNE` (23,129), which 2.4.4 appended inside
  the core block's reserved numbers (`include/sys/fs/zfs.h:1474-1590`).
  Both releases keep `ZFS_IOC_PLATFORM` at 23,168 and `ZFS_IOC_LAST` at 23,177, so the range
  endpoints and every request the binding sends are identical in the two releases.
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
  is `(MAXPATHLEN*3)+MAXNAMELEN+1200` (`:573-576`).
  On Linux `MAXPATHLEN` is 4096 and `MAXNAMELEN` is 256
  (`include/os/linux/spl/sys/sysmacros.h:68-69`), which is 13,744 bytes.
- The request numbers are `enum zfs_ioc` at `include/sys/fs/zfs.h:1474-1590`;
  `ZFS_IOC_FIRST` is `('Z' << 8)` on Linux (`:1480-1481`).
- The 2.2.2 to 2.4.4 change kept the struct's total size and every named offset the two
  releases share, and changed three things a checker can see
  (`include/sys/zfs_ioctl.h:509-562` and `:407-427` at 2.4.4, `:484-525` and `:410-428` at 2.2.2):
  2.4.4 moves `zc_inject_record`, `zc_defer_destroy`, `zc_flags` and `zc_action_handle` into an
  anonymous union holding an anonymous struct, where 2.2.2 has all four as direct members of
  `zfs_cmd_t`; `zinject_record_t` grew from 352 to 368 bytes with `zi_match_count` and
  `zi_inject_count`, and the union's anonymous struct keeps `zc_pad1[352]`, so no offset moves;
  and `dmu_objset_stats` gained `dds_flags` in the byte that was tail padding.
  A check that compares the total size and the offsets alone accepts all three differences;
  a check that also compares member type sizes, the anonymous container's kind and position, and
  `dds_flags` refuses them ([the vet][vet], "Layout against OpenZFS 2.4.4", lines 430-446,
  which names the union and the grown record but not `dds_flags`).

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
  The product's route reads both files into bounded buffers and calls `from_bytes` for the base and
  `from_split_bytes` for the module, which is the cached backend
  (`btf-rs/src/btf.rs:85-107`), and it resolves a module type by name without a file path or a mmap.
  The first version's probe used the mmap backend for the base instead; the gate review re-ran the
  product's route on this host and the same facts held, so no fact here rests on the mmap route,
  which a `v6.8` guest does not offer.
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
- The compared set stops at the interiors of the nested structs the codec only passes through:
  `drr_begin`, `zfs_stat_t` and `zfs_share_t` are compared as members with their own size and
  offset, not member by member, so a change inside one of them that leaves its size alone passes.
  `dmu_objset_stats` and `zinject_record` are compared member by member, because their members are
  the fields behind the 2.2.2 to 2.4.4 differences.
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

Request contract is the one proof the 2.2.2 module would also pass: its enumerators, endpoints and
every request the binding sends hold the same values, since 2.4.4 only appended two requests into
the core block's reserved numbers. The request comparison therefore cannot tell the two releases
apart, and the layout comparison is what does.

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
  It refuses all three 2.2.2 to 2.4.4 differences, including the anonymous union and the
  `dds_flags` member that moved no offset.
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

What it needs measured: items 1 to 12 under
[Owed measurements](#owed-measurements): a pass, the true-positive refusal on the patched 2.4.4
shape, a missing module BTF, the header variants, a short read, the request contract, the
architecture and endianness refusals, the reload race and the start cost.

### Way 2: parse in process and compare a narrow set

The same in-process reader, but the expectations cover only the struct size, every direct
member's bit offset, and the offsets (not the sizes or the anonymous shapes) of the nested fields
the codec reads.

Pros:

- A smaller table and fewer refusal paths than Way 1.
- It catches a moved field the codec uses and a changed total size.

Cons:

- It does not meet the answer. Its compared set sees none of the three 2.2.2 to 2.4.4
  differences: it compares no member size and no anonymous container, and it names `dds_flags`
  only if the codec happens to read that member, so two of the three pass it outright and the third
  depends on the codec.
  Whether it even accepts a 2.2.2 module needs an assumption the design must state: that a
  member of the approved profile which sits inside the anonymous container may be matched by a
  flattened name, and that a member the module lacks is not checked. Neither is stated today.
- A nested type change that moves no direct member passes, which is the class of change
  the runtime check exists to catch.

Cost to build and test:

- About 250 to 400 lines of source and 200 to 300 lines of tests with a reader dependency.
- The same reader and loader cost as Way 1, so the saving is small against the loss.

What it needs measured: item 2 and item 16, where the narrow comparison must accept the patched
module that the full comparison refuses, plus items 1 and 10.
That result is why this way is not ranked: it is the cheaper variant the comparison under
[The comparison](#the-comparison) rules out, and the saving is small against the loss.

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

This way is bounded out by the gate's order, not only by its own cons. The accepted gate says
"At start, before the first ioctl, both binaries must prove compatibility"
([ABI gate and tests][design-abi-gate]), and Way 5 obtains its proof by sending an ioctl.
The proof itself is smaller than its name claims: the kernel copies a fixed
`sizeof (zfs_cmd_t)` (`module/os/linux/zfs/zfs_ioctl_os.c:139-150`), so a success shows only that
`/dev/zfs` exists and that dispatch reached a handler.
It is listed so the boundary is visible, and it is not ranked.

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

Ranking: Way 1 > Way 3. These two prove the five items before the first ioctl and compare the
declared layout, so these two meet the answer.

- Way 1 beats Way 3 because the proof stays in the wrapper's address space: no runtime program
  for the product to find, verify and version, no root-spawned binary whose path and library
  resolution join the privileged boundary, and no text format to parse back into members.
  Way 3 proves the same facts and pays those three costs for them.

Four further ways are not ranked, each for a stated reason:

- Way 2 does not meet the answer. It compares no member size, no anonymous container and no
  `dds_flags`, so the three 2.2.2 to 2.4.4 differences go uncompared, and with the flattening
  assumption it needs they all pass it.
- Way 4 is bounded out by the accepted answer: it would refuse every rebuild.
- Way 5 is bounded out by the gate's order, since it obtains its proof by sending the ioctl the
  gate must precede, and it proves only that the caller's buffer covers the copy.
- Way 6 is the gate the answer replaced: it omits the layout check the answer added.

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
so the design proposes that the binding owns:

- the bounded file loader and the pre-scan;
- the expectations derived from its `#[repr(C)]` type and its request list;
- the comparison and its refusal value.

and that the wrapper and the handler own:

- the three paths, the read order and the per-process cache;
- the target allowlist (architecture) and the module release rule the owner picks;
- the words the caller and `doctor` print.

Both binaries call the same entry point, as the design requires for reads as well as writes.
This placement is a proposal the owner may veto, not a decision. The record decides that the
binding owns the request format and that both binaries share the binding and the gate; both
binaries live in one cargo package, so it does not decide which crate holds the reader
([Binding crate and compatibility][design-binding]).
Rule `OWK` makes a cross-component placement the named owner's.
If the owner moves the reader into `package/git-policy/cli`, the binding keeps the expectations and
the wrapper package holds the loader and the parser, and nothing else in this file changes.

### The read order and the reload race

The gate, for a caller that needs the device:

1.  Check that `/dev/zfs` exists and that the caller can open it.
    A missing or unopenable device refuses with that step and starts no read.
2.  Read `/sys/module/zfs/version` once, bound to one page, and strip one trailing newline.
3.  Read the two BTF files, each opened with `O_RDONLY | O_CLOEXEC | O_NOFOLLOW`,
    each read to a bounded size, each checked against its own header and file size.
4.  Read the version file again. If it differs, retry the whole check once; refuse with
    "module changed while checking" only when the retry's own two version reads also differ.

`doctor` runs the same facts without step 1, so it can report what a caller would meet.

The second version read closes the window in which the module unloads or is replaced between
the first read and the BTF reads. It does not close the window during a single read;
the header and size checks refuse a short or inconsistent file.
The retry is what keeps a reload that has already settled from reading as a refusal: only a
second attempt that also sees two different version reads refuses, and a second attempt that sees
one module passes.

### The bounded pre-scan

Before any parser sees the bytes, the loader checks:

- the magic is `0xeB9F` in one byte order, the version is 1, and the flags are zero;
- `hdr_len` is at least the 24 bytes of the oldest header and at most the read bound;
- the type, string and, if present, layout sections, sorted by offset, are contiguous with no gap
  and no overlap, and the last of them ends at the read length. That is the order `v7.3` accepts
  (`kernel/bpf/btf.c:5697-5735`), and this host's vmlinux is laid out under it as type, then
  layout, then strings;
- with a 24-byte header the string section ends the file, which `v6.8`'s checker requires
  unconditionally (`kernel/bpf/btf.c:5160-5164`) and `v7.3` still requires below the 32-byte
  header (`:5625-5629`);
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
- every member of `dmu_objset_stats` and `zinject_record`, with its offset and size, not only
  the members the codec reads, and the two types' own sizes (288 and 368), as listed under
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

The order the comparison walks is what makes the refusal's first finding predictable, and the
2.2.2 to 2.4.4 differences land at different steps:

- A 2.2.2 module has no anonymous union where the expectations place one, so a walk that checks
  the expected container's kind and bit offset before descending refuses there, at `Type`.
  A walk that compared named members first would refuse at `Member` instead, naming `dds_flags`
  missing or `zc_inject_record`'s 352 against 368 bytes, depending on the order.
  The documented order is container, then member name, then member offset, then member size, so a
  2.2.2 module's first refusal is the container, at `Type`.
- `zinject_record_t`'s size is the difference no offset shows: 352 against 368 bytes, with
  `zc_defer_destroy` at 13,648 in both. Only a member size comparison refuses it.
- `dds_flags` is the difference no size or offset of any other member shows: it is one byte at
  2,296 bits, filling the byte that was tail padding, and `dmu_objset_stats` stays 288 bytes.
  Only an expectation that names `dds_flags` at all refuses it.
- The request contract refuses none of the three: every request the binding sends and both
  endpoints hold the same value in 2.2.2, so the layout comparison is what carries the refusal.

A checker that compares the total size and the offsets only, and that flattens a member found
inside the module's anonymous container to the name the approved profile expects, therefore
accepts the 2.2.2 shape ([the vet][vet], lines 430-446, which names the union and the grown record
but not `dds_flags`). Without that flattening it refuses at the container, which is a different
refusal than the one Way 2's own cons claim, and that is why Way 2 must state the assumption.

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
`git_version.rs` (`package/git-policy/cli/src/native/invocation/git_version.rs:23-60`).
Before each use, the cache's identity is rechecked against the version text and the two files'
identity (device, inode, size and modification time); a change re-runs the check.
Per-process caching with no disk cache is writer-settled: it needs no owner choice while the
start cost is unmeasured.

The wrapper already writes one disk cache: a private per-user file holding the text `git version`
prints, keyed by the executable's canonical path, size, modification time, change time, device and
inode (`package/git-policy/cli/src/native/invocation/git_version_cache.rs:1-30`).
That precedent does not carry over. The cached `git version` text is descriptive, and a wrong
entry changes a warning; a compatibility verdict is the gate that enables raw requests, so a
forged or stale entry in the account's own cache would authorize requests on a module whose layout
nothing compared. That is a different trust class, so this design writes no verdict to disk.
[Owed measurements](#owed-measurements) item 10 measures the cost, and
[Q3](#q3-may-a-verification-be-cached-across-processes) is held until it runs.

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
  `6d771ee9835fcf517dbcf1d29e25a97480425d45`. License `LGPL-2.1-only OR BSD-2-Clause`, read from
  the crates.io metadata for 0.27.2; its `libbpf-sys` dependency is `BSD-2-Clause`.
- Source audit: the safe wrapper around `libbpf-sys`; its default feature is
  `libbpf-sys/vendored-libbpf`, and the manifest says it "requires other necessary libs to be
  present on the system" (`libbpf-rs/Cargo.toml:20-26`).
- Gates: H2 pass, because a dual-licensed crate may be taken under its `BSD-2-Clause` arm, which
  is compatible with the wrapper's `LGPL-3.0-or-later`. H3 fail: the default feature runs
  libbpf's C build and needs the system's libraries.
  A `dlopen` of the system's `libbpf` avoids the link but keeps the native provenance and adds a
  privileged library load in the handler, so H3 and H6 fail there too. Exit on H3.

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
- Conditional candidate, priced in the scored comparison so the owner can weigh it, and not a
  finalist while `btf-rs` passes every hard constraint.

#### Not candidates

- `zfsdev-rs`'s layout code: it hard-codes a layout and does not parse BTF, so it cannot serve a
  runtime check. The design already reuses its codec and layout code for the binding.
- `object`, `goblin`, `scroll` and `libelf`: they expose an ELF section or a byte stream, not BTF
  types, so they are building blocks of the in-house reader, not candidates.
- `trailofbits/btfparse` (C++): discovery found it; a C++ build fails H3.

### Discovery record

The skill asks for a frozen query schedule, a page rule, a count and the new candidates per query,
and one terminal result. The first version read 50 of 81 rows at `per_page=50`, which is a provider
cap, so this revision completes each registry query at the maximum page size and records every row.
The tagging below comes from a crate's name and description only, except where a source audit
exists in [Candidate ledger](#candidate-ledger).

- Class 1, the crates.io registry. `https://crates.io/api/v1/crates?q=btf&per_page=100` returned
  all 81 rows of 81 at `per_page=100`, so this query is complete.
  New candidates among the 31 rows the first version did not read: none that is a parser
  (`padlock-cli` is the padlock analyzer's command, and `btf-derive` is a macro crate).
  All 81 rows:

```text
# crates.io q=btf, per_page=100, 81 of 81 rows, read 2026-11-19
1. btf 0.5.1 (parser, H1 fail)
2. btf-rs 2.0.1 (parser, finalist)
3. ebeepf 0.1.0 (eBPF loader, reads BTF as an input)
4. corelox-build 0.1.0 (CO-RE relocations, reads BTF as an input)
5. btf-derive 0.3.2 (macros for the btf crate, not a parser)
6. btfparse 1.3.9 (parser, H1 fail)
7. btfdump 0.0.5 (parser and tool, H1 fail as a library)
8. btf-rs-derive 2.0.1 (macros for btf-rs, not a parser)
9. breda-btf-data-gen 0.0.0 (reserved)
10. breda-btf-generator 0.0.0 (reserved)
11. libarena-rs 0.3.4 (BPF arena allocator, not a parser)
12. aya-obj 0.6.0 (parser with no base, H1 fail)
13. profile-bee-aya-obj 0.2.2 (fork of aya-obj, same H1 fail)
14. zejtron 80.2.2 (host evidence ledger that reads BTF, a tool)
15. aya-friday-obj 0.2.2 (fork of aya-obj, same H1 fail)
16. aya-obj-nightly 0.0.0 (same as aya-obj)
17. bpf-linker 0.13.0 (BPF linker, reads BTF as an input)
18. btf2wit 0.1.2 (prints BTF as wit, a tool)
19. ntpsec-rs-xdp 0.3.50 (XDP program, not a parser)
20. whytime 0.1.2 (timing profiler, not a parser)
21. padlock-dwarf 0.82.0 (parser in a DWARF analyzer, H1 fail)
22. aa-ebpf 0.0.1-rc.7 (kernel monitoring, not a parser)
23. orb8-proto 0.0.6 (protocol definitions, not a parser)
24. orb8-common 0.0.6 (observability toolkit, not a parser)
25. orb8-cli 0.0.6 (observability toolkit, not a parser)
26. fsblame 0.1.1 (file-change attribution, not a parser)
27. mquire 1.4.1 (memory forensics that reads BTF, a tool)
28. orb8 0.0.6 (observability toolkit, not a parser)
29. orb8-agent 0.0.6 (observability toolkit, not a parser)
30. sipnab-bpf-types 0.1.3 (record layout shared with a BPF program, not a parser)
31. suitesparse_sys 0.1.4 (sparse matrix bindings, not a parser)
32. sipnab 0.5.267 (SIP capture, not a parser)
33. redbpf 2.3.0 (eBPF runtime, reads BTF as an input)
34. rustnet-host 0.6.0 (socket inventory, not a parser)
35. ghostscope-platform 0.1.5 (kernel and BTF quirks behind a debugger, not a parser)
36. linprov-common 0.2.0 (types for linprov, not a parser)
37. linprov 0.4.0 (file policy, not a parser)
38. mem4n6 0.2.0 (memory forensics that reads BTF, a tool)
39. aya 0.16.0 (eBPF library, reads BTF as an input)
40. rvf-types 0.3.1 (vector format types, not a parser)
41. memf-symbols 0.2.4 (symbol backends that read BTF, a tool)
42. procinsh 0.1.20 (process inspector, not a parser)
43. profile-bee-aya 0.13.2 (fork of aya)
44. bioformats 0.1.8 (image format translation, not a parser)
45. bpf-script 0.5.4 (eBPF scripting, reads BTF as an input)
46. aya-friday 0.13.2 (fork of aya)
47. scx_pandemonium 5.23.1 (sched_ext scheduler, not a parser)
48. bpf-api 0.3.1 (eBPF bindings, reads BTF as an input)
49. assayist-capture-resident 0.1.3 (residency capture, not a parser)
50. aya-nightly 0.0.0 (eBPF library, reads BTF as an input)
51. rvf-runtime 0.5.6 (vector format runtime, not a parser)
52. prevail 0.2.6 (eBPF verifier, reads BTF as an input)
53. zelynic 80.4.0 (rate limiter, not a parser)
54. zylaxion 80.2.3 (keyboard sounds, not a parser)
55. ltop 0.2.13 (network TUI, not a parser)
56. padlock-cli 0.82.0 (the padlock analyzer command, not a parser)
57. ebpf-ifc-engine 0.1.8 (eBPF policy engine, not a parser)
58. nalgebra_block_triangularization 0.1.0 (matrix decomposition, not a parser)
59. actplane-ifc-compiler 0.1.8 (eBPF policy compiler, not a parser)
60. bpfix 0.1.15 (verifier diagnostics, reads BTF as an input)
61. tracekit-wire 0.1.0 (kernel and user ABI types, not a parser)
62. actplane-runtime 0.1.8 (eBPF policy engine, not a parser)
63. actplane 0.1.8 (eBPF policy engine, not a parser)
64. wits 2.2.3 (profiler that reads BTF, a tool)
65. bpfdoctor 0.1.0 (BPF doctor, reads BTF as an input)
66. procctx 0.1.0 (process context for eBPF tools, not a parser)
67. bpf-script-derive 0.5.4 (macros for bpf-script)
68. scx_cake 1.2.2 (sched_ext scheduler, not a parser)
69. iodyne 0.9.0 (disk IO TUI, not a parser)
70. arena-linker 0.2.0 (BPF linker, not a parser)
71. compendium 0.2.1 (strace viewer, not a parser)
72. ebpfsieve 0.1.6 (eBPF offload, not a parser)
73. electric-eyed 0.1.0 (activity monitor, not a parser)
74. snitchit 0.1.0 (session recorder, not a parser)
75. btfm-api-structs 0.3.1 (Discord bot API, not a parser)
76. btfs 0.0.4 (in-memory filesystem, not a parser)
77. allenap-libtftp 0.2.2 (TFTP library, not a parser)
78. btfm 0.21.1 (Discord bot, not a parser)
79. btfm-cli 0.7.0 (Discord bot command, not a parser)
80. libtftp 0.1.2 (TFTP server, not a parser)
81. redoubtful 0.0.1 (sandbox runner, not a parser)
```

  `?q=split%20btf&per_page=100` returned all 17 rows of 17; the only BTF parser in it is `btf-rs`,
  and the rest are `aya-obj`'s forks and eBPF tooling.
  `?q=btf%20kernel&per_page=100` returned all 50 rows of 50; no new parser candidate.
- Class 2, the repository host. GitHub repository search
  `BTF parser in:name,description language:rust` returns one row today, `retis-org/btf-rs`.
  The first version recorded seven rows at the same filters and named
  `luzhixing12345/btfparser` (a C function-declaration parser, not a Rust BTF library) and
  `genksome/kxray` (an Android offsets tool) among them, so the two runs disagree on a provider
  whose index changes under the same query; both runs are recorded.
  The wider query `btf in:name,description language:rust` returned 37 rows.
  New names, screened by name and description only: `vobst/btf2json`, `eunomia-bpf/btf2wit`,
  `vadorovsky/aya-btf-map`, `eunomia-bpf/rust-struct-bindgen`, `kuopinghsu/BTFViewer-rust`,
  `flier/rust-btf`, `sysdiglabs/corelox`, `vadorovsky/aya-btf`, `eurecom-s3/lemon-btf2json`,
  `leonardogiovannoni/BTF2LLVM` and `buffalojoec/btf-dumps`.
  Each is a single-blob reader, a BTF printer, or a generator that consumes one blob; none names a
  split base, and none is audited here.
  GitHub code search `from_split_bytes language:rust` returned 25 rows.
  Only `retis-org/btf-rs` is a BTF reader with a split constructor; `likewhatevs/ktstr` holds a
  vmlinux-only struct-field resolver in a larger tool with no module base, and the rest are
  unrelated `split_*` helpers (`quickwit`, `paimon`, `loams`).
- Class 3, broader web search. The literal query `Rust crate parse BTF split BTF module vmlinux
  base BTF parser library` returned 44 result lines, of which the provider showed 10, in the first
  version and again through the second provider used for this revision.
  A provider cap is not exhaustion, so this class is blocked.
  Independent enumeration of what it would add: the crates.io listing above and the code search
  for the split constructor.
  New names it surfaced, screened by description only: `bpf-compatible-rs` (builds BPF-compatible
  binaries, not a BTF parser) and `btf-derive` (macros for the `btf` crate); no parser survivor.
  It also surfaced the `btf-rs` README's statement that `Backend::Mmap` is only supported for base
  BTF files.
- Class 4, this repository. The OpenZFS vet, the design's `pahole` command, the repo's `zfsdev`
  reuse. These are the incumbent and prior-report class, and the vet already names both tools.

Terminal result: blocked by an unenumerable required source, since class 3 cannot be exhausted at
its provider. The skill's rule is that discovery is saturated only when every required source
class is saturated, so nothing here is called saturated, the ledger is dated by this file's commit,
and [The vet artifacts this brief could not write](#the-vet-artifacts-this-brief-could-not-write)
records why no candidate is recommended.

### Scored comparison

Criteria and weights, frozen before the ratings:

- C1. Bounded, fail-closed handling of untrusted bytes, weight 5.
- C2. Release and deployment fit: no runtime program, no C build, both GNU targets, weight 5.
- C3. Privilege surface in the root handler, weight 4 (higher is smaller).
- C4. Diagnostic precision: member-level expected and actual values, weight 4.
- C5. Maintenance and upstream responsiveness, weight 3.
- C6. Integration and test cost, weight 3 (higher is cheaper).
- C7. Start cost, weight 2, low confidence because it is unmeasured.

Only `btf-rs`, `pahole` and `bpftool` are finalists: each passes every hard constraint.
The in-house reader passes H1 to H6 by construction but is not eligible as a candidate while an
existing tool passes them, which is the skill's existing-tools rule and the reason
[Candidate ledger](#candidate-ledger) calls it conditional.
It is still scored, so the owner can price the alternative Q1 offers.

Ratings, 0 to 4, with confidence:

- `btf-rs` 2.0.1: C1 3 high, C2 4 high, C3 4 high, C4 4 high, C5 3 medium, C6 3 medium,
  C7 2.5 low (range 2 to 3).
- In-house reader: C1 4 high, C2 4 high, C3 4 high, C4 4 high, C5 3 medium, C6 2 medium,
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
- C6: the two library routes share most of the work, so the difference is small and both are
  rated close. The shared part is the bounded loader, the pre-scan and its widest-encoding walk
  over every kind, the expectation table with its path grammar, the comparison, the refusal value
  and their tests: the 350 to 550 source lines and 300 to 450 test lines Way 1 prices with a
  dependency. The dependency route adds a resolver the crate does not carry: `btf-rs` gives a
  `size()` per type and `resolve_chained_type` (`btf-rs/src/btf.rs:213-226`), but its array type
  exposes only `len()` (`:499-521`), so array byte sizes and typedef chains are the binding's own
  walk. The in-house route adds the reader and its model, about 350 to 450 source lines and 100 to
  150 test lines on the same shape, plus its fuzz target and its review.
  About two to one in effort, not four to one, so 3 and 2.
  The tools need a spawn wrapper and an output parser each, plus the packaging and the version
  matrix (3).
- C7: unmeasured; `btf-rs` parses the base's type offsets and the module's types;
  in-house can read lazily; the tools pay a process start and a full dump.

Arithmetic:

- `btf-rs`: 5*3 + 5*4 + 4*4 + 4*4 + 3*3 + 3*3 + 2*2.5 = 90 of 104, 86.5.
- In-house: 5*4 + 5*4 + 4*4 + 4*4 + 3*3 + 3*2 + 2*2.5 = 92 of 104, 88.5.
- `pahole`: 5*3 + 5*1 + 4*1 + 4*3 + 3*4 + 3*3 + 2*1.5 = 60 of 104, 57.7.
- `bpftool`: 5*3 + 5*1 + 4*1 + 4*2 + 3*4 + 3*3 + 2*1.5 = 56 of 104, 53.8.

Low-signal endpoint totals, over every combination of the ranges' endpoints:

- `btf-rs`: 89 to 91 (85.6 to 87.5).
- In-house: 91 to 93 (87.5 to 89.4).
- `pahole`: 59 to 61 (56.7 to 58.7).
- `bpftool`: 55 to 57 (52.9 to 54.8).

The weights, the ratings, the confidence levels and the ranges above are every input the
arithmetic uses, and the arithmetic and the matrix under [Sensitivity](#sensitivity) follow from
them.

### Sensitivity

One input at a time, as the skill requires: every weight from 1 to 5, every medium-confidence
rating one step down and up, and both endpoints of every low-signal range (the endpoints are the
plausible values, so no step is taken outside a range).
Twelve tests change an order: eight reverse it, and four leave an exact tie.

Library pair, `btf-rs` against the in-house reader, 90 against 92 at the baseline:

- weight C1 at 1 reverses, 78 against 76;
- weight C1 at 2 reverses, 81 against 80;
- weight C1 at 3 ties, 84 each;
- weight C6 at 5 ties, 96 each;
- `btf-rs` C5 from 3 to 4 reverses, 93 against 92;
- `btf-rs` C6 from 3 to 4 reverses, 93 against 92;
- in-house C5 from 3 to 2 reverses, 89 against 90;
- in-house C6 from 2 to 1 reverses, 89 against 90.

Tools pair, `pahole` against `bpftool`, 60 against 56 at the baseline:

- `bpftool` C1 from 3 to 4 reverses, 61 against 60;
- `pahole` C1 from 3 to 2 reverses, 55 against 56;
- `bpftool` C4 from 2 to 3 ties, 60 each;
- `pahole` C4 from 3 to 2 ties, 56 each.

No test moves a tool past a library, so the two pairs stay apart by more than twenty points of 104.

A tie is not an order the arithmetic decides, and the skill's rule for both a tie and a change is
to gather decisive evidence or ask only the controlling preference.
For the tools, the C4 evidence decides: `pahole`'s `-C` output carries the compared members, while
`bpftool`'s raw dump needs a type-id resolver in the wrapper, so `pahole` stays third when the
rating moves.
For the library pair the evidence does not decide: both routes' C4 and C6 rest on readings of two
different code bases, and the pair is 92 to 90 with eight tests tying or reversing it.
Stability here does not cover simultaneous multi-input changes, and this document makes no claim
it does.
The controlling preference is the owner's, not a weight on this writer's scale: whether to keep
about 350 to 450 source lines and 100 to 150 test lines plus a fuzz target of our own, or to take
an `LGPL-2.1-or-later` dependency whose release lags its own main.
[Q1](#q1-which-reader-parses-the-module-btf) asks it in those words.
The evidence that would settle the ratings instead is owed:
`btf-rs`'s own test suite, which was not run, would settle C5 and C6, and
[Owed measurements](#owed-measurements) item 10 would settle C7.
Until then this section gives a conditional ranking and recommends none.

### Ranking and pros and cons

Ranking: `btf-rs` 2.0.1 > in-house reader > `pahole` 1.32 > `bpftool` 7.8.0.

- `btf-rs` beats the in-house reader because it is the only eligible library: the skill's
  existing-tools rule admits a custom implementation only when every existing tool fails a named
  hard constraint, and `btf-rs` passes all six. The arithmetic does not decide this pair: the
  fallback's corrected 92 of 104 leads `btf-rs`'s 90, the two are two points apart, and one step
  on C6 reverses them ([Sensitivity](#sensitivity)). The two points do not make the fallback
  eligible, and the choice between them is the owner's, asked in plain words in
  [Q1](#q1-which-reader-parses-the-module-btf).
- The in-house reader beats `pahole` because a root-spawned program, a text format and a runtime
  package cost more than the reader they replace, and the check must run in both binaries,
  including the handler. The arithmetic agrees, 92 against 60.
- `pahole` beats `bpftool` because its `-C` output carries the compared members directly, while
  `bpftool`'s raw dump needs a type-id resolver in the wrapper before it can compare anything.
  The arithmetic agrees, 60 against 56, and the pair flips on C1 or C4.

Pros and cons follow each ledger entry. No candidate is chosen here.

### The vet artifacts this brief could not write

The skill asks for a vet report under `doc/audit/` when an evaluation crosses its threshold,
with a fingerprint, a lock, the full evidence records and the pre-discovery check-in.
This brief allows edits to this file only and allows no questions, so:

- the ledger, gates, source audits, arithmetic, sensitivity and the full discovery listing are
  here instead;
- the `doc/audit/tech-...-vet-YYYY-MM-DD.md` report is owed before any dependency is adopted;
- the finalist validation is incomplete: `btf-rs` was built and probed, not run through its own
  test suite, and the two tools were exercised by hand, not through a repeatable harness. The
  skill's rule that an unvalidated finalist cannot be recommended is why this section recommends
  none; the ranking is an input to the owner's pick, not a recommendation;
- discovery's terminal result is blocked, not saturated
  ([Discovery record](#discovery-record)), which is a second reason no candidate is recommended
  and a reason a later round may add a candidate;
- the check-in that lets the owner delegate discovery to a model of their choosing is recorded
  under [Discovery record](#discovery-record) as a known gap, not carried as the owner question the
  first version made of it;
- no candidate is adopted and no crate is added to a manifest.

## Owed measurements

None of these ran. Each runs in a throwaway VM with a file-backed pool, never this host's pool and
never with the host's `/dev/zfs`, except items 5 and 10, which are device-free and run on a
throwaway host build with no ZFS path at all.
Each names its control, because a pass alone does not show the checker can fail.
Every measurement stays owed, and no item here touches this host's pool or modules.

1.  A compatible module passes, in the guest. Build or install a module 2.4.4 with BTF in the
    guest, run the built gate, and assert it reports compatible and that a real read request then
    runs. Control: the same binary on a module without BTF must refuse, so a pass is not vacuous.
2.  A changed layout refuses, in the guest. The patched arms start from the 2.4.4 source with
    `META` untouched, so the module still reports `2.4.4-1`, the release rule passes and the
    comparison runs.
    First arm, a member size: drop `zi_match_count` and `zi_inject_count` so
    `zinject_record_t` is 352 bytes, and patch the anonymous struct's pad to the literal
    `char zc_pad1[352]`. The pad's own expression is `sizeof (zinject_record_t) - 16`
    (`include/sys/zfs_ioctl.h:544-552`), so the second patch is what keeps the union at 368 and
    the total at 13,744; without it the struct shrinks to 13,728 and the module fails its own
    `_Static_assert` instead of loading.
    Assert the gate refuses at `Member` with the path of `zc_inject_record` at 13,296, expected 368
    bytes and actual 352, while the struct's total size and the defer offsets match.
    Second arm, a member no offset shows: drop `dds_flags` from `dmu_objset_stats`. Nothing moves,
    the nested struct stays 288 bytes and the total stays 13,744.
    Assert the gate refuses at `Member`, naming `dds_flags` missing at 2,296 bits.
    Control for both arms: the unpatched module from the same build passes the same binary.
    Third arm, only if the guest's packaged 2.2.2 module carries BTF: it reports
    `2.2.2-0ubuntu9.5`, which the release rule refuses at `Version` before any BTF file is read, so
    a layout refusal from that module is unreachable without a seam, and the first version's
    assertion could never hold. With a test seam that skips only the release rule, assert the
    refusal is the container step at `Type`, naming the member found where the anonymous union was
    expected (`zc_inject_record`), which is the first refusal the documented order gives. Without
    the seam, record that the arm proves the release rule only.
    Every arm's refusal names the field and both values.
3.  A missing module BTF refuses and falls back, in the guest. Use the tarball build path that
    skipped BTF (`guest-openzfs-build.log:834-838`) or hide the base file from the checker's
    namespace, and assert raw requests are disabled, that the fallback is offered for an unused
    slot, and that registrations and recovery state survive.
    Control: the same guest after providing the base and rebuilding with BTF passes.
4.  A missing or unreadable file refuses, in the guest. Hide `/sys/kernel/btf` from a mount
    namespace or make a copy unreadable, and assert the refusal names the file and the step.
    Control: the same binary in the same guest with the file readable passes.
5.  A short read refuses, device-free. A test harness fed a truncated module and base pair must
    refuse with the section or header step, not stall; the unit test and the fuzz target carry
    this, and the guest arm repeats it through the product's loader only if a seam exists.
    Control: the untruncated pair passes.
6.  The header variants both pass, in the guest. A module whose BTF an older tool emitted with the
    24-byte header, and a module whose BTF reports `hdr_len` 32 (the host's kernel is
    `7.3.5-1-cachyos`, unqualified for the product until step 23), must both parse and compare.
    The design's `6.8.0-147-generic` guest emitted no module BTF for its tarball build, so this arm
    needs a build route that emits one, with the 24-byte header as its own case.
    Control: a file with `hdr_len` 16, and a file whose sections overlap, must refuse.
7.  The request contract is checked, in the guest. Renumber one `zfs_ioc` value in a patched module
    and assert the refusal names the request and both values. Control: the unpatched module passes,
    and the patched module still has the same struct layout, so the refusal can only come from the
    request comparison.
8.  Architecture and endianness refuse, in the guest. On an `aarch64` guest (unqualified) the gate
    must refuse before any ioctl; on a big-endian BTF fixture the endianness step must refuse.
    Control: the `x86_64` little-endian guest passes.
9.  A module reload between reads is caught, in the guest. Unload and load a different module
    between the gate's first and second version reads, through a test seam or a scripted guest,
    and assert the checker retries once and refuses only when the retry's own two version reads
    differ as well.
    Control: a single load passes, and a reload that has settled before the retry passes, which is
    the arm that shows the retry is not a refusal in disguise.
10. Start cost and memory, device-free on a throwaway host build. Measure the built gate's wall
    time and peak memory against this host's copied BTF files, interleaved with a same-binary
    control that runs the same code without the ZFS path, and report the ratio and the run-to-run
    spread, per the repository's timing rule. Measure the base parse and the module parse
    separately, and the handler's start as a one-process-per-ZED-event child as well as the
    wrapper's longer process, because the two pay the parse differently.
    Control: the no-op path in the same round, and a version-file-only run, which reads no BTF and
    so separates the read cost from the parse cost.
11. The guest fixture for step 6, in the guest. The tarball module the design's guest built carries
    no BTF. Repeat item 1 with a packaged module that carries BTF or with a tarball build given a
    `vmlinux` in its build directory, record which route produced the BTF and its release string,
    and record whether the guest exposes `/sys/kernel/btf/vmlinux`, which nothing records today.
    Control: the same guest's module built without BTF, which must refuse with the missing-BTF step
    and offer the fallback.
    Handoff to step 2's writer: step 2's layout comparison test needs module BTF
    (step 2 of [the implementation plan][plan-file]), so the guest recipe must produce a 2.4.4
    module with BTF, and this item is the run that proves the recipe does.
12. The refused state in `doctor`, in the guest. With a mismatched module loaded, assert `doctor`
    names the loaded module, the target and the file, keeps its exit status, and does not print a
    bypass; then with a compatible module assert it prints the compatible line.
    Control: both states in the same guest.
13. A tool-based checker's output and absence (Way 3), in the guest. With the tool's packaged
    version, run the tool arm on a compatible and on a mismatched module, and assert the refusal
    names the member and both values. Then remove the tool from the guest and assert the refusal
    names the missing program and its version, not the ABI. Repeat with a second tool version.
    Control: the same module with the tool present and matching passes.
14. The root handler's tool path (Way 3), in the guest. As root, run the handler with a writable
    `PATH` containing a decoy `pahole` or `bpftool`, and assert the handler uses a verified absolute
    path or refuses, never the decoy.
    Control: the verified tool at its absolute path passes, and the decoy is never executed.
15. The digest way's cost (Way 4), in the guest. Rebuild the same 2.4.4 source with a different
    compiler or flag set, record whether the module BTF bytes differ, and assert the digest refuses
    while the full layout check passes. If the bytes are identical, use a patched source change that
    keeps the layout. Control: the full check on the same two modules passes both.
16. The blind spots of Ways 2, 5 and 6, in the guest. Use the patched module whose
    `zinject_record` is 352 rather than 368, and a module with one same-size field moved inside a
    nested struct the narrow comparison does not reach (a member of `drr_begin`, not a direct
    member of `zfs_cmd`), and assert the narrow comparison, the request probe and the
    release-string-only check accept both while the full comparison refuses them.
    Control: the full comparison on the unmodified 2.4.4 module passes.

## Questions for the owner

Each question is ready for the question tool. Options are ranked, best first, and each option says
why it sits where it does. The pick is the owner's.

### Q1: which reader parses the module BTF

Ranked first: `btf-rs` 2.0.1, on eligibility and not on validation.
No hard gate fails on the source evidence read, and the crate holds the split model, the type kinds
and the tests the check needs; this proposal's pre-scan closes its two named residuals.
Its finalist validation is outstanding: nobody ran its own test suite, and the pair of library
options flips or ties on one criterion at a time ([Sensitivity](#sensitivity)), which does not
cover simultaneous multi-input changes.

In plain words, the choice between options 1 and 2 is this: take an `LGPL-2.1-or-later` dependency
whose 2.0.1 release lags its own main branch on the widened BTF masks, or write about 350 to 450
source lines and 100 to 150 test lines of our own reader, plus a fuzz target and a review of it,
and add no new dependency.

Options:

1.  `btf-rs` 2.0.1, ranked first, finalist validation outstanding.
    Pros: split BTF and the type kinds this check needs; pure Rust with default features off;
    a documented API and tests; the probe parses this host's files and walks the anonymous union.
    Cons: `LGPL-2.1-or-later` adds a copyleft-family dependency; no section validation in the crate,
    so the binding keeps the pre-scan; its release's masks lag its main branch; a small maintainer
    group; its own test suite is unrun.
    Cost: the shared shape, about 350 to 550 lines of source and 300 to 450 of tests, plus the
    resolver for arrays and typedef chains and one fuzz target.
    Why here: it is the only eligible library, since the skill admits a custom implementation only
    when every existing tool fails a hard constraint; its 90 of 104 trails the in-house reader's 92
    by two points, inside a band one step on C6 reverses.
2.  An in-house reader in the binding, ranked second.
    Pros: exact bounds and refusal semantics in one place; no new dependency or license; can read
    lazily; no resolver shim, since the reader owns array and typedef sizes.
    Cons: a parser of kernel bytes is ours to review, fuzz and maintain; the skill's existing-tools
    rule keeps it behind `btf-rs` while `btf-rs` passes.
    Cost: the same shared shape plus about 350 to 450 lines of reader source and 100 to 150 of
    tests, and its own fuzz target.
    Why here: its 92 of 104 leads `btf-rs`'s 90, and it stays second because a score does not
    override the eligibility rule. Choosing it means overriding the skill's existing-tools rule for
    this package, which is the owner's call on a named ground such as refusing a copyleft
    dependency; the design would record that as an adopted exception, not as a score.
3.  `pahole` 1.32 as a start-time program, ranked third.
    Pros: the command the design already names for the test-time comparison; `-C` prints the
    compared members; no new Rust dependency.
    Cons: a runtime package; a root-spawned program in the handler; a text format; three
    invocations for the struct, the nested type and the enum.
    Why here: 60 of 104, and a runtime program costs the deployment and privilege points that
    options 1 and 2 do not.
4.  `bpftool` 7.8.0 as a start-time program, ranked fourth.
    Pros: libbpf's reader; a documented base option and a raw type listing.
    Cons: the same runtime-program and root-path costs; the raw listing needs a type-id resolver
    in the wrapper before it can compare a member; the tool's version tracks the kernel's tools,
    not the distribution's.
    Why here: 56 of 104, the lowest of the four, for the resolver its output needs and its
    medium diagnostic rating.

Not offered: `btfparse` (no split base), `aya-obj` (no base, ELF-oriented), `btf` 0.5.1
(unmaintained, no base), `btfdump` (no base, tool first), `padlock-dwarf` (no base, wrong fit),
`libbpf-rs` and `libbpf` (C build), and a BTF digest (the answer the project did not select).

### Q2: which module release strings qualify

Settled by the record, adopted with a veto: match the release by version `2.4.4` and accept any
release suffix, printing the whole string in `doctor`.
The record's reason for rejecting exact profiles was that a rebuild must not disable clones
([ABI verification][design-project-decisions]), and an exact `2.4.4-1` match is that rejected cost
in another form, so this file does not ask about it.
Adopted rule: the string qualifies when it is exactly `2.4.4` or begins `2.4.4-`. That rule cannot
match `2.4.40-1`, because the character after `2.4.4` there is `0`, not `-` or the end of the
string. The writer implements it, and the owner may veto it here.

That rule already accepts every build from a git checkout, because `config/zfs-meta.m4:73-88`
replaces the release with `git describe`'s suffix through `tr - _` when a matching tag exists, so a
checkout one commit past `zfs-2.4.4` reports a string like `2.4.4-5_g1a2b3c4`: commits past the
tag, then `_g` and the commit.
The open part is whether to narrow the adopted rule for exactly that shape. The layout check runs
either way, so the narrowing changes only what the string is allowed to say about where the source
came from, which the layout comparison cannot show.

Options:

1.  Keep the adopted rule, accepting a git-snapshot suffix like any other, ranked first.
    Pros: one rule for every build; a checkout build with the approved layout keeps the managed
    route, which is the record's reason for the runtime check. Cons: the loaded module may come
    from source ahead of the tag, and a patch that keeps the layout passes anyway; `doctor` has to
    say plainly that the build is not the release.
    Why here: it is the rule the record's reason already implies, so it adds no second case to
    explain, test or print, and the layout comparison remains the proof.
2.  Narrow it to refuse a suffix of the form `<digits>_g<hex>`, ranked second.
    Pros: the managed route then means a release build, since a checkout at the tag reports
    `2.4.4-1` through the `META` fallback and only a build past the tag grows that suffix.
    Cons: a checkout build even one commit past the tag disables raw requests until it is rebuilt
    at the tag, which is the exact-profile cost the record rejected in a narrower form, and it
    refuses a build whose compared layout is identical.
    Why here: it buys one string fact, that the source is a release checkout, at the cost the
    record rejected, and that fact is not one the gate's five proofs need.

### Q3: may a verification be cached across processes

Held until [Owed measurements](#owed-measurements) item 10 measures the start cost.
Nothing needs the answer before that run: per-process caching with no disk cache is writer-settled,
so the gate works the moment it is built.
The question returns only if item 10 shows a cost the owner will not pay, and it returns with two
options:

1.  No disk cache, ranked first while the cost is unmeasured.
    Pros: no stale verdict, no new state, no new privilege question; the identity recheck stays
    inside the process. Cons: every ZFS command pays the parse.
    Why here: it is the only option that adds no new input to the gate, and nothing measured says
    the parse needs saving.
2.  A per-boot cache bound to the boot id, the module version, both BTF files' device, inode, size
    and modification time, and the binary's build id, ranked second.
    Pros: the parse is paid once per boot. Cons: a forged entry in the account's own cache would
    enable raw requests on a mismatched module, so the file must be root-owned for the handler and
    validated before use, and that validation is the identity check again.
    Why here: it saves a cost not yet measured, and it needs a root-owned file and an extra trust
    argument before it saves anything.

The wrapper's disk cache of the `git version` text is not a precedent for this one: it holds a
descriptive string, and a wrong entry changes a warning, while a verdict here gates raw requests
([The cache](#the-cache)). That difference in trust class, not the parse cost, is why the first
version made this choice; the cost is what may reopen it.

## What a writer can settle without asking

- The placement is a proposal the owner may veto, not a writer's choice: the checker is proposed
  to live in the binding (`package/zfs/request-to-openzfs-kernel-device`), because the design gives
  the binding the request format and the `zfs_cmd` representation and requires both binaries to
  share the gate. The wrapper module supplies the paths, the target allowlist, the release rule and
  the wording. If the owner moves the reader, only the crate boundary moves.
- The gate runs after the device check and before the first ioctl; `doctor` runs the facts without
  the device check.
- The read order and the second version read, with one retry, as
  [The read order and the reload race](#the-read-order-and-the-reload-race) states.
- The pre-scan rules, including the sorted-by-offset section check, the widest-encoding walk and the
  refusal of a value the chosen reader cannot represent.
- The file bounds, as proposals pending [Owed measurements](#owed-measurements) item 10:
  a page for the version, 64 MiB for the base BTF and 16 MiB for the module BTF.
  The caps are at least ten times the sizes measured here (6,579,400 and 570,234 bytes), so an
  ordinary debug kernel fits, but the only memory measurement is that reading and caching the pair
  of 7,159,634 bytes peaked at 57,808 kB resident, about eight bytes of memory per input byte, so a
  64 MiB base could reach hundreds of MiB. A file that exceeds its cap is a refusal until item 10
  measures the real peak and the constant is set from it.
- The compared set: the struct size, every member of `zfs_cmd` with its offset and size, the
  anonymous containers and their members, every member of `dmu_objset_stats` (including
  `dds_flags`) and of `zinject_record`, the base integer sizes, and every request constant plus
  `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.
  The interiors of `drr_begin`, `zfs_stat_t` and `zfs_share_t` stay unchecked beyond their own size
  and offset, as [Limits](#limits) says.
- The expectations come from the Rust `#[repr(C)]` type where `offset_of!` reaches, with literals
  for the anonymous containers and union members, guarded by an exhaustiveness test.
- The refusal value, its steps, the order the comparison walks and the words the caller and
  `doctor` print, including the loaded module, the target, the file and the first mismatch, and no
  bypass.
- The per-process `OnceLock` and the identity recheck, and no disk cache; Q3 waits on item 10.
- The architecture allowlist: `x86_64` only until an `aarch64` guest qualifies the managed route.
- Endianness from the BTF magic against the target.
- The test placement and fixtures: device-free unit tests with small generated BTF blobs, the
  exhaustiveness tests, the diagnostic tests, the `doctor` rendering test, and the fuzz target for
  the loader and pre-scan.
- The reader Q1 picks serves step 2 as well, because step 2's layout test compares the compiled
  layout with the running module's BTF and that is the same comparison. That test keeps its own
  reader: the plan's `pahole --btf_base` command, or another reader that is not the one the binding
  ships, so the proof does not depend on the parser it checks.
- The step 2 handoff: step 2's guest recipe must produce a 2.4.4 module that carries BTF.
  Its tarball build skipped module BTF for want of a `vmlinux` in the build directory
  (`guest-openzfs-build.log:834-838`), and step 2's layout comparison test needs the module BTF to
  exist (step 2 of [the implementation plan][plan-file]).
  The recipe either builds with a `vmlinux` present or installs a packaged module that carries BTF,
  and it records which route produced it and whether the guest exposes
  `/sys/kernel/btf/vmlinux`, which is unrecorded today.
  [Owed measurements](#owed-measurements) item 11 is the run that proves it.
- The refusal's interaction with the design's fallback for an unused slot and the failure path for
  a bound clone; the design decides the behaviour, the writer writes the words.
- The check's placement in step 6, beside the start probe, and the step-2 test remaining the
  proof that the compiled layout matches the pinned headers.
- No external program if Q1 picks a library, no C build, no `zfs` invocation, no `/dev/zfs` read
  for the layout check itself, and no force flag or environment override.
  If Q1 picks a tool, the external program is that tool and
  [Owed measurements](#owed-measurements) items 13 and 14 qualify it.

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
