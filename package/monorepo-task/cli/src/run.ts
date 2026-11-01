#!/usr/bin/env node

/**
 Mise-free task CLI for this monorepo's CI.

 Runs the same task graph CI used to run through `mise run`, but resolves every
 body from `task.ts` files and executes it with the current node, so CI no
 longer installs or invokes mise. Task bodies keep calling workspace tool
 binaries by bare name because `PATH` carries every `node_modules/.bin` up to
 the repository root, the way `mise.toml` `_.path` did. Supports plain `run`
 strings, `run` arrays (sequenced, with `{ shell }` items), `depends`, `extends`
 onto root templates, `usage` options mapped to `usage_*` variables like mise,
 root `build`/`test`/`lint` umbrella tasks, root inline tasks, and the
 `//package/...:task` form. Tasks without a `usage` spec get post-`--`
 arguments appended to their command as a real argv vector, the way mise did,
 and `usage_args` round-trips words that contain spaces.

 @example
 ```bash
 node package/monorepo-task/cli/src/run.ts //package/logger/core:build
 node package/monorepo-task/cli/src/run.ts //package/logger/core:test:unit -- --all
 node package/monorepo-task/cli/src/run.ts prepare:pnpm
 ```
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readdirSync,
} from 'node:fs';
import { availableParallelism, homedir, } from 'node:os';
import {
  delimiter,
  dirname,
  join,
  resolve,
} from 'node:path';
import {
  loadPackageTasks,
  loadRootTasks,
  repoRoot,
  resolveDefinition,
} from './load.ts';
import type {
  TaskDefinition,
  TaskStep,
} from './types.ts';



//region Constants: repo layout and root umbrella tasks

/**
 File that marks the monorepo root, matching `MISE_MONOREPO` in `module-fs-path`.
 */
const ROOT_MARKER = 'pnpm-workspace.yaml';

/**
 Root umbrella tasks with cross-package fanout: name to the package task suffix
 every package `task.ts` carrying that suffix runs.
 */
const ROOT_FANOUT_TASKS: Readonly<Record<string, string>> = {
  'build': 'build',
  'test': 'test',
  'test:unit': 'test:unit',
  'lint': 'lint',
};

/**
 Directory segments below `package/`: category plus package name.
 */
const PACKAGE_SEGMENT_COUNT = 2;

/**
 Shared JavaScript prelude prepended to `node --input-type=module-typescript`
 bodies, exposing the helpers the old mise vars interpolated.
 */
const BODY_PRELUDE = `
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
const containerCargo = (cargoArgs, containerOptions = {}) => {
  const container = { ...defaultContainer, ...containerOptions }
  let imageExists = true
  try { execFileSync('podman', ['image', 'exists', container.image], { stdio: 'ignore' }) } catch { imageExists = false }
  if (!imageExists) { execFileSync('podman', ['build', '-t', container.image, '-f', container.containerfile, '.'], { stdio: 'inherit' }) }
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
const fanoutChildren = async (taskName) => {
  const separator = taskName.indexOf(':', 2)
  const packagePath = taskName.slice(2, separator)
  const suffix = taskName.slice(separator + 1)
  const tasks = (await import(join(REPO_ROOT, packagePath, 'task.ts'))).default
  const prefix = suffix + ':'
  const depth = prefix.split(':').length
  const children = Object.keys(tasks)
    .filter((name) => name.startsWith(prefix) && name.split(':').length === depth)
    .toSorted()
  if (children.length === 0) { throw new Error('task.ts: no child tasks for ' + taskName) }
  const failures = []
  const processes = children.map((name) => spawn(
    process.execPath,
    [join(REPO_ROOT, 'package', 'monorepo-task', 'cli', 'src', 'run.ts'), '//' + packagePath + ':' + name],
    { stdio: 'inherit', env: { ...process.env, TASK_CLI_CHILD: '1' } },
  ))
  const exits = await Promise.all(processes.map((proc) => once(proc, 'exit')))
  for (const [index, [code]] of exits.entries()) {
    if (code !== 0) { failures.push(children[index]) }
  }
  if (failures.length > 0) { throw new Error('task.ts: fanout child tasks failed: ' + failures.join(', ')) }
}
`;

//endregion

//region Repo layout: locating the root and packages

/**
 Finds the monorepo root by walking up for the workspace marker.

 @returns Absolute path of the monorepo root.
 @throws When no root marker is found from the current directory.
 */
function findRepoRoot(): string {
  let current = process.cwd();
  while (true) {
    if (existsSync(join(current, ROOT_MARKER))) { return current; }
    const parent = dirname(current);
    if (parent === current)
      throw new Error(`task-cli: no ${ROOT_MARKER} above ${process.cwd()}`);
    current = parent;
  }
}

/**
 Manifest files that mark a package directory: the npm, Cargo and Gradle homes
 this monorepo's packages use.
 */
const PACKAGE_MARKERS = ['package.json', 'Cargo.toml', 'build.gradle.kts', 'build.gradle',] as const;

/**
 Resolves the package directory for a `//package/<category>/<name>[:task]` name.

 @param root - Monorepo root.
 @param packagePath - Path like `package/logger/core`.
 @returns Absolute package directory.
 @throws When the path is not a `package/` path with two segments below it, or has no manifest.
 */
function packageDirectory(root: string, packagePath: string): string {
  const segments = packagePath.split('/');
  if (segments[0] !== 'package' || segments.length !== 1 + PACKAGE_SEGMENT_COUNT)
    throw new Error(`task-cli: not a package path: ${packagePath}`);
  const directory = join(root, packagePath);
  if (!PACKAGE_MARKERS.some((marker) => existsSync(join(directory, marker))))
    throw new Error(`task-cli: no package manifest (${PACKAGE_MARKERS.join(', ')}) at ${packagePath}`);
  return directory;
}

/**
 Walks `package/` for task files without reading ignored dependency trees.

 @param root - Monorepo root.
 @returns Absolute paths of every package `task.ts`, sorted.
 */
function findPackageTaskFiles(root: string): readonly string[] {
  const packageRoot = join(root, 'package');
  const files: string[] = [];
  for (const first of readdirSync(packageRoot, { withFileTypes: true, })) {
    if (!first.isDirectory()) { continue; }
    const categoryDirectory = join(packageRoot, first.name);
    for (const second of readdirSync(categoryDirectory, { withFileTypes: true, })) {
      if (!second.isDirectory()) { continue; }
      const taskPath = join(categoryDirectory, second.name, 'task.ts');
      if (existsSync(taskPath)) { files.push(taskPath); }
    }
  }
  return files.toSorted();
}

//endregion

//region Usage options: mapping `usage` specs to `usage_*` variables

/**
 One parsed `usage` flag or arg from a task definition.
 */
type UsageOption = {
  readonly kind: 'flag' | 'arg';
  readonly name: string;
  readonly valueName?: string;
};

/**
 Parses a mise-style `usage` spec into flag and arg descriptors. Flag names
 drop their leading dashes so they match `--name` tokens and the `usage_name`
 variable the old mise mapping produced.

 @param usage - Raw usage string from a task definition.
 @returns Parsed options in declaration order.
 */
function parseUsageSpec(usage: string | undefined): readonly UsageOption[] {
  if (usage === undefined) { return []; }
  const options: UsageOption[] = [];
  const pattern = /(flag|arg)\s+"([^"]+)"(?:\s+var=#true)?/gu;
  for (const match of usage.matchAll(pattern)) {
    const kind = match[1] === 'flag' ? 'flag' : 'arg';
    const body = match[2];
    if (body === undefined) { continue; }
    // A flag's name stops before its `<value>`; an arg's name is its whole first word, `<cache>` included.
    const nameMatch = (kind === 'arg' ? /^(?<name>\S+)/u : /^(?<name>[^\s<]+)/u).exec(body);
    if (nameMatch === null) { continue; }
    // Flags drop their dashes; positional args drop the `[ ]` or `< >` that mark them optional or required.
    const name = nameMatch.groups?.['name']?.replace(/^-+/u, '').replace(/^[[<]|[\]>]$/gu, '');
    if (name === undefined || name === '') { continue; }
    const valueName = /<(?<value>[^>]+)>/u.exec(body)?.groups?.['value'];
    options.push(valueName === undefined ? { kind, name, } : { kind, name, valueName, });
  }
  return options;
}

/**
 Splits task arguments the way `task-run <task> [-- options] [-- passthrough]`
 documents: one leading `--` separates them from anything before it, and the
 next `--` starts raw passthrough.

 @param argv - Arguments after the task name.
 @returns Option arguments before the passthrough separator and raw arguments after it.
 */
function splitPassthrough(argv: readonly string[]): { options: readonly string[]; rest: readonly string[]; } {
  const afterSeparator = argv[0] === '--' ? argv.slice(1) : argv;
  const index = afterSeparator.indexOf('--');
  if (index === -1) { return { options: afterSeparator, rest: [], }; }
  return { options: afterSeparator.slice(0, index), rest: afterSeparator.slice(index + 1), };
}

/**
 Quotes one `usage_args` word for the shell-word grammar `parseUsageArgs`
 reads back: bare when plain, single-quoted with an escaped quote otherwise.

 @param word - Raw argument.
 @returns Encoded word.
 */
function shellQuote(word: string): string {
  if (word.length > 0 && !/[\s'"\\]/u.test(word)) { return word; }
  return `'${word.replaceAll("'", `'\\''`)}'`;
}

/**
 Joins arguments into one `usage_args` value that `parseUsageArgs` recovers
 word for word, so paths keep their spaces the way the old mise var did.

 @param words - Raw arguments.
 @returns Shell-word encoded string.
 */
function shellJoin(words: readonly string[]): string {
  return words.map(shellQuote).join(' ');
}

/**
 Builds the `usage_*` environment for a task from its options.

 Tasks without a usage spec take every post-`--` argument as command argv, the
 way mise appended it, and still see the words in `usage_args`. Tasks with a
 spec map declared flags to `usage_*` variables and keep positional and
 passthrough words in `usage_args`.

 @param options - Parsed usage descriptors.
 @param argv - Task option arguments.
 @param rest - Passthrough arguments after `--`.
 @returns Environment entries to set for the task body.
 @throws On unknown options, missing flag values, or misplaced positional args.
 */
function usageEnvironment(options: readonly UsageOption[], argv: readonly string[], rest: readonly string[],): Record<string, string> {
  const env: Record<string, string> = {};
  if (options.length === 0) {
    const raw = [...argv, ...rest];
    if (raw.length > 0) { env['usage_args'] = shellJoin(raw); }
    return env;
  }
  const positional: string[] = [];
  let index = 0;
  while (index < argv.length) {
    const token = argv[index]!;
    if (token.startsWith('--')) {
      const name = token.slice(2);
      const option = options.find((candidate) => candidate.kind === 'flag' && candidate.name === name,);
      if (option === undefined) { throw new Error(`task-cli: unknown option --${name}`); }
      const variable = `usage_${name.replaceAll('-', '_')}`;
      if (option.valueName === undefined) {
        env[variable] = 'true';
        index += 1;
        continue;
      }
      const value = argv[index + 1];
      if (value === undefined) { throw new Error(`task-cli: option --${name} needs <${option.valueName}>`); }
      env[variable] = value;
      index += 2;
      continue;
    }
    positional.push(token);
    index += 1;
  }
  const args = options.filter((option) => option.kind === 'arg',);
  if (positional.length > 0 && args.length === 0)
    throw new Error(`task-cli: unexpected positional arguments: ${positional.join(' ')}`);
  if (args.length > 0 || rest.length > 0)
    env['usage_args'] = shellJoin([...positional, ...rest]);
  // mise also set one `usage_<name>` variable per declared positional arg, in declaration order; bodies such as
  // the IDE bundle inspection read `usage_file`, `usage_runtime` and `usage_only`. The variadic `args` name keeps
  // meaning the whole word list in `usage_args` above.
  const named = args.filter((option) => option.name !== 'args',);
  for (const [index, option,] of named.entries()) {
    const word = positional[index];
    if (word !== undefined) { env[`usage_${option.name.replaceAll('-', '_')}`] = word; }
  }
  return env;
}

//endregion

//region Task execution: names, depends, body steps and fanout

/**
 Path form of a package directory for `//package/...:task` names.

 @param root - Monorepo root.
 @param cwd - Absolute task working directory.
 @returns `package/...` path relative to the root.
 */
function relativePackagePath(root: string, cwd: string): string {
  return resolve(root, cwd).slice(root.length + 1);
}

/**
 Runs one body step and waits for completion.

 Shell steps receive `argv` the way mise appended post-`--` arguments to the
 command: as a real word vector after the body. Node-inline steps never receive
 argv because node would read the words as its own flags; their bodies read
 `usage_args` instead.

 @param call - Step, working directory, environment and body argv.
 @throws When the shell exits non-zero.
 */
async function runStepAsync({ step, cwd, env, argv, }: { step: TaskStep; cwd: string; env: NodeJS.ProcessEnv; argv: readonly string[]; },): Promise<void> {
  const [shell, rawBody] = typeof step === 'string' ? ['bash', step,] : [step.shell, step.run,];
  const [command, ...shellArgs] = shell.split(' ');
  const isNodeInline = command === 'node' && shellArgs.includes('--input-type=module-typescript');
  // The body braces keep its own `const` imports from colliding with the
  // prelude's helper bindings while the prelude helpers stay visible; the
  // blank line keeps a trailing `//` comment from swallowing the closing brace.
  const body = isNodeInline ? `${BODY_PRELUDE}\n{\n${rawBody}\n\n}\n` : rawBody;
  const script = !isNodeInline && argv.length > 0 ? `${body} "$@"` : body;
  const commandArgs = isNodeInline
    ? [...shellArgs, script,]
    : [...shellArgs, '-c', script, 'task-run', ...argv,];
  const proc = spawn(command!, commandArgs, { cwd, env, stdio: 'inherit', },);
  const [code] = await once(proc, 'exit',);
  if (code !== 0) { throw new Error(`task-cli: step exited ${code}`); }
}

/**
 Executes one named task with options after `--`.

 @param root - Monorepo root.
 @param name - `//package/path:task` or root task name.
 @param argv - Option and passthrough arguments.
 @throws When the task is unknown or any step fails.
 */
async function runTaskByName(root: string, name: string, argv: readonly string[],): Promise<void> {
  const { options: optionArgs, rest, } = splitPassthrough(argv);
  if (name.startsWith('//')) {
    const separator = name.indexOf(':', 2);
    const packagePath = name.slice(2, separator === -1 ? undefined : separator);
    const taskName = separator === -1 ? '' : name.slice(separator + 1);
    const directory = packageDirectory(root, packagePath);
    const tasks = await loadPackageTasks(join(directory, 'task.ts'));
    const raw = tasks[taskName];
    if (raw === undefined) { throw new Error(`task-cli: no task ${taskName} in ${packagePath}`); }
    await executeTask(root, directory, await resolveDefinition(raw,), optionArgs, rest,);
    return;
  }
  if (Object.hasOwn(ROOT_FANOUT_TASKS, name)) {
    if (optionArgs.length > 0 || rest.length > 0)
      throw new Error(`task-cli: ${name} takes no arguments`);
    await fanoutPackages(root, ROOT_FANOUT_TASKS[name]!);
    return;
  }
  const rootTasks = await loadRootTasks();
  const raw = rootTasks.tasks[name];
  if (raw === undefined) { throw new Error(`task-cli: unknown task: ${name}`); }
  await executeTask(root, root, await resolveDefinition(raw,), optionArgs, rest,);
}

/**
 Builds the `PATH` fragment every task body step runs with.

 Ported bodies call workspace tool binaries (`rolldown`, `oxlint`) by bare name,
 the way `mise run` resolved them through the `node_modules/.bin` entries
 `mise.toml` `_.path` lists. Prepending the task directory and every ancestor up
 to the repository root keeps those bodies unchanged while `task-run` resolves
 the same binaries on a runner that never installs mise.

 @param locations - Repository root and task working directory; the walk needs both ends.
 @returns Environment fragment carrying joined `PATH`, inherited value last so system tools stay reachable.
 */
function taskPathEnvironment({ root, cwd, }: { root: string; cwd: string; },): NodeJS.ProcessEnv {
  const pathKey = Object.keys(process.env,).find((key) => key.toLowerCase() === 'path',) ?? 'PATH';
  const binDirectories: string[] = [];
  for (let directory = cwd; ;) {
    binDirectories.push(join(directory, 'node_modules', '.bin',),);
    if (directory === root) { break; }
    const parent = dirname(directory);
    if (parent === directory) { break; }
    directory = parent;
  }
  binDirectories.push(process.env[pathKey] ?? '',);
  return { [pathKey]: binDirectories.join(delimiter,), };
}

/**
 Executes a resolved task: depends first, then its body steps.

 @param root - Monorepo root.
 @param cwd - Directory the body runs in.
 @param definition - Resolved task definition.
 @param optionArgs - Task option arguments.
 @param rest - Passthrough arguments after `--`.
 @throws When a dependency or body step fails.
 */
async function executeTask(root: string, cwd: string, definition: TaskDefinition, optionArgs: readonly string[], rest: readonly string[],): Promise<void> {
  const packagePath = relativePackagePath(root, cwd);
  for (const dependency of definition.depends ?? []) {
    const name = dependency.startsWith('//')
      ? dependency
      : packagePath.startsWith('package/')
        ? `//${packagePath}:${dependency}`
        : dependency;
    await runTaskByName(root, name, [],);
  }
  const steps = definition.run === undefined
    ? []
    : Array.isArray(definition.run) ? definition.run : [definition.run,];
  if (steps.length === 0) { return; }
  // The task's `dir` moves the body into a subdirectory, matching the old
  // mise tasks; without it the body runs in the package directory.
  const bodyCwd = definition.dir === undefined ? cwd : resolve(cwd, definition.dir);
  const usageOptions = parseUsageSpec(definition.usage,);
  const bodyArgv = usageOptions.length === 0 ? [...optionArgs, ...rest,] : [];
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    ...taskPathEnvironment({ root, cwd: bodyCwd, }),
    ...definition.env,
    TASK_REPO_ROOT: root,
    TASK_PACKAGE_PATH: packagePath,
    ...usageEnvironment(usageOptions, optionArgs, rest,),
  };
  for (const step of steps) { await runStepAsync({ step, cwd: bodyCwd, env, argv: bodyArgv, },); }
}

/**
 Runs every package task whose name ends in the suffix, in a bounded pool.

 @param root - Monorepo root.
 @param suffix - Task suffix like `build` or `test:unit`.
 @throws When no package task matches or any child fails.
 */
async function fanoutPackages(root: string, suffix: string,): Promise<void> {
  const runnable: { name: string; }[] = [];
  for (const file of findPackageTaskFiles(root)) {
    const tasks = await loadPackageTasks(file);
    if (!Object.hasOwn(tasks, suffix)) { continue; }
    const packagePath = relativePackagePath(root, dirname(file));
    runnable.push({ name: `//${packagePath}:${suffix}`, });
  }
  if (runnable.length === 0) { throw new Error(`task-cli: no package tasks matched ${suffix}`); }
  const limit = suffix.startsWith('watch') ? runnable.length : Math.max(1, Math.min(availableParallelism(), runnable.length),);
  const failures: string[] = [];
  let next = 0;
  const worker = async (): Promise<void> => {
    while (next < runnable.length) {
      const target = runnable[next]!;
      next += 1;
      try {
        await runTaskByName(root, target.name, [],);
      } catch {
        failures.push(target.name);
      }
    }
  };
  await Promise.all(Array.from({ length: limit, }, worker,),);
  if (failures.length > 0) { throw new Error(`task-cli: package fanout failed: ${failures.join(', ')}`); }
}

//endregion

//region Entrypoint: argument parsing and dispatch

/**
 Parses CLI arguments and runs the named task.

 @throws When arguments are missing or the task fails.
 */
async function main(): Promise<void> {
  const [name, ...argv] = process.argv.slice(2);
  if (name === undefined)
    throw new Error('task-cli: usage: task-run <task> [-- options] [-- passthrough]');
  await runTaskByName(findRepoRoot(), name, argv,);
}

await main();
