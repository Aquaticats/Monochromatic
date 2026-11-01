import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { onHeadingLine, } from './archive-casing-restore.ts';
import {
  midSentence,
  phraseOccurrences,
  titleRuns,
} from './archive-name-runs.ts';
import {
  type SliceReplacement,
  slicesInOrder,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';

import { applySpanRewrites, } from './span-rewrites.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Archive name casing
// CLASS ONE HUNDRED THIRTY-SIX (hulicaijia20, 2026-09-25): the archive names
// a street "Jiefangbei Pedestrian Street" mid-sentence in the body and again
// in a picture caption, never in any other casing, and the bench wrote
// "Jiefangbei pedestrian street" two lines later, so the page carried one
// place under two casings. A run of two or more title-case words the archive
// writes mid-sentence at least twice and never in another casing is the
// page's form of a name; where a shipped slice's prose writes the run's
// first word as the archive does and a later word in another case, the
// archive's form is restored here, where the page is in view. A run that
// only ever opens a sentence is read past, since a sentence's first capital
// says nothing about a name; headings are read past on both sides, as the
// class one hundred twenty-one pass does.

/**
 Fewest mid-sentence uses of a run before it reads as the page's form.
 */
const MIN_USES = 2;

/**
 One rewrite of a name to the archive's form.
 */
type NameRewrite = {
  readonly start: number;
  readonly end: number;
  readonly from: string;
  readonly to: string;
};

/**
 The archive's multi-word title-case names: each written mid-sentence in
 prose at least twice and never in another casing.

 @param archiveText - every archive slice joined

 @returns The names' forms

 @example
 ```ts
 nameForms({ archiveText: 'on Maowu Street, near Maowu Street', },); // Set { 'Maowu Street' }
 ```
 */
function nameForms(
  { archiveText, }: { readonly archiveText: string; },
): ReadonlySet<string> {
  /**
   The archive's non-prose ranges.
   */
  const ranges = protectedRanges({ text: archiveText, },);
  /**
   Mid-sentence prose uses of each run.
   */
  const uses = titleRuns({ text: archiveText, },)
    .filter(function counted({
      phrase,
      start,
    },): boolean {
      return midSentence({
        text: archiveText,
        at: start,
      },)
        && (!onHeadingLine({
          text: archiveText,
          at: start,
        },))
        && inProse({
          ranges,
          start,
          end: start + phrase.length,
        },);
    },)
    .reduce(
    function tally(
      byPhrase,
      { phrase, },
    ): Map<string, number> {
      byPhrase.set(
        phrase,
        (byPhrase.get(phrase,) ?? 0) + 1,
      );
      return byPhrase;
    },
    new Map<string, number>(),
    );
  return new Set([...uses,]
    .filter(function oneCasing([
      phrase,
      count,
    ]: ReadonlyEntry<string, number>,): boolean {
      return (count >= MIN_USES)
        && phraseOccurrences({
          text: archiveText,
          phrase,
        },)
          .every(function asWritten(at,): boolean {
            return archiveText.slice(
              at,
              at + phrase.length,
            ) === phrase;
          },);
    },)
    .map(function phraseOf([phrase,]: ReadonlyEntry<string, number>,): string {
      return phrase;
    },),);
}

/**
 Rewrites every prose spelling of an archive name that keeps its first word
 and changes the case of a later one.

 @param text - shipped text

 @param forms - the archive's name forms

 @returns Rewritten text and each rewrite

 @example
 ```ts
 recaseNames({ text: 'on Maowu street', forms: new Set(['Maowu Street']), },);
 ```
 */
function recaseNames(
  {
    text,
    forms,
  }: {
    readonly text: string;
    readonly forms: ReadonlySet<string>;
  },
): {
  readonly text: string;
  readonly rewrites: readonly NameRewrite[];
} {
  /**
   Every rewrite any form asks for; the applier orders them, the longer first
   where two start together, and withholds any that overlaps one before it.
   */
  const asked = [...forms,]
    .flatMap(function rewritesFor(form,): readonly NameRewrite[] {
      /**
       The form's first word, which the shipped spelling must keep.
       */
      const firstWord = form.slice(
        0,
        form.indexOf(' ',),
      );
      return phraseOccurrences({
        text,
        phrase: form,
      },)
        .map(function toRewrite(at,): NameRewrite {
          return {
            start: at,
            end: at + form.length,
            from: text.slice(
              at,
              at + form.length,
            ),
            to: form,
          };
        },)
        .filter(function restorable({ from, },): boolean {
          return (from !== form) && from.startsWith(`${firstWord} `,);
        },);
    },);
  /**
   Text with the rewrites applied, and the rewrites that were.
   */
  const rebuilt = applySpanRewrites({
    text,
    rewrites: asked,
  },);
  return {
    text: rebuilt.text,
    rewrites: rebuilt.applied,
  };
}

/**
 Restores the archive's title-case form of a multi-word name wherever a
 shipped slice keeps its first word and writes a later word in another case.

 @param slices - prepared pairs, whose archive text carries the forms

 @param replacements - what the page would write per slice

 @returns Replacements with the forms restored, the rewritten rows alone,
 and one finding per rewrite

 @example
 ```ts
 const restored = restoreArchiveNameCasing({ slices, replacements, },);
 ```
 */
export function restoreArchiveNameCasing(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   The archive's name forms across the whole page.
   */
  const forms = nameForms({
    archiveText: slicesInOrder({ slices, },)
      .map(function archiveOf(slice,): string {
        return slice.target
          .text;
      },)
      .join('\n',),
  },);
  /**
   Each row with its rewrites.
   */
  const passes = replacements.map(function restore(row,): {
    readonly row: SliceReplacement;
    readonly rewrites: readonly NameRewrite[];
  } {
    if (forms.size === 0) {
      return {
        row,
        rewrites: [],
      };
    }
    /**
     The row's text with the forms restored.
     */
    const recased = recaseNames({
      text: row.replacementText,
      forms,
    },);
    return {
      row: {
        sliceIndex: row.sliceIndex,
        replacementText: recased.text,
      },
      rewrites: recased.rewrites,
    };
  },);
  /**
   Rows this pass changed.
   */
  const changed = passes.filter(function rewrote({ rewrites, },): boolean {
    return rewrites.length > 0;
  },);
  return {
    replacements: passes.map(function rowOf({ row, },): SliceReplacement {
      return row;
    },),
    restored: changed.map(function rowOf({ row, },): SliceReplacement {
      return row;
    },),
    findings: changed.flatMap(function report({
      row,
      rewrites,
    },): readonly string[] {
      return rewrites.map(function finding(rewrite,): string {
        return `archive-name-casing-restored (slice ${String(row.sliceIndex,)}: "${rewrite.from}" to "${rewrite.to}")`;
      },);
    },),
  };
}
//endregion Archive name casing
