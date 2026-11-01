# The music player's track fingerprint is XXH3-64 from `twox-hash`

## Status

Accepted 2026-10-14 by the repository owner, on issue #545.
Two owner calls shaped it:
first "just migrate to XXH3-128 (no choosing-technology vet needed)",
then, replacing the width,
"if `twox-hash` provides XXH3-64, use XXH3-64 instead of XXH3-128".
`twox-hash` provides it (see "Verified crate facts"), so the fingerprint stays 64 bits.
Evidence for the algorithm is the monorepo manager's vet,
[`tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md`](../audit/tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md),
which this record reads as evidence only;
the monorepo manager's own choice is
[`monorepo-manager-cache-key-hash.md`](monorepo-manager-cache-key-hash.md).
Supersedes the `gxhash` choice recorded in
[`../troubleshooting/gxhash-aes-target-feature.md`](../troubleshooting/gxhash-aes-target-feature.md)
(now marked historical).

## Context

- The desktop and Android peak caches key every row on a `u64` fingerprint of (path, size, mtime).
  Until this change it was `gxhash64` with seed 0 over the path bytes followed by the size and mtime
  as fixed-width little-endian integers.
- `gxhash` 3.5.0 needs the `aes` CPU target feature at build time, has no software fallback,
  and raises SIGILL on a CPU without it.
  The vet found seed-independent collisions between inputs that differ by one byte,
  at 1,024, 2,048, and 4,096 bytes under three seeds.
- Issue #545 asked for a fingerprint hash with run-time CPU feature selection, no build flag,
  a fixed seed policy, and a 128-bit width.

## Decision

- The fingerprint is XXH3-64 from `twox-hash`, through the one-shot `XxHash3_64::oneshot`
  (default seed 0).
- Seed 0 only.
  XXH3 seeds are not domain separators, so the domain tag lives in the hashed bytes.
- The hashed bytes are, in order:
  the fixed tag `monochromatic.music-player.track-fingerprint.v1` (47 bytes),
  the path length as 8 little-endian bytes,
  the path bytes,
  the size as 8 little-endian bytes,
  and the mtime nanoseconds as 16 little-endian bytes.
  The length prefix and the fixed-width integers make the encoding unambiguous.
- One function, `track_fingerprint` in `package/music-player/truepeak-core/src/fingerprint.rs`
  (behind the new `fingerprint` feature), owns the layout and the hash.
  Desktop (`desktop-app/src/peakcache.rs`) and the Android JNI entry
  (`android-app/rust/src/fingerprint.rs`) call it and keep no material builder of their own,
  so identical inputs hash identically on both by construction.
- The width stays 64 bits end to end:
  Rust `u64`, JNI `jlong`, Kotlin `Long`, SQLite `INTEGER`.
  No layer changes type.
- The dependency is `twox-hash` at the requirement `2.1.4` with `default-features = false` and the
  features `std` and `xxhash3_64`, optional, in `truepeak-core` only.
  It is not pinned to an exact release (owner's call);
  `Cargo.lock` records the resolved release and the known-answer tests guard the digests.
- No other hash crate is in the dependency graph of the fingerprint, not even as a dev-dependency.
- Every `-C target-feature=+aes` setting is removed:
  the three tables in `desktop-app/.cargo/config.toml`
  (the Windows linker line stays, another branch owns it)
  and the whole `android-app/rust/.cargo/config.toml`.
  No CI file, mise task, or other config set it.

## The 128-bit criterion was waived

Issue #545 listed a 128-bit width as a criterion.
The owner waived it and chose to keep the old width so that no layer changes type.
A 64-bit digest has a higher accidental-collision chance than the 128-bit one the issue asked for.
For n tracks in one library the probability that any two share a fingerprint is
`p = 1 - exp(-n(n-1) / 2^65)`, computed with `expm1`:

- n = 1,000: 2.7e-14 (about 1 in 3.7e13).
- n = 10,000: 2.7e-12 (about 1 in 3.7e11).
- n = 100,000: 2.7e-10 (about 1 in 3.7e9).
- n = 1,000,000: 2.7e-8 (about 1 in 3.7e7).
- n = 10,000,000: 2.7e-6 (about 1 in 3.7e5).

The 128-bit digest would give `n(n-1) / 2^129`, which is 1.5e-27 at one million tracks.
`gxhash64` had the same 64-bit width, so the accidental-collision bound is unchanged from before this
decision and weaker only than the issue's criterion.
A collision gives one track the other's cached gain until that row is replaced.
The cache is local and has no resistance to crafted collisions, as in the monorepo manager's record.

## Verified crate facts

Read from the `twox-hash` 2.1.4 source in the Cargo registry
(`~/.cargo/registry/src/*/twox-hash-2.1.4/`, identical `src/` to the upstream tag `v2.1.4`):

- `Cargo.toml` defines the features `xxhash3_64`, `std`, and `alloc`;
  the defaults are `random`, `xxhash32`, `xxhash64`, `xxhash3_64`, `xxhash3_128`, and `std`,
  so `default-features = false` drops `rand` and the other algorithms.
- `src/lib.rs:36-42` declares `pub mod xxhash3_64` and `pub use xxhash3_64::Hasher as XxHash3_64`
  behind `feature = "xxhash3_64"`.
- `src/xxhash3_64.rs:34` is `pub fn oneshot(input: &[u8]) -> u64`.
- `src/xxhash3.rs:39` is `pub const DEFAULT_SEED: u64 = 0`.
- `src/xxhash3/large.rs:103-116` selects NEON on aarch64 and AVX2 or SSE2 on x86_64 at run time,
  only with the `std` feature.
  Inputs of 240 bytes or fewer take scalar code with no kernel dispatch.
- The workspace lock resolves 2.1.5, because `turso_core` already depends on `twox-hash` (`>=2.1.1`);
  its `src/` differs from 2.1.4 in four files (a documented speedup of the one-shot entry points),
  and the known-answer tests below pass on it.

## Consequences

- True-peak caches re-key once.
  `SCHEMA_VERSION` in `truepeak-core/src/policy.rs` goes from 1 to 2.
  A row written under version 1 holds a `gxhash` fingerprint, can never match a current read,
  and `DecisionCache::open` deletes every row below the current version
  (`DELETE FROM decisions WHERE schema_version < ?1`) and logs how many.
  A miss re-measures the track.
  Rows of a newer version survive a downgrade and are never matched.
- Android keeps a second store keyed by fingerprint, the duration cache.
  Its file is renamed `track-durations-v2.tsv`, and the retired `track-durations.tsv` and its `.tmp`
  sibling are deleted on first load.
- `doc/troubleshooting/gxhash-aes-target-feature.md` stays as a historical record of external facts
  about `gxhash` and the std detection macro, with the date and the supersession noted.
- Android cross-compilation no longer needs a per-target rustflags file.
- Run-time dispatch is the crate's, not the repository's:
  no startup capability check or build flag exists to maintain.

## Known-answer provenance

Every expected digest in `truepeak-core/src/fingerprint_tests.rs` and
`NativeBridgeTest.fingerprintMatchesTheReferenceKnownAnswersOnDevice` is output of the xxHash reference
C implementation,
`Cyan4973/xxHash` tag `v0.8.3`, commit `e626a72bc2321cd320e953a0ccf1584cad60f363`,
compiled with `gcc -O1` and `#define XXH_INLINE_ALL`,
not of `twox-hash`.
The vectors cover six materials (including a non-ASCII path, a size of `u64::MAX`,
an empty path, and materials of 379 and 1,179 bytes that reach the vector kernels)
and the raw length-class boundaries 0, 1, 3, 4, 8, 9, 16, 17, 128, 129, 240, 241, 512, 1023, 1024,
1025, and 2048.
The empty input digest `2d06800538d394c2` is the published XXH3-64 value of the empty string.

## Known-answer harness

The constants come from this program, built with
`gcc -O1 -I <xxHash checkout> kat.c -o kat && ./kat`:

```c
// kat.c
#define XXH_INLINE_ALL
#include "xxhash.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char TAG[] = "monochromatic.music-player.track-fingerprint.v1";

static void le(unsigned char *out, unsigned __int128 value, int bytes) {
  for (int i = 0; i < bytes; i++) { out[i] = (unsigned char)(value & 0xff); value >>= 8; }
}

static void material_hash(const char *name, const unsigned char *path, size_t path_len,
                          unsigned long long size, unsigned __int128 mtime) {
  size_t tag_len = strlen(TAG);
  size_t total = tag_len + 8 + path_len + 8 + 16;
  unsigned char *buf = malloc(total);
  size_t at = 0;
  memcpy(buf + at, TAG, tag_len); at += tag_len;
  le(buf + at, path_len, 8); at += 8;
  memcpy(buf + at, path, path_len); at += path_len;
  le(buf + at, size, 8); at += 8;
  le(buf + at, mtime, 16); at += 16;
  printf("material %s len=%zu xxh3_64=0x%016llx\n", name, total, (unsigned long long)XXH3_64bits(buf, total));
  free(buf);
}

static unsigned char *long_path(size_t n) {
  unsigned char *p = malloc(n);
  p[0] = '/';
  for (size_t i = 1; i < n; i++) p[i] = (unsigned char)('a' + ((i - 1) % 26));
  return p;
}

int main(void) {
  printf("tag_len=%zu\n", strlen(TAG));
  material_hash("p1", (const unsigned char *)"/music/invented/a.flac", 22, 12345678ULL,
                (unsigned __int128)1700000000123456789ULL);
  material_hash("p2", (const unsigned char *)"", 0, 0ULL, 0);
  unsigned __int128 big = (((unsigned __int128)0x0123456789abcdefULL) << 64) | 0xfedcba9876543210ULL;
  material_hash("p3", (const unsigned char *)"/musique/\xc3\xa9t\xc3\xa9/chanson.ogg",
                strlen("/musique/\xc3\xa9t\xc3\xa9/chanson.ogg"), 0xffffffffffffffffULL, big);
  material_hash("p6", (const unsigned char *)"/musique/\xc3\xa9t\xc3\xa9/chanson.ogg",
                strlen("/musique/\xc3\xa9t\xc3\xa9/chanson.ogg"),
                0xffffffffffffffffULL, (unsigned __int128)0x7fffffffffffffffULL);
  unsigned char *p4 = long_path(300);
  material_hash("p4", p4, 300, 987654321ULL, (unsigned __int128)1600000000000000001ULL);
  unsigned char *p5 = long_path(1100);
  material_hash("p5", p5, 1100, 4294967296ULL, (unsigned __int128)1700000000999999999ULL);
  size_t lens[] = {0,1,3,4,8,9,16,17,128,129,240,241,512,1023,1024,1025,2048};
  for (size_t k = 0; k < sizeof(lens)/sizeof(lens[0]); k++) {
    size_t n = lens[k];
    unsigned char *d = malloc(n + 1);
    for (size_t i = 0; i < n; i++) d[i] = (unsigned char)((i * 131 + 17) & 0xff);
    printf("raw len=%zu xxh3_64=0x%016llx\n", n, (unsigned long long)XXH3_64bits(d, n));
    free(d);
  }
  return 0;
}
```


## Verification

All runs below were made on 2026-10-14 on an x86_64 Linux host with a nightly Rust toolchain
(`rustc 1.101.0-nightly 2026-10-11`).

Tests and lints, fresh after the last code change:

- `truepeak-core`: `mise run //package/music-player/truepeak-core:test` runs 80 tests, 80 passed;
  `lint:clippy` clean.
  The same suite also passes with the crate's kernels forced to scalar and to SSE2
  (`RUSTFLAGS="--cfg _internal_xxhash3_force_scalar"` and `..._force_sse2`), and on exactly `twox-hash` 2.1.4
  (the lock otherwise resolves 2.1.5).
- Desktop: `desktop-app:test` runs 1407 tests, 1407 passed, 1 skipped, in four of six runs on the branch
  (the final tree and the three runs before it);
  `lint:clippy` clean.
  The two earlier runs each failed one timing-based test
  (`measure::tests::two_sweeps_of_one_load_decode_a_track_only_once`, then
  `refresh_tests::flow::a_file_replaced_twice_while_a_sweep_runs_ends_with_the_last_content_measured`)
  and the next run passed.
  The base commit passed 1406 of 1406 (1 skipped) in its one run,
  and a newer `main` passed 1444 of 1444 in another.
  The cause of the two failures is not established.
- Android: `raw::app:testDebugUnitTest` 1251 tests, 0 failures, 0 errors, 0 skipped (summed from the XML);
  `lintDebug` and detekt clean;
  `mise run //package/music-player/android-app:build:native` builds `libmusicplayer_native.so` for
  `arm64-v8a` and `x86_64` with no rustflags file;
  `lint:clippy` for both ABIs clean.
- On a headful x86_64 emulator (Pixel 9 Pro Fold AVD copy, `-gpu host`), `NativeBridgeTest` fingerprint tests
  (the known answers and the determinism test) passed through the real JNI entry: `OK (2 tests)`.
  The whole `NativeBridgeTest` class ran 10 tests with 1 failure,
  `aaudioOutputLatencyOnDevice`, which passes when run alone and does not touch the fingerprint.

Real behavior of the built desktop app, headless against an invented three-track library with an isolated
config directory:

- The base build (gxhash) wrote three rows with `schema_version` 1.
- The new build opened that same database, logged `stale decision rows removed on open removed=3`,
  missed all three tracks, measured them, and wrote three rows with `schema_version` 2;
  none of the old fingerprints remained.
- A second run of the new build opened the database with no stale rows, answered
  `cache get hit` for all three tracks, and decoded nothing (`skip: already exact`).
- The three new fingerprints equal the reference C implementation's `XXH3_64bits` over the material built from the
  real files' paths, sizes, and mtimes, so the whole pipeline matches the reference.

Mutation checks (one textual mutant at a time, whole suite):

- `truepeak-core` mutants, all killed: domain tag changed, domain tag removed, nonzero seed, size field removed,
  mtime placed before size, path length prefix removed, mtime truncated to 8 bytes,
  digest truncated to 32 bits, `SCHEMA_VERSION` left at 1
  (killed at compile time by the constant assertion), stale cleanup removed, cleanup deleting newer rows,
  cleanup deleting current rows.
- Desktop wrapper mutants, both killed: size and mtime passed in each other's slots, path dropped.
- Android Kotlin mutant, killed: the current duration file name set to the retired name.
- Android JNI mutant, killed on the emulator: size and mtime swapped in `fingerprint.rs`
  (`plain path expected:<7674390513079081654> but was:<-4271213150185527448>`).
- Survives: removing the call to `measureDeleteLegacyDurationFiles` from `TrackMeasures.loadDurations`;
  the deletion function is unit-tested, its call site is not.

Cost of one fingerprint (`track_fingerprint` including the material buffer, against the retired
`gxhash64` path with its old layout), 5,000 invented tracks with a mean path of 74 bytes,
inside `podman run --memory=2g --cpus=2`, release build with LTO, 8 rounds of 21 samples of 40 passes:

- `gxhash64`: per 5,000-track pass minimum 166.8 us, median 202.5 us, maximum 484.5 us.
- XXH3-64: per 5,000-track pass minimum 87.6 us, median 107.4 us, maximum 132.1 us.
- Run-to-run spread on unchanged input is large (the maximum is more than twice the minimum for `gxhash64`),
  so the comparison rests on the minimum and the median together:
  XXH3-64 is about 1.9 times faster on both, about 17.5 ns against 33.4 ns per fingerprint at the minimum.
  Either way a 5,000-track library fingerprints in well under a millisecond.

Not verified:

- aarch64 and NEON: no aarch64 CPU or emulator was run, so the NEON kernel and an arm64 device have no known-answer
  run (inputs of 240 bytes or fewer take scalar code with no kernel dispatch,
  and the six material cases include two above that size).
- macOS and Windows builds of the desktop app were not built; removing `+aes` only removes a requirement there.
- No physical Android device was used.

## Rejected

- `gxhash`:
  needs `+aes`, no software fallback, SIGILL on a CPU without AES,
  and collided on one-byte-different inputs in the vet.
- XXH3-128 (the owner's first call, then replaced):
  the 128-bit width criterion of #545 was waived by the owner in favor of keeping every layer's type.
- Another XXH3 crate, or a second hash crate as a cross-check:
  the owner chose `twox-hash` alone, and all XXH3 crates emit identical digests.
- A seed as a domain separator:
  XXH3 seeds are not domain separators.
- A Kotlin copy of the hash for Android:
  a second implementation to keep bit-identical with the Rust one.
- A duplicate fingerprint in each app:
  the single shared function gives bit-parity by construction and lets one host test pin the Android
  hashing, which cannot run host tests itself (its `ndk-sys` dependency refuses to compile off-Android).
