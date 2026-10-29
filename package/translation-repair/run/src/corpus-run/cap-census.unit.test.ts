/**
 Tests for the completion cap census (ledger P10): how it reads calls out of a
 pass-run log and how it applies the cap rule to them.

 Stream lines come from `reportStreamProgress` itself, and spend lines are
 shaped like the ones `reportSpend` writes, with model ids from the roster and
 invented numbers.
 No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMPLETION_CAP,
  MIN_PROVIDER_CALLS,
  MODEL_CARDS,
  POOLED_P90,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  capCensus,
  type CapCensusRow,
  capFlagsOf,
  CAPS_ON_WIRE_AT,
  type CapSample,
  readCapLog,
} from '../../dist/final/node/index.mjs';
import { STAMPS_NOT_WRITTEN, } from '@monochromatic-dev/translation-repair-markup/ts/iso-stamp-text.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  spendLine,
  streamLine,
} from '@monochromatic-dev/translation-repair-provider-stream/ts/corpus-run/cap-census-log.test-fixture.ts';

/**
 Seat served on both Hyper and OpenRouter, with a measured cap.
 */
const SEAT = SEAT_HYPER_OPENROUTER_UNMEASURED;

/**
 The seat's OpenRouter id, as its lines name it.
 */
const OPENROUTER_ID = MODEL_CARDS[SEAT].openrouter?.id ?? 'no openrouter block';

/**
 The seat's Hyper id, as its lines name it.
 */
const HYPER_ID = MODEL_CARDS[SEAT].hyper?.id ?? 'no hyper block';

/**
 Samples of one served id on one provider, one per completion count, all
 since the caps went on the wire.

 @param provider - provider that served them

 @param model - served id

 @param completions - completion counts, one sample each

 @returns Samples

 @example
 ```ts
 const samples = samplesOf({ provider: 'hyper', model: HYPER_ID, completions: [1, 2,], },);
 ```
 */
function samplesOf(
  {
    provider,
    model,
    completions,
  }: {
    readonly provider: CapSample['provider'];
    readonly model: string;
    readonly completions: readonly number[];
  },
): readonly CapSample[] {
  return completions.map(function sampleOf(completion,): CapSample {
    return {
      provider,
      model,
      completion,
      at: CAPS_ON_WIRE_AT,
      content: 1,
    };
  },);
}

/**
 Counts one to `n`, times a step.

 @param n - how many

 @param step - multiplier

 @returns Counts

 @example
 ```ts
 const counts = stepped({ n: 3, step: 10, },);
 ```
 */
function stepped({ n, step, }: { readonly n: number; readonly step: number; },): readonly number[] {
  return Array.from({ length: n, }, function nth(_, index,): number {
    return (index + 1) * step;
  },);
}

/**
 A census row for the flag cases, with one provider's cut counts.

 @param placeholder - whether the card names the pooled 99th

 @param ruleCap - rule's reading

 @param sinceCaps - calls since the caps

 @param atCap - of those, calls at the cap

 @returns Row

 @example
 ```ts
 const row = flagRow({ placeholder: false, ruleCap: 13_082, sinceCaps: 100, atCap: 2, },);
 ```
 */
function flagRow(
  {
    placeholder,
    ruleCap,
    sinceCaps,
    atCap,
  }: {
    readonly placeholder: boolean;
    readonly ruleCap: CapCensusRow['ruleCap'];
    readonly sinceCaps: number;
    readonly atCap: number;
  },
): CapCensusRow {
  return {
    modelId: SEAT,
    cardCap: COMPLETION_CAP[SEAT],
    placeholder,
    ruleCap,
    providers: [{
      provider: 'hyper',
      calls: sinceCaps,
      p99: COMPLETION_CAP[SEAT],
      sinceCaps,
      atCap,
      atCapWithContent: 0,
      atCapNoContent: atCap,
      atCapUnpaired: 0,
    },],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readCapLog.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PAIRS A SPEND LINE WITH ITS STREAM by label within the window, and reads what the stream delivered',
          fn: async () => {
            expect(readCapLog({
              lines: [
                streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: OPENROUTER_ID, outcome: 'completed', content: 59, },),
                spendLine({
                  stamp: '2026-09-28T10:00:00.020Z',
                  tail: `provider=openrouter model=${OPENROUTER_ID} prompt=799 completion=23 cost=0.0625 endpoint=Morph`,
                },),
              ],
            },),).toEqual({
              samples: [{
                provider: 'openrouter',
                model: OPENROUTER_ID,
                completion: 23,
                at: Date.parse('2026-09-28T10:00:00.020Z',),
                content: 59,
              },],
              unstampedLines: 0,
            },);
          },
        },),
        it({
          name: 'READS A STREAM THAT DELIVERED ONE CONTENT CHARACTER, whose line says "1 content char" since the writer '
            + 'counts its noun, and the same stream as a line logged before that said "1 content chars" (ledger B109)',
          fn: async () => {
            /**
             Line the writer writes now for a one-character stream.
             */
            const written = streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: OPENROUTER_ID, outcome: 'completed', content: 1, },);

            /**
             The same line as logged before the writer counted its noun.
             */
            const older = written.replace(' 1 content char,', ' 1 content chars,',);

            expect(written,).toContain(' 1 content char,',);
            expect(older,).toContain(' 1 content chars,',);
            for (const line of [written, older,]) {
              expect(readCapLog({
                lines: [
                  line,
                  spendLine({
                    stamp: '2026-09-28T10:00:00.020Z',
                    tail: `provider=openrouter model=${OPENROUTER_ID} prompt=799 completion=23 cost=0.0625 endpoint=Morph`,
                  },),
                ],
              },).samples[0]?.content,).toBe(1,);
            }
          },
        },),
        it({
          name: 'LEAVES A SPEND LINE UNPAIRED when no completed stream of its label is in the window: one too early, '
            + 'one cut rather than completed',
          fn: async () => {
            expect(readCapLog({
              lines: [
                streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: HYPER_ID, outcome: 'completed', content: 7, },),
                streamLine({ stamp: '2026-09-28T10:00:01.000Z', label: HYPER_ID, outcome: 'cut', content: 7, },),
                spendLine({
                  stamp: '2026-09-28T10:00:01.010Z',
                  tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=13082`,
                },),
              ],
            },)
              .samples
              .map(function contentOf(sample,) {
                return sample.content;
              },),).toEqual(['unpaired',],);
          },
        },),
        it({
          name: 'PAIRS EACH SPEND LINE WITH THE LATEST STREAM STILL WAITING when several of one label wait, and takes '
            + 'each stream once',
          fn: async () => {
            expect(readCapLog({
              lines: [
                streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: HYPER_ID, outcome: 'completed', content: 11, },),
                streamLine({ stamp: '2026-09-28T10:00:00.010Z', label: HYPER_ID, outcome: 'completed', content: 22, },),
                spendLine({
                  stamp: '2026-09-28T10:00:00.020Z',
                  tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=13`,
                },),
                spendLine({
                  stamp: '2026-09-28T10:00:00.030Z',
                  tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=14`,
                },),
              ],
            },)
              .samples
              .map(function contentOf(sample,) {
                return sample.content;
              },),).toEqual([
                22,
                11,
              ],);
          },
        },),
        it({
          name: 'LEAVES A SPEND LINE UNPAIRED when its stream\'s content count is not written in digits: an empty count '
            + 'is no stream that delivered nothing, and a sign or an exponent is no count the stream line writes '
            + '(ledger B73)',
          fn: async () => {
            expect(readCapLog({
              lines: ['', '-5', '1e3',].flatMap(function pairFor(content, second,): readonly string[] {
                return [
                  `[info] [2026-09-28T10:00:0${String(second,)}.000Z] [translation-repair] [reportStreamProgress] stream `
                    + `${HYPER_ID}: completed, elapsed 7304ms, firstByte 3644ms, firstContent 3644ms, maxGap 421ms, 2977 raw chars, `
                    + `0 unreadable frames, ${content} content chars, 0 reasoning chars`,
                  spendLine({
                    stamp: `2026-09-28T10:00:0${String(second,)}.020Z`,
                    tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=13`,
                  },),
                ];
              },),
            },)
              .samples
              .map(function contentOf(sample,) {
                return sample.content;
              },),).toEqual(['unpaired', 'unpaired', 'unpaired',],);
          },
        },),
        it({
          name: 'LEAVES A SPEND LINE UNPAIRED when its stream line is no completed stream as the writer writes one: '
            + 'its unit runs on into another word, or it carries no content count at all, where the line as written '
            + 'pairs',
          fn: async () => {
            /**
             A completed stream as the writer writes it, five content
             characters delivered.
             */
            const written = streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: OPENROUTER_ID, outcome: 'completed', content: 5, },);

            /**
             The same line with its unit run on into another word, and with
             its content field left out.
             */
            const misread = [
              written.replace(' 5 content chars,', ' 5 content charms of luck,',),
              written.replace(' 5 content chars,', '',),
            ];
            expect(misread.includes(written,),).toBe(false,);

            /**
             The call's spend line, inside the pairing window of each.
             */
            const spend = spendLine({
              stamp: '2026-09-28T10:00:00.020Z',
              tail: `provider=openrouter model=${OPENROUTER_ID} prompt=799 completion=23 cost=0.0625 endpoint=Morph`,
            },);
            expect([written, ...misread,].map(function readingOf(line,) {
              return readCapLog({ lines: [line, spend,], },);
            },),).toEqual(([5, 'unpaired', 'unpaired',] as const).map(function readingWith(content,) {
              return {
                samples: [{
                  provider: 'openrouter',
                  model: OPENROUTER_ID,
                  completion: 23,
                  at: Date.parse('2026-09-28T10:00:00.020Z',),
                  content,
                },],
                unstampedLines: 0,
              };
            },),);
          },
        },),
        it({
          name: 'LEAVES OUT A LINE WHOSE STAMP THE LOGGER DID NOT WRITE, rather than dating or pairing a call by a '
            + 'reading of text no writer here makes: a stamp without its zone reads as local time, a date alone as '
            + 'midnight, and no stamp as NaN, which pairs with nothing and compares as no time at all (ledger B73)',
          fn: async () => {
            /**
             A completed stream and its spend line for each spelling, every one
             stamped as the logger never stamps.
             */
            const lines = STAMPS_NOT_WRITTEN.flatMap(function linesFor(stamp,): readonly string[] {
              return [
                streamLine({ stamp, label: HYPER_ID, outcome: 'completed', content: 7, },),
                spendLine({
                  stamp,
                  tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=13`,
                },),
              ];
            },);
            expect(readCapLog({ lines, },),).toEqual({
              samples: [],
              unstampedLines: lines.length,
            },);
          },
        },),
        it({
          name: 'LEAVES OUT AND COUNTS A SPEND LINE THE LOGGER DID NOT PREFIX: the bare line the spend writer '
            + 'returns reads as a record, and so does the same line behind one bracket, and neither holds a second '
            + 'bracket to read a stamp from',
          fn: async () => {
            /**
             A reported call's record as the spend writer returns it, before
             any logger has put a level, a stamp and tags in front of it.
             */
            const bare = `SPEND provider=hyper model=${HYPER_ID} prompt=10 completion=13`;
            expect(readCapLog({
              lines: [
                bare,
                `[info] ${bare}`,
              ],
            },),).toEqual({
              samples: [],
              unstampedLines: 2,
            },);
          },
        },),
        it({
          name: 'LEAVES OUT A RECKONED LINE AND AN UNREPORTED COUNT, since neither is a length the wire measured',
          fn: async () => {
            expect(readCapLog({
              lines: [
                spendLine({
                  stamp: '2026-09-28T10:00:00.000Z',
                  tail: `provider=openrouter model=${OPENROUTER_ID} prompt=900 completion=40 cost=0.125 estimated=abandoned`,
                },),
                spendLine({
                  stamp: '2026-09-28T10:00:01.000Z',
                  tail: `provider=hyper model=${HYPER_ID} prompt=unreported completion=unreported`,
                },),
              ],
            },),).toEqual({
              samples: [],
              unstampedLines: 0,
            },);
          },
        },),
      ],
    },),

    describe({
      name: capCensus.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE RULE as the highest provider 99th percentile among providers with enough calls, floored at '
            + 'the pooled 90th: a provider one call short does not count',
          fn: async () => {
            /**
             Enough Hyper calls stepping up by a hundred tokens.
             */
            const hyper = samplesOf({
              provider: 'hyper',
              model: HYPER_ID,
              completions: stepped({ n: MIN_PROVIDER_CALLS, step: 100, },),
            },);

            /**
             One OpenRouter call short of counting, every one far longer.
             */
            const openrouter = samplesOf({
              provider: 'openrouter',
              model: OPENROUTER_ID,
              completions: stepped({ n: MIN_PROVIDER_CALLS - 1, step: 1_000, },).map(function longer(count,): number {
                return count + 50_000;
              },),
            },);

            /**
             Enough short calls, all under the floor.
             */
            const short = samplesOf({ provider: 'hyper', model: HYPER_ID, completions: stepped({ n: MIN_PROVIDER_CALLS, step: 1, },), },);
            expect({
              ruled: capCensus({ samples: [...hyper, ...openrouter,], },).rows.map(function ruleOf(row,) {
                return row.ruleCap;
              },),
              floored: capCensus({ samples: short, },).rows.map(function ruleOf(row,) {
                return row.ruleCap;
              },),
              tooFew: capCensus({ samples: openrouter, },).rows.map(function ruleOf(row,) {
                return row.ruleCap;
              },),
            },).toEqual({
              ruled: [Math.max(...stepped({ n: MIN_PROVIDER_CALLS, step: 100, },),),],
              floored: [POOLED_P90,],
              tooFew: ['too-few-calls',],
            },);
          },
        },),
        it({
          name: 'COUNTS CUTS SINCE THE CAPS ONLY, split by what the stream delivered: an answer, nothing, or no pair',
          fn: async () => {
            /**
             The seat's cap.
             */
            const cap = COMPLETION_CAP[SEAT];

            /**
             Calls at the cap since the caps, and one from before them.
             */
            const samples: readonly CapSample[] = [
              { provider: 'hyper', model: HYPER_ID, completion: cap, at: CAPS_ON_WIRE_AT, content: 3, },
              { provider: 'hyper', model: HYPER_ID, completion: cap, at: CAPS_ON_WIRE_AT, content: 0, },
              { provider: 'hyper', model: HYPER_ID, completion: cap, at: CAPS_ON_WIRE_AT, content: 'unpaired', },
              { provider: 'hyper', model: HYPER_ID, completion: cap - 1, at: CAPS_ON_WIRE_AT, content: 0, },
              { provider: 'hyper', model: HYPER_ID, completion: cap, at: CAPS_ON_WIRE_AT - 1, content: 0, },
            ];
            expect(capCensus({ samples, },).rows[0]?.providers,).toEqual([{
              provider: 'hyper',
              calls: 5,
              p99: cap,
              sinceCaps: 4,
              atCap: 3,
              atCapWithContent: 1,
              atCapNoContent: 1,
              atCapUnpaired: 1,
            },],);
          },
        },),
        it({
          name: 'NAMES NO ROSTER SEAT A POOLED PLACEHOLDER, none of the cards naming the pooled 99th any more, and '
            + 'not a card on the pooled floor, which is a measured result; and counts calls on ids no card names',
          fn: async () => {
            /**
             The checker seat's served id, a card that carried the pooled 99th until ledger B990.
             */
            const checkerId = MODEL_CARDS[SEAT_OPENROUTER_ONLY_CHECKER].openrouter?.id ?? 'no openrouter block';

            /**
             A seat whose card names the pooled floor, a measured result.
             */
            const flooredId = MODEL_CARDS[SEAT_OPENROUTER_ONLY].openrouter?.id ?? 'no openrouter block';

            /**
             One call of the checker, the floored seat, the measured seat, and a
             retired id.
             */
            const census = capCensus({
              samples: [
                ...samplesOf({ provider: 'openrouter', model: checkerId, completions: [10,], },),
                ...samplesOf({ provider: 'openrouter', model: flooredId, completions: [10,], },),
                ...samplesOf({ provider: 'hyper', model: HYPER_ID, completions: [10,], },),
                ...samplesOf({ provider: 'hyper', model: 'retired-whiskers-0731', completions: [10,], },),
              ],
            },);
            expect({
              placeholders: census.rows.map(function placeholderOf(row,) {
                return [row.modelId, row.placeholder,];
              },),
              offRoster: census.offRoster,
            },).toEqual({
              placeholders: [[SEAT, false,], [SEAT_OPENROUTER_ONLY, false,], [SEAT_OPENROUTER_ONLY_CHECKER, false,],],
              offRoster: 1,
            },);
          },
        },),
        it({
          name: 'NAMES A SEAT A CALLER SAYS STILL RIDES THE POOLED 99TH as a placeholder, since no card on the roster '
            + 'does once every seat with calls carries its own cap (ledger B990)',
          fn: async () => {
            /**
             The checker seat's served id.
             */
            const checkerId = MODEL_CARDS[SEAT_OPENROUTER_ONLY_CHECKER].openrouter?.id ?? 'no openrouter block';
            expect(
              capCensus({
                samples: [
                  ...samplesOf({ provider: 'openrouter', model: checkerId, completions: [10,], },),
                  ...samplesOf({ provider: 'hyper', model: HYPER_ID, completions: [10,], },),
                ],
                placeholderSeats: new Set([SEAT_OPENROUTER_ONLY_CHECKER,],),
              },).rows.map(function placeholderOf(row,) {
                return [row.modelId, row.placeholder,];
              },),
            ).toEqual([[SEAT, false,], [SEAT_OPENROUTER_ONLY_CHECKER, true,],],);
          },
        },),
      ],
    },),

    describe({
      name: capFlagsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FLAGS NOTHING where the card and its calls agree and the cuts stay under the share',
          fn: async () => {
            expect(capFlagsOf({
              row: flagRow({ placeholder: false, ruleCap: COMPLETION_CAP[SEAT], sinceCaps: MIN_PROVIDER_CALLS, atCap: 1, },),
            },),).toEqual([],);
          },
        },),
        it({
          name: 'FLAGS A PLACEHOLDER THE RULE CAN NOW READ and a rule reading off the card, but not a placeholder with '
            + 'too few calls',
          fn: async () => {
            expect({
              readable: capFlagsOf({
                row: flagRow({ placeholder: true, ruleCap: POOLED_P90, sinceCaps: 0, atCap: 0, },),
              },),
              tooFew: capFlagsOf({
                row: flagRow({ placeholder: true, ruleCap: 'too-few-calls', sinceCaps: 0, atCap: 0, },),
              },),
            },).toEqual({
              readable: ['placeholder-with-calls', 'rule-off-card',],
              tooFew: [],
            },);
          },
        },),
        it({
          name: 'FLAGS CUTS OVER THE SHARE only on a provider with enough calls to say: one cut in twenty is not five '
            + 'percent of anything the rule reads',
          fn: async () => {
            expect({
              thin: capFlagsOf({
                row: flagRow({ placeholder: false, ruleCap: COMPLETION_CAP[SEAT], sinceCaps: MIN_PROVIDER_CALLS - 1, atCap: 5, },),
              },),
              enough: capFlagsOf({
                row: flagRow({ placeholder: false, ruleCap: COMPLETION_CAP[SEAT], sinceCaps: MIN_PROVIDER_CALLS, atCap: 2, },),
              },),
            },).toEqual({
              thin: [],
              enough: ['cuts-over-share',],
            },);
          },
        },),
      ],
    },),
  ],
},);
