import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  memoryPathOf,
  readRememberedActivityWord,
} from '../dist/final/node/statusline.mjs';

/**
 Throwaway home directory that removes itself at the end of an `await using` scope.

 Tests pass it as the explicit `home` because sibling tests run concurrently and share `process.env`.
 */
type TempHome = {
  /**
   Absolute path to the throwaway home, which also holds the transcripts.
   */
  readonly path: string;
  /**
   Removes the directory.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a throwaway home.

 @returns disposable home handle

 @example
 ```ts
 await using home = await makeHome();
 ```
 */
async function makeHome(): Promise<TempHome> {
  /**
   Throwaway directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'claude-statusline-memory-',
  ),);

  return {
    path,
    [Symbol.asyncDispose]: async function removeHome(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Writes a transcript holding one assistant text entry, or one user entry when there is no prose.

 @param home - throwaway home directory

 @param sessionId - transcript file name without extension

 @param text - assistant prose; omitted for a transcript without assistant entries

 @returns transcript path

 @example
 ```ts
 await writeTranscript({ home: '/tmp/h', sessionId: 'a', text: 'Compiling' });
 ```
 */
async function writeTranscript({
  home,
  sessionId,
  text,
}: Readonly<{
  home: string;
  sessionId: string;
  text?: string;
}>,): Promise<string> {
  /**
   Transcript path fixture.
   */
  const transcriptPath = join(
    home,
    `${sessionId}.jsonl`,
  );
  /**
   Entry written for the transcript.
   */
  const entry = text === undefined
    ? { type: 'user', }
    : {
      type: 'assistant',
      message: { content: [{ type: 'text', text, },], },
    };
  await writeFile(
    transcriptPath,
    `${JSON.stringify(entry,)}\n`,
  );
  return transcriptPath;
}

await describe({
  name: readRememberedActivityWord.name,
  children: [
    it({
      name: 'returns and persists a freshly extracted word',
      fn: async function testPersists(): Promise<void> {
        await using home = await makeHome();
        /**
         Transcript with a gerund.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-a',
          text: 'Now compiling',
        },);

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('Compiling',);
        expect(await readFile(
          memoryPathOf({
            transcriptPath,
            home: home.path,
          },),
          'utf8',
        ),).toBe('Compiling\n',);
      },
    },),
    it({
      name: 'falls back to the earlier word when the tail has no gerund',
      fn: async function testFallsBack(): Promise<void> {
        await using home = await makeHome();
        await readRememberedActivityWord({
          transcriptPath: await writeTranscript({
            home: home.path,
            sessionId: 'session-a',
            text: 'Now compiling',
          },),
          home: home.path,
        },);
        /**
         Same session whose tail now holds no gerund.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-a',
          text: 'Done here',
        },);

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('Compiling',);
      },
    },),
    it({
      name: 'replaces the remembered word when a new one is extracted',
      fn: async function testReplaces(): Promise<void> {
        await using home = await makeHome();
        await readRememberedActivityWord({
          transcriptPath: await writeTranscript({
            home: home.path,
            sessionId: 'session-a',
            text: 'Now compiling',
          },),
          home: home.path,
        },);
        await readRememberedActivityWord({
          transcriptPath: await writeTranscript({
            home: home.path,
            sessionId: 'session-a',
            text: 'Now testing',
          },),
          home: home.path,
        },);
        /**
         Same session whose tail now holds no assistant prose.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-a',
        },);

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('Testing',);
      },
    },),
    it({
      name: 'returns empty when the session never produced a word',
      fn: async function testNeverProduced(): Promise<void> {
        await using home = await makeHome();
        /**
         Transcript without any gerund.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-a',
        },);

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('',);
      },
    },),
    it({
      name: 'keeps sessions apart',
      fn: async function testSessionsApart(): Promise<void> {
        await using home = await makeHome();
        await readRememberedActivityWord({
          transcriptPath: await writeTranscript({
            home: home.path,
            sessionId: 'session-a',
            text: 'Now compiling',
          },),
          home: home.path,
        },);
        /**
         Different session without a gerund.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-b',
        },);

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('',);
      },
    },),
    it({
      name: 'ignores a memory file that is not a single extractor word',
      fn: async function testCorruptMemory(): Promise<void> {
        await using home = await makeHome();
        /**
         Transcript without any gerund.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-a',
        },);
        /**
         Memory file location of the session.
         */
        const memoryPath = memoryPathOf({
          transcriptPath,
          home: home.path,
        },);
        await mkdir(
          dirname(memoryPath,),
          { recursive: true, },
        );
        await writeFile(
          memoryPath,
          '\u001B[31mCompiling\n',
        );

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('',);
      },
    },),
    it({
      name: 'still returns the fresh word when the memory cannot be written',
      fn: async function testUnwritableMemory(): Promise<void> {
        await using home = await makeHome();
        /**
         Transcript with a gerund.
         */
        const transcriptPath = await writeTranscript({
          home: home.path,
          sessionId: 'session-a',
          text: 'Now compiling',
        },);
        // A file where the memory directory belongs makes the directory creation fail.
        await mkdir(join(
          home.path,
          '.claude',
        ),);
        await writeFile(
          join(
            home.path,
            '.claude',
            'statusline-activity',
          ),
          '',
        );

        expect(await readRememberedActivityWord({
          transcriptPath,
          home: home.path,
        },),).toBe('Compiling',);
      },
    },),
  ],
},);
