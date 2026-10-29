# Mold linker vet report

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/cli/wg-quicker-exempt` is now `package/wg/cli-quicker-exempt`.

Status:
complete;
recommendation stated: the incumbent finalist (keep rust-lld, no product change);
all validation phases finished except the recorded phase D skip;
the cleanup record lands in the final commit.

Lifecycle phase:
recommended;
adoption awaits a separate action request.

Subject:
mold linker.

Decision scope:
Evaluate mold v3.0.0 (`rui314/mold`, the Rust rewrite) as the ELF linker for Monochromatic Linux
cargo builds on the dev host, containers, and CI.
Sole external candidate per the user instruction of 2026-10-05
("Evaluate this specific candidate only").
macOS, Windows MSVC, and Android targets keep their existing linkers because mold links ELF only.
The incumbent (rust-lld 23.1.1, the rustc self-contained linker reached through the cc driver)
is carried as the status-quo baseline candidate required by the replacement parity overlay,
with evidence depth equal to its production use plus the measurements recorded in this report,
not a fresh source audit.

The ranking in this report orders mold against the per-environment status quo only;
no other external linker was evaluated, per the user restriction.

Start date:
2026-10-05.

Last updated:
2026-10-05.

Governing skill commit:
`37117e36397e30233062828581a5e38217da9458`.

Governing skill SHA-256:
`552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.

Compatibility fingerprint:
`85690ebf9c578dc57a6e21498532fb2b9f6ea52ad6241dee69bbe7e0a1c4b978`.
Fingerprint input (schema 1, RFC 8785 canonicalization, SHA-256 over 1599 UTF-8 bytes):
subject, decision scope, hard constraints, deployment, trust boundary, incumbent
(rust-lld; LLD 23.1.1 in the host mise/rustup nightly dev loop, LLD 22.1.8 in rustup-stable
container toolchains), base categories, overlays.
Superseded fingerprints, newest first:

- `6427299613fd2e86e7773be2e7fec33f43c325f7994c181d8335b09ef0784d0e`,
  recorded with a single incumbent version (LLD 23.1.1); toolchain probes then measured three
  contexts (host mise/rustup nightly loop 23.1.1, rustup-stable containers 22.1.8 class, phase
  E/E2 matrix under injected stable 1.98.1), so the incumbent version input changed on
  2026-10-05.
- `89c9982ac41072cc6c17e84414f8a1f7bf9a903306c86a87e622f3a37e24b746`,
  recorded before a hello-world link-driver probe showed that rustc's self-contained rust-lld,
  not GNU ld, is the linker this host's cargo builds consume;
  the incumbent input changed accordingly on 2026-10-05.

Active audit owner:
pi session `01a10d3d-dcca-70df-8a23-041ebf12cd66`, lock record at
`~/temp/agent/technology-vet-locks/var-home-user-Monochromatic-doc-audit-tech-mold-linker-vet-2026-10-05.md.lock/owner.json`.

Prior compatible report:
none. `doc/audit/` contains no earlier mold or linker vet report (checked by `ls doc/audit`
filtered for `mold` on 2026-10-05, result empty).

## Hard constraints

Frozen before candidate ratings:

- HC1: links rustc 1.100-nightly output for `x86_64-unknown-linux-gnu` correctly, proven at the
  consumer boundary (crate test suites pass on mold-linked binaries).
- HC2: license permits proprietary use of the tool and distribution of its linked output.
- HC3: source is inspectable at the pinned revision.
- HC4: build provenance of prebuilt release binaries is verifiable (source-to-artifact mapping,
  published checksums, inspected release workflow).
- HC5: required platforms within scope are supported: Linux ELF `x86_64` (consumed) and `aarch64`
  (available for future Linux VM use). Non-ELF targets are out of scope by classification.
- HC6: no network access and no undeclared subprocess behavior at link time.

## Base components and overlays

Candidate components and classification:

- mold v3.0.0 executable plus `mold-wrapper.so` preload library:
  inspectable open-source local technology (build tool, local executable).
- Prebuilt release tarballs: same component, consumed as native prebuilt binaries.

Active overlays:

- incumbent dependency replacement (replaces rust-lld as the linker rustc reaches through the cc
  driver on Linux; GNU ld remains for non-rustc links);
- high-trust execution (runs in every Linux build on the dev host, in containers, and in CI, and
  writes shipped native artifacts);
- native and prebuilt binary boundary (release tarballs, distro packages, `mold-wrapper.so`,
  statically linked mimalloc, dlopen of compiler LTO plugins);
- multi-platform claim (upstream builds and tests 20 ELF architectures plus Windows/macOS/FreeBSD
  host builds; this decision consumes Linux x86_64 only).

SaaS gates: not applicable, no hosted component.
Proprietary gates: not applicable, source is public and inspectable.

## Context

Measured on 2026-10-05 on the dev host (Fedora 44, kernel 7.2.7, x86_64, 16 cores, 62 GiB RAM):

- Rust toolchain: rustc and cargo `1.100.0-nightly (1303417c4 2026-09-21)`, host
  `x86_64-unknown-linux-gnu`, LLVM 23.1.1. No `rust-toolchain.toml` in the repository.
- C toolchain: GCC 16.2.1 (`cc`), GNU ld 2.46.1-1.fc44. No mold, lld, or gold binary is installed
  on the host PATH (`command -v` sweep).
- Actual rustc link path, measured with `rustc -C link-arg=-Wl,--version` on a hello world
  (2026-10-05): collect2 invokes
  `~/.rustup/toolchains/nightly-2026-09-22-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld`
  through a `-B` directory rustc adds, and the linker reports
  `LLD 23.1.1 (compatible with GNU linkers)`. The incumbent for cargo builds is therefore rustc's
  self-contained rust-lld; GNU ld 2.46.1 handles only non-rustc cc links. The initial incumbent
  record in this report named GNU ld from PATH inspection alone and is superseded by this probe.
- Incumbent per environment (this matters for SC1): the dev host uses rust-lld 23.1.1 (link-path
  probe); the Fedora 44 build container uses GNU ld, proven by the control runs' `.comment`
  output (no LLD marker on default-driver builds, mold marker on mold builds, in "Validation
  results"); GitHub Actions ubuntu runners were not measured (open item), and their
  rustup-toolchain default is inferred to be the self-contained rust-lld from the host probe
  plus upstream issue #1708 (rustup rustc 1.99 on Ubuntu 26.04 defaults to rust-lld);
  the inference is labeled, not measured.
- Toolchain contexts measured (2026-10-05, `.comment` markers plus direct probes): the
  repository dev loop (`mise exec rust`, pinned by `mise.lock`) and the bare rustup default both
  resolve to rustc 1.100.0-nightly (1303417c4 2026-09-21) with self-contained LLD 23.1.1; the
  rustup-stable toolchains inside the terminal, ide, and music-player containers carry the
  stable line's LLD (22.1.8 measured via stable 1.98.1); the phase E/E2 matrix itself ran under
  rustc/cargo stable 1.98.1 because the harness environment injects `RUSTUP_TOOLCHAIN=1.98.1`
  (global mise activation), which the `.comment` markers confirm (`Linker: LLD 22.1.8` on every
  incumbent-arm binary). All contexts use the same self-contained `gcc-ld/ld.lld` mechanism
  through the cc driver, so the matrix is representative of the container toolchains and one
  LLD major below the host nightly loop.
- Linker configuration today:
  - repository root `.cargo/config.toml` sets environment only (Zig cache dir, Slint flag),
    no linker or rustflags;
  - `package/music-player/desktop-app/.cargo/config.toml` sets per-target rustflags
    (`target-feature=+aes`) for linux-gnu, both darwin triples, and windows-msvc, and pins
    `linker = "lld-link.exe"` for `x86_64-pc-windows-msvc` only;
  - all Linux cargo linking therefore goes through the default cc driver, which rustc steers to
    its self-contained rust-lld via `-B` (measured by the link-path probe in "Context");
  - host-level `~/.cargo/config.toml` sets `build.build-dir` into the cargo cache
    (`doc/decision/cargo-build-dir-host-cache.md`), so experiments must override the build dir to
    avoid churning shared fingerprints.
- Rust consumers (main worktree, 19 `Cargo.toml` manifests excluding worktrees and vendored trees):
  CLIs (`package/cli/forbidden-strings`, `package/git-policy/cli`,
  `package/cli/nested-wayland-session`, `package/cli/wg-quicker-exempt`), linters
  (`package/linter/monochromatic-lint`, `package/linter/rust`,
  `package/rust-linter-plugin/builtin`), rust modules (`package/rust-module/*` including fuzz and
  bench sidecars), desktop apps (`package/desktop-app/ide`, `package/desktop-app/terminal`,
  `package/desktop-app/file-manager` gtk/qt/sticky variants), music player
  (`package/music-player/desktop-app`, `truepeak-core` plus bench sidecar,
  `android-app/rust` via `cargo-ndk`).
- Release profile facts relevant to linking: `package/cli/forbidden-strings` uses `lto = true`
  (rustc-internal fat LTO, the linker receives plain objects, no linker LTO plugin),
  `panic = "unwind"`; `package/git-policy/cli` uses `panic = "unwind"`.
- Where Linux links happen: this Fedora dev host, the Fedora 41 dev container
  (`package/desktop-app/terminal/Containerfile`), music-player build containers, the root
  `Dockerfile` (ubuntu:latest), and GitHub Actions ubuntu runners (`cargo-publish.yml`,
  `forbidden-strings.yml`, `logger-fuzz.yml`, `toml-edit-fuzz.yml`).
- Non-Linux link surfaces that mold cannot serve: `aarch64`/`x86_64-apple-darwin` (Apple ld),
  `x86_64-pc-windows-msvc` (lld-link, already configured), `*-linux-android` (NDK clang with its
  own lld), Zig-built terminal components (Zig toolchain linker).

Candidate identity resolution:
the user request said "new shiny rusty mold linker".
Evidence resolves this to `rui314/mold`, whose GitHub description is now
"mold 🦠: A Modern Linker in Rust 🦀" with primary language Rust
(`gh repo view rui314/mold --json description,primaryLanguage`, 2026-10-05).
The C++ implementation (releases through v2.42.1) was rewritten in Rust, announced in the v2.42.1
release notes of 2026-09-11, and v3.0.0, published 2026-10-05T10:23:19Z, is the first Rust release.
The "rusty" descriptor matches the implementation language; mold remains ELF-only, so the
evaluation scope is Linux linking.

## Frozen soft criteria and weights

No explicit user priority statement exists for this decision, so every criterion carries the
default weight 1 (skill rule: unspecified priority gets weight 1). Criteria will be refrozen if
the sensitivity matrix shows an order-controlling preference, in which case the controlling
preference is asked before finalizing.

- SC1: measured link-speed benefit for this repository's Linux cargo builds (weight 1).
- SC2: correctness and compatibility risk on the consumed surface (weight 1).
- SC3: integration and operational cost across host, containers, and CI (weight 1).
- SC4: supply-chain and auditability posture (weight 1).
- SC5: release maturity and track record (weight 1).
- SC6: documentation and ecosystem support (weight 1).

Rating anchors for SC1, frozen before any phase E result was read (2026-10-05):

- 4: mold is at least 2x faster than the environment's incumbent on the median relink cycle of a
  typical repository crate, or saves at least 2 s per median relink.
- 3: at least 25 percent faster and at least 0.3 s saved on the median relink cycle, or a
  clearly measured cold-build saving of the same proportion.
- 2: a real benefit beyond run-to-run spread but below the 25 percent or 0.3 s thresholds.
- 1: within run-to-run spread (no measurable benefit).
- 0: measurably slower than the incumbent beyond spread.

The incumbent baseline is scored on SC1 as the complement of the measured mold result.
Disclosure: this rubric has one benefit axis (SC1) against five risk and cost axes (SC2 through
SC6). Under the default equal weights that structure encodes a burden of proof for adopting a
build-critical tool; it is disclosed here rather than redesigned, and the sensitivity matrix
names which weight controls the order.

Maximum per finalist: `6 * 4 = 24` points, normalized to 100.

Unresolved preferences: none. No preference question arose: the completed sensitivity matrix
contains no order-controlling input (section "Sensitivity"), so per the scoring rule there was
no controlling preference to ask, and the outcome is evidence-decided.

## Discovery

### User restriction

The user instruction "Evaluate this specific candidate only" fixes the external candidate set to
mold. Required source classes were still searched for the candidate's own identity, registry
presence, provenance, and repository incumbents, but no breadth enumeration of alternative linkers
(lld, wild, gold) was performed beyond what mold's own README cites as comparison points.
Terminal discovery result: single candidate by user restriction; alternatives not enumerated.
This is a recorded deviation from the skill's default breadth requirement.

### Query schedule and log

Frozen schedule (executed 2026-10-05):

1. Repository incumbent class: `rg` sweeps for `Cargo.toml`, `.cargo/config.toml`,
   `linker|link-arg|fuse-ld|mold|lld` across the main worktree; `mise.toml`, `Dockerfile`,
   Containerfiles, `.github/workflows`.
   Result: no configured Linux linker override; GNU ld is the incumbent; Windows MSVC pinned to
   `lld-link.exe`.
2. Category registry class (distro): `dnf -q info mold` on Fedora 44.
   Result: `mold 2.40.4-4.fc44`, license `MIT AND (Apache-2.0 OR MIT)`, repository fedora,
   URL `https://github.com/rui314/mold`. Fedora's package predates v3.0.0.
3. Category registry class (crates.io): `GET https://crates.io/api/v1/crates?q=mold&per_page=5`.
   Result: only an unrelated yanked dependency-injection crate `mold 0.0.1` (2019).
   The linker is not distributed through crates.io; upstream issue #1661 asks for that.
4. Repository host class: `gh repo view rui314/mold` (metadata), `gh release list`/`gh release
   view v3.0.0` (assets, digests, notes), `gh api repos/rui314/mold/languages`
   (Rust 1745951, Shell 478085, C 4129 bytes), `gh api .../readme`, `gh api .../commits`,
   `gh api .../contributors?per_page=8`, `gh issue list --search "sort:updated-desc" --limit 10
   --state all`, `gh pr list` likewise, `gh run list --limit 12`,
   `gh api repos/google/oss-fuzz/contents/projects/mold` (404, not covered).
5. Broader web class: web search
   `mold linker rui314 "A Modern Linker" implementation language C++ ELF Mach-O support 2026`
   (Exa). Result: repository, README, man page, historical releases v2.0.0 (2023-07-26, the MIT
   relicense release), v2.35.0, v2.39.0, March 2026 C++-era commits, issue #1551 (LTO symbol
   lookup). No Mach-O support in mold itself; the README architecture list is ELF-only.
6. Vulnerability class: OSV `POST https://api.osv.dev/v1/query` with
   `{"commit":"8de38c35a2df16a25f7ff87ac3ad07156a925beb"}`. Result: `{}`, no advisories affect
   the pinned commit.

Expansion round: taxonomy terms collected from results were `sold`, `wild`, `rust-lld`,
`lld-link`, `setup-mold`. Per the user restriction, no expansion queries were run for alternative
linkers; `wild` and `lld` appear only as mold README benchmark comparisons. `setup-mold` (the
GitHub Action by the same author) is recorded as an integration option, not a separate candidate.

### Candidate ledger

#### mold v3.0.0 (`rui314/mold`)

- Discovery source: named by the user; confirmed through classes 2 through 6.
- Base category: inspectable open-source local technology.
- Overlays: incumbent replacement, high-trust execution, native/prebuilt boundary,
  multi-platform claim.
- Screening result: pass, no screening hard gate failed; promoted to serious alternative, then to
  hard-gate confirmed (section "Hard-gate outcomes"), then finalist.
- Pinned revision: tag `v3.0.0` = commit `8de38c35a2df16a25f7ff87ac3ad07156a925beb`
  (verified by `git ls-remote . refs/tags/v3.0.0` inside the clone and `git tag --points-at HEAD`).
- Clone: `~/temp/agent/mold-2026-10-05` (shallow, tag checkout).

#### Incumbent baseline: rust-lld 23.1.1 (rustc self-contained) via cc driver

- Discovery source: repository incumbent class (class 1) plus the hello-world link-path probe
  recorded in "Context".
- Base category: inspectable open-source local technology (binutils), consumed today as the
  distro-packaged default.
- Role: status-quo baseline required by the replacement parity overlay; kept as a candidate.
- Evidence depth note: daily production use in this repository plus the measurements in this
  report; no fresh LLVM lld source audit (deviation recorded in "Evidence limits and deviations").

#### Not evaluated (user restriction)

lld, wild, gold, and the `setup-mold` action as an independent subject: excluded from evaluation
by the user instruction; recorded here so the ledger is complete.

## Hard-gate outcomes

- HC1 (correct linking): pass. Across three crates and two linkers, test outcomes are identical
  arm-for-arm: the `forbidden-strings` suite passes fully under mold (2 + 40 + 8 tests,
  0 failures), and the pre-existing `monochromatic-lint` (4) and `git-policy-cli` (10) failures
  are byte-equal under both linkers and diagnosed as load-sensitive flakes and worktree-snapshot
  drift against the installed git 2.55.0, not linker effects (section "Validation results");
  mold-linked binaries run correctly at the CLI boundary, pass the repository's nextest runner,
  and carry the expected `.comment` identification.
- HC2 (license): pass. `LICENSE` at v3.0.0 is the MIT License, copyright 2023 Rui Ueyama
  (clone `LICENSE:1-3`). MIT is compatible with proprietary distribution of linked output;
  a linker imposes no license on its output. Fedora's compound string
  `MIT AND (Apache-2.0 OR MIT)` reflects bundled Rust dependency licensing, all permissive.
- HC3 (inspectability): pass. Public repository, cloned at the pinned tag; 97 `src/*.rs` files,
  44371 lines of core Rust plus per-arch crates, readable build system.
- HC4 (build provenance): pass. Release assets carry GitHub-computed sha256 digests
  (x86_64-linux tarball: `6c90d4a474c7c0409dfb575be03a5345878ac14fdba18de8b40fa58c60121189`);
  `release-assets.yml` builds each tarball in CI from the pushed tag via `dist.sh` inside pinned
  podman images; `dist.sh` documents pinned Debian snapshot images, pinned Rust toolchain with
  SHA-256, `--locked` cargo dependencies, pinned file timestamps, and a bit-for-bit reproducibility
  goal with one recorded exception (loongarch64 uses live Debian sid). CI runs on the v3.0.0 tag
  completed successfully (`gh run list`, 2026-10-05). Independent local rebuild is scheduled as
  bonus evidence (phase D in "Execution manifests"); the gate itself is met by the inspected
  mapping.
- HC5 (platforms): pass within scope. ELF-only by design; 20 target arches including x86_64 and
  aarch64 (`arch/` listing at v3.0.0; README architecture sentence). macOS, Windows PE, and
  Android link surfaces are explicitly out of this decision's scope and keep their current linkers;
  this is a scope fact, not a candidate failure, because the user's usage question is Linux cargo
  builds. The `mold-3.0.0-x86_64-windows.zip` asset is a Windows-hosted cross-linker for ELF
  output and is not consumed here.
- HC6 (no network, no undeclared subprocesses): pass on source inspection. `rg` over `src/` finds
  no `std::net`, socket, or HTTP usage (the only `hyper` hits are the HyperLogLog utility).
  `src/subprocess.rs` exists solely for `mold -run`, which execs the user-supplied command with
  `LD_PRELOAD=mold-wrapper.so` (documented in README section "mold -run"; wrapper source is
  `c/mold-wrapper.c`). `src/lto.rs` dlopens only the compiler-provided linker plugin passed via
  `--plugin` (same trust model as GNU ld and lld); rustc-internal LTO in this repository never
  engages that path. `build.rs` runs `git` at build time to embed the commit hash (declared in the
  build script source, build-time only).

Screening hard gates (category fit, license, source availability, provenance, security boundary,
platform): no failure. Security advisories: OSV commit query empty; repository has no published
security advisories (`gh api` security-advisories endpoint not populated; no SECURITY.md regression
found in clone root listing).

## Targeted evidence records

Access date for all URLs and API calls: 2026-10-05. Clone path for all `path:line` citations:
`~/temp/agent/mold-2026-10-05` at commit `8de38c35`.

### Project identity and history

- Claim: mold is a production ELF linker since 2021 by the original LLVM lld developer, now
  rewritten in Rust. Relevance: maturity and provenance. Gate: SC5 input. Status: pass.
  Source: README (`README.md`, "Why does linking speed matter" and "Stability" sections);
  v2.42.1 release notes (2026-09-11) announcing the rewrite and stating 2.42.1 is the last C++
  release; v3.0.0 release notes (2026-10-05) stating drop-in parity with 2.42.1, bounds-checked
  input handling replacing segfaults, and verification via per-target test suites, output
  comparison across real workloads, and full Gentoo package rebuilds with GNU ld as control.
  Corroboration: 17392 stars, 569 forks (`gh repo view`); ASPLOS 2027 paper
  `https://arxiv.org/abs/2608.23228` cited by README.

### Release cadence and timeline

- Claim: releases v2.40.1 (2025-06-09), v2.40.2 (2025-07-12), v2.40.3 (2025-07-30),
  v2.40.4 (2025-08-17), v2.41.0 (2026-04-13), v2.42.0 (2026-08-12), v2.42.1 (2026-09-11),
  v3.0.0 (2026-10-05). Relevance: maintenance activity. Gate: maintenance audit. Status: pass.
  Source: `gh release list --repo rui314/mold --limit 8`. Note: v3.0.0 was published roughly
  8 hours before this evaluation started; the Rust codebase itself is older than the release
  (external PRs merged into it during September 2026, see maintenance sample).

### Maintenance sample (10 most recently updated issues, 10 most recently updated PRs)

- Issues (`gh issue list --state all --search "sort:updated-desc" --limit 10`):
  #1708 open today, "rust-lld cannot link mold-cli": a from-source `cargo build --release` failure
  on Ubuntu 26.04 with rustc 1.99 whose default driver links with rust-lld; at sample time zero
  comments (opened 15:42Z, sampled around 18:20Z). Relevant to building mold from source, not to
  mold's linking behavior; the link-path probe shows this host's rustc also defaults to the
  self-contained rust-lld, so the failure mode is reachable for host from-source builds, while
  the container build uses Fedora rustc 1.98.1 (phase B outcome in "Validation results").
  #1706, #1707, #1661, #1704, #1703: feature requests and packaging feedback, all open,
  updated within the last two days.
  #1701 open: "mold silently ignores --no-mmap-output-file" (a silent-ignore bug report).
  #1699 opened by rui314 (big-endian PPC64 ELFv2 support).
  #1671 closed: single-dash long option parsing fixed in 3.0.0.
  #1668 closed 2026-09-21: incorrect `R_X86_64_GOTOFF64` relocation against shared-library
  symbols, reported with assembly repro by a distro toolchain developer, fixed in 3.0.0
  (release notes cite the issue). Relevance: correctness risk class and fix latency
  (report 2026-09-21 or earlier, fix released 2026-10-05). Status: low-signal to moderate
  concern for SC2, mitigated by release-notes verification claims.
- PRs (`gh pr list --state all --search "sort:updated-desc" --limit 10`):
  #1702 merged 2026-10-04 (external FreeBSD `-B` semantics fix, merged within a day);
  #1690, #1691, #1692 merged 2026-09-29 within minutes of each other (external contributor
  bitemyapp, batch-merged by the maintainer); #1694, #1695 closed unmerged with follow-up issue
  #1696 open (thread-pool tuning discussion); #1705, #1590 open feature PRs.
  Maintainer actions observed: merges, closures, issue opening; labels are not used.
- Maintainer concentration (`gh api repos/rui314/mold/contributors?per_page=8`):
  rui314 7963 contributions; next contributors ishitatsuyuki 63, sicherha 56, marxin 42, ksco 30,
  bitemyapp 22, llunak 21, ZhongRuoyu 21. Extreme single-maintainer concentration (~98%),
  sustained over five years, with corporate sponsorship disclosed in README (Mercury and others,
  $128/month+ tier). Relevance: bus-factor risk. Gate: SC4/SC5 input. Status: scored concern,
  not a hard failure; the skill treats tracker and concentration data as score-level evidence for
  a project with strong source, release, and test evidence.

### Dependency and build surface

- Claim: 70 locked packages in `Cargo.lock` at v3.0.0; direct runtime dependencies are blake3,
  bstr, cpp_demangle, crc32fast, flate2 (zlib backend), getrandom, hashbrown, libc, libz-sys,
  memchr, memmap2, portable-atomic, rayon, rustc-demangle, uuid (v4 only), xxhash-rust, zstd;
  build dependency cc. Relevance: human-auditability and supply-chain overlays. Status: pass
  with noted items. Source: clone `Cargo.toml:29-49`, `Cargo.lock` name/version extraction.
  Noted items: mimalloc comes from the author's fork pinned by git rev
  (`cli/Cargo.toml:17-18`, `https://github.com/rui314/mimalloc_rust` rev
  `3979460494f1cd1e7f936cb8e10f41e927c9f698`; lock entries `libmimalloc-sys 0.1.49`,
  `mimalloc 0.1.52`), a same-author package that extends the audit surface and requires git
  fetching at build time; `--features system-allocator` removes it.
  Enumerated after `cargo fetch --locked` in phase N2 (find over the fetched registry sources and
  git checkouts): 16 registry build scripts (`blake3`, `cpp_demangle`, `crc32fast`,
  `crossbeam-deque`, `crossbeam-epoch`, `crossbeam-utils`, `getrandom`, `libc`, `libz-sys`,
  `portable-atomic`, `proc-macro2`, `quote`, `rayon-core`, `serde_core`, `zstd-safe`, `zstd-sys`),
  plus `libmimalloc-sys/build.rs` in the pinned git checkout (it compiles the vendored
  microsoft/mimalloc submodule, fetched at revs `02a2f5df9d7d46d30263b83832eebeeab62dc5fe` and
  `d4881d338125e1cb7c47ba4cfb398d6f7c0c8d45` per the fetch log), plus mold's own `build.rs` and
  `cli/build.rs` (cc compiles `c/mold-wrapper.c` and `c/lto-message.c`; `git` embeds the commit
  hash at build time). The clone's own `.cargo/config.toml` carries a single documented flag
  (`[target.'cfg(target_arch = "loongarch64")'] rustflags = ["-Ccode-model=medium"]`, needed
  because the cc crate otherwise compiles C with the normal code model whose direct calls reach
  only ±128 MiB); it does not affect x86_64 builds.
- Source quality indicators: `rustfmt.toml` present; workspace-wide edition 2024,
  `rust-version = 1.95`; release profile `panic = "abort"`; unsafe usage concentrated in
  `src/lto.rs` (69 matches, LLVM plugin C API), `src/elf.rs` (27), `src/symbol.rs` (25),
  `src/util/concurrent_map.rs` (19), `src/util/mod.rs` (14), `src/input_files.rs` (11),
  `src/output_file.rs` (9), `src/gdb_index.rs` (6), `src/mapped_file.rs` (5), consistent with
  mmap, atomics, and C interop in a linker; per v3.0.0 notes, input parsing is bounds-checked
  and panics instead of segfaulting on corrupted input.

### Tests and CI

- Claim: 649 shell-based linker tests under `tests/` plus a Rust integration harness
  (`cli/Cargo.toml` `[[test]] integration`, `harness = false`; `tests/` workspace crate).
  CI (`ci.yml`) runs on every push and PR: all-target build and integration suite on ubuntu-24.04
  with cross toolchains and QEMU for 20 architectures (including 32-bit host checks for armv7,
  i686, powerpc), ASan and TSan sanitizer jobs on nightly with `-Zbuild-std`, an MSan job
  (`run-msan.sh`), a distro matrix (alpine, archlinux, debian:12, fedora, gentoo/stage3,
  opensuse/tumbleweed, ubuntu:22.04, ubuntu:25.04) building and running the x86_64 integration
  suite natively, plus macOS, Windows, MSYS, and FreeBSD host-build jobs.
  Source: clone `.github/workflows/ci.yml`, `install-test-deps.sh`,
  `.github/workflows/install-extras.sh` (pinned cross-toolchain and Intel SDE downloads for
  CET tests). Status: pass; strong breadth.
- Fuzzing and mutation testing: absent. `rg -iln 'fuzz|oss-fuzz|afl|libfuzzer'` over the clone
  (excluding .git) found no fuzzing infrastructure, `google/oss-fuzz/projects/mold` returns 404,
  and no mutation-testing configuration or tooling exists in the clone or its CI (a `mutants`
  search is empty; there is no `.cargo/mutants.toml`). Reported inline as required.
  Compensating evidence: ASan/TSan/MSan CI jobs, the bounds-checked Rust rewrite, and
  pre-release Gentoo-wide differential rebuilds against GNU ld (README "Stability"; v3.0.0 notes).
- CI status at the pinned tag: `gh run list` shows CI, "Build & attach tarballs on tag push", and
  "Build native tarballs" all `completed/success` for branch `v3.0.0` on 2026-10-05T07:13Z, and
  green runs on `main` and `stable` after the tag.

### Distribution and ecosystem

- Fedora 44 packages mold 2.40.4 (C++ era); v3.0.0 is not yet in distro repositories
  (`dnf -q info mold`, 2026-10-05). Relevance: SC3 (containers and CI would install from GitHub
  release tarballs or build from source until distros catch up) and SC6.
- README documents cargo/`.cargo/config.toml` integration for Rust projects
  (`rustflags = ["-C", "link-arg=-fuse-ld=mold"]`, clang driver or GCC >= 12.1; host GCC is
  16.2.1), the `-B` prefix method for older GCC, `mold -run` LD_PRELOAD interception, the
  `setup-mold` GitHub Action, and `.comment`-section verification via `readelf -p .comment`.
- Man page: `docs/mold.md` in the clone, published online and updated by `update-manpage.yml`.
- `mise registry` lists mold as `aqua:rui314/mold`; `mise ls-remote aqua:rui314/mold` on
  2026-10-05 lists up to 2.42.1 only, so installing 3.0.0 through the repository's usual mise
  tooling is not yet possible; a 3.0.0 install today means the GitHub release tarball with a
  sha256 pin, a source build, or waiting for registry and distro propagation (Fedora carries
  2.40.4).
- Documented compatibility limits (`docs/mold.md`, "Compatibility" and "Build reproducibility"
  sections): mold is a drop-in replacement for the GNU linkers for user-land linking;
  linker-script support is deliberately minimal (sufficient for glibc's `libc.so` linker script)
  with no plans to extend it, while the 3.x goal per release notes is closing remaining GNU ld
  gaps including linker scripts (upstream PR #1590 for `PROVIDE` is open); mold output is
  deterministic and carries no host-specific defaults.
- Issue #1708 rechecked around 19:45 UTC: still open, still zero comments; the prebuilt-tarball
  install path sidesteps it entirely, since it affects only from-source builds under rustup
  toolchains whose default driver is rust-lld.

## Execution manifests

Default isolation for all phases: podman container, `--cpus 4 --memory 8g --pids-limit 1024`
(recorded deviation from the 2 CPU / 2 GiB default: the host has 16 cores with ~25 GiB available;
rustc links of the mold-cli debug binary are memory-heavy per upstream issue #1708, and the 649-test
suite plus a 20-arch release build under 2 CPUs would exceed reasonable wall-clock bounds;
all phases remain in a disposable container with no ambient credentials, no real home mount, and a
private scratch volume), network enabled only during the fetch phase, `--network=none` afterwards.

- Phase N/N2 (fetch, network on): executed. Image `registry.fedoraproject.org/fedora:44`, pulled
  digest
  `sha256:86289176d4a4af5c9b9c9df225eed4489f075bb67549411da2e0d4c121854491`; flags
  `--cpus 4 --memory 8g --pids-limit 1024` (recorded deviation from the 2 CPU / 2 GiB default:
  the host has 16 cores with roughly 25 GiB available, and the rustc links of mold-cli are
  memory-heavy per upstream issue #1708); volumes `mold-eval-root` at `/root` (CARGO_HOME and
  CARGO_TARGET_DIR) and the pinned clone bind-mounted read-only at `/src` with the SELinux shared
  label `:z` (host Enforcing). `dnf install` per the `install-test-deps.sh` fedora branch plus
  gcc, git, zlib-devel, rust, cargo (deviation from the planned rustup-stable: distro packages
  avoid a curl-piped installer; the measured container toolchain is Fedora rustc/cargo 1.98.1,
  which satisfies the manifest `rust-version = 1.95`). `cargo fetch --locked` with
  `CARGO_NET_GIT_FETCH_WITH_CLI=true` downloaded the crates.io index and every locked crate, the
  pinned mimalloc_rust rev `3979460494f1cd1e7f936cb8e10f41e927c9f698`, and its two
  microsoft/mimalloc submodule revs. Success condition met: exit 0 under `set -eux`, marker
  `PHASE_N2_DONE`, elapsed 27.8 s warm. The provisioned container was committed as image
  `localhost/mold-eval:v3.0.0-deps-n2`
  (`sha256:51ad0a3946265912e7aa173b3e2da25e01b3852170dc0f54ad78f45f74e371db`) so every later
  phase runs from a fixed dependency state with the network off.
- Phase B (build and test, `--network=none`, `CARGO_NET_OFFLINE=true`, from the committed image):
  executed. `cargo build --locked -p mold-cli --no-default-features --features x86_64` (debug),
  `cargo build --release --locked -p mold-cli --no-default-features --features x86_64` (the
  parity binary for phase C2), then
  `cargo test --locked -p mold-cli --no-default-features --features x86_64 --test integration --
  --native`. Wall-clock ceiling 60 minutes (matches upstream `timeout-minutes: 60`); actual
  elapsed 151 s. Outcome in "Validation results": pass, 519 pass / 39 skip / 0 fail.
- Phase R (all-arch release build): omitted with evidence. Exact omitted command:
  `cargo build --release --locked` with default features (20 arches). The all-arch artifact class
  is exactly the release tarball whose published digest phase C1 verified and which the inspected
  tag CI built green; this repository consumes x86_64-linux-gnu links only, which the phase B
  release configuration (the fedora distro CI job's feature set) covers.
- Phase C1 (prebuilt verification): executed. Host download with curl of
  `mold-3.0.0-x86_64-linux.tar.gz` from the v3.0.0 release (download and `tar` only, no host
  execution at this step); `sha256sum --check` against the GitHub-published digest
  `6c90d4a474c7c0409dfb575be03a5345878ac14fdba18de8b40fa58c60121189`: OK. First execution of the
  extracted `bin/mold` happened inside the committed container image, `--network=none`, default
  2 CPU / 2 GiB isolation: `--version`, a GCC 16.2.1 hello world linked through
  `-fuse-ld=mold` with `ld.mold` symlinked on PATH, execution of the result,
  `readelf -p .comment`, `ldd`. Elapsed 0.8 s.
- Phase C2 (prebuilt versus source-built parity): executed. One offline container at default
  isolation linked the same C source through the tarball binary and the phase B release binary;
  compared version strings, runtime behavior, `.comment` identification, and output bytes with
  `cmp`. Elapsed 3.6 s. Outcome in "Validation results".
- Phase D (reproducibility attempt, optional): `./dist.sh x86_64` on the host under podman
  (pinned Debian snapshot image, pinned Rust toolchain with SHA-256, `--locked` deps), comparing
  the resulting binary digest with the published asset. Decided after phase E; HC4 is already met
  by the tag-to-commit mapping, the verified asset digest, the matching embedded commit hash in
  both the prebuilt and the source-built binary (phases C1, C2), and the inspected green tag CI.
  If skipped, the reason is recorded.
- Phase E (consumer boundary, host, incumbent tooling): executed, first pass (950 s total,
  driver `~/temp/agent/phase-e.mjs`). Disposable git worktree of this
  repository created at commit `b8ade98f5` (`~/temp/agent/wt-mold-eval`; concurrent sessions
  committed to main during this evaluation, the worktree stays pinned). Isolation probe measured
  first: the `CARGO_TARGET_DIR` environment variable overrides the host-level `build.build-dir`
  setting (probe artifact landed in the scratch directory), so experiments set
  `CARGO_TARGET_DIR` under `~/temp/agent` and never churn shared cargo fingerprints.
  mold binary: the digest-verified prebuilt from phase C1, exposed as `ld.mold` in a scratch PATH
  directory; `RUSTFLAGS="-C link-arg=-fuse-ld=mold"`. Host execution of the prebuilt is a
  recorded deviation from container isolation, justified by: the consumer boundary requires the
  host rustc 1.100-nightly toolchain and the shared cargo registry cache; the binary's command
  tree is audited (no network code anywhere in `src/`; subprocess spawn only under the explicit
  `mold -run` flag, which these tests never pass; writes confined to the linker output paths
  rustc provides); and the exact artifact digest matches the upstream CI output.
  Crates: `package/cli/forbidden-strings`, `package/linter/monochromatic-lint`,
  `package/git-policy/cli`. Steps per crate: full `cargo build` and `cargo test` under the
  incumbent rust-lld and under mold; interleaved relink timing (5 runs per linker after touching
  the crate root, medians and spread reported per the noise rule); `readelf -p .comment`
  verification of a mold-linked binary; one real CLI invocation per built binary. Raw `cargo`
  instead of `mise run` tasks is a recorded deviation: tasks accept no environment overrides and
  the evaluation must not modify task configuration.
  `package/music-player/desktop-app` was excluded from env-`RUSTFLAGS` testing; the corrected
  E2 precedence probe (synthetic project replicating its config shape) shows env `RUSTFLAGS`
  replaces project target-table rustflags rather than being ignored by them, so an env-based
  mold flag there would drop the required `+aes` feature (a hard `compile_error!` via gxhash,
  not a silent misbuild); that crate's mold integration is an adoption-time merge into its
  `.cargo/config.toml` target table, recorded as SC3 evidence instead of a test run.
  Deviations discovered during execution and folded into phase E2: the host `build.build-dir`
  setting shares intermediates across RUSTFLAGS variants and `CARGO_BUILD_DIR` is ignored
  (measured probe: the override directory stayed empty after a forced recompile), so cold-build
  numbers are contaminated (the `git-policy-cli` pair, 21.9 s versus 1.7 s, is excluded as
  uninterpretable) and relink runs can absorb occasional cross-arm recompiles; phase E2 gives
  every arm a private `CARGO_HOME` (registry symlinked, no config file) plus a private
  `CARGO_TARGET_DIR`. The host was also under active load from a concurrent agent session
  throughout (repository reflog shows feature commits landing mid-run); phase E captured no load
  data, phase E2 tags every timing run with before/after `/proc/loadavg`.
- Phase E2 (supplemental controls and boundary probes, host, after phase E to avoid CPU
  contention with timing runs): three-arm relink control on `forbidden-strings` adding
  `-fuse-ld=bfd` (GNU ld) alongside rust-lld and mold, with per-run load-average capture;
  `-Ztime-passes` linking-pass isolation on `forbidden-strings` in dedicated target directories
  (nightly host toolchain); a second alternating cold-build pair on `git-policy-cli`;
  a cargo precedence probe on a synthetic scratch project replicating
  `package/music-player/desktop-app/.cargo/config.toml`'s target-table rustflags, to prove or
  disprove by measurement that env `RUSTFLAGS` is ignored there; one `cargo nextest run` of a
  crate suite under mold (the repository's actual test runner, cargo-nextest 0.9.146);
  one real guard invocation of the mold-linked `cli-git-native` against a scratch commit inside
  the throwaway worktree (the git shim executes
  `@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs`, which drives the native binary;
  the invocation pattern is taken from that source); a `mise registry` check for a mold backend
  (result on 2026-10-05: `aqua:rui314/mold` exists; the `mise latest` query is recorded in E2);
  and a recheck of upstream issue #1708 (still zero comments as of 19:45 UTC) and the release
  list (no 3.0.x patch) before finalization. Execution note: the first E2 cold builds ran at
  host loadavg 50-80 on 16 cores (concurrent session activity); every E2 timing run carries
  load tags, and runs whose load differs materially between arms are filtered or discarded in
  the analysis; non-timing probes (precedence, guard invocation, nextest, markers) are
  load-insensitive. Executed: 1907 s, exit 0, marker `PHASE_E2_DONE`, zero `PROBLEM` lines
  outside the diagnosed harness defects recorded in "Validation results".
- Phase TP (corrected link-isolation arms; abandoned under saturation, see "Validation
  results"): same per-arm isolation as E2 plus
  `RUSTUP_TOOLCHAIN=nightly` forced in the child environment (fixing the injected stable
  toolchain's `-Z` rejection); `forbidden-strings` and `monochromatic-lint`, arms lld and
  mold, cold plus two touch-relinks each, capturing every `linking` pass line from
  `-Ztime-passes` output plus per-run loadavg. Driver `~/temp/agent/phase-tp.mjs`.

Undeclared command, write, or network endpoint discovered during any phase stops that phase for
manifest update and inspection before continuing.

## Validation results

- Phase N2 (fetch and lifecycle-surface enumeration, network on): pass, exit 0 under `set -eux`.
- Phase B (offline build and native integration suite): pass. Dev profile finished in 33.07 s,
  release profile in 44.36 s, no compiler warnings; the integration harness reported
  `x86_64: pass=519 skip=39 fail=0`; the source-built binary reports
  `mold 3.0.0 (8de38c35a2df16a25f7ff87ac3ad07156a925beb; compatible with GNU ld)`. Total elapsed
  151 s. The 39 skips are per-script self-skips (`tests/lib.rs:459-463` reads a `skipped` marker
  line from each test log): the fedora dependency set installs no clang, gdb, or cross
  toolchains, so those tests opt out; upstream CI runs the same suite on ubuntu-24.04 with clang,
  gdb, ten cross toolchains, QEMU, and Intel SDE and was green on the v3.0.0 tag
  (`gh run list`, 2026-10-05T07:13Z). Issue #1708's rust-lld failure did not reproduce: Fedora
  rustc 1.98.1's default driver linked mold-cli successfully in both profiles.
- Phase C1 (prebuilt digest, first execution, driver flag, `.comment` identification): pass.
  Digest OK; `--version` embeds the pinned tag commit; the linked hello world ran and links only
  libc.
- Phase C2 (prebuilt versus source-built parity): pass. Byte-identical executables from both
  binaries (`cmp` reported no difference), both ran, both carry the mold `.comment`
  identification.
- Control runs (big-link positive control, offline container, 4 CPUs, 119 s total elapsed):
  rebuilding only the `mold-cli` unit (debug profile, `cargo clean -p mold-cli` between runs)
  took 15.0 s under the container's default driver (Fedora rustc 1.98.1 to GNU ld; the output
  `.comment` carried no LLD or mold marker) and 7.2 s with `-fuse-ld=mold` on the second mold
  run (`.comment` confirmed `mold 3.0.0 (8de38c35a2df16a25f7ff87ac3ad07156a925beb; compatible
  with GNU ld)`). The first mold run (49.8 s), the `-fuse-ld=bfd` run (22.0 s), and the trailing
  default rerun (22.4 s) each followed a RUSTFLAGS change, which invalidates every unit
  fingerprint and forced full-tree rebuilds; they are excluded from the same-scope comparison.
  Same-scope result: GNU ld 15.0 s versus mold 7.2 s on a large debug link. The positive control
  passes (the harness resolves a large link-time difference), and the methodology finding is
  folded into phase E: flags stay constant within a target directory. The bfd arm moves to
  phase E2 on the host, against the real host incumbent.
- Harness failures (mine, invalidating, rerun): the first phase N and phase B payloads ran under
  `set -x` without `set -e`, so their exit 0 came from the trailing marker echo; the first phase
  B run found `cargo: command not found` for all three commands (6 s elapsed, zero work done)
  because the dnf-provisioned packages lived in the `--rm`'d phase N container overlay while only
  the `/root` volume persisted; the first phase N build-script enumeration used
  `find -maxdepth 2`, one level too shallow for `registry/src/<index>/<crate>/build.rs`.
  Fixes: `set -eux` payloads, the committed provisioned image for offline phases, `maxdepth 3`,
  and full reruns. The invalid runs are excluded from evidence.
- Phase E (consumer-boundary first pass, host, 950 s): completed; timing caveats recorded in
  "Execution manifests".
  - Correctness: the full `forbidden-strings` `cargo test --locked` suite passes under mold
    exactly as under rust-lld (2 + 40 + 8 tests, 0 failures, identical summaries in both arms).
    The other two crates fail identically under both linkers, so no failure is attributable to
    mold: `git-policy-cli` shows 312 passed / 10 failed in both arms, and the failures are
    git-differential tests (for example
    `command_add::tests::table_and_bulk_spellings_match_git`,
    `command_branch_create::git_tests::tables_match_git`,
    `command_options::git_tests::keep_dashdash_mode_matches_git`) asserting the crate's option
    tables against the installed git 2.55.0, while the worktree is pinned to `b8ade98f5`,
    which predates concurrent in-flight `feat(git-policy-cli)` work on main;
    `monochromatic-lint` shows 378 passed / 4 failed in both arms, and a clean rerun of the same
    suite passed 382 / 0 (elapsed 173.97 s under host load versus 0.79 s during phase E),
    identifying the 4 as load-sensitive flakes, linker-neutral.
  - Linker markers: every mold-arm binary carries
    `mold 3.0.0 (8de38c35a2df16a25f7ff87ac3ad07156a925beb; compatible with GNU ld)` in
    `.comment`; every incumbent-arm binary carries `Linker: LLD 22.1.8`. This proves
    `-C link-arg=-fuse-ld=mold` overrides rustc's self-contained `-B gcc-ld` shim on the host.
  - Binary sizes (debug): `forbidden-strings` 83,785,504 bytes; `monochromatic-lint`
    541,142,680 bytes; `cli-git-native` 7,679,464 bytes.
  - Relink cycles (touch crate root, rebuild; median of 5 runs per arm; phase E isolation
    caveats apply): `forbidden-strings` rust-lld 472 ms versus mold 591 ms (mold about 25
    percent slower; per-run lld 446/465/472/543 plus a 1169 ms first-run outlier, mold
    481/504/591/665/812); `git-policy-cli` rust-lld 145 ms versus mold 317 ms with
    non-overlapping bands (lld 129-179, mold 264-483), consistent with upstream issue #1696
    (thread-setup overhead on small links, open); `monochromatic-lint` rust-lld 4192 ms versus
    mold 4007 ms, within noise, with two unexplained 21 s mold outliers (21105/21553 ms)
    plausibly contention spikes.
  - Cold builds (n=1 per arm, contaminated by shared intermediates, recorded not scored):
    `forbidden-strings` lld 56.2 s / mold 49.5 s; release fat-LTO profile lld 61.8 s /
    mold 61.9 s, identical within a second and compile-dominated; `monochromatic-lint`
    mold-first 81.4 s / lld 65.3 s; `git-policy-cli` excluded as uninterpretable.
  - CLI boundary: mold-linked `forbidden-strings --version` and `--help` exit 0
    (`forbidden-strings 0.4.1`), `monochromatic-lint` exits 0 (`monochromatic-lint 0.1.0`),
    `cli-git-native --version` exits 0 (prints `git version 2.55.0`). The real guard invocation
    runs in phase E2.
  - Test-suite wall times diverged under load (`forbidden-strings` 160.6 s in the lld arm versus
    232.1 s in the mold arm) while summaries stayed identical; treated as contention noise, not
    a linker effect, and never cited as a mold runtime slowdown (the linked code is identical,
    so linker choice cannot change execution speed).
- Phase E2 (supplemental controls and boundary probes, 1907 s, driver
  `~/temp/agent/phase-e2.mjs`, per-arm `CARGO_HOME` with symlinked registry and per-arm
  `CARGO_TARGET_DIR`, `--offline --locked`): completed with the results below. Toolchain
  context: the harness environment's injected `RUSTUP_TOOLCHAIN=1.98.1` put every E2 build on
  rustc/cargo stable 1.98.1 (LLD 22.1.8 incumbent arms), representative of the container
  toolchains.
  - `forbidden-strings` three-arm relink (4 runs per arm, load tags per run): at the three
    lowest-load pairs (loadavg 54-60) lld ran 432/606/641 ms, mold 849/928 ms plus one 10607 ms
    spike, bfd 2710/4580 ms plus one 15988 ms run; higher-load runs (loadavg 65-70) compressed
    toward 2.8-4.7 s across arms. Ordering lld < mold << bfd held at every comparable load.
    `.comment` verified per arm: LLD 22.1.8 / mold 3.0.0 (`8de38c35...`) / no marker (GNU ld).
  - `monochromatic-lint` arms ran at loadavg 37-80: colds 64.4 s (lld) / 113.5 s (mold) /
    262.4 s (bfd); relinks lld 18.0-54.1 s, mold 6.2-30.5 s, bfd 68.8-112.9 s. The lld-mold
    pair is unresolvable at that load; bfd is consistently 3-15x worse than both even saturated.
    Colds excluded from comparison (load deltas between arms up to 2x).
  - `git-policy-cli`: colds reproduced the E1 anomaly under full isolation (lld-first 15970 ms
    versus mold 1371 ms and bfd 1687 ms); the first-cold-in-sequence overhead of roughly 15 s
    is arm-independent and appeared under both isolation schemes, so all gp cold numbers are
    recorded as a harness artifact and excluded. Relinks ran at loadavg 79-88 (lld 1256-14713
    ms, mold 1044-2023 ms, bfd 1731-2463 ms versus E1's clean 129-483 ms): saturated, excluded;
    E1 medians stand.
  - `-Ztime-passes` link-isolation arms failed fast in E2 (status 101 in 0.2-0.5 s at loadavg
    87+, stderr not captured: a harness omission) and the first rerun reproduced the failure
    with output: the injected stable 1.98.1 toolchain rejects `-Z` flags. Fix: force
    `RUSTUP_TOOLCHAIN=nightly` (the host dev-loop toolchain); the corrected rerun (phase TP)
    was abandoned under sustained loadavg 50-90, as recorded in the phase TP entry below.
  - Precedence probe: the in-script attempt returned status 101 with no rustc line captured
    (harness defects: `ld.mold` absent from PATH so the link failed, and the grep took the first
    `Running rustc` line, which is the build-script unit). The corrected manual rerun is
    decisive: for the bin unit, the rustc line carried `-C link-arg=-fuse-ld=mold` and did not
    carry the project config's `-C target-feature=+aes`; env `RUSTFLAGS` replaces target-table
    rustflags. This overturned the report's earlier asserted claim (recorded as superseded in
    the phase E manifest entry) and confirms the music-player integration constraint by
    measurement instead.
  - nextest under mold on `forbidden-strings`: exit 0 (547 s at loadavg around 80; nextest
    exits nonzero on any failure).
  - Guard invocation of the mold-linked `cli-git-native` in a scratch `git init` repository:
    `--help` printed git usage (exit 0), `status` forwarded to real git through a subprocess
    (exit 0, genuine output), and `commit` (with and without pathspec) returned the documented
    refusal "policy execution is not implemented in this native development executable" with
    exit 2, matching the dev-executable design; no scratch-repo mutation occurred beyond the
    refused commands.
- Phase TP (corrected `-Ztime-passes` rerun under forced `RUSTUP_TOOLCHAIN=nightly`, driver
  `~/temp/agent/phase-tp.mjs`, extraction filter fixed to the `run_linker` and `link_crate`
  pass labels): abandoned after the `forbidden-strings` lld arm (cold 107.5 s, relinks
  1945/1178 ms, all status 0, loadavg around 90). Reason: sustained host saturation (loadavg
  50-90 through E2 and TP from concurrent activity) invalidates the sub-second two-linker
  link-only comparison this phase exists to produce, and no rating cites link-only isolation
  (SC1 uses the frozen e2e anchors; direction is already settled by the E1/E2 relink bands and
  the container control). The single manual probe result is recorded in the SC1 entry.

## Score arithmetic

Semantics note frozen before ratings (2026-10-05, after the phase E first pass and before phase
E2 completion): SC1 rates link-cycle performance across the repository's actual Linux
environments, all of which install Rust through rustup (host mise nightly; terminal, ide, and
music-player containers; CI `mise install rust`) and therefore link through the rustc
self-contained rust-lld. The GNU-ld control arm is conditional evidence for environments the
repository does not currently have. The incumbent finalist is rated on the same measured matrix.

mold v3.0.0 ratings, confidence, and evidence:

- SC1 = 0.5 (low-signal range [0, 1], confidence medium). Measured matrix against the actual
  incumbent: `git-policy-cli` relink median 317 ms versus 145 ms, non-overlapping bands
  (anchor 0, measurably slower); `forbidden-strings` relink median 591 ms versus 472 ms
  (anchor 0; direction consistent across both passes; the three lowest-load E2 three-arm runs,
  loadavg 54-60, corroborate: mold 849-928 ms versus lld 432-641 ms versus GNU bfd
  2710-4580 ms);
  `monochromatic-lint` (541,142,680-byte binary) 4007 ms versus 4192 ms, within spread
  (anchor 1); E2 `monochromatic-lint` relink arms under loadavg 44-80 show lld 18-54 s, mold
  6.2-30.5 s, bfd 68.8-112.9 s, the lld-mold pair unresolvable at that load with bfd
  consistently 3-15x worse than both. No crate reached anchor 2 or higher. Mechanism
  corroborated by upstream issue #1696 (thread-setup overhead on small links, open).
  Aggregate min 0, max 1, midpoint 0.5. Aggregation rule stated for the record: the three
  crates weigh equally; the repository's dev cycle is dominated by small incremental links,
  which would push the rating toward 0, so the midpoint is not an artifact of generous
  aggregation. The two unexplained 21 s mold outliers on `monochromatic-lint` in phase E remain
  attributed only plausibly to contention (phase E captured no load data); the attribution
  stays a caveat, not a fact. Toolchain context: the matrix ran under stable 1.98.1 with
  LLD 22.1.8; the host nightly loop carries LLD 23.1.1, and the direction (mold-side overhead
  on small links) is a mold property expected to transfer across LLD majors, with magnitudes
  possibly shifting.
  Link-only isolation (`-Ztime-passes`, pass label `run_linker`): not systematically completed.
  A single manual probe under forced nightly (rustc 1.100.0-nightly, LLD 23.1.1) measured
  `run_linker = 1.639 s` for the `forbidden-strings` 84 MB debug link at loadavg around 90,
  versus e2e relink medians of 432-641 ms at loadavg around 55 in the stable toolchain: host
  saturation invalidates sub-second link-only comparisons. No rating cites link-only isolation;
  SC1 uses the frozen e2e anchors.
- SC2 = 3 (confidence medium). Passes: `forbidden-strings` full suite identical under both
  linkers (2 + 40 + 8 tests, 0 failures); `monochromatic-lint` and `git-policy-cli` failures
  byte-equal across arms and diagnosed linker-neutral (load flakes; snapshot-drift
  git-differential tests); clean `monochromatic-lint` rerun 382/0; 519-test upstream native
  integration suite offline; byte-identical prebuilt/source-built output; deterministic-output
  design; `cargo nextest run --locked --offline` (the repository's actual runner) on the mold
  arm exited 0 after 547 s at loadavg around 80 (nextest exits nonzero on any test failure);
  the mold-linked `cli-git-native` printed git usage for `--help`, forwarded `git status`
  through a real subprocess (exit 0, genuine status output), and rejected repository-changing
  commands with its documented refusal ("policy execution is not implemented in this native
  development executable", exit 2), matching the dev-executable design where policy runs in
  the installed cli-git; the native arg-parsing surface is additionally exercised by the 312
  passing git-differential tests.
  Concerns: release published the day of evaluation; the silent wrong-address relocation class
  (#1668) was fixed in this very release; open silent-ignore report (#1701); linker-script
  minimalism by design; #1708 unanswered at finalization.
- SC3 = 1 (confidence high). No 3.0.0 through mise/aqua (`mise ls-remote aqua:rui314/mold`
  tops out at 2.42.1) or Fedora (2.40.4); adoption means GitHub-tarball sha256 pinning or source
  builds (which hit the #1708 class on rustup toolchains) provisioned into the host, at least
  four container images, and CI (the same-author `setup-mold` action installs a pinnable mold
  version); per-crate config surgery where target-table rustflags exist (`music-player`: the
  measured E2 probe shows env `RUSTFLAGS` replaces, rather than merges with, project
  target-table rustflags; the bin-unit rustc line carried `-C link-arg=-fuse-ld=mold` and lacked
  the config's `target-feature=+aes`, so an env-based mold flag there drops the `+aes` feature
  gxhash requires, which gxhash turns into a hard `compile_error!` rather than a silent
  misbuild); adoption config must be
  `[target.'cfg(target_os = "linux")']`-scoped so `-fuse-ld=mold` never reaches Apple ld and
  breaks the darwin builds.
- SC4 = 3 (confidence medium). Positives: 70 locked mainstream dependencies; 18 build
  scripts enumerated and inspected; no network code in `src/`; subprocess spawn confined to the
  explicit `-run` mode; audited `dist.sh` pinning; published asset digest verified locally;
  embedded commit hash matches the pinned tag in both prebuilt and source builds; byte-identical
  parity; green tag CI. Concerns: git-rev-pinned same-author mimalloc fork; zero fuzzing
  (repository and OSS-Fuzz); maintainer concentration about 98 percent (7963 of roughly 8118
  sampled contributions); adds a new trust root to every build; phase D independent rebuild
  skipped (recorded in "Evidence limits and deviations").
- SC5 = 1.5 (low-signal range [1, 2], confidence medium). v3.0.0 published about 8 hours before
  evaluation; the Rust codebase took external PRs only from September 2026; the strong stated
  verification (2.42.1 parity, per-target suites, Gentoo full rebuild) is upstream-reported, not
  independently reproduced; the C++ lineage in production since 2021 does not transfer runtime
  soak to the new binary.
- SC6 = 4 (confidence high). README, man page, published benchmark suite (Zenodo), ASPLOS 2027
  paper, `setup-mold` action, `-run` mode, repology breadth, CI-updated man page.

mold earned = `0.5 + 3 + 1 + 3 + 1.5 + 4 = 13.0`; maximum = `6 * 4 = 24`;
normalized = `13.0 / 24 * 100 = 54.2`.

Incumbent rust-lld (22.1.8 in the repository toolchain, 23.1.1 in the bare rustup default):

- SC1 = 3 (confidence medium). Same matrix (stable 1.98.1 / LLD 22.1.8 toolchain context):
  fastest measured arm on every crate
  (`forbidden-strings` 472 ms, `git-policy-cli` 145 ms, `monochromatic-lint` 4192 ms medians);
  no documented link-time pain in the repository (searched doc/, READMEs, mise.toml; the only
  hit is an unrelated Kotlin/Native research note); absolute headroom known from upstream
  benchmarks (slower than mold at multi-GB scale, beyond current repository sizes).
- SC2 = 4 (confidence high). Daily production use in every repository Linux build with no open
  correctness signal in repository artifacts; LLVM release engineering; version-locked inside
  the rustc toolchain. Depth deviation: no fresh LLVM source audit (recorded in the ledger).
  Symmetric tracker sample (2026-10-05, same method as the mold sample): the five most recent
  open `lld` issues in llvm-project concern non-ELF flavors (lld-link PE sections, ld64.lld
  performance, Hexagon flavor detection) or building lld itself; an `lld wrong symbol` query
  returns zero open matches. No x86-64 ELF correctness red flag surfaced.
- SC3 = 4 (confidence high). Zero integration work: already the default in every environment.
- SC4 = 4 (confidence medium). No new trust surface (already consumed through the rustup/mise
  toolchain the repository pins); multi-vendor LLVM maintenance.
- SC5 = 4 (confidence high). Mature release line; rustc default driver. Ingestion caveat: the
  host loop consumes it through the rolling mise `nightly` channel (pinned by `mise.lock`, but
  movable without a repository commit) and the containers through rustup stable, so the LLD
  version drifts across contexts (23.1.1 host, 22.1.8 measured under stable 1.98.1); this
  rolling ingestion is a pre-existing property of the repository toolchain, identical with or
  without this decision.
- SC6 = 3 (confidence medium). LLVM documentation strong; the rustc self-contained `-B gcc-ld`
  mechanism is thinly documented (identifying the incumbent required a `-Wl,--version` probe).

Incumbent earned = `3 + 4 + 4 + 4 + 4 + 3 = 22`; maximum = 24; normalized =
`22 / 24 * 100 = 91.7`.

## Sensitivity

One input at a time (each equal-default weight raised 1 to 5; each medium- or low-confidence
exact rating moved one step within 0 through 4; each low-signal range tested at both endpoints):

- SC1 weight 5: mold `5*0.5 + 12.5 = 15/40 = 37.5`, incumbent `5*3 + 19 = 34/40 = 85.0`.
  Preserved. The measured SC1 range endpoint (max 1) caps mold at `5*1 + 12.5 = 17.5/40 = 43.8`
  even at weight 5, while the incumbent's worst single-input case is
  `5*2 + 19 = 29/40 = 72.5` (SC1 rating 2 at weight 5): no weight flips the order, because a
  flip would require an SC1 rating the measurements exclude.
- SC2 weight 5: mold 62.5, incumbent 95.0. Preserved.
- SC3 weight 5: mold 42.5, incumbent 95.0. Preserved.
- SC4 weight 5: mold 62.5, incumbent 95.0. Preserved.
- SC5 weight 5: mold `5*1.5 + 11.5 = 19/40 = 47.5`, incumbent `5*4 + 18 = 38/40 = 95.0`.
  Preserved.
- SC6 weight 5: mold `5*4 + 9 = 29/40 = 72.5`, incumbent `5*3 + 19 = 34/40 = 85.0`. Preserved.
- Rating shifts: mold SC2 3 to 2 or 4 gives 50.0 or 58.3; mold SC4 likewise 50.0 or 58.3;
  mold SC6 4 to 3 gives 50.0; mold SC1 endpoints 0 or 1 give 52.1 or 56.3; mold SC5 endpoints
  1 or 2 give 52.1 or 56.3. Incumbent single-step shifts (SC1 3 to 2 or 4, SC2 4 to 3,
  SC4 4 to 3, SC6 3 to 2 or 4) give 87.5 through 95.8.
- Aggregate one-input ranges: mold [50.0, 58.3], incumbent [87.5, 95.8]. Non-overlapping:
  every defined one-at-a-time test preserves the order. Stability does not cover simultaneous
  multi-input changes; the extreme simultaneous case within the allowed shift classes (every
  mold input at its maximum, every incumbent input at its minimum; high-confidence ratings do
  not shift: mold `1 + 4 + 1 + 4 + 2 + 4 = 16/24 = 66.7` versus incumbent
  `2 + 4 + 4 + 3 + 4 + 2 = 19/24 = 79.2`) still preserves it. No order-controlling preference
  exists to ask; the outcome is evidence-decided.

## Pros and cons

mold v3.0.0 pros:

- Passes every hard gate: MIT license, inspectable source at a pinned tag, provenance verified to
  the commit hash, no network code, deterministic output, bounded subprocess surface.
- Fastest measured linker where GNU ld is the baseline: 2.1x on a large container debug link
  (15.0 s to 7.2 s, mold-cli class) and 5-15x ahead of bfd on the host's 541 MB relink arm even
  under saturation; the `forbidden-strings` bfd arm ran 2.7-16.0 s versus 0.43-0.64 s for lld.
- Broadest target coverage (20 ELF architectures), all-arch plus ASan/TSan/MSan plus 8-distro
  CI, pre-release Gentoo full-package differential rebuilds (upstream-reported).
- The Rust rewrite bounds-checks corrupted input (panic instead of segfault), improving behavior
  on hostile object files.
- Strong documentation and release engineering; byte-identical prebuilt/source parity verified
  locally.

mold v3.0.0 cons:

- No measured benefit in any actual repository environment: slower than the rust-lld incumbent
  on small and medium binaries (2.2x on the 7.7 MB binary, about 25 percent on the 84 MB relink
  medians), parity on the 541 MB binary.
- Day-one major release: v3.0.0 published the evaluation morning; the silent-mislink history
  (#1668) was fixed in this release; an open silent-ignore report (#1701) and an unanswered
  same-day from-source build failure on rustup toolchains (#1708) remain.
- No 3.0.0 packaging path today (aqua/mise 2.42.1, Fedora 2.40.4): adoption requires tarball
  sha256 pinning or source builds across host, containers, and CI, config surgery for
  target-table-rustflags crates, and cfg-scoping to protect the darwin builds.
- Supply-chain additions: git-rev-pinned same-author allocator fork, zero fuzzing, roughly
  98 percent maintainer concentration, and a new trust root in every build.
- Linker-script support minimal by design (the glibc `libc.so` script suffices for the tested
  crates, but exotic link lines are a documented gap area).

Incumbent rust-lld pros:

- Already the default everywhere: zero integration and operational cost.
- Fastest measured on the repository's actual link sizes; no documented link-time pain exists.
- Version-pinned inside the already-managed rust toolchain; multi-vendor LLVM maintenance;
  mature release line.

Incumbent rust-lld cons:

- Slower than mold at multi-GB link scale (upstream benchmarks): headroom risk if repository
  binaries grow (the 541 MB linter debug binary is the current largest).
- The self-contained `-B gcc-ld` default mechanism is thinly documented; identifying the
  incumbent required a probe.
- Version drift across contexts (22.1.8 in the repository mise toolchain versus 23.1.1 in the
  rustup default) is a pre-existing condition, unchanged by this decision.

## Ranking

1. Incumbent rust-lld (keep): `22/24 = 91.7`.
2. mold v3.0.0: `13.0/24 = 54.2`.

Adjacent reason: both pass every hard gate; the incumbent leads on five of the six frozen
criteria and trails only on SC6 (documentation and ecosystem, one point), while mold's SC1
measurement range [0, 1] excludes any weighting that could flip the order; the complete
one-at-a-time sensitivity matrix preserves the ranking with non-overlapping aggregate ranges
(mold [50.0, 58.3] versus incumbent [87.5, 95.8]).

## Recommendation

Recommendation: the incumbent finalist. Keep rust-lld; make no product change. Do not adopt
mold 3.0.0 for this repository's Linux linking at this time.
This is an evidence-decided score outcome, not a hard-gate failure;
mold passes every gate and is a credible future candidate.
Revisit triggers (any one):

1. Upstream fixes small-link overhead (the issue #1696 class), removing the measured regression
   on the repository's dominant link sizes.
2. A hardened 3.0.x patch release lands and the aqua registry or Fedora packaging picks it up
   (probes: `mise ls-remote aqua:rui314/mold`, `dnf info mold`), removing the SC3 and SC5
   blockers.
3. Repository Linux link outputs grow beyond roughly 1-2 GiB, where upstream data shows mold
   pulling ahead of lld by multiples. The largest binary measured here is the 541 MB
   `monochromatic-lint` debug output; a host scan also found `monochromatic-ide` debug at
   124.6 MB and nothing above 100 MB in the host cargo build cache; the container-built
   `terminal` and `music-player` binaries leave no host artifacts and were not measured, so
   this trigger's distance is a lower bound, not a certainty.
4. A GNU-ld-default environment enters the build matrix (measured mold benefit in that
   environment: 2.1x on a large debug link, 5-15x on the 541 MB relink arm).
5. Documented link-time pain emerges (none exists today; searched).

Individual developers may opt in at user scope (`~/.cargo/config.toml`,
`[target.'cfg(target_os = "linux")'] rustflags = ["-C", "link-arg=-fuse-ld=mold"]`) without any
repository change; the measured host numbers (small-binary regression, large-binary parity) give
little upside, so this report does not recommend even that by default. Adoption of any kind
requires a separate action request and a decision record (adoption boundary).

## Evidence limits and deviations

- Single external candidate by explicit user instruction; breadth discovery of alternative linkers
  was not performed (section "Discovery").
- Incumbent baseline evidence depth is production use plus in-report measurements, not a fresh
  binutils source audit (section "Candidate ledger").
- Upstream sanitizer (ASan/TSan/MSan) and cross-architecture QEMU suites are not re-run locally;
  the pinned tag's successful CI runs (all jobs green on `v3.0.0`) are the primary evidence for
  those suites, and the locally consumed surface is x86_64-linux-gnu only, exercised by phase B's
  native integration suite and phase E. Recorded per the omission rule with exact commands in
  `.github/workflows/ci.yml` of the clone.
- Early context probes (2026-10-05T18:10Z through 18:30Z) recorded commands and outputs but not
  per-command elapsed times; all execution phases from phase N onward record elapsed time.
- Elapsed-time and container digest values are recorded inline in "Execution manifests" and
  "Validation results" as each phase completed.
- Phase D (independent bit-for-bit rebuild via `./dist.sh x86_64`) skipped: `dist.sh` was fully
  audited (digest-pinned Debian Stretch base on mirror.gcr.io, sha256-verified pinned Rust
  1.97.1 from static.rust-lang.org, snapshot.debian.org apt sources, `cargo vendor --locked`
  then a `--network=none --frozen` build, `SOURCE_DATE_EPOCH` from the commit timestamp, fixed
  file timestamps, deterministic sorted tar plus `gzip -9n`, sha256sum output). The skip cannot
  affect the recommendation: HC4 already passes on the locally verified asset digest, the
  embedded-commit match in both binaries, byte-identical link-output parity, and the green tag
  CI, and no rating cites a completed rebuild. Arithmetic bound: a successful rebuild could at
  most raise mold SC4 from 3 to 4, moving mold to `14.0/24 = 58.3`, below the incumbent's worst
  legal simultaneous case (`19/24 = 79.2`); the skip is outcome-neutral. A future adoption
  should run it first; the manifest is recorded in "Execution manifests".
- E2 `monochromatic-lint` and later timing runs executed at host loadavg 37-80 (a concurrent
  agent session building); every run carries before/after load tags; the affected timing arms
  are excluded from comparison, and the E1 medians (tighter bands, lighter load) stand as the
  matrix evidence. Non-timing E2 probes are load-insensitive and stand.
- Concurrent sessions committed features to main throughout validation (reflog evidence),
  including `feat(git-policy-cli)` changes that the pinned worktree snapshot predates; the
  worktree pin (`b8ade98f5`) kept measurements self-consistent, and the `git-policy-cli` test
  failures are snapshot drift against the installed git 2.55.0, identical under both linkers.
- Untested link surfaces: cdylib and plugin crates (`rust-linter-plugin/builtin` not built; no
  `crate-type` override found), the terminal crate's Zig and vendored-native link path
  (Fedora 41 container), CI fuzz and ASan instrumented links, release-LTO link portions beyond
  the `forbidden-strings` release-cold parity, aarch64-linux, and musl or fully static linking.
- Gentoo full-rebuild and all-arch QEMU suite claims are upstream-reported (release notes,
  README, green workflow runs at the tag), not independently reproduced.
- The CI ubuntu-runner incumbent is inferred from rustup data points (host probe plus issue
  #1708), not measured on a runner.
- The incumbent finalist received no fresh LLVM source audit (depth deviation recorded in the
  candidate ledger).
- The phase E/E2 timing matrix ran under rustc/cargo stable 1.98.1 (LLD 22.1.8) because the
  harness environment injects `RUSTUP_TOOLCHAIN=1.98.1`; the host `mise run` dev loop uses
  rustc 1.100.0-nightly (LLD 23.1.1). Both contexts use the same self-contained rust-lld
  mechanism, so directions transfer; magnitudes may shift slightly with the LLD major. The TP
  link-isolation arms run on the nightly toolchain and cover that context.
- The `git-policy-cli` first-cold-in-sequence overhead (roughly 15 s, reproduced under two
  isolation schemes, arm-independent) is an unexplained harness artifact; all gp cold numbers
  are excluded from evidence.

## Cleanup record

Executed after the final recommendation commit (filled in that commit's follow-up):

- Removed: disposable worktree `~/temp/agent/wt-mold-eval`, container image
  `localhost/mold-eval:v3.0.0-deps-n2`, volume `mold-eval-root`, scratch targets (`wt-target`,
  `e2`), probe directories (`probe-proj`, `probe-t1`, `probe-t2`, `probe-bd2`), `moldbin`,
  hello-world files.
- Kept as evidence artifacts: the pinned clone `~/temp/agent/mold-2026-10-05` (tag v3.0.0,
  commit `8de38c35a2df16a25f7ff87ac3ad07156a925beb`), the verified tarball
  `~/temp/agent/mold-dist/mold-3.0.0-x86_64-linux.tar.gz` with its extracted tree, the driver
  scripts `phase-e.mjs` and `phase-e2.mjs`, and the process logs under `/tmp/pi-processes-CDjjpP/`.
- Lock released:
  `~/temp/agent/technology-vet-locks/var-home-user-Monochromatic-doc-audit-tech-mold-linker-vet-2026-10-05.md.lock`.
- All steps executed 2026-10-05 after the recommendation commit: worktree removed
  (`git worktree remove --force`; it no longer appears in `git worktree list`), the committed
  image and volume removed (`podman rmi localhost/mold-eval:v3.0.0-deps-n2`,
  `podman volume rm mold-eval-root`), scratch directories removed, and `podman ps -a` lists no
  mold-eval containers. The base image `registry.fedoraproject.org/fedora:44` is retained as a
  standard cached base. The report lock is released immediately after this record commits.

## Open items

- Phase D decision: skipped with recorded evidence (see "Evidence limits and deviations").
- Adoption-time open items (not blockers for this recommendation): measure the CI ubuntu-runner
  incumbent in a real workflow run before any CI adoption; run the `./dist.sh x86_64`
  bit-for-bit verification before pinning a prebuilt tarball in repository config; recheck issue
  #1708 and the 3.0.x release list at adoption time.
- Cleanup executed after the final commit; the record is in "Cleanup record".
- Follow-up 2026-10-14: the user considered mold for every platform and tried it for the Windows build.
  mold 3.0.0 (the latest release) links ELF only, and its unreleased main branch adds a Mach-O linker but no PE/COFF
  output, so it cannot link for Windows.
  The user declined adopting it on Linux alone; Linux stays on rust-lld.
  Record: `doc/decision/music-player-windows-linkers.md`.
