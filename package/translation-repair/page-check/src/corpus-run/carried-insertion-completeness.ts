import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { CarriedInsertion, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  collapseSoftLineBreaks,
  normalizePunctuation,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { shippableReplacements, } from './shippable-replacements.ts';
import type { WouldShipSource, } from './would-ship-text.ts';

//region Carried insertion completeness
// READ THE PAGE THE WAY THE EVIDENCE WAS READ. A carried region is a quote the
// roster anchored through `locateQuote`, which folds soft line breaks and
// normalizes punctuation before it looks, so a region may span a break the
// archive never wrote. The semantic wrap (`consolidate-wrap.ts` and the two
// lane assemblies) then breaks a shipped slice at its clauses: the fifth
// `yuki418330012` launch of 2026-09-08 shipped
// `Contributors for this entry:\nZhenli,\nSansan,\nSuona` where the archive
// had it on one line, the region was read byte for byte against it, and the
// pass stopped as `carried-evidence-lost` with every word of the credits on
// the page. The twentieth class. Folded the same way on both sides, a moved
// break is nothing and a lost word is still lost.
//
// IT REPORTS AND NEVER STOPS THE ENTRY (B1181). A lost region used to throw
// `TranslationRepairInterruptedError`, which the pass files as a stop with no
// page and no artifact; the owner's rules are "no matter what, we must ship"
// and that a page-agreement check ships with the defect reported. So this is a
// check of `publishFixedPage`, which collects its refusal as the
// `carried-evidence` defect and ships the page.
//
// IT STILL READS THE PAGE WITH NO ORIGINAL IN IT, and the footnote check
// beside it no longer does (B1182). A region found only in the original an
// untranslated slice ships under its note would pass for evidence the lanes
// wrote, and the carried passage is exactly one the lanes admitted as already
// rendered in English; the original is no wording of the pipeline.

/**
 Failure raised when the page lost a region that proved a carried passage rendered.

 THE MESSAGE NAMES THE ENTRY, A COUNT AND SLICE NUMBERS ONLY, as a published defect's does; the regions themselves
 are English page text and stay on {@link CarriedEvidenceLostError.findings}, which the publisher writes to the run
 log.

 @example
 ```ts
 throw new CarriedEvidenceLostError({ entryId: 'BookshopCat', findings: ['carried-insertion-evidence-lost (count 1)',], sliceIndices: [3,], });
 ```
 */
export class CarriedEvidenceLostError extends Error {
  /**
   Declares message safe to forward because it names the entry, a count and slice numbers only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose page lost the evidence.
   */
  readonly entryId: string;

  /**
   The count line, then one line per passage naming its slice and the regions it lost.
   */
  readonly findings: readonly string[];

  /**
   @param entryId - entry whose page lost the evidence

   @param findings - count line, then one line per passage

   @param sliceIndices - slices of the passages that lost evidence, in the order admission recorded them
   */
  public constructor(
    {
      entryId,
      findings,
      sliceIndices,
    }: {
      readonly entryId: string;
      readonly findings: readonly string[];
      readonly sliceIndices: readonly number[];
    },
  ) {
    super(
      `entry ${entryId}: carried-insertion-evidence-lost (count ${String(sliceIndices.length,)}) in slices [${
        sliceIndices.join(', ',)
      }]`,
    );
    this.name = 'CarriedEvidenceLostError';
    this.entryId = entryId;
    this.findings = findings;
  }
}

/**
 Text as the anchoring reads it: soft breaks folded, punctuation normalized.

 @param text - page or region

 @returns Folded reading

 @example
 ```ts
 asAnchored({ text: 'Zhenli,\nSansan', },);
 // => 'Zhenli, Sansan'
 ```
 */
function asAnchored({ text, }: { readonly text: string; },): string {
  return collapseSoftLineBreaks({ text: normalizePunctuation({ text, },), },);
}

/**
 Verifies final would-ship page retains exact regions proving carried passages.

 @param entryId - entry whose page is weighed

 @param artifact - final stage decisions used for publication

 @param slices - preparation defining replacement spans

 @param targetText - archive text replacement spans address

 @param carried - source-only passages proven rendered elsewhere before lanes

 @throws {@link CarriedEvidenceLostError} when final stages remove any region supporting carried-complete decision,
 which `publishFixedPage` ships as a reported defect

 @example
 ```ts
 assertCarriedInsertionsRemain({ entryId: 'BookshopCat', artifact, slices, targetText, carried: [], });
 ```
 */
export function assertCarriedInsertionsRemain(
  {
    entryId,
    artifact,
    slices,
    targetText,
    carried,
  }: {
    readonly entryId: string;
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly targetText: string;
    readonly carried: readonly CarriedInsertion[];
  },
): void {
  if (carried.length === 0)
    return;
  /**
   Exact page final stage decisions would publish.
   */
  const finalText = spliceSlices({
    targetText,
    slices,
    replacements: shippableReplacements({
      artifact,
      includeSource: false,
    },),
  },);
  /**
   The page as the anchoring reads it.
   */
  const foldedPage = asAnchored({ text: finalText, },);
  /**
   Carried passages whose evidence the page no longer carries, each with the
   regions it lost; a passage admitted with no region at all is lost too.
   */
  const lost = carried
    .map(function toLoss(candidate,): {
      readonly sliceIndex: number;
      readonly missing: readonly string[];
    } {
      /**
       Count of regions admission recorded.
       */
      const recorded = candidate
        .evidence
        .length;
      /**
       Regions the folded page does not carry.
       */
      const missing = candidate
        .evidence
        .filter(function regionMissing(region,): boolean {
          /**
           The region as the anchoring read it.
           */
          const folded = asAnchored({ text: region, },);
          return !foldedPage.includes(folded,);
        },);
      return {
        sliceIndex: candidate.sliceIndex,
        missing: (recorded === 0) ? [ '(no region recorded)', ] : missing,
      };
    },)
    .filter(function lostAnything(loss,): boolean {
      /**
       Count of regions this passage lost.
       */
      const lostCount = loss
        .missing
        .length;
      return lostCount > 0;
    },);
  if (lost.length === 0)
    return;
  throw new CarriedEvidenceLostError({
    entryId,
    sliceIndices: lost.map(function sliceOf(loss,): number {
      return loss.sliceIndex;
    },),
    findings: [
      `carried-insertion-evidence-lost (count ${String(lost.length,)})`,
      ...lost.map(function spell(loss,): string {
        return `carried-insertion-evidence-lost slice ${String(loss.sliceIndex,)}: ${
          loss.missing
            .map(function quote(region,): string {
              return JSON.stringify(region,);
            },)
            .join(', ',)
        }`;
      },),
    ],
  },);
}

//endregion Carried insertion completeness
