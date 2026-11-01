//region Rotation salt source
// THE REWRITE THE ROTATION SWEEP MAKES (ledger B701). The first round of a
// stage asks a window of seats from a bench rotated by a hash of the prompt
// (`stage/src/stage-fanout-window.ts`, `benchRotation`), so a scripted case
// that names a seat passes only for the sheet text that happens to rotate that
// seat into the window. The sweep runs the suites with the rotation start
// moved by a salt, and this file is the one place that moves it.
//
// THE FUNCTION IS RENAMED AND WRAPPED, NOT EDITED IN PLACE. The built bundle
// is minified (`...,2166136261)%size}`), the source is not (`return hash % size;`),
// and a bundler change reshapes the body. Only the declaration `function
// benchRotation(` is searched for, because the name is exported and the
// callers bind to it; the old body keeps its name with an `Unsalted` suffix
// and a wrapper under the old name adds the salt. A zero or negative size
// still answers zero, as the function does.

/**
 The text a declaration of the function begins with, in the minified bundle
 and in the source alike.
 */
const DECLARATION = 'function benchRotation(';

/**
 The keyword a declaration may carry before `function`, with its space.
 */
const EXPORT_KEYWORD = 'export ';

/**
 Start of every declaration of the function in a module.

 @param source - text of one module

 @returns Index of each `function benchRotation(` in the text, in order

 @example
 ```ts
 const at = declarationStarts({ source: 'function benchRotation(args){}', },); // [0]
 ```
 */
function declarationStarts({ source, }: { readonly source: string; },): readonly number[] {
  /**
   Where each declaration begins.
   */
  const starts: number[] = [];
  /**
   Where the next search begins.
   */
  let from = 0;
  for (let found = source.indexOf(DECLARATION,); found !== (-1); found = source.indexOf(
    DECLARATION,
    from,
  )) {
    starts.push(found,);
    from = found + DECLARATION.length;
  }
  return starts;
}

/**
 Text of a salted wrapper for the declaration, written without type syntax so
 a bundle and a TypeScript source both parse it.

 @param exported - the `export ` keyword the declaration carried, or an empty string

 @param salt - whole number added to the rotation start

 @returns Source of a function named `benchRotation` that calls the renamed original

 @example
 ```ts
 const wrapper = wrapperSource({ exported: '', salt: 3, },);
 ```
 */
function wrapperSource(
  {
    exported,
    salt,
  }: {
    readonly exported: string;
    readonly salt: number;
  },
): string {
  return `\n${exported}function benchRotation(args){`
    + `return args.size<=0?0:(benchRotationUnsalted(args)+${String(salt,)})%args.size}\n`;
}

/**
 A module's source with `benchRotation` shifted by a salt.

 @param source - text of one module

 @param salt - whole number added to the rotation start, below the largest bench of the family or above; zero shifts nothing

 @returns The rewritten text, or the text itself when the module declares no `benchRotation`

 @throws Error when the salt is not a whole number from zero, or the module declares `benchRotation` more than once, since a rewrite of an unknown shape would leave the salt doing nothing and the sweep passing

 @example
 ```ts
 const salted = saltedBenchRotation({ source: 'function benchRotation({size}){return 0}', salt: 2, },);
 ```
 */
export function saltedBenchRotation(
  {
    source,
    salt,
  }: {
    readonly source: string;
    readonly salt: number;
  },
): string {
  if ((!Number.isInteger(salt,)) || (salt < 0))
    throw new Error(`rotation salt must be a whole number from zero, got ${String(salt,)}`,);
  /**
   Every declaration of the function in the module.
   */
  const starts = declarationStarts({ source, },);
  if (starts.length === 0)
    return source;
  if (starts.length !== 1) {
    throw new Error(
      `module declares benchRotation ${String(starts.length,)} times; the rotation sweep rewrites exactly one declaration`,
    );
  }
  /**
   Where the one declaration begins.
   */
  const [start = 0,] = starts;
  /**
   Whether the keyword `export ` stands right before it.
   */
  const isExported = source.slice(
    Math.max(
      0,
      start - EXPORT_KEYWORD.length,
    ),
    start,
  ) === EXPORT_KEYWORD;
  /**
   Where the text kept before the declaration ends: before the keyword when there is one.
   */
  const headEnd = isExported ? start - EXPORT_KEYWORD.length : start;
  return `${source.slice(
    0,
    headEnd,
  )}function benchRotationUnsalted(${source.slice(start + DECLARATION.length,)}${wrapperSource({
      exported: isExported ? EXPORT_KEYWORD : '',
      salt,
    },)}`;
}

//endregion Rotation salt source
