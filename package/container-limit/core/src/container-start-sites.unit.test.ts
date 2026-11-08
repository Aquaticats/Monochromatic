/**
 The ledger of every tracked file that names `podman` or `docker` as a program, and the check that each file that
 starts a container or builds an image carries the shared limits.

 A new file that starts a container fails the first test until it is listed here, and a listed file fails the second
 until it carries the limits (the shared helpers or, in a shell string or a workflow, the literal options).
 `doc/handover/container-limits-everywhere.md` is the inventory this ledger enforces.

 @module
 */

import { execFileSync, } from 'node:child_process';
import { readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  diskWeightArguments,
  LOWEST_DISK_WEIGHT,
} from './index.ts';

/**
 What a ledger entry's file does with the container tool.
 */
type Role = 'starts' | 'builds' | 'passes-limits' | 'reads';

/**
 Words that show a file that starts containers takes the shared weight: a helper from this package or a package that
 re-exports it, the IDE's bound, the workflows' option variable, or the literal option.
 */
const STARTS_MARKERS: readonly string[] = [
  'containerLimitArguments',
  'diskWeightArguments',
  'runLimitArguments',
  'containerBounds',
  'limitArguments',
  'containerRunArguments',
  'CONTAINER_EXTRA_OPTIONS',
  '--blkio-weight',
];

/**
 Words that show a file that builds images bounds the build: a helper, the build bound of a workflow, or a literal memory.
 `podman build` has no weight option, so no weight is required.
 */
const BUILDS_MARKERS: readonly string[] = [
  'imageBuildLimitArguments',
  'buildLimitArguments',
  'boundingFlagsOf',
  'BUILD_MEMORY',
  '--memory',
];

/**
 Words that show a file hands the limits its caller built to the container tool, or takes them from a sibling module.
 */
const PASSES_MARKERS: readonly string[] = [
  'limits',
  'Limits',
  'container-arguments',
];

/**
 Every file the scan can find, with its role.
 The comment above each group says why the role holds.
 */
const ledger: readonly { readonly path: string; readonly role: Role; }[] = [
  // Workflows: the first container start of each job is its probe, and the options come from one job variable.
  ...[
    '.github/workflows/ide-release.yml',
    '.github/workflows/music-player-release.yml',
    '.github/workflows/pnpr-publish.yml',
  ].map((path,) => ({ path, role: 'starts' as const, }),),
  // Task files and scripts that start containers with the limits spelled in the file or taken from a helper.
  ...[
    'mise.toml',
    'package/browserslist/core-rust/bin/regenerate-conformance.ts',
    'package/config/pnpr/mise.toml',
    'package/config/pnpr/task.ts',
    'package/container-limit/core/src/index.ts',
    'package/forbidden-strings/cli/bin/evidence-guard-comparison.ts',
    'package/forbidden-strings/cli/bin/observe-guards-container.ts',
    'package/forbidden-strings/cli/bin/snapshot-constants.ts',
    'package/git-policy/cli/bin/build-git-test-image.ts',
    'package/git-policy/cli/bin/e2e-native-container.ts',
    'package/git-policy/cli/bin/native-planted-controls-only.ts',
    'package/git-policy/cli/bin/test-native-container.ts',
    'package/ide/app/bin/check-nextest-config.ts',
    'package/ide/app/bin/container-bounds.ts',
    'package/ide/app/bin/inspect-annotation-guards.ts',
    'package/ide/app/bin/inspect-find-guards.ts',
    'package/ide/app/bin/inspect-idle-cost.ts',
    'package/ide/app/bin/inspect-ignored-guards.ts',
    'package/ide/app/bin/inspect-language-guards.ts',
    'package/ide/app/bin/inspect-language-lifecycle-guards.ts',
    'package/ide/app/bin/inspect-language-navigation-guards.ts',
    'package/ide/app/bin/inspect-language-reap-rate.ts',
    'package/ide/app/bin/inspect-log-stall.ts',
    'package/ide/app/bin/inspect-refresh-latency.ts',
    'package/ide/app/bin/inspect-search-guards.ts',
    'package/ide/app/bin/inspect-sidebar-guards.ts',
    'package/ide/app/bin/inspect-source-guards.ts',
    'package/ide/app/bin/inspect-theme-guards.ts',
    'package/ide/app/bin/inspect-watch-guards.ts',
    'package/ide/app/bin/inspect-watch-limit.ts',
    'package/ide/app/bin/inspect-watch-scope.ts',
    'package/ide/app/bin/stress-native.ts',
    'package/ide/app/mise.toml',
    'package/ide/app/task.ts',
    'package/linter/monochromatic-lint.fuzz/bin/fuzz-constants.ts',
    'package/linter/monochromatic-lint/bin/clippy-container.ts',
    'package/linter/monochromatic-lint/bin/test-container.ts',
    'package/linter/monochromatic-lint/mise.toml',
    'package/monorepo-task/cli/src/task-script-prelude.ts',
    'package/music-player/desktop-app/mise.toml',
    'package/music-player/desktop-app/task.ts',
    'package/mutation-test/cli-cargo-mutants/src/container-limits.ts',
    'package/mutation-test/cli/src/host/podman.ts',
    'package/nested-wayland-session/cli/mise.toml',
    'package/repository-move/cli/src/guided-cutover-stages.data.ts',
    'package/search-shadow/cli/mise.toml',
    'package/terminal/app/mise.toml',
    'package/terminal/app/task.ts',
    'package/test-harness/core-matrix/src/container.ts',
    'package/test-harness/core/mise.toml',
    'package/upstream-fork/conf/mise.toml',
    'package/upstream-fork/deepmerge-ts.fuzz/mise.toml',
    'package/vm-builder/cli/mise.toml',
    'package/vm-builder/cli/src/container-arguments.ts',
  ].map((path,) => ({ path, role: 'starts' as const, }),),
  // Files whose only container command is `podman build`, which has no weight option.
  ...[
    'package/forbidden-strings/core/mise.toml',
    'package/git-policy/cli.fuzz/bin/container.ts',
    'package/git-policy/cli/bin/native-test-base.ts',
    'package/monorepo-task/core-exec/src/playwright-image-build.ts',
    'package/mutation-test/cli/src/host/image.ts',
    'package/test-harness/core-matrix/src/container-image-rebuild-on-changed-inputs.ts',
  ].map((path,) => ({ path, role: 'builds' as const, }),),
  // Files that run a container with limits their caller or a sibling module built.
  ...[
    'package/forbidden-strings/cli.fuzz/bin/smoke-container.ts',
    'package/forbidden-strings/cli/bin/guard-container.ts',
    'package/forbidden-strings/cli/bin/mutate-container.ts',
    'package/forbidden-strings/cli/bin/snapshot-image.ts',
    'package/forbidden-strings/cli/bin/test-container.ts',
    'package/git-policy/cli.fuzz/bin/fuzz-build.ts',
    'package/git-policy/cli.fuzz/bin/fuzz-run.ts',
    'package/git-policy/cli/bin/container-limits.ts',
    'package/git-policy/cli/bin/native-planted-controls.ts',
    'package/linter/monochromatic-lint.fuzz/bin/fuzz-build.ts',
    'package/linter/monochromatic-lint.fuzz/bin/fuzz-run.ts',
    'package/vm-builder/cli/src/build-and-import.ts',
  ].map((path,) => ({ path, role: 'passes-limits' as const, }),),
  // Files that name the tool without starting a container: allow-lists, version, inspect, tag, login and push calls, image
  // names, stand-ins for tests, and the generic runner whose callers build the arguments.
  ...[
    'package/claude-code-plugin/bash-output-filter/bundle/node/index.mjs',
    'package/claude-code-plugin/source/src/handler/bash-output-filter/validation.ts',
    'package/forbidden-strings/cli/bin/scanner-run.ts',
    'package/forbidden-strings/cli/data/betterleaks-default-config.toml',
    'package/git-policy/cli/bin/native-mutation-parallel.ts',
    'package/git-policy/cli/bin/native-verification-process.ts',
    'package/monorepo-task/core-exec/src/depends.ts',
    'package/monorepo-task/core-exec/src/github-actions-step-fixture-fake-tools.ts',
    'package/monorepo-task/core-exec/src/github-actions-step-fixture.ts',
    'package/monorepo-task/core-exec/src/github-actions-step-support.ts',
    'package/monorepo-task/core-exec/src/playwright-image-logic.ts',
    'package/monorepo-task/core-exec/src/playwright-image-push.ts',
    'package/monorepo-task/core-exec/src/playwright-image.ts',
    'package/mutation-test/cli-cargo-mutants/src/constants.ts',
    'package/mutation-test/cli-cargo-mutants/src/fake-podman-fixture.ts',
    'package/mutation-test/cli-cargo-mutants/src/fake-programs.ts',
    'package/mutation-test/cli-cargo-mutants/src/podman.ts',
    'package/mutation-test/cli-cargo-mutants/src/unit-images.ts',
    'package/test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts',
    'package/test-harness/core-matrix/src/bin/locked-rust-toolchain.ts',
    'package/test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts',
    'package/test-harness/core-matrix/src/cachyos-image-cli.ts',
    'package/test-harness/core-matrix/src/cachyos-image.ts',
    'package/test-harness/core-matrix/src/locked-tool-build-arguments.ts',
    'package/vm-builder/cli/src/sign-and-push.ts',
  ].map((path,) => ({ path, role: 'reads' as const, }),),
];

/**
 Absolute path of the repository root, four folders above this file.
 */
const repositoryRoot = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Folders the scan skips: documents and paused packages.
 */
const SKIPPED_FOLDERS: readonly string[] = [
  'doc/',
  'package-paused/',
];

/**
 Path fragments of tests, goldens and fixtures the scan skips.
 */
const SKIPPED_FRAGMENTS: readonly string[] = [
  '/golden/',
  '/tests/',
  '.test.ts',
];

/**
 Extensions of files that can hold a command.
 */
const SCANNED_EXTENSIONS: readonly string[] = [
  '.ts',
  '.mjs',
  '.toml',
  '.yml',
  '.yaml',
];

/**
 Quote characters that can wrap a program name.
 */
const QUOTES: readonly string[] = [
  '\'',
  '"',
  '`',
];

/**
 Container programs the scan looks for.
 */
const PROGRAMS: readonly string[] = [
  'podman',
  'docker',
];

/**
 Verbs after a program name that start a container or build an image.
 */
const VERBS: readonly string[] = [
  'run',
  'create',
  'build',
];

/**
 Whether a path is one the scan skips.

 @param path - repository-relative path

 @returns true for documents, paused packages, tests, goldens, Containerfiles and files whose extension holds no command

 @example
 ```ts
 const skip = isSkipped('doc/x.md');
 ```
 */
function isSkipped(path: string,): boolean {
  if (SKIPPED_FOLDERS.some((folder,) => path.startsWith(folder,),))
    return true;
  if (SKIPPED_FRAGMENTS.some((fragment,) => path.includes(fragment,),))
    return true;
  if ((path.split('/',).at(-1,) ?? '').startsWith('Containerfile',))
    return true;
  return !SCANNED_EXTENSIONS.some((extension,) => path.endsWith(extension,),);
}

/**
 Whether a text names `podman` or `docker` as a program: quoted as an argument, as a command line, or as a helper call.

 @param text - file text

 @returns true when a program is named

 @example
 ```ts
 const names = namesProgram("spawn('podman', ['run'])");
 ```
 */
function namesProgram(text: string,): boolean {
  return PROGRAMS.some((program,) => {
    if (QUOTES.some((quote,) => text.includes(`${quote}${program}${quote}`,),))
      return true;
    if (text.includes(`${program}([`,))
      return true;
    return VERBS.some((verb,) => text.includes(`${program} ${verb}`,),);
  },);
}

/**
 Text of a tracked file.

 @param path - repository-relative path

 @returns the file's text

 @example
 ```ts
 const text = textOf('mise.toml');
 ```
 */
function textOf(path: string,): string {
  return readFileSync(
    join(
      repositoryRoot,
      path,
    ),
    'utf8',
  );
}

/**
 Tracked files that name a container program.

 @returns repository-relative paths

 @example
 ```ts
 const files = filesNamingAContainerProgram();
 ```
 */
function filesNamingAContainerProgram(): readonly string[] {
  return execFileSync(
    'git',
    ['ls-files', '-z',],
    {
      cwd: repositoryRoot,
      encoding: 'utf8',
      maxBuffer: 1 << 28,
    },
  )
    .split('\0',)
    .filter((path,) => path !== '',)
    .filter((path,) => !isSkipped(path,),)
    .filter((path,) => namesProgram(textOf(path,),),);
}

/**
 The words each role requires one of; `reads` requires none.
 */
const markersByRole: Readonly<Record<Role, readonly string[]>> = {
  'starts': STARTS_MARKERS,
  'builds': BUILDS_MARKERS,
  'passes-limits': PASSES_MARKERS,
  'reads': [],
};

/**
 The words a role requires one of.

 @param role - the entry's role

 @returns the words, or an empty list when the role requires none

 @example
 ```ts
 const words = markersOf('starts');
 ```
 */
function markersOf(role: Role,): readonly string[] {
  return markersByRole[role];
}

await describe({
  name: 'the container start sites ledger',
  children: [
    it({
      name: 'lists every tracked file that names podman or docker as a program',
      fn: async () => {
        /**
         Listed paths.
         */
        const listed = new Set(ledger.map((entry,) => entry.path,),);
        expect(filesNamingAContainerProgram().filter((path,) => !listed.has(path,),),).toEqual([],);
      },
    },),
    it({
      name: 'lists only files that exist',
      fn: async () => {
        for (const { path, } of ledger)
          expect(() => textOf(path,),).not.toThrow();
      },
    },),
    it({
      name: 'lists no file twice',
      fn: async () => {
        expect(new Set(ledger.map((entry,) => entry.path,),).size,).toBe(ledger.length,);
      },
    },),
    it({
      name: 'shows the limits in every file that starts a container, builds an image or hands limits on',
      fn: async () => {
        /**
         Entries whose role requires a marker and whose file lacks it.
         */
        const missing = ledger.filter(({ path, role, },) => {
          /**
           Words of the role.
           */
          const markers = markersOf(role,);
          /**
           Text of the entry's file.
           */
          const text = textOf(path,);
          if (markers.length === 0)
            return false;
          return !markers.some((marker,) => text.includes(marker,),);
        },).map((entry,) => entry.path,);
        expect(missing,).toEqual([],);
      },
    },),
    it({
      name: 'spells the weight of the docker run in pnpr-publish.yml as the shared weight',
      fn: async () => {
        /**
         The workflow text; docker has `--blkio-weight` and no `--cgroup-conf`.
         */
        const workflow = textOf('.github/workflows/pnpr-publish.yml',);
        expect(workflow,).toContain(`--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,);
        expect(diskWeightArguments(),).toContain(`--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,);
      },
    },),
    it({
      name: 'spells the options of the workflow variable as the shared weight options',
      fn: async () => {
        for (const workflow of [
          '.github/workflows/ide-release.yml',
          '.github/workflows/music-player-release.yml',
        ]) {
          /**
           Value lines of the option variable.
           */
          const values = textOf(workflow,)
            .split('\n',)
            .filter((line,) => line.trimStart().startsWith('CONTAINER_EXTRA_OPTIONS:',),);
          expect(values.length,).toBeGreaterThan(0,);
          for (const line of values) {
            for (const option of diskWeightArguments())
              expect(line,).toContain(option,);
          }
        }
      },
    },),
  ],
},);
