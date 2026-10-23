import type { Logger, } from '@monochromatic-dev/module-logger/ts';

import {
  alignDocumentSections,
  type ChunkSlice,
  governedSliceIndices,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isInsertionChunk,
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  buildTranslateMessages,
  isTranslateReportWire,
  SLICE_CHAR_BUDGET,
  subdivideChunkPair,
  TRANSLATE_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  coverageOf,
  sparsestPair,
} from './translate-probe-coverage.ts';
import {
  heardLines,
  PROBE_SLICES,
  sectionLine,
  sliceHeading,
  slicesLine,
} from './translate-probe-lines.ts';

//region Translate probe run
// PROTOTYPE for the translate-first re-design. Asks whether a translate stage
// can do what the repair
// loop demonstrably cannot: render a section the corpus never translated.
//
// The case is real and is the worst one measured. In `XingZ60` an aligned
// section holds 76 source blocks against 5 target blocks. The current pipeline
// treats that as a translation with defects in it, so the critics file omission
// after omission and the editor writes English one accepted issue at a time.
//
// Reports characters in and out, block counts, and how far the voices agree,
// then prints the rendered text so it can be read against the original. It
// writes nothing and changes no pipeline behaviour.

/**
 Runs one translator ensemble over the sparsest aligned section.

 @param entryId - corpus entry to demonstrate on

 @param pin - corpus clone and commit every read resolves against

 @param newClient - builds the client one slice's translators are asked
 through, called afresh for each slice

 @param editorModelIds - translators asked

 @param perCallTimeoutMs - deadline per exchange; the client arms it inside its
 own slot and a Claude seat runs under the larger deadline its card needs, which
 a caller-side timeout signal at this figure would cut short

 @param log - logger the stage logs through

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws StatedRefusalError when either page of the entry is absent at the pin, or
 when no client can be built, which names the provider keys that are unset

 @example
 ```ts
 await probeTranslate({ entryId: 'XingZ60', pin, newClient, editorModelIds, perCallTimeoutMs, log, stageSettings, },);
 ```
 */
export async function probeTranslate(
  {
    entryId,
    pin,
    newClient,
    editorModelIds,
    perCallTimeoutMs,
    log,
    stageSettings,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly newClient: () => ModelClient;
    readonly editorModelIds: readonly RosterModelId[];
    readonly perCallTimeoutMs: number;
    readonly log: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Original document at the pinned commit.
   */
  const sourceText = await readCorpusFile({
    pin,
    relPath: `people/${entryId}/page.md`,
  },);

  /**
   Translation at the same commit.
   */
  const targetText = await readCorpusFile({
    pin,
    relPath: `people/${entryId}/page.en.md`,
  },);

  /**
   Aligned section pairs.
   */
  const alignment = alignDocumentSections({
    source: parseDocument({ text: sourceText, },),
    target: parseDocument({ text: targetText, },),
  },);

  /**
   Sparsest section by block ratio, which is the one worth demonstrating on.
   */
  const choice = sparsestPair({ pairs: alignment.pairs, },);
  if (choice.kind === 'none') {
    console.log('TRANSLATE no aligned section carries source blocks',);
    return;
  }

  /**
   The section the probe demonstrates on.
   */
  const { pair: sparsest, } = choice;

  /**
   Sizes of the section, pulled out so the log line carries no chains.
   */
  const {
    nodes: sourceNodes,
    text: sourceSection,
  } = sparsest.source;

  /**
   Same for the translation side.
   */
  const {
    nodes: targetNodes,
    text: targetSection,
  } = sparsest.target;
  console.log(sectionLine({
    entryId,
    sourceBlocks: sourceNodes.length,
    sourceChars: sourceSection.length,
    targetBlocks: targetNodes.length,
    targetChars: targetSection.length,
    ratio: coverageOf({ pair: sparsest, },),
  },),);

  /**
   Paragraph-bound slices of this section, exactly as the pipeline cuts them.
   */
  const slices = subdivideChunkPair({
    pair: sparsest,
    sourceText,
    targetText,
    baseIndex: 0,
    budget: SLICE_CHAR_BUDGET,
  },);
  console.log(slicesLine({ sliceCount: slices.length, },),);

  /**
   Slices the line-structure rule governs, decided by the pipeline's own
   function over this section and its slices, so the sheet this probe builds is the one a
   run shows rather than one missing the rule (ledger H2).
   */
  const governed = governedSliceIndices({
    chunks: [
      {
        sourceText: sourceSection,
        slices: slices.map(function toSlice(cut,): ChunkSlice {
          return {
            index: cut.target
              .sliceIndex,
            sourceText: cut.source
              .text,
          };
        },),
      },
    ],
  },);

  /* oxlint-disable no-await-in-loop -- sequential by design so this never competes with a running corpus pass for per-model stream slots */
  for (const slice of slices.slice(
    0,
    PROBE_SLICES,
  )) {
    /**
     Texts of this slice.
     */
    const { text: sliceSource, } = slice.source;

    /**
     Translation side, empty where the section was never translated.
     */
    const { text: sliceTarget, } = slice.target;
    console.log(sliceHeading({
      sourceChars: sliceSource.length,
      targetChars: sliceTarget.length,
    },),);
    console.log(`SOURCE: ${sliceSource}`,);

    /**
     Sheet the translators read for this slice.
     */
    const plan = buildTranslateMessages({
      sourceText: sliceSource,
      existingText: sliceTarget,
      incumbentKind: isInsertionChunk(slice.target,) ? 'absent' : 'present',
      lineStructured: governed.has(slice.target
        .sliceIndex,),
    },);

    /**
     Client for this slice, built outside the `try` that reports a failed
     slice: a client that cannot be built (no provider key) is a refusal of
     the whole command, which no slice's round can recover from, and not one
     slice's failure to go on from.
     */
    const client = newClient();

    /**
     Abort of the caller, which this probe never raises: the per-call deadline
     is armed inside the client and not by a signal handed to it.
     */
    const { signal, } = new AbortController();
    try {
      /**
       Translator voices over this slice.
       */
      const gather = await gatherStageVoices({
        client,
        modelIds: editorModelIds,
        messages: plan.messages,
        signal,
        exchangeTimeoutMs: perCallTimeoutMs,
        responseFormat: TRANSLATE_RESPONSE_FORMAT,
        validate: isTranslateReportWire,
        stage: 'translate-probe',
        l: log,
        graceMs: stageSettings
          .grace
          .roundMs,
      },);

      /**
       Voices heard for this slice.
       */
      const {
        voices,
        findings,
      } = gather;
      for (
        const printed of heardLines({
          heard: voices.length,
          asked: editorModelIds.length,
          translations: voices.map(function toTranslation(voice,): {
            readonly modelId: string;
            readonly translation: string;
          } {
            /**
             Reply this voice sent.
             */
            const { value, } = voice;
            return {
              modelId: voice.modelId,
              translation: value.translation,
            };
          },),
          findings,
        },)
      )
        console.log(printed,);
    }
    catch (error) {
      // Reported rather than fatal. The first run died on an uncaught timeout
      // and lost every slice after it, which turns one slow call into no
      // measurement at all.
      console.log(`  SLICE FAILED: ${String(error,)}`,);
    }
  }
  /* oxlint-enable no-await-in-loop */
}

//endregion Translate probe run
