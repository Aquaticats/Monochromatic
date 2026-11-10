# Ktor (ktorio/ktor) vetting report

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Subject:
 Ktor,
 the Kotlin coroutine-based async client and server framework,
 vetted for possible
adoption per the Monochromatic `choosing-technology` skill at FULL-VERIFICATION standard.

Date:
 2026-06-07.
 Vetting agent:
 Claude Code.

## Verdict (up front)

Ktor is a genuinely production-grade,
 actively maintained,
 JetBrains-backed framework that builds,
runs,
 and serves correctly.
 It passed full verification:
 a real Gradle project pulled Ktor 3.5.0 from
Maven Central,
 started a CIO server,
 and a CIO client fetched a known string and matched it.

The strong reservations are not about Ktor's quality.
 They are:

1.  Stack fit.
     Monochromatic is TypeScript-dominant (2901 `.ts`,
     105 `.rs`,
     zero Kotlin/Gradle/Java).
    Adopting Ktor drags in an entire JVM/Kotlin toolchain (JDK,
     Gradle,
     Kotlin compiler) that has no
    foothold in this repo.
     This is a constraint-fit problem,
     not a Ktor defect.
2.  Test-rigor gap.
     No fuzzing,
     no property-based testing,
     no mutation testing,
     and no concurrency
    model-checking (lincheck) anywhere in the tree,
     despite the framework being heavily concurrent.
3.  CI opacity.
     The real test CI runs on JetBrains-internal TeamCity;
     public GitHub Actions do not run
    the test suite.
     You cannot inspect green/red test runs from the public repo.

Recommendation:
 technically sound and safe to adopt for a JVM/Kotlin or Kotlin-Multiplatform project.
For this TypeScript/Bun/Rust monorepo,
 the toolchain cost is the dominant factor and argues against
adoption absent a concrete Kotlin workload.

## 1. Clone

```bash
mkdir --parents /tmp/agent; chmod 700 /tmp/agent
gh repo clone ktorio/ktor /tmp/agent/ktor-vet -- --depth 1
```

HEAD:
 `9d125375bd52408f52a785719c956a42bbed7cba`,
 dated 2026-06-04,
 author `fru1tworld`.
In-repo version:
 `3.6.0-SNAPSHOT` (dev).
 Latest released on Maven Central:
 `3.5.0` (2026-05-14).
License:
 Apache 2.0.

## 2. Source audit (files and lines cited)

### Execution core: the pipeline

- `ktor-utils/common/src/io/ktor/util/pipeline/Pipeline.kt` (538 lines).
   The extensible
  phase-and-interceptor pipeline.
   Uses `kotlinx.atomicfu` for the interceptor cache
  (`Pipeline.kt:73`),
   multiplatform `expect fun pipelineStartCoroutineUninterceptedOrReturn`
  (`Pipeline.kt:20`).
   `mergePhases` carries an explicit O(n^2)-worst-case comment with the rationale
  (`Pipeline.kt:205-207`).
   KDoc on every public member.
- `ktor-utils/common/src/io/ktor/util/pipeline/SuspendFunctionGun.kt` (207 lines).
   The actual
  coroutine state machine that drives interceptors with minimal allocation.
   Hand-rolled use of
  `suspendCoroutineUninterceptedOrReturn`,
   `COROUTINE_SUSPENDED`,
   and a saved-continuation stack
  (`SuspendFunctionGun.kt:115-175`).
   It documents its own non-volatile,
   unsynchronized debug fields
  and the deliberate "do not crash here" stance during debugger stack walks
  (`SuspendFunctionGun.kt:70-84`).
   Conditional redispatch avoids a redundant dispatch when already on
  the right thread (`SuspendFunctionGun.kt:182-188`).
   This is careful,
   performance-focused code.

### Routing

- `ktor-server/ktor-server-core/common/src/io/ktor/server/routing/RoutingResolveContext.kt` (255).
  Depth-first resolution over a routing trie with quality-based best-match scoring
  (`handleRoute`,
   `RoutingResolveContext.kt:101-177`).
   The path parser is a single linear scan with
  no regex (`parse`,
   `RoutingResolveContext.kt:59-84`);
   URL-decode failure is mapped to
  `BadRequestException` (`RoutingResolveContext.kt:54-56`).
   `handleRoute` recurses over children,
  which is a bounded structural tree walk (route depth),
   not unbounded text recursion.
- Routing module spans 15+ files (`RoutingNode.kt` 431,
   `RouteSelector.kt` 843,
   `RoutingRoot.kt` 192).

### Server engine (CIO)

- `ktor-server/ktor-server-cio/common/src/io/ktor/server/cio/backend/ServerPipeline.kt` (265).
  `startServerConnectionPipeline` (`ServerPipeline.kt:43`) runs one coroutine per connection,
   uses a
  bounded actor `Channel(capacity = 3)`,
   an idle timeout,
   supports HTTP pipelining (handler may run
  concurrently),
   and defends against oversized headers via `TooLongLineException`
  (`ServerPipeline.kt:67-70`).
   Tagged logger via `KtorSimpleLogger` (`ServerPipeline.kt:26`).
- `CIOApplicationEngine.kt` (265).
   Modest,
   readable engine files.

### Client core

- `ktor-client/ktor-client-core/common/src/io/ktor/client/HttpClient.kt` (1494).
   Plugin-based,
  multiplatform,
   with very extensive KDoc and runnable examples (`HttpClient.kt:25-120`).
   atomicfu
  for client state (`HttpClient.kt:17`).

### Module sizes (code-quality signal)

Largest production files:
 `ktor-openapi-schema/.../Operation.kt` 1652,
 `HttpClient.kt` 1494,
`client sse/builders.kt` 1265,
 `ktor-http/.../Mimes.kt` 1019,
 `StaticContent.kt` 969,
`ByteReadChannelOperations.kt` 927,
 `LockFreeLinkedList.kt` 855,
 `RouteSelector.kt` 843.
Total source ~152k LoC across ~1559 non-test `.kt` files.
 Modularity is good:
 most core files are
200 to 900 lines.
 A few large files exist but they are cohesive (MIME table,
 OpenAPI schema).

### Error-handling shape

Throw-based with a rich typed hierarchy:
 46+ exception classes across core modules,
 HTTP-status
mapped (`BadRequestException`,
 `NotFoundException`,
 `PayloadTooLargeException`,
`HttpRequestTimeoutException`,
 `ConnectTimeoutException`,
 `MissingRequestParameterException`,
 etc.).
This matches the repo's own PP4 preference (custom error classes,
 throw over codes/null).

### Multiplatform targets

JVM,
 JS,
 `wasmJs`,
 `wasmWasi`,
 and a full Native set declared in `build-logic`:
 `linuxX64`,
`linuxArm64`,
 `macosX64`,
 `macosArm64`,
 `mingwX64` (Windows),
 `iosArm64/iosX64/iosSimulatorArm64`,
`watchos*`,
 `tvos*`,
 `androidNativeArm64`.
 The client targets browser/Wasm/Native;
 the server is
primarily JVM with a Native CIO engine.
 Per-platform socket selectors exist
(`ktor-network/nix/.../SelectUtilsNix.kt` 266,
 `windows/.../SocketUtilsWindows.kt` 300) and a
hand-rolled multiplatform TLS stack (`TLSClientHandshake.kt` 567,
 `Certificates.kt` 583) for
non-JVM targets,
 which is a meaningful security-sensitive surface to be aware of.

## 3. Tests, CI, coverage, and the fuzzing/property/mutation question

- Test volume:
   735 test `.kt` files.
   Multiplatform test source roots (`common/test`,
   `jvm/test`,
  `jvmAndPosix/test`,
   `posix/test`,
   `nix/test`,
   `windows/test`,
   `web/test`).
   Dedicated test-harness
  modules:
   `ktor-server-test-host`,
   `ktor-server-test-suites`,
   `ktor-server-test-base`,
  `ktor-client-tests`.
   Tests are conventional example-based unit and integration tests.
- Coverage tooling:
   Kover `0.9.8` is wired in `gradle/libs.versions.toml` and applied (for example in
  `ktor-io/build.gradle.kts`).
   No public coverage report or badge was found in the repo.
- Fuzzing / property / mutation / concurrency testing:
   ABSENT.
   A whole-tree search found no
  `io.kotest`,
   `net.jqwik`,
   `jazzer`,
   `lincheck`,
   `pitest`,
   or `stryker`.
   The single "kotest" hit was
  a code-comment citation in `build-settings-logic/build.gradle.kts:36`;
   `@Property` and
  `@PropertyKey` matches were Ktor's own DI annotation and an IntelliJ i18n annotation,
   not jqwik.
  For a framework whose core is a concurrent coroutine state machine,
   the absence of property-based
  tests and lincheck concurrency model-checking is the single biggest test-rigor gap.
- CI:
   `teamcity.default.properties` is present and `KTOR-XXXX` references throughout commits/PRs point
  to JetBrains' YouTrack + TeamCity.
   Public `.github/workflows/` contains only `automations.yml`,
  `close-waiting-for-reply.yml`,
   `devcontainer.yml`,
   `gradle-wrapper-validation.yml`,
   `junie.yml`.
  None of them run `gradle test`/`check`.
   So the authoritative test CI is JetBrains-internal and not
  publicly inspectable.
   `gradle-wrapper-validation.yml` is a positive supply-chain signal.

## 4. FULL VERIFICATION (the integration boundary)

Goal:
 prove a downstream consumer can pull Ktor from Maven Central,
 run a server,
 and have the client
talk to it,
 end to end,
 in a bounded throwaway container.

Setup (all files under `/var/tmp/ktor-verify`,
 on disk,
 not tmpfs):

- `build.gradle.kts`:
   Kotlin JVM plugin `2.3.21` (matching Ktor 3.5.0's own stdlib),
  `application` plugin,
   `mavenCentral()`,
   dependencies
  `io.ktor:ktor-server-core:3.5.0`,
   `ktor-server-cio:3.5.0`,
   `ktor-client-core:3.5.0`,
  `ktor-client-cio:3.5.0`,
   `jvmToolchain(21)`.
- `gradle/wrapper/`:
   reused Ktor's wrapper bootstrap jar;
   `distributionUrl` pointed at the public
  `services.gradle.org` `gradle-9.5.1-bin.zip` (not the JetBrains redirector,
   to prove a public path).
- `src/main/kotlin/Main.kt`:
   `embeddedServer(CIO, host=127.0.0.1, port=8080){ routing { get("/")
  { call.respondText("KTOR_VET_OK_a7f3c9") } } }`,
   then `HttpClient(CIO)` GETs it with a 30x200ms
  readiness retry loop and asserts the body equals the token;
   prints `VERIFY_PASS`/`VERIFY_FAIL`,
  exits 0/1;
   finally stops the client and server.

Exact run command:

```bash
podman run --rm --memory=6g --cpus=4 \
  --volume /var/tmp/ktor-verify:/work:Z --workdir /work \
  --env GRADLE_USER_HOME=/work/.gradle-home \
  docker.io/library/eclipse-temurin:21-jdk \
  bash ./gradlew --no-daemon --console=plain run
```

Result:
 PASS.

```text
> Task :run
VERIFY_PASS: server returned expected body=[KTOR_VET_OK_a7f3c9]

BUILD SUCCESSFUL in 6s
2 actionable tasks: 2 executed
WRAPPER_EXIT=0
```

Resolved artifacts (from the Gradle cache,
 confirming real Maven Central downloads):
`ktor-server-core-jvm-3.5.0.jar`,
 `ktor-server-cio-jvm-3.5.0.jar`,
 `ktor-client-core-jvm-3.5.0.jar`,
`ktor-client-cio-jvm-3.5.0.jar`.
 Full log saved at `/tmp/agent/ktor-verify-build.log`.

Iteration notes (honest record):

1.  First container run hit `exit 126` / "Permission denied" on `gradlew`.
     Cause:
     Fedora SELinux
    bind-mount.
     Fix:
     `:Z` relabel on the volume plus invoking via `bash ./gradlew`.
     This is a host
    environment quirk,
     not a Ktor issue.
2.  Second run reached `compileKotlin` (proving the full toolchain and all four Ktor artifacts
    resolved from Maven Central) and failed only on my own API misuse:
     I wrote `install(Routing){}`
    instead of the `routing {}` DSL.
     Fixed and re-ran.
     Third run:
     green.
     The compile-then-run path
    was therefore exercised,
     including a real (uncached) dependency resolution on the first attempt.

## 5. Maintenance signals

- Popularity:
   14,420 stars,
   1,258 forks.
   Created 2015 (~11 years old).
   Apache 2.0.
- Recency:
   last push 2026-06-06 (one day before vetting).
   Last default-branch commits add real
  features (async DNS resolvers for CIO `#5577`,
   HTTP/3 for Netty `#5527`,
   Kotlin `Uuid` conversion
  `#5679`).
   Renovate keeps dependencies current.
- Release cadence:
   roughly monthly.
   3.5.0 (2026-05-18),
   3.4.3 (Apr),
   3.4.2 (Mar),
   3.4.1 (Mar),
  3.4.0 (Jan),
   3.3.3 (Nov 2025),
   back through a steady stream.
   ~12 releases in ~10 months.
- Throughput:
   65 open PRs,
   4054 closed/merged.
   Recent merges land within days,
   by both JetBrains
  staff (osipxd,
   bjhham,
   nomisRev) and community contributors (zibet27).
- Issue responsiveness:
   GitHub shows only 156 open / 1381 closed issues because the primary tracker
  is JetBrains YouTrack (`KTOR-XXXX`).
   GitHub issues still get substantive maintainer triage:
  bjhham (CONTRIBUTOR),
   osipxd,
   and zibet27 (COLLABORATOR) reply with PR links,
   target versions,
   and
  workarounds within days (for example `#5619`,
   `#5589`,
   `#5518`,
   `#5516`).
   Flaky tests are tracked
  openly and honestly (`#5478`,
   `#5477`,
   `#5476`).
- Maintainer concentration:
   heavily JetBrains.
   Top contributors e5l (1204),
   orangy (819),
  rsinukov1 (438),
   osipxd (398),
   bjhham (228),
   Stexxe (173),
   marychatte (118).
   Bus factor sits with
  JetBrains as an organization.
   Given JetBrains owns Kotlin itself,
   this is more a strength
  (aligned commercial incentive,
   long horizon) than a typical single-maintainer risk,
   but it is
  concentration nonetheless.
- Open caveat:
   per-release regressions appear and are fixed in patches (for example an open 3.5.0
  Android client regression `#5678`,
   unanswered for a few days at vetting time).
   Normal for a
  fast-moving framework,
   but worth pinning patch versions and watching the changelog.

Maintenance state:
 actively maintained with strong corporate backing and a large,
 triaged backlog.

## 6. Alternatives (with concrete rejection reasons)

These are the realistic substitutes if the need were a JVM/Kotlin HTTP server-plus-client.
 Ktor is
the subject under vet;
 each alternative is rejected for a specific incompatibility.

- Spring Boot (WebMVC/WebFlux).
   Pros:
   vast ecosystem,
   batteries-included (security,
   data,
   actuator),
  huge hiring pool.
   Cons / rejection:
   JVM-only (no Kotlin Multiplatform Native/JS/Wasm client),
  annotation- and reflection-driven with a heavy classpath and slower cold start;
   not coroutine-first
  (WebFlux is Reactor/`Mono`/`Flux`,
   with coroutine bindings bolted on).
   Far heavier than Ktor's
  install-a-plugin model for a thin async service.
- http4k.
   Pros:
   idiomatic Kotlin "Server as a Function",
   immutable,
   no reflection,
   trivially testable
  (apps are `HttpHandler` functions).
   Cons / rejection:
   Kotlin/JVM-targeted only (no KMP Native/JS
  targets like Ktor's client),
   smaller community and a smaller maintainer team than JetBrains-backed
  Ktor,
   and a synchronous filter/function core rather than Ktor's suspend-native pipeline.
- Vert.
  x.
   Pros:
   very high throughput,
   mature event-loop,
   polyglot.
   Cons / rejection:
   core programming
  model is reactive Verticles with callbacks/`Future` (Kotlin coroutine bindings exist but are a
  layer on top),
   JVM-only,
   and operationally more complex than Ktor's plain coroutine model.
- Plain JDK `com.sun.net.httpserver` (Java 21).
   Pros:
   zero dependencies,
   in the JDK.
   Cons /
  rejection:
   no routing DSL,
   no content negotiation,
   no HTTP client integration,
   no multiplatform,
  minimal TLS ergonomics.
   You would hand-roll exactly the surface Ktor already provides,
   which the
  `choosing-technology` skill explicitly warns against (tool-fit before first-principles).

Ranking for a JVM/Kotlin async-service need:
 Ktor > http4k > Vert.
x > Spring Boot > plain JDK.
Ktor over http4k for multiplatform reach and JetBrains backing;
 http4k over Vert.
x for a simpler,
testable functional model;
 Vert.
x over Spring Boot for a lighter footprint when raw throughput
matters;
 Spring Boot over plain JDK because hand-rolling routing/negotiation is the worst option.
For this TypeScript/Bun monorepo specifically,
 every JVM option (Ktor included) loses to staying
in-runtime;
 the constraint that decides it is "do not introduce a second toolchain,
" not features.

## 7. Constraints honored during this vet

No further Claude/agent sessions were spawned.
 No tracked Monochromatic repo files were modified or
committed.
 All work happened under `/tmp/agent/` and a throwaway `/var/tmp/ktor-verify/` exercised in
a bounded podman container (`--memory=6g --cpus=4`,
 build on `/var/tmp` disk,
 not tmpfs).
