/**
 Tests for what makes two runs' consolidations the same consolidation.

 WHAT THESE PIN is the half of a cache that fails silently. A key that is too
 WIDE discards settled work on an unrelated change, which is expensive and
 obvious. A key that is too NARROW returns a settlement reached under a
 different question, and nothing looks wrong: the texts match, so the key
 matches, and a run reports a decision it never bought.

 The contest ballots are the case this stage adds over the others. They are
 prompt content here, shown to the producers as claims about each lane, so two
 consolidations over identical candidates and different ballots are different
 questions.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { LaneContestBallot, } from '@monochromatic-dev/translation-repair-lane-contest';
import {
  CONSOLIDATE_CACHE_VERSION,
  consolidateRunShape,
  consolidateSliceKey,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster this run seats.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 One contest ballot as the judges record them.
 */
const BALLOT: LaneContestBallot = {
  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  choice: 'repair',
  unsupported: [],
  unsupportedRaw: [],
  dropped: ['translate',],
  droppedRaw: ['translate',],
  reason: 'the repair lane keeps the clause about the window',
} as LaneContestBallot;

/**
 The slice every case keys, which each varies one field of.
 */
const SLICE = {
  runShape: consolidateRunShape({
    modelIds: ROSTER,
    declaredNamePairs: [],
  },),
  sourceText: '猫在窗边睡着了。',
  incumbentText: 'The cat slept by the window.',
  repairText: 'The cat fell asleep by the window.',
  translateText: 'The cat had fallen asleep beside the window.',
  standingText: 'The cat fell asleep by the window.',
  ballots: [BALLOT,],
  lineStructured: false,
};

await describe({
  name: consolidateSliceKey.name,
  children: [
    it({
      name: 'IS AT VERSION 34: the reader of a model\'s reply takes a value that differs from a listed enum or const '
        + 'value only in capitalization as the listed value, and the key hashes the version, the shape of the run and '
        + 'the texts, not the reader, so only this number retires a record written under the reader before',
      fn: async () => {
        expect(CONSOLIDATE_CACHE_VERSION,).toBe(34,);
      },
    },),
    it({
      name: 'GIVES ONE KEY FOR ONE QUESTION, so a resumed run finds what an earlier run settled rather '
        + 'than paying for the same slate twice',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).toBe(consolidateSliceKey({ ...SLICE, },),);
      },
    },),

    it({
      name: 'SEPARATES SLATE JUDGES from the writers they judge (ledger X11): the same writers under another '
        + 'judging bench are another question, since the judges decide which consolidation ships, and a '
        + 'settlement a bench since unseated reached must not resume',
      fn: async () => {
        expect(consolidateRunShape({
          modelIds: ROSTER,
          judgeModelIds: [ROSTER[0],],
          declaredNamePairs: [],
        },),).not.toBe(
          consolidateRunShape({
            modelIds: ROSTER,
            judgeModelIds: [ROSTER[1],],
            declaredNamePairs: [],
          },),
        );
      },
    },),

    it({
      name: 'SEPARATES FINAL POLISH CONFIGURATION from consolidation without naturalness stage',
      fn: async () => {
        expect(consolidateRunShape({
          modelIds: ROSTER,
          declaredNamePairs: [],
        },),).not.toBe(
          consolidateRunShape({
            declaredNamePairs: [],
            modelIds: ROSTER,
            polishConfig: {
              refinerModelIds: [ROSTER[0],],
              judgeModelIds: ROSTER,
              gateModelIds: ROSTER,
              declaredNames: ['Mittens',],
              definitions: '',
            },
          },),
        );
      },
    },),

    it({
      name: 'SEPARATES FRONT MATTER POLICY from identical Markdown consolidation text',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, syntax: 'front-matter', },),
        );
      },
    },),

    it({
      name: 'SEPARATES TWO SLICES THAT DIFFER ONLY IN THE CONTEST BALLOTS, which is what this key adds '
        + 'over the contest key it sits after. The ballots are prompt content: the producers are shown '
        + 'what the judges said about each lane, so the same two candidates with different arguments '
        + 'about them is a different question',
      fn: async () => {
        /**
         The same contest settled the other way.
         */
        const otherWay: LaneContestBallot = {
          ...BALLOT,
          choice: 'translate',
          dropped: ['repair',],
          droppedRaw: ['repair',],
          reason: 'the translate lane reads as English rather than as a gloss',
        };

        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, ballots: [otherWay,], },),
        );
      },
    },),

    it({
      name: 'SEPARATES A SLICE WITH NO STANDING TEXT FROM ONE THAT HAS IT, because the standing text '
        + 'is not derivable from the two lane renderings: a contest that declined leaves nothing '
        + 'standing, and the deciding half then takes a different exit entirely',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, standingText: '', },),
        );
      },
    },),

    it({
      name: 'SEPARATES TWO ROSTERS asking the same slice, since a settlement carries the voices that '
        + 'reached it and a resumed slice must not return ballots a different panel cast',
      fn: async () => {
        /**
         The same run with one more seat.
         */
        const wider = consolidateRunShape({
          declaredNamePairs: [],
          modelIds: [...ROSTER, SEAT_SYNTHETIC_VISION_WITHHELD,] as const,
        },);

        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, runShape: wider, },),
        );
      },
    },),

    it({
      name: 'SEPARATES TWO PAIRS THAT DECLARE DIFFERENT NAMES, because identity context is '
        + 'front-matter-derived prompt content that varies per pair and measurably changes the answer',
      fn: async () => {
        /**
         The same roster over a pair that declares a name.
         */
        const named = consolidateRunShape({
          declaredNamePairs: [],
          modelIds: ROSTER,
          identityContext: 'the subject goes by Whiskers',
        },);

        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, runShape: named, },),
        );
      },
    },),

    it({
      name: 'SEPARATES A PAIR SHOWN THE PAGES THE ORIGINAL CITES from one shown none, and keys an entry '
        + 'that cites nothing exactly as before (class thirty-six, 2026-09-16)',
      fn: async () => {
        /**
         The same roster over a pair whose original cites a page.
         */
        const cited = consolidateRunShape({
          declaredNamePairs: [],
          modelIds: ROSTER,
          referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens had an older sister who was also a tabby.',
        },);

        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, runShape: cited, },),
        );
        expect(consolidateSliceKey(SLICE,),).toBe(
          consolidateSliceKey({
            ...SLICE,
            runShape: consolidateRunShape({
              declaredNamePairs: [],
              modelIds: ROSTER,
              referenceContext: '',
            },),
          },),
        );
      },
    },),

    it({
      name: 'SEPARATES AN ENTRY THAT RECORDS A DEATH, and the subject it names, from one that records none, '
        + 'and keys an entry that records none exactly as before',
      fn: async () => {
        /**
         The same roster over an entry that records a death and names its subject.
         */
        const named = consolidateRunShape({
          declaredNamePairs: [],
          modelIds: ROSTER,
          lifeEnded: true,
          subjectNames: ['Mittens',],
        },);

        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, runShape: named, },),
        );
        expect(consolidateSliceKey({ ...SLICE, runShape: named, },),).not.toBe(
          consolidateSliceKey({
            ...SLICE,
            runShape: consolidateRunShape({
              declaredNamePairs: [],
              modelIds: ROSTER,
              lifeEnded: true,
              subjectNames: ['Whiskers',],
            },),
          },),
        );
        expect(consolidateSliceKey(SLICE,),).toBe(
          consolidateSliceKey({
            ...SLICE,
            runShape: consolidateRunShape({
              declaredNamePairs: [],
              modelIds: ROSTER,
              lifeEnded: false,
            },),
          },),
        );
      },
    },),

    it({
      name: 'SEPARATES AN ENTRY WHOSE SUBJECT CARRIES A DECLARED ALIAS from one that names the subject alone, since the tense candidates read every name',
      fn: async () => {
        expect(consolidateRunShape({
          declaredNamePairs: [],
          modelIds: ROSTER,
          lifeEnded: true,
          subjectNames: [
            'Mittens',
            'Whiskers',
          ],
        },),).not.toBe(consolidateRunShape({
          declaredNamePairs: [],
          modelIds: ROSTER,
          lifeEnded: true,
          subjectNames: ['Mittens',],
        },),);
      },
    },),

    it({
      name: 'SEPARATES TWO PAIRS THAT DECLARE DIFFERENT NAME PAIRS, which the standing and slate floors '
        + 'read, rather than trusting the identity context to carry them (ledger A13)',
      fn: async () => {
        /**
         The same roster over a pair whose front matter declares a name.
         */
        const declared = consolidateRunShape({
          modelIds: ROSTER,
          declaredNamePairs: [{
            source: '毛毛',
            rendering: 'Mittens',
          },],
        },);

        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, runShape: declared, },),
        );
      },
    },),

    it({
      name: 'SEPARATES A CHANGE TO THE ARCHIVE RENDERING, which the consolidation never ships. It is '
        + 'the structural standard the guard floors on, so two slices with identical candidates and '
        + 'different pages are checked against different shapes',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, incumbentText: '> The cat slept by the window.', },),
        );
      },
    },),

    it({
      name: 'SEPARATES A SLICE JUDGED WITH A WINDOW FROM ONE JUDGED WITHOUT, because from 2026-08-22 the '
        + 'judges of a consolidation are shown the passages either side. A settlement decided without them '
        + 'answered a different question, and handing it back for a windowed slice would return an answer '
        + 'to a question nobody asked',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({
            ...SLICE,
            neighbouringSourceText: '她把窗户推开了一条缝。',
          },),
        );
      },
    },),

    it({
      name: 'SEPARATES A SOURCE-ONLY WINDOW FROM AN INCUMBENT-ONLY ONE, which is why the two sides carry '
        + 'their own labels rather than sharing one. A slice can stand beside a section the archive never '
        + 'translated, so the two are independently absent, and folding them under a single label would '
        + 'let a run shown only the Chinese resume a run shown only the English',
      fn: async () => {
        expect(
          consolidateSliceKey({
            ...SLICE,
            neighbouringSourceText: 'beside it',
          },),
        ).not.toBe(
          consolidateSliceKey({
            ...SLICE,
            neighbouringIncumbentText: 'beside it',
          },),
        );
      },
    },),

    it({
      name: 'PINS THE KEY TO A LITERAL, so a change to this material has to be made on purpose. The value '
        + 'moved on 2026-08-22 for line-structure judging and on 2026-08-28 for target-authoritative '
        + 'metadata contributor spelling, final body polish, target body contributor authority, and '
        + 'source-grammar calque removal, absolute naturalness review, required correction mode, and second bounded correction, then on 2026-08-29 for '
        + 'render-aware absolute review, rejected-base correction prompts, absolute correction eligibility, exact-half participation, continuous correction, distinct confirmation responsibility, rejected-strategy threading, durable payload replay, and contributor authority flooring, '
        + 'and on 2026-09-28 for the slate judges beside the writers (ledger X11), and on 2026-10-11 for the run-off '
        + 'rules (version 21), and on 2026-10-15 for prose read from the page parse (versions 22 and 23). '
        + 'These policies are not text fields, '
        + 'so comparing two calls would not notice a stale settlement, which is why this pins a value',
      fn: async () => {
        // THE LITERAL MOVED TWICE ON 2026-08-29. Its fixture roster first
        // replaced GLM-5.2 with GLM-5.3-Flash, which invalidated its own
        // settlements. Version 10 then invalidated settlements bought before
        // corrected absolute-review, correction-generation, selection, and
        // fidelity-gate questions. Version 11 then invalidated corrections
        // whose reviewers, rewriters, and selectors were not required to
        // perform separate whole-paragraph absolute-eligibility passes or
        // confirm an acceptable final reading independently. Version 12 moves
        // every direct roster round from fixed or all-seat waits to exact half.
        // Version 13 continues correction beyond two and separates discovery
        // from acceptance-challenge review. Version 14 threads reviewed
        // rejections and durably replays raw model payloads. Version 15 floors
        // candidates on target-authoritative contributor identity. Version 16
        // continues unendorsed final selection from prior failed settlement.
        // Version 17 (2026-09-27) sends gate objections over an ineligible
        // standing to the polish as an objection correction. Version 18 ships a
        // slate declined twice there by preference, its reasons to the polish.
        // Version 19 refuses disputed wordings and keys the dispute note.
        // Version 20 carries every floor, sheet and quorum change made after 19.
        // On 2026-09-28 the run shape took the slate judges beside the writers
        // (ledger X11), which moved this literal inside 20.
        // Version 21 (2026-10-11) runs off a tied slate whose standing drew no
        // ballot, and runs off on a strict majority of the archive-flawed ballots.
        // Version 22 (2026-10-15) collapses twins and floors guillemets by the
        // prose the page parse shows, which moved this literal.
        // Version 23 (2026-10-15, n159) reads a shortcut reference and an
        // address label as no prose and floors a refused candidate that carries a
        // guillemet, which moved this literal.
        // Version 24 (2026-10-15, n166) states in the house tense rule, in the
        // very sentence that puts the life in the past, that the person's quoted
        // words keep the speaker's own tense, which moved this literal.
        // Version 25 (2026-10-18) lists the narration between marks of different
        // single-quotation pairs among the tense candidates, which moved this
        // literal.
        // Version 26 (2026-10-20) scopes reader protection to the account of how
        // a death or an attempt happened, cuts translators' notes from the
        // judges, and seats a challenge proposal two ballots named over a
        // condemned fallback, which moved this literal.
        // Version 27 (2026-10-20) cuts a translators' note alone on a quoted
        // line with its line, which moved this literal.
        // Version 28 (2026-10-20) cuts the notes between translators from the
        // sheet of the absolute naturalness review, which moved this literal.
        // Version 29 (2026-10-21) settles a polish gate tie to the polished
        // text, shows the polish gate the claims the repair panel accepted, and
        // has every ballot give its reason before its choice, which moved this
        // literal.
        // Version 30 (2026-10-22) shows the polish writers and the judges of their
        // rewrites the claims the repair panel accepted and the dispute note, as
        // the polish gate is shown them (ledger B805), which moved this literal.
        // Version 31 (2026-10-22) declares every answer schema of the settlement
        // in the order its keys sort (ledger B762), which moved this literal.
        // Version 32 (2026-10-22) settles under the quorum rule that lets a
        // short bench close one voice fewer than every live seat once the
        // straggler grace ends (ledger B816), which moved this literal to the
        // value this expectation names.
        // Version 33 (2026-10-22) moved the rendered-break counts out of the system
        // text into the user turn (R4 of the third review of the Claude route).
        // Version 34 (2026-10-23) reads a reply value that differs from a listed
        // enum or const value only in capitalization as the listed value, which
        // moved this literal.
        expect(consolidateSliceKey(SLICE,),).toBe(
          '6a36b56a69f8c798de956690d4e43fa32cb443dff77f8bad9922e57de394b1d1',
        );
      },
    },),

    it({
      name: 'SEPARATES A LINE-STRUCTURED SLICE FROM A PROSE ONE, because the producer sheet carries '
        + 'the rule against merging lines only where the rule governs. A settlement bought without '
        + 'that rule answered a different question, so replaying it for a governed slice would hand '
        + 'back a rendering nobody asked the right question about',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, lineStructured: true, },),
        );
      },
    },),

    it({
      name: 'SEPARATES A SLICE WHOSE PICTURES WERE READ FROM ONE WHOSE WERE NOT, because the readings '
        + 'go on the producer sheet as words. Two runs that read one photograph into different '
        + 'sentences asked different questions however identical their texts were, and a shared key '
        + 'would hand the second run the first run\'s answer',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({
            ...SLICE,
            pictureContext: 'the photograph shows a tabby asleep on a stack of library books',
          },),
        );
      },
    },),

    it({
      name: 'KEYS NO READINGS AND AN EMPTY READING ALIKE, which the driver depends on: it folds a slice its map never '
        + 'mentions into the empty string so the sheet and the key cannot disagree about which spelling the caller '
        + 'used. Were these two keys, every slice near no readable picture would be re-bought the first time a caller '
        + 'changed its mind about how to say nothing',
      fn: async () => {
        expect(consolidateSliceKey({ ...SLICE, pictureContext: '', },),).toBe(
          consolidateSliceKey(SLICE,),
        );
      },
    },),

    it({
      name: 'SEPARATES A SLICE WHOSE LANE TEXTS WERE OFFERED TO THE SLATE from one whose were not, '
        + 'and keys no lane texts and an empty list alike, so a settlement bought before the lanes '
        + 'joined the slate is not served back to a slate that carries them (class forty, 2026-09-17)',
      fn: async () => {
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({
            ...SLICE,
            laneTexts: [
              {
                lane: 'repair',
                text: SLICE.repairText,
              },
            ],
          },),
        );
        expect(consolidateSliceKey({ ...SLICE, laneTexts: [], },),).toBe(
          consolidateSliceKey(SLICE,),
        );
      },
    },),

    it({
      name: 'SEPARATES A SLICE THE REPAIR PANEL ACCEPTED CLAIMS ON from one it accepted none on, and keys no '
        + 'claims and an empty note alike, since the polish gate of the first reads them and the second\'s key '
        + 'is what it always was (CuspariaKLSY21 slice 0, 2026-10-21)',
      fn: async () => {
        /**
         Note naming one accepted claim.
         */
        const acceptedRepairClaimsNote = 'REPAIR PANEL CLAIMS ACCEPTED: (1) accuracy/mistranslation minor: dozed softens.';
        expect(consolidateSliceKey(SLICE,),).not.toBe(
          consolidateSliceKey({ ...SLICE, acceptedRepairClaimsNote, },),
        );
        expect(consolidateSliceKey({ ...SLICE, acceptedRepairClaimsNote: '', },),).toBe(
          consolidateSliceKey(SLICE,),
        );
        expect(consolidateSliceKey({ ...SLICE, acceptedRepairClaimsNote, },),).not.toBe(
          consolidateSliceKey({ ...SLICE, archiveDisputeNote: acceptedRepairClaimsNote, },),
        );
      },
    },),
  ],
},);
