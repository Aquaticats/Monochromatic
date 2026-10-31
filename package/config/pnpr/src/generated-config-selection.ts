import {
  PNPR_EXCLUDED_PACKAGES,
  PNPR_PACKAGE_NAME_CHARACTERS,
  PNPR_PACKAGE_SCOPE,
  PNPR_TEST_FIXTURE_DIRECTORY_PREFIX,
} from './generated-config-settings.ts';
import { isTypeScriptSourcePath, } from './publish-plan.ts';
import type { WorkspaceEntry, } from './registry-read.ts';

//region Which workspace packages publish to pnpr

/**
 Reports whether a manifest entry field names a file Node can load from `node_modules`.

 @param entry - Value of `main`, `module`, or one `bin` target.

 @returns Whether the value is a non-empty path that is not TypeScript source.

 @example
 ```ts
 isPnprLoadableEntryPath('src/index.ts');
 // => false
 ```
 */
function isPnprLoadableEntryPath(entry: unknown,): boolean {
  return ((typeof entry) === 'string')
    && (entry !== '')
    && (!isTypeScriptSourcePath(entry,));
}

/**
 Reports whether a manifest still declares an entry point once `./ts` export subpaths are stripped for publishing,
 ignoring `main` and `module` shadowed by `exports` and entries that name TypeScript source.

 @param manifest - Parsed workspace package.json.

 @returns Whether consumers of the published tarball could import or execute anything.

 @example
 ```ts
 pnprManifestHasEntryPoint({ manifest: { exports: { './ts': './src/index.ts' } } });
 // => false
 ```
 */
export function pnprManifestHasEntryPoint(
  { manifest, }: { readonly manifest: Readonly<Record<string, unknown>>; },
): boolean {
  /**
   Manifest fields that can expose something to consumers of the published tarball.
   */
  const {
    exports: exportsField,
    main,
    module,
    bin,
  } = manifest;

  if (((typeof exportsField) === 'string') && (exportsField !== ''))
    return true;

  if (((typeof exportsField) === 'object') && (exportsField !== null)) {
    /**
     Subpath or condition keys of the exports map.
     */
    const exportKeys = Object.keys(exportsField,);
    /**
     Whether keys are subpaths (`.`-prefixed) rather than root export conditions.
     */
    const isSubpathMap = exportKeys.some(function isSubpath(key,) {
      return key.startsWith('.',);
    },);
    /**
     Export keys that survive `./ts` stripping at publish time.
     */
    const publishedKeys = isSubpathMap
      ? exportKeys.filter(function isNotTsSubpath(key,) {
        return (key !== './ts') && (!key.startsWith('./ts/',));
      },)
      : exportKeys;
    if (publishedKeys.length > 0)
      return true;
  }

  // Node ignores `main` and `module` whenever `exports` exists, and cannot load TypeScript source from `node_modules` (#537).
  if ((exportsField === undefined) && [
    main,
    module,
  ].some(function isLoadableRootEntry(entry,) {
    return isPnprLoadableEntryPath(entry,);
  },))
    return true;

  if ((typeof bin) === 'string')
    return isPnprLoadableEntryPath(bin,);

  return ((typeof bin) === 'object')
    && (bin !== null)
    && Object.values(bin,)
      .some(function isLoadableBin(entry,) {
        return isPnprLoadableEntryPath(entry,);
      },);
}

/**
 Throws unless a package name is inside the hosted scope and uses only npm name characters.

 @param name - Package name about to be interpolated into single-quoted YAML.

 @throws Error when the name is outside the scope or contains other characters.

 @example
 ```ts
 assertPnprPackageName({ name: '@monochromatic-dev/or-throw-core' });
 ```
 */
export function assertPnprPackageName({ name, }: { readonly name: string; },): void {
  /**
   Name after the scope prefix, checked character by character.
   */
  const unscopedName = name.slice(PNPR_PACKAGE_SCOPE.length,);
  if ((!name.startsWith(PNPR_PACKAGE_SCOPE,)) || (unscopedName === ''))
    throw new Error(`pnpr config: package name ${JSON.stringify(name,)} is not a ${PNPR_PACKAGE_SCOPE} name`,);
  // Index scan over UTF-16 units: any non-ASCII unit fails the allow-list, so surrogate pairs cannot slip through.
  for (let index = 0; index < unscopedName.length; index += 1) {
    if (!PNPR_PACKAGE_NAME_CHARACTERS.has(unscopedName.charAt(index,),))
      throw new Error(`pnpr config: package name ${JSON.stringify(name,)} is not a ${PNPR_PACKAGE_SCOPE} name`,);
  }
}

/**
 Selects the workspace packages that publish to pnpr and so populate its OIDC trust list.
 Excludes test fixtures, versionless manifests, entry-less manifests, and reviewed exclusions.

 @param workspacePackages - Every workspace package with its parsed manifest.

 @param excludedPackages - Reviewed exclusions keyed by package name; defaults to the checked-in list.

 @returns Sorted package names.

 @throws Error when an exclusion names no workspace package or a selected or excluded name is invalid.

 @example
 ```ts
 selectPnprPublishedNames({ workspacePackages: [...await readWorkspaceEntries({ rootDir: '.' })] });
 ```
 */
export function selectPnprPublishedNames(
  {
    workspacePackages,
    excludedPackages = PNPR_EXCLUDED_PACKAGES,
  }: {
    readonly workspacePackages: readonly WorkspaceEntry[];
    readonly excludedPackages?: Readonly<Record<string, string>>;
  },
): readonly string[] {
  /**
   Names present in the workspace, for validating exclusions.
   */
  const workspaceNames: ReadonlySet<string> = new Set(workspacePackages.map(function toName(
    workspacePackage: WorkspaceEntry,
  ) {
    return workspacePackage.name;
  },),);
  /**
   Exclusions that no longer match a workspace package.
   */
  const staleExclusions = Object.keys(excludedPackages,)
    .filter(function isStale(name,) {
      return !workspaceNames.has(name,);
    },);
  if (staleExclusions.length > 0)
    throw new Error(`pnpr config: exclusions name no workspace package: ${staleExclusions.join(', ',)}`,);

  /**
   Sorted names that publish to pnpr.
   */
  const publishedNames = workspacePackages
    .filter(function isPublishable(workspacePackage: WorkspaceEntry,) {
      return (!workspacePackage.directory
        .startsWith(PNPR_TEST_FIXTURE_DIRECTORY_PREFIX,))
        // npm refuses to pack or publish a manifest without a version.
        && ((typeof workspacePackage.manifest
          .version) === 'string')
        && pnprManifestHasEntryPoint({ manifest: workspacePackage.manifest, },)
        && (!Object.hasOwn(
          excludedPackages,
          workspacePackage.name,
        ));
    },)
    .map(function toName(workspacePackage: WorkspaceEntry,) {
      return workspacePackage.name;
    },)
    .toSorted();
  for (const name of [
    ...publishedNames,
    ...Object.keys(excludedPackages,),
  ])
    assertPnprPackageName({ name, },);
  return publishedNames;
}

//endregion Which workspace packages publish to pnpr
