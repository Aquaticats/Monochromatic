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
              scopes: [`src/native/${module}.rs`],
              command,
            });
            expect(passOne.scope.mutatedModules).toEqual([module]);
            for (const killer of killers)
              expect(passOne.scope.libraryModules).toContain(killer);
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
          scopes: ['src/native/repo_read_layout.rs'],
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
          scopes: ['src/native/object_abbreviation.rs'],
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
