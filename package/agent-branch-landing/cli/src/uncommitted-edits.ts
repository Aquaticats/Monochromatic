/**
 The uncommitted edits a merge is refused over: the paths Git names as ones the merge would overwrite, and whether each edit is
 a reflow that can be committed as it stands or a change of words that stays where it is.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  fileAtRevision,
  type Repository,
} from './git-commands.ts';

/**
 What one uncommitted edit does to a file.
 */
export type KindOfUncommittedEdit =
  /**
   The same words in the same order, so a format run moved whitespace and nothing else.
   */
  | 'reflow'
  /**
   A word this file did not hold, or a file Git could not show, so the edit is somebody's work and stays.
   */
    | 'changed-words';

/**
 The characters that separate one word from another in a file's text.
 */
const WORD_SEPARATORS: readonly string[] = [
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '\v',
];

/**
 Reads the words of a text, which is how a reflow is told from a change.

 @param text - the text to read

 @returns its words, in order, without the whitespace between them

 @example
 ```ts
 const words = wordsOf('a  b\nc',);
 ```
 */
function wordsOf(text: string,): readonly string[] {
  return WORD_SEPARATORS.reduce(
    function foldOut(
      text_: string,
      separator: string,
    ) {
      return text_.replaceAll(
        separator,
        ' ',
      );
    },
    text,
  )
    .split(' ',)
    .filter(function isWord(word: string,) {
      return word !== '';
    },);
}

/**
 Lists the paths a refused merge names as ones it would overwrite, which is what Git prints after the sentence that says the
 merge would overwrite them.

 @param mergeOutput - what the refused merge printed

 @returns the paths, in the order Git named them

 @example
 ```ts
 const blocked = blockedFilesIn('error: ... would be overwritten by merge:\n\tdoc/a.md\n',);
 ```
 */
export function blockedFilesIn(mergeOutput: string,): readonly string[] {
  /**
   The merge's output, line by line.
   */
  const lines: readonly string[] = mergeOutput.split('\n',);
  /**
   Where Git names the files, or minus one when this refusal is not about files at all.
   */
  const start = lines.findIndex(function namesFiles(line: string,) {
    return line.includes('would be overwritten by merge',);
  },);
  if (start === (-1)) return [];
  return lines.slice(start + 1,)
    .filter(function isPath(line: string,) {
      return line.startsWith('\t',);
    },)
    .map(function trimmed(line: string,) {
      return line.trim();
    },);
}

/**
 What reading a file the main checkout holds gave.
 */
type WorkingText =
  | {
    /**
     Whether the file could be read.
     */
    readonly read: true;

    /**
     What it holds.
     */
    readonly text: string;
  }
    | {
    /**
     Whether the file could be read, which it could not.
     */
      readonly read: false;

    /**
     What the operating system said about it.
     */
      readonly why: string;
    };

/**
 Reads a file the main checkout holds.

 @param path - the file's path on disk

 @returns its text, or why it could not be read

 @example
 ```ts
 const working = await readWorkingText('/repository/doc/a.md',);
 ```
 */
async function readWorkingText(path: string,): Promise<WorkingText> {
  try {
    return {
      read: true,
      text: await readFile(
        path,
        'utf8',
      ),
    };
  }
  catch (caught) {
    // A file Git names that cannot be read has no working text to compare, so the edit reads as a change of words and stays
    // where it is.
    return {
      read: false,
      why: caughtValueText(caught,),
    };
  }
}

/**
 Decides what one uncommitted edit is.

 A reflowed comment gains comment markers, so comparing words calls it changed. A working file byte-identical to the branch's own
 version loses nothing when it is committed as it stands, so it counts as a reflow too.

 @param repository - the main checkout

 @param branch - the branch being merged in

 @param path - the path the merge would overwrite

 @returns the kind of edit, which decides whether the landing may commit it

 @example
 ```ts
 const kind = await kindOfUncommittedEdit({ repository, branch: 'fix/a', path: 'doc/a.md', },);
 ```
 */
export async function kindOfUncommittedEdit({
  repository,
  branch,
  path,
}: {
  readonly repository: Repository;
  readonly branch: string;
  readonly path: string;
},): Promise<KindOfUncommittedEdit> {
  /**
   What the file holds in the main checkout right now.
   */
  const working = await readWorkingText(join(
    repository.folder,
    path,
  ),);
  if (!working.read) return 'changed-words';
  /**
   What the last commit of the main checkout holds.
   */
  const committed = await fileAtRevision({
    repository,
    revision: 'HEAD',
    path,
  },);
  if (committed.status !== 0) return 'changed-words';
  if (working.text === committed.stdout) return 'reflow';
  /**
   The branch's own version of the file.
   */
  const incoming = await fileAtRevision({
    repository,
    revision: branch,
    path,
  },);
  if ((incoming.status === 0) && (working.text === incoming.stdout)) return 'reflow';
  /**
   The words of the committed file.
   */
  const committedWords = wordsOf(committed.stdout,);
  /**
   The words of the working file.
   */
  const workingWords = wordsOf(working.text,);
  return ((committedWords.length === workingWords.length)
    && committedWords.every(function isSameWord(
      word: string,
      index: number,
    ) {
      return word === workingWords[index];
    },))
    ? 'reflow'
    : 'changed-words';
}
