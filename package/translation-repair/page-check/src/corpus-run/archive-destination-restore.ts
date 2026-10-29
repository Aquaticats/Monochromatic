import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { linksOf, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  rewriteEverySlice,
  type TextRewrite,
} from './page-slice-rewrite.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Archive destination restore
// LEDGER A4 (owner, 2026-09-27, "Archive's English"). shihai4h2 linked PTSD
// to zh.wikipedia where the archive links en.wikipedia, and aiyysk linked
// source.android.google.cn where the archive links source.android.com: the
// lanes wrote the original's destination back over the English one the
// archive's translator chose, and no judge objected. This page-assembly pass
// writes the archive's destination back.
//
// WHICH DESTINATIONS. A slice whose original and archive carry the same
// number of links pairs them by position. Where the two differ, the
// original's destination appears nowhere on the archive's page and the
// archive's nowhere in the original, the archive replaced it on purpose, and
// the page keeps the archive's. Measured at pin a41fc607 over 92 entries:
// six such pairs, the three localizations the owner ruled on and three
// replacements of the same kind (a `www.` host, twitter.com to x.com, a moved
// path on one host). A swap of two destinations both sides carry is no
// replacement and is left alone, and so is an original destination the
// archive replaced two different ways.

/**
 Whether a text links to a destination: the destination closing a Markdown
 link, alone or before a link title.

 @param text - text read

 @param href - destination

 @returns Whether some link points there

 @example
 ```ts
 linksTo({ text: '[PTSD](https://en.example/ptsd)', href: 'https://en.example/ptsd', },); // true
 ```
 */
function linksTo(
  {
    text,
    href,
  }: {
    readonly text: string;
    readonly href: string;
  },
): boolean {
  return text.includes(`](${href})`,)
    || text.includes(`](${href} `,);
}

/**
 Destinations the archive replaced, original to archive, read across the
 page.

 @param slices - prepared pairs

 @returns Original destination to the archive's, only where the archive
 replaced it one way

 @example
 ```ts
 const map = replacedDestinations({ slices, },);
 ```
 */
export function replacedDestinations(
  { slices, }: { readonly slices: readonly ChunkPair[]; },
): ReadonlyMap<string, string> {
  /**
   Original as the slices carry it.
   */
  const sourceText = slices
    .map(function sourceOf(slice,): string {
      return slice.source
        .text;
    },)
    .join('\n',);
  /**
   Archive as the slices carry it.
   */
  const archiveText = slices
    .map(function archiveOf(slice,): string {
      return slice.target
        .text;
    },)
    .join('\n',);
  /**
   Every replacement read, original destination first.
   */
  const pairs = slices.flatMap(function pairsOf(slice,): readonly (readonly [
    string,
    string,
  ])[] {
    /**
     Links of the original slice.
     */
    const original = linksOf({ text: slice.source
      .text, },);
    /**
     Links of the archive slice.
     */
    const archived = linksOf({ text: slice.target
      .text, },);
    if (original.length !== archived.length)
      return [];
    return original.flatMap(function replaced(
      link,
      index,
    ): readonly (readonly [
      string,
      string,
    ])[] {
      /**
       Archive's link at the same position.
       */
      const counterpart = archived[index];
      if ((counterpart === undefined) || (counterpart.href === link.href))
        return [];
      /**
       Whether either destination stands on the other side too, which makes
       the difference a reordering rather than a replacement.
       */
      const crossed = linksTo({
        text: archiveText,
        href: link.href,
      },)
        || linksTo({
          text: sourceText,
          href: counterpart.href,
        },);
      return crossed
        ? []
        : [
          [
            link.href,
            counterpart.href,
          ],
        ];
    },);
  },);
  /**
   Archive destinations each original destination was replaced by.
   */
  const targets = pairs.reduce(
    function collect(
      read,
      [from, to,],
    ): Map<string, ReadonlySet<string>> {
      read.set(
        from,
        new Set([
          ...(read.get(from,) ?? []),
          to,
        ],),
      );
      return read;
    },
    new Map<string, ReadonlySet<string>>(),
  );
  return new Map([...targets.entries(),]
    .filter(function oneWay([_from, tos,]: ReadonlyEntry<string, ReadonlySet<string>>,): boolean {
      return tos.size === 1;
    },)
    .map(function first([from, tos,]: ReadonlyEntry<string, ReadonlySet<string>>,): readonly [
      string,
      string,
    ] {
      return [
        from,
        nonNullishOrThrow([...tos,][0],),
      ];
    },),);
}

/**
 One slice's text with every replaced destination pointed back at the
 archive's.

 @param text - slice text

 @param destinations - original destination to the archive's

 @returns Text and one note per destination rewritten

 @example
 ```ts
 const rewritten = redirected({ text, destinations, },);
 ```
 */
function redirected(
  {
    text,
    destinations,
  }: {
    readonly text: string;
    readonly destinations: ReadonlyMap<string, string>;
  },
): TextRewrite {
  /**
   Replacements this text carries a link for.
   */
  const carried = [...destinations.entries(),]
    .filter(function present([from,]: ReadonlyEntry<string, string>,): boolean {
      return linksTo({
        text,
        href: from,
      },);
    },);
  return {
    text: carried.reduce(
      function pointBack(
        current,
        [from, to,]: ReadonlyEntry<string, string>,
      ): string {
        return current
          .replaceAll(
          `](${from})`,
          `](${to})`,
          )
          .replaceAll(
          `](${from} `,
          `](${to} `,
          );
      },
      text,
    ),
    changed: carried.map(function note([from, to,]: ReadonlyEntry<string, string>,): string {
      return `${from} to ${to}`;
    },),
  };
}

/**
 Points every link the archive gave its own destination back at it, the
 slices no lane replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements with the archive's destinations, the rewritten rows
 alone, and one finding per slice changed

 @example
 ```ts
 const page = restoreArchiveDestinations({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function restoreArchiveDestinations(
  {
    slices,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Destinations the archive replaced.
   */
  const destinations = replacedDestinations({ slices, },);
  if (destinations.size === 0) {
    return {
      replacements,
      restored: [],
      findings: [],
    };
  }
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: function rewrite({ text, },): TextRewrite {
      return redirected({
        text,
        destinations,
      },);
    },
    findingName: 'archive-destination-restored',
  },);
}

//endregion Archive destination restore
