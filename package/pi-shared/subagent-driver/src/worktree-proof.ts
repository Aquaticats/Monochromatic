/**
 The proof a making writes beside the worktree it makes: which run's claim made it, for which branch, whether its making finished,
 and when it was last written.

 A worktree is this package's to use only on that proof. A path that holds a `.git` and a clean status proves nothing about who made
 it, so a start that took it on that would work in another start's files, or in what a killed making left; the proof is what says
 both, and it is written before git is asked for the worktree and again once git has finished, so a start killed at any instruction
 leaves a proof saying the making did not finish.

 @module
 */

import {
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end of the file a making writes beside the worktree, which says which run folder made it, on which branch, and whether its
 making finished. The dot at its start keeps every reader that lists the folder's `pi-` entries from taking it for a worktree.
 */
const WORKTREE_PROOF_FILE_END = '.worktree.json';

/**
 The states a worktree's proof can be in: one whose making is under way or ended without finishing, and one whose making finished.
 */
const WORKTREE_PROOF_STATES: ReadonlySet<string> = new Set<string>([
  'making',
  'made',
]);


/**
 What a making writes beside its worktree: which run folder's claim made it, for which branch, whether the making finished, and when
 it was last written.
 */
export type WorktreeProof = {
  /**
   The worktree's path this proof is about.
   */
  readonly path: string;

  /**
   The run's name, which is the run folder's name without its number.
   */
  readonly runName: string;

  /**
   The number-carrying name of the run folder whose claim made this worktree.
   */
  readonly runFolder: string;

  /**
   The branch the worktree was made on, which is the caller's name for the work.
   */
  readonly branch: string;

  /**
   Whether the making finished.
   */
  readonly state: 'making' | 'made';

  /**
   When this proof was last written, as an ISO 8601 instant.
   */
  readonly writtenAt: string;
};

/**
 Reads the file a making writes beside its worktree.

 @param worktree - the worktree's path

 @returns the file's path

 @example
 ```ts
 const file = worktreeProofFileOf({ worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export function worktreeProofFileOf({
  worktree,
}: {
  readonly worktree: string;
},): string {
  return join(
    dirname(worktree,),
    `.${basename(worktree,)}${WORKTREE_PROOF_FILE_END}`,
  );
}

/**
 Reads what one field of a stored proof holds, when it holds the shape this module writes.

 @param stored - the proof as it was read, of any shape

 @returns the proof, or nothing when the stored value is not one

 @example
 ```ts
 const { proof } = proofIn({ stored, },);
 ```
 */
function proofIn({ stored, }: { readonly stored: unknown },): { readonly proof?: WorktreeProof } {
  if (((typeof stored) !== 'object') || (stored === null)) {
    return {};
  }
  /**
   The stored value, as fields this function reads by name.
   */
  const fields: Record<string, unknown> = { ...stored, };
  /**
   Reads one of those fields, which is empty text when it holds anything but text.
   
   @param name - the field's name
   
   @returns the text the field holds, or empty text
   */
  function textAt(name: string,): string {
    /**
     The field's value.
     */
    const value = fields[name];
    return ((typeof value) === 'string') ? value : '';
  }
  /**
   What the proof says, field by field.
   */
  const path = textAt('path',);
  /**
   The run's name.
   */
  const runName = textAt('runName',);
  /**
   The run folder whose claim made the worktree.
   */
  const runFolder = textAt('runFolder',);
  /**
   The branch the worktree is on.
   */
  const branch = textAt('branch',);
  /**
   Whether the making finished.
   */
  const state = textAt('state',);
  /**
   When the proof was last written.
   */
  const writtenAt = textAt('writtenAt',);
  if ((path === '') || (runName === '')
    || (runFolder === '')
    || (branch === '')
    || (!WORKTREE_PROOF_STATES.has(state,))
    || (writtenAt === '')) {
    return {};
  }
  return {
    proof: {
      path,
      runName,
      runFolder,
      branch,
      state: (state === 'made') ? 'made' : 'making',
      writtenAt,
    },
  };
}

/**
 Reads the proof beside a worktree, or nothing when there is none or it cannot be read as one.

 @param worktree - the worktree's path

 @returns the proof, and the problem a file that is there but is no proof gives

 @example
 ```ts
 const { proof } = await proofOf({ worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function proofOf({ worktree, }: { readonly worktree: string },): Promise<{
  readonly proof?: WorktreeProof;
  readonly problem?: string
}> {
  /**
   The file the making writes.
   */
  const file = worktreeProofFileOf({ worktree, },);
  try {
    return proofIn({ stored: JSON.parse(await readFile(
      file,
      'utf8',
    ),), },);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {};
    }
    // A proof nobody can read tells this start nothing, which is the same as no proof at all: it names the file a reader has to
    // look at, and nothing is removed because of it.
    return { problem: `${file} cannot be read as the proof of a worktree: ${String(error,)}`, };
  }
}

/**
 Writes the proof beside a worktree in one step, so a reader never reads half of one.

 @param worktree - the worktree's path

 @param proof - what the making did

 @example
 ```ts
 await writeProof({ worktree: '/repo/.claude/worktrees/pi-a', proof, },);
 ```
 */
export async function writeProof({
  worktree,
  proof,
}: {
  readonly worktree: string;
  readonly proof: WorktreeProof;
},): Promise<void> {
  /**
   The file the making writes.
   */
  const file = worktreeProofFileOf({ worktree, },);
  // The worktrees folder is made by the first making, which has not run when the first proof is written.
  await mkdir(
    dirname(file,),
    { recursive: true, },
  );
  /**
   The name this write lands under before it is renamed over the last proof.
   */
  const written = `${file}.${String(process.pid,)}.writing`;
  await writeFile(
    written,
    `${JSON.stringify(
      proof,
      undefined,
      2,
    )}\n`,
    'utf8',
  );
  await rename(
    written,
    file,
  );
}
