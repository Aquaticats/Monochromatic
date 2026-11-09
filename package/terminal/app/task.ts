/**
 * Package tasks translated from the matching `mise.toml` for
 * `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  "image": {
    description: "Build the CachyOS build/run container image",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()], force: true })
` },
  },
  "image:release": {
    description: "Build the Ubuntu 24.04 release container image: the base of the IDE release image, so shipped binaries keep the glibc of Ubuntu 24.04",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArgument = execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
containerImageBuild({ image: 'localhost/monochromatic/terminal-release', containerfile: 'Containerfile.release', buildArguments: ['--build-arg', rustToolchainBuildArgument, ...zigArchiveBuildArguments()], force: true })
` },
  },
  "build": {
    description: "Release build (host if Zig + dev libs present, else container), then stage runtime shared libraries",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

runCargo(['build', '--release'])
execFileSync('task-run', ['//package/terminal/app:stage:runtime-libs:release'], { stdio: 'inherit' })
` },
  },
  "build:debug": {
    description: "Debug build (host if Zig + dev libs present, else container), then stage runtime shared libraries",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

runCargo(['build'])
execFileSync('task-run', ['//package/terminal/app:stage:runtime-libs:debug'], { stdio: 'inherit' })
` },
  },
  "stage:runtime-libs:release": {
    description: "Copy release libghostty-vt shared libraries into the binary rpath directory",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const stageFs = await import('node:fs')
const stagePath = await import('node:path')
const stageChild = await import('node:child_process')
const cargoOutputRoots = () => {
  const roots = []
  const envRoot = process.env.CARGO_TARGET_DIR
  if (envRoot !== undefined && envRoot !== '') { roots.push(envRoot) }
  try {
    const metadata = JSON.parse(stageChild.execFileSync('cargo', ['metadata', '--format-version', '1', '--no-deps'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }))
    for (const key of ['target_directory', 'build_directory']) {
      const value = metadata[key]
      if (typeof value === 'string' && value !== '') { roots.push(value) }
    }
  } catch {
    // Without cargo metadata only the package-local target dir is left to search.
  }
  roots.push(stagePath.join(process.cwd(), 'target'))
  return [...new Set(roots)]
}
const stageLibghosttyRuntimeLibs = (profile) => {
  const roots = cargoOutputRoots()
  const libDirs = roots.flatMap((root) => [
    ...stageFs.globSync(\`\${root}/\${profile}/build/libghostty-vt-sys-*/out/ghostty-install/lib\`),
    ...stageFs.globSync(\`\${root}/\${profile}/build/libghostty-vt-sys/*/out/ghostty-install/lib\`),
  ]).sort()
  if (libDirs.length === 0) {
    const searched = roots.map((root) => stagePath.join(root, profile, 'build')).join(', ')
    const buildTask = profile === 'release' ? '//package/terminal/app:build' : '//package/terminal/app:build:debug'
    throw new Error(\`missing \${profile} libghostty-vt build output (searched \${searched}); run "task-run \${buildTask}" first\`)
  }
  const sourceLibDir = libDirs[libDirs.length - 1]
  const stagedLibDir = stagePath.join(process.cwd(), 'target', 'lib', 'monochromatic-terminal')
  stageFs.mkdirSync(stagedLibDir, { recursive: true })
  for (const name of ['libghostty-vt.so', 'libghostty-vt.so.0', 'libghostty-vt.so.0.1.0']) {
    stageFs.copyFileSync(stagePath.join(sourceLibDir, name), stagePath.join(stagedLibDir, name))
  }
}

stageLibghosttyRuntimeLibs('release')
` },
  },
  "stage:runtime-libs:debug": {
    description: "Copy debug libghostty-vt shared libraries into the binary rpath directory",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const stageFs = await import('node:fs')
const stagePath = await import('node:path')
const stageChild = await import('node:child_process')
const cargoOutputRoots = () => {
  const roots = []
  const envRoot = process.env.CARGO_TARGET_DIR
  if (envRoot !== undefined && envRoot !== '') { roots.push(envRoot) }
  try {
    const metadata = JSON.parse(stageChild.execFileSync('cargo', ['metadata', '--format-version', '1', '--no-deps'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }))
    for (const key of ['target_directory', 'build_directory']) {
      const value = metadata[key]
      if (typeof value === 'string' && value !== '') { roots.push(value) }
    }
  } catch {
    // Without cargo metadata only the package-local target dir is left to search.
  }
  roots.push(stagePath.join(process.cwd(), 'target'))
  return [...new Set(roots)]
}
const stageLibghosttyRuntimeLibs = (profile) => {
  const roots = cargoOutputRoots()
  const libDirs = roots.flatMap((root) => [
    ...stageFs.globSync(\`\${root}/\${profile}/build/libghostty-vt-sys-*/out/ghostty-install/lib\`),
    ...stageFs.globSync(\`\${root}/\${profile}/build/libghostty-vt-sys/*/out/ghostty-install/lib\`),
  ]).sort()
  if (libDirs.length === 0) {
    const searched = roots.map((root) => stagePath.join(root, profile, 'build')).join(', ')
    const buildTask = profile === 'release' ? '//package/terminal/app:build' : '//package/terminal/app:build:debug'
    throw new Error(\`missing \${profile} libghostty-vt build output (searched \${searched}); run "task-run \${buildTask}" first\`)
  }
  const sourceLibDir = libDirs[libDirs.length - 1]
  const stagedLibDir = stagePath.join(process.cwd(), 'target', 'lib', 'monochromatic-terminal')
  stageFs.mkdirSync(stagedLibDir, { recursive: true })
  for (const name of ['libghostty-vt.so', 'libghostty-vt.so.0', 'libghostty-vt.so.0.1.0']) {
    stageFs.copyFileSync(stagePath.join(sourceLibDir, name), stagePath.join(stagedLibDir, name))
  }
}

stageLibghosttyRuntimeLibs('debug')
` },
  },
  "lint": {
    description: "cargo check (host if Zig + dev libs present, else container)",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

runCargo(['check'])
` },
  },
  "lint:clippy": {
    description: "Run clippy with warnings denied (host if Zig + dev libs present, else container)",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

runCargo(['clippy', '--release', '--all-targets', '--', '--deny', 'warnings'])
` },
  },
  "format:clippy": {
    description: "Apply clippy's machine-applicable suggestions in place (host if Zig + dev libs present, else container)",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

runCargo(['clippy', '--fix', '--allow-dirty', '--allow-staged', '--allow-no-vcs', '--all-targets'])
` },
  },
  "test": {
    description: "cargo nextest run (host if Zig + dev libs present, else container)",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

runCargo(['nextest', 'run', '--release'])
` },
  },
  "mcp": {
    description: "Build --features slint/mcp (debug), stage libs, and run HEADLESS with the embedded Slint MCP server on SLINT_MCP_PORT (default 9316) so an agent can drive/inspect the terminal UI",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

const stageFs = await import('node:fs')
const stagePath = await import('node:path')
const stageChild = await import('node:child_process')
const cargoOutputRoots = () => {
  const roots = []
  const envRoot = process.env.CARGO_TARGET_DIR
  if (envRoot !== undefined && envRoot !== '') { roots.push(envRoot) }
  try {
    const metadata = JSON.parse(stageChild.execFileSync('cargo', ['metadata', '--format-version', '1', '--no-deps'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }))
    for (const key of ['target_directory', 'build_directory']) {
      const value = metadata[key]
      if (typeof value === 'string' && value !== '') { roots.push(value) }
    }
  } catch {
    // Without cargo metadata only the package-local target dir is left to search.
  }
  roots.push(stagePath.join(process.cwd(), 'target'))
  return [...new Set(roots)]
}
const stageLibghosttyRuntimeLibs = (profile) => {
  const roots = cargoOutputRoots()
  const libDirs = roots.flatMap((root) => [
    ...stageFs.globSync(\`\${root}/\${profile}/build/libghostty-vt-sys-*/out/ghostty-install/lib\`),
    ...stageFs.globSync(\`\${root}/\${profile}/build/libghostty-vt-sys/*/out/ghostty-install/lib\`),
  ]).sort()
  if (libDirs.length === 0) {
    const searched = roots.map((root) => stagePath.join(root, profile, 'build')).join(', ')
    const buildTask = profile === 'release' ? '//package/terminal/app:build' : '//package/terminal/app:build:debug'
    throw new Error(\`missing \${profile} libghostty-vt build output (searched \${searched}); run "task-run \${buildTask}" first\`)
  }
  const sourceLibDir = libDirs[libDirs.length - 1]
  const stagedLibDir = stagePath.join(process.cwd(), 'target', 'lib', 'monochromatic-terminal')
  stageFs.mkdirSync(stagedLibDir, { recursive: true })
  for (const name of ['libghostty-vt.so', 'libghostty-vt.so.0', 'libghostty-vt.so.0.1.0']) {
    stageFs.copyFileSync(stagePath.join(sourceLibDir, name), stagePath.join(stagedLibDir, name))
  }
}

const { mkdirSync, copyFileSync } = await import('node:fs')
const { join } = await import('node:path')
process.env.SLINT_EMIT_DEBUG_INFO = '1'
process.env.SLINT_MCP_PORT = process.env.SLINT_MCP_PORT ?? '9316'
process.env.SLINT_BACKEND = process.env.SLINT_BACKEND ?? 'headless'
const xdgRoot = join(process.cwd(), 'target', 'mcp-xdg')
for (const sub of ['config', 'cache', 'data']) { mkdirSync(join(xdgRoot, sub), { recursive: true }) }
process.env.XDG_CONFIG_HOME = join(xdgRoot, 'config')
process.env.XDG_CACHE_HOME = join(xdgRoot, 'cache')
process.env.XDG_DATA_HOME = join(xdgRoot, 'data')
runCargo(['build', '--features', 'slint/mcp'])
// Stage libghostty-vt beside the binary INLINE (not via a nested \`mise run\`, which
// re-loads this repo's sops-encrypted config and fails without the decrypt key).
// Same discovery and copy as stage:runtime-libs:debug.
stageLibghosttyRuntimeLibs('debug')
console.error(\`monochromatic-terminal MCP server: http://127.0.0.1:\${process.env.SLINT_MCP_PORT}/mcp (backend=\${process.env.SLINT_BACKEND || 'default'})\`)
execFileSync('./target/debug/monochromatic-terminal', [], { stdio: 'inherit' })
` },
  },
  "lint:slint": {
    description: "Type-check ui/app.slint with slint-viewer --check (compiles the markup, opens no window, exits non-zero on error)",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: "slint-viewer --check ui/app.slint",
  },
  "screenshot": {
    description: "Render ui/app.slint to target/app.slint.png HEADLESS (slint-viewer --screenshot, default property values, no window) for a quick static-layout snapshot",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { mkdirSync } = await import('node:fs')
mkdirSync('target', { recursive: true })
execFileSync('slint-viewer', ['--screenshot', 'target/app.slint.png', 'ui/app.slint'], { stdio: 'inherit', env: { ...process.env, SLINT_BACKEND: 'software' } })
console.error('rendered ui/app.slint -> target/app.slint.png')
` },
  },
  "build:container": {
    description: "Release build forced inside the container, then stage runtime libraries (asserts the container path)",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

containerCargo(['build', '--release'])
execFileSync('task-run', ['//package/terminal/app:stage:runtime-libs:release'], { stdio: 'inherit' })
` },
  },
  "lint:clippy:container": {
    description: "clippy with warnings denied forced inside the container",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

containerCargo(['clippy', '--release', '--', '--deny', 'warnings'])
` },
  },
  "test:container": {
    description: "cargo nextest run forced inside the container",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

containerCargo(['nextest', 'run', '--release'])
` },
  },
  "verify:container": {
    description: "Assert the container build/clippy/test path works even when the host has Zig and the dev libs",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

execFileSync('task-run', ['image'], { stdio: 'inherit' })
containerCargo(['build', '--release'])
containerCargo(['clippy', '--release', '--', '--deny', 'warnings'])
containerCargo(['nextest', 'run', '--release'])
` },
  },
  "run": {
    description: "Build (host if Zig + dev libs present, else container), install desktop metadata and runtime libraries, then run the GUI on the host",
    env: {"SLINT_ENABLE_EXPERIMENTAL_FEATURES":"1","LIBCLANG_PATH":"/usr/lib64"},
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
const { globSync } = await import('node:fs')
// The lowest disk weight and the podman bound options come from package/container-limit/core (repository rule HXR).
// The repository-root clippy.toml, mounted read-only outside /work and named by CLIPPY_CONF_DIR, is the only home of
// the fleet-wide disallowed-methods ban that [lints.clippy] only activates; clippy reads the nearest clippy.toml up
// from the crate directory, and the container mounts only the package directory at /work.
const { containerLimitArguments } = await import(process.getBuiltinModule('node:url').pathToFileURL(process.getBuiltinModule('node:path').resolve(process.cwd(), '../../container-limit/core/src/index.ts')).href)
// 4 GiB and 4 CPUs bound the Ghostty plus Slint compile (this package's own bound before the weight); 1024 processes is the IDE's
// bound for 4 CPUs (package/ide/app/bin/container-bounds.ts). Measured 2026-11-08T19:53:09-05:00 on a build that\n// reused a warm target directory: peak 3,188,219,904 bytes (2.97 GiB) of 4 GiB, 235 threads of 1024, 207 throttled\n// CPU periods of 2260, no kill. 4 GiB is 74 percent used, so it is not loose (doc/handover/container-limits-everywhere.md).
const containerBound = { memory: '4g', cpus: 4, pidsLimit: 1024 }
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && onPath('zig') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: \`-resource-dir=\${resourceDir}\`, ZIG_GLOBAL_CACHE_DIR: \`\${process.cwd()}/target/zig-global-cache\` }
}
// CACHYOS_IMAGE is the CachyOS image this host's CPU level supports (cachyos-v4, -v3 or plain), resolved by the repository's
// cachyos-image command; it throws on a host that is not x86-64, because these images have no Alpine variant.
const cachyosBuildArguments = (imageName) => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/cachyos-image.ts'), '--image-name', imageName, '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// RUST_TOOLCHAIN is the nightly toolchain.lock.json records; the Containerfile has no default, so a build without it stops.
const rustToolchainBuildArguments = () => ['--build-arg', execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-rust-toolchain.ts'), '--build-argument'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()]
// ZIG_ARCHIVE_URL and ZIG_ARCHIVE_SHA256 are the Zig archive toolchain.lock.json records for this host; the Containerfile has no default, so a build without them stops.
const zigArchiveBuildArguments = () => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts'), 'zig'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim().split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// containerImageBuild builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label and also covers the local images in baseImages), so an image never outlives the lock row it was built from; force builds regardless, boundingFlags limit the build without entering the digest.
const containerImageBuild = ({ image, containerfile, buildArguments, force = false, baseImages = [], boundingFlags = [] }) => execFileSync('node', [process.getBuiltinModule('node:path').resolve(process.cwd(), '../../test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, ...(force ? ['--force'] : []), ...baseImages.flatMap((baseImage) => ['--base-image', baseImage]), ...boundingFlags.map((flag) => '--bounding-flag=' + flag), '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs) => {
  containerImageBuild({ image: 'localhost/monochromatic/terminal', containerfile: 'Containerfile', buildArguments: [...cachyosBuildArguments('terminal'), ...rustToolchainBuildArguments(), ...zigArchiveBuildArguments()] })
  execFileSync('podman', ['run', '--rm', ...containerLimitArguments(containerBound), '--security-opt', 'label=disable', '--volume', \`\${process.cwd()}:/work\`, '--volume', \`\${process.cwd()}/../../../clippy.toml:/clippy-conf/clippy.toml:ro\`, '--env', 'CLIPPY_CONF_DIR=/clippy-conf', '--volume', 'terminal-cargo:/cargo', '--workdir', '/work', 'localhost/monochromatic/terminal', 'cargo', ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => { if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) } else { containerCargo(cargoArgs) } }

const { mkdirSync, copyFileSync } = await import('node:fs')
const { join } = await import('node:path')
const home = process.env.HOME ?? ''
runCargo(['build', '--release'])
execFileSync('task-run', ['//package/terminal/app:stage:runtime-libs:release'], { stdio: 'inherit' })
const stagedLibDir = 'target/lib/monochromatic-terminal'
let bin
if (home !== '') {
  const appsDir = join(home, '.local/share/applications')
  mkdirSync(appsDir, { recursive: true })
  copyFileSync('share/applications/monochromatic.terminal.desktop', join(appsDir, 'monochromatic.terminal.desktop'))
  const binDir = join(home, '.local/bin')
  mkdirSync(binDir, { recursive: true })
  copyFileSync('target/release/monochromatic-terminal', join(binDir, 'monochromatic-terminal'))
  const installedLibDir = join(home, '.local/lib/monochromatic-terminal')
  mkdirSync(installedLibDir, { recursive: true })
  for (const name of ['libghostty-vt.so', 'libghostty-vt.so.0', 'libghostty-vt.so.0.1.0']) {
    copyFileSync(join(stagedLibDir, name), join(installedLibDir, name))
  }
  bin = join(binDir, 'monochromatic-terminal')
} else {
  bin = './target/release/monochromatic-terminal'
}
execFileSync(bin, [], { stdio: 'inherit' })
` },
  },
} satisfies PackageTaskFile;
