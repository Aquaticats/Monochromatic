# Rust crate for OpenZFS: technology vet (2026-11-18)

Not final and not approved.
Status: in progress; lifecycle phase: discovery and screening done, targeted evidence under way.
Checked so far by the coordinating session: the delegated discovery's ledger against its saved provider responses
(spot checks under "Query records"), the screening judgements under "Coordinator's screening judgements", and the
OpenZFS license, interface and coverage facts under "Targeted evidence gathered (2026-11-18)".
No candidate has been cloned, built or run yet. The owner chose the kernel-interface route on 2026-11-18, which
leaves `zfsdev` as the one survivor (section "Owner's route decision").

## Report metadata

- **Subject**: Rust crate for OpenZFS.
- **Scope**: which Rust crate the git wrapper (`package/git-policy/cli`) uses to snapshot, clone, destroy,
  read and set user properties of, and list ZFS datasets, as an unprivileged user holding delegated permissions.
- **Started**: 2026-11-18T17:45:00-05:00. **Last updated**: 2026-11-18T18:31:39-05:00.
- **Governing skill**: `.agents/skills/choosing-technology/SKILL.md` at commit
  `45d42d9b23d2f2752f9c512683b45905e13afe99`, SHA-256
  `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
- **Compatibility fingerprint**: `407e8ac931da8102b402d2356e322daaef1a0bdf224b8caf612c2c64d1d3ae2e`
  (input and script: `~/temp/agent/rust-crate-for-openzfs-vet-2026-11-18/keep/compute-the-vet-report-fingerprint.ts`).
- **Active audit owner**: the coordinating Claude Code session; the discovery phase is run by a Pi run on
  DeepSeek V4.1 Flash, by the owner's answer of 2026-11-18: "Use pi with deepseek to run it."
- **Prior compatible report**: none; no file under `doc/audit/` names ZFS.
- **Discovery run**: Pi on `synthetic` `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017` through
  `package/pi-shared/subagent-driver`, run folder `~/temp/agent/pi-runs/rust-crate-for-openzfs-discovery-1`;
  its queries ran 2026-11-18T17:26:59-05:00 to 17:38:00-05:00 by the saved responses' times;
  81 turns, no retries, no rate-limit errors, cost 0.52 US dollars by the driver's `summary.json`.
  It ended `stop-file`: its work was done and the driver kept it open for unrelated processes in the main checkout
  (issue 6311), so the coordinator ended it with the driver's stop file.
- **Discovery records**: every query record, page count and screened repository is in
  [`tech-rust-crate-for-openzfs-vet-2026-11-18-discovery-records.md`](tech-rust-crate-for-openzfs-vet-2026-11-18-discovery-records.md),
  the delegate's file as it wrote it; the provider responses it read are kept under
  `~/temp/agent/rust-crate-for-openzfs-vet-2026-11-18/keep/discovery-raw/`.

## Hard constraints

- Builds and links on Linux x86-64 against OpenZFS 2.4.4 (the version installed here, `zfs version`).
- Covers snapshot, clone, destroy, user-property get and set, and dataset listing.
- Works as an unprivileged user holding delegated ZFS permissions, with no root at run time.
- Its license, and the license of every OpenZFS library it links, is compatible with a program under
  `LGPL-3.0-or-later` (the wrapper's `license` in `package/git-policy/cli/Cargo.toml`).
- Its source is inspectable, and any native linking or generated binding maps to source.
- Builds under cargo with the repository's toolchain (Rust 2024 edition).

## Base components and overlays

- **Base category**: inspectable open-source local technology (a library crate linked into the wrapper).
- **Overlay, high-trust execution**: the wrapper runs in agent and person shells and acts on their repositories
  and datasets.
- **Overlay, native boundary**: the known candidates bind OpenZFS's C libraries (`libzfs`, `libzfs_core`, `libnvpair`)
  through foreign-function calls and build scripts.
- **Not applicable, with reasons**: replacement parity (the wrapper has no ZFS code; the planning page recommended
  the `zfs` program, which the owner declined, so nothing is being replaced); managed service (none is a service);
  sensitive data (dataset names and properties only); multi-platform claim (Linux x86-64 only).

## Context

The owner chose a crate on 2026-11-18 ("A Rust crate"), against the recommendation of the `zfs` program in
`doc/planning/cli-git-zfs-worktree.md`, section "Owner's answers of 2026-11-18".
The two options as the owner saw them:
"Run `zfs` with `-H` for machine-readable output ... Cons: a new process for every call ... errors come back as text
to parse", and "A Rust crate: libzetta 0.5.0, razor-libzfscore 0.13.1 and others call ZFS's C libraries from inside
the program. Pros: typed calls, no new process."
The design there: worktrees are ZFS clones of a fresh snapshot of the checkout's dataset, made by the user under
delegated permissions (`create,clone,snapshot,destroy,mount,promote,userprop`); a ZED handler mounts and unmounts
them as root, prompted by user properties named `org.monochromatic:*`; the crate never mounts.
So the crate needs: create a snapshot, clone a snapshot, destroy a dataset (and its snapshots), get and set user
properties, list datasets under a parent with chosen properties, and report errors with their cause.

Flagged for the hard gates: OpenZFS's userland libraries are said to be under CDDL-1.0 (to be read from the
OpenZFS source at tag `zfs-2.4.4`), and the wrapper is under `LGPL-3.0-or-later`; whether linking them is compatible
is decided from the license texts, not assumed.
A crate that speaks the `/dev/zfs` ioctl interface itself, without linking the C libraries, would avoid that question
but must track the ioctl interface of 2.4.4.
A binding written for this repository is eligible only if every existing crate fails a named hard constraint.

## Frozen criteria and weights

Frozen before any candidate-specific rating; every weight is 1, since the owner stated no preference among them.

- **Fit to OpenZFS 2.4.4**: the needed operations work against 2.4.4 in validation.
- **Maintenance**: releases, maintainer response, and whether fixes reach a release.
- **Auditability**: non-test source lines, runtime dependencies, size of the `unsafe` and foreign-function surface.
- **Build cost**: what a build needs beyond cargo (bindgen and libclang, development headers, pkg-config).
- **Test evidence**: tests against a real pool, and what CI runs.
- **Error reporting**: whether a failure carries its cause in a typed form the wrapper can act on.

## Query schedule (frozen 2026-11-18T17:45:00-05:00)

Each query is run as written; each result page is recorded under "Query records".
Paginated sources continue until two complete pages in a row add no screening survivor or the provider says it has
no more; a provider cap reached first marks the class blocked, and the record says so.

### Source class 1: crates.io

- Search API, sorted by relevance, 100 per page, every page:
  `https://crates.io/api/v1/crates?q=<query>&per_page=100&page=<n>&sort=relevance`,
  for each of the queries `zfs`, `openzfs`, `libzfs`, `libzfs_core`, `lzc`, `zpool`, `nvpair`, `zfs snapshot`,
  `zfs clone`, `zfs ioctl`.
- Crates with the keyword `zfs`: `https://crates.io/api/v1/crates?keyword=zfs&per_page=100&page=<n>`.
- Crates in a filesystem category, if crates.io has one: `https://crates.io/api/v1/categories` to find it,
  then `https://crates.io/api/v1/crates?category=<slug>&q=zfs&per_page=100&page=<n>`.
- Reverse dependencies of each candidate that binds C (for example `libzfs-sys`, `razor-libzfscore`, `nvpair-sys`):
  `https://crates.io/api/v1/crates/<name>/reverse_dependencies?per_page=100&page=<n>`.

### Source class 2: GitHub

- `gh search repos zfs --language=Rust --limit 100` and the same with `--topic=zfs`, then `openzfs`, `libzfs`,
  `libzfs_core`; record the result counts and every repository not already found.
- `gh search code lzc_snapshot --language=Rust --limit 100`, and the same for `lzc_clone`, `lzc_destroy`,
  `zfs_prop_set`, `ZFS_IOC_`.
- `gh repo list openzfs --limit 200 --json name,primaryLanguage,description`, for any Rust project of the OpenZFS
  organization itself.

### Source class 3: the wider web

- The search tool with each of: `rust zfs bindings`, `rust libzfs_core crate`, `openzfs rust crate`,
  `zfs ioctl rust`, `rust zfs snapshot clone library`, `rust nvpair crate`.
- lib.rs: `https://lib.rs/keywords/zfs` and `https://lib.rs/search?q=zfs`.

### Source class 4: this repository

- `rg --ignore-case --files-with-matches 'zfs' --glob 'Cargo.toml' --glob 'Cargo.lock' package`.
- `ls doc/decision doc/audit` for any record that names ZFS.

### Expansion round

Every new term found while running the schedule (a crate family, a library name, a synonym) gets one more run of
the same queries in each class with that term; the schedule is then frozen and later terms are only recorded.

Run once, with the terms `zfsdev`, `libnvpair`, `libnv`, `illumos-nvpair`, `razor`, `libzetta`, `open3`, `veneer`
and `uzfs`; it added no candidate.

## Candidate ledger

Screening used metadata only. Each subsection names the candidate, its version and date, its repository, what it
binds or speaks, its license as stated, and the screening result after the coordinator's judgements.
The delegate's fuller entries, with every query that found each candidate, are in the discovery records file.

### libzfs, `whamcloud/rust-libzfs` (enters targeted evidence)

- `libzfs` 0.6.16 (2020-08-28) with `libzfs-sys` 0.5.11 and `libzfs-types` 0.1.2; repository archived, last push
  2023-07-11.
- Binds `libzfs` through checked-in bindgen output generated for libzfs 0.7.13 (its README: "they should only be used
  against this version"), and `libnvpair` through `nvpair-sys`.
- License: MIT. Screening: pass. Open: whether bindings generated for 0.7.13 can be correct against 2.4.4.

### zfs-core, `codyps/rust-libzfs` (enters targeted evidence)

- `zfs-core` 0.5.0 (2020-12-14) with `zfs-core-sys` 0.5.0, `nvpair` 0.5.0, `nvpair-sys` 0.4.0; last push 2023-03-10.
- Binds `libzfs_core` and `libnvpair` through pre-generated bindings and `pkg-config`.
- License: `Apache-2.0 OR MIT`. Screening: pass. Open: 2020 bindings against 2.4.4.

### razor family, `razor-zfs/razor-rs` (enters targeted evidence)

- `razor-libzfscore` 0.13.1, `razor-libnvpair` 0.13.1, `razor-nvpair` 0.13.1 (2023-07-31); `razor-libzfs` 0.12.0
  and `razor-safe-lzc` 0.12.0 (2022-07-26); last push 2023-08-27.
- Binds `libzfs`, `libzfs_core` and `libnvpair` through bindgen at build time, so a build needs libclang and the
  OpenZFS development files.
- License: `MIT OR Apache-2.0`. Screening: pass. Open: no release since 2023.

### libzetta, `Inner-Heaven/libzetta-rs` (enters targeted evidence, with a category question)

- `libzetta` 0.5.0 (2023-05-13) with `libzetta-zfs-core-sys` 0.5.2; last push 2026-02-23.
- Binds `libzfs_core`, uses FreeBSD's `libnv` through `libnv ^0.4.2`, and by its README "resorts to `zfs(8)` and
  `zpool(8)` where `lzc` falls shorts".
- License: BSD-2-Clause. Screening: pass, pending targeted evidence on which of the six needed operations it performs
  by running `zfs(8)`; any that it does fails the category for that operation (see the judgements section).

### zfsdev, `Ukko-Ylijumala/zfsdev-rs` (enters targeted evidence, on a reading open to veto)

- 0.4.8 at the repository head; `publish = false`, so no crates.io release; last push 2026-11-09.
- Speaks `/dev/zfs` directly in Rust ("No libzfs, no FFI, no zfs/zpool output scraping"), with its own packed-nvlist
  codec; mutating operations behind a `write` feature; `edition = "2024"`.
- License: `MIT OR Apache-2.0` as stated in its `Cargo.toml`. Screening: pass on the gates.
  Two open points: it is reachable only as a git dependency, which the wrapper cannot publish with (judgements
  section), and it must follow the 2.4.4 ioctl interface itself.

### clinta/zfs-rs (enters targeted evidence, weak fit)

- 0.0.1 at the repository head, never published; last push 2020-10-06.
- Sends ioctls to `/dev/zfs` itself and links only `libnvpair`.
- License: `MIT OR Apache-2.0` as stated in its `Cargo.toml`. Screening: pass on the gates; git dependency only.

### moretea/rust-zfs (enters targeted evidence, weak fit)

- 0.1.0 at the repository head, never published; last push 2018-09-16.
- Binds `libzfs`. License: LGPL-3.0, the wrapper's own family. Screening: pass on the gates; git dependency only.

### libzfs_core-sys (pending)

- 0.1.0 (2017-11-05), no repository declared; checked-in bindings over `libzfs_core`, depends on FreeBSD's
  `libnv-sys`; a raw binding layer with no API above it.
- License: BSD-2-Clause. Screening: pending one build attempt on Linux x86-64.

### zfskit, `okhsunrog/zfskit` (exit: category)

- 0.3.0 (2026-10-06), `MIT OR Apache-2.0`. Its README: "CLI-only, no FFI"; it runs `zfs(8)` and `zpool(8)` and
  parses their JSON output.
- Exits on category: it is the option the owner declined, run through a crate (judgements section).

### nessie-backend-zfs (exit: category)

- 0.4.4 (2026-11-03), `MIT OR Apache-2.0`; runs `zfs`, `zpool` and `exportfs` behind a `CommandRunner` seam as a
  storage backend for another daemon.
- Exits on category for the same reason as `zfskit`; its API is also a storage-backend trait, not a ZFS one.

### Exits by group

- **AGPL-licensed** (license; both also run the `zfs` program): `zfs-cmd-api` 0.2.0, `ChocolateLoverRaj/zfs_wrapper`.
- **No license stated, so no grant** (license): `wfraser/libzfs-rs`, `kavan-mevada/libzfs`, `akhilerm/libzfs-rs`,
  `problame/libzfs-rs`, `sgrankin/zfs-sys`.
- **On-disk format readers or reimplementations** (category): `zfs` 0.1.0, `lamzfs`, `zfs-forensic-core`, `filer-zfs`,
  `lcpfs`, `turlando/zfs-rs`, `okhsunrog/zfs-rs`.
- **Empty or placeholder publishes** (no inspectable source): `rzfs_lib`, `rzfs_userspace`, the four `zfs-rs*` 0.0.0
  crates, `abgleich-lib`.
- **FreeBSD only** (platform): the four `freqlabs/rs-*-sys` repositories.
- **nvpair or libnv codecs alone** (category; dependencies of candidates, not candidates): `nvpair-rs`,
  `stone-libnvpair`, `stone-libnvpair-sys`, `illumos-nvpair`, `illumos-nvpair-sys`, `libnv`, `libnv-sys`.
- **Raw binding layers with no API** (nothing to drive): `johnramsden/rust-zfs-sys`.
- **Applications, not libraries** (category): `chithi`, `sam-zfs-unlocker`, `zfs-autosnap`, `shock`, `zfshealth`,
  `zftop`, `ztop`, `DanielArmengod/zfs-rs`, `InkerBot/zfs-unlock`, `oxidecomputer/zfs-atomic-change-key`,
  `komori/Honeydew`.
- **Neighbouring projects** (category): `iomesh/libuzfs-rs`, `robn/veneer`, `truenas/truenas_ros`,
  `AlexanderThaller/calipe-rs`.

## Coordinator's screening judgements

Each is a reading of a settled fact or decision, stated so it can be overturned.

1.  **A crate that runs the `zfs` program is the declined option.**
    The owner chose between running `zfs` ("a new process for every call", "errors come back as text to parse") and a
    crate that calls ZFS "from inside the program" ("typed calls, no new process").
    `zfskit` and `nessie-backend-zfs` run the program, so they carry exactly the costs the owner declined and exit on
    category. `libzetta` runs it for part of its surface; it stays in only for operations it performs without the
    program, which targeted evidence checks per operation.
2.  **A crate in-process without the C libraries fits the owner's choice.**
    The option's text named the C libraries as how existing crates do it; the stated reasons were typed calls and no
    new process, which `zfsdev` and `clinta/zfs-rs` give through `/dev/zfs`. They stay in; the owner can strike them at
    the ranking.
3.  **No license stated is no grant.** Five repositories state none; with no grant there is nothing to link under,
    so they exit on the license gate. Adopted from the delegate's handling.
4.  **AGPL does not fit an `LGPL-3.0-or-later` program as a whole.** Both AGPL crates also run the `zfs` program, so
    judgement 1 already removes them and this reading decides nothing on its own.
5.  **A git-only crate needs a crates.io copy before the wrapper can use it.** The wrapper is published to crates.io
    (`.github/workflows/crate-release.yml` dispatches `cargo-publish.yml`), and the Cargo book says
    "crates.io does not allow packages to be published with dependencies on code published outside of crates.io
    itself (dev-dependencies are ignored)"
    ([Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html), read
    2026-11-18; page saved as `keep/cargo-book-specifying-dependencies-2026-11-18.html`).
    So `zfsdev`, `clinta/zfs-rs` and `moretea/rust-zfs` would each need their code vendored into the wrapper or
    published under a name of ours, which their permissive licenses allow; that cost is scored under maintenance and
    build cost, not treated as an exit.

## Query records

The full records, per query and page, with filters, sort order, counts and new candidates, are in the discovery
records file. The class results:

- **crates.io**: saturated. The ten searches took 13 pages (`zfs` 222 results over three pages, `zfs clone` 103 over
  two, the rest one page each), plus the `zfs` keyword listing, the category lookup and the `filesystem` category
  search, and 21 reverse-dependency queries, none over one page.
  The delegate read every page to each provider total instead of stopping after two pages without a survivor,
  because the `libzfs` crates are on the last page of `zfs`; the frozen stop rule would have missed them.
- **GitHub**: saturated. Two of the eight repository searches returned exactly the `--limit 100` they asked for
  (`zfs --language=Rust`, 100 of 155; `zfs --topic=zfs`, 100 of 761); the delegate ran those two again at
  `--limit 1000`, recorded both runs, and screened every repository, listed in the records file's appendix.
- **Wider web and lib.rs**: saturated; nothing that the registries had not found.
- **This repository**: saturated; no `Cargo.toml` or `Cargo.lock` under `package/` names a ZFS crate, and no
  decision or audit record names one, so there is no incumbent.

Spot checks by the coordinator against the saved responses, 2026-11-18:
`raw/crates-q-zfs-p3.json` reports total 222 and lists `libzfs-types`, `libzfs-sys`, `libzfs` and `libzfs_core-sys`;
`raw/crate-libzfs.json` gives 0.6.16 updated 2020-08-28 at `whamcloud/rust-libzfs`;
`raw/crate-libzetta.json` gives 0.5.0 updated 2023-05-13 at `Inner-Heaven/libzetta-rs`;
`raw/repos-zfs-language-rust.json` holds 100 entries and `raw/probe-repos-zfs-language-rust-limit-1000.json` 155.
Each matches the ledger.

## Discovery result

Saturated, with at least two survivors.
Entering targeted evidence: `libzfs` (whamcloud), `zfs-core`, the razor family, `libzetta`, `zfsdev`,
`clinta/zfs-rs`, `moretea/rust-zfs`; `libzfs_core-sys` is pending one Linux build.

## Owner's route decision (2026-11-18, before 18:21:50-05:00)

Asked through the question tool with the facts under "Targeted evidence gathered (2026-11-18)".

- **How the crate reaches ZFS**: "Kernel interface (Recommended)": the crate talks to `/dev/zfs` itself, in Rust,
  and links no OpenZFS library. Declined: "C libs, exception + run-time load" and "C libs, no license change".
  The answer also settles judgement 2: a crate that skips the C libraries counts as "a Rust crate".
- **Crates that run the `zfs` program**: "Out (Recommended)", which confirms judgement 1.

What this does to the ledger, as a category exit under the owner's decision (links an OpenZFS library, so it is not
a kernel-interface crate):

- `libzfs` (whamcloud), `zfs-core`, the razor family, `libzetta`, `moretea/rust-zfs` and `libzfs_core-sys` exit:
  each binds `libzfs`, `libzfs_core` or both.
- `clinta/zfs-rs` exits: it sends ioctls itself but links `libnvpair`, which is CDDL-1.0.
- `zfskit` and `nessie-backend-zfs` stay out (judgement 1, now the owner's).

Result: one survivor, `zfsdev`. The skill's rule for that case applies ("If saturated discovery finds one survivor,
record every exhausted source and continue with that survivor"): every source class is recorded as saturated under
"Query records", and `zfsdev` goes through the full open-source gates and validation alone.
A binding written for this repository stays eligible only if `zfsdev` fails a named hard constraint.

## Targeted evidence gathered (2026-11-18)

Source: OpenZFS cloned at tag `zfs-2.4.4` (annotated tag `f75f3256a0`, commit
`71a9f9578616a90c3c14bb59629fb4d31bfd68d1`, 2026-08-21T09:19:47-07:00) into
`~/temp/agent/rust-crate-for-openzfs-vet-2026-11-18/openzfs-at-zfs-2.4.4`.
Installed here: `zfs-utils 2.4.4-3` owns `/usr/lib/libzfs_core.so`; `pkg-config` reports `libzfs_core` and `libzfs`
2.4.4 with `-lzfs -lzfs_core -lnvpair`; headers are under `/usr/include/libzfs`; there is no `libnvpair.pc`.

### The OpenZFS libraries are CDDL-1.0

- Of the 25 `.c` and `.h` files under `lib/libzfs_core`, `lib/libzfs` and `lib/libnvpair`, 23 carry the
  "CDDL HEADER START" block; `lib/libnvpair/libnvpair_json.c` carries `SPDX-License-Identifier: CDDL-1.0` and the
  illumos CDDL notice instead; `lib/libzfs/os/freebsd/libzfs_zmount.c` is FreeBSD-only and not built here.
- `COPYRIGHT`: "Unless otherwise noted, all files in this distribution are released under the Common Development and
  Distribution License (CDDL)"; the one noted exception in these libraries is `lib/libzfs/THIRDPARTYLICENSE.openssl`
  (PBKDF2).
- CDDL-1.0 section 3.6 (`LICENSE` line 216): "You may create a Larger Work by combining Covered Software with other
  code not governed by the terms of this License and distribute the Larger Work as a single product. In such a case,
  You must make sure the requirements of this License are fulfilled for the Covered Software."
  So the CDDL side permits the combination.

### What the wrapper's license says about linking them

The wrapper is `LGPL-3.0-or-later`, which incorporates GPL-3.0. Its additional permissions are about a
"Combined Work", "a work produced by combining or linking an Application with the Library", where an "Application"
"makes use of an interface provided by the Library" (`LICENSES/LGPL-3.0-or-later.txt` lines 19 to 30, section 4 from
line 79). The wrapper calling `libzfs_core` is the wrapper using another library's interface, not an application
using the wrapper, so those permissions do not reach it and GPL-3.0's terms decide.

- GPL-3.0 section 1 (`LICENSES/GPL-3.0-or-later.txt` lines 134 to 145): Corresponding Source "does not include the
  work's System Libraries", and "includes ... the source code for shared libraries and dynamically linked subprograms
  that the work is specifically designed to require".
- GPL-3.0 section 1, lines 123 to 132: System Libraries are what "is included in the normal form of packaging a Major
  Component, but which is not part of that Major Component, and ... serves only to enable use of the work with that
  Major Component", a Major Component being "a major essential component (kernel, window system, and so on) of the
  specific operating system".
- The FSF's own reading ([Interpreting, enforcing and changing the GNU GPL, as applied to combining Linux and ZFS](https://www.fsf.org/licensing/zfs-and-linux),
  read 2026-11-18 through the search tool): "dynamic vs. static linking never makes any difference on the outcome of
  the analysis", and, under "Copyright Holders Can Explicitly Authorize Additional Uses", the copyright holders "are
  legally free to give additional permission for use of their work".
- Counter-reading, low weight (a mailing-list opinion, not a ruling): on guix-devel in July 2021 a participant held
  that the conflict "is specific to CDDL+GPL only, not CDDL+LGPL, so distributing a dynamically linked libvirt with ZFS
  support is allowed" (lists.gnu.org/r/guix-devel/2021-07/msg00012.html).

Result: a wrapper binary that links `libzfs_core`, `libzfs` or `libnvpair` is compatible with its license only on
the System Libraries reading (the libraries ship with the ZFS kernel module, arguably a Major Component on a ZFS root)
or with an additional permission from the wrapper's copyright holders. Choosing between those, or refusing both, is a
licensor's decision, so it goes to the owner. It applies to every survivor except `zfsdev`, which links no OpenZFS
library; `clinta/zfs-rs` links `libnvpair`.

Who could grant an additional permission, read 2026-11-18:
`git log --format='%an' -- package/git-policy/cli` counts 7,144 commits by the owner's account, 94 by
`github-actions[bot]`, and one by an author named `T` (8ed5daa61b, 2026-09-25, "refactor(git-policy-cli): satisfy lint
rules in the concurrent commit transaction").
`cargo metadata --locked` for the wrapper lists 326 packages; the only `LGPL` ones are six crates of this repository
(`forbidden-regex`, `forbidden-strings`, `monochromatic-catalog-specifier`, `monochromatic-git-policy-cli`,
`monochromatic-jsonc-edit`, `monochromatic-lint`), and the one other copyleft mention, `r-efi`, is
`MIT OR Apache-2.0 OR LGPL-2.1-or-later`, so a permissive choice. No third-party copyleft code would stand in the way
of an exception; the `T` commit is the one authorship to settle if one is written.

### Where the wrapper ships and what it links today

- `toolchain.lock.json` installs the wrapper by `"route": "archive"` from release archives for `linux-x64` and
  `linux-arm64` (the `-musl` rows point at the same `gnu` archives); `os` is `linux` only.
- `.github/workflows/cargo-publish.yml` builds those archives on GitHub-hosted runners (`ubuntu-24.04`,
  `ubuntu-24.04-arm`, and macOS and Windows runners for the other targets in its matrix).
- `ldd` on the wrapper built here from main lists only `linux-vdso`, `libgcc_s`, `libm`, `libc` and the loader.

So a crate that links `libzfs_core`, `libzfs` or `libnvpair` at build time would make those libraries a load-time
requirement of every released archive: the wrapper would stop starting on any machine or container without OpenZFS's
user-space libraries, and the release runners would need OpenZFS development files installed. Avoiding that takes a
cargo feature left out of the released archives (which would leave the installed wrapper without ZFS support) or
loading the libraries at run time, which none of the survivors does. `zfsdev` needs neither: it opens `/dev/zfs`, which
is simply absent where ZFS is not.

### libzfs_core is not a committed interface

`lib/libzfs_core/libzfs_core.c` lines 45 to 50: "The libzfs_core interface will be committed ... Currently, the
interface is Evolving (not Committed), but we intend to commit to it once it is more complete".
Line 55 to 57: it is "a thin layer, marshaling arguments to/from the kernel ioctls ... generally a 1:1 correspondence
between libzfs_core functions and ioctls". So a crate on `libzfs_core` and a crate on the ioctls carry much the same
interface risk. What OpenZFS promises for `libzfs` itself was not read; no claim is made about it here.

### libzfs_core's typed calls do not set a property after creation or list datasets

`include/libzfs_core.h` at 2.4.4 declares `lzc_snapshot`, `lzc_clone`, `lzc_create`, `lzc_destroy`,
`lzc_destroy_snaps`, `lzc_get_props`, `lzc_exists`, `lzc_rename`, `lzc_promote` and `lzc_channel_program`, among
others, and no typed function that sets a property on an existing dataset or lists datasets.
What it does offer, read from `lib/libzfs_core/libzfs_core.c`:

- `lzc_create`, `lzc_clone` and `lzc_snapshot` take a `props` nvlist (lines 303, 328, 407), so user properties can be
  set when a dataset or snapshot is made; whether delegation accepts user properties there is to be tested.
- `lzc_ioctl_fd(int, unsigned long, struct zfs_cmd *)` is exported (header line 46; source line 183) and passes any
  ioctl through, so set and list are reachable by building `zfs_cmd_t` by hand, which is the work an ioctl crate does.
- Channel programs cannot stand in: `man/man8/zfs-program.8` line 40, "Channel programs may only be run with root
  privileges."

The design needs a set after creation (a worktree asks the ZED handler to unmount it before it is destroyed, through a
user property) and a listing (`doctor` and sweeps over the worktrees parent), so a crate that offers only
`libzfs_core`'s typed calls covers those two only through `libzfs` or hand-built ioctls. This is read from headers and
sources, not yet run. To confirm per crate from its source: `zfs-core` (lzc and nvpair only by its metadata),
`razor-libzfscore` and `razor-safe-lzc` (the razor family also binds `libzfs`), and `libzetta` (which reaches
`zfs(8)` for the rest).

## zfsdev at 4559ffc711 (2026-11-10T00:21:12+02:00): gates read so far

Clone with history at `~/temp/agent/rust-crate-for-openzfs-vet-2026-11-18/zfsdev-rs-2026-11-18` (pushes disabled).

### Source, dependencies and license

- `Cargo.toml`: `zfsdev` 0.4.8, `license = "MIT OR Apache-2.0"`, `publish = false`, edition 2024, `rust-version = "1.88"`;
  runtime dependencies `strum` 0.28 and `thiserror` 2, `libc` 0.2 on Linux only, and `sha1` and `pbkdf2` behind the
  off-by-default `wrapkey` feature. Mutating requests sit behind the `write` feature. No `build.rs`.
- `doc/` holds vendored OpenZFS reference files under the CDDL and is excluded from packaging (`exclude = ["doc/", "fuzz/"]`);
  none of it is compiled.
- Size: 11,112 lines under `src/` and 929 in `tests/live_zfs.rs`; `unsafe` appears 15 times (12 in `src/ioctl.rs`,
  2 in `src/mount.rs`, 1 in `src/kstat.rs`).
- Tests: `tests/live_zfs.rs` against a real pool and two fuzz targets (`fuzz/fuzz_targets/nvlist.rs`, `kstat.rs`);
  no CI configuration in the repository.

### Layout against OpenZFS 2.4.4: verified against the running kernel

- The source says its struct layouts "mirror OpenZFS 2.2.2" and const-asserts `size_of::<ZfsCmd>() == 13744` and a
  352-byte `ZinjectRecord` (`src/ioctl.rs` lines 1033 to 1076).
- OpenZFS changed `zfs_cmd_t` after 2.2.2: at 2.4.4 `zc_defer_destroy`, `zc_flags` and `zc_action_handle` sit in a
  union with a grown `zinject_record_t` (diff of the `zfs_cmd` blocks of `doc/reference/zfs_ioctl.h` and
  `include/sys/zfs_ioctl.h`). zfsdev's own `doc/reference/2.4/README.md` analyses this at tag `zfs-2.4.4` and
  concludes the offsets and the 13,744-byte size are unchanged.
- Checked independently on the running module (`/sys/module/zfs/version` `2.4.4-1`) with
  `pahole --btf_base /sys/kernel/btf/vmlinux -C zfs_cmd /sys/kernel/btf/zfs`: size 13,744; `zc_inject_record` at
  13,296 with size 368; `zc_defer_destroy` at 13,648, `zc_flags` at 13,652, `zc_action_handle` at 13,656, which is
  352 bytes into the record, as zfsdev lays them out (output kept as
  `keep/running-kernel-zfs-cmd-layout-from-btf.txt`). A field-by-field comparison with zfsdev's compiled offsets is
  still to run.
- Request numbers: the 90 entries of `zfs_ioc_t` before the platform block sit at the same positions in zfsdev's
  vendored 2.2.2 `zfs.h` and in 2.4.4; 2.4.4 adds `ZFS_IOC_POOL_PREFETCH` and `ZFS_IOC_DDT_PRUNE` there, and the
  platform block keeps its assigned start (`keep/compare-zfsdev-ioctl-list-with-openzfs-2-4-4.ts`).
- Kernel-version gate: outside OpenZFS 2.0 to 2.4 a handle refuses mutating requests unless opted in, and every
  request travels in a zeroed 16 KiB buffer larger than the struct (README, "Platform and kernel support").

### Coverage of the six operations: clone is missing

- Present: `snapshot` (`ZFS_IOC_SNAPSHOT`), `destroy` and `destroy_snaps`, `set_prop` and `inherit_prop`,
  `objset_stats` (properties, user properties included), `datasets` (children, `ZFS_IOC_DATASET_LIST_NEXT`),
  `snapshots`, `create`, `rename` (`src/ioctl.rs` lines 2122 to 2408).
- Absent: clone and promote. zfsdev's request list has no `ZFS_IOC_CLONE` (0x5a42 at 2.4.4) or `ZFS_IOC_PROMOTE`
  (0x5a22), and no method, test or README line clones a dataset.
- What a clone takes at 2.4.4: `lzc_clone` (`lib/libzfs_core/libzfs_core.c` lines 328 to 341) sends
  `ZFS_IOC_CLONE` with the new name in `zc_name` and an nvlist holding `origin` and optional `props`;
  the kernel's `zfs_ioc_clone` (`module/zfs/zfs_ioctl.c` line 3844) reads the same two keys. zfsdev already sends
  requests of this shape (`snapshot`, `destroy_snaps`), so the missing piece is one request number and one method.

So `zfsdev` as published fails the hard constraint "Covers snapshot, clone, destroy, user-property get and set, and
dataset listing". By the report's context, that makes a binding written for this repository eligible too.

### Maintenance

- 106 commits from 2026-06-12 to 2026-11-10 by one maintainer (104 commits; 2 authored as "Claude"); 45 commits carry
  `Co-authored-by` trailers naming Claude models.
- GitHub repository created 2026-10-07, last push 2026-11-09; 0 stars, 0 forks, no issues and no pull requests
  (`gh issue list` and `gh pr list --state all`); tags `v0.4.1` (2026-10-07) to `v0.4.8` (2026-11-10).

## Targeted evidence still to gather

Each applies to every survivor it names, at equal depth.

- **License gate and route**: the owner's decision on linking the CDDL libraries, put together with the load-time
  finding under "Where the wrapper ships and what it links today".
- **Safety of validation**: read each survivor's `build.rs` before building; build in a container with no home mount;
  regenerate checked-in bindings against the 2.4.4 headers and diff them before calling anything; run old bindings and
  raw ioctls only against a file-backed pool inside a throwaway VM, never against `zpcachyos`.
- **libzetta per operation**: which of the six needed operations go through `zfs(8)`.
- **Clone and read**: each survivor at its exact revision, its build script, its foreign-function or ioctl surface,
  its error type, its tests and CI, and its maintenance record, as the skill's open-source gates list.
- **Validation**: build each against OpenZFS 2.4.4 on this machine and run the six operations as an unprivileged
  user holding delegated permissions on a throwaway dataset.
