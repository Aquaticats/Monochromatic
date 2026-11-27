import { mkdir, mkdtempDisposable, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  containerProcesses,
  processLabelOf,
} from './native-gate-container-processes.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

await describe({
  name: processLabelOf.name,
  children: [
    it({
      name: 'names the test a nextest process runs, from the exact filter that process was started with',
      fn: async () => {
        expect(processLabelOf({
          argv: [
            '/work/package/git-policy/cli/target/debug/build/monochromatic-git-policy-cli/85367b807f819bb5/out/git_policy_cli-85367b807f819bb5',
            '--exact',
            'repo_read::repo_read_config_typed::tests::an_empty_value_over_every_documented_key_declines_or_equals_git',
            '--nocapture',
          ],
        }),).toBe('repo_read::repo_read_config_typed::tests::an_empty_value_over_every_documented_key_declines_or_equals_git',);
      },
    }),
    it({
      name: 'names any other program by its own name and first argument, so a fixture\'s Git is told from a test',
      fn: async () => {
        expect(processLabelOf({
          argv: [
            '/usr/bin/git',
            'diff-tree',
            '--root',
            '--no-commit-id',
          ],
        }),).toBe('git diff-tree',);
      },
    }),
    it({
      name: 'names a program with no argument by its own name alone',
      fn: async () => {
        expect(processLabelOf({ argv: ['/usr/bin/cat',], },),).toBe('cat',);
      },
    }),
    it({
      name: 'answers nothing for a process whose command line is empty, which a zombie or a kernel thread is',
      fn: async () => {
        expect(typeof processLabelOf({ argv: [], },),).toBe('symbol',);
      },
    }),
  ],
});

await describe({
  name: containerProcesses.name,
  children: [
    it({
      name: 'names and charges the processes the container\'s cgroup lists, heaviest first',
      fn: async () => {
        await using tree = await mkdtempDisposable(join(
          scratchParent,
          'gate-processes-test-',
        ),);
        /**
         A fixture scope holding this test process alone, which is a process whose files are there to read.
         */
        const scope = join(
          tree.path,
          'processes.scope',
        );
        await mkdir(
          scope,
          { recursive: true, },
        );
        await writeFile(
          join(scope, 'cgroup.procs',),
          `${String(process.pid)}\n`,
        );
        /**
         The processes the reader answered with.
         */
        const processes = await containerProcesses({
          cgroupPath: scope,
          count: 1,
        },);
        expect(processes,).toHaveLength(1,);
        expect(processes[0]?.pid,).toBe(process.pid,);
        expect(processes[0]?.rssBytes,).toBeGreaterThan(0,);
        expect((processes[0]?.label ?? '').length,).toBeGreaterThan(0,);
      },
      timeout: 20_000,
    }),
    it({
      name: 'narrows to the heaviest processes when the container holds more',
      fn: async () => {
        await using tree = await mkdtempDisposable(join(
          scratchParent,
          'gate-processes-count-',
        ),);
        /**
         A fixture scope holding this test process alone, listed as if the container held more than it names.
         */
        const scope = join(
          tree.path,
          'processes.scope',
        );
        await mkdir(
          scope,
          { recursive: true, },
        );
        await writeFile(
          join(scope, 'cgroup.procs',),
          `${String(process.pid)}\n1\n`,
        );
        expect(await containerProcesses({
          cgroupPath: scope,
          count: 1,
        },),).toHaveLength(1,);
      },
      timeout: 20_000,
    }),
  ],
});
