import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

//region Package families

// A package family is a set of workspace packages split out of one package, whose classes are checked by identity across
// package boundaries. A library build that inlined its siblings would hand a test one copy of a sibling class from its own
// build and another from the sibling's build, and `instanceof` would fail between them. So a family member's library build
// keeps its siblings as imports, and the emitted import names the sibling's built entry rather than its source subpath.

/**
 Workspace source subpath every package exports, which a family member's
 emitted imports trade for the bare package name.
 */
const SOURCE_SUBPATH = '/ts';

/**
 Read which production dependencies of a package belong to its family.

 @param packageDir - directory holding the consuming package.json; defaults to the build cwd
 
 @param familyPrefix - package-name prefix every member of the family shares

 @returns Family members the package depends on, by package name

 @throws When package.json, or its `dependencies` value, is not a JSON object

 @example
 ```ts
 const family = await familyDependencies({ familyPrefix: '\@monochromatic-dev/translation-repair-', },);
 ```
 */
export async function familyDependencies({
  packageDir = process.cwd(),
  familyPrefix,
}: {
  readonly packageDir?: string;
  readonly familyPrefix: string;
},): Promise<readonly string[]> {
  /**
   Manifest read, named in every refusal so a reader knows which package to fix.
   */
  const manifestPath = join(
    packageDir,
    'package.json',
  );
  /**
   Parsed manifest, not yet known to be an object.
   */
  const manifest: unknown = JSON.parse(await readFile(
    manifestPath,
    'utf8',
  ),);
  if (((typeof manifest) !== 'object') || (manifest === null)
    || Array.isArray(manifest,))
    throw new Error(`${manifestPath} must hold a JSON object`,);
  /**
   Production dependency map; a manifest without one depends on nothing.
   */
  const dependencies: unknown = (manifest as { readonly dependencies?: unknown; }).dependencies ?? {};
  if (((typeof dependencies) !== 'object') || (dependencies === null)
    || Array.isArray(dependencies,))
    throw new Error(`${manifestPath} "dependencies" must be a JSON object when present`,);
  return Object.keys(dependencies,)
    .filter(function isMember(name: string,): boolean {
      return name.startsWith(familyPrefix,);
    },);
}

/**
 Name the import a family member's build emits for one module id: a sibling's
 source subpath becomes the sibling's bare name, which resolves to its built
 entry, and every other id is kept.

 @param id - module id the bundle imports
 
 @param family - family members the package depends on

 @returns Specifier the emitted chunk imports

 @throws When the id names a file under a member's source subpath,
   which the member's built entry does not expose on its own

 @example
 ```ts
 familyEntrySpecifier({ id: '\@x/a/ts', family: new Set(['\@x/a',],), },); // '@x/a'
 ```
 */
export function familyEntrySpecifier({
  id,
  family,
}: {
  readonly id: string;
  readonly family: ReadonlySet<string>
},): string {
  for (const name of family) {
    if (id === `${name}${SOURCE_SUBPATH}`)
      return name;
    if (id.startsWith(`${name}${SOURCE_SUBPATH}/`,)) {
      throw new Error(
        `${id} names one file of family member ${name}; import ${name}${SOURCE_SUBPATH}, whose built entry ${name} exposes it`,
      );
    }
  }
  return id;
}

//endregion Package families
