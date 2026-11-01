/**
 Package manifest parsing and shipping-entry extraction.
 
 Collects every path a manifest declares as something consumers load, from
 `exports`, `main`, and `bin`. The `./ts` and `./ts/*` export keys are skipped
 because they are the sanctioned source channel, not a shipped artifact.
 
 Parsing lives here rather than at the caller so the manifest object never
 crosses a function boundary. Only its name and its declared targets leave,
 both of them plain strings, which keeps every later step working on values
 that share no identity with the parsed tree.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger for manifest parsing.
 */
const l = tagged({ tag: 'package-manifest', },);

/**
 Fields of a `package.json` this plugin reads.
 
 @internal
 */
export type PackageManifest = {
  /**
   Package name, required for a directory to count as a package root.
   */
  readonly name: string;
  /**
   Subpath exports map, string target, or absent.
   */
  readonly exports?: unknown;
  /**
   Legacy single entry point.
   */
  readonly main?: unknown;
  /**
   Executable entry point, string or name-to-path map.
   */
  readonly bin?: unknown;
};

/**
 Everything one manifest contributes, reduced to strings.
 
 @internal
 */
export type ManifestFacts = {
  /**
   Declared package name, matched against bare import specifiers.
   */
  readonly name: string;
  /**
   Package-relative or absolute target specifiers, unfiltered.
   */
  readonly shippingTargets: readonly string[];
};

/**
 Sentinel meaning manifest text describes no package root.
 
 @internal
 */
export const MANIFEST_UNUSABLE: unique symbol = Symbol(
  'manifest text that is unparseable or carries no package name',
);

/**
 Export key naming the TypeScript source entry.
 */
const SOURCE_EXPORT_KEY = './ts';

/**
 Prefix of export keys naming TypeScript source subpaths.
 */
const SOURCE_EXPORT_PREFIX = './ts/';

/**
 Narrows parsed JSON to a manifest carrying a package name.
 
 A directory without a named manifest is not a package root, so the ancestor
 walk keeps going rather than stopping at, say, a bare `tsconfig` folder that
 happens to hold a nameless `package.json`.
 
 @param value - parsed `package.json` contents
 
 @returns true when value carries a string `name`
 
 @example
 ```ts
 isPackageManifest(JSON.parse(text));
 ```
 
 @internal
 */
export function isPackageManifest(value: unknown,): value is PackageManifest {
  if (((typeof value) !== 'object') || (value === null))
    return false;
  if (!('name' in value))
    return false;
  return (typeof value.name) === 'string';
}

/**
 Narrows an unknown value to an inspectable non-null object.
 
 Takes a positional parameter because a type predicate cannot reference a
 destructured binding.
 
 @param value - candidate manifest subtree
 
 @returns true when the value has enumerable fields worth walking
 
 @example
 ```ts
 isRecordLike({ default: './a.mjs' });
 ```
 */
function isRecordLike(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Collects every string leaf of one `exports` subtree.
 
 Conditional exports nest objects and arrays to arbitrary depth, so the walk
 uses an explicit work stack rather than recursion.
 
 @param node - `exports` value, one condition subtree, `main`, or `bin`
 
 @returns every string target found beneath node, in discovery order
 
 @example
 ```ts
 stringTargets({ node: { types: './a.d.mts', default: './a.mjs' } });
 ```
 
 @internal
 */
export function stringTargets({ node, }: {
  /**
   Manifest value to flatten into its string leaves.
   */
  readonly node: unknown;
},): readonly string[] {
  /**
   Collected string leaves.
   */
  const targets: string[] = [];
  /**
   Pending subtrees, popped until the whole value is flattened.
   */
  const pending: unknown[] = [node,];
  while (pending.length > 0) {
    /**
     Next subtree to inspect; the guard above proves the pop is present.
     */
    const current = pending.pop();
    if ((typeof current) === 'string') {
      targets.push(current,);
      continue;
    }
    if (Array.isArray(current,)) {
      /**
       Array fallbacks re-typed as unknown; `Array.isArray` widens to `any[]`.
       */
      const fallbacks = current as unknown[];
      pending.push(...fallbacks,);
      continue;
    }
    if (isRecordLike(current,))
      pending.push(...Object.values(current,),);
  }
  return targets;
}

/**
 Parses manifest text, treating malformed JSON as absence.
 
 @param text - `package.json` contents
 
 @returns parsed contents, or undefined when the text is not JSON
 
 @example
 ```ts
 parseManifestText({ text: '{"name":"\@scope/pkg"}' });
 ```
 */
function parseManifestText({ text, }: {
  /**
   Manifest file contents.
   */
  readonly text: string;
},): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error: unknown) {
    l.debug(`manifest text rejected as malformed: ${String(error,)}`,);
    return undefined;
  }
}

/**
 Reduces manifest text to the package name and its declared shipping targets.
 
 Reads `exports` (skipping the source subpath keys), `main`, and `bin`. A
 `null` export target flattens to nothing, so blocked subpaths contribute no
 directory.
 
 An `exports` object counts as a subpath map only when every key names a
 subpath. That distinguishes `{ '.': ... }` from a condition shorthand such as
 `{ types: ..., default: ... }`, and only the former carries source subpath
 keys worth skipping.
 
 @param text - `package.json` contents
 
 @returns package name and declared targets, or {@link MANIFEST_UNUSABLE}
 
 @example
 ```ts
 manifestFacts({ text: '{"name":"\@scope/pkg","main":"./dist/final/node/index.mjs"}' });
 ```
 
 @internal
 */
export function manifestFacts({ text, }: {
  /**
   Manifest file contents.
   */
  readonly text: string;
},): ManifestFacts | typeof MANIFEST_UNUSABLE {
  /**
   Parsed manifest, kept local so no caller-owned value reaches the walk below.
   */
  const parsed = parseManifestText({ text, },);
  if (!isPackageManifest(parsed,))
    return MANIFEST_UNUSABLE;

  /**
   Declared `exports` field, in whichever of its shapes was authored.
   */
  const exportsField: unknown = parsed.exports;
  /**
   Export entries, empty unless the field is an object worth keying.
   */
  const exportEntries = isRecordLike(exportsField,)
    ? Object.entries(exportsField,)
    : [];
  /**
   Whether every key names a subpath, which an array's numeric keys never do.
   */
  const keysAreSubpaths = (exportEntries.length > 0)
    && exportEntries.every(function namesSubpath([key,],): boolean {
      return key.startsWith('.',);
    },);

  /**
   Targets contributed by `exports`, with the source subpath keys removed.
   */
  const exportTargets = keysAreSubpaths
    ? exportEntries
      .filter(function keepShippingKey([key,],): boolean {
        return (key !== SOURCE_EXPORT_KEY) && (!key.startsWith(SOURCE_EXPORT_PREFIX,));
      },)
      .flatMap(function targetsOfEntry([, value,],): readonly string[] {
        return stringTargets({ node: value, },);
      },)
    : stringTargets({ node: exportsField, },);

  return {
    name: parsed.name,
    shippingTargets: [
      ...exportTargets,
      ...stringTargets({ node: parsed.main, },),
      ...stringTargets({ node: parsed.bin, },),
    ],
  };
}
