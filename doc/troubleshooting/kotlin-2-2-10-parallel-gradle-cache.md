# Kotlin 2.2.10 concurrent Gradle invocations collide on one incremental cache and fall back

## Symptom

Two repository tasks were started concurrently against the same Android Gradle project:

```text
mise run //package/music-player/android-app:prototype:build
mise run //package/music-player/android-app:lint
```

`mise run //package/music-player/android-app:prototype:build` was removed on 2026-10-06 by
`0b49ae109e`;
the package declares no `prototype:*` task today.

The `prototype:build` address is not on `main`:
branch `prototype/music-player-theme-compose` declares it on 2026-09-04 (commit `d99257112e`),
and `main`'s `package/music-player/android-app/mise.toml` has no `prototype:*` task.
Every `prototype:build` address in this file must be read against that branch;
`lint` still resolves on `main`.

Both underlying commands use Gradle 9.5.1 with `--no-daemon`.
 The Kotlin compile emitted:

```text
[KOTLIN] [IC] Incremental compilation was attempted but failed:
Failed to compute files to recompile: java.lang.IllegalStateException:
Storage for [.../app/build/kotlin/compileDebugKotlin/cacheable/caches-jvm/jvm/
kotlin/source-to-classes.tab] is already registered
```

It then reported:

```text
Falling back to non-incremental compilation (reason = IC_FAILED_TO_COMPUTE_FILES_TO_RECOMPILE)
```

The fallback completed and both Gradle invocations ultimately succeeded.
 The incident is
still a verification defect:
 parallel commands contended for one mutable project output and produced a
non-incremental rebuild plus a daemon-loss stack trace.

## Root cause

The project pins Kotlin Compose plugin 2.2.10 in
`package/music-player/android-app/app/build.gradle.kts:5` and Gradle 9.5.1 in
`package/music-player/android-app/gradle/wrapper/gradle-wrapper.properties:3`.
 Both mise
tasks run in the same Gradle project and therefore address the same `app/build/kotlin/`
cache directory.

Kotlin 2.2.10 names the affected map `source-to-classes` in
`build-common/src/org/jetbrains/kotlin/incremental/AbstractIncrementalCache.kt:66` at
tag `v2.2.10` (`c448af19ded1b1a4e96e9af6412cd9acb100ce1a`):

```kotlin
@JvmStatic
protected val SOURCE_TO_CLASSES = "source-to-classes"
```

`LazyStorage` opens that path as an IntelliJ `PersistentHashMap` in
`build-common/src/org/jetbrains/kotlin/incremental/storage/LazyStorage.kt:45-62`:

```kotlin
private fun getStorageIfExists(): PersistentHashMap<KEY, VALUE>? {
    return storage ?: when {
        isStorageFileExist -> createMap().also { storage = it }
        else -> null
    }
}

private fun createMap() =
    PersistentHashMap(storageFile.toPath(), keyDescriptor, valueExternalizer)
```

The synchronization in this class is instance-local.
 For example,
`LazyStorage.kt:70-80` synchronizes methods on one `LazyStorage` object:

```kotlin
@Synchronized
override fun contains(key: KEY): Boolean =
    getStorageIfExists()?.containsMapping(key) ?: false

@Synchronized
override fun get(key: KEY): VALUE? =
    getStorageIfExists()?.get(key)
```

Separate Gradle invocations construct separate owners and storage instances.
 Their
monitors do not serialize access to the shared file.
 `BasicMapsOwner` also registers maps
inside one owner in
`build-common/src/org/jetbrains/kotlin/incremental/storage/BasicMapsOwner.kt:34-38`:

```kotlin
@Synchronized
protected fun <K, V, M : BasicMap<K, V>> registerMap(map: M): M {
    maps.add(map)
    return map
}
```

That source explains why two processes can reach the same persistent map concurrently even
though methods are marked synchronized.
 The harness created the race by parallelizing two
Gradle task invocations with one output directory.

## Verification

Versions and source:

- Kotlin Gradle plugin:
  2.2.10.
- Gradle wrapper:
  9.5.1.
- Kotlin source:
  tag `v2.2.10`,
  commit `c448af19ded1b1a4e96e9af6412cd9acb100ce1a`.
- Source clone:
  `/home/user/temp/agent/kotlin-2026-09-08`.

The triggering harness started these two commands in parallel from the same worktree:

```text
cd /var/home/user/temp/agent/music-player-theme-compose-prototype
mise run //package/music-player/android-app:prototype:build
mise run //package/music-player/android-app:lint
```

That worktree is the `prototype/music-player-theme-compose` checkout that declares
`prototype:build`;
the `main` checkout does not declare it.

### Working catalog

- `prototype:build` alone completed successfully.
- `lint` alone completed successfully.
- `prototype:build` followed by `lint` completed successfully with no cache-registration
  diagnostic.
- `prototype:install`,
  whose assemble and install steps are sequential,
  completed
  successfully.

### Failing catalog

- `prototype:build` and `lint` started concurrently against the same worktree emitted
  `Storage for [...] source-to-classes.tab is already registered`.
- The build recovered through non-incremental compilation,
  so process success alone did not
  reveal the collision.

## Verified workarounds

### Serialize Gradle invocations per project output

Run same-project checks sequentially:

```text
mise run //package/music-player/android-app:prototype:build
mise run //package/music-player/android-app:lint
```

The `prototype:build` address resolves only on the
`prototype/music-player-theme-compose` branch (commit `d99257112e` on 2026-09-04).

Both commands passed sequentially after the incident.
 Tradeoff:
 independent checks no longer
consume CPU concurrently,
 but their caches remain incremental and deterministic.
 Tasks for
different Gradle projects may still run concurrently when they do not share an output tree.

## What does not work

- `--no-daemon` does not isolate the output cache.
  Both triggering task definitions already
  use it,
  yet each invocation still opened the same `source-to-classes.tab` file.
- Treating final exit code zero as proof of a clean build misses the incident.
  Kotlin recovered
  through a non-incremental fallback and printed the failure before Gradle reported success.
- Deleting caches is unnecessary for this incident.
  Sequential commands passed without deleting
  build output,
  so deletion would hide the scheduling error rather than address it.

## Upstream filing decision

1.  **Is it really upstream's fault?**
    No.
    The repository harness launched two commands
    concurrently against one mutable Gradle output directory.
2.  **Can upstream fix it?**
    Not evaluated because the first constraint fails.
3.  **Are they supporting this use case?**
    No evidence was found that separate Gradle
    invocations may mutate one project cache concurrently.
4.  **Would the repository welcome our contribution?**
    Not evaluated because this is a
    consumer scheduling error.
5.  **Will they likely fix it?**
    Not applicable.
6.  **Have we prototyped a minimal fix?**
    The consumer-side fix is verified serialization;
    no upstream patch is warranted.

No matching exemption exists under `.out-of-scope/`.
 Searches of open and closed
`JetBrains/kotlin` issues for `source-to-classes.tab` plus `already registered` found no
matching report.
 There is nothing to file upstream and no issue draft to retain.
