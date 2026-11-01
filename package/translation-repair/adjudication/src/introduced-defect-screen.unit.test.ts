/**
 Tests for the deterministic half of the introduced-defect probe: what a
 prober's quote actually proves about the region it names.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { flattenSpace, } from '@monochromatic-dev/translation-repair-house-rule';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type IntroducedDefectCheckWire,
  type RepairRegion,
  resolveProberChecks,
  screenEvidence,
  screenIntroducedDefects,
} from '../dist/final/node/index.mjs';

/**
 Region whose replacement fixed a tense error and, for the damage fixtures,
 dropped the second clause.
 */
const REGION: RepairRegion = {
  envelopeId: 'envelope/nap',
  issueIds: ['adjudicated/nap',],
  before: 'The cat is doing the sleeping, and she wakes at dusk.',
  editorAfter: 'The cat sleeps.',
};

/**
 Builds one prober check with empty text fields unless overridden.

 @param verdict - closed-vocabulary verdict, or a wire fault to be dropped

 @param evidence - wording quoted from the replacement

 @param omittedText - wording quoted from the text the replacement replaced

 @param region - one-based region number on the sheet

 @returns Check the screen reads

 @example
 ```ts
 const check = catCheck({ verdict: 'uncertain', },);
 ```
 */
function catCheck(
  {
    verdict,
    evidence = '',
    omittedText = '',
    region = 1,
  }: {
    readonly verdict: string;
    readonly evidence?: string;
    readonly omittedText?: string;
    readonly region?: number;
  },
): IntroducedDefectCheckWire {
  return {
    region,
    verdict,
    category: 'omission',
    severity: 'major',
    evidence,
    omittedText,
    verdictReason: 'the second clause is gone',
  };
}

/**
 Wording the critic objected to, quoted target-side, as an accepted issue the
 region serves.

 Only the fields `collectPriorQuotes` reads are populated. Widening it would
 make the fixture harder to read without testing anything more.

 @param quotedText - target-side wording the critic complained about

 @returns Issue shaped as the screen reads it

 @example
 ```ts
 const issues = [catIssue({ quotedText: 'is doing the sleeping', },),];
 ```
 */
function catIssue(
  { quotedText, }: { readonly quotedText: string; },
): AdjudicatedIssue {
  return {
    issueId: 'adjudicated/nap',
    claims: [
      {
        claim: {
          spans: [
            {
              side: 'target',
              quotedText,
            },
          ],
        },
      },
    ],
  } as unknown as AdjudicatedIssue;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: screenEvidence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'corroborates a quote that appears in the replacement and not in '
            + 'the text it replaced, which is the only differential the probe can '
            + 'establish mechanically',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: 'The cat sleeps.',
                omittedText: '',
                region: REGION,
              },),
            ).toBe('corroborated',);
          },
        },),

        it({
          name: 'corroborates a quote copied across a note between translators in the replacement, '
            + 'because the prober was shown the replacement without the note and the quote is found there',
          fn: async () => {
            /** Region whose replacement keeps a note that ends its first line. */
            const noted: RepairRegion = {
              envelopeId: 'envelope/nap',
              issueIds: ['adjudicated/nap',],
              before: 'The cat slept.\nShe woke at dusk.',
              editorAfter: 'The cat sleeps.{/* 这句不太对劲 */}\nShe wakes at noon.',
            };
            expect(
              screenEvidence({
                evidence: 'The cat sleeps.\nShe wakes at noon.',
                omittedText: '',
                region: noted,
                original: '猫睡了。她黄昏醒来。',
              },),
            ).toBe('corroborated',);
          },
        },),

        it({
          name: 'corroborates the removal of wording copied across a note between translators in the replaced text',
          fn: async () => {
            /** Region whose replaced text held a note that ended its first line. */
            const noted: RepairRegion = {
              envelopeId: 'envelope/nap',
              issueIds: ['adjudicated/nap',],
              before: 'The cat slept.{/* 这句不太对劲 */}\nShe woke at dusk.',
              editorAfter: 'The cat slept.',
            };
            expect(
              screenEvidence({
                evidence: '',
                omittedText: 'The cat slept.\nShe woke at dusk.',
                region: noted,
                original: '猫睡了。她黄昏醒来。',
              },),
            ).toBe('removal-corroborated',);
          },
        },),

        it({
          name: 'leaves a quote across a note unanchored when no original is given, since the note then '
            + 'stands in the text it reads',
          fn: async () => {
            /** Region whose replacement keeps a note that ends its first line. */
            const noted: RepairRegion = {
              envelopeId: 'envelope/nap',
              issueIds: ['adjudicated/nap',],
              before: 'The cat slept.\nShe woke at dusk.',
              editorAfter: 'The cat sleeps.{/* 这句不太对劲 */}\nShe wakes at noon.',
            };
            expect(
              screenEvidence({
                evidence: 'The cat sleeps.\nShe wakes at noon.',
                omittedText: '',
                region: noted,
              },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'contradicts a quote that already occurred before the edit, '
            + 'because replacing text cannot introduce wording the text already had',
          fn: async () => {
            // "The cat" is in both sides, so no replacement could have introduced
            // it. This is the screen's whole job: it dismisses an impossible claim
            // without needing to judge translation quality.
            expect(
              screenEvidence({
                evidence: 'The cat',
                omittedText: '',
                region: REGION,
              },),
            ).toBe('contradicted',);
          },
        },),

        it({
          name: 'leaves a quote that appears in neither side unanchored rather '
            + 'than dismissing it, since a mangled quote is not evidence of '
            + 'innocence',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: 'the dog barks',
                omittedText: '',
                region: REGION,
              },),
            ).toBe('unanchored',);
            expect(
              screenEvidence({
                evidence: '',
                omittedText: '',
                region: REGION,
              },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'corroborates dropped content quoted from the text the edit '
            + 'replaced, which is the only way an omission can ever be proved: its '
            + 'absence IS the defect, so there is nothing in the new text to quote',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: '',
                omittedText: 'and she wakes at dusk',
                region: REGION,
              },),
            ).toBe('removal-corroborated',);
          },
        },),

        it({
          name: 'contradicts a dropped-content claim whose wording is still there '
            + 'after the edit, the mirror of the added-damage contradiction',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: '',
                omittedText: 'The cat',
                region: REGION,
              },),
            ).toBe('contradicted',);
          },
        },),

        it({
          name: 'refuses a dropped-content claim quoting wording that was never in '
            + 'the replaced text, since nothing proves it was ever there to drop',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: '',
                omittedText: 'and she hunts at dawn',
                region: REGION,
              },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'refuses a claim anchored in both directions at once, because '
            + 'screening each and taking the better answer would let a prober '
            + 'launder a contradicted quote by attaching a second one',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: 'The cat sleeps.',
                omittedText: 'and she wakes at dusk',
                region: REGION,
              },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'READS A SIDE THAT SHOWS A READER NOTHING AS NO ANCHOR (ledgers B40 and B128): a claim is '
            + 'judged on its one visible side where the other holds only a zero-width space or a Hangul '
            + 'filler, and two sides that show nothing anchor on nothing',
          fn: async () => {
            /**
             Verdicts for a claim of added wording and one of dropped wording,
             the other side of each holding only one invisible character.
             */
            const verdicts = [
              '\u{200B}',
              '\u{3164}',
            ].map(function screened(invisible,) {
              return [
                screenEvidence({
                  evidence: 'The cat sleeps.',
                  omittedText: invisible,
                  region: REGION,
                },),
                screenEvidence({
                  evidence: invisible,
                  omittedText: 'and she wakes at dusk',
                  region: REGION,
                },),
              ];
            },);
            expect(verdicts,).toEqual([
              [
                'corroborated',
                'removal-corroborated',
              ],
              [
                'corroborated',
                'removal-corroborated',
              ],
            ],);
            expect(
              screenEvidence({
                evidence: '\u{200B}',
                omittedText: '\u{3164}',
                region: REGION,
              },),
            ).toBe('unanchored',);
          },
        },),

        it({
          name: 'proves a deletion that emptied its region entirely, the case a '
            + 'forward-only screen could never have anchored',
          fn: async () => {
            /** Region the editors emptied outright. */
            const deleted: RepairRegion = {
              envelopeId: 'envelope/gone',
              issueIds: [],
              before: 'She wakes at dusk.',
              editorAfter: '',
            };
            expect(
              screenEvidence({
                evidence: '',
                omittedText: 'She wakes at dusk.',
                region: deleted,
              },),
            ).toBe('removal-corroborated',);
          },
        },),

        it({
          name: 'matches across differing whitespace, so a quote rewrapped by the '
            + 'model still resolves against the region',
          fn: async () => {
            expect(
              screenEvidence({
                evidence: 'The\n  cat   sleeps.',
                omittedText: '',
                region: REGION,
              },),
            ).toBe('corroborated',);
            expect(flattenSpace({ text: '  The  cat\n\tnaps  ', },),).toBe('The cat naps',);
          },
        },),

        it({
          name: 'MATCHES A QUOTE WHOSE MARKS DIFFER FROM THE REGION\'S (ledger B24): a straight apostrophe or straight '
            + 'quotes against the edit\'s curly ones, added or dropped, as it already matches across whitespace',
          fn: async () => {
            /** Region written with curly marks on both sides. */
            const curly: RepairRegion = {
              envelopeId: 'envelope/curly',
              issueIds: [],
              before: 'She didn’t wake at dusk.',
              editorAfter: 'The cat’s “nap” ran long.',
            };
            expect(
              screenEvidence({
                evidence: 'The cat\'s "nap"',
                omittedText: '',
                region: curly,
              },),
            ).toBe('corroborated',);
            expect(
              screenEvidence({
                evidence: '',
                omittedText: 'She didn\'t wake',
                region: curly,
              },),
            ).toBe('removal-corroborated',);
          },
        },),
      ],
    },),

    describe({
      name: resolveProberChecks.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE FIRST CHECK A PROBER CAST ON A REGION WITH A KNOWN VERDICT and names each later one '
            + 'as a duplicate-check finding under its prober, passing over a check whose verdict the screen '
            + 'does not know without a finding',
          fn: async () => {
            /**
             Checks resolved over one prober that answered region one three times and an unknown verdict once.
             */
            const resolved = resolveProberChecks({
              ballots: {
                'hf:cat/one': [
                  catCheck({ verdict: 'purring', region: 1, },),
                  catCheck({ verdict: 'uncertain', region: 1, },),
                  catCheck({ verdict: 'no-introduced-defect-found', region: 1, },),
                  catCheck({ verdict: 'uncertain', region: 2, },),
                  catCheck({ verdict: 'uncertain', region: 1, },),
                ],
                'hf:cat/two': [catCheck({ verdict: 'uncertain', region: 1, },),],
              },
            },);
            expect(resolved.findings,).toEqual([
              'hf:cat/one: duplicate-check (1)',
              'hf:cat/one: duplicate-check (1)',
            ],);
            expect(resolved.checks
              .get('hf:cat/one',)
              ?.get(1,)
              ?.verdict,).toBe('uncertain',);
            expect([...resolved.checks
              .get('hf:cat/one',)
              ?.keys() ?? [],],).toEqual([1, 2,],);
          },
        },),

        it({
          name: 'WRITES NO FINDING FOR PROBERS THAT CAST ONE CHECK PER REGION',
          fn: async () => {
            expect(resolveProberChecks({
              ballots: {
                'hf:cat/one': [
                  catCheck({ verdict: 'uncertain', region: 1, },),
                  catCheck({ verdict: 'uncertain', region: 2, },),
                ],
              },
            },).findings,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: screenIntroducedDefects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'counts each prober under the admissibility its own quote earned, '
            + 'keeping the claims for later calibration',
          fn: async () => {
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  evidence: 'The cat sleeps.',
                },),],
                'hf:cat/two': [catCheck({
                  verdict: 'introduced-defect',
                  evidence: 'The cat',
                },),],
                'hf:cat/three': [catCheck({ verdict: 'no-introduced-defect-found', },),],
                'hf:cat/four': [catCheck({ verdict: 'uncertain', },),],
                'hf:cat/five': [catCheck({
                  verdict: 'introduced-defect',
                  omittedText: 'and she wakes at dusk',
                },),],
              },
            },);
            expect(tally?.corroborated,).toBe(1,);
            expect(tally?.removalCorroborated,).toBe(1,);
            expect(tally?.contradicted,).toBe(1,);
            expect(tally?.unanchored,).toBe(0,);
            expect(tally?.noneFound,).toBe(1,);
            expect(tally?.uncertain,).toBe(1,);
            expect(tally?.claims,).toHaveLength(3,);
            expect(tally?.claims[0]?.modelId,).toBe('hf:cat/one',);
            expect(tally?.envelopeId,).toBe('envelope/nap',);
          },
        },),

        it({
          name: 'DISMISSES a claim whose evidence restates an accepted issue where its omitted side is only '
            + 'spaces, and KEEPS the same claim where its evidence restates none (ledger B128)',
          fn: async () => {
            /**
             One prober's claim of added damage, its omitted side padded with
             spaces: the differential reads that side as no anchor and
             corroborates the claim on its evidence.
             */
            const ballots = {
              'hf:cat/one': [catCheck({
                verdict: 'introduced-defect',
                evidence: 'cat sleeps',
                omittedText: '   ',
              },),],
            };

            /**
             The claim as the screen keeps it, short of its admissibility.
             */
            const kept = {
              modelId: 'hf:cat/one',
              category: 'omission',
              severity: 'major',
              evidence: 'cat sleeps',
              omittedText: '   ',
              reason: 'the second clause is gone',
            };

            /**
             Every count of a region no claim was counted under.
             */
            const quiet = {
              envelopeId: 'envelope/nap',
              issueIds: ['adjudicated/nap',],
              corroborated: 0,
              removalCorroborated: 0,
              contradicted: 0,
              unanchored: 0,
              preExisting: 0,
              noneFound: 0,
              uncertain: 0,
            };
            expect(screenIntroducedDefects({
              regions: [REGION,],
              ballots,
              issues: [catIssue({ quotedText: 'the cat sleeps', },),],
            },),).toEqual([{
              ...quiet,
              preExisting: 1,
              claims: [{
                ...kept,
                admissibility: 'pre-existing',
              },],
            },],);
            expect(screenIntroducedDefects({
              regions: [REGION,],
              ballots,
              issues: [catIssue({ quotedText: 'she wakes at dusk', },),],
            },),).toEqual([{
              ...quiet,
              corroborated: 1,
              claims: [{
                ...kept,
                admissibility: 'corroborated',
              },],
            },],);
          },
        },),

        it({
          name: 'DISMISSES a claim whose evidence restates an accepted issue where its omitted side holds '
            + 'only a zero-width space or a Hangul filler (ledgers B40 and B128)',
          fn: async () => {
            /**
             What the screen makes of the claim under each invisible omitted
             side.
             */
            const admissibilities = [
              '\u{200B}',
              '\u{3164}',
            ].map(function screened(invisible,) {
              const [tally,] = screenIntroducedDefects({
                regions: [REGION,],
                ballots: {
                  'hf:cat/one': [catCheck({
                    verdict: 'introduced-defect',
                    evidence: 'cat sleeps',
                    omittedText: invisible,
                  },),],
                },
                issues: [catIssue({ quotedText: 'the cat sleeps', },),],
              },);
              return tally?.claims.map(function toAdmissibility(claim,) {
                return claim.admissibility;
              },);
            },);
            expect(admissibilities,).toEqual([
              ['pre-existing',],
              ['pre-existing',],
            ],);
          },
        },),

        it({
          name: 'KEEPS A CLAIM CORROBORATED where the only prior quote of the accepted issue is one invisible '
            + 'character the claim\'s wording happens to carry, which shows a reader nothing to restate '
            + '(ledgers B40 and B195)',
          fn: async () => {
            /**
             Region whose replacement carries a zero-width space and a Hangul filler inside the claimed wording.
             */
            const spaced: RepairRegion = {
              envelopeId: 'envelope/spaced',
              issueIds: ['adjudicated/nap',],
              before: 'The cat is doing the sleeping.',
              editorAfter: 'The cat\u{200B} sleeps\u{3164}.',
            };
            /**
             Tallies under each invisible prior quote.
             */
            const tallies = [
              '\u{200B}',
              '\u{3164}',
            ].map(function screened(invisible,) {
              const [tally,] = screenIntroducedDefects({
                regions: [spaced,],
                ballots: {
                  'hf:cat/one': [catCheck({
                    verdict: 'introduced-defect',
                    evidence: 'cat\u{200B} sleeps\u{3164}',
                  },),],
                },
                issues: [catIssue({ quotedText: invisible, },),],
              },);
              return [tally?.corroborated, tally?.preExisting,];
            },);
            expect(tallies,).toEqual([
              [1, 0,],
              [1, 0,],
            ],);
          },
        },),
        it({
          name: 'drops a check whose verdict is outside the vocabulary instead of '
            + 'folding it into uncertain, so schema noise never reads as doubt',
          fn: async () => {
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/one': [catCheck({ verdict: 'looks-fine-to-me', },),],
              },
            },);
            expect(tally?.noneFound,).toBe(0,);
            expect(tally?.uncertain,).toBe(0,);
            expect(tally?.claims,).toHaveLength(0,);
          },
        },),

        it({
          name: 'routes each check to the region its number names, so a prober '
            + 'answering out of order cannot move a claim onto another edit',
          fn: async () => {
            /** Second region, whose replacement shares no wording with the first. */
            const other: RepairRegion = {
              envelopeId: 'envelope/chase',
              issueIds: ['adjudicated/chase',],
              before: 'She chase butterflies.',
              editorAfter: 'She chases butterflies.',
            };

            const tallies = screenIntroducedDefects({
              regions: [
                REGION,
                other,
              ],
              ballots: {
                'hf:cat/one': [
                  catCheck({
                    verdict: 'introduced-defect',
                    evidence: 'She chases butterflies.',
                    region: 2,
                  },),
                  catCheck({
                    verdict: 'no-introduced-defect-found',
                    region: 1,
                  },),
                ],
              },
            },);
            expect(tallies[0]?.noneFound,).toBe(1,);
            expect(tallies[0]?.claims,).toHaveLength(0,);
            expect(tallies[1]?.corroborated,).toBe(1,);
            expect(tallies[1]?.envelopeId,).toBe('envelope/chase',);
          },
        },),

        it({
          name: 'ignores a check naming a region that is not on the sheet, rather '
            + 'than letting it land on the last region by accident',
          fn: async () => {
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  evidence: 'The cat sleeps.',
                  region: 7,
                },),],
              },
            },);
            expect(tally?.corroborated,).toBe(0,);
            expect(tally?.claims,).toHaveLength(0,);
          },
        },),

        it({
          name: 'KEEPS a removal claim whose dropped wording CONTAINS the prior '
            + 'quote, which is the over-deletion signal itself: the edit took the '
            + 'objected-to phrase and unrelated content with it. Checking '
            + 'containment both ways here suppressed exactly this, and it is the '
            + 'damage a reader found as a deleted contributor credit. Measured: '
            + 'removal-corroborated ran 159 across the original 56-entry run and 0 '
            + 'across every run after the reclassification landed',
          fn: async () => {
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  omittedText: 'is doing the sleeping, and she wakes at dusk',
                },),],
              },
              issues: [catIssue({ quotedText: 'is doing the sleeping', },),],
            },);

            expect(tally?.removalCorroborated,).toBe(1,);
            expect(tally?.preExisting,).toBe(0,);
          },
        },),

        it({
          name: 'still DISCOUNTS a removal claim whose dropped wording sits INSIDE '
            + 'the prior quote, since removing the phrase the critic objected to is '
            + 'what the repair was for. Without this the "KEEPS a removal claim whose dropped wording '
            + 'CONTAINS the prior quote" fix would just turn '
            + 'the suppression off and count every licensed removal as damage',
          fn: async () => {
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  omittedText: 'doing the sleeping',
                },),],
              },
              issues: [catIssue({ quotedText: 'is doing the sleeping', },),],
            },);

            expect(tally?.removalCorroborated,).toBe(0,);
            expect(tally?.preExisting,).toBe(1,);
          },
        },),

        it({
          name: 'leaves ADDED-wording claims discounted in both containment '
            + 'directions, because those quote the AFTER text: a prober pointing at '
            + 'wording the editor KEPT is restating the accepted issue however much '
            + 'of it it quoted',
          fn: async () => {
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  evidence: 'The cat sleeps.',
                },),],
              },
              issues: [catIssue({ quotedText: 'cat sleeps', },),],
            },);

            expect(tally?.corroborated,).toBe(0,);
            expect(tally?.preExisting,).toBe(1,);
          },
        },),
        it({
          name: 'READS A RESTATEMENT THROUGH THE EVIDENCE FOLD (ledger B24): a prober quoting with a straight apostrophe '
            + 'what the accepted issue quoted with a curly one is restating it',
          fn: async () => {
            /** Region whose replacement writes a curly apostrophe, under the accepted issue. */
            const curly: RepairRegion = {
              envelopeId: 'envelope/curly',
              issueIds: ['adjudicated/nap',],
              before: 'She naps.',
              editorAfter: 'The cat’s nap ran long.',
            };
            const [tally,] = screenIntroducedDefects({
              regions: [curly,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  evidence: 'The cat\'s nap',
                },),],
              },
              issues: [catIssue({ quotedText: 'cat’s nap', },),],
            },);

            expect(tally?.corroborated,).toBe(0,);
            expect(tally?.preExisting,).toBe(1,);
            // The mirror: the prober writes the curly form and the issue the
            // straight one, so each side's fold is read on its own.
            const [mirror,] = screenIntroducedDefects({
              regions: [curly,],
              ballots: {
                'hf:cat/one': [catCheck({
                  verdict: 'introduced-defect',
                  evidence: 'The cat’s nap',
                },),],
              },
              issues: [catIssue({ quotedText: 'cat\'s nap', },),],
            },);

            expect(mirror?.corroborated,).toBe(0,);
            expect(mirror?.preExisting,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: 'one check per prober per region',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS a prober once on a region however many checks it cast there, and counts a prober that '
            + 'skipped the region as uncertain, so the tallies never exceed nor undercount the probers heard',
          fn: async () => {
            /**
             Tally of the one region under two probers: one answering twice, one
             saying nothing about it.
             */
            const [tally,] = screenIntroducedDefects({
              regions: [REGION,],
              ballots: {
                'hf:cat/Cat-A': [
                  catCheck({ verdict: 'no-introduced-defect-found', },),
                  catCheck({ verdict: 'no-introduced-defect-found', },),
                ],
                'hf:cat/Cat-B': [],
              },
            },);

            expect(tally?.noneFound,).toBe(1,);
            expect(tally?.uncertain,).toBe(1,);
            expect(tally?.claims,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
