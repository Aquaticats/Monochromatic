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
  systemdRunProgram,
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
  unitName: 'cli-git-publish-dry',
};

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
            expect(commitPaths({ ...gitPolicyOptions, pinFiles: [] })).toEqual([
              `package/git-policy/cli/${manifestFileName}`,
              `package/git-policy/cli/${lockFileName}`,
            ]);
          },
        }),
        it({
          name: 'lists a path once when a pin file is the crate manifest',
          fn: async () => {
            expect(commitPaths({ ...gitPolicyOptions, pinFiles: [`package/git-policy/cli/${manifestFileName}`] })).toEqual([
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
            })).toEqual([
              '--user',
              '--unit=cli-git-publish-dry',
              '--collect',
              '--wait',
              '--pipe',
              '--working-directory=/repo/package/git-policy/cli',
              '--property=MemoryMax=12G',
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
            const args = buildPublishDryRunArguments({ unitName: 'u', workingDirectory: `/r/a b'c;d`, memoryMax: '1G' });
            expect(args).toContain(`--working-directory=/r/a b'c;d`);
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
      ],
    }),
  ],
});
