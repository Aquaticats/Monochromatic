/**
 * Creates a performance benchmark fixture at /tmp/file-enforcer-perf/
 * with ~240 files across 20 simulated packages, including 6-level deep
 * directory nesting for testing glob traversal performance.
 *
 * Files are small (100-3000 bytes each): no large file testing since
 * managing files over 1MB is considered a user error for file-enforcer.
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  generateDocContent,
  generateSettingsJson,
  generateTsContent,
} from './setup-fixture-generators.ts';

/** Root directory for all fixture files, respects $TMPDIR for sandbox compatibility */
const FIXTURE_DIR = join(scratchRoot(), 'file-enforcer-perf',);

/** Number of simulated packages to create */
const PACKAGE_COUNT = 20;

/** Document file basenames inside each package's doc/ directory */
const DOC_NAMES = ['readme', 'guide', 'api', 'changelog', 'contributing',] as const;

/** TypeScript file basenames inside each package's lib/ directory */
const LIB_NAMES = ['index', 'utils', 'helpers',] as const;

/**
 * Writes a single documentation file for a package.
 *
 * @param pkgDir - Absolute path to the package directory
 *
 * @param pkgIndex - Package index (0-19)
 *
 * @param docName - Document basename
 */
function writeDocFile(pkgDir: string, pkgIndex: number, docName: string,): Promise<void> {
  return writeFile(join(pkgDir, 'docs', `${docName}.md`,),
    generateDocContent(pkgIndex, docName,),);
}

/**
 * Writes a single library TypeScript file for a package.
 *
 * @param pkgDir - Absolute path to the package directory
 *
 * @param pkgIndex - Package index (0-19)
 *
 * @param libName - Library file basename
 */
function writeLibFile(pkgDir: string, pkgIndex: number, libName: string,): Promise<void> {
  return writeFile(join(pkgDir, 'lib', `${libName}.ts`,),
    generateTsContent(pkgIndex, libName,),);
}

/**
 * Creates all files for a single simulated package.
 *
 * @param pkgIndex - Package index (0-19)
 */
async function createPackage(pkgIndex: number,): Promise<void> {
  const pkgName = `pkg-${String(pkgIndex,).padStart(2, '0',)}`;
  const pkgDir = join(FIXTURE_DIR, 'src', pkgName,);

  // Create directory structure including 6-level deep nesting
  await Promise.all([
    mkdir(join(pkgDir, 'config',), { recursive: true, },),
    mkdir(join(pkgDir, 'docs',), { recursive: true, },),
    mkdir(join(pkgDir, 'lib', 'deep', 'nested', 'very', 'deep',), { recursive: true, },),
    mkdir(join(pkgDir, 'types',), { recursive: true, },),
  ],);

  // Write all files for this package in parallel
  await Promise.all([
    writeFile(join(pkgDir, 'config', 'settings.json',), generateSettingsJson(pkgIndex,),),
    ...DOC_NAMES.map(function writeDoc(docName,) {
      return writeDocFile(pkgDir, pkgIndex, docName,);
    },),
    ...LIB_NAMES.map(function writeLib(libName,) {
      return writeLibFile(pkgDir, pkgIndex, libName,);
    },),
    writeFile(
      join(pkgDir, 'lib', 'deep', 'nested', 'very', 'deep', 'module.ts',),
      generateTsContent(pkgIndex, 'deep-module',),
    ),
    writeFile(
      join(pkgDir, 'types', 'index.d.ts',),
      `export type Pkg${
        String(pkgIndex,).padStart(2, '0',)
      }Config = { readonly name: string; };`,
    ),
    writeFile(
      join(pkgDir, 'types', 'models.d.ts',),
      `export type Pkg${
        String(pkgIndex,).padStart(2, '0',)
      }Model = { readonly id: number; };`,
    ),
  ],);
}

/**
 * Creates a package for a given array index.
 *
 * @param _unused - Unused array element placeholder
 *
 * @param index - Package index (0-19)
 */
function createPackageByIndex(_unused: unknown, index: number,): Promise<void> {
  return createPackage(index,);
}

// Clean any previous fixture
await rm(FIXTURE_DIR, { recursive: true, force: true, },);

// Create all packages in parallel
await Promise.all(
  Array.from({ length: PACKAGE_COUNT, }, createPackageByIndex,),
);

// Create empty dest directory
await mkdir(join(FIXTURE_DIR, 'dest',), { recursive: true, },);

/** Total files: 20 packages * (1 json + 5 md + 3 ts + 1 deep ts + 2 d.ts) = 240 */
const TOTAL_FILES = PACKAGE_COUNT * (1 + DOC_NAMES.length + LIB_NAMES.length + 1 + 2);
console.log(
  `[perf-fixture] created ${String(TOTAL_FILES,)} files across ${
    String(PACKAGE_COUNT,)
  } packages at ${FIXTURE_DIR}`,
);
