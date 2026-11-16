/**
 Reads the prose statement this machine's coordinator asks a review started another way to write: at the top of `review.md`, the
 range reviewed, how many of the brief's files were read in full ("N of M"), and what was built and run.

 That form says a count rather than each file's state, so a review that names none of the files it did not read whole is read as
 thin with those files unnamed: a second review reads every file the brief lists. The commands it lists state no exit code, so their
 exits are reported as uncomparable rather than compared with a code the review never gave.

 @module
 */

import {
  basename,
} from 'node:path';
import {
  NO_COUNTS,
  countsIn,
  isACommitHash,
} from './prose-numbers.ts';
import { pathOf, } from './git-quoted-path.ts';
import {
  REVIEW_STATEMENT_HEADING,
  type ReviewCommandRun,
  type ReviewFileRead,
  type ReviewStatement,
  ReviewStatementError,
} from './review-statement-shape.ts';

/**
 The mark a range puts between its two commits.
 */
const RANGE_MARK = '...';

/**
 The commands whose word at the front makes a backticked text a command rather than a file name.
 */
const COMMANDS: ReadonlySet<string> = new Set([
  'bash',
  'bun',
  'cargo',
  'cargo-nextest',
  'deno',
  'git',
  'mise',
  'nextest',
  'node',
  'npm',
  'pnpm',
  'python3',
  'rg',
  'rolldown',
  'sh',
  'systemd-run',
  'time',
  'timeout',
  'tsc',
  'yarn',
]);

/**
 How many lines after the coverage line are read for the files a review did not read whole.
 */
const UNREAD_WINDOW_LINES = 40;

/**
 How many lines of a "built and run" block are read for its commands.
 */
const COMMANDS_WINDOW_LINES = 40;

/**
 A prose review whose statement of coverage cannot be read.
 */
export class ReviewProseStatementError extends ReviewStatementError {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewProseStatementError';
}

/**
 Reads the backticked texts of one line.

 @param line - one line of a review

 @returns each text between two backticks, in order

 @example
 ```ts
 const texts = backtickedIn({ line: 'run `mise run :build` then `true`' });
 ```
 */
function backtickedIn({ line, }: { readonly line: string },): readonly string[] {
  /**
   The texts found so far.
   */
  const found: string[] = [];
  /**
   Where the next opening backtick is looked for.
   */
  const cursor = { at: 0, };
  for (;;) {
    /**
     The opening backtick.
     */
    const open = line.indexOf(
      '`',
      cursor.at,
    );
    if (open === (-1)) {
      return found;
    }
    /**
     The closing backtick.
     */
    const close = line.indexOf(
      '`',
      open + 1,
    );
    if (close === (-1)) {
      return found;
    }
    found.push(line.slice(
      open + 1,
      close,
    ),);
    cursor.at = close + 1;
  }
}

/**
 Says whether a backticked text is a command.

 @param text - the text between two backticks

 @returns whether its first word names a command

 @example
 ```ts
 const command = isACommand('mise run //package/a:build');
 ```
 */
function isACommand(text: string,): boolean {
  /**
   The text's words.
   */
  const words = text.trim()
    .split(' ',)
    .filter(function notEmpty(word,) {
      return word !== '';
    },);
  /**
   The first word's own name.
   */
  const first = basename(words[0] ?? '',);
  return COMMANDS.has(first,);
}

/**
 Reads the number of brief files a review says it read in full.

 @param lines - the review file's lines

 @returns the count read whole and the count given, or nothing when the review states neither

 @example
 ```ts
 const counts = coverageCountIn({ lines });
 ```
 */
function coverageCountIn({ lines, }: { readonly lines: readonly string[] },):
  | {
    readonly kind: 'the counts';
    readonly readWhole: number;
    readonly given: number;
    readonly at: number
  }
    | { readonly kind: 'no counts' } {
  for (const [at, line,] of lines.entries()) {
    /**
     The line's own text, in one case.
     */
    const lowered = line.toLowerCase();
    if ((!lowered.includes('read in full',)) && (!lowered.includes('read fully',))) {
      continue;
    }
    /**
     The counts the line states as "N of M".
     */
    const counted = countsIn({ line, },);
    if (counted.kind === 'the counts') {
      return {
        kind: 'the counts',
        readWhole: counted.readWhole,
        given: counted.given,
        at,
      };
    }
  }
  return NO_COUNTS;
}

/**
 Reads the commit a review says it reviewed.

 @param lines - the review file's lines

 @returns the commit, or empty text when no line names one

 @example
 ```ts
 const commit = commitIn({ lines });
 ```
 */
function commitIn({ lines, }: { readonly lines: readonly string[] },): string {
  for (const line of lines) {
    /**
     The line's own text, in one case.
     */
    const lowered = line.toLowerCase();
    if (
      (!lowered.includes('range reviewed',)) && (!lowered.includes('reviewed commit',))
        && (!lowered.includes('commit reviewed',))
        && (!lowered.includes('range:',))
    ) {
      continue;
    }
    for (const text of backtickedIn({ line, },)) {
      /**
       The commit after the last `...` of a range, which is the one reviewed.
       */
      const after = text.slice(text.lastIndexOf(RANGE_MARK,) + RANGE_MARK.length,);
      if (isACommitHash(after,)) {
        return after;
      }
    }
  }
  return '';
}

/**
 Reads the files a review names as read only in part, within the lines after its coverage line.

 @param lines - the review file's lines

 @param from - where the coverage line is

 @param briefFiles - the files the brief lists

 @returns the brief files the review names as not read whole

 @example
 ```ts
 const named = unreadNamesIn({ lines, from: 10, briefFiles });
 ```
 */
function unreadNamesIn({
  lines,
  from,
  briefFiles,
}: {
  readonly lines: readonly string[];
  readonly from: number;
  readonly briefFiles: readonly string[];
},): readonly string[] {
  /**
   The brief's files, for a path to be one of them.
   */
  const brief = new Set(briefFiles.map(function pathOfFile(file,) {
    return pathOf(file,);
  },),);
  /**
   The names found so far.
   */
  const found: string[] = [];
  /**
   Whether a block that names files read in part has started.
   */
  const inBlock = { open: false, };
  for (const line of lines.slice(
    from + 1,
    from + 1
      + UNREAD_WINDOW_LINES,
  )) {
    /**
     The line's own text, in one case.
     */
    const lowered = line.toLowerCase();
    if (lowered.includes('read in part',) || lowered.includes('not read',)
      || lowered.includes('read only',)
      || lowered.includes('read partially',)) {
      inBlock.open = true;
    }
    if (line.startsWith('## ',) && inBlock.open) {
      break;
    }
    if (!inBlock.open) {
      continue;
    }
    for (const text of backtickedIn({ line, },)) {
      /**
       The path the text names, when it names one.
       */
      const path = pathOf(text,);
      if (brief.has(path,) && (!found.includes(path,))) {
        found.push(path,);
      }
    }
  }
  return found;
}

/**
 Reads the commands a review lists as built and run.

 @param lines - the review file's lines

 @returns the commands, each once, with no exit code stated

 @example
 ```ts
 const commands = commandsIn({ lines });
 ```
 */
function commandsIn({ lines, }: { readonly lines: readonly string[] },): readonly ReviewCommandRun[] {
  /**
   The lines that state what was built and run, and the lines after the first of them.
   */
  const at = lines.findIndex(function isTheBuiltAndRunLine(line,) {
    /**
     The line's own text, in one case.
     */
    const lowered = line.toLowerCase();
    return lowered.includes('built and run',) || lowered.includes('built and ran',)
      || lowered.includes('built or run',)
      || lowered.includes('built or ran',)
      || lowered.includes('what i ran',);
  },);
  /**
   The lines to read for the commands.
   */
  const window = (at === (-1))
    ? lines.slice(
      0,
      Math.min(
        lines.length,
        COMMANDS_WINDOW_LINES,
      ),
    )
    : lines.slice(
      at,
      Math.min(
        lines.length,
        at + COMMANDS_WINDOW_LINES,
      ),
    );
  /**
   The commands found so far.
   */
  const found: ReviewCommandRun[] = [];
  for (const line of window) {
    if ((at !== (-1)) && (line !== lines[at])
      && line.startsWith('## ',)) {
      break;
    }
    for (const text of backtickedIn({ line, },)) {
      if (!isACommand(text,)) {
        continue;
      }
      /**
       The command as the review wrote it.
       */
      const command = text.trim();
      if (found.some(function isListed(one,) {
        return one.command === command;
      },)) {
        continue;
      }
      found.push({
        command,
        exit: 0,
        exitStated: false,
      },);
    }
  }
  return found;
}

/**
 Reads the statement a review wrote in the prose form the coordinator asks for.

 @param shown - the review file's text

 @param file - the review file's path, for the message

 @param briefFiles - the files the brief lists, from the top of the checkout

 @param diffLines - the lines of the diff file

 @param recordedCommit - the commit the review's record names, used when no line states one

 @returns the statement

 @throws ReviewStatementError when no line states how many brief files were read in full

 @example
 ```ts
 const statement = proseStatementIn({ shown, file: 'a/review.md', briefFiles: ['package/a/src/x.ts'], diffLines: 7, recordedCommit: 'abc1234' });
 ```
 */
export function proseStatementIn({
  shown,
  file,
  briefFiles,
  diffLines,
  recordedCommit,
}: {
  readonly shown: string;
  readonly file: string;
  readonly briefFiles: readonly string[];
  readonly diffLines: number;
  readonly recordedCommit: string;
},): ReviewStatement {
  /**
   The review file's lines, with line endings made uniform.
   */
  const lines = shown.replaceAll(
    '\r\n',
    '\n',
  )
    .replaceAll(
    '\r',
    '\n',
    )
    .split('\n',);
  /**
   The coverage the review states.
   */
  const coverage = coverageCountIn({ lines, },);
  if (coverage.kind === 'no counts') {
    throw new ReviewProseStatementError(
      `${file}: no line states how many of the brief's files the review read in full and it has no `
        + `"${REVIEW_STATEMENT_HEADING}" section, so what it read cannot be read`,
    );
  }
  /**
   The brief's files, each as the brief gave it.
   */
  const brief = briefFiles.map(function pathOfFile(path,) {
    return pathOf(path,);
  },);
  /**
   The files the review names as not read whole.
   */
  const named = unreadNamesIn({
    lines,
    from: coverage.at,
    briefFiles: brief,
  },);
  /**
   How many files the review says it did not read whole.
   */
  const missing = Math.max(
    0,
    coverage.given - coverage.readWhole,
  );
  /**
   Whether the review names none of the files it did not read whole.
   */
  const unnamedNotWhole = (named.length === 0) && (missing > 0);
  /**
   Each brief file's state, from the count and the names.
   */
  const files: ReviewFileRead[] = brief.map(function stateOf(path,) {
    return {
      path,
      read: named.includes(path,) ? 'part' : 'whole',
    };
  },);
  return {
    commit: commitIn({ lines, },) || recordedCommit,
    diff: {
      lines: diffLines,
      linesRead: diffLines,
    },
    files,
    commands: commandsIn({ lines, },),
    findings: [],
    prose: {
      filesReadWholeCount: coverage.readWhole,
      filesGivenCount: coverage.given,
      unnamedNotWhole,
    },
  };
}
