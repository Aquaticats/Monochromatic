/**
 The text of the issue the daily update opens for a package it broke, and of the comment that stands in for it.

 @module
 */

import { causeOf, } from './daily-update-first-error.ts';
import { reproduceCommand, } from './daily-update-package-check-discovery.ts';
import type {
  FirstError,
  LockMove,
  MoveSource,
  PackageCheck,
  RenderedIssue,
} from './daily-update-package-check-types.ts';

/**
 Lines of moved versions an issue lists.
 Measured on the first daily update (commit `d61dc1934a`): across 28 `Cargo.lock` files the median package had 52 moved
 crates and the largest three 206, 124 and 104. A hundred lines of about seventy characters stay far below the 65,536
 characters GitHub accepts, and the entries the first error names come first, so the cap cuts the unrelated ones.
 */
export const MAXIMUM_MOVE_LINES = 100;

/**
 The label every issue carries: nobody has evaluated an automatic report yet (`doc/agent/triage-labels.md`).
 */
export const REPORT_LABEL = 'needs-triage';

/**
 Words that appear in every report body, which is what a later run searches for before it compares markers.
 */
export const REPORT_SEARCH_PHRASE = 'toolchain daily update report';

/**
 Characters of a message shown in a title.
 */
const TITLE_MESSAGE_CHARACTERS = 60;

/**
 One moved version with the lock it moved in and whether the first error names it.
 */
type MoveEntry = {
  /**
   Lock the version moved in.
   */
  readonly source: string;
  /**
   The move.
   */
  readonly move: LockMove;
  /**
   Whether the first error names the crate or package.
   */
  readonly named: boolean;
};

/**
 Builds the hidden line that names a package, a check and a cause.
 Each field is followed by a fixed word, so the marker of one package is never found inside the marker of another
 whose path starts the same.

 @param package - package directory

 @param task - check task

 @param cause - cause, from `causeOf`

 @returns the marker

 @example
 ```ts
 issueMarker({ package: 'package/a/b', task: 'lint:clippy', cause: 'E0433', },);
 ```
 */
export function issueMarker({
  package: packageDirectory,
  task,
  cause,
}: {
  readonly package: string;
  readonly task: string;
  readonly cause: string;
},): string {
  return `<!-- toolchain-daily-update package=${packageDirectory} task=${task} cause=${cause} -->`;
}

/**
 Writes one moved version as a list line.

 @param entry - the move and whether the first error names it

 @returns the list line

 @example
 ```ts
 moveLine({ source: 'a', move: { name: 'zbus', from: '5', to: '6', }, named: false, },); // '- `zbus` 5 to 6'
 ```
 */
export function moveLine(entry: MoveEntry,): string {
  /**
   What happened to the version.
   */
  const what = entry.move
    .from
    === ''
    ? `added at ${entry.move
      .to}`
    : entry.move
      .to
      === ''
      ? `removed (was ${entry.move
        .from})`
      : `${entry.move
        .from} to ${entry.move
          .to}`;
  return `- \`${entry.move
    .name}\` ${what}${entry.named ? ' (named in the first error)' : ''}`;
}

/**
 Lists every move with its lock and whether the error names it, the named ones first.

 @param moves - moves per lock

 @param errorText - the first error's excerpt

 @returns the entries, named first and otherwise in the order given

 @example
 ```ts
 entriesOf({ moves: [], errorText: '', },); // []
 ```
 */
export function entriesOf({
  moves,
  errorText,
}: {
  readonly moves: readonly MoveSource[];
  readonly errorText: string;
},): readonly MoveEntry[] {
  return moves
    .flatMap(function flatten({
      source,
      moves: inSource,
    },): readonly MoveEntry[] {
      return inSource.map(function withSource(move,): MoveEntry {
        return {
          source,
          move,
          named: errorText.includes(`\`${move.name}\``,) || errorText.includes(`${move.name}::`,),
        };
      },);
    },)
    .toSorted(function namedFirst(
      left,
      right,
    ) {
      return Number(right.named,) - Number(left.named,);
    },);
}

/**
 Writes the section of the body that lists what moved.

 @param moves - moves per lock

 @param errorText - the first error's excerpt, which decides which moves are listed first

 @returns the section's lines

 @example
 ```ts
 movesSection({ moves: [], errorText: '', },); // says that no lock moved
 ```
 */
export function movesSection({
  moves,
  errorText,
}: {
  readonly moves: readonly MoveSource[];
  readonly errorText: string;
},): readonly string[] {
  /**
   Every move, the named ones first.
   */
  const entries = entriesOf({
    moves,
    errorText,
  },);
  if (entries.length === 0) {
    return [
      'No lock of this package moved in the update, so the cause is outside the package\'s locked versions:',
      'a tool or compiler change, or another commit in the same window.',
    ];
  }
  /**
   The moves that fit the cap.
   */
  const shown = entries.slice(
    0,
    MAXIMUM_MOVE_LINES,
  );
  /**
   One block of lines per lock that has a shown move, then the note about moves left out.
   */
  const blocks: (readonly string[])[] = moves.flatMap(function blockOf({ source, },) {
    /**
     Shown moves of this lock.
     */
    const inSource = shown.filter(function isOfSource(entry,) {
      return entry.source === source;
    },);
    return inSource.length === 0 ? [] : [[
      `### ${source}`,
      '',
      ...inSource.map(moveLine,),
    ],];
  },);
  if (entries.length > shown.length) {
    blocks.push([
      `and ${entries.length - shown.length} more moves in the locks; \`git diff\` between the commits lists them.`,
    ],);
  }
  // A blank line separates one block from the next, and none ends the section.
  return blocks.flatMap(function separated(
    block,
    position,
  ) {
    return position === 0 ? block : [
      '',
      ...block,
    ];
  },);
}

/**
 Short statement of a first error for a title.

 @param error - the first error

 @returns the code, or the start of the message

 @example
 ```ts
 titleCauseOf({ code: 'E0433', message: 'm', excerpt: '', },); // 'E0433'
 ```
 */
export function titleCauseOf(error: FirstError,): string {
  return error.code === '' ? error.message
    .slice(
    0,
    TITLE_MESSAGE_CHARACTERS,
    ) : error.code;
}

/**
 Writes the issue for a package the update broke.

 @param check - the check that newly fails

 @param firstError - its first error

 @param updateCommits - commits the update made, none when only the machine-global toolchain moved

 @param baselineCommit - the commit the repository was at before the update, where the check passed

 @param moves - what moved, per lock

 @returns the title, body, comment and labels

 @example
 ```ts
 const issue = renderIssue({ check, firstError, updateCommits: ['d61dc1934a',], baselineCommit: 'c0ffee0001', moves: [], },);
 ```
 */
export function renderIssue({
  check,
  firstError,
  updateCommits,
  baselineCommit,
  moves,
}: {
  readonly check: PackageCheck;
  readonly firstError: FirstError;
  readonly updateCommits: readonly string[];
  readonly baselineCommit: string;
  readonly moves: readonly MoveSource[];
},): RenderedIssue {
  /**
   The hidden line later runs search for.
   */
  const marker = issueMarker({
    package: check.package,
    task: check.task,
    cause: causeOf(firstError,),
  },);
  /**
   Commits of the update, written for a list line.
   */
  const commits = updateCommits.length === 0
    ? 'no commit: only the machine-global toolchain moved, and it lives outside the repository'
    : updateCommits.map(function quoted(commit,) {
      return `\`${commit}\``;
    },)
      .join(', ',);
  /**
   Words that say what the first error is.
   */
  const errorHeading = firstError.code === '' ? 'No error code' : `Code: \`${firstError.code}\``;
  /**
   Lines of the body.
   */
  const body = [
    `The daily toolchain and dependency update broke the check \`${check.task}\` of \`${check.package}\`:`,
    'it passed before the update and fails after it.',
    'The update is kept, and this issue is the fix-forward report',
    '(`doc/planning/toolchain-always-latest.md`, section "Checking every package after the update").',
    '',
    '## Update',
    '',
    `- Update commits: ${commits}.`,
    `- The repository was at \`${baselineCommit}\` when the check last passed, before the update.`,
    '',
    '## What moved',
    '',
    ...movesSection({
      moves,
      errorText: firstError.excerpt,
    },),
    '',
    '## First error',
    '',
    errorHeading,
    '',
    '```text',
    firstError.excerpt,
    '```',
    '',
    '## Reproduce',
    '',
    'From the repository root, on the update commit:',
    '',
    '```sh',
    reproduceCommand(check,),
    '```',
    '',
    `This is a ${REPORT_SEARCH_PHRASE}.`,
    'A check that was already failing before the update is never reported, and a package that stays broken adds no comment.',
    '',
    marker,
  ].join('\n',);
  /**
   Text of the comment for an open issue with the same marker.
   */
  const comment = [
    `The same check (\`${check.task}\`) broke again after a daily update, for the same cause (${titleCauseOf(firstError,)}).`,
    '',
    `- Update commits: ${commits}.`,
    `- The repository was at \`${baselineCommit}\` when the check last passed.`,
    '',
    '```text',
    firstError.excerpt,
    '```',
    '',
    `Reproduce: \`${reproduceCommand(check,)}\``,
  ].join('\n',);
  return {
    marker,
    title: `${check.package}: ${check.task} fails after the daily update (${titleCauseOf(firstError,)})`,
    body,
    comment,
    labels: [REPORT_LABEL,],
  };
}
