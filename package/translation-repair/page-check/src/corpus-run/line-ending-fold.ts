import { foldCarriageReturns, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Line ending fold
// A MODEL'S CARRIAGE RETURNS NEVER REACH AN LF PAGE (ledger A3). The corpus
// read folds every page to LF (`line-endings.ts`), but a lane wording is a
// model's text and arrived with Windows line endings on one page: three lines
// of mikaela17 shipped ending in a carriage return. This pass folds every
// replacement first, before any other page pass reads it, and records each
// slice it changed as a row the page carries.

/**
 One replacement beside its folded text.
 */
type FoldedReplacement = {
  /**
   Replacement as the lane wrote it.
   */
  readonly replacement: SliceReplacement;

  /**
   Its text with Windows line endings folded, and how many were.
   */
  readonly folded: {
    readonly text: string;
    readonly folded: number;
  };
};

/**
 Replacement as the page carries it once folded.

 @param entry - replacement beside its folded text

 @returns The replacement unchanged when nothing folded, else with the folded text

 @example
 ```ts
 const row = foldedRow({ entry, },);
 ```
 */
function foldedRow({ entry, }: { readonly entry: FoldedReplacement; },): SliceReplacement {
  /**
   Replacement and its fold.
   */
  const {
    replacement,
    folded,
  } = entry;
  return (folded.folded === 0)
    ? replacement
    : {
      ...replacement,
      replacementText: folded.text,
    };
}

/**
 Every replacement with its Windows line endings folded to LF.

 @param replacements - replacements the page would write

 @returns Folded replacements, the rows whose text changed, and one finding
 per changed slice

 @example
 ```ts
 const { replacements, restored, findings, } = foldReplacementLineEndings({ replacements, },);
 ```
 */
export function foldReplacementLineEndings(
  { replacements, }: { readonly replacements: readonly SliceReplacement[]; },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Each replacement beside its folded text.
   */
  const folds: readonly FoldedReplacement[] = replacements.map(function fold(replacement,): FoldedReplacement {
    return {
      replacement,
      folded: foldCarriageReturns({ text: replacement.replacementText, },),
    };
  },);
  /**
   Replacements whose text carried at least one Windows line ending.
   */
  const changed = folds.filter(function carriedEnding({ folded, }: FoldedReplacement,): boolean {
    return folded.folded > 0;
  },);
  return {
    replacements: folds.map(function toReplacement(entry: FoldedReplacement,): SliceReplacement {
      return foldedRow({ entry, },);
    },),
    restored: changed.map(function toRestored(entry: FoldedReplacement,): SliceReplacement {
      return foldedRow({ entry, },);
    },),
    findings: changed.map(function toFinding(
      {
        replacement,
        folded,
      }: FoldedReplacement,
    ): string {
      return `line-endings: slice ${String(replacement.sliceIndex,)} folded ${String(folded.folded,)} `
        + 'Windows line ending(s) a lane wording carried into the LF page';
    },),
  };
}

//endregion Line ending fold
