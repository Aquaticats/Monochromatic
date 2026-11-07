import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseCrateRegistry } from './crate-registry.ts';
import { assertDependenciesReachCratesIo } from './crates-io-dependency-check.ts';
import { CrateReleaseError } from './error.ts';

/**
 Repository root of the fixture.
 */
const root = '/repo';

/**
 A registry of three crates: the one released, a dependency and another crate.
 */
const registry = parseCrateRegistry(JSON.stringify({
  'app-cli': { crateDir: 'package/app/cli', scope: 'app/cli' },
  'dep-core': { crateDir: 'package/dep/core', scope: 'dep/core' },
  'dep-core-extra': { crateDir: 'package/dep/extra', scope: 'dep/extra' },
  'other-core': { crateDir: 'package/other/core', scope: 'other/core' },
}));

/**
 Manifest of a crate at a version, with the lines that follow its `[package]` table.

 @param version - version of the crate
 @param rest - text after the package table
 @returns manifest text
 */
function manifest({
  version,
  rest = '',
}: {
  readonly version: string;
  readonly rest?: string;
}): string {
  return `[package]\nname = "x"\nversion = "${version}"\n${rest}`;
}

/**
 Run the check against manifests held in memory.

 @param files - manifest text by repository-relative directory
 @param to - version being released
 @param crateDirectory - directory of the crate being released
 @returns the refusal message, or the empty text when the check passed
 */
async function refusalOf({
  files,
  to,
  crateDirectory = 'package/app/cli',
}: {
  readonly files: Readonly<Record<string, string>>;
  readonly to: string;
  readonly crateDirectory?: string;
}): Promise<string> {
  try {
    await assertDependenciesReachCratesIo({
      crateName: 'app-cli',
      crateDirectory,
      to,
      registry,
      root,
      readText: async function readText(path) {
        const found = Object.entries(files)
          .find(function isManifest([directory]) {
            return path === join(root, directory, 'Cargo.toml');
          });
        if (found === undefined)
          throw new Error(`the fixture has no ${path}`);
        return found[1];
      },
    });
    return '';
  }
  catch (error) {
    if (error instanceof CrateReleaseError)
      return error.message;
    throw error;
  }
}

/**
 Manifests of an app that depends on `dep-core` with an inline pin, and of the dependency at a version.

 @param dependencyVersion - version of the dependency's own manifest
 @param pin - exact pin the app holds
 @returns the fixture files
 */
function inlineFiles({
  dependencyVersion,
  pin = dependencyVersion,
}: {
  readonly dependencyVersion: string;
  readonly pin?: string;
}): Readonly<Record<string, string>> {
  return {
    'package/app/cli': manifest({
      version: '1.0.0',
      rest: `\n[dependencies]\ndep-core = { path = "../../dep/core", version = "=${pin}" }\n`,
    }),
    'package/dep/core': manifest({ version: dependencyVersion }),
    'package/dep/extra': manifest({ version: '0.1.0-alpha.1' }),
    'package/other/core': manifest({ version: '0.1.0-alpha.1' }),
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: assertDependenciesReachCratesIo.name,
      children: [
        it({
          name: 'refuses a release candidate whose dependency is at alpha, and names the dependency and its version',
          fn: async () => {
            const message = await refusalOf({
              files: inlineFiles({ dependencyVersion: '0.3.0-alpha.2' }),
              to: '1.0.0-rc.1',
            });
            expect(message).toContain('dep-core');
            expect(message).toContain('0.3.0-alpha.2');
            expect(message).toContain('app-cli');
            expect(message).toContain('1.0.0-rc.1');
            expect(message).toContain('package/dep/core/Cargo.toml');
          },
        }),
        it({
          name: 'refuses a plain number whose dependency is at beta',
          fn: async () => {
            const message = await refusalOf({
              files: inlineFiles({ dependencyVersion: '0.3.0-beta.4' }),
              to: '1.0.0',
            });
            expect(message).toContain('dep-core');
            expect(message).toContain('0.3.0-beta.4');
          },
        }),
        it({
          name: 'refuses when the exact pin is at alpha even though the dependency manifest is not',
          fn: async () => {
            const message = await refusalOf({
              files: inlineFiles({ dependencyVersion: '0.3.0', pin: '0.3.0-alpha.2' }),
              to: '1.0.0',
            });
            expect(message).toContain('dep-core');
            expect(message).toContain('0.3.0-alpha.2');
            expect(message).toContain('package/app/cli/Cargo.toml');
          },
        }),
        it({
          name: 'refuses a dependency written as a table of its own, a build dependency and a dev dependency with a version',
          fn: async () => {
            const messages = await Promise.all(['dependencies', 'build-dependencies', 'dev-dependencies'].map(async function refusalFor(header) {
              return await refusalOf({
                files: {
                  ...inlineFiles({ dependencyVersion: '0.3.0-alpha.2' }),
                  'package/app/cli': manifest({
                    version: '1.0.0',
                    rest: `\n[${header}.dep-core]\npath = "../../dep/core"\nversion = "=0.3.0-alpha.2"\n`,
                  }),
                },
                to: '1.0.0',
              });
            }));
            for (const message of messages)
              expect(message).toContain('dep-core');
          },
        }),
        it({
          name: 'refuses a dependency at a pre-release that has no alpha, beta or rc label, because crates.io never holds it',
          fn: async () => {
            const message = await refusalOf({
              files: inlineFiles({ dependencyVersion: '0.3.0-dev.1' }),
              to: '1.0.0',
            });
            expect(message).toContain('0.3.0-dev.1');
          },
        }),
        it({
          name: 'names every dependency that is at alpha or beta',
          fn: async () => {
            const message = await refusalOf({
              files: {
                ...inlineFiles({ dependencyVersion: '0.3.0-alpha.2' }),
                'package/app/cli': manifest({
                  version: '1.0.0',
                  rest: '\n[dependencies]\ndep-core = { path = "../../dep/core", version = "=0.3.0-alpha.2" }\n'
                    + 'other-core = { path = "../../other/core", version = "=0.1.0-alpha.1" }\n',
                }),
              },
              to: '1.0.0',
            });
            expect(message).toContain('dep-core');
            expect(message).toContain('other-core');
          },
        }),
        it({
          name: 'allows an alpha or a beta release whose dependency is at alpha, because neither goes to crates.io',
          fn: async () => {
            const messages = await Promise.all(['1.0.0-alpha.3', '1.0.0-beta.1'].map(async function refusalFor(to) {
              return await refusalOf({
                files: inlineFiles({ dependencyVersion: '0.3.0-alpha.2' }),
                to,
              });
            }));
            expect(messages).toEqual(['', '']);
          },
        }),
        it({
          name: 'allows a release candidate and a plain number whose dependencies are at rc or a plain number',
          fn: async () => {
            const messages = await Promise.all(['1.0.0-rc.1', '1.0.0'].flatMap(function casesOf(to) {
              return ['0.3.0-rc.2', '0.3.0'].map(async function refusalFor(dependencyVersion) {
                return await refusalOf({
                  files: inlineFiles({ dependencyVersion }),
                  to,
                });
              });
            }));
            expect(messages).toEqual(['', '', '', '']);
          },
        }),
        it({
          name: 'ignores a crate whose name only starts with the dependency name, and registry crates the manifest does not use',
          fn: async () => {
            const message = await refusalOf({
              files: {
                ...inlineFiles({ dependencyVersion: '0.3.0' }),
                'package/app/cli': manifest({
                  version: '1.0.0',
                  rest: '\n[dependencies]\ndep-core = { path = "../../dep/core", version = "=0.3.0" }\n'
                    + 'dep-core-extra-more = { version = "=0.1.0-alpha.1" }\n',
                }),
              },
              to: '1.0.0',
            });
            expect(message).toBe('');
          },
        }),
        it({
          name: 'ignores a dependency that carries no version, which a published manifest drops',
          fn: async () => {
            const message = await refusalOf({
              files: {
                ...inlineFiles({ dependencyVersion: '0.3.0-alpha.2' }),
                'package/app/cli': manifest({
                  version: '1.0.0',
                  rest: '\n[dev-dependencies]\ndep-core = { path = "../../dep/core" }\n',
                }),
              },
              to: '1.0.0',
            });
            expect(message).toBe('');
          },
        }),
        it({
          name: 'reads the crate manifest from the directory it is given, so a crate outside the registry is checked too',
          fn: async () => {
            const message = await refusalOf({
              files: {
                ...inlineFiles({ dependencyVersion: '0.3.0-alpha.2' }),
                'elsewhere/app': manifest({
                  version: '1.0.0',
                  rest: '\n[dependencies]\ndep-core = { path = "../../dep/core", version = "=0.3.0-alpha.2" }\n',
                }),
              },
              to: '1.0.0',
              crateDirectory: 'elsewhere/app',
            });
            expect(message).toContain('dep-core');
          },
        }),
      ],
    }),
  ],
});
