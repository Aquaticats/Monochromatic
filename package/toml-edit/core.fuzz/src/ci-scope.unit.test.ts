/**
 Exercise the source artifact through the exact workflow launcher before dependencies exist.
 Source execution is intentional: CI deploys this entry directly, without a build.

 @module
 */

import { execFileSync, spawnSync, } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync, } from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { dirname, join, } from 'node:path';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

//region Fixture ownership and workflow extraction

/** Linux-hosted workflow uses system Git, not the developer's cli-git wrapper. */
const GIT = '/usr/bin/git';
/** Optional older Node executable tests the deployed artifact without downgrading the test harness. */
const SCOPE_NODE = process.env.SCOPE_TEST_NODE ?? process.execPath;
/** Preexisting output proves failure did not append a false success result. */
const OUTPUT_SENTINEL = 'previous=value\n';
/** Paths declared in both workflow event filters. Each is also tested through the command. */
const FILTERS = [
  'package/toml-edit/core/**',
  'package/toml-edit/core.fuzz/**',
  'package/test-fixture/toml-edit/**',
  'doc/decision/toml-edit-fuzzing.md',
  '.github/workflows/toml-edit-fuzz.yml',
] as const;
/** Workflow source, not a second hand-maintained launcher. */
const workflow = readFileSync(new URL('../../../../.github/workflows/toml-edit-fuzz.yml', import.meta.url,), 'utf8',);
/** Scope step starts before its shell, environment, and run fields. */
const scopeStart = workflow.indexOf('      id: scope\n',);
/** Scope step ends before another step can contribute a shell or environment binding. */
const scopeEnd = workflow.indexOf('\n    - name:', scopeStart,);
if ((scopeStart === (-1)) || (scopeEnd === (-1))) throw new Error('Scope step boundaries changed',);
/** Fixed workflow step whose formatting is deliberately part of this integration fixture. */
const scopeStep = workflow.slice(scopeStart, scopeEnd,);
/** Scope body must exist before any fixture executes extracted code. */
const runStart = scopeStep.indexOf('      run: |\n',);
if (runStart === (-1)) throw new Error('Scope run body is missing',);
/** Source lines following the scope run field, terminated explicitly for the final body line. */
const runLines = `${scopeStep.slice(runStart + '      run: |\n'.length,)}\n`.split('\n',);
/** The first dedented line ends this fixed workflow body. */
const bodyEnd = runLines.findIndex(line => !line.startsWith('        ',));
/** Exact Node body written to an extensionless file as the Actions runner does. */
const launcher = runLines.slice(0, bodyEnd,).map(line => line.slice(8,),).join('\n',);

/** Captured consumer-boundary result without throwing away failed subprocess output. */
type ScopeResult = { readonly status: number; readonly stdout: string; readonly stderr: string; readonly output: string; };
/** Disposable fixture owns its repository, home, launcher, copied artifact, and output. */
type Fixture = {
  readonly root: string;
  readonly base: string;
  readonly git: (args: readonly string[]) => string;
  readonly put: (path: string) => void;
  readonly commit: () => string;
  readonly run: (overrides?: Readonly<NodeJS.ProcessEnv>) => ScopeResult;
  readonly [Symbol.dispose]: () => void;
};

/** Create a dependency-free consumer checkout with no inherited Git configuration or hooks. */
function fixture(): Fixture {
  /** Every mutable resource stays under this disposable parent. */
  const temporary = mkdtempSync(join(scratchRoot(), 'toml-edit-scope-',),);
  /** Git checkout used by the production decision. */
  const root = join(temporary, 'repository',);
  mkdirSync(root,);
  /** Minimal child environment excludes credentials, Git overrides, and developer wrappers. */
  const environment = {
    PATH: '/usr/bin:/bin',
    HOME: temporary,
    TMPDIR: temporary,
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: join(temporary, 'missing-global-config',),
    GIT_AUTHOR_NAME: 'Scope fixture',
    GIT_AUTHOR_EMAIL: 'scope@example.invalid',
    GIT_COMMITTER_NAME: 'Scope fixture',
    GIT_COMMITTER_EMAIL: 'scope@example.invalid',
  };
  /** Fixture Git is isolated from the user's wrapper and repository state. */
  function git(args: readonly string[],): string {
    return execFileSync(GIT, [...args,], { cwd: root, env: environment, encoding: 'utf8', },).trim();
  }
  git(['init', '--quiet', '--initial-branch=main',],);
  git(['commit', '--quiet', '--allow-empty', '--message', 'initial fixture',],);
  /** Empty base permits explicit relevant and irrelevant changes per case. */
  const base = git(['rev-parse', 'HEAD',],);
  /** Copied source verifies bootstrap without repository node_modules or build artifacts. */
  const sourceDirectory = join(root, 'package/toml-edit/core.fuzz/src',);
  mkdirSync(sourceDirectory, { recursive: true, },);
  writeFileSync(join(root, 'package.json',), '{"type":"module"}\n',);
  for (const name of ['ci-scope.ts', 'ci-scope-git.ts', 'ci-scope-error.ts',]) {
    cpSync(new URL(name, import.meta.url,), join(sourceDirectory, name,),);
  }
  // Keep the deployed fixture program out of scenario commits and their changed-path sets.
  writeFileSync(join(root, '.git/info/exclude',), '/package/toml-edit/core.fuzz/src/ci-scope*.ts\n/package.json\n',);
  /** Runner temporary scripts live outside the repository's module scope. */
  const launcherPath = join(temporary, 'scope-launcher',);
  writeFileSync(launcherPath, launcher,);
  /** Output belongs to the fixture, never the calling agent's GITHUB_OUTPUT. */
  const output = join(temporary, 'github-output',);
  /** Create arbitrary Git names without shell interpolation. */
  function put(path: string,): void {
    mkdirSync(dirname(join(root, path,),), { recursive: true, },);
    writeFileSync(join(root, path,), `fixture ${path}\n`,);
  }
  /** Commit only disposable fixture changes. */
  function commit(): string {
    git(['add', '--all', '--', '.',],);
    git(['commit', '--quiet', '--allow-empty', '--message', 'scenario fixture',],);
    return git(['rev-parse', 'HEAD',],);
  }
  /** Exercise the exact Node launcher and inspect the output file as its consumer does. */
  function run(overrides: Readonly<NodeJS.ProcessEnv> = {},): ScopeResult {
    writeFileSync(output, OUTPUT_SENTINEL,);
    /** Captured failure remains inspectable rather than escaping the assertion. */
    const result = spawnSync(SCOPE_NODE, [launcherPath,], {
      cwd: root,
      env: {
        ...environment,
        GITHUB_EVENT_NAME: 'merge_group',
        GITHUB_OUTPUT: output,
        SCOPE_BASE_SHA: base,
        SCOPE_HEAD_SHA: git(['rev-parse', 'HEAD',],),
        ...overrides,
      },
      encoding: 'utf8',
      timeout: 10_000,
    },);
    if (result.error !== undefined) throw result.error;
    if (result.status === null) throw new Error(`Scope fixture was terminated by ${String(result.signal,)}`,);
    return { status: result.status, stdout: result.stdout, stderr: result.stderr, output: readFileSync(output, 'utf8',), };
  }
  return {
    root, base, git, put, commit, run,
    [Symbol.dispose](): void {
      rmSync(temporary, { recursive: true, force: true, },);
    },
  };
}

/** Assert both the exit status and the workflow-consumed output, not logs alone. */
function expectDecision({ result, run, }: { readonly result: ScopeResult; readonly run: boolean; },): void {
  expect(result.stderr,).toBe('',);
  expect(result.status,).toBe(0,);
  expect(result.output,).toBe(`${OUTPUT_SENTINEL}run=${String(run,)}\n`,);
}

/** Every rejection must preserve output instead of authorizing a successful skip. */
function expectFailure(result: ScopeResult,): void {
  expect(result.status,).not.toBe(0,);
  expect(result.stderr,).not.toBe('',);
  expect(result.output,).toBe(OUTPUT_SENTINEL,);
}

//endregion Fixture ownership and workflow extraction

await describe({
  name: 'pre-install workflow scope',
  concurrency: 1,
  children: [
    it({ name: 'keeps event path filters and the Node launcher contract explicit', fn: async () => {
      expect(scopeStart,).not.toBe(-1,);
      expect(launcher,).toContain("require('node:child_process').execFileSync",);
      expect(scopeStep,).toContain('      shell: node {0}\n',);
      expect(scopeStep,).toContain(`SCOPE_BASE_SHA: \${{ github.event.merge_group.base_sha }}`,);
      expect(scopeStep,).toContain(`SCOPE_HEAD_SHA: \${{ github.event.merge_group.head_sha }}`,);
      /** Each current paths block must contain exactly the tested scope set. */
      const blocks = workflow.split('    paths:\n',).slice(1,);
      expect(blocks,).toHaveLength(2,);
      for (const block of blocks) {
        /** Consecutive list entries end at the next event or workflow field. */
        const paths = block.split('\n',).filter(line => line.startsWith("    - '",),)
          .map(line => line.slice("    - '".length, -1,),);
        expect(paths,).toEqual([...FILTERS,],);
      }
    }, },),
    ...FILTERS.map(path => it({ name: `runs for ${path}`, fn: async () => {
      using state = fixture();
      state.put(path.endsWith('/**',) ? `${path.slice(0, -2,)}changed.ts` : path,);
      state.commit();
      expectDecision({ result: state.run(), run: true, },);
    }, },)),
    ...[
      'unrelated.txt',
      'package/toml-edit/core-editor/file.ts',
      'package/toml-edit/core.fuzzier/file.ts',
      'doc/decision/toml-edit-fuzzing.md.bak',
      '.github/workflows/toml-edit-fuzz.yml.bak',
      'unrelated\npackage/toml-edit/core/fake.ts',
    ].map(path => it({ name: `skips only a successful irrelevant comparison: ${JSON.stringify(path,)}`, fn: async () => {
      using state = fixture();
      state.put(path,);
      state.commit();
      expectDecision({ result: state.run(), run: false, },);
    }, },)),
    it({ name: 'accepts a successful empty diff', fn: async () => {
      using state = fixture();
      expectDecision({ result: state.run(), run: false, },);
    }, },),
    it({ name: 'preserves relevant tabs, quotes, non-ASCII, and embedded newlines', fn: async () => {
      using state = fixture();
      state.put('package/toml-edit/core/\t雪\n"\'$(name).ts',);
      state.commit();
      expectDecision({ result: state.run(), run: true, },);
    }, },),
    it({ name: 'runs for a modified relevant file mixed with irrelevant changes', fn: async () => {
      using state = fixture();
      /** Tracked path exercises modification rather than only additions. */
      const path = 'package/toml-edit/core/modified.ts';
      state.put(path,);
      /** Comparison starts after this file's introduction. */
      const base = state.commit();
      writeFileSync(join(state.root, path,), 'changed contents\n',);
      state.put('unrelated.txt',);
      state.commit();
      expectDecision({ result: state.run({ SCOPE_BASE_SHA: base, },), run: true, },);
    }, },),
    it({ name: 'runs when a relevant file is deleted', fn: async () => {
      using state = fixture();
      state.put('package/toml-edit/core/deleted.ts',);
      const base = state.commit();
      rmSync(join(state.root, 'package/toml-edit/core/deleted.ts',),);
      state.commit();
      expectDecision({ result: state.run({ SCOPE_BASE_SHA: base, },), run: true, },);
    }, },),
    ...[true, false,].map(into => it({ name: `runs when renaming ${into ? 'into' : 'out of'} scope`, fn: async () => {
      using state = fixture();
      const relevant = 'package/toml-edit/core/moved.ts';
      const other = 'outside/moved.ts';
      const from = into ? other : relevant;
      const to = into ? relevant : other;
      state.put(from,);
      const base = state.commit();
      mkdirSync(dirname(join(state.root, to,),), { recursive: true, },);
      renameSync(join(state.root, from,), join(state.root, to,),);
      state.commit();
      expectDecision({ result: state.run({ SCOPE_BASE_SHA: base, },), run: true, },);
    }, },)),
    it({ name: 'uses the event base even when origin/main already points at HEAD', fn: async () => {
      using state = fixture();
      state.put('package/toml-edit/core/relevant.ts',);
      const head = state.commit();
      state.git(['update-ref', 'refs/remotes/origin/main', head,],);
      expectDecision({ result: state.run(), run: true, },);
    }, },),
    ...['push', 'pull_request',].map(event => it({ name: `${event} runs without merge-group revisions`, fn: async () => {
      using state = fixture();
      expectDecision({ result: state.run({ GITHUB_EVENT_NAME: event, SCOPE_BASE_SHA: '', SCOPE_HEAD_SHA: '', PATH: '', },), run: true, },);
    }, },)),
    ...[
      { SCOPE_BASE_SHA: '', },
      { SCOPE_BASE_SHA: 'origin/main', },
      { SCOPE_BASE_SHA: '--help', },
      { SCOPE_BASE_SHA: 'g'.repeat(40,), },
      { SCOPE_BASE_SHA: '1'.repeat(40,), },
      { SCOPE_HEAD_SHA: '', },
      { SCOPE_HEAD_SHA: '2'.repeat(40,), },
      { GITHUB_EVENT_NAME: 'unexpected', },
      { GITHUB_EVENT_NAME: '', },
      { GITHUB_EVENT_NAME: undefined, },
      { GITHUB_OUTPUT: '', },
      { GITHUB_OUTPUT: undefined, },
      { PATH: '', },
    ].map(overrides => it({ name: `fails closed for ${Object.keys(overrides,).join(',',)}: ${JSON.stringify(overrides,)}`, fn: async () => {
      using state = fixture();
      expectFailure(state.run(overrides,),);
    }, },)),
    it({ name: 'fails when checkout does not match the event head', fn: async () => {
      using state = fixture();
      state.put('unrelated.txt',);
      state.commit();
      const result = state.run({ SCOPE_HEAD_SHA: state.base, },);
      expectFailure(result,);
      expect(result.stderr,).toContain('does not match merge-group head',);
    }, },),
    it({ name: 'fails when the event base is not an ancestor of its head', fn: async () => {
      using state = fixture();
      state.put('unrelated.txt',);
      const future = state.commit();
      state.git(['checkout', '--quiet', '--detach', state.base,],);
      const result = state.run({ SCOPE_BASE_SHA: future, },);
      expectFailure(result,);
      expect(result.stderr,).toContain('merge-base --is-ancestor',);
    }, },),
    it({ name: 'propagates a git diff failure after commit and ancestry validation', fn: async () => {
      using state = fixture();
      state.put('unrelated.txt',);
      state.commit();
      const tree = state.git(['rev-parse', 'HEAD^{tree}',],);
      rmSync(join(state.root, '.git/objects', tree.slice(0, 2,), tree.slice(2,),),);
      const result = state.run();
      expectFailure(result,);
      expect(result.stderr,).toContain('git diff --name-only --no-renames -z',);
    }, },),
    it({ name: 'the failure oracle rejects a deliberately restored fail-open Git adapter', fn: async () => {
      using state = fixture();
      state.put('unrelated.txt',);
      state.commit();
      /** Break only the disposable repository's tree, leaving commit validation successful. */
      const tree = state.git(['rev-parse', 'HEAD^{tree}',],);
      rmSync(join(state.root, '.git/objects', tree.slice(0, 2,), tree.slice(2,),),);
      /** Mutate only the deployed fixture copy; committed production sources remain untouched. */
      const adapter = join(state.root, 'package/toml-edit/core.fuzz/src/ci-scope-git.ts',);
      /** Source under test was committed before introducing this mutation control. */
      const source = readFileSync(adapter, 'utf8',);
      /** The adapter's sole failure throw is the guard being removed. */
      const guard = source.indexOf('    throw new ScopeError(',);
      if ((guard === (-1)) || (guard !== source.lastIndexOf('    throw new ScopeError(',)))
        throw new Error('Scope Git guard changed; update the mutation control',);
      writeFileSync(adapter, `${source.slice(0, guard,)}    console.error(error);\n    return '';\n  }\n}\n`,);
      /** This faulty adapter reproduces the original error-to-empty-list conversion. */
      const result = state.run();
      expect(result.status,).toBe(0,);
      expect(result.output,).toBe(`${OUTPUT_SENTINEL}run=false\n`,);
      expect(() => expectFailure(result,),).toThrow();
    }, },),
    it({ name: 'fails when the output cannot be appended', fn: async () => {
      using state = fixture();
      expectFailure(state.run({ GITHUB_OUTPUT: state.root, },),);
    }, },),
  ],
},);
