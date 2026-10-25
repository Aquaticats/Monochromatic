/**
 Tests for the preparation's page title lexicon (ledger H16): when it asks,
 whom it asks (the roster its hook hands over, ledger X12), what it stores,
 what a resume reads back, and what names the question.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { createHash, } from 'node:crypto';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  PAGE_TITLE_CACHE_VERSION,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type BenchSeating,
  keepBench,
  type SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  openPageTitleCache,
  pageTitleKey,
  type PageTitleLexiconRecord,
  passPageTitles,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the round writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-page-titles-test', },);

/**
 Roster the preparation started on.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
];

/**
 Roster a hook hands back after a dry-out, none of it the fixture's own.
 */
const RESEATED: readonly RosterModelId[] = [
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
];

/**
 Original repeating one title the archive does not pair.
 */
const REPEATING = '## 猫之歌\n\n小猫唱了《猫之歌》。\n';

/**
 Original repeating two titles the archive does not pair.
 */
const REPEATING_TWO = `${REPEATING}\n## 鱼之舞\n\n小猫跳了《鱼之舞》。\n`;

/**
 Original naming a title once.
 */
const ONCE = '小猫唱了《猫之歌》。\n';

/**
 Archive rendering no heading.
 */
const ARCHIVE = 'The kitten sang all afternoon.\n';

/**
 Builds a client recording the seats asked, every seat rendering the title
 alike, or refusing the sheet when told to.

 @param asked - sink for the seat of every call

 @param usable - whether the replies read, so a round can hear nobody

 @returns Client serving only structured calls

 @example
 ```ts
 const client = titleClient({ asked: [], usable: true, },);
 ```
 */
function titleClient(
  {
    asked,
    usable,
    answering,
    renderingsBySeat,
  }: {
    readonly asked: RosterModelId[];
    readonly usable: boolean;

    /**
     The only seats whose replies read, every seat when absent.
     */
    readonly answering?: readonly RosterModelId[];

    /**
     The renderings a seat gives, by title number from one, for a seat that
     names its own; every other seat renders the first title alike.
     */
    readonly renderingsBySeat?: Readonly<Partial<Record<RosterModelId, readonly string[]>>>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      /**
       This seat's reply: its own renderings where it names them, the one title alike otherwise.
       */
      const value: unknown = {
        titles: (renderingsBySeat?.[request.modelId] ?? ['Song of the Cat',])
          .map(function titled(
            rendering,
            at,
          ) {
            return { title: at + 1, rendering, };
          },),
      };
      return (usable && ((answering === undefined) || answering.includes(request.modelId,)) && request.validate(value,))
        ? { kind: 'ok', value, rawText: JSON.stringify(value,), }
        : { kind: 'schema-mismatch', rawText: '{}', detail: 'fixture refuses the sheet', };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 A store resuming from one map and persisting into another.

 @param resumed - records an earlier run stored

 @param persisted - sink for what this run stores

 @returns Cache over the two maps

 @example
 ```ts
 const cache = memoryCache({ resumed: new Map(), persisted: new Map(), },);
 ```
 */
function memoryCache(
  {
    resumed,
    persisted,
  }: {
    readonly resumed: ReadonlyMap<string, PageTitleLexiconRecord>;
    readonly persisted: Map<string, string>;
  },
): SliceCache<PageTitleLexiconRecord> {
  return {
    resumed,
    persist: async ({ key, serialized, },) => {
      persisted.set(key, serialized,);
    },
  };
}

/**
 Runs the lexicon over an original, recording seats, hook reads and what it stored.

 @param sourceText - original page

 @param usable - whether the bench's replies read

 @param resumed - records an earlier run stored

 @param reseat - whether the hook re-seats the round elsewhere

 @returns Lines, seats asked, hook reads and records stored

 @example
 ```ts
 const run = await lexiconRun({ sourceText: REPEATING, usable: true, resumed: new Map(), reseat: true, },);
 ```
 */
async function lexiconRun(
  {
    sourceText,
    usable,
    resumed,
    reseat,
    answering,
    renderingsBySeat,
  }: {
    readonly sourceText: string;
    readonly usable: boolean;
    readonly resumed: ReadonlyMap<string, PageTitleLexiconRecord>;
    readonly reseat: boolean;
    readonly answering?: readonly RosterModelId[];
    readonly renderingsBySeat?: Readonly<Partial<Record<RosterModelId, readonly string[]>>>;
  },
) {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   Records this run stored.
   */
  const persisted = new Map<string, string>();
  /**
   Reads the hook took.
   */
  const hook = { reads: 0, };
  /**
   What the lexicon added.
   */
  const added = await passPageTitles({
    client: titleClient({
      asked,
      usable,
      // Conditional spread keeps the field absent where every seat answers.
      ...((answering === undefined) ? {} : { answering, }),
      ...((renderingsBySeat === undefined) ? {} : { renderingsBySeat, }),
    },),
    modelIds: ROSTER,
    beforeItem: async (): Promise<BenchSeating> => {
      hook.reads += 1;
      return reseat ? { modelIds: RESEATED, } : await keepBench();
    },
    sourceText,
    targetText: ARCHIVE,
    identityContext: '',
    cache: memoryCache({ resumed, persisted, },),
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    added,
    asked,
    hookReads: hook.reads,
    persisted,
  };
}

/**
 Stored records read back as the cache would hand them over.

 @param persisted - records a run stored

 @returns Them, parsed

 @example
 ```ts
 const resumed = readBack({ persisted, },);
 ```
 */
function readBack(
  { persisted, }: { readonly persisted: ReadonlyMap<string, string>; },
): ReadonlyMap<string, PageTitleLexiconRecord> {
  return new Map([...persisted.entries(),].map(function parsed([key, serialized,],): readonly [
    string,
    PageTitleLexiconRecord,
  ] {
    return [key, JSON.parse(serialized,) as PageTitleLexiconRecord,];
  },),);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${passPageTitles.name} (ledger H16)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS NO HOOK AND ASKS NOBODY for a page that repeats no unpaired title',
          fn: async () => {
            /**
             Round over a title written once.
             */
            const once = await lexiconRun({ sourceText: ONCE, usable: true, resumed: new Map(), reseat: true, },);
            expect({
              asked: once.asked,
              hookReads: once.hookReads,
              lines: once.added.lines,
              stored: once.persisted.size,
            },).toEqual({
              asked: [],
              hookReads: 0,
              lines: [],
              stored: 0,
            },);
          },
        },),
        it({
          name: 'ASKS THE ROSTER ITS HOOK HANDS OVER, carries the settled title and stores the round once',
          fn: async () => {
            /**
             Round re-seated elsewhere.
             */
            const moved = await lexiconRun({ sourceText: REPEATING, usable: true, resumed: new Map(), reseat: true, },);
            expect({
              askedAny: moved.asked.length > 0,
              offReseated: moved.asked.filter(function outside(seat,): boolean {
                return !RESEATED.includes(seat,);
              },),
              hookReads: moved.hookReads,
              carries: moved.added.lines.some(function names(line,): boolean {
                return line.startsWith('- 猫之歌',) && line.includes('"Song of the Cat"',);
              },),
              stored: moved.persisted.size,
              // Stored under the roster that answered, so a resume on another
              // bench asks again rather than reading this bench's answer.
              storedUnderAnswering: moved.persisted.has(pageTitleKey({
                sourceText: REPEATING,
                spans: [{ source: '猫之歌', occurrences: 2, },],
                identityContext: '',
                modelIds: RESEATED,
              },),),
            },).toEqual({
              askedAny: true,
              offReseated: [],
              hookReads: 1,
              carries: true,
              stored: 1,
              storedUnderAnswering: true,
            },);
          },
        },),
        it({
          name: 'RESUMES A STORED ROUND without asking, so the lines, and every slice key they reach, stay put',
          fn: async () => {
            /**
             First round, which stores its answer.
             */
            const first = await lexiconRun({ sourceText: REPEATING, usable: true, resumed: new Map(), reseat: false, },);
            /**
             Second round over the stored answer.
             */
            const second = await lexiconRun({
              sourceText: REPEATING,
              usable: true,
              resumed: readBack({ persisted: first.persisted, },),
              reseat: false,
            },);
            expect({
              asked: second.asked,
              sameLines: JSON.stringify(second.added.lines,) === JSON.stringify(first.added.lines,),
              sameFindings: JSON.stringify(second.added.findings,) === JSON.stringify(first.added.findings,),
            },).toEqual({
              asked: [],
              sameLines: true,
              sameFindings: true,
            },);
          },
        },),
        it({
          name: 'STORES NOTHING when nobody answered usably, since an unreachable bench is not an answer',
          fn: async () => {
            /**
             Round the bench never answered usably.
             */
            const silent = await lexiconRun({ sourceText: REPEATING, usable: false, resumed: new Map(), reseat: false, },);
            expect({
              askedAny: silent.asked.length > 0,
              lines: silent.added.lines,
              stored: silent.persisted.size,
            },).toEqual({
              askedAny: true,
              lines: [],
              stored: 0,
            },);
          },
        },),
        it({
          name: 'SETTLES AND STORES NOTHING when one seat of two answered, since one judge alone never decides (the '
            + 'lead\'s ruling of 2026-10-25): the page keeps its current titles, and a later run with a healthier '
            + 'bench asks again rather than resuming a thin round',
          fn: async () => {
            /**
             Round where only the first seat's reply read.
             */
            const thin = await lexiconRun({
              sourceText: REPEATING,
              usable: true,
              resumed: new Map(),
              reseat: false,
              answering: [ROSTER[0] ?? SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            },);
            expect({
              askedAny: thin.asked.length > 0,
              lines: thin.added.lines,
              stored: thin.persisted.size,
            },).toEqual({
              askedAny: true,
              lines: [],
              stored: 0,
            },);
          },
        },),
        it({
          name: 'STORES NOTHING when both seats answered and split one against one, since no title settled and a '
            + 'round that leaves a title unsettled is no answer (the lead\'s ruling on re-check 3, Q1): a later run '
            + 'on a fuller bench asks again instead of resuming the split',
          fn: async () => {
            /**
             Round where the two seats render the one title differently.
             */
            const split = await lexiconRun({
              sourceText: REPEATING,
              usable: true,
              resumed: new Map(),
              reseat: false,
              renderingsBySeat: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['Song of the Cat',],
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['The Cat Song',],
              },
            },);
            /**
             The next run, resuming whatever the split round stored.
             */
            const next = await lexiconRun({
              sourceText: REPEATING,
              usable: true,
              resumed: readBack({ persisted: split.persisted, },),
              reseat: false,
            },);
            expect({
              askedAny: split.asked.length > 0,
              lines: split.added.lines,
              stored: split.persisted.size,
              nextAsksAgain: next.asked.length > 0,
            },).toEqual({
              askedAny: true,
              lines: [],
              stored: 0,
              nextAsksAgain: true,
            },);
          },
        },),
        it({
          name: 'STORES NOTHING when one title of two settled and the other split, but still carries the settled '
            + 'title in this run\'s lines: a round is stored only when every title it asked is settled',
          fn: async () => {
            /**
             Round where both seats agree on the first title and split on the second.
             */
            const partial = await lexiconRun({
              sourceText: REPEATING_TWO,
              usable: true,
              resumed: new Map(),
              reseat: false,
              renderingsBySeat: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: [
                  'Song of the Cat',
                  'Dance of the Fish',
                ],
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: [
                  'Song of the Cat',
                  'The Fish Dance',
                ],
              },
            },);
            expect({
              carriesSettled: partial.added.lines.some(function names(line,): boolean {
                return line.startsWith('- 猫之歌',) && line.includes('"Song of the Cat"',);
              },),
              carriesSplit: partial.added.lines.some(function names(line,): boolean {
                return line.startsWith('- 鱼之舞',);
              },),
              stored: partial.persisted.size,
            },).toEqual({
              carriesSettled: true,
              carriesSplit: false,
              stored: 0,
            },);
          },
        },),
        it({
          name: 'STORES THE ROUND when both titles settled on two voices apiece, which is the control for the '
            + 'two cases above: the assertion has to be reading whether every title settled and not whether the '
            + 'round heard enough voices',
          fn: async () => {
            /**
             Round where both seats agree on both titles.
             */
            const agreed = await lexiconRun({
              sourceText: REPEATING_TWO,
              usable: true,
              resumed: new Map(),
              reseat: false,
              renderingsBySeat: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: [
                  'Song of the Cat',
                  'Dance of the Fish',
                ],
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: [
                  'Song of the Cat',
                  'Dance of the Fish',
                ],
              },
            },);
            expect({
              titleLines: agreed.added.lines
                .filter(function isTitleLine(line,): boolean {
                  return line.startsWith('- ',);
                },)
                .length,
              stored: agreed.persisted.size,
            },).toEqual({
              titleLines: 2,
              stored: 1,
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${openPageTitleCache.name} (ledger T8)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RESUMES A STORED ROUND FROM DISK, and refuses a stored value that is not a round, or holds a title or a '
            + 'finding of the wrong shape, so a damaged record is asked again rather than read as settled',
          fn: async () => {
            /**
             Entry cache directory this case owns.
             */
            await using scratch = await scratchDir({ prefix: 'page-title-cache-', },);
            const dir = scratch.path;
            /**
             One settled title, every field of the right shape.
             */
            const title = {
              source: '猫之歌',
              occurrences: 2,
              rendering: 'Song of the Cat',
              voices: 2,
              heard: 3,
            };
            /**
             A round a run stored.
             */
            const round: PageTitleLexiconRecord = {
              titles: [title,],
              findings: ['the kitten sang twice',],
            };
            /**
             Stored values by label, the round first and every refusal after it.
             */
            const stored: readonly (readonly [string, unknown])[] = [
              ['round', round,],
              ['not a record', [round,],],
              ['titles not a list', { titles: title, findings: [], },],
              ['findings not a list', { titles: [], findings: 'none', },],
              ['finding not text', { titles: [], findings: [1,], },],
              ['title not a record', { titles: ['猫之歌',], findings: [], },],
              ['source not text', { titles: [{ ...title, source: 1, },], findings: [], },],
              ['rendering not text', { titles: [{ ...title, rendering: null, },], findings: [], },],
              ['occurrences not whole', { titles: [{ ...title, occurrences: 1.5, },], findings: [], },],
              ['voices not whole', { titles: [{ ...title, voices: '2', },], findings: [], },],
              ['heard not whole', { titles: [{ ...title, heard: undefined, },], findings: [], },],
            ];
            /**
             Key a label is stored under, shaped as the lexicon's keys are.

             @param label - stored value's label

             @returns Its key
             */
            function keyOf(label: string,): string {
              return createHash('sha256',).update(label, 'utf8',).digest('hex',);
            }
            /**
             Cache the run wrote through.
             */
            const written = await openPageTitleCache({
              dir,
              generation: 'page-title-cache-test',
            },);
            await Promise.all(stored.map(async function persistOne([label, value,],): Promise<void> {
              await written.persist({
                key: keyOf(label,),
                serialized: JSON.stringify(value,),
              },);
            },),);
            /**
             Cache a resumed run opens over the same directory.
             */
            const reopened = await openPageTitleCache({
              dir,
              generation: 'page-title-cache-test',
            },);
            expect([...reopened.resumed.entries(),],).toEqual([[keyOf('round',), round,],],);
          },
        },),
      ],
    },),

    describe({
      name: `${pageTitleKey.name} (ledger H16)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE QUESTION BY THE ORIGINAL, THE TITLES, THE DECLARED IDENTITY AND THE ROSTER, and by nothing else '
            + '(ledger B28)',
          fn: async () => {
            /**
             The titles asked.
             */
            const spans = [{ source: '猫之歌', occurrences: 2, },];
            /**
             Key of the fixture question.
             */
            const key = pageTitleKey({ sourceText: REPEATING, spans, identityContext: '', modelIds: ROSTER, },);
            expect({
              same: pageTitleKey({ sourceText: REPEATING, spans: [...spans,], identityContext: '', modelIds: [...ROSTER,], },)
                === key,
              roster: pageTitleKey({ sourceText: REPEATING, spans, identityContext: '', modelIds: RESEATED, },) === key,
              titles: pageTitleKey({
                sourceText: REPEATING,
                spans: [{ source: '猫之歌', occurrences: 3, },],
                identityContext: '',
                modelIds: ROSTER,
              },) === key,
              original: pageTitleKey({ sourceText: `${REPEATING}喵。\n`, spans, identityContext: '', modelIds: ROSTER, },) === key,
              identity: pageTitleKey({
                sourceText: REPEATING,
                spans,
                identityContext: '- 猫之歌 (web lookup): "Song of the Cat"',
                modelIds: ROSTER,
              },) === key,
            },).toEqual({
              same: true,
              roster: false,
              titles: false,
              original: false,
              identity: false,
            },);
          },
        },),
        it({
          name: 'STANDS AT VERSION EIGHT since the sheet of the page title reading carries the house rule that quoted words keep '
            + 'the speaker\'s tense (n161), which a reading cached under version one was written without, says so in the '
            + 'very sentence that puts the life in the past (n166), which a reading cached under version two was written without, '
            + 'and protects readers only from the account of how a death or an attempt happened (the XingZ6015 read), which a '
            + 'reading cached under version three was written without, and asks for the rendering before the title '
            + '(ledger B762), which a reading cached under version four was written without, and closes its rounds under the '
            + 'last-seat grace of 2026-10-22 (ledger B816), which a reading cached under version five was decided without, and settles no title on fewer than two heard voices '
            + '(the lead\'s ruling of 2026-10-25), which a reading cached under version six was decided without, and '
            + 'settles a title only on a rendering two voices gave (re-check 2, finding F1), which a reading cached under '
            + 'version seven was decided without',
          fn: async () => {
            expect(PAGE_TITLE_CACHE_VERSION,).toBe(8,);
          },
        },),
        it({
          name: 'IS THE SHA-256 OF ONE JSON VALUE carrying the cache version, so moving the version moves every key '
            + '(ledger M25) and no two questions share bytes (ledger X15)',
          fn: async () => {
            /**
             The titles asked.
             */
            const spans = [{ source: '猫之歌', occurrences: 2, },];
            expect(pageTitleKey({ sourceText: REPEATING, spans, identityContext: '', modelIds: ROSTER, },),).toBe(
              createHash('sha256',)
                .update(
                JSON.stringify({
                  version: PAGE_TITLE_CACHE_VERSION,
                  source: hashContent({ content: REPEATING, },),
                  titles: spans,
                  identity: hashContent({ content: '', },),
                  roster: ROSTER,
                },),
                'utf8',
                )
                .digest('hex',),
            );
          },
        },),
      ],
    },),
  ],
},);
