import { introducedFootnoteFindings, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { footnoteMentions, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Page footnote integrity
// THE PAGE IS A DOCUMENT THE LANES NEVER ASSEMBLED. Each lane guards its own
// assembly (`guardFootnoteAssembly`), but the page ships per slice whatever
// the contest and the consolidation chose, and that composition is a third
// document nobody parsed whole. The nineteenth `hakureico` pass of 2026-09-09
// shipped `“Mayday”[^1]` from a consolidation candidate with no `[^1]`
// definition anywhere: the translate lane had rendered the definition and its
// own guard had withdrawn it, and the page carried the reference without the
// note. Read here the way each lane reads its assembly, against the archive,
// so a defect the archive already carried is never blamed on the page.
//
// IT REPORTS AND NEVER STOPS THE ENTRY (B1181). Until 2026-11-01 UTC this check
// threw `TranslationRepairInterruptedError`, which the pass files as a stop with
// no page: XingZ6023 settled every slice, shipped no page and wrote no artifact
// for an orphan definition (slice 44 kept an archive without the reference
// while the translate lane's definition 3 shipped at slice 115). The owner's
// rules are "no matter what, we must ship" and that a page-agreement check ships
// with the defect reported, so this is a check of `publishFixedPage`, which
// collects its refusal as the `page-footnote-integrity` defect and ships.
//
// IT READS THE PAGE THAT SHIPS (B1182), the original of an untranslated
// passage under its note included. It once read the page with no original in it,
// so that the original's own footnote graph could never withdraw a slice the
// archive does not hold; nothing withdraws on this check now, and the lane
// assembly's proof of a withdrawal counts a withdrawn anchor as carrying its
// original (`assembly-withdrawal-proof.ts`), so the two read one page.

/**
 The original of an untranslated passage as the page carries it.

 @example
 ```ts
 const original: ShippedOriginal = { sliceIndex: 4, text: '她说[^7]\n', };
 ```
 */
export type ShippedOriginal = {
  /**
   Slice whose original it is.
   */
  readonly sliceIndex: number;

  /**
   The original as written on the page.
   */
  readonly text: string;
};

/**
 Failure raised when the page that ships carries a footnote defect its archive did not.

 @example
 ```ts
 throw new PageFootnoteIntegrityError({ entryId: 'BookshopCat', findings: ['page-footnote-orphan-definition gfm 3',], });
 ```
 */
export class PageFootnoteIntegrityError extends Error {
  /**
   Declares message safe to forward because it names the entry and footnote identifiers only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose page carries the defects.
   */
  readonly entryId: string;

  /**
   One finding per introduced defect, spelled `page-footnote-<kind> <convention> <identifier>`.
   */
  readonly findings: readonly string[];

  /**
   @param entryId - entry whose page carries the defects

   @param findings - one per introduced defect
   */
  public constructor(
    {
      entryId,
      findings,
    }: {
      readonly entryId: string;
      readonly findings: readonly string[];
    },
  ) {
    super(
      `entry ${entryId}: page introduces footnote defects the archive did not carry (count ${
        String(findings.length,)
      }): ${findings.join('; ',)}`,
    );
    this.name = 'PageFootnoteIntegrityError';
    this.entryId = entryId;
    this.findings = findings;
  }
}

/**
 Refuses a page that introduces a footnote defect the archive did not carry.

 @param entryId - entry whose page is weighed

 @param pageText - page as it ships, the original of an untranslated passage under its note included

 @param targetText - archive text the page is weighed against

 @param shippedOriginals - the original of each untranslated passage as the page carries it, with its slice, so a finding
 about a footnote that sits in one is named by the slice (ledger B1210); none by default

 @throws {@link PageFootnoteIntegrityError} naming each introduced defect, which `publishFixedPage` ships as a
 reported defect

 @example
 ```ts
 assertPageFootnotesIntact({ entryId: 'BookshopCat', pageText, targetText, },);
 ```
 */
export function assertPageFootnotesIntact(
  {
    entryId,
    pageText,
    targetText,
    shippedOriginals = [],
  }: {
    readonly entryId: string;
    readonly pageText: string;
    readonly targetText: string;
    readonly shippedOriginals?: readonly ShippedOriginal[];
  },
): void {
  /**
   Footnote defects the page carries and the archive did not.
   */
  const introduced = introducedFootnoteFindings({
    incumbentText: targetText,
    assembledText: pageText,
  },);
  if (introduced.length === 0)
    return;

  /**
   Each shipped original beside the footnotes it mentions, in any role.
   */
  const mentioning = shippedOriginals.map(function mentionsOf({
    sliceIndex,
    text,
  },): {
    readonly sliceIndex: number;
    readonly keys: ReadonlySet<string>;
  } {
    return {
      sliceIndex,
      keys: new Set(footnoteMentions({ text, },)
        .map(function keyOf({
          convention,
          identifier,
        },): string {
          return `${convention} ${identifier}`;
        },),),
    };
  },);
  throw new PageFootnoteIntegrityError({
    entryId,
    findings: introduced.map(function spell(finding,): string {
      /**
       Slices whose shipped original holds the footnote the finding is about.
       */
      const inside = mentioning
        .filter(function holdsIt({ keys, },): boolean {
          return keys.has(`${finding.convention} ${finding.identifier}`,);
        },)
        .map(function sliceOf({ sliceIndex, },): string {
          return String(sliceIndex,);
        },);
      /**
       Where the footnote sits, empty for one outside every shipped original.
       */
      const where = (inside.length === 0)
        ? ''
        : ` (inside the ${(inside.length === 1) ? 'original' : 'originals'} shipped at ${
          (inside.length === 1) ? 'slice' : 'slices'
        } ${inside.join(', ',)})`;
      return `page-footnote-${finding.kind} ${finding.convention} ${finding.identifier}${where}`;
    },),
  },);
}

//endregion Page footnote integrity
