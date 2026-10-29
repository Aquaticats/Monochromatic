/**
 Tests for the page-level consistency pass's reply: the labels of the places,
 the shape of a ballot, and which ballots can be counted against the places a
 sheet showed.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  consistencyLabel,
  consistencyLabels,
  isConsistencyBallot,
  isConsistencyBallotWire,
  isUsableConsistencyBallot,
  readConsistencyBallotWire,
  usableConsistencyBallotFor,
} from '../dist/final/node/index.mjs';

/**
 Builds one ballot as a gather stores it, with the stored keys.

 @param over - fields this case replaces

 @returns Ballot as stored

 @example
 ```ts
 const ballot = ballotWith({ choice: 'OCCURRENCE 2', },);
 ```
 */
function ballotWith(over: Record<string, unknown> = {},): Record<string, unknown> {
  return {
    reason: 'The first rendering is the more faithful.',
    quotes: [
      { label: 'OCCURRENCE 1', quote: 'The sunny window put the kitten to sleep', },
      { label: 'OCCURRENCE 2', quote: 'the sunlight on the sill lulled the kitten to sleep', },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
    ...over,
  };
}

/**
 A well-formed reply naming the first of two places.

 @param over - fields this case replaces

 @returns Reply as a judge might send it

 @example
 ```ts
 const reply = replyWith({ choice: 'OCCURRENCE 2', },);
 ```
 */
function replyWith(over: Record<string, unknown> = {},): Record<string, unknown> {
  return {
    assessment: 'The first rendering is the more faithful.',
    candidateQuotes: [
      { label: 'OCCURRENCE 1', quote: 'The sunny window put the kitten to sleep', },
      { label: 'OCCURRENCE 2', quote: 'the sunlight on the sill lulled the kitten to sleep', },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
    ...over,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencyLabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LABELS THE PLACES from one in the order they stand, and the label list agrees',
          fn: async () => {
            expect({
              first: consistencyLabel({ position: 0, },),
              third: consistencyLabel({ position: 2, },),
              list: consistencyLabels({ count: 3, },),
              none: consistencyLabels({ count: 0, },),
            },).toEqual({
              first: 'OCCURRENCE 1',
              third: 'OCCURRENCE 3',
              list: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
              none: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: isConsistencyBallotWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS A REPLY WITH A REASON, QUOTES, A CHOICE AND KEPT LABELS, a slot being optional',
          fn: async () => {
            expect({
              plain: isConsistencyBallotWire(replyWith(),),
              slotted: isConsistencyBallotWire(replyWith({
                candidateQuotes: [{ label: 'OCCURRENCE 1', quote: 'a chubby little treasure', slot: 'chubby', },],
              },),),
              noQuotes: isConsistencyBallotWire(replyWith({ candidateQuotes: [], },),),
            },).toEqual({
              plain: true,
              slotted: true,
              noQuotes: true,
            },);
          },
        },),
        it({
          name: 'REFUSES A REPLY MISSING A FIELD OR HOLDING ONE OF THE WRONG SHAPE',
          fn: async () => {
            expect({
              notRecord: isConsistencyBallotWire(['OCCURRENCE 1',],),
              noReason: isConsistencyBallotWire(replyWith({ assessment: undefined, },),),
              reasonNumber: isConsistencyBallotWire(replyWith({ assessment: 3, },),),
              noChoice: isConsistencyBallotWire(replyWith({ choice: undefined, },),),
              quotesNotList: isConsistencyBallotWire(replyWith({ candidateQuotes: 'OCCURRENCE 1', },),),
              quoteNotRecord: isConsistencyBallotWire(replyWith({ candidateQuotes: ['x',], },),),
              quoteNoLabel: isConsistencyBallotWire(replyWith({ candidateQuotes: [{ quote: 'x', },], },),),
              quoteNoText: isConsistencyBallotWire(replyWith({ candidateQuotes: [{ label: 'OCCURRENCE 1', },], },),),
              slotNumber: isConsistencyBallotWire(replyWith({
                candidateQuotes: [{ label: 'OCCURRENCE 1', quote: 'x', slot: 1, },],
              },),),
              keepNotList: isConsistencyBallotWire(replyWith({ keep: 'OCCURRENCE 2', },),),
              keepItemNumber: isConsistencyBallotWire(replyWith({ keep: [2,], },),),
            },).toEqual({
              notRecord: false,
              noReason: false,
              reasonNumber: false,
              noChoice: false,
              quotesNotList: false,
              quoteNotRecord: false,
              quoteNoLabel: false,
              quoteNoText: false,
              slotNumber: false,
              keepNotList: false,
              keepItemNumber: false,
            },);
          },
        },),
        it({
          name: 'REFUSES A REPLY THAT STILL SPELLS THE REASON `reason` AND THE QUOTES `quotes`, since the reply keys are '
            + '`assessment` and `candidateQuotes` (ledger B762)',
          fn: async () => {
            const stored = ballotWith();
            expect(isConsistencyBallotWire(stored,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isConsistencyBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A BALLOT UNDER THE SPELLING IT WAS STORED UNDER, `reason` AND `quotes`, so a cache or an artifact '
            + 'written before the reply keys were renamed still counts (ledger B762), and refuses the reply spelling',
          fn: async () => {
            expect({
              stored: isConsistencyBallot(ballotWith(),),
              storedWithSeat: isConsistencyBallot(ballotWith({ seat: 'hyper-cat', },),),
              reply: isConsistencyBallot(replyWith(),),
              noReason: isConsistencyBallot(ballotWith({ reason: undefined, },),),
              quotesNotList: isConsistencyBallot(ballotWith({ quotes: 'OCCURRENCE 1', },),),
              keepItemNumber: isConsistencyBallot(ballotWith({ keep: [2,], },),),
              notRecord: isConsistencyBallot(['OCCURRENCE 1',],),
            },).toEqual({
              stored: true,
              storedWithSeat: true,
              reply: false,
              noReason: false,
              quotesNotList: false,
              keepItemNumber: false,
              notRecord: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: readConsistencyBallotWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STORES THE ASSESSMENT AS THE REASON AND THE CANDIDATE QUOTES AS THE QUOTES, keeping the choice and the '
            + 'kept places, so what a gather stores is spelled as before the rename',
          fn: async () => {
            expect(readConsistencyBallotWire({
              wire: {
                assessment: 'The first rendering is the more faithful.',
                candidateQuotes: [{ label: 'OCCURRENCE 1', quote: 'The sunny window', slot: 'sunny', },],
                choice: 'OCCURRENCE 1',
                keep: ['OCCURRENCE 2',],
              },
            },),).toEqual({
              reason: 'The first rendering is the more faithful.',
              quotes: [{ label: 'OCCURRENCE 1', quote: 'The sunny window', slot: 'sunny', },],
              choice: 'OCCURRENCE 1',
              keep: ['OCCURRENCE 2',],
            },);
          },
        },),
      ],
    },),

    describe({
      name: isUsableConsistencyBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS A BALLOT that chooses a shown place, keeps shown places once and quotes shown places once',
          fn: async () => {
            const labels = consistencyLabels({ count: 2, },);
            expect(isUsableConsistencyBallot({
              ballot: ballotWith({ keep: ['OCCURRENCE 2',], },) as never,
              labels,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'COUNTS NO BALLOT whose reason shows a reader nothing, since the reason comes first and is the rule',
          fn: async () => {
            const labels = consistencyLabels({ count: 2, },);
            expect({
              empty: isUsableConsistencyBallot({ ballot: ballotWith({ reason: '', },) as never, labels, },),
              spaces: isUsableConsistencyBallot({ ballot: ballotWith({ reason: '  ', },) as never, labels, },),
              invisible: isUsableConsistencyBallot({ ballot: ballotWith({ reason: '\u{200B}', },) as never, labels, },),
            },).toEqual({
              empty: false,
              spaces: false,
              invisible: false,
            },);
          },
        },),
        it({
          name: 'COUNTS NO BALLOT that chooses, keeps or quotes a place the sheet never showed',
          fn: async () => {
            const labels = consistencyLabels({ count: 2, },);
            expect({
              choice: isUsableConsistencyBallot({ ballot: ballotWith({ choice: 'OCCURRENCE 3', },) as never, labels, },),
              keep: isUsableConsistencyBallot({ ballot: ballotWith({ keep: ['OCCURRENCE 3',], },) as never, labels, },),
              quote: isUsableConsistencyBallot({
                ballot: ballotWith({ quotes: [{ label: 'OCCURRENCE 3', quote: 'x', },], },) as never,
                labels,
              },),
            },).toEqual({
              choice: false,
              keep: false,
              quote: false,
            },);
          },
        },),
        it({
          name: 'COUNTS NO BALLOT that keeps one place twice or quotes one place twice',
          fn: async () => {
            const labels = consistencyLabels({ count: 2, },);
            expect({
              keepTwice: isUsableConsistencyBallot({
                ballot: ballotWith({ keep: ['OCCURRENCE 2', 'OCCURRENCE 2',], },) as never,
                labels,
              },),
              quoteTwice: isUsableConsistencyBallot({
                ballot: ballotWith({
                  quotes: [
                    { label: 'OCCURRENCE 1', quote: 'x', },
                    { label: 'OCCURRENCE 1', quote: 'y', },
                  ],
                },) as never,
                labels,
              },),
            },).toEqual({
              keepTwice: false,
              quoteTwice: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: usableConsistencyBallotFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GUARDS THE GATHER with the shape and the count test together, so a reply that cannot be counted is asked again',
          fn: async () => {
            const validate = usableConsistencyBallotFor({ labels: consistencyLabels({ count: 2, },), },);
            expect({
              usable: validate(replyWith(),),
              unusable: validate(replyWith({ choice: 'OCCURRENCE 9', },),),
              notABallot: validate('OCCURRENCE 1',),
            },).toEqual({
              usable: true,
              unusable: false,
              notABallot: false,
            },);
          },
        },),
      ],
    },),
  ],
},);
