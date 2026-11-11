import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialLines } from './adversarial-fixture.ts';
import { buildCommitMessage } from './commit-message.ts';
import {
  cargoProgram,
  commitType,
  gitProgram,
  lockFileName,
  manifestFileName,
  maximumUnitIoWeight,
  systemdRunProgram,
  unitIoWeight,
} from './constants.ts';
import type { ReleaseOptions } from './release-options.ts';
import {
  buildCheckArguments,
  buildCommitArguments,
  buildLockArguments,
  buildPublishDryRunArguments,
  buildPublishDryRunTail,
  buildReleaseSteps,
  commitPaths,
  lockFileBeside,
  releasePlanNotes,
  releaseRunsCargoCheck,
} from './release-steps.ts';

/**
 Options of the crate the original scripts released.
 */
const gitPolicyOptions: ReleaseOptions = {
  root: '/repo',
  crateName: 'monochromatic-git-policy-cli',
  crateDirectory: 'package/git-policy/cli',
  scope: 'git-policy/cli',
  from: '0.4.2',
  to: '0.4.3',
  note: 'fix the thing',
  pinFiles: ['package/git-policy/cli.fuzz/Cargo.toml'],
  checkBins: ['monochromatic-git-policy-cli'],
  lockDirectories: ['package/git-policy/cli.fuzz'],
  host: 'local',
  memoryMax: '12G',
  cpuQuota: '1600%',
  tasksMax: '512',
  unitName: 'cli-git-publish-dry',
};

/**
 Highest `IOWeight` a per-command cgroup may carry: the owner's rule in `CLAUDE.md` (RXS) says never above 10.
 The test states it itself so a change to the implementation's own constant cannot move it.
 */
const ownerMaximumIoWeight = 10;

/**
 Values a unit's arguments give one `--property` name, in order.

 @param args - arguments after `systemd-run`
 @param property - systemd property name such as `TasksMax`
 @returns the text after `--property=<name>=` of every argument that sets the property
 */
function propertyValues({ args, property }: { readonly args: readonly string[]; readonly property: string }): readonly string[] {
  const prefix = `--property=${property}=`;
  return args
    .filter((argument) => argument.startsWith(prefix))
    .map((argument) => argument.slice(prefix.length));
}

await describe({
  name: '',
  children: [
    describe({
      name: lockFileBeside.name,
      children: [
        it({
          name: 'puts the lockfile in the directory of the manifest',
          fn: async () => {
            expect(lockFileBeside(`package/a/b/${manifestFileName}`)).toBe(`package/a/b/${lockFileName}`);
          },
        }),
        it({
          name: 'handles a manifest at the repository root',
          fn: async () => {
            expect(lockFileBeside(manifestFileName)).toBe(lockFileName);
          },
        }),
      ],
    }),
    describe({
      name: commitPaths.name,
      children: [
        it({
          name: 'lists the manifest and lockfile of the crate and of each pin file, in that order',
          fn: async () => {
            expect(commitPaths(gitPolicyOptions)).toEqual([
              `package/git-policy/cli/${manifestFileName}`,
              `package/git-policy/cli/${lockFileName}`,
              `package/git-policy/cli.fuzz/${manifestFileName}`,
              `package/git-policy/cli.fuzz/${lockFileName}`,
            ]);
          },
        }),
        it({
          name: 'lists only the crate files when there is no pin file',
          fn: async () => {
            expect(commitPaths({ ...gitPolicyOptions, pinFiles: [], lockDirectories: [] })).toEqual([
              `package/git-policy/cli/${manifestFileName}`,
              `package/git-policy/cli/${lockFileName}`,
            ]);
          },
        }),
        it({
          name: 'adds the lockfile of each lock directory that no pin file sits in, after the pin files, because the refresh changes it',
          fn: async () => {
            expect(commitPaths({
              ...gitPolicyOptions,
              lockDirectories: ['package/file-enforcer/cli', 'package/git-policy/cli.fuzz', 'package/git-policy/cli'],
            })).toEqual([
              `package/git-policy/cli/${manifestFileName}`,
              `package/git-policy/cli/${lockFileName}`,
              `package/git-policy/cli.fuzz/${manifestFileName}`,
              `package/git-policy/cli.fuzz/${lockFileName}`,
              `package/file-enforcer/cli/${lockFileName}`,
            ]);
          },
        }),
        it({
          name: 'names a lock directory\'s lockfile only, never its manifest, which a lock refresh does not change',
          fn: async () => {
            const paths = commitPaths({ ...gitPolicyOptions, pinFiles: [], lockDirectories: ['package/a/b'] });
            expect(paths).toContain(`package/a/b/${lockFileName}`);
            expect(paths).not.toContain(`package/a/b/${manifestFileName}`);
          },
        }),
        it({
          name: 'lists a path once when a pin file is the crate manifest',
          fn: async () => {
            expect(commitPaths({
              ...gitPolicyOptions,
              pinFiles: [`package/git-policy/cli/${manifestFileName}`],
              lockDirectories: [],
            })).toEqual([
              `package/git-policy/cli/${manifestFileName}`,
              `package/git-policy/cli/${lockFileName}`,
            ]);
          },
        }),
      ],
    }),
    describe({
      name: buildCheckArguments.name,
      children: [
        it({
          name: 'checks the named binary offline on a local host',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: ['monochromatic-git-policy-cli'], host: 'local' })).toEqual([
              'check',
              '--offline',
              '--bin',
              'monochromatic-git-policy-cli',
            ]);
          },
        }),
        it({
          name: 'builds each named binary with its own --bin',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: ['a', 'b'], host: 'local' })).toEqual([
              'check',
              '--offline',
              '--bin',
              'a',
              '--bin',
              'b',
            ]);
          },
        }),
        it({
          name: 'checks the whole crate offline when no binary is named',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: [], host: 'local' })).toEqual(['check', '--offline']);
          },
        }),
        it({
          name: 'leaves cargo free to use the network on a runner, whose registry cache is empty',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: ['a'], host: 'runner' })).toEqual(['check', '--bin', 'a']);
            expect(buildCheckArguments({ checkBins: [], host: 'runner' })).toEqual(['check']);
          },
        }),
      ],
    }),
    describe({
      name: buildLockArguments.name,
      children: [
        it({
          name: 'records only the workspace members, offline on a local host',
          fn: async () => {
            expect(buildLockArguments('local')).toEqual(['update', '--offline', '--workspace']);
          },
        }),
        it({
          name: 'records only the workspace members, with the network open on a runner',
          fn: async () => {
            expect(buildLockArguments('runner')).toEqual(['update', '--workspace']);
          },
        }),
      ],
    }),
    describe({
      name: buildPublishDryRunTail.name,
      children: [
        it({
          name: 'is the locked dry-run publish that allows the edited manifests',
          fn: async () => {
            expect(buildPublishDryRunTail()).toEqual(['publish', '--dry-run', '--locked', '--allow-dirty']);
          },
        }),
      ],
    }),
    describe({
      name: buildPublishDryRunArguments.name,
      children: [
        it({
          name: 'runs the locked dry-run publish in a capped waited unit',
          fn: async () => {
            expect(buildPublishDryRunArguments({
              unitName: 'cli-git-publish-dry',
              workingDirectory: '/repo/package/git-policy/cli',
              memoryMax: '12G',
              cpuQuota: '800%',
              tasksMax: '2048',
            })).toEqual([
              '--user',
              '--unit=cli-git-publish-dry',
              '--collect',
              '--wait',
              '--pipe',
              '--working-directory=/repo/package/git-policy/cli',
              '--property=MemoryMax=12G',
              '--property=CPUQuota=800%',
              '--property=TasksMax=2048',
              `--property=IOWeight=${String(unitIoWeight)}`,
              cargoProgram,
              'publish',
              '--dry-run',
              '--locked',
              '--allow-dirty',
            ]);
          },
        }),
        it({
          name: 'keeps a working directory with spaces and quotes as one argument',
          fn: async () => {
            const args = buildPublishDryRunArguments({
              unitName: 'u',
              workingDirectory: `/r/a b'c;d`,
              memoryMax: '1G',
              cpuQuota: '100%',
              tasksMax: '64',
            });
            expect(args).toContain(`--working-directory=/r/a b'c;d`);
          },
        }),
        it({
          name: 'sets memory, CPU, task and disk-weight limits on the unit, one of each',
          fn: async () => {
            const args = buildPublishDryRunArguments({
              unitName: 'u',
              workingDirectory: '/r/c',
              memoryMax: '3G',
              cpuQuota: '700%',
              tasksMax: '77',
            });
            expect(propertyValues({ args, property: 'MemoryMax' })).toEqual(['3G']);
            expect(propertyValues({ args, property: 'CPUQuota' })).toEqual(['700%']);
            expect(propertyValues({ args, property: 'TasksMax' })).toEqual(['77']);
            expect(propertyValues({ args, property: 'IOWeight' })).toHaveLength(1);
          },
        }),
        it({
          name: 'never sets a disk weight above the owner limit',
          fn: async () => {
            const args = buildPublishDryRunArguments({
              unitName: 'u',
              workingDirectory: '/r/c',
              memoryMax: '3G',
              cpuQuota: '700%',
              tasksMax: '77',
            });
            const [weight] = propertyValues({ args, property: 'IOWeight' });
            expect(Number(weight)).toBeGreaterThan(0);
            expect(Number(weight)).toBeLessThanOrEqual(ownerMaximumIoWeight);
            expect(maximumUnitIoWeight).toBeLessThanOrEqual(ownerMaximumIoWeight);
            expect(unitIoWeight).toBeLessThanOrEqual(maximumUnitIoWeight);
          },
        }),
      ],
    }),
    describe({
      name: buildCommitArguments.name,
      children: [
        it({
          name: 'commits exactly the named paths with the message',
          fn: async () => {
            expect(buildCommitArguments({ message: 'm', paths: ['a', 'b'] })).toEqual([
              'commit',
              '--message',
              'm',
              '--',
              'a',
              'b',
            ]);
          },
        }),
        ...adversarialLines.map(function noteCase(note) {
          return it({
            name: `passes the message holding ${JSON.stringify(note)} as exactly one argument`,
            fn: async () => {
              const message = buildCommitMessage({ scope: 's', version: '1.0.0', note });
              const args = buildCommitArguments({ message, paths: ['a'] });
              expect(args).toEqual(['commit', '--message', message, '--', 'a']);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildReleaseSteps.name,
      children: [
        it({
          name: 'gives the local publish dry run all four limits and the runner none of them',
          fn: async () => {
            const local = buildReleaseSteps(gitPolicyOptions)
              .find((step) => step.label === 'publish dry run');
            expect(local?.program).toBe(systemdRunProgram);
            for (const property of ['MemoryMax', 'CPUQuota', 'TasksMax', 'IOWeight'])
              expect(propertyValues({ args: local?.args ?? [], property })).toHaveLength(1);
            expect(propertyValues({ args: local?.args ?? [], property: 'CPUQuota' })).toEqual([gitPolicyOptions.cpuQuota]);
            expect(propertyValues({ args: local?.args ?? [], property: 'TasksMax' })).toEqual([gitPolicyOptions.tasksMax]);
            const runner = buildReleaseSteps({ ...gitPolicyOptions, host: 'runner' })
              .find((step) => step.label === 'publish dry run');
            expect(runner?.program).toBe(cargoProgram);
          },
        }),
        it({
          name: 'plans the four steps of a release in order with their working directories',
          fn: async () => {
            expect(buildReleaseSteps(gitPolicyOptions)).toEqual([
              {
                label: 'cargo check',
                program: cargoProgram,
                args: ['check', '--offline', '--bin', 'monochromatic-git-policy-cli'],
                cwd: '/repo/package/git-policy/cli',
              },
              {
                label: 'lock members',
                program: cargoProgram,
                args: ['update', '--offline', '--workspace'],
                cwd: '/repo/package/git-policy/cli.fuzz',
              },
              {
                label: 'publish dry run',
                program: systemdRunProgram,
                args: [
                  '--user',
                  '--unit=cli-git-publish-dry',
                  '--collect',
                  '--wait',
                  '--pipe',
                  '--working-directory=/repo/package/git-policy/cli',
                  '--property=MemoryMax=12G',
                  `--property=CPUQuota=${gitPolicyOptions.cpuQuota}`,
                  `--property=TasksMax=${gitPolicyOptions.tasksMax}`,
                  `--property=IOWeight=${String(unitIoWeight)}`,
                  cargoProgram,
                  'publish',
                  '--dry-run',
                  '--locked',
                  '--allow-dirty',
                ],
                cwd: '/repo/package/git-policy/cli',
              },
              {
                label: 'commit',
                program: gitProgram,
                args: [
                  'commit',
                  '--message',
                  `${commitType}(git-policy/cli): release 0.4.3: fix the thing`,
                  '--',
                  ...commitPaths(gitPolicyOptions),
                ],
                cwd: '/repo',
              },
            ]);
          },
        }),
        it({
          name: 'plans the steps of a runner with the network open and the dry run in no unit',
          fn: async () => {
            expect(buildReleaseSteps({ ...gitPolicyOptions, host: 'runner' })).toEqual([
              {
                label: 'cargo check',
                program: cargoProgram,
                args: ['check', '--bin', 'monochromatic-git-policy-cli'],
                cwd: '/repo/package/git-policy/cli',
              },
              {
                label: 'lock members',
                program: cargoProgram,
                args: ['update', '--workspace'],
                cwd: '/repo/package/git-policy/cli.fuzz',
              },
              {
                label: 'publish dry run',
                program: cargoProgram,
                args: ['publish', '--dry-run', '--locked', '--allow-dirty'],
                cwd: '/repo/package/git-policy/cli',
              },
              {
                label: 'commit',
                program: gitProgram,
                args: [
                  'commit',
                  '--message',
                  `${commitType}(git-policy/cli): release 0.4.3: fix the thing`,
                  '--',
                  ...commitPaths(gitPolicyOptions),
                ],
                cwd: '/repo',
              },
            ]);
          },
        }),
        it({
          name: 'never starts mise or a systemd unit on a runner, whatever the unit options say',
          fn: async () => {
            const programs = buildReleaseSteps({ ...gitPolicyOptions, host: 'runner', memoryMax: '1G', unitName: 'u' })
              .map(function programOf(step) {
                return step.program;
              });
            expect(programs).toEqual([cargoProgram, cargoProgram, cargoProgram, gitProgram]);
          },
        }),
        it({
          name: 'plans one lock step per lock directory, in its own directory, and none when there is no directory',
          fn: async () => {
            const two = buildReleaseSteps({ ...gitPolicyOptions, lockDirectories: ['package/a', 'package/b c'] });
            expect(two.filter(function isLock(step) {
              return step.label === 'lock members';
            }).map(function directoryOf(step) {
              return step.cwd;
            })).toEqual(['/repo/package/a', '/repo/package/b c']);
            const none = buildReleaseSteps({ ...gitPolicyOptions, lockDirectories: [] });
            expect(none.map(function labelOf(step) {
              return step.label;
            })).toEqual(['cargo check', 'publish dry run', 'commit']);
          },
        }),
        it({
          name: 'uses the root as the working directory of the commit whatever the crate directory',
          fn: async () => {
            const steps = buildReleaseSteps({ ...gitPolicyOptions, root: '/elsewhere' });
            expect(steps.at(-1)?.cwd).toBe('/elsewhere');
            expect(steps[0]?.cwd).toBe('/elsewhere/package/git-policy/cli');
          },
        }),
        it({
          name: 'plans no publish dry run for a version that never goes to crates.io, on either host',
          fn: async () => {
            for (const host of ['local', 'runner'] as const)
              for (const to of ['0.4.3-beta.2', '0.4.3-dev.1']) {
                const labels = buildReleaseSteps({ ...gitPolicyOptions, host, to })
                  .map(function labelOf(step) {
                    return step.label;
                  });
                expect(labels).toEqual(['cargo check', 'lock members', 'commit']);
              }
          },
        }),
        it({
          name: 'plans the publish dry run for a release candidate and a plain number, on either host',
          fn: async () => {
            for (const host of ['local', 'runner'] as const)
              for (const to of ['0.4.3-rc.1', '0.4.3']) {
                const labels = buildReleaseSteps({ ...gitPolicyOptions, host, to })
                  .map(function labelOf(step) {
                    return step.label;
                  });
                expect(labels).toEqual(['cargo check', 'lock members', 'publish dry run', 'commit']);
              }
          },
        }),
        it({
          name: 'plans no cargo check for an alpha on either host, and lands its version in the lockfile instead',
          fn: async () => {
            for (const host of ['local', 'runner'] as const) {
              const steps = buildReleaseSteps({
                ...gitPolicyOptions,
                host,
                to: '0.4.3-alpha.2'
              });
              expect(steps.map(function labelOf(step) {
                return step.label;
              })).toEqual(['lock the crate', 'lock members', 'commit']);
              // The crate's own refresh runs in the crate directory, with the lock command of this host.
              expect(steps[0]?.cwd).toBe('/repo/package/git-policy/cli');
              expect(steps[0]?.args).toEqual(buildLockArguments(host));
              // The commit still names the path the check used to build, so an alpha's commit is the same commit.
              expect(steps.at(-1)?.args).toContain('package/git-policy/cli/Cargo.toml');
            }
          },
        }),
        it({
          name: 'records the new version in the crate\'s own lockfile before the commit, whatever the label',
          fn: async () => {
            for (const to of ['0.4.3-alpha.2', '0.4.3-beta.2', '0.4.3-rc.1', '0.4.3', '0.4.3-dev.1']) {
              const steps = buildReleaseSteps({
                ...gitPolicyOptions,
                host: 'runner',
                to
              });
              // The first step in the crate directory either checks the crate, which rewrites Cargo.lock as a side
              // effect, or refreshes the lockfile on purpose; either way the pushed commit's locked build resolves.
              expect(steps[0]?.cwd).toBe('/repo/package/git-policy/cli');
              expect(steps[0]?.program).toBe(cargoProgram);
              expect(steps[0]?.label).toBe(releaseRunsCargoCheck(to) ? 'cargo check' : 'lock the crate');
            }
          },
        }),
        it({
          name: 'says in a note what an alpha leaves out and why, so the log never looks like a whole plan',
          fn: async () => {
            const notes = releasePlanNotes({
              ...gitPolicyOptions,
              to: '0.4.3-alpha.2'
            });
            expect(notes).toHaveLength(2);
            expect(notes[0]).toContain('no cargo check');
            expect(notes[0]).toContain('alpha');
            expect(notes[0]).toContain('cargo-publish.yml');
            expect(notes[0]).toContain('lockfile is still refreshed');
            expect(notes[1]).toContain('no publish dry run');
            expect(notes[1]).toContain('the alpha label is not published to crates.io');
          },
        }),
        it({
          name: 'names only the step the label rule leaves out, and nothing at all for a release that runs the whole plan',
          fn: async () => {
            expect(releasePlanNotes({
              ...gitPolicyOptions,
              to: '0.4.3-beta.2'
            })).toEqual(['no publish dry run: the beta label is not published to crates.io (only a plain number and -rc.N are)']);
            expect(releasePlanNotes({
              ...gitPolicyOptions,
              to: '0.4.3'
            })).toEqual([]);
          },
        }),
        it({
          name: 'keeps the compile before the commit for every release that is not an alpha',
          fn: async () => {
            for (const to of ['0.4.3', '0.4.3-rc.1', '0.4.3-beta.2', '0.4.3-dev.1']) {
              expect(releaseRunsCargoCheck(to)).toBe(true);
              expect(buildReleaseSteps({
                ...gitPolicyOptions,
                to
              })[0]?.label).toBe('cargo check');
            }
            expect(releaseRunsCargoCheck('0.4.3-alpha.2')).toBe(false);
          },
        }),
      ],
    }),
  ],
});
