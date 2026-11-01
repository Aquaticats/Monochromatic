import { MIN_RATIO_SOURCE_CHARS, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type RatioImplausibility,
  ratioImplausibility,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Contest size note
// Size evidence handed to a contest judge when one rendering of a passage is
// far out of proportion to the Chinese it renders.
//
// EVIDENCE, NOT A FAULT NAME, and that distinction is the whole design. A named
// fault in the FAR LONGER direction would contradict `CONTEST_POLICY` itself,
// which already tells judges that where the Chinese is silent rather than
// contradicting, keeping page apparatus or a whole page region the archive
// carries is CORRECT. A candidate preserving a
// long page-only region is far longer than the Chinese AND is the right
// candidate, so a name there would instruct a judge to penalise exactly the
// behaviour that rule protects. That is the shape of a criterion once
// removed: one that licenses the wrong outcome.
//
// MEASURED BEFORE IT WAS BUILT, over all 11 settled artifacts, 184 rows and 116
// eligible after the floor. No slice any lane actually PRODUCED trips a tail:
// 0 of 92, with produced ratios reaching 9.27 against a 10 endpoint and the
// translate lane never passing 4.59. Every trip is a slice where a lane handed
// back the archive unchanged. Both far-longer trips proved to be page-only
// content rather than displacement, established by profiling the whole document
// and finding NO DONOR SLICE: on `dogesir_` slice 3 the archive runs 15.49
// times its Chinese while every other slice runs 0.88 to 6.10 against a
// document of 2.75, so no other slice is starved of the text this one holds.
//
// THE FLOOR IS ONE-DIRECTIONAL, and it did not start that way. A single
// source-length floor guarded both directions until a census over every
// settled artifact showed it inverting the instrument: 40 far-longer
// candidates exist, the note reported 3, and the floor silenced 37. The three
// it reported ran 15.5, 15.5 and 10.1; among those it hid were 185.4, 137.0,
// 100.8, 99.1 and 94.5. The largest, a translate candidate of 10381 characters
// against a 56-character original, is a lane looping rather than a rendering,
// and the judges deciding that passage were told nothing about it.
//
// WHY NO DOCUMENT BASELINE IS THREADED HERE. `PLAUSIBLE_BASELINE_MIN` of 1.9
// and `PLAUSIBLE_BASELINE_MAX` of 4.5 sit strictly inside the 0.8 and 10
// endpoints, so a slice tripping an absolute tail is already outside every
// document norm the estimator will accept. Reading the document's own baseline
// could not change whether the note appears, and threading it would put
// document state through two wire builders for nothing.
//
// WHY NO VERSE MARKER IS THREADED EITHER. A fault name would have needed one,
// because a line-structured original expands unusually and a judge told
// "this is a fault" has no way to discount it. Evidence needs no such guard:
// the policy states outright that a large ratio can be innocent and names verse
// as one reason. Revisit if judges are measured misreading verse ratios.

/**
 Reasons a short original cannot support, because over a twenty-character
 line the ratio reports rounding rather than a rendering.

 ONLY THE SHORTER DIRECTION, and the asymmetry is the whole point. A
 20-character original against a 12-character rendering is noise, and reading
 it as a shortfall would fire the note on every short line in the corpus. A
 56-character original against a 10381-character rendering is not rounding in
 either direction: it is a runaway, and it is exactly what this note exists
 to put in front of a judge.

 A DELIBERATE SUBSET of `RatioImplausibility`, which is `target-far-shorter`
 and `target-far-longer`: this names the first, and `target-far-longer` is
 the direction left out. A ratio reason the measurement gains is therefore not
 floored until it is added here.

 @example
 ```ts
 FLOORED_REASONS.has('target-far-shorter',);
 ```
 */
const FLOORED_REASONS: ReadonlySet<RatioImplausibility> = new Set([
  'target-far-shorter',
],);

/**
 One rendering of a passage, under the name its own contest calls it.

 THE LABEL IS THE CALLER'S, because the two contests name their candidates
 differently and `CONTEST_POLICY` is shared between them precisely by naming
 neither. A builder hardcoding one contest's vocabulary could not serve both.
 */
export type ContestRendering = {
  /**
   How the surrounding message refers to this rendering.
   */
  readonly label: string;

  /**
   Rendering itself.
   */
  readonly text: string;
};

/**
 Whether one rendering's size against its original is outside plausible range.

 RATIO TAILS ONLY, read off character counts. A block-count gap says the two
 sides may not be the same passage, which makes a ratio meaningless rather
 than extreme, and it was the sole cause for 20 of 36 flagged slices while
 being the thing a re-pairing moves; showing a judge a ratio the pairing does
 not support would be showing it noise. Reading no blocks also means parsing
 no rendering, so a rendering the slice grammar refuses still gets its note
 (ledger B68).

 @param sourceText - original passage

 @param text - one rendering of it

 @returns Whether a ratio tail applies, with the source-length floor applied
 only to the reasons {@link FLOORED_REASONS} names

 @example
 ```ts
 const tailed = tripsARatioTail({ sourceText, text: repairText, },);
 ```
 */
function tripsARatioTail(
  {
    sourceText,
    text,
  }: {
    readonly sourceText: string;
    readonly text: string;
  },
): boolean {
  /**
   Every ratio tail this pair's character counts reach.
   */
  const reasons = ratioImplausibility({
    sourceChars: sourceText.length,
    targetChars: text.length,
  },);

  /**
   Whether the original is long enough for a SHORTFALL against it to mean
   anything. A surplus needs no such support, per {@link FLOORED_REASONS}.
   */
  const longEnoughToFallShortOf = sourceText.length >= MIN_RATIO_SOURCE_CHARS;

  return reasons.some(function describesTheRendering(
    reason: RatioImplausibility,
  ): boolean {
    if (!FLOORED_REASONS.has(reason,))
      return true;
    return longEnoughToFallShortOf;
  },);
}

/**
 Builds the size evidence for one contested passage, or nothing to say.

 SILENT UNLESS SOMETHING IS OUT OF PROPORTION, so that a judge reading a note
 knows the note is about this passage rather than boilerplate it can skim. On
 the settled corpus that is 2 of 116 eligible rows.

 A SHORTFALL against an original shorter than {@link MIN_RATIO_SOURCE_CHARS}
 is passed over, because a ratio over a twenty-character line reports
 rounding. A SURPLUS is reported at any original length, and
 {@link FLOORED_REASONS} carries the measurement behind that split.

 A rendering of zero length raises nothing here, because
 {@link ratioImplausibility} reports no reason when either side is empty. An
 empty candidate is a different failure, and the stage guards name it.

 @param sourceText - original passage, which every ratio is taken against

 @param renderings - archive rendering and both candidates, each under its own
 contest's name

 @returns Note for the message, or an empty string when nothing is out of
 proportion

 @example
 ```ts
 const note = contestSizeNote({
   sourceText,
   renderings: [ { label: 'ARCHIVE RENDERING', text: incumbentText, }, ],
 },);
 ```
 */
export function contestSizeNote(
  {
    sourceText,
    renderings,
  }: {
    readonly sourceText: string;
    readonly renderings: readonly ContestRendering[];
  },
): string {
  /**
   Original's size, which every ratio divides by.
   */
  const sourceChars = sourceText.length;

  if (!renderings.some(function isOutOfProportion(rendering,): boolean {
    return tripsARatioTail({
      sourceText,
      text: rendering.text,
    },);
  },))
    return '';

  return [
    'SIZE NOTE for this passage, in characters. Evidence, not a verdict.',
    `  Chinese original: ${String(sourceChars,)}`,
    ...renderings.map(function toLine(
      {
        label,
        text,
      },
    ): string {
      /**
       This rendering's own size.
       */
      const chars = text.length;

      /**
       How far it departs from its original in size.
       */
      const ratio = chars / sourceChars;

      return `  ${label}: ${String(chars,)} (${ratio.toFixed(1,)} times the original)`;
    },),
  ].join('\n',);
}

/**
 Policy teaching a judge how to read {@link contestSizeNote}.

 NAMES NO CANDIDATE, because it joins `CONTEST_POLICY`, which the lane contest
 and the consolidate gate share and which their differing candidate names
 would otherwise split in two.

 BOTH DIRECTIONS GET A READING RATHER THAN A NAME. The two readings differ in
 what they ask the judge to do, which is the property that made two named
 faults attractive in the first place, and they carry it without asserting a
 fault where the existing rule says the behaviour is correct.

 SILENT SURPLUS IS PAGE CONTENT ONLY WHERE THE ARCHIVE CARRIES IT (ledger S7).
 The note exists for the looping candidate (10381 characters against a
 56-character original), and the sentence that called any surplus the Chinese
 is silent about page content told the judge the loop's repetition was a page
 region to keep. Surplus the archive does not carry is the candidate's own.
 */
export const SIZE_NOTE_POLICY: string = [
  'A SIZE NOTE MAY APPEAR WITH THE PASSAGES, reporting how many characters each rendering runs against the Chinese.',
  'It is evidence about where to look, never a verdict, and it names no fault by itself.',
  'Where a rendering is FAR SHORTER than the Chinese, it may have left Chinese content unrendered: put the DROPPED question to that rendering in particular.',
  'Where a rendering is FAR LONGER than the Chinese, two readings are open, and the surplus text decides between them.',
  'Surplus the ARCHIVE RENDERING also carries, where the Chinese is SILENT about it, is page content: the DROPPED-ALSO rule governs it and the shorter candidate is the one that lost something.',
  'Surplus the archive rendering does not carry is the rendering\'s own and is unsupported: a passage repeated or looping, or text the Chinese never states. So is surplus that CONTRADICTS the Chinese, or that reads as belonging to a different passage.',
  'SIZE ALONE SETTLES NEITHER READING. A line-structured original, and one the archive spells out at length, both produce a large ratio with nothing wrong.',
].join('\n',);

//endregion Contest size note
