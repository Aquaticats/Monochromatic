/**
 npm package-name validation used by the workspace catalog parser.
 
 The validator is intentionally a string scan rather than a regular
 expression, matching this repository's no-regex rule for simple grammars.
 
 @module
 */

//region Package-name grammar

/**
 Reports whether `character` is an ASCII lowercase letter or digit.
 
 @param character - character to inspect
 
 @returns whether the character may begin a package-name segment
 
 @example
 ```ts
 isAlphanumeric('a'); // true
 isAlphanumeric('_'); // false
 ```
 */
function isAlphanumeric(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= '0') && (character <= '9'));
}

/**
 Reports whether `character` is allowed after the first segment character.
 
 @param character - character to inspect
 
 @returns whether the character is allowed inside a package-name segment
 
 @example
 ```ts
 isSegmentCharacter('-'); // true
 isSegmentCharacter('/'); // false
 ```
 */
function isSegmentCharacter(character: string,): boolean {
  return isAlphanumeric(character,)
    || (character === '-')
    || (character === '.')
    || (character === '_');
}

/**
 Reports whether `segment` is a valid non-empty npm name segment.
 
 @param segment - unscoped name or scope segment
 
 @returns whether the segment begins with lowercase ASCII alphanumeric text and contains only allowed characters
 
 @example
 ```ts
 isNameSegment('oxlint'); // true
 isNameSegment('__proto__'); // false
 ```
 */
function isNameSegment(segment: string,): boolean {
  if (segment.length
    === 0)
    return false;
  if (!isAlphanumeric(segment.charAt(0,),))
    return false;
  for (const character of segment) {
    if (!isSegmentCharacter(character,))
      return false;
  }
  return true;
}

/**
 Validates an npm package name, allowing one optional `@scope/` prefix.
 
 @param name - package name to validate
 
 @returns whether `name` matches the accepted npm package-name shape
 
 @example
 ```ts
 isValidPackageName('\@types/node'); // true
 isValidPackageName('oxlint'); // true
 isValidPackageName('__proto__'); // false
 ```
 */
export function isValidPackageName(name: string,): boolean {
  /**
   Whether `name` has a scoped-package prefix.
   */
  const scoped = name.startsWith('@',);
  /**
   Package-name body after removing the optional scope marker.
   */
  const body = scoped ? name.slice(1,) : name;
  /**
   Position of the only permitted scope separator.
   */
  const slashIndex = body.indexOf('/',);

  if (scoped) {
    if (slashIndex <= 0)
      return false;
    return isNameSegment(body.slice(
      0,
      slashIndex,
    ),)
      && isNameSegment(body.slice(slashIndex + 1,),);
  }

  if (slashIndex !== (-1))
    return false;
  return isNameSegment(body,);
}

//endregion Package-name grammar
