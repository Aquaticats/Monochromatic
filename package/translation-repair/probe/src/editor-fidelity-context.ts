import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  neighbouringIncumbent,
  neighbouringSource,
} from '@monochromatic-dev/translation-repair-repair/ts';
import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import type { ReviewedFidelityReference, } from './fidelity-reference-model.ts';

//region Editor fidelity sheet context
// THE CONTEXT THE PIPELINE PUTS ON AN EDITOR'S SHEET, prepared the way the pipeline prepares it. The editor stage is
// handed the sections either side of the slice it repairs, the declared names of the page, and the whole original of
// the page; a probe that sends only the passage and its issue asks an easier question than production does.
//
// THE PREPARATION AND THE WINDOW READERS ARE THE PIPELINE'S OWN. `prepareDocumentPair` slices the whole pair of pages
// (keyless: it spends nothing and takes no client) and `neighbouringSource` and `neighbouringIncumbent` read the
// positional window of a slice, the passage before and the passage after, with the heading rule. Without a block
// pairing from a roster, which production buys, a slice is a whole aligned section, so the unit here is the section.
//
// WHAT IS NOT REPRODUCED, stated as limits. The pipeline's slices are at most `SLICE_CHAR_BUDGET` characters where a
// section here may be several times that, so the probe's editor sees more of the page in its own section than
// production's does. The reference context (what the pages the original cites say) is bought from web lookups with a
// key, and the entry's `info.yml` (a recorded death) is not read; both are absent. The line-structure rule addendum is
// not applied.

/**
 What the sheet carries besides the passage and its issue.
 */
export type EditorSheetContext = {
  /**
   Original of the section before and the section after the passage, as the pipeline's window reader gives it.
   */
  readonly neighbouringSourceText: string;

  /**
   Archive wording of the same sections.
   */
  readonly neighbouringIncumbentText: string;

  /**
   The whole original page, which the preservation gate reads markup against.
   */
  readonly documentSourceText: string;

  /**
   Declared names and handles from the pair's front matter, the original's pronoun and the notes both pages carry;
   absent when the pair declares none.
   */
  readonly identityContext?: string;
};

/**
 Prepares the context of a reference's sheet from the whole pages it was cut from.

 @param reference - reviewed reference, with the pages its passage was cut from

 @returns The neighbouring sections, the whole original and the identity context

 @throws {@link FidelityReferenceError} when no slice of the prepared page lies over the passage's range in the archive,
 since the window of a slice that is not there would read as a lone slice with no neighbours

 @example
 ```ts
 const context = editorSheetContextOf({ reference, },);
 ```
 */
export function editorSheetContextOf({ reference, }: { readonly reference: ReviewedFidelityReference; },): EditorSheetContext {
  /**
   The pair of whole pages, sliced and given its identity context as the pipeline prepares them.
   */
  const prepared = prepareDocumentPair({
    sourceText: reference.pages
      .sourceFile,
    targetText: reference.pages
      .archiveFile,
  },);
  /**
   Archive range of the passage.
   */
  const {
    startOffset: passageStart,
    endOffset: passageEnd,
  } = reference.spec
    .archive;
  /**
   Positions of the slices lying over the passage.
   */
  const over = prepared.slices
    .flatMap(function positionOf(
      slice,
      position,
    ): readonly number[] {
      /**
       Archive range of the slice.
       */
      const {
        startOffset,
        endOffset,
      } = slice.target;
      return ((startOffset < passageEnd) && (endOffset > passageStart))
        ? [position,]
        : [];
    },);
  /**
   First slice over the passage.
   */
  const [first,] = over;
  /**
   Last slice over the passage.
   */
  const last = over.at(-1,);
  if ((first === undefined) || (last === undefined)) {
    throw new FidelityReferenceError({
      referenceId: reference.spec
        .id,
      operation: 'archive',
    },);
  }
  /**
   The slices with the ones over the passage taken as one, so that the passage is one position and its neighbours are
   the slices either side of it.
   */
  const slices = [
    ...prepared.slices
      .slice(
      0,
      first + 1,
      ),
    ...prepared.slices
      .slice(last + 1,),
  ];
  return {
    neighbouringSourceText: neighbouringSource({
      slices,
      slicePosition: first,
    },),
    neighbouringIncumbentText: neighbouringIncumbent({
      slices,
      slicePosition: first,
    },),
    documentSourceText: reference.pages
      .sourceFile,
    ...((prepared.identityContext === undefined) ? {} : { identityContext: prepared.identityContext, }),
  };
}

/**
 The context as the fields the sheet builders take.

 @param context - context prepared for a trial

 @returns The neighbouring texts, the whole original and the identity context when there is one, to spread into the
 call that builds the sheet

 @example
 ```ts
 const plan = buildEditorMessages({ sourceText, targetText, envelopes, issues, ...sheetFieldsOf({ context, },), },);
 ```
 */
export function sheetFieldsOf({ context, }: { readonly context: EditorSheetContext; },): {
  readonly neighbouringSourceText: string;
  readonly neighbouringIncumbentText: string;
  readonly documentSourceText: string;
  readonly identityContext?: string;
} {
  return {
    neighbouringSourceText: context.neighbouringSourceText,
    neighbouringIncumbentText: context.neighbouringIncumbentText,
    documentSourceText: context.documentSourceText,
    ...((context.identityContext === undefined) ? {} : { identityContext: context.identityContext, }),
  };
}

//endregion Editor fidelity sheet context
