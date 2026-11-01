/**
 Tests for how a judge's reply becomes a ballot.

 WHAT THIS FILE EXISTS TO STOP. The findings guard once demanded that every
 member of `unsupported` and `dropped` name a candidate, so a judge answering
 with the offending phrases lost its entire ballot, including a choice that
 was perfectly usable. Calibration lost two of its first sixty voices that
 way. The choice is the thing the contest counts, so the wording of a finding
 may never cost a voice.

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
  buildLaneContestMessages,
  isLaneContestWire,
  readLaneContestBallot,
} from '../dist/final/node/index.mjs';
import {
  TENSE_EXPLANATION,
  tenseBlocksOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/tense-sheet.test-fixture.ts';

/**
 One contested slice whose texts hold present-tense candidates, with no death
 recorded until a case says so.
 */
const TENSE_SUBJECT = {
  lineStructured: false,
  sourceText: '猫睡了。',
  incumbentText: 'Mittens is loud.',
  repairText: 'The cat sleeps.',
  translateText: 'She naps by the window.',
};

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
      name: isLaneContestWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS findings written as phrases rather than candidate names',
          fn: async () => {
            // THE REGRESSION. This exact shape arrived from a real judge, carrying
            // `choice: "repair"`, and was thrown away whole.
            expect(isLaneContestWire({
              verdict: 'repair',
              unsupported: [ 'napping in the sun', 'a second bowl', ],
              dropped: [ 'Although', ],
              assessment: 'the translate candidate invents an afternoon',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ACCEPTS a candidate name carrying its own annotation',
          fn: async () => {
            expect(isLaneContestWire({
              verdict: 'neither',
              unsupported: [ 'repair (changes the bowl to a saucer)', ],
              dropped: [],
              assessment: 'both stray',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ACCEPTS a reply whose findings are not a list of strings, since the choice is what the contest '
            + 'counts (ledger B46)',
          fn: async () => {
            // THE SHAPE OF A FINDING NO LONGER COSTS A VOICE EITHER. This case
            // once pinned the refusal, so the reader would never receive a value
            // it could not read; the reader now reads such a field as no findings,
            // as the consolidation gate's has since 2026-08-26.
            expect(isLaneContestWire({
              verdict: 'repair',
              unsupported: [ 7, ],
              dropped: [],
              assessment: 'x',
            },),).toBe(true,);
            expect(isLaneContestWire({
              verdict: 'repair',
              unsupported: 'napping',
              dropped: [],
              assessment: 'x',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a reply naming a candidate that does not exist',
          fn: async () => {
            expect(isLaneContestWire({
              verdict: 'incumbent',
              unsupported: [],
              dropped: [],
              assessment: 'x',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES a reply it cannot read as a ballot (not an object, null, or with the choice or the reason '
            + 'missing, or a reason that is not text), and ACCEPTS the same reply whole or with a findings list missing '
            + '(ledger B46)',
          fn: async () => {
            /** The whole reply first, then that reply with one part taken away or broken. */
            const replies: readonly unknown[] = [
              { verdict: 'repair', unsupported: [], dropped: [], assessment: 'the original says the cat slept', },
              'repair',
              null,
              { unsupported: [], dropped: [], assessment: 'the original says the cat slept', },
              { verdict: 'repair', dropped: [], assessment: 'the original says the cat slept', },
              { verdict: 'repair', unsupported: [], assessment: 'the original says the cat slept', },
              { verdict: 'repair', unsupported: [], dropped: [], },
              { verdict: 'repair', unsupported: [], dropped: [], assessment: 7, },
            ];
            expect(replies.map(function usable(reply,): boolean {
              return isLaneContestWire(reply,);
            },),).toEqual([ true, false, false, false, true, true, false, false, ],);
          },
        },),
      ],
    },),

    describe({
      name: readLaneContestBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS an annotated candidate name as blaming that candidate',
          fn: async () => {
            const ballot = readLaneContestBallot({
              wire: {
                verdict: 'translate',
                unsupported: [ 'repair (invents an afternoon in the sun)', ],
                dropped: [],
                assessment: 'the original says only that the cat slept',
              },
            },);
            expect(ballot.unsupported,).toEqual([ 'repair', ],);
          },
        },),
        it({
          name: 'READS a longer word beginning with a candidate name as blaming nobody',
          fn: async () => {
            // `repairing` is a word about repairing, not the name of the repair
            // candidate. A bare prefix test would blame the wrong lane here.
            const ballot = readLaneContestBallot({
              wire: {
                verdict: 'neither',
                unsupported: [ 'repairing the fence', 'translated loosely', ],
                dropped: [],
                assessment: 'neither is worse',
              },
            },);
            expect(ballot.unsupported,).toEqual([],);
          },
        },),
        it({
          name: 'KEEPS phrase findings verbatim even when they blame no candidate',
          fn: async () => {
            const ballot = readLaneContestBallot({
              wire: {
                verdict: 'repair',
                unsupported: [ 'napping in the sun', ],
                dropped: [ 'the second bowl', ],
                assessment: 'the translate candidate adds an afternoon',
              },
            },);
            expect(ballot.choice,).toBe('repair',);
            expect(ballot.unsupported,).toEqual([],);
            expect(ballot.unsupportedRaw,).toEqual([ 'napping in the sun', ],);
            expect(ballot.dropped,).toEqual([],);
            expect(ballot.droppedRaw,).toEqual([ 'the second bowl', ],);
          },
        },),
        it({
          name: 'KEEPS the choice of a ballot whose findings are not lists of strings, and reads them as none '
            + '(ledger B46)',
          fn: async () => {
            /** A reply whose findings are a stray number and a bare phrase. */
            const reply: unknown = {
              verdict: 'repair',
              unsupported: [ 7, ],
              dropped: 'the second bowl',
              assessment: 'the translate candidate adds an afternoon',
            };
            if (!isLaneContestWire(reply,))
              throw new Error('the guard refused a reply whose choice and reason it can read',);
            /** The ballot read off it. */
            const ballot = readLaneContestBallot({ wire: reply, },);
            expect([
              ballot.choice,
              ballot.unsupported,
              ballot.unsupportedRaw,
              ballot.dropped,
              ballot.droppedRaw,
            ],).toEqual([ 'repair', [], [], [], [], ],);
          },
        },),
        it({
          name: 'READS repeated blame of one candidate as naming it once',
          fn: async () => {
            const ballot = readLaneContestBallot({
              wire: {
                verdict: 'translate',
                unsupported: [ 'repair (adds an afternoon)', 'repair (adds a sunbeam)', ],
                dropped: [],
                assessment: 'two inventions from the same candidate',
              },
            },);
            expect(ballot.unsupported,).toEqual([ 'repair', ],);
          },
        },),
        it({
          name: 'READS blame of both candidates in canonical order',
          fn: async () => {
            const ballot = readLaneContestBallot({
              wire: {
                verdict: 'neither',
                unsupported: [ 'translate: drops the bowl', 'repair: adds an afternoon', ],
                dropped: [],
                assessment: 'both stray',
              },
            },);
            expect(ballot.unsupported,).toEqual([ 'repair', 'translate', ],);
          },
        },),
      ],
    },),

    describe({
      name: buildLaneContestMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS NO TENSE BLOCK when the entry records no death, whatever the texts say',
          fn: async () => {
            expect(tenseBlocksOf({
              content: buildLaneContestMessages({ subject: TENSE_SUBJECT, },)
                .at(1,)
                ?.content ?? '',
            },),).toEqual([],);
            expect(tenseBlocksOf({
              content: buildLaneContestMessages({
                subject: {
                  ...TENSE_SUBJECT,
                  lifeEnded: false,
                  subjectNames: ['Mittens',],
                },
              },)
                .at(1,)
                ?.content ?? '',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'SHOWS THE TENSE CANDIDATES of each text that has one under its sheet label, and none for a text without, where the entry records a death and names its subject',
          fn: async () => {
            expect(tenseBlocksOf({
              content: buildLaneContestMessages({
                subject: {
                  ...TENSE_SUBJECT,
                  lifeEnded: true,
                  subjectNames: ['Mittens',],
                },
              },)
                .at(1,)
                ?.content ?? '',
            },),).toEqual([
              [
                '===== TENSE CANDIDATES =====',
                TENSE_EXPLANATION,
                'ARCHIVE RENDERING:',
                '- "Mittens is loud." (matched "Mittens is", rule copula-or-auxiliary)',
                'CANDIDATE "translate":',
                '- "She naps by the window." (matched "She naps", rule s-ending-verb)',
                '=====',
              ].join('\n',),
            ],);
          },
        },),

        it({
          name: 'LEAVES A SENTENCE UNFLAGGED that opens with the subject name when the sheet is given no name, which is the weaker detector the name closes',
          fn: async () => {
            expect(tenseBlocksOf({
              content: buildLaneContestMessages({
                subject: {
                  ...TENSE_SUBJECT,
                  lifeEnded: true,
                },
              },)
                .at(1,)
                ?.content ?? '',
            },),).toEqual([
              [
                '===== TENSE CANDIDATES =====',
                TENSE_EXPLANATION,
                'CANDIDATE "translate":',
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
              content: buildLaneContestMessages({
                subject: {
                  ...TENSE_SUBJECT,
                  incumbentText: 'The cat sleeps.',
                  translateText: 'She naps ===*=== by the window.',
                  lifeEnded: true,
                  subjectNames: ['Mittens',],
                },
              },)
                .at(1,)
                ?.content ?? '',
            },),).toEqual([
              [
                '======= TENSE CANDIDATES =======',
                TENSE_EXPLANATION,
                'CANDIDATE "translate":',
                '- "She naps ====== by the window." (matched "She naps", rule s-ending-verb)',
                '=======',
              ].join('\n',),
            ],);
          },
        },),

        it({
          name: 'SHOWS THE DISPUTE NOTE after both candidates on a disputed slice, and no such heading elsewhere (class one hundred eight, one entry\'s slice 3, 2026-09-24)',
          fn: async () => {
            // THE FAILURE THIS CLOSES. One entry's slice 3: a contest ballot
            // preferred the repair candidate for carrying a detail the original
            // never states and the repair lane's own adjudicators had accepted as
            // an invented addition.
            const subject = {
              lineStructured: false,
              sourceText: '猫睡了。',
              incumbentText: 'The cat slept after chasing a moth.',
              repairText: 'The cat chased a moth and slept.',
              translateText: 'The cat slept.',
            };
            const bare = buildLaneContestMessages({ subject, },)
              .at(1,)
              ?.content ?? '';
            expect(bare.includes('ARCHIVE RENDERING DISPUTED',),).toBe(false,);
            const asked = buildLaneContestMessages({
              subject: {
                ...subject,
                archiveDisputeNote: 'ARCHIVE RENDERING DISPUTED: the repair lane\'s adjudicators accepted 1 accuracy/addition claim(s); (1) accuracy/addition critical: The translation invents the moth.',
              },
            },)
              .at(1,)
              ?.content ?? '';
            expect(asked,).toContain('The translation invents the moth.',);
            expect(asked.indexOf('ARCHIVE RENDERING DISPUTED',),).toBeGreaterThan(asked.indexOf('CANDIDATE "translate"',),);
            // LEDGER S20: the note is fenced like every other enclosed text.
            /** Lines of the sheet, and where the note stands in them. */
            const lines = asked.split('\n',);
            const at = lines.findIndex(function isNote(line,): boolean {
              return line.endsWith('The translation invents the moth.',);
            },);
            const fence = lines[at + 1] ?? '';
            expect(fence.length,).toBeGreaterThan(0,);
            expect(lines[at - 1],).toBe(`${fence} ARCHIVE RENDERING DISPUTED ${fence}`,);
          },
        },),
        it({
          name: 'SHOWS the declared names, so an attested one is not read as an invention',
          fn: async () => {
            // THE FAILURE THIS CLOSES. Front matter is document-level and this
            // stage sees one slice, so a declared name reaches the judge only if
            // it is put here. Without it the name appears in the archive and in
            // one candidate and nowhere in the original, and calling it unsupported
            // is the correct inference from the wrong evidence.
            const messages = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: '猫睡了。',
                incumbentText: 'Mittens (Whiskers) slept.',
                repairText: 'Mittens (Whiskers) slept.',
                translateText: 'Mittens slept.',
                identityContext: 'name: 猫猫 / Mittens\nalias: Whiskers',
              },
            },);
            const asked = messages.at(1,)?.content ?? '';
            expect(asked.includes('Whiskers',),).toBe(true,);
            expect(asked.includes('DECLARED NAMES',),).toBe(true,);
          },
        },),
        it({
          name: 'SHOWS the pages the original cites with the candidate rule after the passages, and no such '
            + 'heading when it cites none (class thirty-six, 2026-09-16)',
          fn: async () => {
            // THE FAILURE THIS CLOSES. One run: the panel had the cited page and
            // rejected the addition claim, the repair lane kept the attested
            // detail, and three of five contest judges shown only the original
            // called the repair candidate unsupported for carrying it.
            const subject = {
              lineStructured: false,
              sourceText: '猫有一个弟弟。',
              incumbentText: 'Mittens has a younger brother who also naps in boxes.',
              repairText: 'Mittens has a younger brother who also naps in boxes.',
              translateText: 'Mittens has a younger brother.',
            };
            const bare = buildLaneContestMessages({ subject, },)
              .at(1,)
              ?.content ?? '';
            expect(bare.includes('CITED REFERENCES',),).toBe(false,);
            const asked = buildLaneContestMessages({
              subject: {
                ...subject,
                referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens had a younger brother who also napped in boxes.',
              },
            },)
              .at(1,)
              ?.content ?? '';
            expect(asked,).toContain('CITED REFERENCES, EVIDENCE ONLY',);
            expect(asked,).toContain('also naps in boxes',);
            expect(asked,).toContain('never count it unsupported',);
            expect(asked.indexOf('CITED REFERENCES',),).toBeGreaterThan(asked.indexOf('CANDIDATE "translate":',),);
          },
        },),
        it({
          name: 'NAMES THE LANE LACKING THE COMMUNITY RENDERING after the passages, and leaves the archive '
            + 'and the lane carrying it unnamed (owner, 2026-09-09)',
          fn: async () => {
            const messages = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: '小猫最爱的玩偶是超天酱。',
                incumbentText: 'Her favourite plush toy was KAngel.',
                repairText: 'Her favourite plush toy was KAngel.',
                translateText: 'Her favourite plush toy was Choco-chan.',
              },
            },);
            const asked = messages.at(1,)?.content ?? '';
            expect(asked,).toContain('COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',);
            expect(asked,).toContain('- CANDIDATE "translate" carries none of the community\'s renderings of 超天酱',);
            expect(asked.includes('CANDIDATE "repair" carries none',),).toBe(false,);
            expect(asked.includes('ARCHIVE RENDERING carries none',),).toBe(false,);
          },
        },),
        it({
          name: 'JUDGES FRONT MATTER AS YAML and makes source visible name authoritative',
          fn: async () => {
            const system = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: '---\nname: 猫猫\n---\n',
                incumbentText: '---\nname: EntryId\n---\n',
                repairText: '---\nname: EntryId\n---\n',
                translateText: '---\nname: Maomao\n---\n',
                syntax: 'front-matter',
              },
            },).at(0,)?.content ?? '';
            expect(system.includes('complete YAML front matter',),).toBe(true,);
            expect(system.includes('must not be replaced by an entry directory id',),).toBe(true,);
            expect(system.includes('name and info.alias are the same identity',),).toBe(true,);
            expect(system.includes('established target contributor spelling',),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES DETERMINISTICALLY REJECTED CANDIDATES before syntax panel votes',
          fn: async () => {
            const messages = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: '---\nname: 猫猫\n---\n',
                incumbentText: '---\nname: EntryId\n---\n',
                repairText: '---\nname: EntryId\n---\n',
                translateText: '---\nname: Maomao\n---\n',
                syntax: 'front-matter',
                ineligibleCandidates: [
                  'archive',
                  'repair',
                ],
              },
            },);
            const asked = messages.at(1,)?.content ?? '';
            expect(asked,).toContain('DETERMINISTIC PUBLICATION ADMISSION rejects: archive, repair',);
            expect(asked,).toContain('violating choices are excluded rather than redirected',);
          },
        },),

        it({
          name: 'OMITS the block entirely when neither document declares a name',
          fn: async () => {
            // An empty heading would read as "nothing is declared about this
            // person", which is a claim, rather than as an absent section.
            const messages = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: '猫睡了。',
                incumbentText: 'The cat slept.',
                repairText: 'The cat slept.',
                translateText: 'The cat slept soundly.',
              },
            },);
            expect((messages.at(1,)?.content ?? '').includes('DECLARED NAMES',),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES to call a passage that names nobody a dropped detail',
          fn: async () => {
            // THIS ASSERTION WAS REVERSED, deliberately. It used to require the
            // opposite, that a candidate omitting a declared name HAS dropped
            // something, and that wording was measured causing two failures: a
            // judge abstained from a whole slate because no candidate carried the
            // declared LOCATION, and a shipped rendering signed a note left by a
            // FRIEND of the deceased with the deceased's own name, alias and city.
            // The selection sheet was corrected then; this sheet, which decides
            // which LANE ships, kept the old wording and this test held it there.
            const policy = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: '猫睡了。',
                incumbentText: 'x',
                repairText: 'y',
                translateText: 'z',
              },
            },).at(0,)?.content ?? '';
            expect(policy.includes('DECLARED NAMES ARE ATTESTED FACTS',),).toBe(true,);
            expect(policy.includes('HAS dropped something',),).toBe(false,);
            expect(policy.includes('has dropped nothing',),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'buildLaneContestMessages size note',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the size note into the message when one rendering is far out of proportion, '
            + 'which is the only place the evidence can reach a judge',
          fn: async () => {
            const asked = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: SIZED_SOURCE,
                incumbentText: PAGE_HEAVY,
                repairText: PAGE_HEAVY,
                translateText: IN_PROPORTION,
              },
            },).at(1,)?.content ?? '';

            expect(asked.includes('SIZE NOTE',),).toBe(true,);
            expect(asked.includes('CANDIDATE "repair"',),).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES THE MESSAGE ALONE when every rendering is in proportion, so a judge reading a '
            + 'note knows it is about this passage rather than boilerplate',
          fn: async () => {
            const asked = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: SIZED_SOURCE,
                incumbentText: IN_PROPORTION,
                repairText: IN_PROPORTION,
                translateText: IN_PROPORTION,
              },
            },).at(1,)?.content ?? '';

            expect(asked.includes('SIZE NOTE',),).toBe(false,);
          },
        },),

        it({
          name: 'CARRIES the reading for both directions in the policy, so the note is evidence a '
            + 'judge knows how to weigh rather than a number with no rule attached',
          fn: async () => {
            const policy = buildLaneContestMessages({
              subject: {
                lineStructured: false,
                sourceText: SIZED_SOURCE,
                incumbentText: IN_PROPORTION,
                repairText: IN_PROPORTION,
                translateText: IN_PROPORTION,
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
      name: `${buildLaneContestMessages.name} with probe claims`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS corroborated added-damage claims against the repair candidate as evidence after both '
            + 'candidates, and SHOWS NOTHING of the kind when there are none (one entry, 2026-09-03: the '
            + 'contest chose a tense-damaged repair lane 7 of 7 without ever seeing the claim)',
          fn: async () => {
            /**
             Subject every case shares.
             */
            const subject = {
              lineStructured: false,
              sourceText: '猫猫在书店的阁楼里睡觉。',
              incumbentText: 'The cat slept in the bookshop attic.',
              repairText: 'The cat sleeps in the bookshop attic.',
              translateText: 'The cat dozed in the attic of the bookshop.',
            };
            /**
             User message with one claim shown.
             */
            const withClaims = buildLaneContestMessages({
              subject: {
                ...subject,
                repairDamageClaims: [
                  '- minimax-m3 [tense] on the accuracy repair quotes "sleeps": the page holds past tense',
                ],
              },
            },).at(1,)?.content ?? '';
            expect(withClaims.includes('CORROBORATED ADDED-DAMAGE CLAIMS against CANDIDATE "repair"',),).toBe(true,);
            expect(
              withClaims.includes('- minimax-m3 [tense] on the accuracy repair quotes "sleeps": the page holds past tense',),
            ).toBe(true,);
            // The block is written one line per sentence fragment, so the assertion
            // stays inside one line of it.
            expect(withClaims.includes('naturalness rewrite, which started from the repaired text',),).toBe(true,);
            // After the translate candidate, before the closing questions.
            expect(withClaims.indexOf('CORROBORATED ADDED-DAMAGE',),).toBeGreaterThan(withClaims.indexOf('CANDIDATE "translate":',),);
            expect(withClaims.indexOf('CORROBORATED ADDED-DAMAGE',),).toBeLessThan(withClaims.indexOf('Return JSON',),);

            /**
             User message with no claim.
             */
            const without = buildLaneContestMessages({ subject, },).at(1,)?.content ?? '';
            expect(without.includes('CORROBORATED ADDED-DAMAGE',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
