# Vet: Jazzer (com.code-intelligence:jazzer / jazzer-junit) as the JVM fuzzer

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-07.
 Standard:
 choosing-technology,
 FULL-VERIFICATION.
Context:
 replacement for the disqualified kotlinx.
fuzz;
 target is a Kotlin/JVM Android app,
fuzzing pure JVM business logic in CI (no device,
 container only).

## Verdict

Recommended as the JVM fuzzing pick.
 It is the only candidate surveyed that is a true
coverage-guided fuzzer for the JVM,
 it resolves cleanly from Maven Central,
 and it passed
full container verification (built a Gradle/Kotlin project,
 ran a bounded fuzz session,
found the planted crash,
 and emitted a reproducer).

## 1. Source audit (clone: /tmp/agent/jazzer-vet, main @ 50a0e8f2, committed 2026-03-06)

JUnit5 `@FuzzTest` integration:

- `src/main/java/com/code_intelligence/jazzer/junit/FuzzTest.java`
  A JUnit5 meta-annotation:
   `@ParameterizedTest` + `@ArgumentsSource(SeedArgumentsProvider)` +
  `@ExtendWith(FuzzTestExtensions.class)` + a global `@ResourceLock` (fuzz tests can't run in
  parallel because the last finding lives in a global).
   Two modes:
   regression (default,
   runs the
  seed corpus as ordinary parameterized cases) and fuzzing (when env `JAZZER_FUZZ` is non-empty).
  `maxDuration()` default `"5m"`,
   `maxExecutions()` cap,
   `lifecycle()` PER_TEST/PER_EXECUTION.
  Accepts `byte[]`,
   `FuzzedDataProvider`,
   or reflectively-built typed params.
- `src/main/java/com/code_intelligence/jazzer/junit/FuzzingArgumentsProvider.java`
  Emits marked arguments only when `isFuzzing(context)`;
   otherwise empty (so regression mode runs).
- `src/main/java/com/code_intelligence/jazzer/junit/FuzzTestExecutor.java`
  Builds the libFuzzer argv:
   `-max_total_time` from `maxDuration`,
   `-runs` from `maxExecutions`,
  `-rss_limit_mb=0`,
   optional `-use_value_profile=1`,
   JUnit `@Timeout` -> `-timeout=`,
  `-artifact_prefix=<inputs dir or project root>` (this is where reproducers land),
   corpus/seed
  dirs.
   Runs `FuzzTargetRunner.startLibFuzzer(...)`;
   on a finding wraps it as
  `FuzzTestFindingException` (execute(),
   line 342).
- `src/main/java/com/code_intelligence/jazzer/junit/FuzzTestExtensions.java`
  `InvocationInterceptor` that installs the agent (`configureAndInstallAgent`),
   forwards
  user-provided seeds into the fuzzer (`addSeed`),
   and rethrows findings as JUnit failures.

FuzzedDataProvider:

- `src/main/java/com/code_intelligence/jazzer/api/FuzzedDataProvider.java`
  Clean public interface:
   `consumeBoolean/Byte/Short/Int/Long/.../consumeRemainingAsString/Bytes`,
  bounded variants (`consumeInt(min,max)`),
   array consumers.
- `src/main/java/com/code_intelligence/jazzer/driver/FuzzedDataProviderImpl.java`
  Native-backed (JNI `jazzer_fuzzed_data_provider` loaded via RulesJni;
   C++
  `fuzzed_data_provider.cpp`).
   `withJavaData` / `withNativeData` / `setNativeData`;
   AutoCloseable.

Agent install:
 `src/main/java/com/code_intelligence/jazzer/agent/AgentInstaller.java` uses
`net.bytebuddy.agent.ByteBuddyAgent.install()` (runtime self-attach).
 On JDK 21 self-attach is
blocked by default,
 so the consuming project must pass `-Djdk.attach.allowAttachSelf=true`
(handled in the verification build below).
 Apache-2.0;
 all source headers "Code Intelligence GmbH".

Tests/CI present:
 extensive `tests/` and `examples/junit/` (e.g. `ThrowingFuzzTest`,
`ValueProfileFuzzTest`,
 `MutatorFuzzTest`,
 lifecycle tests),
 Bazel build,
 a `selffuzz/` module
(Jazzer fuzzing itself).
 Mutation/fuzz evidence is the product itself plus self-fuzz harnesses.

## 2. Maven Central availability (the kotlinx.fuzz blocker was a dead artifact host)

Both artifacts resolve on repo1.
maven.
org:

- `com.code-intelligence:jazzer`: versions up to 0.30.0,
   metadata lastUpdated 2026-02-24.
- `com.code-intelligence:jazzer-junit`: versions up to 0.30.0,
   lastUpdated 2026-02-24.
- `jazzer-junit:0.30.0` POM transitive deps:
   `jazzer-api`,
   `jazzer` (agent + native driver),
  `junit-jupiter-api/params` 5.9.0,
   `junit-platform-commons/launcher` 1.9.0 (baseline,
   overridable
  via junit-bom).
   Verified live during the build:
   dependencies downloaded and the native libFuzzer
  driver loaded and ran.
   Unlike kotlinx.
  fuzz,
   Jazzer actually resolves and installs.

## 3. Maintenance signals (gh, 2026-06-07)

- Backing:
   Code Intelligence GmbH (commercial company);
   active contributors simonresch,
   oetr,
  HenrichN,
   kyakdan are members/contributors.
   Apache-2.0,
   not archived.
- Popularity:
   1232 stars,
   167 forks,
   42 open issues.
- Releases:
   roughly monthly through late 2025 (0.25.0 Aug,
   0.25.1 Sep,
   0.26.0 Oct,
   0.27.0/0.28.0
  Nov,
   0.29.1 Dec 2025),
   then 0.30.0 on 2026-02-24 (latest,
   ~3.5 months old).
- Commits:
   default branch `main` last commit 2026-03-06 (~3 months ago).
   Repo `pushed_at`
  2026-06-05 reflects pushes to open PR branches,
   not main.
- Responsiveness:
   strong through 2025: maintainers gave substantive answers and linked PRs on
  #951,
   #971,
   #977,
   #978,
   #1022,
   #939 (labeling,
   explaining,
   fixing).
   Visible cooling in H1 2026:
  #1053 ("Build Jazzer for Android is broken",
   2026-03-24) has no maintainer reply,
   and PRs
  #1048/#1051/#1052/#1054/#1056 (Mar-Apr) sit open without merge.
- Interpretation:
   active,
   company-backed,
   releases regularly,
   historically responsive,
   but
  public-repo activity has slowed in the last quarter.
   State:
   "active releases with a recent
  slowdown in public triage",
   not abandoned.

Android nuance (important,
 not a blocker):
 issue #1053 and the paused-Android statement (#939)
refer to running Jazzer's instrumentation ON Android Runtime (ART/Dalvik).
 The use case here is
fuzzing pure Kotlin/JVM logic on a desktop/CI JVM via JUnit5,
 which is the mainline,
 fully
supported and verified path.
 On-device Android fuzzing is not needed.

## 4. FULL VERIFICATION (bounded fuzz in a podman container, crash FOUND)

Project:
 `/var/tmp/jazzer-verify` (Gradle 8.10.2 Kotlin,
 jazzer-junit 0.30.0).
 The fuzz target is
`S3KeyParser.parse` (src/main/kotlin/com/example/S3KeyParser.
kt),
 an S3-style `<bucket>/<key>?bytes=<a>-<b>`
parser.
 Malformed input raises `IllegalArgumentException` (caught/expected);
 the planted bug is an
unguarded `IllegalStateException("path traversal in key: ...")` when the key contains `..`
(S3KeyParser.
kt:
46).
 One `@FuzzTest(maxDuration = "60s")` taking a `FuzzedDataProvider`
(src/test/kotlin/com/example/S3KeyParserFuzzTest.
kt) catches the expected exception and lets the
planted one escape.

Exact command:

```bash
podman run --rm --memory=6g --cpus=4 \
  --volume /var/tmp/jazzer-verify:/work:Z --workdir /work \
  --env TMPDIR=/work/.tmp \
  docker.io/library/eclipse-temurin:21-jdk bash /work/run-in-container.sh
```

`run-in-container.sh` downloads Gradle 8.10.2 (build on the /var/tmp mount,
 not tmpfs),
 then runs
`JAZZER_FUZZ=1 gradle --no-daemon test`.
 The test JVM gets
`-Djdk.attach.allowAttachSelf=true -XX:+EnableDynamicAgentLoading` (build.
gradle.
kts) for
ByteBuddy self-attach on JDK 21.

Result (full log:
 /var/tmp/jazzer-verify/full-run.
log):
 Jazzer loaded its hooks
(843 TraceCmp + sanitizers),
 instrumented `com.example.S3KeyParser`,
 and fuzzed from an empty
corpus.
 Key output:

```text
#2     INITED cov: 8 ft: 8 corp: 1/1b ...
#2387  REDUCE cov: 22 ... DE: "?bytes="-
#3202  NEW    cov: 29 ...
== Java Exception: java.lang.IllegalStateException: path traversal in key: ...
artifact_prefix='/work/'; Test unit written to /work/crash-10ae6353e04fe1b2ff245f596f619468aef85b60
Base64: P2JdL3kuLi4uLi4uLi4uLi4uLi4uLi4uLi4uLi4uLi4uLi4uLi4uLi4uLnRlcz1zPQ==
Done 6307 runs in 0 second(s)

S3KeyParserFuzzTest > fuzzParse(FuzzedDataProvider) > Fuzzing... FAILED
    com.code_intelligence.jazzer.junit.FuzzTestFindingException: java.lang.IllegalStateException: path traversal in key: ...
        at ...FuzzTestExecutor.execute(FuzzTestExecutor.java:342)
        Caused by:
        java.lang.IllegalStateException: path traversal in key: ...
            at com.example.S3KeyParser.parse(S3KeyParser.kt:46)
            at com.example.S3KeyParserFuzzTest.fuzzParse(S3KeyParserFuzzTest.kt:11)
BUILD FAILED in 43s
```

Crash found in 6307 runs (<1s of fuzzing;
 43s total incl.
 compile).
 Reproducer emitted:
`/work/crash-10ae6353e04fe1b2ff245f596f619468aef85b60` (49 bytes,
 the crashing input,
 also Base64).
The failing test points exactly at the planted bug.
 Coverage guidance worked:
 it discovered the
`?bytes=` delimiter and the `..` token via compare hooks rather than blind luck.
 The reproducer
doubles as a regression test (re-run without JAZZER_FUZZ replays it).
 This is full pass:
build + run + integration boundary (JUnit failure + reproducer file) all exercised.

## 5. Alternatives (with concrete rejection reasons)

- kotlinx.
  fuzz, already disqualified:
   uninstallable (dead artifact host;
   does not resolve).
  Note:
   kotlinx.
  fuzz itself wraps Jazzer as its engine,
   which reinforces Jazzer as the right
  primitive to depend on directly.
- jqwik (net.
  jqwik:
  jqwik,
   Maven Central 200): property-based testing for JUnit5 via `@ForAll`
  generators with shrinking.
   Black-box:
   no bytecode instrumentation,
   no coverage-guided feedback,
  no corpus persistence,
   no security sanitizers (SQLi/SSRF/deserialization/path-traversal hooks).
  It generates well-distributed random data but cannot steer toward structured crashing inputs the
  way Jazzer's compare hooks do;
   finding the `?bytes=`/`..` structure would be luck,
   not guidance.
  Complementary for invariants,
   not a fuzzer replacement.
- Kotest property (io.
  kotest:
  kotest-property,
   Maven Central 200): `Arb`/`Gen`-based property
  testing with shrinking.
   Same gap as jqwik:
   black-box random generation,
   no coverage feedback,
  no instrumentation,
   no sanitizers,
   no persisted corpus.
   Good for Kotlin-idiomatic property
  assertions,
   not coverage-guided fuzzing of parsers.

Both jqwik and Kotest are useful alongside Jazzer (property assertions on the same code) but
neither is a coverage-guided fuzzer,
 which is the stated requirement.

## Artifacts

- Clone:
   /tmp/agent/jazzer-vet
- Verification project:
   /var/tmp/jazzer-verify (build.
  gradle.
  kts,
   run-in-container.
  sh,
   full-run.
  log,
  crash-10ae6353e04fe1b2ff245f596f619468aef85b60)
