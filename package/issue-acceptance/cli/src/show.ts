/**
 Prints issues for reading: the whole issue, or just what it asked for.

 A closing comment has to answer what the issue asked,
 so the acceptance sections are printed on their own,
 and the whole issue is printed when the words around them matter.

 @module
 */

import type {
  IssueSnapshot,
  IssueTracker,
} from './issue-tracker.ts';

/**
 Line prefix of a second-level Markdown heading, which starts a section.
 */
const SECTION_PREFIX = '## ';

/**
 Heading words that name what an issue asks for.
 */
// oxlint-disable-next-line no-restricted-syntax/no-regex -- one anchored prefix test over one heading line; a scan for five literals would be longer and no faster, and the pattern has no nested quantifier. Justification recorded in this package's README.
const ACCEPTANCE_HEADING = /^(?:Acceptance|Expected|Suggested|Fix|Required)/iu;

/**
 Characters of an acceptance section a reader gets,
 which is enough for the wording and short of pasting a whole body.
 */
const ACCEPTANCE_SECTION_LIMIT = 520;

/**
 Characters of the body quoted when an issue names no acceptance section.
 */
const ACCEPTANCE_FALLBACK_LIMIT = 300;

/**
 Characters of the title a reader gets.
 */
const TITLE_LIMIT = 110;

/**
 Which part of an issue to print.
 */
export type ShowMode = 'full' | 'acceptance';

/**
 One issue and the mode to print it in.
 */
export type RenderIssueRequest = {
  readonly snapshot: IssueSnapshot;
  readonly mode: ShowMode;
};

/**
 Renders one issue in one line, naming its number, state, and title.
 
 @param snapshot - Issue as the tracker reported it.
 
 @returns The heading line.
 
 @example
 ```ts
 const line = renderHeading({ snapshot, mode: 'full' });
 ```
 */
function renderHeading({ snapshot, }: { readonly snapshot: IssueSnapshot; },): string {
  return `#${String(snapshot.number,)} [${snapshot.state}] ${snapshot.title
    .slice(
      0,
      TITLE_LIMIT,
    )}`;
}

/**
 Renders the acceptance sections, or the opening of the body when none is named.
 
 @param snapshot - Issue as the tracker reported it.
 
 @returns One indented line per named section, or the fallback.
 
 @example
 ```ts
 const sections = renderAcceptance({ snapshot });
 ```
 */
function renderAcceptance({ snapshot, }: { readonly snapshot: IssueSnapshot; },): string {
  /**
   Acceptances sections the body names, each cut to the reader's budget.
   */
  const wanted = snapshot.body
    .split(SECTION_PREFIX,)
    .filter(function namesAcceptance(section,): boolean {
      return ACCEPTANCE_HEADING.test(section,);
    },)
    .map(function toLine(section,): string {
      return `  ${section.slice(
        0,
        ACCEPTANCE_SECTION_LIMIT,
      )
        .replaceAll(
          '\n',
          ' ',
        )}`;
    },);
  if (wanted.length > 0) {
    return wanted.join('\n',);
  }
  return `  (no named section) ${snapshot.body
    .slice(
      0,
      ACCEPTANCE_FALLBACK_LIMIT,
    )
    .replaceAll(
      '\n',
      ' ',
    )}`;
}

/**
 Renders one issue.
 
 @param snapshot - Issue as the tracker reported it.
 
 @param mode - Whether to print the whole issue or only what it asked for.
 
 @returns The block a reader gets, without a trailing newline.
 
 @example
 ```ts
 const block = renderIssue({ snapshot, mode: 'acceptance' });
 ```
 */
export function renderIssue({
  snapshot,
  mode,
}: RenderIssueRequest,): string {
  /**
   Heading line every mode starts with.
   */
  const heading = renderHeading({ snapshot, },);
  if (mode === 'acceptance') {
    return `${heading}\n${renderAcceptance({ snapshot, },)}`;
  }
  return [
    heading,
    snapshot.body,
    ...snapshot.comments
      .map(function toComment(body,): string {
        return `--- comment: ${body}`;
      },),
  ].join('\n',);
}

/**
 One issue read request.
 */
export type ShowIssuesRequest = {
  readonly numbers: readonly number[];
  readonly tracker: IssueTracker;
  readonly mode: ShowMode;
  readonly write: (line: string,) => void;
};

/**
 Reads and prints each named issue.
 
 @param numbers - Issues to print, in the order given.
 
 @param tracker - Tracker the issues are read from.
 
 @param mode - Whether to print each issue whole or only what it asked for.
 
 @param write - Where each block goes.
 
 @example
 ```ts
 await showIssues({ numbers: [1], tracker, mode: 'full', write: process.stdout.write.bind(process.stdout) });
 ```
 */
export async function showIssues({
  numbers,
  tracker,
  mode,
  write,
}: ShowIssuesRequest,): Promise<void> {
  for (const number of numbers) {
    /**
     Issue as the tracker reports it.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- the issues are printed one after another so the order reads as asked; see doc/lint/sequential-awaits.md
    const snapshot = await tracker.read(number,);
    write(`${renderIssue({
      snapshot,
      mode,
    },)}\n`,);
  }
}
