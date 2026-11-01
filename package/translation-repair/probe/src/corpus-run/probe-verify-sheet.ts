import { createHash, } from 'node:crypto';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { fenceForMarkdown, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ScreenedDefectClaim, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { RelabelCase, } from './probe-relabel-case.ts';

//region Probe verify sheet
// Formats the sheet that decides whether the unlabelled probe finds damage or
// invents it.
//
// The relabel runs left exactly one thing unsettled. Withholding the accepted
// issues makes the probe flag every damaged region and roughly four in ten
// unflagged ones, and "unflagged" only ever meant "nobody read it". If those
// four are real damage the reader never saw, the probe is a detector worth
// gating on. If they are inventions, its precision is near half and gating
// would discard correct repairs.
//
// The sheet is therefore BLIND. Items are ordered by a digest of their identity
// rather than by kind, and nothing on the page says whether an item came from
// the damaged set or the control set, because a grader told which is which
// would be answering a different question.
//
// The sheet quotes unlicensed corpus text. It is written outside the repository
// and must never be committed, pasted into a third-party model, or shared.

/**
 One region to be judged, with what the probe said about it.

 @example
 ```ts
 const item: VerifyItem = { relabelCase, claims, kind: 'control', };
 ```
 */
export type VerifyItem = {
  /**
   Region and the texts surrounding it.
   */
  readonly relabelCase: RelabelCase;

  /**
   Claims the unlabelled probe raised. A sheet that prints claims is handed
   only those the screen corroborated; the damage sample keeps every claim for
   its manifest and prints none.
   */
  readonly claims: readonly ScreenedDefectClaim[];

  /**
   Which partition this item belongs to, recorded in the manifest and never
   on the sheet.

   Deliberately a free label rather than a fixed pair, because the two sheets
   that share this formatter partition on different things and reusing one
   vocabulary for both would put the same word on two meanings. The
   verification sheet splits by what a READER already believed, and the damage
   sample splits by what the PROBE said. A scorer reads the label as written
   and never infers a meaning from it.
   */
  readonly kind: string;
};

/**
 One verify item paired with the digest its blind order sorts on.

 Named rather than inferred, because an inferred object literal carries
 writable properties, and the comparator and unwrapping map that read it then
 take mutable parameters they never mutate.
 */
type DigestedItem = Readonly<{
  /**
   Item being ordered.
   */
  item: VerifyItem;

  /**
   Stable hash of the edit's identity, the sort key.
   */
  digest: string;
}>;

/**
 Orders items by a digest of their identity.

 Deterministic so a re-run produces the same sheet, and independent of kind so
 the damaged and control items interleave. Ordering by anything the grader
 could infer, insertion order included, would leak the answer.

 @param items - items to order

 @returns Same items, digest order

 @example
 ```ts
 const ordered = orderBlind({ items, },);
 ```
 */
export function orderBlind(
  { items, }: { readonly items: readonly VerifyItem[]; },
): readonly VerifyItem[] {
  return items
    .map(function withDigest(item,): DigestedItem {
      /**
       Region and texts this item judges.
       */
      const { relabelCase, } = item;

      /**
       Entry and region naming the edit.
       */
      const {
        entryId,
        region,
      } = relabelCase;

      /**
       Identity of the edit this item judges.
       */
      const identity = `${entryId} ${region.envelopeId}`;

      return {
        item,
        digest: createHash('sha256',)
          .update(identity,)
          .digest('hex',),
      };
    },)
    .toSorted(function byDigest(
      left,
      right,
    ) {
      return compareCodePoints({
        left: left.digest,
        right: right.digest,
      },);
    },)
    .map(function toItem(entry,) {
      return entry.item;
    },);
}

/**
 Renders one claim as the reviewer's stated finding.

 The side quoted is the side the screen corroborated the claim on, read off
 the claim's own admissibility. Choosing it by whether the raw omitted text
 was empty disagreed with the screen, which reads a side of spaces as no
 anchor: such a claim was printed as dropped wording over a blank quote
 (ledger B128).

 @param claim - claim the screen corroborated

 @returns Markdown lines for this claim

 @throws {@link Error} when the screen did not corroborate the claim, which
 no sheet that prints claims is handed

 @example
 ```ts
 const block = renderClaim({ claim, },);
 ```
 */
function renderClaim({ claim, }: { readonly claim: ScreenedDefectClaim; },): string {
  if ((claim.admissibility !== 'corroborated') && (claim.admissibility !== 'removal-corroborated')) {
    throw new Error(
      `unreachable: a claim the screen read as ${claim.admissibility} on a sheet, though the probe hands a sheet only claims the screen corroborated, each with one anchored side to quote`,
    );
  }

  /**
   Whether the screen corroborated the claim on wording the edit dropped.
   */
  const removal = claim.admissibility === 'removal-corroborated';

  /**
   Wording the claim anchors on, as the prober wrote it.
   */
  const quoted = removal ? claim.omittedText : claim.evidence;

  /**
   Which direction the claim runs, in plain words.
   */
  const direction = removal
    ? 'wording the edit DROPPED'
    : 'wording the edit ADDED or altered';

  return [
    `-   Says the edit introduced: ${claim.category}`,
    `-   Quotes this as ${direction}:`,
    '',
    fenceForMarkdown({ text: quoted, },),
    '',
    `-   Its reason:`,
    '',
    fenceForMarkdown({ text: claim.reason, },),
  ].join('\n',);
}

/**
 Whether an item carries a machine claim to show the reader.

 @param item - item being rendered

 @returns Whether any claim accompanies it

 @example
 ```ts
 const shown = hasClaims({ item, },);
 ```
 */
function hasClaims({ item, }: { readonly item: VerifyItem; },): boolean {
  /**
   Claims accompanying this item.
   */
  const { claims, } = item;

  return claims.length > 0;
}

/**
 Renders one sheet item.

 @param item - item to render

 @param position - one-based position on the sheet

 @returns Markdown section

 @example
 ```ts
 const section = renderItem({ item, position: 1, },);
 ```
 */
function renderItem(
  {
    item,
    position,
  }: {
    readonly item: VerifyItem;
    readonly position: number;
  },
): string {
  /**
   Region and texts this item judges.
   */
  const { relabelCase, } = item;

  /**
   Slice texts and the replaced region.
   */
  const {
    sourceText,
    baselineText,
    region,
  } = relabelCase;

  /**
   Replaced text and its replacement.
   */
  const {
    before,
    editorAfter,
  } = region;

  return [
    `### ${String(position,)}. grade: [ ]  (Y = the edit damaged the text · N = the edit is acceptable)`,
    '',
    'Original:',
    '',
    fenceForMarkdown({ text: sourceText, },),
    '',
    'Translation of that passage BEFORE the edit:',
    '',
    fenceForMarkdown({ text: baselineText, },),
    '',
    'The edit replaced this text:',
    '',
    fenceForMarkdown({ text: before, },),
    '',
    'with this text:',
    '',
    fenceForMarkdown({ text: editorAfter, },),
    '',
    // A machine claim is shown only when the sheet's purpose is to judge THAT
    // claim. On a sheet drawn to measure repair quality it is withheld, because
    // telling a reader what an automated reviewer already concluded moves the
    // answer, which is the same reason the detection sheet shows no correction.
    ...(hasClaims({ item, },)
      ? [
        'An automated reviewer says the edit introduced a defect:',
        '',
        ...item.claims
          .map(function toBlock(claim,) {
            return renderClaim({ claim, },);
          },),
        '',
      ]
      : []),
    'Y if the edit lost or distorted something the original supports.',
    'N if the edit is acceptable, even where it is not the wording you would',
    'have chosen, and N if the only problem is a defect that was already there',
    'before the edit.',
    '',
  ].join('\n',);
}

/**
 What the sheet tells the grader about the reviewer.

 `reviewer-claims`: every item was flagged and the claims are printed, which
 is the verify sheet. `blind`: the items mix flagged and silent ones with the
 claims stripped, which is the damage sheet; telling that grader every item
 was flagged primed a Y on exactly the partition scored as probe misses.

 @example
 ```ts
 const framing: SheetFraming = 'blind';
 ```
 */
export type SheetFraming = 'reviewer-claims' | 'blind';

/**
 Formats the whole blind verification sheet.

 @param items - items to judge, any order; ordering is applied here

 @param framing - what to tell the grader about the reviewer; the verify
 sheet keeps the default, the damage sheet is blind

 @returns Sheet markdown

 @example
 ```ts
 const sheet = formatVerifySheet({ items, },);
 ```
 */
export function formatVerifySheet(
  {
    items,
    framing = 'reviewer-claims',
  }: {
    readonly items: readonly VerifyItem[];
    readonly framing?: SheetFraming;
  },
): string {
  /**
   Items in blind order.
   */
  const ordered = orderBlind({ items, },);

  return [
    '# Does the unlabelled probe find damage, or invent it?',
    '',
    ...((framing === 'reviewer-claims')
      ? [
        'Every item below is an edit the pipeline applied, and an automated reviewer',
        'claims each one introduced a defect. Some of these claims are probably',
        'right and some are probably wrong; the point of this sheet is to find out',
        'which, and the items are deliberately in an order that tells you nothing.',
      ]
      : [
        'Every item below is an edit the pipeline applied. An automated reviewer',
        'flagged some of them as damaging and stayed silent on the others; you are',
        'not told which are which, the point of this sheet is to find out whether',
        'it flagged the right ones, and the items are deliberately in an order that',
        'tells you nothing.',
      ]),
    '',
    'Judge only whether the EDIT damaged the translation. A passage that was',
    'already wrong before the edit is not damage the edit caused, and the',
    'reviewer has been told that too, so a claim pointing at a pre-existing',
    'problem is a wrong claim.',
    '',
    'Replace each `[ ]` on a heading with `Y` or `N`.',
    '',
    `Items: ${String(ordered.length,)}`,
    '',
    ...ordered
      .map(function toSection(
        item,
        index,
      ) {
        return renderItem({
          item,
          position: index + 1,
        },);
      },),
  ].join('\n',);
}

/**
 Builds the manifest that scores the sheet.

 Written beside the sheet rather than into it, because the sheet is blind and
 a grader who can see which items came from the damaged set is answering a
 different question than the one being asked.

 @param items - items to judge, any order; ordering matches the sheet

 @returns Manifest JSON

 @example
 ```ts
 const manifest = formatVerifyManifest({ items, },);
 ```
 */
export function formatVerifyManifest(
  { items, }: { readonly items: readonly VerifyItem[]; },
): string {
  return JSON.stringify(
    {
      items: orderBlind({ items, },)
        .map(function toEntry(
          item,
          index,
        ) {
          /**
           Region and texts this entry names.
           */
          const { relabelCase, } = item;

          /**
           Entry and region naming the edit.
           */
          const {
            entryId,
            region,
          } = relabelCase;

          return {
            position: index + 1,
            entryId,
            envelopeId: region.envelopeId,
            kind: item.kind,
            claimants: item.claims
              .map(function toModelId(claim,) {
                return claim.modelId;
              },),
          };
        },),
    },
    undefined,
    2,
  );
}

//endregion Probe verify sheet
