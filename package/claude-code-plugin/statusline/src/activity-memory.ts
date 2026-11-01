/**
 Per-session memory of the last activity word, so renders whose transcript tail holds no gerund keep the previous one.

 The statusline command is a fresh process on every render, so the memory lives on disk.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  extname,
  join,
} from 'node:path';

import { readActivityWord, } from './activity.ts';
import { findGerundInText, } from './gerund.ts';

//region Memory file

/**
 Directory under the Claude home that holds one remembered word per session.
 */
const MEMORY_DIRECTORY = 'statusline-activity';

/**
 Locates the memory file of the session that owns a transcript.

 @param transcriptPath - path to session transcript JSONL, whose file name is the session id

 @param home - user home directory; an empty value yields a relative path whose write simply fails

 @returns memory file path under the user's `~/.claude`

 @example
 ```ts
 memoryPathOf({
   transcriptPath: '/home/me/.claude/projects/x/3baeb2d2.jsonl',
   home: '/home/me',
 });
 ```
 */
function memoryPathOf({
  transcriptPath,
  home,
}: Readonly<{
  transcriptPath: string;
  home: string;
}>,): string {
  /**
   Session id: the transcript file name without its extension.
   */
  const sessionId = basename(
    transcriptPath,
    extname(transcriptPath,),
  );
  return join(
    home,
    '.claude',
    MEMORY_DIRECTORY,
    `${sessionId}.txt`,
  );
}

/**
 Reads the word remembered for a session.

 @param memoryPath - memory file path

 @returns remembered word, or empty string when absent, unreadable, or not exactly one word the extractor could have produced

 @example
 ```ts
 await readRememberedWord({ memoryPath: '/home/me/.claude/statusline-activity/3baeb2d2.txt' });
 ```
 */
async function readRememberedWord({
  memoryPath,
}: Readonly<{
  memoryPath: string;
}>,): Promise<string> {
  try {
    /**
     Stored text.
     */
    const stored = await readFile(
      memoryPath,
      'utf8',
    );
    /**
     Stored word without the trailing newline.
     */
    const word = stored.trim();
    // Re-running the extractor guards the terminal against edited or corrupt files.
    return findGerundInText(word,) === word
      ? word
      : '';
  }
  catch (_error: unknown) {
    return '';
  }
}

/**
 Stores the word for a session.

 @param memoryPath - memory file path

 @param word - word to remember

 @example
 ```ts
 await writeRememberedWord({ memoryPath: '/home/me/.claude/statusline-activity/3baeb2d2.txt', word: 'Testing' });
 ```
 */
async function writeRememberedWord({
  memoryPath,
  word,
}: Readonly<{
  memoryPath: string;
  word: string;
}>,): Promise<void> {
  try {
    await mkdir(
      dirname(memoryPath,),
      { recursive: true, },
    );
    await writeFile(
      memoryPath,
      `${word}\n`,
    );
  }
  catch (_error: unknown) {
    // A statusline must still render when the memory is not writable.
  }
}

//endregion Memory file

//region Public reader

/**
 Reads the activity word, falling back to the word extracted on an earlier render of the same session.

 A freshly extracted word replaces the remembered one; the file is only written when the word changed.

 @param transcriptPath - path to session transcript JSONL

 @param home - user home directory holding the memory; defaults to `HOME` so tests can inject a disposable home

 @returns capitalized activity word, or empty string when this session never produced one

 @example
 ```ts
 await readRememberedActivityWord({ transcriptPath: '/tmp/session.jsonl' });
 ```
 */
async function readRememberedActivityWord({
  transcriptPath,
  home,
}: Readonly<{
  transcriptPath: string;
  home?: string;
}>,): Promise<string> {
  /**
   Home from the environment, used when the caller injects none.
   */
  const { HOME: environmentHome = '', } = process.env;
  /**
   Memory file of this session.
   */
  const memoryPath = memoryPathOf({
    transcriptPath,
    home: home ?? environmentHome,
  },);
  /**
   Word extracted from the transcript tail on this render.
   */
  const fresh = await readActivityWord({ transcriptPath, },);
  /**
   Word kept from an earlier render.
   */
  const remembered = await readRememberedWord({ memoryPath, },);

  if (fresh.length === 0)
    return remembered;
  if (fresh !== remembered)
    await writeRememberedWord({
      memoryPath,
      word: fresh,
    },);
  return fresh;
}

//endregion Public reader

export {
  memoryPathOf,
  readRememberedActivityWord,
  readRememberedWord,
  writeRememberedWord,
};
