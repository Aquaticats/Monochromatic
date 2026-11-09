/**
 Version 4 UUID layout predicate shared by the unit suites.

 More than one suite classifies a drawn UUID, so the layout lives here: a
 correction lands in one place, and the suites cannot disagree about what a
 valid UUID is.

 @module
 */

//region Constants

/**
 Characters a hexadecimal position may hold.
 */
const HEX_CHARACTERS = '0123456789abcdef';

/**
 Length of a canonical UUID string: 32 hexadecimal characters and 4 dashes.
 */
const UUID_LENGTH = 36;

/**
 Positions carrying a dash in a canonical UUID.
 */
const UUID_DASH_POSITIONS: ReadonlySet<number> = new Set([
  8,
  13,
  18,
  23,
],);

/**
 Position of the version nibble.
 */
const UUID_VERSION_POSITION = 14;

/**
 Version nibble a version 4 UUID carries.
 */
const UUID_VERSION = '4';

/**
 Position of the variant nibble.
 */
const UUID_VARIANT_POSITION = 19;

/**
 Variant nibbles RFC 9562 allows for a version 4 UUID.
 */
const UUID_VARIANT_CHARACTERS = '89ab';

//endregion Constants

//region Predicate

/**
 Whether `text` is a lowercase canonical version 4 UUID.

 Checked by index scan rather than pattern match: the shape is fully expressible
 as character positions, and the repository bans regex literals without a scoped
 justification.

 @param text - value to classify

 @returns whether every position matches the version 4 UUID layout

 @example
 ```ts
 isUuidV4('3f2504e0-4f89-41d3-9a0c-0305e82c3301'); // true
 ```
 */
export function isUuidV4(text: string,): boolean {
  if (text.length !== UUID_LENGTH) {
    return false;
  }
  /**
   Candidate characters, one entry per UTF-16 code unit so a position here
   means the same thing it does in the length check above.
   */
  const characters = text.split('',);
  for (const [index, character,] of characters.entries()) {
    if (UUID_DASH_POSITIONS.has(index,)) {
      if (character !== '-') {
        return false;
      }
      continue;
    }
    if (!HEX_CHARACTERS.includes(character,)) {
      return false;
    }
  }
  return (text.charAt(UUID_VERSION_POSITION,) === UUID_VERSION)
    && UUID_VARIANT_CHARACTERS.includes(text.charAt(UUID_VARIANT_POSITION,),);
}

//endregion Predicate
