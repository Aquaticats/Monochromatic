import {
  glob,
  readFile,
} from 'node:fs/promises';
import { isBuiltin, } from 'node:module';
import { join, } from 'node:path';
import {
  dirname,
  join as joinPosix,
  normalize,
} from 'node:path/posix';

import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  isFamilyPackage,
  targetOf,
} from './claude-schema-target.test-fixture.ts';
import {
  moduleLoadOf,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region Specifiers the check cannot resolve
// A specifier names a module through the file system, the lockfile or the runtime, and the search reads only the first
// of these for the files it holds. A specifier that is no built-in module, no package the lockfile holds as a
// dependency of a package of the family, and no file (one of the sources, or one that exists beside them under a
// package's `src`, or its `package.json`) can name a module through a map the search does not read, as an alias that
// begins with `#` does, so it is reported where it is written. A package of the family is the exception to the
// lockfile: the lockfile lists it for the packages that depend on it, so it names only the files of the sources, by
// its entry point, its `/ts` subpath and `/ts/<file>`; any other subpath of it is reported. A type-only import or
// export and a type are left out, since they load nothing. The lockfile and the files beside the sources are read
// once, when the module loads, since the check itself is not asynchronous; the lockfile is the one at the root of the
// repository, and its importers of the family give the packages.

/**
 Directory holding the family's packages.
 */
const FAMILY_ROOT = join(
  import.meta.dirname,
  '..',
  '..',
);

/**
 The lockfile of the repository.
 */
const LOCKFILE = join(
  FAMILY_ROOT,
  '..',
  '..',
  'pnpm-lock.yaml',
);

/**
 The start of the line that opens the block of a package of the family among the importers of the lockfile.
 */
const IMPORTER_START = '  package/translation-repair/';

/**
 The start of a line that holds the name of a dependency in an importer's block, which is indented by six spaces.
 */
const DEPENDENCY_START = '      ';

/**
 Where the reading of the lockfile stands: whether the lines read are in the block of a package of the family, and the
 names of the dependencies found.
 */
type Reading = {
  /**
   Whether the lines read now are in the block of a package of the family.
   */
  readonly isInFamily: boolean;

  /**
   Names found so far, scope included.
   */
  readonly names: readonly string[];
};

/**
 The name of the dependency a line of the lockfile holds.

 @param line - line of the lockfile

 @returns Name without quotes, empty for a line that is not a key at the depth of the dependencies

 @example
 ```ts
 const name = dependencyNameOf({ line: '      \'@scope/name\':', },);
 // => '@scope/name'
 ```
 */
function dependencyNameOf({ line, }: { readonly line: string; },): string {
  if (!line.startsWith(DEPENDENCY_START,))
    return '';
  if (line.startsWith(`${DEPENDENCY_START} `,))
    return '';
  if (!line.endsWith(':',))
    return '';
  /**
   The key with its colon taken off.
   */
  const key = line.slice(
    DEPENDENCY_START.length,
    -1,
  );
  return key.replaceAll(
    '\'',
    '',
  );
}

/**
 The reading after one more line of the lockfile.

 @param reading - reading so far

 @param line - next line

 @returns Reading: a line that starts at the top level leaves a block, a line indented by two spaces opens a block of a
 package of the family or leaves one, and a line at the depth of the dependencies in a block of the family names one

 @example
 ```ts
 const next = readLine({ reading: { isInFamily: true, names: [], }, line: '      zod:', },);
 // => { isInFamily: true, names: ['zod'] }
 ```
 */
function readLine(
  {
    reading,
    line,
  }: {
    readonly reading: Reading;
    readonly line: string;
  },
): Reading {
  if ((line !== '') && (!line.startsWith(' ',))) {
    return {
      isInFamily: false,
      names: reading.names,
    };
  }
  if (line.startsWith('  ',) && (!line.startsWith('   ',))) {
    return {
      isInFamily: line.startsWith(IMPORTER_START,),
      names: reading.names,
    };
  }
  /**
   The name the line holds, empty when it holds none or is outside the block of a package of the family.
   */
  const name = reading.isInFamily ? dependencyNameOf({ line, },) : '';
  return (name === '')
    ? reading
    : {
      isInFamily: true,
      names: [
        ...reading.names,
        name,
      ],
    };
}

/**
 The packages a lockfile holds as dependencies of the packages of the family.

 @param text - text of the lockfile

 @returns Names of the packages, scope included

 @example
 ```ts
 const packages = lockedPackagesIn({ text, },);
 // => Set { '@monochromatic-dev/or-throw-core', ... }
 ```
 */
function lockedPackagesIn({ text, }: { readonly text: string; },): ReadonlySet<string> {
  /**
   The lines of the lockfile.
   */
  const lines = text.split('\n',);
  /**
   The reading after the last line.
   */
  const finished = lines.reduce(
    function step(
      reading: Reading,
      line: string,
    ): Reading {
      return readLine({
        reading,
        line,
      },);
    },
    {
      isInFamily: false,
      names: [],
    } satisfies Reading,
  );
  return new Set(finished.names,);
}

/**
 The packages the lockfile holds as dependencies of the packages of the family, read when the module loads.
 */
const LOCKED_PACKAGES: ReadonlySet<string> = lockedPackagesIn({
  text: await readFile(
    LOCKFILE,
    'utf8',
  ),
},);

/**
 The files that exist beside the sources, by path under the family's root: every file under a package's `src` and each
 package's `package.json`, read when the module loads.
 */
const FILES_BESIDE_SOURCES: ReadonlySet<string> = new Set(
  (await Array.fromAsync(glob(
    [
      '*/src/**/*',
      '*/package.json',
    ],
    { cwd: FAMILY_ROOT, },
  ),))
    .filter(function isNotInstalled(file,): boolean {
      return !file.includes('node_modules',);
    },),
);

/**
 The name of the package a specifier names: the first segment, and the second for a scoped package.

 @param specifier - specifier as written

 @returns Package name, the specifier itself when it has one segment

 @example
 ```ts
 const name = packageNameOf({ specifier: '@monochromatic-dev/or-throw-core/ts', },);
 // => '@monochromatic-dev/or-throw-core'
 ```
 */
function packageNameOf({ specifier, }: { readonly specifier: string; },): string {
  /**
   The parts of the specifier between slashes.
   */
  const segments = specifier.split('/',);
  /**
   How many segments the name has.
   */
  const length = specifier.startsWith('@',) ? 2 : 1;
  return segments.slice(
    0,
    length,
  )
    .join('/',);
}

/**
 Whether the check can resolve a specifier.

 @param family - modules of the sources

 @param file - path of the module that writes the specifier

 @param specifier - specifier as written

 @returns Whether it is a file of the sources, a file that exists beside them, a built-in module or a package of the
 lockfile

 @example
 ```ts
 const isResolved = resolves({ family, file: 'a/src/a.ts', specifier: 'node:path', },);
 // => true
 ```
 */
function resolves(
  {
    family,
    file,
    specifier,
  }: {
    readonly family: ReadonlyMap<string, unknown>;
    readonly file: string;
    readonly specifier: string;
  },
): boolean {
  if (targetOf({
    family,
    from: file,
    specifier,
  },) !== '')
    return true;
  // The lockfile lists a package of the family for the others that depend on it, which says nothing of the files it
  // holds, so a specifier of such a package resolves only to a file of the sources.
  if (isFamilyPackage({ specifier, },))
    return false;
  if (specifier.startsWith('.',)) {
    /**
     The path the specifier names, from the root of the family.
     */
    const joined = joinPosix(
      dirname(file,),
      specifier,
    );
    return FILES_BESIDE_SOURCES.has(normalize(joined,),);
  }
  if (isBuiltin(specifier,))
    return true;
  return LOCKED_PACKAGES.has(packageNameOf({ specifier, },),);
}

/**
 The specifier a node writes and loads, when it loads a module.

 @param node - node read

 @returns Specifier as written, empty for a node that loads none, for a path that is no literal and for a type-only
 import or export

 @example
 ```ts
 const specifier = loadedSpecifierOf({ node, },);
 ```
 */
function loadedSpecifierOf({ node, }: { readonly node: TreeNode; },): string {
  if ((node.importKind === 'type') || (node.exportKind === 'type'))
    return '';
  if ((node.type === 'ImportDeclaration') || (node.type === 'ExportAllDeclaration')
    || (node.type === 'ExportNamedDeclaration')) {
    return stringValueOf({ node: node.source, },);
  }
  if (node.type === 'TSExternalModuleReference')
    return stringValueOf({ node: node.expression, },);
  if ((node.type !== 'ImportExpression') && (node.type !== 'CallExpression'))
    return '';
  return moduleLoadOf({ node, },)
    .specifier;
}

/**
 The places in a file that write a specifier the check cannot resolve.

 @param family - modules of the sources

 @param file - path of the file

 @param nodes - every node of the file

 @returns One entry per place, in the order of the nodes

 @example
 ```ts
 const found = unresolvedSpecifiersIn({ family, file: 'a/src/a.ts', nodes, },);
 ```
 */
export function unresolvedSpecifiersIn(
  {
    family,
    file,
    nodes,
  }: {
    readonly family: ReadonlyMap<string, unknown>;
    readonly file: string;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  return nodes.flatMap(function shapesOfNode(node,): readonly FoundShape[] {
    /**
     The specifier the node loads.
     */
    const specifier = loadedSpecifierOf({ node, },);
    if ((specifier === '') || resolves({
      family,
      file,
      specifier,
    },))
      return [];
    return [
      {
        shape: 'unresolved-module-specifier',
        node,
        position: `${specifier} is no built-in module, no package of the lockfile and no file`,
      },
    ];
  },);
}

//endregion Specifiers the check cannot resolve
