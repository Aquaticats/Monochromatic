/**
 * Package tasks translated from the matching `mise.toml` for
 * `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  "image": {
    description: "Rebuild the bounded native IDE toolchain image",
    run: { shell: "node --input-type=module-typescript -e", run: `const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
const { execFileSync } = await import('node:child_process')
execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', 'localhost/monochromatic/ide', '--file', 'Containerfile', '.'], { stdio: 'inherit' })
` },
  },
  "image:release": {
    description: "Rebuild the Ubuntu 24.04 release image that shipped IDE binaries are compiled in (FROM localhost/monochromatic/terminal-release, built by //package/terminal/app:image:release); the other tasks use it when IDE_RELEASE=1",
    run: { shell: "node --input-type=module-typescript -e", run: `const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
const { execFileSync } = await import('node:child_process')
execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', 'localhost/monochromatic/ide-release', '--file', 'Containerfile.release', '.'], { stdio: 'inherit' })
` },
  },
  "fetch": {
    description: "Fetch the selected Rust dependencies in the bounded build container",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['fetch'], { network: true })
` },
  },
  "runtime": {
    description: "Prepare pinned Helix grammars, queries, and notices for the measured language inventory in bounded build-only storage",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const { existsSync, mkdirSync, readFileSync, readdirSync } = await import('node:fs')
const { dirname, join, resolve } = await import('node:path')
// Callers that depend on this task can start together (build and test, or two sessions in one tree). The helper module
// holds the lock that serializes them and the publishing functions that stage each file under a name of its own, skip
// a file that is already current, and rename into place (bin/runtime-publish.ts, tested in runtime-publish.test.ts).
const { publishBytes, publishFile, publishTree, removeStaging, withExclusiveLock } = await import(resolve('bin/runtime-publish.ts'))
const revision = 'ba40e547426b0f9896c8bdc699a4ab11f2b37dbc'
// REUSE-layout sources keep license texts in LICENSES/ and the copyright line only in source headers.
// Accept them only when every header names one license whose text is present; otherwise the notice is missing.
const reuseHeaders = (sourceRoot, name) => {
  const texts = existsSync(join(sourceRoot, 'LICENSES')) ? readdirSync(join(sourceRoot, 'LICENSES')) : []
  const lines = ['grammar.js', 'src/scanner.c', 'src/scanner.cc', 'src/parser.c'].filter(file => existsSync(join(sourceRoot, file)))
    .flatMap(file => readFileSync(join(sourceRoot, file), 'utf8').split(String.fromCharCode(10), 8).filter(line => /Copyright|SPDX-License-Identifier/.test(line)).map(line => file + ': ' + line))
  const declared = lines.filter(line => line.includes('SPDX-License-Identifier'))
  const covered = declared.every(line => texts.includes((line.match(/SPDX-License-Identifier: *([A-Za-z0-9.+-]+) *$/)?.[1] ?? '') + '.txt'))
  if (!lines.some(line => line.includes('Copyright')) || !declared.length || !covered) throw new Error('Missing grammar license notices for ' + name)
  return lines.join(String.fromCharCode(10)) + String.fromCharCode(10)
}
const metadata = JSON.parse(runCargo(['metadata', '--format-version', '1'], { network: true, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 16 * 1024 * 1024 }))
const loader = metadata.packages.find(item => item.name === 'helix-loader')
if (!loader?.source?.endsWith('#' + revision)) throw new Error('Cargo did not resolve the pinned helix-loader revision')
const source = join(dirname(dirname(loader.manifest_path)), 'runtime')
mkdirSync('target', { recursive: true })
// Everything below writes shared state: the ide-runtime binary, the private fetch and build directory
// target/runtime-config (git checkouts and compiled grammars), and target/<profile>/runtime. One caller at a time does
// it; the next one finds the grammars built and publishes nothing that is already current. The kernel releases the lock
// when its holder ends in any way, so a killed run leaves none behind.
await withExclusiveLock({ lockFile: 'target/runtime.lock', work: () => {
  runCargo(['build', '--no-default-features', '--bin', 'ide-runtime'])
  const privateEnvironment = ['env', '--unset=CARGO_MANIFEST_DIR', 'XDG_CONFIG_HOME=/work/target/runtime-config', 'XDG_CACHE_HOME=/work/target/runtime-cache', 'XDG_DATA_HOME=/work/target/runtime-data']
  runBounded([...privateEnvironment, '/work/target/debug/ide-runtime', 'fetch', source], { network: true })
  runBounded([...privateEnvironment, '/work/target/debug/ide-runtime', 'build', source])
  const built = 'target/runtime-config/helix/runtime'
  const grammars = readdirSync(join(built, 'grammars'), { withFileTypes: true }).filter(entry => entry.isFile() && entry.name.endsWith('.so'))
  const selected = JSON.parse(readFileSync(join(built, 'selected-grammars.json'), 'utf8'))
  if (grammars.length !== selected.length || !selected.every(name => grammars.some(entry => entry.name === name + '.so'))) {
    throw new Error('Built grammar files do not match the selected runtime inventory')
  }
  for (const profile of ['debug', 'release']) {
    const destination = join('target', profile, 'runtime')
    mkdirSync(join(destination, 'grammars'), { recursive: true })
    // The build script reads manifest.json and then every file it lists, and embeds every file below queries and
    // licenses. A reader must see the previous or the new version of each file and never a half-written one, so each
    // file goes through a staging name of its own, and the manifest, which makes the directory valid, is written last.
    removeStaging({ root: destination })
    publishTree({ source: join(built, 'queries'), destination: join(destination, 'queries') })
    publishFile({ source: join(built, 'Helix-LICENSE'), output: join(destination, 'Helix-LICENSE') })
    for (const grammar of grammars) {
      publishFile({ source: join(built, 'grammars', grammar.name), output: join(destination, 'grammars', grammar.name) })
      const name = grammar.name.slice(0, -3)
      const sourceRoot = join(built, 'grammars', 'sources', name)
      const notices = readdirSync(sourceRoot).filter(file => /^(?:licen[cs]e|copying|notice|authors)(?:[._-]|$)/i.test(file))
      const headers = notices.length ? '' : reuseHeaders(sourceRoot, name)
      const licenseDirectory = join(destination, 'licenses', name)
      mkdirSync(licenseDirectory, { recursive: true })
      if (headers) {
        publishBytes({ output: join(licenseDirectory, 'REUSE-headers.txt'), bytes: Buffer.from(headers) })
        notices.push('LICENSES')
      }
      for (const notice of notices) publishTree({ source: join(sourceRoot, notice), destination: join(licenseDirectory, notice) })
    }
    const manifest = JSON.stringify({ helixRevision: revision, grammars: grammars.map(entry => entry.name) }, null, 2)
    publishBytes({ output: join(destination, 'manifest.json'), bytes: Buffer.from(manifest) })
  }
} })
` },
  },
  "notices": {
    description: "Collect the license text of every Rust crate the executable is built from with cargo-about into target/crate-licenses.json",
    depends: ["fetch"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const { createHash } = await import('node:crypto')
const { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } = await import('node:fs')
const output = 'target/crate-licenses.json'
const stamp = 'target/crate-licenses.inputs'
// cargo-about is installed in the build image (Containerfile); an image built before 2026-10-06 lacks it.
let version = ''
try {
  version = runBounded(['cargo-about', '--version'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim()
} catch (error) {
  version = 'none (' + String(error.message).split(String.fromCharCode(10))[0] + ')'
}
if (version !== 'cargo-about 0.9.2') {
  throw new Error('The build image has no cargo-about 0.9.2 (found: ' + version + '). Rebuild the image with task-run //package/ide/app:image, which installs it, then run this task again.')
}
// The list depends only on these inputs, and one run takes about a minute, so an unchanged list is kept.
const inputs = createHash('sha256')
for (const file of ['Cargo.lock', 'Cargo.toml', 'about.toml', 'about.json.hbs']) inputs.update(file + String.fromCharCode(0)).update(readFileSync(file)).update(String.fromCharCode(0))
inputs.update(version)
const digest = inputs.digest('hex')
if (existsSync(output) && existsSync(stamp) && readFileSync(stamp, 'utf8').trim() === digest) {
  console.log(output + ' is current for these inputs')
} else {
  mkdirSync('target', { recursive: true })
  rmSync(output + '.new', { force: true })
  // --frozen: the locked graph, crate sources from the ide-cargo volume (the fetch task fills it), and no network.
  // --fail: a crate whose license cannot be determined or is not accepted by about.toml stops the task.
  runBounded(['cargo-about', '--color', 'never', 'generate', '--frozen', '--fail', '--config', 'about.toml', '--output-file', output + '.new', 'about.json.hbs'])
  const list = JSON.parse(readFileSync(output + '.new', 'utf8'))
  if (!Array.isArray(list.licenses) || list.licenses.length === 0) throw new Error('cargo-about wrote no license text to ' + output + '.new')
  renameSync(output + '.new', output)
  writeFileSync(stamp, digest + String.fromCharCode(10))
  console.log('Wrote ' + output + ': ' + list.licenses.length + ' license texts for ' + new Set(list.licenses.flatMap(item => item.crates.map(used => used.name + ' ' + used.version))).size + ' crates')
}
` },
  },
  "linked-crates": {
    description: "List the crates linked into the application binary (no procedural-macro or build edges) in target/linked-crates.txt for the bundle checks",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const { writeFileSync } = await import('node:fs')
// cargo tree --edges normal,no-proc-macro follows what is compiled into the binary for its one target: the target the
// build container compiles for natively (the container runs the host's architecture, so an arm64 runner builds
// aarch64-unknown-linux-gnu and every other host builds x86_64-unknown-linux-gnu).
const hostTarget = process.arch === 'arm64' ? 'aarch64-unknown-linux-gnu' : 'x86_64-unknown-linux-gnu'
console.error('Linked crates for ' + hostTarget)
const tree = runCargo(['tree', '--frozen', '--edges', 'normal,no-proc-macro', '--target', hostTarget, '--prefix', 'none', '--format', '{p}'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 16 * 1024 * 1024 })
const crates = [...new Set(tree.split(String.fromCharCode(10)).filter(Boolean).map(line => line.replace(/ [(][*][)]$/, '').replace(/ [(].*[)]$/, '').replace(' v', ' ')))].sort()
writeFileSync('target/linked-crates.txt', crates.join(String.fromCharCode(10)) + String.fromCharCode(10))
console.log('Wrote target/linked-crates.txt: ' + crates.length + ' crates, the package itself included')
` },
  },
  "inspect:no-software-renderer": {
    description: "Fail when Slint's software renderer is linked into any build the package defines, and prove the check can see it by requesting that renderer explicitly",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

// The inverted tree of the CPU renderer over every edge kind that reaches a compiled binary, for one release target.
// A package that is not in the build graph at all makes cargo print "nothing to print" to stderr and nothing to stdout.
// The checks run offline (--frozen); \`online\` is the index pass below, which uses --locked and the network instead.
// Both Linux release targets are checked on every host: \`cargo tree --target\` reads the lock file and the manifests of
// the crates, so the aarch64 graph can be checked on an x86_64 host and the other way round (platform-specific
// dependencies differ per target, so a renderer that only one target pulls in is still seen).
const releaseTargets = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu']
const invertedTree = (features, { online = false, target = releaseTargets[0] } = {}) => runCargo(['tree', online ? '--locked' : '--frozen', '--edges', 'normal,dev,no-proc-macro', '--target', target, '--invert', 'i-slint-renderer-software', ...features], { network: online, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] })
// A cargo volume that is new (a CI runner, a fresh clone) holds registry index entries only for what the \`fetch\` task
// resolved. The slint/mcp positive control below resolves crates that no build of this package uses (async-net, for
// one), and --frozen cannot download their index entries: on a fresh volume it stopped with "no matching package named
// \`async-net\` found" (release workflow run 39832705367). One pass per request with the network on and --locked, which
// reads the same lock file and writes nothing, fills those entries; the checks below still run offline. Without a
// network the pass reports why it was skipped, and the checks use whatever the volume already holds.
for (const features of [['--all-features'], ['--features', 'slint/renderer-software'], ['--features', 'slint/mcp']]) {
  try { invertedTree(features, { online: true }) } catch (error) { console.error('Index pass for ' + features.join(' ') + ' skipped: ' + String(error.message).split(String.fromCharCode(10))[0]) }
}
for (const target of releaseTargets) {
  // --all-features turns on every feature the package defines, so a feature added later that reaches the renderer is seen.
  const tree = invertedTree(['--all-features'], { target })
  if (tree.trim() !== '') { throw new Error('Slint software renderer is linked for ' + target + ':' + String.fromCharCode(10) + tree) }
  // Positive controls: each request below must reach the renderer for this target, or the empty tree above proves
  // nothing. The second is Slint's MCP server, whose screenshots use the software renderer and which this package no
  // longer builds.
  for (const feature of ['slint/renderer-software', 'slint/mcp']) {
    if (invertedTree(['--features', feature], { target }).trim() === '') { throw new Error('The positive control --features ' + feature + ' did not reach the software renderer for ' + target + ', so this check cannot see it') }
  }
}
console.log('Slint software renderer is not linked into any build of this package for ' + releaseTargets.join(' and ') + '; --features slint/renderer-software and slint/mcp do reach it for each')
` },
  },
  "build:debug": {
    description: "Build the debug IDE and its helper binaries in the bounded container",
    depends: ["runtime","notices"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['build'])
` },
  },
  "build": {
    description: "Build the optimized application binary in the bounded container, as sibling applications name their release build",
    depends: ["runtime","notices"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

// Only the application binary: the runtime, inspection, and scripted-server helpers are build and test tools.
// SLINT_EMIT_DEBUG_INFO keeps element ids, which the native tests need to find elements by name; sibling release builds do not set it.
// The earlier fixed 2 GiB and 2 CPU bound was enough: measured on 2026-10-05, a build from an empty release directory
// peaked at 1,116 MiB of anonymous memory with no out-of-memory kill and took 15 to 19 minutes on a loaded host.
runBounded(['env', '--unset=SLINT_EMIT_DEBUG_INFO', 'cargo', 'build', '--release', '--bin', 'monochromatic-ide'])
` },
  },
  "bundle": {
    description: "Copy the release binary to dist/monochromatic-ide: one executable that carries its language runtime and license texts",
    depends: ["build"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { chmodSync, copyFileSync, existsSync, mkdirSync, renameSync, rmSync, statSync } = await import('node:fs')
const binary = 'target/release/monochromatic-ide'
if (!existsSync(binary)) throw new Error('Missing ' + binary + '; run the build task')
// The build script embedded target/release/runtime and the license texts into the binary; nothing goes beside it.
// It is copied unstripped, so its symbol names stay in backtraces and profiles.
mkdirSync('dist', { recursive: true })
const destination = 'dist/monochromatic-ide'
const staging = destination + '.new'
rmSync(staging, { recursive: true, force: true })
copyFileSync(binary, staging)
chmodSync(staging, 0o755)
// Until 2026-10-06 the bundle was a directory of this name; the file replaces it.
rmSync(destination, { recursive: true, force: true })
renameSync(staging, destination)
console.log('Copied ' + destination + ' (' + statSync(destination).size + ' bytes)')
` },
  },
  "install": {
    description: "Install dist/monochromatic-ide as ~/.local/bin/monochromatic-ide and its launcher entry in ~/.local/share/applications",
    depends: ["bundle"],
    run: "node bin/install.ts",
  },
  "inspect:bundle": {
    description: "Check a single application executable: every runtime file, notice, and Rust crate license embedded byte for byte, every linked crate covered, and lone copies that highlight, reuse and repair their cache, ignore a user Helix runtime, start concurrently, and report a damaged embedded part",
    depends: ["linked-crates"],
    usage: `arg "[file]" default="dist/monochromatic-ide" help="Single executable to check"
arg "[runtime]" default="target/release/runtime" help="Runtime directory the executable was built from"
arg "[only]" help="Comma-separated check names to run; default is every check"
`,
    run: "node bin/inspect-bundle.ts",
  },
  "inspect:bundle-guards": {
    description: "Damage copies of a single application executable and observe each bundle check fail on its damage",
    depends: ["linked-crates"],
    usage: `arg "[file]" default="dist/monochromatic-ide" help="Single executable to copy and damage"
arg "[runtime]" default="target/release/runtime" help="Runtime directory the executable was built from"
arg "[only]" help="Comma-separated case names to run; default is every case"
`,
    run: "node bin/inspect-bundle-guards.ts",
  },
  "run:bundle": {
    description: "Run the single application executable; without a folder argument it opens the home folder",
    run: "./dist/monochromatic-ide",
  },
  "test": {
    description: "Run library and integration tests in the bounded container",
    depends: ["runtime"],
    usage: "arg \"[filter]\" help=\"Optional nextest filter expression that selects or excludes tests, for example not test(/every_row_of_real_source/); default is every library and integration test\"",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

// A filtered run keeps going after a failure so that every selected test reports, as the native task does.
const filter = process.env.usage_filter ? ['--no-fail-fast', '--filter-expr', process.env.usage_filter] : []
runCargo(['nextest', 'run', '--no-default-features', '--lib', '--tests', ...filter])
` },
  },
  "test:wrap": {
    description: "Run the soft-wrap break rule unit tests and the wrapping integration tests in the bounded container",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--lib', 'wrap::'])
runCargo(['nextest', 'run', '--no-default-features', '--test', 'wrap', '--test', 'wrap_rows', '--test', 'row_map', '--test', 'wrap_keys', '--test', 'vertical_motion'])
` },
  },
  "test:minimap": {
    description: "Run the minimap geometry and picture-scene tests in the bounded container",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--test', 'minimap', '--test', 'minimap_scene'])
` },
  },
  "test:native": {
    description: "Exercise native window events and fonts in process-isolated tests that render on the GPU without a display",
    depends: ["runtime","notices"],
    usage: "arg \"[filter]\" help=\"Optional nextest filter expression, for example test(/sidebar/); default is every native test\"",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

// Language navigation tests start the scripted language server from beside the test binary.
runCargo(['build', '--bin', 'ide-scripted-lsp'])
// A filtered run prints what its tests print, such as measured contrast ratios, and runs every selected test.
const filter = process.env.usage_filter ? ['--no-capture', '--no-fail-fast', '--filter-expr', process.env.usage_filter] : []
runBounded(['cargo', 'nextest', 'run', '--bin', 'monochromatic-ide', ...filter], { gpu: true })
` },
  },
  "test:native:stress": {
    description: "Run filtered native tests many times at once in a GPU container limited to 2 CPUs and 2 GiB, and count failures per test (see bin/stress-native.ts)",
    depends: ["runtime","notices"],
    usage: `flag "--filter <expression>" help="Nextest filter expression, for example test(rows_arriving_in_view)"
flag "--count <n>" default="20" help="Iterations each copy runs one after the other"
flag "--copies <n>" default="4" help="Concurrent nextest invocations in the container"
flag "--retries <n>" default="0" help="Nextest retries per test"
flag "--threads <n>" default="2" help="Test processes each copy runs at once; the copies share 2 CPUs"
flag "--deadline-minutes <n>" default="60" help="The container is removed after this long"
`,
    run: "node bin/stress-native.ts --filter \"${usage_filter}\" --count \"${usage_count}\" --copies \"${usage_copies}\" --retries \"${usage_retries}\" --threads \"${usage_threads}\" --deadline-minutes \"${usage_deadline_minutes}\"",
  },
  "test:fonts": {
    description: "Exercise embedded font features and ligature selection through production geometry",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--lib', '--test', 'font_features', '--test', 'inter_features', '--test', 'variable_fonts', '--no-capture'])
` },
  },
  "test:search": {
    description: "Check native ripgrep records and editord-compatible filename matching",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--lib', '--test', 'search_protocol', '--test', 'search_worker', '--test', 'search_scope', '--test', 'search_input'])
` },
  },
  "test:reader-lifecycle": {
    description: "Exercise real disconnected reader channels without crashing worker threads",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--lib'])
` },
  },
  "test:open": {
    description: "Verify latest-request source opening, project boundaries, cancellation, and syntax revisions",
    depends: ["runtime"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--test', 'file_open'])
` },
  },
  "test:cli": {
    description: "Verify native-path argument grammar and executable help without display startup",
    depends: ["runtime","notices"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--test', 'cli_args', '--test', 'cli_process'])
` },
  },
  "test:nextest-config": {
    description: "Check in the build container that cargo-nextest reads .config/nextest.toml of this package and that its timeout kills a hung test",
    run: "node bin/check-nextest-config.ts",
  },
  "test:bin": {
    description: "Check the compositor version reader, the container bound, the frame-pattern analysis, and the runtime publishing helpers of the bin scripts with node:test",
    run: "node --test bin/nested-compositor.test.ts bin/container-bounds.test.ts bin/cadence.test.ts bin/stress-native-report.test.ts bin/runtime-publish.test.ts",
  },
  "test:ignored": {
    description: "Run the gitignore matching unit tests, the tree and reader tests that carry its answers, and the rule file notification test",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--lib', 'git_ignore::'])
// The timing measurement behind the entry bound, ignored by default, prints its numbers.
runCargo(['nextest', 'run', '--no-default-features', '--lib', 'git_ignore::tests::measure', '--run-ignored', 'only', '--no-capture'])
runCargo(['nextest', 'run', '--no-default-features', '--test', 'file_tree', '--test', 'file_tree_validation', '--test', 'file_tree_requests', '--test', 'file_tree_ignored', '--test', 'directory_worker', '--test', 'directory_worker_ignored'])
runCargo(['nextest', 'run', '--no-default-features', '--test', 'change_watch', 'an_ignore_rule_file'])
` },
  },
  "test:tree": {
    description: "Exercise lazy tree snapshots, native names, expansion, and invalid reply rejection",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--test', 'file_tree', '--test', 'file_tree_validation', '--test', 'file_tree_requests', '--test', 'directory_worker'])
` },
  },
  "lint:clippy": {
    description: "Check all IDE targets with warnings denied",
    depends: ["runtime","notices"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['clippy', '--all-targets', '--', '--deny', 'warnings'])
` },
  },
  "lint:release": {
    description: "Check the application binary under the release profile with warnings denied, as the build task compiles it",
    depends: ["runtime","notices"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

// Code behind cfg(debug_assertions) is absent from release builds, so unused items there show only in this profile.
runBounded(['env', '--unset=SLINT_EMIT_DEBUG_INFO', 'cargo', 'clippy', '--release', '--bin', 'monochromatic-ide', '--', '--deny', 'warnings'])
` },
  },
  "inspect:search-runtime": {
    description: "Probe the actual ripgrep runtime available in the bounded IDE environment",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runBounded(['rg', '--version'])
` },
  },
  "inspect:listview-algorithm": {
    description: "Run isolated scroll-algorithm controls against a supplied Slint repeater source file",
    usage: "arg \"<source>\" help=\"Path to Slint internal/core/model/repeater.rs\"",
    run: { shell: "node --input-type=module-typescript -e", run: `const { mkdtempSync, readFileSync, writeFileSync, copyFileSync } = await import('node:fs')
const { homedir } = await import('node:os')
const { join, resolve } = await import('node:path')
const { execFileSync } = await import('node:child_process')
const source = readFileSync(process.env.usage_source, 'utf8')
const start = source.indexOf('fn update_visible_instances(')
const end = source.indexOf('/// Adapter implementing [\`RepeaterInstanceOps\`]', start)
if (start < 0 || end < start) throw new Error('Slint virtualization function boundaries were not found')
const directory = mkdtempSync(join(homedir(), 'temp', 'agent', 'ide-listview-algorithm-'))
writeFileSync(join(directory, 'algorithm.rs'), source.slice(start, end))
copyFileSync(resolve('../../../doc/troubleshooting/slint-listview-random-seek-probe.rs'), join(directory, 'harness.rs'))
const { containerBounds } = await import(resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
const args = ['run', '--rm', '--network=none', '--memory=' + containerLimits.memory, '--cpus=' + containerLimits.cpus, '--pids-limit=128', '--security-opt', 'label=disable', '--volume', directory + ':/work', '--workdir', '/work', 'localhost/monochromatic/ide']
console.log('Standalone algorithm controls: ' + directory)
execFileSync('podman', [...args, 'rustc', '--edition=2024', '--test', 'harness.rs', '--out-dir', '/work'], { stdio: 'inherit' })
execFileSync('podman', [...args, '/work/harness', '--nocapture'], { stdio: 'inherit' })
` },
  },
  "inspect:layout-constraints": {
    description: "Check accepted and rejected Slint sizing declarations using disposable markup",
    run: { shell: "node --input-type=module-typescript -e", run: `const { mkdtempSync, writeFileSync } = await import('node:fs')
const { homedir } = await import('node:os')
const { join } = await import('node:path')
const { spawnSync } = await import('node:child_process')
const directory = mkdtempSync(join(homedir(), 'temp', 'agent', 'ide-layout-constraints-'))
const cases = [
  { name: 'fixed-plus-min', body: 'HorizontalLayout { Rectangle { width: 256px; min-width: 256px; } }', diagnostic: "Cannot specify both 'width' and 'min-width'" },
  { name: 'touch-fixed-plus-min', body: 'TouchArea { width: 48px; min-width: 48px; }', diagnostic: "Cannot specify both 'width' and 'min-width'" },
  { name: 'equal-min-max', body: 'HorizontalLayout { Rectangle { min-width: 256px; max-width: 256px; } }' },
  { name: 'bounded-row-touch', body: 'HorizontalLayout { Rectangle { min-width: 48px; min-height: 48px; TouchArea { width: parent.width; height: parent.height; } } }' },
]
const results = []
for (const item of cases) {
  const file = join(directory, item.name + '.slint')
  writeFileSync(file, 'export component Case inherits Window { ' + item.body + ' }')
  const result = spawnSync('slint-viewer', ['--check', file], { encoding: 'utf8' })
  if (result.error) throw result.error
  const stderr = result.stderr ?? ''
  const matches = item.diagnostic ? result.status !== 0 && stderr.includes(item.diagnostic) : result.status === 0
  if (!matches) throw new Error('Unexpected Slint result for ' + item.name + ': ' + stderr)
  results.push({ name: item.name, exit: result.status, stderr })
}
writeFileSync(join(directory, 'results.json'), JSON.stringify(results, null, 2))
console.log(JSON.stringify({ directory, results }, null, 2))
` },
  },
  "test:find": {
    description: "Check the plain in-file matcher, match navigation, worker identity, match painting, and pinned browser differences",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--lib', '--test', 'find', '--test', 'find_navigation', '--test', 'find_worker', '--test', 'find_reference', '--test', 'find_paint', '--test', 'source_frame'])
` },
  },
  "test:annotations": {
    description: "Check the vertical row mapping, virtual-row rules, inlay-hint and diagnostic layout, painting, stale-snapshot rejection, and the measured placement probe",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--test', 'row_map', '--test', 'virtual_row', '--test', 'annotation', '--test', 'annotation_frame', '--test', 'annotation_layout', '--test', 'annotation_placement', '--test', 'wrap_annotations', '--test', 'source_frame', '--no-capture'])
` },
  },
  "test:annotations-native": {
    description: "Exercise injected inlay hints and diagnostics through native window events in process-isolated tests that render on the GPU",
    depends: ["runtime","notices"],
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runBounded(['cargo', 'nextest', 'run', '--bin', 'monochromatic-ide', '--no-capture', '--filter-expr', 'test(/annotation/)'], { gpu: true })
` },
  },
  "inspect:find-regex": {
    description: "Print the production plain matcher's per-case comparison against captured Chromium find cases",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['nextest', 'run', '--no-default-features', '--test', 'find_reference', '--no-capture'])
` },
  },
  "inspect:find-reference": {
    description: "Probe browser-provided literal find against disposable Unicode and whitespace fixtures",
    run: "node bin/inspect-find-reference.ts",
  },
  "inspect:search-guards": {
    description: "Remove search guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-search-guards.ts",
  },
  "inspect:find-guards": {
    description: "Remove in-file find guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-find-guards.ts",
  },
  "inspect:source-guards": {
    description: "Remove source-view reading guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-source-guards.ts",
  },
  "inspect:sidebar-guards": {
    description: "Remove sidebar-resize guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-sidebar-guards.ts",
  },
  "inspect:theme-guards": {
    description: "Remove live color-scheme repaint guards only in a disposable copy and observe the named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-theme-guards.ts",
  },
  "inspect:annotation-guards": {
    description: "Remove inlay-hint and diagnostic guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-annotation-guards.ts",
  },
  "inspect:ignored-guards": {
    description: "Remove ignored-entry guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-ignored-guards.ts",
  },
  "inspect:watch-guards": {
    description: "Remove change-watch and reread-schedule guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-watch-guards.ts",
  },
  "inspect:refresh-latency": {
    description: "Time external writes until the tree rows and displayed source change, in a disposable package copy",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[runs]" default="3" help="Repeated runs of the same build, for run-to-run spread"
arg "[package]" help="Measure this crate directory below the private agent scratch root instead of the working tree, with the current latency test injected"
arg "[filter]" default="refresh_latency" help="Name part of the ignored native measurement tests to run, for example write_wait_pause"
`,
    run: "node bin/inspect-refresh-latency.ts",
  },
  "inspect:idle-cost": {
    description: "Measure the idle IDE's CPU time, read system calls, and listings with expanded folders, at the shipped safety sweep and a comparison interval",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root, for the chosen profile"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[profile]" default="debug" help="Cargo profile to build and measure: debug or release"
arg "[compare]" default="10" help="Safety sweep of the comparison build, in whole seconds"
arg "[folders]" default="1,12,100" help="Comma-separated counts of expanded folders"
arg "[seconds]" default="60" help="Length of each measured interval; at least three safety sweeps of the slower build"
arg "[runs]" default="3" help="Runs per build and folder count, for run-to-run spread"
`,
    run: "node bin/inspect-idle-cost.ts",
  },
  "inspect:watch-limit": {
    description: "Run the IDE in a disposable user namespace with a lowered inotify watch limit and check one warning, few failing watch calls, refresh through timers, and one line when watches return",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[folders]" default="12" help="Expanded sibling folders"
arg "[limit]" default="4" help="Watch limit of the namespace; below the number of shown directories"
arg "[seconds]" default="60" help="Length of the observed interval under the limit"
`,
    run: "node bin/inspect-watch-limit.ts",
  },
  "inspect:watch-scope": {
    description: "Compare watching every expanded folder with watching only the tree viewport: kernel watches with 60 expanded folders and staleness when a changed folder is scrolled into view",
    usage: `arg "<cache>" help="Disposable Cargo target directory below the private agent scratch root, for the shipped scope"
arg "<viewport_cache>" help="A second disposable Cargo target directory, for the viewport-scope copy"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[runs]" default="3" help="Alternating runs per scope, for run-to-run spread"
`,
    run: "node bin/inspect-watch-scope.ts",
  },
  "inspect:server-watches": {
    description: "Count the inotify watches of confined language servers on large disposable Rust and TypeScript projects",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['build', '--no-default-features', '--bin', 'ide-language-inspect'])
execFileSync('node', ['bin/inspect-server-watches.ts'], { stdio: 'inherit' })
` },
  },
  "inspect:search-accessibility": {
    description: "Verify accepted search landmarks and rejected dialog role declarations in disposable markup",
    run: { shell: "node --input-type=module-typescript -e", run: `const { mkdtempSync, writeFileSync } = await import('node:fs')
const { homedir } = await import('node:os')
const { join } = await import('node:path')
const { spawnSync } = await import('node:child_process')
const directory = mkdtempSync(join(homedir(), 'temp', 'agent', 'ide-search-accessibility-'))
const version = spawnSync('slint-viewer', ['--version'], { encoding: 'utf8' })
if (version.error || version.status !== 0) throw version.error ?? new Error(version.stderr)
const results = []
for (const role of ['search', 'region', 'dialog']) {
  const path = join(directory, role + '.slint')
  writeFileSync(path, 'export component Case inherits Window { Rectangle { accessible-role: ' + role + '; } }')
  const result = spawnSync('slint-viewer', ['--check', path], { encoding: 'utf8' })
  if (result.error) throw result.error
  const expected = role === 'dialog' ? result.status !== 0 && result.stderr.includes("Unknown unqualified identifier 'dialog'") : result.status === 0
  if (!expected) throw new Error('Unexpected accessible role result: ' + role + ': ' + result.stderr)
  results.push({ role, status: result.status, stderr: result.stderr })
}
const record = { directory, version: version.stdout.trim(), results }
writeFileSync(join(directory, 'results.json'), JSON.stringify(record, null, 2))
console.log(JSON.stringify(record, null, 2))
` },
  },
  "lint:slint": {
    description: "Check the native UI declaration",
    run: [
      "slint-viewer --check ui/app.slint",
      "slint-viewer --check ui/tree.slint",
      "slint-viewer --check ui/search.slint",
      "slint-viewer --check ui/find.slint",
      "slint-viewer --check ui/query-input.slint",
      "slint-viewer --check ui/divider.slint",
      "slint-viewer --check ui/hover.slint",
      "slint-viewer --check ui/references.slint",
    ],
  },
  "lint:rustfmt": {
    description: "Check that application-owned Rust sources are formatted as cargo fmt writes them",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['fmt', '--check'])
` },
  },
  "lint": {
    run: [
      "task-run //package/ide/app:lint:clippy",
      "task-run //package/ide/app:lint:rustfmt",
      "task-run lint:monochromatic-lint -- package/ide/app",
      "task-run //package/ide/app:lint:slint",
    ],
  },
  "format:rust": {
    description: "Format application-owned Rust sources",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['fmt'])
` },
  },
  "test:syntax": {
    description: "Check syntax recognition, highlighting, injections, Markdown roles, and their faces: the syntax integration tests",
    usage: "arg \"[filter]\" help=\"Optional test-name filter passed to the test harness\"",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const filter = process.env.usage_filter ? [process.env.usage_filter] : []
runCargo(['nextest', 'run', '--no-default-features', '--test', 'syntax', '--test', 'syntax_injection', '--test', 'syntax_inventory', '--test', 'syntax_markdown', '--test', 'syntax_markdown_faces', '--test', 'syntax_provisioning', '--test', 'syntax_shadowing', ...filter, ...(filter.length ? ['--', '--nocapture'] : [])])
` },
  },
  "test:language": {
    description: "Check the headless Language module: unit rules and scripted-server sessions",
    usage: "arg \"[filter]\" help=\"Optional test-name filter passed to the test harness\"",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const filter = process.env.usage_filter ? [process.env.usage_filter] : []
runCargo(['nextest', 'run', '--no-default-features', '--lib', '--test', 'language', ...filter])
` },
  },
  "inspect:language": {
    description: "Run the five Language feature paths, a reload, and the stale-reply case against real TypeScript and Rust servers on disposable projects",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['build', '--no-default-features', '--bin', 'ide-language-inspect'])
execFileSync('node', ['bin/inspect-language.ts'], { stdio: 'inherit' })
` },
  },
  "inspect:language-lifecycle": {
    description: "Check on real confined servers that open, one request, close, and shutdown log no ERROR-level record or bare shutdown error, and that SIGKILL of the application leaves no server or sandbox process",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['build', '--no-default-features', '--bin', 'ide-language-inspect'])
execFileSync('node', ['bin/inspect-language-lifecycle.ts'], { stdio: 'inherit' })
` },
  },
  "inspect:language-confinement": {
    description: "Run the write-confinement acceptance tests: real servers in bubblewrap on disposable projects, escape and delegation probes, fail-closed cases, and the unconfined guard control",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['build', '--no-default-features', '--bin', 'ide-language-inspect', '--bin', 'ide-scripted-lsp'])
execFileSync('node', ['bin/inspect-language-confinement.ts'], { stdio: 'inherit' })
` },
  },
  "inspect:language-toolchain": {
    description: "Run real servers found through the toolchain lookup of disposable worktrees: a Node and a binary server started confined, a failed lookup that falls back to PATH with its notice, a folder without the toolchain that keeps the PATH message, a hung entry point that is stopped, and project trees compared before and after",
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

runCargo(['build', '--no-default-features', '--bin', 'ide-language-inspect'])
execFileSync('node', ['bin/inspect-language-toolchain.ts'], { stdio: 'inherit' })
` },
  },
  "inspect:language-guards": {
    description: "Remove Language fencing, readiness, edit-refusal, ask-again, hint-window, diagnostics-hold, and start-allowance guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-language-guards.ts",
  },
  "inspect:language-lifecycle-guards": {
    description: "Observe the quiet-lifetime and logging regressions failing in a disposable copy: each helix-lsp re-labelling removed or widened, a full log queue that waits, and the worker's shutdown request, wait, or reaping removed",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated case names to run; default is every case"
`,
    run: "node bin/inspect-language-lifecycle-guards.ts",
  },
  "inspect:log-stall": {
    description: "Check in the nested compositor that a reader that stops reading the IDE's log does not freeze the window, and that the synchronous writer it replaced does (guard control)",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[seconds]" help="Length of each measurement while keys move the caret; default 30"
`,
    run: "node bin/inspect-log-stall.ts",
  },
  "inspect:language-reap-rate": {
    description: "Count, in a disposable copy and a bounded container, how often a lifetime test finds a server process left as a zombie or still running, with the worker's reaping and with the fixed pause it replaced",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[runs]" help="Test processes per measurement; default 600"
arg "[parallel]" help="Test processes at a time, sharing two processors; default 16"
arg "[rounds]" help="Alternations of the two variants; default 2"
`,
    run: "node bin/inspect-language-reap-rate.ts",
  },
  "inspect:language-navigation-guards": {
    description: "Remove native language-navigation guards only in a disposable copy and observe named regression failures",
    usage: `arg "<cache>" help="Existing disposable Cargo target directory below the private agent scratch root"
arg "[cargo]" help="Private copy of the Cargo home below the private agent scratch root; default is the shared ide-cargo volume"
arg "[only]" help="Comma-separated guard names to run; default is every guard"
`,
    run: "node bin/inspect-language-navigation-guards.ts",
  },
  "inspect:native": {
    description: "Run the IDE in the repo compositor with isolated theme and font configuration; the printed control socket takes screenshot, click, move, drag, scroll, key, type, and a11y commands",
    depends: ["runtime","notices"],
    usage: `arg "[scheme]" default="dark" help="Private system appearance: dark or light"
arg "[fixture]" default="basic" help="Source fixture: basic, scroll, syntax, or annotations (injected inlay hints and diagnostics)"
`,
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const { mkdtempSync, mkdirSync, writeFileSync } = await import('node:fs')
const { homedir } = await import('node:os')
const { join, resolve } = await import('node:path')
const scheme = process.env.usage_scheme ?? 'dark'
const fixture = process.env.usage_fixture ?? 'basic'
if (scheme !== 'dark' && scheme !== 'light') { throw new Error('scheme must be dark or light') }
if (!['basic', 'scroll', 'syntax', 'annotations'].includes(fixture)) { throw new Error('fixture must be basic, scroll, syntax, or annotations') }
runCargo(['build', '--bin', 'monochromatic-ide'])
const scratch = mkdtempSync(join(homedir(), 'temp', 'agent', 'monochromatic-ide-native-'))
for (const name of ['config', 'cache', 'data']) { mkdirSync(join(scratch, name)) }
const fontConfig = join(scratch, 'fonts.conf')
writeFileSync(fontConfig, '<fontconfig><dir>/usr/share/fonts</dir><cachedir prefix="xdg">fontconfig</cachedir></fontconfig>')
const env = {
  ...process.env,
  HELIX_RUNTIME: resolve('target/debug/runtime'),
  SLINT_BACKEND: 'winit',
  FONTCONFIG_FILE: fontConfig,
  XDG_CONFIG_HOME: join(scratch, 'config'),
  XDG_CACHE_HOME: join(scratch, 'cache'),
  XDG_DATA_HOME: join(scratch, 'data'),
}
for (const family of ['Inter Variable', 'JetBrains Mono']) {
  const fallback = execFileSync('fc-match', ['--format', '%{family}', family], { env, encoding: 'utf8' }).trim()
  if (fallback.split(',').includes(family)) { throw new Error('font isolation still exposes ' + family) }
  console.error('System-font isolation: ' + family + ' resolves to ' + fallback)
}
const socket = join(scratch, 'control.sock')
// IDE_NATIVE_PROJECT and IDE_NATIVE_FILE open an existing disposable project instead of the generated fixture.
// Language servers run confined and cannot write into the project they analyze; a directory below the
// system temporary directory is still required, so a session never points real servers at a real project.
const { realpathSync } = await import('node:fs')
const { sep } = await import('node:path')
let project = join(scratch, 'project')
let sourceFile = join(project, 'fixture.ts')
if (process.env.IDE_NATIVE_PROJECT) {
  project = realpathSync(process.env.IDE_NATIVE_PROJECT)
  if (!project.startsWith(realpathSync(join(homedir(), 'temp', 'agent')) + sep)) { throw new Error('IDE_NATIVE_PROJECT must be a disposable directory below ' + join(homedir(), 'temp', 'agent')) }
  sourceFile = realpathSync(resolve(project, process.env.IDE_NATIVE_FILE ?? ''))
  if (!sourceFile.startsWith(project + sep)) { throw new Error('IDE_NATIVE_FILE must name a file inside IDE_NATIVE_PROJECT') }
} else {
mkdirSync(project)
const lines = fixture === 'syntax' ? [
  'const 猫: string = "cat";',
  '// Mixed source scripts keep their shared baseline.',
  'function describe(name: string): string {',
  '  return name + " 猫";',
  '}',
  'const answer: number = 42;',
  'const accent = "é";',
  'describe(猫);',
] : ['I am a big cat.', 'Select text and use the native Copy action.', '猫 and é are grapheme test cases.']
if (fixture === 'scroll') { for (let index = 0; index < 200; index++) { lines.push('Line ' + String(index + 4) + ': smooth pixel scrolling, 猫 and Latin.') } }
// The annotations fixture: hints and diagnostics are injected from a file through the debug-only inspection path.
// Its first lines hold one scene each: several hints on one row, hints that would touch on further rows,
// a tab and CJK line with a hint and a message, one message, a message of every severity on one line,
// a long message, a message with line breaks, a range over several lines, and hints together with messages.
// Further lines make the file long, with hints and messages spread over it and on its last line.
// IDE_NATIVE_ANNOTATION_DELAYS=HINTS,DIAGNOSTICS makes them arrive that many milliseconds after startup, and
// IDE_NATIVE_ANNOTATION_RELOAD_DELAYS=HINTS,DIAGNOSTICS makes them arrive again that long after each external reload.
if (fixture === 'annotations') {
  const tab = String.fromCharCode(9)
  const newline = String.fromCharCode(10)
  const accent = 'e' + String.fromCharCode(0x301)
  lines.splice(0, lines.length,
    '// Hints stand on rows above their line, each at the x of its anchor.',
    'const label = "héllo 😀", count = 3, ratio = 0.5;',
    'const total = area(2, 3) + area(count, ratio);',
    'const sized = resize(box(1, 2), 3, 4, 5);',
    tab + 'const 猫 = ' + accent + ' !== x;',
    '// Diagnostic messages stand on rows of their own, beneath the hint rows.',
    'const wrong: number = "not a number";',
    'const every = [first, second, third, fourth, fifth];',
    'const long = configure({ strict: true });',
    'const shape: Shape = { kind: "circle", radius: "1" };',
    'const range = compute(',
    '  first,',
    ');',
    'const mixed = area(count, label);',
    'function area(width: number, height: number): number {',
    '  return width * height;',
    '}')
  const scenes = lines.length
  const filler = 220
  for (let index = 0; index < filler; index++) { lines.push('const value' + String(index) + ' = area(' + String(index) + ', ' + String(index + 1) + ');') }
  lines.push('const last = area(7, 8);', 'export const done = last + missing;')
  const points = (value) => [...value].length
  const starts = []
  let total = 0
  for (const line of lines) { starts.push(total); total += points(line) + 1 }
  const on = (line, needle, within = 0) => { const index = lines[line].indexOf(needle); if (index < 0) { throw new Error('fixture line ' + String(line) + ' lacks ' + needle) } return starts[line] + points(lines[line].slice(0, index)) + within }
  const hint = (line, needle, within, label, kind) => ({ position: on(line, needle, within), label, kind })
  const problem = (line, needle, length, severity, source, code, message) => ({ source, start: on(line, needle), end: on(line, needle, length), severity, code, message })
  const hints = [
    hint(1, 'label', 5, ': string', 'type'), hint(1, 'count', 5, ': number', 'type'), hint(1, 'ratio', 5, ': number', 'type'),
    hint(2, 'total', 5, ': number', 'type'), hint(2, '2,', 0, 'width:', 'parameter'), hint(2, '3)', 0, 'height:', 'parameter'),
    hint(2, 'count', 0, 'width:', 'parameter'), hint(2, 'ratio', 0, 'height:', 'parameter'),
    hint(3, 'sized', 5, ': Box', 'type'), hint(3, 'box(', 0, 'shape:', 'parameter'), hint(3, '1,', 0, 'width:', 'parameter'),
    hint(3, '2)', 0, 'height:', 'parameter'), hint(3, '3,', 0, 'scale:', 'parameter'), hint(3, '4,', 0, 'dx:', 'parameter'), hint(3, '5)', 0, 'dy:', 'parameter'),
    hint(4, '猫', 1, ': boolean', 'type'),
    hint(13, 'mixed', 5, ': number', 'type'), hint(13, 'count', 0, 'width:', 'parameter'), hint(13, 'label', 0, 'height:', 'parameter'),
  ]
  const diagnostics = [
    problem(4, '猫', 1, 'warning', 'ts', '6133', "'猫' is declared but its value is never read."),
    problem(6, '"not a number"', 14, 'error', 'ts', '2322', "Type 'string' is not assignable to type 'number'."),
    problem(7, 'first', 5, 'error', 'ts', '2304', "Cannot find name 'first'."),
    problem(7, 'second', 6, 'warning', 'eslint', 'no-undef', "'second' is not defined."),
    problem(7, 'third', 5, 'information', 'ts', '80001', 'File is a CommonJS module; it may be converted to an ES module.'),
    problem(7, 'fourth', 6, 'hint', 'ts', '6385', "'fourth' is deprecated."),
    problem(7, 'fifth', 5, undefined, 'lint', undefined, 'A server may leave the severity out; such a problem is shown as a warning.'),
    problem(8, '{ strict: true }', 16, 'warning', 'ts', '2345', "Argument of type '{ strict: boolean; }' is not assignable to parameter of type 'Options', because the property 'target' is missing in type '{ strict: boolean; }' but required in type 'Options', which is declared in 'configure.d.ts' and exported again from the entry point of the package."),
    problem(9, 'shape', 5, 'error', 'ts', '2322', ["Type '{ kind: string; radius: string; }' is not assignable to type 'Shape'.", "  Types of property 'radius' are incompatible.", "    Type 'string' is not assignable to type 'number'."].join(newline)),
    { source: 'ts', start: on(10, 'compute'), end: on(12, ')', 1), severity: 'error', code: '2554', message: 'Expected 2 arguments, but got 1.' },
    problem(13, 'label', 5, 'error', 'ts', '2345', "Argument of type 'string' is not assignable to parameter of type 'number'."),
    problem(13, 'mixed', 5, 'hint', 'eslint', 'no-unused-vars', "'mixed' is assigned a value but never used."),
  ]
  for (let index = 0; index < filler; index++) {
    const line = scenes + index
    const name = 5 + String(index).length
    if (index % 6 === 3) { hints.push(hint(line, 'value', name, ': number', 'type')) }
    if (index % 14 === 5) { hints.push(hint(line, '(', 1, 'width:', 'parameter'), hint(line, ', ', 2, 'height:', 'parameter')) }
    if (index % 23 === 11) { diagnostics.push(problem(line, 'value', name, 'hint', 'ts', '6133', "'value" + String(index) + "' is declared but its value is never read.")) }
  }
  const end = lines.length - 1
  hints.push(hint(end - 1, 'last', 4, ': number', 'type'), hint(end - 1, '7', 0, 'width:', 'parameter'), hint(end - 1, '8', 0, 'height:', 'parameter'), hint(end, 'done', 4, ': number', 'type'))
  diagnostics.push(problem(end, 'missing', 7, 'error', 'ts', '2304', "Cannot find name 'missing'."))
  const pair = (name) => {
    const raw = process.env[name]
    if (!raw) { return undefined }
    if (!/^[0-9]+,[0-9]+$/.test(raw)) { throw new Error(name + ' must be HINTS,DIAGNOSTICS in milliseconds') }
    const [early, late] = raw.split(',').map(Number)
    return { hints: early, diagnostics: late }
  }
  const annotations = { hints, diagnostics, delays: pair('IDE_NATIVE_ANNOTATION_DELAYS'), reload_delays: pair('IDE_NATIVE_ANNOTATION_RELOAD_DELAYS') }
  const annotationFile = join(scratch, 'annotations.json')
  writeFileSync(annotationFile, JSON.stringify(annotations, null, 2))
  env.IDE_INSPECT_ANNOTATIONS = annotationFile
  console.error('IDE_INSPECT_ANNOTATIONS=' + annotationFile)
}
writeFileSync(sourceFile, lines.join(String.fromCharCode(10)))
}
console.error('IDE_NATIVE_SOCKET=' + socket)
console.error('IDE_NATIVE_FIXTURE=' + sourceFile)
const { compositor, demandCurrentCompositor } = await import(resolve('bin/nested-compositor.ts'))
demandCurrentCompositor()
// IDE_NATIVE_SIZE starts the nested screen at another size; a locked host session ignores the control socket's resize.
const size = process.env.IDE_NATIVE_SIZE ?? '1100x660'
if (!/^[1-9][0-9]*x[1-9][0-9]*$/.test(size)) { throw new Error('IDE_NATIVE_SIZE must be WIDTHxHEIGHT') }
// IDE_NATIVE_SCALE starts the nested output at another scale; the window keeps its logical size.
const scale = process.env.IDE_NATIVE_SCALE ?? '1'
if (!/^[0-9]+([.][0-9]+)?$/.test(scale)) { throw new Error('IDE_NATIVE_SCALE must be a number such as 2 or 1.25') }
// --accessibility serves the accessibility bus, so the control socket's a11y command reads the window's element tree
// (roles, names, states, and screen bounds). It is on here for the agent that drives this session and off in the
// measuring tasks, which run the application as shipped.
execFileSync(compositor, ['--socket', socket, '--size', size, '--scale', scale, '--color-scheme', scheme, '--accessibility', '--', resolve('target/debug/monochromatic-ide'), project, '--file', sourceFile], { env, stdio: 'inherit' })
` },
  },
  "inspect:cadence": {
    description: "Record the IDE's frames in the repo compositor during a scroll or a held minimap drag and print how many changed or stayed identical, with the longest identical run and every run",
    depends: ["runtime","notices"],
    usage: `arg "<scenario>" help="Input to send: scroll (a two-finger scroll over the source) or drag (hold the minimap viewport box and pull it down)"
arg "<project>" help="Disposable project directory below the system temporary directory"
arg "<file>" help="File to open, relative to the project"
arg "[profile]" default="debug" help="Cargo profile of the binary to build and drive: debug or release"
arg "[size]" default="1100x660" help="Nested screen size as WIDTHxHEIGHT"
arg "[steps]" default="120" help="Pointer events of the input, sent 16 ms apart"
`,
    run: { shell: "node --input-type=module-typescript -e", run: `const { execFileSync } = await import('node:child_process')
// Node strips the types of the shared bound module; the import names an absolute path because the task runs from -e.
const { containerBounds } = await import((await import('node:path')).resolve('bin/container-bounds.ts'))
const containerLimits = containerBounds()
console.error(containerLimits.summary)
// IDE_RELEASE=1 selects the Ubuntu 24.04 release image that shipped binaries are compiled in (Containerfile.release).
const release = process.env.IDE_RELEASE === '1'
const image = release ? 'localhost/monochromatic/ide-release' : 'localhost/monochromatic/ide'
const imageCheck = (await import('node:child_process')).spawnSync('podman', ['image', 'exists', image])
if (imageCheck.status !== 0) {
  execFileSync('podman', ['build', ...containerLimits.imageBuildArguments, '--tag', image, '--file', release ? 'Containerfile.release' : 'Containerfile', '.'], { stdio: 'inherit' })
}
// The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
// It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
// empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
const rootClippyConfig = (await import('node:path')).resolve(process.cwd(), '../../../clippy.toml')
// A gpu run passes the host's render nodes and the RADV driver file (bin/gpu-arguments.ts, shared with the guard
// inspection scripts). Pass it only to the tasks that render on the GPU.
const { gpuArguments } = await import((await import('node:path')).resolve('bin/gpu-arguments.ts'))
const runBounded = (args, { network = false, gpu = false, ...options } = {}) => execFileSync('podman', [
  'run', '--rm', ...(network ? [] : ['--network=none']), ...(gpu ? gpuArguments : []), ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', process.cwd() + ':/work', '--volume', 'ide-cargo:/cargo',
  '--volume', rootClippyConfig + ':/clippy-conf/clippy.toml:ro', '--env', 'CLIPPY_CONF_DIR=/clippy-conf',
  '--workdir', '/work', ...containerLimits.environmentArguments,
  '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  image, ...args
], { stdio: 'inherit', ...options })
const runCargo = (args, options = {}) => runBounded(['cargo', ...args], options)

const profile = process.env.usage_profile ?? 'debug'
if (profile !== 'debug' && profile !== 'release') { throw new Error('profile must be debug or release') }
// Only the application binary, built as the build:debug and build tasks build it, so the driven binary is the shipped one.
if (profile === 'release') {
  runBounded(['env', '--unset=SLINT_EMIT_DEBUG_INFO', 'cargo', 'build', '--release', '--bin', 'monochromatic-ide'])
} else {
  runCargo(['build', '--bin', 'monochromatic-ide'])
}
execFileSync('node', ['bin/inspect-cadence.ts'], { stdio: 'inherit' })
` },
  },
  "run": {
    description: "Run the debug IDE from the build:debug task against a local root supplied as an argument",
    run: "./target/debug/monochromatic-ide",
  },
} satisfies PackageTaskFile;
