import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { CarriedInsertion, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  collapseSoftLineBreaks,
  normalizePunctuation,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
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

 @param artifact - final stage decisions used for publication

 @param slices - preparation defining replacement spans

 @param targetText - archive text replacement spans address

 @param carried - source-only passages proven rendered elsewhere before lanes

 @throws {@link TranslationRepairInterruptedError} when final stages remove
 any region supporting carried-complete decision

 @example
 ```ts
 assertCarriedInsertionsRemain({ artifact, slices, targetText, carried: [], });
 ```
 */
export function assertCarriedInsertionsRemain(
  {
    artifact,
    slices,
    targetText,
    carried,
  }: {
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
  throw new TranslationRepairInterruptedError({
    reason: 'carried-evidence-lost',
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
