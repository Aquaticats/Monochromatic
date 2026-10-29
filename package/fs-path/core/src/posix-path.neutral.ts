/**
 Pure-JS POSIX path operations, selected by `package.json` `imports` under
 the `default` condition of `#posix-path`: browsers, workers, and every
 bundler that does not resolve the `node` condition. Semantics follow
 `node:path/posix`, which `posix-path.node.ts` delegates to under Node
 and Bun; both modules export the same names with the same signatures.

 @module
 */

//region Shared

/**
 Unicode code point for `/`.
 */
const SLASH_CODE_POINT = 47;

/**
 POSIX path separator.
 */
export const sep = '/';

/**
 Whether a path starts with `/`.

 @param filePath - Path to check

 @returns True when the first code point is `/`

 @example
 ```ts
 isAbsolute('/foo/bar'); // true
 isAbsolute('foo/bar');  // false
 ```
 */
export function isAbsolute(filePath: string,): boolean {
  return (filePath.length > 0) && (filePath.codePointAt(0,) === SLASH_CODE_POINT);
}

//endregion Shared

//region normalize: resolve `.` and `..`, collapse slashes

/**
 Normalizes a path by resolving `.` and `..` segments and collapsing
 consecutive slashes. Does not resolve against cwd; just cleans the string.

 @param filePath - Raw path to normalize

 @returns Normalized path

 @example
 ```ts
 normalize('/foo/bar//baz/./qux/../quux'); // '/foo/bar/baz/quux'
 normalize('');                             // '.'
 ```
 */
export function normalize(filePath: string,): string {
  if (filePath === '')
    return '.';

  /**
   Whether the input is rooted
   */
  const isRoot = isAbsolute(filePath,);
  /**
   Whether the input ends with a trailing slash
   */
  const trailingSlash = filePath.codePointAt(filePath.length - 1,) === SLASH_CODE_POINT;

  /**
   Path segments split on `/`
   */
  const parts = filePath.split('/',);
  /**
   Stack of resolved segments built by walking the input
   */
  const resolved: string[] = [];

  for (const part of parts) {
    if ((part === '') || (part === '.'))
      continue;
    if (part === '..') {
      // Don't pop past root
      if ((resolved.length > 0) && (resolved.at(-1,) !== '..'))
        resolved.pop();
      else if (!isRoot)
        resolved.push('..',);
    }
    else {
      resolved.push(part,);
    }
  }

  /**
   Joined result without root prefix
   */
  let result = resolved.join('/',);

  if (isRoot)
    result = `/${result}`;
  if ((result === '') || (result === '/'))
    return isRoot ? '/' : '.';
  if (trailingSlash)
    result += '/';
  return result;
}

//endregion normalize

//region dirname

/**
 Finds the last character that is not a slash, with one backward pass.

 @param filePath - POSIX path

 @returns index of the last non-slash character, or -1 when every character is a slash

 @example
 ```ts
 lastNonSlashIndex('/a/b//'); // 3
 ```
 */
function lastNonSlashIndex(filePath: string,): number {
  for (let index = filePath.length - 1; index >= 0; index -= 1) {
    if (filePath.codePointAt(index,) !== SLASH_CODE_POINT)
      return index;
  }
  return -1;
}

/**
 Finds the slash that separates the directory portion from the basename,
 ignoring trailing slashes: the last slash before the basename's last
 character, at index 1 or later.

 @param filePath - non-empty POSIX path

 @returns index of the separating slash, or -1 when there is none past index 0

 @example
 ```ts
 directoryBoundary('/a/b/'); // 2
 ```
 */
function directoryBoundary(filePath: string,): number {
  /**
   Last character of the basename; below 1 means the path has no basename
   past a leading slash.
   */
  const basenameEnd = lastNonSlashIndex(filePath,);
  if (basenameEnd < 1)
    return -1;
  /**
   Slash preceding the basename, if any.
   */
  const boundary = filePath.lastIndexOf(
    '/',
    basenameEnd,
  );
  return (boundary >= 1) ? boundary : -1;
}

/**
 Returns the directory portion of a POSIX path.

 @param filePath - POSIX path

 @returns Parent directory path

 @example
 ```ts
 dirname('/foo/bar/baz.ts'); // '/foo/bar'
 dirname('/foo/bar/');       // '/foo'
 dirname('');                // '.'
 ```
 */
export function dirname(filePath: string,): string {
  if (filePath === '')
    return '.';

  /**
   Whether the input path is rooted
   */
  const isRoot = isAbsolute(filePath,);
  /**
   Index of the slash that ends the directory portion: the first slash met
   scanning backward once the trailing run of slashes and the basename are
   behind, or -1 when the basename reaches the start. Mirrors the scan in
   `node:path/posix`, which the node build delegates to, so both builds
   agree on every input.
   */
  const boundary = directoryBoundary(filePath,);

  if (boundary === (-1))
    return isRoot ? '/' : '.';
  if (isRoot && (boundary === 1))
    return '//';
  return filePath.slice(
    0,
    boundary,
  );
}

//endregion dirname

//region join

/**
 Joins path segments with `/` and normalizes the result.

 @param segments - Path segments to join

 @returns Joined and normalized path

 @example
 ```ts
 join(['foo', 'bar', 'baz']);   // 'foo/bar/baz'
 join(['/root', '../sibling']); // '/sibling'
 join([]);                      // '.'
 ```
 */
export function join(segments: readonly string[],): string {
  if (segments.length === 0)
    return '.';
  /**
   Raw concatenation of all non-empty segments
   */
  const joined = segments
    .filter(function isNonEmpty(segment,) {
      return segment !== '';
    },)
    .join('/',);
  if (joined === '')
    return '.';
  return normalize(joined,);
}

//endregion join

//region resolve

/**
 Working directory used as the base of relative resolution: the process
 working directory where a `process.cwd` function exists (Deno, or a Node
 consumer whose bundler resolved the `default` condition), otherwise `/`.

 @returns Absolute base directory

 @example
 ```ts
 resolutionBase(); // '/' in a browser
 ```
 */
function resolutionBase(): string {
  if (((typeof process) !== 'undefined') && ((typeof process.cwd) === 'function'))
    return process.cwd();
  return '/';
}

/**
 Resolves a sequence of paths to an absolute path.

 Processes segments right-to-left: each absolute segment resets the base,
 relative segments prepend to the current result. When no segment is
 absolute, prepends {@link resolutionBase}.

 @param segments - Path segments to resolve

 @returns Absolute, normalized path

 @example
 ```ts
 resolve(['/foo', 'bar', './baz']); // '/foo/bar/baz'
 resolve(['foo', '/bar', 'baz']);   // '/bar/baz'
 ```
 */
export function resolve(segments: readonly string[],): string {
  /**
   Index of the rightmost segment starting with `/`. Matches the
   right-to-left walk semantics of `node:path.resolve`: only segments
   from that point onward contribute, since each absolute segment
   discards everything to its left.
   */
  const absoluteIndex = segments.findLastIndex(function isAbsoluteSegment(segment,) {
    return isAbsolute(segment,);
  },);

  /**
   Segments from the rightmost absolute (or start when none) to end.
   */
  const relevantSegments = segments.slice(absoluteIndex === (-1) ? 0 : absoluteIndex,);
  /**
   Joined path built from the relevant segments, dropping empty entries.
   */
  const partial = relevantSegments
    .filter(function isNonEmpty(segment,) {
      return segment !== '';
    },)
    .join('/',);

  /**
   Absolute composition: prepend the base when no segment supplied a root.
   */
  const composed = absoluteIndex === (-1) ? `${resolutionBase()}/${partial}` : partial;
  /**
   Normalized absolute path
   */
  const normalized = normalize(composed,);

  if ((normalized.length > 1)
    && (normalized.codePointAt(normalized.length - 1,) === SLASH_CODE_POINT)) {
    return normalized.slice(
      0,
      -1,
    );
  }
  return normalized;
}

//endregion resolve
