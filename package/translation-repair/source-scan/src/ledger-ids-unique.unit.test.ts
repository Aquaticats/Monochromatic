/**
 Guards the audit ledger against an entry id that heads two entries and against an index line that lists an id no
 ledger file defines (ledger B953). Two branches each took the next free `B` number for an entry, the merge kept
 both, and nothing read the headings: only a hand check of the merged index found that `B948` named two entries, one
 of them in a file the other branch did not touch. A repeated id sends a reader of the by-id index to the wrong entry
 and makes a citation such as "see B948" ambiguous for good.

 WHAT IS READ is every `.md` file of the package's `doc` folder whose name opens with `audit-ledger`, bar the by-id
 index. A heading is an entry heading when it is at the second level and reads `B` or `M`, a number, an optional
 `to` and a second number, then a colon: `## B948: ...` or `## B480 to B487: ...`. A range names every id in it. An
 id heads two entries when two entry headings name it, in one file or in two. A heading that continues an entry
 (`## B899, round 8`, `## B832 follow-ups`) or sits below the second level (`### B920: ...`) defines its id for the
 index and is never counted as a second entry. A line inside a fenced code block is not a heading. The by-id index
 lists an id when a line starts `- [` and an id or a range of ids; the id must be defined by some heading of the
 ledger files.

 THE FIXTURES COME FIRST, so the real-docs case is read against a check shown able to find each shape (ledger M21),
 and the real-docs case then proves the reader sees the real headings by adding one copy of a real heading and one
 index line for an id nothing defines, each of which must be found. Fixtures are cat-themed.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup';
import {
  FAMILY_PACKAGES,
  REPOSITORY_ROOT,
} from './living-docs.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Ledger ids
// The reader of a heading, the two findings, and the read of the real docs.

/**
 File name of the by-id index, which lists entries and defines none.
 */
const INDEX_NAME = 'audit-ledger-entries-by-id.md';

/**
 Letters an entry id opens with: `B` for a finding, `M` for a process mistake.
 */
const ID_LETTERS: ReadonlySet<string> = new Set(['B', 'M',],);

/**
 What `findIndex` answers where no element matches.
 */
const NO_INDEX = -1;

/**
 Most ids one range may name. A longer or backward range names its two ends only, so no heading can make the reader
 loop for long.
 */
const MAX_RANGE = 1_000;

/**
 One doc of the ledger.
 */
type LedgerDoc = {
  /**
   Doc's path from the repository root, or any label unique among the docs read.
   */
  readonly path: string;

  /**
   Doc's text.
   */
  readonly text: string;
};

/**
 What a line's start names: ledger ids and how much of the line they took.
 */
type NamedIds = {
  /**
   Every id the text names, a range expanded.
   */
  readonly ids: readonly string[];

  /**
   Characters of the text the ids took.
   */
  readonly length: number;
};

/**
 One heading that names ledger ids.
 */
type LedgerHeading = {
  /**
   Heading's level: the number of `#` marks.
   */
  readonly level: number;

  /**
   Every id the heading names, a range expanded.
   */
  readonly ids: readonly string[];

  /**
   Whether the heading opens an entry of its own: at the second level, with a colon after the id or the range.
   */
  readonly entry: boolean;
};

/**
 One heading with the place it stands at.
 */
type PlacedHeading = LedgerHeading & {
  /**
   Heading's line number in its doc, from one.
   */
  readonly line: number;

  /**
   Heading's doc path.
   */
  readonly path: string;
};

/**
 How far the reading of one doc has got.
 */
type DocReading = {
  /**
   Whether the lines read last are inside a fenced code block.
   */
  readonly fenced: boolean;

  /**
   Headings met so far, outside fenced blocks.
   */
  readonly found: readonly PlacedHeading[];
};

/**
 What a text that opens with no id names.
 */
const NAMES_NO_IDS: NamedIds = { ids: [], length: 0, };

/**
 What a line that is no ledger heading reads as: no ids, so nothing to count or to define.
 */
const NOT_A_LEDGER_HEADING: LedgerHeading = { level: 0, ids: [], entry: false, };

/**
 The run of ASCII digits a text holds from an offset.

 @param text - text read

 @param from - offset the run may start at

 @returns Digits, empty where the text has none there

 @example
 ```ts
 digitsAt({ text: 'B948:', from: 1, },); // '948'
 ```
 */
function digitsAt({ text, from, }: { readonly text: string; readonly from: number; },): string {
  /**
   The text from the offset on.
   */
  const rest = text.slice(from,);
  /**
   Offset in the rest of the first character that is no digit, negative where every character is one. The run is
   ASCII, so a code point count and a UTF-16 offset agree up to it.
   */
  const stop = Array.from(rest,).findIndex(function isNoDigit(character,): boolean {
    return !isAsciiDigit({ character, },);
  },);
  return (stop === NO_INDEX) ? rest : rest.slice(0, stop,);
}

/**
 Every id a number or a range of numbers names, with the letter in front.

 @param letter - id letter

 @param first - first number

 @param last - last number, the first one for a single id

 @returns Ids of the range, or the two ends where the range runs backward or is longer than the bound

 @example
 ```ts
 idsOf({ letter: 'B', first: 1, last: 3, },); // ['B1', 'B2', 'B3']
 ```
 */
function idsOf(
  {
    letter,
    first,
    last,
  }: {
    readonly letter: string;
    readonly first: number;
    readonly last: number;
  },
): readonly string[] {
  if ((last < first) || ((last - first) > MAX_RANGE))
    return [`${letter}${String(first,)}`, `${letter}${String(last,)}`,];
  return Array.from({ length: (last - first) + 1, }, function idAt(_, at,): string {
    return `${letter}${String(first + at,)}`;
  },);
}

/**
 The ids a text names at its start, as an entry heading or an index line writes them: a letter and a number, and
 optionally ` to ` with a second number that may carry its own letter.

 @param text - text read, starting at the letter

 @returns Ids and length of text they took; no ids and no length where the text does not open with an id

 @example
 ```ts
 namedIds({ text: 'B480 to B487: the rest', },); // ids B480 to B487, length 12
 ```
 */
function namedIds({ text, }: { readonly text: string; },): NamedIds {
  /**
   The id letter.
   */
  const letter = text.charAt(0,);
  if (!ID_LETTERS.has(letter,))
    return NAMES_NO_IDS;
  /**
   First number's digits.
   */
  const firstDigits = digitsAt({ text, from: 1, },);
  if (firstDigits === '')
    return NAMES_NO_IDS;
  /**
   First number.
   */
  const first = Number(firstDigits,);
  /**
   Offset just past the first number.
   */
  const firstEnd = 1 + firstDigits.length;
  /**
   Offset just past the word that joins the two numbers of a range.
   */
  const joined = firstEnd + ' to '.length;
  if (!text.startsWith(' to ', firstEnd,))
    return { ids: idsOf({ letter, first, last: first, },), length: firstEnd, };
  /**
   Offset the second number may start at, past a letter it may carry.
   */
  const secondFrom = joined + (text.startsWith(letter, joined,) ? 1 : 0);
  /**
   Second number's digits.
   */
  const secondDigits = digitsAt({ text, from: secondFrom, },);
  if (secondDigits === '')
    return { ids: idsOf({ letter, first, last: first, },), length: firstEnd, };
  return {
    ids: idsOf({ letter, first, last: Number(secondDigits,), },),
    length: secondFrom + secondDigits.length,
  };
}

/**
 The heading a line is, when it names ledger ids.

 @param text - one line of a doc

 @returns Level, ids and whether it opens an entry; no ids for a line that is no heading of the second to
 fourth level or names no id at its start

 @example
 ```ts
 ledgerHeadingOf({ text: '## B948: the named place', },); // level 2, ids B948, entry true
 ledgerHeadingOf({ text: '## B899, round 8: bench', },); // level 2, ids B899, entry false
 ```
 */
function ledgerHeadingOf({ text, }: { readonly text: string; },): LedgerHeading {
  /**
   Count of `#` marks the line opens with, the line's length where it is all marks.
   */
  const marks = Array.from(text,).findIndex(function isNoMark(character,): boolean {
    return character !== '#';
  },);
  /**
   Level the marks make.
   */
  const level = (marks === NO_INDEX) ? text.length : marks;
  if ((level < 2) || (level > 4) || (text.charAt(level,) !== ' '))
    return NOT_A_LEDGER_HEADING;
  /**
   What follows the marks and the space.
   */
  const rest = text.slice(level + 1,);
  /**
   The ids the heading opens with.
   */
  const named = namedIds({ text: rest, },);
  /**
   The character after the ids, empty at the end of the line.
   */
  const after = rest.charAt(named.length,);
  if ((named.ids.length === 0) || ((after !== ':') && (after !== ',') && (after !== ' ') && (after !== '')))
    return NOT_A_LEDGER_HEADING;
  return { level, ids: named.ids, entry: (level === 2) && (after === ':'), };
}

/**
 Every heading of the ledger docs that names ids, outside fenced code blocks.

 @param docs - ledger docs

 @returns Headings in the order of the docs and of the lines

 @example
 ```ts
 const headings = ledgerHeadings({ docs: [{ path: 'cat.md', text: '## B1: nap\n', },], },);
 ```
 */
function ledgerHeadings({ docs, }: { readonly docs: readonly LedgerDoc[]; },): readonly PlacedHeading[] {
  return docs.flatMap(function inDoc(doc,): readonly PlacedHeading[] {
    /**
     Where a doc's reading starts: outside a fenced block, with no heading met.
     */
    const start: DocReading = { fenced: false, found: [], };
    return doc.text.split('\n',).reduce(function readLine(reading, text, at,): DocReading {
      if (text.startsWith('```',) || text.startsWith('~~~',))
        return { fenced: !reading.fenced, found: reading.found, };
      if (reading.fenced)
        return reading;
      /**
       The heading this line is, with no ids for any other line.
       */
      const heading = ledgerHeadingOf({ text, },);
      return (heading.ids.length === 0)
        ? reading
        : { fenced: false, found: [...reading.found, { ...heading, line: at + 1, path: doc.path, },], };
    }, start,).found;
  },);
}

/**
 Every id that heads two entries, each as one finding naming both places.

 @param docs - ledger docs, without the by-id index

 @returns Findings, in the order of the ids' first entries

 @example
 ```ts
 const findings = repeatedEntryFindings({ docs: [], },); // []
 ```
 */
function repeatedEntryFindings({ docs, }: { readonly docs: readonly LedgerDoc[]; },): readonly string[] {
  /**
   Places of the entry headings that name each id, by id.
   */
  const places = new Map<string, string[]>();
  for (const heading of ledgerHeadings({ docs, },)) {
    if (!heading.entry)
      continue;
    for (const id of heading.ids) {
      places.set(id, [...(places.get(id,) ?? []), `${heading.path}:${String(heading.line,)}`,],);
    }
  }
  return [...places,]
    .filter(function isRepeated([, where,],): boolean {
      return where.length > 1;
    },)
    .map(function finding([id, where,],): string {
      return `${id} heads ${String(where.length,)} entries, at ${where.join(' and ',)}; `
        + 'give the entry added last the next free id';
    },);
}

/**
 Every index line that lists an id no heading of the ledger docs defines, each as one finding.

 @param index - by-id index

 @param docs - ledger docs, without the by-id index

 @returns Findings, in the order of the index lines

 @example
 ```ts
 const findings = unknownIndexIdFindings({ index: { path: 'ids.md', text: '- [B1, nap](cat.md)\n', }, docs: [], },);
 ```
 */
function unknownIndexIdFindings(
  {
    index,
    docs,
  }: {
    readonly index: LedgerDoc;
    readonly docs: readonly LedgerDoc[];
  },
): readonly string[] {
  /**
   Every id some heading of the docs names, of any level and form.
   */
  const defined: ReadonlySet<string> = new Set(
    ledgerHeadings({ docs, },).flatMap(function idsOfHeading(heading,): readonly string[] {
      return heading.ids;
    },),
  );
  return index.text.split('\n',).flatMap(function atLine(text, at,): readonly string[] {
    if (!text.startsWith('- [',))
      return [];
    /**
     The ids the line lists, none where it lists none.
     */
    const named = namedIds({ text: text.slice('- ['.length,), },);
    return named.ids
      .filter(function isUndefined(id,): boolean {
        return !defined.has(id,);
      },)
      .map(function finding(id,): string {
        return `${index.path}:${String(at + 1,)} lists ${id}, and no ledger file has a heading that names it`;
      },);
  },);
}

/**
 The ledger docs of the package, split into the by-id index and the files that hold entries.

 @returns Index and other `audit-ledger*.md` files, each with its path from the repository root

 @throws {@link Error} when the folder holds no by-id index

 @example
 ```ts
 const { index, docs, } = await readLedgerDocs();
 ```
 */
async function readLedgerDocs(): Promise<{ readonly index: LedgerDoc; readonly docs: readonly LedgerDoc[]; }> {
  /**
   Folder of the package's docs, from the repository root.
   */
  const folder = join(FAMILY_PACKAGES, 'doc',);
  /**
   Names of the ledger files, sorted so findings read in a stable order.
   */
  const names = (await readdir(join(REPOSITORY_ROOT, folder,),))
    .filter(function isLedgerFile(name,): boolean {
      return name.startsWith('audit-ledger',) && name.endsWith('.md',);
    },)
    .toSorted();
  /**
   Each ledger file with its text.
   */
  const all = await Promise.all(names.map(async function withText(name,): Promise<LedgerDoc> {
    return { path: join(folder, name,), text: await readFile(join(REPOSITORY_ROOT, folder, name,), 'utf8',), };
  },),);
  /**
   The by-id index, absent when the folder lacks it.
   */
  const index = all.find(function isIndex(doc,): boolean {
    return doc.path.endsWith(INDEX_NAME,);
  },);
  if (index === undefined)
    throw new Error(`${join(folder, INDEX_NAME,)} is not among the ledger files`,);
  return {
    index,
    docs: all.filter(function holdsEntries(doc,): boolean {
      return doc !== index;
    },),
  };
}

await describe({
  name: 'Audit ledger entry ids',
  children: [
    it({
      name: 'READS a heading\'s ids, a range expanded, and whether it opens an entry: colon form at the second level '
        + 'only',
      fn: async () => {
        expect(ledgerHeadingOf({ text: '## B948: the named place', },),)
          .toEqual({ level: 2, ids: ['B948',], entry: true, },);
        expect(ledgerHeadingOf({ text: '## B480 to B487: the rest', },).ids,).toEqual([
          'B480',
          'B481',
          'B482',
          'B483',
          'B484',
          'B485',
          'B486',
          'B487',
        ],);
        expect(ledgerHeadingOf({ text: '## M12 to 13: two naps', },).ids,).toEqual(['M12', 'M13',],);
        expect(ledgerHeadingOf({ text: '## B899, round 8: bench', },),)
          .toEqual({ level: 2, ids: ['B899',], entry: false, },);
        expect(ledgerHeadingOf({ text: '## B832 follow-ups', },),)
          .toEqual({ level: 2, ids: ['B832',], entry: false, },);
        expect(ledgerHeadingOf({ text: '### B920: peak rates', },),)
          .toEqual({ level: 3, ids: ['B920',], entry: false, },);
        expect(ledgerHeadingOf({ text: '### B827', },),)
          .toEqual({ level: 3, ids: ['B827',], entry: false, },);
        expect(ledgerHeadingOf({ text: '## B9 to B7: backward', },).ids,).toEqual(['B9', 'B7',],);
        expect(ledgerHeadingOf({ text: '# B1: title level', },).ids,).toEqual([],);
        expect(ledgerHeadingOf({ text: '##### B1: too deep', },).ids,).toEqual([],);
        expect(ledgerHeadingOf({ text: '## D12: other family', },).ids,).toEqual([],);
        expect(ledgerHeadingOf({ text: '## B12x: not an id', },).ids,).toEqual([],);
        expect(ledgerHeadingOf({ text: '## Bowl: not an id', },).ids,).toEqual([],);
        expect(ledgerHeadingOf({ text: 'B1: not a heading', },).ids,).toEqual([],);
        expect(ledgerHeadingOf({ text: '####', },).ids,).toEqual([],);
      },
    },),
    it({
      name: 'FINDS an id that heads two entries in one file, in two files and inside a range, and passes a '
        + 'continuation heading, a deeper heading, a quoted heading and an id that heads one entry',
      fn: async () => {
        expect(repeatedEntryFindings({
          docs: [
            {
              path: 'doc/cat-naps.md',
              text: '## B1: first nap\n\n## B2: second nap\n\n## B2: second nap again\n\n## B3 to B5: three naps\n',
            },
            {
              path: 'doc/cat-meals.md',
              text: '## B4: a meal\n\n## B1, round 2: more naps\n\n### B1: a deeper heading\n\n## B6: a lone meal\n',
            },
            {
              path: 'doc/cat-quotes.md',
              text: '## M1: a mistake\n\n```md\n## M1: quoted, not an entry\n```\n\n~~~\n## B6: quoted\n~~~\n',
            },
          ],
        },),).toEqual([
          'B2 heads 2 entries, at doc/cat-naps.md:3 and doc/cat-naps.md:5; give the entry added last the next free id',
          'B4 heads 2 entries, at doc/cat-naps.md:7 and doc/cat-meals.md:1; give the entry added last the next free id',
        ],);
      },
    },),
    it({
      name: 'FINDS an index line that lists an id, or a range holding one, that no heading names, and passes an id a '
        + 'range, a deeper heading or a continuation heading names',
      fn: async () => {
        expect(unknownIndexIdFindings({
          index: {
            path: 'doc/cat-ids.md',
            text: [
              '# Cat ids',
              '',
              '- [B1 to B3, naps](cat-naps.md)',
              '- [B4, a meal](cat-meals.md)',
              '- [B5 to B7, naps and a meal](cat-naps.md)',
              '- [B9 follow-ups, a later meal](cat-meals.md)',
              '- [M2, a mistake](cat-meals.md)',
              '- [B20, no such nap](cat-naps.md)',
              'B30 is not a list line',
              '',
            ].join('\n',),
          },
          docs: [
            { path: 'doc/cat-naps.md', text: '## B1 to B3: naps\n\n## B5: a nap\n\n## B7: a nap\n', },
            { path: 'doc/cat-meals.md', text: '### B4: a meal\n\n## B9, round 2: more\n\n## M2: a mistake\n', },
          ],
        },),).toEqual([
          'doc/cat-ids.md:5 lists B6, and no ledger file has a heading that names it',
          'doc/cat-ids.md:8 lists B20, and no ledger file has a heading that names it',
        ],);
      },
    },),
    it({
      name: 'FINDS NO ID THAT HEADS TWO ENTRIES and no index id nothing defines in the package\'s audit ledger, and '
        + 'reads the real headings: a copied heading and an invented index line are both found',
      fn: async () => {
        /**
         The real ledger docs.
         */
        const { index, docs, } = await readLedgerDocs();
        /**
         Entry headings of the real docs, to show the reader sees them.
         */
        const entries = ledgerHeadings({ docs, },).filter(function isEntry(heading,): boolean {
          return heading.entry;
        },);
        expect(entries.length,).toBeGreaterThan(900,);
        /**
         The first real entry heading, whose id a copy in a doc of its own repeats.
         */
        const [first,] = entries;
        if (first === undefined)
          throw new Error('the real ledger has no entry heading',);
        /**
         The first real entry's id.
         */
        const [firstId = '',] = first.ids;
        /**
         What the scan finds over the real docs.
         */
        const repeated = repeatedEntryFindings({ docs, },);
        /**
         What the scan finds over the real index.
         */
        const unknown = unknownIndexIdFindings({ index, docs, },);
        expect(repeatedEntryFindings({
          docs: [...docs, { path: 'doc/cat-copy.md', text: `## ${firstId}: a copy of a real id\n`, },],
        },).length - repeated.length,).toBe(1,);
        expect(unknownIndexIdFindings({
          index: { path: index.path, text: `${index.text}\n- [B99999, no such entry](cat.md)\n`, },
          docs,
        },).length - unknown.length,).toBe(1,);
        expectNoFindings({ findings: repeated, },);
        expectNoFindings({ findings: unknown, },);
      },
    },),
  ],
},);

//endregion Ledger ids
