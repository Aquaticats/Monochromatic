/**
 Checks that no workflow installs a runtime at a version of its own instead of the one `toolchain.lock.json` records.

 `.github/actions/task-cli-setup` installs the Node and pnpm the lock lists whenever its `node-version` and
 `pnpm-version` inputs are empty, and `.github/actions/rust-toolchain-setup` installs the Rust toolchain the lock names,
 which is the documented way for a workflow to read the lock. A workflow that passes a runtime version of its own runs
 one no developer, no release job and no other workflow uses, and the value drifts away from the lock silently: the
 pinned Node of this repository has moved on while a workflow kept a hand-written version, and a workflow that installed
 the floating `nightly` channel built a release under a compiler the container gate never ran.

 The check reports the file and the line it fails on, so a job that needs another runtime on purpose can replace it with
 the reason it needs one.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  glob,
  readFile,
} from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

//region Reading the workflows

/**
 One workflow file: its repository-relative path, its text, and its lines.
 */
type Workflow = {
  readonly path: string;
  readonly text: string;
  readonly lines: readonly string[];
};

/**
 The repository root, which the task runner does not leave as the working directory: a test file runs from its package
 folder, so a glob without this would match nothing and every override would go unseen.
 */
const repositoryRoot = fileURLToPath(new URL('../../../../', import.meta.url,),);

/**
 Repository-relative paths of every workflow file, in sorted order so a message does not depend on directory order.
 */
const workflowPaths = (await Array.fromAsync(glob('.github/workflows/*.yml', { cwd: repositoryRoot, },),)).toSorted();

/**
 Every workflow of the repository.
 */
const workflows: readonly Workflow[] = await Promise.all(
  workflowPaths.map(async function read(path,): Promise<Workflow> {
    /**
     Whole text of the workflow file.
     */
    const text = await readFile(
      new URL(`../../../../${path}`, import.meta.url,),
      'utf8',
    );
    return {
      path,
      text,
      lines: text.split('\n',),
    };
  },),
);

/**
 The action whose inputs decide which Node and pnpm a job installs.
 */
const setupAction = await readFile(new URL('../../../../.github/actions/task-cli-setup/action.yml', import.meta.url,), 'utf8',);

/**
 The action that installs the Rust toolchain the lock names, which every workflow that compiles Rust uses.
 */
const rustSetupAction = await readFile(new URL('../../../../.github/actions/rust-toolchain-setup/action.yml', import.meta.url,), 'utf8',);

/**
 The step program the Rust action runs, and the module that reads the lock row for it.
 */
const rustSetupProgramPath = 'package/monorepo-task/core-exec/src/rust-toolchain-setup.ts';
const rustLockReaderPath = 'package/monorepo-task/core-exec/src/locked-rust-toolchain.ts';
const rustSetupProgram = await readFile(new URL(`../../../../${rustSetupProgramPath}`, import.meta.url,), 'utf8',);
const rustLockReader = await readFile(new URL(`../../../../${rustLockReaderPath}`, import.meta.url,), 'utf8',);

/**
 Version inputs of the setup action that a workflow may pass to override the lock, with the lock row each one falls back
 to when it is empty.
 */
const overridableInputs: readonly { readonly input: string; readonly lockRow: string }[] = [
  { input: 'node-version', lockRow: '.tools.node.version' },
  { input: 'pnpm-version', lockRow: '.tools.pnpm.version' },
];

/**
 Row of `toolchain.lock.json` that names the Rust toolchain every build of this repository compiles with: a dated
 nightly, which the daily update keeps at the newest nightly and nothing else (`doc/handover/cli-git-rust-implementation.md`,
 owner decision of 2026-11-15: "Everything nightly. [...] And the lock's nightly is always the latest nightly anyway.").
 */
const rustLockRow = '.tools.rust.version';

//endregion

//region Reading a rustup command out of a line

/**
 Words of one line, with the empty strings a run of spaces leaves between them dropped.

 YAML forbids a tab in indentation, and these commands are code rather than prose, so a plain space is the separator.

 @param line - one line of a workflow file

 @returns its words, in order

 @example
 ```ts
 wordsOf('    rustup default nightly'); // ['rustup', 'default', 'nightly']
 ```
 */
function wordsOf(line: string,): readonly string[] {
  return line.split(' ',)
    .filter(function notEmpty(word,) {
      return word !== '';
    },);
}

/**
 A word with the quotes a shell command may put around it taken off.

 @param word - one word of a line

 @returns the word without its double quotes

 @example
 ```ts
 unquoted('"$toolchain"'); // '$toolchain'
 ```
 */
function unquoted(word: string,): string {
  return word.replaceAll('"', '',);
}

/**
 The words of a `rustup` command after its `rustup` word, when the line holds one.

 @param line - one line of a workflow file

 @returns the command and its arguments as the only member, or an empty list when the line runs no `rustup`

 @example
 ```ts
 rustupArguments('        rustup default "$RUST_TOOLCHAIN"'); // [['default', '"$RUST_TOOLCHAIN"']]
 ```
 */
function rustupArguments(line: string,): readonly (readonly string[])[] {
  /**
   Words of the line.
   */
  const words = wordsOf(line,);
  for (const [index, word] of words.entries()) {
    if (word === 'rustup')
      return [words.slice(index + 1,)];
  }
  return [];
}

/**
 The toolchain name a line hands to `rustup`, when the line installs or selects a toolchain at all.

 @param line - one line of a workflow file

 @returns the toolchain word as the line spells it, quotes included, as the only member, or an empty list when the line
 is not a `rustup toolchain install` or `rustup default`, or stops before naming a toolchain

 @example
 ```ts
 rustupToolchainWord('        rustup default "$RUST_TOOLCHAIN"'); // ['"$RUST_TOOLCHAIN"']
 ```
 */
function rustupToolchainWord(line: string,): readonly string[] {
  /**
   Command and its arguments of the line's `rustup` command, when it has one.
   */
  const [command] = rustupArguments(line,);
  if (command === undefined)
    return [];
  /**
   The command's own name and its first argument.
   */
  const [name, first] = command;
  // An install names its toolchain as its fourth word, a default as its third; `slice` answers an empty list when the
  // command stops before its toolchain, which the caller reads as nothing typed.
  if (name === 'toolchain') {
    return first === 'install'
      ? command.slice(
        2,
        3,
      )
      : [];
  }
  if (name === 'default')
    return command.slice(
      1,
      2,
    );
  return [];
}

/**
 Whether a line installs a Rust toolchain.

 @param line - one line of a workflow file

 @returns whether the line runs `rustup toolchain install`

 @example
 ```ts
 installsRustToolchain('        rustup toolchain install "$TOOLCHAIN"'); // true
 ```
 */
function installsRustToolchain(line: string,): boolean {
  /**
   Command and its arguments of the line's `rustup` command, when it has one.
   */
  const [command] = rustupArguments(line,);
  if (command === undefined)
    return false;
  /**
   The command's own name and its first argument.
   */
  const [name, first] = command;
  if (name !== 'toolchain')
    return false;
  return first === 'install';
}

//endregion

await describe({
  name: 'workflow toolchain versions',
  children: [
    it({
      name: 'finds the workflows it reads, so an empty selection cannot pass the checks below',
      fn: async function findsItsSubjects() {
        expect(workflowPaths.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'leave every version input of the setup action empty, so each job installs the toolchain the lock records',
      fn: async function noOverrides() {
        const overrides = workflows.flatMap(function overridesOf(workflow,) {
          return workflow.lines.flatMap(function inputsOf(line, index,) {
            const trimmed = line.trim();
            const passesInput = overridableInputs.some(function isVersionInput({ input, },) {
              return trimmed.startsWith(`${input}:`);
            },);
            return passesInput ? [`${workflow.path}:${index + 1}: ${trimmed}`] : [];
          },);
        },);
        expect(overrides,).toEqual([],);
      },
    },),
    it({
      name: 'take both versions from toolchain.lock.json when a job passes neither, so emptying an input cannot leave a guess in place',
      fn: async function defaultsReadTheLock() {
        for (const { lockRow } of overridableInputs) expect(setupAction,).toContain(lockRow,);
      },
    },),
    it({
      name: 'name a shell variable rather than a typed toolchain in every rustup install or default, so a build gets the compiler the gate ran',
      fn: async function noTypedRustToolchain() {
        const typed = workflows.flatMap(function typedIn(workflow,) {
          return workflow.lines.flatMap(function namesAToolchain(line, index,) {
            /**
             Toolchain word of the line, when the line installs or selects one.
             */
            const [toolchain] = rustupToolchainWord(line,);
            if (toolchain === undefined)
              return [];
            /**
             Whether the workflow reads the name from a variable instead of typing it.
             */
            const namesAVariable = unquoted(toolchain,)
              .startsWith('$',);
            return namesAVariable ? [] : [`${workflow.path}:${index + 1}: ${line.trim()}`];
          },);
        },);
        expect(typed,).toEqual([],);
      },
    },),
    it({
      name: 'carry the lock row in every workflow that installs a Rust toolchain, so a direct install reads the lock itself',
      fn: async function installsReadTheLock() {
        const withoutTheLock = workflows
          .filter(function installsRust(workflow,) {
            return workflow.lines.some(function installs(line,) {
              return installsRustToolchain(line,);
            },);
          })
          .filter(function lacksTheRow(workflow,) {
            return !workflow.text.includes(rustLockRow,);
          })
          .map(function pathOf(workflow,) {
            return workflow.path;
          });
        expect(withoutTheLock,).toEqual([],);
      },
    },),
    it({
      name: 'take the Rust version from the lock through the action, whose step names the program that reads the row',
      fn: async function rustActionReadsTheLock() {
        expect(rustSetupAction,).toContain(`node ${rustSetupProgramPath}`,);
        expect(rustSetupProgram,).toContain('lockedRustToolchainVersion',);
        expect(rustLockReader,).toContain(rustLockRow,);
      },
    },),
  ],
},);
