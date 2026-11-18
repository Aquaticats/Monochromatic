# Rust crate for OpenZFS: technology vet (2026-11-18)

Not final and not approved.
Status: in progress; lifecycle phase: context and rubric frozen, discovery delegated.

## Report metadata

- **Subject**: Rust crate for OpenZFS.
- **Scope**: which Rust crate the git wrapper (`package/git-policy/cli`) uses to snapshot, clone, destroy,
  read and set user properties of, and list ZFS datasets, as an unprivileged user holding delegated permissions.
- **Started**: 2026-11-18T17:45:00-05:00. **Last updated**: 2026-11-18T17:45:00-05:00.
- **Governing skill**: `.agents/skills/choosing-technology/SKILL.md` at commit
  `45d42d9b23d2f2752f9c512683b45905e13afe99`, SHA-256
  `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
- **Compatibility fingerprint**: `407e8ac931da8102b402d2356e322daaef1a0bdf224b8caf612c2c64d1d3ae2e`
  (input and script: `~/temp/agent/rust-crate-for-openzfs-vet-2026-11-18/keep/compute-the-vet-report-fingerprint.ts`).
- **Active audit owner**: the coordinating Claude Code session; the discovery phase is run by a Pi run on
  DeepSeek V4.1 Flash, by the owner's answer of 2026-11-18: "Use pi with deepseek to run it."
- **Prior compatible report**: none; no file under `doc/audit/` names ZFS.

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

## Candidate ledger

One subsection per candidate: the crate or repository, the exact latest version and its date, the source repository,
where it was found, its base category, its overlays, the license its registry entry states, which OpenZFS libraries it
links or which interface it speaks, and its screening result from metadata alone (pass, fail with the evidence, or
pending targeted evidence).
Screening uses metadata only; nothing is cloned, built or run in this phase.

Not yet filled.

## Query records

Not yet filled.

## Discovery result

Pending.
