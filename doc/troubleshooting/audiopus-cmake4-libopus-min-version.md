# `audiopus_sys 0.2.2` fails under CMake 4.x: bundled libopus declares `cmake_minimum_required(VERSION 3.1)`

Building the `opus` crate (0.3.1 -> `audiopus_sys 0.2.2`) on a machine with CMake
4.
x aborts in the `audiopus_sys` build script:
 it compiles the bundled libopus
source through the `cmake` crate,
 and that libopus `CMakeLists.txt` declares
`cmake_minimum_required(VERSION 3.1)`.
 CMake 4.0 removed compatibility with
`cmake_minimum_required` values below 3.5,
 so configure fails before any C code is
compiled.
 This surfaced porting `music-player` to Windows (CMake 4.3.3);
 it would
hit any platform with CMake >= 4 that builds bundled libopus (no system libopus).

## Symptom

`cargo build` fails compiling `audiopus_sys v0.2.2` with:

```text
error: failed to run custom build command for `audiopus_sys v0.2.2`
  ...
  cargo:info=Building Opus via CMake.
  -- Configuring incomplete, errors occurred!
  --- stderr
  CMake Error at CMakeLists.txt:1 (cmake_minimum_required):
    Compatibility with CMake < 3.5 has been removed from CMake.

    Update the VERSION argument <min> value.  Or, use the <min>...<max> syntax
    to tell CMake that the project requires at least <min> but has been updated
    to work with policies introduced by <max> or earlier.

    Or, add -DCMAKE_POLICY_VERSION_MINIMUM=3.5 to try configuring anyway.
```

It happens only when `audiopus_sys` builds the BUNDLED libopus (the default when no
system libopus is found via pkg-config,
 e.g. on Windows).
 On Linux/macOS where
pkg-config finds a system libopus,
 `audiopus_sys` links it and never invokes CMake,
so the error does not appear there.

## Root cause

Three facts combine;
 none is a bug on its own.

### 1. `audiopus_sys 0.2.2` bundles a 2021 libopus and builds it with CMake

`audiopus_sys` vendors xiph/opus as a git submodule.
 Version 0.2.2 (the newest
ever published;
 the series is 0.1.0 to 0.2.2 with no 0.2.3 or 0.3.
x) pins commit
`7b05f44f4baadf34d8d1073f4ff69f1806d5cdb4` (2021-03-03,
 between libopus v1.3.1 and
v1.4).
 Its build script builds that source with the `cmake` crate ("Building Opus
via CMake" in the stdout above).

### 2. That bundled libopus declares `cmake_minimum_required(VERSION 3.1)`

`opus/CMakeLists.txt:1` in the published `audiopus_sys-0.2.2.crate`:

```cmake
cmake_minimum_required(VERSION 3.1)
```

xiph/opus kept `VERSION 3.1` from v1.3.1 through v1.5.1 and only raised it to 3.16
at v1.5.2:

```text
xiph/opus CMakeLists.txt:1 cmake_minimum_required by tag:
  v1.3.1 -> 3.1     v1.5   -> 3.1
  v1.4   -> 3.1     v1.5.1 -> 3.1
                    v1.5.2 -> 3.16    v1.6 / v1.6.1 / master -> 3.16
```

`audiopus_sys 0.2.2` pins the 3.1-era commit,
 so it is stuck on `VERSION 3.1`.

### 3. CMake 4.0 removed compatibility with `VERSION < 3.5`

CMake 4.0 deletes the old-policy compatibility path;
 a project that asks for a
minimum below 3.5 is rejected with the error above (the message names the escape
hatch,
 `-DCMAKE_POLICY_VERSION_MINIMUM=3.5`,
 which CMake also reads from the
`CMAKE_POLICY_VERSION_MINIMUM` environment variable).
 So CMake 4.3.3 + the 3.1-era
bundled libopus = hard configure failure.

## Verification

Versions under test:

- Windows 10 (build 19044),
   `x86_64-pc-windows-msvc`,
   CMake 4.3.3,
   Visual Studio +
  LLVM,
   rustc nightly.
- `opus 0.3.1` -> `audiopus_sys 0.2.2` (bundled libopus commit `7b05f44`).
- `opus` crate = SpaceManiac/opus-rs;
   `audiopus_sys` = lakelezz/audiopus_sys;
  libopus = xiph/opus.

Fails:
 a bare `cargo build` of the `music-player` package on the Windows box aborts
at `audiopus_sys` configure with the CMake `cmake_minimum_required` error above.

Works (both confirmed by a full `cargo build` to `Finished` on the Windows box):

```text
1. opus 0.3.1 + CMAKE_POLICY_VERSION_MINIMUM=3.5 in the environment:
     cargo build -> Finished dev profile in 42.87s   (libopus built, linked via lld-link)
2. opus pinned to opus-rs git HEAD (opusic-sys 0.7.3, libopus 1.6.1):
     cargo build -> Finished, with NO CMAKE_POLICY_VERSION_MINIMUM set
```

## Verified workarounds

### Pin `opus` to opus-rs git HEAD (applied)

opus-rs HEAD (post-0.3.1) replaced `audiopus_sys` with the maintained
`opusic-sys 0.7.3`,
 which bundles libopus **1.6.1** whose `CMakeLists.txt:1` is
`cmake_minimum_required(VERSION 3.16)` and so configures cleanly under CMake 4 with
no override.
 In the consuming `Cargo.toml`:

```toml
opus = { git = "https://github.com/SpaceManiac/opus-rs", rev = "559876660603dc8079a053e03e6438766f669e69" }
```

The public `opus` API (`Decoder`,
 `Channels`,
 `decode_float`,
 `reset_state`,
`Error`) is unchanged at this rev,
 so no consumer code changes.

Tradeoffs:
 it pins an UNPUBLISHED git revision (no crates.
io release carries the
opusic-sys backend yet),
 so `cargo update` cannot move it and it must be reverted
to a `version` once a release ships.
 More importantly,
 `opusic-sys`'s `bundled`
feature is `default = ["bundled"]` and the `opus` crate pulls it with default
features,
 so feature unification forces libopus to be **compiled from source via
CMake on every platform**,
 not just Windows.
 Linux and macOS therefore also need
`cmake` (plus a C compiler and a generator:
 `make` or `ninja`) at build time,
 where
`audiopus_sys` previously linked a system libopus via pkg-config.
 The non-bundled
(system-libopus) path is unreachable without forking the `opus` crate to set
`default-features = false` on `opusic-sys`.

### Set `CMAKE_POLICY_VERSION_MINIMUM=3.5` (alternative, keeps `opus 0.3.1`)

Keep `opus = "0.3"` and export `CMAKE_POLICY_VERSION_MINIMUM=3.5` in the build
environment.
 CMake then treats the bundled libopus's `VERSION 3.1` as 3.5 and
configures it.
 Verified working (build 1 above).

Tradeoffs:
 it papers over the old bundled libopus rather than upgrading it (the
2021 libopus,
 with whatever CVEs/bugs fixed since,
 keeps shipping).
 It is,
 however,
surgical:
 it changes nothing on Linux/macOS (no CMake runs there for opus,
 which
links system libopus),
 so it only affects the Windows build.
 It relies on a CMake
migration shim that a future CMake may also remove.

## What does not work

- **Bumping `opus`/`audiopus_sys` to a newer published version.
  ** There is none
  that helps:
   the latest `opus` is 0.3.1,
   it depends on `audiopus_sys ^0.2.0`,
   and
  the newest `audiopus_sys` is 0.2.2 (no 0.2.3/0.3.
  x exists),
   which bundles the
  3.1-era libopus.
   A pure version bump cannot reach a `>= 3.5` libopus.
- **Relying on a system libopus on Windows to skip the CMake build.
  ** Windows has
  no pkg-config libopus by default,
   so `audiopus_sys` falls back to the bundled
  CMake build,
   which is exactly the failing path.
   (Providing one via vcpkg would
  avoid CMake but adds a separate dependency-management story.
  )
- **Disabling `opusic-sys`'s `bundled` feature from the consumer** (to link system
  libopus on Linux/macOS).
   Cargo feature unification is additive:
   the `opus` crate
  enables `opusic-sys` with default features (`bundled`),
   so a consumer's
  `default-features = false` cannot turn it back off.

## Upstream filing decision

No `.out-of-scope/` exemption matches CMake,
 libopus/xiph,
 audiopus_sys,
 opusic-sys,
or opus-rs;
 checked and found none.
 The 6-constraint check:

1.  **Upstream's fault?
    ** Not really,
     and spread across projects.
     CMake 4 dropping
    `< 3.5` compatibility is an intended,
     announced removal.
     xiph/opus already raised
    its minimum to 3.16 (v1.5.2+).
     The stale link is `audiopus_sys` pinning a 2021
    libopus,
     and that crate appears unmaintained (no release since 0.2.2).
     The
    user-facing failure is the interaction,
     not a single defect.
2.  **Can upstream fix it?
    ** `audiopus_sys` could bump its libopus submodule;
     opus-rs
    already did the equivalent by switching to `opusic-sys` (PR #27).
     So it is
    fixable,
     and is effectively already fixed upstream in opus-rs HEAD.
3.  **Supporting this use case?
    ** opusic-sys explicitly bundles and builds modern
    libopus;
     opus-rs HEAD adopts it.
     The combination is supported there.
4.  **Welcome a contribution?
    ** Not assessed (not reached):
     the fix already exists in
    opus-rs HEAD,
     so there is no patch to contribute.
5.  **Will they fix it?
    ** opus-rs already has (the opusic-sys backend is on HEAD,
    awaiting a release).
     audiopus_sys looks dormant;
     filing there is low-value.
6.  **Prototyped a minimal fix?
    ** The consumer-side fix (pin to opus-rs HEAD,
     or set
    the policy env var) fully resolves it;
     both are verified above.
     No upstream patch
    is needed.

Decision:
 do not file.
 The fix exists upstream (opus-rs HEAD via opusic-sys) and is
applied at our boundary by the pin;
 the only remaining upstream action would be to
nudge a crates.
io release of opus-rs,
 which is a release-timing matter,
 not a bug
report.
 Recorded here so the pin can be reverted to a published `version` once that
release lands.
