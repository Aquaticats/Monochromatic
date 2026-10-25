#!/usr/bin/env node

/**
 Mise-free task CLI for this monorepo's CI.

 Runs the same task graph CI used to run through `mise run`, but resolves every
 body from `task.ts` files and executes it with the current node, so CI no
 longer installs or invokes mise. Supports plain `run` strings, `run` arrays
 (sequenced, with `{ shell }` items), `depends`, `extends` onto root templates,
 `usage` options mapped to `usage_*` variables like mise, root `build`/`test`/
 `lint` umbrella tasks, root inline tasks, and the `//package/...:task` form.

 @example
 ```bash
 node package/dev-script/task-cli/src/run.ts //package/module/logger:build
 node package/dev-script/task-cli/src/run.ts //package/module/logger:test:unit -- --all
 node package/dev-script/task-cli/src/run.ts prepare:pnpm
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
const { availableParallelism, homedir, tmpdir } = await import('node:os')
const { dirname, join, relative, resolve } = await import('node:path')
const REPO_ROOT = process.env.TASK_REPO_ROOT
const runWorkspaceNode = (packagePath, entryName, entryArgs) => {
  const packageRoot = join(REPO_ROOT, packagePath)
  const built = join(packageRoot, 'dist', 'final', 'node', entryName + '.mjs')
  const source = join(packageRoot, 'src', entryName + '.ts')
  const entry = existsSync(built) ? built : existsSync(source) ? source : null
  if (entry === null) { throw new Error('task.ts: missing built or source entry: ' + packagePath + '/' + entryName) }
  execFileSync(process.execPath, [entry, ...entryArgs], { stdio: 'inherit' })
}
const runTaskCli = (name, args = []) => {
  execFileSync(process.execPath, [join(REPO_ROOT, 'package', 'dev-script', 'task-cli', 'src', 'run.ts'), name, ...args], { stdio: 'inherit', env: { ...process.env, TASK_CLI_CHILD: '1' } })
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
    [join(REPO_ROOT, 'package', 'dev-script', 'task-cli', 'src', 'run.ts'), '//' + packagePath + ':' + name],
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
 Resolves the package directory for a `//package/<category>/<name>[:task]` name.

 @param root - Monorepo root.
 @param packagePath - Path like `package/module/logger`.
 @returns Absolute package directory.
 @throws When the path is not a `package/` path with two segments below it.
 */
function packageDirectory(root: string, packagePath: string): string {
  const segments = packagePath.split('/');
  if (segments[0] !== 'package' || segments.length !== 1 + PACKAGE_SEGMENT_COUNT)
    throw new Error(`task-cli: not a package path: ${packagePath}`);
  const directory = join(root, packagePath);
  if (!existsSync(join(directory, 'package.json')) && !existsSync(join(directory, 'Cargo.toml')))
    throw new Error(`task-cli: no package.json or Cargo.toml at ${packagePath}`);
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
 Parses a mise-style `usage` spec into flag and arg descriptors.

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
    const nameMatch = /^(?<name>[^\s<]+)/u.exec(body);
    if (nameMatch === null) { continue; }
    const valueMatch = /<(?<value>[^>]+)>/u.exec(body);
    options.push({
      kind,
      name: nameMatch.groups!['name']!,
      valueName: valueMatch?.groups?.['value'],
    });
  }
  return options;
}

/**
 Splits CLI arguments at `--` into task options and passthrough arguments.

 @param argv - Arguments after the task name.
 @returns Options before `--` and raw arguments after it.
 */
function splitPassthrough(argv: readonly string[]): { options: readonly string[]; rest: readonly string[]; } {
  const index = argv.indexOf('--');
  if (index === -1) { return { options: argv, rest: [], }; }
  return { options: argv.slice(0, index), rest: argv.slice(index + 1), };
}

/**
 Builds the `usage_*` environment for a task from its options.

 @param options - Parsed usage descriptors.
 @param argv - Task option arguments.
 @param rest - Passthrough arguments after `--`.
 @returns Environment entries to set for the task body.
 @throws On unknown options, missing flag values, or misplaced positional args.
 */
function usageEnvironment(options: readonly UsageOption[], argv: readonly string[], rest: readonly string[],): Record<string, string> {
  const env: Record<string, string> = {};
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
    env['usage_args'] = [...positional, ...rest].join(' ');
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
 Runs a shell body step and waits for completion.

 @param step - Shell string or step with its shell.
 @param cwd - Working directory for the step.
 @param env - Environment for the step.
 @throws When the shell exits non-zero.
 */
async function runStepAsync(step: TaskStep, cwd: string, env: NodeJS.ProcessEnv,): Promise<void> {
  const [shell, rawBody] = typeof step === 'string' ? ['bash', step,] : [step.shell, step.run,];
  const [command, ...args] = shell.split(' ');
  const isNodeInline = command === 'node' && args.includes('--input-type=module-typescript');
  const script = isNodeInline ? `${BODY_PRELUDE}\n${rawBody}` : rawBody;
  const argv = isNodeInline ? [...args, script,] : [...args, '-c', script,];
  const proc = spawn(command!, argv, { cwd, env, stdio: 'inherit', },);
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
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    ...definition.env,
    TASK_REPO_ROOT: root,
    TASK_PACKAGE_PATH: packagePath,
    ...usageEnvironment(parseUsageSpec(definition.usage,), optionArgs, rest,),
  };
  for (const step of steps) { await runStepAsync(step, cwd, env,); }
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
