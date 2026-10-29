import {
  dirname,
  join,
  posix,
  relative,
  resolve,
  sep,
} from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  presentNamesOfKind,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  BundleMaps,
} from './coverage-bundle-maps.ts';
import { requireCoverageBuild, } from './coverage-census-build.ts';
import type { ReadText, } from './coverage-census-read-text.ts';

//region Coverage census family
// Ledger T8: the census measures the family of packages the translation-repair
// package was split into, not the one package it stands in. A test in one
// package runs code another package built, so the code no test ran can only be
// told by reading every package's coverage build and running every package's
// unit suite under one coverage directory.
//
// A MEMBER is a directory beside the census's own that holds a coverage build
// config (`rolldown.coverage.config.ts`) and so has a coverage build to read,
// or a task file (`mise.toml`) and so has a unit suite to run. A package with
// a suite and no build, such as the source scans, adds coverage of the code it
// loads but has no bundle of its own.
//
// Bundles are named by their path from the family directory, as
// `<member>/dist/final/node/<bundle>`, and sources are named from the family
// directory too, as `<member>/src/<file>`.

/**
 Name of the file whose presence makes a directory a member with a coverage
 build.
 */
const COVERAGE_CONFIG = 'rolldown.coverage.config.ts';

/**
 Name of the file whose presence makes a directory a member with a unit suite.
 */
const TASK_FILE = 'mise.toml';

/**
 Where a member's bundles stand inside the member.
 */
const BUNDLE_DIRECTORY = 'dist/final/node';

/**
 Index of the file name in a bundle's path from the family directory, after
 the member and the three directories of the build.
 */
const BUNDLE_DIRECTORY_END = 4;

/**
 Segments of a bundle's path from the family directory: member, three
 directories and the file.
 */
const FAMILY_BUNDLE_SEGMENTS = 5;

/**
 Whether a script of a coverage file is a bundle of a member's build, asked of
 its path from the family directory: not a source file a test loaded from a
 member's `src`, which has no map and no bundle.

 @param bundle - script's path from the family directory

 @returns Whether it is `<member>/dist/final/node/<file>`

 @example
 ```ts
 isFamilyBundleName('stage/dist/final/node/index.mjs',); // true
 ```
 */
export function isFamilyBundleName(bundle: string,): boolean {
  /**
   The path's segments.
   */
  const segments = bundle.split('/',);
  return (segments.length === FAMILY_BUNDLE_SEGMENTS)
    && (segments[0] !== '')
    && (segments.slice(
      1,
      BUNDLE_DIRECTORY_END,
    )
      .join('/',)
      === BUNDLE_DIRECTORY)
    && (segments[BUNDLE_DIRECTORY_END] !== '');
}

/**
 The family the census measures.

 @example
 ```ts
 const family: CensusFamily = { directory: '/repo/package/translation-repair', builds: ['stage',], suites: ['stage', 'source-scan',], };
 ```
 */
export type CensusFamily = {
  /**
   Directory holding every member.
   */
  readonly directory: string;

  /**
   Members with a coverage build, in code-point order.
   */
  readonly builds: readonly string[];

  /**
   Members with a unit suite, in code-point order.
   */
  readonly suites: readonly string[];
};

/**
 One member's suite run.
 */
export type FamilySuiteRun = {
  /**
   Member whose suite it is.
   */
  readonly member: string;

  /**
   Directory the suite runs in.
   */
  readonly cwd: string;

  /**
   Absolute paths of the test files to run, none for the whole suite.
   */
  readonly files: readonly string[];
};

/**
 What the census measures: the family of packages a package stands in, or the
 package alone.

 @example
 ```ts
 const scope: CensusScope = { kind: 'alone', };
 ```
 */
export type CensusScope =
  | {
    /**
     The package and the packages beside it.
     */
    readonly kind: 'family';

    /**
     The family.
     */
    readonly family: CensusFamily;
  }
    | {
    /**
     The package alone, which has no coverage build config.
     */
      readonly kind: 'alone';
    };

/**
 Finds what the census measures from the directory it runs in.

 @param packageDirectory - directory the census runs from

 @returns Family of the packages beside it where the package has a
 coverage build config, else the directory alone

 @example
 ```ts
 const scope = await censusScopeOf({ packageDirectory: process.cwd(), },);
 ```
 */
export async function censusScopeOf(
  { packageDirectory, }: { readonly packageDirectory: string; },
): Promise<CensusScope> {
  if (!(await presentNamesOfKind({
    dir: packageDirectory,
    kind: 'file',
  },)).includes(COVERAGE_CONFIG,))
    return { kind: 'alone', };
  /**
   Directory holding the package and the members beside it.
   */
  const directory = dirname(packageDirectory,);
  /**
   Every directory beside the package, itself included, in code-point order.
   */
  const siblings = textsInCodePointOrder({ texts: await presentNamesOfKind({
    dir: directory,
    kind: 'directory',
  },), },);
  /**
   The files each sibling holds, in the same order.
   */
  const held = await allInInputOrder({
    members: siblings.map(function filesOf(sibling,): Promise<readonly string[]> {
      return presentNamesOfKind({
        dir: join(
          directory,
          sibling,
        ),
        kind: 'file',
      },);
    },),
  },);
  /**
   Siblings holding a file, in the order of the siblings.

   @param name - file a member holds

   @returns Their names
   */
  function holding(name: string,): readonly string[] {
    return siblings.filter(function has(
      _sibling,
      index,
    ): boolean {
      return held[index]
        ?.includes(name,)
        === true;
    },);
  }
  return {
    kind: 'family',
    family: {
      directory,
      builds: holding(COVERAGE_CONFIG,),
      suites: holding(TASK_FILE,),
    },
  };
}

/**
 Reads every member's coverage build and refuses one the census cannot read,
 before any suite runs.

 @param family - family

 @param l - logger the progress lines go to

 @param readText - reads one bundle's text, passed in so a case scripts it

 @returns Every member's bundles, named by their path from the family
 directory, split by whether a map stands beside each

 @throws StatedRefusalError where a member's build directory cannot be listed,
 holds no bundle, holds none with a map beside it, or holds a minified build,
 naming that member's build directory

 @example
 ```ts
 const bundleMaps = await requireFamilyCoverageBuild({ family, l, readText: readUtf8Text, },);
 ```
 */
export async function requireFamilyCoverageBuild(
  {
    family,
    l,
    readText,
  }: {
    readonly family: CensusFamily;
    readonly l: Logger;
    readonly readText: ReadText;
  },
): Promise<BundleMaps> {
  /**
   Each member's bundles, in the order of the members.
   */
  const memberMaps = await allInInputOrder({
    members: family.builds
      .map(async function readMember(member,): Promise<BundleMaps> {
      /**
       Names a member's bundle by its path from the family directory.

       @param bundle - bundle in the member's build directory

       @returns Path
       */
        function namedFromFamily(bundle: string,): string {
          return posix.join(
            member,
            'dist',
            'final',
            'node',
            bundle,
          );
        }
      /**
       The member's bundles, named inside its build directory.
       */
        const inside = await requireCoverageBuild({
          distDirectory: join(
            family.directory,
            member,
            'dist',
            'final',
            'node',
          ),
          l,
          readText,
        },);
        return {
          mapped: inside.mapped
            .map(namedFromFamily,),
          unmapped: inside.unmapped
            .map(namedFromFamily,),
        };
      },),
  },);
  return {
    mapped: textsInCodePointOrder({ texts: memberMaps.flatMap(function mappedOf({ mapped, },): readonly string[] {
      return mapped;
    },), },),
    unmapped: textsInCodePointOrder({ texts: memberMaps.flatMap(function unmappedOf({ unmapped, },): readonly string[] {
      return unmapped;
    },), },),
  };
}

/**
 Groups the test files a command line named by the member whose suite runs
 them, or lists every member's whole suite where none is named.

 @param family - family

 @param testFiles - test files named, each by absolute path or from the family
 directory

 @returns One run per member, in the family's order, each with the absolute
 paths of its files

 @throws StatedRefusalError where a file does not stand in a member with a unit suite

 @example
 ```ts
 const runs = familySuiteRunsOf({ family, testFiles: ['stage/src/nap.unit.test.ts',], },);
 ```
 */
export function familySuiteRunsOf(
  {
    family,
    testFiles,
  }: {
    readonly family: CensusFamily;
    readonly testFiles: readonly string[];
  },
): readonly FamilySuiteRun[] {
  if (testFiles.length === 0) {
    return family.suites
      .map(function wholeSuite(member,): FamilySuiteRun {
        return {
          member,
          cwd: join(
            family.directory,
            member,
          ),
          files: [],
        };
      },);
  }
  /**
   Each file named, absolute, beside the member it stands in.
   */
  const placed = testFiles.map(function placedFile(file,): {
    readonly member: string;
    readonly absolute: string;
  } {
    /**
     The file by absolute path.
     */
    const absolute = resolve(
      family.directory,
      file,
    );
    /**
     The member directory the file stands in, if it stands in one.
     */
    const member = relative(
      family.directory,
      absolute,
    )
      .split(sep,)[0]
      ?? '';
    if (!family.suites
      .includes(member,))
      throw new StatedRefusalError({
        says: `${absolute} is not a test file of a package beside this one; name it from the family directory `
          + `(${family.directory}), as <package>/src/<file>.unit.test.ts, or by its absolute path`,
      },);
    return {
      member,
      absolute,
    };
  },);
  return family.suites
    .flatMap(function runOf(member,): readonly FamilySuiteRun[] {
    /**
     Files named for this member, in the order named.
     */
      const files = placed
        .filter(function standsIn(file,): boolean {
          return file.member === member;
        },)
        .map(function absoluteOf({ absolute, },): string {
          return absolute;
        },);
      return (files.length === 0)
        ? []
        : [{
          member,
          cwd: join(
            family.directory,
            member,
          ),
          files,
        },];
    },);
}

//endregion Coverage census family
