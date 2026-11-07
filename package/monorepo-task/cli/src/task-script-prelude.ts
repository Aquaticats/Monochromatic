/**
 Shared JavaScript prelude prepended to `node --input-type=module-typescript`
 bodies, exposing the helpers the old mise vars interpolated.
 */
export const BODY_PRELUDE = `
const { execFileSync, spawn } = await import('node:child_process')
const { once } = await import('node:events')
const { existsSync, mkdirSync, chmodSync, readdirSync, statSync, mkdtempSync, readFileSync, globSync } = await import('node:fs')
const { availableParallelism, homedir } = await import('node:os')
const { dirname, join, relative, resolve } = await import('node:path')
const REPO_ROOT = process.env.TASK_REPO_ROOT
const scratchBase = () => {
  const base = join(REPO_ROOT, 'node_modules', '.monochromatic', 'task-scratch')
  mkdirSync(base, { recursive: true })
  return base
}
const runWorkspaceNode = (packagePath, entryName, entryArgs) => {
  const packageRoot = join(REPO_ROOT, packagePath)
  const built = join(packageRoot, 'dist', 'final', 'node', entryName + '.mjs')
  const source = join(packageRoot, 'src', entryName + '.ts')
  const entry = existsSync(built) ? built : existsSync(source) ? source : null
  if (entry === null) { throw new Error('task.ts: missing built or source entry: ' + packagePath + '/' + entryName) }
  execFileSync(process.execPath, [entry, ...entryArgs], { stdio: 'inherit' })
}
const runTaskCli = (name, args = []) => {
  execFileSync(process.execPath, [join(REPO_ROOT, 'package', 'monorepo-task', 'cli', 'src', 'run.ts'), name, ...args], { stdio: 'inherit', env: { ...process.env, TASK_CLI_CHILD: '1' } })
}
const newestMtime = (path) => {
  const stats = statSync(path)
  if (!stats.isDirectory()) { return stats.mtimeMs }
  return readdirSync(path, { withFileTypes: true }).reduce((newest, entry) => {
    const entryPath = join(path, entry.name)
    if (entry.isDirectory()) { return Math.max(newest, newestMtime(entryPath)) }
    if (entry.isFile()) { return Math.max(newest, statSync(entryPath).mtimeMs) }
    return newest
  }, stats.mtimeMs)
}
const ensureOxlintConfig = () => {
  const oxlintConfigPackageDir = join(REPO_ROOT, 'package', 'config', 'oxlint')
  const oxlintPluginPackagesDir = join(REPO_ROOT, 'package', 'oxlint-plugin')
  const oxlintPluginPackageDirs = existsSync(oxlintPluginPackagesDir)
    ? readdirSync(oxlintPluginPackagesDir, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => join(oxlintPluginPackagesDir, entry.name))
      .filter((packageDir) => existsSync(join(packageDir, 'package.json')))
    : []
  const oxlintPackageDirs = [oxlintConfigPackageDir, ...oxlintPluginPackageDirs]
  const oxlintConfigDistDir = join(oxlintConfigPackageDir, 'dist', 'final', 'node')
  const oxlintConfigSrcDir = join(oxlintConfigPackageDir, 'src')
  const oxlintSidecarOutputPaths = readdirSync(oxlintConfigSrcDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.startsWith('plugin-') && entry.name.endsWith('.ts'))
    .map((entry) => join(oxlintConfigDistDir, entry.name.slice(0, -3) + '.mjs'))
  const outputPaths = [join(oxlintConfigDistDir, 'index.mjs'), ...oxlintSidecarOutputPaths]
  const sourcePaths = [
    ...oxlintPackageDirs.flatMap((packageDir) => [
      join(packageDir, 'src'),
      join(packageDir, 'package.json'),
      join(packageDir, 'tsconfig.json'),
      join(packageDir, 'rolldown.node.config.ts'),
    ]),
    join(REPO_ROOT, 'pnpm-lock.yaml'),
    join(REPO_ROOT, 'pnpm-workspace.yaml'),
  ].filter((path) => existsSync(path))
  const missingOutput = outputPaths.some((outputPath) => !existsSync(outputPath))
  const stale = !missingOutput
    && Math.max(...sourcePaths.map(newestMtime)) > Math.min(...outputPaths.map((outputPath) => newestMtime(outputPath)))
  if (missingOutput || stale) { runTaskCli('//package/config/oxlint:build:js:node') }
}
const parseUsageArgs = (raw) => {
  const argv = []
  let current = null
  let index = 0
  while (index < raw.length) {
    const char = raw[index]
    if (current === null) {
      if (char === ' ') { index += 1; continue }
      current = ''
    }
    if (char === '\\\\') { current += raw[index + 1] ?? ''; index += 2; continue }
    if (char === "'" || char === '"') {
      index += 1
      while (index < raw.length && raw[index] !== char) { current += raw[index]; index += 1 }
      index += 1
      continue
    }
    if (char === ' ') { argv.push(current); current = null; index += 1; continue }
    current += char
    index += 1
  }
  if (current !== null) { argv.push(current) }
  return argv
}
const runTestFiles = async (files, nodeArgs) => {
  if (files.length === 0) { return }
  const testFileCwd = (file) => {
    let current = dirname(resolve(file))
    while (true) {
      if (existsSync(join(current, 'package.json')) && existsSync(join(current, 'task.ts'))) { return current }
      const parent = dirname(current)
      if (parent === current) { return REPO_ROOT }
      current = parent
    }
  }
  const env = { ...process.env }
  if (env.MONOCHROMATIC_LOG_DIRECTORY === undefined) {
    const agentScratch = join(homedir(), 'temp', 'agent')
    execFileSync('mkdir', ['-p', '-m', '700', agentScratch], { stdio: 'inherit' })
    const stamp = new Date().toISOString().replaceAll(':', '-')
    const runLogDirectory = join(agentScratch, 'module-test-logs', stamp + '-' + process.pid)
    execFileSync('mkdir', ['-p', '-m', '700', runLogDirectory], { stdio: 'inherit' })
    env.MONOCHROMATIC_LOG_DIRECTORY = runLogDirectory
  }
  const limit = Math.max(1, Math.min(availableParallelism(), files.length))
  const failures = []
  let next = 0
  const worker = async () => {
    while (next < files.length) {
      const file = files[next]
      next += 1
      const proc = spawn(process.execPath, [...nodeArgs, file], { cwd: testFileCwd(file), stdio: 'inherit', env })
      const [code] = await once(proc, 'exit')
      if (code !== 0) { failures.push(file) }
    }
  }
  await Promise.all(Array.from({ length: limit }, worker))
  if (failures.length > 0) { throw new Error('task.ts: test files failed: ' + failures.join(', ')) }
}
const onPath = (command) => { try { execFileSync('which', [command], { stdio: 'ignore' }); return true } catch { return false } }
const pkgConfigOk = () => { try { execFileSync('pkg-config', ['--exists', 'libpipewire-0.3', 'fontconfig', 'freetype2'], { stdio: 'ignore' }); return true } catch { return false } }
const hostOk = onPath('cargo') && pkgConfigOk()
const hostEnv = () => {
  const clangDirs = globSync('/usr/lib/clang/*').sort()
  const resourceDir = clangDirs[clangDirs.length - 1]
  return { ...process.env, LIBCLANG_PATH: '/usr/lib64', BINDGEN_EXTRA_CLANG_ARGS: '-resource-dir=' + resourceDir }
}
const defaultContainer = { image: 'localhost/monochromatic/music-player', containerfile: 'Containerfile', cargoVolume: 'music-player-cargo', limits: [], env: [], program: 'cargo' }
// The build arguments the Containerfiles declare without a default: the CachyOS base image of this host's CPU level, the Rust
// nightly and the Zig archive that toolchain.lock.json records. Each comes from a core-matrix command, never from a copy here.
const coreMatrixCommand = (name, commandArgs) => execFileSync('node', [join(REPO_ROOT, 'package', 'test-harness', 'core-matrix', 'src', 'bin', name + '.ts'), ...commandArgs], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim()
const cachyosBuildArguments = (imageName) => ['--build-arg', coreMatrixCommand('cachyos-image', ['--image-name', imageName, '--build-argument'])]
const rustToolchainBuildArguments = () => ['--build-arg', coreMatrixCommand('locked-rust-toolchain', ['--build-argument'])]
const zigArchiveBuildArguments = () => coreMatrixCommand('locked-tool-build-arguments', ['zig']).split(String.fromCharCode(10)).flatMap((line) => ['--build-arg', line])
// Builds the image when it is missing or was built from other build arguments than these (the digest rides on the image as a label).
const containerImageBuild = ({ image, containerfile, buildArguments }) => execFileSync('node', [join(REPO_ROOT, 'package', 'test-harness', 'core-matrix', 'src', 'bin', 'container-image-build-if-inputs-changed.ts'), '--image', image, '--containerfile', containerfile, '--', ...buildArguments], { stdio: 'inherit' })
const containerCargo = (cargoArgs, containerOptions = {}) => {
  const container = { ...defaultContainer, ...containerOptions }
  containerImageBuild({ image: container.image, containerfile: container.containerfile, buildArguments: [...cachyosBuildArguments(container.image.split('/').pop()), ...rustToolchainBuildArguments(), ...(container.containerfile === 'Containerfile' ? zigArchiveBuildArguments() : [])] })
  const debugInfoEnv = process.env.SLINT_EMIT_DEBUG_INFO ? ['-e', 'SLINT_EMIT_DEBUG_INFO=1'] : []
  const siblingMounts = [['../truepeak-core', '/truepeak-core'], ['../android-app/app/src/test/resources', '/android-app/app/src/test/resources'], ['../../../clippy.toml', '/clippy.toml']]
  const siblingMountArgs = siblingMounts.flatMap(([relative, inside]) => {
    const host = resolve(process.cwd(), relative)
    if (!existsSync(host)) { throw new Error('containerCargo needs ' + host + ' (mounted read-only at ' + inside + ') but it does not exist') }
    return ['-v', host + ':' + inside + ':ro']
  })
  execFileSync('podman', ['run', '--rm', ...container.limits, '--security-opt', 'label=disable', ...debugInfoEnv, ...container.env, '-v', process.cwd() + ':/work', ...siblingMountArgs, '-v', container.cargoVolume + ':/cargo', '-w', '/work', container.image, container.program, ...cargoArgs], { stdio: 'inherit' })
}
const runCargo = (cargoArgs) => {
  if (process.platform !== 'linux') { execFileSync('cargo', cargoArgs, { stdio: 'inherit' }) }
  else if (hostOk) { execFileSync('cargo', cargoArgs, { stdio: 'inherit', env: hostEnv() }) }
  else { containerCargo(cargoArgs) }
}
const gnuTarget = 'x86_64-pc-windows-gnu'
const gnuContainer = {
  image: 'localhost/monochromatic/music-player-windows-gnu',
  containerfile: 'Containerfile.windows-gnu',
  cargoVolume: 'music-player-cargo-windows-gnu',
  limits: ['--memory=6g', '--cpus=4'],
  env: ['-e', 'CARGO_TARGET_DIR=/work/target/windows-gnu'],
}
const runWindowsGnu = (cargoArgs) => {
  if (process.platform === 'win32') { execFileSync('cargo', cargoArgs, { stdio: 'inherit' }) }
  else if (process.platform === 'linux') { containerCargo(cargoArgs, gnuContainer) }
  else { throw new Error('The x86_64-pc-windows-gnu tasks run on Windows (MSYS2 MinGW-w64) or Linux (container + Wine), not ' + process.platform) }
}

`;
