/**
 Guards ledger X19 at the preparation: every outside read goes through the
 readers its caller hands over, each asked once about the original, and what
 each returns reaches the sheets, so a test that hands over readers reading
 nothing buys no web search and reads no corpus.

 The run's readers are built by `outsideReadsFrom` over the environment, the
 transport and the corpus, and its cases hand over a stub transport, a
 throwaway cache directory and an invented corpus (ledger T8), so the wiring
 a run buys through is driven with nothing leaving the process.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { EXA_CONTENTS_URL, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  EXA_API_KEY_VAR,
  EXA_SEARCH_URL,
  LOOKUP_CACHE_DIR_VAR,
  REFERENCE_CACHE_SUBDIR,
} from '@monochromatic-dev/translation-repair-reference';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  outsideReadsFrom,
  type PassOutsideReads,
  preparePassEntry,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the preparation writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-outside-reads-test', },);

/**
 Roster the preparation asks.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
];

/**
 Cache generation the preparation's caches are stamped with.
 */
const DIGEST = 'pass-outside-reads-test' as PipelineDigest;

/**
 Original naming a cat another entry declares.
 */
const SOURCE = '猫糖在窗台上睡着了。';

/**
 Evidence line the work-title reader answers with.
 */
const WORK_TITLE_LINE = 'WORK TITLE EVIDENCE: 猫之歌 is published as "Song of the Cat"';

/**
 Client pairing the one block of each side, refusing every other sheet.

 @returns Client serving only structured calls

 @example
 ```ts
 const client = pairingClient();
 ```
 */
function pairingClient(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Every pairing round's reply.
       */
      const value: unknown = { pairs: [{ source: 0, target: 0, },], };
      return request.validate(value,)
        ? { kind: 'ok', value, rawText: JSON.stringify(value,), }
        : { kind: 'schema-mismatch', rawText: '{}', detail: 'fixture answers pairing only', };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Page the original links, an invented address the stub transport answers.
 */
const LINK = 'https://example.test/mittens-memo';

/**
 Original naming a work and linking a page.
 */
const CITING_SOURCE = `猫糖读过《猫之歌》，见[备忘](${LINK})。`;

/**
 Key the environment carries in these cases, which only the stub ever sees.
 */
const KEY = 'whisker-key';

/**
 Clock a case hands over in place of the wall clock.
 */
const FIXED = new Date('2026-09-29T08:00:00.000Z',);

/**
 Corpus pin no case reads, the stubs answering in its place.
 */
const PIN = {
  cloneDir: '/nonexistent',
  commitSha: 'deadbeef',
};

/**
 One call a stub transport saw.
 */
type SeenCall = {
  readonly url: string;
  readonly key: string;
};

/**
 Transport answering the search and the contents endpoints with invented
 pages, and recording each call's endpoint and key.

 @returns Transport plus the calls it saw

 @example
 ```ts
 const { fetchFn, seen, } = exaStub();
 ```
 */
function exaStub(): {
  readonly fetchFn: typeof fetch;
  readonly seen: SeenCall[];
} {
  /**
   Calls seen so far.
   */
  const seen: SeenCall[] = [];
  /**
   Transport under test.

   @param input - endpoint production asks

   @param init - request production builds

   @returns Endpoint's invented answer

   @throws Error for any endpoint but the two the readers use
   */
  async function stubbed(
    input: string | URL | Request,
    init?: RequestInit,
  ): Promise<Response> {
    /**
     Endpoint asked.
     */
    const url = ((typeof input) === 'string') ? input : ((input instanceof URL) ? input.href : input.url);
    seen.push({
      url,
      key: new Headers(init?.headers,).get('x-api-key',) ?? '',
    },);
    if (url === EXA_SEARCH_URL) {
      return Response.json({
        results: [{
          title: 'Song of the Cat - Catopedia',
          url: 'https://example.test/song-of-the-cat',
          highlights: ['Song of the Cat (猫之歌) is a novel about a tabby.',],
          highlightScores: [0.9,],
        },],
      },);
    }
    if (url === EXA_CONTENTS_URL) {
      return Response.json({
        results: [{
          url: LINK,
          title: 'Mittens memo',
          text: 'Mittens slept by the stove.',
        },],
        statuses: [{
          id: LINK,
          status: 'success',
        },],
      },);
    }
    throw new Error(`the outside readers asked an endpoint no stub answers: ${url}`,);
  }
  return {
    fetchFn: stubbed,
    seen,
  };
}

/**
 Corpus lister for the web cases, which read no corpus.

 @returns Never

 @throws Error always
 */
async function listNoPeople(): Promise<readonly string[]> {
  throw new Error('the web readers list no corpus entries',);
}

/**
 Corpus reader for the web cases, which read no corpus.

 @returns Never

 @throws Error always
 */
async function readNoFile(): Promise<string> {
  throw new Error('the web readers read no corpus document',);
}

/**
 Every record under one cache directory and its reference subdirectory.

 @param dir - lookup cache directory a case owns

 @returns Each record's stamp, lookup records first

 @example
 ```ts
 const stamps = await recordStamps({ dir, },);
 ```
 */
async function recordStamps({ dir, }: { readonly dir: string; },): Promise<readonly string[]> {
  /**
   Record files in one directory, by path.

   @param at - directory read

   @returns Paths of its JSON files
   */
  async function jsonFiles(at: string,): Promise<readonly string[]> {
    return (await readdir(at,))
      .filter(function isRecord(name,): boolean {
        return name.endsWith('.json',);
      },)
      .map(function pathOf(name,): string {
        return join(at, name,);
      },);
  }
  /**
   Lookup records, then reference records.
   */
  const paths = [
    ...await jsonFiles(dir,),
    ...await jsonFiles(join(dir, REFERENCE_CACHE_SUBDIR,),),
  ];
  return await Promise.all(paths.map(async function stampOf(path,): Promise<string> {
    /**
     Record as written.
     */
    const record: unknown = JSON.parse(await readFile(path, 'utf8',),);
    return (record as { readonly fetchedAt: string; }).fetchedAt;
  },),);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'preparePassEntry reads outside the pipeline only through its readers (ledger X19)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EACH READER ONCE ABOUT THE ORIGINAL, and carries the work-title evidence and the other '
            + 'entry\'s name on the sheets',
          fn: async () => {
            /**
             Originals each reader that reads one was asked about.
             */
            const askedAbout: Record<'workTitles' | 'references', string[]> = {
              workTitles: [],
              references: [],
            };
            /**
             Reads the corpus-name reader took, which reads no original.
             */
            const corpusReads = { count: 0, };
            /**
             Readers recording what they were asked.
             */
            const outsideReads: PassOutsideReads = {
              workTitles: async ({ sourceText, },) => {
                askedAbout.workTitles.push(sourceText,);
                return [WORK_TITLE_LINE,];
              },
              references: async ({ sourceText, },) => {
                askedAbout.references.push(sourceText,);
                return '';
              },
              corpusNames: async () => {
                corpusReads.count += 1;
                return [{ source: '猫糖', renderings: ['Cat Candy',], entryId: 'gum', },];
              },
              entryInfo: async () => ({ kind: 'absent', } as const),
            };
            /**
             Directory this case owns for its entry caches.
             */
            await using scratch = await scratchDir({ prefix: 'pass-outside-reads-', },);
            const dir = scratch.path;
            /**
             The preparation.
             */
            const paired = await preparePassEntry({
              client: pairingClient(),
              entryId: 'CatEntry',
              entryCacheDir: dir,
              pipelineDigest: DIGEST,
              modelIds: ROSTER,
              sourceText: SOURCE,
              targetText: 'Cat Candy fell asleep on the windowsill.',
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              outsideReads,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            /**
             What every slice sheet reads about the page.
             */
            const identityContext = paired.prepared.identityContext ?? '';
            expect({
              askedAbout,
              corpusReads: corpusReads.count,
              workTitle: identityContext.includes(WORK_TITLE_LINE,),
              corpusName: identityContext.includes('- 猫糖 (entry gum): "Cat Candy"',),
            },).toEqual({
              askedAbout: {
                workTitles: [SOURCE,],
                references: [SOURCE,],
              },
              corpusReads: 1,
              workTitle: true,
              corpusName: true,
            },);
          },
        },),
        it({
          name: 'CARRIES THE RECORDED DEATH of the entry\'s info.yml onto the prepared pair, asking the info reader once about the entry, '
            + 'and names the date and the subject on the pair',
          fn: async () => {
            /**
             Entries the info reader was asked about.
             */
            const askedAbout: string[] = [];
            /**
             Readers whose info reader answers with a recorded death.
             */
            const outsideReads: PassOutsideReads = {
              workTitles: async () => [],
              references: async () => '',
              corpusNames: async () => [],
              entryInfo: async ({ entryId, },) => {
                askedAbout.push(entryId,);
                return {
                  kind: 'read',
                  info: { info: { died: '2025-02-26', }, },
                } as const;
              },
            };
            await using scratch = await scratchDir({ prefix: 'pass-entry-info-', },);
            /**
             The preparation of a page whose English front matter names the cat.
             */
            const paired = await preparePassEntry({
              client: pairingClient(),
              entryId: 'CatEntry',
              entryCacheDir: scratch.path,
              pipelineDigest: DIGEST,
              modelIds: ROSTER,
              sourceText: SOURCE,
              targetText: '---\nname: Mittens\n---\n\nMittens falls asleep on the windowsill.\n',
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              outsideReads,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              askedAbout,
              lifeEnded: paired.prepared.lifeEnded,
              subjectNames: paired.prepared.subjectNames,
              dateLine: (paired.prepared.identityContext ?? '').split('\n',).filter(function namesDeath(line,): boolean {
                return line.startsWith('- died:',);
              },),
            },).toEqual({
              askedAbout: ['CatEntry',],
              lifeEnded: true,
              subjectNames: ['Mittens',],
              dateLine: ['- died: the date this person died, as the corpus records it: "2025-02-26".',],
            },);
          },
        },),
      ],
    },),

    describe({
      name: outsideReadsFrom.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUYS THE WORK AND THE LINKED PAGE through the transport it is handed, with the environment\'s key, into '
            + 'the environment\'s cache directory, stamped by the wall clock unless a clock is handed in (ledger T8)',
          fn: async () => {
            /**
             Cache directory for the wall-clock reads.
             */
            await using walledScratch = await scratchDir({ prefix: 'outside-reads-wall-', },);
            const walled = walledScratch.path;
            /**
             Cache directory for the fixed-clock reads.
             */
            await using fixedScratch = await scratchDir({ prefix: 'outside-reads-fixed-', },);
            const fixed = fixedScratch.path;
            const { fetchFn, seen, } = exaStub();
            /**
             Readers over one cache directory, on the wall clock or the fixed one.

             @param dir - lookup cache directory

             @param clock - whether to hand over the fixed clock

             @returns What each web reader answered
             */
            async function readAll(
              {
                dir,
                clock,
              }: {
                readonly dir: string;
                readonly clock: 'wall' | 'fixed';
              },
            ): Promise<{ readonly titles: string; readonly references: string; }> {
              /**
               Readers under test.
               */
              const reads = outsideReadsFrom({
                env: {
                  [EXA_API_KEY_VAR]: KEY,
                  [LOOKUP_CACHE_DIR_VAR]: dir,
                },
                fetchFn,
                pin: PIN,
                listPeople: listNoPeople,
                readFile: readNoFile,
                ...((clock === 'fixed') ? { now: () => FIXED, } : {}),
              },);
              /**
               Arguments both readers take.
               */
              const asked = {
                sourceText: CITING_SOURCE,
                signal: new AbortController().signal,
                l,
              };
              return {
                titles: (await reads.workTitles(asked,)).join('\n',),
                references: await reads.references(asked,),
              };
            }
            /**
             Wall clock before the reads, as the stamps write it: ISO text of one
             length sorts as the moments it names (ledger B78).
             */
            const before = new Date().toISOString();
            /**
             What the wall-clock readers answered.
             */
            const answered = await readAll({
              dir: walled,
              clock: 'wall',
            },);
            /**
             Wall clock after the reads, as the stamps write it.
             */
            const after = new Date().toISOString();
            await readAll({
              dir: fixed,
              clock: 'fixed',
            },);

            expect(answered.titles,).toContain('Song of the Cat',);
            expect(answered.references,).toContain('Mittens slept by the stove.',);
            expect(seen,).toEqual([
              { url: EXA_SEARCH_URL, key: KEY, },
              { url: EXA_CONTENTS_URL, key: KEY, },
              { url: EXA_SEARCH_URL, key: KEY, },
              { url: EXA_CONTENTS_URL, key: KEY, },
            ],);
            /**
             Stamps of the wall-clock records, as written.
             */
            const wallStamps = await recordStamps({ dir: walled, },);
            expect(wallStamps.length,).toBe(2,);
            for (const stamp of wallStamps) {
              expect(stamp.length,).toBe(before.length,);
              expect((stamp >= before) && (stamp <= after),).toBe(true,);
            }
            expect(await recordStamps({ dir: fixed, },),).toEqual([
              FIXED.toISOString(),
              FIXED.toISOString(),
            ],);
          },
        },),
        it({
          name: 'SEARCHES NO TITLE AND READS NO PAGE WITHOUT A KEY, asking the transport nothing, so a run with no key '
            + 'buys nothing',
          fn: async () => {
            /**
             Cache directory the reads would write to.
             */
            await using scratch = await scratchDir({ prefix: 'outside-reads-keyless-', },);
            const dir = scratch.path;
            const { fetchFn, seen, } = exaStub();
            /**
             Readers over an environment with no key.
             */
            const reads = outsideReadsFrom({
              env: { [LOOKUP_CACHE_DIR_VAR]: dir, },
              fetchFn,
              pin: PIN,
              listPeople: listNoPeople,
              readFile: readNoFile,
            },);
            /**
             Arguments both readers take.
             */
            const asked = {
              sourceText: CITING_SOURCE,
              signal: new AbortController().signal,
              l,
            };
            expect(await reads.workTitles(asked,),).toEqual([],);
            expect(await reads.references(asked,),).toBe('',);
            expect(seen,).toEqual([],);
          },
        },),
        it({
          name: 'READS THE ENTRY\'S info.yml at the pin it is handed, through the reader it is handed, and records the parsed death',
          fn: async () => {
            /**
             Pin and path each corpus read was asked at.
             */
            const asked: unknown[] = [];
            /**
             Readers over an invented corpus whose one entry records a death.
             */
            const reads = outsideReadsFrom({
              env: {},
              fetchFn: exaStub().fetchFn,
              pin: PIN,
              listPeople: listNoPeople,
              readFile: async function readInfo({
                pin,
                relPath,
              },) {
                asked.push({ pin, relPath, },);
                return 'info:\n  born: 2019-05-02\n  died: 2025-02-26\n';
              },
            },);
            expect(await reads.entryInfo({ entryId: 'gum', l, },),).toEqual({
              kind: 'read',
              info: { info: { born: '2019-05-02', died: '2025-02-26', }, },
            },);
            expect(asked,).toEqual([{ pin: PIN, relPath: 'people/gum/info.yml', },],);
          },
        },),
        it({
          name: 'ANSWERS A DAMAGED info.yml with the unreadable record instead of raising, so the entry the preparation reads '
            + 'it for still ships (the owner\'s rule)',
          fn: async () => {
            /**
             Readers over an invented corpus whose one entry has a file that is not YAML.
             */
            const reads = outsideReadsFrom({
              env: {},
              fetchFn: exaStub().fetchFn,
              pin: PIN,
              listPeople: listNoPeople,
              readFile: async function readDamaged() {
                return 'info: [\n';
              },
            },);
            expect(await reads.entryInfo({ entryId: 'gum', l, },),).toEqual({
              kind: 'unreadable',
              refusal: 'The info.yml of entry gum at the pin refused to parse at line 2 column 1 (BAD_INDENT); '
                + 'corpus metadata parses upstream, so this signals corruption.',
            },);
          },
        },),
        it({
          name: 'READS THE NAMES EVERY ENTRY DECLARES at the pin it is handed, through the lister and reader it is handed',
          fn: async () => {
            /**
             Pins each corpus read was asked at.
             */
            const pins: unknown[] = [];
            /**
             Readers over an invented corpus of one entry.
             */
            const reads = outsideReadsFrom({
              env: {},
              fetchFn: exaStub().fetchFn,
              pin: PIN,
              listPeople: async function listGum({ pin, },) {
                pins.push(pin,);
                return ['gum',];
              },
              readFile: async function readGum({
                pin,
                relPath,
              },) {
                pins.push(pin,);
                return (relPath === 'people/gum/page.md')
                  ? '---\nname: 猫糖\n---\n\n猫糖睡了。\n'
                  : '---\nname: Cat Candy\n---\n\nCat Candy slept.\n';
              },
            },);
            expect(await reads.corpusNames(),).toEqual([{
              source: '猫糖',
              renderings: ['Cat Candy',],
              entryId: 'gum',
            },],);
            expect(pins,).toEqual([
              PIN,
              PIN,
              PIN,
            ],);
          },
        },),
      ],
    },),
  ],
},);
