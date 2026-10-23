import { introducedFootnoteFindings, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { shippableReplacements, } from './publish-fixed.ts';
import type { WouldShipSource, } from './would-ship-text.ts';

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

/**
 Refuses a would-ship page that introduces a footnote defect the archive
 did not carry.

 @param artifact - final stage decisions used for publication

 @param slices - preparation defining replacement spans

 @param targetText - archive text the replacement spans address

 @throws {@link TranslationRepairInterruptedError} as `page-footnote-integrity`
 naming each introduced defect

 @example
 ```ts
 assertPageFootnotesIntact({ artifact, slices, targetText, },);
 ```
 */
export function assertPageFootnotesIntact(
  {
    artifact,
    slices,
    targetText,
  }: {
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly targetText: string;
  },
): void {
  /**
   The page as it would ship.
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
   Footnote defects the page carries and the archive did not.
   */
  const introduced = introducedFootnoteFindings({
    incumbentText: targetText,
    assembledText: finalText,
  },);
  if (introduced.length === 0)
    return;
  throw new TranslationRepairInterruptedError({
    reason: 'page-footnote-integrity',
    findings: [
      `page-footnote-integrity (count ${String(introduced.length,)})`,
      ...introduced.map(function spell(finding,): string {
        return `page-footnote-${finding.kind} ${finding.convention} ${finding.identifier}`;
      },),
    ],
  },);
}

//endregion Page footnote integrity
