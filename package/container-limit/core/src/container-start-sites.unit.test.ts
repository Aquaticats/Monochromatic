/**
 The ledger of every tracked file that names `podman` or `docker` as a program, and the check that each file that
 starts a container or builds an image carries the shared limits.

 A new file that starts a container fails the first test until it is listed here, and a listed file fails the second
 until it carries the limits (the shared helpers or, in a shell string or a workflow, the literal options).
 The third check reads each start the file writes, because a file that imports a helper and never calls it in a start's
 own argument list used to pass on the helper's name alone (`package/terminal/app/mise.toml` before `bbadfac560`).
 `doc/handover/container-limits-everywhere.md` is the inventory this ledger enforces.

 @module
 */

import { execFileSync, } from 'node:child_process';
import { existsSync, readFileSync, } from 'node:fs';
import {
  dirname,
  join,
  normalize,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { argumentTextWithBoundInitializers, } from './argument-text-with-definitions.ts';
import type { ContainerStartInvocation, } from './container-start-vocabulary.ts';
import {
  containerStartInvocations,
} from './container-starts-in-source-text.ts';
import { definedTextByName, } from './defined-text-in-source-text.ts';
import { importedModuleByName, } from './imported-module-by-name.ts';
import {
  stripSourceComments,
  type SourceLanguage,
} from './source-text-without-comments.ts';
import {
  DISK_WEIGHT_OFF,
  DISK_WEIGHT_SWITCH,
  diskWeightArguments,
  LOWEST_DISK_WEIGHT,
} from './index.ts';

/**
 What a ledger entry's file does with the container tool.
 */
type Role = 'starts' | 'hosted-runner' | 'builds' | 'passes-limits' | 'reads';

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
 Words that show a workflow that starts containers on a hosted runner, which cannot set the disk weight: the switch that
 turns the weight off for every task of the job, which the test below ties to a measured refusal.
 */
const HOSTED_RUNNER_MARKERS: readonly string[] = [
  DISK_WEIGHT_SWITCH,
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
 The comment syntax each scanned extension holds.
 */
const LANGUAGES_BY_EXTENSION: ReadonlyMap<string, SourceLanguage> = new Map([
  ['.ts', 'typescript',],
  ['.mjs', 'typescript',],
  ['.toml', 'toml',],
  ['.yml', 'yaml',],
  ['.yaml', 'yaml',],
],);

/**
 Paths whose start still has no bound, each waiting on a size decision the owner has not made.

 Reading each start finds these, and neither may take a number invented here (repository rule LQM).
 `doc/handover/container-limits-everywhere.md` carries the decision each waits on, under `### Open`.
 The paired test below fails when a listed path no longer has an unbounded start, so an entry leaves this list with its
 fix.
 */
const UNBOUNDED_SITE_DECISIONS: readonly { readonly path: string; readonly reason: string; }[] = [
  {
    path: 'package/monorepo-task/core-exec/src/playwright-image-logic.ts',
    reason: 'the local build of playwright.Dockerfile has no memory or CPU bound, and its size is unmeasured',
  },
  {
    path: 'package/test-harness/core-matrix/src/container-image-rebuild-on-changed-inputs.ts',
    reason: 'the build every containerImageBuild caller reaches applies only the bounds a caller passes, so a default'
      + ' needs a size the inventory lists as unmeasured',
  },
];

/**
 Every file the scan can find, with its role.
 The comment above each group says why the role holds.
 */
const ledger: readonly { readonly path: string; readonly role: Role; }[] = [
  // Workflows on hosted runners, which refuse the disk weight (run 42447009237 of music-player-release.yml): they carry
  // a process limit and the switch that turns the weight off, and say why.
  ...[
    '.github/workflows/ide-release.yml',
    '.github/workflows/music-player-release.yml',
  ].map((path,) => ({ path, role: 'hosted-runner' as const, }),),
  // The docker run of pnpr-publish.yml, which the runner's root docker daemon accepts with `--blkio-weight`.
  { path: '.github/workflows/pnpr-publish.yml', role: 'starts' as const, },
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
    'package/container-limit/core/src/argument-text-of-a-container-start.ts',
    'package/container-limit/core/src/container-start-vocabulary.ts',
    'package/container-limit/core/src/container-starts-in-source-text.ts',
    'package/container-limit/core/src/positions-in-source-text.ts',
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
    'package/test-harness/core-matrix/src/pnpm-store-mount.ts',
    'package/test-harness/core-matrix/src/locked-tool-build-arguments.ts',
    'package/vm-builder/cli/src/image-signing.ts',
  ].map((path,) => ({ path, role: 'reads' as const, }),),
];

/**
 Package task files that run cargo clippy in a container, with the words that show each hands the lint the repository
 root `clippy.toml`.

 Clippy starts its configuration lookup in `CLIPPY_CONF_DIR`, else `CARGO_MANIFEST_DIR`, and walks up the directories
 from there (`doc/troubleshooting/clippy-toml-unstable-note.md`). A container that mounts only the package directory at
 `/work` therefore finds no `clippy.toml`, and the fleet-wide `disallowed-methods` ban, which lives only in the root
 file, silently stops applying: each crate's `[lints.clippy] disallowed_methods = "deny"` only switches that ban on
 (issue 1756).
 */
const CONTAINER_LINT_CONFIG_SITES: readonly { readonly path: string; readonly markers: readonly string[]; }[] = [
  { path: 'package/ide/app/mise.toml', markers: ['CLIPPY_CONF_DIR=/clippy-conf',], },
  { path: 'package/ide/app/task.ts', markers: ['CLIPPY_CONF_DIR=/clippy-conf',], },
  { path: 'package/music-player/desktop-app/mise.toml', markers: ['../../../clippy.toml',], },
  { path: 'package/music-player/desktop-app/task.ts', markers: ['../../../clippy.toml',], },
  { path: 'package/nested-wayland-session/cli/mise.toml', markers: ['CLIPPY_CONF_DIR=/clippy-conf',], },
  { path: 'package/terminal/app/mise.toml', markers: ['CLIPPY_CONF_DIR=/clippy-conf',], },
  { path: 'package/terminal/app/task.ts', markers: ['CLIPPY_CONF_DIR=/clippy-conf',], },
];

/**
 Package task files whose container cargo lint reads the repository root `clippy.toml` from an image that a build stages
 it into, with the words that show the staging.

 The linter's Clippy runs against a mount-free snapshot image, so a bind mount cannot reach it: `bin/test-container.ts`
 copies the root file into the build context and `test.Containerfile` copies it to `/work/clippy.toml`, where Clippy
 finds it above the crate it lints.
 */
const IMAGE_STAGED_LINT_CONFIG_SITES: readonly {
  readonly path: string;
  readonly stager: string;
  readonly image: string;
  readonly marker: string;
}[] = [
  {
    path: 'package/linter/monochromatic-lint/mise.toml',
    stager: 'package/linter/monochromatic-lint/bin/test-container.ts',
    image: 'package/linter/monochromatic-lint/test.Containerfile',
    marker: 'COPY clippy.toml /work/clippy.toml',
  },
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
 Words that show a cargo clippy argument list, quoted as a program argument.
 */
const CLIPPY_ARGUMENT_WORDS: readonly string[] = QUOTES.map((quote,) => `${quote}clippy${quote}`,);

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
 Texts already read, so the checks that read the same file do not read it again.

 The scan reads a file once per test and once more for each name it follows into a module, and the repository holds
 files of a megabyte.
 */
const textsRead = new Map<string, string>();

/**
 The text each name of a file declares, keyed by path, so the starts of one file share one pass over it.
 */
const definitionsRead = new Map<string, ReadonlyMap<string, string>>();

/**
 The text each name a file declares stands for, computed once per file.

 @param parameters - the repository-relative path of the file

 @returns one entry per declared name, mapped to the text of its initializer or body

 @example
 ```ts
 const definitions = definitionsOf({ path: 'mise.toml' });
 ```
 */
function definitionsOf({ path, }: {
  readonly path: string;
},): ReadonlyMap<string, string> {
  /**
   The index computed for this file earlier in the run, when it was computed already.
   */
  const earlier = definitionsRead.get(path,);
  if (earlier !== undefined)
    return earlier;
  /**
   The language whose comment syntax the file holds, when its extension holds one.
   */
  const language = LANGUAGES_BY_EXTENSION.get(extensionOf(path,),);
  /**
   The index itself, which stays empty for a path no start can be written in.
   */
  const definitions = language === undefined
    ? new Map<string, string>()
    : definedTextByName({
      blankedSourceText: stripSourceComments({ text: textOf(path,), language, },),
    },);
  definitionsRead.set(path, definitions,);
  return definitions;
}

/**
 Text of a tracked file.

 @param path - repository-relative path

 @returns the file's text

 @throws When the path names no file

 @example
 ```ts
 const text = textOf('mise.toml');
 ```
 */
function textOf(path: string,): string {
  /**
   Text read earlier in this run, when the file has been read already.
   */
  const earlier = textsRead.get(path,);
  if (earlier !== undefined)
    return earlier;
  /**
   The file's text, kept for the reads that follow.
   */
  const text = readFileSync(
    join(
      repositoryRoot,
      path,
    ),
    'utf8',
  );
  textsRead.set(path, text,);
  return text;
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
  'hosted-runner': HOSTED_RUNNER_MARKERS,
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

/**
 One start whose own argument list holds no word that shows where its limits come from.
 */
type UncoveredStart = {
  /**
   Repository-relative path of the file that starts the container.
   */
  readonly path: string;
  /**
   Line of the program name in that file.
   */
  readonly line: number;
  /**
   Argument text that reaches the program.
   */
  readonly argvText: string;
};

/**
 The extension a path ends with, which is the last dot and whatever follows it.

 @param path - repository-relative path

 @returns the extension, dot included, or the empty string when the path holds no dot

 @example
 ```ts
 const extension = extensionOf('mise.toml');
 ```
 */
function extensionOf(path: string,): string {
  for (let index = path.length - 1; index >= 0; index -= 1) {
    if (path[index] === '.')
      return path.slice(index,);
  }
  return '';
}

/**
 Words that show a start's own argument list takes the limits.

 A start whose arguments a builder returns has no verb here, so either bound answers for it.

 @param start - the start to find the words for

 @returns the words, any one of which shows where the start's limits come from

 @example
 ```ts
 const markers = markersReaching(start);
 ```
 */
function markersReaching(start: ContainerStartInvocation,): readonly string[] {
  if (start.verb === 'build')
    return [...BUILDS_MARKERS, ...PASSES_MARKERS,];
  if ((start.verb === 'run') || (start.verb === 'create'))
    return [...STARTS_MARKERS, ...PASSES_MARKERS,];
  return [...STARTS_MARKERS, ...BUILDS_MARKERS, ...PASSES_MARKERS,];
}

/**
 The definitions a file's imports give its names, read one module deep into the file each import names.

 A start can spread a builder an adjacent module exports (`...buildBounds`), so a name the file does not declare
 itself is read from the module its import names. A package specifier is not followed: this ledger reads the
 repository's own files.

 @param path - the repository-relative path of the importing file

 @returns the text each imported name stands for in the module its import names, for the names this repository's files
   answer

 @example
 ```ts
 const imported = importedDefinitionsOf({ path: 'package/x/bin/build.ts' });
 ```
 */
function importedDefinitionsOf({ path, }: {
  readonly path: string;
},): ReadonlyMap<string, string> {
  const imported = new Map<string, string>();
  /**
   The importing file's own language, when its extension holds one.
   */
  const language = LANGUAGES_BY_EXTENSION.get(extensionOf(path,),);
  if (language === undefined)
    return imported;
  /**
   Each imported name of the file, with the module its import names.
   */
  const modules = importedModuleByName({ text: textOf(path,), language, },);
  for (const [name, specifier,] of modules) {
    if (!specifier.startsWith('.',))
      continue;
    /**
     Repository-relative path of the module the import names.
     */
    const modulePath = normalize(join(dirname(path,), specifier,),);
    /**
     Whether the module is a file of this repository that a start can be written in.
     */
    const readable = LANGUAGES_BY_EXTENSION.has(extensionOf(modulePath,),)
      && existsSync(join(repositoryRoot, modulePath,),);
    if (!readable)
      continue;
    /**
     The text the module gives that name.
     */
    const text = definitionsOf({ path: modulePath, },).get(name,);
    if (text !== undefined)
      imported.set(name, text,);
  }
  return imported;
}

/**
 Each start in the repository whose own argument list holds no word that shows where its limits come from.

 @returns one entry per such start, in ledger order then line order

 @example
 ```ts
 const uncovered = uncoveredStarts();
 ```
 */
function uncoveredStarts(): readonly UncoveredStart[] {
  const uncovered: UncoveredStart[] = [];
  for (const { path, } of ledger) {
    /**
     The language whose comment syntax the file holds, when its extension holds one.
     */
    const language = LANGUAGES_BY_EXTENSION.get(extensionOf(path,),);
    if (language === undefined)
      continue;
    /**
     The file's text.
     */
    const text = textOf(path,);
    /**
     The definitions the file's imports give its names, from the modules beside it.
     */
    const importedDefinitions = importedDefinitionsOf({ path, },);
    /**
     The text each name the file declares stands for.
     */
    const definitions = definitionsOf({ path, },);
    for (const start of containerStartInvocations({ text, language, importedDefinitions, },)) {
      /**
       The start's own argument text, with the text of every binding and builder it names.
       */
      const reaching = argumentTextWithBoundInitializers({ argumentText: start.argvText, definitions, importedDefinitions, },);
      if (markersReaching(start,).some((marker,) => reaching.includes(marker,),))
        continue;
      uncovered.push({
        path,
        line: start.line,
        argvText: start.argvText,
      },);
    }
  }
  return uncovered;
}

/**
 One uncovered start, written for a failure message.

 @param start - the start to describe

 @returns the file, the line and the argument text that reaches the program

 @example
 ```ts
 const line = describeUncoveredStart(start);
 ```
 */
function describeUncoveredStart({ path, line, argvText, }: UncoveredStart,): string {
  return `${path}:${String(line,)} ${argvText.trim().replaceAll('\n', ' ',).slice(0, 160,)}`;
}

/**
 Package task files the ledger lists that pass a cargo clippy argument to a container program.

 @returns repository-relative paths

 @example
 ```ts
 const files = packageTaskFilesRunningAContainerLint();
 ```
 */
function packageTaskFilesRunningAContainerLint(): readonly string[] {
  return ledger
    .map((entry,) => entry.path,)
    .filter((path,) => path.endsWith('/mise.toml',) || path.endsWith('/task.ts',),)
    .filter((path,) => {
      /**
       Text of the entry's file.
       */
      const text = textOf(path,);
      return namesProgram(text,)
        && CLIPPY_ARGUMENT_WORDS.some((word,) => text.includes(word,),);
    },);
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
      name: 'shows the limits in each start the file writes, not only somewhere in the file',
      fn: async () => {
        /**
         Paths whose unbounded start waits on a size decision of the owner.
         */
        const waitingOnADecision = new Set(UNBOUNDED_SITE_DECISIONS.map((entry,) => entry.path,),);
        expect(
          uncoveredStarts()
            .filter((start,) => !waitingOnADecision.has(start.path,),)
            .map(describeUncoveredStart,),
        ).toEqual([],);
      },
    },),
    it({
      name: 'keeps a listed size decision only while a start there still has no bound',
      fn: async () => {
        /**
         Starts whose own argument list holds no word that shows where their limits come from.
         */
        const uncovered = uncoveredStarts();
        for (const { path, } of UNBOUNDED_SITE_DECISIONS)
          expect(uncovered.some((start,) => start.path === path,),).toBe(true,);
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
      name: 'turns the weight off in every workflow job that starts containers on a hosted runner, and says why',
      fn: async () => {
        for (const workflow of [
          '.github/workflows/ide-release.yml',
          '.github/workflows/music-player-release.yml',
          '.github/workflows/env-local-json.yml',
        ]) {
          /**
           Text of the workflow.
           */
          const text = textOf(workflow,);
          expect(text,).toContain(`${DISK_WEIGHT_SWITCH}: '${DISK_WEIGHT_OFF}'`,);
          expect(text,).toContain('run 42447009237',);
          expect(text,).toContain("crun: opening file 'io.weight' for writing",);
        }
      },
    },),
    it({
      name: 'hands the repository root clippy.toml to every container cargo lint in a package task file',
      fn: async () => {
        /**
         Paths covered by either list: a bind mount in the task file, or an image that stages the file.
         */
        const listed = new Set([
          ...CONTAINER_LINT_CONFIG_SITES.map((site,) => site.path,),
          ...IMAGE_STAGED_LINT_CONFIG_SITES.map((site,) => site.path,),
        ],);
        expect(packageTaskFilesRunningAContainerLint().filter((path,) => !listed.has(path,),),).toEqual([],);
        /**
         Sites whose file lacks every word that would hand the lint the root file.
         */
        const withoutMarker = CONTAINER_LINT_CONFIG_SITES
          .filter((site,) => !site.markers.some((marker,) => textOf(site.path,).includes(marker,),),)
          .map((site,) => site.path,);
        expect(withoutMarker,).toEqual([],);
      },
    },),
    it({
      name: 'stages the repository root clippy.toml into the image that carries the linter container lint',
      fn: async () => {
        for (const site of IMAGE_STAGED_LINT_CONFIG_SITES) {
          expect(textOf(site.stager,),).toContain('../../../clippy.toml',);
          expect(textOf(site.image,),).toContain(site.marker,);
        }
      },
    },),
    it({
      name: 'keeps a process limit and no weight option in the option variable of the hosted-runner workflows',
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
            expect(line,).toContain('--pids-limit=',);
            for (const option of diskWeightArguments({ environment: {}, },))
              expect(line,).not.toContain(option,);
          }
        }
      },
    },),
  ],
},);
