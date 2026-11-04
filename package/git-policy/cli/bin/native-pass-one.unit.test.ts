import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  passOneFor,
  slowTestTargets,
} from './native-pass-one.ts';

/**
 The crate under test: this package.
 */
const crateDirectory = join(
  import.meta.dirname,
  '..',
);

/**
 The campaign command the cases extend; its contents do not matter to the selection.
 */
const command = ['cargo', 'mutants', '--', '--', '--skip', 'x'];

/**
 Library modules whose tests killed the mutants of three files in the campaigns of 2026-11-02, read from each mutant's
 first failing test (`doc/planning/mutation-campaign-per-mutant-cost.md`), by the mutated file's module.
 Modules of the binary suite are left out: pass 1 does not run that suite.
 */
const measuredKillers: Readonly<Record<string, readonly string[]>> = {
  repo_read_query: [
    'repo_read_hook',
    'candidate_known_content',
    'commit_convergence',
    'config_snapshot',
    'repo_read_config_typed',
  ],
  object_abbreviation: ['object_abbreviation', 'landed_summary'],
  repo_read_layout: [
    'repo_read_layout',
    'git_system_config',
    'capture_invocation',
    'lfs_object_plan',
  ],
};

/**
 The source folder of each module above. The library's top-level modules are the folders, so pass 1 selects the tests
 of the mutated file's folder and of every folder that imports it, which contains the tests that killed the measured
 mutants.
 */
const folderOfModule: Readonly<Record<string, string>> = {
  repo_read_query: 'repo_read',
  repo_read_hook: 'repo_read',
  candidate_known_content: 'candidate',
  commit_convergence: 'commit',
  config_snapshot: 'config',
  repo_read_config_typed: 'repo_read',
  object_abbreviation: 'repo_read',
  landed_summary: 'landing',
  repo_read_layout: 'repo_read',
  git_system_config: 'repo_read',
  capture_invocation: 'commit',
  lfs_object_plan: 'lfs',
};

/**
 The folder of a module in the table above.

 @param module - module name

 @returns the folder that holds the module's file

 @example
 ```ts
 folderOf('repo_read_query');
 ```
 */
function folderOf(module: string): string {
  const folder = folderOfModule[module];
  if (folder === undefined)
    throw new Error(`No folder is listed for module ${module}.`);
  return folder;
}

await describe({
  name: passOneFor.name,
  children: [
    ...Object.entries(measuredKillers)
      .map(function killersCase([module, killers]) {
        return it({
          name: `selects the tests that killed the measured mutants of ${module}`,
          fn: async () => {
            /**
             Pass 1 of a campaign over the module's file.
             */
            const passOne = await passOneFor({
              crateDirectory,
              scopes: [`src/native/${folderOf(module)}/${module}.rs`],
              command,
            });
            expect(passOne.scope.mutatedModules).toEqual([folderOf(module)]);
            for (const killer of killers)
              expect(passOne.scope.libraryModules).toContain(folderOf(killer));
            expect(passOne.filterset).toContain('kind(lib) & test(/^(?:');
          },
          timeout: 60_000,
        });
      }),
    it({
      name: 'leaves out the binary suite, which exercises everything and is the slowest',
      fn: async () => {
        /**
         Pass 1 of a campaign over a module that most of the library reaches.
         */
        const passOne = await passOneFor({
          crateDirectory,
          scopes: ['src/native/repo_read/repo_read_layout.rs'],
          command,
        });
        for (const target of slowTestTargets) {
          expect(passOne.scope.testTargets).not.toContain(target);
          expect(passOne.filterset).not.toContain(`binary(=${target})`);
        }
      },
      timeout: 60_000,
    }),
    it({
      name: 'puts the test selection into the campaign command and leaves the rest of it alone',
      fn: async () => {
        /**
         Pass 1 of a campaign over a small module.
         */
        const passOne = await passOneFor({
          crateDirectory,
          scopes: ['src/native/repo_read/object_abbreviation.rs'],
          command,
        });
        expect(passOne.command.slice(0, 2)).toEqual(['cargo', 'mutants']);
        expect(passOne.command.slice(-4)).toEqual(['--', '--', '--skip', 'x']);
        expect(passOne.command).toContain(`--cargo-test-arg=--filterset=${passOne.filterset}`);
      },
      timeout: 60_000,
    }),
    it({
      name: 'refuses a campaign without a file, a glob scope, and a file the library does not declare',
      fn: async () => {
        await expect(passOneFor({
          crateDirectory,
          scopes: [],
          command,
        })).rejects.toThrow(CargoMutantsToolingError);
        await expect(passOneFor({
          crateDirectory,
          scopes: ['src/native/repo_read_*.rs'],
          command,
        })).rejects.toThrow('literal --file paths');
        await expect(passOneFor({
          crateDirectory,
          scopes: ['src/native/no_such_module.rs'],
          command,
        })).rejects.toThrow('found no library module');
      },
      timeout: 60_000,
    }),
  ],
});
