import { pinyin, } from 'pinyin-pro';

import {
  continuesLatinWord,
  isHanCharacter,
  isLatinWordCharacter,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Handle reading
// CLASS EIGHTY-THREE (XingZ622, 2026-09-22). A signer the archive never
// rendered shipped in Han: 锦心 in the Part Ten heading and signature, where
// XingZ619 wrote "Jinxin", XingZ620 "Jin Xin" and XingZ616 "Brocade Heart";
// 雨狸 as "Yu Li" on one song attribution and 雨狸 on the next. The house
// rules had no sentence on a handle with no declared, archive or corpus
// form, and the judges filled the gap with "keep the original form".
// Owner, 2026-09-22: pinyin in capitalised syllable groups (never one joined
// word), with the literal translation in parentheses. The rule reaches every
// sheet through the
// house policy; this module is the deterministic half the page assembly
// applies where a rendering still carries Han: the reading of the handle,
// and the tolerance for a rendering that carries its gloss.

/**
 Syllables one capitalised group holds. Owner, 2026-09-22: syllable groups,
 never one joined word ("Jiecheng Tianzou" or "Jie Cheng Tian Zou", not
 "Jiechengtianzou"); pairs, since the corpus's own romanisations pair them
 ("Baimao suki").
 */
const GROUP = 2;

/**
 Run start marking that no Han run is open.
 */
const NO_RUN = -1;

/**
 Capitalises a word.

 @param word - lower-case syllables joined

 @returns Word with its first letter upper-case

 @example
 ```ts
 capitalised({ word: 'jinxin', },); // 'Jinxin'
 ```
 */
function capitalised({ word, }: { readonly word: string; },): string {
  return `${word.slice(
    0,
    1,
  )
    .toUpperCase()}${word.slice(1,)}`;
}

/**
 Pinyin of one run of Han in capitalised groups of two syllables, a lone
 trailing syllable standing by itself.

 @param run - consecutive Han characters

 @returns Their toneless syllables paired, capitalised and spaced

 @example
 ```ts
 romanisedRun({ run: '锦心', },); // 'Jinxin'
 ```
 */
function romanisedRun({ run, }: { readonly run: string; },): string {
  /**
   Toneless syllables, one per character.
   */
  const syllables = pinyin(
    run,
    {
      toneType: 'none',
      type: 'array',
    },
  );
  /**
   Capitalised groups in order.
   */
  const groups: string[] = [];
  for (let at = 0; at < syllables.length; at += GROUP) {
    groups.push(capitalised({ word: syllables.slice(
      at,
      at + GROUP,
    )
      .join('',), },),);
  }
  return groups.join(' ',);
}

/**
 Reads a handle the archive never rendered: every run of Han becomes pinyin
 in capitalised groups of two syllables, everything else stays as written,
 and a romanised run is kept a space apart from Latin letters or digits the
 original wrote against it (洁澄天奏Official reads "Jiecheng Tianzou Official").

 @param name - handle as the original signs it

 @returns Handle as the page renders it

 @example
 ```ts
 handleReading({ name: '锦心', },); // 'Jinxin'
 ```
 */
export function handleReading({ name, }: { readonly name: string; },): string {
  /**
   Pieces of the rendering, in order, joined once at the end rather than
   grown a character at a time, per `RG2` (ledger B29).
   */
  const pieces: string[] = [];
  /**
   Where the open Han run starts, `NO_RUN` while none is open.
   */
  let runStart = NO_RUN;
  /**
   Offset of the character being read, in UTF-16 units.
   */
  let at = 0;
  /**
   Whether the character last written out ends a Latin word (a letter,
   accented or not, a digit, or a combining mark on a letter), which a run
   opening right after it is kept apart from; ASCII letters alone left
   `Café猫` as `CaféMao` (ledger B18).
   */
  let afterLatin = false;
  for (const character of name) {
    if (isHanCharacter({ character, },)) {
      if (runStart === NO_RUN) {
        if (afterLatin)
          pieces.push(' ',);
        runStart = at;
      }
      at += character.length;
      continue;
    }
    if (runStart !== NO_RUN) {
      pieces.push(romanisedRun({
        run: name.slice(
          runStart,
          at,
        ),
      },),);
      runStart = NO_RUN;
      if (isLatinWordCharacter({ character, },))
        pieces.push(' ',);
    }
    pieces.push(character,);
    afterLatin = continuesLatinWord({ character, },);
    at += character.length;
  }
  if (runStart !== NO_RUN)
    pieces.push(romanisedRun({ run: name.slice(runStart,), },),);
  /**
   Rendering as the page shows it.
   */
  const rendered = pieces.join('',);
  return rendered;
}

/**
 Rendering without a trailing parenthetical gloss, which the house rule puts
 after a romanised handle at its first appearance.

 @param rendering - rendering as the page wrote it

 @returns Rendering before its gloss, trimmed

 @example
 ```ts
 withoutGloss({ rendering: 'Jinxin (Brocade Heart)', },); // 'Jinxin'
 ```
 */
export function withoutGloss({ rendering, }: { readonly rendering: string; },): string {
  if (!rendering.endsWith(')',))
    return rendering;
  /**
   Where the gloss opens, -1 for none.
   */
  const open = rendering.lastIndexOf(' (',);
  if (open <= 0)
    return rendering;
  return rendering.slice(
    0,
    open,
  )
    .trim();
}

/**
 Whether a written name already carries the rendering, alone or with its
 gloss in parentheses.

 @param written - name as the page wrote it

 @param rendering - rendering the page uses everywhere

 @returns Whether the line needs no rewrite

 @example
 ```ts
 carriesRendering({ written: 'Jinxin (Brocade Heart)', rendering: 'Jinxin', },); // true
 ```
 */
export function carriesRendering(
  {
    written,
    rendering,
  }: {
    readonly written: string;
    readonly rendering: string;
  },
): boolean {
  return (written === rendering) || (withoutGloss({ rendering: written, },) === rendering);
}

//endregion Handle reading
