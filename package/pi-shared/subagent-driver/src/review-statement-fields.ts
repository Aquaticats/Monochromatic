/**
 Reads one statement block into the shape a review's statement must hold, field by field: each value's type, each allowed word,
 and each field the shape does not carry, refused with the file and the field named.

 A field outside the shape is refused rather than dropped, and a block that copies the skeleton without replacing its values is
 answered with the field that still holds a placeholder rather than a statement nobody wrote.

 @module
 */

import {
  type FileReadState,
  type FindingShownBy,
  type ReviewCommandRun,
  type ReviewFileRead,
  type ReviewFinding,
  type ReviewStatement,
  ReviewStatementError,
} from './review-statement-shape.ts';
import {
  objectIn,
  objectsIn,
  parsedJsonOf,
  refuseUnreadFields,
  reviewStatementErrorOf,
  textIn,
  wholeNumberIn,
} from './review-statement-values.ts';

/**
 The fields the statement's top level carries, which a field outside refuses rather than drops.
 */
const STATEMENT_FIELDS: readonly string[] = [
  'commit',
  'diff',
  'files',
  'commands',
  'findings',
];

/**
 The fields one file entry carries.
 */
const FILE_FIELDS: readonly string[] = [
  'path',
  'read',
];

/**
 The fields one command entry carries.
 */
const COMMAND_FIELDS: readonly string[] = [
  'command',
  'exit',
  'ranBuiltProgram',
];

/**
 The fields one finding entry carries.
 */
const FINDING_FIELDS: readonly string[] = [
  'finding',
  'shownBy',
];

/**
 The fields the diff entry carries.
 */
const DIFF_FIELDS: readonly string[] = [
  'lines',
  'linesRead',
];

/**
 How a review may say it read one file.
 */
const FILE_READ_STATES: readonly string[] = [
  'whole',
  'part',
  'none',
];

/**
 How a review may say one finding was shown.
 */
const FINDING_SHOWN_BY: readonly string[] = [
  'run',
  'reading',
];

/**
 Reads how a review says it read one file, from the word the statement gives.

 @param read - the word the statement gives, which the caller already found in the allowed list

 @returns the word as the type the shape carries

 @example
 ```ts
 const state = fileReadStateOf({ read: 'part' });
 ```
 */
function fileReadStateOf({ read, }: { readonly read: string },): FileReadState {
  return ((read === 'part') || (read === 'none')) ? read : 'whole';
}

/**
 Reads how a review says one finding was shown, from the word the statement gives.

 @param shownBy - the word the statement gives, which the caller already found in the allowed list

 @returns the word as the type the shape carries

 @example
 ```ts
 const shownBy = findingShownByOf({ shownBy: 'run' });
 ```
 */
function findingShownByOf({ shownBy, }: { readonly shownBy: string },): FindingShownBy {
  return (shownBy === 'run') ? 'run' : 'reading';
}

/**
 Reads one file entry.

 @param entry - the entry as the reviewer wrote it

 @param at - its place in the list, from one

 @param file - the review file, for the message

 @returns how the review read that file

 @throws ReviewStatementError when the entry does not fit

 @example
 ```ts
 const file = fileReadIn({ entry, at: 1, file: 'a/review.md' });
 ```
 */
function fileReadIn({
  entry,
  at,
  file,
}: {
  readonly entry: Readonly<Record<string, unknown>>;
  readonly at: number;
  readonly file: string;
},): ReviewFileRead {
  /**
   The entry's path, for the message.
   */
  const where = `files entry ${String(at,)}`;
  refuseUnreadFields({
    held: entry,
    read: FILE_FIELDS,
    where,
    file,
  },);
  /**
   How the review read the file, as the entry says.
   */
  const read = textIn({
    value: entry.read,
    where: `${where}.read`,
    file,
  },);
  if (!FILE_READ_STATES.includes(read,)) {
    throw reviewStatementErrorOf({
      file,
      message: `${where}.read is "${read}", which is not one of ${FILE_READ_STATES.join(', ')}`,
    });
  }
  return {
    path: textIn({
      value: entry.path,
      where: `${where}.path`,
      file,
    },),
    read: fileReadStateOf({ read, },),
  };
}

/**
 Reads one command entry.

 @param entry - the entry as the reviewer wrote it

 @param at - its place in the list, from one

 @param file - the review file, for the message

 @returns one command the review ran

 @throws ReviewStatementError when the entry does not fit

 @example
 ```ts
 const command = commandRunIn({ entry, at: 1, file: 'a/review.md' });
 ```
 */
function commandRunIn({
  entry,
  at,
  file,
}: {
  readonly entry: Readonly<Record<string, unknown>>;
  readonly at: number;
  readonly file: string;
},): ReviewCommandRun {
  /**
   The entry's path, for the message.
   */
  const where = `commands entry ${String(at,)}`;
  refuseUnreadFields({
    held: entry,
    read: COMMAND_FIELDS,
    where,
    file,
  },);
  return {
    command: textIn({
      value: entry.command,
      where: `${where}.command`,
      file,
    },),
    exit: wholeNumberIn({
      value: entry.exit,
      where: `${where}.exit`,
      file,
    },),
    ...((entry.ranBuiltProgram === undefined)
      ? {}
      : {
        ranBuiltProgram: textIn({
          value: entry.ranBuiltProgram,
          where: `${where}.ranBuiltProgram`,
          file,
        },),
      }),
  };
}

/**
 Reads one finding entry.

 @param entry - the entry as the reviewer wrote it

 @param at - its place in the list, from one

 @param file - the review file, for the message

 @returns one finding the review reports

 @throws ReviewStatementError when the entry does not fit

 @example
 ```ts
 const finding = findingIn({ entry, at: 1, file: 'a/review.md' });
 ```
 */
function findingIn({
  entry,
  at,
  file,
}: {
  readonly entry: Readonly<Record<string, unknown>>;
  readonly at: number;
  readonly file: string;
},): ReviewFinding {
  /**
   The entry's path, for the message.
   */
  const where = `findings entry ${String(at,)}`;
  refuseUnreadFields({
    held: entry,
    read: FINDING_FIELDS,
    where,
    file,
  },);
  /**
   How the review says the finding was shown, as the entry says.
   */
  const shownBy = textIn({
    value: entry.shownBy,
    where: `${where}.shownBy`,
    file,
  },);
  if (!FINDING_SHOWN_BY.includes(shownBy,)) {
    throw reviewStatementErrorOf({
      file,
      message: `${where}.shownBy is "${shownBy}", which is not one of ${FINDING_SHOWN_BY.join(', ')}`,
    });
  }
  return {
    finding: textIn({
      value: entry.finding,
      where: `${where}.finding`,
      file,
    },),
    shownBy: findingShownByOf({ shownBy, },),
  };
}

/**
 Reads the JSON block a statement section holds as the review's statement.

 @param block - the text inside the fenced block

 @param file - the review file, for the message

 @returns the statement

 @throws ReviewStatementError when the block's shape does not fit

 @example
 ```ts
 const statement = statementOf({ block: '{"commit":"abc"}', file: 'a/review.md' });
 ```
 */
export function statementOf({
  block,
  file,
}: {
  readonly block: string;
  readonly file: string;
},): ReviewStatement {
  /**
   The statement's top level, read from the block.
   */
  const top = objectIn({
    value: parsedJsonOf({
      block,
      file,
    },),
    where: 'the statement',
    file,
  },);
  refuseUnreadFields({
    held: top,
    read: STATEMENT_FIELDS,
    where: 'the statement',
    file,
  },);
  /**
   The diff entry.
   */
  const diff = objectIn({
    value: top.diff,
    where: 'diff',
    file,
  },);
  refuseUnreadFields({
    held: diff,
    read: DIFF_FIELDS,
    where: 'diff',
    file,
  },);
  /**
   The lines of the diff in total, and the lines of it the review read.
   */
  const lines = wholeNumberIn({
    value: diff.lines,
    where: 'diff.lines',
    file,
  },);
  /**
   The lines of the diff the review says it read.
   */
  const linesRead = wholeNumberIn({
    value: diff.linesRead,
    where: 'diff.linesRead',
    file,
  },);
  if (linesRead > lines) {
    throw reviewStatementErrorOf({
      file,
      message: `diff.linesRead says ${String(linesRead,)} lines read of a diff of ${String(lines,)} lines`,
    });
  }
  /**
   The files the review says how it read.
   */
  const files = objectsIn({
    value: top.files,
    where: 'files',
    file,
  },)
    .map(function asAFileRead(
      entry,
      at
    ) {
      return fileReadIn({
        entry,
        at: at + 1,
        file,
      },);
    },);
  /**
   The paths already named, so a file repeated is refused rather than read twice.
   */
  const named = new Set<string>();
  for (const fileRead of files) {
    if (named.has(fileRead.path,)) {
      throw reviewStatementErrorOf({
        file,
        message: `files names ${fileRead.path} more than once, so how it was read cannot be read`,
      });
    }
    named.add(fileRead.path,);
  }
  return {
    commit: textIn({
      value: top.commit,
      where: 'commit',
      file,
    },),
    diff: {
      lines,
      linesRead,
    },
    files,
    commands: objectsIn({
      value: top.commands,
      where: 'commands',
      file,
    },)
      .map(function asACommandRun(
        entry,
        at
      ) {
        return commandRunIn({
          entry,
          at: at + 1,
          file,
        },);
      },),
    findings: objectsIn({
      value: top.findings,
      where: 'findings',
      file,
    },)
      .map(function asAFinding(
        entry,
        at
      ) {
        return findingIn({
          entry,
          at: at + 1,
          file,
        },);
      },),
  };
}
