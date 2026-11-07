import {
  join,
  posix,
} from 'node:path';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import {
  exactPinPrefix,
  manifestFileName,
} from './constants.ts';
import {
  type CrateRegistry,
  findCrate,
  registryCrateNames,
} from './crate-registry.ts';
import { goesToCratesIo } from './crates-io-version-label.ts';
import {
  inlinePinSites,
  type PinSite,
  tablePinSites,
} from './dependency-pin-sites.ts';
import { CrateReleaseError } from './error.ts';
import { readPackageVersion } from './package-version.ts';
import { splitManifest } from './toml-line.ts';

/**
 A dependency of the released crate that crates.io will not hold, with where its versions were read.
 */
type UnpublishedDependency = {
  readonly name: string;
  readonly manifestPath: string;
  readonly version: string;
  readonly pins: readonly string[];
};

/**
 Find the pin sites of a crate in a manifest, whichever of the two written forms holds them.

 A dependency that carries no quoted version is a path dependency that a published manifest drops, so it has no site
 and crates.io is never asked for it.

 @param bodies - manifest line bodies

 @param crateName - crate the dependency is on

 @returns the sites of the inline form and of the table form, with a form that has no version left out

 @example
 ```ts
 dependencySites({ bodies: ['a = { version = "=1.0.0" }'], crateName: 'a' });
 ```
 */
function dependencySites({
  bodies,
  crateName,
}: {
  readonly bodies: readonly string[];
  readonly crateName: string;
}): readonly PinSite[] {
  return [
    inlinePinSites,
    tablePinSites
  ].flatMap(function sitesOf(find): readonly PinSite[] {
    try {
      return find({
        bodies,
        crateName
      });
    }
    catch (error) {
      if (error instanceof CrateReleaseError)
        return [];
      throw error;
    }
  });
}

/**
 Read a manifest, naming it when it cannot be read.

 @param path - absolute path

 @param readText - reads a file by absolute path

 @returns the manifest text

 @throws CrateReleaseError when the file cannot be read

 @example
 ```ts
 await readManifest({ path: '/r/package/a/Cargo.toml', readText });
 ```
 */
async function readManifest({
  path,
  readText,
}: {
  readonly path: string;
  readonly readText: (path: string) => Promise<string>;
}): Promise<string> {
  try {
    return await readText(path);
  }
  catch (error) {
    throw new CrateReleaseError(
      `Cannot read ${path} to check the dependencies of the release: ${caughtValueText(error)}`,
      { cause: error },
    );
  }
}

/**
 Refuse a release that crates.io would take while a dependency of the crate is one it never holds.

 crates.io refuses a crate that uses a crate it does not hold, and it holds a version only when the label rule sends
 it there: a plain number or `-rc.N`. So a release that goes to crates.io needs every other crate of the repository
 that it depends on to be at such a version, in the dependency's own manifest and in the exact pin of the crate's
 manifest. A release that stays on GitHub (`-alpha.N`, `-beta.N`, or a form the rule does not name) has no such need.
 A dev dependency that carries a version counts, because a published manifest keeps it; a dependency without a version
 does not, because the published manifest drops it.

 @param crateName - crate being released

 @param crateDirectory - repository-relative directory of the crate, with its manifest

 @param to - version being released

 @param registry - every releasable crate, which are all the crates of the repository that crates.io could hold

 @param root - absolute repository root

 @param readText - reads a file by absolute path

 @throws CrateReleaseError naming each dependency, its version and the manifests that hold it, when the release goes to
 crates.io and a dependency does not

 @example
 ```ts
 await assertDependenciesReachCratesIo({ crateName: 'a', crateDirectory: 'package/a', to: '1.0.0', registry, root: '/r', readText });
 ```
 */
export async function assertDependenciesReachCratesIo({
  crateName,
  crateDirectory,
  to,
  registry,
  root,
  readText,
}: {
  readonly crateName: string;
  readonly crateDirectory: string;
  readonly to: string;
  readonly registry: CrateRegistry;
  readonly root: string;
  readonly readText: (path: string) => Promise<string>;
}): Promise<void> {
  if (!goesToCratesIo(to))
    return;
  /**
   Repository-relative path of the manifest of the released crate.
   */
  const cratePath = posix.join(
    crateDirectory,
    manifestFileName
  );
  /**
   Lines of that manifest.
   */
  const { bodies } = splitManifest(await readManifest({
    path: join(
      root,
      ...cratePath.split(posix.sep)
    ),
    readText
  }));
  /**
   Dependencies of the crate that are at a version crates.io never holds, one entry per crate, in registry order.
   */
  const unpublished = (await Promise.all(registryCrateNames(registry)
    .filter(function isOtherCrate(name) {
      return name !== crateName;
    })
    .map(async function readDependency(name): Promise<UnpublishedDependency[]> {
      /**
       The registry entry of the crate, which holds its directory.
       */
      const entry = findCrate({
        registry,
        name
      });
      /**
       The exact pins the crate's manifest holds for the dependency.
       */
      const pins = dependencySites({
        bodies,
        crateName: name
      })
        .map(function pinOf(site) {
          return site.location
            .value
            .startsWith(exactPinPrefix) ? site.location
              .value
              .slice(exactPinPrefix.length) : site.location
                .value;
        });
      if (pins.length === 0)
        return [];
      /**
       Repository-relative path of the dependency's own manifest.
       */
      const manifestPath = posix.join(
        entry.crateDirectory,
        manifestFileName
      );
      /**
       The version the dependency's manifest holds.
       */
      const own = readPackageVersion(await readManifest({
        path: join(
          root,
          ...manifestPath.split(posix.sep)
        ),
        readText
      }));
      if (!own.found)
        throw new CrateReleaseError(`${manifestPath}: ${own.reason}`);
      /**
       The versions among the manifest's own and the pins that crates.io does not hold, first the dependency's own.
       */
      const refused = [
        own.value,
        ...pins
      ].filter(function neverOnCratesIo(version) {
        return !goesToCratesIo(version);
      });
      return refused.length === 0 ? [] : [
        {
          name,
          manifestPath,
          version: own.value,
          pins
        }
      ];
    }))).flat();
  if (unpublished.length === 0)
    return;
  throw new CrateReleaseError(
    `${crateName} ${to} would go to crates.io, which refuses a crate that uses a crate it does not hold, and these `
      + `dependencies are at a version that never goes to crates.io (only a plain number and -rc.N do):\n${
     unpublished.map(function describeDependency({
      name,
      manifestPath,
      version,
      pins,
    }) {
      return `- ${name}: ${manifestPath} holds ${version}, and ${cratePath} pins ${pins.join(', ')}`;
    })
       .join('\n')
     }\nRelease the dependencies as -rc.N or a plain number first, or release ${crateName} as -alpha.N or -beta.N, `
      + `which stays on GitHub.`,
  );
}
