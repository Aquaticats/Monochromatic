/**
 Version 4 UUID layout predicate shared by the unit suites.

 More than one suite classifies a drawn UUID, so the layout lives here: a
 correction lands in one place, and the suites cannot disagree about what a
 valid UUID is.

 @module
 */

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
  if (text.length !== 36) {
    return false;
  }
  /**
   Characters a hexadecimal position may hold.
   */
  const hex = '0123456789abcdef';
  for (let index = 0; index < 36; index += 1) {
    /**
     Character at this position.
     */
    const character = text.charAt(index,);
    if ((index === 8) || (index === 13)
      || (index === 18)
      || (index === 23)) {
      if (character !== '-') {
        return false;
      }
      continue;
    }
    if (!hex.includes(character,)) {
      return false;
    }
  }
  // Position 14 is the version nibble and position 19 the variant nibble.
  return (text.charAt(14,) === '4') && '89ab'.includes(text.charAt(19,),);
}
