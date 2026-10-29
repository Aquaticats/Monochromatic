/**
 Tests for reading `SPEND` lines back out of a run log.

 THE CASE THAT BINDS THE PAIR is the round trip: a line `reportSpend` returned
 is handed straight to `readSpendLine`. Writing that case is what found the
 defect it now guards, since the marker originally carried a leading space and
 the writer's own output read as prose.

 PROSE MENTIONING THE MARKER IS NOT A RECORD, and gets its own case. A run log
 carries commit messages, task notes and test output, any of which may write
 the word. `meter-sample-read.ts` learned this from its own summary line,
 which claimed a hole in a log that had none.

 A RECORD THAT WILL NOT PARSE IS COUNTED, NOT DROPPED, which is the other half
 of the same discipline: a reader that skipped malformed records would report
 a cleaner log than the one it read, and interleaved writes from concurrent
 stages produce them.

 Model identifiers come from the catalog. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportSpend, } from '@monochromatic-dev/translation-repair-provider';
import {
  readSpendLine,
  tallySpend,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Log line as a logger writes one, tags and stamp in front of the record.

 @param tail - record text, marker word onward

 @returns Line shaped the way a run log holds it

 @example
 ```ts
 const line = logged({ tail: 'SPEND provider=hyper model=m prompt=1 completion=2', },);
 ```
 */
function logged({ tail, }: { readonly tail: string; },): string {
  return `[info] [2026-08-25T01:28:57.289Z] [translation-repair] [reportSpend] ${tail}`;
}

/**
 Record naming the metered provider, used wherever a case needs a valid one.
 */
const HYPER_TAIL = 'SPEND provider=hyper model=qwen3.8-max prompt=5120 completion=3072';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readSpendLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a record out of a logged line, which is the control the rest '
            + 'of these cases depart from one field at a time',
          fn: async () => {
            expect(readSpendLine({ line: logged({ tail: HYPER_TAIL, },), },),)
              .toEqual({
                provider: 'hyper',
                model: 'qwen3.8-max',
                prompt: 5_120,
                completion: 3_072,
                costUsd: 'unreported',
                reckoning: 'reported',
              },);
          },
        },),

        it({
          name: 'READS a line carrying the trailing endpoint field the OpenRouter writer adds, which this '
            + 'reader does not tally, so the attribution field costs the accounting nothing',
          fn: async () => {
            expect(
              readSpendLine({
                line: reportSpend({
                  provider: 'openrouter',
                  label: 'minimax/minimax-m3',
                  extracted: {
                    text: 'The cat approved this rendering.',
                    usage: {
                      prompt_tokens: 2_263,
                      completion_tokens: 117,
                    },
                  },
                  costUsd: 0.00032778,
                  endpoint: 'Google AI Studio',
                },),
              },),
            )
              .toEqual({
                provider: 'openrouter',
                model: 'minimax/minimax-m3',
                prompt: 2_263,
                completion: 117,
                costUsd: 0.00032778,
                reckoning: 'reported',
              },);
          },
        },),

        it({
          name: 'READS the cost field an OpenRouter line carries, in USD as the wire reported it, and '
            + 'round-trips it through the writer',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=openrouter model=deepseek/deepseek-v4.1-flash prompt=342 completion=400 cost=0.00015646',
                },),
              },),
            )
              .toEqual({
                provider: 'openrouter',
                model: 'deepseek/deepseek-v4.1-flash',
                prompt: 342,
                completion: 400,
                costUsd: 0.00015646,
                reckoning: 'reported',
              },);
            expect(
              readSpendLine({
                line: reportSpend({
                  provider: 'openrouter',
                  label: 'moonshotai/kimi-k3',
                  extracted: {
                    text: 'The cat approved this rendering.',
                    usage: {
                      prompt_tokens: 12,
                      completion_tokens: 34,
                    },
                  },
                  costUsd: 0.000546,
                },),
              },),
            )
              .toEqual({
                provider: 'openrouter',
                model: 'moonshotai/kimi-k3',
                prompt: 12,
                completion: 34,
                costUsd: 0.000546,
                reckoning: 'reported',
              },);
          },
        },),

        it({
          name: 'READS a line the writer returned, with no logger prefix in front '
            + 'of it, so what `reportSpend` hands back round-trips through this '
            + 'reader rather than reading as prose',
          fn: async () => {
            expect(
              readSpendLine({
                line: reportSpend({
                  provider: 'hyper',
                  label: SEAT_HYPER_VISION,
                  extracted: {
                    text: 'The cat approved this rendering.',
                    usage: {
                      prompt_tokens: 12,
                      completion_tokens: 34,
                    },
                  },
                },),
              },),
            )
              .toEqual({
                provider: 'hyper',
                model: SEAT_HYPER_VISION,
                prompt: 12,
                completion: 34,
                costUsd: 'unreported',
                reckoning: 'reported',
              },);
          },
        },),

        it({
          name: 'KEEPS a provider silence as the named absence rather than folding '
            + 'it into zero, since a run nobody metered and a run that spent nothing '
            + 'are not the same run',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=hyper model=qwen3.8-max prompt=unreported completion=unreported',
                },),
              },),
            )
              .toEqual({
                provider: 'hyper',
                model: 'qwen3.8-max',
                prompt: 'unreported',
                completion: 'unreported',
                costUsd: 'unreported',
                reckoning: 'reported',
              },);
          },
        },),

        it({
          name: 'READS a model id carrying colons, slashes and dots, which every '
            + 'Synthetic seat does',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=synthetic model=hf:zai-org/GLM-5.3-Flash prompt=1 completion=2',
                },),
              },),
            )
              .toEqual({
                provider: 'synthetic',
                model: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                prompt: 1,
                completion: 2,
                costUsd: 'unreported',
                reckoning: 'reported',
              },);
          },
        },),

        it({
          name: 'REPORTS an ordinary log line as no record at all',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({ tail: 'stream qwen3.8-max: completed, firstByte 2111ms', },),
              },),
            )
              .toBe('not-a-record',);
          },
        },),

        it({
          name: 'REPORTS prose that mentions the marker as no record, since a run '
            + 'log carries notes and test output that write the word and a hole '
            + 'claimed in a whole log is worse than a line skipped',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({ tail: 'the SPEND line above is the record for this call', },),
              },),
            )
              .toBe('not-a-record',);
          },
        },),

        it({
          name: 'REPORTS a record truncated before its model as unreadable rather '
            + 'than as no record, so an interleaved write is counted',
          fn: async () => {
            expect(readSpendLine({ line: logged({ tail: 'SPEND provider=hyper mod', },), },),)
              .toBe('unreadable',);
          },
        },),

        it({
          name: 'REPORTS a count that is not a number as unreadable, against the '
            + 'same rule that just accepted the named absence: one is a value this '
            + 'writer emits and the other is damage',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=hyper model=qwen3.8-max prompt=lots completion=3072',
                },),
              },),
            )
              .toBe('unreadable',);
          },
        },),

        it({
          name: 'REPORTS a negative count as unreadable, since a token count is a '
            + 'count and a sum over one would understate the bill',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=hyper model=qwen3.8-max prompt=-1 completion=3072',
                },),
              },),
            )
              .toBe('unreadable',);
          },
        },),

        it({
          name: 'REPORTS a negative cost as unreadable, since a cost is money spent and a sum over one would '
            + 'understate the bill',
          fn: async () => {
            expect(readSpendLine({
              line: logged({
                tail: 'SPEND provider=openrouter model=minimax/minimax-m3 prompt=5 completion=3072 cost=-0.5',
              },),
            },),).toBe('unreadable',);
          },
        },),

        it({
          name: 'REPORTS an empty count as unreadable rather than as zero, which is '
            + 'what `Number` alone would have made of it',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=hyper model=qwen3.8-max prompt= completion=3072',
                },),
              },),
            )
              .toBe('unreadable',);
          },
        },),

        it({
          name: 'REPORTS a count or a cost not spelled the way the writer spells a number as unreadable: a '
            + 'hexadecimal, an exponent or a plus sign on a count, and a hexadecimal or a trailing zero on a cost '
            + '(ledger B73)',
          fn: async () => {
            expect([
              'prompt=0x5 completion=3072',
              'prompt=5e0 completion=3072',
              'prompt=+5 completion=3072',
              'prompt=5 completion=3072 cost=0x1',
              'prompt=5 completion=3072 cost=1.50',
            ].map(function readingOf(fields,) {
              return readSpendLine({
                line: logged({ tail: `SPEND provider=openrouter model=minimax/minimax-m3 ${fields}`, },),
              },);
            },),).toEqual([
              'unreadable',
              'unreadable',
              'unreadable',
              'unreadable',
              'unreadable',
            ],);
          },
        },),

        it({
          name: 'REPORTS an unknown provider as unreadable, so a third meter added '
            + 'later cannot be silently totalled against the two that are priced',
          fn: async () => {
            expect(
              readSpendLine({
                line: logged({
                  tail: 'SPEND provider=bookshop model=qwen3.8-max prompt=1 completion=2',
                },),
              },),
            )
              .toBe('unreadable',);
          },
        },),

        it({
          name: 'REFUSES a first field that names no record, a negative count, and a record missing its prompt '
            + 'or its completion, and READS the record that carries them all',
          fn: async () => {
            expect(readSpendLine({ line: logged({ tail: 'SPEND meow', },), },),).toBe('not-a-record',);
            expect(readSpendLine({
              line: logged({ tail: 'SPEND provider=hyper model=qwen3.8-max prompt=-5 completion=3072', },),
            },),).toBe('unreadable',);
            expect(readSpendLine({
              line: logged({ tail: 'SPEND provider=hyper model=qwen3.8-max completion=3072', },),
            },),).toBe('unreadable',);
            expect(readSpendLine({
              line: logged({ tail: 'SPEND provider=hyper model=qwen3.8-max prompt=5120', },),
            },),).toBe('unreadable',);
            expect(readSpendLine({ line: logged({ tail: HYPER_TAIL, },), },),).toEqual({
              provider: 'hyper',
              model: 'qwen3.8-max',
              prompt: 5_120,
              completion: 3_072,
              costUsd: 'unreported',
              reckoning: 'reported',
            },);
          },
        },),

        it({
          name: 'REPORTS a record that writes one field twice as unreadable, whether one count is repeated or two '
            + 'interleaved records share a line, since the writer writes each field once and reading the later '
            + 'value would count one call and lose the other',
          fn: async () => {
            expect([
              `${HYPER_TAIL} prompt=9`,
              `${HYPER_TAIL} SPEND provider=hyper model=${SEAT_HYPER_VISION} prompt=7 completion=3`,
            ].map(function readingOf(tail,) {
              return readSpendLine({ line: logged({ tail, },), },);
            },),).toEqual([
              'unreadable',
              'unreadable',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: tallySpend.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SUMS calls and both token counts per seat',
          fn: async () => {
            expect(
              tallySpend({
                lines: [
                  logged({ tail: HYPER_TAIL, },),
                  logged({ tail: HYPER_TAIL, },),
                ],
              },).seats,
            )
              .toEqual([
                {
                  provider: 'hyper',
                  model: 'qwen3.8-max',
                  calls: 2,
                  promptTokens: 10_240,
                  completionTokens: 6_144,
                  unreportedCalls: 0,
                  costUsd: 0,
                  costedCalls: 0,
                  reckonedCalls: 0,
                },
              ],);
          },
        },),

        it({
          name: 'KEEPS one model served by both providers as two seats, since only '
            + 'one of the two meters is priced per token',
          fn: async () => {
            expect(
              tallySpend({
                lines: [
                  logged({ tail: 'SPEND provider=hyper model=kimi-k3 prompt=1 completion=2', },),
                  logged({ tail: 'SPEND provider=synthetic model=kimi-k3 prompt=3 completion=4', },),
                ],
              },)
                .seats
                .length,
            )
              .toBe(2,);
          },
        },),

        it({
          name: 'COUNTS an unreported call beside the totals rather than inside '
            + 'them, so a reader can see how much of the run the floor covers',
          fn: async () => {
            expect(
              tallySpend({
                lines: [
                  logged({ tail: HYPER_TAIL, },),
                  logged({
                    tail: 'SPEND provider=hyper model=qwen3.8-max prompt=unreported completion=unreported',
                  },),
                ],
              },).seats,
            )
              .toEqual([
                {
                  provider: 'hyper',
                  model: 'qwen3.8-max',
                  calls: 2,
                  promptTokens: 5_120,
                  completionTokens: 3_072,
                  unreportedCalls: 1,
                  costUsd: 0,
                  costedCalls: 0,
                  reckonedCalls: 0,
                },
              ],);
          },
        },),

        it({
          name: 'COUNTS a record that will not parse rather than dropping it, so a '
            + 'damaged log cannot report as a clean one',
          fn: async () => {
            expect(
              tallySpend({
                lines: [
                  logged({ tail: HYPER_TAIL, },),
                  logged({ tail: 'SPEND provider=hyper mod', },),
                  logged({ tail: 'an ordinary line', },),
                ],
              },).unreadableLines,
            )
              .toBe(1,);
          },
        },),

        it({
          name: 'ORDERS the seats by completion tokens, so the seat that cost the '
            + 'most reads first and thinking is what ranks it',
          fn: async () => {
            expect(
              tallySpend({
                lines: [
                  logged({ tail: 'SPEND provider=hyper model=gemma-4-26b-a4b-it prompt=9 completion=1', },),
                  logged({ tail: 'SPEND provider=hyper model=qwen3.8-max prompt=1 completion=99', },),
                ],
              },)
                .seats
                .map(function named(seat,): string {
                  return seat.model;
                },),
            )
              .toEqual([
                'qwen3.8-max',
                SEAT_HYPER_TEXT_BEDROCK,
              ],);
          },
        },),

        it({
          name: 'REPORTS no seats and no damage for a log that never called a '
            + 'model, rather than inventing a zero-cost seat',
          fn: async () => {
            expect(tallySpend({ lines: [logged({ tail: 'nothing happened', },),], },),)
              .toEqual({
                seats: [],
                unreadableLines: 0,
              },);
          },
        },),
      ],
    },),

    describe({
      name: 'reckoned spend lines (ledger P14)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS WHY A LINE IS RECKONED and counts such calls apart, since the writer marks an abandoned attempt '
            + 'and a bound with `estimated=` so a reader can total them apart, and this reader dropped the field: '
            + '2,178 reckoned lines in the run logs read as calls the wire reported',
          fn: async () => {
            /**
             Line the writer returns for an attempt it reckoned.
             */
            const reckoned = reportSpend({
              provider: 'openrouter',
              label: 'minimax/minimax-m3',
              extracted: {
                text: '',
                usage: {
                  prompt_tokens: 900,
                  completion_tokens: 40,
                },
              },
              costUsd: 0.125,
              estimated: 'abandoned',
            },);

            /**
             What the reader made of it.
             */
            const reading = readSpendLine({ line: reckoned, },);

            /**
             Seat totals over the reckoned line and one the wire reported.
             */
            const [seat,] = tallySpend({
              lines: [
                reckoned,
                logged({ tail: 'SPEND provider=openrouter model=minimax/minimax-m3 prompt=10 completion=20 cost=0.0625', },),
              ],
            },).seats;
            expect({
              reading,
              seat,
              unknownMark: readSpendLine({
                line: logged({ tail: 'SPEND provider=bedrock model=google.gemma-4-e2b prompt=1 completion=2 estimated=guessed', },),
              },),
            },).toEqual({
              reading: {
                provider: 'openrouter',
                model: 'minimax/minimax-m3',
                prompt: 900,
                completion: 40,
                costUsd: 0.125,
                reckoning: 'abandoned',
              },
              seat: {
                provider: 'openrouter',
                model: 'minimax/minimax-m3',
                calls: 2,
                promptTokens: 910,
                completionTokens: 60,
                unreportedCalls: 0,
                costUsd: 0.1875,
                costedCalls: 2,
                reckonedCalls: 1,
              },
              unknownMark: 'unreadable',
            },);
          },
        },),

        it({
          name: 'READS BACK EVERY MARK A WRITER PASSES, listed here apart from the shared list: a mutant dropping '
            + '`unreported-bound` from `SPEND_RECKONINGS` passed every runtime test, with only the type check to catch it',
          fn: async () => {
            /**
             Marks the writers pass today: `openrouter-abandoned-spend.ts` the first,
             `bedrock-client.ts` through `bedrock-bound-ledger.ts` the other two.
             */
            const written = ['abandoned', 'abandoned-bound', 'unreported-bound',] as const;
            expect(written.map(function readBack(mark,): unknown {
              /**
               What the reader made of a line carrying this mark.
               */
              const reading = readSpendLine({
                line: reportSpend({
                  provider: 'bedrock',
                  label: 'google.gemma-4-e2b',
                  extracted: {
                    text: '',
                    usage: {
                      prompt_tokens: 64,
                      completion_tokens: 8,
                    },
                  },
                  estimated: mark,
                },),
              },);
              return ((typeof reading) === 'object') ? reading : `${mark} read as ${reading}`;
            },),).toEqual(written.map(function expected(mark,): unknown {
              return {
                provider: 'bedrock',
                model: 'google.gemma-4-e2b',
                prompt: 64,
                completion: 8,
                costUsd: 'unreported',
                reckoning: mark,
              };
            },),);
          },
        },),
      ],
    },),
  ],
},);
