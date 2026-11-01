import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { IncumbentKind, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  MAX_INCUMBENT_TO_SOURCE_RATIO,
  MIN_PROTECTED_INCUMBENT,
} from './translate-alignment.ts';
import {
  type TranslateModels,
  TRANSLATE_SLICE_CACHE_VERSION,
} from './translate-document-contract.ts';

//region Translate slice key
// What makes two runs' translate slices the SAME slice, for cache purposes.
//
// Split from the driver for the same two reasons `repair-slice-key.ts` was: the
// key is the one piece of a driver that can be tested without a client, and a
// reader of the driver does not want the cache reasoning in the middle of it.
// The two lanes cache into one directory, so their key derivations are meant to
// be read side by side.

/**
 Everything about this run that changes what the models are ASKED, folded into
 every cache key.

 Without it a resumed slice could return a record produced under a different
 roster or a different alignment threshold, and nothing would look wrong: the
 texts match, so the key matches. Identity context belongs here for the same
 reason, since it is front-matter-derived prompt content that varies per pair.

 `perCallTimeoutMs` is deliberately ABSENT. It changes how long a voice has to
 answer, not what it is asked, and the roster retries to quorum, so the heard
 set already varies between runs over one key. Including it would make every
 deadline change discard every settled translation in the corpus.

 @param models - translator and judge rosters

 @param identityContext - declared names travelling with every slice

 @param lifeEnded - whether the entry records a death, folded in only when it
 does, since the judges then see the tense candidates

 @param subjectNames - names the translation declares for its subject, folded in
 with a recorded death, since the tense candidates read each as a subject

 @param referenceContext - what the pages the original cites say, folded in
 only when the original cites any

 @param attestedLines - attested details the translators are shown, folded in
 only when there are any: the same slice written before the writer sheet
 carried them is a different question (class thirty-nine, 2026-09-16)

 @returns Stable string for the key

 @example
 ```ts
 const runShape = translateRunShape({ models, identityContext, },);
 ```
 */
export function translateRunShape(
  {
    models,
    identityContext,
    lifeEnded,
    subjectNames,
    referenceContext,
    attestedLines = [],
  }: {
    readonly models: TranslateModels;
    readonly identityContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly referenceContext?: string;
    readonly attestedLines?: readonly string[];
  },
): string {
  return JSON.stringify([
    models.translatorModelIds,
    models.judgeModelIds,
    identityContext ?? '',
    ...(((referenceContext ?? '') === '') ? [] : [referenceContext,]),
    ...((attestedLines.length === 0) ? [] : [attestedLines,]),
    ...((lifeEnded === true)
      ? [
        [
          'life-ended',
          subjectNames ?? [],
        ],
      ]
      : []),
    MIN_PROTECTED_INCUMBENT,
    MAX_INCUMBENT_TO_SOURCE_RATIO,
  ],);
}

/**
 Cross-run key for one slice under the translate lane.

 THE SLICE INDEX IS NOT IN IT, since version 2, and that is the whole design.
 A key is what makes two runs' slices the same slice, and what a translator is
 asked is the source text, the incumbent, the governance flag and the run
 shape. Where the slice happens to sit changes none of it.

 WHAT KEEPING IT COST. Any renumbering invalidated every slice after the
 change however untouched its text: inserting one slice at the top of a
 document discarded the whole document's settled work, and one-sided slicing inserts
 slices for every untranslated section. The corpus would have been rebought on
 that change and on every slicing change after it.

 WHAT DROPPING IT COSTS, measured rather than assumed: two slices carrying
 identical source text, identical incumbent and identical governance inside
 one document now share an entry. Their models would decide identically, so
 the shared record is right rather than merely cheap, and the caller stamps
 the index it asked under onto what it resumes. Across the 92 pinned documents
 and 1260 slices there is not one such pair; the probe was validated first on
 an invented document with two identical sections, where it finds the pair.

 @param runShape - what this run asks, from {@link translateRunShape}

 @param sourceText - slice original

 @param incumbentText - translation already there

 @param incumbentKind - whether there is a translation to fall back on

 @param syntax - syntax role changing model instructions and validation

 @param lineStructured - whether the enclosing chunk is line-structured

 @param neighbouringSourceText - original of the sections either side, shown to
 the judges as context. Absent for every ordinary run, and absence is
 serialized as though this parameter did not exist, so a settled corpus keyed
 before it existed stays valid

 @param pictureContext - what the pictures this slice and its neighbours show
 were read as, appended only when a reading was corroborated. A slice judged
 with a picture's text in front of it can reach a different answer than one
 judged without, so the two are not the same question and must not share a key

 @param neighbouringIncumbentText - archive English of the sections either
 side, shown so a passage missing here can be recognised next door rather than
 read as one the archive never had

 @param archiveDisputeNote - the dispute note a disputed slice is judged
 under, absent on an undisputed one

 @returns Hash keying this slice's record

 @example
 ```ts
 const key = translateSliceKey({ runShape, sourceText, incumbentText, incumbentKind, lineStructured, },);
 ```
 */
export function translateSliceKey(
  {
    runShape,
    sourceText,
    incumbentText,
    incumbentKind,
    syntax,
    lineStructured,
    neighbouringIncumbentText,
    neighbouringSourceText,
    pictureContext,
    archiveDisputeNote,
  }: {
    readonly runShape: string;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly incumbentKind: IncumbentKind;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly pictureContext?: string;
    readonly archiveDisputeNote?: string;
  },
): string {
  return hashContent({
    content: JSON.stringify([
      'translate',
      TRANSLATE_SLICE_CACHE_VERSION,
      runShape,
      sourceText,
      incumbentText,
      // THE QUESTION, not just its inputs. What a run asks about a slice with
      // no translation differs from what it asks about one that has any: the
      // first must be filled or left as a gap, the second may settle on what is
      // already there. Keying the mode makes the two separate questions rather
      // than one question with two answers.
      //
      // NOT A COLLISION FIX, which an earlier note here claimed. A whitespace-
      // only content span carries its whitespace rather than the empty string,
      // and a content chunk covering nothing is refused by the layout guard, so
      // no two currently valid slices carry identical texts and different
      // modes. This is explicit domain separation, and it holds if either of
      // those facts ever stops being true.
      incumbentKind,
      ...((syntax === undefined)
        ? []
        : [
          'syntax',
          syntax,
        ]),
      // Two slices can carry identical text and still be governed differently,
      // because the verdict belongs to the enclosing chunk.
      lineStructured,
      // APPENDED ONLY WHEN PRESENT, which is what keeps every settled slice in
      // the corpus valid: with no wider window the array is byte-identical to
      // what it always was, so no cache entry is discarded by this field
      // existing. Supplying one produces a different key, which it must, because
      // the judges are shown different evidence and can reach a different
      // answer. Two arms of one comparison sharing a key would have the second
      // read the first's result and report the two as identical.
      ...((neighbouringSourceText === undefined) || (neighbouringSourceText === '')
        ? []
        : [
          'neighbouring',
          neighbouringSourceText,
        ]),
      ...((neighbouringIncumbentText === undefined) || (neighbouringIncumbentText === '')
        ? []
        : [
          'neighbouring-incumbent',
          neighbouringIncumbentText,
        ]),
      // THE READING'S TEXT RATHER THAN ITS KEY, deliberately, even though the
      // readings are stored under a key of their own. What changes a judge's
      // answer is the words it was shown, and two runs that read one picture
      // into different words asked different questions however identical their
      // inputs were. The reading store is what stops that happening on a
      // RESUME; this is what makes it honest when it does.
      ...((pictureContext === undefined) || (pictureContext === '')
        ? []
        : [
          'pictures',
          pictureContext,
        ]),
      // THE DISPUTE NOTE, appended only when present like every optional field
      // of this key: a
      // slice judged under accepted claims against its archive asks a different
      // question from the same slice undisputed, and one whose archive is
      // refused asks another (owner, 2026-09-27).
      ...((archiveDisputeNote === undefined) || (archiveDisputeNote === '')
        ? []
        : [
          'dispute',
          archiveDisputeNote,
        ]),
    ],),
  },);
}

//endregion Translate slice key
