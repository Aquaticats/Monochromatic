/**
 Checks that a repository-root build file installs its toolchain from an archive `toolchain.lock.json` pins and verifies,
 never by piping a downloaded script into a shell.

 A file at the repository root has no `COPY` step it could read the lock with, so the version and digests it needs are
 restated in the file itself. The restatement is what this check owns: the values must equal the lock's rows, so a lock
 bump that forgets the build file fails here instead of producing images whose toolchain content depends on whatever an
 installer endpoint serves at build time.
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

//region The build files and the lock

/**
 Repository-root build files, which build with the repository root as their context.
 */
const buildFilePaths = (await Array.fromAsync(glob('*Dockerfile',),)).toSorted();

/**
 The lock every tool version comes from.
 */
type Lock = {
  readonly tools: Readonly<Record<string, {
    readonly version: string;
    readonly artifacts: Readonly<Record<string, { readonly url: string; readonly sha256: string }>>;
  }>>;
};

/**
 The parsed lock.
 */
const lock = JSON.parse(await readFile(new URL('../../../../toolchain.lock.json', import.meta.url,), 'utf8',),) as Lock;

/**
 Text of each repository-root build file, with its path.
 */
const buildFiles = await Promise.all(
  buildFilePaths.map(async function read(path,) {
    return { path, text: await readFile(new URL(`../../../../${path}`, import.meta.url,), 'utf8',) };
  },),
);

/**
 The characters a shell name is made of, so the scan stops at a space, a quote or a semicolon.
 */
const lowercaseLetters = 'abcdefghijklmnopqrstuvwxyz';

/**
 Shells a build could pipe a download into.
 */
const shells: readonly string[] = ['sh', 'bash'];

/**
 The command a pipe feeds on one line: the word after the `|`, or an empty text when nothing follows it or the line has
 no pipe.

 @param line - one line of a build file

 @returns the shell a download would be piped into

 @example
 ```ts
 commandAfterPipe('curl -fsSL https://example.test/i.sh | sh') === 'sh';
 ```
 */
function commandAfterPipe(line: string,): string {
  const pipeAt = line.indexOf('|',);
  if (pipeAt === (-1)) return '';
  const rest = line.slice(pipeAt + 1,);
  const from = rest.length - rest.trimStart().length;
  const word: string[] = [];
  for (let index = from; index < rest.length; index += 1) {
    const character = rest.charAt(index,);
    if (!lowercaseLetters.includes(character,)) break;
    word.push(character,);
  }
  return word.join('',);
}

/**
 Whether any line of a build file pipes a download into a shell.

 @param text - contents of a build file

 @returns true when a shell runs a downloaded script

 @example
 ```ts
 pipesIntoShell('RUN curl https://example.test/i.sh | sh') === true;
 ```
 */
function pipesIntoShell(text: string,): boolean {
  return text.split('\n',)
    .some(function feedsAShell(line,) {
      return shells.includes(commandAfterPipe(line,),);
    },);
}

/**
 The tool this repository ships inside the browser-test image, whose row the image restates.
 */
const browserImageTool = 'bun';

//endregion

await describe({
  name: 'repository-root build files',
  children: [
    it({
      name: 'are found, so an empty selection cannot pass the checks below',
      fn: async function hasBuildFiles() {
        expect(buildFilePaths.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'never pipe a downloaded script into a shell, which would run whatever the endpoint serves as root at build time',
      fn: async function noInstallerScripts() {
        const piped = buildFiles.filter(function runsAnInstaller(file,) {
          return pipesIntoShell(file.text,);
        },)
          .map(function name(file,) {
            return file.path;
          },);
        expect(piped,).toEqual([],);
      },
    },),
    it({
      name: 'restate the locked version and digests of the toolchain they install, so a lock bump reaches the image',
      fn: async function restatesTheLock() {
        const row = lock.tools[browserImageTool];
        expect(row,).not.toBe(undefined,);
        if (row === undefined) return;
        const carries = buildFiles.filter(function namesTheTool(file,) {
          return file.text.includes(browserImageTool,);
        },);
        expect(carries.map(function name(file,) {
          return file.path;
        },),).not.toEqual([],);
        // The image is a glibc distribution, so the musl rows of the lock are archives it cannot run and cannot carry.
        const runnable = Object.entries(row.artifacts,)
          .filter(function notMusl([platform,],) {
            return !platform.includes('musl',);
          },);
        expect(runnable.length,).toBeGreaterThan(0,);
        for (const file of carries) {
          expect(file.text,).toContain(`BUN_VERSION=${row.version}`,);
          for (const [, artifact] of runnable) expect(file.text,).toContain(artifact.sha256,);
        }
      },
    },),
  ],
},);
