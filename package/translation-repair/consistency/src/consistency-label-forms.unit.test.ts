/**
 Tests for the forms a judge may write a shown label in (the lead's ruling of 2026-10-15 on the run of TianqiChen66625,
 where seven replies that named the option that won anyway were refused for writing `Occurrence 3`, `"3"` or
 `Candidate 2`): a reply that names exactly one shown label counts as that label, stored in its canonical form, for the
 choice of a run-off or of a carry and for the choice, keep and quote labels of the first round; an unknown label, a
 name that fits two labels or a number outside the shown ones is still refused.

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
  choiceTallyOf,
  type ConsistencyBallotWire,
  type ConsistencyChoiceVoice,
  consistencyLabels,
  type ConsistencyVoice,
  isUsableChoice,
  isUsableConsistencyBallot,
  majorityChoiceOf,
  quotesByLabelOf,
  usableChoiceFor,
  usableChoicesOf,
  usableConsistencyBallotFor,
  usableVoicesOf,
} from '../dist/final/node/index.mjs';

/**
 Labels the run-off showed: the two places that tied.
 */
const FINALISTS = ['OCCURRENCE 3', 'OCCURRENCE 4',] as const;

/**
 Labels a choice among candidates showed.
 */
const CANDIDATES = ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 4',] as const;

/**
 Builds one reply that only chooses.

 @param choice - what the judge wrote
 @returns Reply

 @example
 ```ts
 const reply = replyOf({ choice: 'Occurrence 3', },);
 ```
 */
function replyOf({ choice, }: { readonly choice: string; },): { readonly reason: string; readonly choice: string; } {
  return {
    reason: 'The kitten reads better here.',
    choice,
  };
}

/**
 Builds one reply that only chooses, as the judge writes it (ledger B762).

 @param choice - what the judge wrote

 @returns Reply

 @example
 ```ts
 const reply = wireReplyOf({ choice: 'Occurrence 3', },);
 ```
 */
function wireReplyOf({ choice, }: { readonly choice: string; },): { readonly assessment: string; readonly choice: string; } {
  return {
    assessment: 'The kitten reads better here.',
    choice,
  };
}

/**
 Spells a stored first-round ballot with the reply's own keys (ledger B762).

 @param ballot - ballot as stored

 @returns Reply that would have been stored as that ballot

 @example
 ```ts
 const reply = ballotWireOf({ ballot: ballotVoiceOf({ seat: 'a', choice: '2', keep: [], labels: ['1', '2',], },), },);
 ```
 */
function ballotWireOf({ ballot, }: { readonly ballot: ConsistencyVoice; },): ConsistencyBallotWire {
  return {
    assessment: ballot.reason,
    candidateQuotes: ballot.quotes,
    choice: ballot.choice,
    keep: ballot.keep,
  };
}

/**
 Builds one heard choice with its seat.

 @param seat - seat that answered
 @param choice - what the judge wrote
 @returns Choice with its seat

 @example
 ```ts
 const voice = choiceVoiceOf({ seat: 'a', choice: '3', },);
 ```
 */
function choiceVoiceOf({ seat, choice, }: { readonly seat: string; readonly choice: string; },): ConsistencyChoiceVoice {
  return {
    seat,
    ...replyOf({ choice, },),
  };
}

/**
 Builds one first-round ballot with its seat.

 @param seat - seat that answered
 @param choice - what the judge wrote as the chosen place
 @param keep - what the judge wrote as the kept places
 @param labels - what the judge wrote as the label of each quote, in order
 @returns Ballot

 @example
 ```ts
 const voice = ballotVoiceOf({ seat: 'a', choice: '2', keep: [], labels: ['1', '2',], },);
 ```
 */
function ballotVoiceOf(
  {
    seat,
    choice,
    keep,
    labels,
  }: {
    readonly seat: string;
    readonly choice: string;
    readonly keep: readonly string[];
    readonly labels: readonly string[];
  },
): ConsistencyVoice {
  return {
    seat,
    reason: 'The first place words it best.',
    quotes: labels.map(function quoteOf(label, position,) {
      return {
        label,
        quote: `The cat naps ${String(position,)}.`,
        slot: 'naps',
      };
    },),
    choice,
    keep,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: usableChoiceFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS A REPLY THAT NAMES ONE SHOWN LABEL IN ANOTHER FORM: its case, white space around it, a bare number, '
            + 'and the word with the number',
          fn: async () => {
            const validate = usableChoiceFor({ labels: FINALISTS, },);
            expect([
              'OCCURRENCE 3',
              'Occurrence 3',
              'occurrence 4',
              '  Occurrence   4\n',
              '3',
              ' 4 ',
            ].map(function accepted(choice,): boolean {
              return validate(wireReplyOf({ choice, },),);
            },),).toEqual([true, true, true, true, true, true,],);
            const validateCandidates = usableChoiceFor({ labels: CANDIDATES, },);
            expect(['Candidate 2', 'candidate 4', '2', '4',].map(function accepted(choice,): boolean {
              return validateCandidates(wireReplyOf({ choice, },),);
            },),).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES AN UNKNOWN LABEL, A NUMBER OUTSIDE THE SHOWN ONES, THE WORD OF ANOTHER SHEET AND A NAME THAT FITS NONE '
            + 'OR SEVERAL: the run-off shows only the finalists, so the number of a place that did not tie names nothing',
          fn: async () => {
            const validate = usableChoiceFor({ labels: FINALISTS, },);
            expect([
              'OCCURRENCE 1',
              'Occurrence 5',
              '1',
              '03',
              '',
              '   ',
              'Candidate 3',
              '3 or 4',
              'OCCURRENCE 3 and OCCURRENCE 4',
              'the third',
            ].map(function accepted(choice,): boolean {
              return validate(wireReplyOf({ choice, },),);
            },),).toEqual([false, false, false, false, false, false, false, false, false, false,],);
            // A sheet whose two labels share a number: the bare number fits both, so it names neither.
            const ambiguous = usableChoiceFor({ labels: ['CANDIDATE 1', 'OCCURRENCE 1',], },);
            expect([ambiguous(wireReplyOf({ choice: '1', },),), ambiguous(wireReplyOf({ choice: 'occurrence 1', },),),],).toEqual([
              false,
              true,
            ],);
          },
        },),
      ],
    },),
    describe({
      name: isUsableChoice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS A CHOICE IN ANOTHER FORM OF A SHOWN LABEL, and still wants a reason a reader can see',
          fn: async () => {
            expect([
              isUsableChoice({ ballot: replyOf({ choice: 'Candidate 2', },), labels: CANDIDATES, },),
              isUsableChoice({ ballot: replyOf({ choice: '4', },), labels: CANDIDATES, },),
              isUsableChoice({ ballot: replyOf({ choice: '3', },), labels: CANDIDATES, },),
              isUsableChoice({ ballot: { reason: '  ', choice: '2', }, labels: CANDIDATES, },),
            ],).toEqual([true, true, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: usableChoicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS BACK EACH COUNTED CHOICE IN THE CANONICAL FORM OF ITS LABEL, beside its seat and its reason',
          fn: async () => {
            const usable = usableChoicesOf({
              voices: [
                choiceVoiceOf({ seat: 'mercury', choice: 'Occurrence 3', },),
                choiceVoiceOf({ seat: 'glm', choice: '3', },),
                choiceVoiceOf({ seat: 'kimi', choice: 'OCCURRENCE 4', },),
                choiceVoiceOf({ seat: 'stray', choice: 'OCCURRENCE 1', },),
              ],
              labels: FINALISTS,
            },);
            expect(usable,).toEqual([
              choiceVoiceOf({ seat: 'mercury', choice: 'OCCURRENCE 3', },),
              choiceVoiceOf({ seat: 'glm', choice: 'OCCURRENCE 3', },),
              choiceVoiceOf({ seat: 'kimi', choice: 'OCCURRENCE 4', },),
            ],);
          },
        },),
      ],
    },),
    describe({
      name: choiceTallyOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE FORMS OF ONE LABEL AS ONE LABEL, so the run-off of group 1 reads 3 of 4 for OCCURRENCE 3 whichever way '
            + 'each seat wrote it',
          fn: async () => {
            expect(
              choiceTallyOf({
                voices: [
                  choiceVoiceOf({ seat: 'qwen', choice: 'OCCURRENCE 3', },),
                  choiceVoiceOf({ seat: 'mercury', choice: 'Occurrence 3', },),
                  choiceVoiceOf({ seat: 'glm', choice: '3', },),
                  choiceVoiceOf({ seat: 'deepseek', choice: ' occurrence 4 ', },),
                ],
                labels: FINALISTS,
              },),
            ).toEqual({ usable: 4, chosen: ['OCCURRENCE 3',], },);
          },
        },),
      ],
    },),
    describe({
      name: usableConsistencyBallotFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS A BALLOT WHOSE CHOICE, KEPT PLACES AND QUOTE LABELS NAME SHOWN PLACES IN OTHER FORMS',
          fn: async () => {
            const validate = usableConsistencyBallotFor({ labels: consistencyLabels({ count: 3, },), },);
            expect(
              validate(ballotWireOf({
                ballot: ballotVoiceOf({
                  seat: 'minimax',
                  choice: 'occurrence 2',
                  keep: ['1', ' Occurrence 3 ',],
                  labels: ['1', ' OCCURRENCE 2 ', 'occurrence 3',],
                },),
              },),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES A BALLOT THAT NAMES AN UNKNOWN PLACE, OR THE SAME PLACE TWICE IN TWO FORMS',
          fn: async () => {
            const labels = consistencyLabels({ count: 3, },);
            const validate = usableConsistencyBallotFor({ labels, },);
            expect([
              ballotVoiceOf({ seat: 'a', choice: '9', keep: [], labels: ['1',], },),
              ballotVoiceOf({ seat: 'a', choice: '1', keep: ['4',], labels: ['1',], },),
              ballotVoiceOf({ seat: 'a', choice: '1', keep: [], labels: ['Candidate 1',], },),
              ballotVoiceOf({ seat: 'a', choice: '1', keep: ['2', 'OCCURRENCE 2',], labels: ['1',], },),
              ballotVoiceOf({ seat: 'a', choice: '1', keep: [], labels: ['2', 'occurrence 2',], },),
            ].map(function accepted(ballot,): boolean {
              return validate(ballotWireOf({ ballot, },),);
            },),).toEqual([false, false, false, false, false,],);
            expect(isUsableConsistencyBallot({
              ballot: ballotVoiceOf({ seat: 'a', choice: '1', keep: ['2',], labels: ['1', '2',], },),
              labels,
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: usableVoicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS BACK EACH COUNTED BALLOT WITH ITS CHOICE, KEPT PLACES AND QUOTE LABELS IN CANONICAL FORM, keeping its seat and '
            + 'its slot, so the majority of the choice and the grouping of the quotes by place read one label per place',
          fn: async () => {
            const labels = consistencyLabels({ count: 3, },);
            const usable = usableVoicesOf({
              voices: [
                ballotVoiceOf({ seat: 'kimi', choice: '1', keep: ['2',], labels: ['1', '2',], },),
                ballotVoiceOf({ seat: 'minimax', choice: 'occurrence 1', keep: [' occurrence 3',], labels: [' Occurrence 1', '2',], },),
                ballotVoiceOf({ seat: 'mercury', choice: 'OCCURRENCE 2', keep: [], labels: ['OCCURRENCE 1',], },),
                ballotVoiceOf({ seat: 'stray', choice: '7', keep: [], labels: [], },),
              ],
              labels,
            },);
            expect(usable.map(function summary(voice,): unknown {
              return {
                seat: voice.seat,
                choice: voice.choice,
                keep: voice.keep,
                labels: voice.quotes.map(function labelOf(quote,): string {
                  return quote.label;
                },),
                slots: voice.quotes.map(function slotOf(quote,): string {
                  return quote.slot ?? '(no slot)';
                },),
              };
            },),).toEqual([
              { seat: 'kimi', choice: 'OCCURRENCE 1', keep: ['OCCURRENCE 2',], labels: ['OCCURRENCE 1', 'OCCURRENCE 2',], slots: ['naps', 'naps',], },
              {
                seat: 'minimax',
                choice: 'OCCURRENCE 1',
                keep: ['OCCURRENCE 3',],
                labels: ['OCCURRENCE 1', 'OCCURRENCE 2',],
                slots: ['naps', 'naps',],
              },
              { seat: 'mercury', choice: 'OCCURRENCE 2', keep: [], labels: ['OCCURRENCE 1',], slots: ['naps',], },
            ],);
            expect(majorityChoiceOf({ voices: usable, },),).toEqual(['OCCURRENCE 1',],);
            expect([...quotesByLabelOf({ voices: usable, },).keys(),],).toEqual(['OCCURRENCE 1', 'OCCURRENCE 2',],);
          },
        },),
      ],
    },),
  ],
},);
