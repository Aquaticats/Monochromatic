/**
 Tests for the availability arithmetic: what a series of meter readings says
 about how much of the time a provider could be spent on, and about the
 longest stretch it could not.

 THE CASES THAT MATTER ARE THE BOUNDS. Sampling is irregular, so an outage is
 only ever known to lie between two readings. Every test in this file checks both
 ends of that range, and the open-ended cases check that a stretch running off
 either end of the record is reported as open rather than as a number.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  countStates,
  drySpans,
  dutyCycle,
  longestDrySpan,
  seriesFor,
} from '../../dist/final/node/index.mjs';

/**
 One minute in milliseconds, so a fixture reads as minutes.
 */
const MINUTE = 60_000;

/**
 Builds a sample series for one provider from a list of states, one per
 minute, with the other provider held wet throughout.

 @param states - what the provider's meter said, minute by minute

 @returns Samples ready for `seriesFor`

 @example
 ```ts
 const samples = minuteByMinute({ states: ['wet', 'dry', 'wet',], },);
 ```
 */
function minuteByMinute(
  { states, }: { readonly states: readonly ('wet' | 'dry' | 'unreadable')[]; },
): readonly {
  at: number;
  neuralwatt: 'absent';
  synthetic: 'wet' | 'dry' | 'unreadable';
  hyper: 'wet';
  bedrock: 'absent';
  deepseek: 'absent';
  xiaomi: 'absent';
  anthropic: 'absent';
  openrouter: 'absent';
  levels: readonly string[];
}[] {
  return states.map(function toSample(state, index,) {
    return {
      at: index * MINUTE,
      // A record from before the fifth provider.
      neuralwatt: 'absent' as const,
      synthetic: state,
      hyper: 'wet' as const,
      // A record from before the third provider, which contributes no
      // reading to its series.
      bedrock: 'absent' as const,
      deepseek: 'absent' as const,
      xiaomi: 'absent' as const,
      anthropic: 'absent' as const,
      openrouter: 'absent' as const,
      // Spans are arithmetic over states; what the meters read does not enter.
      levels: [],
    };
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: drySpans.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'finds no stretch in a record where the meter never ran out',
          fn: async () => {
            /**
             Three readings, all reporting budget left.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({ states: ['wet', 'wet', 'wet',], },),
                provider: 'synthetic',
              },),
            },);

            expect(spans.length,).toBe(0,);
          },
        },),

        it({
          name: 'bounds a single dry reading by the wet readings either side of it',
          fn: async () => {
            /**
             One outage caught by exactly one reading, between two wet ones.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({ states: ['wet', 'dry', 'wet',], },),
                provider: 'synthetic',
              },),
            },);

            expect(spans.length,).toBe(1,);

            // Nothing confirms it lasted any time at all: one reading caught it.
            expect(spans[0]?.confirmedMs,).toBe(0,);

            // It began after minute 0 and ended before minute 2, so two minutes
            // is everything the record permits.
            expect(spans[0]?.boundedByMs,).toBe(2 * MINUTE,);
            expect(spans[0]?.openBefore,).toBe(false,);
            expect(spans[0]?.openAfter,).toBe(false,);
          },
        },),

        it({
          name: 'confirms the stretch between the first and last dry reading',
          fn: async () => {
            /**
             An outage caught by three consecutive readings.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({ states: ['wet', 'dry', 'dry', 'dry', 'wet',], },),
                provider: 'synthetic',
              },),
            },);

            expect(spans.length,).toBe(1,);
            expect(spans[0]?.confirmedMs,).toBe(2 * MINUTE,);
            expect(spans[0]?.boundedByMs,).toBe(4 * MINUTE,);
          },
        },),

        it({
          name: 'REFUSES to bound a stretch that runs off the start of the record',
          fn: async () => {
            /**
             A record that opens with the provider already out, so nothing says
             when the outage began.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({ states: ['dry', 'dry', 'wet',], },),
                provider: 'synthetic',
              },),
            },);

            expect(spans[0]?.openBefore,).toBe(true,);
            expect(spans[0]?.boundedByMs,).toBe(undefined,);
            expect(spans[0]?.confirmedMs,).toBe(MINUTE,);
          },
        },),

        it({
          name: 'REFUSES to bound a stretch still running at the end of the record',
          fn: async () => {
            /**
             A record that ends with the provider out, which is the live case:
             the outage may still be going.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({ states: ['wet', 'dry', 'dry',], },),
                provider: 'synthetic',
              },),
            },);

            expect(spans[0]?.openAfter,).toBe(true,);
            expect(spans[0]?.boundedByMs,).toBe(undefined,);
          },
        },),

        it({
          name: 'splits confirmation across an unreadable meter while keeping one shared bound',
          fn: async () => {
            /**
             An outage with a reading in the middle that answered nothing. The
             provider might have recovered and failed again behind it, so
             neither stretch is confirmed through it, and both are bounded by
             the same pair of wet readings.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({
                  states: ['wet', 'dry', 'unreadable', 'dry', 'wet',],
                },),
                provider: 'synthetic',
              },),
            },);

            expect(spans.length,).toBe(2,);
            expect(spans[0]?.confirmedMs,).toBe(0,);
            expect(spans[1]?.confirmedMs,).toBe(0,);

            // Both stretches lie inside minute 0 to minute 4, and neither is open.
            expect(spans[0]?.boundedByMs,).toBe(4 * MINUTE,);
            expect(spans[1]?.boundedByMs,).toBe(4 * MINUTE,);
          },
        },),
      ],
    },),

    describe({
      name: longestDrySpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ranks by what is confirmed rather than by the widest bound',
          fn: async () => {
            /**
             A short outage sampled sparsely, then a long one sampled closely.
             The first has the wider bound; the second is the longer outage.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({
                  states: ['wet', 'dry', 'wet', 'dry', 'dry', 'dry', 'wet',],
                },),
                provider: 'synthetic',
              },),
            },);

            /**
             Whichever stretch the ranking picked.
             */
            const worst = longestDrySpan({ spans, },);

            expect(worst === 'no-outage' ? worst : worst.confirmedMs,).toBe(2 * MINUTE,);
          },
        },),

        it({
          name: 'ACCEPTS a record with no outage, reporting none rather than zero',
          fn: async () => {
            expect(longestDrySpan({ spans: [], },),).toBe('no-outage',);
          },
        },),

        it({
          name: 'keeps the first outage where a later one is shorter',
          fn: async () => {
            /**
             An outage three readings confirm, open at the start of the
             record, then one a single reading caught, open at its end.
             */
            const spans = drySpans({
              series: seriesFor({
                samples: minuteByMinute({
                  states: [
                    'dry',
                    'dry',
                    'dry',
                    'wet',
                    'dry',
                  ],
                },),
                provider: 'synthetic',
              },),
            },);
            expect(longestDrySpan({ spans, },),).toEqual({
              confirmedMs: 2 * MINUTE,
              openBefore: true,
              openAfter: false,
              firstAt: 0,
              lastAt: 2 * MINUTE,
            },);
          },
        },),
      ],
    },),

    describe({
      name: dutyCycle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'counts only readings whose meter answered',
          fn: async () => {
            /**
             Four readings, one of which answered nothing. The fraction is over
             the three that did.
             */
            const counts = countStates({
              series: seriesFor({
                samples: minuteByMinute({
                  states: ['wet', 'wet', 'unreadable', 'dry',],
                },),
                provider: 'synthetic',
              },),
            },);

            expect(counts,).toEqual({
              wet: 2,
              dry: 1,
              unreadable: 1,
            },);
            expect(dutyCycle({ counts, },),).toBe(2 / 3,);
          },
        },),

        it({
          name: 'REFUSES a fraction where no reading answered at all',
          fn: async () => {
            /**
             A record where the meter endpoint was down throughout. There is no
             honest fraction to report.
             */
            const counts = countStates({
              series: seriesFor({
                samples: minuteByMinute({ states: ['unreadable', 'unreadable',], },),
                provider: 'synthetic',
              },),
            },);

            expect(dutyCycle({ counts, },),).toBe('none-answered',);
          },
        },),
      ],
    },),

    describe({
      name: seriesFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads no state for a provider a sample predates, since a record from before it existed '
            + 'says nothing about it',
          fn: async () => {
            expect(seriesFor({
              samples: minuteByMinute({
                states: [
                  'wet',
                  'dry',
                ],
              },),
              provider: 'bedrock',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
