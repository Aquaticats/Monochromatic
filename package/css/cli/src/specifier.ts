/**
 CSS @import specifier classification utilities.
 
 Classifies bare specifiers as relative or package references and splits
 package specifiers into name and subpath components. Specifier extraction
 from `\@import` preludes happens token-level in `import.ts`.
 */

//region Specifier Parsing

/**
 Whether a specifier looks like a package reference (not relative or absolute).
 
 @param specifier - Bare import specifier
 
 @returns True for package-like specifiers (`\@scope/pkg/...` or `pkg/...`)
 
 @example
 ```ts
 isPackageSpecifier('\@scope/pkg/style.css') // → true
 isPackageSpecifier('./local.css')          // → false
 ```
 */
export function isPackageSpecifier(specifier: string,): boolean {
  return (!specifier.startsWith('.',)) && (!specifier.startsWith('/',));
}

/**
 Splits a package specifier into package name and subpath.
 Handles scoped (`\@scope/pkg/sub.css`) and unscoped (`pkg/sub.css`) packages.
 
 @param specifier - Bare package specifier
 
 @returns Tuple of [packageName, subpath] where subpath starts with `./` or is `.`
 
 @example
 ```ts
 splitPackageSpecifier('\@scope/pkg/sub/path.css') // → ['\@scope/pkg', './sub/path.css']
 splitPackageSpecifier('pkg')                     // → ['pkg', '.']
 ```
 */
export function splitPackageSpecifier(specifier: string,): [
  string,
  string,
] {
  if (specifier.startsWith('@',)) {
    // Scoped: @scope/pkg or @scope/pkg/sub/path.css
    /**
     Index of the second slash (after \@scope/pkg).
     */
    const secondSlash = specifier.indexOf(
      '/',
      specifier.indexOf('/',)
        + 1,
    );
    if (secondSlash === (-1)) {
      return [
        specifier,
        '.',
      ];
    }
    return [
      specifier.slice(
        0,
        secondSlash,
      ),
      `./${specifier.slice(secondSlash + 1,)}`,
    ];
  }

  // Unscoped: pkg or pkg/sub/path.css
  /**
   Index of the first slash
   */
  const firstSlash = specifier.indexOf('/',);
  if (firstSlash === (-1)) {
    return [
      specifier,
      '.',
    ];
  }
  return [
    specifier.slice(
      0,
      firstSlash,
    ),
    `./${specifier.slice(firstSlash + 1,)}`,
  ];
}

//endregion Specifier Parsing
