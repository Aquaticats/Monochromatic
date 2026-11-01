/**
 Tests for carving a document pair the way the run that settled it did.

 WHAT THESE PIN is that a recorded recipe reproduces the run's own slicing,
 proved by the identity hash rather than by inspection, and that a missing
 recipe half is named rather than guessed silently. Every rebuild case carries
 a positive control showing the recipe actually moved the slicing, since a
 fixture the deterministic aligner slices identically would pass for the
 wrong reason.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import {
  parseSettledTwoLaneArtifact,
  rebuildPreparation,
} from '../../dist/final/node/index.mjs';
import { settledArtifactText, } from './settled-artifact.test-fixture.ts';

/**
 Two sections, equal shape, so the deterministic aligner pairs by index and a
 supplied crossing is visibly different.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Translation of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 One section of three blocks each side, so a block pairing that declines the
 middle block changes what the slice carries.
 */
const BLOCKY_SOURCE = '## 第一节\n\n猫猫在窗台上睡觉。\n\n猫猫喜欢晒太阳。\n\n猫猫有自己的碗。\n';

/**
 Translation of the same three blocks.
 */
const BLOCKY_TARGET = '## Section one\n\nThe cat sleeps on the sill.\n\nThe cat likes the sun.\n\nThe cat has a bowl.\n';

/**
 Builds an artifact over a preparation and reads it back through JSON.

 @param prepared - preparation the artifact records

 @param strip - preparation keys to delete before parsing, which is how a
 file written before those fields existed looks to a reader

 @param generation - schema generation to label fixture with

 @returns Parsed artifact

 @example
 ```ts
 const artifact = writeAndRead({ prepared, strip: [], },);
 ```
 */
function writeAndRead(
  {
    prepared,
    strip,
    generation = 5,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly strip: readonly string[];
    readonly generation?: 4 | 5 | 11 | 14 | 15;
  },
): ReturnType<typeof parseSettledTwoLaneArtifact> {
  /**
   The artifact as a file on disk carries it.
   */
  const written = JSON.parse(settledArtifactText({
    prepared,
    entryId: 'CatEntry1',
  },),) as Record<string, unknown>;

  /**
   The artifact without the consistency section, which generations before
   fifteen do not carry; a generation fifteen fixture keeps the section, which
   its reader requires.
   */
  const { consistency: ignored, ...olderThanFifteen } = written;
  /**
   Preparation record, with the named keys removed.
   */
  const preparation = Object.fromEntries(
    Object.entries(written.preparation as Record<string, unknown>,)
      .filter(function kept([key,],): boolean {
        return !strip.includes(key,);
      },),
  );
  return parseSettledTwoLaneArtifact({
    value: {
      ...((generation === 15) ? written : olderThanFifteen),
      artifactSchemaVersion: generation,
      preparation,
    },
  },);
}

await describe({
  name: rebuildPreparation.name,
  children: [
    it({
      name: 'REBUILDS GENERATION 4 WITHOUT FRONT MATTER SLICE and generation 5 with it',
      fn: async () => {
        /**
         Original carrying visible metadata.
         */
        const sourceText = '---\nname: 猫猫\n---\n\n猫睡了。\n';
        /**
         Translation carrying corresponding metadata.
         */
        const targetText = '---\nname: Maomao\n---\n\nThe cat slept.\n';
        /**
         Body-only slicing generation 4 recorded.
         */
        const legacyPrepared = prepareDocumentPair({
          sourceText,
          targetText,
          includeFrontMatter: false,
        },);
        /**
         Metadata-inclusive slicing generation 5 records.
         */
        const currentPrepared = prepareDocumentPair({
          sourceText,
          targetText,
        },);
        /**
         Rebuild under generation 4 semantics.
         */
        const legacy = rebuildPreparation({
          artifact: writeAndRead({
            prepared: legacyPrepared,
            strip: [],
            generation: 4,
          },),
          sourceText,
          targetText,
        },);
        /**
         Rebuild under generation 5 semantics.
         */
        const current = rebuildPreparation({
          artifact: writeAndRead({
            prepared: currentPrepared,
            strip: [],
            generation: 5,
          },),
          sourceText,
          targetText,
        },);

        expect(legacy.prepared.slices.some(function hasMetadata(slice,): boolean {
          return slice.syntax === 'front-matter';
        },),).toBe(false,);
        expect(current.prepared.slices.at(0,)?.syntax,).toBe('front-matter',);
        expect(preparationIdentity({ prepared: legacy.prepared, }),)
          .toBe(preparationIdentity({ prepared: legacyPrepared, }),);
        expect(preparationIdentity({ prepared: current.prepared, }),)
          .toBe(preparationIdentity({ prepared: currentPrepared, }),);
      },
    },),
    it({
      name: 'REBUILDS GENERATION 11 WITHOUT SLICE ZERO where the record says the archive\'s front matter '
        + 'stood, reading that off the file rather than recomputing the rule, and with it where the '
        + 'record says nothing',
      fn: async () => {
        /**
         Original carrying visible metadata.
         */
        const sourceText = '---\nname: 猫猫\n---\n\n猫睡了。\n';
        /**
         Translation whose metadata the archive already translated.
         */
        const targetText = '---\nname: Maomao\n---\n\nThe cat slept.\n';
        /**
         Slicing that left the archive's front matter alone.
         */
        const standingPrepared = prepareDocumentPair({
          sourceText,
          targetText,
          frontMatterAuthority: 'archive',
        },);
        /**
         Slicing that rendered it, as every generation before eleven did.
         */
        const renderedPrepared = prepareDocumentPair({
          sourceText,
          targetText,
        },);
        /**
         The standing record read back.
         */
        const standingArtifact = writeAndRead({
          prepared: standingPrepared,
          strip: [],
          generation: 11,
        },);
        expect(standingArtifact.preparation.frontMatterAuthority,).toBe('archive',);
        /**
         Rebuild off the standing record.
         */
        const standing = rebuildPreparation({
          artifact: standingArtifact,
          sourceText,
          targetText,
        },);
        expect(standing.prepared.slices.some(function hasMetadata(slice,): boolean {
          return slice.syntax === 'front-matter';
        },),).toBe(false,);
        expect(standing.prepared.frontMatterAuthority,).toBe('archive',);
        expect(preparationIdentity({ prepared: standing.prepared, }),)
          .toBe(preparationIdentity({ prepared: standingPrepared, }),);
        /**
         Rebuild off a record that says nothing, which renders.
         */
        const rendered = rebuildPreparation({
          artifact: writeAndRead({
            prepared: renderedPrepared,
            strip: [],
            generation: 11,
          },),
          sourceText,
          targetText,
        },);
        expect(rendered.prepared.slices.at(0,)?.syntax,).toBe('front-matter',);
        expect(rendered.prepared.frontMatterAuthority,).toBeUndefined();
      },
    },),
    it({
      name:
        'REPRODUCES the run\'s own slicing from a recorded section pairing, proved by the identity hash: '
          + 'the deterministic aligner pairs these sections by index, the run crossed them, and the '
          + 'rebuild lands on the crossing rather than the index order',
      fn: async () => {
        /**
         How the run carved it: sections crossed, block rounds asked and
         silent.
         */
        const crossed = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          sectionPairing: [{
            source: 0,
            target: 1,
          },],
          blockPairings: new Map(),
        },);

        /**
         How the bare aligner carves it.
         */
        const bare = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);

        // POSITIVE CONTROL: the crossing has to move the slicing.
        expect(preparationIdentity({ prepared: crossed, },),).not
          .toBe(preparationIdentity({ prepared: bare, },),);

        /**
         Rebuild from what the artifact recorded.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: crossed,
            strip: [],
          },),
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);
        expect(rebuilt.unrecorded,).toEqual([],);
        expect(preparationIdentity({ prepared: rebuilt.prepared, },),).toBe(
          preparationIdentity({ prepared: crossed, },),
        );
        expect(rebuilt.reproduction,).toStrictEqual({ kind: 'reproduced', },);
      },
    },),
    it({
      name:
        'SAYS it did not reproduce the run\'s carve where the recipe is complete and the carve still moved '
          + '(ledger A12: mikaela15 recorded 34 slices, rebuilt to 32 after the class one hundred twelve '
          + 'slicer change, and nothing said so)',
      fn: async () => {
        /**
         How the run carved it.
         */
        const crossed = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          sectionPairing: [{
            source: 0,
            target: 1,
          },],
          blockPairings: new Map(),
        },);

        /**
         Rebuild over an original whose paired section has since changed,
         standing in for any change that moves the carve under a complete
         recipe. The original, not the archive: the rebuild carves the archive
         the artifact stored (ledger A18), so only the original can drift.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: crossed,
            strip: [],
          },),
          sourceText: SOURCE_DOC.replace(
            '猫猫在窗台上睡觉。',
            '猫猫在窗台上睡了一整天。',
          ),
          targetText: TARGET_DOC,
        },);
        expect(rebuilt.unrecorded,).toEqual([],);
        expect(rebuilt.reproduction.kind,).toBe('moved',);
      },
    },),
    it({
      name:
        'REPRODUCES a recorded block pairing, keyed back by aligned section: a pairing that declines the '
          + 'middle block takes it out of the slice, and the rebuild carries the same slice text',
      fn: async () => {
        /**
         How the run carved it: the middle block declined by the roster.
         */
        const declined = prepareDocumentPair({
          sourceText: BLOCKY_SOURCE,
          targetText: BLOCKY_TARGET,
          blockPairings: new Map([[
            0,
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 2,
                target: 2,
              },
            ],
          ],],),
        },);

        /**
         How the bare aligner carves it.
         */
        const bare = prepareDocumentPair({
          sourceText: BLOCKY_SOURCE,
          targetText: BLOCKY_TARGET,
        },);

        // POSITIVE CONTROL: declining a block has to move the slicing.
        expect(preparationIdentity({ prepared: declined, },),).not
          .toBe(preparationIdentity({ prepared: bare, },),);

        /**
         Rebuild from what the artifact recorded.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: declined,
            strip: [],
          },),
          sourceText: BLOCKY_SOURCE,
          targetText: BLOCKY_TARGET,
        },);
        expect(rebuilt.unrecorded,).toEqual([],);
        expect(preparationIdentity({ prepared: rebuilt.prepared, },),).toBe(
          preparationIdentity({ prepared: declined, },),
        );
      },
    },),
    it({
      name:
        'ANSWERS MOVED IN THE REFUSAL\'S WORDS where the recorded block pairing names a translation block the '
          + 'carved text lacks, and carves every section\'s blocks by the deterministic aligner instead',
      fn: async () => {
        /**
         How the run carved it: each of the section's four blocks paired with
         its counterpart, the last pair naming translation block 3.
         */
        const carved = prepareDocumentPair({
          sourceText: BLOCKY_SOURCE,
          targetText: BLOCKY_TARGET,
          blockPairings: new Map([[
            0,
            [
              0,
              1,
              2,
              3,
            ].map(function samePlace(at,): {
              readonly source: number;
              readonly target: number;
            } {
              return {
                source: at,
                target: at,
              };
            },),
          ],],),
        },);

        /**
         Corpus copy the rebuild is handed, with two of its paragraphs joined
         into one, so its section holds three translation blocks.
         */
        const shorterTarget = BLOCKY_TARGET.replace(
          'sill.\n\nThe cat likes',
          'sill and likes',
        );

        /**
         Rebuild of a file that predates storing its archive, so it carves
         the corpus copy.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: carved,
            strip: ['archiveText',],
          },),
          sourceText: BLOCKY_SOURCE,
          targetText: shorterTarget,
        },);
        expect({
          unrecorded: rebuilt.unrecorded,
          reproduction: rebuilt.reproduction,
          identity: preparationIdentity({ prepared: rebuilt.prepared, },),
        },).toStrictEqual({
          unrecorded: [],
          reproduction: {
            kind: 'moved',
            detail: 'the recorded block pairing does not fit the text carved (pairing names translation block 3, '
              + 'and there are 3), so every section\'s blocks were carved by the deterministic aligner',
          },
          identity: preparationIdentity({
            prepared: prepareDocumentPair({
              sourceText: BLOCKY_SOURCE,
              targetText: shorterTarget,
            },),
          },),
        },);
      },
    },),
    it({
      name:
        'KEEPS THE DEATH THE ENTRY RECORDS in the carve it makes again after setting a recorded block pairing '
          + 'aside, for an artifact of generation fifteen',
      fn: async () => {
        /**
         How the run carved it, the last pair naming translation block 3.
         */
        const carved = prepareDocumentPair({
          sourceText: BLOCKY_SOURCE,
          targetText: BLOCKY_TARGET,
          blockPairings: new Map([[
            0,
            [
              0,
              1,
              2,
              3,
            ].map(function samePlace(at,): {
              readonly source: number;
              readonly target: number;
            } {
              return {
                source: at,
                target: at,
              };
            },),
          ],],),
        },);
        /**
         Corpus copy with two paragraphs joined, so the recorded pairing no longer fits.
         */
        const shorterTarget = BLOCKY_TARGET.replace(
          'sill.\n\nThe cat likes',
          'sill and likes',
        );
        /**
         The carve made again, told the entry's death.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: carved,
            strip: ['archiveText',],
            generation: 15,
          },),
          sourceText: BLOCKY_SOURCE,
          targetText: shorterTarget,
          sourceEntryInfo: { info: { died: '2025-02-26', }, },
        },);
        expect(rebuilt.reproduction.kind,).toBe('moved',);
        expect(rebuilt.prepared.lifeEnded,).toBe(true,);
      },
    },),
    it({
      name:
        'NAMES every recipe half a file does not record and rebuilds with the deterministic default, '
          + 'which is what every artifact settled before the fields existed looks like',
      fn: async () => {
        /**
         Run carved deterministically, then recorded as an old file would be.
         */
        const bare = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);

        /**
         Rebuild from a file carrying neither half.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: bare,
            strip: [
              'sectionPairing',
              'blockPairing',
            ],
          },),
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);
        expect(rebuilt.unrecorded,).toEqual([
          'sectionPairing',
          'blockPairing',
        ],);
        expect(preparationIdentity({ prepared: rebuilt.prepared, },),).toBe(
          preparationIdentity({ prepared: bare, },),
        );
      },
    },),
    it({
      name:
        'NAMES only the missing half when the other is recorded, so a reader can say which default was '
          + 'guessed rather than reporting a whole recipe as absent',
      fn: async () => {
        /**
         Run that asked the block rounds and recorded them, read through a
         file that lost the section decider.
         */
        const asked = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          blockPairings: new Map(),
        },);
        expect(rebuildPreparation({
          artifact: writeAndRead({
            prepared: asked,
            strip: ['sectionPairing',],
          },),
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },).unrecorded,).toEqual(['sectionPairing',],);
      },
    },),
    it({
      name: 'CARVES OVER THE ARCHIVE THE ARTIFACT STORED rather than the corpus copy it is handed, since the '
        + 'pass reshapes the archive before it carves (ledger A18)',
      fn: async () => {
        /**
         How the run carved the archive it had reshaped.
         */
        const carved = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);

        /**
         The rebuild, handed a corpus copy carrying a line the run never carved.
         */
        const rebuilt = rebuildPreparation({
          artifact: writeAndRead({
            prepared: carved,
            strip: [],
          },),
          sourceText: SOURCE_DOC,
          targetText: `${TARGET_DOC}\nA stray corpus line the run never carved.\n`,
        },);
        expect(rebuilt.prepared.targetText,).toBe(TARGET_DOC,);
        expect(rebuilt.reproduction,).toStrictEqual({ kind: 'reproduced', },);
      },
    },),
    it({
      name: 'TELLS THE DEATH THE ENTRY RECORDS only in the identity block of an artifact of generation fifteen, '
        + 'whose run told its models the date, and leaves it out for generation fourteen, whose run told them none',
      fn: async () => {
        /**
         Entry info the corpus holds for the entry, which records a death.
         */
        const sourceEntryInfo = { info: { died: '2025-02-26', }, };
        /**
         Preparation the run of generation fifteen made, knowing the death.
         */
        const told = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          sourceEntryInfo,
        },);
        /**
         Preparation a run of generation fourteen made, which read no info.yml.
         */
        const untold = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);
        /**
         Rebuild of the generation fifteen artifact.
         */
        const current = rebuildPreparation({
          artifact: writeAndRead({
            prepared: told,
            strip: [],
            generation: 15,
          },),
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          sourceEntryInfo,
        },).prepared;
        /**
         Rebuild of the generation fourteen artifact over the same entry info.
         */
        const legacy = rebuildPreparation({
          artifact: writeAndRead({
            prepared: untold,
            strip: [],
            generation: 14,
          },),
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          sourceEntryInfo,
        },).prepared;
        expect({
          current: [
            current.lifeEnded,
            current.identityContext === told.identityContext,
            current.identityContext?.includes('"2025-02-26"',),
          ],
          legacy: [
            legacy.lifeEnded,
            legacy.identityContext === untold.identityContext,
            legacy.identityContext?.includes('died',),
          ],
        },).toEqual({
          current: [
            true,
            true,
            true,
          ],
          legacy: [
            false,
            true,
            false,
          ],
        },);
      },
    },),
    it({
      name: 'CARVES OVER THE CORPUS COPY where the artifact predates storing its archive, the control',
      fn: async () => {
        /**
         Corpus copy the rebuild is handed.
         */
        const corpusCopy = `${TARGET_DOC}\nA stray corpus line the run never carved.\n`;
        expect(rebuildPreparation({
          artifact: writeAndRead({
            prepared: prepareDocumentPair({
              sourceText: SOURCE_DOC,
              targetText: TARGET_DOC,
            },),
            strip: ['archiveText',],
          },),
          sourceText: SOURCE_DOC,
          targetText: corpusCopy,
        },).prepared.targetText,).toBe(corpusCopy,);
      },
    },),
  ],
},);
