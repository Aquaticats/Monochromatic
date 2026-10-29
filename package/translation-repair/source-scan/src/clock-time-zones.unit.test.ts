/**
 Guards against clock times written with no zone (ledger D12 and D25). The
 package dates in UTC; a time copied from `git log` or `find` comes out in the
 local zone, and one written with no zone cannot be told from one that did
 not. D12 gave every Markdown time its zone by hand, with no guard, and 198
 zone-less times stood in the living docs a week later.

 WHAT IS READ is prose: the comments of every TypeScript file under `src`
 (tests included, since their comments document too), the package's docs and
 README, the translation-repair decision records at the repository root, the
 canonical handover and the snapshot it links, the readiness signal and the
 OpenRouter pass log it links as current (ledger D29), and the
 translation-repair runbooks and troubleshooting docs (ledger D30). String
 literals are fixture
 text and are not read, and neither is code: a code span or a fenced block
 quotes a command or a format, not a moment.

 WHAT PASSES: a time followed by `UTC`, `EDT` or `Z` (seconds and a fraction
 may come between), the first time of a range whose second time carries the
 zone ("06:05 to 06:55 UTC", across a line break too), and a time whose next
 words say its zone was never established.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each shape (ledger M21). Fixtures are cat-themed.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup';
import {
  FAMILY_PACKAGES,
  readLivingRepositoryDocs,
  readPackageDocPaths,
  REPOSITORY_ROOT,
} from './living-docs.test-fixture.ts';
import {
  packageDirectoryOf,
  pathUnderSrc,
  readPackageSource,
} from './family-source.test-fixture.ts';
import { parseSource, } from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Words that close a time with its zone.
 */
const ZONES: readonly string[] = ['UTC', 'EDT', 'Z',];

/**
 Words joining the first time of a range to the second.
 */
const RANGE_JOINERS: readonly string[] = ['to', 'and', '-',];

/**
 Statement that a time's zone is unknown, which is honest rather than missing.
 */
const UNKNOWN_ZONE = 'zone it was observed in was never established';

/**
 How far past a time the unknown-zone statement may sit.
 */
const UNKNOWN_ZONE_REACH = 80;

/**
 What `indexOf` returns when nothing is left to find.
 */
const NOT_FOUND = -1;

/**
 What `clockEnd` returns for a colon that is not a clock time's.
 */
const NOT_A_TIME = -1;

/**
 Highest hour a clock time can carry.
 */
const LAST_HOUR = 23;

/**
 Highest minute a clock time can carry.
 */
const LAST_MINUTE = 59;

/**
 Characters from a time's colon to just past its minutes.
 */
const PAST_MINUTES = 3;

/**
 Characters from a time's colon back to the one before its hour.
 */
const BEFORE_HOUR = 3;

/**
 Characters a seconds field takes, its colon included.
 */
const SECONDS_WIDTH = 3;

/**
 One line of prose and where it stands.
 */
type ProseLine = {
  /**
   Line number in its file, from one.
   */
  readonly line: number;

  /**
   Its prose, comment markers and code spans removed.
   */
  readonly text: string;
};

/**
 A file's prose, line by line.
 */
type Prose = {
  /**
   Path from the repository root.
   */
  readonly path: string;

  /**
   Its prose lines, in order.
   */
  readonly lines: readonly ProseLine[];
};

/**
 A line with its inline code spans removed.

 @param text - one line

 @returns Text outside the spans

 @example
 ```ts
 outsideCodeSpans({ text: 'run `nap 12:30` at noon', },); // 'run   at noon'
 ```
 */
function outsideCodeSpans({ text, }: { readonly text: string; },): string {
  return text
    .split('`',)
    .map(function keepOutside(piece, at,): string {
      return ((at % 2) === 0) ? piece : ' ';
    },)
    .join('',);
}

/**
 Whether a character is a digit, empty at a text's edge.

 @param character - one character

 @returns Whether it is an ASCII digit

 @example
 ```ts
 isDigitAt({ character: '7', },); // true
 ```
 */
function isDigitAt({ character, }: { readonly character: string; },): boolean {
  return (character !== '') && isAsciiDigit({ character, },);
}

/**
 Where the clock time whose colon sits at an offset ends, or nothing when the
 colon is not one: two digits either side, an hour and a minute in range, and
 no digit, colon or point extending the number before it or a digit after.

 @param text - flat prose

 @param colon - offset of a colon

 @returns Offset just past the minutes, or {@link NOT_A_TIME}

 @example
 ```ts
 clockEnd({ text: 'at 14:05 UTC', colon: 5, },); // 8
 ```
 */
function clockEnd(
  {
    text,
    colon,
  }: {
    readonly text: string;
    readonly colon: number;
  },
): number {
  /**
   The four digits around the colon.
   */
  const digits = [text.charAt(colon - 2,), text.charAt(colon - 1,), text.charAt(colon + 1,), text.charAt(colon + 2,),];
  if (!digits.every(function isDigit(character,): boolean {
    return isDigitAt({ character, },);
  },))
    return NOT_A_TIME;
  /**
   Character before the hour.
   */
  const before = text.charAt(colon - BEFORE_HOUR,);
  if (isDigitAt({ character: before, },) || (before === ':') || (before === '.'))
    return NOT_A_TIME;
  if (isDigitAt({ character: text.charAt(colon + PAST_MINUTES,), },))
    return NOT_A_TIME;
  if ((Number(text.slice(colon - 2, colon,),) > LAST_HOUR) || (Number(text.slice(colon + 1, colon + PAST_MINUTES,),) > LAST_MINUTE))
    return NOT_A_TIME;
  return colon + PAST_MINUTES;
}

/**
 Where a time's seconds and fraction end, from just past its minutes.

 @param text - flat prose

 @param from - offset just past the minutes

 @returns Offset past any `:ss` and `.fff`

 @example
 ```ts
 pastSeconds({ text: '08:31:40.5 UTC', from: 5, },); // 10
 ```
 */
function pastSeconds(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Offset read so far, which is also what the function returns.
   */
  let at = from;
  if ((text.charAt(at,) === ':') && isDigitAt({ character: text.charAt(at + 1,), },)
    && isDigitAt({ character: text.charAt(at + 2,), },))
    at += SECONDS_WIDTH;
  if ((text.charAt(at,) === '.') && isDigitAt({ character: text.charAt(at + 1,), },)) {
    at += 1;
    while (isDigitAt({ character: text.charAt(at,), },))
      at += 1;
  }
  return at;
}

/**
 Offset past any run of spaces from a point, where a line break joined as a
 space may have left several.

 @param text - flat prose

 @param from - offset to start at

 @returns Offset of the first character that is not a space

 @example
 ```ts
 pastSpaces({ text: 'to   06:55', from: 2, },); // 5
 ```
 */
function pastSpaces(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Offset read so far, which is what the function returns.
   */
  let at = from;
  while (text.charAt(at,) === ' ')
    at += 1;
  return at;
}

/**
 Whether the text from an offset opens with a zone, spaces allowed first
 except before `Z`.

 @param text - flat prose

 @param from - offset just past a time

 @returns Whether a zone follows

 @example
 ```ts
 opensWithZone({ text: '14:05 UTC', from: 5, },); // true
 ```
 */
function opensWithZone(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): boolean {
  if (text.startsWith('Z', from,))
    return true;
  /**
   Offset past any spaces.
   */
  const start = pastSpaces({ text, from, },);
  return ZONES.some(function opens(zone,): boolean {
    return (zone !== 'Z') && text.startsWith(zone, start,);
  },);
}

/**
 Whether the time ending at an offset carries its zone: directly, as the first
 of a range whose second time does, or beside a statement that its zone is
 unknown.

 @param text - flat prose

 @param end - offset just past the minutes

 @returns Whether the time is zoned

 @example
 ```ts
 isZoned({ text: 'from 06:05 to 06:55 UTC', end: 10, },); // true
 ```
 */
function isZoned(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): boolean {
  /**
   Offset past the time's seconds and fraction.
   */
  const past = pastSeconds({ text, from: end, },);
  if (opensWithZone({ text, from: past, },))
    return true;
  if (text.slice(past, past + UNKNOWN_ZONE_REACH,).includes(UNKNOWN_ZONE,))
    return true;
  /**
   Offset past any spaces, where a range's joiner would stand.
   */
  const joinerAt = pastSpaces({ text, from: past, },);
  /**
   The joiner standing there, if any.
   */
  const joiner = RANGE_JOINERS.find(function joins(word,): boolean {
    return text.startsWith(word, joinerAt,);
  },);
  if (joiner === undefined)
    return false;
  /**
   Where the range's second time ends, if a time stands after the joiner.
   */
  const secondEnd = clockEnd({ text, colon: pastSpaces({ text, from: joinerAt + joiner.length, },) + 2, },);
  return (secondEnd !== NOT_A_TIME) && opensWithZone({ text, from: pastSeconds({ text, from: secondEnd, },), },);
}

/**
 Where each line starts once the lines are joined by single spaces.

 @param lines - prose lines in order

 @returns Offset of each line in the joined text

 @example
 ```ts
 lineStarts({ lines: [{ line: 1, text: 'nap', }, { line: 2, text: 'purr', },], },); // [0, 4]
 ```
 */
function lineStarts({ lines, }: { readonly lines: readonly ProseLine[]; },): readonly number[] {
  /**
   Starts found so far, which is what the function returns.
   */
  const starts: number[] = [];
  /**
   Offset the next line starts at.
   */
  let next = 0;
  for (const { text, } of lines) {
    starts.push(next,);
    next += text.length + 1;
  }
  return starts;
}

/**
 Offsets of every colon in a text.

 @param text - text searched

 @returns Colon offsets in order

 @example
 ```ts
 colonOffsets({ text: 'a:b:c', },); // [1, 3]
 ```
 */
function colonOffsets({ text, }: { readonly text: string; },): readonly number[] {
  /**
   Offsets found so far, which is what the function returns.
   */
  const offsets: number[] = [];
  /**
   Offset of the next colon, {@link NOT_FOUND} when none is left.
   */
  let at = text.indexOf(':',);
  while (at !== NOT_FOUND) {
    offsets.push(at,);
    at = text.indexOf(':', at + 1,);
  }
  return offsets;
}

/**
 Every clock time in a file's prose that carries no zone, each as
 `path:line HH:MM`.

 @param prose - one file's prose lines

 @returns Zone-less times in order

 @example
 ```ts
 const found = zonelessTimes({ prose, },);
 ```
 */
function zonelessTimes({ prose, }: { readonly prose: Prose; },): readonly string[] {
  /**
   Where each line starts in the flat text.
   */
  const starts = lineStarts({ lines: prose.lines, },);
  /**
   Every line joined by a space, so a range can cross a line break.
   */
  const flat = prose.lines
    .map(function textOf({ text, },): string {
      return text;
    },)
    .join(' ',);
  return colonOffsets({ text: flat, },).flatMap(function found(colon,): readonly string[] {
    /**
     Where the time with this colon ends, if it is one.
     */
    const end = clockEnd({ text: flat, colon, },);
    if ((end === NOT_A_TIME) || isZoned({ text: flat, end, },))
      return [];
    /**
     Index of the line holding the colon.
     */
    const lineIndex = starts.findLastIndex(function startsBefore(start,): boolean {
      return start <= colon;
    },);
    return [`${prose.path}:${String(prose.lines[lineIndex]?.line ?? 0,)} ${flat.slice(colon - 2, end,)}`,];
  },);
}

/**
 Lines outside fenced blocks, the fence lines themselves dropped, with their
 code spans removed.

 @param lines - raw lines with their numbers

 @returns Prose lines

 @example
 ```ts
 const prose = outsideFences({ lines, },);
 ```
 */
function outsideFences({ lines, }: { readonly lines: readonly ProseLine[]; },): readonly ProseLine[] {
  /**
   Lines kept so far, which is what the function returns.
   */
  const kept: ProseLine[] = [];
  /**
   Whether the scan stands inside a fenced block.
   */
  let inside = false;
  for (const { line, text, } of lines) {
    if (text.trimStart().startsWith('```',)) {
      inside = !inside;
      continue;
    }
    if (!inside)
      kept.push({ line, text: outsideCodeSpans({ text, },), },);
  }
  return kept;
}

/**
 Markdown prose: every line outside fenced blocks, code spans removed.

 @param path - path from the repository root

 @param text - file text

 @returns Its prose

 @example
 ```ts
 const prose = markdownProse({ path: 'doc/cat.md', text, },);
 ```
 */
function markdownProse(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): Prose {
  return {
    path,
    lines: outsideFences({
      lines: text.split('\n',).map(function located(line, at,): ProseLine {
        return { line: at + 1, text: line, };
      },),
    },),
  };
}

/**
 A comment line without its leading markers: `/**`, `//`, a closing `*\/` and
 a leading `*`.

 @param line - one line of a comment

 @returns Its text

 @example
 ```ts
 commentText({ line: ' * naps at noon', },); // ' naps at noon'
 ```
 */
function commentText({ line, }: { readonly line: string; },): string {
  /**
   The line with its opening and closing markers removed.
   */
  const text = line.trimStart().replace('/**', '',).replace('*/', '',).replace('//', '',);
  return text.startsWith('*',) ? text.slice(1,) : text;
}

/**
 TypeScript prose: the text of every comment, markers, fenced examples and
 code spans removed.

 @param path - path from the repository root

 @param source - file as the source scan reads it

 @returns Its prose

 @example
 ```ts
 const prose = commentProse({ path: 'src/cat.ts', source, },);
 ```
 */
function commentProse(
  {
    path,
    source,
  }: {
    readonly path: string;
    readonly source: { readonly path: string; readonly text: string; readonly isTest: boolean; };
  },
): Prose {
  /**
   Comment offsets from the parser.
   */
  const { comments, } = parseSource({ file: source, },);
  return {
    path,
    lines: comments.flatMap(function commentLines({ start, end, },): readonly ProseLine[] {
      /**
       Line number the comment opens on.
       */
      const first = source.text.slice(0, start,).split('\n',).length;
      return outsideFences({
        lines: source.text
          .slice(start, end,)
          .split('\n',)
          .map(function stripped(line, at,): ProseLine {
            return { line: first + at, text: commentText({ line, },), };
          },),
      },);
    },),
  };
}

/**
 Every prose file the guard reads.

 @returns Prose of the package's source comments, docs and README, the
 decision records, the current handover, the planning docs it links as current
 (ledger D29), and the runbooks and troubleshooting docs

 @throws {@link Error} when the living docs cannot be located, since the guard
 would then read less than it claims

 @example
 ```ts
 const files = await readLivingProse();
 ```
 */
async function readLivingProse(): Promise<readonly Prose[]> {
  /**
   Living repository-level docs.
   */
  const { decisionRecords, handover, currentPlanning, operations, } = await readLivingRepositoryDocs();
  /**
   Markdown paths from the repository root.
   */
  const markdown = [
    ...await readPackageDocPaths(),
    ...decisionRecords,
    ...handover,
    ...currentPlanning,
    ...operations,
  ];
  /**
   Markdown prose.
   */
  const docs = await Promise.all(markdown.map(async function read(path,): Promise<Prose> {
    return markdownProse({ path, text: await readFile(join(REPOSITORY_ROOT, path,), 'utf8',), },);
  },),);
  /**
   Comment prose of every TypeScript file.
   */
  const comments = (await readPackageSource()).map(function prose(source,): Prose {
    return commentProse({
      path: join(FAMILY_PACKAGES, packageDirectoryOf({ path: source.path, },), 'src', pathUnderSrc({ path: source.path, },),),
      source,
    },);
  },);
  return [...comments, ...docs,];
}

await describe({
  name: 'clock times written with no zone',
  children: [
    it({
      name: 'FINDS a time with no zone and passes one with its zone, a range stating it once across a line '
        + 'break, seconds before the zone, a code span, a fenced block and a stated unknown zone',
      fn: async () => {
        expect(zonelessTimes({
          prose: markdownProse({
            path: 'doc/cat.md',
            text: [
              'The cat napped at 14:05 and woke at 14:40 UTC.',
              'Fed from 06:05 to',
              '06:55 UTC, then again at 08:31:40.5 UTC and 2026-09-27T04:26:00Z.',
              'Run `nap --at 12:30` for a short one.',
              '```sh',
              'purr 23:59',
              '```',
              'The bowl refills at 02:53 (the zone it was observed in was never established).',
              'Ratios like 124:992 and times like 24:00 or 9:15 are not clock times here.',
              'Last seen at 21:57;',
            ].join('\n',),
          },),
        },),).toEqual(['doc/cat.md:1 14:05', 'doc/cat.md:10 21:57',],);
      },
    },),
    it({
      name: 'READS comments and not string literals in TypeScript',
      fn: async () => {
        /**
         A fixture file with a zone-less time in a comment and one in a string.
         */
        const source = {
          path: 'cat.ts',
          text: [
            '// The cat napped at 03:34 by the log.',
            '// Fed from 06:05 to',
            '// 06:55 UTC by the bowl.',
            '/**',
            ' Fed at 07:34 UTC.',
            ' */',
            'export const meal = \'dinner at 18:00\';',
          ].join('\n',),
          isTest: false,
        };
        expect(zonelessTimes({ prose: commentProse({ path: 'src/cat.ts', source, },), },),)
          .toEqual(['src/cat.ts:1 03:34',],);
      },
    },),
    it({
      name: 'FINDS NO ZONE-LESS CLOCK TIME in the package\'s comments and docs, the decision records, the '
        + 'current handover, the planning docs it links as current, or the runbooks and troubleshooting docs',
      fn: async () => {
        /**
         Every prose file read.
         */
        const files = await readLivingProse();
        expect(files.some(function isHandover({ path, },): boolean {
          return path.includes('translation-repair-handover-',);
        },),).toBe(true,);
        expect(files.some(function isPassLog({ path, },): boolean {
          return path.includes('translation-repair-openrouter-2026-09-03-proposal-',);
        },),).toBe(true,);
        expectNoFindings({
          findings: files.flatMap(function inFile(prose,): readonly string[] {
            return zonelessTimes({ prose, },);
          },),
        },);
      },
    },),
  ],
},);
