import {
  dirname,
  join,
  normalize,
} from 'node:path/posix';

//region Which file of the sources a specifier names
// What `claude-schema-resolve.test-fixture.ts` follows an import or an export through: the file a relative specifier or a
// package specifier of the family names, when the sources hold it.

/**
 The start of the name of every package of the family.
 */
const PACKAGE_PREFIX = '@monochromatic-dev/translation-repair-';

/**
 The file of the family that a package specifier names: the package's entry point for the package and for its `/ts`
 subpath, one file under its `src` for `/ts/<file>`.

 @param specifier - the specifier as written

 @returns Path of the file, empty when the specifier names no package of the family or a subpath that is none of those

 @example
 ```ts
 const file = packagedFileOf({ specifier: '@monochromatic-dev/translation-repair-provider/ts', },);
 // => 'provider/src/index.ts'
 ```
 */
function packagedFileOf({ specifier, }: { readonly specifier: string; },): string {
  if (!specifier.startsWith(PACKAGE_PREFIX,))
    return '';
  /**
   The package's directory and what follows it in the specifier.
   */
  const rest = specifier.slice(PACKAGE_PREFIX.length,);
  /**
   Where the directory ends.
   */
  const slash = rest.indexOf('/',);
  if (slash === (-1))
    return `${rest}/src/index.ts`;
  /**
   What follows the directory.
   */
  const subpath = rest.slice(slash + 1,);
  if (subpath === 'ts')
    return `${rest.slice(
      0,
      slash,
    )}/src/index.ts`;
  return subpath.startsWith('ts/',) ? `${rest.slice(
    0,
    slash,
  )}/src/${subpath.slice('ts/'.length,)}` : '';
}

/**
 The module of the sources that a specifier names.

 @param family - the modules of the sources

 @param from - path of the module that writes the specifier

 @param specifier - the specifier as written

 @returns Path of the module, empty when the specifier names none of the sources

 @example
 ```ts
 const target = targetOf({ family, from: 'a/src/a.ts', specifier: './b.ts', },);
 // => 'a/src/b.ts'
 ```
 */
export function targetOf(
  {
    family,
    from,
    specifier,
  }: {
    readonly family: ReadonlyMap<string, unknown>;
    readonly from: string;
    readonly specifier: string;
  },
): string {
  /**
   The path a relative specifier names, from the root of the family.
   */
  const relative = normalize(join(
    dirname(from,),
    specifier,
  ),);
  /**
   The path the specifier names, empty when it names a package outside the family.
   */
  const path = specifier.startsWith('.',) ? relative : packagedFileOf({ specifier, });
  return family.has(path,) ? path : '';
}

//endregion Which file of the sources a specifier names
