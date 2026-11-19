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
import { isDigitsOnly, } from './text-of-an-option.ts';

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
 The program that wrote a proof: its process number and the moment the kernel says that process started, which together name one
 process and cannot be confused with a later process that took the same number. A record without a maker is one an earlier version of
 this package wrote, and its maker can never be shown gone.
 */
export type ProofMaker = {
  /**
   The process's number.
   */
  readonly pid: number;

  /**
   The moment the kernel says that process started, as `/proc/<pid>/stat` prints it.
   */
  readonly startedAt: string;
};

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

  /**
   The process that wrote it, or nothing for a proof an earlier version of this package wrote.
   */
  readonly maker?: ProofMaker;
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
 Reads a process identity only when both fields could identify a maker.

 @param stored - value of the proof's maker field

 @returns validated maker field, or no field when identity cannot be recovered
 */
function makerIn(stored: unknown,): { readonly maker?: ProofMaker } {
  if (((typeof stored) !== 'object') || (stored === null)) {
    return {};
  }
  /**
   Untrusted fields from the recorded process identity.
   */
  const fields: Readonly<Record<string, unknown>> = { ...stored, };
  /**
   Process number and kernel start ticks, both required to recognise a maker.
   */
  const {
    pid,
    startedAt,
  } = fields;
  if (((typeof pid) !== 'number')
    || (!Number.isSafeInteger(pid,))
    || (pid <= 0)
    || ((typeof startedAt) !== 'string')
    || (!isDigitsOnly(startedAt,))) {
    return {};
  }
  return { maker: {
    pid,
    startedAt,
  }, };
}

/**
 Reads a stored proof when it has the shape this module writes.

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
      ...makerIn(fields.maker,),
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

/**
 Claims the proof beside a worktree, with one exclusive create, so exactly one start's making owns the path.

 A start that cannot create the file reads what is there and decides from it; it never writes over another making's proof, which is
 what let one start erase the record of the worktree another start had made.

 @param worktree - the worktree's path

 @param proof - what this making is

 @returns whether this start's claim is the one that won

 @example
 ```ts
 const claimed = await claimProof({ worktree: '/repo/.claude/worktrees/pi-a', proof, },);
 ```
 */
export async function claimProof({
  worktree,
  proof,
}: {
  readonly worktree: string;
  readonly proof: WorktreeProof;
},): Promise<boolean> {
  /**
   The file the making writes.
   */
  const file = worktreeProofFileOf({ worktree, },);
  await mkdir(
    dirname(file,),
    { recursive: true, },
  );
  try {
    await writeFile(
      file,
      `${JSON.stringify(
        proof,
        undefined,
        2,
      )}\n`,
      {
        encoding: 'utf8',
        flag: 'wx',
      },
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === 'EEXIST') {
      return false;
    }
    throw error;
  }
}

/**
 Moves a proof out of the way, so the making that read it can claim the path.

 Only one caller can move a given file: a second rename of the same file finds it gone. That is what makes taking over a proof whose
 maker is gone a decision one start makes.

 @param worktree - the worktree's path

 @returns whether this caller moved it

 @example
 ```ts
 const moved = await moveTheProofAway({ worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function moveTheProofAway({ worktree, }: { readonly worktree: string },): Promise<boolean> {
  /**
   The file the making wrote.
   */
  const file = worktreeProofFileOf({ worktree, },);
  try {
    await rename(
      file,
      `${file}.${String(process.pid,)}.given-up`,
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Position of start ticks in the fields after the PID and parenthesized process name.
 The kernel numbers starttime as field 22, with field 3 at index zero of this suffix.
 */
const PROC_STAT_START_TICKS_INDEX = 19;

/**
 Reads the moment a process started, out of the line `/proc/<pid>/stat` holds.

 The kernel writes the program's name in parentheses, where a name may hold spaces and parentheses of its own, so the fields are read
 after the last parenthesis.

 @param stat - the line `/proc/<pid>/stat` holds

 @returns the start moment as the kernel prints it, or nothing when the line is no stat line

 @example
 ```ts
 const startedAt = startMomentIn({ stat: '1234 (bash) S 1 1234 ... 98765 ...', },);
 ```
 */
export function startMomentIn({ stat, }: { readonly stat: string },): { readonly startedAt?: string } {
  /**
   Where the program's name ends, after which the kernel's fields begin.
   */
  const close = stat.lastIndexOf(')',);
  if (close === (-1)) {
    return {};
  }
  /**
   The fields after the name, of which the twentieth is the start moment: the third field is the state, so the twenty-second is
   nineteenth after the name.
   */
  const fields = stat.slice(close + 1,)
    .trim()
    .split(' ',);
  /**
   The start moment, as the kernel prints it.
   */
  const startedAt = fields.at(PROC_STAT_START_TICKS_INDEX,);
  return ((startedAt === undefined) || (!isDigitsOnly(startedAt,))) ? {} : { startedAt, };
}

/**
 Reads the process a proof names, as it is now.

 @param maker - the process a proof names

 @param processFolder - the folder holding one entry per process, `/proc` here

 @returns the maker unless its process is demonstrably absent or the start ticks differ

 @example
 ```ts
 const stillThere = await makerNow({ maker, processFolder: '/proc', },);
 ```
 */
export async function makerNow({
  maker,
  processFolder,
}: {
  readonly maker: ProofMaker;
  readonly processFolder: string;
}): Promise<{ readonly maker?: ProofMaker }> {
  try {
    /**
     The line the kernel holds for that process number.
     */
    const stat = await readFile(
      join(
        processFolder,
        String(maker.pid,),
        'stat',
      ),
      'utf8',
    );
    /**
     The moment that process started.
     */
    const { startedAt, } = startMomentIn({ stat, },);
    // An unreadable identity cannot prove the maker gone. Only a different identity or a missing PID can.
    return ((startedAt === undefined) || (startedAt === maker.startedAt)) ? { maker, } : {};
  }
  catch (error) {
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (errorCodeOf(error,) === 'ESRCH')) {
      return {};
    }
    throw error;
  }
}

/**
 Says whether the process a proof names is still the one that wrote it, which decides whether a making may still be under way.

 A proof an earlier version of this package wrote names no maker, and no maker that is not named can be shown gone: such a proof is
 never a licence to remove what stands at the path.

 @param proof - the proof that says a making did not finish

 @param processFolder - the folder holding one entry per process

 @returns whether the maker may still be making the worktree

 @example
 ```ts
 const mayStillBeMaking = await itsMakerMayStillBeMaking({ proof, processFolder: '/proc', },);
 ```
 */
export async function itsMakerMayStillBeMaking({
  proof,
  processFolder,
}: {
  readonly proof: WorktreeProof;
  readonly processFolder: string;
}): Promise<boolean> {
  if (proof.maker === undefined) {
    return true;
  }
  return (await makerNow({
    maker: proof.maker,
    processFolder,
  },)).maker !== undefined;
}

/**
 Reads the maker of this process, which a new proof names.

 @param processFolder - the folder holding one entry per process

 @returns this process, as a proof names it

 @throws Error when this process's own line cannot be read, which no start can work around

 @example
 ```ts
 const maker = await makerOfThisProcess({ processFolder: '/proc', },);
 ```
 */
export async function makerOfThisProcess({ processFolder, }: { readonly processFolder: string },): Promise<ProofMaker> {
  /**
   This process's own line, which the kernel holds for every running process.
   */
  const stat = await readFile(
    join(
      processFolder,
      String(process.pid,),
      'stat',
    ),
    'utf8',
  );
  /**
   The moment this process started.
   */
  const { startedAt, } = startMomentIn({ stat, },);
  if (startedAt === undefined) {
    throw new Error(`this process's own line in ${processFolder} names no start moment, so a proof cannot name the making it is: ${stat}`,);
  }
  return {
    pid: process.pid,
    startedAt,
  };
}
