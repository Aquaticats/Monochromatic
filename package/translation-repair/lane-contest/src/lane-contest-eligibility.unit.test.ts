/**
 Tests syntax-bearing lane winner publication eligibility.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyLaneContestEligibility,
  describeInadmissibleLanes,
  frontMatterContestEligibility,
  ineligibleCandidatesOf,
  LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,
  laneContestChoiceVerdict,
  type LaneContestBallot,
  type LaneContestOutcome,
  settleEligibleLaneContestBallots,
} from '../dist/final/node/index.mjs';

/**
 Source identity repeated as visible name and alias.
 */
const SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';

/**
 Archive retaining entry id beside translated alias.
 */
const ARCHIVE = '---\nname: CatEntry\ninfo:\n  alias: Maomao\n---\n';

/**
 Syntax-valid translated identity.
 */
const TRANSLATED = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n';

/**
 Builds contest outcome with chosen lane.

 @param choice - lane panel selected

 @returns Quorum-complete synthetic outcome

 @example
 ```ts
 const outcome = outcomeFor({ choice: 'repair', });
 ```
 */
function outcomeFor(
  { choice, ballots = [], }: {
    readonly choice: LaneContestOutcome['choice'];
    readonly ballots?: readonly LaneContestBallot[];
  },
): LaneContestOutcome {
  /**
   Raw ballots fixture outcome retains.
   */
  const usable = ballots.length === 0 ? 2 : ballots.length;
  return {
    choice,
    ballots,
    usable,
    findings: [],
  };
}

/**
 Builds raw contest ballot choosing one lane.

 @param choice - unmodified model choice

 @returns Complete ballot fixture

 @example
 ```ts
 const ballot = ballotFor({ choice: 'translate', });
 ```
 */
function ballotFor(
  { choice, }: { readonly choice: LaneContestBallot['choice']; },
): LaneContestBallot {
  return {
    choice,
    unsupported: [],
    unsupportedRaw: [],
    dropped: [],
    droppedRaw: [],
    reason: 'fixture reads one candidate as source-faithful',
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: ineligibleCandidatesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the candidates the syntax policy marks ineligible in contest order, and none where every candidate is admissible',
          fn: async () => {
            expect(ineligibleCandidatesOf({
              eligibility: frontMatterContestEligibility({
                sourceText: SOURCE,
                incumbentText: ARCHIVE,
                repairText: ARCHIVE,
                translateText: TRANSLATED,
              },),
            },),).toEqual(['archive', 'repair',],);
            expect(ineligibleCandidatesOf({
              eligibility: frontMatterContestEligibility({
                sourceText: SOURCE,
                incumbentText: TRANSLATED,
                repairText: TRANSLATED,
                translateText: TRANSLATED,
              },),
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: settleEligibleLaneContestBallots.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'EXCLUDES INVALID LANE VOTES without redirecting them to eligible lane',
          fn: async () => {
            const eligibility = frontMatterContestEligibility({
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
            },);
            const ballots = [
              ballotFor({ choice: 'repair', },),
              ballotFor({ choice: 'repair', },),
              ballotFor({ choice: 'translate', },),
              ballotFor({ choice: 'translate', },),
            ];
            expect(settleEligibleLaneContestBallots({
              ballots,
              eligibility,
            },),).toBe('translate',);
            const admitted = applyLaneContestEligibility({
              outcome: outcomeFor({ choice: 'neither', ballots, },),
              eligibility,
            },);
            expect(admitted.choice,).toBe('translate',);
            expect(admitted.ballots,).toEqual(ballots,);
          },
        },),

        it({
          name: 'RETURNS NEITHER when eligible lane lacks direct quorum',
          fn: async () => {
            const eligibility = frontMatterContestEligibility({
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
            },);
            expect(settleEligibleLaneContestBallots({
              ballots: [
                ballotFor({ choice: 'repair', },),
                ballotFor({ choice: 'repair', },),
                ballotFor({ choice: 'translate', },),
              ],
              eligibility,
            },),).toBe('neither',);
          },
        },),

        it({
          name: 'KEEPS A BALLOT CHOOSING NEITHER LANE, which names no lane for the floor to exclude and '
            + 'moves neither lane\'s count, so the eligible lane still wins on its own votes',
          fn: async () => {
            /**
             Admission where the archive and repair wordings keep the directory
             id and the translate wording carries the source identity.
             */
            const eligibility = frontMatterContestEligibility({
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
            },);
            expect(settleEligibleLaneContestBallots({
              ballots: [
                ballotFor({ choice: 'translate', },),
                ballotFor({ choice: 'neither', },),
                ballotFor({ choice: 'translate', },),
                ballotFor({ choice: 'neither', },),
                ballotFor({ choice: 'neither', },),
              ],
              eligibility,
            },),).toBe('translate',);
          },
        },),
      ],
    },),

    describe({
      name: 'whether a lane contest choice may ship',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES FRONT MATTER winner retaining directory id',
          fn: async () => {
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'repair', },),
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
              lineStructured: false,
              syntax: 'front-matter',
            },).mayShip,).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS FRONT MATTER winner preserving source identity relation',
          fn: async () => {
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'translate', },),
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
              lineStructured: false,
              syntax: 'front-matter',
            },).mayShip,).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES ORDINARY REPAIR-LANE WINNER that respells target contributor authority',
          fn: async () => {
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'repair', },),
              sourceText: '本条目贡献者：雪猫',
              incumbentText: 'Contributors for this entry: [Snow](https://example.test/snow)',
              repairText: 'Contributors for this entry: Snowflake',
              translateText: 'Contributors for this entry: [Snow](https://example.test/snow)',
              lineStructured: false,
            },).mayShip,).toBe(false,);
          },
        },),

        it({
          name: 'LEAVES ORDINARY PROSE AND DECLINED CONTEST outside syntax rejection',
          fn: async () => {
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'repair', },),
              sourceText: '猫。',
              incumbentText: 'Cat.',
              repairText: 'Cat.',
              translateText: 'A cat.',
              lineStructured: false,
            },).mayShip,).toBe(true,);
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'neither', },),
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
              lineStructured: false,
              syntax: 'front-matter',
            },).mayShip,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: laneContestChoiceVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES THE FINDINGS behind a refusal, and agrees with the boolean',
          fn: async () => {
            /**
             Verdict on a front matter winner that retains the directory id.
             */
            const verdict = laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'repair', },),
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
              lineStructured: false,
              syntax: 'front-matter',
            },);
            expect(verdict.mayShip,).toBe(false,);
            expect(verdict.findings.length > 0,).toBe(true,);
            expect(verdict.findings.join(' ',),).toContain('name',);
          },
        },),

        it({
          name: 'CARRIES NO FINDINGS on a pass',
          fn: async () => {
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'translate', },),
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
              lineStructured: false,
              syntax: 'front-matter',
            },),).toEqual({
              mayShip: true,
              findings: [],
            },);
          },
        },),

        it({
          name: 'NAMES EACH EXCLUDED LANE AND ITS FINDING for the log, since the floor finding alone says '
            + 'nothing about why (Uekawakuyuurei, 2026-09-04: the translate lane restored a location field the '
            + 'archive had dropped, and the reason lived only in the slice cache)',
          fn: async () => {
            /**
             Translate offer restoring a field the archive does not carry.
             */
            const restored = '---\nname: Maomao\ninfo:\n  alias: Maomao\n  location: Catford\n---\n';

            /**
             What the log gets for an archive breaking the identity rule beside
             a repair keeping it and a translate reshaping it.
             */
            const lines = describeInadmissibleLanes({
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: TRANSLATED,
              translateText: restored,
            },);

            expect(lines.length,).toBe(2,);
            expect(lines[0],).toContain('archive inadmissible: Your translation must carry the name',);
            expect(lines[1],).toContain('translate inadmissible: Your translation changed YAML field names',);
            expect(describeInadmissibleLanes({
              sourceText: SOURCE,
              incumbentText: TRANSLATED,
              repairText: TRANSLATED,
              translateText: TRANSLATED,
            },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'NAMES EVERY LANE INADMISSIBLE, with the reason, when the original\'s front matter carries no '
            + 'fence to read, since no lane can be compared against it',
          fn: async () => {
            expect(describeInadmissibleLanes({
              sourceText: 'name: 猫猫\n',
              incumbentText: TRANSLATED,
              repairText: TRANSLATED,
              translateText: TRANSLATED,
            },),).toStrictEqual([
              'archive inadmissible: no comparison was possible: source front matter could not be read',
              'repair inadmissible: no comparison was possible: source front matter could not be read',
              'translate inadmissible: no comparison was possible: source front matter could not be read',
            ],);
          },
        },),
        it({
          name: 'REFUSES A WINNER WHOSE ORIGINAL NO GRAMMAR READS, saying no comparison was possible and why, '
            + 'rather than shipping a wording nothing checked',
          fn: async () => {
            /**
             Verdict where the original opens an expression brace it never closes.
             */
            const verdict = laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'repair', },),
              sourceText: '猫猫在{窗台上打盹。',
              incumbentText: 'The cat naps on the windowsill.',
              repairText: 'The cat dozes on the windowsill.',
              translateText: 'The cat naps on the sill.',
              lineStructured: false,
            },);
            expect(verdict.mayShip,).toBe(false,);
            expect(verdict.findings.length,).toBe(1,);
            expect(verdict.findings[0]?.startsWith('no comparison was possible: original could not be read: ',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'NAMES THE FLOOR FINDING when a syntax decline stood for an empty eligible slate',
          fn: async () => {
            expect(laneContestChoiceVerdict({
              outcome: {
                ...outcomeFor({ choice: 'neither', },),
                findings: [LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,],
              },
              sourceText: SOURCE,
              incumbentText: ARCHIVE,
              repairText: ARCHIVE,
              translateText: TRANSLATED,
              lineStructured: false,
              syntax: 'front-matter',
            },),).toEqual({
              mayShip: false,
              findings: [LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,],
            },);
            expect(laneContestChoiceVerdict({
              outcome: outcomeFor({ choice: 'neither', },),
              sourceText: '猫。',
              incumbentText: 'Cat.',
              repairText: 'Cat.',
              translateText: 'A cat.',
              lineStructured: false,
            },),).toEqual({
              mayShip: true,
              findings: [],
            },);
          },
        },),
        it({
          name: 'REFUSES A WINNER MERGING THE LINES OF A GOVERNED SLICE, and passes the same winner where no line '
            + 'rule governs and the lane keeping the lines where one does (ledger H2)',
          fn: async () => {
            /**
             Two-line verse slice whose translate lane merged its lines.
             */
            const verse = {
              outcome: outcomeFor({ choice: 'translate', },),
              sourceText: '猫在窗台上，\n狗在门口边。',
              incumbentText: 'A cat on the windowsill,\na dog beside the door.',
              repairText: 'A cat upon the windowsill,\na dog beside the door.',
              translateText: 'A cat upon the windowsill and a dog beside the door.',
            };

            /**
             Verdict where the line-structure rule governs the slice.
             */
            const governed = laneContestChoiceVerdict({
              ...verse,
              lineStructured: true,
            },);
            expect(governed.mayShip,).toBe(false,);
            expect(governed.findings.join(' ',),).toContain('LINE-STRUCTURED',);
            expect(laneContestChoiceVerdict({
              ...verse,
              lineStructured: false,
            },),).toEqual({
              mayShip: true,
              findings: [],
            },);
            expect(laneContestChoiceVerdict({
              ...verse,
              outcome: outcomeFor({ choice: 'repair', },),
              lineStructured: true,
            },),).toEqual({
              mayShip: true,
              findings: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
