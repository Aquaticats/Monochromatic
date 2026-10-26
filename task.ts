/**
 Root task definitions for `@monochromatic-dev/dev-script-task-cli`.

 These bodies are the mise-free equivalents of the root `mise.toml` tasks CI
 used to run through `mise run`. Templates hold the shared package-task bodies
 (`build`, `test:unit`, `lint`, `build:js:*`) that package `task.ts` files
 `extends`; `tasks` holds the root umbrella and inline tasks. Everything runs
 through the current node, so CI never installs mise.
 */

import { execFileSync, spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  statSync,
} from 'node:fs';
import {
  availableParallelism,
  homedir,
  tmpdir,
} from 'node:os';
import {
  dirname,
  join,
  relative,
  resolve,
} from 'node:path';
import {
  loadPackageTasks,
  runTaskCli,
} from './package/dev-script/task-cli/src/load.ts';
import type {
  RootTaskFile,
  TaskDefinition,
} from './package/dev-script/task-cli/src/types.ts';



//region Helpers: dispatcher, test runner and usage parsing

/**
 Monorepo root from this file's location, so task bodies need no `cwd` lookup.
 */
const REPO_ROOT = import.meta.dirname;

/**
 Runs a workspace package's node entry: built `dist/final/node/<entry>.mjs` when
 present, else `src/<entry>.ts` through the current node.

 @param packagePath - Path like `package/dev-script/task-util`.
 @param entryName - Entry base name like `oxlint-wrapper`.
 @param entryArgs - Arguments passed to the entry.
 @throws When neither built nor source entry exists.
 */
function runWorkspaceNode(packagePath: string, entryName: string, entryArgs: readonly string[],): void {
  const packageRoot = join(REPO_ROOT, packagePath);
  const built = join(packageRoot, 'dist', 'final', 'node', `${entryName}.mjs`);
  const source = join(packageRoot, 'src', `${entryName}.ts`);
  const entry = existsSync(built) ? built : existsSync(source) ? source : null;
  if (entry === null)
    throw new Error(`task.ts: missing built or source entry: ${packagePath}/${entryName}`);
  execFileSync(process.execPath, [entry, ...entryArgs,], { stdio: 'inherit', },);
}

/**
 Newest modification time over a file or the whole tree under a directory.

 @param path - File or directory to measure.
 @returns Milliseconds since the epoch of the newest entry.
 */
function newestMtime(path: string): number {
  const stats = statSync(path);
  if (!stats.isDirectory()) { return stats.mtimeMs; }
  return readdirSync(path, { withFileTypes: true, }).reduce((newest, entry) => {
    const entryPath = join(path, entry.name);
    if (entry.isDirectory()) { return Math.max(newest, newestMtime(entryPath),); }
    if (entry.isFile()) { return Math.max(newest, statSync(entryPath).mtimeMs,); }
    return newest;
  }, stats.mtimeMs,);
}

/**
 Builds the root Oxlint config bundle when missing or stale.

 Mirrors `ensureOxlintConfig` from the root mise vars so lint tasks see the
 same generated config the old `mise run lint:oxlint` bodies built.
 */
function ensureOxlintConfig(): void {
  const oxlintConfigPackageDir = join(REPO_ROOT, 'package', 'config', 'oxlint');
  const oxlintPluginPackagesDir = join(REPO_ROOT, 'package', 'oxlint-plugin');
  const oxlintPluginPackageDirs = existsSync(oxlintPluginPackagesDir)
    ? readdirSync(oxlintPluginPackagesDir, { withFileTypes: true, })
      .filter((entry) => entry.isDirectory(),)
      .map((entry) => join(oxlintPluginPackagesDir, entry.name),)
      .filter((packageDir) => existsSync(join(packageDir, 'package.json')),)
    : [];
  const oxlintPackageDirs = [oxlintConfigPackageDir, ...oxlintPluginPackageDirs,];
  const oxlintConfigDistDir = join(oxlintConfigPackageDir, 'dist', 'final', 'node');
  const oxlintConfigSrcDir = join(oxlintConfigPackageDir, 'src');
  const oxlintSidecarOutputPaths = readdirSync(oxlintConfigSrcDir, { withFileTypes: true, })
    .filter((entry) => entry.isFile() && entry.name.startsWith('plugin-') && entry.name.endsWith('.ts'),)
    .map((entry) => join(oxlintConfigDistDir, `${entry.name.slice(0, -3)}.mjs`),);
  const outputPaths = [join(oxlintConfigDistDir, 'index.mjs'), ...oxlintSidecarOutputPaths,];
  const sourcePaths = [
    ...oxlintPackageDirs.flatMap((packageDir) => [
      join(packageDir, 'src'),
      join(packageDir, 'package.json'),
      join(packageDir, 'tsconfig.json'),
      join(packageDir, 'rolldown.node.config.ts'),
    ],),
    join(REPO_ROOT, 'pnpm-lock.yaml'),
    join(REPO_ROOT, 'pnpm-workspace.yaml'),
  ].filter((path) => existsSync(path),);
  const missingOutput = outputPaths.some((outputPath) => !existsSync(outputPath),);
  const stale = !missingOutput
    && Math.max(...sourcePaths.map(newestMtime),) > Math.min(...outputPaths.map((outputPath) => newestMtime(outputPath),),);
  if (missingOutput || stale) { runTaskCli('//package/config/oxlint:build:js:node'); }
}

/**
 Runs test files as node subprocesses bounded to availableParallelism, each
 from its nearest package root, with a per-run `MONOCHROMATIC_LOG_DIRECTORY`
 scratch under `${HOME}/temp/agent` (issue 576).

 @param files - Test file paths to run.
 @param nodeArgs - Extra node arguments before each file.
 @throws When any test file exits non-zero.
 */
async function runTestFiles(files: readonly string[], nodeArgs: readonly string[],): Promise<void> {
  if (files.length === 0) { return; }
  const testFileCwd = (file: string): string => {
    let current = dirname(resolve(file));
    while (true) {
      if (existsSync(join(current, 'package.json')) && existsSync(join(current, 'task.ts'))) { return current; }
      const parent = dirname(current);
      if (parent === current) { return REPO_ROOT; }
      current = parent;
    }
  };
  const env = { ...process.env, };
  if (env['MONOCHROMATIC_LOG_DIRECTORY'] === undefined) {
    const agentScratch = join(homedir(), 'temp', 'agent');
    execFileSync('mkdir', ['-p', '-m', '700', agentScratch,], { stdio: 'inherit', },);
    const stamp = new Date().toISOString().replaceAll(':', '-',);
    const runLogDirectory = join(agentScratch, 'module-test-logs', `${stamp}-${process.pid}`,);
    execFileSync('mkdir', ['-p', '-m', '700', runLogDirectory,], { stdio: 'inherit', },);
    env['MONOCHROMATIC_LOG_DIRECTORY'] = runLogDirectory;
  }
  const limit = Math.max(1, Math.min(availableParallelism(), files.length),);
  const failures: string[] = [];
  let next = 0;
  const worker = async (): Promise<void> => {
    while (next < files.length) {
      const file = files[next]!;
      next += 1;
      const proc = spawn(process.execPath, [...nodeArgs, file,], { cwd: testFileCwd(file), stdio: 'inherit', env, },);
      const [code] = await once(proc, 'exit',);
      if (code !== 0) { failures.push(file); }
    }
  };
  await Promise.all(Array.from({ length: limit, }, worker,),);
  if (failures.length > 0) { throw new Error(`task.ts: test files failed: ${failures.join(', ')}`); }
}

/**
 Shell-word parser for `usage_args`, matching the old mise var: bare tokens
 split on spaces, quoted spans kept whole, backslash escapes honoured.

 @param raw - Raw `usage_args` string.
 @returns Recovered argument vector.
 */
function parseUsageArgs(raw: string): string[] {
  const argv: string[] = [];
  let current: string | null = null;
  let index = 0;
  while (index < raw.length) {
    const char = raw[index]!;
    if (current === null) {
      if (char === ' ') { index += 1; continue; }
      current = '';
    }
    if (char === '\\') { current += raw[index + 1] ?? ''; index += 2; continue; }
    if (char === "'" || char === '"') {
      index += 1;
      while (index < raw.length && raw[index] !== char) { current += raw[index]!; index += 1; }
      index += 1;
      continue;
    }
    if (char === ' ') { argv.push(current); current = null; index += 1; continue; }
    current += char;
    index += 1;
  }
  if (current !== null) { argv.push(current); }
  return argv;
}

/**
 Runs every direct child task of a package task like `build:js` in parallel.

 @param taskName - Full task name like `//package/module/logger:build:js`.
 @throws When a child task is missing or any child fails.
 */
async function fanoutChildren(taskName: string): Promise<void> {
  const separator = taskName.indexOf(':', 2);
  const packagePath = taskName.slice(2, separator);
  const suffix = taskName.slice(separator + 1);
  const tasks = await loadPackageTasks(join(REPO_ROOT, packagePath, 'task.ts'));
  const prefix = `${suffix}:`;
  const depth = prefix.split(':').length;
  const children = Object.keys(tasks)
    .filter((name) => name.startsWith(prefix) && name.split(':').length === depth)
    .toSorted();
  if (children.length === 0) { throw new Error(`task.ts: no child tasks for ${taskName}`); }
  const failures: string[] = [];
  const processes = children.map((name) => spawn(
    process.execPath,
    [join(REPO_ROOT, 'package', 'dev-script', 'task-cli', 'src', 'run.ts'), `//${packagePath}:${name}`,],
    { stdio: 'inherit', env: { ...process.env, TASK_CLI_CHILD: '1', }, },
  ),);
  const exits = await Promise.all(processes.map((proc) => once(proc, 'exit',),));
  for (const [index, [code]] of exits.entries()) {
    if (code !== 0) { failures.push(children[index]!); }
  }
  if (failures.length > 0) { throw new Error(`task.ts: fanout child tasks failed: ${failures.join(', ')}`); }
}

//endregion

//region Templates: shared package task bodies package task files extend

const templates: Record<string, TaskDefinition> = {
  'build': {
    description: 'Build this package, fanout to its build:* tasks',
    run: { shell: 'node --input-type=module-typescript -e', run: 'await fanoutChildren(`//${process.env.TASK_PACKAGE_PATH}:build`)', },
  },
  'build:js': {
    description: 'Build this package JavaScript entries, fanout to build:js:* tasks',
    run: { shell: 'node --input-type=module-typescript -e', run: 'await fanoutChildren(`//${process.env.TASK_PACKAGE_PATH}:build:js`)', },
  },
  'build:js:browser': {
    run: 'rolldown --configLoader native --config rolldown.browser.config.ts',
  },
  'build:js:client': {
    run: 'rolldown --configLoader native --config rolldown.client.config.ts',
  },
  'build:js:node': {
    run: 'rolldown --configLoader native --config rolldown.node.config.ts',
  },
  'test': {
    description: 'Run this package test suites, fanout to test:* tasks',
    run: { shell: 'node --input-type=module-typescript -e', run: 'await fanoutChildren(`//${process.env.TASK_PACKAGE_PATH}:test`)', },
  },
  'test:unit': {
    description: 'Run active-tree *.unit.test.ts in parallel via node (excludes *.expensive.* unless --all); pass file paths to filter',
    usage: 'flag "--all" help="Include active-tree *.expensive.* files; cannot be combined with file paths"\narg "[args]" var=#true help="Test file paths to run (when provided, only those files run); cannot be combined with --all"',
    run: [
      {
        shell: 'node --input-type=module-typescript -e',
        run: `
const args = parseUsageArgs(process.env.usage_args ?? '')
const all = (process.env.usage_all ?? '').trim() === 'true'
if (all && args.length > 0) { throw new Error('--all cannot be combined with file paths') }
const isInactiveTest = (file) => file.includes('/deprecated/') || file.startsWith('package-deprecated/') || file.includes('/package-deprecated/') || file.startsWith('package-paused/') || file.includes('/package-paused/')
let files
if (args.length === 0) {
  const { glob } = await import('node:fs/promises')
  files = []
  for await (const file of glob('**/*.unit.test.ts', { exclude: (name) => name.includes('node_modules') })) {
    if (!isInactiveTest(file) && (all || !file.includes('.expensive.'))) { files.push(file) }
  }
} else {
  const { existsSync } = await import('node:fs')
  const { join } = await import('node:path')
  const base = process.env.TASK_PACKAGE_PATH === undefined ? process.cwd() : join(process.env.TASK_REPO_ROOT, process.env.TASK_PACKAGE_PATH)
  files = args.map((file) => existsSync(file) ? file : join(base, file))
}
await runTestFiles(files, [])
`,
      },
    ],
  },
  'lint': {
    description: 'Lint this package, fanout to lint:* tasks',
    run: { shell: 'node --input-type=module-typescript -e', run: 'await fanoutChildren(`//${process.env.TASK_PACKAGE_PATH}:lint`)', },
  },
  'lint:oxlint': {
    description: 'Lint with Oxlint',
    run: [
      { shell: 'node --input-type=module-typescript -e', run: 'ensureOxlintConfig()', },
      { shell: 'node --input-type=module-typescript -e', run: 'runWorkspaceNode(\'package/dev-script/task-util\', \'oxlint-wrapper\', [])', },
    ],
  },
  'format:oxlint': {
    description: 'Auto-fix this package with Oxlint using its package-local tsconfig',
    run: [
      { shell: 'node --input-type=module-typescript -e', run: 'ensureOxlintConfig()', },
      { shell: 'node --input-type=module-typescript -e', run: 'runWorkspaceNode(\'package/dev-script/task-util\', \'oxlint-wrapper\', [\'--fix\'])', },
    ],
  },
  'lint:types': {
    description: 'TypeScript',
    run: { shell: 'node --input-type=module-typescript -e', run: 'runWorkspaceNode(\'package/dev-script/task-util\', \'tsc-filter\', [\'--build\'])', },
  },
  'watch:build': {
    description: 'Watch-build this package, fanout to watch:build:* tasks',
    run: { shell: 'node --input-type=module-typescript -e', run: 'await fanoutChildren(`//${process.env.TASK_PACKAGE_PATH}:watch:build`)', },
  },
  'watch:build:js': {
    description: 'Watch-build this package JavaScript entries, fanout to watch:build:js:* tasks',
    run: { shell: 'node --input-type=module-typescript -e', run: 'await fanoutChildren(`//${process.env.TASK_PACKAGE_PATH}:watch:build:js`)', },
  },
  'watch:build:js:browser': {
    run: 'rolldown --watch --configLoader native --config rolldown.browser.config.ts',
  },
  'watch:build:js:client': {
    run: 'rolldown --watch --configLoader native --config rolldown.client.config.ts',
  },
  'watch:build:js:node': {
    run: 'rolldown --watch --configLoader native --config rolldown.node.config.ts',
  },
  'watch:lint:types': {
    description: 'Type check in watch mode',
    run: 'tsc --watch',
  },
};

//endregion

//region Root tasks: toolchain, prepare, lint, npm and changeset bodies from mise.toml

const tasks: Record<string, TaskDefinition> = {
  'toolchain:sync': {
    description: 'Converge every tool to the versions toolchain.lock.json records (lock-constrained installs, the pnpm-install model)',
    run: 'node package/dev-script/file-enforcer/src/toolchain/sync.ts',
  },
  'toolchain:status': {
    description: 'Print observed versus locked tool versions and exit non-zero on drift',
    run: 'node package/dev-script/file-enforcer/src/toolchain/status.ts',
  },
  'bootstrap': {
    description: 'Alias of toolchain:sync: converge every tool to the versions toolchain.lock.json records (runs on directory entry)',
    run: { shell: 'node --input-type=module-typescript -e', run: "runTaskCli('toolchain:sync')", },
  },
  'prepare:android': {
    description: 'Alias of toolchain:sync: install the Android SDK components and rustup Android targets package/music-player/android-app needs (EXPENSIVE on a fresh machine, ~2.9 GB; idempotent once provisioned)',
    run: { shell: 'node --input-type=module-typescript -e', run: "runTaskCli('toolchain:sync')", },
  },
  'prepare:pnpm': {
    description: 'Install dependencies and sync managed files',
    run: [
      'pnpm install',
      'pnpm approve-builds sharp',
      'node file-enforcer.config.ts',
    ],
  },
  'prepare:pnpm:install': {
    description: 'Install dependencies',
    run: 'pnpm install',
  },
  'lint:monochromatic-lint': {
    description: 'Lint Rust, Markdown and MDX with monochromatic-lint, optionally restricted to specific paths',
    run: 'monochromatic-lint',
  },
  'npm:module:pack': {
    description: 'Pack the publishable workspace modules',
    run: 'pnpm pack --recursive',
  },
  'npm:module:publish': {
    description: 'Publish the packed workspace modules',
    run: 'pnpm publish --recursive --no-git-checks',
  },
  'changeset:add': {
    description: 'Add a changeset',
    run: 'changeset add',
  },
  'changeset:status': {
    description: 'Show changeset status',
    run: 'changeset status',
  },
  'changeset:version': {
    description: 'Version packages from changesets',
    run: 'changeset version',
  },
  'changeset:build': {
    description: 'Build every package the changeset publish plan will publish',
    run: {
      shell: 'node --input-type=module-typescript -e',
      run: `
const planPath = join(mkdtempSync(join(tmpdir(), 'changeset-plan-')), 'plan.json')
execFileSync('changeset', ['publish-plan', '--output', planPath], { stdio: 'inherit' })
const { plan } = JSON.parse(readFileSync(planPath, 'utf8'))
const names = plan.flat().filter((entry) => entry.kind === 'publish').map((entry) => entry.name)
if (names.length === 0) {
  console.log('changeset:build: the publish plan is empty; nothing to build')
} else {
  const workspace = JSON.parse(execFileSync('pnpm', ['list', '--recursive', '--depth', '-1', '--json'], { encoding: 'utf8' }))
  for (const name of names) {
    const entry = workspace.find((candidate) => candidate.name === name)
    if (entry === undefined)
      throw new Error('changeset:build: workspace package ' + name + ' is in the publish plan but not in the workspace')
    const task = '//' + relative(process.env.TASK_REPO_ROOT, entry.path) + ':build'
    console.log('changeset:build: ' + name + ' -> task ' + task)
    runTaskCli(task)
  }
}
`,
    },
  },
};

//endregion

export default { templates, tasks, } satisfies RootTaskFile;
