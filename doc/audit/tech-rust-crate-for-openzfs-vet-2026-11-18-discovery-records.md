# Rust crate for OpenZFS: discovery ledger and query records (2026-11-18)

Not final and not approved.
Status: discovery phase run, the coordinator folds this file into
`doc/audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md`, which owns the frozen context.

## How this file was produced

- Run folder: `~/temp/agent/pi-runs/rust-crate-for-openzfs-discovery-1`.
  Every raw provider body sits in `raw/`, `raw/expansion/` and `raw/repos/`.
- Run window, measured from the saved raw files: 2026-11-18T17:26:59-05:00 to
  2026-11-18T17:38:00-05:00.
- Provider of the run: Pi on DeepSeek V4.1 Flash, per the report's "Active audit owner".
- Bounds kept: metadata only. Nothing was cloned, built, or run from any candidate.
  Every fact below comes from a registry response, a repository page, or a README.
  crate registry calls used `curl --silent --user-agent tech-vet-discovery`; GitHub calls used `gh`.
- Screening proves or disproves the four hard gates the report lists and nothing else:
  category mismatch, no inspectable source, a plainly incompatible stated license,
  a plainly unsupported platform.
  Operation coverage, bindings against 2.4.4, maintenance and test evidence are the
  frozen criteria and are left to later phases.
- The CDDL question (linking OpenZFS userland libraries into an LGPL-3.0-or-later program)
  is untouched, as the report directs.

## Candidate ledger

Grouped by what each candidate binds or speaks.
"Found by" names the query that first returned it, in run order;
the per-page detail is in "Query records".

### libzfs, the wrapper around `libzfs` (route 1, survivor)

- Latest version and date: `libzfs` 0.6.16, published 2020-08-28.
  Companions in the same repository and the same route: `libzfs-sys` 0.5.11 (2019-08-05)
  and `libzfs-types` 0.1.2 (2020-08-28).
- Source repository: `https://github.com/whamcloud/rust-libzfs`, archived (last push 2023-07-11).
- Found by: crates.io search `zfs` page 3; GitHub query `zfs --language=Rust`;
  GitHub code search `lzc_snapshot`; lib.rs keyword page.
- Binds or speaks: `libzfs` through `libzfs-sys`, which is bindgen output over the
  `libzfs` headers; its README says "Bindings to libzfs 0.7.13" and "they should only be
  used against this version". `libnvpair` comes in through `nvpair-sys ^0.1`.
- License stated by the registry: MIT, on all three crates.
- Build shape, read from the published `build.rs` and the registry dependency list of
  `libzfs-sys` 0.5.11: the build runs `pkg_config` and probes `libzfs` with
  `atleast_version("0.7.13")`, then links `zpool`; the bindgen output is checked in as
  `src/bindings.rs`, so bindgen and libclang are only needed to regenerate it.
  A build therefore needs `pkg-config` and a `libzfs.pc` that reports 0.7.13 or newer.
- Screening: pass. Category is a ZFS administration library, source is inspectable,
  MIT is compatible, and Linux x86-64 is what `libzfs-sys` was generated on.
  Two facts belong to later criteria, not to a gate: the upstream is archived, and the
  bindings are pinned to a 2019 header set, so "works against OpenZFS 2.4.4" is open.

### zfs-core, the lzc and nvpair bindings (route 2, survivor)

- Latest version and date: `zfs-core` 0.5.0, published 2020-12-14.
  Companions: `zfs-core-sys` 0.5.0 (2020-12-14), `nvpair` 0.5.0 (2020-12-14),
  `nvpair-sys` 0.4.0 (2020-09-12).
- Source repository: `https://github.com/jmesmon/rust-libzfs`, which redirects to
  `https://github.com/codyps/rust-libzfs`; last push 2023-03-10; 18 stars.
- Found by: crates.io search `zfs` page 1; GitHub code search `lzc_snapshot`, `lzc_clone`,
  `lzc_destroy`, `zfs_prop_set`; GitHub query `libzfs --language=Rust`.
- Binds or speaks: `libzfs_core` (lzc) through `zfs-core-sys`, and `libnvpair` through
  `nvpair-sys`. `zfs-core-sys` declares build dependency `pkg-config ^0.3` and no bindgen,
  so its bindings are pre-generated.
- License stated by the registry: `Apache-2.0 OR MIT`, on all four crates.
- Screening: pass. Library category, inspectable source, compatible license, and the
  repository's own CI note ("Workaround for missing libs on freebsd 13") shows both
  FreeBSD and Linux are built.
  Open for later phases: 2020-era `libzfs_core` bindings against 2.4.4.

### razor-rs, the razor bindings family (route 3, survivor)

- Latest version and date: `razor-libzfscore` 0.13.1, `razor-libzfscore-sys` 0.13.1,
  `razor-libnvpair` 0.13.1, `razor-libnvpair-sys` 0.13.1, `razor-nvpair` 0.13.1,
  all published 2023-07-31; `razor-libzfs` 0.12.0, `razor-libzfs-sys` 0.12.0 and
  `razor-safe-lzc` 0.12.0, published 2022-07-26.
- Source repository: `https://github.com/razor-zfs/razor-rs`, last push 2023-08-27, 4 stars.
- Found by: crates.io search `zfs` page 1; GitHub query `openzfs --language=Rust`;
  GitHub code search `lzc_snapshot`, `lzc_clone`, `lzc_destroy`, `zfs_prop_set`.
- Binds or speaks: `libzfs` (`razor-libzfs-sys`, bindgen 0.60 plus pkg-config),
  `libzfs_core` (`razor-libzfscore-sys`, bindgen 0.66 plus pkg-config), `libnvpair`
  (`razor-libnvpair-sys`, bindgen 0.66 plus pkg-config). A build therefore needs libclang
  and the OpenZFS development files.
- License stated by the registry: `MIT OR Apache-2.0` on all eight crates.
- Screening: pass. Library category, inspectable source, compatible license, Linux is the
  platform the Razor project targets (its crates carry `zfs` keyword and Linux-oriented docs).
  Open for later phases: bindings generated in 2022 and 2023, and no release since.

### libzetta (route 4, survivor)

- Latest version and date: `libzetta` 0.5.0, published 2023-05-13.
  Companion: `libzetta-zfs-core-sys` 0.5.2 (2023-04-01).
- Source repository: `https://github.com/Inner-Heaven/libzetta-rs`, last push 2026-02-23,
  76 stars. (`asomers/libzetta-rs` is a fork of it.)
- Found by: crates.io search `zfs` page 1; lib.rs keyword page; web search
  `zfs ioctl rust`, `rust zfs snapshot clone library`.
- Binds or speaks: `libzfs_core` through `libzetta-zfs-core-sys` (declares `pkg-config` and
  `build-env` build dependencies), `libnvpair` through `libnv ^0.4.2`, and falls back to the
  `zfs(8)` and `zpool(8)` programs where lzc falls short (its README: "This library resorts
  to `zfs(8)` and `zpool(8)` where `lzc` falls shorts"). It also declares a `cmake` build
  dependency.
- License stated by the registry: `BSD-2-Clause`, which the repository confirms.
- Screening: pass. Library category, inspectable source, BSD-2-Clause is compatible,
  and the README states Linux support ("Verified on what is available for Ubuntu 20.04 at
  the time of writing it's `0.8.3`") alongside its FreeBSD focus.
  Open for later phases: the `libnv` dependency is FreeBSD's libnv, its last release is
  2023, and it shells out for part of the surface.

### zfskit (route 5, survivor with a design question)

- Latest version and date: `zfskit` 0.3.0, published 2026-10-06.
- Source repository: `https://github.com/okhsunrog/zfskit`, last push 2026-11-04, 1 star.
- Found by: crates.io search `zfs` page 1; lib.rs keyword page; web search
  `rust zfs snapshot clone library`; GitHub query `zfs --language=Rust`.
- Binds or speaks: neither OpenZFS library. It drives the `zfs(8)` and `zpool(8)` programs
  and parses their `-j` JSON output (its README: "CLI-only, no FFI").
- License stated by the registry: `MIT OR Apache-2.0`.
- Screening: pass on the four gates: library category, inspectable source, compatible
  license, Linux supported, and it declares no FFI at all.
  The question this candidate raises is a design one for the coordinator, not a gate:
  a crate that shells out to `zfs` reaches the same program the owner declined on
  2026-11-18, so "passes the gates" is not the same as "is the chosen crate".
  Its runtime dependencies are `tokio`, `serde`, `serde_json`, `regex`, `thiserror`,
  `async-trait` and `zeroize`.

### nessie-backend-zfs (route 6, survivor, weak fit)

- Latest version and date: `nessie-backend-zfs` 0.4.4, published 2026-11-03.
- Source repository: `https://github.com/Gilamonster-Foundation/nessie-store`,
  last push 2026-11-03.
- Found by: crates.io search `zfs` page 1.
- Binds or speaks: the `zfs`, `zpool` and `exportfs` programs, behind a `CommandRunner`
  seam, as a storage backend for the nessie-store daemon.
- License stated by the registry: `MIT OR Apache-2.0`.
- Screening: pass on the four gates. Its API is a storage-backend trait rather than a ZFS
  administration API, so it is a weak candidate for the wrapper's needs;
  that judgement belongs to the fit criterion, and it is recorded here so the
  coordinator can drop it early.

### zfsdev, pure Rust over `/dev/zfs` (route 7, survivor)

- Latest version and date: 0.4.8, read from the `Cargo.toml` at the repository head.
  The crate sets `publish = false`, so there is no crates.io version and no registry license
  entry. Releases are git tags `vX.Y.Z`.
- Source repository: `https://github.com/Ukko-Ylijumala/zfsdev-rs`, last push 2026-11-09.
- Found by: GitHub query `zfs --language=Rust`, which returned it in the frozen 100;
  also returned by the `ZFS_IOC_` code search and by the expansion run for `zfsdev`.
- Binds or speaks: the `/dev/zfs` ioctl interface directly. Its README: "No libzfs, no FFI,
  no zfs/zpool output scraping. The crate builds `zfs_cmd_t` requests itself and speaks the
  kernel's packed-nvlist wire format with its own codec." Mutating operations sit behind the
  `write` feature. `edition = "2024"`, `rust-version = "1.88"`.
- License stated where it is stated: `MIT OR Apache-2.0`, from the `license` field in
  `Cargo.toml` and the one-line `LICENSE` file. It is not a registry entry, so this is a
  repository statement, not a registry one.
- Screening: pass. Library category, inspectable source, compatible license, Linux only by
  construction (it is the Linux ioctl interface).
  Two facts for later phases: it vendors OpenZFS reference material under the CDDL in a
  `doc/` folder that the crate excludes from packaging, and it must track the 2.4.4 ioctl
  interface itself, which is exactly the maintenance cost the report flagged.
  A git dependency is also outside the registry's version resolution.

### clinta/zfs-rs, direct ioctls with libnvpair (route 8, survivor, weak fit)

- Latest version and date: `0.0.1` in the `Cargo.toml` at the repository head; never
  published. Last push 2020-10-06.
- Source repository: `https://github.com/clinta/zfs-rs`, 1 star.
- Found by: GitHub query `zfs --language=Rust`; GitHub code search `ZFS_IOC_`,
  `zfs_prop_set`.
- Binds or speaks: `/dev/zfs` ioctls directly plus `libnvpair` (its README: "this now
  directly sends ioctls to /dev/zfs. The only C library required for usage is libnvpair").
- License stated by the repository: `MIT OR Apache-2.0`, from the `Cargo.toml` `license`
  field (no registry entry).
- Screening: pass on the four gates. It is a 2020 work in progress with no release and a
  typed-properties generator, so its fit is doubtful; recorded so the coordinator can drop
  it early.

### moretea/rust-zfs (route 9, survivor, weak fit)

- Latest version and date: `0.1.0` in the repository `Cargo.toml`; never published.
  Last push 2018-09-16.
- Source repository: `https://github.com/moretea/rust-zfs`, 2 stars.
- Found by: GitHub query `zfs --language=Rust`.
- Binds or speaks: `libzfs` ("Rust binding to libzfs").
- License stated by the repository: LGPL-3.0, which is the wrapper's own license family and
  therefore compatible.
- Screening: pass on the four gates, on a 2018 repository with no release.
  Recorded for completeness; nothing in its metadata suggests it can drive 2.4.4.

### libzfs_core-sys (pending)

- Latest version and date: 0.1.0, published 2017-11-05.
- Source repository: none declared in the registry entry.
- Found by: crates.io search `zfs` page 3.
- Binds or speaks: `libzfs_core`, with a checked-in `bindings.rs`, `build.rs` and `zfscore.h`;
  it depends on `libnv-sys ^0.1.2`, which is the FreeBSD `libnv`.
- License stated by the registry: `BSD-2-Clause`.
- Screening: pending targeted evidence. The only open gate is platform: the crate's
  dependency on FreeBSD's `libnv-sys` and its 2017 vintage leave whether it builds on
  Linux x86-64 unproven from metadata, and it is a raw binding layer with no API to drive.
  A targeted answer is one build attempt in the validation phase, not this phase.

### Crates that read or reimplement the ZFS on-disk format (category fail)

Each of these is about the ZFS format, not about administering a pool, so it cannot create a
snapshot, clone, destroy, or set a user property.

- `zfs` 0.1.0 (2016-11-18), MIT, `https://github.com/ticki/zfs`: "A ZFS Implementation for
  Redox", an on-disk file system, unrelated to `libzfs`.
- `lamzfs` 0.1.4 (2026-10-15), MIT, `https://github.com/lamco-admin/lamzfs`: read-only
  `no_std` pool reader for UEFI bootloaders.
- `zfs-forensic-core` 0.2.1 (2026-11-11), Apache-2.0,
  `https://github.com/SecurityRonin/zfs-forensic`: on-disk reader for forensic analysis.
- `filer-zfs` 0.0.4 (2026-11-05), Unlicense, `https://github.com/artob/filer`: read-only
  file system driver for a Filer API.
- `lcpfs` 2026.1.102 (2026-01-03), Apache-2.0: "A ZFS-inspired copy-on-write filesystem",
  an unrelated file system that borrows ZFS ideas.
- `turlando/zfs-rs` and `okhsunrog/zfs-rs`, found by the GitHub query
  `zfs --language=Rust`: further from-scratch on-disk implementations, read-only.

### Placeholder and empty publishes (no inspectable implementation)

- `rzfs_lib` 0.0.0 and `rzfs_userspace` 0.0.0 (2024-02-05), `GPL-2.0+ OR MIT`,
  no repository: the published `lib.rs` is 43 bytes.
- `zfs-rs`, `zfs-rs-macros`, `zfs-rs-nvpair-sys`, `zfs-rs-zfs-sys` 0.0.0 (2020-09-21),
  `MIT OR Apache-2.0`, no repository: the published `lib.rs` is zero bytes.
- `abgleich-lib` 0.2.0-alpha6 (2026-04-20), `MIT OR Apache-2.0`,
  `https://github.com/pleiszenburg/abgleich`: the published `lib.rs` is 207 bytes and the
  project it belongs to is Python, so the Rust crate is a shim.

These fail as "no inspectable source": the only source the registry distributes is empty or
a re-export, and no repository holds a fuller version.

### AGPL-licensed candidates (license fail)

- `zfs-cmd-api` 0.2.0 (2019-11-22), `https://github.com/jmesmon/zoop`: the registry states
  `AGPL-3.0-or-later`, and it drives the `zfs` command line.
  AGPL-3.0's network clause and its copyleft reach the whole combined program, which a
  distributor of the wrapper could not honour under an LGPL-3.0-or-later notice, so this is
  the license gate failing rather than the deferred CDDL question.
  It is the one license judgement this phase makes on its own reading; if the license phase
  reads AGPL-3.0 section 13 to permit this link, this entry moves to pending.
- `zfs_wrapper` 0.1.0 (2024 edition), `https://github.com/ChocolateLoverRaj/zfs_wrapper`:
  GitHub detects AGPL-3.0. A library over the `zpool` and `zfs` commands, never published.

### Repositories that bind `libzfs` without stating a license (fail: no grant)

None of these four has a `LICENSE` file or a `license` field, so there is no grant to link it
at all, which fails the license constraint more plainly than an incompatible license would.

- `https://github.com/wfraser/libzfs-rs`, 2025-10-12, "Idiomatic Rust bindings for libzfs",
  `libzfs` 0.1.0 plus a `libzfs-sys` folder. The most recent of the four.
- `https://github.com/kavan-mevada/libzfs`, 2021-12-03, "Rust bindings of libZFS",
  `zfs` 0.1.0 with `build.rs` and `wrapper.h`.
- `https://github.com/akhilerm/libzfs-rs`, 2019-08-07, `libzfs-rs` 0.1.0 plus `libzfs-sys`.
- `https://github.com/problame/libzfs-rs`, 2018-09-23, `libzfs-bindings` and `libzfs-rs`.
- `https://github.com/sgrankin/zfs-sys`, 2017-08-23, `zfs-sys` 0.7.1, no license either.

### FreeBSD-only bindings (platform fail)

- `freqlabs/rs-libzfs-sys`, `freqlabs/rs-libzfs_core-sys`, `freqlabs/rs-libnvpair-sys` and
  `freqlabs/rs-libavl-sys` on `gitlab.com` (found through the web search `rust zfs bindings`
  and the gitlab topic page): "Rust bindings for libzfs system library on FreeBSD 11.2+".
  The stated platform is FreeBSD, not Linux x86-64.

### `libnvpair` and `libnv` codecs only (category fail, kept because routes need them)

These bind one OpenZFS C library and provide no dataset operations, so none can be the
wrapper's driver; they are dependencies of routes 1, 2 and 4, and are recorded so a later
phase knows what each route pulls in.

- `nvpair` 0.5.0 and `nvpair-sys` 0.4.0: already counted inside route 2.
- `razor-nvpair` 0.13.1, `razor-libnvpair` 0.13.1 and `razor-libnvpair-sys` 0.13.1:
  already counted inside route 3.
- `nvpair-rs` 0.3.0, `stone-libnvpair` 0.2.0, `stone-libnvpair-sys` 0.2.0 (2022-09),
  MIT, `https://github.com/stone-rs/nvpair-rs`: `stone-libnvpair-sys` declares `bindgen`
  and `cc` build dependencies, so it compiles vendored C.
- `illumos-nvpair` 0.4.0 (2026-05-11) and `illumos-nvpair-sys` 0.2.0 (2026-04-13),
  MPL-2.0, `https://github.com/oxidecomputer/illumos-nvpair`: bindings to illumos's
  `libnvpair`, so the platform is illumos rather than Linux.
- `libnv` 0.5.2 (2026-06-18), BSD-2-Clause, `https://github.com/Inner-Heaven/libnv-rs`,
  with its sibling `libnv-sys`: bindings to FreeBSD's `libnv`.

### Raw binding layers with no API (fail: nothing to drive)

- `libzfs-sys` 0.5.11 and `libzfs-types` 0.1.2 are raw layers; they survive only as part of
  route 1, which has a wrapper.
- `johnramsden/rust-zfs-sys` 0.0.0 (2019-01-05), MIT,
  `https://github.com/johnramsden/rust-zfs-sys`: "Rust C bindings for libzfs", no wrapper and
  no README.
- `zfs-rs-zfs-sys`, `zfs-rs-nvpair-sys` 0.0.0: empty, listed above.

### Applications, not libraries (category fail)

Each is a program built on ZFS; none publishes a library the wrapper could link as its driver.

- `chithi` 0.2.0 (2026-05-17), GPL-3.0-or-later, `https://github.com/ifazk/chithi`:
  OpenZFS replication tools, a binary with a 2.5 KB `lib.rs` re-export.
- `sam-zfs-unlocker` 0.2.0 (2025-12-16), MIT,
  `https://github.com/TheQuantumPhysicist/sam-zfs-unlocker`: lock and unlock via `sudo`
  commands, which contradicts the no-root hard constraint as well as the category.
- `zfs-autosnap` 0.4.0, `shock` 0.1.11, `zfshealth` 0.1.3, `zftop` 0.3.1, `ztop` 0.3.1:
  snapshot, scrub, monitoring and dashboard tools found by the keyword and category listings.
- `DanielArmengod/zfs-rs` 0.1.0, `https://github.com/DanielArmengod/zfs-rs`: a snapshot
  replication CLI, not a library.
- `InkerBot/zfs-unlock` 0.3.0, MIT, `https://github.com/InkerBot/zfs-unlock`: a dataset
  unlock CLI, found by the code searches for `zfs_prop_set` and `libnvpair`.
- `oxidecomputer/zfs-atomic-change-key`, MPL-2.0: a tool that changes encryption keys through
  ZFS channel programs, found by the web search `rust zfs bindings`.
- `komori/Honeydew` (2022), a snapshot cleaner, found by the GitHub query
  `openzfs --language=Rust`.

### Neighbouring projects that are not a Rust crate for this purpose

- `iomesh/libuzfs-rs` (`uzfs` 0.1.0), Apache-2.0,
  `https://github.com/iomesh/libuzfs-rs`: Rust APIs over ZFS userland code compiled into a
  Rust process, for building a userspace file system. It is not an administration API, and
  it compiles ZFS C code, which is the CDDL question in a different shape.
- `robn/veneer`, MPL-2.0, `https://github.com/robn/veneer`: "Some sketches and experiments
  for an alternate ZFS userspace", no crate and no version.
- `truenas/truenas_ros`, MIT, `https://github.com/truenas/truenas_ros`: Rust bindings to
  Linux file system and mount syscalls for TrueNAS, no ZFS administration API.
- `AlexanderThaller/calipe-rs`, no license stated: a Prometheus metrics agent that reads ZFS
  counters.
- `openzfs/zfs` itself, C, CDDL: the library the routes link, not a candidate.

## Query records

Conventions used in every record below.

- "Returned" is the count the provider reported for that page; "total" is the provider's own
  reported total for the query.
- "New candidates" and "new survivors" count first appearances, cumulatively across the whole
  run, so a page that repeats a crate found earlier shows an empty list.
- A "candidate" is a crate or repository whose own metadata answers the report's question:
  a library that could create, clone, destroy, list and set properties of datasets.
  A ZFS-backed backup tool, dashboard, or on-disk reader is not a candidate and appears only
  in the appendix and in the ledger's fail groups.
- A "survivor" is a candidate that passed all four screening gates at the point of first
  appearance; the pending and failed candidates are named in the ledger.

### Source class 1: crates.io

Provider: `https://crates.io/api/v1/crates`, `curl --silent --user-agent tech-vet-discovery`.
Filters: none beyond the query string. Sort: `sort=relevance`, `per_page=100`.

#### Search query `zfs`

- Literal query: `https://crates.io/api/v1/crates?q=zfs&per_page=100&page=1&sort=relevance`.
- Total reported: 222.
- Page 1: returned 100. New candidates: `zfs`, `zfs-forensic-core`, `zfskit`,
  `nessie-backend-zfs`, `zfs-rs-zfs-sys`, `filer-zfs`, `abgleich-lib`, `libzetta`, `lamzfs`,
  `sam-zfs-unlocker`, `zfs-rs-nvpair-sys`, `zfs-rs-macros`, `zfs-cmd-api`, `zfs-rs`,
  `libzetta-zfs-core-sys`, `zfs-core-sys`, `zfs-core`, `razor-libzfscore-sys`,
  `razor-libzfscore`, `lcpfs`, `razor-safe-lzc`, `razor-libnvpair-sys`, `razor-libnvpair`,
  `razor-nvpair`, `razor-libzfs-sys`, `razor-libzfs`, `chithi`, `rzfs_lib`, `rzfs_userspace`.
  New survivors: `zfskit`, `nessie-backend-zfs`, `libzetta`, `zfs-core-sys`, `zfs-core`,
  `razor-libzfscore-sys`, `razor-libzfscore`, `razor-safe-lzc`, `razor-libzfs-sys`,
  `razor-libzfs`.
- Page 2: returned 100. New candidates: `illumos-nvpair-sys`, `libnv`. New survivors: none.
- Page 3: returned 22, which exhausts the reported total. New candidates: `libzfs-types`,
  `libzfs-sys`, `libzfs`, `libzfs_core-sys`. New survivors: `libzfs-types`, `libzfs-sys`,
  `libzfs`.

#### Search query `openzfs`

- Literal query: `https://crates.io/api/v1/crates?q=openzfs&per_page=100&page=1&sort=relevance`.
- Total reported: 31. Page 1: returned 31, which exhausts the total.
- New candidates: none (every crate naming OpenZFS was already recorded above).
- New survivors: none.

#### Search query `libzfs`

- Literal query: `https://crates.io/api/v1/crates?q=libzfs&per_page=100&page=1&sort=relevance`.
- Total reported: 17. Page 1: returned 17, which exhausts the total.
- New candidates: none. New survivors: none.

#### Search query `libzfs_core`

- Literal query: `https://crates.io/api/v1/crates?q=libzfs_core&per_page=100&page=1&sort=relevance`.
- Total reported: 11. Page 1: returned 11, which exhausts the total.
- New candidates: none. New survivors: none.

#### Search query `lzc`

- Literal query: `https://crates.io/api/v1/crates?q=lzc&per_page=100&page=1&sort=relevance`.
- Total reported: 15. Page 1: returned 15, which exhausts the total.
- New candidates: none. New survivors: none.
- Note: this query also returns `lzc-sdk` (LazyCat application APIs), `newtua-lzw-z` and the
  `lz*` compression family. They are named "lzc" and are not ZFS; no candidate.

#### Search query `zpool`

- Literal query: `https://crates.io/api/v1/crates?q=zpool&per_page=100&page=1&sort=relevance`.
- Total reported: 25. Page 1: returned 25, which exhausts the total.
- New candidates: none. New survivors: none.

#### Search query `nvpair`

- Literal query: `https://crates.io/api/v1/crates?q=nvpair&per_page=100&page=1&sort=relevance`.
- Total reported: 11. Page 1: returned 11, which exhausts the total.
- New candidates: `nvpair`, `nvpair-rs`, `illumos-nvpair`, `nvpair-sys`, `stone-libnvpair-sys`,
  `stone-libnvpair`.
- New survivors: none (all six are nvpair codecs; route 2's `nvpair` and `nvpair-sys` were
  first seen under `zfs`).

#### Search query `zfs snapshot`

- Literal query: `https://crates.io/api/v1/crates?q=zfs%20snapshot&per_page=100&page=1&sort=relevance`.
- Total reported: 70. Page 1: returned 70, which exhausts the total.
- New candidates: none. New survivors: none.

#### Search query `zfs clone`

- Literal query: `https://crates.io/api/v1/crates?q=zfs%20clone&per_page=100&page=1&sort=relevance`.
- Total reported: 103.
- Page 1: returned 100. New candidates: none. New survivors: none.
- Page 2: returned 3, which exhausts the total. New candidates: none. New survivors: none.
- Pagination stopped here by the report's rule: two complete pages in a row added no
  screening survivor.

#### Search query `zfs ioctl`

- Literal query: `https://crates.io/api/v1/crates?q=zfs%20ioctl&per_page=100&page=1&sort=relevance`.
- Total reported: 4. Page 1: returned 4, which exhausts the total.
- New candidates: none. New survivors: none.

#### Keyword listing `zfs`

- Literal query: `https://crates.io/api/v1/crates?keyword=zfs&per_page=100&page=1`.
- Total reported: 40. Page 1: returned 40, which exhausts the total.
- New candidates: none. New survivors: none.

#### Category lookup

- Literal query: `https://crates.io/api/v1/categories`, then the same with `page=1&per_page=100`.
- Total reported: 58 categories, returned 58.
- The registry does have a filesystem category: slug `filesystem`, 4728 crates.
  Also present and rejected as the wrong category: `os`, `hardware-support`.

#### Category search `filesystem` with `q=zfs`

- Literal query:
  `https://crates.io/api/v1/crates?category=filesystem&q=zfs&per_page=100&page=1`.
- Total reported: 81. Page 1: returned 81, which exhausts the total.
- New candidates: none. New survivors: none.

#### Reverse dependencies

Each query is `https://crates.io/api/v1/crates/<name>/reverse_dependencies?per_page=100&page=1`;
every one reported a total of 6 or fewer and returned it on page 1, so each ran one page.
Dependents are the crates that depend on the named crate.

- `libzfs`: total 0.
- `libzfs-sys`: total 1, dependent `libzfs`.
- `libzfs-types`: total 2, dependents `device-types`, `libzfs`.
- `libzfs_core-sys`: total 0.
- `zfs-core`: total 0.
- `zfs-core-sys`: total 1, dependent `zfs-core`.
- `nvpair`: total 1, dependent `zfs-core`.
- `nvpair-sys`: total 6, dependents `libnv`, `libzetta-zfs-core-sys`, `libzfs`, `libzfs-sys`,
  `nvpair`, `zfs-core-sys`.
- `razor-libzfs`: total 0.
- `razor-libzfs-sys`: total 1, dependent `razor-libzfs`.
- `razor-libzfscore`: total 1, dependent `razor-safe-lzc`.
- `razor-libzfscore-sys`: total 1, dependent `razor-libzfscore`.
- `razor-libnvpair`: total 5, dependents `razor-libzfs`, `razor-libzfs-sys`,
  `razor-libzfscore`, `razor-libzfscore-sys`, `razor-nvpair`.
- `razor-libnvpair-sys`: total 1, dependent `razor-libnvpair`.
- `razor-nvpair`: total 1, dependent `razor-safe-lzc`.
- `razor-safe-lzc`: total 0.
- `libzetta-zfs-core-sys`: total 1, dependent `libzetta`.
- `nvpair-rs`: total 0.
- `stone-libnvpair-sys`: total 1, dependent `stone-libnvpair`.
- `illumos-nvpair-sys`: total 2, dependents `illumos-nvpair`, `libtopo`.
- `libnv`: total 2, dependents `capsicum`, `libzetta`.

New candidates from these 21 pages: none. `libtopo`, `capsicum` and `device-types` are
illumos and FreeBSD support crates, not ZFS administration crates.
`zfsdev`, `zfsdev-rs`, `clinta/zfs-rs` and the other repository-only candidates have no
crates.io entry, so they have no reverse dependencies to ask for.

### Source class 2: GitHub

Provider: the `gh` CLI (version 2.105.0, account `Aquaticat`), `gh search repos`,
`gh search code`, `gh repo list`. One result set per query, requested with `--limit=100`
as the schedule fixes.

#### Repository searches

- `gh search repos zfs --language=Rust --limit=100`: returned 100, which is the requested
  limit, and `gh` reports no total. A supplementary run at `--limit=1000`
  (recorded below) shows 155 repositories exist, so this frozen query is capped.
  New candidates among the 100: `whamcloud/rust-libzfs`, `Inner-Heaven/libzetta-rs`,
  `okhsunrog/zfskit`, `Ukko-Ylijumala/zfsdev-rs`, `robn/veneer`, `iomesh/libuzfs-rs`,
  `clinta/zfs-rs`, `DanielArmengod/zfs-rs`, `ChocolateLoverRaj/zfs_wrapper`,
  `johnramsden/rust-zfs-sys`, `moretea/rust-zfs`, `sgrankin/zfs-sys`, `ticki/zfs`.
  New survivors: `whamcloud/rust-libzfs`, `Inner-Heaven/libzetta-rs`, `okhsunrog/zfskit`,
  `Ukko-Ylijumala/zfsdev-rs`, `clinta/zfs-rs`, `moretea/rust-zfs`.
- `gh search repos zfs --topic=zfs --limit=100`: returned 100, the requested limit, again
  capped; the supplementary run shows 761 exist.
  New candidates: none. New survivors: none.
- `gh search repos openzfs --language=Rust --limit=100`: returned 12, below the limit.
  New candidates: `ifazk/chithi`, `razor-zfs/razor-rs`. New survivors: `razor-zfs/razor-rs`.
  `komori/Honeydew` also appeared and is an application.
- `gh search repos openzfs --topic=zfs --limit=100`: returned 60, below the limit.
  New candidates: none; its Rust members were already found, and its other 47 are Shell,
  Python, Go, Nix and documentation repositories.
- `gh search repos libzfs --language=Rust --limit=100`: returned 8, below the limit.
  New candidates: `codyps/rust-libzfs`, `akhilerm/libzfs-rs`, `kavan-mevada/libzfs`,
  `problame/libzfs-rs`, `wfraser/libzfs-rs`. New survivors: `codyps/rust-libzfs`.
- `gh search repos libzfs --topic=zfs --limit=100`: returned 7, below the limit.
  New candidates: none; the rest are Python, D and C projects (`py-libzfs`,
  `symmetryinvestments/libzfs`, `Thalhammer/zfspp`, `simons-public/snapzap`).
- `gh search repos libzfs_core --language=Rust --limit=100`: returned 1, below the limit.
  New candidates: none (`codyps/rust-libzfs`, already found).
- `gh search repos libzfs_core --topic=zfs --limit=100`: returned 1, below the limit.
  New candidates: none (`symmetryinvestments/libzfs-core-d`, D language).

#### Supplementary repository probes (beyond the frozen limit)

Recorded separately because they exceed the schedule's `--limit=100`; they exist only to say
whether the frozen limit hid repositories.

- `gh search repos zfs --language=Rust --limit=1000`: returned 155. New candidates beyond the
  frozen 100: `chrisdsa/zfs2s3`-style tools only, no new candidate; the additions are
  backup, monitoring and OS-integration programs.
- `gh search repos zfs --topic=zfs --limit=1000`: returned 761, of which 43 are Rust and all
  43 already appear in the `zfs --language=Rust` set. New candidates: none.

#### Code searches

Each of these reports hit and repository counts, not a paging total; each returned fewer than
the requested 100 hits.

- `gh search code lzc_snapshot --language=Rust --limit=100`: 6 hits in 4 repositories:
  `codyps/rust-libzfs`, `razor-zfs/razor-rs`, `sgrankin/zfs-sys`, `Inner-Heaven/libzetta-rs`.
  New candidates: none; all four were already returned by the repository searches above.
  New survivors: none.
- `gh search code lzc_clone --language=Rust --limit=100`: 4 hits in 3 repositories, all
  already found.
- `gh search code lzc_destroy --language=Rust --limit=100`: 6 hits in 4 repositories, all
  already found.
- `gh search code zfs_prop_set --language=Rust --limit=100`: 8 hits in 5 repositories:
  `sgrankin/zfs-sys`, `razor-zfs/razor-rs`, `InkerBot/zfs-unlock`, `codyps/rust-libzfs`,
  `clinta/zfs-rs`. New candidates: none; `InkerBot/zfs-unlock` is an application
  (its ledger entry is with the applications). New survivors: none.
- `gh search code ZFS_IOC_ --language=Rust --limit=100`: 13 hits in 10 repositories.
  New candidates: none; it returned four repositories the repository searches had not:
  `AlexanderThaller/calipe-rs` (metrics agent), `truenas/truenas_ros` (syscall bindings),
  `mminkus/zfs-explorer` (TypeScript) and `tim-day-387/ktest`, and none of the four is a
  candidate. It also returned `Ukko-Ylijumala/zfsdev-rs`, `robn/veneer`, `ticki/zfs`,
  `clinta/zfs-rs`, `sgrankin/zfs-sys` and `codyps/rust-libzfs` again.
  New survivors: none.

#### The OpenZFS organization itself

- `gh repo list openzfs --limit 200 --json name,primaryLanguage,description`: returned 10
  repositories, below the limit, so nothing was cut. Their languages are C (`zfs`, `spl`,
  `openzfs`), Python, Shell, Groovy, HTML and Dockerfile. No Rust project.

### Source class 3: the wider web

Provider: the agent's `web_search` tool, and `web_fetch` for the two lib.rs pages.

- `rust zfs bindings`: 38 results. New candidates: `freqlabs/rs-libzfs-sys`,
  `freqlabs/rs-libzfs_core-sys`, `freqlabs/rs-libnvpair-sys` (GitLab, FreeBSD),
  `DeterminateSystems/rust-libzfs` (an archived fork of the whamcloud repository whose
  `libzfs-sys` README says the bindings are for libzfs 0.7.13).
  New survivors: none.
- `rust libzfs_core crate`: 47 KB of results. New candidates: none; it returns
  `delphix/rust-libzfs`, `Inner-Heaven/rust-libzfs` and `razor-safe-lzc`, all already known
  (the first two are earlier names or forks of the `codyps` repository).
- `openzfs rust crate`: 93 KB of results. New candidates: none.
  `openzfsonwindows/openzfs` is a C project; `lib.rs` mentioned `zfskit`, already known.
- `zfs ioctl rust`: 78 KB of results. New candidates: none.
  It surfaced a lobste.rs discussion of `libzfs` and a `Project-FiFo/vmadm` source file with a
  `zfs` module, which is a Zig project, not a Rust crate.
- `rust zfs snapshot clone library`: 102 KB of results. New candidates: none.
  `nessie-backend-zfs`, `zfskit` and `mitosis-db` are already known or are applications.
- `rust nvpair crate`: 100 KB of results. New candidates: `oxidecomputer/illumos-nvpair`
  (and its `-sys` companion). New survivors: none.
- `https://lib.rs/keywords/zfs`: one page, 9.6 KB, 52 crate links. New candidates: none;
  every crate on it was already found by the crates.io queries.
- `https://lib.rs/search?q=zfs`: one page, 16 KB, 76 crate links. New candidates: none.

### Source class 4: this repository

Provider: `rg` and `ls` against the checkout at `/home/user/Monochromatic`.

- `rg --ignore-case --files-with-matches 'zfs' --glob 'Cargo.toml' --glob 'Cargo.lock' package`:
  no matches, exit status 1. No package in this repository depends on, or is named after, any
  ZFS crate, so the wrapper has no incumbent to compare against and no existing choice to
  contradict.
- `ls doc/decision doc/audit` plus `rg` over those folders for `zfs`:
  the records that name ZFS are `doc/decision/cachyos-zfs-desktop.md`,
  `doc/audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md` (this vet),
  `doc/audit/tech-rolling-linux-desktop-with-encrypted-boot-m-vet-2026-08-29.md` and its
  `-911c4730` copy, `doc/audit/tech-meow-cache-key-hash-vet-2026-09-17/screening-crates.md`
  and its `-600031ed` copy, `doc/planning/cli-git-zfs-worktree.md`,
  `doc/planning/filesystem-for-the-repository-volume.md`,
  `doc/planning/monorepo-manager-from-scratch-design.md`,
  `doc/planning/zfs-and-btrfs-delete-stalls-in-a-vm.md`,
  `doc/planning/btrfs-and-zfs-each-at-its-best.md`, and two `depot` evidence JSON files.
  None of them names a Rust crate, so this phase had no prior candidate list to honour.

## Discovery result

Saturated with survivors: 9 routes passed all four screening gates.

The survivors, in the order the schedule found them.

1. `libzfs` 0.6.16 with `libzfs-sys` 0.5.11 and `libzfs-types` 0.1.2,
   from `whamcloud/rust-libzfs` (MIT, archived upstream).
2. `zfs-core` 0.5.0 with `zfs-core-sys` 0.5.0, `nvpair` 0.5.0 and `nvpair-sys` 0.4.0,
   from `codyps/rust-libzfs` (Apache-2.0 OR MIT).
3. The razor family: `razor-libzfs` 0.12.0, `razor-libzfs-sys` 0.12.0, `razor-libzfscore`
   0.13.1, `razor-libzfscore-sys` 0.13.1, `razor-safe-lzc` 0.12.0,
   from `razor-zfs/razor-rs` (MIT OR Apache-2.0).
4. `libzetta` 0.5.0 with `libzetta-zfs-core-sys` 0.5.2, from `Inner-Heaven/libzetta-rs`
   (BSD-2-Clause).
5. `zfskit` 0.3.0, from `okhsunrog/zfskit` (MIT OR Apache-2.0), which drives the `zfs` program.
6. `nessie-backend-zfs` 0.4.4, from `Gilamonster-Foundation/nessie-store`
   (MIT OR Apache-2.0), also a `zfs` program route and a weak fit.
7. `zfsdev` 0.4.8, from `Ukko-Ylijumala/zfsdev-rs` (MIT OR Apache-2.0, git only), pure Rust
   over `/dev/zfs`.
8. `clinta/zfs-rs` 0.0.1, from `clinta/zfs-rs` (MIT OR Apache-2.0, unpublished), direct
   ioctls plus `libnvpair`.
9. `moretea/rust-zfs` 0.1.0, from `moretea/rust-zfs` (LGPL-3.0, unpublished), a 2018
   `libzfs` binding.

One candidate is pending targeted evidence rather than resolved: `libzfs_core-sys` 0.1.0,
whose Linux build is unproven and whose dependency is FreeBSD's `libnv-sys`.

No source class is blocked. Two of the eight frozen GitHub repository queries reached their
`--limit=100` cap (`zfs --language=Rust`, 100 of 155; `zfs --topic=zfs`, 100 of 761).
For those two only, and recorded as supplementary runs, the same queries were repeated at
`--limit=1000` so the whole result set could be screened. The crates.io, web, lib.rs and
repository classes reached their ends without a cap.

## Expansion round

Run once, after the frozen schedule, as the report directs. Terms taken from what the schedule
turned up: `zfsdev` (a crate and library name), `libnvpair`, `libnv`, `illumos-nvpair`
(library names), `razor`, `libzetta` (crate family names), `open3` (a library the libzetta
README delegates work to), `veneer` (a project name), `uzfs` (a library name).
Every term got one more run of the same queries in each class.

### Expansion, source class 1 (crates.io search, one page each)

- `zfsdev`: total 0. New candidates: none. The crate sets `publish = false`.
- `libnvpair`: total 8, all returned. New candidates: none (all eight are nvpair codecs).
- `libnv`: total 9, all returned. New candidates: none.
- `illumos-nvpair`: total 2, both returned. New candidates: none.
- `razor`: total 83. New candidates: none; the razor ZFS crates were already found, and the
  rest belong to unrelated `razor-*` crate families.
- `libzetta`: total 3, all returned. New candidates: none.
- `open3`: total 1, `libzetta`. New candidates: none.
- `veneer`: total 13. New candidates: none; none of the 13 relates to ZFS.
- `uzfs`: total 0. New candidates: none.

### Expansion, source class 2 (GitHub repositories, `--language=Rust --limit=100`)

- `zfsdev`: 1 repository, `Ukko-Ylijumala/zfsdev-rs`, already found.
- `libnvpair`: 1, `oxidecomputer/illumos-nvpair`, already found.
- `libnv`: 6, all already found or unrelated (`libnvme`, `libnvsmi`, `libnvram`).
- `illumos-nvpair`: 1, already found.
- `razor`: 46, of which only `razor-zfs/razor-rs` is ZFS; the rest are unrelated `razor`
  projects.
- `libzetta`: 2, `Inner-Heaven/libzetta-rs` and a mirror.
- `open3`: 7, none ZFS.
- `veneer`: 13, of which only `robn/veneer` is ZFS.
- `uzfs`: 1, `iomesh/libuzfs-rs`, already found.
- New candidates from all nine: none.

### Expansion, source class 2 (GitHub code search)

Only the three symbol-shaped terms were run as code terms, because the other six are crate or
project names whose appearance in Rust source would say nothing about ZFS; that judgement is
recorded here rather than acted on silently.

- `zfsdev`: 20 hits in 8 repositories, none new (`Ukko-Ylijumala/zfsdev-rs`,
  `AlexanderThaller/calipe-rs`, `akivajp/rigtop` and others that only name it).
- `libnvpair`: 43 hits in 15 repositories, none new (`oxidecomputer/*`, `razor-zfs/razor-rs`,
  `codyps/rust-libzfs`, `clinta/zfs-rs`, `InkerBot/zfs-unlock`, and illumos projects).
- `libnv`: 100 hits, the requested limit, dominated by `libnvidia` and CUDA crates;
  the ZFS-related hits are `Inner-Heaven/libnv-rs`, `Inner-Heaven/libzetta-rs`,
  `clinta/zfs-rs` and `dlrobertson/capsicum-rs`, none new as a candidate.

### Expansion, source class 3 (the wider web, `rust <term>`)

Nine searches, one per term. New candidates: none.
`rust libnvpair`, `rust illumos-nvpair` and `rust libnv` returned only crates already
recorded, plus `libnv-sys`, the `-sys` sibling of `libnv`, which is noted in the ledger.
`rust razor`, `rust open3` and `rust uzfs` returned no ZFS material at all.
`rust veneer` returned furniture veneer pages.
`rust zfsdev` returned forum threads about ZFS disk names.
`rust libzetta` returned the two rust-libzfs repositories and a dependency-status page.

### Expansion, source class 4 (this repository)

`rg --ignore-case --files-with-matches '<term>' --glob 'Cargo.toml' --glob 'Cargo.lock' package`
for all nine terms: no matches in any of them.
Over `doc/`, the only hits are `libnvpair` and `razor` in this vet's own record,
`razor` and `libzetta` in `doc/planning/cli-git-zfs-worktree.md`, and `razor` in a `depot`
evidence file that names an unrelated package.

### Expansion verdict

The expansion round added no candidate to the ledger and no survivor.
With the schedule now frozen, later terms are only recorded.

## What I could not run as written, and what I would have asked

Deviation, recorded rather than performed silently.

- The two capped GitHub repository queries. `gh search repos` reports no total and honours
  `--limit`; at the frozen `--limit=100` two queries returned exactly 100 results, so their
  result sets were cut. I ran the same two queries again at `--limit=1000` and screened the
  whole 155 and 761, and recorded both runs in "Query records". The class is not blocked,
  because the supplementary runs reached the ends of the sets, but the frozen query as
  written was not sufficient on its own.
- Reverse dependencies for repository-only candidates. The schedule asks for the reverse
  dependencies of each candidate that binds C, naming crates. `zfsdev`, `clinta/zfs-rs` and
  the other unpublished repositories have no crates.io entry, so there is nothing to ask.

Judgements made where the gates run out, each stated in the ledger so it can be overturned.

- Crates with no license stated at all (five repositories and `sgrankin/zfs-sys`) were failed
  rather than left pending: the hard constraint is that the license is compatible, and with
  no grant there is nothing compatible to point at. The gate's wording only covers a stated
  license that is plainly incompatible. I would have asked whether the report wants a fifth
  gate reading "no license grant" separately from "incompatible license".
- The AGPL crates were failed on the license gate, and that is the one substantive license
  reading this phase makes: AGPL-3.0's network clause reaches the whole combined program.
  If the license phase reads AGPL-3.0 section 13 the other way, both entries move to pending.
- Crates that only codec `nvpair` or `libnv` were failed on category, not on capability:
  they bind an OpenZFS library but expose no dataset operation. I recorded them anyway,
  because every surviving FFI route depends on one of them, and I would have asked whether the
  ledger should keep them as candidates or as dependencies of candidates.
- Crates that drive the `zfs` program (`zfskit`, `nessie-backend-zfs`) pass the four gates as
  written, because nothing in the gates mentions the program. Since the owner declined the
  program on 2026-11-18, I would have asked whether the gates should include "does not shell
  out to the `zfs` program", which would move both out of the survivor list, or whether that
  belongs to the fit criterion where they will be outscored anyway.
- `zfsdev` is a git dependency (`publish = false`). Nothing in the gates rejects that, but it
  sits outside the registry's version resolution and needs a pinned tag; I recorded it and
  would have asked whether a git dependency is acceptable to the wrapper at all.

Where I think the schedule itself is weak.

- Class 1's ten queries overlap heavily: eight of the thirteen pages added no screening
  survivor, and every survivor was found by two queries or more. Two queries (`zfs`,
  and `libzfs` with its reverse dependencies) plus the keyword listing would have produced the
  same survivor list. That is a cost observation, not a coverage defect.
- The reverse-dependency step is defined over "each candidate that binds C", which is circular
  when the candidate list is the output of the earlier steps. Running it after screening, on
  the survivors' dependency crates, would have found the same crates in 21 fewer requests.
- The stop rule ("two complete pages in a row add no screening survivor") would have stopped
  the `zfs` query at page 2, one page before `libzfs`, `libzfs-sys` and `libzfs-types` appear
  on page 3. I read every page up to the provider's total instead. Sorting by relevance is
  what puts the last three survivors on the last page of that query.
- Class 2's code searches are the only place where direct-ioctl crates surface
  (`ZFS_IOC_` found `zfsdev-rs`). A class 2 pass keyed on `libzfs_core_sys` or `zfs_cmd_t`
  would likely find the same set faster than five separate symbol queries.

## Appendix: every repository returned by a GitHub query and not carried into the ledger

The names below are the repositories each query returned that are not already described by a
ledger subsection. Each is a ZFS-adjacent program, documentation set, OS integration, or an
unrelated project matching the term. The full provider responses, with descriptions and
stars, are in `raw/probe-repos-zfs-language-rust-limit-1000.json` and its siblings.

### repos-zfs-language-rust (the frozen --limit=100 run and the --limit=1000 probe)

Returned 155; not carried into the ledger: 138.

- manoeldesouza/zc
- numtide/disko-zfs
- okhsunrog/archinstall_zfs
- kimono-koans/httm
- andaag/zfs-to-glacier
- asomers/ztop
- Mic92/zfs-dedup
- rollcat/zfs-autosnap
- whamcloud/integrated-manager-for-lustre
- seuros/blackship
- redox-os/redoxfs
- killercup/zfs-gui-thing
- ashuio/shavee
- arctic-penguin/zfs-undelete
- nohajc/anylinuxfs
- kamacite-linux/beadm
- fenio/anylinuxfs-gui
- SecurityRonin/zfs-forensic
- Dicklesworthstone/frankenfs
- scttnlsn/vume
- Clete2/zfs_space_visualizer
- OtherJohnGray/izb
- peteonrails/retrograde
- Oxyra/zfstop
- lutzky/zsnapfree
- exrook/zfs-rs
- problame/zquash
- lheckemann/prune-rs
- SeanOMik/ZFS-PrometheusExporter
- jkelleyrtp/zfs-diff-parser
- barodeur/zfs-cloud-backup
- okhsunrog/arctern
- 0x0177b11f/zfs-issue-15526-check-file
- NBnet/btm
- 0xCCF4/zrb
- chrisdsa/zfs2s3
- akivajp/rigtop
- andoriyu/gazpacho
- ahouts/autosnapd
- snltd/zedfs
- joshtgl/zfshealth
- Boswell-Digital-Solutions/zfss
- BlackLuny/zfstack
- galic1987/zfswatch
- jamesmcdonald/zfsbackup
- fedor4445/zfsctl
- goyman/prometheus-zfs-exporter
- d3zd3z/rack
- amcchord/zfs-send-explore
- theatrus/zpool-notify
- arennow/zfs_arc
- chindit/zfs-monitor
- myst3k/zfsbackup-rs
- tedsta/zfs-rs
- bytex64/zfs-exporter
- anandab/zfs-dfree
- hnefatl/zfs-replicator
- tukzedo/zfs-gpl
- imkebe/zfs-io
- XSnow/zfs-autosync
- jclulow/junk-zfswtf
- migmedia/zfs-snappers
- zetafunction/unlock-zfs
- lhjnano/zresmon
- apestel/rusty-zfs-exporter
- okhsunrog/zbm-rs
- kkernick/zirconium
- ifazk/chobi
- sinitcin/zfs_snap_control
- phaus/zfs-snap-stats
- Lightgaming/nix-install-zfs
- aluminous/zfs-evacuation-controller
- PierreZ/zfs-dmu-rs
- aaronjanse/zbak
- ReactorScram/atomicwrites-zfs-repro
- snltd/zfs-tools-rs
- gmatiukhin/ybikey-zfs-pass
- roosmaa/zfs-http-query
- nourbenmoulehem/zfs-nfs-exporter
- sergiovibes/zfs-dashboard-rust
- lschuermann/zfs-snap-prune
- AaryanSinghChauhan09/SigmaOS-zfs
- TheQuantumPhysicist/sam-zfs-unlocker
- poesterlin/zfs-ha-monitor
- MaxMac99/ZFS-Prometheus-Exporter
- ChocolateLoverRaj/zfs-incremental-backup
- TheQuantumPhysicist/sam-remote-zfs-unlocker
- AaryanSinghChauhan09/SigmaOS-zfs-progs
- dheshanm/rs-zfs-ds-tracker
- eminence/snapshot_trim
- click0/zvolrescue
- Maxwell-lt/zpool-exporter
- wfraser/zsnapmgr_rust
- csinko/setlist-os
- tonymmm1/zsnap
- goopey7/brig
- doprz/ballast
- Dishwasher-DEV/freebsd-timeline
- cross/zfileinfo
- mingcheng/zwatch
- chaoyi/zboot
- ipetkov/shock
- hazayan/zhamel
- SilverBut/snapshot-to-s3
- mgrom/pg_data_branching
- khodorovskyi/vboxdump
- veeableful/fridge
- kamacite-linux/cosmic-beadm-applet
- robertek/zxp
- nietonchique/aoostar-lcd
- Error1000/szfs
- lutzky/reboot_with_passphrase
- JeffResc/FSSnapshotBackup
- heikofkoehler/cownfs
- dinglebear-ai/synapse
- handtrixx/parasanoid
- mavnezz/zendiloOS
- olexiyb/talos9s
- hey-jj/lzjb
- sshine/boxdrop
- OtherJohnGray/reflectron
- OtherJohnGray/file-backup
- asdacap/greendotrdma
- DarkWaveLabs/rtop
- dedupcommando/DedupCommando
- lamco-admin/lamzfs
- ecoPrimals/nestGate
- ChronicleFS/ChronicleFS
- GladTechInfo/iztop
- corbet-nix/nixnas
- lilrags16/TapeScale
- alvaromendoooo/file-system
- FieldmouseWorks/burrow
- zyvorai/zyvor-atlas
- JBelliriGenty/Cairn-NAS
- crustates/crustates
- StatusCode404/CPU-Grid
- dodSONSoftware/truenas-prometheus-exporter

### repos-zfs-topic-zfs (the --limit=1000 probe; its 43 Rust members are all in the set above)

Returned 761; not carried into the ledger: 757.

- zbm-dev/zfsbootmenu
- openebs/zfs-localpv
- zrepl/zrepl
- psy0rz/zfs_autobackup
- pdf/zfs_exporter
- ivanhao/pvetools
- optimans/cockpit-zfs-manager
- someone1/zfsbackup-go
- 64kramsystem/zfs-installer
- markusressel/zfs-inplace-rebalancing
- pr1ntf/iohyve
- bdrewery/zfstools
- containerd/zfs
- eoli3n/archiso-zfs
- jonmatifa/zfsmanager
- bashclub/proxmox-zfs-postinstall
- nchevsky/systemrescue-zfs
- st771207/qnap-zfs-rescue
- johnramsden/zectl
- cbreak-black/ZetaWatch
- TrilliumIT/docker-zfs-plugin
- stevleibelt/arch-linux-live-cd-iso-with-zfs
- geerlingguy/arm-nas
- aaronhurt/zfs-replicate
- macgaver/zfsnas-chezmoi
- spl/zfs-on-mac
- dasJ/sd-zfs
- bashclub/zamba-lxc-toolbox
- j-keck/zfs-snap-diff
- alexhaydock/zfs-on-wsl
- vpsfreecz/vpsadminos
- okhsunrog/archinstall_zfs
- gentics/kubernetes-zfs-provisioner
- aisbergg/ansible-role-zfs
- eoli3n/arch-config
- blind-oracle/zabbix-zfs
- ncabatoff/zfs-exporter
- ddebeau/zfs_uploader
- FreeBSD-Ask/FreeBSD-Ask
- truenas/middleware
- kimono-koans/httm
- puppetlabs/puppetlabs-zfs_core
- PVE-Tools/PVE-Tools-9
- apprell/proxmox-autosnap
- norsemangrey/zfs-health-check
- Jehops/zap
- ipaqmaster/zfsUnlocker
- alunduil/zfs-replicate
- bashclub/check-zfs-replication
- ankek/awesome-zfs
- sensu-plugins/sensu-plugins-zfs
- asteven/local-zfs-provisioner
- bodgit/puppet-zfs
- johnramsden/zedenv
- Napsty/check_zpools
- laurent85v/archuseriso
- psy0rz/alpinebox
- MathiasPius/terraform-provider-zfs
- nephri/FreeNas-DiskList
- dzacca/zfs_on_root
- adlibre/adlibre-backup
- iocage/iocage
- BastilleBSD/bastille
- pirate/zfsify
- lorenz/go-zfs
- whamcloud/integrated-manager-for-lustre
- ghfields/rpooler
- dominicusin/zfs-win
- matusnovak/prometheus-zfs
- hiliev/py-zfs-rescue
- iBug/zfs-recompress.py
- volkerp/zfstui
- chadmiller/zpool-iostat-viz
- fabrik-red/images
- eoli3n/void-config
- integralstor/integralstor_centos
- symmetryinvestments/zfs-on-root-installer
- frappe/atlas
- PedroHLC/system-setup
- Corsinvest/eve4pve-zstorage
- gytis-ivaskevicius/nixfiles
- natankeddem/bale
- mminkus/zfs-explorer
- nadavgolden/zfs-auto-mirror
- kvaps/kube-linstor
- jceel/py-libzfs
- bashclub/zfs-housekeeping
- varasys/corezfs
- dmuth/zfs-playground
- proxmox/pve-zsync
- omniti-labs/zetaback
- waterlemon0108/nexboot
- pirate/zfsbox
- markusressel/zfs-file-history
- bartmichu/zfs-zabbix-userparams
- IndrekHaav/zfs-vault
- sleleko/devops-kb
- dim-geo/zfs_frag
- redox-os/redoxfs
- stewartadam/zfs-keyvault
- hkbakke/zfssnap
- ossobv/planb
- digineo/zackup
- Spearfoot/FreeNAS-scripts
- cego/zfs-cleaner
- zfs9p/zfs
- osboot/make-initrd
- szorfein/Gentoo-ZFS
- bakins/zfs-flex-volume
- DtxdF/AppJail
- snsinfu/hetzner-zfs-image
- svalouch/python-simplezfs
- atheriel/snapshot-explorer
- fnichol/cachyos-zfs-installer
- democratic-csi/democratic-csi
- hugojosefson/proxmox-root-on-encrypted-zfs
- illiliti/tinyramfs
- johnramsden/pyzfscmds
- mmitch/debian-live-mitch-zfs
- anderbubble/autoscrub
- dsnet/zsync
- dapuru/zfsbackup
- TheUbuntuGuy/zfs-backup-manager
- HomeSecExplorer/Proxmox-Hardening-Guide
- evan-king/grub2-zfs-be
- stuvusIT/zfs-storage
- LINBIT/linstor-server
- ashuio/shavee
- wuxxin/machine-bootstrap
- salt-extensions/saltext-zfs
- prod-feng/Install-Lustre2.12.8-on-CentOS-7.9-2009
- ZentriaMC/docker-zfs-plugin
- arctic-penguin/zfs-undelete
- Patryk27/lxd-snapper
- basnijholt/zfs-tenant
- xen-tools/xen-tools
- fenio/modern-fs-benchmark
- acidvegas/zpulse
- borgmatic-collective/borgmatic
- jersobh/zfs-resty
- j-keck/zsd
- mojtabamoaddab/zfshotter
- algotitanhq/uberblok
- asimkadav/zfs-analysis
- x0rzavi/archiso-zfs
- niamtokik/zfs
- jrcichra/rpi-zfs-root
- RainbowHackerHorse/vzvol
- johnramsden/zedenv-grub
- rejsmont/zfsgui
- PjSalty/terraform-provider-truenas
- jdrch/Hardware
- Zaryob/zfs_kitabi
- reconquest/zeus
- nohajc/anylinuxfs
- simonbuehler/zfs-salvage
- nbari/freebsd
- cozystack/blockstor
- briandowns/sky-island
- bardulos/optimal_zfs_recordsize
- geerlingguy/ansible-role-sanoid
- kamacite-linux/beadm
- fenio/tns-csi
- michaelsproul/autozfs
- fnichol/arch-linux-installer
- TemaSM/ZFSh
- pando85/homelab
- krystal/go-zfs
- reefland/motd
- DerickEddington/nixos-config
- asteven/zfs-provisioner
- openmilanese/zfs
- cieplak/jz
- fenio/anylinuxfs-gui
- yaroslav-gwit/HosterCore
- BillieBadin/Pi-BSD
- coalsont/zfs-scripts
- gnacho/easyzfs
- timlinux/tuinix
- mdlayher/zedhook
- ccebelenski/anas
- midzelis/zquickinit
- dandudikof/zfs-auto-mod
- huangwb8/openmediavault_zfs
- backup-blob/zfs-backup-blob
- Josef-Friedrich/check_zfs_snapshot
- freecore-project/freecore
- lukas2511/debian-installer
- basnijholt/zfs-unlock
- rlaager/zfs-replicate
- bjschafer/zfs-rclone
- mminkus/zfs-ondiskformat
- random-archer/mkinitcpio-systemd-tool
- stblassitude/zfssync
- prod-feng/Install-Lustre-2.15-with-ZFS-2.1.2-on-Rocky-Linux-8.6-
- SmilyOrg/timeship
- snapsinazfs/snapsinazfs
- tgerczei/zfs-backup
- makhomed/autosnap
- heavybullets8/zfs-scrubber
- claui/znapzupport
- janprzy/zfs-snapsize
- mminkus/zfs-codebase
- benjaminjonard/CM3588-NixOS-ZFS
- swills/zfstools-go
- click0/ansible-freebsd-install-on-zfs-lite
- prod-feng/Luste-KMOD-2.12.9-with-ZFS-0.7.13-on-Centos-7.9
- reefland/ansible-motd-zfs-smartd
- markusressel/cli2telegram
- buildthehomelab/storage-planner
- Negashev/rancher-saas
- binRick/ZFSManager
- sotormd/nixos
- infraguys/gci_ubuntu_zfs
- Siddharthsinghkumar/truenas-zfs-recovery-lab
- sineer/zrepover
- Dicklesworthstone/frankenfs
- fusion809/freebsd-zfs-manual-install
- nmarasoiu/zfs-scripts
- edacval/zfs-meta
- luoxu34/zfs-completion
- yaroslav-gwit/HosterRed-HyperVisor
- Photonios/pgcow
- bashclub/zsync
- qdm12/welcome
- scttnlsn/vume
- macskas/nagios-plugins
- FlorianHeigl/Tegile_Storage_Info
- norsemangrey/zfs-snapshot-diff
- glenjbarber/apiary
- grimne/fbsd-nas
- thnee/zsm
- chong601/pyzfs-rest-api
- genneko/freebsd-install-on-zfs
- AlteredAdmin/ZFS-Mirrored-Encrypted-Setup
- ljrnbg/zfs-previous-versions-macos
- mxk/zfscrypt
- callmezatiel/zarch
- fyrkat/zfsroot
- Thalhammer/zfspp
- ansemjo/autosnap
- frebib/zfs-exporter
- acrion/ditana-installer
- 4nonX/DPlaneOS
- khimaros/raid-explorations
- geraldwuhoo/homelab-iac
- exrook/zfs-rs
- gikeymarcia/ansible-role-sanoid
- joaquinv98/zfs-iscsi-multipath
- problame/zquash
- mmitch/list-zfs-snapshots
- rahimkhoja/chia-plotter-deployment
- LINBIT/lbtest
- borodark/zed
- SynergOps/zfs-mini-lab
- lheckemann/prune-rs
- PsychoLlama/home-lab
- hungrywolf27/zfs-list-gtk
- mmoole/ansible-role-zfs
- modem7/docker-borgmatic
- gnos-project/gnos-bootclone
- okhsunrog/arctern
- stuvusIT/ansible_zfs_snap_manager
- markomarkovic/grafana-zfs-alloy-dashboard
- mchangrh/ZoD
- vaamonde/truenas
- ZappedC64/ubuntu-2604-zfs-cis
- Obmondo/puppet-sanoid
- blokin/zfsSender
- Avvessalom/zfsGangster
- aamkye/ubuntu_on_WD_PRx100
- guardian-intelligence/verself
- juanmitaboada/zark
- timaebi/zfs2glacier
- deevus/pixels
- go-freebsd/zfs
- langerma/dumpstore
- symmetryinvestments/libzfs-core-d
- NBnet/btm
- binRick/zfsBackupManager
- rstellma/zfs
- jrcichra/zpool-status-prometheus-exporter
- ncrmro/astrocyte
- AwesomeDog/awesome-nas
- dim-geo/cpmv
- freecore-project/build
- damex-el-packages/zfs
- innolabsinfo/zfs-omarchy-iso-maker
- JoeKun/freebsd-configuration
- akivajp/rigtop
- fhofherr/zsm
- fnep/ZAS
- makhomed/ct
- skrp/DTRACE
- transitiv/ansible-role-zrepl
- tuaris/zfsguru
- dbohdan/zfs-recompress
- dbalan/zfsborgmeup
- freecore-project/py-libzfs
- kznrluk/zfs-scripts
- FreeBSD-Ask/openzfs-docs-cn
- pbek/zfsguard
- ChristopherClure/zfsmanual
- rabkazoo/zfsutilsnap
- wallart1/ZFSutilities
- sophonet/zfskeyprovider
- versus167/zfsnappy
- AlexNortonFenn/zfspulse
- ecnepsnai/zfssnap
- bradford-hamilton/homelab
- boinkor-net/zpool-exporter-textfile
- mikispag/zfsbackup
- freecore-project/freenas-pkgtools
- hyperparabolic/nix-config
- meabert/proxmox-tips-n-tricks
- SilverWolfOfficial/ArchLinuxOnZFS
- Finkregh/deboostrap-zfs-root
- awfulwoman/zfs-api
- jnyilas/ZFS_Get
- adfinis/backupctl
- loozhengyuan/ansible-zfs
- zzir/citus
- skrp/SICC
- daviswr/ZenPacks.daviswr.ZFS
- makhomed/autosync
- sown/backup
- DocBrown101/zfs-spectator
- nonifo/diskrisk
- D4rk-5ky/Syncerate
- jobts/zfsonarch_manual
- jhgorrell/zfs-mksnap
- rkojedzinszky/zfs-scripts
- migmedia/zfs-snappers
- spranab/truenas-mcp
- ziwon/ai-data-center-systems
- openmilanese/zfs_iscsi
- Danathar/Kinoite-ZFS
- weak-head/zfs-utils
- bartmichu/zfs-scenarios
- anandab/zfs-dfree
- mminkus/zfs-comphist
- Ratakor/zfs-restore
- kambiz-aghaiepour/ZFS-Guide
- nielsd/zfs_snapper
- MynaITLabs/zfs-addons
- karlh001/zfs-snapper
- szorfein/eclosion
- riiengineering/zfs-snapsend
- shakemid/zfs_replication
- katbyte/zfs-exporter
- faken/zfs_snapshot
- ortocomputer/zfsnap-quota
- CNyaotian-Lunar/zfs-tool
- AndiDog/zfs-lockup
- mikeplem/zfs_replace_disk
- SilverWolfOfficial/archiso-zfs
- acrion/zfs-autosnap
- mietzen/smb-zfs
- kazootkr/freebsd-vagrant
- chaos7x/zfs-root-installer
- julp/freebsd-scripts
- TheSynus/zfs-offline-backup
- oposs/zzclone
- perryflynn/zfs-autosnapshot-rotation
- vladvasiliu/plymouth-zfs-mkinitcpio
- okhsunrog/zbm-rs
- extide/zfs-automatic-snapshots
- Knallli/zfs-backup-script
- ozgurhepsag/Basic-z-OS-Utilities-and-Practices
- DerickEddington/my-zfs-backup
- ryanmaclean/zfs_home_nas
- Wu-Wu/proxmox-zfs-migrate
- patte/zfs-autobackup-docker
- moterpent/zz
- angristan/ansible-zfs-auto-snapshot
- eoli3n/zectl-pacman-hook
- lordfolken/ansible-role-zfs
- sergiovibes/zfs-dashboard-rust
- flohoff/check_opene_zfs
- abaeyens/zfs-nas-dashboard
- styliteag/opene-zfs-pull-backup
- clesssalvein/zfs-volume-replication
- styliteag/zfs-jbod-tools
- digitalloggers/docker-zfs-patches
- yaroslav-gwit/proxmox-zfs-snapshots
- jnyilas/ZFS-DataPump
- omasakun/wsl-zfs-module
- fluminis-scientiae-oraculum/zfs-snapshot-retention
- PieterScheffers/freebsd-zfs-backup
- Rau-N/zfs-key-loader
- tschuering/zfs-exporter-helm
- decibel-inc/zfs-snapshot-sync
- mikoim/ansible-archiso-zfs
- sergiovibes/zfs-dashboard-python
- nmasse-itix/zfs-aarch64
- mbuse79/zfs-manager-pve
- amyeo/nagios_zfs_statecheck
- nnhck/zfs_disk_map
- felipe-xavier-cyber/incident-response-zfs
- basnijholt/zfs-encrypt-migrate
- DMarkStorage/ZFS_Appliance_Hardware
- honest-hosting/nomad-csi-driver
- dim-geo/hanoizfs
- Unnamed0834/zfs-zil-parse-recovery
- snenyl/zfs-chunk-transfer-threader
- zoni/zfs-snapshots-prometheus-collector
- NikitaPuglachenko/proxmox-zfs-arc-tuner
- dfberlin/zbmcloudimage
- githubfoam/beegfs-sandbox
- incognitte/tpm-zfs-initramfs-tool
- alexbarton/backup-script
- koro666/zipref
- murrayju/docker-node-zfs-base
- system-slactivist/Unraid-ZFS-Snapshot-Manager
- Cormoran96/TPI-KVM-ZFS-DRP
- Evaneos/rancher-os-zfs-setup
- stationgroup/backup-zfs-rsync.net
- bbbbbrie/zfs-management-with-ansible
- jnyilas/ZFS-Automatic-Snapshot-Manager
- skrp/old_MKRX
- jgtux/grpc-zfs-monitor-demo
- clawoflight/telescope-zfs-snapshots.nvim
- ancwrd1/zsnaptool
- fmarl/snapup.sh
- andrewpgit/zfs-remove-old-snap
- feffi/ansible-arch-partition-zfs
- Frostman/k8s-zfs
- BoHauHuang/ComputerSystemAdministration
- BrewedIntel/homelab-resources
- devds96/UserRemoved-ZFS-Unload-Home
- freecore-project/webui
- freecore-project/iocage
- SoumyaPurani/TrueNASScale-TUI
- stesind/trash-empty
- nvass/spieglein
- binRick/fix-openvz-veid-zfs-private
- AkihiroSuda/containerd-example-custom-daemon
- gh4rib/linux-zfs-on-root-installer
- tuxcrate/zrepl
- incognitte/tpm2-zfs-initramfs-tool
- gozoinks/gozoinks.github.io
- ashlin4010/truenas-backup-scripts
- pdf/boomerangz
- Josef-Friedrich/zfs-delete-empty-snapshots.sh
- johnramsden/zsh-zsnapac
- tterpelle/docker-slack64-current-zfs
- handcode/check_mk_zfs_arc_freebsd
- tonymmm1/zsnap
- stuckj/znapzend-docker-proxmox
- natasha0824inkf/zelta-space-theme-pack
- kerberizer/archzfs-git
- reefland/ansible-zrepl_sink_tls_client
- makhomed/vm
- stevefulme1/ansible-truenas
- iwvelando/telegraf-exec-zpool-status
- lflare/rewrite-inplace
- MikeCaldera/Fdiskbatch
- karolyi/ansible-freebsd-jailhost-tools
- Dishwasher-DEV/freebsd-timeline
- githubfoam/lustre-sandbox
- andras-tim/zgui
- itsmrval/cmds
- mikanmi/zxybackupcloser
- mingcheng/zwatch
- wolffaxn/bsdbox
- pxlwh/homelab-monitoring
- NYCU-WinLab/strata-p100-deployment
- kenwi/OmaUse
- Sidias-Korrado/baltuzee
- builtbycodyrd/homelab
- sarahthiel/zhanoi
- ipetkov/shock
- 42LoCo42/zfullfs
- slb350/sabrent-ds-sc5b-linux-zfs-workaround
- Beanow/satactl-vs-hba
- DncrJim/disktest
- jgtux/abyssal
- asdrubalinea/source-of-truth
- 3v1n0/qnap-extra-fs
- kvaps/opennebula-addon-linstor_un
- mikanmi/elephant-backup
- MilesBHuff/Zebiantu
- Tenyun/pve-status
- edvler/zrep-offsite-wrapper
- corbet-nix/nixstorage-corbet-ch
- nocloudlab/deeppool
- SilverBut/snapshot-to-s3
- freecore-project/iocage-freecore-plugins
- Csontikka/ds220plus-proxmox
- fhdytt/lxd-project
- isac322/pillar-csi
- ykgmfq/SystemPub
- onezeroone-dev/ozo-znap-and-zhip
- cfcs/nvlist
- OpenVTL/OpenVTL
- rawstor/librawstor
- SoulDeity/infra
- wjdp/tetanus
- MVladislav/ansible-updater
- stesind/zpool-scrub
- jlbyh2o/ezdr
- oshogbo/crrd
- sophonet/vzesync
- olgeni/facl
- mietzen/cockpit-plugins
- sebastian13/zabbix-template-pyznap
- garenwen/freebsd-manager
- blake-hamm/bhamm-lab
- MarkProminic/ha_disk_monitor
- ipaqmaster/fioBench
- Error1000/szfs
- zhengyp36/pyzdb
- akrush24/loganalyzer
- r34lcl/Disk-Monitor
- ahmetem/homelab-proxmox-mcp
- dinglebear-ai/synapse
- Tungsten0/homelab
- RooSoft/zman
- lspagnol/hvm
- QiaoLei-88/zpool-disk-waiter
- somnium78/proxmox-stuff
- wnstify/self-hosting-101
- lspagnol/jbod
- mavnezz/zendiloOS
- emirvatandaslar/virtualization-storage-cheatsheets
- camj2/server
- qoxi-cloud/proxmox-installer
- laeti-tia/vbox-helpers
- johnramsden/iocage-plugin-zrepl
- guru-bharadwaj20/Private-Cloud
- bzdOS/jailrun
- staticrevolution-com/sr-truenas-mcp
- CjCrump/homelab
- stratastor/rodent
- MilesBHuff/Misc-code
- AndrewTrieu/selfhosted
- PauTentX/homelab
- braincraft-gmbh/ansible-lxc
- sageframe-no-kaji/palana
- raohwork/moobk
- RuiGRMargarido/homelab
- proksid/zlue
- DROOdotFOO/nix-mox
- hey-jj/lzjb
- DiecoTheBxxst/CentralConsole
- Cfirth725/systems-engineering-journal
- mgrzybek/ansible-ha-storage
- carolina-queiros/linux-infra-labs
- servercrate/restic-toolkit
- cdesiles/ansible-playbooks
- Nielk1/ZorkFileSystemTools
- krlex/freebsd-cbsd
- hattybin/restoid
- jaredglaser/homelab-manager
- hugojosefson/truenas-unlock-dataset-once-after-each-boot
- yvoinov/ramdisk_smf
- zamkara/kentut
- mgrzybek/ansible-bsd-iscsi-target
- bitranox/proxmox_srvbackup
- floppy007/floppyops-lite
- szorfein/getch
- bcmumma/homelab-infrastructure
- luczakconsulting/infra-scripts
- dryvist/ansible-proxmox
- decentralizeallengineering/nascare
- lahavrud/homelab
- corbet-nix/nixkernel
- FCLC/Multi-Plexer
- mikeotoole/pullbackup
- tekmyster/Hoardarr
- johanVcode/homelab-server
- ohmantics/zbmsetup
- japatton/ha-truenas-ng
- stuckj/snapdump-docker
- DarkWaveLabs/rtop
- ahmd-soliman/naswarden
- brcak-zmaj/scripts_and_shells
- AlexMaina05/hp-fortress-homelab
- aamkye/nas-system-exportes
- VivianSobers/Private-Cloud
- mnheia/storage_early_warning
- jameskoziol/ha-qnap
- Taoidle/dsh-truenas
- seanpm2001/DeciCube
- FraGoTe/zend3-dashboard
- reefland/zrepl_sink
- valerio-afk/nms
- dedupcommando/DedupCommando
- rdm375/SyncVault
- lamco-admin/lamzfs
- knachiketa04/aihomelab
- tlosrod672/homelab-proxmox
- nijave/device_info_exporter
- Abdelmadjiddazi/pve-preflight-check
- crazyuploader/hostglance
- 0x64616e6e/kaiseki
- SoyMoikas/homelab
- derdennis1012/proxmox-ops-mcp
- helgehelge123/napback
- danieltucker/BayWatch
- Hexadecimalz/Things-I-Learned
- Termix-SSH/Plugin-Storage-Health
- gerardrecinto/ingest-node-lab
- bristolpie/ProxmoxFanControl
- georgejpark/homelab
- Groupe-maison-fr/Sylar
- sh0shin/ansible-role-base_freebsd
- DiecoTheBxxst/ObjMail
- ottermq/jaildeck
- G0rri/nas-homelab
- sliptripfalldown/asustor-nas-media-server
- jacedeno/restic-r2-backup
- islemazz/truenas-storage-lab
- Nickhades/fmon
- dengrenhao/freebsd-sysadmin-toolkit
- CNyaotian-Lunar/nas-container-watchdog
- romupegar/enterprise-siem-lab-blueprint
- mswdev/better-unraid-mcp
- ufuk64-del/freebsd-vnet-jails
- benibr/xpire
- nihil2040/pysanoid
- meineerde/ohsnap
- kborovik/acumatica-devops
- hastmu/avahi-backup
- vicenteliu/SE-Agent-Environment-Sandbox
- PyHackSecGP/p7-autopatch
- Tur0l/proxmox-ha-ceph-nextcloud
- akai-hana/akai-linux-kernel
- and2rey/homelab
- AcidDemon/ansible-base-freebsd
- nicolasnkGH/ai-stack
- zed-0xff/binsearch
- theOtherLuke/helper-scripts
- Kheopsian/minimalist-dashboard
- StealthCat/NASitron
- yacchi/pve-qnap-plugin
- bearbinary/omni-infra-provider-truenas
- ishandutta2007/Awesome-Managed-Openzfs-File-Storage
- iceman010-hub/homelab
- JohnLHolloway/nas-lights
- AIops-tools/TrueNAS-AIops
- fcelsa/nascheck
- ataylor-us/nas-infra
- GladTechInfo/iztop
- marcokuoni/script_scheduler
- ecoPrimals/nestGate
- kratocz/tech-comparisons
- corbet-nix/nixnas
- sageframe-no-kaji/hozo-sentinel
- ishandutta2007/Awesome-File-Systems-Volume-Managers
- fusion809/freebsd-scripts
- paepckehh/zdash
- fraenki/puppet-zettarepl
- corbet-nix/nixbackup-corbet-ch
- pithyx/pithyx
- sasharif87/homelab
- jorritvanderheide/snapback
- lilrags16/TapeScale
- Lunetikk/check_snmp_zpool
- ottermq/otter_cloud
- gfriloux/stc
- thefiven/homelab
- DjangoAILab/pve-storage-guard
- teo-devops/platform-proxmox
- open-img-cloud/freebsd
- Erdnusschokolade/jf-precache
- diriartev/vintum-infra
- bmoyo413/homelab
- mel0k1/SeaBSD
- deadlinkuser/DataSaverVideoFinder
- Dalil-Tech/raid-capacity-cli
- lucassebastiang/infraestructura-autohospedada
- wantguns/dotfiles
- nelsudev/vps-multi-tenant-sovereign
- labwatch-dev/labwatch
- costis-t/linuxulator_filesystem_compatibility_shim
- JabawokJayUK/jtop
- oxagast/butterscotch
- katbyte/truenas-exporter
- FieldmouseWorks/burrow
- ak95-io/homelab
- dev-dull/terraform-provider-trueform
- maverick0628/nasbridge
- pxlwh/homelab-backup
- faust64/puppet
- hugojosefson/dropbear-auto-unlock
- punith1006/LaaS-Strata
- alancuzen/modern-smart-home
- Trytonottry/proxmox-oneclick-deploy
- mkorthof/check_truenas
- artem-korolev/workstation_forge
- MechanicalCoderX/ktn-enclosure-manager
- avsharbeyi/depsis
- zyvorai/zyvor-atlas
- MMDBadCoder/MMD-DEV
- juanmitaboada/zark-gnome-extension
- ji-podhead/Network-Guides
- mexhi-byte/eznas
- FireflyTang/hddmon
- darthsteven444-code/CIS_Lab
- CyborgFingers/vessapp-a3120-monitor
- TheFox/docs
- alfonsokuen/pegaprox-plugin-truenas
- 16bitsrl/boxe
- xO-ox-ai/dynamix.file.recycle
- morph-eos/tidepool
- lamine67/server-panel
- zettalane-systems/open-pnfs-cloud
- idkmanager/truenas-proxmox-plugin
- walter-luo/zos-agent
- gfriloux/nixos
- ipaqmaster/fioReporter
- AcidDemon/bastille-jails
- tukzedo/tukfs-hash
- joeboy0722/just-NAS
- crustates/crustates
- abdlmalekluttee/freenas-drive-locator
- benoitpoveda/homelab-proxmox
- 0xCyberLiTech/Proxmox
- zombocoder/go-freebsd-jails-exporter
- ldilley/techdocs
- OfBirds/RestoreGuard
- marion909/minecraft-saas
- JBelliriGenty/Cairn-NAS
- NgarumaVTC/KaziLab
- stuvusIT/vm-facts
- OzarkMountainPirate/utilities
- piwi3910/truenas-csi
- katbyte/truenas-mcp
- githubfoam/multipass-travisci
- kekzl/proxmox-tools
- bitranox/check_zpools
- BladimirF7/linux-operations-toolkit
- VoidedKN0X/TrueCtrl
- JoeKun/homelab-documentation
- 1immortal/ytn
- CopperArch/CopperBSD
- leviathonbeast/homestack
- jason-yi4/proxmox-homelab
- dodSONSoftware/truenas-prometheus-exporter
- stuckj/znapzend-full-backup
- SpyderMYK/homelab
- JBelliriGenty/Cairn-Map
- H4ad/omarchy-truenas
- NgarumaVTC/nixos
- dropnook/UnraidSecretaryOffice
- nks-hub/st1600fm0013-528b-sectors
- LiYiwei-dev/appjail
- SpyderMYK/homelab-field-guide
- notoriousduc/R-Undelete-2026
- johan-socarras/VMware-Homelab
- Anton-Babaskin/proxmox-hetzner-guide
- aquilaelabs/aerieos
- zettalane-systems/zettabranch
- sarabelinformatika/proxmox-backup-server-enterprise-guide
- Ashutosh0x/creating-your-own-servers-from-scratch
- Kalle2011/bast

### repos-openzfs-language-rust

Returned 12; not carried into the ledger: 7.

- redox-os/redoxfs
- okhsunrog/archinstall_zfs
- SecurityRonin/zfs-forensic
- problame/zquash
- okhsunrog/arctern
- GladTechInfo/iztop
- dedupcommando/DedupCommando

### repos-openzfs-topic-zfs

Returned 60; not carried into the ledger: 59.

- johnramsden/zectl
- st771207/qnap-zfs-rescue
- ankek/awesome-zfs
- mminkus/zfs-explorer
- redox-os/redoxfs
- blind-oracle/zabbix-zfs
- dominicusin/zfs-win
- zbm-dev/zfsbootmenu
- Zaryob/zfs_kitabi
- jrcichra/rpi-zfs-root
- freecore-project/freecore
- stevleibelt/arch-linux-live-cd-iso-with-zfs
- svalouch/python-simplezfs
- mminkus/zfs-ondiskformat
- basnijholt/zfs-unlock
- okhsunrog/archinstall_zfs
- mminkus/zfs-codebase
- gnacho/easyzfs
- FlorianHeigl/Tegile_Storage_Info
- algotitanhq/uberblok
- snapsinazfs/snapsinazfs
- FreeBSD-Ask/openzfs-docs-cn
- basnijholt/zfs-tenant
- ishandutta2007/Awesome-Managed-Openzfs-File-Storage
- problame/zquash
- basnijholt/zfs-encrypt-migrate
- swills/zfstools-go
- mminkus/zfs-comphist
- slb350/sabrent-ds-sc5b-linux-zfs-workaround
- tschuering/zfs-exporter-helm
- okhsunrog/arctern
- simonbuehler/zfs-salvage
- oshogbo/crrd
- mmoole/ansible-role-zfs
- ryanmaclean/zfs_home_nas
- AlexNortonFenn/zfspulse
- Unnamed0834/zfs-zil-parse-recovery
- QiaoLei-88/zpool-disk-waiter
- bashclub/zsync
- StealthCat/NASitron
- openmilanese/zfs
- valerio-afk/nms
- nihil2040/pysanoid
- seanpm2001/DeciCube
- joeboy0722/just-NAS
- NikitaPuglachenko/proxmox-zfs-arc-tuner
- paepckehh/zdash
- kambiz-aghaiepour/ZFS-Guide
- mbuse79/zfs-manager-pve
- bartmichu/zfs-scenarios
- juanmitaboada/zark-gnome-extension
- GladTechInfo/iztop
- nmarasoiu/zfs-scripts
- juanmitaboada/zark
- bitranox/check_zpools
- githubfoam/multipass-travisci
- openmilanese/zfs_iscsi
- katbyte/zfs-exporter
- dedupcommando/DedupCommando

### repos-libzfs-language-rust

Returned 8; not carried into the ledger: 0.


### repos-libzfs-topic-zfs

Returned 7; not carried into the ledger: 4.

- jceel/py-libzfs
- symmetryinvestments/libzfs-core-d
- Thalhammer/zfspp
- freecore-project/py-libzfs

### repos-libzfs_core-language-rust

Returned 1; not carried into the ledger: 0.


### repos-libzfs_core-topic-zfs

Returned 1; not carried into the ledger: 1.

- symmetryinvestments/libzfs-core-d

### expansion: repos razor

Returned 46; not carried into the ledger: 45.

- salmans/rusty-razor
- am0d/rusty-razor
- maxdeviant/razorbill
- swar09/razorpay-rs
- davidfowl/razor-rust
- NaturalIO/razor-rpc
- madushankaf/razorback-sabre-contract
- RazorClient/RazorVM
- johnyrahul92/razorshot
- shridhargsabat/razorpdf
- viveret/rshtml
- NeverGET/zed-razor
- hellosam123/Razor
- AyoKeito/razor
- scss28/razor
- jiliangzhu/Razor
- 2075/razor
- f0dn/razor
- Plaaasma/razor
- aribambang/razor
- axmad386/razor
- treygoff24/razor
- bhargavsaid226/razor
- SashaNullptr/RazorBlades
- dafioram/RazorQ
- dubvulture/rustedrazors
- Priyanshu-Developer/razorpay
- rhyven-ai/razorback
- aarcange/ripgrep-edit
- sajuthankappan/razorpay-rs
- ScribeK2/zombos-razor
- fisherjacobc/razorback-vsc
- infinityabundance/GlyphRazor
- OutOfBoundCats/Razorpay_rust
- cwhittl/zed-razor
- bozidarsk/zed-razor
- 21pages/razor_wrapper
- shafiqaimanx/RustyRazor
- Salman-Sali/razorpay-client
- PeltyCMS/addon-gateway-razorpay
- IModer/razor-lang-rust
- kernelorg-mirror/linux_kernel_git_andrea_ripgrep-edit
- dsbraz/zed-file-ext-switcher
- 0xA672/qwqlang
- quantexpy69/quantumdaw

### expansion: repos veneer

Returned 13; not carried into the ledger: 12.

- saethlin/veneer
- diorgulescu/veneer
- codixor/veneer
- AngeloRusu24/Veneer
- ericcurtin/veneer
- LLCForge/Veneer
- hajunwon/veneer
- LukasParke/veneer
- damienstanton/veneer
- ascarter/veneer-theme
- ModernRetroDev/retro-veneer
- bmander/routelab

### expansion: repos libnv

Returned 6; not carried into the ledger: 5.

- oxidecomputer/libnvme
- peterhj/libnvsmi
- ionosnetworks/libnvds-sys
- tomato64/libnvram-rs
- Cyberpsych0s1s/libnvme-rs

### expansion: repos open3

Returned 7; not carried into the ledger: 7.

- Jordan-Hall/open365
- bobmcwhirter/open3pl
- pupgrammer/open303rs
- carlesllavata/Open3D
- ReanGD/rust-open3d
- Open330/open330-repo-pulse
- choompol-boonmee/open3d-01

### expansion: repos libzetta

Returned 2; not carried into the ledger: 1.

- vijayraopopen/libzetta-rs

### expansion: repos libnvpair

Returned 1; not carried into the ledger: 0.


### expansion: repos illumos-nvpair

Returned 1; not carried into the ledger: 0.


### expansion: repos uzfs

Returned 1; not carried into the ledger: 0.


### expansion: repos zfsdev

Returned 1; not carried into the ledger: 0.
