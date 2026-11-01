/**
 Tests that every sheet deciding or writing English is told what settles tense.

 WHAT THIS FILE EXISTS TO STOP, measured rather than imagined. On the sixth
 consolidation bed run the shipped rendering of one slice moved a life told in
 the past into the present, on a page whose neighbouring chunks were both past,
 and the shipped sentence disagreed with ITSELF: it opened in the present and
 finished with a clause in the past.

 WHY NO SHEET CAUGHT IT. The rule saying a tense is forced on English by
 English, never chosen from the Chinese, closes with "hold it against a
 candidate only when the choice it made is the WRONG one, and say which reading
 the ORIGINAL supports". For tense the ORIGINAL supports NEITHER reading, so a
 judge that spotted the drift could not discharge the second half and had no
 ground to stand on. Naming the English as the authority for this one forced
 choice gives the finding its evidence.

 WHERE THE RULE LIVES AND WHY. The half needing no page, that one sentence
 holds one tense, sits in the block every tier inherits. The half needing the
 English already on the page is stated again in each producing sheet's own
 vocabulary, because those sheets label that evidence ARCHIVE RENDERING and
 EXISTING TRANSLATION rather than "the passage being replaced".

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { CONTEST_POLICY, } from '@monochromatic-dev/translation-repair-lane-contest';
import { buildTranslateMessages, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buildCandidateSelectMessages,
  buildIntroducedDefectMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildConsolidateMessages,
  type ConsolidateSubject,
} from '../dist/final/node/index.mjs';
import { systemOf, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

//region Fixtures

/**
 Opening every sheet in this file carries, so the rule found is the rule under
 test rather than one a fixture smuggled in.
 */
const TENSE_AUTHORITY =
  'WHERE THE FORCED CHOICE IS TENSE, THE ORIGINAL SUPPORTS NEITHER READING AND THE ENGLISH IS THE AUTHORITY INSTEAD';

/**
 Cat-themed source, since none of these sheets vary with what they are given.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。';

/**
 Existing English for the slice, told in the past.
 */
const EXISTING_TEXT = 'The cat slept on the windowsill.';

/**
 Subject carrying only what a consolidation call must have.
 */
const SUBJECT: ConsolidateSubject = {
  sourceText: SOURCE_TEXT,
  incumbentText: EXISTING_TEXT,
  repairText: 'The cat slept on the window sill.',
  translateText: 'The cat is napping on the ledge.',
  ballots: [],
  lineStructured: false,
};

/**
 Issue standing in for one a panel adjudicated, so the damage prober has work.
 */
const NAPPING_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/napping',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/napping',
      claim: {
        category: 'style/awkward-phrasing',
        severity: 'major',
        summary: 'Rendering flattens the cat\'s waking into a bare clause.',
        spans: [],
      },
    },
  ],
  tallies: {},
};

//endregion Fixtures

await describe({
  name: 'tense authority reaches every sheet',
  children: [
    it({
      name: 'NAMES the English as the authority to a judge choosing between candidates',
      fn: async () => {
        // Without this the judge sees drift, cannot say which reading the
        // ORIGINAL supports, and drops the finding.
        const system = systemOf({
          messages: buildCandidateSelectMessages({
            task: 'Each candidate is a rendering of the passage below.',
            criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
            evidence: [{ label: 'ORIGINAL (Chinese)', text: SOURCE_TEXT, },],
            rendered: [EXISTING_TEXT, 'The cat is napping on the sill.',],
          },),
        },);

        expect(system,).toContain(TENSE_AUTHORITY,);
        // The replaced text's own tense is no longer the authority (class
        // seventy-six, owner 2026-09-21): the house rule's past tense comes first.
        expect(system,).toContain('the past tense the house rule sets for a life that has ended',);
        expect(system,).toContain('the text being replaced had it so has kept the wrong tense',);
      },
    },),

    it({
      name: 'NAMES the English as the authority to the contest deciding which lane ships',
      fn: async () => {
        // The gate that shipped the drifting rendering reads this policy, and
        // two of its judges had voted for the standing text without it.
        expect(CONTEST_POLICY,).toContain(TENSE_AUTHORITY,);
      },
    },),

    it({
      name: 'NAMES the English as the authority to the sheet measuring damage',
      fn: async () => {
        // This sheet holds a BEFORE and an AFTER, so the authority it is told
        // to name is evidence it already has.
        const system = systemOf({
          messages: buildIntroducedDefectMessages({
            sourceText: SOURCE_TEXT,
            baselineText: EXISTING_TEXT,
            regions: [
              {
                envelopeId: 'envelope/napping',
                issueIds: ['adjudicated/napping',],
                before: 'slept on the windowsill',
                editorAfter: 'is napping on the windowsill',
              },
            ],
            issues: [NAPPING_ISSUE,],
          },).messages,
        },);

        expect(system,).toContain(TENSE_AUTHORITY,);
        // The before-edit tense is no longer the authority (class seventy-six).
        expect(system,).toContain('the past tense the house rule sets for a life that has ended',);
        expect(system,).toContain('the tense the text had before the edit is no authority',);
      },
    },),

    it({
      name: 'TELLS every tier that one sentence holds one tense',
      fn: async () => {
        // Needs no page and no neighbour, so it is stated once in the block all
        // three tiers inherit rather than three times.
        const producer = systemOf({ messages: buildConsolidateMessages({ subject: SUBJECT, },), },);
        const judge = CONTEST_POLICY;
        const measurer = systemOf({
          messages: buildIntroducedDefectMessages({
            sourceText: SOURCE_TEXT,
            baselineText: EXISTING_TEXT,
            regions: [
              {
                envelopeId: 'envelope/napping',
                issueIds: ['adjudicated/napping',],
                before: 'slept on the windowsill',
                editorAfter: 'is napping on the windowsill',
              },
            ],
            issues: [NAPPING_ISSUE,],
          },).messages,
        },);

        expect(producer,).toContain('Tense is chosen once and held.',);
        expect(judge,).toContain('Tense is chosen once and held.',);
        expect(measurer,).toContain('Tense is chosen once and held.',);
      },
    },),

    it({
      name: 'CARRIES the tense rule onto the consolidation producer in its own vocabulary',
      fn: async () => {
        // The producer that shipped the drift labels its English evidence
        // ARCHIVE RENDERING, which the inherited bullet never says.
        const system = systemOf({ messages: buildConsolidateMessages({ subject: SUBJECT, },), },);

        expect(system,).toContain('ONE TENSE, IN THIS ORDER.',);
        expect(system,).toContain('the past tense the house rule sets for the life of a person who has died',);
        expect(system,).toContain('a line of the life the ARCHIVE RENDERING tells in the present is moved to the past',);
      },
    },),

    it({
      name: 'CARRIES THE ORDERED TENSE RULE onto the translate writer, prose and verse, and never tells it to keep the '
        + 'existing translation\'s tense (the whole-package audit of 2026-09-27: class seventy-six\'s wording '
        + 'survived on this sheet beside the house rule saying the opposite)',
      fn: async () => {
        // The verse rule outranks the shape rule. It must not be read
        // as outranking this one, which is about neither shape nor lines.
        const prose = systemOf({
          messages: buildTranslateMessages({ sourceText: SOURCE_TEXT, existingText: EXISTING_TEXT, },).messages,
        },);
        const verse = systemOf({
          messages: buildTranslateMessages({
            sourceText: SOURCE_TEXT,
            existingText: EXISTING_TEXT,
            lineStructured: true,
          },).messages,
        },);

        expect(prose,).not.toContain('KEEP THE TENSE OF THE EXISTING TRANSLATION',);
        expect(verse,).not.toContain('KEEP THE TENSE OF THE EXISTING TRANSLATION',);
        expect(prose,).toContain('ONE TENSE, IN THIS ORDER.',);
        expect(prose,).toContain(
          'a line of the life the EXISTING TRANSLATION tells in the present is moved to the past',
        );
        expect(verse,).toContain('ONE TENSE, IN THIS ORDER.',);
        expect(verse,).toContain('This is not the shape rule and the verse rule does not displace it.',);
        // The rule after the apparatus clause must not reopen what the clause
        // closes: the existing translation's narrative is no licence to add.
        expect(prose,).not.toContain('and the existing translation does not already carry',);
      },
    },),
  ],
},);
