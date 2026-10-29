# Repository-owned gxhash: design research

Merged into "Repository-owned gxhash" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-17.
Withdrawn the same day:
meow's cache keys left `gxhash` after the collision findings below,
and use XXH3-128 from `twox-hash`
(`doc/decision/monorepo-manager-cache-key-hash.md`),
so no repository-owned reimplementation is planned for meow.
The collision and undefined-behavior evidence is why the music player must switch too,
tracked in issue #545.

## Scope, sources, and labels

- Research date:
   2026-09-16 to 2026-09-17.
- Upstream clone:
   `~/temp/agent/gxhash-2026-09-16`
   (`gh repo clone ogxd/gxhash`,
   full history),
   `main` at `55bde47` (2025-05-18),
   latest tag `3.5.0`.
- Issue and pull request dumps:
   `scratchpad/gxhash/threads/` (`gh issue view` and `gh pr view` JSON rendered to text).
- Lab crate with a bit-exact prototype,
   differential test,
   benchmarks,
   and probes:
   `scratchpad/gxhash/lab/`;
   Miri probe `scratchpad/gxhash/miri-probe/`;
   runtime-detection probe `scratchpad/gxhash/nofeat-probe/`.
  `scratchpad` means `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad`.
- Nothing inside the repository was modified,
   built,
   or installed.
  Outside it:
   the Miri component was added to the `nightly-2026-09-16` toolchain,
   and crates were downloaded into `~/.cargo/registry`.
- Labels:
   "Verified" cites a file and line,
   a fetched page with quoted text,
   or a command with output;
   "Unverified" marks reasoning or recall that was not checked.
- Line numbers in upstream files refer to tag `3.5.0` unless a commit is named.

## Summary

### Findings

- A repository-owned gxhash can reproduce gxhash 3 output exactly without reading past the input.
  The lab prototype matched `gxhash` 3.5.0 on 97,545 (input,
   seed) pairs on x86_64,
   on aarch64 under QEMU,
   and against upstream's VAES `hybrid` build,
  runs clean under Miri on x86_64,
  and needs `unsafe` only for the call into each `#[target_feature]` entry (Verified).
- gxhash 3.5.0 has undefined behavior on inputs of 1 to 16 bytes:
   Miri stops with "attempting to access 16 bytes,
   but got alloc311 which is only 8 bytes from the end of the allocation" (Verified).
  The music player never hashes fewer than 24 bytes;
  the monorepo manager would,
   for the 89 tracked files under 16 bytes (Verified).
  Issue #111's panic depends on toolchain and frame layout and did not reproduce on Linux,
  with a positive control,
   in the platform probes (Verified by reading `probe-platforms.md`).
- gxhash has seed-independent structural collisions between inputs that differ in one
   or two bytes of a lane's last block:
   1 full 128-bit collision among the 1,044,480 single-byte edits of one random 4096-byte buffer,
   2 on SMHasher3's zero-base OneByte keyset,
   matching SMHasher3's report (Verified).
  Keeping gxhash 3 output keeps them;
   two lab candidates for a new output version showed 0 on the same OneByte keysets
   and have had no other quality testing.
- Upstream's reasons for no runtime detection and nightly-only VAES are outdated:
   safe `#[target_feature]` functions are stable since 1.86 and VAES intrinsics since 1.89 (Verified).
  Measured cost of runtime detection:
   +0.54 to +2.18 ns per hash on 70 to 242-byte fingerprint material,
   nothing measurable on file contents or 16 KiB buffers.
- Speed (bounded container,
   Zen 4,
   bands in "Benchmarks"):
  - Fingerprint material,
     64-bit,
     baseline `+aes,+sse2` build:
     gxhash64 6.787 ns,
     XXH3-64 6.728,
     rapidhash v3 6.924,
     within band;
     x86-64-v3 build:
     gxhash64 5.100 against XXH3-64 6.784,
     beyond band.
  - Fingerprint material,
     128-bit:
     gxhash128 6.579 ns against XXH3-128 9.592 and MuseAir-bfast-128 9.252 (baseline),
     beyond band.
  - File contents:
     gxhash 45 to 47 GiB/s in both builds;
     XXH3 29 GiB/s with SSE2 and 45 GiB/s with AVX2;
     the 16 KiB cache-resident buffer shows the hashes themselves reach 70 GiB/s (SSE) and 94 GiB/s (VAES),
     so whole-file hashing is limited by memory access at these speeds.
  - 128-bit against 64-bit gxhash:
     no measurable cost on file contents or 16 KiB buffers;
     from 3% faster to 22% slower on fingerprint material across binaries,
     tracking code layout.

### Ranking

A (dirty-room reimplementation) > B (fork) > C (published crate with workarounds) > D (different hash).

- A over B:
   both end as repository-owned code under MXR and RDC,
   but B starts from 56 `unsafe` occurrences,
   pointer walking,
   undocumented items,
   and inline tests that the rules force to be rewritten,
   while A already has a bit-exact,
   Miri-clean prototype;
   upstream is idle,
   so tracking it buys little.
- B over C:
   B can remove the short-input undefined behavior and add runtime detection;
   C keeps undefined behavior that content hashing hits,
   a padded-buffer workaround passes Miri only under Tree Borrows,
   and C contradicts the user's decision.
- C over D:
   C keeps the decided algorithm and the music player's bit-parity contract;
   D contradicts the decision and adds a second hash family.

### Output compatibility, in brief

- gxhash 3 output keeps the music player's caches,
   the published crate as a test oracle,
   and parallel evaluation of long inputs,
  and keeps the structural collisions (a stale-hit risk for a content cache whose real-world rate was not measured).
- A new output version can remove those collisions
  (`candidate128_chained`:
   +4% on 16 KiB in the baseline build,
   serial;
  `candidate128_extra_round`:
   +16% to +19%,
   stays parallel),
  at the cost of re-keying the music player once if it follows,
   losing the oracle,
   and owning quality validation.
- The two consumers can decide differently.

## Upstream algorithm, gxhash 3.5.0

Everything in this section is Verified against the tag `3.5.0` sources in the clone,
and the reimplementation it describes was checked bit for bit
(section "Reimplementation check").

### Notation

- `State` is a 128-bit vector,
   16 bytes,
   byte `i` of the vector is input byte `i` (little-endian loads).
  `__m128i` on x86 (`src/gxhash/platform/x86.rs:14`),
  `int8x16_t` on ARM (`src/gxhash/platform/arm.rs:11`).
- `aesenc(x, k)`:
   one AES encryption round (SubBytes,
   ShiftRows,
   MixColumns) on `x`,
   then XOR `k`.
  x86 `_mm_aesenc_si128` (`x86.rs:52-54`);
  aarch64 `veorq_u8(vaesmcq_u8(vaeseq_u8(x, 0)), k)` (`arm.rs:49-56`).
- `aesenclast(x, k)`:
   SubBytes and ShiftRows on `x`,
   then XOR `k`,
   with no MixColumns.
  x86 `_mm_aesenclast_si128` (`x86.rs:58-60`);
  aarch64 `veorq_u8(vaeseq_u8(x, 0), k)` (`arm.rs:60-65`).
- `add8(a, b)`:
   bytewise wrapping addition (`_mm_add_epi8`,
   `vaddq_s8`).
- `dup8(n)`:
   byte `n` repeated 16 times.
  `dup32(n)`:
   the 4 little-endian bytes of `n as u32` repeated 4 times.
- `load(b)`:
   16 input bytes as one `State` (`_mm_loadu_si128`,
   `vld1q_s8`).

### Constants and seed

- `KEYS` is 12 `u32` values (`src/gxhash/platform/mod.rs:45-48`):
   `0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E, 0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39132BD9, 0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B`.
- Round keys are 16-byte little-endian loads of that array:
   `K0 = KEYS[0..4]`,
   `K1 = KEYS[4..8]`,
   `K2 = KEYS[8..12]`
   (`ld(KEYS.as_ptr().offset(4))` in `platform/mod.rs:39` and `x86.rs:64-66`).
- The seed is an `i64` whose 8 little-endian bytes fill both halves of the state:
   `S = _mm_set1_epi64x(seed)` (`x86.rs:22-24`),
   `vdupq_n_s64(seed)` (`arm.rs:19-21`).

### Compression, `compress_all` (`src/gxhash/mod.rs:74-126`)

With `n` the input length:

1. `n == 0`:
    return the zero vector (`mod.rs:79-81`).
1. `1 <= n <= 16`:
    return `add8(P, dup8(n))`,
    where `P` is the `n` input bytes followed by zero bytes
    (`get_partial`,
    `mod.rs:86`;
    see "Partial-block read").
1. `n > 16`:
    - `e = n % 16`.
    - If `e == 0`,
       `h = load(input[0..16])` and the cursor moves 16 bytes;
      otherwise `h = add8(load(input[0..16]) masked to its first e bytes, dup8(e))`
      and the cursor moves `e` bytes (`mod.rs:92-103`).
      This leading partial block reads 16 bytes that are all inside the input.
    - `v = load(next 16)`;
      if `n > 32`,
       `v = aesenc(v, load(next 16))`;
      if `n > 48`,
       `v = aesenc(v, load(next 16))`;
      if `n > 64`,
       `h = compress_many(rest, h, n)` (`mod.rs:105-122`).
    - Return `aesenclast(h, aesenc(aesenc(v, K0), K1))` (`mod.rs:124-125`).
      The argument order matters:
       `h` is the AES data and the `v` chain is the round key.

### Long inputs, `compress_many` and `compress_8`

`compress_many` (`mod.rs:129-150`):
 the remaining bytes are a whole number of 16-byte blocks,
 `m` blocks.
The first `m % 8` blocks are absorbed one at a time,
 `h = aesenc(h, block)`.
The rest go to `compress_8` in groups of 8 blocks.

`compress_8`,
 non-hybrid (`x86.rs:68-107`,
 `arm.rs:73-110`):

1. `t1 = t2 = 0`,
    `lane1 = lane2 = h`.
1. For each group of blocks `b0..b7`:
    - `x1 = aesenc(aesenc(aesenc(b0, b2), b4), b6)`
    - `x2 = aesenc(aesenc(aesenc(b1, b3), b5), b7)`
    - `t1 = add8(t1, K0)`,
       `t2 = add8(t2, K1)`
    - `lane1 = aesenclast(aesenc(x1, t1), lane1)`
    - `lane2 = aesenclast(aesenc(x2, t2), lane2)`
1. After the loop,
    which may run zero times:
    `lane1 = add8(lane1, dup32(n))`,
    `lane2 = add8(lane2, dup32(n))`,
    return `aesenc(lane1, lane2)`.

Two consequences follow from the definition of `aesenclast`
(Verified by the split-accumulator check in "Reimplementation check"):

- Each lane update is `lane = SR(SB(aesenc(x, t))) XOR lane`,
  so a lane is the XOR of `h` and one independent term per group.
  The groups can be evaluated in any order or in parallel.
- A one-byte change in `b6` or `b7` reaches the lane after only one full round and one final round,
  a 4-byte difference.
  This is the structure behind the OneByte collisions in "Quality and collision resistance".

The length enters only here and only as `n mod 2^32`,
 added bytewise without carry.

### Seed, finalization, and outputs

- `gxhash(input, seed) = finalize(aesenc(compress_all(input), S))` (`mod.rs:69-71`).
- `finalize(x) = aesenclast(aesenc(aesenc(x, K0), K1), K2)` (`platform/mod.rs:37-43`).
- `gxhash32`,
   `gxhash64`,
   `gxhash128` read the low 4,
   8,
   or 16 bytes of the final state as a little-endian integer
  through a pointer cast (`mod.rs:15-20`,
   `:32-37`,
   `:49-54`).
- The seed touches the state only after compression,
  so any collision inside `compress_all` holds for every seed
  (issue #83;
   reproduced in "Quality and collision resistance").

### Partial-block read (`src/gxhash/platform/mod.rs:18-34`)

Used only when `1 <= n <= 16`:

- `check_same_page(p)`:
   `(address & 0xFFF) < 0x1000 - 16`,
   assuming 4 KiB pages (`platform/mod.rs:14-15`,
   `:28-34`).
- Same page:
   `get_partial_unsafe` loads 16 bytes at the input pointer,
   up to 15 bytes past the end of the slice,
   ANDs them with `cmpgt(dup8(n), [0, 1, ..., 15])`,
   and adds `dup8(n)` (`x86.rs:43-48`,
   `arm.rs:40-45`).
- Otherwise:
   `get_partial_safe` copies `n` bytes into a zeroed 16-byte stack buffer and loads that (`x86.rs:32-40`,
   `arm.rs:29-37`).
- On `main` after PR #118 (commit `390c3e5`),
   the 16-byte load is inline assembly
   (`movdqu` on x86,
   `ld1 {v0.16b}` on ARM,
   `options(nostack, preserves_flags, readonly)`),
   which is unreleased
   (`git diff 3.5.0 HEAD -- src/` shows only this change;
   no tag after `3.5.0`).

### Platform paths

- x86 and x86_64:
   `compile_error!` unless `aes` and `sse2` are enabled at compile time (`x86.rs:1-2`).
  There is no runtime detection and no fallback.
- x86 `hybrid` feature:
   `compile_error!` unless `aes` and `avx2` are enabled (`x86.rs:4-5`);
   note that `vaes` is not checked,
   although `_mm256_aesenc_epi128` needs it.
  `compress_8` then loads 256-bit pairs `(b0, b1)`,
   `(b2, b3)`,
   `(b4, b5)`,
   `(b6, b7)`
   and runs both lanes in one `ymm` register with `t = (t1, t2)` (`x86.rs:109-146`),
   which is the same arithmetic.
  The crate root needs nightly for it:
   `#![cfg_attr(feature = "hybrid", feature(stdarch_x86_avx512))]` (`src/lib.rs:3`).
- ARM and aarch64:
   `compile_error!` unless `aes` and `neon` are enabled (`arm.rs:1-2`).
  AES rounds are emulated from `vaeseq_u8` with a zero key plus `vaesmcq_u8` and XOR.
- 32-bit `x86` and `arm` share these files through `cfg(target_arch)` (`platform/mod.rs:1-7`).

### `Hasher` streaming API (`src/hasher.rs`)

- `GxHasher { state }` (`hasher.rs:16-18`).
  `Default` starts from the zero vector (`:48-51`);
  `with_seed(s)` starts from `S` (`:76-79`).
- `write(bytes)`:
   `state = aesenclast(compress_all(bytes), aesenc(state, K0))` (`:115-118`).
- `write_u8` through `write_i128`:
   `state = aesenclast(dupN(value), aesenc(state, K0))`,
   where `dupN` repeats the value's bytes to 16 bytes
   (`_mm_set1_epi8/16/32/64x`;
   raw 16 bytes for 128-bit values) (`:94-103`,
   `:120-129`,
   `x86.rs:148-198`).
- `write_usize`,
   `write_isize`,
   and `write_str` use the standard library defaults,
  which forward to `write` with the native-endian bytes
  (the Miri trace in "Debug-assertions panic" shows `write_usize` calling `write`).
- `finish()` is the low 64 bits of `finalize(state)`,
   and `finish_u128()` the full state (`:84-91`,
   `:107-112`).
  There is no separate seed round.
- `GxBuildHasher::default()` transmutes a `RandomState` into the state,
  or uses `42` with the `deterministic` feature (`:157-169`).
- Consequences:
   `Hasher` output never equals the one-shot functions,
   and splitting the same bytes across two `write` calls changes the result.
  Upstream documents this as intended (README "Consistency of Hashes When Using the `Hasher` Trait",
   `README.md:45-46`)
  and closed issue #127 on that basis.

### Stability promise

README `Hashes Stability` (`README.md:42-43`):
 "All generated hashes for a given major version of GxHash are stable,
 meaning that for a given input the output hash will be the same across all supported platforms."
Upstream's golden values are in `mod.rs:217-225` (`is_stable`).

### Reimplementation check

- `scratchpad/gxhash/lab/src/owned.rs` implements the one-shot `gxhash64` and `gxhash128`
  from the description above for x86_64 and aarch64,
  with no raw pointers and no read past the slice.
- `lab/src/bin/difftest.rs` compares it with `gxhash` 3.5.0 for every length from 0 to 2100
  at offsets 0,
   1,
   7,
   15,
   and 4093,
  9 seeds (including `i64::MIN` and `i64::MAX`),
  3000 random lengths up to 65,000 bytes,
  and upstream's `is_stable` vectors.
  Verified outputs:
  - x86_64 native:
     `difftest OK: 97545 (input, seed) pairs matched on x86_64, gxhash hybrid feature = false`.
  - aarch64,
     static musl built with `rust-lld`,
     run with `qemu-aarch64-static -cpu max`:
     `difftest OK: 97545 (input, seed) pairs matched on aarch64, gxhash hybrid feature = false`.
  - x86_64 against upstream built with `--features hybrid` and `+aes,+sse2,+avx2,+vaes` on nightly:
     `difftest OK: 97545 (input, seed) pairs matched on x86_64, gxhash hybrid feature = true`.
- The same runs also compare `gxhash128` with a lab variant
   that accumulates the per-group lane terms in two separate XOR accumulators (even and odd groups)
   and combines them at the end;
  all 97,545 pairs matched on x86_64 and on aarch64 under QEMU (Verified,
   second prototype shape),
  which confirms the order-independence stated in "Long inputs".
- Positive control:
   a mutant that flips one bit only for 11-byte inputs failed with
   `mismatch len=11 seed=0: gx64=f5deda344759e9e3 own64=6bd5ea82ccc23a2b ...`.
- Short inputs use in-bounds overlapping word loads (bytes `0..8` plus the last 8 shifted,
   and so on),
  the technique wyhash-family hashes use (Unverified attribution),
  and give identical bytes to upstream's masked over-read.

### License and crediting

- `LICENSE`:
   MIT,
   "Copyright (c) 2023 Olivier Giniaux" (Verified).
  Its condition:
   "The above copyright notice and this permission notice shall be included in all copies
   or substantial portions of the Software."
- `Cargo.toml`:
   `license = "MIT"`,
   `authors = ["Olivier Giniaux"]` (Verified).
- `CITATION.cff`:
   "If you use this software,
   please cite it as below",
   title "GxHash:
   A High-Throughput,
   Non-Cryptographic Hashing Algorithm Leveraging Modern CPU Capabilities",
   DOI `10.5281/zenodo.8368254` (Verified;
   its `orcid` field is the placeholder `1234-5678-9101-1121`).
- What crediting requires (Unverified legal reading,
   not advice):
   the MIT condition binds copies and substantial portions of the code.
  A reimplementation written after reading the source,
   reusing the `KEYS` table and the construction,
   is safest treated as derived,
   so the owned crate would ship the MIT notice
   (for example `LICENSES/MIT.txt` with that copyright line)
   next to the repository's `LGPL-3.0-or-later`,
   and credit ogxd and the DOI in its README.
  Citation is a request,
   not a license term.
- The repository has no REUSE configuration (`rg --files` for `REUSE.toml` and `.reuse/dep5` found none),
  and packages carry `LICENSES/GPL-3.0-or-later.txt` and `LICENSES/LGPL-3.0-or-later.txt`
  (for example `package/forbidden-strings/core/LICENSES`) (Verified).

## Upstream issues and pull requests

Status as of 2026-09-16,
 from `gh issue list` and `gh pr list --state all` (Verified);
quotes come from the thread dumps in `scratchpad/gxhash/threads/`.
Upstream activity:
 last commit `55bde47` on 2025-05-18,
 last release `3.5.0` (commit `6438a7b`,
 2025-03-12) (Verified,
 `git log`).

### Debug-assertions panic and undefined behavior

- #111 "Panic when debug-assertions=true in release profile":
   OPEN since 2025-01-10.
  The panic text is
   "unsafe precondition(s) violated:
   ptr::copy_nonoverlapping requires that both pointer arguments are aligned and non-null
   and the specified memory ranges do not overlap",
   reported on Apple Silicon with Rust 1.84 and gxhash 3.4.1,
   and "Still an issue with gxhash `v3.5.0`" (2025-03-22).
  The reproducer (`pedantic79/gxhash-test`,
   cloned to `~/temp/agent/gxhash-test-2026-09-16`) hashes `usize` keys,
   so `write_usize` passes an 8-byte temporary into the over-read path.
  So the check that fired is the overlap precondition of the 16-byte load,
   not a bounds check.
- Local reproduction attempts,
   nightly-2026-09-12 (Verified command output):
   the reproducer plus extra short stack and heap inputs printed `part2 = 31` and exited 0
   in all of x86_64 debug,
   x86_64 release with `debug-assertions`,
   aarch64 debug,
   and aarch64 release with `debug-assertions` (QEMU user mode).
  This lab run had no positive control of its own.
- The platform probes supply one (`doc/planning/monorepo-manager-route-research/probe-platforms.md`,
   "Probe 5:
   gxhash issue 111",
   Verified by reading):
   0 panics in 12 aarch64 musl runs on nightly-2026-09-12,
   12 on Rust 1.84.0,
   6 with `-C target-cpu=apple-m1`,
   and the x86_64 runs,
   over `gxhash64`,
   `gxhash128`,
   and `GxHasher` for lengths 0 to 67 plus the issue's `HashMap` reproducer,
   while a deliberately overlapping `copy_nonoverlapping` control aborted "with the issue's exact message on both
   compilers".
  Its source explanation for nightly:
   aarch64 `vld1q_s8` calls `ptr::read_unaligned`,
   which now calls `read`,
   and only `copy_nonoverlapping` carries the overlap precondition;
   Rust 1.84.0's `read_unaligned` still called `copy_nonoverlapping`.
  Why 1.84.0 did not reproduce on Linux is open there (frame layout differs from `aarch64-apple-darwin`,
   which QEMU user mode cannot run).
  In the lab sources,
   x86 `_mm_loadu_si128` still uses `ptr::copy_nonoverlapping` (`x86/sse2.rs:1339-1346` in nightly-2026-09-12,
   Verified).
- So #111 is a symptom whose trigger depends on toolchain and frame layout,
  while the read past the input it exposes is present on every platform (Miri,
   next bullet).
- Miri,
   nightly-2026-09-16,
   `scratchpad/gxhash/miri-probe` (Verified):
   upstream 3.5.0 with a `gxhash::HashMap<usize, usize>` insert stops with
   "Undefined Behavior:
   memory access failed:
   attempting to access 16 bytes,
   but got alloc311 which is only 8 bytes from the end of the allocation",
   inside `get_partial_unsafe` (`x86.rs:46`) via `write_usize`.
  The lab's owned prototype runs clean under Miri for every length 0 to 300 from a `Vec`,
   subslices,
   and an 8-byte stack array
   (`owned ok dc68532f08ff74cbd4158a8a1ae1f08e`).
  Miri cannot interpret the aarch64 path:
   "unsupported operation:
   can't call LLVM intrinsic `llvm.aarch64.crypto.aese` on architecture `aarch64`".
- #123 "undefined behavior for keys shorter than 16 bytes":
   CLOSED 2025-11-03 after the reporter found their own bug.
  Owner,
   2025-10-30:
   "this is how gxhash is intentionally designed ...
   The latest version on the main branch uses inline assembly to explicitely read beyond bounds without being flagged".
- #82 "Consider using simd_masked_load for the Read Beyond of Death trick":
   CLOSED 2026-07-20.
  RalfJung (2024-07-30):
   "If an allocation has size 5 and you are reading 8 bytes,
   that's UB.
   Checking page boundaries makes no difference."
  Owner's micro-benchmark of the partial load alone (2025-05-17,
   AMD Ryzen 9 5950X):
   "copy:
   8.0726 ns,
   urbd:
   0.9856 ns,
   urbd_asm:
   0.9748 ns,
   simd_masked_load:
   2.5433 ns,
   portable_simd:
   2.5748 ns"
   (Apple M1:
   5.2783,
   1.2450,
   1.2444,
   3.0270,
   3.8833 ns).
  That benchmark has no in-bounds word-load variant.
- #98 "Experiment with simd_masked_load to read beyond without undefined behavior":
   CLOSED unmerged 2025-05-18,
   "Closing in favor of #118".
  The #111 reporter noted it fixed the panic "but since it uses inline assembly,
   miri no longer works" (their wording;
   #98 itself is the `simd_masked_load` branch).
- #100 "Read beyond asm":
   CLOSED unmerged 2025-05-17.
- #118 "Use inline assembly to read beyond bounds":
   MERGED 2025-05-18,
   unreleased (no tag after `3.5.0`).
- Music player exposure (Verified by reading code):
   both fingerprint call sites hash `path + 8 + 16` bytes
   (`package/music-player/desktop-app/src/peakcache.rs` `fingerprint`,
   `package/music-player/android-app/rust/src/fingerprint.rs` `compute`),
   so inputs are always at least 24 bytes and never take the `n <= 16` over-read path.
- Monorepo manager exposure (Verified by measurement):
   89 git-tracked files are smaller than 16 bytes,
   so content hashing with 3.5.0 would take the over-read path for them.

### Software fallback and runtime detection

- #47 "Add fallback for platforms without AES acceleration":
   OPEN since 2023-12-22,
   no body.
- #54 "Add software AES fallback":
   OPEN draft PR (`src/gxhash/platform/soft.rs`,
   144 lines).
  A contributor (2024-01-05):
   "I got it mostly working (fails `is_stable`),
   however the performance is,
   as I feared,
   abysmal."
  The owner listed four paths and noted the stability promise constrains them;
   a user (2024-05-01):
   "the reason I use `gxhash` is its stability".
- #59 "Add runtime feature detection":
   CLOSED unmerged 2024-01-20 in favor of #61.
- #61 "Enforce target feature":
   OPEN.
  Owner (2024-01-21):
   "the target_feature disables inlining,
   which defeats the purpose of an fast hashing algorithm",
   then switched to the build-time `compile_error!` (#62).
- #66 SIGILL on an Intel Core i7 930 (no AES-NI):
   CLOSED;
   owner suggested detecting AES at runtime and switching hashers.
- Two facts that change this trade-off since 2024 (Verified):
  - Rust 1.86.0 "Allow safe functions to be marked with the `#[target_feature]` attribute" (`RELEASES.md:1549`).
  - Rust 1.89.0 "Stabilize the avx512 target features" (`RELEASES.md:1139`);
    `_mm256_aesenc_epi128` is `#[stable(feature = "stdarch_x86_avx512", since = "1.89")]`
    (`stdarch/crates/core_arch/src/x86/vaes.rs:42`).
  - Building upstream `hybrid` on stable 1.98.1 still fails only on the crate attribute:
     "error[E0554]:
     `#![feature]` may not be used on the stable release channel" at `gxhash-3.5.0/src/lib.rs:3:33`.

### Quality and collision resistance

- #83 "Hash has arbitrary seed-independent multicollisions,
   is not DoS resistant":
   OPEN since 2024-06-02 (orlp).
  purplesyringa (2024-12-10):
   "`compress_all` is completely independent from the seed,
   so a collision in `compress_all` affects all seeds."
  Owner (2024-12-10):
   "Mixing the seed at the start instead of the end should significantly improve DoS resistance.
   I'm currently (slowly) exploring this path."
- #124 "seems to be failing some tests on smhasher3":
   OPEN since 2025-11-14.
  Owner (2025-11-22):
   "I fail to see why gxhash would have to comply to all benchmarks in the www."
- SMHasher3 raw results,
   fetched from `gitlab.com/fwojcik/smhasher3` (Verified):
  - `results/raw/gxhash.txt` (128-bit):
     "Overall result:
     FAIL ( 225 / 250 passed)",
     failures "Sparse :
     [2/2048]",
     "TwoBytes :
     [2048,
     4096]",
     "SeedZeroes :
     [1280,
     8448]",
     "SeedSparse" and "Seed" at 10 key lengths.
  - `Keyset 'Sparse' - 2048-byte keys with up to 2 bits set - 134225921 keys`:
     "Testing all collisions ( 128-bit) - Expected 0.0,
     actual 4".
  - `Keyset 'OneByte ' - all 4096-byte keys with 1 non-zero byte - 1044480 keys`:
     "Testing all collisions ( 128-bit) - Expected 0.0,
     actual 2".
  - `results/raw/gxhash-64.txt`:
     "Overall result:
     FAIL ( 226 / 250 passed)".
  - `results/README.md` lists as passing all 250 tests,
     among others,
     `a5hash`,
     `MuseAir`,
     `MuseAir-128`,
     `rapidhash`,
     `komihash`,
     `polymurhash`,
     `MeowHash`;
     it lists `XXH3-64` failing 27 and `XXH3-128` failing 36,
     `gxhash-64` failing 24,
     `gxhash` failing 25.
- Local reproduction,
   bounded container,
   `lab/src/bin/onebyte.rs` (Verified):
  - Zero base,
     4096 bytes,
     seed 0:
     `gxhash128 len=4096 keys=1044480 collisions=2`,
     the same count SMHasher3 reports.
    Pairs:
     byte 755 = `0x3a` versus byte 1785 = `0xb0` (both in block 7 of an 8-block group),
     and byte 363 = `0x92` versus byte 2923 = `0x92` (both in block 6).
    The owned prototype gives the same two pairs;
     `xxh3_128` and `museair128` give 0.
  - Random base content,
     4096 bytes,
     every single-byte edit:
     `gxhash128 len=4096 keys=1044480 collisions=1` (byte 354 = `0xf3` versus byte 877 = `0xbc`,
     both block 6),
     and the pair stays colliding under seeds `1, -1, 42, i64::MAX, -6917529027641081856`:
     `[true, true, true, true, true]`.
    `xxh3_128` gives 0.
  - Zero base,
     2048 bytes:
     `gxhash128 len=2048 keys=522240 collisions=1` (the same 755 and 1785 pair).
  - Every colliding byte sits in the last block of a lane,
    which matches the 4-byte difference described in "Long inputs".
- Ecosystem precedent (Verified):
   crate `whasher` 0.1.8 wraps gxhash 3.5.0 and says it routes inputs of 32 KiB
   and more to independent streaming checksums
   "规避 gxhash 单次直算的结构性碰撞族" (to avoid gxhash one-shot's structural collision family)
   (`whasher-0.1.8/src/lib.rs:22`,
   also `:12-14`).
- Earlier fixed defects,
   for history:
   #37 out-of-bounds read for sizes of 80 bytes and up,
   fixed in 2.2.5 with 2.x up to 2.2.4 yanked;
   #44 `Hasher` permutation collisions,
   fixed in 2.3.1 (#48).

### Output-changing work in progress

- #89 "Release v4":
   OPEN since 2024-06-12,
   base `main`,
   head `v4`.
- #81 "Change seed from i64 to u64":
   MERGED into `v4` (2024-06-12).
- #88 "Improve DOS resistance":
   OPEN,
   base `v4`,
   changes `src/gxhash/mod.rs` (+76 -12).
- #109 "If fallthrough":
   OPEN draft.
  It starts from the seed,
   absorbs whole blocks first and the partial block last,
  replaces `KEYS`,
   and comments out `is_stable`,
  so it changes output (Verified,
   `gh pr diff 109`).

### Performance and API requests

- #101 "Reduce register spilling":
   OPEN,
   owner's own to-do.
- #122 "Throughput benchmark fairness suggestions":
   OPEN;
   other hashers were measured through `hash_one`,
   which adds a length prefix,
   while gxhash was called directly;
   also asks for `lto = "fat"` and `codegen-units = 1`.
- #121 "Fallback algorithm suggestion:
   rapidhash":
   OPEN;
   a gxhash user answered "I am using `gxhash` at work and we rely on it producing the same hash on every system."
- #126 `std::io::Write` for `GxHasher`,
   #128 streaming hasher in `no_std`,
   #125 multi-process hashing and binaries,
   #65 random seed once per thread,
   #63 best `write` chunk size,
   #112 CLI,
   #7 WASM:
   all OPEN.
- #127 "gxhash128 and finish_u128 should be same result":
   CLOSED as intended;
   the reporter's follow-up shows reading a FIFO in varying chunk sizes gives different hashes.

## Repository constraints that shape the options

- MXR (`AGENTS.md:1012-1020`):
   `.rs` files have a 300 code-line budget from `monochromatic-rust-linter` rule `max-lines`;
   `tests/`,
   `*_tests.rs`,
   `fuzz/`,
   and `build.rs` are exempt;
   "never disable" (Verified).
- RDC (`AGENTS.md:1022-1028`):
   rustdoc on every documentable item,
   private included,
   from `require-rustdoc`;
   tests and fuzz exempt;
   "never disable" (Verified).
- The built-in linter overrides exempt only `**/tests/**`,
   `**/*_tests.rs`,
   `**/fuzz/**`,
   `build.rs`,
   and `**/fixture/**`,
   `**/test-fixture/**`,
   `**/invalid/**`
   (`package/rust-module/rust-linter-core/default.toml`,
   Verified).
  There is no carve-out for vendored third-party code.
- `unsafe` policy (Verified by search):
  - No lint forbids or restricts `unsafe`:
     `rg 'unsafe_code|unsafe_op_in_unsafe_fn|undocumented_unsafe|forbid\(unsafe|deny\(unsafe'` over the repository matched only prose in docs.
  - Root `clippy.toml` bans only `Result::unwrap`;
     crate manifests deny `disallowed_methods` and `implicit_return`
     (for example `package/forbidden-strings/core/Cargo.toml:22-25`).
  - The word `unsafe` appears in 27 Rust source files (`rg --count-matches '\bunsafe\b'` over `package`),
     including SIMD kernels in `package/forbidden-strings/core/src/dfa/sheng.rs`
     (`is_x86_feature_detected!` guards,
     `#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")] unsafe fn`,
     lines 227-282).
  - An earlier route study recorded the same:
     "No lint forbids `unsafe_code`" (`doc/planning/monorepo-manager-route-research/stack-rust-crates.md:82`).
- Publishing precedent:
   `forbidden-regex` 0.1.1 is a published crate with `include`,
   `LICENSES/*.txt`,
   `lto = true`,
   `codegen-units = 1`,
   `overflow-checks = true`,
   and sibling `.bench` and `.fuzz` packages (`package/forbidden-strings/core*`,
   Verified).
- Existing consumers (Verified,
   `rg 'GxHasher|GxBuildHasher|gxhash::Hash(Map|Set)|gxhash32|gxhash128'` outside docs matched only a doc comment):
   the two music-player crates call only `gxhash64(&material, 0)`,
   with `gxhash = "3"` and `+aes` in `.cargo/config.toml` for each target triple.
- Music player contract (`package/music-player/PROPOSAL.shared-core.md:362-370`,
   Verified):
   "Keep the `gxhash` major version,
   the fixed seed,
   the exact (path,
   size,
   mtime) byte layout,
   and the AES target feature both `.cargo/config.toml` files set.
   Any change re-keys both peak caches."
  A re-key costs one re-measurement per track,
   because a cache miss is treated as "not measured yet" (`doc/troubleshooting/gxhash-aes-target-feature.md:128-130`).
- Monorepo manager plan (`doc/planning/monorepo-manager-from-scratch-design.md`,
   "Cache",
   Verified):
   `gxhash128` for cache keys,
   content hashing as the source of truth,
   a startup CPU check that exits with a diagnostic,
   Linux x86_64 and aarch64,
   glibc and static musl.
  The CPU check there uses `__cpuid(1)` because `is_x86_feature_detected!("aes")` is constant `true` in a `+aes` build.
- Repository toolchain:
   floating `nightly` in root `mise.toml:94` (Verified);
   the lab used nightly-2026-09-12 (`rustc 1.100.0-nightly`,
   LLVM 23.1.1).

## Options

Each option is designed until a disqualifying problem appears or none is found.
The output-version question is separate and is in "Output compatibility";
every option below can target gxhash 3 output.

### Option A: dirty-room reimplementation in the repository

Shape:

- A new crate,
   for example under `package/rust-module/`,
  written from the algorithm description,
  with ogxd credited and the MIT notice shipped for the derived parts.
  The crates.io name `gxhash` is taken (`cargo search gxhash` lists `gxhash = "3.5.0"`,
   Verified).
- Module split that fits MXR by construction:
   primitive wrappers per architecture,
   in-bounds short loads,
   compression,
   8-block lanes (128-bit,
   and wide VAES as a separate module),
   CPU detection,
   public API.
  The lab prototype `lab/src/owned.rs` covers x86_64 and aarch64 plus three lab-only variants in 270 lines (Verified,
   `wc --lines`).
- API with a capability token instead of build flags:
   `detect()` returns a zero-sized token or an error naming the missing feature (`aes`,
   `sse2`,
   `neon`,
   and `vaes` for the wide path);
   hashing methods take the token.
  The only `unsafe` is the call from each token method into its `#[target_feature]` entry,
  justified by the token's existence.
  Safe `#[target_feature]` functions are stable since Rust 1.86 (Verified,
   `RELEASES.md:1549`).
- Without a crate-wide `+aes`,
   `is_x86_feature_detected!` really detects at runtime,
   which removes the `__cpuid(1)` workaround the design doc needs:
   the no-flag probe binary calls `std_detect::detect::cache::detect_and_initialize` (Verified,
   disassembly of `nofeat-probe/target-nofeat/release/bench`).
- Tests (see "Testing and publishing").

Pros:

- No undefined behavior on short inputs:
   the prototype has no raw pointer reads and runs clean under Miri on x86_64 (Verified).
  That also removes the code path behind #111.
- Bit-exact gxhash 3 output is reachable:
   the differential test passes on x86_64,
   on aarch64 under QEMU,
   and against upstream's `hybrid` build (Verified).
- Runtime detection gives the warning the user asked for instead of SIGILL,
  and consumers no longer need per-triple `-C target-feature=+aes` or hit `compile_error!`.
  Its cost is measured in "Benchmarks" ("Runtime detection cost").
- Wide VAES paths on stable Rust (Verified,
   `vaes.rs:42`,
   stable since 1.89),
  known-length streaming,
   and parallel lane evaluation become possible
  (see "Optimization opportunities").
- Fits MXR,
   RDC,
   and the publishing precedent from the start.

Cons:

- The repository owns SIMD code for two architectures,
  and its speed is sensitive to code shape:
   the first prototype took 7.44 ns per fingerprint hash against upstream's 4.73 ns in the x86-64-v3 build
   because its inner function was not inlined and returned its state through memory,
   and the restructured prototype took 5.07 ns against 5.10 ns (see "Benchmarks");
   `#[inline(always)]` is rejected on `#[target_feature]` functions on stable 1.98.1 and nightly-2026-09-12
   ("cannot use `#[inline(always)]` with `#[target_feature]`",
   issue 145574,
   Verified),
   so inlining has to be arranged by structure (one caller per inner copy).
- aarch64 speed cannot be measured here;
   QEMU user mode gives correctness only.
- Miri cannot run the aarch64 AES path (Verified),
   so aarch64 soundness rests on the absence of pointer code.
- Reading upstream makes this a derived work in practice,
   not clean-room (Unverified legal reading).

Disqualifying problems found:
 none.

### Option B: fork of upstream in the repository

Shape:

- Copy upstream `src/` (983 lines at `3.5.0`,
   56 occurrences of the word `unsafe`,
   Verified by script)
  or `main` (inline assembly read) into the repository with its MIT license,
  optionally with history,
  then modify.

Pros:

- Starts from upstream's exact behavior,
   including `GxHasher`,
   `HashMap` aliases,
   `no_std`,
   32-bit targets,
   and `hybrid`.
- Upstream's own tests and golden values come along.
- Attribution is automatic.

Cons:

- As copied it breaks repository rules that have no vendored-code exemption:
   upstream items carry no rustdoc (for example `x86.rs:16-66`),
   tests sit inline in `mod.rs` and `hasher.rs` instead of `*_tests.rs`,
   and `#[rustfmt::skip]` wraps the module (`src/lib.rs:5`).
  MXR and RDC say "never disable",
   and the linter defaults exempt only tests,
   fuzz,
   build scripts,
   and fixtures (Verified).
- Removing the undefined behavior means replacing the pointer-walking design,
  or keeping `main`'s inline assembly,
   which stops Miri:
   #123 "git `main` branch of `gxhash` cannot be run with `miri` due to usage of inline assembly" (Verified quote).
- `compile_error!` gates and the nightly-only `hybrid` attribute need rework for runtime detection and stable VAES.
- Little to track:
   upstream's last commit is 2025-05-18 and its v4 branch (#89) has been open since 2024-06-12.

Disqualifying problems found:
 kept as upstream code,
 it cannot satisfy MXR and RDC;
 conformed,
 it becomes Option A with more code to remove.

### Option C: stay on the published crate with workarounds

Shape:

- Keep `gxhash = "3"` (resolving to 3.5.0),
  add `-C target-feature=+aes,+sse2` (x86_64) and `+aes,+neon` (aarch64) to every build of the monorepo manager,
  keep the `__cpuid(1)` and `AT_HWCAP` startup check from the design doc,
  run aarch64 tests in release builds,
  and route inputs of 16 bytes or fewer around the over-read.

Pros:

- No owned hash code;
   output is the reference by definition.
- Works today in the music player,
   whose inputs are never shorter than 24 bytes (Verified).

Cons:

- Undefined behavior on inputs of 1 to 16 bytes (Miri,
   Verified),
  which content hashing hits for at least the 89 tracked files under 16 bytes.
  A wrapper that copies inputs of 16 bytes or fewer into a zeroed 32-byte buffer and passes `&buf[..n]` keeps the output
  but still reads past the slice:
   Miri with its default Stacked Borrows reports
   "Undefined Behavior:
   attempting a read access using <1477> at alloc742[0x1],
   but that tag does not exist in the borrow stack for this location",
   while `-Zmiri-tree-borrows` prints `wrapper ok` (`miri-probe/src/bin/wrapper.rs`,
   Verified).
  Its soundness therefore depends on which aliasing model Rust adopts.
- #111 stays open and unreleased fixes stay unreleased (no release since 2025-03-12).
- SIGILL instead of a warning if the startup check is skipped or a new code path hashes first.
- `hybrid` needs nightly;
   no streaming API equal to the one-shot result (#127).
- It contradicts the user's 2026-09-16 decision:
   "Dirty room (crediting ogxd) reimplement gxhash ourselves.
   Or forking it.
   ... so we are doing it."

Disqualifying problems found:
 conflicts with the user's decision,
 and short-input undefined behavior for content hashing that the wrapper removes only under Tree Borrows.

### Option D: a different stable hash for content hashing

Shape:

- XXH3-128 (`xxhash-rust`),
   MuseAir-128 (`museair`),
   or 64-bit rapidhash v3 (`rapidhash::v3`) for the monorepo cache,
  leaving the music player on gxhash.

Pros:

- Portable,
   no AES requirement,
   no CPU check.
- MuseAir and rapidhash pass all SMHasher3 tests (Verified,
   `results/README.md`);
  XXH3-128 and MuseAir-128 had 0 OneByte collisions locally (Verified).

Cons:

- Contradicts the user's decision that content hashing uses gxhash.
- Speed trade-offs are in "Benchmarks".
- A second hash family in the repository.

Disqualifying problems found:
 conflicts with the user's decision;
 included only because the benchmark bears on its premise.

### Complement, not an alternative: contributing upstream

- The in-bounds short read (no assembly,
   Miri-clean,
   bit-exact) and a stable-Rust `hybrid` gate could be offered to ogxd/gxhash.
- It does not replace A,
   B,
   or C in the near term,
   since upstream has not released since 2025-03-12.
- It is external communication,
   which needs the user's authorization under repository rules PX3 and XCM.

### Testing and publishing, shared by A and B

- Differential oracle:
   `gxhash = "=3.5.0"` as a dev-dependency with `+aes` only in test builds;
   the lab's check covers every length 0 to 2100,
   5 offsets,
   9 seeds,
   and 3000 long random inputs,
   and caught a one-length mutant (Verified).
- Golden vectors committed as data,
   so tests without the oracle still pin output.
- Property tests:
   every input byte affects the hash,
   bytes outside the slice never do (upstream `all_blocks_are_consumed` and `does_not_hash_outside_of_bounds`),
   split-accumulator evaluation equals serial evaluation (lab check,
   Verified),
   wide VAES equals 128-bit,
   runtime-detected entry equals compile-time entry,
   known-length streaming equals one-shot for random chunkings.
- Miri on x86_64 for the whole crate (Verified feasible;
   needs the pre-fetch workaround for the Miri sysroot,
   see "Process notes").
- aarch64 correctness in CI through QEMU user mode (Verified feasible with `rust-lld` and `aarch64-unknown-linux-musl`).
- Fuzzing:
   a `cargo-fuzz` differential target against the oracle,
   following `package/forbidden-strings/core.fuzz`.
- Mutation testing:
   `forbidden-regex` keeps `mutants.out` (Verified),
   a precedent for `cargo-mutants`.
- Publishing:
   a new crate name;
   license expression covering MIT for derived code plus the repository's `LGPL-3.0-or-later`
   or MIT alone (user decision).

### Ranking

A > B > C > D.

- A over B:
   both end as repository-owned code under MXR and RDC,
   but B starts from 56 `unsafe` occurrences,
   raw pointer walking,
   undocumented items,
   and inline tests that must all be rewritten,
   while A already reached bit parity in a Miri-clean prototype;
   upstream is idle,
   so B's head start in tracking upstream buys little.
- B over C:
   B can remove the short-input undefined behavior and add runtime detection;
   C keeps undefined behavior that content hashing would hit and contradicts the user's decision.
- C over D:
   C keeps the decided algorithm and the music player's bit-parity contract;
   D contradicts the decision and splits the repository across two hash families.

## Output compatibility

Not decided here.
There are two separable decisions:
 the output of the monorepo manager's cache keys,
 and the output of the music player's fingerprint.
An owned crate can serve both with versioned modules,
 as `rapidhash` does with `rapidhash::v1`,
 `v2`,
 `v3`
 ("These are stable functions whose output will not change between crate versions",
 `rapidhash-4.5.1/README.md`,
 Verified).

### Keeping gxhash 3 output bit for bit

What it keeps:

- The music player's contract (`PROPOSAL.shared-core.md:362-370`) with no re-key,
  so both flavors could move to the owned crate without touching either peak cache,
  and gain the missing-AES warning on Android devices without the ARMv8 crypto extensions.
- The published crate as a test oracle,
   the strongest cheap correctness evidence available.
- Known quality:
   SMHasher3's existing gxhash port and its results (fails 24 and 25 of 250).
- Parallel evaluation of long inputs:
   because each lane is an XOR of per-group terms,
   AVX-512 VAES and multi-threaded hashing can keep the exact output
   (split-accumulator check,
   Verified).

What it keeps that may matter:

- Structural collisions between inputs that differ in one or two bytes placed in the last block of a lane:
   1 full 128-bit collision among the 1,044,480 single-byte edits of one random 4096-byte buffer,
   which is about 5.5 × 10^11 pairs,
   and 2 among the zero-base OneByte keyset (Verified,
   bounded container).
  A change confined to one lane of one group cannot collide with the unchanged input,
   because every step on that path is a bijection for fixed other inputs (Unverified derivation);
   the observed collisions are between two different edits of the same base.
  For a content-addressed cache this is the stale-hit case:
   a file edited one way,
   cached,
   then edited another way at a colliding position gets the old result.
  How often real edits land on colliding pairs was not measured.
- Seed-independent collisions (#83):
   irrelevant for a fixed-seed local cache with no adversary (the design doc keeps the cache local only).
- Length mixed only as `n mod 2^32` (`dup32(n)` in `compress_8`):
   relevant only for inputs of 4 GiB or more.
- `GxHasher` output compatibility is not needed:
   nothing in the repository uses the `Hasher` API (Verified,
   search in "Repository constraints").

### A new output version

What it could change:

- Remove the structural collisions.
  Two lab candidates,
   neither validated beyond these checks:
  - `candidate128_chained`:
     `lane = aesenc(lane, aesenc(x, t))`,
     so the lane passes through a full round per group;
     0 OneByte collisions at 2048 and 4096 bytes with zero and random bases (Verified);
     serial,
     so it gives up parallel lane evaluation.
  - `candidate128_extra_round`:
     one more round per group before the XOR,
     `lane = aesenclast(aesenc(aesenc(x, t), K2), lane)`;
     keeps the XOR structure and parallel evaluation;
     OneByte result in "Benchmarks" ("Candidate checks").
  Speed of both is in "Benchmarks".
- Mix the seed first (#83,
   #88,
   #109),
   mix the full 64-bit length,
   and take a `u64` seed (#81).
- Absorb whole blocks first and the partial block last,
   as #109 does,
  so a streaming API can equal the one-shot result without knowing the length up front.

What it costs:

- The music player re-keys both caches once,
   and the contract text in `PROPOSAL.shared-core.md` changes;
  per `doc/troubleshooting/gxhash-aes-target-feature.md:128-130` the cost is one re-measurement per track.
- No oracle:
   correctness rests on golden vectors,
   cross-architecture equality,
   and property tests.
- Quality becomes the repository's job:
   an SMHasher3 run needs a C-callable build or a port,
   and the local OneByte harness covers one keyset only.
- A name and version scheme that makes silent output changes impossible,
  for example versioned modules and golden-vector tests per version.

### If the two decisions differ

- Keeping gxhash 3 output for the music player and a new version for the monorepo cache means one crate
   with two output families,
  shared primitives,
   and two sets of golden vectors.
- The cache key already covers tool versions (design doc,
   "Cache"),
  so changing the cache hash re-keys every task once.

## Optimization opportunities upstream has not taken

"Keeps gxhash 3 output" means the one-shot `gxhash64` and `gxhash128` results stay bit-identical.

### Keeps gxhash 3 output

1.  In-bounds short-input loads.
    Inputs of 1 to 16 bytes load `bytes[0..8]` and the last 8 bytes shifted (4 and 1-byte variants below 8),
    instead of a 16-byte over-read behind a 4 KiB page check.
    Removes the undefined behavior,
     the page-size assumption,
     and the #111 code path.
    Evidence:
     difftest and Miri (Verified);
     speed in "Benchmarks".
    Upstream's #82 benchmark compared only copy,
     over-read,
     masked load,
     and `portable_simd`,
     not this.
1.  Runtime CPU detection with safe `#[target_feature]` functions and a capability token.
    Replaces `compile_error!`,
     per-triple `RUSTFLAGS`,
     SIGILL,
     and the `__cpuid(1)` workaround with a diagnostic.
    Upstream dropped runtime detection in 2024 because `#[target_feature]` blocked inlining (#61);
    the measured cost today is in "Benchmarks" ("Runtime detection cost").
1.  Inlining by structure.
    `#[inline(always)]` is not allowed on `#[target_feature]` functions (Verified),
    so each public entry needs its own single-caller copy of the inner functions
    (the lab uses an extra const generic).
    Measured effect in "Benchmarks" ("Owned prototype,
     first and second shape").
1.  Wide VAES on stable Rust.
    `_mm256_aesenc_epi128` is stable since 1.89 (Verified);
    upstream's `hybrid` fails on stable only because of `#![feature(stdarch_x86_avx512)]` (Verified,
     E0554).
    Upstream's gate checks `avx2` but not `vaes` (`x86.rs:4`);
     a runtime `vaes` check closes that gap.
    Measured in "Benchmarks" (`v3_vaes_hybrid` build).
1.  AVX-512 VAES,
     two 8-block groups per 512-bit step.
    `_mm512_aesenc_epi128` is stable since 1.89 (`vaes.rs:89-92`,
     Verified).
    Output is kept because lanes are XORs of per-group terms (split-accumulator check,
     Verified).
    Not built or measured.
1.  Multi-threaded hashing of large inputs,
     by the same XOR argument.
    Not built or measured;
    the file-contents benchmark suggests memory bandwidth,
     not the hash,
     limits whole-file hashing at these speeds
    (see "Benchmarks").
1.  Known-length streaming equal to the one-shot result.
    The layout of `compress_all` depends only on the total length `n`
    (leading partial block of `n % 16` bytes,
     three `v` blocks,
     `m % 8` single blocks,
     then 128-byte groups),
    so an incremental hasher told `n` up front can buffer at most one group and produce the one-shot result
    (Unverified until built).
    Needs the file size before reading and a check that the size did not change.
    Addresses #126,
     #127,
     and #128 for the cases where the length is known.
1.  Building with `-C target-cpu=x86-64-v3` plus `aes`.
    VEX three-operand encoding changes gxhash's speed on fingerprint-size inputs;
    see "Benchmarks".
    Output is unaffected (difftest passes in that build,
     Verified).
    Upstream #101 ("Reduce register spilling") is the same topic.
1.  aarch64 SVE2 AES (`svaese_u8`,
     `svaesmc_u8`).
    Present in stdarch but unstable behind `stdarch_aarch64_sve` (issue 145052) (Verified);
    hardware with SVE2 AES was not checked (Unverified).
    Output is kept only if the lane layout stays two 128-bit lanes.
    Not actionable on stable Rust today.

### Changes output

1.  Remove the structural lane collisions
    (lab candidates `candidate128_chained` and `candidate128_extra_round`;
     checks and speed in "Benchmarks").
1.  Mix the seed before compression (#83,
     #88,
     #109).
    Only matters with an adversary;
     the cache seed is fixed.
1.  Mix the full 64-bit length.
1.  Absorb the partial block last (#109),
    which allows streaming equal to one-shot without knowing the length first.
1.  `u64` seed (#81,
     merged into upstream's unreleased `v4` branch).

### Changes only the `Hasher` API output

1.  Override `write_usize` and `write_isize` with fixed-width loads,
    so `usize` keys stop going through the short-input path.
1.  `std::io::Write` (#126) and a `no_std` streaming hasher (#128).
1.  Draw the random seed once per thread (#65).

Nothing in the repository uses the `Hasher` API (Verified),
so these matter only if the owned crate also offers `HashMap` support.

## Benchmarks

All numbers are Verified outputs of `scratchpad/gxhash/analyze.ts` over `results.tsv` (first series)
and `results-r2.tsv` (second series).
x86_64 only:
 QEMU user mode translates aarch64 instructions on the x86_64 host,
 so its timings would describe the emulator,
 not NEON hardware (Unverified that no host acceleration applies;
 not attempted).

### Setup

- Host CPU:
   AMD Ryzen 7 8700F (Zen 4),
   8 cores and 16 threads;
   `lscpu` flags include `aes`,
   `avx2`,
   `avx512f`,
   `avx512bw`,
   `avx512vl`,
   `vaes`,
   `vpclmulqdq` (Verified).
- Every run:
   `podman run --rm --memory=2g --cpus=2 --security-opt label=disable`,
   image `registry.fedoraproject.org/fedora:latest`,
   binaries and data read-only.
- Toolchain nightly-2026-09-12,
   profile `opt-level = 3`,
   `lto = "fat"`,
   `codegen-units = 1`,
   `panic = "abort"`.
- Builds:
  - `baseline_aes_sse2`:
     `-C target-feature=+aes,+sse2`,
     the music player's flag set.
  - `v3_vaes_hybrid`:
     `-C target-cpu=x86-64-v3 -C target-feature=+aes,+vaes`,
     gxhash `hybrid` on;
     `objdump` counts 10 `vaesenc` instructions on `ymm` registers
     (the baseline build has 92 `aesenc` and no `ymm` use).
     `xxhash-rust` picks its AVX2 path from `target_feature = "avx2"` at compile time (`xxh3.rs:16-24`).
- Crates:
   `gxhash` 3.5.0,
   `xxhash-rust` 0.8.15 (`xxh3_64`,
   `xxh3_128`),
   `rapidhash` 4.5.1 (`v3::rapidhash_v3`),
   `museair` 0.6.0 (`hash`,
   `hash128`,
   `bfast::hash`,
   `bfast::hash128`),
   `komihash` 0.5.0 (`v5::komihash`);
   seed 0 or default secrets.
- Call shape:
   every algorithm sits behind an `#[inline(never)] fn(&[u8]) -> u128` wrapper,
   called directly from a loop monomorphized per algorithm;
   results are XOR-folded and passed to `black_box`.
  Call sites that inline the hash could differ.
- Workloads:
  - `fp`:
     2,226 fingerprint materials,
     one `Vec` each.
    Path lengths come from the audio files found under the home directory (`find` to depth 5):
     minimum 46,
     median 82,
     90th percentile 127,
     maximum 218 bytes,
     so materials are 70 to 242 bytes.
    Path bytes are pseudo-random printable ASCII,
     then 8 size bytes and 16 mtime bytes,
     as in `peakcache.rs`.
    Android `content://` URIs were not measured.
  - `files`:
     the contents of all 8,093 git-tracked files,
     151,796,144 bytes,
     one `Vec` each
     (median 4,039 bytes,
     99th percentile 238,724,
     maximum 22,908,459;
     89 files under 16 bytes).
    Hashing only;
     no file I/O is timed.
  - `hot16k`:
     one 16,384-byte buffer,
     small enough for the 32 KiB per-core L1d cache (`lscpu`:
     256 KiB over 8 instances).
- Timing:
   a warm-up of a quarter of the budget,
   then whole passes until the budget (250 ms for `fp` and `hot16k`,
   400 ms for `files`);
   algorithm order rotates every round.
- Band:
   (maximum minus minimum) divided by the median,
   over every round of every container run.
  A difference counts only when it exceeds the band of both compared rows.

### Run-to-run band on one unchanged build

Series `band`:
 `gxhash64` alone,
 baseline build,
 3 container runs of 10 rounds each.

- `fp`:
   median 6.665 ns per hash,
   range 6.522 to 7.811,
   band 19.3%.
- `files`:
   median 380.453 ns per file,
   range 364.360 to 454.093,
   band 23.6% (45.91 GiB/s).

Bands in the comparison series ranged from about 1% to 28%;
in the wide ones the minimum stays close to the median while the maximum jumps,
 which reads as occasional slow rounds rather than drift (Unverified interpretation).

### Fingerprint material, 70 to 242 bytes

Second series (restructured prototype),
 median ns per hash,
 [minimum to maximum],
 band:

- `baseline_aes_sse2`:
  - `owned64` 5.111 [5.045 to 5.323] 5.4%
  - `owned128` 5.356 [5.204 to 5.563] 6.7%
  - `candidate128_chained` 5.636 [5.582 to 5.911] 5.8%
  - `candidate128_extra_round` 5.690 [5.605 to 5.768] 2.9%
  - `gxhash128` 6.579 [6.519 to 6.933] 6.3%
  - `xxh3_64` 6.728 [6.680 to 6.943] 3.9%
  - `gxhash64` 6.787 [6.686 to 8.199] 22.3%
  - `rapidhash_v3_64` 6.924 [6.859 to 7.037] 2.6%
  - `museair_bfast128` 9.252 [9.191 to 9.641] 4.9%
  - `xxh3_128` 9.592 [9.515 to 9.987] 4.9%
- `v3_vaes_hybrid`:
  - `owned64` 5.071 [4.856 to 5.450] 11.7%
  - `gxhash64` 5.100 [4.621 to 5.379] 14.9%
  - `candidate128_chained` 5.472 [5.409 to 5.770] 6.6%
  - `owned128` 5.536 [5.408 to 5.884] 8.6%
  - `candidate128_extra_round` 5.653 [5.448 to 5.851] 7.1%
  - `gxhash128` 5.772 [5.716 to 5.938] 3.8%
  - `xxh3_64` 6.784 [6.715 to 7.336] 9.2%
  - `rapidhash_v3_64` 6.883 [6.800 to 7.289] 7.1%
  - `xxh3_128` 9.101 [9.034 to 9.996] 10.6%
  - `museair_bfast128` 9.212 [9.110 to 9.584] 5.1%
- First series only,
   baseline:
   `museair_bfast64` 9.070 (9.5%),
   `komihash64` 9.314 (8.5%),
   `museair128` 9.921 (10.8%),
   `museair64` 9.924 (9.8%).

Reading:

- 64-bit,
   baseline build:
   `gxhash64`,
   `xxh3_64`,
   and `rapidhash_v3_64` are within band of each other.
- 64-bit,
   v3 build:
   `gxhash64` at 5.100 is 25% below `xxh3_64` at 6.784,
   beyond both bands.
- 128-bit:
   `gxhash128` is 31% below `xxh3_128` in the baseline build (6.579 against 9.592)
   and 37% below in v3 (5.772 against 9.101),
   beyond bands.
- Owned prototype against upstream:
   baseline `owned64` 5.111 against `gxhash64` 6.787,
   a 25% difference just beyond `gxhash64`'s 22.3% band;
   v3 5.071 against 5.100,
   within band.

### 64-bit against 128-bit output width

Measured as asked,
 separately on each workload.
The algorithm does the same work for both;
the 128-bit result only adds the extraction of the upper 64 bits
(`_mm_unpackhi_epi64` plus `_mm_cvtsi128_si64` in the prototype,
 a pointer read in upstream).

- `fp`,
   `gxhash128` against `gxhash64`:
   baseline 6.579 against 6.787 (128-bit 3% faster,
   within band);
   v3 5.772 against 5.100 (128-bit 13% slower,
   within `gxhash64`'s 14.9% band,
   beyond `gxhash128`'s 3.8%).
  First series:
   baseline 6.811 against 6.652 (+2.4%,
   within band);
   v3 5.774 against 4.729 (+22%,
   beyond band).
- `fp`,
   owned:
   baseline 5.356 against 5.111 (+4.8%,
   within band);
   v3 5.536 against 5.071 (+9.2%,
   within band).
- `files`:
   baseline 372.191 against 375.072 ns per file;
   v3 389.627 against 386.862;
   within band in both.
- `hot16k`:
   baseline 215.783 against 216.760 ns;
   v3 163.024 against 162.887;
   within band in both.
- Conclusion:
   no measurable cost on file contents or 16 KiB buffers;
   on fingerprint-size inputs the difference ranged from 3% faster to 22% slower across binaries,
   which tracks code layout more than width.

### File contents

Second series,
 median ns per file (GiB/s),
 band:

- `baseline_aes_sse2`:
   `candidate128_extra_round` 372.087 (46.95) 12.5%,
   `gxhash128` 372.191 (46.93) 6.8%,
   `owned128` 373.491 (46.77) 8.2%,
   `gxhash64` 375.072 (46.57) 10.4%,
   `owned64` 378.159 (46.19) 25.7%,
   `candidate128_chained` 378.980 (46.09) 18.8%,
   `xxh3_64` 599.240 (29.15) 5.7%,
   `xxh3_128` 600.961 (29.07) 5.7%,
   `museair_bfast128` 629.988 (27.73) 4.0%,
   `rapidhash_v3_64` 653.417 (26.73) 4.3%.
  First series adds `museair_bfast64` 627.262,
   `museair64` 671.383,
   `museair128` 673.065,
   `komihash64` 803.790.
- `v3_vaes_hybrid`:
   `owned64` 380.868 (45.86),
   `candidate128_extra_round` 381.916 (45.74),
   `candidate128_chained` 384.078 (45.48),
   `owned128` 384.577 (45.42),
   `xxh3_64` 384.680 (45.41) 12.7%,
   `gxhash64` 386.862 (45.16) 22.1%,
   `gxhash128` 389.627 (44.83) 16.8%,
   `xxh3_128` 390.044 (44.79) 9.4%,
   `museair_bfast128` 599.649 (29.13),
   `rapidhash_v3_64` 630.442 (27.71).
  First series:
   `komihash64` 1968.992 in this build against 803.790 in baseline (not investigated).
- Reading:
   everything AES-based,
   and XXH3 with AVX2,
   lands near 45 to 47 GiB/s,
   within band of each other,
   while the same hashes run 70 to 94 GiB/s on the cache-resident `hot16k` buffer.
  So this workload is limited by memory access,
   not by the hash (Unverified cause;
   the `hot16k` contrast is the evidence).
  With SSE2 only,
   XXH3 takes 60% longer than gxhash here (599 against 375 ns),
   beyond band.

### Cache-resident 16 KiB buffer

Second series,
 median ns per hash (GiB/s),
 band:

- `baseline_aes_sse2`:
   `owned128` 215.026 (70.96),
   `gxhash128` 215.783 (70.71),
   `owned64` 216.318 (70.54),
   `gxhash64` 216.760 (70.39),
   `candidate128_chained` 224.111 (68.09) 8.6%,
   `candidate128_extra_round` 255.170 (59.80) 9.3%,
   `xxh3_128` 522.190 (29.22),
   `xxh3_64` 522.949 (29.18),
   `museair_bfast128` 549.212 (27.78),
   `rapidhash_v3_64` 571.834 (26.68).
- `v3_vaes_hybrid`:
   `gxhash64` 162.887 (93.68) 3.6%,
   `gxhash128` 163.024 (93.60) 1.4%,
   `owned64` 197.504 (77.26),
   `owned128` 198.178 (77.00) 1.2%,
   `candidate128_chained` 198.215 (76.98),
   `candidate128_extra_round` 229.562 (66.47),
   `xxh3_128` 278.346 (54.82),
   `xxh3_64` 279.108 (54.67),
   `museair_bfast128` 527.231 (28.94),
   `rapidhash_v3_64` 546.821 (27.90).
- Reading:
   upstream's VAES `hybrid` path is 18% faster than the 128-bit path in the same build (163.0 against 198.2 ns),
   beyond bands;
   the owned prototype has no VAES path yet.
  Candidate cost against `owned128`:
   `chained` +4% baseline and 0% v3;
   `extra_round` +19% baseline and +16% v3.

### Owned prototype, first and second shape

- First shape (first series),
   `fp`:
   baseline `owned64` 6.391 against `gxhash64` 6.652 (within band);
   v3 `owned64` 7.435 against `gxhash64` 4.729,
   57% slower,
   beyond band.
  Disassembly of the v3 wrapper:
   `call gxlab::owned::gxhash::<false>` with the state returned through the stack
   (`mov %rsp,%rdi` then `mov (%rsp),%rax`).
- Second shape,
   one single-caller inner copy per entry through an extra const generic:
   the wrapper contains the whole hash (no call),
   and `fp` v3 is 5.071 against 5.100.
- Output was identical in both shapes (difftest,
   Verified).

### Runtime detection cost

`nofeat-probe`:
 the second-shape owned hash behind a cached `is_x86_feature_detected!` check,
 built without any `target-feature` flag (`runtime_detect_no_aes_flag`,
 entry called out of line)
 and with `+aes,+sse2` (`compile_time_aes_flag`,
 entry inlined),
 3 container runs each.

- `fp`:
   `checked64` 7.633 [7.559 to 9.432] 24.5% against 5.452 [5.385 to 5.832] 8.2%,
   +2.18 ns (+40%),
   beyond both bands;
   `checked128` 6.124 [6.013 to 6.397] 6.3% against 5.581 [5.515 to 5.946] 7.7%,
   +0.54 ns (+10%),
   beyond both bands.
- `files`:
   382.709 against 381.378 and 380.670 against 381.731 ns per file,
   within band.
- `hot16k`:
   219.230 against 213.731 (+2.6%) and 212.248 against 213.748 (-0.7%),
   within band.

### Candidate checks

OneByte collisions,
 bounded container,
 `lab/src/bin/onebyte.rs` (Verified):

- `candidate128_chained`:
   0 at 4096 bytes with zero base,
   0 with random base,
   0 at 2048 bytes with zero base.
- `candidate128_extra_round`:
   0,
   0,
   and 0 on the same three keysets;
   `gxhash128` gave 2 in the same run.
- These candidates have seen no other quality test.

### Caveats

- One host and one microarchitecture;
   Intel and ARM hardware were not measured.
- Code layout can move results by more than the band on fingerprint-size inputs:
   `owned128` and `candidate128_chained` compute identical results with identical steps for inputs under 177 bytes,
   the shortest length with a full 8-block group (2,142 of the 2,226 `fp` items,
   96.2%,
   measured),
   yet measured 7.060 against 5.697 ns in the first series baseline build,
   beyond both bands (8.9% and 15.0%),
   and 5.356 against 5.636 in the second,
   within band.
- `komihash64`'s v3 result was not investigated.

## Open questions for the user

Each question is separable;
 answering one does not settle another.

1.  Cache-key output for the monorepo manager:
    gxhash 3 bit for bit (keeps the oracle and parallel evaluation,
     keeps the OneByte-class collisions),
    or a new output version (removes them,
     loses the oracle;
     the two lab candidates trade speed differently)?
1.  Music player fingerprint output:
    keep gxhash 3 (no re-key,
     contract unchanged),
    or move to the new version if one is chosen (one re-measurement per track on both flavors)?
1.  Build-flag model:
    runtime detection through a capability token (no `RUSTFLAGS`,
     no `compile_error!`,
     a warning instead of SIGILL,
     the measured per-call cost in "Runtime detection cost"),
    or `-C target-feature=+aes` builds with the startup check (no per-call cost)?
    Both can warn;
     they differ in who must set build flags and in the per-call cost.
1.  License expression for the owned crate:
    the repository's `LGPL-3.0-or-later` plus the MIT notice for derived code,
    or MIT alone to match upstream?
1.  Publishing:
    publish to crates.io under a new name,
     or keep it repository-internal?
    If published,
     which name?
1.  Scope:
    one-shot hashing only,
     or also `Hasher`,
     `HashMap` aliases,
     and known-length streaming?
    Nothing in the repository uses the `Hasher` API today.
1.  Upstream:
    offer the in-bounds short read and the stable `hybrid` gate to ogxd/gxhash?
    This is external communication and needs your go-ahead.
1.  aarch64 speed:
    measure the NEON path on real hardware
    (`AGENTS.md` HRM names `ssh m1`,
     and the music player's tests used a Pixel 6)?
    QEMU user mode here gives correctness only.

## Process notes

- Cargo reached crates.io from the scratchpad (`cargo generate-lockfile`,
   `cargo info`,
   `cargo fetch` all succeeded),
  but `cargo miri setup` failed on the std sysroot's dependencies with
   "Could not connect to server (Failed to connect to index.crates.io:443 after 22 ms ...)"
   while `curl` to `https://index.crates.io/config.json` returned `200` in the same shell (Verified).
  This matches the per-application network policy recorded in
  `doc/planning/monorepo-manager-route-research/stack-rust-crates.md`.
  Workaround used,
   without bypassing the policy:
   a scratch manifest depending on the 31 registry packages in the toolchain's `library/Cargo.lock` was fetched
   from the scratchpad (`scratchpad/gxhash/sysroot-fetch`),
   then `CARGO_NET_OFFLINE=true cargo +nightly-2026-09-16 miri setup` succeeded.
- aarch64 cross builds used `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld` with `aarch64-unknown-linux-musl`,
  no C cross toolchain (Verified).
- Heavy runs (benchmarks and collision searches) ran in
   `podman run --rm --memory=2g --cpus=2 --security-opt label=disable`
   on `registry.fedoraproject.org/fedora:latest` (glibc 2.43,
   same as the host),
   with binaries and data mounted read-only from the scratchpad.
  `label=disable` avoided relabeling host directories.
- Commands a future session can rerun:
  - `RUSTFLAGS='-C target-feature=+aes,+sse2' cargo build --release --bin difftest` in `scratchpad/gxhash/lab`,
     then `./target/release/difftest`.
  - `node scratchpad/gxhash/run-bench-r2.ts`,
     then `RESULTS=scratchpad/gxhash/results-r2.tsv node scratchpad/gxhash/analyze.ts r2`.
  - `node scratchpad/gxhash/pack-files.ts` rebuilds `data/files.bin` from `git ls-files` (read-only on the repository).
