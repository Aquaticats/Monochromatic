import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialLines } from './adversarial-fixture.ts';
import { buildCommitMessage } from './commit-message.ts';
import type { ReleaseOptions } from './release-options.ts';
import {
  buildCheckArguments,
  buildCommitArguments,
  buildPublishDryRunArguments,
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
  checkBins: ['cli-git-native'],
  lockTasks: ['//package/git-policy/cli.fuzz:lock:members'],
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
            expect(lockFileBeside('package/a/b/Cargo.toml')).toBe('package/a/b/Cargo.lock');
          },
        }),
        it({
          name: 'handles a manifest at the repository root',
          fn: async () => {
            expect(lockFileBeside('Cargo.toml')).toBe('Cargo.lock');
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
              'package/git-policy/cli/Cargo.toml',
              'package/git-policy/cli/Cargo.lock',
              'package/git-policy/cli.fuzz/Cargo.toml',
              'package/git-policy/cli.fuzz/Cargo.lock',
            ]);
          },
        }),
        it({
          name: 'lists only the crate files when there is no pin file',
          fn: async () => {
            expect(commitPaths({ ...gitPolicyOptions, pinFiles: [] })).toEqual([
              'package/git-policy/cli/Cargo.toml',
              'package/git-policy/cli/Cargo.lock',
            ]);
          },
        }),
        it({
          name: 'lists a path once when a pin file is the crate manifest',
          fn: async () => {
            expect(commitPaths({ ...gitPolicyOptions, pinFiles: ['package/git-policy/cli/Cargo.toml'] })).toEqual([
              'package/git-policy/cli/Cargo.toml',
              'package/git-policy/cli/Cargo.lock',
            ]);
          },
        }),
      ],
    }),
    describe({
      name: buildCheckArguments.name,
      children: [
        it({
          name: 'checks the named binary offline',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: ['cli-git-native'] })).toEqual([
              'check',
              '--offline',
              '--bin',
              'cli-git-native',
            ]);
          },
        }),
        it({
          name: 'builds each named binary with its own --bin',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: ['a', 'b'] })).toEqual(['check', '--offline', '--bin', 'a', '--bin', 'b']);
          },
        }),
        it({
          name: 'checks the whole crate offline when no binary is named',
          fn: async () => {
            expect(buildCheckArguments({ checkBins: [] })).toEqual(['check', '--offline']);
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
              'cargo',
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
                program: 'cargo',
                args: ['check', '--offline', '--bin', 'cli-git-native'],
                cwd: '/repo/package/git-policy/cli',
              },
              {
                label: 'lock members',
                program: 'mise',
                args: ['run', '//package/git-policy/cli.fuzz:lock:members'],
                cwd: '/repo',
              },
              {
                label: 'publish dry run',
                program: 'systemd-run',
                args: [
                  '--user',
                  '--unit=cli-git-publish-dry',
                  '--collect',
                  '--wait',
                  '--pipe',
                  '--working-directory=/repo/package/git-policy/cli',
                  '--property=MemoryMax=12G',
                  'cargo',
                  'publish',
                  '--dry-run',
                  '--locked',
                  '--allow-dirty',
                ],
                cwd: '/repo/package/git-policy/cli',
              },
              {
                label: 'commit',
                program: 'git',
                args: [
                  'commit',
                  '--message',
                  'chore(git-policy/cli): release 0.4.3: fix the thing',
                  '--',
                  'package/git-policy/cli/Cargo.toml',
                  'package/git-policy/cli/Cargo.lock',
                  'package/git-policy/cli.fuzz/Cargo.toml',
                  'package/git-policy/cli.fuzz/Cargo.lock',
                ],
                cwd: '/repo',
              },
            ]);
          },
        }),
        it({
          name: 'plans one lock step per lock task and none when there is no task',
          fn: async () => {
            const two = buildReleaseSteps({ ...gitPolicyOptions, lockTasks: ['//a:lock', '//b:lock'] });
            expect(two.filter(function isLock(step) {
              return step.program === 'mise';
            }).map(function taskOf(step) {
              return step.args[1];
            })).toEqual(['//a:lock', '//b:lock']);
            const none = buildReleaseSteps({ ...gitPolicyOptions, lockTasks: [] });
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
