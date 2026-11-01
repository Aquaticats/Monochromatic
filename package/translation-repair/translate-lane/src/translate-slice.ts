import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  declaredNameRefusalFinding,
  findDroppedDeclaredNames,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { assertAbsentSliceFilled, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  countQuotedPassages,
  dropsQuotedPassage,
  quoteLossRefusalFinding,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { restoreTargetOnlyRun, } from './target-only-run.ts';
import { assessSliceAlignment, } from './translate-alignment.ts';
import {
  ARCHIVE_INELIGIBLE_FINDING,
  type ArchiveFloorVerdict,
  archiveFloorVerdict,
} from './translate-archive-floor.ts';
import {
  type TranslateModels,
  type TranslateSliceRecord,
  TRANSLATE_SLICE_CACHE_VERSION,
} from './translate-document-contract.ts';
import {
  runTranslateStage,
  type TranslationRepairInterruptedError,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { translateSliceInput, } from './translate-slice-input.ts';
import {
  type ArchiveDispute,
  archiveDisputeNote,
  describeArchiveDispute,
  disputedWordingsOf,
} from '@monochromatic-dev/translation-repair-archive/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Translate slice
// One slice from prepared pair to settled record: translate it, judge it, and
// decide whether the archive text may be replaced by what won.
//
// The alignment guard runs AFTER the stage rather than instead of it. Running
// it first would make the lane skip slices, and a skipped slice is
// indistinguishable in every artifact from a slice the judges left alone. It
// also throws away the evidence: what the judges chose for a mispaired slice is
// exactly what says the pairing was wrong.

/**
 Translates one slice and settles what the driver accepts for it.

 @param client - injected model client

 @param slice - prepared slice pair

 @param prepared - document the slice came from, for declared names and
 governance

 @param models - translator and judge rosters

 @param neighbouringSourceText - original of the sections either side, shown to
 the judges as context they are not asked to render. Absent by default, so the
 lane behaves exactly as it did; the window trial supplies it on the slices
 the displacement screen flags, to read whether the replacement rate falls when a judge can see
 that the archive put this slice's content next door

 @param neighbouringIncumbentText - archive English of the sections either
 side, shown so a passage missing here can be recognised next door rather than
 read as one the archive never had

 @param pictureContext - what the pictures this slice and its neighbours show
 were read as, shown to translators and judges as source evidence they could
 otherwise not see

 @param pictureFindings - one line per picture no reading is available for,
 carried into the record so a run says which pictures went unread rather than
 leaving their absence indistinguishable from a slice showing none

 @param archiveDispute - dispute over this slice's archive rendering (class
 one hundred seven), whose repair text is judged as the incumbent in the
 archive's place and ships where the judges keep it

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - driver logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Settled record, whether the stage's text was accepted or refused

 @throws {@link TranslationRepairInterruptedError}
 when absent-passage correction repeats exact task or providers remain unavailable

 @example
 ```ts
 const record = await settleTranslateSlice({ client, slice, prepared, models, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function settleTranslateSlice(
  {
    client,
    slice,
    prepared,
    models,
    neighbouringIncumbentText,
    neighbouringSourceText,
    pictureContext,
    pictureFindings = [],
    archiveDispute,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly slice: ChunkPair;
    readonly archiveDispute?: ArchiveDispute;
    readonly prepared: PreparedDocumentPair;
    readonly models: TranslateModels;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly pictureContext?: string;
    readonly pictureFindings?: readonly string[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<TranslateSliceRecord> {
  /**
   Global slice index every record and replacement names.
   */
  const {
    sliceIndex,
    text: pageWording,
  } = slice.target;

  /**
   Wordings a disputed slice refuses, none elsewhere; the stage and
   `archiveFloor` read the same list.
   */
  const disputedWordings = (archiveDispute === undefined)
    ? []
    : disputedWordingsOf({
      dispute: archiveDispute,
      archiveText: pageWording,
    },);

  /**
   Shared pre-stage protection and governance, without publication-disposition decisions.
   */
  const {
    archiveText,
    protectedText,
    stageInput,
  } = translateSliceInput({
    slice,
    prepared,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
    ...((pictureContext === undefined) ? {} : { pictureContext, }),
    // THE STAND-IN ONLY WHERE IT MAY STAND; elsewhere the archive stays the
    // incumbent and is refused with the repair lane's text (owner,
    // 2026-09-27, "No eligible standing").
    ...((archiveDispute === undefined)
      ? {}
      : {
        ...(archiveDispute.standInEligible ? { archiveStandIn: archiveDispute.standIn, } : {}),
        archiveDisputeNote: archiveDisputeNote({ dispute: archiveDispute, },),
        disputedWordings,
      }),
  },);
  /**
   The exact staged surface is also what later publication guards compare.
   */
  const {
    sourceText,
    incumbentText,
    incumbentKind,
  } = stageInput;
  /**
   Whether a stand-in differing from the archive's own bytes is on the slice,
   so a kept or refused surface still changes the page (class one hundred
   seven).
   */
  const standInDiffers = (archiveDispute?.standInEligible === true)
    && (archiveText !== pageWording);
  /**
   Whether the incumbent is wording the slice refuses, so no refusal this
   function raises may keep it: there is nothing to keep, and the consolidation's floors judge
   what the stage chose (owner, 2026-09-27).
   */
  const incumbentRefused = (archiveDispute !== undefined) && (!archiveDispute.standInEligible);
  /**
   Whether there is incumbent wording a replacement could damage.
   */
  const guardedIncumbent = (incumbentKind === 'present') && (!incumbentRefused);

  if (protectedText !== '')
    l.info(
      `translate slice ${String(sliceIndex,)}: holding ${
        String(protectedText.length,)
      } ${
        wordForCount({
          count: protectedText.length,
          one: 'character',
          many: 'characters',
        },)
      } of target-only English, with the line breaks before it, out of translation, `
        + `judging ${String(incumbentText.length,)} of ${String(archiveText.length,)}`,
    );

  /**
   What the translators wrote and the judges decided, through the unchanged stage operation.
   */
  const stageResult = await runTranslateStage({
    client,
    translatorModelIds: models.translatorModelIds,
    judgeModelIds: models.judgeModelIds,
    ...stageInput,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   The publication rule's answer on the archive a refusal in this function would keep,
   asked only where one could keep it (ledger X6): the consolidation refuses
   such an archive as a standing, so keeping it here only withheld the
   judges' replacement from the slate. Admitted where nothing would be kept.
   */
  const archiveFloor: ArchiveFloorVerdict = (guardedIncumbent && (stageResult.text !== incumbentText))
    ? archiveFloorVerdict({
      slice,
      prepared,
      archiveText,
      disputedWordings,
    },)
    : { admitted: true, };
  if (!archiveFloor.admitted) {
    l.warn(
      `translate slice ${String(sliceIndex,)}: the archive fails the publication rule (${archiveFloor.reason}); `
        + 'no refusal keeps it',
    );
  }

  /**
   Whether a refusal in this function may keep the archive: there is one to damage and
   the publication rule admits it.
   */
  const keepsArchive = guardedIncumbent && archiveFloor.admitted;

  /**
   What this slice reports, the stage's own findings plus one line per picture
   nobody could read.

   A PICTURE THAT WENT UNREAD IS NOT THE SAME AS A SLICE SHOWING NONE, and
   without this line the two are identical in every artifact. The reading is
   evidence the translators and judges were promised and did not get, so a
   reader asking why a slice decided as it did needs to know it was missing.
   */
  const findings: readonly string[] = [
    ...stageResult.findings,
    ...pictureFindings,
    ...((archiveDispute === undefined) ? [] : [describeArchiveDispute({ dispute: archiveDispute, },),]),
    ...(archiveFloor.admitted ? [] : [ARCHIVE_INELIGIBLE_FINDING,]),
  ];

  /**
   Whether this slice's two sides can be the same passage.
   */
  const alignment = assessSliceAlignment({
    sourceText,
    incumbentText,
  },);

  // A SLICE NOBODY WAS ASKED ABOUT SETTLES ON THE ARCHIVE'S OWN BYTES (ledger
  // B43): the floor could compare nothing here, so a disputed slice's
  // stand-in, the repair lane's text, is as unchecked as any candidate would
  // have been, and a record that heard nobody may carry only the archive
  // (`assertUnheardKeptIncumbent`).
  if (stageResult.decision === 'unfloored') {
    return {
      kind: 'translate-slice',
      schemaVersion: TRANSLATE_SLICE_CACHE_VERSION,
      sliceIndex,
      stageResult,
      outputText: pageWording,
      changed: false,
      disposition: 'stage-result',
      alignment,
      findings,
    };
  }

  /**
   Whether the stage wants to change the archive text at all.
   */
  const wantsReplacement = stageResult.text !== incumbentText;

  /**
   Whether the guard stands in the way of that.

   Only a REPLACEMENT can be refused. A slice the judges left alone needs no
   permission to stay as it is, and refusing it would report a protection that
   protected nothing.

   AND ONLY WHERE THERE IS SOMETHING TO PROTECT. The guard exists to stop a
   short source replacing a long translation the source cannot account for; at
   an anchor there is no translation to lose, so a refusal there would put the
   empty string back over a rendering the judges chose and settle the slice as
   an ordinary unchanged one, which is the exact wrong-success state absent
   mode exists to remove.
   */
  const refused = keepsArchive
    && wantsReplacement
    && (alignment.kind === 'incumbent-dominates-source');

  /**
   Whether this slice is one the quote and declared-name guards apply to at
   all.

   Only a slice whose archive text is being replaced can lose a quote or a
   name from it.
   */
  const guardsThisSlice = keepsArchive
    && wantsReplacement
    && (!refused);

  // WHETHER THE REPLACEMENT WOULD LEAVE FEWER QUOTED PASSAGES than the judged
  // archive carries, its held-out transcript aside, since that is restored
  // onto any replacement.
  //
  // A SEPARATE GUARD FROM THE ALIGNMENT ONE, because a ratio and a structure
  // catch different things. The alignment guard refuses above sixteen times the
  // source length, and the two transcripts measured on 2026-08-18 sat at 15.49
  // and 8.71: a near miss and nowhere near. Counting quoted passages catches
  // both, and over sixty-nine natural rows it caught nothing else.
  //
  // THE FLOOR ASKS THIS FIRST NOW (ledger B110): `floorTranslateVoices`
  // compares every translator candidate's quoted passages, at every depth,
  // against this same `incumbentText`, read by the same `readPageSkeleton`,
  // so on a Markdown slice a winner that passed the floor carries as many.
  // Since ledger B112 the floor reads each answer folded, as it ships, so a
  // fold after the floor no longer reaches here either.
  //
  // NOT ON FRONT MATTER (ledger B113). Front matter is YAML, where a line
  // opening with `>` is a scalar's content, not a quoted passage; counted as
  // Markdown it refused a rendering its own floor, which checks YAML, had
  // passed. So no slice the pipeline settles reaches this refusal now. It
  // stays as one that ships the whole archive, the outcome a run must reach
  // whatever fails (owner, 2026-09-27), rather than a throw that would fail
  // the entry, for any future path that skips the floor's count.
  /**
   Whether the quote guard reads this slice: Markdown only.
   */
  const guardQuotedPassages = (slice.syntax === 'front-matter')
    ? false
    : guardsThisSlice;
  if (guardQuotedPassages) {
    /**
     Quoted passages on both sides, as the floor reads a page.
     */
    const quotedPassages = countQuotedPassages({
      incumbentText,
      shippedText: stageResult.text,
    },);
    if (dropsQuotedPassage({ quotedPassages, },)) {
      l.warn(
        quoteLossRefusalFinding({
          sliceIndex,
          quotedPassages,
        },),
      );
      return {
        kind: 'translate-slice',
        schemaVersion: TRANSLATE_SLICE_CACHE_VERSION,
        sliceIndex,
        stageResult,
        // The whole archive, for the reason the alignment refusal gives.
        outputText: archiveText,
        changed: standInDiffers,
        disposition: 'refused-quote-loss',
        quotedPassages,
        alignment,
        findings,
      };
    }
  }
  /**
   Whether target-declared forms govern this ordinary prose slice.

   Front matter is where declarations themselves are corrected from source,
   so protecting target values there would make metadata unrepairable.
   */
  const guardDeclaredNames = slice.syntax === 'front-matter'
    ? false
    : guardsThisSlice;
  /**
   Declared names the archive text carries and an ordinary prose
   replacement does not.

   CHECKED RATHER THAN ASKED FOR. Probed against the repair lane's own judge
   sheet and roster, six of six judges preferred a candidate that dropped a
   declared alias, and stating the exception in the criterion moved their
   reasoning without moving the vote.
   */
  const droppedDeclaredNames = guardDeclaredNames
    ? findDroppedDeclaredNames({
      forms: prepared.declaredNames,
      ...((prepared.declaredSubjectForms === undefined) ? {} : { subjectForms: prepared.declaredSubjectForms, }),
      baseText: incumbentText,
      candidateText: stageResult.text,
    },)
    : [];
  if (droppedDeclaredNames.length > 0) {
    l.warn(
      declaredNameRefusalFinding({
        sliceIndex,
        dropped: droppedDeclaredNames,
      },),
    );
    return {
      kind: 'translate-slice',
      schemaVersion: TRANSLATE_SLICE_CACHE_VERSION,
      sliceIndex,
      stageResult,
      // The whole archive, for the reason the alignment refusal gives.
      outputText: archiveText,
      changed: standInDiffers,
      disposition: 'refused-declared-name',
      droppedDeclaredNames,
      alignment,
      findings,
    };
  }
  if (refused) {
    l.warn(
      `translate slice ${String(sliceIndex,)}: keeping the archive text, `
        + `${String(alignment.incumbentCodePoints,)} code points against a `
        + `source of ${String(alignment.sourceCodePoints,)}`,
    );
    return {
      kind: 'translate-slice',
      schemaVersion: TRANSLATE_SLICE_CACHE_VERSION,
      sliceIndex,
      stageResult,
      // THE WHOLE ARCHIVE, protected run included, rather than the judged part.
      // A retention has to leave the document byte-identical, and the judged
      // part is a slice of the archive rather than the archive.
      outputText: archiveText,
      changed: standInDiffers,
      disposition: 'refused-alignment',
      alignment,
      // NOT the refusal sentence, which names a slice by its index. This record
      // is STORED, and since translate version 2 its key no longer carries the
      // index, so the same record can be resumed at a different position and is
      // re-stamped when it is. A stored sentence saying `slice 7` would survive
      // that re-stamping and contradict the record carrying it. Nothing is lost
      // by leaving it out: `disposition` and `alignment` are both here, so the
      // driver derives the sentence from them and from the index the record was
      // actually stamped with.
      findings,
    };
  }

  // WHAT AN ABSENT SLICE MAY SETTLE ON, stated where the record is built rather
  // than trusted to the paths that produce it: a record for a passage the
  // archive never translated always carries a translation.
  assertAbsentSliceFilled({
    incumbentKind,
    text: stageResult.text,
    findings: stageResult.findings,
  },);

  /**
   What this slice leaves the document with.

   THE ARCHIVE'S OWN BYTES WHEN NOTHING CHANGED, rather than a reconstruction
   of them. Restoring a protected run onto an unchanged judged part rebuilds
   the same passage, and a rebuild that differs by so much as a trailing
   newline reports a change nobody made.
   */
  const outputText = wantsReplacement
    ? restoreTargetOnlyRun({
      text: stageResult.text,
      protectedText,
    },)
    : archiveText;

  return {
    kind: 'translate-slice',
    schemaVersion: TRANSLATE_SLICE_CACHE_VERSION,
    sliceIndex,
    stageResult,
    outputText,
    changed: wantsReplacement || standInDiffers,
    disposition: 'stage-result',
    alignment,
    findings,
  };
}

//endregion Translate slice
