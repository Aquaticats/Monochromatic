/**
 Unit tests for the rule that a repository-published tool is never replaced by an older version:
 the semver order, the repository-published signal, and the outcome classification.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isNewerVersion,
  isRepositoryPublished,
  keptNewerWarning,
  lockOutcome,
  NOT_INSTALLED,
  toolById,
  type ToolPlanEntry,
} from '../../dist/final/node/index.mjs';

/**
 Builds a plan entry shaped like the repository's `cargo-binstall` entries.

 @param id - Plan entry id

 @param packageId - Crate the entry installs

 @param version - Version the lock pins

 @param route - Install route

 @returns A plan entry
 */
function cargoEntry(
  {
    id = 'cargo:monochromatic-fixture',
    packageId = 'monochromatic-fixture',
    version = '0.4.0',
    route = 'cargo-binstall',
  }: {
    readonly id?: string;
    readonly packageId?: string;
    readonly version?: string;
    readonly route?: ToolPlanEntry['route'];
  },
): ToolPlanEntry {
  return {
    id,
    request: { track: 'latest', },
    version,
    route,
    manager: 'cargo',
    packageId,
    bins: [packageId],
    options: {},
    env: {},
    artifacts: {},
  };
}

await describe({
  name: 'repository tools are never downgraded',
  children: [
    describe({
      name: isNewerVersion.name,
      children: [
        it({
          name: 'orders major, minor and patch numerically, not as text',
          fn: async () => {
            expect(isNewerVersion({
              candidate: '0.4.4',
              than: '0.4.0',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '0.10.0',
              than: '0.9.9',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '2.0.0',
              than: '10.0.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '0.3.0',
              than: '0.4.0',
            },),)
              .toBe(false,);
          },
        },),
        it({
          name: 'is false for equal versions, a leading v and build metadata',
          fn: async () => {
            expect(isNewerVersion({
              candidate: '0.4.0',
              than: '0.4.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: 'v0.4.0',
              than: '0.4.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '0.4.0+build.7',
              than: '0.4.0',
            },),)
              .toBe(false,);
          },
        },),
        it({
          name: 'puts a release above its pre-release and orders pre-release identifiers by semver rule 11',
          fn: async () => {
            expect(isNewerVersion({
              candidate: '1.0.0',
              than: '1.0.0-rc.1',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '1.0.0-rc.1',
              than: '1.0.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '1.0.0-rc.2',
              than: '1.0.0-rc.1',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '1.0.0-rc.10',
              than: '1.0.0-rc.9',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '1.0.0-rc',
              than: '1.0.0-alpha',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '1.0.0-alpha.1',
              than: '1.0.0-alpha',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '1.0.0-alpha',
              than: '1.0.0-1',
            },),)
              .toBe(true,);
            expect(isNewerVersion({
              candidate: '1.0.0-1',
              than: '1.0.0-alpha',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '0.5.0-rc.1',
              than: '0.4.9',
            },),)
              .toBe(true,);
          },
        },),
        it({
          name: 'is false when either side is not a semantic version, so those fall back to equality',
          fn: async () => {
            expect(isNewerVersion({
              candidate: 'nightly-2026-10-21',
              than: '0.4.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '0.4.0',
              than: 'ref:85e3eb7',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '1.2',
              than: '1.1.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '1.2.x',
              than: '1.1.0',
            },),)
              .toBe(false,);
            expect(isNewerVersion({
              candidate: '1.2.3-',
              than: '1.1.0',
            },),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isRepositoryPublished.name,
      children: [
        it({
          name: 'recognizes the repository crates in the real plan by their name prefix',
          fn: async () => {
            /**
             Repository crates in the plan.
             */
            const lint = toolById('cargo:monochromatic-lint',);
            const gitPolicy = toolById('cargo:monochromatic-git-policy-cli',);
            expect(isRepositoryPublished(lint,),)
              .toBe(true,);
            expect(isRepositoryPublished(gitPolicy,),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects third-party crates and non-cargo routes, even with the prefix in the name',
          fn: async () => {
            /**
             Entries that are not repository crates installed through the shared cargo route.
             */
            const nextest = toolById('cargo:cargo-nextest',);
            const node = toolById('node',);
            const gitBuilt = cargoEntry({ route: 'cargo-git', },);
            const lookalike = cargoEntry({
              id: 'cargo:fixture',
              packageId: 'not-monochromatic-fixture',
            },);
            expect(isRepositoryPublished(nextest,),)
              .toBe(false,);
            expect(isRepositoryPublished(node,),)
              .toBe(false,);
            expect(isRepositoryPublished(gitBuilt,),)
              .toBe(false,);
            expect(isRepositoryPublished(lookalike,),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: lockOutcome.name,
      children: [
        it({
          name: 'keeps a repository tool installed newer than the lock',
          fn: async () => {
            expect(lockOutcome({
              entry: cargoEntry({ version: '0.4.0', },),
              observed: '0.4.4',
            },),)
              .toBe('kept-newer',);
          },
        },),
        it({
          name: 'replaces a repository tool installed older than the lock',
          fn: async () => {
            expect(lockOutcome({
              entry: cargoEntry({ version: '0.4.4', },),
              observed: '0.4.0',
            },),)
              .toBe('replace',);
          },
        },),
        it({
          name: 'replaces a third-party tool installed newer than the lock, as before',
          fn: async () => {
            expect(lockOutcome({
              entry: cargoEntry({
                id: 'cargo:fixture-crate',
                packageId: 'fixture-crate',
                version: '0.4.0',
              },),
              observed: '0.4.4',
            },),)
              .toBe('replace',);
          },
        },),
        it({
          name: 'leaves equal versions at the lock and reports an absent tool as absent',
          fn: async () => {
            expect(lockOutcome({
              entry: cargoEntry({ version: '0.4.4', },),
              observed: '0.4.4',
            },),)
              .toBe('at-lock',);
            expect(lockOutcome({
              entry: cargoEntry({ version: '0.4.4', },),
              observed: NOT_INSTALLED,
            },),)
              .toBe('absent',);
          },
        },),
        it({
          name: 'replaces a repository tool whose installed version is not a semantic version',
          fn: async () => {
            expect(lockOutcome({
              entry: cargoEntry({ version: '0.4.0', },),
              observed: 'unknown-build',
            },),)
              .toBe('replace',);
          },
        },),
      ],
    },),
    describe({
      name: keptNewerWarning.name,
      children: [
        it({
          name: 'names the tool, both versions, why the newer one runs, and the remedy',
          fn: async () => {
            /**
             Warning for lint installed at 0.4.4 under a lock that pins 0.4.0.
             */
            const warning = keptNewerWarning({
              id: 'cargo:monochromatic-lint',
              observed: '0.4.4',
              locked: '0.4.0',
            },);
            expect(warning,)
              .toContain('cargo:monochromatic-lint',);
            expect(warning,)
              .toContain('pins 0.4.0',);
            expect(warning,)
              .toContain('runs 0.4.4',);
            expect(warning,)
              .toContain('older repository tools are never kept',);
            expect(warning,)
              .toContain('mise run toolchain:update',);
            expect(warning,)
              .toContain('merging main',);
          },
        },),
      ],
    },),
  ],
},);
