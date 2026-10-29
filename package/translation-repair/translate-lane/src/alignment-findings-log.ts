import type { Logger, } from '@monochromatic-dev/logger-core/ts';

//region Alignment findings log
// WHAT THE PREPARATION RECORDED ABOUT HOW THE TWO DOCUMENTS LINE UP, written
// where a reader of the run log looks. The findings ride the artifact
// (`preparation.alignmentFindings`) and the `TALLY` line counts them, but the
// structure and section-pairing ones were printed nowhere: the check of
// XingZ6020 found two `structure-mismatch` findings (the original's sections
// 13 and 14, with no translation section to pair with) and a `section-pairing`
// finding only by opening the artifact.

/**
 Prefixes of the findings that say the two documents' sections or structure
 did not line up: the aligner's own findings, which `prepareDocumentPair`
 writes as `alignment <kind> (...)`, and the roster's section-pairing ones.

 THE BLOCK-PAIRING AND STAGE FINDINGS ARE LEFT OUT: the block-pairing rounds
 and the stages that voted on them already write their own log lines as they
 run, so repeating them here would print each twice.
 */
const STRUCTURE_FINDING_PREFIXES: readonly string[] = [
  'alignment ',
  'section-pairing',
];

/**
 Writes the structure and section-pairing alignment findings to the log, one
 warn line each, in the order the preparation recorded them.

 @param findings - preparation's alignment findings, every kind

 @param l - logger the driver writes to

 @example
 ```ts
 logAlignmentFindings({ findings: prepared.alignmentFindings, l, },);
 ```
 */
export function logAlignmentFindings(
  {
    findings,
    l,
  }: {
    readonly findings: readonly string[];
    readonly l: Logger;
  },
): void {
  for (const finding of findings) {
    if (STRUCTURE_FINDING_PREFIXES.some(function startsIt(prefix,): boolean {
      return finding.startsWith(prefix,);
    },))
      l.warn(`alignment finding recorded: ${finding}`,);
  }
}

//endregion Alignment findings log
