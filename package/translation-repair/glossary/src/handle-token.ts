import {
  codePointAt,
  codePointBefore,
  isAsciiAlphanumeric,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Handle token
// AN ACCOUNT HANDLE OR A LATIN NAME READ AS A WHOLE TOKEN (ledger B23), for
// the link-name floor (`translate-declared-link-name.ts`) and the page-name
// glossary's note (`linked-title-declared-name.ts`), which must read one link
// text alike. A hyphen and an underscore join a handle (`mi-mi-42`,
// `Cang_Shan_Jing_Ye`), so these edges are handle edges, not the word edges
// of `word-bounds.ts`: `Tomcat` and `@Tom_Cat` name no `Tom`, and
// `@mi-mi-420` does not carry `@mi-mi-42`.

/**
 Characters an account handle is written with besides letters and digits.
 */
const HANDLE_PUNCTUATION: ReadonlySet<string> = new Set([
  '-',
  '_',
],);

/**
 Whether a character continues an account handle: an ASCII letter or digit,
 a hyphen or an underscore.

 @param character - one character, empty past a text's edge

 @returns Whether a handle may carry it

 @example
 ```ts
 isHandleCharacter({ character: '_', },); // true
 ```
 */
export function isHandleCharacter({ character, }: { readonly character: string; },): boolean {
  return isAsciiAlphanumeric({ character, },) || HANDLE_PUNCTUATION.has(character,);
}

/**
 Whether a needle found at an offset stands as a whole handle token: where
 its first character could continue a handle the one before it does not, and
 where its last could the one after it does not. A Han or `@` edge needs no
 boundary.

 @param text - text the needle was found in

 @param at - offset it was found at

 @param needle - source form or handle

 @returns Whether no handle runs on from either end

 @example
 ```ts
 standsAsHandle({ text: 'Tomcat', at: 0, needle: 'Tom', },); // false
 ```
 */
export function standsAsHandle(
  {
    text,
    at,
    needle,
  }: {
    readonly text: string;
    readonly at: number;
    readonly needle: string;
  },
): boolean {
  /**
   Whether the needle opens on a character a handle is written with.
   */
  const opensHandle = isHandleCharacter({
    character: codePointAt({
      text: needle,
      at: 0,
    },),
  },);

  /**
   Whether it closes on one.
   */
  const closesHandle = isHandleCharacter({
    character: codePointBefore({
      text: needle,
      at: needle.length,
    },),
  },);
  if (opensHandle && isHandleCharacter({
    character: codePointBefore({
      text,
      at,
    },),
  },))
    return false;
  return !(closesHandle && isHandleCharacter({
    character: codePointAt({
      text,
      at: at + needle.length,
    },),
  },));
}

/**
 Whether a text carries a needle as a whole handle token somewhere.

 @param text - link text

 @param needle - source form or handle

 @returns Whether some occurrence stands as a whole token

 @example
 ```ts
 carriesHandleToken({ text: '@mi-mi-420', needle: '@mi-mi-42', },); // false
 ```
 */
export function carriesHandleToken(
  {
    text,
    needle,
  }: {
    readonly text: string;
    readonly needle: string;
  },
): boolean {
  if (needle === '')
    return false;
  for (let at = text.indexOf(needle,); at !== (-1); at = text.indexOf(
    needle,
    at + 1,
  )) {
    if (standsAsHandle({
      text,
      at,
      needle,
    },))
      return true;
  }
  return false;
}

//endregion Handle token
