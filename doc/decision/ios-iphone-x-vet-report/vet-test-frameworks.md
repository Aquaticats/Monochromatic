# iOS test frameworks: unit, property, fuzz, mutation, run to green per ecosystem

Vet date:
 2026-06-13.
Scope:
 the in-process test tooling for the iOS survivors,
 the iOS analog of
`../kotlin-android-kopia-pcloud-vet-report/vet-test-frameworks.md`.

The analogy path spelled `../kotlin-android-kopia-pcloud-vet-reports/` when this vet was recorded on 2026-06-13.
The folder became `kotlin-android-kopia-pcloud-vet-report/` on 2026-07-15 (`bd19d57e20`),
so the `../kotlin-android-kopia-pcloud-vet-reports/` spelling no longer resolves.

Test tooling follows the language,
 not the UI framework,
 so this is organized by ecosystem (Rust core,
Kotlin/Native,
 JS/TS) and each ecosystem maps to the survivors that use it.
 Each tool is run to green,
 not just
audited;
 the iOS-specific deltas (running on the `aarch64-apple-ios-sim` target,
 the Kotlin/Native test runner,
the PITest gap on native) are the point.

This matches the Android depth deliberately.
 The Android `vet-test-frameworks.md` ran kotlin.
test and Kotest in
a Podman container (host-side),
 not on the device;
 only the e2e vet was device-bound.
 So running these
host-side or on the simulator target IS the Android depth,
 and the genuinely iOS-specific proofs added here are
`cargo test --target aarch64-apple-ios-sim` and Kotlin/Native `iosSimulatorArm64Test`,
 both executing the test
binary inside the simulator.

## Why serial, and why mutation runs in a Mac container

This vet ran serially on the one Mac rather than fanning out parallel agents.
 The device,
 the codesign keychain
(its search-list mutation is a global,
 added-then-restored operation),
 and the single booted simulator are
single physical resources;
 parallel signing or parallel simulator sessions would corrupt the keychain search
list or race the device.
 That hardware serialism overrides a parallelism preference here.

Per owner directive,
 all mutation testing runs inside a bounded Podman container on the Mac's own Podman
machine,
 never on the host or the device.
 This matches `../../decisions/mutation-testing.md` (moved on 2026-07-15 to
`../../decision/mutation-testing.md`) (Stryker "inside
one restricted Podman container per source file").
 The Rust mutation run below used the Mac's
`podman-machine-default` with explicit `--memory`/`--cpus` bounds.
 The Mac's Podman VM mounts `/Users` but not
`/Volumes/MacData`,
 so the container source must sit on the internal disk.

## Rust core (Dioxus and the shared kopia/music-player cores)

Survivors served:
 Dioxus (its UI is Rust,
 zero FFI to the core) and every track's shared Rust core.
 A tiny crate
`rust-test-ios` (`/Volumes/MacData/ios-vet/rust-test-ios`) holds pure roundtrip logic (`pack_rgb`/`unpack_rgb`,
the kind of pure logic a music-player core carries) with a kotlin-mirrored unit test and a proptest property.

### Unit and property tests, host and iOS-simulator target

`cargo test` on the host (macOS arm64) and then on the `aarch64-apple-ios-sim` target,
 the latter with the test
binary executed inside the booted simulator via a runner:

```sh
export CARGO_TARGET_AARCH64_APPLE_IOS_SIM_RUNNER="xcrun simctl spawn <sim-udid>"
export PROPTEST_DISABLE_FAILURE_PERSISTENCE=1
cargo test --target aarch64-apple-ios-sim
```

Both legs pass:

```txt
----- HOST (macOS arm64) -----
test tests::unit_pack_known_values ... ok
test tests::prop_rgb_roundtrip ... ok
test result: ok. 2 passed; 0 failed; ...
----- iOS SIMULATOR target -----
     Running unittests src/lib.rs (target/aarch64-apple-ios-sim/debug/deps/rust_test_ios-...)
test tests::unit_pack_known_values ... ok
test tests::prop_rgb_roundtrip ... ok
test result: ok. 2 passed; 0 failed; ...
```

So the standard Rust test harness (libtest) and proptest both run on the iOS simulator target,
 not only on the
host.
 The runner is `xcrun simctl spawn`,
 which launches the libtest binary inside the simulator runtime;
 no
extra tooling (cargo-dinghy) was needed.

### Fuzzing (cargo-fuzz, libFuzzer)

A fuzz target over the same roundtrip,
 bounded by time and RSS:

```sh
cargo fuzz run roundtrip -- -max_total_time=20 -rss_limit_mb=2048
```

```txt
#34414012  DONE   cov: 14 ft: 14 corp: 1/1b lim: 4096 exec/s: 1638762 rss: 620Mb
Done 34414012 runs in 21 second(s)
```

34.4 million coverage-guided executions in 21 seconds,
 no crash or panic:
 the property holds under fuzzing.
cargo-fuzz is the same coverage-guided engine the repo already uses (`../../decisions/forbidden-strings-fuzzing.md`
 (moved on 2026-07-15 to `../../decision/forbidden-strings-fuzzing.md`),
`../../handover/resharp-fuzz.md`).
 Fuzzing is host-logic,
 not iOS-target-specific,
 so it runs once on the shared
core,
 the same posture the synthesis doc states.

### Mutation (cargo-mutants, in a Mac Podman container)

```sh
podman run --rm --memory=4g --cpus=4 --volume ~/ios-vet/rust-mut:/work --workdir /work \
  docker.io/library/rust:latest \
  bash -c "cargo install cargo-mutants --locked; cargo mutants --no-shuffle --timeout 60"
```

```txt
Found 24 mutants to test
ok       Unmutated baseline in 4s build + 0s test
MISSED   src/lib.rs:6:44: replace | with ^ in pack_rgb ...
MISSED   src/lib.rs:6:24: replace | with ^ in pack_rgb ...
24 mutants tested in 15s: 2 missed, 22 caught
```

22 of 24 mutants caught by the unit-plus-property suite.
 The 2 "missed" are not a test weakness:
 both replace
`|` with `^` in `pack_rgb`,
 where the three bytes occupy disjoint bit ranges (`r<<16`,
 `g<<8`,
 `b`),
 so OR and
XOR produce identical output.
 They are equivalent mutants,
 which by definition no test can kill.
 That is exactly
the analysis mutation testing exists to surface,
 and it confirms the property test is strong (it killed all 22
non-equivalent mutants,
 including the boundary and arithmetic-operator changes).

## Kotlin/Native (Compose Multiplatform)

Survivor served:
 Compose Multiplatform,
 whose shared logic compiles to Kotlin/Native for iOS.
 The Compose gate
project (`/Volumes/MacData/ios-vet/composegate`,
 Kotlin 2.4.0,
 the same Kotlin the Android vet used) gained a
`commonTest` source set with the iOS-mirrored color logic,
 a kotlin.
test unit test,
 and a kotest-property
`checkAll`,
 run on the `iosSimulatorArm64` target:

```sh
./gradlew :shared:iosSimulatorArm64Test
```

The Kotlin/Native test runner compiled the tests to a native `.kexe`,
 ran it in the simulator,
 and both passed
(from the JUnit-format result `shared/build/test-results/iosSimulatorArm64Test/TEST-ColorTest.xml`):

```xml
<testsuite name="ColorTest" tests="2" skipped="0" failures="0" errors="0" ...>
  <testcase name="unitPackKnownValues[iosSimulatorArm64]" classname="ColorTest" .../>
  <testcase name="propRgbRoundtrip[iosSimulatorArm64]" classname="ColorTest" .../>
```

So kotlin.
test (the JetBrains assertion + annotation layer the Android vet recommended) runs on the iOS target
via the Kotlin/Native test runner,
 and kotest-property's `checkAll` with its `Arb` generators runs natively in
the simulator,
 the same property framework cleared on the JVM in the Android vet,
 now proven on iOS.

### iOS-specific friction (recorded for reproducibility)

- Gradle JDK:
   Gradle 8.14 rejects JDK 26 (the only JDK initially installed) with a bare-version error;
   install a
  supported JDK (Temurin 21 here,
   via mise) and point Gradle at it.
   The Android vet likewise used Temurin 17/21
  for Gradle.
- Duplicate konan distributions:
   two Kotlin/Native prebuilt dists coexist on this Mac (`~/.konan` and
  `/Volumes/MacData/konan`),
   and Kotlin 2.4.0 defaults to `deny` on duplicate platform-klib `unique_name`s.
   Add
  `-Xklib-duplicated-unique-name-strategy=allow-first-with-warning` to the compiler options.
- Suspend bridge:
   kotest-property's `checkAll` is a suspend function,
   so a kotlin.
  test `@Test` must wrap it in
  `runBlocking` (pulling `kotlinx-coroutines-core` into `commonTest`) and the test function must return Unit
  (use a block body,
   not an expression body whose value is the `PropertyContext`).

### Mutation gap on Kotlin/Native

PITest,
 the mutation engine the Android vet found Kotest integrates with,
 is JVM-only:
 it mutates JVM bytecode,
so it cannot mutate Kotlin/Native iOS binaries.
 There is no mature mutation tester for Kotlin/Native.
 So for a
Compose Multiplatform stack,
 mutation coverage of shared logic should run where that same Kotlin also targets
the JVM (a `jvm()`/Android target with PITest),
 or live in the Rust core (cargo-mutants above) for logic shared
across tracks.
 This is a real,
 decision-relevant limitation of the Compose MP test story on iOS,
 not a tooling
oversight.

## JS/TS (React Native, NativeScript, Lynx, and the WebKit shells' web layer)

Survivors served:
 React Native,
 NativeScript,
 Lynx,
 and the web layer of the WKWebView shells (Capacitor,
Cordova,
 Ionic,
 Framework7,
 Onsen,
 Quasar).
 Their business logic is JS/TS tested on host Node,
 the same depth as
the Android container runs.

### Property testing (fast-check)

fast-check is already the repo's accepted property tool (`../../decisions/fast-check.md` (moved on 2026-07-15 to
`../../decision/fast-check.md`)).
 A roundtrip property
over the JS color packing,
 with a falsifiability check (the Android vet's discipline:
 prove the property bites,
not just that it is green):

```txt
roundtrip property: PASS (1000 runs)
falsifiability: broken impl shrank to Counterexample: [0,0,1]
```

1000 runs green,
 and a deliberately broken implementation (dropping the blue channel) shrank to the minimal
counterexample `[0,0,1]`,
 confirming the property actually exercises the code and fast-check's shrinking works,
the same property-plus-shrinking capability proven for Kotest on the JVM in the Android vet.
 fast-check 4.8.0,
pure JS,
 runs under Node and Bun.

### Mutation testing (Stryker)

Already vetted and accepted by the owner (`../../decisions/mutation-testing.md` (moved on 2026-07-15 to
`../../decision/mutation-testing.md`)):
 Stryker with the command
runner,
 one restricted Podman container per source file.
 Not re-vetted or re-run here.
 The only iOS-relevant
note is that it satisfies the same mutation-in-a-container rule applied to cargo-mutants above;
 it runs on the
Mac's Podman,
 not the device.

## Bottom-ranked tracks (tooling posture, not run)

Effort is matched to ranking.
 The two lowest tracks are documented at the posture level rather than spending
Flutter-SDK or .
NET-workload setup plus device time on full suites for frameworks already at the bottom of the
combined ranking.

- Flutter/Dart (bottom of the allowed-and-non-allowed combined ranking):
   `flutter_test` is the in-framework
  unit/widget layer and is mature;
   Dart property testing is thin (no fast-check-class library),
   and Dart
  mutation testing is weak.
   A Flutter stack would lean its generative and mutation coverage on the shared Rust
  core,
   with `flutter_test` for widget behavior.
   Consistent with the synthesis doc's note that Dart mutation
  testing is weak.
- .
  NET/C# (MAUI,
   Avalonia,
   Uno;
   ruled out as the kopia host by the Go-plus-Mono device finding):
   xUnit/NUnit
  plus FsCheck (property) plus Stryker.
  NET (mutation) is a mature stack on the JVM-like CLR,
   but the track is
  last,
   so it is recorded as posture only.

## Bottom line

- Rust core:
   unit and proptest pass on the host and on the `aarch64-apple-ios-sim` target (in the simulator);
  cargo-fuzz ran 34.4M executions clean;
   cargo-mutants caught 22 of 24 mutants in a Mac Podman container,
   the 2
  missed being provable equivalent mutants.
   The full Rust QA stack runs for iOS.
- Kotlin/Native (Compose MP):
   kotlin.
  test and kotest-property both run green on `iosSimulatorArm64`;
   mutation has
  a real gap on native (PITest is JVM-only).
- JS/TS:
   fast-check property plus shrinking proven;
   Stryker mutation already owner-vetted and container-run.
- Bottom tracks (Flutter/Dart,
   .
  NET) documented at posture level.

The testing axis is now run,
 not just enumerated as a stage-2 roadmap,
 across the ecosystems the top survivors
use,
 at the same depth as the Android vet.
