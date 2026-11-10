/**
 Shape checks for plan values that must match a fixed grammar.

 Both are index scans over short strings,
 so no regular expression runs over anything a caller could grow.

 @module
 */

/**
 Characters a repository slug's parts may hold,
 the same set GitHub allows in an owner or repository name.
 */
const SLUG_CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._-';

/**
 Characters an abbreviated or complete commit id may hold.
 */
const COMMIT_CHARACTERS = '0123456789abcdef';

/**
 Shortest commit id Git prints in its own messages.
 */
const COMMIT_SHORTEST = 7;

/**
 Longest commit id, which is a full SHA-1.
 */
const COMMIT_LONGEST = 40;

/**
 Separator between the owner and the repository name.
 */
const SLUG_SEPARATOR = '/';

/**
 Answers whether every character of a value is one of the allowed characters.
 
 @param text - Value to scan.
 
 @param allowed - Characters the value may hold.
 
 @returns True when no character falls outside the allowed set.
 
 @example
 ```ts
 const ok = onlyAllowedCharacters({ text: 'abc', allowed: 'abcdef' });
 ```
 */
function onlyAllowedCharacters({
  text,
  allowed,
}: {
  readonly text: string;
  readonly allowed: string;
},): boolean {
  for (const character of text) {
    if (!allowed.includes(character,)) {
      return false;
    }
  }
  return true;
}

/**
 Answers whether a value is an OWNER/NAME repository slug.
 
 @param value - Value to check.
 
 @returns True for one separator with a non-empty allowed part on each side.
 
 @example
 ```ts
 const ok = looksLikeRepository('Aquaticats/Monochromatic');
 ```
 */
export function looksLikeRepository(value: string,): boolean {
  /**
   The slug's parts, of which a repository has exactly two.
   */
  const parts = value.split(SLUG_SEPARATOR,);
  if (parts.length !== 2) {
    return false;
  }
  return parts.every(function isSlugPart(part,): boolean {
    return (part !== '') && onlyAllowedCharacters({
      text: part,
      allowed: SLUG_CHARACTERS,
    },);
  },);
}

/**
 Answers whether a value is a lowercase hexadecimal commit id.
 
 @param value - Value to check.
 
 @returns True for the allowed length of only hexadecimal characters.
 
 @example
 ```ts
 const ok = looksLikeCommit('2b309f8f25');
 ```
 */
export function looksLikeCommit(value: string,): boolean {
  if ((value.length < COMMIT_SHORTEST) || (value.length > COMMIT_LONGEST)) {
    return false;
  }
  return onlyAllowedCharacters({
    text: value,
    allowed: COMMIT_CHARACTERS,
  },);
}
