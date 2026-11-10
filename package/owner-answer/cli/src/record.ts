/**
 The text operations that record one block of answers in the questions file.

 Every operation is pure and line based,
 and a name that matches nothing throws before any byte is written.

 @module
 */

import {
  BULLET_CONTINUATION,
  SUBSECTION_HEADINGS,
  type Answers,
} from './answers.ts';
import { AnswersError, } from './errors.ts';

/**
 Lines to change, and the heading whose section goes.
 */
export type RemoveSectionRequest = {
  readonly lines: readonly string[];
  readonly heading: string;
};

/**
 Lines to change, and the bullet whose item goes.
 */
export type RemoveBulletRequest = {
  readonly lines: readonly string[];
  readonly bullet: string;
};

/**
 Lines to change, and the exact consecutive lines that go.
 */
export type RemoveRunRequest = {
  readonly lines: readonly string[];
  readonly run: readonly string[];
};

/**
 Lines to change, the heading to insert before, and the block to insert.
 */
export type InsertBeforeRequest = {
  readonly lines: readonly string[];
  readonly anchor: string;
  readonly block: readonly string[];
};

/**
 Where the block landed in the rewritten file.
 */
export type Insertion = {
  readonly lines: readonly string[];
  readonly at: number;
};

/**
 One file to rewrite from one answers block.
 */
export type RecordRequest = {
  readonly text: string;
  readonly answers: Answers;
};

/**
 What the rewrite removed and where it inserted.
 */
export type RecordResult = {
  readonly text: string;
  readonly removedSections: number;
  readonly removedBullets: number;
  readonly removedRuns: number;
  readonly insertedAtLine: number;
};

/**
 Drops one section, from its heading through the line before the next heading.
 
 @param lines - File lines.
 
 @param heading - Heading that starts the section, matched as a prefix.
 
 @returns The lines without that section.
 
 @throws {@link AnswersError} when no line starts with the heading, or no heading follows it.
 
 @example
 ```ts
 const kept = removeSection({ lines: ['## A', '### Q', '### R'], heading: '### Q' });
 ```
 */
export function removeSection({
  lines,
  heading,
}: RemoveSectionRequest,): readonly string[] {
  /**
   Line the section starts on.
   */
  const start = lines.findIndex(function startsSection(line,): boolean {
    return line.startsWith(heading,);
  },);
  if (start === (-1)) {
    throw new AnswersError(`No section starts with: ${heading}`,);
  }
  /**
   Line the next heading starts on, which ends the section.
   */
  const end = lines.findIndex(function startsNextHeading(
    line,
    index,
  ): boolean {
    return (index > start) && SUBSECTION_HEADINGS.some(function isHeading(prefix,): boolean {
      return line.startsWith(prefix,);
    },);
  },);
  if (end === (-1)) {
    throw new AnswersError(`No heading follows the section: ${heading}`,);
  }
  return [
    ...lines.slice(
      0,
      start,
    ),
    ...lines.slice(end,),
  ];
}

/**
 Drops one bullet with its indented continuation.
 
 @param lines - File lines.
 
 @param bullet - First line of the bullet, matched as a prefix.
 
 @returns The lines without that bullet.
 
 @throws {@link AnswersError} when no line starts with the bullet, or a line that ends it is not there.
 
 @example
 ```ts
 const kept = removeBullet({ lines: ['- a', '  more', '- b'], bullet: '- a' });
 ```
 */
export function removeBullet({
  lines,
  bullet,
}: RemoveBulletRequest,): readonly string[] {
  /**
   Line the bullet starts on.
   */
  const start = lines.findIndex(function startsBullet(line,): boolean {
    return line.startsWith(bullet,);
  },);
  if (start === (-1)) {
    throw new AnswersError(`No bullet starts with: ${bullet}`,);
  }
  /**
   First line after the bullet that does not continue it.
   */
  const end = lines.findIndex(function endsBullet(
    line,
    index,
  ): boolean {
    return (index > start) && (!line.startsWith(BULLET_CONTINUATION,));
  },);
  if (end === (-1)) {
    throw new AnswersError(`Nothing follows the bullet: ${bullet}`,);
  }
  return [
    ...lines.slice(
      0,
      start,
    ),
    ...lines.slice(end,),
  ];
}

/**
 Drops one run of exact consecutive lines.
 
 @param lines - File lines.
 
 @param run - Lines that must appear in order, byte for byte.
 
 @returns The lines without that run.
 
 @throws {@link AnswersError} when the run does not appear.
 
 @example
 ```ts
 const kept = removeRun({ lines: ['a', 'b', 'c'], run: ['b'] });
 ```
 */
export function removeRun({
  lines,
  run,
}: RemoveRunRequest,): readonly string[] {
  /**
   First line of the run, which every match must start with.
   */
  const [first,] = run;
  if (first === undefined) {
    throw new AnswersError('One line run of the answers file holds no line.',);
  }
  /**
   Line the run starts on.
   */
  const start = lines.findIndex(function startsRun(
    line,
    index,
  ): boolean {
    return (line === first) && run.every(function matches(
      item,
      offset,
    ): boolean {
      return lines[index + offset] === item;
    },);
  },);
  if (start === (-1)) {
    throw new AnswersError(`No run of lines starts with: ${first}`,);
  }
  return [
    ...lines.slice(
      0,
      start,
    ),
    ...lines.slice(start + run.length,),
  ];
}

/**
 Inserts one block before the line that starts the anchor heading.
 
 @param lines - File lines.
 
 @param anchor - Heading the block goes before, matched as a prefix.
 
 @param block - Lines to insert.
 
 @returns The lines with the block inserted, and the line it starts on.
 
 @throws {@link AnswersError} when no line starts with the anchor.
 
 @example
 ```ts
 const inserted = insertBefore({ lines: ['## Recorded'], anchor: '## Recorded', block: ['## Answered'] });
 ```
 */
export function insertBefore({
  lines,
  anchor,
  block,
}: InsertBeforeRequest,): Insertion {
  /**
   Line the anchor heading starts on.
   */
  const at = lines.findIndex(function startsAnchor(line,): boolean {
    return line.startsWith(anchor,);
  },);
  if (at === (-1)) {
    throw new AnswersError(`No heading starts with: ${anchor}`,);
  }
  return {
    lines: [
      ...lines.slice(
        0,
        at,
      ),
      ...block,
      ...lines.slice(at,),
    ],
    at,
  };
}

/**
 Drops one trailing empty line, which is what a Markdown block written as a template ends with.
 
 @param lines - Lines to trim.
 
 @returns The lines without a last line that is empty.
 
 @example
 ```ts
 const block = withoutTrailingEmptyLine({ lines: ['## Answered', ''] });
 ```
 */
function withoutTrailingEmptyLine({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.at(-1,) === '' ? lines.slice(
    0,
    -1,
  ) : lines;
}

/**
 Records one answers block: removes every named part, then inserts the block.
 
 @param text - Questions file contents.
 
 @param answers - Answers to record.
 
 @returns The rewritten text and what the rewrite did.
 
 @throws {@link AnswersError} when a named part is not in the file.
 
 @example
 ```ts
 const result = recordAnswers({ text, answers });
 ```
 */
export function recordAnswers({
  text,
  answers,
}: RecordRequest,): RecordResult {
  /**
   Lines without the named sections.
   */
  const withoutSections = answers.remove
    .sections
    .reduce(
    function dropSection(
      lines,
      heading,
    ): readonly string[] {
      return removeSection({
        lines,
        heading,
      },);
    },
    text.split('\n',),
    );
  /**
   Lines without the named bullets.
   */
  const withoutBullets = answers.remove
    .bullets
    .reduce(
    function dropBullet(
      lines,
      bullet,
    ): readonly string[] {
      return removeBullet({
        lines,
        bullet,
      },);
    },
    withoutSections,
    );
  /**
   Lines without the named runs.
   */
  const withoutRuns = answers.remove
    .lineRuns
    .reduce(
    function dropRun(
      lines,
      run,
    ): readonly string[] {
      return removeRun({
        lines,
        run,
      },);
    },
    withoutBullets,
    );
  /**
   Block lines, without the trailing empty line a template ends with.
   */
  const block = withoutTrailingEmptyLine({ lines: answers.record
    .split('\n',), },);
  /**
   File with the block inserted.
   */
  const inserted = insertBefore({
    lines: withoutRuns,
    anchor: answers.anchor,
    block,
  },);
  return {
    text: inserted.lines
      .join('\n',),
    removedSections: answers.remove
      .sections
      .length,
    removedBullets: answers.remove
      .bullets
      .length,
    removedRuns: answers.remove
      .lineRuns
      .length,
    insertedAtLine: inserted.at,
  };
}
