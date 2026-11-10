# Technology vetting: kotlinx.fuzz

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Vetted against the `choosing-technology` skill FULL-VERIFICATION standard.
Date of vetting:
 2026-06-07.

## Verdict

Do not adopt.
 **kotlinx.
fuzz is disqualified:
 it cannot be installed or built today.
**
Its sole artifact repository (`plan-maven.apal-research.com`) no longer resolves in
public DNS (NXDOMAIN),
 and nothing is mirrored to Maven Central or the Gradle Plugin
Portal.
 Following the project README in a clean environment fails at plugin resolution,
and building from source fails at the same dead host.
 Per the skill,
 a candidate that
cannot be built or run via any reproducible path is disqualified by that fact.

The library is also pre-1.
x in everything but its version string:
 research-org owned
(`JetBrains-Research`,
 the PLAN / Applied Program Analysis Lab),
 JVM-only,
 JDK 8 pinned,
config "subject to change",
 last release ~14 months ago,
 last `main` commit ~10 months
ago,
 and a 3-person maintainer team.

## Repository identity correction

The task named `Kotlin/kotlinx.fuzz`;
 that path does not exist (404).
 The real repo is
`JetBrains-Research/kotlinx.fuzz`.
 This matters:
 it is a **research-lab** project under
JetBrains-Research,
 not an official Kotlin Foundation / JetBrains-product library.
 The
`kotlinx.` prefix and `org.jetbrains` group ID give it more official appearance than its
ownership and publication channel warrant.

- Cloned:
   `gh repo clone JetBrains-Research/kotlinx.fuzz /tmp/agent/kotlinx-fuzz-vet -- --depth 1`
- License:
   Apache-2.0.
   Stars:
   123.
   Forks:
   7.
   Open issues:
   6.
- Created 2024-09-09;
   last push 2025-11-13 (a closed KMP PR branch);
   last `main`
  commit 2025-08-07.

## Source audit (files + lines)

Multi-module Gradle build.
 Modules:
 `kotlinx.fuzz.api`,
 `.engine`,
 `.jazzer`,
 `.gradle`,
`.junit`,
 `.serialization`,
 plus `.test` (integration) and `.examples`.

### Engine / Jazzer integration boundary

- `kotlinx.fuzz.engine/.../KFuzzEngine.kt:6-36`: clean engine SPI
  (`initialise` / `runTarget` / `finishExecution`),
   Jazzer is one implementation.
- `kotlinx.fuzz.jazzer/.../JazzerLauncher.kt` is the real integration.
   It binds to
  **internal** Jazzer driver classes,
   not Jazzer's public API:
  - imports `com.code_intelligence.jazzer.driver.{FuzzTargetHolder,FuzzTargetRunner,
    LifecycleMethodsInvoker,Opt}`,
     `...agent.AgentInstaller`,
     `...utils.Log`
    (lines 3-8).
  - `:78-90` builds raw libFuzzer args (`-rss_limit_mb`,
     `-artifact_prefix`,
    `-max_total_time` derived from `maxFuzzTime`).
  - `:106-111` `FuzzTargetRunner.registerFatalFindingDeterminatorForJUnit { ... }` then
    `FuzzTargetRunner.startLibFuzzer(libFuzzerArgs)`: libFuzzer-backed run.
  - `:150-164` `Opt.instrumentationIncludes.setIfDefault(...)`,
     custom-hook wiring,
    `AgentInstaller.install(...)`.
  Binding to Jazzer driver internals is why the project depends on a **forked,
     republished
  Jazzer** (`org.jetbrains:jazzer:0.0.5`,
     version catalog `plan-jazzer`) rather than
  upstream `com.code-intelligence:jazzer`.
     Upstream does not expose these internals as a
  consumable dependency.
     This is a structural coupling/fragility risk and the proximate
  cause of the install failure (the fork lives only on the dead PLAN maven repo).

### Gradle plugin

- `kotlinx.fuzz.gradle/.../KFuzzPlugin.kt:36-41` hard-codes `pluginVersion = "1.0.0"` and
  injects `org.jetbrains:kotlinx.fuzz.{api,junit,jazzer}:1.0.0` as test dependencies on
  apply.
   So merely applying the plugin pulls 1.0.0 artifacts from the dead repo.
- `:71-100` registers `fuzz` task (extends `Test`,
   JUnit Platform engine `kotlinx.fuzz`),
  `:102-121` `regression` task.
   `:195-229` JaCoCo coverage report generation.
- DSL property is `maxFuzzTimePerTarget` (`FuzzConfigDSL.kt:53`),
   mapping to
  `TargetConfig.maxFuzzTime` (`TargetConfig.kt:20-24`).
   NOTE:
   the README "Configuration
  options" section calls it `maxSingleTargetFuzzTime`,
   which does not exist in source (stale docs).

### Tests and CI

- Unit tests:
   51 `@Test` across api/engine/gradle/junit/serialization (config builder,
  CharacterSet,
   KFuzzerImpl,
   DSL,
   JUnit engine,
   serialization round-trips).
- Integration:
   20 `@KFuzzTest` targets in `kotlinx.fuzz.test` + `kotlinx.fuzz.examples`
  (hooks,
   protobuf,
   real-user-code sample).
- CI `.github/workflows/gradle.yml`:
   matrix `{ubuntu,windows,macos} x {x86_64,arm64}`,
  runs `./gradlew test`,
   `build`,
   and `:kotlinx.fuzz.test:fuzz`.
   Reasonable breadth,
  but every job depends on the now-dead PLAN maven repo to resolve the Jazzer fork,
   so CI
  itself would no longer pass on a clean checkout.
- Coverage:
   JaCoCo is wired for the *user's* fuzzing reports;
   no published coverage badge
  for the library's own test suite.

### Maturity markers

- README:
  18: "works only for JVM and requires JDK 8 ... built using Kotlin 2.0.21" (the
  code has since moved to 2.2.0;
   README is stale).
- README:
  149: "Design,
   implementation and default values of configuration properties are
  subject to change in the future releases.
  "
- No Kotlin `@RequiresOptIn`/`@Experimental` gate on the kotlinx.
  fuzz API itself
  (the only `ExperimentalSerializationApi` opt-ins are kotlinx.
  serialization's).
  Instability is prose-only,
   not enforced.
- 15 TODO/FIXME lines across `.kt` (low).
   Supported target:
   **JVM only** (KMP requested
  in issues #73/#78;
   PR #82 attempted KMP support and was closed unmerged 2025-11-13).

## Publication status (disqualifying)

- README setup adds only `https://plan-maven.apal-research.com` (PLAN lab) +
  the kotlin-ide-plugin-dependencies space repo.
   `apal-research.com` returns **NXDOMAIN**
  from both `8.8.8.8` and `1.1.1.1`.
   The hosting domain is gone.
- Maven Central:
   `org.jetbrains:kotlinx.fuzz.jazzer:1.0.0` → 404;
   forked
  `org.jetbrains:jazzer:0.0.5` → 404.
   Upstream `com.code-intelligence:jazzer` → 200
  (only the upstream,
   which the project does not use).
- Gradle Plugin Portal:
   not published.
   The deploy workflow's "Publish gradle plugin to
  Gradle plugin portal" step is **commented out** (`deploy.yml`),
   and commit
  `disable gradle plugin portal publishing for now` (2025-04-09) confirms it.
- Undocumented backend:
   issue #72 reveals the PLAN maven was a front for GitHub Packages
  `maven.pkg.github.com/plan-research/kotlin-maven/`.
   That endpoint returns 401 (GitHub
  Packages requires authentication for Maven reads even when public).
   The README never
  documents this URL,
   and consuming it needs a `read:packages` PAT (a normal `repo`-scoped
  token returns 401).
   So even the backdoor is auth-gated and undocumented.

## Maintenance health

- Releases (all in a 2-month burst,
   then silence):
   0.1.0 (2025-02-05),
   0.2.0,
   0.2.1,
  0.2.2 (2025-03-04),
   1.0.0 (2025-04-09,
   latest).
   No release in ~14 months.
- Commits:
   last on `main` 2025-08-07 ("Kotlin 2.2.0 and Gradle 8.14.3");
   ~10 months stale.
  ~795 commits total.
- Maintainer concentration:
   3 core authors `ilma4` (250),
   `FerrumBrain` (248),
  `AbdullinAM` (247),
   then `DLochmelis33` (49) and one drive-by.
   Classic small academic
  team;
   high bus-factor risk.
- Issue/PR responsiveness:
   was healthy during the active window (PRs #60-#81 reviewed and
  merged with low latency through mid-2025).
   Since then it has gone quiet:
   open issues are
  all feature requests (KMP,
   Maven plugin,
   dynamic tests),
   and a trivial README typo fix
  (PR #83,
   opened 2026-03-24) has sat open and unmerged.
   State:
   **active during 2025 H1,
  effectively dormant since ~2025 H2.
  **
- The tool did genuinely find real bugs when live (`doc/Trophy list.md`:
   multiple
  kotlinx.
  serialization,
   kotlinx-datetime,
   kotlinx-io,
   kotlinx.
  cli issues filed upstream).
  So the engine was technically real;
   the blocker is purely distribution/maintenance.

## FULL VERIFICATION (bounded podman)

Environment:
 `docker.io/library/eclipse-temurin:21-jdk`,
 `--memory=8g --cpus=4`,
 build
on disk under `/var/tmp` (LUKS-backed,
 not tmpfs),
 bind mount `:Z`.
 Minimal consumer
project written per the README;
 per-target fuzz time capped at 10s via
`maxFuzzTimePerTarget = 10.seconds`;
 container-level `timeout` as an outer bound.

### Path 1: documented README path (consume published 1.0.0)

Project:
 `settings.gradle.kts` (pluginManagement:
 gradlePluginPortal + PLAN maven + space),
`build.gradle.kts` (`kotlin("jvm") 2.0.21`,
 `id("org.jetbrains.kotlinx.fuzz") 1.0.0`,
`testRuntimeOnly("org.jetbrains:kotlinx.fuzz.jazzer:1.0.0")`,
 `fuzzConfig{...}`),
 and one
`@KFuzzTest` over a deliberately buggy `parsePairs` ("k=v;
..." → `substring(0,-1)` throws
on a token without `=`).

Command:

```bash
podman run --rm --memory=8g --cpus=4 \
  --volume /var/tmp/kxfuzz-verify:/work:Z --workdir /work \
  --env GRADLE_USER_HOME=/work/.gradle \
  docker.io/library/eclipse-temurin:21-jdk \
  bash -lc 'timeout 600 bash gradlew --no-daemon --console=plain fuzz'
```

Result:
 Gradle 8.14.3 downloaded and ran;
 **BUILD FAILED in 15s** (exit 1):

```text
Plugin [id: 'org.jetbrains.kotlinx.fuzz', version: '1.0.0'] was not found in any of the following sources:
- Gradle Core Plugins (...)
- Plugin Repositories (could not resolve plugin artifact
  'org.jetbrains.kotlinx.fuzz:org.jetbrains.kotlinx.fuzz.gradle.plugin:1.0.0')
  Searched in: Gradle Central Plugin Repository,
  maven(https://plan-maven.apal-research.com),
  maven2(https://maven.pkg.jetbrains.space/.../kotlin-ide-plugin-dependencies)
```

The plugin marker is unresolvable.
 The buggy target is never reached because the build
cannot get past plugin resolution.

### Path 2: build from source (the clone)

Every module depends on the Jazzer fork via `libs.plan.jazzer` (`org.jetbrains:jazzer:0.0.5`),
declared in `gradle/libs.versions.toml:37-39` and wired through
`buildSrc/.../kotlinx.fuzz.src-module.gradle.kts:13` (the PLAN maven repo).
 A throwaway
project resolving just that artifact:

```bash
podman run --rm --memory=8g --cpus=4 \
  --volume /var/tmp/kxfuzz-fork-check:/work:Z --volume /var/tmp/kxfuzz-verify/.gradle:/ghome:Z \
  --workdir /work --env GRADLE_USER_HOME=/ghome \
  docker.io/library/eclipse-temurin:21-jdk \
  bash -lc 'timeout 300 bash gradlew --no-daemon --console=plain resolveFork'
```

Result:
 **BUILD FAILED in 5s** (exit 1):

```text
Could not resolve org.jetbrains:jazzer:0.0.5.
> Could not GET 'https://plan-maven.apal-research.com/org/jetbrains/jazzer/0.0.5/jazzer-0.0.5.pom'.
  > plan-maven.apal-research.com: Name or service not known
```

Both the published path and the source path are blocked by the same dead host.
 No
reproducible validation path exists.
 The crash-detection capability could not be exercised
because the tool could not be assembled at all.

## Alternatives (named with concrete rejection reasons)

The realistic fallback for coverage-guided JVM/Kotlin fuzzing is **Jazzer directly**: it
is what kotlinx.
fuzz wraps,
 and unlike kotlinx.
fuzz it is actually installable.

1. **Jazzer directly** (`com.code-intelligence:jazzer`,
    `jazzer-junit`).
   Pros:
    the actual coverage-guided (libFuzzer + sanitizers) engine;
    on Maven Central
   (verified 200);
    JUnit5 `@FuzzTest` integration;
    the upstream kotlinx.
   fuzz itself relies
   on it.
   Rejection reason:
    Java-first ergonomics: you consume `FuzzedDataProvider` instead of
   kotlinx.
   fuzz's typed `KFuzzer` API,
    there is no Gradle `fuzz`/`regression` task,
    no
   crash-dedup/reproducer-`.kt` generation,
    and JVM-only.
    You lose kotlinx.
   fuzz's Kotlin
   conveniences,
    but you gain a maintained,
    installable engine.
    If coverage-guided fuzzing
   is a hard requirement,
    this is the pick,
    not kotlinx.
   fuzz.

2. **jqwik** (`net.jqwik:jqwik`).
   Pros:
    mature JUnit5 property-based testing with shrinking;
    on Maven Central;
    stable.
   Rejection reason:
    property-based,
    **not** coverage-guided.
    Inputs come from random/
   exhaustive generators with no libFuzzer corpus evolution or instrumentation feedback,
    so
   it will not drive deep multi-branch paths (the `int()%2 / %3 / %31` style guarded
   crashes) the way a coverage-guided fuzzer does.
    Kotlin support is Java-centric.

3. **Kotest property testing** (`io.kotest:kotest-property`).
   Pros:
    idiomatic Kotlin `Arb`/`checkAll` API,
    best Kotlin ergonomics of the three,
    on
   Maven Central,
    actively maintained.
   Rejection reason:
    same class limitation as jqwik: property-based generation with
   shrinking,
    no coverage instrumentation or corpus feedback.
    Good for shallow invariant
   checking,
    not for guided crash discovery.
    Not a fuzzer.

## Files inspected

- `/tmp/agent/kotlinx-fuzz-vet/README.md`,
   `doc/How to get started.md`,
   `doc/Trophy list.md`
- `gradle/libs.versions.toml`,
   `settings.gradle.kts`,
   `build.gradle.kts`,
  `buildSrc/src/main/kotlin/kotlinx.fuzz.src-module.gradle.kts`,
  `gradle/wrapper/gradle-wrapper.properties`
- `kotlinx.fuzz.jazzer/.../JazzerLauncher.kt`,
   `kotlinx.fuzz.engine/.../KFuzzEngine.kt`,
  `kotlinx.fuzz.gradle/.../KFuzzPlugin.kt`,
   `kotlinx.fuzz.api/.../KFuzzer.kt`,
  `.../config/TargetConfig.kt`,
   `.../config/GlobalConfig.kt`,
   `kotlinx.fuzz.api/.../KFuzzTest.kt`
- `.github/workflows/{gradle,deploy}.yml`
- Verification projects:
   `/var/tmp/kxfuzz-verify`,
   `/var/tmp/kxfuzz-fork-check`
