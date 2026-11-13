/**
 Finds the statement section a review file holds as its first section, reads its one fenced block, and hands the block to the
 reader that keeps it to the shape the brief asked for.

 The section is the first one after the review file's title, so a reader meets it before any finding, and so a review file with no
 title or with another section first is answered with what is wrong rather than searched for a block that may not be the statement.

 @module
 */

import {
  type ReviewStatement,
  REVIEW_STATEMENT_HEADING,
  REVIEW_STATEMENT_LANGUAGE,
} from './review-statement-shape.ts';
import { statementOf, } from './review-statement-fields.ts';
import { reviewStatementErrorOf, } from './review-statement-values.ts';

/**
 How many backticks open a fenced block, which is what a block's language begins after.
 */
const FENCE_CHARACTERS = 3;

/**
 The closing of a fenced block this reader never found, as a symbol so that no line number can stand for it.
 */
const NOT_CLOSED: unique symbol = Symbol('fenced block reaches the section end unclosed',);

/**
 Reads the fenced block a statement section holds.

 @param section - the lines between the statement heading and the next heading

 @param file - the review file, for the message

 @returns the block's text, without its fences

 @throws ReviewStatementError when the section holds no single block of the right language

 @example
 ```ts
 const block = statementBlockIn({ section: ['```review-statement', '{}', '```'], file: 'a/review.md' });
 ```
 */
function statementBlockIn({
  section,
  file,
}: {
  readonly section: readonly string[];
  readonly file: string;
},): string {
  /**
   Every fenced block the section holds, by its language.
   */
  const blocks: {
    readonly language: string;
    readonly lines: readonly string[]
  }[] = [];
  /**
   Where the next line is read from, which advances past each block it reads.
   */
  const cursor = { at: 0, };
  while (cursor.at < section.length) {
    /**
     The line that may open a fence.
     */
    const line = section[cursor.at] ?? '';
    if (!line.startsWith('```',)) {
      cursor.at += 1;
      continue;
    }
    /**
     Where the block's content starts.
     */
    const start = cursor.at + 1;
    /**
     Where its closing fence is, or the symbol that says it is never closed.
     */
    const closeAt = closingFenceAt({
      section,
      from: start,
    },);
    if (closeAt === NOT_CLOSED) {
      throw reviewStatementErrorOf({
        file,
        message: 'the statement section holds a fenced block that is never closed',
      },);
    }
    blocks.push({
      language: line.slice(FENCE_CHARACTERS,)
        .trim(),
      lines: section.slice(
        start,
        closeAt,
      ),
    },);
    cursor.at = closeAt + 1;
  }
  /**
   The blocks tagged as a review statement.
   */
  const statements = blocks.filter(function isAStatement(block,) {
    return block.language === REVIEW_STATEMENT_LANGUAGE;
  },);
  if (statements.length === 0) {
    /**
     The languages the section's blocks are tagged with, for the message.
     */
    const found = blocks.map(function languageOf(block,) {
      return (block.language === '') ? 'untagged' : block.language;
    },);
    throw reviewStatementErrorOf({
      file,
      message: (found.length === 0)
        ? `"${REVIEW_STATEMENT_HEADING}" holds no fenced block tagged "${REVIEW_STATEMENT_LANGUAGE}"`
        : `"${REVIEW_STATEMENT_HEADING}" holds a fenced block tagged ${found.join(', ')} and none tagged `
          + `"${REVIEW_STATEMENT_LANGUAGE}"`,
    },);
  }
  if (statements.length > 1) {
    throw reviewStatementErrorOf({
      file,
      message: `"${REVIEW_STATEMENT_HEADING}" holds ${String(statements.length,)} blocks tagged `
        + `"${REVIEW_STATEMENT_LANGUAGE}", so which one states the review cannot be read`,
    },);
  }
  return statements[0]
    ?.lines
    .join('\n',)
    ?? '';
}

/**
 Finds the closing fence of a block.

 @param section - the lines between the statement heading and the next heading

 @param from - the line the block's content starts at

 @returns the closing fence's line, or {@link NOT_CLOSED} when the block never closes

 @example
 ```ts
 const closeAt = closingFenceAt({ section, from: 1 });
 ```
 */
function closingFenceAt({
  section,
  from,
}: {
  readonly section: readonly string[];
  readonly from: number;
},): number | typeof NOT_CLOSED {
  for (const [at, line] of section.entries()) {
    if ((at >= from) && (line.trim() === '```')) {
      return at;
    }
  }
  return NOT_CLOSED;
}

/**
 Reads the statement a review file holds as its first section.

 @param shown - the review file's text

 @param file - the review file's path, for the message

 @returns the statement

 @throws ReviewStatementError when the file holds no statement section, the section is not the first one, or the block does not
 fit the shape

 @example
 ```ts
 const statement = statementInReview({ shown: await readFile('a/review.md', 'utf8'), file: 'a/review.md' });
 ```
 */
export function statementInReview({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): ReviewStatement {
  /**
   The review file's lines.
   */
  const lines = shown.split('\n',);
  /**
   Where the review's title is, which the statement follows.
   */
  const titleAt = lines.findIndex(function isATitle(line,) {
    return line.startsWith('# ',);
  },);
  if (titleAt === (-1)) {
    throw reviewStatementErrorOf({
      file,
      message: `has no title line starting "# ", so where its statement section belongs cannot be read`,
    });
  }
  /**
   Where the review's first section is.
   */
  const sectionAt = lines.findIndex(function isASection(
    line,
    at,
  ) {
    return (at > titleAt) && line.startsWith('## ',);
  },);
  if (sectionAt === (-1)) {
    throw reviewStatementErrorOf({
      file,
      message: `has no "${REVIEW_STATEMENT_HEADING}" section after its title`,
    });
  }
  if (lines[sectionAt] !== REVIEW_STATEMENT_HEADING) {
    throw reviewStatementErrorOf({
      file,
      message: `has "${lines[sectionAt] ?? ''}" as its first section, not "${REVIEW_STATEMENT_HEADING}"; the statement is the `
        + 'first section after the title',
    });
  }
  /**
   Where the section after the statement begins, or the end of the file.
   */
  const nextSectionAt = lines.findIndex(function isTheNextSection(
    line,
    at,
  ) {
    return (at > sectionAt) && line.startsWith('## ',);
  },);
  return statementOf({
    block: statementBlockIn({
      section: lines.slice(
        sectionAt + 1,
        (nextSectionAt === (-1)) ? lines.length : nextSectionAt,
      ),
      file,
    },),
    file,
  },);
}

/**
 Says whether the commit a statement names is the commit the review recorded it read, allowing an abbreviated id.

 @param stated - the commit the statement names

 @param recorded - the commit the review's record names

 @returns whether they name the same commit

 @example
 ```ts
 const same = commitIsTheSame({ stated: 'a723733c55', recorded: 'a723733c5553aa463732823268f15d649b91ea69' });
 ```
 */
export function commitIsTheSame({
  stated,
  recorded,
}: {
  readonly stated: string;
  readonly recorded: string;
},): boolean {
  return stated.startsWith(recorded,)
    || recorded.startsWith(stated,);
}
