/**
 Tests for the sheet and the ballot reading of the consolidation gate.

 WHAT THIS FILE EXISTS TO STOP. The gate asks the lane contest's question over
 a different pair of names, and the pieces it shares with that contest are
 shared rather than copied. These tests hold the shared reading to the same
 behaviour on this vocabulary: a finding written as a phrase never costs a
 voice, and an annotated name still names its candidate.

 Fixtures mirror corpus structure only. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildConsolidateGateMessages,
  isConsolidateGateWire,
  readConsolidateGateBallot,
} from '../dist/final/node/index.mjs';
import {
  TENSE_EXPLANATION,
  tenseBlocksOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/tense-sheet.test-fixture.ts';

/**
 What the sheet builder takes, so a case may add the optional evidence.
 */
type GateSubject = Parameters<typeof buildConsolidateGateMessages>[0]['subject'];

/**
 Sheet note a disputed slice carries (class one hundred eight).
 */
const DISPUTE_NOTE = 'ARCHIVE RENDERING DISPUTED: the repair lane\'s adjudicators accepted 1 accuracy/addition claim(s) that the archive rendering says what the ORIGINAL never states; (1) accuracy/addition major: The translation adds that the cat climbed the drainpipe. A detail those claims name is not page content.';


/**
 One gated slice, standing in for a corpus passage.
 */
const SUBJECT: GateSubject = {
  lineStructured: false,
  sourceText: '猫在窗台上睡觉。',
  incumbentText: 'The cat sleeps on the sill, purring.',
  consolidatedText: 'The cat sleeps on the window sill.',
  standingText: 'The cat is napping on the ledge.',
};

/**
 One gated slice whose texts hold present-tense candidates, with no death
 recorded until a case says so.
 */
const TENSE_SUBJECT: GateSubject = {
  lineStructured: false,
  sourceText: '猫在窗台上睡觉。',
  incumbentText: 'Mittens is loud.',
  consolidatedText: 'The cat sleeps on the window sill.',
  standingText: 'She naps by the window.',
};

/**
 Joins the content of every message in one exchange.

 @param subject - what the judge is shown

 @returns Every message's content, joined

 @example
 ```ts
 const shown = exchangeFor({ subject: SUBJECT, },);
 ```
 */
function exchangeFor(
  { subject, }: { readonly subject: GateSubject; },
): string {
  return buildConsolidateGateMessages({ subject, },)
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

/**
 Joins only what the judge is shown, leaving out the instructions.

 SEPARATE FROM THE WHOLE EXCHANGE, because the shared policy names declared
 names as a rule whether or not this slice has any, so a test asking whether
 the block was rendered would pass on the rule's own wording.

 @param subject - what the judge is shown

 @returns User content

 @example
 ```ts
 const shown = shownFor({ subject: SUBJECT, },);
 ```
 */
function shownFor(
  { subject, }: { readonly subject: GateSubject; },
): string {
  return buildConsolidateGateMessages({ subject, },)
    .filter(function isShown(message,): boolean {
      return message.role === 'user';
    },)
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

/**
 Original long enough for the size floor to pass, so a ratio is measured
 rather than skipped.
 */
const SIZED_SOURCE = '猫睡在窗台上，看着一只蛾子飞过。'.repeat(6,);

/**
 Archive rendering far longer than that original, which is the shape a
 page-only region produces.
 */
const PAGE_HEAVY = 'the archive spells this out at length. '.repeat(30,);

/**
 Rendering in proportion to the original, at roughly three times its size.
 */
const IN_PROPORTION = 'the cat slept on the sill and watched a moth. '.repeat(6,);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isConsolidateGateWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS each name this contest allows',
          fn: async () => {
            for (const choice of [ 'consolidated', 'standing', 'neither', ])
              expect(isConsolidateGateWire({
                verdict: choice,
                unsupported: [],
                dropped: [],
                assessment: 'the original supports it',
              },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a lane name, which belongs to the other contest',
          fn: async () => {
            expect(isConsolidateGateWire({
              verdict: 'repair',
              unsupported: [],
              dropped: [],
              assessment: 'x',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'ACCEPTS findings written as phrases rather than candidate names',
          fn: async () => {
            // NO WORDING OF A FINDING MAY COST A VOICE, which is what the lane
            // contest's calibration paid two voices to learn.
            expect(isConsolidateGateWire({
              verdict: 'standing',
              unsupported: [ 'napping in the sun', ],
              dropped: [ 'the second bowl', ],
              assessment: 'x',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a reply it cannot read as a ballot (not an object, null, or with the choice or the reason '
            + 'missing, or a reason that is not text), and ACCEPTS the same reply whole or with a findings key missing '
            + '(ledger B46)',
          fn: async () => {
            /** The whole reply first, then that reply with one part taken away or broken. */
            const replies: readonly unknown[] = [
              { verdict: 'standing', unsupported: [], dropped: [], assessment: 'the original supports it', },
              'standing',
              null,
              { unsupported: [], dropped: [], assessment: 'the original supports it', },
              { verdict: 'standing', dropped: [], assessment: 'the original supports it', },
              { verdict: 'standing', unsupported: [], assessment: 'the original supports it', },
              { verdict: 'standing', unsupported: [], dropped: [], },
              { verdict: 'standing', unsupported: [], dropped: [], assessment: 7, },
            ];
            expect(replies.map(function usable(reply,): boolean {
              return isConsolidateGateWire(reply,);
            },),).toEqual([ true, false, false, false, true, true, false, false, ],);
          },
        },),
      ],
    },),

    describe({
      name: readConsolidateGateBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads an annotated name as naming that candidate',
          fn: async () => {
            const ballot = readConsolidateGateBallot({
              wire: {
                verdict: 'standing',
                unsupported: [ 'consolidated (invents an afternoon)', ],
                dropped: [],
                assessment: 'x',
              },
            },);
            expect(ballot.unsupported,).toEqual([ 'consolidated', ],);
          },
        },),
        it({
          name: 'REFUSES to read a longer word beginning with a name as that name',
          fn: async () => {
            const ballot = readConsolidateGateBallot({
              wire: {
                verdict: 'standing',
                unsupported: [ 'consolidating the two loses the ledge', ],
                dropped: [],
                assessment: 'x',
              },
            },);
            expect(ballot.unsupported,).toEqual([],);
          },
        },),
        it({
          name: 'keeps a judge\'s own words beside the narrowed list',
          fn: async () => {
            const ballot = readConsolidateGateBallot({
              wire: {
                verdict: 'consolidated',
                unsupported: [],
                dropped: [ 'the window ledge', ],
                assessment: 'the original names a sill',
              },
            },);
            expect(ballot.dropped,).toEqual([],);
            expect(ballot.droppedRaw,).toEqual([ 'the window ledge', ],);
          },
        },),
      ],
    },),

    describe({
      name: buildConsolidateGateMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names the two renderings the judge must choose between',
          fn: async () => {
            const shown = shownFor({ subject: SUBJECT, },);
            expect(shown,).toContain('CANDIDATE "consolidated"',);
            expect(shown,).toContain('CANDIDATE "standing"',);
          },
        },),
        it({
          name: 'TELLS THE JUDGE WHEN THE STANDING CANNOT SHIP, naming the deterministic refusal (class '
            + 'fifty-six, one entry\'s slice 33, 2026-09-18): a gate that kept an untranslated pronoun over a valid '
            + 'consolidation was never told the text it kept could not ship; and says nothing of it when '
            + 'the standing is eligible',
          fn: async () => {
            /**
             Why the deterministic rule refused the standing, as the run log words it.
             */
            const refusal = 'Your translation carries the pronoun untranslated as "Ta" (1 time)';
            const shown = shownFor({
              subject: {
                ...SUBJECT,
                standingRefusal: refusal,
              },
            },);
            expect(shown,).toContain('CANDIDATE "standing" CANNOT SHIP',);
            expect(shown,).toContain(refusal,);
            expect(shownFor({ subject: SUBJECT, },),).not.toContain('CANNOT SHIP',);
          },
        },),
        it({
          name: 'TELLS THE JUDGE THE CONSOLIDATION SHIPS over a standing that cannot ship (class one hundred '
            + 'eighty-five, one entry\'s slice 9, 2026-09-27): a gate preferring such a standing ships the '
            + 'slate\'s choice with the preference recorded, so a sheet saying the choice stops the entry misleads '
            + 'the judge about what the ballot does',
          fn: async () => {
            const shown = shownFor({
              subject: {
                ...SUBJECT,
                standingRefusal: 'Your translation carries the pronoun untranslated as "Ta" (1 time)',
              },
            },);
            expect(shown,).not.toContain('stops this entry',);
            expect(shown,).toContain('"consolidated" ships whichever you choose',);
          },
        },),
        it({
          name: 'shows the original as the standard and the archive as evidence',
          fn: async () => {
            const shown = exchangeFor({ subject: SUBJECT, },);
            expect(shown,).toContain('THE ORIGINAL IS THE STANDARD',);
            expect(shown,).toContain('ARCHIVE RENDERING, evidence only',);
          },
        },),
        it({
          name: 'APPLIES YAML POLICY at final consolidation gate',
          fn: async () => {
            const system = buildConsolidateGateMessages({
              subject: {
                ...SUBJECT,
                syntax: 'front-matter',
              },
            },).at(0,)?.content ?? '';
            expect(system,).toContain('complete YAML front matter',);
            expect(system,).toContain('entry directory id',);
            expect(system,).toContain('name and info.alias are the same identity',);
            expect(system,).toContain('established target contributor spelling',);
          },
        },),
        it({
          name: 'asks the two findings questions the lane contest asks',
          fn: async () => {
            const shown = exchangeFor({ subject: SUBJECT, },);
            expect(shown,).toContain('UNSUPPORTED',);
            expect(shown,).toContain('DROPPED',);
          },
        },),
        it({
          name: 'SHOWS the pages the original cites with the candidate rule after the candidates '
            + '(class thirty-six, 2026-09-16)',
          fn: async () => {
            const shown = buildConsolidateGateMessages({
              subject: {
                ...SUBJECT,
                referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens had an older sister who was also a tabby.',
              },
            },)
              .filter(function isShown(message,): boolean {
                return message.role === 'user';
              },)
              .map(function contentOf(message,): string {
                return message.content;
              },)
              .join('\n',);
            expect(shown,).toContain('CITED REFERENCES, EVIDENCE ONLY',);
            expect(shown,).toContain('also a tabby',);
            expect(shown,).toContain('never count it unsupported',);
            expect(shown.indexOf('CITED REFERENCES',),).toBeGreaterThan(shown.indexOf('CANDIDATE "standing":',),);
          },
        },),
        it({
          name: 'REFUSES to head a declared-names block when neither side declares any',
          fn: async () => {
            const shown = shownFor({ subject: SUBJECT, },);
            expect(shown.includes('DECLARED NAMES',),).toBe(false,);
            expect(shown.includes('CITED REFERENCES',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'buildConsolidateGateMessages tense candidates',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS NO TENSE BLOCK when the entry records no death, whatever the texts say',
          fn: async () => {
            expect(tenseBlocksOf({
              content: exchangeFor({ subject: TENSE_SUBJECT, },),
            },),).toEqual([],);
            expect(tenseBlocksOf({
              content: exchangeFor({
                subject: {
                  ...TENSE_SUBJECT,
                  lifeEnded: false,
                  subjectNames: ['Mittens',],
                },
              },),
            },),).toEqual([],);
          },
        },),

        it({
          name: 'SHOWS THE TENSE CANDIDATES of each text that has one under its sheet label, and none for a text without, where the entry records a death and names its subject',
          fn: async () => {
            expect(tenseBlocksOf({
              content: exchangeFor({
                subject: {
                  ...TENSE_SUBJECT,
                  lifeEnded: true,
                  subjectNames: ['Mittens',],
                },
              },),
            },),).toEqual([
              [
                '===== TENSE CANDIDATES =====',
                TENSE_EXPLANATION,
                'ARCHIVE RENDERING:',
                '- "Mittens is loud." (matched "Mittens is", rule copula-or-auxiliary)',
                'CANDIDATE "standing":',
                '- "She naps by the window." (matched "She naps", rule s-ending-verb)',
                '=====',
              ].join('\n',),
            ],);
          },
        },),

        it({
          name: 'FENCES THE TENSE BLOCK with a fence longer than any run of the fence character inside it, though reading '
            + 'a sentence drops the star between two shorter runs of it',
          fn: async () => {
            expect(tenseBlocksOf({
              content: exchangeFor({
                subject: {
                  ...TENSE_SUBJECT,
                  incumbentText: 'The cat sleeps.',
                  standingText: 'She naps ===*=== by the window.',
                  lifeEnded: true,
                  subjectNames: ['Mittens',],
                },
              },),
            },),).toEqual([
              [
                '======= TENSE CANDIDATES =======',
                TENSE_EXPLANATION,
                'CANDIDATE "standing":',
                '- "She naps ====== by the window." (matched "She naps", rule s-ending-verb)',
                '=======',
              ].join('\n',),
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'buildConsolidateGateMessages size note',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the size note into the message when one rendering is far out of proportion, '
            + 'which is the only place the evidence can reach a judge',
          fn: async () => {
            const asked = buildConsolidateGateMessages({
              subject: {
                lineStructured: false,
                sourceText: SIZED_SOURCE,
                incumbentText: PAGE_HEAVY,
                consolidatedText: PAGE_HEAVY,
                standingText: IN_PROPORTION,
              },
            },).at(1,)?.content ?? '';

            expect(asked.includes('SIZE NOTE',),).toBe(true,);
            expect(asked.includes('CANDIDATE "consolidated"',),).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES THE MESSAGE ALONE when every rendering is in proportion, so a judge reading a '
            + 'note knows it is about this passage rather than boilerplate',
          fn: async () => {
            const asked = buildConsolidateGateMessages({
              subject: {
                lineStructured: false,
                sourceText: SIZED_SOURCE,
                incumbentText: IN_PROPORTION,
                consolidatedText: IN_PROPORTION,
                standingText: IN_PROPORTION,
              },
            },).at(1,)?.content ?? '';

            expect(asked.includes('SIZE NOTE',),).toBe(false,);
          },
        },),

        it({
          name: 'CARRIES the reading for both directions in the policy, so the note is evidence a '
            + 'judge knows how to weigh rather than a number with no rule attached',
          fn: async () => {
            const policy = buildConsolidateGateMessages({
              subject: {
                lineStructured: false,
                sourceText: SIZED_SOURCE,
                incumbentText: IN_PROPORTION,
                consolidatedText: IN_PROPORTION,
                standingText: IN_PROPORTION,
              },
            },).at(0,)?.content ?? '';

            expect(policy.includes('FAR SHORTER',),).toBe(true,);
            expect(policy.includes('FAR LONGER',),).toBe(true,);
            expect(policy.includes('SIZE ALONE SETTLES NEITHER READING',),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'a list field the model did not write as a list',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS the choice and reads the field as an empty list, since a wrong type is not a wording and no '
            + 'wording of a finding may cost a voice',
          fn: async () => {
            /**
             Reply whose list fields are a null and a bare word.
             */
            const reply: unknown = {
              verdict: 'standing',
              unsupported: null,
              dropped: 'none',
              assessment: 'the original supports it',
            };

            expect(isConsolidateGateWire(reply,),).toBe(true,);
            if (!isConsolidateGateWire(reply,))
              throw new Error('the guard refused the reply it just accepted',);

            /**
             Ballot read off it.
             */
            const ballot = readConsolidateGateBallot({ wire: reply, },);

            expect(ballot.choice,).toBe('standing',);
            expect(ballot.unsupported,).toEqual([],);
            expect(ballot.unsupportedRaw,).toEqual([],);
            expect(ballot.dropped,).toEqual([],);
            expect(ballot.droppedRaw,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: 'community renderings on the gate sheet',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE RENDERING LACKING THE COMMUNITY WORD after the passages, and leaves the ones '
            + 'carrying it unnamed (owner, 2026-09-09)',
          fn: async () => {
            /** What the judge is shown over a consolidation that lost 自切. */
            const shown = exchangeFor({
              subject: {
                lineStructured: false,
                sourceText: '那只猫讲起自切的经历时，朋友们都安静地听着。',
                incumbentText: 'When the cat told of her self-surgery, her friends listened quietly.',
                consolidatedText: 'When the cat told of her operation, her friends sat quietly.',
                standingText: 'When she told of her self-surgery, her friends listened quietly.',
              },
            },);
            expect(shown,).toContain('COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',);
            expect(shown,).toContain('- CANDIDATE "consolidated" carries none of the community\'s renderings of 自切',);
            expect(shown.includes('CANDIDATE "standing" carries none',),).toBe(false,);
            expect(shown.includes('ARCHIVE RENDERING carries none',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'consolidate gate wire dispute note (class one hundred eight, one entry\'s slice 3, 2026-09-24)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS the dispute note after the candidates on a disputed slice, so keeping the stand-in\'s softened addition is not the safe choice',
          fn: async () => {
            const shown = shownFor({
              subject: {
                ...SUBJECT,
                archiveDisputeNote: DISPUTE_NOTE,
              },
            },);
            expect(shown,).toContain(DISPUTE_NOTE,);
            expect(shown.indexOf('ARCHIVE RENDERING DISPUTED',),).toBeGreaterThan(shown.indexOf('CANDIDATE "standing"',),);
            // LEDGER S20: the note is fenced like every other enclosed text.
            /** Lines of the sheet, and where the note stands in them. */
            const lines = shown.split('\n',);
            const at = lines.indexOf(DISPUTE_NOTE,);
            const fence = lines[at + 1] ?? '';
            expect(fence.length,).toBeGreaterThan(0,);
            expect(lines[at - 1],).toBe(`${fence} ARCHIVE RENDERING DISPUTED ${fence}`,);
          },
        },),
        it({
          name: 'SHOWS no dispute heading on an ordinary slice',
          fn: async () => {
            expect(shownFor({ subject: SUBJECT, },).includes('ARCHIVE RENDERING DISPUTED',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
