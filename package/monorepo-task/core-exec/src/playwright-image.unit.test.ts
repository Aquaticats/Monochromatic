import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import {
  dirname,
  join,
} from 'node:path';

import {
  ensurePlaywrightImage,
  PLAYWRIGHT_LOCAL_IMAGE,
  PLAYWRIGHT_REGISTRY_IMAGE,
  PlaywrightImageError,
  type Podman,
  type PodmanResult,
  playwrightImageTag,
} from '../dist/final/node/testing.mjs';

//region Fixtures: a scripted podman

/**
 Bytes of a Dockerfile the decisions are tried on, and the tag its SHA-256 gives.
 */
const DOCKERFILE = new TextEncoder().encode('abc',);

/**
 SHA-256 of `abc` is `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` (FIPS 180-2 test vector).
 */
const TAG = 'ba7816bf8f01cfea';

/**
 Registry reference and local names for {@link TAG}.
 */
const REFERENCE = `ghcr.io/aquaticats/monochromatic-playwright:${TAG}`;
const TAGGED = `monochromatic-playwright:${TAG}`;

/**
 What a scripted podman answers per subcommand; a missing subcommand exits 0 with no output.
 */
type Script = Readonly<Record<string, PodmanResult>>;

/**
 Whether text is exactly 16 lowercase hex digits.

 @param text - Candidate tag

 @returns True for a well-formed tag

 @example
 ```ts
 isSixteenHexDigits('ba7816bf8f01cfea'); // true
 ```
 */
function isSixteenHexDigits(text: string,): boolean {
  return (text.length === 16) && text.split('',).every(isLowercaseHexDigit,);
}

/**
 Whether one character is a lowercase hex digit.

 @param character - One character

 @returns True for `0` to `9` and `a` to `f`

 @example
 ```ts
 isLowercaseHexDigit('f'); // true
 ```
 */
function isLowercaseHexDigit(character: string,): boolean {
  return '0123456789abcdef'.includes(character,);
}

/**
 A podman that records every call and answers from a script.

 @param script - Answers per subcommand

 @returns The podman function, the recorded calls and the printed log lines

 @example
 ```ts
 const { podman, calls, } = scripted({ pull: { exitCode: 1, output: 'unauthorized', }, },);
 ```
 */
function scripted(script: Script,) {
  /**
   Every call, as the command line after `podman`.
   */
  const calls: string[] = [];
  /**
   Lines printed through `log`.
   */
  const lines: string[] = [];
  /**
   The script as a map, so that a subcommand it is silent about is a plain miss.
   */
  const answers = new Map(Object.entries(script,),);
  async function podman(args: readonly string[],): Promise<PodmanResult> {
    calls.push(args.join(' ',),);
    return answers.get(args.at(0,) ?? '',) ?? { exitCode: 0, output: '', };
  }
  function log(line: string,): void {
    lines.push(line,);
  }
  return { podman: podman satisfies Podman, calls, lines, log, };
}

//endregion Fixtures: a scripted podman

//region Fixtures: the real entry with a fake podman program

/**
 Directory of this package, where fixtures go under `dist/temp/test`.
 */
const PACKAGE_DIRECTORY = join(import.meta.dirname, '..',);

/**
 The command-line entry, run from source as mise does when nothing is built.
 */
const ENTRY = join(PACKAGE_DIRECTORY, 'src', 'playwright-image.ts',);

/**
 Source of a fake `podman`: it appends its arguments to `calls.log` beside it and exits as the environment says.
 */
const FAKE_PODMAN = `#!/usr/bin/env node
import { appendFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
const calls = join(dirname(process.argv[1]), 'calls.log');
const args = process.argv.slice(2);
appendFileSync(calls, args.join(' ') + '\\n');
const exits = { image: process.env.FAKE_EXISTS_EXIT, pull: process.env.FAKE_PULL_EXIT, build: process.env.FAKE_BUILD_EXIT };
const code = Number(exits[args[0]] ?? '0');
if (code !== 0) console.error(args[0] === 'pull' ? 'Error: initializing source docker://x: unauthorized' : 'step failed in ' + args[0]);
process.exit(code);
`;

/**
 Result of running the entry.
 */
type EntryRun = {
  readonly exitCode: number;
  readonly output: string;
  readonly calls: readonly string[];
};

/**
 Runs the entry in a fresh directory with a `playwright.Dockerfile` and a fake `podman` first on `PATH`.

 @param environment - Exit codes the fake podman uses per subcommand
 @param withPodman - Whether the fake podman is on `PATH` at all

 @returns Exit status, combined output and the arguments podman was given

 @example
 ```ts
 const run = await runEntry({ environment: { FAKE_PULL_EXIT: '1', }, withPodman: true, },);
 ```
 */
async function runEntry(
  { environment, withPodman, }: { readonly environment: Readonly<Record<string, string>>; readonly withPodman: boolean; },
): Promise<EntryRun> {
  /**
   Fresh fixture directory, removed when this function ends.
   */
  using fixture = makeFixture({ withPodman, },);
  const { directory, } = fixture;
  /**
   Where the fake podman records its calls.
   */
  const callsLog = join(directory, 'bin', 'calls.log',);
  /**
   Result of the entry; a non-zero exit rejects.
   */
  const done = await spawn(process.execPath, [ENTRY,], {
    cwd: directory,
    // The fake podman is a node script, so `node` has to be found; without podman the directory is empty.
    env: { PATH: withPodman ? `${join(directory, 'bin',)}:${dirname(process.execPath,)}` : join(directory, 'bin',), ...environment, },
  },).then(
    function ok(result,) {
      return { exitCode: 0, output: result.output, };
    },
    function failed(error: SubprocessError,) {
      return { exitCode: error.exitCode ?? (-1), output: error.output, };
    },
  );
  /**
   Arguments the fake podman recorded; none when podman was absent.
   */
  const calls = existsSync(callsLog,) ? readFileSync(callsLog, 'utf8',).split('\n',).filter(Boolean,) : [];
  return { ...done, calls, };
}

/**
 A fixture directory that deletes itself.

 @example
 ```ts
 using fixture = makeFixture({ withPodman: true, },);
 ```
 */
type Fixture = {
  readonly directory: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Makes a directory with a `playwright.Dockerfile` and, optionally, the fake `podman` in its `bin` folder.

 @param withPodman - Whether to write the fake podman

 @returns The directory, removed by `using`

 @example
 ```ts
 using fixture = makeFixture({ withPodman: false, },);
 ```
 */
function makeFixture({ withPodman, }: { readonly withPodman: boolean; },): Fixture {
  /**
   Unique directory below the package's own `dist/temp/test`.
   */
  const directory = join(
    PACKAGE_DIRECTORY,
    'dist',
    'temp',
    'test',
    `playwright-image-${Date.now()}-${Math.random().toString(36,).slice(2, 8,)}`,
  );
  mkdirSync(join(directory, 'bin',), { recursive: true, },);
  writeFileSync(join(directory, 'playwright.Dockerfile',), DOCKERFILE,);
  if (withPodman) {
    writeFileSync(join(directory, 'bin', 'podman',), FAKE_PODMAN,);
    chmodSync(join(directory, 'bin', 'podman',), 0o755,);
  }
  return {
    directory,
    [Symbol.dispose]() {
      rmSync(directory, { recursive: true, force: true, },);
    },
  };
}

//endregion Fixtures: the real entry with a fake podman program

await describe({
  name: '',
  children: [
    describe({
      name: playwrightImageTag.name,
      children: [
        it({
          name: 'is the first 16 hex digits of the SHA-256 of the bytes',
          fn: async () => {
            expect(playwrightImageTag(DOCKERFILE,),).toBe(TAG,);
            expect(
              isSixteenHexDigits(playwrightImageTag(DOCKERFILE,),),
            ).toBe(true,);
            expect(isSixteenHexDigits('BA7816BF8F01CFEA',),).toBe(false,);
            expect(isSixteenHexDigits('ba7816bf8f01cfe',),).toBe(false,);
          },
        },),
        it({
          name: 'changes with a single byte, comments included',
          fn: async () => {
            expect(
              playwrightImageTag(new TextEncoder().encode('abd',),),
            ).not.toBe(TAG,);
          },
        },),
        it({
          name: 'equals sha256sum of the repository playwright.Dockerfile cut to 16 characters, which the workflow uses',
          fn: async () => {
            /**
             The file CI hashes and `prepare:playwright` reads.
             */
            const file = join(PACKAGE_DIRECTORY, '..', '..', '..', 'playwright.Dockerfile',);
            const shell = await spawn('sha256sum', [file,],);
            expect(
              playwrightImageTag(readFileSync(file,),),
            ).toBe(shell.stdout.slice(0, 16,),);
          },
        },),
      ],
    },),
    describe({
      name: ensurePlaywrightImage.name,
      children: [
        it({
          name: 'names the registry image exactly as the owner decided',
          fn: async () => {
            expect(PLAYWRIGHT_REGISTRY_IMAGE,).toBe('ghcr.io/aquaticats/monochromatic-playwright',);
            expect(PLAYWRIGHT_LOCAL_IMAGE,).toBe('monochromatic-playwright',);
          },
        },),
        it({
          name: 'keeps a local image of this tag, points the plain name at it, and pulls and builds nothing',
          fn: async () => {
            const { podman, calls, lines, log, } = scripted({ image: { exitCode: 0, output: '', }, },);
            const source = await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman, log, },);
            expect(source,).toBe('present',);
            expect(calls,).toEqual([`image exists ${TAGGED}`, `tag ${TAGGED} monochromatic-playwright`,],);
            expect(lines,).toEqual([`using ${TAGGED} (already local)`,],);
          },
        },),
        it({
          name: 'pulls the registry tag and gives it both local names',
          fn: async () => {
            const { podman, calls, lines, log, } = scripted({ image: { exitCode: 1, output: '', }, },);
            const source = await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman, log, },);
            expect(source,).toBe('pulled',);
            expect(calls,).toEqual([
              `image exists ${TAGGED}`,
              `pull ${REFERENCE}`,
              `tag ${REFERENCE} ${TAGGED}`,
              `tag ${REFERENCE} monochromatic-playwright`,
            ],);
            expect(lines,).toEqual([`pulled ${REFERENCE}`,],);
          },
        },),
        it({
          name: 'prints why the pull failed and builds the file locally with both names',
          fn: async () => {
            const { podman, calls, lines, log, } = scripted({
              image: { exitCode: 1, output: '', },
              pull: { exitCode: 125, output: 'Trying to pull x...\nGetting image source signatures\nError: unauthorized\n\n', },
            },);
            const source = await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman, log, },);
            expect(source,).toBe('built',);
            expect(calls,).toEqual([
              `image exists ${TAGGED}`,
              `pull ${REFERENCE}`,
              `build --tag ${TAGGED} --tag monochromatic-playwright --file playwright.Dockerfile .`,
            ],);
            expect(lines[0],).toBe(
              `could not pull ${REFERENCE}: Error: unauthorized; building playwright.Dockerfile locally`,
            );
            expect(lines[1],).toBe(`built ${TAGGED}`,);
          },
        },),
        it({
          name: 'says podman printed nothing when a failed pull has no output, and keeps only the last three lines of one without an Error line',
          fn: async () => {
            const quiet = scripted({ image: { exitCode: 1, output: '', }, pull: { exitCode: 1, output: '', }, },);
            await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman: quiet.podman, log: quiet.log, },);
            expect(quiet.lines[0],).toContain('podman printed nothing',);

            const long = scripted({ image: { exitCode: 1, output: '', }, pull: { exitCode: 1, output: 'one\ntwo\nthree\nfour\nfive', }, },);
            await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman: long.podman, log: long.log, },);
            expect(long.lines[0],).toContain(': three / four / five;',);
            expect(long.lines[0],).not.toContain('two',);
          },
        },),
        it({
          name: 'gives the last Error line as the reason, not the retry warnings that come before it',
          fn: async () => {
            const { podman, lines, log, } = scripted({
              image: { exitCode: 1, output: '', },
              pull: {
                exitCode: 125,
                output: [
                  'Trying to pull x...',
                  'time="2026-10-31T20:16:51-04:00" level=warning msg="Failed, retrying in 1s ... (1/3). Error: initializing source"',
                  'Error: unable to copy from source: Requesting bearer token: received unexpected HTTP status: 403 Forbidden',
                  '',
                ].join('\n',),
              },
            },);
            await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman, log, },);
            expect(lines[0],).toBe(
              `could not pull ${REFERENCE}: Error: unable to copy from source: Requesting bearer token: received unexpected HTTP status: 403 Forbidden; building playwright.Dockerfile locally`,
            );
          },
        },),
        it({
          name: 'throws with both reasons and the build output when the pull and the build both fail',
          fn: async () => {
            const { podman, log, } = scripted({
              image: { exitCode: 1, output: '', },
              pull: { exitCode: 125, output: 'Error: unauthorized', },
              build: { exitCode: 125, output: 'newuidmap: executable file not found', },
            },);
            let caught: unknown;
            try {
              await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman, log, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(PlaywrightImageError,);
            const {message} = (caught as Error);
            expect(message,).toContain(`${REFERENCE} could not be pulled (Error: unauthorized)`,);
            expect(message,).toContain('podman build exited 125',);
            expect(message,).toContain('newuidmap: executable file not found',);
          },
        },),
        it({
          name: 'throws with the command when tagging a pulled image fails, instead of reporting it pulled',
          fn: async () => {
            const { podman, log, } = scripted({
              image: { exitCode: 1, output: '', },
              tag: { exitCode: 125, output: 'image not known', },
            },);
            let caught: unknown;
            try {
              await ensurePlaywrightImage({ dockerfile: DOCKERFILE, dockerfilePath: 'playwright.Dockerfile', podman, log, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(PlaywrightImageError,);
            expect((caught as Error).message,).toContain(`podman tag ${REFERENCE} ${TAGGED} exited 125: image not known`,);
          },
        },),
        it({
          name: 'treats an exit status other than 0 from image exists as not local, as 1 and 125 both are',
          fn: async () => {
            /**
             Sources and second calls for each exit status, tried together.
             */
            const outcomes = await Promise.all([1, 125, 127,].map(async function tryExitCode(exitCode,) {
              const { podman, calls, log, } = scripted({ image: { exitCode, output: '', }, },);
              const source = await ensurePlaywrightImage({
                dockerfile: DOCKERFILE,
                dockerfilePath: 'playwright.Dockerfile',
                podman,
                log,
              },);
              return { source, second: calls[1], };
            },),);
            expect(outcomes,).toEqual(
              [1, 2, 3,].map(function pulledAfterNotLocal() {
                return { source: 'pulled', second: `pull ${REFERENCE}`, };
              },),
            );
          },
        },),
      ],
    },),
    describe({
      name: 'command-line entry',
      children: [
        it({
          name: 'pulls when the image is not local and exits 0',
          fn: async () => {
            const run = await runEntry({ environment: { FAKE_EXISTS_EXIT: '1', }, withPodman: true, },);
            expect(run.exitCode,).toBe(0,);
            expect(run.output,).toContain(`[prepare:playwright] pulled ${REFERENCE}`,);
            expect(run.calls,).toEqual([
              `image exists ${TAGGED}`,
              `pull ${REFERENCE}`,
              `tag ${REFERENCE} ${TAGGED}`,
              `tag ${REFERENCE} monochromatic-playwright`,
            ],);
          },
        },),
        it({
          name: 'does nothing but tag when the image of this content is already local',
          fn: async () => {
            const run = await runEntry({ environment: {}, withPodman: true, },);
            expect(run.exitCode,).toBe(0,);
            expect(run.output,).toContain(`[prepare:playwright] using ${TAGGED} (already local)`,);
            expect(run.calls,).toEqual([`image exists ${TAGGED}`, `tag ${TAGGED} monochromatic-playwright`,],);
          },
        },),
        it({
          name: 'falls back to a local build with the printed reason and exits 0',
          fn: async () => {
            const run = await runEntry({ environment: { FAKE_EXISTS_EXIT: '1', FAKE_PULL_EXIT: '125', }, withPodman: true, },);
            expect(run.exitCode,).toBe(0,);
            expect(run.output,).toContain('could not pull ghcr.io/aquaticats/monochromatic-playwright:',);
            expect(run.output,).toContain('Error: initializing source docker://x: unauthorized',);
            expect(run.output,).toContain('building playwright.Dockerfile locally',);
            expect(run.calls.at(-1,),).toBe(
              `build --tag ${TAGGED} --tag monochromatic-playwright --file playwright.Dockerfile .`,
            );
          },
        },),
        it({
          name: 'exits 1 and shows the build output when the pull and the build both fail',
          fn: async () => {
            const run = await runEntry({
              environment: { FAKE_EXISTS_EXIT: '1', FAKE_PULL_EXIT: '125', FAKE_BUILD_EXIT: '125', },
              withPodman: true,
            },);
            expect(run.exitCode,).toBe(1,);
            expect(run.output,).toContain('building playwright.Dockerfile locally failed',);
            expect(run.output,).toContain('step failed in build',);
          },
        },),
        it({
          name: 'exits 1 with a message that podman could not be started when podman is not on PATH',
          fn: async () => {
            const run = await runEntry({ environment: {}, withPodman: false, },);
            expect(run.exitCode,).toBe(1,);
            expect(run.output,).toContain('podman could not be started',);
          },
        },),
      ],
    },),
  ],
},);
