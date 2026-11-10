/**
 Guard rule SGD's name-equals-path invariant for this package.

 Rule SGD requires every workspace package name to equal `@monochromatic-dev/`
 plus the package path below the workspace packages root with `/` replaced by `-`.
 This package sits at `package/pi-plugin/goal`, so its manifest name must be
 `@monochromatic-dev/pi-plugin-goal`; a name that drops a path segment hides which
 directory the package comes from.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { dirname, join, sep, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/** npm scope every workspace package name carries. */
const WORKSPACE_PACKAGE_SCOPE = '@monochromatic-dev';

/** Directory name of the workspace packages root. */
const WORKSPACE_PACKAGES_ROOT_NAME = 'package';

/** Separator rule SGD substitutes for path separators. */
const PATH_SEGMENT_SEPARATOR = '-';

/** Value `lastIndexOf` returns when the requested segment is absent. */
const SEGMENT_NOT_FOUND_INDEX = -1;

/** Manifest of this package, resolved from the source directory holding the tests. */
const MANIFEST_PATH = join(import.meta.dirname, '..', 'package.json',);

/**
 Derive the package path below the workspace packages root.

 @param packageDirectory - absolute package directory

 @returns path segments below the workspace packages root
 */
function workspaceRelativeSegments(packageDirectory: string,): string[] {
  const segments = packageDirectory.split(sep,);
  const rootIndex = segments.lastIndexOf(WORKSPACE_PACKAGES_ROOT_NAME,);
  if (rootIndex === SEGMENT_NOT_FOUND_INDEX) {
    throw new Error(`package directory ${packageDirectory} is not below ${WORKSPACE_PACKAGES_ROOT_NAME}${sep}`,);
  }
  return segments.slice(rootIndex + 1,);
}

/**
 Narrow parsed JSON to a manifest that declares a string name.

 @param manifest - parsed `package.json` contents

 @returns whether the manifest declares a string name
 */
function isNamedManifest(manifest: unknown,): manifest is { readonly name: string } {
  if ((manifest === null) || ((typeof manifest) !== 'object'))
    return false;
  if (!('name' in manifest))
    return false;
  const { name, } = manifest;
  return (typeof name) === 'string';
}

/**
 Read the manifest name from parsed package JSON.

 @param manifest - parsed `package.json` contents

 @returns declared package name
 */
function readManifestName(manifest: unknown,): string {
  if (!isNamedManifest(manifest))
    throw new Error('package.json declares no string name',);
  return manifest.name;
}

await describe({
  name: 'package name',
  children: [
    it({
      name: 'equals the scope plus the path below the workspace packages root',
      fn: async () => {
        const manifest: unknown = JSON.parse(await readFile(MANIFEST_PATH, 'utf8',),);
        const segments = workspaceRelativeSegments(dirname(import.meta.dirname,),);
        const expectedName = `${WORKSPACE_PACKAGE_SCOPE}/${segments.join(PATH_SEGMENT_SEPARATOR,)}`;
        expect(readManifestName(manifest,),).toBe(expectedName,);
      },
    },),
  ],
},);
