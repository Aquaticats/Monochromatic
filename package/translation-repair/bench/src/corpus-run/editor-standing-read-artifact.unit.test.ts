/**
 Tests for what the editor standing read makes of one artifact: its rounds by
 seat and chunk, or which of three named absences it is, with the line each
 has to say on stderr.

 SAID, NOT PRINTED. The reader returns its line, and the procedure prints
 every line in the order the artifacts were listed, so nothing here captures
 a stream.

 Every artifact is written under a scratch directory. Fixtures are
 cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readStandingArtifact, } from '../../dist/final/node/index.mjs';
import {
  carrying,
  JUDGED_SEATED_ROUND,
  JUDGED_SELECTION,
  OFF_ROSTER_ROUND,
  refusedEmpty,
  SEATED_ROUND,
  SEATED_SELECTION,
  standingRun,
  WITHOUT_ROUNDS,
} from './editor-standing-read.test-fixture.ts';

/**
 Built pipeline every fixture artifact records.
 */
const DIGEST = `sha256-tree-v1:${'0'.repeat(64,)}`;

await describe({
  name: readStandingArtifact.name,
  children: [
    it({
      name: 'READS AN ARTIFACT\'S ROUNDS BY SEAT AND CHUNK, an editor round at the editor seat and a refine round at '
        + 'the refiner seat, and says nothing',
      fn: async () => {
        /**
         The refiner's round, a refine stage over the same candidate.
         */
        const refined = {
          ...JUDGED_SEATED_ROUND,
          stage: 'refine',
          envelopeId: 'env/2',
        };
        await using run = await standingRun({
          artifacts: { 'Mittens.json': carrying({ rounds: [SEATED_ROUND, refined,], },), },
        },);

        expect(await readStandingArtifact({ path: join(run.dir, 'artifacts', 'Mittens.json',), },),).toEqual({
          kind: 'read',
          said: [],
          reading: {
            digest: DIGEST,
            entryId: 'Mittens',
            editor: [[SEATED_SELECTION,],],
            refiner: [[JUDGED_SELECTION,],],
          },
        },);
      },
    },),

    it({
      name: 'NAMES AN ARTIFACT WHOSE ROUND NAMES A MODEL THE ROSTER DROPPED as off-roster, saying which model',
      fn: async () => {
        await using run = await standingRun({
          artifacts: { 'Mittens.json': carrying({ rounds: [OFF_ROSTER_ROUND,], },), },
        },);

        expect(await readStandingArtifact({ path: join(run.dir, 'artifacts', 'Mittens.json',), },),).toEqual({
          kind: 'off-roster',
          said: [
            'editor-standing-read: Mittens.lanes.repair.result.chunks[0].rounds[0].slate[0].producer.modelId names '
              + 'not-on-roster, which no longer holds a place in the roster, so this record was written under an '
              + 'earlier seating and cannot be read as current',
          ],
        },);
      },
    },),

    it({
      name: 'NAMES AN ARTIFACT SETTLED BEFORE THE LANE RECORDED ROUNDS as earlier-schema, and says nothing, since the '
        + 'summary counts it',
      fn: async () => {
        await using run = await standingRun({ artifacts: { 'Mittens.json': WITHOUT_ROUNDS, }, },);

        expect(await readStandingArtifact({ path: join(run.dir, 'artifacts', 'Mittens.json',), },),).toEqual({
          kind: 'earlier-schema',
          said: [],
        },);
      },
    },),

    it({
      name: 'REFUSES AN ARTIFACT THE PARSE REFUSES, saying its path and why',
      fn: async () => {
        await using run = await standingRun({ artifacts: { 'Mittens.json': {}, }, },);

        /**
         The artifact the parse refuses for want of a schema version.
         */
        const path = join(
          run.dir,
          'artifacts',
          'Mittens.json',
        );

        expect(await readStandingArtifact({ path, },),).toEqual({
          kind: 'refused',
          said: [refusedEmpty({ path, },).trimEnd(),],
        },);
      },
    },),

    it({
      name: 'REFUSES AN ARTIFACT THAT CANNOT BE READ, saying its path and the system\'s own code',
      fn: async () => {
        await using run = await standingRun({ artifacts: {}, },);

        /**
         An artifact file nobody wrote.
         */
        const path = join(
          run.dir,
          'artifacts',
          'Pepper.json',
        );

        expect(await readStandingArtifact({ path, },),).toEqual({
          kind: 'refused',
          said: [`editor-standing-read: ${path} refused, could not read Pepper.json as JSON (ENOENT)`,],
        },);
      },
    },),
  ],
},);
