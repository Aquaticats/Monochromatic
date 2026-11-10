import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join, resolve } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findCrate,
  loadCrateRegistry,
  parseCrateRegistry,
  registryCrateNames,
} from './crate-registry.ts';
import {
  manifestFileName,
  registryFileName,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 A valid registry with a full entry and a minimal one.
 */
const validRegistry = {
  'monochromatic-git-policy-cli': {
    crateDir: 'package/git-policy/cli',
    scope: 'git-policy/cli',
    pinFiles: ['package/git-policy/cli.fuzz/Cargo.toml'],
    checkBins: ['monochromatic-git-policy-cli'],
    lockDirs: ['package/git-policy/cli.fuzz'],
  },
  'plain-crate': {
    crateDir: 'package/plain/cli',
    scope: 'plain/cli',
  },
};

/**
 Run a parse that must throw and return what it threw.

 @param text - registry text
 @returns the thrown value
 */
function thrownBy(text: string): unknown {
  try {
    parseCrateRegistry(text);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Wait for a promise that must reject and return what it rejected with.

 @param promise - promise under test
 @returns the rejection, or undefined when it resolved
 */
async function rejectionOf(promise: Promise<unknown>): Promise<unknown> {
  try {
    await promise;
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Expect a registry text to be refused with a message that holds the given text.

 @param registry - registry value, written as JSON
 @param mention - text the message must hold
 */
function expectRefusal({
  registry,
  mention,
}: {
  readonly registry: unknown;
  readonly mention: string;
}): void {
  const caught = thrownBy(JSON.stringify(registry));
  expect(caught).toBeInstanceOf(CrateReleaseError);
  expect((caught as Error).message).toContain(mention);
}

await describe({
  name: '',
  children: [
    describe({
      name: parseCrateRegistry.name,
      children: [
        it({
          name: 'reads every field of an entry and keeps the crates in file order',
          fn: async () => {
            const registry = parseCrateRegistry(JSON.stringify(validRegistry));
            expect(registryCrateNames(registry)).toEqual(['monochromatic-git-policy-cli', 'plain-crate']);
            expect(registry.get('monochromatic-git-policy-cli')).toEqual({
              crateDirectory: 'package/git-policy/cli',
              scope: 'git-policy/cli',
              pinFiles: ['package/git-policy/cli.fuzz/Cargo.toml'],
              checkBins: ['monochromatic-git-policy-cli'],
              lockDirectories: ['package/git-policy/cli.fuzz'],
            });
          },
        }),
        it({
          name: 'reads the dispatch gate of an entry that names one, as the command and its arguments',
          fn: async () => {
            const registry = parseCrateRegistry(JSON.stringify({
              gated: {
                crateDir: 'package/gated/cli',
                scope: 'gated/cli',
                dispatchGate: [
                  'node',
                  'bin/host-controls.ts',
                ],
              },
            }));
            expect(registry.get('gated')?.dispatchGate).toEqual({
              program: 'node',
              args: ['bin/host-controls.ts'],
            });
          },
        }),
        it({
          name: 'leaves the dispatch gate off an entry that names none, so its releases keep the checks they had',
          fn: async () => {
            expect(parseCrateRegistry(JSON.stringify(validRegistry)).get('plain-crate')?.dispatchGate).toBeUndefined();
          },
        }),
        it({
          name: 'refuses a dispatch gate that is not a list of text',
          fn: async () => {
            expectRefusal({
              registry: {
                gated: {
                  crateDir: 'package/gated/cli',
                  scope: 'gated/cli',
                  dispatchGate: 'node bin/host-controls.ts',
                },
              },
              mention: 'dispatchGate',
            });
          },
        }),
        it({
          name: 'refuses a dispatch gate that is empty or holds an empty word, which would wait for nothing',
          fn: async () => {
            expectRefusal({
              registry: {
                gated: {
                  crateDir: 'package/gated/cli',
                  scope: 'gated/cli',
                  dispatchGate: [],
                },
              },
              mention: 'dispatchGate',
            });
            expectRefusal({
              registry: {
                gated: {
                  crateDir: 'package/gated/cli',
                  scope: 'gated/cli',
                  dispatchGate: ['node', ''],
                },
              },
              mention: 'dispatchGate',
            });
          },
        }),
        it({
          name: 'reads an entry with only the required fields as one without pins, binaries or lock directories',
          fn: async () => {
            expect(parseCrateRegistry(JSON.stringify(validRegistry)).get('plain-crate')).toEqual({
              crateDirectory: 'package/plain/cli',
              scope: 'plain/cli',
              pinFiles: [],
              checkBins: [],
              lockDirectories: [],
            });
          },
        }),
        it({
          name: 'refuses text that is not JSON and says so',
          fn: async () => {
            const caught = thrownBy('{"a":');
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain(`${registryFileName} is not JSON`);
          },
        }),
        it({
          name: 'refuses a registry that is not an object of crates',
          fn: async () => {
            expectRefusal({ registry: [], mention: 'must be an object of crates by name' });
            expectRefusal({ registry: 'x', mention: 'must be an object of crates by name' });
            expectRefusal({ registry: null, mention: 'must be an object of crates by name' });
          },
        }),
        it({
          name: 'refuses a registry that lists no crate',
          fn: async () => {
            expectRefusal({ registry: {}, mention: 'lists no crate' });
          },
        }),
        it({
          name: 'refuses a crate name that is not a crate name',
          fn: async () => {
            expectRefusal({ registry: { 'a b': { crateDir: 'p', scope: 's' } }, mention: 'crate name' });
          },
        }),
        it({
          name: 'refuses an entry that is not an object, naming the crate',
          fn: async () => {
            expectRefusal({ registry: { a: 'p' }, mention: 'crate "a" must be an object' });
          },
        }),
        it({
          name: 'refuses a missing or non-text crateDir and scope, naming the field',
          fn: async () => {
            expectRefusal({ registry: { a: { scope: 's' } }, mention: 'needs "crateDir" as text' });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 5 } }, mention: 'needs "scope" as text' });
          },
        }),
        it({
          name: 'refuses a field the registry does not define and lists the fields it does',
          fn: async () => {
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', lockTasks: ['x'] } }, mention: '["lockTasks"]' });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', lockTasks: ['x'] } }, mention: 'lockDirs' });
          },
        }),
        it({
          name: 'refuses a list field that is not a list of text',
          fn: async () => {
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', pinFiles: 'x' } }, mention: '"pinFiles" as a list of text' });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', checkBins: [1] } }, mention: '"checkBins" as a list of text' });
          },
        }),
        it({
          name: 'refuses paths that leave the repository or are not manifests, and binary names with shell syntax',
          fn: async () => {
            expectRefusal({ registry: { a: { crateDir: '../x', scope: 's' } }, mention: '"crateDir"' });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', pinFiles: ['p/Other.toml'] } }, mention: manifestFileName });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', lockDirs: ['/abs'] } }, mention: '"lockDirs"' });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 's', checkBins: ['a;b'] } }, mention: '"checkBins"' });
            expectRefusal({ registry: { a: { crateDir: 'p', scope: 'a):b' } }, mention: '"scope"' });
          },
        }),
      ],
    }),
    describe({
      name: findCrate.name,
      children: [
        it({
          name: 'returns the entry of a listed crate',
          fn: async () => {
            const registry = parseCrateRegistry(JSON.stringify(validRegistry));
            expect(findCrate({ registry, name: 'plain-crate' }).scope).toBe('plain/cli');
          },
        }),
        it({
          name: 'names the crates the registry lists and the file to add the crate to when it is not listed',
          fn: async () => {
            const registry = parseCrateRegistry(JSON.stringify(validRegistry));
            let caught: unknown;
            try {
              findCrate({ registry, name: 'nope' });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('"nope"');
            expect((caught as Error).message).toContain('monochromatic-git-policy-cli, plain-crate');
            expect((caught as Error).message).toContain(registryFileName);
          },
        }),
      ],
    }),
    describe({
      name: loadCrateRegistry.name,
      children: [
        it({
          name: 'reads the registry file that sits in the package directory',
          fn: async () => {
            const paths: string[] = [];
            const registry = await loadCrateRegistry({
              packageDirectory: '/pkg',
              readText: async function readText(path) {
                paths.push(path);
                return JSON.stringify(validRegistry);
              },
            });
            expect(paths).toEqual([join('/pkg', registryFileName)]);
            expect(registryCrateNames(registry)).toEqual(['monochromatic-git-policy-cli', 'plain-crate']);
          },
        }),
        it({
          name: 'turns a file that cannot be read into a refusal that names the path',
          fn: async () => {
            const caught = await rejectionOf(loadCrateRegistry({
              packageDirectory: '/pkg',
              readText: async function readText(): Promise<never> {
                throw new Error('ENOENT');
              },
            }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain(`${join('/pkg', registryFileName)} could not be read`);
            expect((caught as Error).message).toContain('ENOENT');
          },
        }),
        it({
          name: 'passes the refusal of an invalid registry through unchanged',
          fn: async () => {
            const caught = await rejectionOf(loadCrateRegistry({
              packageDirectory: '/pkg',
              readText: async function readText() {
                return '{}';
              },
            }));
            expect((caught as Error).message).toBe(`${registryFileName} lists no crate.`);
          },
        }),
        it({
          name: 'accepts the registry file this package ships and lists the crates the release tasks name',
          fn: async () => {
            const registry = await loadCrateRegistry({
              packageDirectory: resolve(import.meta.dirname, '..'),
              readText: async function readText(path) {
                return await readFile(path, 'utf8');
              },
            });
            // The release tasks are the mise tasks that run the dispatch command with `--crate <name>`; the
            // registry lists exactly the crates they name.
            const repositoryRoot = resolve(import.meta.dirname, '..', '..', '..', '..');
            const taskFiles = await Array.fromAsync(glob('package/**/mise.toml', {
              cwd: repositoryRoot,
              exclude: ['**/node_modules/**'],
            }));
            const marker = 'dispatch-release.ts --crate ';
            const named = (await Promise.all(taskFiles.map(async function releaseTaskCrates(path) {
              const text = await readFile(join(repositoryRoot, path), 'utf8');
              return text.split('\n').flatMap(function crateOfLine(line) {
                // A line without the marker splits into one part, so the text after it is absent.
                const [, afterMarker] = line.split(marker);
                if (afterMarker === undefined)
                  return [];
                const crate = afterMarker.split('"')[0]?.split(' ')[0] ?? '';
                return crate === '' ? [] : [crate];
              });
            }))).flat();
            expect(named.length).toBeGreaterThan(0);
            expect(registryCrateNames(registry).toSorted()).toEqual([...new Set(named)].toSorted());
            expect(findCrate({ registry, name: 'forbidden-strings' }).pinFiles).toEqual([
              'package/git-policy/cli/Cargo.toml',
              'package/forbidden-strings/cli.fuzz/Cargo.toml',
              'package/forbidden-strings/core.bench/Cargo.toml',
            ]);
          },
        }),
      ],
    }),
  ],
});
