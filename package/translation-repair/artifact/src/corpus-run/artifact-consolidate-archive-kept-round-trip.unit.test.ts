/**
 Tests that the settlement `applyFinalPolish` returns for a lost contributor name form (B1240) is written by
 `describeConsolidateSlice` and read back by `readSettledArtifact`, the round trip that B1240 named and the unit case
 of `consolidation-polish-apply.unit.test.ts` does not make.

 WHY THE ROUND TRIP: `applyFinalPolish` keeps the archive at the slice and relabels the terminal `consolidated` as
 `gate-kept-standing`, since the artifact writer tests the terminal first and writes the kind `consolidated` with the
 settlement's text for a `consolidated` terminal whatever the archive mark says, and the artifact reader refuses an
 archive-kept record under that terminal. The relabel was verified by reading the writer; this file runs the real
 function, the real writer and the real reader of a whole artifact.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  applyFinalPolish,
  type ConsolidationSettlement,
} from '@monochromatic-dev/translation-repair-consolidate';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  describeConsolidateSlice,
  readSettledArtifact,
} from '../../dist/final/node/index.mjs';
import { settledArtifactOver, } from './settled-artifact.test-fixture.ts';

/**
 Original with a contributors line.
 */
const SOURCE_DOC = '本条目贡献者：雪猫\n';

/**
 Archive English that links the contributor's name form, which the consolidated text lost.
 */
const ARCHIVE_DOC = 'Contributors for this entry: [Snow](https://example.test/snow)\n';

/**
 The consolidated wording that lost the name form.
 */
const LOST_NAME_TEXT = 'Contributors for this entry: Snowflake\n';

/**
 A client no round of this case may ask: the lost name is caught before any model is.
 */
const SILENT_CLIENT: ModelClient = {
  chatText: async () => {
    throw new Error('model calls must not run before the baseline floor',);
  },
  chatJson: async () => {
    throw new Error('model calls must not run before the baseline floor',);
  },
  quotas: async () => {
    throw new Error('meter unused by the baseline floor',);
  },
};

await describe({
  name: 'the settlement applyFinalPolish keeps at the archive for a lost contributor name form',
  children: [
    it({
      name: 'ROUND-TRIPS THROUGH THE WRITER AND THE READER OF A WHOLE ARTIFACT (B1240): the relabelled terminal '
        + '`gate-kept-standing` is the record of an archive kept at the slice, and the reader accepts it, where the '
        + 'terminal `consolidated` it replaces is the one the reader refuses an archive-kept record under',
      fn: async () => {
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: ARCHIVE_DOC,
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },);
        /**
         What the stage returned before the polish: a consolidation that won, with the text that lost the name.
         */
        const won: ConsolidationSettlement = {
          terminal: 'consolidated',
          text: LOST_NAME_TEXT,
          floor: {
            kind: 'proposals',
            validModelIds: [],
          },
          verdicts: [],
          rewrapped: false,
          demoted: false,
          findings: [],
        };
        /**
         What `applyFinalPolish` made of it.
         */
        const kept = await applyFinalPolish({
          client: SILENT_CLIENT,
          settlement: won,
          subject: {
            sourceText: SOURCE_DOC,
            incumbentText: ARCHIVE_DOC,
          },
          lineStructured: false,
          sliceIndex: 0,
          eligible: true,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'archive-kept-round-trip-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /**
         The record the artifact writer makes of it.
         */
        const record = describeConsolidateSlice({
          sliceIndex: 0,
          settlement: kept,
        },);
        /**
         The artifact with that record, as the builder returns it.
         */
        const built = settledArtifactOver({
          prepared,
          entryId: 'CatEntry1',
          laneSelection: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: { kind: 'quorum-not-met', },
              ballots: [],
              usable: 0,
            },],
          },
          consolidation: {
            kind: 'settled',
            slices: [record,],
          },
          translateDiffersAt: [0,],
        },);
        /**
         The bytes a file on disk carries.
         */
        const bytes = JSON.stringify(built,);
        /**
         The artifact as the file's reader receives it.
         */
        const written: unknown = JSON.parse(bytes,);
        /**
         What the reader made of the file.
         */
        const read = readSettledArtifact({ value: written, },);

        expect({
          terminal: record.terminal,
          shipped: record.shipped.kind,
          read: read.kind,
        },).toEqual({
          terminal: 'gate-kept-standing',
          shipped: 'archive',
          read: 'version-2',
        },);
      },
    },),
  ],
},);
