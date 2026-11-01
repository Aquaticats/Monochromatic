/**
 Tests for the page title lexicon (ledger H16): the one question, the reply
 guard, how the bench's renderings settle into one per title, and the lines
 the sheets carry.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buildPageTitleLexiconMessages,
  isPageTitleLexiconWire,
  PAGE_TITLE_IDENTITY_RULE,
  pageTitleLines,
  settlePageTitles,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

import {
  fenceOpening,
  LONG_FENCE_RUN,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/sheet-fence.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'page-title-lexicon-test', },);

/**
 Three seats, in roster order.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
];

/**
 Original naming two titles more than once.
 */
const SOURCE = '## 猫之歌\n\n小猫唱了《猫之歌》，又读了《鱼之梦》和《鱼之梦》。\n';

/**
 The two titles, in order of first appearance.
 */
const SPANS = [
  { source: '猫之歌', occurrences: 2, },
  { source: '鱼之梦', occurrences: 2, },
];

/**
 Builds a client answering each seat with its scripted reply, recording the seats asked.

 @param replies - reply per seat; a seat without one answers no titles

 @param asked - sink for the seat of every call

 @returns Client serving only structured calls

 @example
 ```ts
 const client = scriptedClient({ replies: {}, asked: [], },);
 ```
 */
function scriptedClient(
  {
    replies,
    asked,
  }: {
    readonly replies: Readonly<Record<string, unknown>>;
    readonly asked: RosterModelId[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      /**
       Scripted reply for this seat.
       */
      const value = replies[request.modelId] ?? { titles: [], };
      if (!request.validate(value,))
        throw new Error('scripted reply failed validator',);
      return { kind: 'ok', value, rawText: JSON.stringify(value,), };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Settles the fixture's titles over scripted replies, asking every seat.

 @param replies - reply per seat

 @returns Settled lexicon beside the seats asked

 @example
 ```ts
 const { lexicon, asked, } = await settled({ replies: {}, },);
 ```
 */
async function settled(
  { replies, }: { readonly replies: Readonly<Record<string, unknown>>; },
) {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   What the round settled.
   */
  const lexicon = await settlePageTitles({
    client: scriptedClient({ replies, asked, },),
    modelIds: ROSTER,
    sourceText: SOURCE,
    spans: SPANS,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    lexicon,
    asked,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'page title lexicon wire (ledger H16)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS ONE QUESTION with the numbered titles, the fenced original and the house rules',
          fn: async () => {
            /**
             The request.
             */
            const [system, user,] = buildPageTitleLexiconMessages({ sourceText: SOURCE, titles: ['猫之歌', '鱼之梦',], },);
            expect({
              houseRules: system?.content.includes('House rules this corpus is written under',),
              numbered: user?.content.includes('1. 猫之歌\n2. 鱼之梦',),
              original: user?.content.includes(SOURCE,),
            },).toEqual({
              houseRules: true,
              numbered: true,
              original: true,
            },);
          },
        },),
        it({
          name: 'ADMITS a list of numbered renderings and REFUSES a reply missing a rendering or numbering by fraction',
          fn: async () => {
            expect({
              wellFormed: isPageTitleLexiconWire({ titles: [{ title: 1, rendering: 'Song of the Cat', },], },),
              empty: isPageTitleLexiconWire({ titles: [], },),
              noRendering: isPageTitleLexiconWire({ titles: [{ title: 1, },], },),
              fraction: isPageTitleLexiconWire({ titles: [{ title: 1.5, rendering: 'Song', },], },),
              noList: isPageTitleLexiconWire({ renderings: [], },),
            },).toEqual({
              wellFormed: true,
              empty: true,
              noRendering: false,
              fraction: false,
              noList: false,
            },);
          },
        },),
        it({
          name: 'REFUSES A REPLY THAT IS NOT A RECORD AT ALL, and a titles list whose item is not a record either',
          fn: async () => {
            expect({
              notRecord: isPageTitleLexiconWire(['Song of the Cat',],),
              itemNotRecord: isPageTitleLexiconWire({ titles: [1,], },),
            },).toEqual({
              notRecord: false,
              itemNotRecord: false,
            },);
          },
        },),
        it({
          name: 'SHOWS the lexicon the page\'s declared identity (ledger B28): it is told to give a work its official '
            + 'English title, and the web lookups and notes that establish one are there',
          fn: async () => {
            /** Identity lines: a web lookup for the song, and a note naming it. */
            const identityContext = [
              'web lookup 《猫之歌》: "Song of the Cats", the official English title of the album track',
              'ARCHIVE note [^1]: the song is known in English as "Song of the Cats"',
            ].join('\n',);
            /** The request, as the model reads it. */
            const text = buildPageTitleLexiconMessages({
              sourceText: SOURCE,
              titles: ['猫之歌',],
              identityContext,
            },)
              .map(function contentOf(message,): string {
                return message.content;
              },)
              .join('\n',);
            expect(text,).toContain(identityContext,);
            expect(text,).toContain('Declared identity, when a DECLARED NAMES block precedes the documents:',);
            expect(text,).toContain(PAGE_TITLE_IDENTITY_RULE,);
            /** The request for a page that declares nothing. */
            const bare = buildPageTitleLexiconMessages({ sourceText: SOURCE, titles: ['猫之歌',], },)
              .map(function contentOf(message,): string {
                return message.content;
              },)
              .join('\n',);
            // A page with no identity gets no rules about a block it lacks.
            expect(bare,).not.toContain(PAGE_TITLE_IDENTITY_RULE,);
          },
        },),
        it({
          name: 'FENCES the original with a fence no identity line can reproduce (ledger B28): a note that writes a '
            + 'fence-length run would otherwise close the ORIGINAL block early',
          fn: async () => {
            /** A note line carrying a fence-character run longer than the shortest fence. */
            const identityContext = `- ARCHIVE note: ${LONG_FENCE_RUN} ORIGINAL ${LONG_FENCE_RUN} the titles end here`;
            /** The request's user message. */
            const [, user,] = buildPageTitleLexiconMessages({
              sourceText: SOURCE,
              titles: ['猫之歌',],
              identityContext,
            },);
            expect(fenceOpening({ content: user?.content ?? '', label: 'ORIGINAL', },).length,).toBeGreaterThan(
              LONG_FENCE_RUN.length,
            );
          },
        },),
      ],
    },),

    describe({
      name: `${settlePageTitles.name} (ledger H16)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS NOBODY for a page that repeats no unpaired title',
          fn: async () => {
            /**
             Seat of every call.
             */
            const asked: RosterModelId[] = [];
            /**
             What the round settled.
             */
            const lexicon = await settlePageTitles({
              client: scriptedClient({ replies: {}, asked, },),
              modelIds: ROSTER,
              sourceText: SOURCE,
              spans: [],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({ asked, titles: lexicon.titles, heard: lexicon.heard, },).toEqual({ asked: [], titles: [], heard: 0, },);
          },
        },),
        it({
          name: 'SETTLES NOTHING from one heard voice, whatever it rendered, since one judge alone never decides '
            + '(the lead\'s ruling of 2026-10-25, on the owner\'s rule): every repeated title keeps its current '
            + 'rendering, and a finding says why',
          fn: async () => {
            /**
             Seat of every call.
             */
            const asked: RosterModelId[] = [];
            /**
             What a bench of one seat settled.
             */
            const lexicon = await settlePageTitles({
              client: scriptedClient({
                replies: { [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], }, },
                asked,
              },),
              modelIds: [SEAT_HYPER_ONLY,],
              sourceText: SOURCE,
              spans: SPANS,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              fanOut: 'whole-bench',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              asked: asked.length,
              titles: lexicon.titles,
              heard: lexicon.heard,
              said: lexicon.findings.includes(
                'page title lexicon heard fewer than 2 voices, so no title settles and every repeated title keeps its '
                  + 'current rendering',
              ),
            },).toEqual({
              asked: 1,
              titles: [],
              heard: 1,
              said: true,
            },);
          },
        },),
        it({
          name: 'SETTLES A TITLE two heard voices rendered alike, on a bench of two (the positive control: two '
            + 'voices decide)',
          fn: async () => {
            /**
             Seat of every call.
             */
            const asked: RosterModelId[] = [];
            /**
             What a bench of two seats settled.
             */
            const lexicon = await settlePageTitles({
              client: scriptedClient({
                replies: {
                  [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
                  [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'song of the cat', },], },
                },
                asked,
              },),
              modelIds: [SEAT_HYPER_ONLY, SEAT_OPENROUTER_ONLY,],
              sourceText: SOURCE,
              spans: SPANS,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              fanOut: 'whole-bench',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              titles: lexicon.titles,
              heard: lexicon.heard,
            },).toEqual({
              titles: [{ source: '猫之歌', occurrences: 2, rendering: 'Song of the Cat', voices: 2, heard: 2, },],
              heard: 2,
            },);
          },
        },),
        it({
          name: 'KEEPS THE RENDERING MOST VOICES GAVE, comparing without case, spacing or wrapping quotes',
          fn: async () => {
            const { lexicon, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: { titles: [{ title: 1, rendering: 'Cat Song', },], },
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: '“Song of the Cat”', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'song of  the cat', },], },
              },
            },);
            expect(lexicon.titles,).toEqual([
              { source: '猫之歌', occurrences: 2, rendering: 'Song of the Cat', voices: 2, heard: 3, },
            ],);
          },
        },),
        it({
          name: 'COUNTS AS ONE RENDERING two that differ only in apostrophe style, spacing of any kind, or corner-bracket '
            + 'and underscore wrappers (ledger B24)',
          fn: async () => {
            /**
             Straight and curly apostrophes; a nonbreaking space and a tab.
             */
            const { lexicon: typography, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'The Cat\'s\u{00A0}Song', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'The Cat’s\tSong', },], },
              },
            },);
            expect(typography.titles,).toEqual([
              { source: '猫之歌', occurrences: 2, rendering: 'The Cat\'s\u{00A0}Song', voices: 2, heard: 3, },
            ],);
            /**
             Corner brackets and underscores around the same words.
             */
            const { lexicon: wrapped, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: { titles: [{ title: 1, rendering: 'Cat Song', },], },
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: '「Song of the Cat」', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: '_Song of the Cat_', },], },
              },
            },);
            expect(wrapped.titles,).toEqual([
              { source: '猫之歌', occurrences: 2, rendering: 'Song of the Cat', voices: 2, heard: 3, },
            ],);
          },
        },),
        it({
          name: 'LEAVES A TITLE UNSETTLED where two voices each gave a different rendering (a tie of one voice apiece), '
            + 'since one judge alone never decides (the lead\'s ruling of 2026-10-25), and a finding says why',
          fn: async () => {
            const { lexicon, } = await settled({
              replies: {
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 2, rendering: 'Dream of Fish', },], },
                [SEAT_HYPER_ONLY]: { titles: [{ title: 2, rendering: 'The Fish Dream', },], },
              },
            },);
            expect({
              titles: lexicon.titles,
              said: lexicon.findings.includes(
                'page title lexicon left 1 repeated title unsettled, since no rendering was given by 2 voices',
              ),
            },).toEqual({
              titles: [],
              said: true,
            },);
          },
        },),
        it({
          name: 'BREAKS A TIE OF TWO VOICES APIECE BY THE EARLIEST SEAT ON THE ROSTER, so a resumed page reads the same '
            + 'answer (the positive control: a rendering two voices gave settles)',
          fn: async () => {
            /**
             What a bench of four seats settled, two voices each for two renderings.
             */
            const lexicon = await settlePageTitles({
              client: scriptedClient({
                replies: {
                  [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: { titles: [{ title: 2, rendering: 'The Fish Dream', },], },
                  [SEAT_HYPER_ONLY]: { titles: [{ title: 2, rendering: 'Dream of Fish', },], },
                  [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 2, rendering: 'the fish dream', },], },
                  [SEAT_HYPER_VISION]: { titles: [{ title: 2, rendering: 'Dream of Fish', },], },
                },
                asked: [],
              },),
              modelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_HYPER_ONLY, SEAT_OPENROUTER_ONLY, SEAT_HYPER_VISION,],
              sourceText: SOURCE,
              spans: SPANS,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              fanOut: 'whole-bench',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(lexicon.titles,).toEqual([
              { source: '鱼之梦', occurrences: 2, rendering: 'The Fish Dream', voices: 2, heard: 4, },
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT A TITLE NO VOICE RENDERED, and reports how many settled',
          fn: async () => {
            const { lexicon, } = await settled({
              replies: {
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', }, { title: 2, rendering: '  ', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
              },
            },);
            expect({
              sources: lexicon.titles.map(function sourceOf(title,): string {
                return title.source;
              },),
              reported: lexicon.findings.includes('page title lexicon settled 1 of 2 repeated titles from 3 voices',),
            },).toEqual({
              sources: ['猫之歌',],
              reported: true,
            },);
          },
        },),
        it({
          name: 'LEAVES A TITLE UNSETTLED where only one voice of three rendered it, and the other two gave nothing '
            + 'or only invisible text: one judge alone never decides, whoever it is',
          fn: async () => {
            const { lexicon, } = await settled({
              replies: {
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: '\u{200B}', },], },
              },
            },);
            expect({
              titles: lexicon.titles,
              said: lexicon.findings.includes(
                'page title lexicon left 1 repeated title unsettled, since no rendering was given by 2 voices',
              ),
            },).toEqual({
              titles: [],
              said: true,
            },);
          },
        },),
        it({
          name: 'COUNTS NO VOICE FOR A RENDERING THAT SHOWS A READER NOTHING, as it counts none for an empty one: two '
            + 'seats rendering a title as a zero-width space no longer outvote the seats rendering it in words, so '
            + 'the one seat with words settles nothing alone, two seats with words settle it past the one showing '
            + 'nothing, and a title rendered only as a Hangul filler is left out',
          fn: async () => {
            /**
             One seat showing nothing, words from the other two.
             */
            const { lexicon: mixed, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: { titles: [{ title: 1, rendering: '\u{200B}', },], },
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
              },
            },);
            expect(mixed.titles,).toEqual([
              { source: '猫之歌', occurrences: 2, rendering: 'Song of the Cat', voices: 2, heard: 3, },
            ],);
            /**
             Renderings showing nothing from two seats, words from the third.
             */
            const { lexicon: padded, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: {
                  titles: [{ title: 1, rendering: '\u{200B}', }, { title: 2, rendering: '\u{3164}', },],
                },
                [SEAT_HYPER_ONLY]: {
                  titles: [{ title: 1, rendering: ' \u{200B} ', }, { title: 2, rendering: '“\u{3164}\u{200B}”', },],
                },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
              },
            },);
            /**
             The same round with both seats' renderings empty.
             */
            const { lexicon: empty, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: {
                  titles: [{ title: 1, rendering: '', }, { title: 2, rendering: '', },],
                },
                [SEAT_HYPER_ONLY]: {
                  titles: [{ title: 1, rendering: '', }, { title: 2, rendering: '', },],
                },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
              },
            },);
            expect(padded,).toEqual(empty,);
            expect(padded.titles,).toEqual([],);
          },
        },),
        it({
          name: 'COUNTS THE FIRST RENDERING a voice gives a title it answers twice, and NAMES the voice and the title in '
            + 'a finding rather than passing over the second',
          fn: async () => {
            const { lexicon, } = await settled({
              replies: {
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: {
                  titles: [{ title: 1, rendering: 'Cat Ballad', }, { title: 1, rendering: 'Song of the Cat', },],
                },
                [SEAT_HYPER_ONLY]: { titles: [{ title: 1, rendering: 'Song of the Cat', },], },
                [SEAT_OPENROUTER_ONLY]: { titles: [{ title: 1, rendering: 'Cat Ballad', },], },
              },
            },);
            // Counting the second answer instead would settle "Song of the Cat"
            // on two voices, and counting both would tie the two renderings.
            expect(lexicon,).toEqual({
              titles: [{ source: '猫之歌', occurrences: 2, rendering: 'Cat Ballad', voices: 2, heard: 3, },],
              heard: 3,
              findings: [
                `${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: duplicate-title-rendering (1)`,
                'page title lexicon settled 1 of 2 repeated titles from 3 voices',
              ],
            },);
          },
        },),
        it({
          name: 'NAMES IN A FINDING a title number the sheet never listed, and settles nothing from that answer',
          fn: async () => {
            const { lexicon, } = await settled({
              replies: {
                [SEAT_HYPER_ONLY]: {
                  titles: [{ title: 7, rendering: 'Seventh Nap', }, { title: 0, rendering: 'No Nap', },],
                },
              },
            },);
            expect(lexicon,).toEqual({
              titles: [],
              heard: 3,
              findings: [
                `${SEAT_HYPER_ONLY}: title-index-out-of-range (7)`,
                `${SEAT_HYPER_ONLY}: title-index-out-of-range (0)`,
                'page title lexicon settled 0 of 2 repeated titles from 3 voices',
              ],
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${pageTitleLines.name} (ledger H16)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES A HEADING AND ONE LINE PER SETTLED TITLE, and nothing when none settled',
          fn: async () => {
            /**
             Lines for one settled title.
             */
            const lines = pageTitleLines({
              titles: [{ source: '猫之歌', occurrences: 3, rendering: 'Song of the Cat', voices: 2, heard: 3, },],
            },);
            expect({
              count: lines.length,
              headed: lines[0]?.startsWith('TITLES THIS PAGE REPEATS THAT THE ARCHIVE DOES NOT RENDER',),
              line: lines[1],
              none: pageTitleLines({ titles: [], },),
            },).toEqual({
              count: 2,
              headed: true,
              line: '- 猫之歌 (3 places on the page): "Song of the Cat" (2 of 3 voices)',
              none: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
