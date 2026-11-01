import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Editor standing read fixtures
// SETTLED ARTIFACTS AND THE LINES `editor-standing-read` PRINTS OVER THEM, shared
// by the as-built suite and the unit tests of the command's modules, so each
// literal is written once.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Fixtures are cat-themed invention; no
// corpus content appears here.

/**
 Hexadecimal digits of a SHA-256 digest, which every invented identity here
 fills with zeroes.
 */
const SHA256_HEX_DIGITS = 64;

/**
 One decision that accepted the text `y`, as every fixture slice records it.
 */
const DECIDED = {
  kind: 'decided',
  acceptedText: 'y',
};

/**
 A replacement that reached the page, as every fixture slice records it.
 */
const SHIPPED = { kind: 'replacement-shipped', };

/**
 One slice's text record, the archive's `x` replaced by `y`.
 */
const SLICE_TEXT = {
  chunkIndex: 0,
  incumbentKind: 'present',
  incumbentText: 'x',
  outcome: DECIDED,
  text: 'y',
};

/**
 One slice's delivery record, the archive's `x` replaced by `y` on the page.
 */
const SLICE_DELIVERY = {
  chunkIndex: 0,
  sourceText: 'x',
  incumbentKind: 'present',
  incumbentText: 'x',
  outcome: DECIDED,
  shippedText: 'y',
  delivery: SHIPPED,
};

/**
 The settled artifact the schema walk of 2026-10-04 mapped, recording no
 rounds, with its shape known so `carrying` reaches its repair lane.
 */
const ARTIFACT_WITHOUT_ROUNDS = {
  artifactSchemaVersion: 3,
  id: 'Mittens',
  tip: 'abc',
  pipelineDigest: `sha256-tree-v1:${'0'.repeat(SHA256_HEX_DIGITS,)}`,
  corpusSha: 'feedfacefeedfacefeedfacefeedfacefeedface',
  callConfig: {},
  durationMs: 1,
  timestamp: '2026-09-28T10:00:00.000Z',
  preparation: {
    alignmentPairCount: 1,
    identity: `sha256-preparation-v2:${'0'.repeat(SHA256_HEX_DIGITS,)}`,
    sliceCount: 1,
    sourceChars: 1,
    targetChars: 1,
    sourceBytes: 1,
    alignmentFindings: [],
  },
  lanes: {
    repair: {
      result: {
        status: 'repaired',
        sliceCount: 1,
        changedSliceIndices: [0,],
        withdrawnSliceIndices: [],
        sliceTexts: [SLICE_TEXT,],
      },
      delivery: [SLICE_DELIVERY,],
    },
    translate: {
      result: {
        status: 'complete',
        sliceCount: 1,
        changedSliceCount: 1,
        withdrawnSliceCount: 0,
        changedSliceIndices: [0,],
        withdrawnSliceIndices: [],
        sliceTexts: [SLICE_TEXT,],
      },
      delivery: [SLICE_DELIVERY,],
    },
  },
  comparison: [{
    chunkIndex: 0,
    incumbentKind: 'present',
    incumbentText: 'x',
    repairText: 'y',
    translateText: 'y',
    laneRelation: 'both-agree',
    repairOutcome: DECIDED,
    translateOutcome: DECIDED,
    decisionComparison: {
      kind: 'comparable',
      verdict: 'same',
    },
    repairDelivery: SHIPPED,
    translateDelivery: SHIPPED,
  },],
  laneSelection: { kind: 'pending-human-decision', },
  consolidation: { kind: 'not-run', },
};

/**
 The settled artifact the schema walk of 2026-10-04 mapped, recording no
 rounds; `carrying` adds the rounds the repair lane records under its result.
 */
export const WITHOUT_ROUNDS: Readonly<Record<string, unknown>> = ARTIFACT_WITHOUT_ROUNDS;

/**
 A round naming a model the roster no longer seats, no ballot cast over its
 one candidate, with its shape known so the seated rounds can be built from it.
 */
const ROUND_OFF_ROSTER = {
  stage: 'chunk-patch',
  modelId: 'not-on-roster',
  kind: 'selected',
  envelopeId: 'env/1',
  slate: [{
    index: 1,
    producer: {
      kind: 'model',
      modelId: 'not-on-roster',
    },
    rendered: 'y',
    hash: 'h',
  },],
  ballots: [],
  tally: {
    judgesAvailable: 1,
    heard: 1,
    quorum: 1,
    probe: 1,
    ballots: 1,
    abstentions: 0,
    selfVotes: 0,
  },
  perCandidate: [],
  selectionReason: 'x',
  selectedIndex: 1,
  selectedText: 'y',
  judgeStatements: [],
  voteWeight: 1,
};

/**
 A round naming a model the roster no longer seats, no ballot cast over its
 one candidate.
 */
export const OFF_ROSTER_ROUND: Readonly<Record<string, unknown>> = ROUND_OFF_ROSTER;

/**
 The seated model's round as the repair lane records it, no ballot cast over
 its one candidate.
 */
export const SEATED_ROUND: Readonly<Record<string, unknown>> = {
  ...ROUND_OFF_ROSTER,
  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  slate: [{
    index: 1,
    producer: {
      kind: 'model',
      modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
    },
    rendered: 'y',
    hash: 'h',
  },],
};

/**
 The one ballot cast on the seated model's judged round.
 */
const SEATED_BALLOTS = [{
  modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  best: 1,
  reason: 'x',
  weight: 1,
  selfVote: false,
},];

/**
 The seated model's round with a ballot cast on it.
 */
export const JUDGED_SEATED_ROUND: Readonly<Record<string, unknown>> = {
  ...SEATED_ROUND,
  ballots: SEATED_BALLOTS,
};

/**
 `SEATED_ROUND` as the reader hands it on: its candidates' producers and the
 ballots cast, none.
 */
export const SEATED_SELECTION: Readonly<Record<string, unknown>> = {
  producers: [{
    kind: 'model',
    modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  },],
  ballots: [],
};

/**
 `JUDGED_SEATED_ROUND` as the reader hands it on, with its one ballot.
 */
export const JUDGED_SELECTION: Readonly<Record<string, unknown>> = {
  ...SEATED_SELECTION,
  ballots: SEATED_BALLOTS,
};

/**
 The settled artifact whose one chunk records the rounds given.

 @param rounds - rounds the artifact's one chunk records, with or without a
 ballot as the comparison needs

 @returns Artifact value written as one of the run's artifacts

 @example
 ```ts
 const artifact = carrying({ rounds: [SEATED_ROUND,], },);
 ```
 */
export function carrying({ rounds, }: { readonly rounds: readonly unknown[]; },): Readonly<Record<string, unknown>> {
  /**
   The repair lane the rounds are recorded under.
   */
  const { repair, } = ARTIFACT_WITHOUT_ROUNDS.lanes;
  return {
    ...ARTIFACT_WITHOUT_ROUNDS,
    lanes: {
      ...ARTIFACT_WITHOUT_ROUNDS.lanes,
      repair: {
        ...repair,
        result: {
          ...repair.result,
          chunks: [{ rounds, },],
        },
      },
    },
  };
}

/**
 The note every report prints under its summary line.
 */
export const OBSERVATIONAL: string = '  OBSERVATIONAL. Only models that held a seat ever wrote a candidate, so an '
  + 'absent model is unmeasured rather than last. Rounds inside one entry are correlated, so read the entry count, '
  + 'not the round count. Digests are never pooled.\n';

/**
 The line a report prints when nothing it read recorded a round at all.
 */
export const NO_ROUNDS: string = '  NO ROUNDS. Nothing read here recorded a judged round. Artifacts settled before '
  + 'the rounds were stored carry none, and an entry whose every chunk was left unchanged carries none either.\n';

/**
 The summary line of a report that read no artifact into a reading.

 @param archives - archives the command line named

 @param artifacts - artifact files found under them

 @returns The whole line with its newline

 @example
 ```ts
 const summary = summaryOf({ archives: 1, artifacts: 1, },);
 ```
 */
export function summaryOf(
  {
    archives,
    artifacts,
  }: {
    readonly archives: number;
    readonly artifacts: number;
  },
): string {
  return `editor-standing-read: archives=${String(archives,)} artifacts=${String(artifacts,)} read=0 earlierRoster=0 `
    + 'earlierSchema=0 digestsJudged=0 digestsUnjudged=0 unjudgedDigestRounds=0\n';
}

/**
 The stderr line naming an artifact written as an empty object, which the
 parse refuses for want of a schema version.

 @param path - the artifact file the line names

 @returns The whole line with its newline

 @example
 ```ts
 const said = refusedEmpty({ path, },);
 ```
 */
export function refusedEmpty({ path, }: { readonly path: string; },): string {
  return `editor-standing-read: ${path} refused, artifact parse failed at artifact.artifactSchemaVersion: expected a `
    + 'number.\n';
}

/**
 The standing line of the seated model over the one round a ballot was cast on.
 */
export const JUDGED_STANDING: string =
  `      ${SEAT_SYNTHETIC_VISION_WITHHELD}: 100.0% (1 of 1 disinterested ballot, over 1 candidate)\n`;

/**
 Writes one throwaway run carrying the artifacts named under its own
 `artifacts/` directory.

 @param artifacts - artifact values by the file name each is written as

 @returns The run directory, removed on dispose

 @example
 ```ts
 await using run = await standingRun({ artifacts: { 'Mittens.json': WITHOUT_ROUNDS, }, },);
 ```
 */
export async function standingRun(
  { artifacts, }: { readonly artifacts: Readonly<Record<string, unknown>>; },
): Promise<AsyncDisposable & { readonly dir: string; }> {
  return await scratchDirWith({
    prefix: 'editor-standing-run-',
    setup: async function seeded({ path, },): Promise<{ readonly dir: string; }> {
      await mkdir(
        join(
          path,
          'artifacts',
        ),
        { recursive: true, },
      );
      await Promise.all(
        Object
          .entries(artifacts,)
          .map(async function written([name, artifact,],): Promise<void> {
            await writeFile(
              join(
                path,
                'artifacts',
                name,
              ),
              JSON.stringify(artifact,),
              'utf8',
            );
          },),
      );
      return { dir: path, };
    },
  },);
}

//endregion Editor standing read fixtures
