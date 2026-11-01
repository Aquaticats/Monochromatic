import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { pageTextOf, } from './assembly-page-text.ts';
import {
  type Signature,
  signaturesOf,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import { carriesHan, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  handleReading,
  withoutGloss,
} from './handle-reading.ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Contributor name authorities
// WHICH RENDERING A SIGNER TAKES, decided once per page for the contributor
// name restore (`contributor-name-restore.ts`, class sixty-seven): the
// archive's aligned signature where the archive carries it, else the page's
// own aligned signature, else (class eighty-three, XingZ622, 2026-09-22) the
// handle's pinyin reading, since a page rendering that still carries Han is
// no rendering. Every signer takes an authority, headed or not: 雨狸, whom no
// heading names, shipped as "Yu Li" on one song credit and 雨狸 on the next.

/**
 Where a name's rendering comes from.
 */
export type Authority = {
  /**
   Rendering the page uses everywhere.
   */
  readonly rendering: string;

  /**
   What rendered it, as the finding names it.
   */
  readonly origin:
    | 'the archive\'s signature rendering'
      | 'the page\'s signature rendering'
      | 'the pinyin reading of the original\'s handle';
};

/**
 Signature at one position of a text, when that text signs as often as the
 original does; the archive or the page may split or merge a signature line.

 @param text - archive or page text of the slice

 @param at - position among the original's signatures

 @param count - how many signatures the original writes in the slice

 @returns Signatures aligned by position, empty when the counts differ

 @example
 ```ts
 const aligned = alignedSignature({ text, at: 0, count: 1, },);
 ```
 */
function alignedSignature(
  {
    text,
    at,
    count,
  }: {
    readonly text: string;
    readonly at: number;
    readonly count: number;
  },
): readonly Signature[] {
  /**
   Signatures the text writes.
   */
  const signatures = signaturesOf({ text, },);
  if (signatures.length !== count)
    return [];
  return signatures.slice(
    at,
    at + 1,
  );
}

/**
 Authority per name the original signs: the archive's aligned signature,
 else the page's own aligned signature where it renders the name in
 something a reader sees other than Han, else the handle's pinyin reading
 (class eighty-three).

 @param slices - prepared pairs in slice order

 @param pageText - page text per slice

 @returns Rendering and its origin per original name

 @example
 ```ts
 const authorities = nameAuthorities({ slices, pageText, },);
 ```
 */
export function nameAuthorities(
  {
    slices,
    pageText,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly pageText: ReadonlyMap<number, string>;
  },
): ReadonlyMap<string, Authority> {
  /**
   Authority per name, the first signature's.
   */
  const authorities = new Map<string, Authority>();
  for (const slice of slices) {
    /**
     Signatures the original writes in this slice.
     */
    const signed = signaturesOf({ text: slice.source
      .text, },);
    signed.forEach(function settle(
      signature,
      at,
    ): void {
      /**
       Name as the original signs it.
       */
      const { name, } = signature;
      if (authorities.has(name,))
        return;
      /**
       Archive's rendering at the same position.
       */
      const [archive,] = alignedSignature({
        text: slice.target
          .text,
        at,
        count: signed.length,
      },);
      // WHAT SHOWS A READER NOTHING RENDERS NO ONE here either: the archive's
      // signature is parsed text and so was left (ledger B194), but a name of
      // one zero-width space written there would be restored over the heading
      // naming the signer.
      if ((archive !== undefined) && (!rendersAsNothing({ text: archive.name, },))) {
        authorities.set(
          name,
          {
            rendering: archive.name,
            origin: 'the archive\'s signature rendering',
          },
        );
        return;
      }
      /**
       Page's rendering at the same position, without a gloss it carries.
       */
      const [page,] = alignedSignature({
        text: pageTextOf({
          pageText,
          sliceIndex: slice.target
            .sliceIndex,
        },),
        at,
        count: signed.length,
      },);
      /**
       Page's rendering as the authority, empty where the page has none at
       this position.
       */
      const rendered = (page === undefined) ? '' : withoutGloss({ rendering: page.name, },);
      /**
       Whether the page's rendering stands as the authority: it shows a
       reader something, and none of it is Han.

       WHAT SHOWS A READER NOTHING RENDERS NO ONE (ledger B40). The page text
       is what a lane wrote, and asked of an empty-string comparison a
       signature of one zero-width space or Hangul filler became the signer's
       rendering, which the restore then wrote over the heading naming them.
       */
      const pageRenders = (!rendersAsNothing({ text: rendered, },)) && (!carriesHan({ text: rendered, },));
      if (pageRenders) {
        authorities.set(
          name,
          {
            rendering: rendered,
            origin: 'the page\'s signature rendering',
          },
        );
        return;
      }
      authorities.set(
        name,
        {
          rendering: handleReading({ name, },),
          origin: 'the pinyin reading of the original\'s handle',
        },
      );
    },);
  }
  return authorities;
}

//endregion Contributor name authorities
