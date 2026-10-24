/**
 Tests for saying whether audited wording is wording a document would carry.

 THE ORDER OF THE BRANCHES IS THE SUBJECT, not an implementation detail. On an
 artifact no stage has decided, every would-ship reading names the archive, so
 a classifier that compared text before asking whether a decision exists would
 report a displacement on all 227 of the population's undecided subjects. That
 is the exact wrong answer this module was built to avoid, and it is pinned
 first.

 THE PERSISTED READER IS TESTED AGAINST ABSENCE. Rows come off disk through an
 unchecked cast, so `unrecorded` is reachable in production and reading a
 missing field as `survives` would assert the strongest claim here from no
 evidence at all.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  pageRelationFor,
  pageRelationLabel,
  pageRelationOf,
  relationTallyOf,
  type SettledAuditRow,
  type SettledPageRelation,
} from '../../dist/final/node/index.mjs';

/**
 Wording the translate lane decided on, which is what an audit is shown.
 */
const LANE_NAP = 'The cat naps on the windowsill.';

/**
 Wording a later stage put in its place.
 */
const LATER_NAP = 'The cat is napping on the windowsill.';

/**
 Selection an artifact carries once a contest has settled it.
 */
const CONTESTED = {
  kind: 'contested',
  slices: [],
} as unknown as Parameters<typeof pageRelationOf>[0]['laneSelection'];

/**
 Selection an artifact carries while nobody has decided it.
 */
const UNDECIDED = {
  kind: 'pending-human-decision',
} as unknown as Parameters<typeof pageRelationOf>[0]['laneSelection'];

/**
 Builds a reading carrying wording, defaulting to a later stage's.

 @param decidedBy - stage whose decision survived

 @param text - what would stand there

 @returns Reading as the would-ship reader returns it

 @example
 ```ts
 const reading = wordingFrom({ decidedBy: 'contest', },);
 ```
 */
function wordingFrom(
  {
    decidedBy,
    text = LATER_NAP,
  }: {
    readonly decidedBy: string;
    readonly text?: string;
  },
): Parameters<typeof pageRelationOf>[0]['reading'] {
  return {
    kind: 'wording',
    text,
    decidedBy,
  } as unknown as Parameters<typeof pageRelationOf>[0]['reading'];
}

/**
 Builds a persisted row carrying a relation and a number of claims.

 @param pageRelation - relation to record, omitted for a row written before
 the field existed

 @param claims - how many anchored claims its roster made

 @returns Row as a run file carries it

 @example
 ```ts
 const row = rowCarrying({ pageRelation: { kind: 'survives', }, claims: 2, },);
 ```
 */
function rowCarrying(
  {
    pageRelation,
    claims = 0,
  }: {
    readonly pageRelation?: unknown;
    readonly claims?: number;
  },
): SettledAuditRow {
  return {
    runSet: 'kittens',
    entryId: 'Tabby',
    sliceIndex: 0,
    deliveryKind: 'replacement-shipped',
    auditsArchiveText: false,
    ...((pageRelation === undefined) ? {} : { pageRelation, }),
    artifactDigest: 'sha256-tree-v1:whiskers',
    corpusSha: 'paws',
    identityKind: 'none',
    textIdentity: { kind: 'unrecorded', },
    report: {
      rows: [
        {
          findings: Array.from(
            { length: claims, },
            function claim(): Record<string, unknown> {
              return { kind: 'kept', };
            },
          ),
        },
      ],
    },
  } as unknown as SettledAuditRow;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: pageRelationOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ANSWERS UNDECIDED BEFORE COMPARING ANY TEXT, so an entry no stage has decided never reads '
              + 'as a settled displacement. On such an artifact every reading names the archive, which '
              + 'differs from the lane wording under audit, so a classifier that compared first would '
              + 'report an overrule that no stage performed',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: UNDECIDED,
              reading: wordingFrom({ decidedBy: 'archive', },),
              candidateText: LANE_NAP,
            },),).toEqual({ kind: 'undecided', },);
          },
        },),

        it({
          name:
            'ANSWERS UNDECIDED EVEN WHERE THE TEXT MATCHES, which is the half of the order a '
              + 'differing-text case cannot reach. An undecided slice whose archive wording happens to '
              + 'equal the lane wording would otherwise read as SURVIVES, asserting that a document '
              + 'would carry it when no stage has decided the entry at all',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: UNDECIDED,
              reading: wordingFrom({
                decidedBy: 'archive',
                text: LANE_NAP,
              },),
              candidateText: LANE_NAP,
            },),).toEqual({ kind: 'undecided', },);
          },
        },),

        it({
          name:
            'NAMES THE CONSOLIDATION when the third rendering replaced the lane wording, which is the '
              + 'largest displacing stage in the settled population',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: CONTESTED,
              reading: wordingFrom({ decidedBy: 'consolidation', },),
              candidateText: LANE_NAP,
            },),).toEqual({
              kind: 'displaced',
              decidedBy: 'consolidation',
            },);
          },
        },),

        it({
          name:
            'NAMES THE CONTEST when the lane lost the contest itself, keeping it apart from a '
              + 'consolidation override: the two are different facts about which stage to look at',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: CONTESTED,
              reading: wordingFrom({ decidedBy: 'contest', },),
              candidateText: LANE_NAP,
            },),).toEqual({
              kind: 'displaced',
              decidedBy: 'contest',
            },);
          },
        },),

        it({
          name:
            'NAMES THE ARCHIVE on a DECIDED artifact whose contest declined, where the incumbent stands '
              + 'and the lane wording is displaced by no lane at all. This is a decision, unlike the '
              + 'identical-looking reading on an artifact nobody has decided',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: CONTESTED,
              reading: wordingFrom({ decidedBy: 'archive', },),
              candidateText: LANE_NAP,
            },),).toEqual({
              kind: 'displaced',
              decidedBy: 'archive',
            },);
          },
        },),

        it({
          name:
            'READS WORDING THAT MATCHES AS SURVIVING, whichever stage settled it: a lane whose text a '
              + 'later stage re-endorsed unchanged has not been overruled',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: CONTESTED,
              reading: wordingFrom({
                decidedBy: 'consolidation',
                text: LANE_NAP,
              },),
              candidateText: LANE_NAP,
            },),).toEqual({ kind: 'survives', },);
          },
        },),

        it({
          name:
            'CARRIES THE SILENCE REASON rather than naming a displacing stage, since a slice where '
              + 'nothing would stand has no replacement to attribute and calling it displaced would '
              + 'invite a reader to go looking for one',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: CONTESTED,
              reading: {
                kind: 'nothing-ships',
                reason: 'contest-declined-and-archive-silent',
                incumbentKind: 'present',
              } as unknown as Parameters<typeof pageRelationOf>[0]['reading'],
              candidateText: LANE_NAP,
            },),).toEqual({
              kind: 'nothing-would-ship',
              reason: 'contest-declined-and-archive-silent',
              incumbentKind: 'present',
            },);
          },
        },),

        it({
          name:
            'READS A SLICE THAT SHIPS ITS ORIGINAL AS SHIPPING NO ENGLISH (the owner, 2026-10-23, "Source text + '
              + 'note"), so the four recorded kinds stay the four and the audit is not shown a wording that does '
              + 'not stand there',
          fn: async () => {
            expect(pageRelationOf({
              laneSelection: CONTESTED,
              reading: {
                kind: 'source-ships',
                reason: 'lanes-agreed-on-no-wording',
                incumbentKind: 'absent',
                text: '猫\n',
                opensRun: true,
              },
              candidateText: LANE_NAP,
            },),).toEqual({
              kind: 'nothing-would-ship',
              reason: 'lanes-agreed-on-no-wording',
              incumbentKind: 'absent',
            },);
          },
        },),
      ],
    },),

    describe({
      name: pageRelationFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS BACK a relation a run recorded, unchanged',
          fn: async () => {
            expect(pageRelationFor({
              row: rowCarrying({
                pageRelation: {
                  kind: 'displaced',
                  decidedBy: 'contest',
                },
              },),
            },),).toEqual({
              kind: 'displaced',
              decidedBy: 'contest',
            },);
          },
        },),

        it({
          name:
            'READS A ROW WRITTEN BEFORE THE FIELD EXISTED as unrecorded, because such a run is a valid '
              + 'run whose other readings all still answer, and refusing to read the file would cost '
              + 'every one of them to serve this',
          fn: async () => {
            expect(pageRelationFor({ row: rowCarrying({},), },),)
              .toEqual({ kind: 'unrecorded', },);
          },
        },),

        it({
          name:
            'REFUSES A KIND THIS BUILD DOES NOT NAME, reading it as unrecorded rather than passing it '
              + 'through: a relation a later generation added means nothing to a reader written before '
              + 'it, and printing the unknown token would read as a measurement',
          fn: async () => {
            expect(pageRelationFor({
              row: rowCarrying({ pageRelation: { kind: 'displaced-by-something-else', }, },),
            },),).toEqual({ kind: 'unrecorded', },);
          },
        },),

        it({
          name:
            'READS A NULL FIELD as unrecorded, which the type check must be written to catch: `typeof '
              + 'null` is `object`, so a guard that stopped at the type would go on to read a '
              + 'discriminant off nothing',
          fn: async () => {
            expect(pageRelationFor({ row: rowCarrying({ pageRelation: null, },), },),)
              .toEqual({ kind: 'unrecorded', },);
          },
        },),
      ],
    },),

    describe({
      name: pageRelationLabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'NAMES THE DECIDER on a displacement and the REASON on a silence, so one column says which '
              + 'stage to look at without a reader opening the artifact',
          fn: async () => {
            expect(pageRelationLabel({
              relation: {
                kind: 'displaced',
                decidedBy: 'consolidation',
              } as SettledPageRelation,
            },),).toBe('displaced:consolidation',);
            expect(pageRelationLabel({
              relation: {
                kind: 'nothing-would-ship',
                reason: 'lanes-agreed-on-no-wording',
                incumbentKind: 'present',
              } as SettledPageRelation,
            },),).toBe('emptied:lanes-agreed-on-no-wording',);
          },
        },),

        it({
          name:
            'SEPARATES A GAP FROM AN EMPTYING on the same reason, which is the whole point of carrying '
              + 'the kind. Every reason names a stage and none of them says whether there was anything here: `lanesAgreedOn` covers a gap neither lane wrote into and text both lanes removed under one string, and a reader scanning this column has to tell a change from nothing happening',
          fn: async () => {
            /**
             Silence over a span the archive rendered, which the deciders emptied.
             */
            const emptied = pageRelationLabel({
              relation: {
                kind: 'nothing-would-ship',
                reason: 'lanes-agreed-on-no-wording',
                incumbentKind: 'present',
              } as SettledPageRelation,
            },);

            /**
             The SAME reason at an anchor, where nothing was ever rendered.
             */
            const gap = pageRelationLabel({
              relation: {
                kind: 'nothing-would-ship',
                reason: 'lanes-agreed-on-no-wording',
                incumbentKind: 'absent',
              } as SettledPageRelation,
            },);

            expect(emptied,).toBe('emptied:lanes-agreed-on-no-wording',);
            expect(gap,).toBe('gap:lanes-agreed-on-no-wording',);
            expect(emptied,).not.toBe(gap,);
          },
        },),

        it({
          name:
            'KEEPS THE OLD WORD for a row written before the kind was recorded, rather than guessing '
              + 'the commoner half. A run that never observed whether the archive had a span here has not said the deciders removed anything, and printing `emptied` would put a removal nobody observed into a report',
          fn: async () => {
            expect(pageRelationLabel({
              relation: {
                kind: 'nothing-would-ship',
                reason: 'contest-unasked-and-archive-silent',
              } as unknown as SettledPageRelation,
            },),).toBe('silent:contest-unasked-and-archive-silent',);
          },
        },),

        it({
          name: 'names the bare kind where there is nothing further to say',
          fn: async () => {
            expect(pageRelationLabel({ relation: { kind: 'survives', } as SettledPageRelation, },),)
              .toBe('survives',);
            expect(pageRelationLabel({ relation: { kind: 'undecided', } as SettledPageRelation, },),)
              .toBe('undecided',);
            expect(pageRelationLabel({ relation: { kind: 'unrecorded', } as SettledPageRelation, },),)
              .toBe('unrecorded',);
          },
        },),
      ],
    },),

    describe({
      name: relationTallyOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'COUNTS CLAIMS BESIDE SUBJECTS, since those answer different questions: a displaced subject '
              + 'nobody claimed anything about cost a call, and a displaced subject carrying claims means '
              + 'the instrument reported defects in wording no reader would meet',
          fn: async () => {
            /**
             Two displaced subjects carrying claims, one surviving carrying none.
             */
            const tallies = relationTallyOf({
              rows: [
                rowCarrying({
                  pageRelation: {
                    kind: 'displaced',
                    decidedBy: 'consolidation',
                  },
                  claims: 2,
                },),
                rowCarrying({
                  pageRelation: {
                    kind: 'displaced',
                    decidedBy: 'consolidation',
                  },
                  claims: 3,
                },),
                rowCarrying({ pageRelation: { kind: 'survives', }, },),
              ],
            },);

            expect(tallies,).toEqual([
              {
                label: 'displaced:consolidation',
                subjects: 2,
                claimed: 5,
              },
              {
                label: 'survives',
                subjects: 1,
                claimed: 0,
              },
            ],);
          },
        },),

        it({
          name:
            'TALLIES AN UNRECORDED ROW AS UNRECORDED rather than dropping it, so a run mixing rows from '
              + 'two builds reports a denominator matching its own row count instead of quietly shrinking',
          fn: async () => {
            /**
             One row from before the field existed, beside one from after.
             */
            const tallies = relationTallyOf({
              rows: [
                rowCarrying({},),
                rowCarrying({ pageRelation: { kind: 'survives', }, },),
              ],
            },);

            expect(tallies.reduce(function total(sum, tally,): number {
              return sum + tally.subjects;
            }, 0,),).toBe(2,);
            expect(tallies.some(function isUnrecorded(tally,): boolean {
              return tally.label === 'unrecorded';
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
