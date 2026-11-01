/**
 Classification of a single import specifier against its owning package.
 
 Only the owning package's own code is judged. Specifiers naming another
 workspace package are left alone, including that package's `/ts` subpath,
 because importing a sibling package's TypeScript source through `/ts` is the
 sanctioned cross-package channel.
 
 @module
 */

import { isFixtureModule, } from './checked-file.ts';
import type { OwningPackage, } from './owning-package.ts';
import {
  isUnderAnyDirectory,
  resolvePosix,
} from './posix-path.ts';

/**
 Subpath of a package's own name that serves its TypeScript source.
 */
const SOURCE_SUBPATH = '/ts';

/**
 Prefix of deeper source subpaths such as `/ts/backend/api.ts`.
 */
const SOURCE_SUBPATH_PREFIX = '/ts/';

/**
 Verdict for one import specifier.
 
 @internal
 */
export type ImportOutcome =
  /**
   Target is the package's eventual artifact, or a test-only fixture or helper.
   */
  | 'allowed'
  /**
   Relative specifier reaching package source rather than built output.
   */
    | 'relative-source'
  /**
   Bare specifier reaching the package's own source through its `/ts` subpath.
   */
    | 'own-source-subpath'
  /**
   Specifier naming another package or a runtime module; outside this rule.
   */
    | 'unchecked';

/**
 Classifies one import specifier appearing in a checked file.
 
 The owning package's bare name is allowed because it resolves through the
 exports map to a shipped entry, which is the one import form that exercises
 the export map itself.
 
 @param specifier - literal specifier text from the import declaration
 
 @param containingDirectory - directory of the file holding the import
 
 @param owner - package that owns the file holding the import
 
 @param fixturePatterns - configured fixture globs
 
 @returns verdict for this specifier
 
 @example
 ```ts
 classifyImport({ specifier: './toml-set.ts', containingDirectory, owner, fixturePatterns });
 ```
 
 @internal
 */
export function classifyImport({
  specifier,
  containingDirectory,
  owner,
  fixturePatterns,
}: {
  /**
   Literal specifier text from the import declaration.
   */
  readonly specifier: string;
  /**
   Directory of the file holding the import.
   */
  readonly containingDirectory: string;
  /**
   Package that owns the file holding the import.
   */
  readonly owner: OwningPackage;
  /**
   Configured fixture globs.
   */
  readonly fixturePatterns: readonly string[];
},): ImportOutcome {
  if (specifier.startsWith('.',))
    return classifyRelative({
      specifier,
      containingDirectory,
      owner,
      fixturePatterns,
    },);

  if (specifier === owner.name)
    return 'allowed';

  if ((specifier === `${owner.name}${SOURCE_SUBPATH}`)
    || specifier.startsWith(`${owner.name}${SOURCE_SUBPATH_PREFIX}`,))
    {
    return 'own-source-subpath';
  }

  return 'unchecked';
}

/**
 Classifies a relative specifier by where it lands.
 
 Resolution is lexical, so a specifier naming a not-yet-built artifact
 classifies the same before and after a build.
 
 @param specifier - relative specifier text
 
 @param containingDirectory - directory of the file holding the import
 
 @param owner - package that owns the file holding the import
 
 @param fixturePatterns - configured fixture globs
 
 @returns verdict for this specifier
 */
function classifyRelative({
  specifier,
  containingDirectory,
  owner,
  fixturePatterns,
}: {
  /**
   Relative specifier text.
   */
  readonly specifier: string;
  /**
   Directory of the file holding the import.
   */
  readonly containingDirectory: string;
  /**
   Package that owns the file holding the import.
   */
  readonly owner: OwningPackage;
  /**
   Configured fixture globs.
   */
  readonly fixturePatterns: readonly string[];
},): ImportOutcome {
  /**
   Import target resolved against the importing file's directory.
   */
  const target = resolvePosix({
    base: containingDirectory,
    specifier,
  },);

  if (isUnderAnyDirectory({
    directories: owner.artifactDirectories,
    path: target,
  },))
    return 'allowed';

  if (isFixtureModule({
    patterns: fixturePatterns,
    path: target,
  },))
    return 'allowed';

  return 'relative-source';
}
