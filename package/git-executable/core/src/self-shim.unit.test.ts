/**
 Self-shim markers name the wrapper as this workspace produces it. @module
 */

import { existsSync, } from 'node:fs';
import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SELF_SHIM_MARKERS, } from '../dist/final/node/index.mjs';

//region Workspace ownership fixtures

/**
 Repository root four directories above this test file.
 */
const REPOSITORY_ROOT = fileURLToPath(new URL('../../../../', import.meta.url,),);

/**
 Separator a Windows command shim writes between package path segments.
 */
const WINDOWS_PATH_SEPARATOR = '\\';

/**
 Separator every platform spells a package path or a repository path with.
 */
const PORTABLE_PATH_SEPARATOR = '/';

/**
 Scope prefix of a workspace package name,
 which no repository path begins with.
 */
const PACKAGE_NAME_PREFIX = '@';

/**
 Path marker no workspace tree can produce,
 proving the ownership check reports one.
 */
const ABSENT_PATH_MARKER = 'package/git-policy/cli/dist/final/node/absent.mjs';

/**
 Reports which markers name nothing this workspace declares or produces.
 
 @param markers - Text markers searched for in script candidates.
 
 @param declaredNames - Names the workspace package manifests declare.
 
 @returns Markers with no owner, in the order given.
 
 @example
 ```ts
 await markersWithoutOwner({ markers: [...SELF_SHIM_MARKERS], declaredNames });
 // => []
 ```
 */
function markersWithoutOwner({
  markers,
  declaredNames,
}: {
  readonly markers: readonly string[];
  readonly declaredNames: ReadonlySet<string>;
},): readonly string[] {
  return markers.filter(function namesNothingThisWorkspaceHas(marker,) {
    /**
     Marker with platform separators folded to the portable spelling,
     so a Windows command shim's marker is judged by the package it names.
     */
    const portableMarker = marker.replaceAll(WINDOWS_PATH_SEPARATOR, PORTABLE_PATH_SEPARATOR,);
    if (portableMarker.startsWith(PACKAGE_NAME_PREFIX,))
      return !declaredNames.has(portableMarker,);
    return !existsSync(join(REPOSITORY_ROOT, portableMarker,),);
  },);
}

/**
 Names every package manifest under the repository's package directory declares.
 
 @returns Declared names of workspace packages, dependency trees excluded.
 
 @example
 ```ts
 await declaredPackageNames();
 // => Set(1) { '@monochromatic-dev/git-executable-core' }
 ```
 */
/**
 Reports whether a parsed package manifest declares a package name.
 
 @param parsed - Value parsed from a package manifest.
 
 @returns Whether the value is an object whose `name` is a string.
 
 @example
 ```ts
 isNamedManifest({ name: '@monochromatic-dev/git-executable-core' });
 // => true
 ```
 */
function isNamedManifest(parsed: unknown,): parsed is { readonly name: string } {
  if ((typeof parsed) !== 'object')
    return false;
  if (parsed === null)
    return false;
  if (!('name' in parsed))
    return false;
  return (typeof parsed.name) === 'string';
}

/**
 Names every package manifest under the repository's package directory declares.
 
 @returns Declared names of workspace packages, dependency trees excluded.
 
 @example
 ```ts
 await declaredPackageNames();
 // => Set(1) { '@monochromatic-dev/git-executable-core' }
 ```
 */
async function declaredPackageNames(): Promise<ReadonlySet<string>> {
  /**
   Workspace manifests outside dependency and build directories.
   */
  const manifests = await Array.fromAsync(glob('package/*/*/package.json', {
    cwd: REPOSITORY_ROOT,
    exclude: [
      '**/node_modules/**',
      '**/target/**',
    ],
  },),);
  /**
   Manifest texts, read together so one slow disk read does not delay the rest.
   */
  const manifestTexts: string[] = await Promise.all(
    manifests.map(async function readManifest(manifest,) {
      return await readFile(join(REPOSITORY_ROOT, manifest,), 'utf8',);
    },),
  );
  /**
   Names the manifests declare.
   */
  const names = new Set<string>();
  for (const text of manifestTexts) {
    /**
     Parsed manifest carrying the declared name.
     */
    const parsed: unknown = JSON.parse(text,);
    if (isNamedManifest(parsed,))
      names.add(parsed.name,);
  }
  return names;
}

//endregion Workspace ownership fixtures

await describe({
  name: 'self shim markers',
  children: [
    it({
      name: 'name a package this workspace declares or a file it produces',
      fn: async function markersNameWorkspaceOutput(): Promise<void> {
        /**
         Names the workspace package manifests declare.
         */
        const declaredNames = await declaredPackageNames();
        /**
         Markers searched for in script candidates.
         */
        const markers = [...SELF_SHIM_MARKERS,];
        // A marker set with no member would make every check below pass without reading anything.
        expect(markers.length,).toBeGreaterThan(0,);

        expect(markersWithoutOwner({
          markers,
          declaredNames,
        },),).toEqual([],);
        // The same check must report a path marker the workspace cannot produce, or it proves nothing.
        expect(markersWithoutOwner({
          markers: [...markers, ABSENT_PATH_MARKER,],
          declaredNames,
        },),).toEqual([ABSENT_PATH_MARKER,],);
      },
    },),
  ],
},);
