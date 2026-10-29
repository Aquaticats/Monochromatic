/**
 * Package tasks translated from the matching `mise.toml` for
 * `@monochromatic-dev/dev-script-task-cli`.
 */

import type { PackageTaskFile, } from '../../dev-script/task-cli/src/types.ts';

export default {
  "build:native": {
  description: "Cross-compile the native engine .so for both 64-bit ABIs (arm64-v8a for real devices, x86_64 for emulators; 16KB-aligned via the NDK 29 linker default, API 26 floor) into app/src/main/jniLibs. Derives and exports ANDROID_NDK_HOME from the mise-provisioned $ANDROID_HOME/ndk (run prepare:android once).",
  dir: "rust",
  run: { shell: "node --input-type=module-typescript -e", run: "const { existsSync, readdirSync, readFileSync, rmSync } = await import(\"node:fs\");\nconst { join } = await import(\"node:path\");\nconst sdk = process.env.ANDROID_HOME;\nif (!sdk) {\n  throw new Error(\"ANDROID_HOME is unset; the android-sdk mise tool exports it. Run `mise install` at the repo root.\");\n}\nconst ndkDir = join(sdk, \"ndk\");\n// An NDK install is a version dir carrying the CMake toolchain file; ignore anything else.\nconst ndks = existsSync(ndkDir)\n  ? readdirSync(ndkDir).filter((e) => existsSync(join(ndkDir, e, \"build\", \"cmake\", \"android.toolchain.cmake\")))\n  : [];\nif (ndks.length === 0) {\n  throw new Error(`No NDK under ${ndkDir}; run \\`mise run prepare:android\\` once to install it.`);\n}\n// Pick the highest version, matching cargo-ndk's own autodetect choice.\nconst ndkHome = join(ndkDir, ndks.sort((a, b) => a.localeCompare(b, undefined, { numeric: true })).at(-1));\nconst toolchainFile = join(ndkHome, \"build\", \"cmake\", \"android.toolchain.cmake\");\n// Self-heal against poisoned opusic-sys CMake caches for both ABIs. A configure\n// without the Android toolchain, against another NDK, or before this checkout was\n// moved leaves compiler checks and absolute CMAKE_CACHEFILE_DIR paths that CMake\n// never re-evaluates. Purge only caches whose toolchain, target path, or x86 SSE\n// marker is wrong; healthy caches stay incremental.\nconst buildRoots = [\"target/aarch64-linux-android\", \"target/x86_64-linux-android\"];\nfor (const buildRoot of buildRoots) {\n  if (!existsSync(buildRoot)) {\n    continue;\n  }\n  for (const profile of readdirSync(buildRoot)) {\n    const profBuild = join(buildRoot, profile, \"build\");\n    if (!existsSync(profBuild)) {\n      continue;\n    }\n    for (const entry of readdirSync(profBuild)) {\n      if (!entry.startsWith(\"opusic-sys-\")) {\n        continue;\n      }\n      const cacheDir = join(process.cwd(), profBuild, entry, \"out\", \"build\");\n      const cachePath = join(cacheDir, \"CMakeCache.txt\");\n      if (!existsSync(cachePath)) {\n        continue;\n      }\n      const cache = readFileSync(cachePath, \"utf8\");\n      const currentDirectory = `CMAKE_CACHEFILE_DIR:INTERNAL=${cacheDir}`;\n      const healthy = cache.includes(toolchainFile)\n        && cache.includes(currentDirectory)\n        && !cache.includes(\"SSE1_SUPPORTED:INTERNAL=1\");\n      if (!healthy) {\n        console.log(`Purging poisoned opusic-sys CMake cache (forces clean reconfigure): ${join(profBuild, entry)}`);\n        rmSync(join(profBuild, entry), { recursive: true, force: true });\n      }\n    }\n  }\n}\n// cargo-ndk autodetects the NDK for its own Rust cross-toolchain, but it does NOT\n// export ANDROID_NDK_HOME to child build scripts. opusic-sys (bundled libopus)\n// reads ANDROID_NDK_HOME to apply the CMake android toolchain; without it that\n// build silently falls back to a HOST compile (and a /usr/local install). So set\n// it explicitly, derived from the mise SDK, keeping the build self-contained.\nconst { execFileSync } = await import(\"node:child_process\");\n// Build both 64-bit ABIs in one invocation: arm64-v8a for physical devices and\n// x86_64 for emulators (which run the x86_64 system image on an x86_64 host). A\n// repeated -t makes cargo-ndk cross-compile each and drop libmusicplayer_native.so\n// into the matching jniLibs/<abi>/ dir (jniLibs/arm64-v8a, jniLibs/x86_64). Without\n// the x86_64 .so the app dies on an x86_64 emulator with UnsatisfiedLinkError\n// (\"library libmusicplayer_native.so not found\") the instant PlaybackService loads\n// the engine. Both ABIs inherit the NDK 29 linker's 16KB max-page-size default, so\n// neither trips the emulator's non-16KB-alignment warning. The 32-bit ABIs\n// (armeabi-v7a, x86) are intentionally omitted: no current target needs them.\nexecFileSync(\n  \"cargo\",\n  [\"ndk\", \"-t\", \"arm64-v8a\", \"-t\", \"x86_64\", \"--platform\", \"26\", \"-o\", \"../app/src/main/jniLibs\", \"build\", \"--release\"],\n  { stdio: \"inherit\", env: { ...process.env, ANDROID_NDK_HOME: ndkHome } },\n);", },
  },
  "build": {
  description: "Assemble the debug APK (rebuilds the native .so first)",
  depends: ["build:native"],
  run: "./gradlew :app:assembleDebug --no-daemon --console=plain",
  },
  "build:release": {
  description: "Assemble the optimized (non-debuggable) release APK, debug-signed so it installs unless the MUSIC_PLAYER_KEYSTORE_FILE, _KEYSTORE_PASSWORD, _KEY_ALIAS and _KEY_PASSWORD values are set (rebuilds the native .so first)",
  depends: ["build:native"],
  run: "./gradlew :app:assembleRelease --no-daemon --console=plain",
  },
  "notices": {
  description: "Write app/build/third-party-licenses/THIRD-PARTY-LICENSES.txt: the license notices of the release APK (Android libraries checked by licensee, native crates by cargo-about)",
  run: [
    "./gradlew :app:licenseeAndroidRelease :app:collectAndroidLibraryNotices --no-daemon --console=plain",
    "node ../desktop-app/bin/third-party-notices.mjs --product android --crate rust --output app/build/third-party-licenses/THIRD-PARTY-LICENSES.txt --android-libraries app/build/reports/licensee/androidRelease/artifacts.json --android-library-notices app/build/third-party-licenses/android-library-notices.txt",
  ],
  },
  "install": {
  description: "Install the debug APK on the connected device (rebuilds the native .so first)",
  depends: ["build:native"],
  run: "./gradlew :app:installDebug --no-daemon --console=plain",
  },
  "run:release": {
  description: "Build the debug-signed release APK, install it on every attached device, and launch it on each of them.",
  depends: ["build:release"],
  run: { shell: "node --input-type=module-typescript -e", run: "const { execFileSync } = await import(\"node:child_process\");\nconst { join } = await import(\"node:path\");\nconst sdk = process.env.ANDROID_HOME;\nif (!sdk) {\n  throw new Error(\"ANDROID_HOME is unset; the android-sdk mise tool exports it. Run `mise install` at the repo root.\");\n}\nconst adb = join(sdk, \"platform-tools\", \"adb\");\n// Install the optimized release APK. Gradle's installRelease already targets every\n// attached device, so no per-device loop is needed for the install itself.\nexecFileSync(\"./gradlew\", [\":app:installRelease\", \"--no-daemon\", \"--console=plain\"], { stdio: \"inherit\" });\n// Enumerate attached devices. `adb devices` prints a header line, then one\n// \"<serial>\\t<state>\" line per device. Keep only the entries whose state is\n// exactly `device` (drop the header and any `offline` / `unauthorized` entries).\nconst serials = execFileSync(adb, [\"devices\"], { encoding: \"utf8\" })\n  .split(\"\\n\")\n  .slice(1)\n  .map((line) => line.trim())\n  .filter((line) => line.length > 0)\n  .map((line) => line.split(\"\\t\"))\n  .filter((parts) => parts[1] === \"device\")\n  .map((parts) => parts[0]);\nif (serials.length === 0) {\n  throw new Error(\"No attached device in `device` state; connect or boot one, then re-run.\");\n}\n// Launch MainActivity on each device, pinning -s <serial> so a multi-device attach\n// no longer fails with \"more than one device/emulator\".\nfor (const serial of serials) {\n  console.log(`Launching on ${serial}`);\n  execFileSync(adb, [\"-s\", serial, \"shell\", \"am\", \"start\", \"-n\", \"dev.monochromatic.musicplayer/.MainActivity\"], { stdio: \"inherit\" });\n}", },
  },
  "test:unit": {
  description: "Run host JVM unit tests for the pure-logic core (no device)",
  run: "./gradlew :app:testDebugUnitTest --no-daemon --console=plain",
  },
  "test:instrumented:build": {
  description: "Assemble the app + instrumented-test APKs (native .so included; no device). Run on device with am instrument or connectedAndroidTest.",
  depends: ["build:native"],
  run: "./gradlew :app:assembleDebug :app:assembleDebugAndroidTest --no-daemon --console=plain",
  },
  "test:instrumented": {
  description: "Run instrumented tests on the connected device (pin ANDROID_SERIAL; hold the adb flock; rebuilds the native .so first)",
  depends: ["build:native"],
  run: "./gradlew :app:connectedDebugAndroidTest --no-daemon --console=plain",
  },
  "test:instrumented:device": {
  description: "Non-destructive on-device instrumented run (install -r + am instrument; keeps the SAF grant + warm cache). Defaults to the silent PeakSweepWorkerTest; override with the `class` arg.",
  depends: ["test:instrumented:build"],
  usage: "arg \"class\" help=\"Task argument class\"",
  run: [
    "adb install -r app/build/outputs/apk/debug/app-debug.apk",
    "adb install -r app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk",
    "adb shell am instrument -w -e class ${usage_class:-dev.monochromatic.musicplayer.PeakSweepWorkerTest} dev.monochromatic.musicplayer.test/androidx.test.runner.AndroidJUnitRunner",
  ],
  },
  "lint": {
  description: "Run Android Lint on the debug variant",
  run: "./gradlew :app:lintDebug --no-daemon --console=plain",
  },
  "lint:detekt": {
  description: "Enforce KDoc on this app's Kotlin via the shared kotlin-linter",
  run: "../../linter/kotlin/gradlew --project-dir ../../linter/kotlin detektCheck -PdetektInput=$TASK_REPO_ROOT/$TASK_PACKAGE_PATH/app/src/main/kotlin --no-daemon --console=plain",
  },
  "format:clippy": {
  description: "Apply Clippy's machine-applicable suggestions to every Android-native Rust target",
  dir: "rust",
  run: { shell: "node --input-type=module-typescript -e", run: "\nconst { existsSync, readdirSync } = await import(\"node:fs\")\nconst { join } = await import(\"node:path\")\nconst { execFileSync } = await import(\"node:child_process\")\nconst sdk = process.env.ANDROID_HOME\nif (!sdk) {\n  throw new Error(\"ANDROID_HOME is unset; run `mise install` at the repo root.\")\n}\nconst ndkDir = join(sdk, \"ndk\")\nconst ndks = existsSync(ndkDir)\n  ? readdirSync(ndkDir).filter((entry) => existsSync(join(ndkDir, entry, \"build\", \"cmake\", \"android.toolchain.cmake\")))\n  : []\nif (ndks.length === 0) {\n  throw new Error(`No NDK under ${ndkDir}; run \\`mise run prepare:android\\` once to install it.`)\n}\nconst ndkHome = join(ndkDir, ndks.sort((left, right) => left.localeCompare(right, undefined, { numeric: true })).at(-1))\nconst runAndroidClippy = (cargoArgs) => {\n  execFileSync(\n    \"cargo\",\n    [\"ndk\", \"--target\", \"arm64-v8a\", \"--target\", \"x86_64\", \"--platform\", \"26\", ...cargoArgs],\n    { stdio: \"inherit\", env: { ...process.env, ANDROID_NDK_HOME: ndkHome } },\n  )\n}\n\nrunAndroidClippy([\"clippy\", \"--fix\", \"--allow-dirty\", \"--allow-staged\", \"--allow-no-vcs\", \"--all-targets\"])", },
  },
  "lint:clippy": {
  description: "Run Clippy over every Android-native Rust target for both supported ABIs",
  dir: "rust",
  run: { shell: "node --input-type=module-typescript -e", run: "\nconst { existsSync, readdirSync } = await import(\"node:fs\")\nconst { join } = await import(\"node:path\")\nconst { execFileSync } = await import(\"node:child_process\")\nconst sdk = process.env.ANDROID_HOME\nif (!sdk) {\n  throw new Error(\"ANDROID_HOME is unset; run `mise install` at the repo root.\")\n}\nconst ndkDir = join(sdk, \"ndk\")\nconst ndks = existsSync(ndkDir)\n  ? readdirSync(ndkDir).filter((entry) => existsSync(join(ndkDir, entry, \"build\", \"cmake\", \"android.toolchain.cmake\")))\n  : []\nif (ndks.length === 0) {\n  throw new Error(`No NDK under ${ndkDir}; run \\`mise run prepare:android\\` once to install it.`)\n}\nconst ndkHome = join(ndkDir, ndks.sort((left, right) => left.localeCompare(right, undefined, { numeric: true })).at(-1))\nconst runAndroidClippy = (cargoArgs) => {\n  execFileSync(\n    \"cargo\",\n    [\"ndk\", \"--target\", \"arm64-v8a\", \"--target\", \"x86_64\", \"--platform\", \"26\", ...cargoArgs],\n    { stdio: \"inherit\", env: { ...process.env, ANDROID_NDK_HOME: ndkHome } },\n  )\n}\n\nrunAndroidClippy([\"clippy\", \"--release\", \"--all-targets\", \"--\", \"--deny\", \"warnings\"])", },
  },
  "clean": {
  description: "Delete Gradle build outputs",
  run: "./gradlew clean --no-daemon --console=plain",
  },
} satisfies PackageTaskFile;
