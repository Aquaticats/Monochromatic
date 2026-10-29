/**
 Guards the length and the naming of the package's Markdown docs and of the
 repository-level translation-repair docs (ledger M163 and M165). The owner
 ruled on 2026-10-12 that a Markdown file holds at most 1,000 lines, and up to
 2,000 where the lead records why 1,000 is too tight. The audit ledger had
 grown to 41,252 lines and the OpenRouter pass log to 26,295, each appended to
 with no bound, so a reader or an agent could open neither whole.

 A RECORD PAST THE LIMIT keeps its path as an index and files its sections
 beside it, each file one subject, phase or span of dates and named for it. The
 first split cut at whichever heading came before the limit and numbered the
 pieces; the owner rejected it, since a numbered name says nothing of what a
 file holds. So a file named by a counter is a finding, and so is a file that
 names a record whose index does not link it, which no reader starting at the
 index would find.

 WHAT IS READ is every `.md` file in the package's `doc` folder, its README,
 and every doc under the repository's `doc` families whose name starts with
 `translation-repair` or with one of the other translation-repair stems
 listed in {@link OTHER_STEMS}. Lines are counted as `wc --lines` counts them.

 THE FIXTURES COME FIRST, so the docs case is read against a check shown able
 to find each shape (ledger M21). Fixtures are cat-themed.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isAsciiDigits, } from '@monochromatic-dev/translation-repair-markup';
import {
  DOC_FAMILIES,
  OTHER_STEMS,
  readPackageDocPaths,
  REPOSITORY_ROOT,
  withFiles,
} from './living-docs.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Most lines a doc holds unless it is allowed more.
 */
const LINE_LIMIT = 1_000;

/**
 Most lines any allowance reaches.
 */
const ALLOWED_CEILING = 2_000;

/**
 A doc allowed past {@link LINE_LIMIT}, with the lead's reason.
 */
type LineAllowance = {
  /**
   Doc's path from the repository root.
   */
  readonly path: string;

  /**
   Lines it may hold, at most {@link ALLOWED_CEILING}.
   */
  readonly lines: number;

  /**
   Why splitting it at 1,000 lines would read worse.
   */
  readonly reason: string;
};

/**
 Docs allowed past the limit, each at the size it has. None now: the heading aligner plan, the last doc allowed, was
 filed by subject into files that each stay within the limit.
 */
const ALLOWANCES: readonly LineAllowance[] = [];

/**
 Name stems whose files keep a counter in their names, with the reason. None now: the history archive, the last
 record named by segment, was re-filed by what each file holds.
 */
const COUNTER_NAME_EXEMPTIONS: readonly { readonly opening: string; readonly reason: string; }[] = [];

/**
 Words that count a file where a subject belongs, when digits follow them.
 */
const COUNTER_WORDS: ReadonlySet<string> = new Set(['part', 'segment', 'subpart', 'chunk', 'section', 'page',],);

/**
 Lines in a text as `wc --lines` counts them: its line feeds, plus one for a
 last line with none.

 @param text - file's text

 @returns Line count

 @example
 ```ts
 lineCount({ text: 'nap\npurr\n', },); // 2
 ```
 */
function lineCount({ text, }: { readonly text: string; },): number {
  /**
   Line feeds in the text.
   */
  const feeds = text.split('\n',).length - 1;
  return ((text === '') || text.endsWith('\n',)) ? feeds : feeds + 1;
}

/**
 Every doc over its bound, and every allowance past the ceiling or no longer
 needed, each as one finding.

 @param counts - each doc's path and line count

 @param allowances - docs allowed past the limit

 @returns Findings, in the order of the counts and then the allowances

 @example
 ```ts
 const findings = lengthFindings({ counts: [{ path: 'doc/cat.md', lines: 1001, },], allowances: [], },);
 ```
 */
function lengthFindings(
  {
    counts,
    allowances,
  }: {
    readonly counts: readonly { readonly path: string; readonly lines: number; }[];
    readonly allowances: readonly LineAllowance[];
  },
): readonly string[] {
  /**
   Allowances by path.
   */
  const allowed = new Map(allowances.map(function byPath(allowance,): [string, LineAllowance,] {
    return [allowance.path, allowance,];
  },),);
  /**
   Lines each counted doc holds, by path.
   */
  const counted = new Map(counts.map(function byPath(count,): [string, number,] {
    return [count.path, count.lines,];
  },),);
  return [
    ...counts.flatMap(function over(count,): readonly string[] {
      /**
       Lines this doc may hold.
       */
      const bound = Math.min(allowed.get(count.path,)?.lines ?? LINE_LIMIT, ALLOWED_CEILING,);
      return (count.lines > bound)
        ? [`${count.path}: ${String(count.lines,)} lines, past its bound of ${String(bound,)}; file a subject or a span of `
          + 'dates in a file named for it',]
        : [];
    },),
    ...allowances.flatMap(function stale(allowance,): readonly string[] {
      if (allowance.lines > ALLOWED_CEILING)
        return [`${allowance.path}: allowed ${String(allowance.lines,)} lines, past the ceiling of ${String(ALLOWED_CEILING,)}`,];
      /**
       Lines the allowed doc holds, absent when no doc read has that path.
       */
      const lines = counted.get(allowance.path,);
      if (lines === undefined)
        return [`${allowance.path}: allowed, but no doc read has that path`,];
      return (lines <= LINE_LIMIT) ? [`${allowance.path}: allowed, but holds ${String(lines,)} lines, within the limit`,] : [];
    },),
  ];
}

/**
 Every doc whose name ends in a counter where a subject belongs, each as one
 finding. A counter is `part`, `segment`, `subpart`, `chunk`, `section` or
 `page` followed by digits, or a name whose last word is digits alone and no
 part of a date.

 @param paths - docs' paths from the repository root

 @param exemptions - name stems left alone

 @returns Findings, in the order of the paths

 @example
 ```ts
 counterNameFindings({ paths: ['doc/cat-part-02.md',], exemptions: [], },); // one finding
 ```
 */
function counterNameFindings(
  {
    paths,
    exemptions,
  }: {
    readonly paths: readonly string[];
    readonly exemptions: readonly { readonly opening: string; }[];
  },
): readonly string[] {
  return paths.flatMap(function named(path,): readonly string[] {
    /**
     Doc's name without its folder or extension.
     */
    const stem = basename(path, '.md',);
    if (exemptions.some(function opens({ opening, },): boolean {
      return stem.startsWith(opening,);
    },))
      return [];
    /**
     Name's words, as its hyphens part them.
     */
    const words = stem.split('-',);
    /**
     Name's last word and the one before it, empty where the name is too short to have one.
     */
    const [before, last,] = [words.at(-2,) ?? '', words.at(-1,) ?? '',];
    /**
     Whether the name carries a date, as year, month and day apart or as eight digits run together; its span
     may close on a bare day or month.
     */
    const dated = words.some(function opensDate(word, at,): boolean {
      if (!isAsciiDigits({ text: word, },))
        return false;
      if (word.length === 8)
        return true;
      return (word.length === 4)
        && isAsciiDigits({ text: words[at + 1] ?? '', },)
        && isAsciiDigits({ text: words[at + 2] ?? '', },);
    },);
    /**
     Whether the name's last word is a number.
     */
    const numbered = isAsciiDigits({ text: last, },);
    /**
     Whether the name ends in a counter word and its number.
     */
    const counted = numbered && COUNTER_WORDS.has(before,);
    /**
     Whether the name ends in a number no date in it accounts for.
     */
    const bare = numbered && (!dated);
    return (counted || bare)
      ? [`${path}: named by a counter; name the file for the subject or the span of dates it holds`,]
      : [];
  },);
}

/**
 Every file that names a record it is part of while that record's index does
 not link it, each as one finding.

 @param docs - each doc's path and text

 @returns Findings, in the order of the docs

 @example
 ```ts
 unlistedFileFindings({ docs: [{ path: 'doc/cat.md', text: '# Cat\n', },], },); // none
 ```
 */
function unlistedFileFindings(
  { docs, }: { readonly docs: readonly { readonly path: string; readonly text: string; }[]; },
): readonly string[] {
  /**
   Each doc's text, by path.
   */
  const texts = new Map(docs.map(function byPath(doc,): [string, string,] {
    return [doc.path, doc.text,];
  },),);
  return docs.flatMap(function filed(doc,): readonly string[] {
    /**
     Line naming the record this doc is part of, absent when it names none.
     */
    const line = doc.text.split('\n',).find(function namesRecord(candidate,): boolean {
      return candidate.startsWith('Part of [',) && candidate.endsWith('.md),',);
    },);
    if (line === undefined)
      return [];
    /**
     Record's file name, from the link's target.
     */
    const record = line.slice(line.lastIndexOf('](',) + 2, -2,);
    /**
     Record's index text, absent when no doc read has that path.
     */
    const index = texts.get(join(dirname(doc.path,), record,),);
    if (index === undefined)
      return [`${doc.path}: part of ${record}, which no doc read has as its path`,];
    return index.includes(`](${basename(doc.path,)})`,)
      ? []
      : [`${doc.path}: part of ${record}, whose text does not link it; list the file in that index`,];
  },);
}

/**
 Every doc the guard reads, as paths from the repository root.

 @returns Paths, sorted within each folder

 @example
 ```ts
 const paths = await readGuardedDocs();
 ```
 */
async function readGuardedDocs(): Promise<readonly string[]> {
  /**
   Docs in each repository family the guard reads.
   */
  const families = await Promise.all(DOC_FAMILIES.map(async function inFamily(family,): Promise<readonly string[]> {
    return (await readdir(join(REPOSITORY_ROOT, 'doc', family,),))
      .filter(function isTranslationRepair(name,): boolean {
        return name.endsWith('.md',) && (name.startsWith('translation-repair',) || OTHER_STEMS.some(function opens(stem,): boolean {
          return name.startsWith(stem,);
        },));
      },)
      .toSorted()
      .map(function underFamily(name,): string {
        return join('doc', family, name,);
      },);
  },),);
  return [
    ...families.flat(),
    ...await readPackageDocPaths(),
  ];
}

await describe({
  name: 'Markdown docs past the line limit',
  children: [
    it({
      name: 'COUNTS lines as wc does: a last line without a line feed counts, an empty text holds none',
      fn: async () => {
        expect([
          lineCount({ text: '', },),
          lineCount({ text: 'nap', },),
          lineCount({ text: 'nap\n', },),
          lineCount({ text: 'nap\npurr', },),
          lineCount({ text: '\n\n', },),
        ],).toEqual([0, 1, 1, 2, 2,],);
      },
    },),
    it({
      name: 'FINDS a doc one line past the limit, one past its allowance, an allowance past the ceiling, one for a '
        + 'doc within the limit and one for no doc, and passes a doc at the limit and one within its allowance',
      fn: async () => {
        expect(lengthFindings({
          counts: [
            { path: 'doc/nap.md', lines: 1_000, },
            { path: 'doc/purr.md', lines: 1_001, },
            { path: 'doc/whisker.md', lines: 1_500, },
            { path: 'doc/tail.md', lines: 1_601, },
            { path: 'doc/paw.md', lines: 2_500, },
            { path: 'doc/bowl.md', lines: 900, },
          ],
          allowances: [
            { path: 'doc/whisker.md', lines: 1_500, reason: 'one table of naps', },
            { path: 'doc/tail.md', lines: 1_600, reason: 'one list of tails', },
            { path: 'doc/paw.md', lines: 2_500, reason: 'one long paw', },
            { path: 'doc/bowl.md', lines: 1_200, reason: 'a bowl since emptied', },
            { path: 'doc/vet.md', lines: 1_200, reason: 'a vet since moved', },
          ],
        },),).toEqual([
          'doc/purr.md: 1001 lines, past its bound of 1000; file a subject or a span of dates in a file named for it',
          'doc/tail.md: 1601 lines, past its bound of 1600; file a subject or a span of dates in a file named for it',
          'doc/paw.md: 2500 lines, past its bound of 2000; file a subject or a span of dates in a file named for it',
          'doc/paw.md: allowed 2500 lines, past the ceiling of 2000',
          'doc/bowl.md: allowed, but holds 900 lines, within the limit',
          'doc/vet.md: allowed, but no doc read has that path',
        ],);
      },
    },),
    it({
      name: 'FINDS NO DOC PAST ITS BOUND among the package\'s docs, its README and the repository-level '
        + 'translation-repair docs',
      fn: async () => {
        /**
         Every doc the guard reads.
         */
        const paths = await readGuardedDocs();
        expect([
          join('package', 'translation-repair', 'doc', 'audit-ledger.md',),
          join('doc', 'planning', 'translation-repair-openrouter-2026-09-03.md',),
          join('doc', 'planning', 'the-third-rendering.md',),
        ].filter(function isUnread(path,): boolean {
          return !paths.includes(path,);
        },),).toEqual([],);
        /**
         Each doc's line count.
         */
        const counts = await Promise.all(paths.map(async function counted(path,): Promise<{ readonly path: string; readonly lines: number; }> {
          return { path, lines: lineCount({ text: await readFile(join(REPOSITORY_ROOT, path,), 'utf8',), },), };
        },),);
        expectNoFindings({ findings: lengthFindings({ counts, allowances: ALLOWANCES, },), },);
      },
    },),
    it({
      name: 'FINDS a name ending in a counter word and its number or in a bare number, and passes a name ending in a '
        + 'subject, in a date, in a clock span, or under an exempted stem',
      fn: async () => {
        expect(counterNameFindings({
          paths: [
            'doc/cat-part-02.md',
            'doc/cat-segment-1.md',
            'doc/cat-naps-on-the-sofa.md',
            'doc/cat-2026-09-22.md',
            'doc/cat-2026-09-22-0020-to-0350-utc.md',
            'doc/cat-2026-08-25-to-27.md',
            'doc/cat-reading-20260826.md',
            'doc/cat-naps-3.md',
            'doc/old-cat-part-01.md',
          ],
          exemptions: [{ opening: 'old-cat-', },],
        },),).toEqual([
          'doc/cat-part-02.md: named by a counter; name the file for the subject or the span of dates it holds',
          'doc/cat-segment-1.md: named by a counter; name the file for the subject or the span of dates it holds',
          'doc/cat-naps-3.md: named by a counter; name the file for the subject or the span of dates it holds',
        ],);
      },
    },),
    it({
      name: 'FINDS a file part of a record whose index does not link it and one part of a record no doc has, and '
        + 'passes a file its index links and a doc part of no record',
      fn: async () => {
        expect(unlistedFileFindings({
          docs: [
            { path: 'doc/cat.md', text: '# Cat\n\n- [Naps](cat-naps.md):\n', },
            { path: 'doc/cat-naps.md', text: '# Naps\n\nPart of [Cat](cat.md),\nwhich lists every file.\n', },
            { path: 'doc/cat-meals.md', text: '# Meals\n\nPart of [Cat](cat.md),\nwhich lists every file.\n', },
            { path: 'doc/cat-walks.md', text: '# Walks\n\nPart of [Dog](dog.md),\nwhich lists every file.\n', },
            { path: 'doc/bowl.md', text: '# Bowl\n', },
          ],
        },),).toEqual([
          'doc/cat-meals.md: part of cat.md, whose text does not link it; list the file in that index',
          'doc/cat-walks.md: part of dog.md, which no doc read has as its path',
        ],);
      },
    },),
    it({
      name: 'FINDS NO DOC NAMED BY A COUNTER and no file its record does not list, and reads the audit ledger as '
        + 'an index with files',
      fn: async () => {
        /**
         Every doc the guard reads.
         */
        const paths = await readGuardedDocs();
        expect((await withFiles({ path: join('package', 'translation-repair', 'doc', 'audit-ledger.md',), },)).length,)
          .toBeGreaterThan(20,);
        expectNoFindings({ findings: counterNameFindings({ paths, exemptions: COUNTER_NAME_EXEMPTIONS, },), },);
        /**
         Each doc's text.
         */
        const docs = await Promise.all(paths.map(async function withText(path,): Promise<{ readonly path: string; readonly text: string; }> {
          return { path, text: await readFile(join(REPOSITORY_ROOT, path,), 'utf8',), };
        },),);
        expectNoFindings({ findings: unlistedFileFindings({ docs, },), },);
      },
    },),
  ],
},);
