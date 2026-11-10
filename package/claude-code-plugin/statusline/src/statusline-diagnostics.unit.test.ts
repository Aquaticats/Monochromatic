/**
 Diagnostics the statusline reports for content it cannot render.

 Claude Code renders this command's stdout as the status line itself, so a
 diagnostic has to reach stderr instead: each arm below makes one degraded read
 fail, then asserts the caught value arrived on stderr while stdout still carried
 exactly the rendered line and nothing else.

 Every arm runs the built command in a child process with a disposable HOME,
 because the command reads the session transcript, the remembered activity word,
 and the effort level from the home directory.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { spawnSync, } from 'node:child_process';
import {
  dirname,
  join,
} from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Built command under test, resolved against this file so the arm runs the same
 artifact a Claude Code session runs.
 */
const STATUSLINE_COMMAND = new URL(
  '../dist/final/node/statusline.mjs',
  import.meta.url,
);

/**
 Working directory for the child command, which also resolves the bare bundle
 path against the source directory.
 */
const SOURCE_DIRECTORY = new URL('.', import.meta.url,);

/**
 Payload whose model name carries the default version and context, so the
 rendered model segment is exactly `Sonnet`.
 */
const MODEL_ONLY_INPUT = {
  model: { display_name: 'Sonnet 4.6 (200K context)', },
} as const;

/**
 Rendered status line holding the model segment alone, which every arm that
 yields no activity word must print.
 */
const MODEL_LINE = 'Sonnet\n';

/**
 Status line for the session whose transcript names a gerund.
 */
const MODEL_AND_ACTIVITY_LINE = 'Sonnet    Compiling\n';

/**
 Assistant transcript line naming an activity word.
 */
const COMPILING_TRANSCRIPT = `${JSON.stringify({
  type: 'assistant',
  message: {
    content: [{ type: 'text', text: 'Compiling the statusline package.', },],
  },
},)}\n`;

/**
 Transcript tail that is not valid JSON, as a live session leaves it while the
 final line is still being written.
 */
const TRUNCATED_TRANSCRIPT = '{"type":"assistant","message":\n';

/**
 Settings text that cannot be parsed, which leaves the effort level unknown.
 */
const UNPARSABLE_SETTINGS = '{"effortLevel":\n';

/**
 Directory under the home that holds one remembered activity word per session.
 */
const MEMORY_DIRECTORY = join(
  '.claude',
  'statusline-activity',
);

/**
 Name of the file holding the effort level Claude Code was last told to use.
 */
const SETTINGS_FILE = join(
  '.claude',
  'settings.json',
);

/**
 Bound on one child run, so a command that never exits cannot stall the suite.
 */
const CHILD_TIMEOUT_MS = 60_000;

//endregion Fixtures

//region Helpers

/**
 Disposable home that removes itself when its `await using` scope ends.
 */
type TempHome = {
  /**
   Absolute path to the throwaway home.
   */
  readonly path: string;
  /**
   Removes the directory recursively.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a throwaway home.

 @returns disposable home handle
 */
async function makeHome(): Promise<TempHome> {
  /**
   Throwaway home directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'claude-statusline-diagnostics-',
  ),);

  return {
    path,
    [Symbol.asyncDispose]: async function removeHome(): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 One captured child run.

 @example
 ```ts
 const { stderr, stdout } = runStatusline({ home, input: MODEL_ONLY_INPUT });
 ```
 */
type StatuslineRun = {
  /**
   Diagnostic output of the run, where every caught value must appear.
   */
  readonly stderr: string;
  /**
   Rendered status line, which must never carry a diagnostic.
   */
  readonly stdout: string;
};

/**
 Writes the session transcript and returns its path.

 @param home - throwaway home holding the transcript

 @param text - transcript body

 @returns absolute transcript path
 */
async function writeTranscript({
  home,
  text,
}: Readonly<{
  home: string;
  text: string;
}>): Promise<string> {
  /**
   Transcript path handed to the command.
   */
  const transcriptPath = join(
    home,
    'session.jsonl',
  );
  await writeFile(
    transcriptPath,
    text,
  );
  return transcriptPath;
}

/**
 Writes the global Claude settings file.

 @param home - throwaway home holding the settings

 @param text - settings body
 */
async function writeSettings({
  home,
  text,
}: Readonly<{
  home: string;
  text: string;
}>): Promise<void> {
  await mkdir(
    join(
      home,
      '.claude',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      home,
      SETTINGS_FILE,
    ),
    text,
  );
}

/**
 Runs the built command against a throwaway home.

 `MONOCHROMATIC_VERBOSE=true` is what makes the statusline's ordinary degraded
 reads report at all: without it the logger keeps them silent, which is the point
 of an ordinary render.

 @param home - throwaway home the command reads its state from

 @param input - statusline payload written to the command's stdin

 @param verbose - whether the run carries `MONOCHROMATIC_VERBOSE=true`

 @returns captured stdout and stderr
 */
function runStatusline({
  home,
  input,
  verbose = true,
}: Readonly<{
  home: string;
  input: unknown;
  verbose?: boolean;
}>): StatuslineRun {
  /**
   Child run of the built command.
   */
  const result = spawnSync(
    process.execPath,
    ['../dist/final/node/statusline.mjs',],
    {
      cwd: SOURCE_DIRECTORY,
      encoding: 'utf8',
      env: {
        ...process.env,
        HOME: home,
        ...(verbose ? { MONOCHROMATIC_VERBOSE: 'true', } : {}),
      },
      input: JSON.stringify(input,),
      timeout: CHILD_TIMEOUT_MS,
    },
  );
  if (result.error !== undefined)
    throw result.error;
  return {
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

//endregion Helpers

await describe({
  name: 'statusline diagnostics',
  concurrency: 1,
  children: [
    describe({
      name: 'degraded reads',
      children: [
        it({
          name: 'reports settings JSON that cannot be parsed and still prints the line',
          fn: async function testUnparsableSettings(): Promise<void> {
            await using home = await makeHome();
            await writeSettings({
              home: home.path,
              text: UNPARSABLE_SETTINGS,
            },);

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: MODEL_ONLY_INPUT,
            },);

            expect(stderr,).toContain('settings JSON is unparsable',);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'reports a settings file that cannot be read and still prints the line',
          fn: async function testUnreadableSettings(): Promise<void> {
            await using home = await makeHome();

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: MODEL_ONLY_INPUT,
            },);

            expect(stderr,).toContain(`effort level unreadable from ${join(
              home.path,
              SETTINGS_FILE,
            )}`,);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'reports a transcript line that is not JSON and still prints the line',
          fn: async function testTruncatedTranscriptLine(): Promise<void> {
            await using home = await makeHome();
            const transcriptPath = await writeTranscript({
              home: home.path,
              text: TRUNCATED_TRANSCRIPT,
            },);

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: {
                ...MODEL_ONLY_INPUT,
                transcript_path: transcriptPath,
              },
            },);

            expect(stderr,).toContain('transcript line is not JSON',);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'reports a transcript that cannot be read and still prints the line',
          fn: async function testUnreadableTranscript(): Promise<void> {
            await using home = await makeHome();
            /**
             Transcript path that no file occupies.
             */
            const transcriptPath = join(
              home.path,
              'absent-session.jsonl',
            );

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: {
                ...MODEL_ONLY_INPUT,
                transcript_path: transcriptPath,
              },
            },);

            expect(stderr,).toContain(`activity word unavailable because ${transcriptPath}`,);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'reports a remembered word that cannot be read and still prints the line',
          fn: async function testUnreadableRememberedWord(): Promise<void> {
            await using home = await makeHome();
            const transcriptPath = await writeTranscript({
              home: home.path,
              text: '',
            },);
            /**
             Memory path of this session, occupied by a directory so the read fails
             with a reason other than absence.
             */
            const memoryPath = join(
              home.path,
              MEMORY_DIRECTORY,
              'session.txt',
            );
            await mkdir(
              memoryPath,
              { recursive: true, },
            );

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: {
                ...MODEL_ONLY_INPUT,
                transcript_path: transcriptPath,
              },
            },);

            expect(stderr,).toContain(`remembered activity word unreadable from ${memoryPath}`,);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'reports a remembered word that cannot be stored and still prints the word',
          fn: async function testUnwritableMemory(): Promise<void> {
            await using home = await makeHome();
            const transcriptPath = await writeTranscript({
              home: home.path,
              text: COMPILING_TRANSCRIPT,
            },);
            /**
             Memory directory path, occupied by a regular file so creating the
             directory fails.
             */
            const memoryPath = join(
              home.path,
              MEMORY_DIRECTORY,
              'session.txt',
            );
            await mkdir(
              join(
                home.path,
                '.claude',
              ),
              { recursive: true, },
            );
            await writeFile(
              join(
                home.path,
                MEMORY_DIRECTORY,
              ),
              '',
            );

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: {
                ...MODEL_ONLY_INPUT,
                transcript_path: transcriptPath,
              },
            },);

            expect(stderr,).toContain(`remembered activity word not persisted to ${memoryPath}`,);
            expect(stdout,).toBe(MODEL_AND_ACTIVITY_LINE,);
          },
        },),
      ],
    },),
    describe({
      name: 'reported levels',
      children: [
        it({
          name: 'reports durable corruption without the verbose flag',
          fn: async function testWarnNeedsNoVerbose(): Promise<void> {
            await using home = await makeHome();
            await writeSettings({
              home: home.path,
              text: UNPARSABLE_SETTINGS,
            },);

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: MODEL_ONLY_INPUT,
              verbose: false,
            },);

            expect(stderr,).toContain('settings JSON is unparsable',);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'keeps an absent settings file quiet without the verbose flag',
          fn: async function testAbsentFileNeedsNoReport(): Promise<void> {
            await using home = await makeHome();

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: MODEL_ONLY_INPUT,
              verbose: false,
            },);

            expect(stderr,).toBe('',);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
        it({
          name: 'reports a refused permission without the verbose flag',
          fn: async function testRefusedPermissionWarns(): Promise<void> {
            await using home = await makeHome();
            const transcriptPath = await writeTranscript({
              home: home.path,
              text: '',
            },);
            /**
             Memory path of this session, occupied by a directory so the read
             fails with a reason other than absence.
             */
            const memoryPath = join(
              home.path,
              MEMORY_DIRECTORY,
              'session.txt',
            );
            await mkdir(
              memoryPath,
              { recursive: true, },
            );

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: {
                ...MODEL_ONLY_INPUT,
                transcript_path: transcriptPath,
              },
              verbose: false,
            },);

            expect(stderr,).toContain(`remembered activity word unreadable from ${memoryPath}`,);
            expect(stdout,).toBe(MODEL_LINE,);
          },
        },),
      ],
    },),
    describe({
      name: 'ordinary render',
      children: [
        it({
          name: 'stays silent when every read succeeds',
          fn: async function testQuietRender(): Promise<void> {
            await using home = await makeHome();
            await writeSettings({
              home: home.path,
              text: JSON.stringify({ effortLevel: 'high', },),
            },);
            const transcriptPath = await writeTranscript({
              home: home.path,
              text: COMPILING_TRANSCRIPT,
            },);
            /**
             Memory path of this session, holding the word the transcript yields,
             so this render reads and stores nothing.
             */
            const memoryPath = join(
              home.path,
              MEMORY_DIRECTORY,
              'session.txt',
            );
            await mkdir(
              dirname(memoryPath,),
              { recursive: true, },
            );
            await writeFile(
              memoryPath,
              'Compiling\n',
            );

            const { stderr, stdout, } = runStatusline({
              home: home.path,
              input: {
                ...MODEL_ONLY_INPUT,
                transcript_path: transcriptPath,
              },
            },);

            expect(stderr,).toBe('',);
            expect(stdout,).toBe(MODEL_AND_ACTIVITY_LINE,);
          },
        },),
      ],
    },),
  ],
},);
