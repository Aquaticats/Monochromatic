/**
 Reads whether a deletion target has collapsed to a broader path.
 
 The literal-path rule exists because a value the shell could not supply
 collapses to itself:
 `rm -rf "$HOME"` removes the home folder,
 `rm -rf "/$DIR"` removes a top-level folder when `DIR` is empty,
 and `rm -rf "${PREFIX}//$X"` removes whatever lies under a doubled separator.
 A target written that way needs no variable to be dangerous,
 so the shape is refused on its own:
 `/`, the account's home folder, a top-level folder,
 a path holding `//`,
 and a path with a segment `undefined` or `null`.
 
 The home folder is read from the caller,
 never from a username written here,
 because the account that runs the guard is the only one it can name.
 
 @module
 */

//region Constants

/**
 Sentinel returned when a target names one readable path below the refused shapes.
 */
const NOT_COLLAPSED: unique symbol = Symbol('target names one readable path',);

/**
 Root of the filesystem, which no deletion may name.
 */
const FILESYSTEM_ROOT = '/';

/**
 Separator between path segments, doubled where an empty value collapsed.
 */
const PATH_SEPARATOR = '/';

/**
 Length a path keeps when only its separators are left, which is the bare root.
 */
const ROOT_LENGTH = 1;

/**
 Two separators in a row, which only an empty path segment writes.
 */
const DOUBLED_SEPARATOR = `${PATH_SEPARATOR}${PATH_SEPARATOR}`;

/**
 Segment values a shell writes when the value behind a segment was undefined.
 */
const EMPTY_SEGMENT_NAMES: ReadonlySet<string> = new Set([
  'null',
  'undefined',
]);

//endregion Constants

//region Types

/**
 Why a deletion target names more than the path it looks like.
 */
type CollapsedTargetReason =
  | 'a doubled separator'
    | 'a segment named undefined or null'
    | 'a top-level folder'
    | 'the account home folder'
    | 'the filesystem root';

//endregion Types

//region Reads

/**
 Drops the separators that follow a path's last named segment.
 
 @param value - path as the shell would hand it to the program
 
 @returns the path without trailing separators, keeping a bare root
 
 @example
 ```ts
 withoutTrailingSeparators('/home/user/'); // '/home/user'
 ```
 */
function withoutTrailingSeparators(value: string,): string {
  return (function readsNamedEnd(): string {
    /**
     Offset just past the last segment the path names.
     */
    let end = value.length;
    while ((end > ROOT_LENGTH) && (value[end - 1] === PATH_SEPARATOR)) end -= 1;
    return value.slice(
      0,
      end,
    );
  })();
}

/**
 Reports whether a path names one folder directly under the filesystem root.
 
 @param value - path without trailing separators
 
 @returns whether the path is one segment deep, such as `/home` or `/usr`
 
 @example
 ```ts
 isTopLevelFolder('/home'); // true
 isTopLevelFolder('/home/user'); // false
 ```
 */
function isTopLevelFolder(value: string,): boolean {
  if (!value.startsWith(PATH_SEPARATOR,))
    return false;
  /**
   Path text after the leading separator.
   */
  const belowRoot = value.slice(PATH_SEPARATOR.length,);
  return (belowRoot.length > 0)
    && (!belowRoot.includes(PATH_SEPARATOR,));
}

/**
 Reports whether one path segment is a value a shell writes for an undefined one.
 
 @param segment - one segment of the path
 
 @returns whether the segment spells `undefined` or `null`
 
 @example
 ```ts
 isEmptySegmentName('undefined'); // true
 isEmptySegmentName('user'); // false
 ```
 */
function isEmptySegmentName(segment: string,): boolean {
  return EMPTY_SEGMENT_NAMES.has(segment,);
}

//endregion Reads

//region Predicates

/**
 Reads why one deletion target names more than the path it looks like.
 
 The shapes are read from the value the program would receive,
 so a quoted `"/"` and a bare `/` are the same target.
 
 @param value - target value after quote removal
 
 @param homeFolder - the account's home folder, injected so a fixture need not run as that account
 
 @returns why the target collapsed, or {@link NOT_COLLAPSED} for one readable path
 
 @example
 ```ts
 collapsedTargetReason({ value: '/', homeFolder: '/home/user' }); // 'the filesystem root'
 collapsedTargetReason({ value: '/home/user/temp/agent/x', homeFolder: '/home/user' }); // NOT_COLLAPSED
 ```
 */
function collapsedTargetReason({
  value,
  homeFolder,
}: {
  readonly value: string;
  readonly homeFolder: string;
},): CollapsedTargetReason | typeof NOT_COLLAPSED {
  if (value.length === 0)
    return NOT_COLLAPSED;
  if (value.includes(DOUBLED_SEPARATOR,))
    return 'a doubled separator';
  if (value.split(PATH_SEPARATOR,)
    .some(isEmptySegmentName,))
    return 'a segment named undefined or null';

  /**
   Path without the separators that follow its last named segment.
   */
  const namedPath = withoutTrailingSeparators(value,);
  if (namedPath === FILESYSTEM_ROOT)
    return 'the filesystem root';
  /**
   Whether the caller named a home folder at all, which an empty value does not.
   */
  const homeFolderNamed = homeFolder.length > 0;
  if (homeFolderNamed
    && (namedPath === withoutTrailingSeparators(homeFolder,)))
    return 'the account home folder';
  if (isTopLevelFolder(namedPath,))
    return 'a top-level folder';
  return NOT_COLLAPSED;
}

//endregion Predicates

export {
  collapsedTargetReason,
  isEmptySegmentName,
  isTopLevelFolder,
  NOT_COLLAPSED,
  withoutTrailingSeparators,
};
export type { CollapsedTargetReason, };
