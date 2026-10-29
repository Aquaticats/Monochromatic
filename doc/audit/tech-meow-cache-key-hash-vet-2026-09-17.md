# Technology vet: meow cache key hash

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Status:
 complete.
Lifecycle phase:
 recommended;
 discovery saturated,
 four finalists validated and scored,
 sensitivity stable for the winner,
 recommendation `twox-hash` 2.1.4 (XXH3-128) relying on the SIMD reading of hardware acceleration
 ("Recommendation").
Not adopted:
 no decision record exists.

Subject:
 meow cache key hash.

Decision scope:
 select a non-cryptographic hash library for the 128-bit content and cache keys of meow
 (placeholder name of the single-file all-Rust monorepo manager),
 replacing the planned `gxhash128`,
 with hardware acceleration on x86_64 and aarch64 Linux.

Start date:
 2026-09-17.

Last updated:
 2026-09-17.

Governing skill:

- Commit `a05818ad70a40e5769a36de669697ba109891b31`
   (last commit touching `.agents/skills/choosing-technology/SKILL.md`;
   `.claude/skills/choosing-technology/SKILL.md` is an ignored mirror with identical bytes).
- SHA-256 `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

Compatibility fingerprint:
 `0c09784add54f3c69ce4d734f0e5a8ffeb0322ef178e4025506c3a173c40fac1`.
Superseded fingerprint:
 `9d52f6e06051398b78178a00ccbf917d5c64a24a9a0f061c4938362985c9ad6e`,
 before the user's workload corrections of 2026-09-17 added HC10 and the input-size deployment facts
 ("Workload correction").
Fingerprint input and lock tooling:
 `~/temp/agent/hashvet-2026-09-17/scripts/vet-fingerprint-lock.ts` and `report-write.ts`.

Active audit owner:
 Claude Code session `e28ad59c-f3f5-46e9-8c8c-59a61c331610` (subagent).

Prior compatible report:
 none.
When this audit started,
 no file in `doc/audit/` started with `tech-meow-cache-key-hash-vet-`.
Related but incompatible:
 [`tech-monorepo-manager-vet-2026-09-16.md`](tech-monorepo-manager-vet-2026-09-16.md) selects the tool itself,
 and [`gxhash-owned.md`](../planning/monorepo-manager-route-research/gxhash-owned.md)
 is a design study of owning gxhash,
 not a vet report.

## Context

Measured or read on 2026-09-17:

- meow is designed but not implemented
   (`doc/planning/monorepo-manager-from-scratch-design.md`, "Status").
  No code consumes a cache key hash yet,
   so the consumer boundary is exercised through a probe binary,
   not product code.
- Planned incumbent:
   `gxhash` 3.5.0 `gxhash128` for cache keys
   (design doc, "Cache").
  The user removed it on 2026-09-17:
   "Pick some other hash that has hardware acceleration on x86 and ARM instead"
   (design doc, "Cache key hash after the collision findings").
- Reason for removal:
   seed-independent full-width `gxhash128` collisions on the one-byte keyset,
   reproduced locally
   (1 collision on a random 4096-byte base, 2 on a zero base)
   and matching SMHasher3 and upstream issues #83 and #124
   (`gxhash-owned.md`, "Quality and collision resistance").
- Cache key contents:
   task definition,
   argument vector,
   input file content hashes,
   declared environment,
   tool versions,
   the package's lockfile slice,
   dependency outputs,
   platform,
   and a salt
   (design doc, "Cache").
  Two input shapes follow:
   whole file contents,
   and short key material that concatenates metadata and per-file hashes.
- Content hashing is the source of truth for change detection;
   the cache is local only,
   with no adversary,
   so no cryptographic hash is needed (user, 2026-09-16).
- Supported release-blocking targets:
   Linux x86_64 and aarch64,
   each glibc-linked and static musl;
   aarch64 musl is a custom static-pie target built with `-Z build-std` and `rust-lld`,
   with no C cross compiler
   (design doc, "Platform probes", "Build matrix").
- Toolchain:
   floating `nightly` in root `mise.toml`;
   the probes used `nightly-2026-09-12` (`rustc 1.100.0-nightly`).
- Repository Rust crates are `LGPL-3.0-or-later`
   (`package/cli/forbidden-strings/Cargo.toml:21`).
- Repository parallel systems:
   the music player calls `gxhash64` for peak-cache fingerprints
   (`package/music-player/desktop-app/src/peakcache.rs`,
   `package/music-player/android-app/rust/src/fingerprint.rs`);
   `twox-hash` 2.1.2 and `rapidhash` 4.4.x arrive transitively through `turso_core`
   in `package/music-player/truepeak-core/Cargo.lock` and `package/music-player/desktop-app/Cargo.lock`;
   file-enforcer uses SHA-256 in TypeScript for its staleness manifest
   (`package/dev-script/file-enforcer/src/io/staleness-hash.ts:20`).
- Development host:
   AMD Ryzen 7 8700F (Zen 4),
   Fedora Atomic.
  Real aarch64 hardware (`ssh m1`) is powered off;
   aarch64 speed is not measured in this audit,
   and QEMU user mode is used for output equality only.

## Two readings of hardware acceleration

The user's wording,
 "hardware acceleration on x86 and ARM",
 has two readings.
Both are recorded and applied to every candidate.

- Reading R1, dedicated instructions:
   instructions that implement a specific primitive in silicon.
  On x86_64:
   AES-NI (`aesenc`),
   carry-less multiply (`pclmulqdq`, `vpclmulqdq`),
   SSE4.2 `crc32`,
   SHA extensions.
  On aarch64:
   the Cryptography Extension (`aese`, `aesmc`, `pmull`, SHA)
   and the CRC32 extension.
- Reading R2, SIMD vectorization:
   general-purpose vector instructions used by a designed vector path.
  On x86_64:
   SSE2 through AVX-512.
  On aarch64:
   NEON (Advanced SIMD) and SVE.
- Under neither reading:
   scalar 64-bit multiply-and-fold designs (wyhash family, rapidhash, komihash, MuseAir)
   and compiler auto-vectorization without a designed vector path.
  They use ordinary integer instructions every 64-bit CPU has.

Screening applies HC1 when each architecture has a path under R1 or under R2.
The ledger records R1 and R2 separately,
 and the recommendation names the reading it relies on.
For reference,
 gxhash satisfies R1 on both architectures (AES rounds)
 and uses wide vectors only in its x86 `hybrid` build.

## Classification

- Base category for every library candidate:
   inspectable open-source local technology.
- Overlays:
  - Incumbent dependency replacement:
     the planned `gxhash` 3.5.0 `gxhash128` is replaced.
    Keeping it is recorded as a candidate and exits on HC9 (user decision).
  - High-trust execution:
     the library runs inside meow,
     which runs tasks locally and in CI with repository access.
    Human-auditability measurements apply.
  - Native and generated-code boundary:
     SIMD and AES intrinsics are `unsafe` native code paths,
     and any build script or C source is a provenance boundary.
  - Multi-platform claim:
     identical output on x86_64 and aarch64 and across dispatch paths.
- Not applicable:
   SaaS gates (no hosted component)
   and the sensitive-data overlay (inputs are repository files already on the machine, and nothing leaves it).

Scope boundary for cryptographic functions:
 the decision scope names a non-cryptographic hash,
 and the user stated none is needed.
Cryptographic hashes and MACs therefore exit screening as outside the decision scope.
BLAKE3 (requested as a control)
 and any cryptographic construction that is the only kind of function meeting R1 on both architectures
 are still measured as controls,
 so the user can see what widening the scope would buy.
Controls are not ranked finalists.

## Workload correction

Relayed by the coordinator from the user on 2026-09-17,
 after the query schedule was frozen and before any candidate was rated.
Two corrections:

- meow hashes huge inputs:
   the cache records every declared task output with a content hash,
   and dependency output hashes feed downstream keys.
  The user asked for streaming hashing whose output does not depend on chunking,
   with memory bounded independent of file size;
   throughput on multi-GB inputs read from disk,
   page-cache cold and warm,
   in the bounded container with the run-to-run band first;
   whether a candidate can use several cores on one large input with stable output;
   and whether I/O or the hash limits throughput at those sizes.
- Short strings under 1 KiB are not meow's main workload,
   and about 20 KiB files are not small inputs.
  The user asked to weight the ranking by where hashed bytes and hashing time go:
   whole-file hashing from KiB to multi-GB,
   streamed;
   short-input speed matters only for composing cache keys from already-hashed parts.

Measured here on 2026-09-17:

- `package/dev-script/vm-builder/output/qcow2/disk.qcow2`:
   3,966,238,720 bytes,
   3,954,937,856 bytes allocated (`du --block-size=1`),
   so reading it exercises real storage rather than holes.
- Music player debug binaries under `package/music-player/desktop-app/target/debug/`:
   520,131,792 to 662,710,296 bytes (`find -size +400M`).
- Git-tracked files at `88cf6c0e4`:
   8,101 files,
   152,231,070 bytes,
   median 4,051 bytes,
   90th percentile 19,301,
   99th percentile 238,507,
   maximum 22,908,459 (`package/music-player/design/questions/current.html`);
   937 files are 240 bytes or shorter,
   the lengths where XXH3 has no vector path.
  The coordinator's byte shares for the same tree:
   files under 1 KiB are 22.0% of files but 0.39% of bytes,
   1 KiB to 64 KiB hold 33.46% of bytes,
   and 64 KiB and above hold 66.15%.
- Storage under `~/temp/agent` and the repository:
   btrfs on LUKS on an NVMe drive (`SPCC M.2 PCIe SSD`),
   62 GiB RAM.

What changed in this report:

- HC10 added ("Frozen hard constraints").
- Soft criteria re-weighted and split,
   so short-input speed carries weight 1
   and whole-file streamed throughput from KiB to multi-GB carries weight 5
   ("Frozen soft criteria").
- The fingerprint changed;
   the superseded one is listed in the header.
- No ranking existed yet,
   so no adjacent pair had been decided on fingerprint-sized inputs.
  The earlier gxhash study ranked on 70 to 242 byte material
   (`gxhash-owned.md`, "Benchmarks");
   none of its fingerprint-sized orderings is reused here.

## Frozen hard constraints

A candidate must satisfy every constraint.
Hard gates stay outside score arithmetic.

- HC1 hardware acceleration:
   a hardware-accelerated code path on both x86_64 and aarch64 under R1 or R2
   ("Two readings of hardware acceleration").
- HC2 width:
   a native 128-bit output defined by the algorithm.
  Concatenating two 64-bit hashes does not qualify,
   because their collisions are correlated by construction.
  The width is carried over from the `gxhash128` choice of 2026-09-16;
   the audit reports any evidence that argues for another width.
- HC3 stability:
   output of the used function is documented or specified as stable across library releases,
   and is identical on x86_64,
   aarch64,
   and every dispatch path.
- HC4 build:
   builds for `x86_64-unknown-linux-gnu`,
   `x86_64-unknown-linux-musl`,
   `aarch64-unknown-linux-gnu`,
   and the aarch64 musl static-pie target
   with the repository's nightly toolchain,
   the accelerated paths compiled by rustc alone
   and linked by `rust-lld` on aarch64 musl,
   without a C or assembly toolchain.
- HC5 CPU capabilities:
   a missing required CPU capability yields a clear diagnostic instead of a crash,
   either because the library dispatches at run time with a fallback
   or because every required feature can be checked at startup before hashing.
- HC6 license:
   permits distribution inside an `LGPL-3.0-or-later` binary.
- HC7 provenance:
   inspectable source,
   a crates.io package that maps to repository source,
   and no prebuilt or downloaded artifacts.
- HC8 collision gate:
   zero full-width 128-bit collisions on the one-byte keysets,
   meaning every single-byte variant of a zero 4096-byte base and of a random 4096-byte base,
   under several seeds.
  An ideal 128-bit function expects about 1.6 × 10^-27 collisions among 1,044,480 keys,
   so any collision marks a structural flaw.
  Checked during finalist validation,
   after a positive control shows the harness finds known collisions;
   a failure removes the finalist.
- HC9 not gxhash:
   neither `gxhash` nor output derived from it,
   per the user on 2026-09-17.
- HC10 streaming,
   added after the workload correction:
   an incremental or streaming API whose output does not depend on how the input is chunked
   and equals the one-shot result,
   with memory bounded independent of input size.

## Frozen soft criteria

Refrozen after the workload correction,
 before any candidate rating.
Each rating uses 0 through 4 with a confidence;
 low-signal ratings carry a range.

Weights:

- The user's statement to weight the ranking by where hashed bytes and hashing time go
   sets S2 and S3 to weight 5
   and S1 to weight 1.
- Every other criterion has no stated priority and weight 1.

Maximum score:
 weights sum to 19,
 so 76 points.

- S1 key-composition speed, weight 1:
   x86_64 time per hash on short key material
   (70 to 242 byte fingerprint material and per-package key material of 50 bytes to 50 KiB),
   baseline x86-64 and x86-64-v3 builds,
   with differences counted only beyond the measured run-to-run band.
- S2 whole-file throughput, KiB to tens of MiB, weight 5:
   x86_64 time over the contents of every git-tracked file,
   one-shot and streamed in 64 KiB chunks,
   plus a cache-resident 16 KiB buffer,
   both builds,
   same band rule.
- S3 multi-GB throughput from disk, weight 5:
   streamed hashing of a 3.97 GB and a 663 MB file,
   page-cache cold and warm,
   in the bounded container,
   against read-only throughput of the same files,
   so the report states whether I/O or the hash limits.
- S4 multi-core hashing of one large input with stable output, weight 1.
- S5 aarch64 acceleration, weight 1:
   what the aarch64 path accelerates,
   from source,
   plus upstream-published aarch64 measurements;
   low-signal ranges,
   because no aarch64 hardware run happens here.
- S6 quality evidence beyond HC8, weight 1:
   SMHasher3 results and their relevance to a fixed-seed local cache,
   design analysis,
   and additional local keysets.
- S7 stability assurance, weight 1:
   documented freeze,
   upstream golden vectors,
   tests that compare dispatch paths,
   and local cross-architecture equality.
- S8 CPU capability handling, weight 1:
   what meow must add to meet HC5,
   and whether build flags or run-time dispatch are needed.
- S9 auditability and dependency surface, weight 1:
   non-test lines on the used path,
   `unsafe` occurrences,
   runtime dependencies.
- S10 upstream verification, weight 1:
   tests,
   CI matrix,
   fuzzing,
   mutation testing,
   reference-vector comparisons.
- S11 maintenance, weight 1:
   releases,
   issue and pull request handling in the last 12 months,
   maintainer concentration.

Criteria removed from every denominator:

- Price:
   every candidate is free.
- Binary size:
   no size limit is stated beyond rejecting a bundled Node runtime.
- Platforms beyond Linux:
   not release-blocking (user, 2026-09-16).
- Adversarial resistance:
   the cache is local only.

Sensitivity follows the skill:
 each weight-1 criterion is raised through 5 one at a time,
 and S2 and S3 are also lowered to 1 one at a time,
 because their weight is a mapping of the user's words rather than a number the user gave.

## Unresolved preferences

- Which reading of hardware acceleration the user meant.
  Not asked here:
   the coordinator requested both readings with the recommendation's reliance stated.
- Whether a C toolchain for aarch64 musl would be acceptable if it enabled a stronger candidate.
  HC4 follows today's build;
   the report names any candidate that fails only on HC4.
- Whether cryptographic constructions are acceptable,
   now that multi-core hashing of one large input is a scored criterion
   and BLAKE3 is the main library with a tree mode.
  Recorded as a control result,
   not as a finalist.

## Discovery protocol

### Frozen query schedule

Frozen before any query ran.
Result pages are fetched unfiltered;
 screening happens after collection.

#### Source class 1: crates.io registry

Endpoint `https://crates.io/api/v1/crates`,
 `per_page=100`,
 generic User-Agent without personal identifiers,
 at most one request per second.
Default relevance sort for `q=` queries;
 `sort=downloads` for keyword listings.
Pages continue until the registry is exhausted or two consecutive complete pages add no screening survivor.

- CR01 `q=non-cryptographic hash`
- CR02 `q=128-bit hash`
- CR03 `q=simd hash`
- CR04 `q=aes hash`
- CR05 `q=neon hash`
- CR06 `q=hardware accelerated hash`
- CR07 `q=fast hash function`
- CR08 `q=gxhash`
- CR09 `q=fingerprint hash`
- CR10 `q=crc simd checksum`
- CR11 `keyword=hash`
- CR12 `keyword=hasher`
- CR13 `keyword=non-cryptographic`
- CR14 named families,
   one query each:
   `q=xxh3`,
   `q=xxhash`,
   `q=highwayhash`,
   `q=museair`,
   `q=rapidhash`,
   `q=komihash`,
   `q=wyhash`,
   `q=t1ha`,
   `q=meowhash`,
   `q=ahash`,
   `q=foldhash`,
   `q=blake3`,
   `q=umash`,
   `q=clhash`,
   `q=crc32c`,
   `q=crc64`.

Metadata screening is backed by a source scan of each result's newest `.crate` archive from `static.crates.io`:
 intrinsics or SIMD abstractions per architecture,
 128-bit outputs,
 build scripts,
 and C sources.
The scan never builds or runs an archive.

#### Source class 2: GitHub

`gh api search/repositories` with `per_page=100`,
 best-match order,
 pages until exhaustion or two consecutive pages without a new screening survivor;
 the API caps each query at 1,000 results.
`gh api search/code` for the code queries.

- GH01 `topic:hash-function language:Rust`
- GH02 `topic:hashing language:Rust`
- GH03 `topic:non-cryptographic-hash-function`
- GH04 `hash simd neon language:Rust`
- GH05 `aes hash neon language:Rust`
- GH06 `128-bit hash language:Rust`
- GH07 `non-cryptographic hash in:name,description`
- GH08 `gxhash alternative`
- GC01 code `vaeseq_u8 _mm_aesenc_si128 language:Rust`
- GC02 code `vmull_p64 _mm_clmulepi64_si128 language:Rust`
- GC03 code `__crc32cd _mm_crc32_u64 language:Rust`

Hash-family indexes on repository hosts:
 the SMHasher3 results list (`gitlab.com/fwojcik/smhasher3`, `results/README.md`)
 and the `rurban/smhasher` README,
 with every listed family screened at family level.

#### Source class 3: broader web

Linkup search,
 `depth` standard,
 no domain filter.
The provider exposes no page cursor;
 that limit is recorded and covered by registry and repository enumeration.

- W01 `non-cryptographic 128-bit hash with hardware acceleration on x86 and ARM`
- W02 `gxhash alternative`
- W03 `fastest non-cryptographic hash NEON AVX2 128-bit output`
- W04 `AES-NI and ARMv8 crypto extensions based non-cryptographic hash function`
- W05 `Rust hash crate stable output SIMD aarch64`
- W06 `XXH3 vs HighwayHash vs MuseAir vs rapidhash comparison`
- W07 `CRC32 instruction based 128-bit hash function`
- W08 `SMHasher3 results hardware accelerated hash functions`

#### Source class 4: this repository

Uncapped `rg` over the working tree,
 then reading every match in plans,
 decisions,
 audits,
 manifests,
 and lockfiles:

- RP01 `gxhash`
- RP02 `xxh3|xxhash|twox`
- RP03 `highway`
- RP04 `museair|rapidhash|komihash|wyhash|foldhash|ahash`
- RP05 `blake3|siphash|siphasher|crc32|crc64|fnv`
- RP06 `doc/decision/` and `doc/audit/` file names containing `hash`

#### Expansion round

After the initial schedule:

1.  Collect new taxonomy terms from the candidate ledger.
2.  Append one de-duplicated query per applicable source class.
3.  Freeze the schedule.
4.  Record later terms without new queries.

## Discovery results

Executed 2026-09-17.
Raw pages:
 `~/temp/agent/hashvet-2026-09-17/data/crates/`,
 `data/github/`,
 and the saved web results;
 source archives under `data/crate-src/`.

### Source class 1: crates.io

Every query used `per_page=100` and the User-Agent `hash-library-survey/1.0`.
No result was filtered out before screening.
A result counts as reviewed when it is a hash, checksum, MAC, or hash adapter crate;
 every other result is screened by the source scan and listed in the appendix.

Source-scan method and control:

- Each result's newest stable `.crate` archive was downloaded from `static.crates.io` and extracted, never built.
- Non-test Rust files were searched for x86 intrinsics (`_mm*_` calls, AES, CLMUL, CRC32, and SHA intrinsics),
   aarch64 intrinsics (NEON `v*q` calls, `vaeseq_u8`, `vmull_p64`, `__crc32*`, SHA intrinsics),
   SIMD abstractions (`std::simd`, `wide`, `safe_arch`, `pulp`, `multiversion`),
   128-bit outputs,
   build scripts,
   and C, C++, or assembly files
   (`scripts/crates-scan.ts`).
- Control:
   crates known to carry accelerated paths were flagged on both architectures
   (`gxhash` x86 46/7 and aarch64 48/3 general/dedicated matches,
   `highway` 131 and 44,
   `twox-hash` 16 and 27,
   `sha2` 58/4 and 178/72),
   and crates known to be scalar had zero matches
   (`rapidhash`, `museair`, `komihash`, `wyhash`, `foldhash`, `seahash`, `siphasher`, `polymur-hash`).
  `blake3` 1.8.7 matched on x86 only,
   consistent with its NEON path being C (`c/blake3_neon.c`, compiled in `build.rs`).
- Limit:
   a crate that delegates acceleration to a dependency shows no matches of its own;
   dependents of the candidate crates were listed and are adapters or applications.
- Totals:
   3,322 distinct crates,
   158 with matches on both architectures,
   85 with a SIMD abstraction,
   47 without a downloadable stable version
   (mostly `0.0.0` placeholders).

Query ledger,
 in schedule order,
 with the expansion round (XR) after the initial schedule:

- CR01 `q=non-cryptographic hash`:
   170 reported,
   2 page(s) and 170 results read,
   exhausted;
   new reviewed candidates:
   `noncrypto-digests`, `wyhash`, `gxhash`, `mwhash`, `fasthash`, `drtahash`, `ahash`,
   `hashkit`, `polymur-hash`, `t1ha`, `tenthash`, `hashcrew`, `rscrypto`, `axhash-core`,
   `foldhash`, `rapidhash`, `whasher`, `rotohash-rs`, `blazehash-core`;
   new screening survivors: `hashcrew`.
- CR02 `q=128-bit hash`:
   749 reported,
   5 page(s) and 500 results read,
   two-page rule met;
   new reviewed candidates:
   `museair`, `siphasher`, `kangarootwelve`, `rustc-stable-hash`, `polyval`, `twox-hash`,
   `lockstitch`, `highway`, `ghash`, `hashcodecs`, `argon2-rust`;
   new screening survivors: `twox-hash`, `highway`.
- CR03 `q=simd hash`:
   724 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates:
   `seq-hash`, `cubehash`, `crc32fast`, `thread-utilities`, `keccak-batch`, `autobahn-hash`,
   `gearhash`, `pocx_hashlib`, `simd-adler32`, `tape-sha256`, `blake3-std`, `adler32-simd`,
   `mm3h`, `halftime`, `blake3`;
   new screening survivors: `autobahn-hash`.
- CR04 `q=aes hash`:
   878 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates: `sponge-hash-aes256`, `purecrypto`;
   new screening survivors: none.
- CR05 `q=neon hash`:
   243 reported,
   3 page(s) and 243 results read,
   exhausted;
   new reviewed candidates: `sha3-selkie`, `xxh3`, `graviola`, `polyhash`, `xxhash-rust`;
   new screening survivors: `xxhash-rust`.
- CR06 `q=hardware accelerated hash`:
   266 reported,
   3 page(s) and 266 results read,
   exhausted;
   new reviewed candidates: `verify-beacon`, `crc32c`, `lib-q-keccak`;
   new screening survivors: none.
- CR07 `q=fast hash function`:
   1,862 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates: `seahash`, `umash`, `hud-slice-by-8`;
   new screening survivors: none.
- CR08 `q=gxhash`:
   24 reported,
   1 page(s) and 24 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR09 `q=fingerprint hash`:
   1,117 reported,
   5 page(s) and 500 results read,
   two-page rule met;
   new reviewed candidates: none;
   new screening survivors: none.
- CR10 `q=crc simd checksum`:
   83 reported,
   1 page(s) and 83 results read,
   exhausted;
   new reviewed candidates: `crc64fast`, `crc-fast`, `crc64fast-nvme`, `librscrc`, `oxiarc-core`;
   new screening survivors: none.
- CR11 `keyword=hash&sort=downloads`:
   919 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates: `sha2`, `sha1`, `bitcoin_hashes`, `k12`, `meowhash`, `sha2ni`, `xxhash-c-sys`, `cyfs-sha2`;
   new screening survivors: none.
- CR12 `keyword=hasher&sort=downloads`:
   46 reported,
   1 page(s) and 46 results read,
   exhausted;
   new reviewed candidates: `slice-by-8`, `axhash`;
   new screening survivors: none.
- CR13 `keyword=non-cryptographic&sort=downloads`:
   6 reported,
   1 page(s) and 6 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-xxh3 `undefined`:
   115 reported,
   2 page(s) and 115 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-xxhash `undefined`:
   143 reported,
   2 page(s) and 143 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-highwayhash `undefined`:
   7 reported,
   1 page(s) and 7 results read,
   exhausted;
   new reviewed candidates: `highwayhash`;
   new screening survivors: none.
- CR14-museair `undefined`:
   3 reported,
   1 page(s) and 3 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-rapidhash `undefined`:
   36 reported,
   1 page(s) and 36 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-komihash `undefined`:
   3 reported,
   1 page(s) and 3 results read,
   exhausted;
   new reviewed candidates: `komihash`;
   new screening survivors: none.
- CR14-wyhash `undefined`:
   35 reported,
   1 page(s) and 35 results read,
   exhausted;
   new reviewed candidates: `wyhash2`;
   new screening survivors: none.
- CR14-t1ha `undefined`:
   4 reported,
   1 page(s) and 4 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-meowhash `undefined`:
   1 reported,
   1 page(s) and 1 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-ahash `undefined`:
   193 reported,
   2 page(s) and 193 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-foldhash `undefined`:
   30 reported,
   1 page(s) and 30 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-blake3 `undefined`:
   1,447 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates: `chacha20-blake3`, `spg-crypto`;
   new screening survivors: none.
- CR14-umash `undefined`:
   2 reported,
   1 page(s) and 2 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-clhash `undefined`:
   2 reported,
   1 page(s) and 2 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- CR14-crc32c `undefined`:
   176 reported,
   2 page(s) and 176 results read,
   exhausted;
   new reviewed candidates: `bitcoin-crc32c`, `turbo_crc`;
   new screening survivors: none.
- CR14-crc64 `undefined`:
   41 reported,
   1 page(s) and 41 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- XR01 `q=incremental hash simd`:
   144 reported,
   2 page(s) and 144 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- XR02 `q=high-bandwidth checksum`:
   12 reported,
   1 page(s) and 12 results read,
   exhausted;
   new reviewed candidates: none;
   new screening survivors: none.
- XR03 `q=parallel tree hash`:
   899 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates: none;
   new screening survivors: none.
- XR04 `q=universal hash`:
   931 reported,
   3 page(s) and 300 results read,
   two-page rule met;
   new reviewed candidates: none;
   new screening survivors: none.

Pagination:
 CR02 and CR09 had a survivor on page 3,
 so pages 4 and 5 were fetched;
 neither added a survivor.
Result:
 source class 1 is saturated.

### Source class 2: GitHub

`gh api search/repositories` and `search/code`,
 best-match order,
 `per_page=100`;
 every query ran until GitHub reported fewer than 100 items on a page,
 so no query reached the 1,000-result cap.

- GH01 `topic:hash-function language:Rust`:
   10 results,
   exhausted;
   hash libraries already in class 1 (`museair`) plus cryptographic and Murmur3 repositories;
   no new survivor.
- GH02 `topic:hashing language:Rust`:
   174 results over 2 pages,
   exhausted;
   `aHash`,
   `gxhash`,
   `twox-hash`,
   `rapidhash`,
   `wyhash-rs`,
   `metrohash-rs` (scalar, HC1),
   `fastmurmur3` (scalar, HC1),
   `autobahn-hash`,
   `cityhash-sys` (C bindings, HC4),
   and applications;
   no new survivor.
- GH03 `topic:non-cryptographic-hash-function`:
   0 results.
- GH04 `hash simd neon language:Rust`:
   2 results
   (a hash table and `keccak-batch`, cryptographic);
   no survivor.
- GH05 `aes hash neon language:Rust`:
   0 results.
- GH06 `128-bit hash language:Rust`:
   1 result
   (`forrus`, a fixed 128-bit-input function, category mismatch).
- GH07 `non-cryptographic hash in:name,description`:
   215 results over 3 pages,
   exhausted;
   families in other languages
   (xxHash C, komihash, a5hash, pengyhash, SMHasher forks),
   Rust repositories not published on crates.io
   (`s0uthview/fernhash`, `hwadii/hache`, `yescallop/mixhash`, `AlyssaRoseDev/cmhash`, `101shaan/BlitzHash`;
   crates.io API returned no crate for each, HC7),
   and `meow-land/mwhash` and `cessen/tenthash` (published, scalar, HC1);
   no new survivor.
- GH08 `gxhash alternative`:
   0 results.
- GC01 code `vaeseq_u8 _mm_aesenc_si128 language:Rust`:
   110 results in 119 repositories over 2 pages;
   vendored `ahash` copies,
   `haraka-rs` (fixed-length cryptographic),
   RandomX and proof-of-space application code,
   an AES random generator,
   and a MeowHash attack tool;
   no hash library with both paths beyond `ahash` (HC2, HC3) and `gxhash` (HC9).
- GC02 code `vmull_p64 _mm_clmulepi64_si128 language:Rust`:
   97 results;
   carry-less multiply utilities,
   GF(2^128) field arithmetic,
   CRC libraries,
   and SIMD parsers;
   no 128-bit hash library.
- GC03 code `__crc32cd _mm_crc32_u64 language:Rust`:
   28 results;
   CRC32C libraries (HC2) and application-internal hash-table hashers;
   no 128-bit hash library.
- XG01 `rotohash` (expansion):
   2 results,
   `jandrewrogers/RotoHash` (C++ reference, x86-64 only per its README, HC4 as a Rust candidate)
   and `int08h/rotohash-rs` (already in class 1).
- XG02 `incremental hash simd language:Rust` (expansion):
   0 results.
- XG03 `parallel hash tree language:Rust` (expansion):
   1 result
   (`dupefind`, an application).

Hash-family indexes:

- SMHasher3 `results/README.md`
   (fetched 2026-09-17 from `gitlab.com/fwojcik/smhasher3/-/raw/main/results/README.md`, 546 lines).
  Families with hardware-specific paths:
   XXH3 (vector, both architectures),
   HighwayHash (vector, both),
   UMASH (CLMUL and PMULL, C),
   MeowHash (x86 AES),
   falkhash, aesnihash, and t1ha0.aes (x86 AES),
   CLhash (x86 CLMUL),
   CityHashCrc and MetroHashCrc (CRC32 instructions, C++),
   FARSH (x86 vector),
   HalftimeHash, VHASH, NMHASH, khashv (outputs of 64 bits or fewer),
   gxhash (HC9),
   and BLAKE3 (cryptographic).
  Pure-Rust implementations with streaming exist only for XXH3 and HighwayHash;
   UMASH, CityHashCrc, and MetroHashCrc exist for Rust only as C or C++ bindings or scalar ports (HC4 or HC1),
   and the x86-only families fail HC1.
- `rurban/smhasher` README
   (fetched 2026-09-17, 486 lines):
   adds `crc32_hw`, `crc64_hw`, and `crc32_pclmul` (32 and 64-bit, HC2),
   `metrohash128crc` (CRC32 instructions on SSE4.2 and NEON, C++, flagged "UB"),
   and `pearsonhash128` (SSSE3 only, HC1).

Result:
 source class 2 and the family indexes are saturated.

### Source class 3: broader web

Linkup search,
 `depth` standard,
 no domain filter,
 no page cursor;
 each query returned one result list of about 10 to 20 entries.

- W01 `non-cryptographic 128-bit hash with hardware acceleration on x86 and ARM`:
   `JohanLindvall/haste` (Go XXH3 with generated SSE2, AVX2, AVX-512, NEON, and SVE2 kernels; not Rust),
   xxHash,
   SHA extension and AES instruction references,
   and a Cryptography Stack Exchange question "What is the fastest stable 128-bit non-cryptographic hash function?"
   (read through the Stack Exchange API because the page returned HTTP 403;
   it has no answers).
- W02 `gxhash alternative`:
   comparison-site pages and gxhash articles;
   no new candidate.
- W03 `fastest non-cryptographic hash NEON AVX2 128-bit output`:
   `haste`,
   a Go XXH3 package,
   Ash Vardanian's post on AWS Graviton checksums
   (StringZilla: 64-bit AES-based hashing in C, HC2 and HC4).
- W04 `AES-NI and ARMv8 crypto extensions based non-cryptographic hash function`:
   instruction-set references and the RustCrypto `aes` crate description;
   no new hash library.
- W05 `Rust hash crate stable output SIMD aarch64`:
   `highway-rs`,
   `crc32fast`,
   and `xxhash-rust` ("Neon - Enabled by default on aarch64 targets");
   no new candidate.
- W06 `XXH3 vs HighwayHash vs MuseAir vs rapidhash comparison`:
   MuseAir,
   rapidhash,
   and xxHash issue #257 comparing 128-bit hashes;
   no new candidate.
- W07 `CRC32 instruction based 128-bit hash function`:
   MatrixOne's AES and CRC32 hash-table hashers in Go assembly,
   CityHash CRC variants;
   no Rust library.
- W08 `SMHasher3 results hardware accelerated hash functions`:
   SMHasher3,
   rapidhash,
   rainstorm (scalar);
   no new candidate.
- W09 `RotoHash alternative high-throughput 128-bit checksum for large files` (expansion):
   RotoHash,
   the XXH3 announcement ("Presenting XXH3", on UMAC's flaw for large checksums),
   BLAKE3 articles;
   no new library.
- W10 `multi-core parallel non-cryptographic hash with stable output for multi-gigabyte files` (expansion):
   Stack Overflow and Stack Exchange threads on parallel and tree hashing
   (BLAKE3, Skein, KangarooTwelve, MD6, and chunk-then-hash constructions,
   all cryptographic or consumer-built);
   no new non-cryptographic library.

Limit:
 the provider exposes no pagination,
 so this class cannot be enumerated.
Registry and repository enumeration,
 which are complete,
 covered every web-found Rust library.

### Source class 4: this repository

`rg --count-matches --ignore-case` over the working tree,
 excluding `node_modules` and lockfiles,
 then reading every match in plans,
 decisions,
 audits,
 manifests,
 and lockfiles:

- RP01 `gxhash`:
   the music player's two call sites and manifests,
   the gxhash study,
   the design doc,
   the troubleshooting doc,
   and `doc/decision/monorepo-manager-all-rust.md`.
- RP02 `xxh3|xxhash|twox`:
   `twox-hash` 2.1.2 through `turso_core` in two music-player lockfiles;
   the handover records that a static musl daemon skeleton with XXH3 built and ran on 2026-09-16
   (`doc/handover/monorepo-manager.md:94`).
- RP03 `highway`:
   only the Highway SIMD C++ library in the package table
   (`package/dev-script/file-enforcer/src/data/packages.generated.ts:5758`),
   unrelated to HighwayHash.
- RP04 `museair|rapidhash|komihash|wyhash|foldhash|ahash`:
   the gxhash study's benchmarks;
   `rapidhash` 4.4.x and `ahash` 0.8.12 as transitive dependencies.
- RP05 `blake3|siphash|siphasher|crc32|crc64|fnv`:
   an earlier all-Rust design that planned `blake3` 1.8.7 content hashes
   (`doc/planning/monorepo-manager-route-research/stack-rust.md:201`, `:547`),
   later replaced by gxhash;
   a TypeScript CRC-32 in `package/module/zip-writer`;
   and transitive `siphasher`.
- RP06:
   no file name in `doc/decision/` or `doc/audit/` contains `hash`
   other than this report.

No hand-rolled 128-bit hash exists in the repository.
The gxhash reimplementation prototype lives in the session scratchpad only.

### Expansion round

New taxonomy terms from the ledger:
 incremental hashing,
 high-bandwidth checksum (RotoHash),
 tree or parallel hashing (BLAKE3, bao),
 universal hashing (HalftimeHash, POLYVAL).
One de-duplicated round ran:
 XR01 to XR04 on crates.io,
 XG01 to XG03 on GitHub,
 W09 and W10 on the web.
The schedule is frozen;
 later terms (Merkle tree, KangarooTwelve) were recorded without new queries.

### Terminal discovery result

Saturated with at least two survivors:
 `xxhash-rust`,
 `twox-hash`,
 `hashcrew`,
 `highway`,
 and `autobahn-hash` passed screening.
The broader-web class has no cursor;
 its results were covered by the complete registry and repository classes.

## Screening

Every result is listed with its outcome in
 [`screening-crates.md`](tech-meow-cache-key-hash-vet-2026-09-17/screening-crates.md)
 (3,322 crates).
This section is the candidate ledger for every hash library reviewed by hand,
 grouped by outcome.
Unless stated otherwise,
 each candidate's base category is inspectable open-source local technology
 with all four overlays from "Classification",
 and evidence comes from its crates.io archive under `data/crate-src/` read on 2026-09-17.

### Result under each reading of hardware acceleration

- R2, SIMD vectorization:
   five screening survivors,
   all XXH3-128 or HighwayHash implementations.
- R1, dedicated instructions:
   no non-cryptographic survivor.
  The functions that meet R1 on both architectures are
   `gxhash` (HC9),
   `ahash` (HC2, HC3),
   `rotohash-rs` (HC10, HC3),
   and the cryptographic `sha2` (SHA extensions) and AES-CMAC through `aes` (AES instructions),
   which are outside the decision scope and measured as controls.
- R1 and R2 together:
   only `rotohash-rs` and `gxhash`,
   both excluded.

So any recommendation relies on R2.

### Screening survivors

#### `xxhash-rust` 0.8.18

- Discovery:
   CR05 first,
   also CR11, CR12, CR14, W05.
- Function:
   XXH3-128 one-shot `xxh3_128` and streaming `Xxh3::digest128` (`src/xxh3.rs:1228`).
- R1:
   no.
- R2:
   yes on both;
   SSE2, AVX2, and AVX-512 paths on x86 and NEON on aarch64,
   chosen by `cfg(target_feature)` at compile time (`src/xxh3.rs:16-55`);
   the vector path covers inputs over 240 bytes.
- Screening gates:
   HC2 pass (128-bit),
   HC3 pending targeted evidence (XXH3 output frozen upstream),
   HC4 pending (no build script, no C),
   HC6 pending (BSL-1.0),
   HC10 pending (streaming API present).
- Result:
   serious alternative.

#### `twox-hash` 2.1.4

- Discovery:
   CR02 first,
   also CR11, CR12, CR14, GH02;
   already a transitive dependency of two music-player lockfiles (2.1.2).
- Function:
   XXH3-128 `XxHash3_128::oneshot` and streaming `write`/`finish_128` (`src/xxhash3_128.rs:35`, `:132`, `:141`).
- R1:
   no.
- R2:
   yes on both;
   run-time AVX2 or SSE2 dispatch on x86 and NEON on aarch64 with the `std` feature (`src/xxhash3/large.rs:100-120`).
- Screening gates:
   HC2 pass,
   HC3 pending (README "Portability": output does not depend on the platform),
   HC4 pending,
   HC6 pending (MIT),
   HC10 pending.
- Result:
   serious alternative.

#### `hashcrew` 0.3.0

- Discovery:
   CR01 first,
   also CR05, CR06, CR09, CR14.
- Function:
   XXH3-128 `xxh3_128` and streaming `Xxh3_128` (`src/xxhash/xxh3.rs:190`, `:1082-1089`).
- R1:
   no for XXH3
   (its CRC family uses CRC and CLMUL instructions but outputs 32 bits).
- R2:
   yes on both;
   NEON, SSE2, and AVX2 kernels for inputs over 240 bytes,
   run-time detection with `std` (`src/xxhash/kernel/mod.rs:58-90`).
- Screening gates:
   HC2 pass,
   HC3 pending,
   HC4 pending,
   HC6 pending (Apache-2.0),
   HC10 pending.
- Result:
   serious alternative.

#### `highway` 1.3.0

- Discovery:
   CR02 first,
   also CR03, CR05, CR07, CR11, CR12, CR14, W05.
- Function:
   HighwayHash-128 through `HighwayHasher::hash128` and streaming `append`/`finalize128` (`src/builder.rs:119-139`).
- R1:
   no.
- R2:
   yes on both;
   SSE4.1 and AVX2 with run-time detection and a portable fallback on x86 (`src/builder.rs:147-185`),
   NEON on aarch64.
- Screening gates:
   HC2 pass,
   HC3 pending (README: "generate consistent 64, 128, and 256bit hashes across all hardware"),
   HC4 pending,
   HC6 pending (MIT),
   HC10 pending.
- Result:
   serious alternative.

#### `autobahn-hash` 0.1.0

- Discovery:
   CR03 first,
   also CR14, GH02.
- Function:
   HighwayHash `hash_128` through nightly `portable_simd` and `multiversion` (`src/lib.rs:1`, `:318`).
- R2:
   yes on both through `core::simd`.
- Result at screening:
   serious alternative;
   exited at targeted evidence on HC4 ("Hard-gate outcomes for the finalists").

### Exits among named and near-miss candidates

- `gxhash` 3.5.0,
   the incumbent kept as a candidate:
   HC9 (user decision of 2026-09-17).
  Also HC10:
   upstream closed #127 because `Hasher` output differs from the one-shot result and depends on write chunking
   (`gxhash-owned.md`, "`Hasher` streaming API").
- `rotohash-rs` 0.1.2
   (CR01, XR02, XG01),
   RotoHash, 128-bit,
   AES-NI with AVX2 or VAES with AVX-512 on x86 and NEON with the AES extension on aarch64:
   the only candidate meeting R1 and R2 on both architectures.
  HC10:
   the crate offers one-shot `hash` and `hash_with_seed` only (`src/lib.rs:87`, `:158`),
   although the reference README says the algorithm "supports incremental hashing".
  HC3 not established:
   the reference README says "The baseline algorithm is unlikely to change
   but analysis and verification of hash quality is ongoing",
   and RotoHash is absent from the SMHasher3 results list.
  Without HC10 it would have needed targeted evidence;
   hashing multi-GB outputs through it would require memory-mapping whole files.
- `hashcodecs` 1.4.1 (CR02):
   HC10 for its XXH3-128,
   which has one-shot, prepared-seed, and batch APIs but no streaming state;
   HC1 for its MurmurHash3 x64-128,
   whose aarch64 body is the scalar path (`src/murmur3/x64_128.rs:175-176`).
- `xxh3` 0.1.1 (CR05):
   HC10,
   one-shot `hash128_with_seed` only (`src/xxh3.rs:86`);
   last release 2022-05-20.
- `museair` 0.6.0 (CR02, CR14):
   HC1,
   scalar multiply design with no intrinsics (source scan 0 and 0);
   it passes all SMHasher3 tests,
   which does not offset a hard gate.
- `rapidhash` 4.5.1,
   `komihash` 0.5.0,
   `wyhash` 0.6.0 and its ports,
   `polymur-hash` 0.2.2,
   `seahash` 4.1.0:
   HC1 (scalar) and HC2 (64-bit).
- `foldhash` 0.2.0:
   HC1,
   HC2,
   and HC3 (output not stable by design).
- `ahash` 0.8.12:
   HC2 (64-bit `Hasher`)
   and HC3 (README: output is not stable across versions or platforms).
- `drtahash` 0.0.17:
   HC3 (per-map keys) and HC2.
- `t1ha` 0.1.2:
   HC1 (x86 intrinsics only)
   and HC3 (`t1ha0` output differs by platform).
- `meowhash` 0.3.0:
   HC1 ("aarch64 support has been disabled as of version 0.2", `src/arm.rs:9`)
   and HC3 (MeowHash 0.5 not declared final).
- `axhash-core` 1.0.0,
   `mm3h` 0.1.3:
   HC2 (64-bit outputs).
- `siphasher` 1.0.3,
   `rustc-stable-hash` 0.1.2,
   `tenthash` 1.1.0,
   `metrohash` 1.0.7,
   `cityhasher` 0.1.0,
   `cityhash-rs` 1.0.1,
   `fastmurmur3` 0.2.0,
   `murmur3` 0.5.2,
   `zwohash` 0.1.2,
   `mwhash` 0.1.1:
   HC1 (scalar).
- CRC and Adler crates
   (`crc-fast` 1.10.0, `crc32fast` 1.5.2, `crc32c` 0.6.8, `crc64fast` 1.1.0, `crc64fast-nvme` 1.2.1,
   `librscrc`, `turbo_crc`, `slice-by-8`, `hud-slice-by-8`, `bitcoin-crc32c`, `adler32-simd`, `simd-adler32`):
   HC2,
   outputs of 64 bits or fewer,
   although several meet R1 on both architectures.
- `umash` 0.6.1 through `umash-sys`,
   `highwayhash` 0.0.14,
   `fasthash` 0.4.0,
   `xxhash-c-sys` 0.8.7,
   `cityhash` 0.1.1,
   `clhash-sys`:
   HC4,
   C or C++ sources compiled by build scripts.
- `halftime` 0.1.1,
   `polyval` 0.7.3,
   `polyhash` 0.3.1,
   `ghash`:
   category mismatch,
   universal hashes that need key material and a padding scheme chosen by the consumer.
- `noncrypto-digests` 0.4.0,
   `hashkit` 0.1.5,
   `blazehash-core`,
   `thread-utilities`,
   `whasher`:
   adapters over other hash crates;
   their algorithms are screened through the underlying crates.
- Repositories not on crates.io
   (`fernhash`, `hache`, `mixhash`, `cmhash`, `BlitzHash`):
   HC7.
- Families available only in other languages or as bindings:
   RotoHash C++ reference (x86-64 only),
   UMASH (C),
   MetroHash128CRC and CityHashCrc (C++),
   MeowHash (C, x86 AES),
   `haste` (Go),
   StringZilla (C, 64-bit):
   HC4,
   with HC1 or HC2 where noted.

### Cryptographic functions carried as controls

- `blake3` 1.8.7:
   outside the decision scope.
  On aarch64 its NEON implementation is C (`c/blake3_neon.c`, built in `build.rs` unless `pure`),
   so under HC4 its aarch64 path is portable code:
   it fails HC1 under both readings when no C toolchain is allowed.
  Kept as the requested control
   and because its `update_rayon` tree mode is the one library way to hash one input on several cores.
- `sha2` 0.11.0:
   outside the decision scope;
   meets R1 on both architectures through SHA extensions with run-time detection;
   measured as the R1 control,
   truncated to 128 bits.
- AES-CMAC through `cmac` 0.8.0 and `aes` 0.9.3:
   outside the decision scope;
   meets R1 on both through AES instructions with run-time detection;
   measured as the AES control.
- Other cryptographic crates
   (`sha1`, `k12`, `kangarootwelve`, `blake3-std`, `cubehash`, `keccak-batch`, `graviola`, `rscrypto`,
   and others in the appendix):
   outside the decision scope,
   not measured.

## Targeted evidence and hard-gate outcomes

### Shared evidence fields

Unless a record says otherwise:

- Access date:
   2026-09-17.
- Host:
   Fedora Atomic,
   kernel `7.2.0-ogc6.1.fc44.x86_64`,
   AMD Ryzen 7 8700F (Zen 4, AVX-512, VAES, VPCLMULQDQ, SHA extensions),
   62 GiB RAM.
- Scratch root:
   `~/temp/agent/hashvet-2026-09-17/`,
   mode 700 (`scripts/`, `lab/`, `probe/`, `data/`).
- Containers,
   every one started by `scripts/run-box.ts` with
   `podman run --rm --init --memory=2g --cpus=2 --pids-limit=512 --ulimit nofile=4096:4096 --network=none`:
  - `docker.io/library/rust:slim`,
     digest `sha256:a2de23e559fd8afd260d22beb00f3987073ea0dcc2ba2646cccdaeda6a62a095`
     (Debian 13.6, image `cargo 1.98.1`).
  - `localhost/hashvet-rusttest:1`,
     digest `sha256:d06f70a8120a32de15a5e08634d84c7190a6e770a0895d6873b04cbf32f9213d`:
     `rust:slim` plus Debian `valgrind` 3.24.0,
     `gcc-aarch64-linux-gnu` 14.2.0,
     `libc6-dev-arm64-cross`,
     and `git`
     (`containers/rusttest/Containerfile`).
  - `localhost/hashvet-rusttest:2`,
     digest `sha256:efe5b287e1875e44129878945b0a2cbc630bec17eec0b9b063fbf5e37822c3bf`:
     `rusttest:1` plus Debian `g++`,
     for libFuzzer (`containers/rusttest2/Containerfile`).
  - `localhost/hashvet-qemu:44`,
     digest `sha256:34174d1c6644bb4287eaeaf57f635a6dad37cdbbe3dc585a2ad5b7e0d3f4baaa`:
     Fedora 44 with `qemu-user-static` 10.2.2 for x86_64 CPU models and aarch64.
- Toolchains,
   mounted read-only at `/toolchain`:
   `nightly-2026-09-12` (`rustc 1.100.0-nightly (0fc141305 2026-09-11)`) for the lab, the probe, and aarch64 suites;
   `nightly-2026-09-16` (`215a8af4b`) with its Miri sysroot for Miri suites.
- Mounts:
   the scratch root read-write at `/w`,
   the Cargo registry read-only,
   one finalist clone at `/src` for upstream suites,
   no home directory,
   no credentials,
   no repository mount.
- Logs:
   `data/logs/<id>.log`,
   with one JSON line per execution in `data/logs/executions.jsonl`
   (command, image digest, bounds, exit status, elapsed seconds).

### Identity, provenance, and license

Each crates.io archive was downloaded,
 its SHA-256 compared with the registry checksum in the lab lockfile,
 its `.cargo_vcs_info.json` commit fetched in the clone,
 and every archived file compared byte for byte with that commit
 (`scripts/provenance.ts`, output `data/provenance.txt`).

- `xxhash-rust` 0.8.18:
   archive SHA-256 `aee1b19627c7c60102ab80d3a9cbe18de90bfe03bfa6c3715447681f0e8c8af6`;
   commit `f93abc7ce036` (2026-07-21);
   clone `~/temp/agent/xxhash-rust-2026-09-17` from `DoumanAsh/xxhash-rust`;
   18 files identical,
   none different.
  License `BSL-1.0` (`LICENSE:1`, "Boost Software License - Version 1.0"):
   pass for HC6.
- `twox-hash` 2.1.4:
   `5283634e518fe9e82c7b20520bb4bc209009fd16c82077c802f8111ecbb0117a`;
   commit `6f866bffe739` (2026-08-27);
   clone `~/temp/agent/twox-hash-2026-09-17` from `shepmaster/twox-hash`;
   22 files identical.
  License `MIT` (`LICENSE.txt`):
   pass.
- `hashcrew` 0.3.0:
   `f73782af6df9e45939f4e6206b646f3cbab68c2cc5c0c375dab045800cafb018`;
   commit `ff8860c72e3a` (2026-09-15), path `hashcrew/`;
   clone `~/temp/agent/hashcrew-2026-09-17` from `fast/hashcrew`;
   20 files identical;
   `LICENSE` and `README.md` come from the workspace root
   (`hashcrew/LICENSE` is a symlink, mode 120000)
   and are identical to the root files at that commit.
  License `Apache-2.0`,
   with incorporated xxHash code under BSD-2-Clause and `twox-hash` code under MIT
   (`src/xxhash/kernel/x86.rs:14-19`;
   notices in `LICENSE:233-237` and `:286-288`):
   pass;
   a binary distribution carries all three notices.
- `highway` 1.3.0:
   `9040319a6910b901d5d49cbada4a99db52836a1b63228a05f7e2b7f8feef89b1`;
   commit `680018dd5219` (2025-01-11);
   clone `~/temp/agent/highway-rs-2026-09-17` from `nickbabcock/highway-rs`,
   checked out at that commit because later commits are unreleased;
   19 files identical.
  License `MIT` (`Cargo.toml`);
   the archive ships no license text (`Cargo.toml.orig:11`, `include = ["src/**/*.rs", "benches"]`),
   so the notice must be taken from the repository:
   pass.
- `autobahn-hash` 0.1.0:
   commit `f35d18565b99` (2023-06-05);
   11 files identical.

No finalist has a build script,
 C or assembly source,
 generated code,
 or a runtime dependency in the feature sets meow would use
 (`probe/Cargo.lock` lists no `dependencies` for any of the four).
None publishes from a release workflow,
 so provenance rests on the commit mapping and byte comparison,
 which pass for HC7.

### Hard-gate outcomes for the finalists

#### HC1 hardware acceleration

Every finalist passes under R2 and none under R1.

- `xxhash-rust`:
   compile-time selection among AVX-512, AVX2, SSE2, NEON, and scalar kernels
   (`src/xxh3.rs:16-28`, `:43-55`);
   a default x86_64 build uses SSE2,
   and AVX2 or AVX-512 needs `-Ctarget-cpu` or `-Ctarget-feature`.
  Upstream issue #48 "Dynamic CPU feature detection" is open since 2024-09-21.
- `twox-hash`:
   run-time AVX2 then SSE2 detection on x86_64 and NEON detection on aarch64
   with the `std` feature (`src/xxhash3/large.rs:103-120`).
- `hashcrew`:
   compile-time selection when the target enables AVX2 or NEON,
   otherwise run-time detection cached in a `OnceLock`
   (`src/xxhash/kernel/mod.rs:98-132`).
- `highway`:
   compile-time AVX2 or SSE4.1 when enabled,
   otherwise run-time detection with the `std` feature,
   then the portable path on x86_64 (`src/builder.rs:147-183`);
   NEON unconditionally on aarch64 (`src/builder.rs:185-197`).

#### HC2 width

Pass for all four:
 `xxh3_128` returns `u128` (`xxhash-rust` `src/xxh3.rs:1620`, `hashcrew` `src/xxhash/xxh3.rs:190`),
 `XxHash3_128::oneshot` returns `u128` (`twox-hash` `src/xxhash3_128.rs:35`),
 and `hash128` returns `[u64; 2]` (`highway` `src/traits.rs:12`),
 which meow must serialize in a fixed word and byte order.

#### HC3 stability

- XXH3-128 (three finalists):
   'Starting from v0.8.0, it's also labelled "stable",
   meaning that any future version will also generate the same hash value'
   (`xxhash.h:1094-1095` at xxHash 0.8.3, saved as `data/xxhash.h`).
  `twox-hash` README "Portability":
   "The output does not depend on the platform ...
   The Rust implementation is verified against the reference C implementation"
   (`README.md:104-110`).
  `hashcrew` README "Portability":
   "Raw and streaming digests are stable across platforms for identical byte streams" (`README.md:194`).
  `xxhash-rust` states no separate promise;
   its tests compare with the C library.
- HighwayHash-128:
   "we have declared all (64/128/256 bit) variants of HighwayHash frozen, i.e. unchanging forever"
   (`google/highwayhash` README "Versioning and stability", saved as `data/highwayhash-readme.md:146-147`);
   `highway` README:
   "generate consistent 64, 128, and 256bit hashes across all hardware" (`README.md:11`).
- Local equality:
   the probe `meowprobe corpus` (`probe/src/main.rs:53-73`) hashes 1,106 inputs
   (every length 0 to 1,100, plus 4,095, 4,096, 4,097, 65,536, and 1,048,583 bytes)
   one-shot and streamed in random 1 to 777 byte chunks,
   and prints a digest of all one-shot outputs.
  XXH3-128 printed `ac8e3fece2937b39a5de424207c9cedf` for all three crates,
   and HighwayHash-128 printed `460c0924896da339c9e57da0ebf75500`,
   in all of these runs,
   each with `stream_mismatches=0`
   (`scripts/probe-matrix.ts`, logs `probe-*.log`):
  - x86_64 glibc native;
  - x86_64 musl native at the baseline, `x86-64-v3`, and `x86-64-v4` builds;
  - QEMU CPU models `qemu64` (SSE2 only), `Nehalem` (SSE4.2, no AVX2), and `Haswell` (AVX2);
  - aarch64 musl under QEMU with CPU models `max` and `cortex-a53`.
  The empty-input value `99aa06d3014798d86001c324468d497f` equals the upstream vector
   asserted in `hashcrew` `tests-integration/tests/xxhash.rs:77`.

Pass for all four.

#### HC4 build

The probe crate depends on all four finalists
 and was built with `nightly-2026-09-12` in `rust:slim`
 by `cargo build --release --offline --locked --jobs 4 --target <triple>`:

- `x86_64-unknown-linux-gnu`:
   exit 0 (`hc4-x86_64-gnu`).
- `x86_64-unknown-linux-musl`:
   exit 0,
   also with `RUSTFLAGS=-Ctarget-cpu=x86-64-v3` and `x86-64-v4`.
- `aarch64-unknown-linux-musl` linked by `rust-lld`
   (`CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER` set to
   `/toolchain/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld`):
   exit 0,
   and the static binary ran under QEMU.
- `aarch64-unknown-linux-gnu`:
   the four crates compiled to rlibs with rustc alone (exit 0);
   the upstream aarch64 suites linked glibc test binaries with `aarch64-linux-gnu-gcc` as linker only.

Deviation:
 the repository's custom aarch64 musl static-pie target with `-Z build-std` was not built,
 because its sysroot objects are not present in this environment.
Evidence it cannot change HC4:
 none of the four has a build script,
 C,
 assembly,
 or `#[link]`,
 so the custom target differs only in link mode and relocation model,
 which Rust source without link directives does not observe.

Pass for all four.

`autobahn-hash` 0.1.0 fails HC4:
 `cargo build --release --offline --locked --jobs 4 --bins --features autobahn` in the lab exited 101 with
 `error[E0599]: no method named cast found for struct Simd<T, N>` at `autobahn-hash-0.1.0/src/lib.rs:87`
 (`build-baseline-autobahn.log`);
 rustc's help says "trait `SimdUint` which provides `cast` is implemented but not in scope",
 so the crate no longer compiles against current nightly `portable_simd`.
It exits before finalist validation,
 and its source has not changed since 2023-06-06.

#### HC5 CPU capabilities

Measured:
 the `x86-64-v3` musl probe run under QEMU `qemu64` died with signal 4
 ("uncaught target signal 4 (Illegal instruction)", exit 132),
 while the baseline build ran on the same model.
So any build that raises `-Ctarget-cpu` needs a startup feature check in meow before any hashing,
 whichever finalist it uses.

- `xxhash-rust`:
   at the default target features,
   SSE2 on x86_64 and NEON on aarch64 are part of the target baseline,
   so no CPU can lack them;
   AVX2 speed requires a raised build and therefore meow's own startup check and diagnostic.
  Pass.
- `twox-hash`,
   `hashcrew`,
   `highway`:
   a default build selects AVX2 at run time and falls back without crashing,
   as the `qemu64` and `Nehalem` runs show.
  Pass.

#### HC6, HC7, HC9

Pass for all four ("Identity, provenance, and license").
None is `gxhash` or derived from it.

#### HC8 one-byte collision gate

Harness:
 `lab/src/bin/onebyte.rs`,
 every single-byte variant of a 4096-byte base
 (1,044,480 keys),
 sorted and counted for full 128-bit, low 64-bit, and high 64-bit equality,
 seeds 0, 1, and 987654321,
 zero base and xorshift random base.
Command:
 `BASE=<zero|random> SEEDS=0,1,987654321 /w/target-gxref/release/onebyte 4096`
 in `rust:slim`
 (build `RUSTFLAGS=-Ctarget-feature=+aes`, feature `gxref`).

Positive control first:
 `gxhash` 3.5.0 `gxhash128` produced 2 full-width collisions on the zero base
 (positions 755 and 1785, 363 and 2923)
 and 1 on the random base (positions 354 and 877)
 under every seed,
 matching `gxhash-owned.md`, "Quality and collision resistance".

Results,
 exit 0,
 `onebyte-zero-4096.log` and `onebyte-random-4096.log`:
 0 full-width,
 0 low-half,
 and 0 high-half collisions for
 `xxhash-rust`,
 `twox-hash`,
 `hashcrew`,
 and `highway`
 under all six base and seed combinations,
 and likewise for the controls BLAKE3, SHA-256, AES-CMAC, and the scalar MuseAir.

Pass for all four.
The three XXH3 crates produce identical outputs,
 so their HC8 result is one algorithm result reached three times.

#### HC10 streaming

Harness:
 `lab/src/bin/streamcheck.rs` compares each streaming API with its one-shot function
 over 612 inputs
 (every length 0 to 600, and lengths around 1 KiB, 4 KiB, 64 KiB, 1 MiB, and 5 MiB)
 with 14 chunk plans
 (fixed 1, 7, 16, 32, 63, 64, 65, 240, 241, 256, 1,024, 4,096, and 65,536 bytes, and random sizes;
 inputs over 100 KiB use the first four plans):
 8,548 comparisons per function.

Positive control first:
 `STREAMCHECK_CONTROL=drop-last` withholds the final byte
 and reported 68,272 mismatches,
 exactly 8 functions × (8,548 − 14 empty-input comparisons),
 exit 1 (`streamcheck-control-baseline.log`).

Results:
 0 mismatches for all four finalists and the controls
 in both the baseline and `x86-64-v3` builds
 (`streamcheck2-baseline.log`, `streamcheck2-v3.log`).
The consumer probe repeats the check on every platform in HC3.

Bounded memory,
 from source:
 each streaming state is a fixed-size value
 (`xxhash-rust` `Xxh3`, `src/xxh3.rs:1117-1125`;
 `hashcrew` `StreamState`, `src/xxhash/xxh3.rs:701-709`;
 `twox-hash` a fixed-size buffer plus the 192-byte default secret copied to the heap once,
 `src/xxhash3/streaming.rs:386-391`;
 `highway` a fixed-size union of state words, `src/builder.rs:144-198`),
 and peak resident memory over multi-GB files is reported in "Benchmarks".

Pass for all four.

### Hard-gate confirmed

`xxhash-rust` 0.8.18,
 `twox-hash` 2.1.4,
 `hashcrew` 0.3.0,
 and `highway` 1.3.0
 are hard-gate confirmed and become finalists.

## Finalist validation

Every finalist received the same validation:
 its upstream CI-equivalent jobs and relevant non-default suites,
 aarch64 suites under QEMU,
 the local reference, collision, streaming, and cross-dispatch checks,
 the consumer probe on every reachable target,
 benchmarks on the same workloads,
 and the same source, auditability, and maintenance audits.

### Execution manifest

Shared by every execution in this audit:

- Pinned inputs:
   clone commits and archive checksums in "Identity, provenance, and license";
   lockfiles generated on 2026-09-17 with `cargo +nightly-2026-09-12 generate-lockfile`
   where upstream ships none (`xxhash-rust`, `twox-hash`, `highway-rs`, `highway-rs/fuzz`),
   upstream `Cargo.lock` for `hashcrew`.
- Dependency-fetch phase:
   `cargo +nightly-2026-09-12 fetch` on the host,
   the one network phase,
   run before any build;
   it downloads crate archives into `~/.cargo/registry` and runs nothing
   (`scripts/fetch-upstream.ts`).
  Deviation:
   it ran on the host rather than in a container,
   because only the host's nightly Cargo binary is allowed to reach `index.crates.io`;
   Cargo `fetch` executes no package code.
- Container images:
   "Shared evidence fields".
  Deviation:
   `rusttest:1` and `rusttest:2` were built with network access to Debian mirrors
   (`podman build --memory=2g --cpu-quota=200000`),
   installing only the Debian packages named there.
- Bounds:
   2 GiB memory,
   2 CPUs,
   512 processes,
   4,096 file descriptors,
   no network,
   container removed on exit.
  Wall-clock ceiling:
   none beyond the container;
   libFuzzer runs carry `-max_total_time`.
- Mounts and policy:
   no credentials,
   no home directory,
   no repository mount,
   scratch root read-write,
   Cargo registry and toolchains read-only.
  Deviation:
   the scratch root is read-write because build outputs,
   Cargo's extracted sources (`/w/cargo-home`),
   and logs live there;
   the clone mounted at `/src` is also read-write,
   because Cargo may rewrite the generated `Cargo.lock`;
   `git -C <clone> status --short` after the suites prints nothing for any of the four clones,
   and the generated lockfiles are ignored by each upstream `.gitignore`.
- Expected reads:
   the clone,
   the registry,
   the toolchain.
  Expected writes:
   `/w/targets/<repo>-<toolchain>`,
   `/w/cargo-home`,
   `/tmp` inside the container.
  Expected subprocesses:
   `cargo`, `rustc`, build scripts, test binaries, `valgrind`, `qemu-aarch64-static`, `cc`/`c++` from build scripts.
  Network endpoints:
   none.
- Success:
   exit 0 with every `test result: ok`.
  Failure:
   any nonzero exit,
   failed test,
   or valgrind error,
   which stops and triggers diagnosis.
  Cleanup:
   `--rm`;
   scratch outputs stay for audit.
- Stop condition for undeclared behavior:
   none observed;
   no container attempted network access (`--network=none` would have failed it),
   and every build script that ran is on the lists that follow.

Build scripts and proc macros reachable from each upstream workspace with all features
 (`scripts/build-surface.ts`, `data/build-surface.txt`),
 and the ones that compiled native code in the suites (`targets/*/build/`):

- `xxhash-rust`:
   `getrandom`, `libc` (rustc cfg probes)
   and `xxhash-c-sys` 0.8.7,
   which compiles vendored xxHash 0.8.3 `xxhash.c` with `cc` for the reference comparison.
- `twox-hash`:
   30 build scripts or proc macros,
   mostly rustc cfg probes of `serde`, `proc-macro2`, `crossbeam`, `rayon-core`, `zerocopy`, and `rustix`;
   native code from the workspace member `xx_hash-sys`,
   which compiles the xxHash submodule `xxhash.c` as scalar, SSE2, AVX2, and on aarch64 NEON variants
   (`xx_hash-sys/build.rs:13-40`),
   and from `alloca` 0.4.0 (`build.rs:2-12`, one small C file for a benchmark dependency).
- `hashcrew`:
   17,
   mostly cfg probes;
   `crc32c` and `crc32fast` build scripts only query `rustc --version`;
   no native code in the suites run.
- `highway-rs`:
   27,
   cfg probes and `no-panic`,
   whose build script only queries `rustc --version`;
   no native code in the suites run.
- `highway-rs/fuzz`:
   `libfuzzer-sys` 0.4.13 compiles bundled libFuzzer C++ (`build.rs`, `CUSTOM_LIBFUZZER_PATH` unset),
   and the fuzz crate compiles the `google/highwayhash` submodule `c/highwayhash.c` at `faca2cb`.
  `fuzz-control/` is a copy with one changed line for the positive control,
   built against the clone mounted read-only.
- Local reference check `refcheck/`:
   `xxhash-c-sys` 0.8.7 and `c/highwayhash.c` copied from the same submodule commit.

The wasm, Windows, and `winapi` build scripts in these graphs are cfg-gated to other targets
 and did not run.

### Upstream CI inventory and suites run

Suites ran through `scripts/upstream-suites.ts` in `localhost/hashvet-rusttest:1`,
 with the clone at `/src`,
 image `cargo 1.98.1` unless a toolchain is named,
 and aarch64 test binaries linked by `aarch64-linux-gnu-gcc` and run by `qemu-aarch64-static -L /usr/aarch64-linux-gnu`.
Totals come from every `test result` line in each log (`scripts/suite-totals.ts`, `data/suite-totals.txt`).
Every suite exited 0 with no failed test.

#### `xxhash-rust` (`.github/workflows/rust.yml`)

Upstream jobs:
 `min-rust-check`,
 `full-test` (check, test, three valgrind variants),
 `wasm-platform-test`,
 `cross-platform-test` (arm, i586, powerpc, aarch64 musl).

- xr1 `cargo test --offline --features xxh32,const_xxh32,xxh64,const_xxh64,xxh3,const_xxh3`:
   12 passed.
- xr2 the same with `--release` and `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER`
   `valgrind --leak-check=full --error-exitcode=1 --suppressions=valgrind.supp`:
   12 passed,
   5 valgrind runs with 0 errors.
- xr3 as xr2 with `RUSTFLAGS=-Ctarget-feature=+avx2`:
   12 passed,
   0 valgrind errors.
- xr4 as xr2 with `RUSTFLAGS=-Ctarget-feature=-sse2`:
   12 passed,
   0 valgrind errors;
   rustc warned "target feature `sse2` must be enabled to ensure that the ABI of the current target can be
   implemented correctly";
   the flag is upstream's way to reach the scalar kernel (`src/xxh3.rs:28`),
   and this audit did not confirm which kernel that build selected.
- xr5 `cargo test --offline --release --target aarch64-unknown-linux-gnu` with `nightly-2026-09-12`,
   NEON kernel under QEMU:
   12 passed.

The XXH3 test compares one-shot and single-update streaming `xxh3_128` and `xxh3_128_with_seed`
 with the C library for random inputs of every length 0 to 4,095
 (`tests/assert_correctness.rs:189-260`).

#### `twox-hash` (`.github/workflows/ci.yml`)

Upstream jobs:
 `library` (unit and property tests on stable, beta, nightly, MSRV, macOS, Windows),
 `miri` (x86_64, i686, s390x),
 `lints`,
 `no-std`,
 `features`,
 `minimal-versions`.

- tw1 `cargo test --offline --all-features`:
   67 passed,
   1 ignored.
- tw2 `cargo test --offline -p comparison --all-features`:
   34 passed;
   proptests compare `XxHash3_128` one-shot, seeded, custom-secret, and chunked streaming output
   with the C library built as scalar, SSE2, AVX2, and native variants
   (`xx_hash-sys/build.rs:18-58`; `comparison/src/lib.rs:366-545`,
   data up to 32 KiB, chunk vectors up to 100 × 100 bytes).
- tw3 `cargo miri test --offline --all-features` with `nightly-2026-09-16`
   and `MIRIFLAGS=--cfg _internal_xxhash3_force_scalar`:
   67 passed,
   1 ignored.
- tw4 `cargo test --offline --all-features --target aarch64-unknown-linux-gnu` under QEMU:
   67 passed,
   1 ignored.
- tw5 `cargo test --offline -p comparison --all-features --target aarch64-unknown-linux-gnu` under QEMU,
   against the C scalar and NEON builds:
   34 passed.

The ignored test is `xxhash32::test::length_overflows_32bit` in tw1, tw3, and tw4,
 outside XXH3.

#### `hashcrew` (`.github/workflows/ci.yml`, `xtask/src/main.rs`)

Upstream jobs:
 `check` (`cargo x lint`, `cargo x check`),
 `test` (`cargo x build`, `cargo x test` on Linux, macOS, Windows, stable and 1.89.0),
 `miri` (`cargo x miri`),
 `target` (check-only builds for x86_64 AVX2, aarch64 NEON, wasm32, thumbv7em).
`cargo x test` runs three commands (`xtask/src/main.rs:201-213`),
 run here as hc1 to hc3.

- hc1 `cargo test --offline --workspace --all-features`:
   73 passed.
- hc2 `cargo test --offline --package hashcrew --no-default-features`
   `--features cityhash,crc,fnv,md5,murmur,xxhash`:
   30 passed.
- hc3 `cargo test --offline --package tests-integration --release`:
   43 passed;
   XXH3-128 one-shot and seeded outputs compared with `xxhash-rust` as the reference
   (`tests-integration/tests/xxhash.rs:38`, `:81-107`, `:263-311`),
   for lengths around every stripe and block boundary to 16,384 bytes and seeds 0, 1, `0x0123456789abcdef`, `u64::MAX`,
   streaming compared with one-shot for many chunk sizes, random partitions, and every two-way partition
   (`:314-450`),
   plus the official empty-input vectors;
   no direct C comparison.
- hc4 `cargo miri test --offline --package hashcrew --lib --no-default-features --release --features ...`
   with `MIRIFLAGS=-Zmiri-strict-provenance` on `nightly-2026-09-16` (`cargo x miri`):
   19 passed.
- hc5 hc3 with `--target aarch64-unknown-linux-gnu` under QEMU:
   43 passed.

#### `highway-rs` (`.github/workflows/main.yml`)

Upstream jobs:
 `test` (build, docs, tests, no-std tests, release tests, `no_panic` example,
 bench compile, Miri, fuzz compile;
 on pinned 1.59, stable, 32-bit, big-endian, beta, nightly, macOS, Windows, aarch64 through `cross`),
 `wasm`,
 `instructions` (four `-sse4.1`/`+sse4.1` × `-avx2`/`+avx2` builds, tests and no-std tests).

- hw1 `cargo test --offline`:
   47 passed.
- hw2 `cargo test --offline --no-default-features`:
   45 passed.
- hw3 `cargo test --offline --release`:
   47 passed.
- hw4 `cargo build --offline --release --example no_panic`:
   exit 0;
   `no_panic` makes the release build fail to link if `append`, `checkpoint`, or `finalize64` can panic
   (`examples/no_panic.rs:6-12`);
   `finalize128` is not covered.
- hw5 to hw8 `cargo test --offline` with `RUSTFLAGS=-C target-feature=` each of
   `-sse4.1,-avx2`, `-sse4.1,+avx2`, `+sse4.1,+avx2`, `+sse4.1,-avx2`:
   47 passed each.
- hw9 `cargo miri test --offline` on `nightly-2026-09-16`:
   40 passed,
   7 ignored by upstream `cfg_attr(miri, ignore)`
   (`src/x86/v2x64u.rs` 6, `src/x86/sse.rs` 1).
- hw10 `cargo test --offline --release --target aarch64-unknown-linux-gnu` under QEMU:
   36 passed
   (x86-only tests are compiled out).

`tests/hash.rs:451-456` asserts reference 64, 128, and 256-bit vectors for input lengths 0 to 63;
 longer inputs are covered by cross-implementation equality (`tests/properties.rs`, quickcheck)
 and by the fuzz target,
 which compares only the 64-bit output with C.

### Omitted upstream jobs

- Toolchain matrix entries (`xxhash-rust` MSRV 1.64 check,
   `twox-hash` MSRV 1.81 and minimal versions,
   `hashcrew` 1.89.0,
   `highway-rs` pinned 1.59, beta, nightly builds):
   purpose,
   older or other compilers.
  Cannot affect the consumed surface:
   meow builds with the repository's floating nightly,
   and the suites ran on stable 1.98.1 and nightly 2026-09-12 and 2026-09-16.
- Non-target platforms
   (32-bit x86, arm, powerpc, s390x, big-endian, thumb, wasm, macOS, Windows):
   not release-blocking targets (user, 2026-09-16);
   their code paths are cfg-gated away from x86_64 and aarch64 Linux builds.
- Lints and docs
   (`twox-hash` `lints`, `hashcrew` `cargo x lint` with `hawkeye`, `taplo`, `typos`, `rustfmt`, `clippy`,
   `highway-rs` `cargo doc`):
   style and documentation,
   no runtime behavior.
- Compile-only matrices
   (`twox-hash` `features` powerset `cargo check`,
   `hashcrew` `cargo x check` and `target`,
   `highway-rs` bench `--no-run`):
   the feature sets meow would use were compiled and run in the lab, probe, and suites,
   and `hashcrew`'s x86_64 AVX2 and aarch64 NEON checks are subsumed by hc3 and hc5, which run tests.
- `twox-hash` `no-std` on `thumbv6m-none-eabi` and `hashcrew` bare metal:
   not targets;
   hc2 and hw2 ran the no-std tests on x86_64.
- `xxhash-rust` `cross-platform-test` for `aarch64-unknown-linux-musl`:
   replaced by xr5 on aarch64 glibc under QEMU plus the probe's aarch64 musl run;
   the crate's kernels depend on `target_feature`, not on the C library.

### Source quality and human auditability

Measured with `scripts/audit-size.ts` (`data/audit-size.txt`)
 over each crates.io archive:
 non-blank, non-comment Rust lines outside `#[cfg(test)]` modules,
 and `unsafe` occurrences,
 for the whole `src/` and for the modules meow's calls reach.

- `xxhash-rust` 0.8.18:
   used path 4 files,
   1,413 code lines,
   29 `unsafe`
   (whole crate 11 files, 2,327 lines, 31 `unsafe`);
   largest file `src/xxh3.rs` at 1,649 lines.
  Lint policy:
   `#![warn(missing_docs)]` only (`src/lib.rs:79`);
   no `// SAFETY` comments.
  Open issue #29 (2023-08-24, 15 comments):
   private functions dereference raw pointers without being `unsafe fn`;
   the maintainer answered with coverage, valgrind, and Miri runs, and the issue stays open.
  Control flow:
   compile-time `cfg` selection,
   no run-time dispatch.
- `twox-hash` 2.1.4:
   used path 10 files,
   1,662 lines,
   66 `unsafe`
   (whole crate 13 files, 2,621 lines, 72);
   largest used file `src/xxhash3.rs` at 425 lines,
   one file per SIMD kernel under `src/xxhash3/large/`.
  Lint policy:
   `#![deny(rust_2018_idioms, missing_docs, unnameable_types)]` (`src/lib.rs:2-4`),
   denied Clippy lints in `Cargo.toml` `[lints.clippy]`,
   `check-cfg` for the four force-kernel cfgs;
   52 `// Safety` comments.
- `hashcrew` 0.3.0:
   used path 8 files,
   1,440 lines,
   32 `unsafe`
   (whole crate 19 files, 4,286 lines, 72, because it also ships CityHash, CRC, FNV, MD5, and Murmur families
   behind opt-in features);
   largest used file `src/xxhash/xxh3.rs` at 1,429 lines.
  Lint policy,
   workspace `Cargo.toml`:
   `unsafe_op_in_unsafe_fn = "forbid"`,
   `clippy::undocumented_unsafe_blocks = "deny"`,
   `clippy::missing_safety_doc = "deny"`,
   `missing_debug_implementations = "deny"`;
   45 `// SAFETY` comments;
   an explicit `Backend` enum with `is_available` checks (`src/xxhash/kernel/mod.rs:41-96`).
- `highway` 1.3.0:
   16 files,
   2,646 lines,
   196 `unsafe`
   (the wasm module is compiled out on Linux but counted);
   largest file `src/builder.rs` at 412 lines.
  Lint policy:
   `#![deny(unsafe_code)]` with `#[allow(unsafe_code)]` on six modules (`src/lib.rs:163-164`);
   no `// SAFETY` comments;
   explicit per-instruction-set types (`PortableHash`, `SseHash`, `AvxHash`, `NeonHash`)
   whose x86 SIMD constructors return `Option` (`src/x86/avx.rs:94`).

Shared:
 no runtime dependencies,
 no same-author dependencies,
 no filesystem, network, process, or credential access,
 no generated code,
 no event bus or plugin boundary.
The only high-trust boundary is `unsafe` SIMD code on untrusted input lengths,
 which valgrind (`xxhash-rust`), Miri on the scalar or portable paths (`twox-hash`, `hashcrew`, `highway`),
 and the local checks exercise.
None has error paths on the hashing calls meow would use;
 `hashcrew` and `twox-hash` return errors only for undersized custom secrets.

### Maintenance audit

Sample rule:
 issues and pull requests created or updated since 2025-09-17,
 all when at most 20,
 otherwise the 10 most recently updated
 (`scripts/gh-maint.ts`, `data/maint/*.json`,
 plus `gh pr list --json mergedBy,reviews` and `gh issue list --state open` on 2026-09-17).
Releases from the crates.io API (`scripts/release-history.ts`, `data/release-history.txt`).

- `xxhash-rust` (`DoumanAsh/xxhash-rust`):
  - Issues in window, all 4:
     #53 and #54 (`Xxh3Builder` ignored a seed-derived or custom secret on inputs over 240 bytes,
     reported 2026-07-14 and 2026-07-15),
     first maintainer responses after about 3 hours and 13 minutes,
     4 and 2 maintainer comments,
     closed by the maintainer on 2026-07-15 and 2026-07-21,
     the publish dates of 0.8.17 and 0.8.18;
     #52 (private security contact), answered in 26 minutes and closed;
     #48 (run-time CPU detection), open since 2024-09-21 with 1 maintainer comment.
  - Pull requests in window, all 3:
     external #56, #55, #51,
     merged by the maintainer after 1, 6, and 2 days,
     1 recorded review;
     the maintainer pushes most work directly (122 of 136 contributor commits).
     An older external PR, #47, waited from 2024-09-21 to 2025-09-06.
  - Releases in window:
     0.8.16 (2026-07-01), 0.8.17 (2026-07-15), 0.8.18 (2026-07-21);
     no git tags;
     0.8.18's commit `f93abc7` is the repository head.
  - Concentration:
     one maintainer, 90% of commits.
  - Class:
     responsive single maintainer with fast bug turnaround;
     #53 shows that releases before 0.8.17 returned wrong XXH3 output from a seeded `Xxh3Builder`
     on inputs over 240 bytes,
     a path meow would not call.
- `twox-hash` (`shepmaster/twox-hash`):
  - Issues in window, all 5:
     #123 (hashes differ across 32 and 64-bit through the `Hash` trait),
     answered in about 7 hours and closed by documentation PR #125;
     #120 (`XxHash32` debug overflow panic), answered in about 3.5 hours,
     fixed by #121 and released in 2.1.3;
     #81 (reproducibility question) closed by #125;
     #115 and #114 open feature or showcase threads with maintainer comments.
  - 10 most recently updated pull requests:
     4 maintainer-authored (#121, #122, #125, #126), self-merged within 2 days, no reviews;
     external #116 merged after about 5 months with 4 maintainer comments;
     external #124 closed unmerged after 2 maintainer comments;
     external #127 open 10 days without response;
     stale external #100 and #99 from 2024 closed unmerged on 2026-09-07;
     collaborator draft #52 from 2020 closed on 2026-08-27.
  - Releases in window:
     2.1.3 (2026-07-19), 2.1.4 (2026-08-27);
     tag `v2.1.4` points at the release commit,
     with no commits after it.
  - Older open bugs #92 and #79 concern the 1.x `xxh3` module,
     which 2.0.0 replaced ("a complete rewrite of the crate", `CHANGELOG.md:69`).
  - Concentration:
     one maintainer, 400 of 428 commits.
  - Class:
     active releases,
     responsive on bugs,
     slower on external feature pull requests.
- `hashcrew` (`fast/hashcrew`):
  - Issues in window, all 1:
     #17, filed and closed by the main author without comment.
  - 10 most recently updated of 27 pull requests (#19 to #28):
     all authored by `tisonkun`;
     9 merged by the author after 7 minutes to 6 days,
     #25 closed unmerged;
     no human reviews
     (one automated Copilot review on #22).
  - Releases in window:
     0.1.0 to 0.3.0 between 2026-09-02 and 2026-09-15,
     tags signed (`v0.3.0` tag and commit `ff8860c` verified by GitHub);
     `RELEASE.md` documents the process.
  - Concentration:
     2 contributors, 24 and 10 commits;
     no external issue or pull request yet.
  - Class:
     low-signal tracker activity in a project two weeks old,
     evaluated through source, tests, release, and signing evidence.
- `highway` (`nickbabcock/highway-rs`):
  - Issues in window, all 1:
     #103 (naming ambiguity), answered in about 4 hours and closed.
  - Pull requests in window, all 9:
     4 maintainer-authored and self-merged without review (#101, #104, #105, #106);
     maintainer draft #102 (loop unrolling) open since 2026-03-23;
     3 Dependabot updates open since 2026-07-23;
     Dependabot #86 from 2024 closed by the maintainer.
  - Releases:
     none since 1.3.0 on 2025-01-11;
     16 commits on the default branch after tag `v1.3.0`
     (`gh api repos/nickbabcock/highway-rs/compare/v1.3.0...HEAD`),
     including wasm64 support,
     so maintenance merges have not reached a release in 20 months.
  - Concentration:
     one maintainer, 348 of 362 commits.
  - Class:
     maintained,
     slow release cadence;
     the frozen algorithm reduces the need for releases.

None of the four has an open security report or abandonment statement in the sampled trackers.

### Replacement parity overlay

- Incumbent paths consumed:
   meow is unimplemented,
   so the planned consumer is one-shot `gxhash128(bytes, seed)` for file contents and key material
   (design doc, "Cache").
  The music player's `gxhash64` calls belong to another package and are outside this decision.
- Incumbent defects:
   one-byte full-width collisions (reproduced in HC8),
   `Hasher` output that depends on write chunking (upstream #127),
   and no stable streaming API for multi-GB outputs.
- Each finalist against those defects:
   0 collisions in HC8,
   streaming equal to one-shot in HC10,
   and a documented or specified freeze in HC3.
- Transitive dependencies:
   `gxhash` 3.5.0 has one,
   the `rustversion` proc macro (`Cargo.toml` `[dependencies.rustversion]`);
   no finalist has any in the feature sets meow would use.
- Native boundary:
   `gxhash` requires AES and SSE2 or NEON at compile time and stops the build otherwise
   (`src/gxhash/platform/x86.rs:2`, `src/gxhash/platform/arm.rs:2`, `compile_error!`);
   `xxhash-rust` is also compile-time;
   `twox-hash`, `hashcrew`, and `highway` dispatch at run time ("HC5 CPU capabilities").
- Maintenance:
   the finalists' samples are in "Maintenance audit";
   `gxhash`'s trackers are reviewed in the gxhash study,
   "Upstream issues and pull requests",
   and were not resampled because HC9 excludes it.
- Consumer boundary:
   exercised through `probe/` on every reachable target and through the lab binaries on multi-GB files.
- Keeping the incumbent:
   excluded by HC9.

### Native overlay

- Source-to-artifact mapping:
   byte-identical archive and commit comparison ("Identity, provenance, and license").
- Compiler and linker flags:
   the lab and probe used `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`,
   with no `RUSTFLAGS` for baseline builds and `-Ctarget-cpu=x86-64-v3` or `x86-64-v4` for raised builds;
   aarch64 musl linked by `rust-lld`.
- Imported and exported host functions:
   none;
   the crates call only `core::arch` intrinsics and `std::arch` feature detection.
- Build scripts:
   none in any finalist.
- Release workflow and signing:
   none automated;
   `hashcrew` signs tags and commits,
   `highway` tags are unsigned annotated tags,
   `twox-hash` tags are lightweight,
   `xxhash-rust` has no tags.
- Target matrix and reproducible verification:
   "HC3 stability" and "HC4 build".

### Reference equality

`refcheck/` (`src/main.rs`, `build.rs`) links the C references into one binary:
 xxHash 0.8.3 `XXH3_128bits_withSeed` through `xxhash-c-sys` 0.8.7,
 and `HighwayHash128` from `google/highwayhash` `c/highwayhash.c` at `faca2cb`.
For every length 0 to 4,100,
 plus 8,191 to 8,193, 65,535 to 65,537, 100,003, 1,048,583, and 16,777,219 bytes of random data,
 it compares each finalist's one-shot output
 and its streaming output over random chunks of 1 to 70,000 bytes
 with C,
 under seeds 0 and `0x9E3779B97F4A7C15` for XXH3
 and the all-zero key and a byte-sequence key for HighwayHash:
 65,760 comparisons per run.

- Positive control,
   `REFCHECK_CONTROL=flip` (one input bit flipped on the Rust side only):
   65,760 mismatches,
   exit 1 (`refcheck-control.log`).
- x86_64 baseline build:
   0 mismatches (`refcheck-x86_64-baseline.log`).
- x86_64 `x86-64-v3` build:
   0 mismatches (`refcheck-x86_64-v3.log`).
- aarch64 glibc build under QEMU (`qemu-aarch64-static -L /usr/aarch64-linux-gnu`):
   0 mismatches (`refcheck-aarch64-qemu.log`).

The C code was compiled by `cc` in `localhost/hashvet-rusttest:1`
 (Debian GCC 14 for x86_64, `aarch64-linux-gnu-gcc` 14.2.0 for aarch64),
 only for this check;
 no finalist needs it.

This closes the gap left by upstream `highway` tests,
 whose 128-bit reference vectors stop at 63 bytes,
 and by `hashcrew`,
 whose tests compare with `xxhash-rust` rather than C.

### Backend selection check

`refcheck/src/bin/backend.rs` prints `hashcrew::xxhash::kernel::selected_backend()`:
 `Avx2` in both the baseline and the `x86-64-v3` build on this host
 (`hashcrew-backend-baseline.log`, `hashcrew-backend-v3.log`).
So `hashcrew`'s slow baseline results are not a fallback to SSE2 or scalar code:
 the baseline build reaches its AVX2 kernel through a call per 64-byte stripe
 into a `#[target_feature(enable = "avx2")]` function
 (`src/xxhash/kernel/x86.rs:117`, `:139`),
 which the compiler cannot inline into a loop built without AVX2.

### Additional one-byte keysets

The HC8 harness ran again at key lengths 8, 16, 64, 128, 240, 241, 1,024, and 2,048 bytes,
 zero and random bases,
 seeds 0, 1, and 987654321,
 for the four finalists and `gxhash`
 (`onebyte-lengths-zero.log`, `onebyte-lengths-random.log`, 120 results each).
Every finalist result has 0 full-width, 0 low-half, and 0 high-half collisions.
`gxhash` collided again:
 1 full-width collision at 2,048 bytes on the zero base,
 and 1 each at 1,024 and 2,048 bytes on the random base,
 under every seed.

### Fuzzing and mutation

Search:
 `rg --files-with-matches --ignore-case --hidden` over the four clones
 (vendored C submodules excluded)
 for `cargo-mutants`, `mutants.toml`, `mutagen`, `proptest`, `quickcheck`, `libfuzzer`, `afl`, `honggfuzz`, and `bolero`
 matched only `highway-rs` (`Cargo.toml`, `tests/properties.rs`, `fuzz/`)
 and `twox-hash` (`comparison/Cargo.toml`, `comparison/src/lib.rs`).

- `xxhash-rust`:
   no fuzz target,
   no property tests;
   random-input comparison with C in `tests/assert_correctness.rs`.
- `twox-hash`:
   no Rust fuzz target
   (the vendored xxHash submodule's `fuzz/fuzzer.c` targets the C library);
   `proptest` comparison with C in `comparison/`,
   run as tw2 and tw5.
- `hashcrew`:
   no fuzz target,
   no property-test crate;
   seeded random partitions in `tests-integration`,
   run as hc3 and hc5.
- `highway`:
   `fuzz/fuzz_targets/fuzz_highway.rs` asserts that `PortableHash::hash64` equals the C `HighwayHash64`
   and that the builder, SSE4.1, and AVX2 hashers equal `PortableHash` for 64, 128, and 256-bit output;
   upstream CI only compiles it (`cargo fuzz build fuzz_highway`, nightly job);
   `tests/properties.rs` holds `quickcheck` properties.
- Mutation testing:
   none in any finalist.

The `highway` fuzz target was run here,
 because it is the only fuzz suite and covers dispatch equality:

- Build:
   `cargo build --manifest-path /src/fuzz/Cargo.toml --target x86_64-unknown-linux-gnu --release`
   `--config profile.release.debug="line-tables-only" --bin fuzz_highway --offline --locked --jobs 4`
   with the `RUSTFLAGS` that `cargo fuzz build` sets by default
   (sanitizer coverage, `--cfg fuzzing`, `-Zsanitizer=address`, `-Cdebug-assertions`, `-Ccodegen-units=1`,
   read from `rust-fuzz/cargo-fuzz` `src/project.rs:150-300` at `bf2fc66`),
   in `localhost/hashvet-rusttest:2`;
   exit 0 (`fuzz-highway-build.log`).
- Positive control,
   run after the campaign on a copy of the target whose C call receives a key with one bit flipped
   (`fuzz-control/fuzz_targets/fuzz_highway.rs`, built the same way):
   the assertion failed after 3,086 executions
   ("assertion `left == right` failed" at `fuzz_targets/fuzz_highway.rs:33:9`, exit 77, `fuzzcontrol-run.log`).
- Campaign:
   `fuzz_highway -max_total_time=600 -rss_limit_mb=1536` from an empty corpus:
   23,513,644 executions in 601 seconds,
   coverage 333 edges,
   39 corpus entries,
   inputs up to libFuzzer's default 4,096 bytes,
   0 crashes,
   no artifacts
   (`fuzz-highway-run.log`).

The target compares 128-bit output only across Rust implementations;
 128-bit equality with C is covered by "Reference equality".

## Benchmarks

All benchmarks ran on the x86_64 host inside
 `podman run --rm --init --memory=2g --cpus=2 --pids-limit=512 --ulimit nofile=4096:4096 --network=none`
 in `docker.io/library/rust:slim`,
 with no credentials and no repository mount.
No aarch64 benchmark ran:
 QEMU user mode would measure the emulator,
 and `ssh m1` is powered off ("aarch64 and the m1 question").

Builds of the lab crate (`lab/`, `nightly-2026-09-12`, `opt-level = 3`, fat LTO, one codegen unit):

- baseline:
   no `RUSTFLAGS`,
   so the x86-64 baseline (SSE2) at compile time;
   `twox-hash`, `hashcrew`, and `highway` may still select AVX2 at run time.
- `x86-64-v3`:
   `RUSTFLAGS=-Ctarget-cpu=x86-64-v3` (AVX2, BMI2).

Method:

- Each benchmark is a separate container run;
   five container runs per build and workload,
   baseline build first.
- The run-to-run band is `(max − min) / median` of the five per-run medians
   for one algorithm on one unchanged build.
- A difference between two results counts only when their five-run ranges do not overlap;
   otherwise the report says "within band".
- Inside a run,
   algorithm order rotates every round,
   each measurement warms for a quarter of its time budget,
   and calls go through non-inlined wrappers with `black_box` inputs
   (`lab/src/bin/bench.rs`).
- Host load average during the runs was 1.4 to 2.9 on 16 hardware threads
   (`data/bench/progress-a.txt`),
   from other processes this audit does not control;
   the band includes that noise.

Workloads,
 in the order of the frozen criteria:

- S1 key composition:
  - `fp`:
     2,226 synthetic items of 70 to 242 bytes
     (a random path of the music player's measured path lengths, 46 to 218 bytes, plus 24 bytes of numbers;
     `data/fp-lengths.txt`, `lab/src/bin/bench.rs` `load_fp`).
  - `keymat`:
     222 key-material records of 50 to 50,206 bytes (median 1,349),
     one per package directory,
     concatenating for each tracked file its path, a NUL, its 64-bit size, and a 16-byte content digest
     (`scripts/pack-data.ts`).
- S2 whole files:
  - `files`:
     the contents of all 8,101 git-tracked files at `88cf6c0e4` (152,231,070 bytes),
     one call per file.
  - `files-stream64k`:
     the same files,
     a fresh streaming hasher per file fed 64 KiB slices.
  - `hot16k`:
     one cache-resident 16 KiB buffer.
  - `mem256m-stream1m`:
     one 256 MiB buffer fed in 1 MiB slices,
     the in-memory ceiling for large outputs (supporting S3).
- S3 multi-GB from disk (`lab/src/bin/bigfile.rs`, `scripts/bench-b.ts`):
   reflink copies of `disk.qcow2` (3,966,238,720 bytes)
   and the 662,710,296-byte music player debug binary
   in `~/temp/agent/hashvet-2026-09-17/big/`,
   read in 1 MiB `read` calls and streamed into each hasher,
   five repetitions per build, file, and cache mode,
   with a read-only pass leading every rotation.
  - Cold:
     the container calls `posix_fadvise(POSIX_FADV_DONTNEED)` on the file before reading.
    Positive control first (`scripts/cache-control.ts`, `data/bench/cache-control.txt`):
     host `fincore` showed 0% resident,
     100% after a host `cat`,
     and 0% after the container's `fadvise`,
     for both files.
  - Warm:
     the host reads the file first,
     and `fincore` confirms at least 99.9% residency before every run.
  - Storage:
     btrfs (crc32c checksums, no compression, 588 and 114 extents) on LUKS on the NVMe drive.
  - The container's 2 GiB limit also bounds its page cache,
     so a cold read of the 3.97 GB image runs under memory-cgroup reclaim.
- S4 multi-core,
   same files and modes with 2 threads:
  - `blake3_128_rayon`:
     BLAKE3 `update_rayon`,
     the control's built-in tree mode,
     output identical to single-threaded BLAKE3.
  - `twox_xxh3_128` and `highway_128` with 2 threads:
     a lab-only composition that hashes consecutive 16 MiB chunks in parallel
     and then hashes the ordered chunk digests and lengths;
     its output is a different function from the one-shot hash and would be a meow-defined format.

### Run-to-run band

Measured first on the unchanged baseline build (five container runs),
 then on the `x86-64-v3` build
 (`data/bench/summary-a.txt`, `scripts/bench-a-summary.ts`).
Finalist bands on the baseline build:
 0.2% to 2.6% for `twox-hash`, `xxhash-rust`, and `highway` on every in-memory workload,
 and 0.8% to 8.3% for `hashcrew`,
 whose baseline results have one slow outlier run in `files`, `files-stream64k`, `keymat`, and `mem256m-stream1m`.
On the `x86-64-v3` build the finalist bands are 0.3% to 4.5%.
Multi-GB results have wider bands,
 given with each result.

Values are the median of five runs in GiB/s with the [minimum, maximum] of the per-run medians.

### S2 whole files, KiB to tens of MiB (weight 5)

- `files`, one call per tracked file:
  - baseline:
     `twox-hash` 49.3 [49.0, 50.3],
     `xxhash-rust` 29.1 [29.0, 29.3],
     `hashcrew` 16.0 [15.9, 17.1],
     `highway` 12.4 [12.4, 12.5];
     every adjacent pair resolved beyond band.
  - `x86-64-v3`:
     `xxhash-rust` 49.3 [48.8, 50.0],
     `hashcrew` 47.2 [46.9, 48.2],
     `twox-hash` 46.9 [45.8, 48.0],
     `highway` 12.5 [12.5, 12.6];
     `xxhash-rust` over `hashcrew` resolved (1.04×),
     `hashcrew` and `twox-hash` within band.
- `files-stream64k`, a streaming hasher per file:
  - baseline:
     `twox-hash` 43.4 [43.0, 43.7],
     `xxhash-rust` 26.2 [25.8, 26.4],
     `hashcrew` 16.7 [15.4, 16.8],
     `highway` 12.4 [12.4, 12.4];
     all pairs resolved.
  - `x86-64-v3`:
     `xxhash-rust` 45.5 [45.2, 46.0],
     `twox-hash` 45.3 [45.1, 45.9],
     `hashcrew` 43.9 [43.5, 44.6],
     `highway` 12.6 [12.5, 12.6];
     `xxhash-rust` and `twox-hash` within band,
     `twox-hash` over `hashcrew` resolved (1.03×).
  Against one-shot `files`,
   streaming costs `twox-hash` 12% and `xxhash-rust` 10% in the baseline build
   and 3% and 8% in the raised build;
   `hashcrew` streams 4% faster in the baseline build and 7% slower in the raised build;
   `highway` is unchanged.
- `hot16k`, one cache-resident 16 KiB buffer:
  - baseline:
     `twox-hash` 56.5 [56.1, 56.9],
     `xxhash-rust` 29.5 [29.2, 29.6],
     `hashcrew` 16.0 [15.9, 16.0],
     `highway` 12.4 [12.3, 12.4].
  - `x86-64-v3`:
     `xxhash-rust` 55.3 [55.2, 55.8],
     `twox-hash` 54.5 [53.9, 54.7],
     `hashcrew` 54.1 [54.0, 54.5],
     `highway` 12.5 [12.4, 12.5];
     `xxhash-rust` over `twox-hash` resolved (1.01×),
     `twox-hash` and `hashcrew` within band.

### In-memory ceiling for large inputs

`mem256m-stream1m`,
 one 256 MiB buffer in 1 MiB slices:

- baseline:
   `twox-hash` 52.8 [50.5, 52.9],
   `xxhash-rust` 28.2 [28.1, 28.4],
   `hashcrew` 17.7 [16.4, 17.7],
   `highway` 12.9 [12.9, 12.9].
- `x86-64-v3`:
   `hashcrew` 52.6 [52.1, 53.0],
   `xxhash-rust` 52.4 [51.4, 52.6],
   `twox-hash` 51.8 [51.4, 52.2]
   (all within band of each other),
   `highway` 12.9 [12.8, 12.9].

### S3 multi-GB from disk (weight 5)

Every run exited 0;
 400 runs,
 5 per build, file, mode, and algorithm
 (`data/bench/bigfile.jsonl`, `data/bench/summary-b.txt`).
Every algorithm produced one digest per file across builds, modes, and repetitions,
 and the three XXH3 crates produced the same digest
 (`47e66eee4ae2143c1071a33f1f301dbd` for the debug binary,
 `da7085172f72b431cb3efca3af95eb7e` for the image).

- Page-cache cold,
   662,710,296-byte debug binary:
  - read only:
     baseline 1.42 [0.17, 1.48],
     `x86-64-v3` 1.58 [0.80, 1.59].
  - finalists,
     baseline:
     `xxhash-rust` 1.45 [1.29, 1.48],
     `highway` 1.42 [1.02, 1.43],
     `hashcrew` 1.41 [1.25, 1.44],
     `twox-hash` 1.28 [1.19, 1.49];
     `x86-64-v3`:
     1.53 to 1.57,
     all within band of each other and of the read-only pass.
  - SHA-256 control:
     0.96 baseline and 1.06 `x86-64-v3`,
     below the read ceiling.
- Page-cache cold,
   3,966,238,720-byte image:
  - read only:
     baseline 0.30 [0.26, 0.37],
     `x86-64-v3` 0.39 [0.29, 0.40].
  - finalists,
     both builds:
     0.36 to 0.39,
     bands 7% to 54%,
     all within band of each other and of the read-only pass.
  - The single host `cat` of the same file in the eviction control took 14 seconds (about 0.26 GiB/s)
     outside any container,
     which points at the storage path rather than the container's memory limit
     (one unbanded host measurement, taken while upstream suites were running).
- Page-cache warm,
   debug binary:
  - read only:
     baseline 15.3 [15.1, 15.6],
     `x86-64-v3` 15.3 [14.5, 15.7].
  - baseline:
     `twox-hash` 11.2 [11.0, 11.4],
     `xxhash-rust` 8.6 [8.3, 8.7],
     `hashcrew` 6.8 [6.8, 7.1],
     `highway` 6.0 [5.9, 6.4];
     all pairs resolved.
  - `x86-64-v3`:
     `hashcrew` 11.3 [10.8, 11.4],
     `twox-hash` 11.2 [11.0, 11.3],
     `xxhash-rust` 11.0 [10.5, 11.9]
     (within band of each other),
     `highway` 5.9 [5.9, 6.1].
- Page-cache warm,
   image:
  - read only:
     baseline 13.7 [13.6, 14.7],
     `x86-64-v3` 15.0 [14.0, 15.7].
  - baseline:
     `twox-hash` 11.5 [10.4, 12.4],
     `xxhash-rust` 8.5 [8.4, 9.5],
     `hashcrew` 7.2 [6.9, 8.3],
     `highway` 6.8 [6.5, 7.1];
     `twox-hash` over `xxhash-rust` and `xxhash-rust` over `hashcrew` resolved,
     `hashcrew` and `highway` within band.
  - `x86-64-v3`:
     `twox-hash` 11.7 [10.7, 13.0],
     `xxhash-rust` 11.3 [10.6, 12.8],
     `hashcrew` 11.3 [11.2, 13.2]
     (within band),
     `highway` 6.4 [6.2, 7.4].
- Peak resident memory:
   3.2 to 3.4 MiB for every single-threaded run on both files
   (the harness's 1 MiB buffer included),
   so memory does not grow with a 3.97 GB input.

Whether I/O or the hash limits throughput:

- Cold reads:
   I/O limits.
  Every finalist runs within the read-only band on both files and both builds;
   the image reads at about 0.3 to 0.4 GiB/s and the debug binary at about 1.4 to 1.6 GiB/s,
   roughly an order of magnitude below the slowest finalist's in-memory speed.
  Only SHA-256 (0.96 GiB/s against a 1.42 read ceiling) is hash-limited cold.
- Warm reads:
   both limit.
  Copying from the page cache runs at 13.7 to 15.3 GiB/s;
   the XXH3 crates' AVX2 paths hash at about 52 GiB/s,
   so the copy dominates and the combined rate lands near 11 GiB/s,
   matching `1 / (1/15.3 + 1/52)`.
  `highway` hashes at 12.9 GiB/s in memory,
   so the hash and the copy limit it about equally (about 6 GiB/s combined).
  The SSE2 and per-stripe-dispatch baseline paths of `xxhash-rust` and `hashcrew` are hash-limited enough to show.
- Every result is from this host's storage;
   faster storage moves cold reads toward the warm case.

### S4 multi-core on one large input (weight 1)

- `twox-hash` and `highway` in the lab's two-thread chunk composition (16 MiB chunks):
   warm debug binary 6.9 and 5.4 GiB/s against 11.2 and 6.0 in one thread;
   warm image 7.9 and 5.8 against 11.5 and 6.8;
   cold within the read band.
  Two threads were slower,
   because reading stays sequential and the copy into chunk buffers is the limit;
   peak memory rose to 34 MiB.
  The composition's digests differ from the one-shot digests,
   as expected for a different construction.
- BLAKE3 control,
   `update_rayon` with 2 threads:
   warm debug binary 4.6 against 3.7 in one thread,
   warm image 5.4 against 4.5,
   same digest as single-threaded BLAKE3,
   peak memory 18.9 MiB.

No finalist offers a library tree mode,
 and on this host no two-thread construction reaches a single-threaded XXH3 finalist.

### S1 key composition (weight 1)

- `keymat`, 50 bytes to 50 KiB:
  - baseline:
     `twox-hash` 60.5 ns per record [60.3, 61.8],
     `xxhash-rust` 101.1,
     `hashcrew` 173.3,
     `highway` 259.7;
     all pairs resolved.
  - `x86-64-v3`:
     `xxhash-rust` 57.4,
     `twox-hash` 58.0,
     `hashcrew` 58.6
     (within band of each other),
     `highway` 243.5.
- `fp`, 70 to 242 bytes:
  - baseline:
     `xxhash-rust` 10.8 ns per item,
     `twox-hash` 11.9,
     `hashcrew` 24.0,
     `highway` 46.6;
     all pairs resolved.
  - `x86-64-v3`:
     `xxhash-rust` 9.7,
     `twox-hash` 11.9,
     `hashcrew` 18.6,
     `highway` 32.7;
     all pairs resolved.

### `x86-64-v3` against baseline

Beyond band for the finalists on file contents (`files`):
 `hashcrew` 2.95×,
 `xxhash-rust` 1.69×,
 `highway` 1.01×,
 and `twox-hash` 0.95× (slower in the raised build, also on `hot16k` at 0.96×).
On `mem256m-stream1m`,
 `twox-hash` and `highway` are within band,
 `xxhash-rust` 1.86× and `hashcrew` 2.98×.
On `fp`,
 `highway` 1.43×,
 `hashcrew` 1.29×,
 `xxhash-rust` 1.12×,
 `twox-hash` within band.
BLAKE3,
 SHA-256,
 and AES-CMAC stay within band on every file-sized workload,
 because they dispatch at run time already.

### Mechanical speed ratings

S1,
 S2,
 and S3 ratings follow one rule for every finalist
 (`scripts/speed-ratings.ts`, `data/speed-ratings.txt`):
 in each workload and build cell (and file and cache mode for S3),
 a finalist's ratio is its median over the best finalist's median,
 or 1 when its five-run range overlaps the best finalist's range;
 the criterion's value is the geometric mean of its cells,
 mapped to 4 at 0.90 or more,
 3 at 0.70,
 2 at 0.50,
 1 at 0.25,
 else 0.
A value within 0.03 of a threshold gets medium confidence,
 otherwise high.
The rule was fixed after the in-memory results and before the multi-GB results were complete,
 and it applies the same way to every finalist.

- S1 (4 cells):
   `twox-hash` 0.927 → 4 (medium),
   `xxhash-rust` 0.880 → 3 (medium),
   `hashcrew` 0.535 → 2 (high),
   `highway` 0.248 → 0 (medium).
- S2 (6 cells):
   `twox-hash` 0.989 → 4,
   `xxhash-rust` 0.756 → 3,
   `hashcrew` 0.564 → 2,
   `highway` 0.251 → 1 (medium).
- S3 (8 cells, 4 of them cold and within band for everyone):
   `twox-hash` 1.000 → 4,
   `xxhash-rust` 0.932 → 4,
   `hashcrew` 0.887 → 3 (medium),
   `highway` 0.740 → 3.

## Quality evidence

### SMHasher3

Source:
 SMHasher3 results summary and raw logs
 (`https://gitlab.com/fwojcik/smhasher3/-/blob/main/results/README.md` and `results/raw/`),
 accessed 2026-09-17,
 saved as `data/smhasher3-results-readme.md`, `data/smh3-XXH3-128.txt`, `data/smh3-XXH3-128.regen.txt`,
 and `data/smh3-HighwayHash-128.txt`.
SMHasher3 tests the algorithms through its own C and C++ implementations,
 not through the Rust crates;
 HC3 equality ties the crates to the same outputs.

- HighwayHash-128:
   "Overall result: pass ( 238 / 238 passed)" (`smh3-HighwayHash-128.txt:3733`),
   listed among passing hashes (`smhasher3-results-readme.md:63`).
  SMHasher3 applies 238 tests to it and 250 to XXH3-128.
- XXH3-128:
   "Overall result: FAIL ( 214 / 250 passed)" (`smh3-XXH3-128.txt:3820`),
   listed among failing hashes with 36 failed tests (`smhasher3-results-readme.md:209`).
  The failures split into two groups:
  - Tests that vary the seed
     (`SeedZeroes`, `SeedSparse`, `SeedBlockLen`, `SeedBlockOffset`, `SeedBIC`).
    Full-width 128-bit collisions appear only here:
     for example 534,343 among 2,196,480 hashes of 8-byte keys
     when seeds and one 4-byte block each have up to 2 set bits
     (`smh3-XXH3-128.txt:2804-2805`),
     and in every `SeedBlockLen` and `SeedBlockOffset` keyset (lines 2805 to 3303).
    Different seeds with related keys collide,
     so XXH3's seed is not a safe domain separator for short inputs.
  - Tests at a fixed seed
     (`BIC` for 3 to 15-byte keys, `Sparse` 3-byte keys with up to 20 set bits,
     `PerlinNoise`, `Bitflip` for 3, 4, and 8-byte keys).
    These report bias or collisions in 40-bit or narrower slices
     (for example "high 40-bit ... actual 470 (3.673x)" at `smh3-XXH3-128.txt:606`),
     with no nonzero 128-bit collision count.
- XXH3-128 with the secret regenerated per seed (`XXH3-128.regen`):
   231 of 250,
   with the `SeedBlockLen` and `SeedBlockOffset` failures gone,
   other seed tests still failing (`SeedZeroes`, `SeedSparse`, `Seed`, `SeedBitflip`),
   and the fixed-seed failures the same except one more `BIC` length (11 bytes)
   (`smh3-XXH3-128.regen.txt:3818-3827`).
  None of the Rust finalists exposes this variant as its default.
- For comparison:
   `gxhash` (128-bit) fails 25 tests,
   BLAKE3 passes 235 of 235,
   and MuseAir-128 passes all 250 (`smhasher3-results-readme.md`).

Relevance to meow:

- meow's cache is local and uses one fixed seed,
   so the seed-varying collisions do not arise
   if meow puts its salt and domain tags in the hashed bytes
   instead of the XXH3 seed.
  That is a usage rule meow must follow with any XXH3 finalist.
- The fixed-seed failures are bias or partial-width collisions,
   and every one except `PerlinNoise` uses keys of 15 bytes or shorter.
  The measured per-package key material is 50 bytes or longer,
   but 89 tracked files are shorter than 16 bytes ("Workload correction").
  No fixed-seed test reports a full-width collision,
   so these failures do not predict colliding cache keys;
   they do lower the quality margin compared with a function that passes every test.
- HighwayHash-128 has no known SMHasher3 failure.

### Local keysets

- HC8 one-byte keysets at 4,096 bytes:
   0 full-width,
   0 low-half,
   and 0 high-half collisions for every finalist ("HC8 one-byte collision gate").

## aarch64 and the m1 question

### What the aarch64 paths accelerate (S5 evidence)

- `xxhash-rust`:
   a NEON stripe accumulator selected by `cfg(target_feature = "neon")`,
   which every `aarch64-unknown-linux-*` target enables (`src/xxh3.rs:55`, `:252-268`).
  No published aarch64 throughput.
  xr5 ran its C comparison on the NEON kernel under QEMU.
- `twox-hash`:
   NEON accumulate and scramble kernels (`src/xxhash3/large/neon.rs`, 210 lines),
   selected at run time with `std`.
  Published by upstream on an Apple M1 Max
   (`comparison/README.md`, "xxHash3 (128-bit)", "Oneshot hashing"):
   Rust 34.4 GiB/s,
   C with NEON 34.6 GiB/s,
   C scalar 21.3 GiB/s,
   for 256 KiB to 4 MiB buffers.
  tw5 compared the NEON kernel with the C NEON and scalar builds under QEMU.
- `hashcrew`:
   NEON kernels (`src/xxhash/kernel/neon.rs`, 130 lines),
   selected at compile time on aarch64 because the target enables NEON
   (`src/xxhash/kernel/mod.rs:98-104`),
   so the run-time dispatch cost measured on x86_64 ("Benchmarks") does not apply there.
  No published aarch64 throughput.
- `highway`:
   a NEON implementation of the four-lane update with `vmull_u32` multiplies (`src/aarch64.rs`, 551 lines),
   used unconditionally on aarch64.
  The README claims "> 10 GB/s with SIMD (SSE 4.1 AVX 2, NEON)" without naming a machine or input size (`README.md:12`);
   `assets/highway.csv` has x86 results only.

Ratings for S5 are ranges,
 because no aarch64 hardware ran:
 `twox-hash` 3 to 4 (published NEON parity with C),
 `xxhash-rust` and `hashcrew` 2 to 4 (same algorithm and NEON kernels, no published numbers),
 `highway` 1 to 3 (its x86 vector paths run at about a quarter of XXH3's speed on this host,
 and nothing published shows a different ratio on aarch64).

### What an m1 run would add

`ssh m1` is a macOS machine
 (`AGENTS.md` rule `HRM` points write-heavy work at `/Volumes/MacData` on it),
 by its name an Apple M1 (inferred, not probed, because it is powered off),
 so a run there would cover the NEON kernels on real silicon
 through `aarch64-apple-darwin`,
 not the Linux aarch64 targets themselves.
It would add:

- real NEON throughput for the four finalists on the in-memory workloads and on files,
   replacing the S5 ranges with measured ratings;
- confirmation on hardware that NEON outputs equal the x86 outputs,
   which QEMU already showed for the probe corpus and the reference check;
- the relative cost of the 128-bit finalization and of `highway`'s NEON update on a core with a different
   multiply and memory profile.

Could it change the ranking:
 only through S5,
 whose weight is 1,
 or through a hard-gate failure on real hardware.
The sensitivity matrix tests both S5 range endpoints for every finalist
 and S5 weights up to 5,
 and none of those tests changes the order ("Sensitivity").
Even outside the evidence ranges,
 S5 at 0 for one finalist and 4 for the finalist ranked below it
 leaves every adjacent pair in order
 (`twox-hash` 65 against `xxhash-rust` 59,
 `xxhash-rust` 55 against `hashcrew` 50,
 `hashcrew` 46 against `highway` 43).
So an m1 run cannot change the ranking through measured speed under the frozen rubric.
A hard-gate failure would require QEMU's NEON emulation to differ from silicon on these kernels;
 no evidence suggests that,
 but this audit cannot exclude it.

## Reliance on the readings

- R1, dedicated instructions:
   no non-cryptographic library passes every hard gate
   ("Result under each reading of hardware acceleration" in "Screening").
  Under R1 alone the terminal result would be "No serious alternative":
   the candidates that meet R1 on both architectures fail HC9 (`gxhash`),
   HC2 and HC3 (`ahash`, CRC crates),
   or HC10 and HC3 (`rotohash-rs`),
   and SHA-256 and AES-CMAC,
   which meet R1,
   are outside the non-cryptographic scope.
  The R1 controls are also far slower than every finalist on file contents
   ("Benchmarks"),
   so dedicated instructions buy no speed for this workload on this host.
- R2, SIMD vectorization:
   four finalists,
   all with designed vector kernels on x86_64 and aarch64.

The ranking and recommendation rely on R2.
If the user meant R1,
 the only change to a hard constraint that admits a candidate is dropping HC10 streaming for `rotohash-rs`
 (which also lacks a stability statement)
 or widening the scope to cryptographic functions;
 both are questions for the user,
 not assumptions of this report.

## Key width

HC2 carried the 128-bit width over from the `gxhash128` decision.
Evidence checked for another width:

- Birthday bound:
   the probability of any collision among n random keys is about n² / 2^(b+1).
  For a cache holding 10^7 keys that is about 2.7 × 10^-6 at 64 bits
   and 1.5 × 10^-25 at 128 bits.
  A 64-bit key is therefore not negligible for a long-lived cache that silently reuses outputs on collision;
   128 bits is.
- Cost:
   XXH3-128 and XXH3-64 share the long-input kernels,
   so the width costs nothing measurable on file contents;
   on short key material the 128-bit finalization costs more,
   and short inputs carry weight 1 here.
- SMHasher3 full-width XXH3-128 collisions occur only when the seed varies ("Quality evidence"),
   which is an argument about seeding,
   not width.

No evidence argues for a different width;
 128 bits is retained.

## Controls

Controls are measured to show what leaving the decision scope or the hard gates would buy.
They are not ranked.

- `gxhash` 3.5.0 `gxhash128`:
   the HC8 positive control,
   2 full-width collisions on the zero base and 1 on the random base under every seed.
- MuseAir-128 (`museair` 0.6.0),
   a scalar design that fails HC1:
   0 collisions in HC8,
   passes all 250 SMHasher3 tests,
   and hashes file contents at 26.1 GiB/s on the baseline build,
   faster than every finalist on fingerprint material on the baseline build
   (9.8 ns per item against 10.8 ns for `xxhash-rust`,
   where XXH3 has no vector path at 240 bytes or less),
   but about half as fast as AVX2 XXH3 on files.
  So under R2 the vector kernels,
   not the multiply design,
   carry the whole-file advantage on this host.
- BLAKE3 (`blake3` 1.8.7),
   cryptographic:
   3.9 GiB/s on file contents and 5.6 GiB/s on the 256 MiB buffer in one thread;
   warm multi-GB streaming 3.7 to 4.5 GiB/s in one thread and 4.6 to 5.4 with `update_rayon` on 2 threads.
  The lab build compiled BLAKE3's upstream x86 assembly
   (build script output `blake3_sse2_ffi`, `blake3_sse41_ffi`, `blake3_avx2_ffi`, `blake3_avx512_ffi`),
   which needs a C toolchain;
   a rustc-only build (`pure` feature) was not measured.
- SHA-256 (`sha2` 0.11.0, SHA extensions),
   the R1 control:
   1.86 GiB/s on file contents,
   1.4 to 1.7 GiB/s warm multi-GB,
   and the only algorithm below the read ceiling on the cold debug binary (0.96 against 1.42).
- AES-CMAC (`cmac` 0.8.0 with `aes` 0.9.3, AES-NI),
   the AES control:
   1.43 GiB/s on file contents.

Every control produced 0 streaming mismatches in HC10 except `gxhash` and MuseAir,
 which were not given streaming wrappers.

## Ratings for S4 to S11

- S4 multi-core hashing of one large input, weight 1:
   no finalist offers a parallel or tree mode;
   any of them can sit under a meow-defined chunk tree,
   which is a new output format meow would own ("Benchmarks", S4 results).
  All four:
   1,
   high confidence,
   because the structure is identical and the measured two-thread composition did not beat one thread
   on warm or cold reads on this host.
- S5 aarch64 acceleration, weight 1,
   low-signal ranges ("aarch64 and the m1 question"):
   `twox-hash` 3 to 4 (midpoint 3.5),
   `xxhash-rust` 2 to 4 (3),
   `hashcrew` 2 to 4 (3),
   `highway` 1 to 3 (2).
- S6 quality beyond HC8, weight 1 ("Quality evidence"):
   the three XXH3 crates compute one function,
   so they share one rating:
   2,
   medium confidence,
   for 36 SMHasher3 failures that do not predict full-width collisions at a fixed seed
   but narrow the quality margin.
  `highway`:
   4,
   medium confidence,
   for 238 of 238 SMHasher3 tests and no local collision.
- S7 stability assurance, weight 1:
  - `twox-hash` 4, high:
     frozen XXH3 specification,
     README portability promise,
     CI proptests against the C library for scalar, SSE2, AVX2, and NEON kernels (tw2, tw5),
     and local equality across every dispatch path, aarch64, and the C reference.
  - `xxhash-rust` 3, medium:
     frozen specification and C comparison for lengths 0 to 4,095 on every kernel it compiles,
     local equality everywhere;
     but no written stability promise of its own,
     and #53 shipped wrong seeded-builder output until 0.8.17.
  - `hashcrew` 3, medium:
     frozen specification,
     README stability promise,
     tests against `xxhash-rust` rather than C,
     local equality everywhere and against C;
     two weeks of release history.
  - `highway` 3, medium:
     the algorithm is frozen "unchanging forever",
     README promise,
     reference 128-bit vectors only up to 63 bytes upstream,
     64-bit fuzzing against C;
     local 128-bit equality against C on long inputs ("Reference equality") closes most of that gap.
- S8 CPU capability handling, weight 1 ("HC5 CPU capabilities"):
  - `twox-hash`, `hashcrew`, `highway` 4, high:
     run-time selection in a default build;
     meow adds nothing.
  - `xxhash-rust` 2, high:
     compile-time only;
     AVX2 requires meow to ship a raised build with its own startup check and diagnostic,
     and upstream's run-time detection request (#48) is open.
  Speed differences between dispatch styles count under S2 and S3 only.
- S9 auditability and dependency surface, weight 1 ("Source quality and human auditability"):
  - `hashcrew` 4, medium:
     1,440 used-path lines,
     32 `unsafe`,
     forbidden `unsafe_op_in_unsafe_fn` and denied undocumented `unsafe` blocks.
  - `twox-hash` 3, medium:
     1,662 lines,
     66 `unsafe`,
     52 safety comments,
     strict lints.
  - `xxhash-rust` 3, medium:
     1,413 lines,
     29 `unsafe`,
     but no safety comments and an open report of raw-pointer functions not marked `unsafe` (#29).
  - `highway` 2, medium:
     2,646 lines,
     196 `unsafe`,
     no safety comments.
  All four have zero runtime dependencies.
- S10 upstream verification, weight 1 ("Upstream CI inventory and suites run"):
  - `twox-hash` 4, high:
     C comparison proptests for every kernel,
     Miri,
     CI on x86_64 and macOS aarch64.
  - `hashcrew` 3, medium:
     broad boundary and partition tests,
     Miri with strict provenance,
     CI on three operating systems;
     reference is another Rust crate.
  - `xxhash-rust` 3, medium:
     C comparison with valgrind on three kernels and cross-architecture tests;
     no Miri or property tests in CI.
  - `highway` 3, medium:
     reference vectors,
     quickcheck cross-implementation properties,
     an instruction-set matrix,
     Miri,
     a fuzz target that CI only compiles ("Fuzzing and mutation").
  No finalist has mutation testing.
- S11 maintenance, weight 1 ("Maintenance audit"):
  - `twox-hash` 3, medium:
     releases in the last 2 months,
     bugs answered within hours,
     one maintainer.
  - `xxhash-rust` 3, medium:
     three releases in July 2026,
     bugs fixed within a week,
     one maintainer,
     slow external pull requests in the past.
  - `highway` 2, medium:
     responsive to its one issue,
     but no release for 20 months with 16 unreleased commits.
  - `hashcrew` 1 to 3 (midpoint 2), low:
     active and signed releases,
     but two weeks old,
     self-merged,
     with no external tracker history.

## Scoring

Criteria and weights as frozen in "Frozen soft criteria":
 S1 1,
 S2 5,
 S3 5,
 S4 to S11 1 each;
 maximum 19 × 4 = 76.
Inputs are in `data/ratings.json`
 (built by `scripts/make-ratings.ts` from "Mechanical speed ratings" and "Ratings for S4 to S11");
 arithmetic and sensitivity by `scripts/score.ts`,
 output `data/score.txt`.

Ratings (value, confidence):

- `twox-hash`:
   S1 4 medium,
   S2 4 high,
   S3 4 high,
   S4 1 high,
   S5 3 to 4 low,
   S6 2 medium,
   S7 4 high,
   S8 4 high,
   S9 3 medium,
   S10 4 high,
   S11 3 medium.
- `xxhash-rust`:
   S1 3 medium,
   S2 3 high,
   S3 4 high,
   S4 1 high,
   S5 2 to 4 low,
   S6 2 medium,
   S7 3 medium,
   S8 2 high,
   S9 3 medium,
   S10 3 medium,
   S11 3 medium.
- `hashcrew`:
   S1 2 high,
   S2 2 high,
   S3 3 medium,
   S4 1 high,
   S5 2 to 4 low,
   S6 2 medium,
   S7 3 medium,
   S8 4 high,
   S9 4 medium,
   S10 3 medium,
   S11 1 to 3 low.
- `highway`:
   S1 0 medium,
   S2 1 medium,
   S3 3 high,
   S4 1 high,
   S5 1 to 3 low,
   S6 4 medium,
   S7 3 medium,
   S8 4 high,
   S9 2 medium,
   S10 3 medium,
   S11 2 medium.

Arithmetic in criterion order S1 to S11,
 low-signal ranges at their midpoints:

```text
# data/score.txt
twox-hash:   1×4 + 5×4 + 5×4 + 1×1 + 1×3.5 + 1×2 + 1×4 + 1×4 + 1×3 + 1×4 + 1×3 = 68.5 / 76 = 90.1
xxhash-rust: 1×3 + 5×3 + 5×4 + 1×1 + 1×3   + 1×2 + 1×3 + 1×2 + 1×3 + 1×3 + 1×3 = 58   / 76 = 76.3
hashcrew:    1×2 + 5×2 + 5×3 + 1×1 + 1×3   + 1×2 + 1×3 + 1×4 + 1×4 + 1×3 + 1×2 = 49   / 76 = 64.5
highway:     1×0 + 5×1 + 5×3 + 1×1 + 1×2   + 1×4 + 1×3 + 1×4 + 1×2 + 1×3 + 1×2 = 41   / 76 = 53.9
```

Totals across the low-signal ranges:

- `twox-hash` 68 to 69 (89.5 to 90.8);
- `xxhash-rust` 57 to 59 (75.0 to 77.6);
- `hashcrew` 47 to 51 (61.8 to 67.1);
- `highway` 40 to 42 (52.6 to 55.3).

No two adjacent ranges overlap.

Provisional order:
 `twox-hash` > `xxhash-rust` > `hashcrew` > `highway`.

### Fingerprint-sized inputs

S1 is the only criterion measured on fingerprint-sized material,
 and it carries weight 1.
With S1 removed from every total,
 the scores are 64.5, 55, 47, and 41 of 72:
 the same order.
No adjacent pair is decided by fingerprint-sized or key-material benchmarks;
 each is decided on file-sized and multi-GB inputs ("Ranking").

## Sensitivity

One input at a time,
 94 tests:

- each weight-1 criterion (S1, S4 to S11) raised to 2, 3, 4, and 5;
- S2 and S3 each lowered to 4, 3, 2, and 1;
- every medium-confidence exact rating moved one step down and one step up within 0 to 4;
- both endpoints of every low-signal range.

Results:

- The winner is `twox-hash` in all 94 tests.
- 93 tests keep the full order `twox-hash` > `xxhash-rust` > `hashcrew` > `highway` strictly.
- One test,
   S6 quality at weight 5,
   ties `hashcrew` and `highway` at 62.0
   (`twox-hash` 83.2, `xxhash-rust` 71.7),
   and no test puts `highway` above `hashcrew`.
- Closest margin per adjacent pair across all tests,
   normalized to the 76-point scale:
  - `twox-hash` over `xxhash-rust`:
     8.23 points, with S2 lowered to weight 1.
  - `xxhash-rust` over `hashcrew`:
     0.83 points, with S8 CPU capability handling raised to weight 5;
     the order holds,
     but a user who weighs run-time dispatch heavily is close to indifferent between them.
  - `hashcrew` over `highway`:
     0.00 points, the S6 tie.

Treatment of the tie:
 it concerns only the order of third and fourth place,
 no evidence this audit can gather changes a weight,
 and the weight of S6 is a user preference,
 so it is recorded as a conditional ranking and a question for the user
 ("Questions for the user").
Under S6 weight 5 the ranking is
 `twox-hash` > `xxhash-rust` > `hashcrew` = `highway`.
The recommendation does not depend on it.

Stability covers one-at-a-time changes only,
 not simultaneous changes to several weights or ratings.

## Pros and cons

### `twox-hash` 2.1.4 (XXH3-128)

Pros:

- Fastest finalist on file contents in the default build:
   49.3 GiB/s on the tracked tree and 56.5 GiB/s on a hot 16 KiB buffer,
   because it selects AVX2 at run time around the whole long-input loop.
- No build flag and no startup check needed for full speed or for older CPUs.
- The strongest upstream verification:
   CI proptests against the C library for scalar, SSE2, AVX2, and NEON,
   Miri,
   and a written portability promise.
- Published NEON parity with C on Apple silicon.
- Already a transitive dependency of two music-player lockfiles.

Cons:

- `XxHash3_128::new` copies the 192-byte default secret into a heap allocation
   (`src/xxhash3/streaming.rs:386-391`),
   one small allocation per streamed file.
- Single maintainer (400 of 428 commits);
   external feature pull requests can wait months.
- In the `x86-64-v3` build it is slightly slower than `xxhash-rust` on file contents
   (46.9 against 49.3 GiB/s, beyond the band).
- XXH3-128 fails 36 SMHasher3 tests,
   and its seed must not be used for domain separation.

### `xxhash-rust` 0.8.18 (XXH3-128)

Pros:

- Fastest finalist on fingerprint-sized material (10.8 ns per item baseline)
   and fastest or within band of the fastest on every workload in the `x86-64-v3` build.
- Smallest used path (1,413 lines, 29 `unsafe`),
   valgrind-checked C comparison on every kernel,
   and an AVX-512 kernel for `x86-64-v4` builds.
- Responsive maintainer with three releases in July 2026.

Cons:

- Compile-time kernel selection only:
   the default build runs SSE2 at 0.59 of `twox-hash` on file contents,
   and AVX2 needs a raised build plus a meow-owned startup check.
- No safety comments,
   and an open report about raw-pointer functions not marked `unsafe`.
- Releases before 0.8.17 returned wrong seeded-builder output on long inputs.
- No git tags;
   provenance rests on `.cargo_vcs_info.json`.
- Same XXH3 quality caveats.

### `hashcrew` 0.3.0 (XXH3-128)

Pros:

- Strictest `unsafe` policy of the four,
   with documented safety for every block,
   and a small used path (1,440 lines, 32 `unsafe`).
- Signed tags and commits.
- Run-time dispatch with a public backend API,
   and full AVX2 speed in the `x86-64-v3` build (47.2 GiB/s on file contents).
- Extensive boundary, partition, and streaming tests,
   and Miri with strict provenance.

Cons:

- Its run-time-dispatched AVX2 path is slow:
   16.0 GiB/s on file contents in the default build,
   below `xxhash-rust`'s SSE2 path,
   because the dispatch sits at each 64-byte stripe (`#[target_feature]` per-stripe kernels,
   `src/xxhash/kernel/x86.rs:117`, `:139`) and cannot inline into the loop.
- Two weeks old,
   one main author who self-merges,
   no external users on the tracker.
- Tests compare with another Rust crate, not C.
- Same XXH3 quality caveats.

### `highway` 1.3.0 (HighwayHash-128)

Pros:

- Passes every SMHasher3 test,
   with an algorithm frozen "unchanging forever" and a keyed design.
- Run-time dispatch to AVX2 or SSE4.1 at full speed in the default build.
- Instruction-set CI matrix, Miri, quickcheck properties, and a fuzz target against C.

Cons:

- About a quarter of XXH3's speed on file contents (12.4 GiB/s)
   and a fifth on fingerprint material (46.6 ns per item baseline).
- Largest `unsafe` surface (196 occurrences) with no safety comments.
- No release for 20 months,
   16 unreleased commits,
   single maintainer.
- Upstream reference 128-bit vectors only cover inputs shorter than 64 bytes,
   and the crate archive omits its license text.

## Ranking

Ranking:
 `twox-hash` 2.1.4 > `xxhash-rust` 0.8.18 > `hashcrew` 0.3.0 > `highway` 1.3.0.

- `twox-hash` over `xxhash-rust` (68.5 against 58):
   decided by S2 whole-file throughput (5 points)
   and S8 CPU capability handling (2 points),
   with S7, S10, S1, and S5 adding 3.5.
  In the default build `twox-hash` selects AVX2 at run time and hashes the tracked tree at 49.3 GiB/s,
   against `xxhash-rust`'s compile-time SSE2 path at 29.1,
   and streams warm multi-GB files at 11.2 to 11.5 against 8.5 to 8.6;
   in the `x86-64-v3` build they are within band of each other,
   but reaching it with `xxhash-rust` requires a raised build and a startup check in meow.
  The S1 point is not needed:
   without S1 the margin is 9.5 points,
   and the closest sensitivity test leaves 8.2.
- `xxhash-rust` over `hashcrew` (58 against 49):
   decided by S2 and S3 at weight 5 (10 points for `xxhash-rust`),
   against S8 and S9 (3 points for `hashcrew`),
   with S1 and S11 adding 2 for `xxhash-rust`.
  On the default build `xxhash-rust`'s SSE2 path hashes file contents at 1.8× `hashcrew`'s run-time-selected AVX2 path
   (29.1 against 16.0 GiB/s),
   because `hashcrew` calls its AVX2 kernel once per 64-byte stripe;
   on the raised build both are near 47 to 49.
  This pair is the least stable:
   with S8 at weight 5 the margin shrinks to 0.83 points of 76.
- `hashcrew` over `highway` (49 against 41):
   decided by S2 (5 points),
   S1 (2 points),
   S9 (2 points),
   and S5 (1 point),
   against S6 quality (2 points for `highway`).
  HighwayHash-128 hashes file contents at about 12.4 GiB/s in both builds,
   below `hashcrew`'s slowest path (16.0) and far below its raised-build path (47.2),
   and warm multi-GB streaming shows the same order in both builds
   (within band only on the image in the baseline build).
  `highway`'s full SMHasher3 pass is its one decisive advantage;
   if the user weighs quality beyond HC8 at 5,
   the two tie.

## Confidence and evidence limits

- One x86_64 host (Zen 4 with AVX-512) measured every speed;
   CPUs without AVX2,
   or with slower memory,
   would change absolute numbers and could change the SSE2 against AVX2 gaps.
- No aarch64 speed was measured;
   S5 is a low-signal range ("aarch64 and the m1 question").
- Cold-read results depend on this btrfs-on-LUKS NVMe volume and on the 2 GiB container's page-cache limit;
   on other storage the point where the hash rather than I/O limits throughput moves.
- The multi-core composition was measured with 16 MiB chunks against 1 MiB single-thread reads,
   so its result mixes thread count with chunk size.
- No finalist was fuzzed for 128-bit output against C except through the local reference check,
   and no mutation testing exists upstream.
- SMHasher3 results test the algorithms through SMHasher3's own implementations,
   tied to the crates by output equality rather than by running SMHasher3 on the crates.
- Stability promises are statements by upstream authors;
   the local tests show equality today,
   not a guarantee for future releases,
   so meow should pin versions and keep golden vectors in its own tests.
- The custom aarch64 musl static-pie target was not built ("HC4 build").
- Sensitivity covers one input at a time,
   not simultaneous changes of several weights or ratings.

## Recommendation

Recommended:
 `twox-hash` 2.1.4,
 `XxHash3_128` (XXH3-128),
 archive SHA-256 `5283634e518fe9e82c7b20520bb4bc209009fd16c82077c802f8111ecbb0117a`,
 commit `6f866bffe73900c63df2650be4eed41e3ed9b500`.

This relies on reading R2,
 SIMD vectorization:
 AVX2 or SSE2 on x86_64 and NEON on aarch64.
Under R1,
 dedicated instructions,
 no non-cryptographic candidate passes the hard gates
 ("Result under each reading of hardware acceleration").

Material risks,
 stated with the recommendation:

- XXH3's seed is not a safe domain separator
   (SMHasher3 full-width collisions when seeds vary, "Quality evidence").
  meow must hash with the default seed and put its salt and any domain tags into the hashed bytes.
- XXH3-128 fails 36 SMHasher3 tests at partial width or with varying seeds;
   `highway` is the only finalist that passes all,
   at about a quarter of the speed on file contents.
- Output stability rests on the frozen XXH3 specification and upstream promises;
   meow should pin `=2.1.4`
   and keep golden vectors in its own tests
   (for example the empty input `99aa06d3014798d86001c324468d497f`
   and the file digests in "S3 multi-GB from disk (weight 5)"),
   so an upgrade that changes output fails its tests.
- One maintainer.

How meow would use it,
 from the measurements:

- Dependency features:
   `default-features = false, features = ["std", "xxhash3_128"]`;
   `std` enables run-time AVX2 and NEON detection (`std = ["alloc"]`),
   and the default feature set would add `rand`.
- A default build needs no `-Ctarget-cpu` flag and no startup CPU check to reach AVX2 speed;
   a raised build still needs meow's own startup check ("HC5 CPU capabilities").
- Stream files with a fresh `XxHash3_128::new()` per file and reads of 64 KiB to 1 MiB;
   memory stays near 3 MiB for a 3.97 GB file.
- Serialize the `u128` in one fixed byte order in the cache format.
- Parallelize across files rather than within one file;
   within a file,
   reading limits throughput on this host ("S4 multi-core on one large input (weight 1)").

Recommendation is not adoption:
 no product code,
 dependency,
 configuration,
 or decision record was changed.

## Questions for the user

- Which reading of "hardware acceleration" did you mean?
  This report relies on R2 (SIMD vectors).
  If you meant dedicated instructions (R1),
   no non-cryptographic library passes,
   and the choices are to accept SIMD,
   to widen the scope to cryptographic R1 functions such as SHA-256 with SHA extensions
   (1.86 GiB/s on file contents against 49.3 for `twox-hash`),
   or to drop the streaming requirement for `rotohash-rs`
   (which also has no stability statement).
- Should quality evidence beyond the one-byte gate (S6) weigh more than the default 1?
  At weight 5 `hashcrew` and `highway` tie for third place;
   the recommendation does not change.
- Do you want `ssh m1` powered on to measure NEON speed and output equality on real silicon?
  Under the frozen rubric it cannot change the ranking:
   S5 at both range endpoints and at weight 5 keeps the order,
   and so does S5 at 0 for any finalist with 4 for the one ranked below it;
   it could only surface a hard-gate failure that QEMU missed.
- Would a C toolchain for the aarch64 musl build be acceptable?
  It changes no finalist,
   because none uses C;
   it would only give the BLAKE3 control its NEON path if cryptographic functions ever enter scope.
