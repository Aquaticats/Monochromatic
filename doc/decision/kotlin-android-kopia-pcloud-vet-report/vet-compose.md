# Vet: Compose Multiplatform + runComposeUiTest

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-07
Standard:
 choosing-technology skill,
 FULL-VERIFICATION (clone + source-audit + build + run on real targets).
Verdict summary at the bottom.

## What was vetted

- JetBrains/compose-multiplatform (the Gradle plugin,
   components,
   examples,
   docs hub).
- JetBrains/compose-multiplatform-core (the AOSP Jetpack Compose fork where the runtime + ui-test live).
- runComposeUiTest,
   the multiplatform UI-test entry point used to drive composables in tests.

## 1. Source audit

Clones (shallow,
 depth 1):

- `/tmp/agent/compose-mp-vet` -> JetBrains/compose-multiplatform
- `/tmp/agent/compose-mp-core` -> JetBrains/compose-multiplatform-core
- `/tmp/agent/compose-mp-template` -> JetBrains/compose-multiplatform-template

### Repo structure and relation to Google androidx.compose

`compose-multiplatform` is the distribution/tooling repo:
 Gradle plugin (`gradle-plugins/`),
components (`components/resources`,
 navigation),
 examples,
 tutorials,
 CI helpers.
The README at `compose/README.md` states development happens in `compose-multiplatform-core`,
"There Compose Multiplatform team and contributors adopt Jetpack Compose for iOS,
 Desktop and Web targets.
"

`compose-multiplatform-core` is a fork of AOSP androidx.
 Every source file carries
`Copyright <year> The Android Open Source Project` and Apache-2.0 headers
(e.g. `compose/ui/ui-test/src/commonMain/kotlin/androidx/compose/ui/test/ComposeUiTest.kt:1`).
Packages are `androidx.compose.*`.
 So Compose Multiplatform IS Google's Jetpack Compose,
with JetBrains adding non-Android targets (Desktop/iOS/Web/native) on top of Skiko/Skia.

### runComposeUiTest entry points (cited)

Multiplatform expect/actual:

- expect:
   `compose/ui/ui-test/src/commonMain/kotlin/androidx/compose/ui/test/ComposeUiTest.kt:85`
  `expect fun runComposeUiTest(effectContext, runTestContext, testTimeout, block): TestResult`
- Android actual:
   `.../androidMain/.../ComposeUiTest.android.kt:183` delegates to
  `runAndroidComposeUiTest(ComponentActivity::class.java, ...)` (launches a real Activity;
   Espresso/idling-backed).
- Skiko actual (desktop/web/native):
   `.../skikoMain/.../ComposeUiTest.skiko.kt:86` delegates to
  `runSkikoComposeUiTest` -> `SkikoComposeUiTest`,
   which renders OFFSCREEN to an
  `org.jetbrains.skia.Surface` (default 1024x768,
   density 1f).
   No display required.
- Desktop adds `runDesktopComposeUiTest`/`DesktopComposeUiTest`
  (`.../desktopMain/.../ComposeUiTest.desktop.kt:41,67`),
   a `SkikoComposeUiTest` subclass
  with size control + `registerIdlingResource`.

The `ComposeUiTest` interface (`ComposeUiTest.kt:117`) extends `SemanticsNodeInteractionsProvider`
and exposes `mainClock` (MainTestClock),
 `runOnIdle`,
 `runOnUiThread`,
 `waitForIdle`
for deterministic synchronization.
 `assertExists` is a MEMBER of `SemanticsNodeInteraction`
(`SemanticsNodeInteraction.kt:124`);
 `assertIsDisplayed`/`onNodeWithText` are top-level extensions
(`Assertions.kt:32`,
 Finders).

Important API-evolution finding:
 in the latest source (jb-main / 1.12-alpha) the classic
`runComposeUiTest` is `@Deprecated(WARNING)` in favor of
`androidx.compose.ui.test.v2.runComposeUiTest` (v2 uses StandardTestDispatcher to better match
production coroutine behavior).
 Both exist;
 classic still works.
 Migration guide referenced in the deprecation message.

### Supported targets

`compose/ui/ui-test/src` source sets:
 `androidMain`,
 `desktopMain`,
 `iosMain`,
 `macosMain`,
`jsMain`,
 `wasmJsMain`,
 `nativeMain`,
 `skikoMain`,
 plus test source sets
`skikoTest`,
 `desktopTest`,
 `androidDeviceTest`,
 `androidHostTest`,
 `webTest`.
So UI testing is wired for Android (device + Robolectric host),
 Desktop JVM,
 iOS,
 macOS,
 JS,
 Wasm.

### Tests, CI, fuzzing

- ui-test is self-tested:
   `skikoTest/SkikoComposeUiTestTest.kt:41` (`canSetContent`,
   `canAwaitIdle`
  using onNodeWithText/assertIsDisplayed,
   `canDriveAnimationsFromTest`),
   plus
  `androidDeviceTest/ComposeUiTestTest.kt`,
   `ui-test-junit4/.../desktopTest`.
   ~171 `*Test.kt` under ui-test/src.
- Android ui-test supports Robolectric host execution (`androidMain/RobolectricIdlingStrategy.android.kt`,
  `androidHostTest/RobolectricComposeTest.kt`) AND on-device instrumentation.
- CI (public GitHub Actions in core,
   `.github/workflows/compose-tests.yml`):
   jobs for
  Compose Desktop (ubuntu + Xvfb,
   `./gradlew testDesktop`),
   iOS,
   iOS-utils,
   iOS-instrumented (matrix of simulators),
  Web Chrome,
   with screenshot golden artifacts uploaded on failure.
   The distribution repo
  (`compose-multiplatform/.github/workflows`) covers gradle-plugin tests + benchmarks-smoke.
- Fuzzing / property-based testing:
   NONE found in ui-test (no proptest/jqwik/quickcheck/fuzz harness).
  Screenshot/golden testing is the main robustness mechanism.
   Report absence inline.

### Licensing

Apache-2.0 throughout (both repos;
 `LICENSE.txt`,
 AOSP file headers).
 Clean for repo use.

## 2. Maintenance signals (via gh)

- Backing:
   JetBrains.
   Commits + merged PRs in the last weeks are almost all JetBrains staff
  (kropp,
   terrakok,
   MatkovIvan,
   svastven,
   mazunin-v-jb,
   pjBooms,
   Nikita Lipsky,
   Victor Kropp...).
- Activity:
   19.1k stars,
   1401 forks,
   created 2020,
   pushed 2026-06-06 (active daily).
- Release cadence:
   frequent.
   1.11.1 (2026-06-02) latest stable;
   1.11.0 (2026-05-13);
  1.10.
  x patches through Mar 2026;
   1.12.0-alpha01 (2026-05-19) in flight.
   Alpha/beta/rc channels per release.
- PR responsiveness:
   internal PRs merge same-day / 1-2 days (sampled #5604-#5616).
  Some external community PRs sit longer (e.g. #5512 tvOS since 2026-01,
   #5482 since 2025-11) awaiting review.
- Issues:
   GitHub Issues is NOT the real tracker.
   Only ~1 open issue;
   members (kropp,
   ivakub) auto-redirect
  reporters to YouTrack (youtrack.
  jetbrains.
  com,
   project CMP).
   #5482 even adds a PR template requiring a
  YouTrack reference.
   So "1 open issue" is a routing artifact,
   not abandonment.
   Real triage lives in YouTrack.
- Kotlin version alignment:
   TIGHT and is the central operational constraint.
   Since Kotlin 2.0 the Compose
  COMPILER ships with Kotlin (`org.jetbrains.kotlin.plugin.compose`,
   versioned == Kotlin);
   Compose MP ships
  the runtime.
   Compatibility table:
   jetbrains.
  com/help/kotlin-multiplatform-dev/compose-compatibility-and-versioning.
  html.
  Current example stack (May 2026):
   Kotlin 2.3.20,
   Compose 1.10.1,
   AGP 9.2.1,
   Gradle 9.5.0.

## 3. Full verification

Toolchain installed on host:
 Temurin JDK 17 + JDK 21 (via mise),
 Android cmdline-tools 11076708,
platform-android-34/36,
 build-tools 34.0.0/36.0.0.
 Heavy builds on /var/tmp (not tmpfs).
Device:
 Pixel 6 (oriole),
 Android 16 / API 36,
 connected via adb.

### 3a. App scaffold

Based on JetBrains/compose-multiplatform-template (3 modules:
 shared / androidApp / desktopApp / iosApp).
`shared/src/commonMain/kotlin/App.kt` rewritten to a button + counter:
Text("Count:
 N") + Button("Increment") incrementing remembered state.

### 3a-bis. Version finding (why "latest" matters here)

First build used the template's PINNED versions (Compose 1.5.11 / Kotlin 1.9.21 / AGP 8.0.2 / Gradle 8.2.1).
Two findings:
- `compose.uiTest` Gradle DSL accessor does NOT exist in 1.5.11 (added later).
   Worked around with the
  explicit coordinate `org.jetbrains.compose.ui:ui-test:1.5.11`,
   but then:
- the desktop `runComposeUiTest`/`ComposeUiTest` entry point is ABSENT from 1.5.11's ui-test artifact
  (only the finder/assertion DSL ships).
   So the offscreen multiplatform runner matured after 1.5.
  x.
  => The official template is ~2.5 years stale and predates the very API under test.

Rebuilt against the current JetBrains stack:
 Kotlin 2.3.20 / Compose 1.11.1 / AGP 9.2.1 / Gradle 9.5.0,
JDK 21 to run (compile target Java 17),
 compileSdk/targetSdk 36,
 minSdk 24.

### 3a-ter. AGP 9 migration finding (real friction)

AGP 9.0 dropped compatibility between the legacy `com.android.library`/`com.android.application` plugins
and `org.jetbrains.kotlin.multiplatform`:
"The 'com.
android.
library' ... plugin is not compatible with the 'org.
jetbrains.
kotlin.
multiplatform' plugin
since AGP 9.0.
" The official template uses exactly that broken combo.
 The documented bypass
(`android.builtInKotlin=false`,
 `android.newDsl=false`) got past the hard stop but then crashed on the
template's legacy `android { sourceSets[...] }` DSL (ClassCastException DefaultAndroidLibrarySourceSet).
Proper fix (what JetBrains' own current examples do):
 migrate the shared module to
`com.android.kotlin.multiplatform.library` (android config inside `kotlin { android { ... } }`,
 no `androidTarget()`),
and make androidApp a plain `com.android.application` (AGP-9 built-in Kotlin) consuming `:shared`.
Did that migration.
 This is surmountable but is real adoption friction:
 the public template is broken on
the latest AGP and there is no one-line escape.

### 3b. Desktop JVM + runComposeUiTest: PASS

Test `shared/src/desktopTest/kotlin/AppDesktopTest.kt`:
```text
@OptIn(ExperimentalTestApi::class)
class AppDesktopTest {
    @Test fun counterIncrementsOnButtonClick() = runComposeUiTest {
        setContent { App() }
        onNodeWithText("Count: 0").assertExists()
        onNodeWithText("Increment").performClick()
        onNodeWithText("Count: 1").assertExists()
        onNodeWithText("Increment").performClick()
        onNodeWithText("Count: 2").assertExists()
    }
}
```
Dependency:
 `compose.uiTest` accessor (exists in 1.11.1) + `compose.desktop.currentOs`.
Note:
 `assertExists` is a member of SemanticsNodeInteraction (no import);
 importing it as top-level fails to compile.

Command:
```text
JAVA_HOME=<temurin-21> ANDROID_HOME=<sdk> GRADLE_USER_HOME=/var/tmp/compose-vet/gradle-home \
  ./gradlew :shared:desktopTest --no-daemon --console=plain
```
Output:
```text
> Task :shared:desktopTest
BUILD SUCCESSFUL in 10s
```
JUnit XML (`shared/build/test-results/desktopTest/TEST-AppDesktopTest.xml`):
```text
<testsuite name="AppDesktopTest[desktop]" tests="1" skipped="0" failures="0" errors="0" time="0.672">
  <testcase name="counterIncrementsOnButtonClick[desktop]" classname="AppDesktopTest" time="0.67"/>
</testsuite>
```
Ran headless via Skiko offscreen Surface (no Xvfb needed;
 DISPLAY was present but the runner does not use it).

### 3c. Android on Pixel 6 (oriole, Android 16 / API 36): PASS

Build:
```text
./gradlew :androidApp:assembleDebug --no-daemon --console=plain
> BUILD SUCCESSFUL in 30s
APK: androidApp/build/outputs/apk/debug/androidApp-debug.apk (9.4 MB)
```
(Note:
 build prints noisy `KMP Dependencies Resolution Failure` diagnostics from the multiplatform
plugin during metadata resolution;
 assembleDebug still succeeds and produces the APK.
)

Install + launch + confirm running (from HOST adb):
```text
adb -s 1C171FDF600KWW install -r androidApp-debug.apk      -> Success
adb -s 1C171FDF600KWW shell monkey -p com.myapplication.MyApplication -c android.intent.category.LAUNCHER 1
adb -s 1C171FDF600KWW shell dumpsys activity activities | grep ResumedActivity
  -> topResumedActivity=...com.myapplication.MyApplication/com.myapplication.MainActivity
adb -s 1C171FDF600KWW shell pidof com.myapplication.MyApplication  -> 19472 (running)
```

On-device functional check (renders as ComposeView;
 semantics exposed to uiautomator):
```text
uiautomator dump -> text="Count: 0"; clickable Increment button at center ~ (540,130)
adb shell input tap 540 130 -> text="Count: 1"
adb shell input tap 540 130 -> text="Count: 2"
```
So the counter increments on the real phone,
 not just in the desktop test.

Edge-to-edge / camera-cutout fix (found during on-device review):
 targetSdk 36 on Android 16 forces
edge-to-edge,
 so the unmodified app drew "Count:
 0" at y=0,
 under the status bar / punch-hole
(bounds [439,0][641,77]).
 Fix:
- shared `App.kt`:
   root Column gets `Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing)`
  (multiplatform foundation API;
   no-op insets on desktop,
   real insets on Android).
- androidApp `MainActivity`:
   `enableEdgeToEdge()` + `WindowCompat.getInsetsController(...).apply {
  isAppearanceLightStatusBars = true; isAppearanceLightNavigationBars = true }` so the system bar
  icons are dark and legible on the light app surface (device is in dark mode,
   which otherwise washed
  out the bars).
After fix:
   "Count:
   0" bounds moved to [439,128][641,205] (below the status bar/cutout).
   Status + nav
bars visible with dark icons.

Instrumented on-device runComposeUiTest:
 NOT run (optional).
 The desktop runComposeUiTest already exercises
the same multiplatform API;
 an Android instrumented variant (`androidInstrumentedTest` + connectedAndroidTest)
adds source-set wiring under the new KMP library plugin and was out of scope once the API was proven on desktop
and the app was proven on-device.

Cleanup:
 app uninstalled from the phone after verification (see end).

## 4. Alternatives

### UI layer alternatives (vs Compose Multiplatform)

- Slint (incumbent for this repo's desktop).
   Pros:
   Rust-native,
   small footprint,
   already in the stack
  (`cargo:https://github.com/slint-ui/slint` pinned in `mise.toml`),
   good for desktop/embedded.
  Rejection vs Compose MP for shared mobile+desktop UI:
   Slint's Android/iOS support is comparatively
  immature,
   it has its own `.slint` markup DSL rather than the repo's Kotlin/TS,
   and it has no
  semantics-tree UI-test harness equivalent to runComposeUiTest (testing is mostly manual / screenshot).
  It stays the right call for a pure-desktop Rust component,
   not for one shared UI across phone+desktop.
- Flutter.
   Pros:
   most mature cross-platform (Android/iOS/desktop/web),
   excellent tooling,
   strong
  `flutter_test`/WidgetTester.
   Rejection:
   Dart + its own engine/VM is a third language runtime in a
  Kotlin/TS/Rust monorepo;
   no reuse of existing JVM/Kotlin or TS code;
   large toolchain to adopt.
- React Native.
   Pros:
   TS/JS (matches repo strength),
   web code reuse,
   big ecosystem.
   Rejection:
   desktop
  is second-class (react-native-windows/macos;
   no first-class Linux desktop,
   which is this repo's desktop
  target),
   native-bridge perf variability,
   and no offscreen component-test runner comparable to
  runComposeUiTest's deterministic clock/synchronization.
- Native Jetpack Compose (Android-only).
   Pros:
   first-party Google,
   identical API,
   best Android fit.
  Rejection:
   Android ONLY,
   so it abandons desktop/iOS/web entirely.
   Same `androidx.compose` API as
  Compose MP,
   so adopting Compose MP loses nothing on Android while keeping the other targets.

Ranking (for a shared phone+desktop UI in this repo):
 Compose Multiplatform > Flutter > native Jetpack
Compose > Slint > React Native.
 Compose MP wins on Kotlin-native + shared-code + the strongest in-process
test harness;
 Flutter is the closest true-cross-platform rival but costs a new language;
 native Compose is
a strict subset of targets;
 Slint is desktop-strong but mobile-weak and untestable at semantics level;
React Native's desktop weakness is disqualifying for this repo's desktop focus.

### runComposeUiTest alternatives

- Espresso.
   Pros:
   Google's standard Android UI test framework,
   mature,
   on-device.
   Rejection:
   Android-only
  and View-centric;
   it does not test desktop/iOS/web composables.
   runComposeUiTest's Android actual already
  wraps the same idling/Espresso machinery (`ComposeIdlingResource.android.kt`),
   so Espresso is effectively
  a subset,
   not a replacement,
   for a multiplatform codebase.
- Maestro.
   Pros:
   black-box cross-platform E2E (Android/iOS) via declarative YAML flows,
   resilient,
   no code.
  Rejection:
   black-box,
   so no access to the composition/semantics tree,
   no `MainTestClock` control,
   and it
  needs a running device/emulator (no offscreen desktop run).
   Good as an E2E smoke layer on top,
   not as the
  fast deterministic component-test layer runComposeUiTest provides.
- (In-family note) `createComposeRule()` from `ui-test-junit4` is the JUnit4-bound sibling;
   runComposeUiTest
  is the JUnit-independent,
   multiplatform entry point and is the right default for shared/commonTest.

Ranking (for testing shared composables):
 runComposeUiTest > Maestro > Espresso.
 runComposeUiTest is the
only option that runs the same test across all targets,
 offscreen and deterministically;
 Maestro adds value
only as a separate E2E layer;
 Espresso is Android-only and subsumed.

## Friction log (every obstacle hit, in order)

The technologies work,
 but they produced roughly as much grief as value.
 Full list:

1.  Split repos:
     `compose-multiplatform` does NOT contain the runtime/ui-test source;
     it lives in
    `compose-multiplatform-core` (a second clone was needed to audit runComposeUiTest at all).
2.  Official template is stale:
     pins Compose 1.5.11 / Kotlin 1.9.21 / AGP 8.0.2 / Gradle 8.2.1 (~2.5 yrs old).
3.  `compose.uiTest` Gradle DSL accessor does not exist in 1.5.11 -> `Unresolved reference: uiTest`.
    Worked around with explicit coordinate `org.jetbrains.compose.ui:ui-test:1.5.11`.
4.  Even then,
     `runComposeUiTest` / `ComposeUiTest` / `setContent` / `assertExists` are ABSENT from
    1.5.11's ui-test artifact (only the finder/assertion DSL ships).
     The template's pinned version
    literally cannot run the API under test.
     => had to abandon the template's versions.
5.  Moving to the current stack required JDK 21 to RUN AGP 9.2.1 (the repo's own CI Docker uses JDK 21),
    even though compile target stays Java 17.
     Had to install a second JDK.
6.  AGP 9.0 hard break:
     `com.android.library`/`com.android.application` + `org.jetbrains.kotlin.multiplatform`
    is rejected ("not compatible ... since AGP 9.0").
     The official template uses exactly that combo,
    so the template does not build on the current AGP at all.
7.  The documented bypass (`android.builtInKotlin=false`,
     `android.newDsl=false`) cleared the hard stop
    but then threw `ClassCastException: DefaultAndroidLibrarySourceSet_Decorated` on the template's legacy
    `android { sourceSets["main"]... }` DSL.
     Bypass is not actually usable with the template.
8.  Real fix = restructure to match JetBrains' current examples:
    - shared -> `com.android.kotlin.multiplatform.library`,
       android config moved INSIDE
      `kotlin { android { namespace; compileSdk; minSdk; compilerOptions { jvmTarget } } }`,
       drop `androidTarget()`.
    - androidApp -> plain `com.android.application` (AGP-9 built-in Kotlin,
       no kotlin-multiplatform),
      sources moved `src/androidMain` -> `src/main`,
       `AppCompatActivity` -> `ComponentActivity`,
      `Theme.AppCompat` -> `@android:style/Theme.Material...`.
    A non-trivial migration just to compile on the latest toolchain.
9.  API gotcha:
     `assertExists` is a MEMBER of `SemanticsNodeInteraction` (no import);
     `assertIsDisplayed`,
    `onNodeWithText` are top-level extensions.
     Importing `assertExists` as top-level fails to compile.
    This single wrong import was the last thing standing between "looks broken" and "green test".
10. Every Android assemble prints `❌ KMP Dependencies Resolution Failure` diagnostics (non-fatal;
     APK
    still builds),
    noisy and alarming.
11. Edge-to-edge is default on targetSdk 36 / Android 16:
     the app drew "Count:
     0" at y=0,
     under the
    status bar / camera punch-hole.
     Needed `Modifier.windowInsetsPadding(WindowInsets.safeDrawing)`
    in the shared composable + `enableEdgeToEdge()` on Android.
12. System bar icons were light-on-white (device in dark mode) and effectively invisible.
     Needed
    `WindowCompat.getInsetsController(...).isAppearanceLightStatusBars/NavigationBars = true` so the
    status bar and nav bar are legible.
     After this,
     both bars show clearly with dark icons and content
    sits below them (verified by screenshot `screen-bars.png`).
13. Tight version coupling (Kotlin <-> compose-compiler-plugin <-> Compose runtime <-> AGP <-> Gradle):
    mismatches fail fast and the official template/wizard lags the release train.

What worked well (the value side):
- Source is clean Apache-2.0,
   a real AOSP Jetpack Compose fork.
   runComposeUiTest is a well-designed
  multiplatform expect/actual with deterministic synchronization (mainClock,
   waitForIdle) and offscreen
  Skiko rendering needing no display.
   It is self-tested in-repo.
   This was the SMOOTHEST part of the vet.
- Once on the correct current stack + structure:
   desktop runComposeUiTest passes (0.67s),
   and the Android
  APK builds,
   installs,
   launches,
   and the counter increments on a real Pixel 6.
- Strong,
   active JetBrains maintenance;
   fast internal PR merges;
   frequent releases;
   public CI matrix.

## Verdict

Both work.
 Neither is "toast".
 But adoption cost is real and roughly equal to the payoff right now.

Compose Multiplatform:
 VIABLE,
 recommend WITH CAVEATS.
- Verified on both integration boundaries on the latest stack (Kotlin 2.3.20 / Compose 1.11.1 / AGP 9.2.1
  / Gradle 9.5.0):
   desktop JVM UI test green AND a debug APK running on a physical Pixel 6 (Android 16).
- It is genuine Google Jetpack Compose (Apache-2.0) plus JetBrains' Desktop/iOS/Web targets;
   actively
  maintained by JetBrains.
- Caveats that cost real time:
   the official template/wizard is broken on the current AGP 9;
   you must use
  JDK 21,
   migrate to the new `com.android.kotlin.multiplatform.library` plugin,
   and pin a known-good
  (Kotlin,
   Compose,
   AGP,
   Gradle) quad.
   Budget for the AGP-9 migration and for edge-to-edge inset handling.
- If the repo's need is desktop-ONLY,
   the incumbent Slint is lower-friction;
   Compose MP earns its keep
  specifically when one Kotlin UI must span phone + desktop (+ iOS/web).

runComposeUiTest:
 VIABLE,
 recommend.
- The API itself was the least painful component:
   clean multiplatform surface,
   offscreen desktop execution
  (no Xvfb),
   deterministic clock,
   in-repo self-tests.
   Desktop test passed first try once the stack was right.
- Caveats:
   the classic `runComposeUiTest` is `@Deprecated(WARNING)` as of 1.12-alpha in favor of
  `androidx.compose.ui.test.v2.runComposeUiTest` (StandardTestDispatcher semantics) -> plan to adopt v2.
  Watch the member-vs-top-level assertion split.
   On-device instrumented variant not exercised (desktop
  proved the API;
   device proved the app).

Artifacts:
 app under `/var/tmp/compose-vet/app`;
 logs under `/var/tmp/compose-vet/*.log`;
screenshots `/var/tmp/compose-vet/screen-{initial,fixed,bars}.png`;
desktop JUnit XML `/var/tmp/compose-vet/app/shared/build/test-results/desktopTest/`.
