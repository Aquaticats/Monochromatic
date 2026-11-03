import {
  spawn,
  spawnSync,
} from 'node:child_process';
import { once } from 'node:events';
import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { acquireCampaignLock } from './campaign-lock.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Longest a test waits for the kernel to give back the lock of a process that was killed.
 */
const releaseWaitMilliseconds = 10_000;

/**
 Make a disposable directory under the agent scratch area, never the system temporary directory.

 @returns the directory, removed when disposed

 @example
 ```ts
 await using directory = await scratch();
 ```
 */
async function scratch(): Promise<AsyncDisposable & { readonly path: string }> {
  /**
   Parent of every scratch directory.
   */
  const parent = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    parent,
    { recursive: true },
  );
  return mkdtempDisposable(join(
    parent,
    'cargo-mutants-lock-',
  ));
}

/**
 The outcome of trying to take the lock.
 */
type Attempt = {
  /**
   The lock, as the only element, or none when it was refused.
   */
  readonly lock: readonly AsyncDisposable[];
  /**
   Why it was refused, empty when it was not.
   */
  readonly reason: string;
};

/**
 Try to take the lock.

 @param directory - evidence directory

 @returns the lock or the reason it was refused

 @example
 ```ts
 await attempt('/e/native-mutation-abc');
 ```
 */
async function attempt(directory: string): Promise<Attempt> {
  try {
    return {
      lock: [await acquireCampaignLock(directory)],
      reason: '',
    };
  }
  catch (error) {
    return {
      lock: [],
      reason: error instanceof CargoMutantsToolingError ? error.message : 'unexpected',
    };
  }
}

/**
 Script of a child process that takes the lock with the given `PATH`, says what became of it, and keeps the lock until it is
 killed.
 */
const ownerWithPathScript = (lockModule: string, evidence: string): string => `import { acquireCampaignLock } from ${JSON.stringify(lockModule)};
try { await acquireCampaignLock(${JSON.stringify(evidence)}); console.log('acquired'); setTimeout(() => process.exit(0), 30000); }
catch (error) { console.log('refused ' + String(error)); }`;

/**
 Run a child node process whose `PATH` holds only the given programs, and read its first line.

 @param programs - names of programs to find on the host and link into the child's `PATH`

 @param fakes - the programs to write into the child's `PATH` as given text, by name

 @param evidence - evidence directory the child takes the lock on

 @returns the child, still alive when it holds the lock, and its first line

 @example
 ```ts
 await runWithPath({ programs: ['flock', 'sh'], fakes: {}, evidence: '/e/x' });
 ```
 */
async function runWithPath({
  programs,
  fakes,
  evidence,
}: {
  readonly programs: readonly string[];
  readonly fakes: Readonly<Record<string, string>>;
  readonly evidence: string;
}): Promise<{ readonly child: ReturnType<typeof spawn>; readonly line: string }> {
  await using bin = await scratch();
  for (const program of programs) {
    const found = spawnSync('sh', ['-c', `command -v ${program}`], { encoding: 'utf8' }).stdout.trim();
    await symlink(found, join(bin.path, program));
  }
  for (const [name, text] of Object.entries(fakes)) {
    await writeFile(join(bin.path, name), text);
    await chmod(join(bin.path, name), 0o755);
  }
  const child = spawn(
    process.execPath,
    ['--input-type=module', '--eval', ownerWithPathScript(join(import.meta.dirname, 'campaign-lock.ts'), evidence)],
    { stdio: ['ignore', 'pipe', 'inherit'], env: { ...process.env, PATH: bin.path } },
  );
  const [chunk] = await once(child.stdout!, 'data');
  return { child, line: String(chunk).trim() };
}

await describe({
  name: acquireCampaignLock.name,
  children: [
    it({
      name: 'refuses a second run while the first holds the lock, and allows one after the first released it',
      fn: async () => {
        await using directory = await scratch();
        {
          await using held = await acquireCampaignLock(directory.path);
          expect(held).toBeDefined();
          await expect(acquireCampaignLock(directory.path)).rejects.toThrow(CargoMutantsToolingError);
        }
        await using again = await acquireCampaignLock(directory.path);
        expect(again).toBeDefined();
      },
    }),
    it({
      name: 'lets exactly one of several runs that start together take the lock, over a leftover lock file of the old runner, and leaves no file of its own',
      fn: async () => {
        await using directory = await scratch();
        // The old runner left a lock file that names a process that is gone; it means nothing now, and no contender has to
        // judge it, which is the step two contenders used to race on.
        await writeFile(join(directory.path, 'campaign.lock'), '999999 some-boot:1\n');
        const settled = await Promise.all(Array.from(
          { length: 8 },
          function start() {
            return attempt(directory.path);
          },
        ));
        const held = settled.flatMap(function lockOf(result) {
          return result.lock;
        });
        expect(held.length).toBe(1);
        expect(settled.filter((result) => result.lock.length === 0).every((result) => result.reason.includes('Another run'))).toBe(true);
        expect((await readdir(directory.path)).toSorted()).toEqual(['campaign.lock']);
        await held[0]?.[Symbol.asyncDispose]();
        await using again = await acquireCampaignLock(directory.path);
        expect(again).toBeDefined();
      },
    }),
    it({
      name: 'is given back by the kernel when the process that holds it is killed, so a killed run never leaves a lock to take over',
      fn: async () => {
        await using directory = await scratch();
        const lockModule = join(import.meta.dirname, 'campaign-lock.ts');
        /**
         A process that takes the lock and waits.
         */
        const owner = spawn(
          process.execPath,
          [
            '--input-type=module',
            '--eval',
            `import { acquireCampaignLock } from ${JSON.stringify(lockModule)}; await acquireCampaignLock(${JSON.stringify(directory.path)}); console.log('owner holds'); setTimeout(() => process.exit(0), 30000);`,
          ],
          { stdio: ['ignore', 'pipe', 'inherit'] },
        );
        await once(owner.stdout, 'data');
        expect((await attempt(directory.path)).lock).toEqual([]);
        owner.kill('SIGKILL');
        await once(owner, 'close');
        const deadline = Date.now() + releaseWaitMilliseconds;
        /**
         Take the lock as soon as the kernel has dropped it.

         @returns the lock, as the only element, or none within the wait
         */
        async function takeWhenFree(): Promise<readonly AsyncDisposable[]> {
          const result = await attempt(directory.path);
          if (result.lock.length > 0)
            return result.lock;
          if (Date.now() > deadline)
            return [];
          await sleep(50);
          return await takeWhenFree();
        }
        const taken = await takeWhenFree();
        expect(taken.length).toBe(1);
        await taken[0]?.[Symbol.asyncDispose]();
      },
    }),
    it({
      name: 'keeps the lock when every child process of the lock owner is killed, because the campaign process itself holds it',
      fn: async () => {
        await using directory = await scratch();
        // The owner is a separate process, so killing its children cannot touch another test's holders.
        const { child: owner, line } = await runWithPath({ programs: ['flock', 'sh', 'cat'], fakes: {}, evidence: directory.path });
        try {
          expect(line).toBe('acquired');
          const children = spawnSync('pgrep', ['--parent', String(owner.pid)], { encoding: 'utf8' }).stdout.split('\n').filter((entry) => entry !== '');
          for (const child of children)
            process.kill(Number(child), 'SIGKILL');
          await sleep(500);
          const second = await attempt(directory.path);
          expect(second.lock.length).toBe(0);
          expect(second.reason.includes('Another run')).toBe(true);
        }
        finally {
          owner.kill('SIGKILL');
          await once(owner, 'close');
        }
      },
    }),
    it({
      name: 'does not report a lock held that a helper without cat would drop at once: with no cat on PATH a second run is still refused',
      fn: async () => {
        await using directory = await scratch();
        const { child, line } = await runWithPath({ programs: ['flock', 'sh'], fakes: {}, evidence: directory.path });
        try {
          if (line === 'acquired') {
            await sleep(500);
            const second = await attempt(directory.path);
            expect(second.lock.length).toBe(0);
          }
          else
            expect(line.startsWith('refused')).toBe(true);
        }
        finally {
          child.kill('SIGKILL');
          await once(child, 'close');
        }
      },
    }),
    it({
      name: 'fails closed when flock reports anything but success or a conflict, even after it printed the old readiness word',
      fn: async () => {
        await using directory = await scratch();
        for (const body of ["process.stdout.write('held\\n'); process.exit(3);", 'process.exit(3);', "process.stdout.write('held\\n'); process.kill(process.pid, 'SIGKILL');"]) {
          const { child, line } = await runWithPath({
            programs: [],
            fakes: { flock: `#!${process.execPath}\n${body}\n` },
            evidence: directory.path,
          });
          try {
            expect(line.startsWith('refused')).toBe(true);
            expect(line.includes('Could not take the lock')).toBe(true);
          }
          finally {
            child.kill('SIGKILL');
            await once(child, 'close');
          }
        }
      },
    }),
    it({
      name: 'does not take a lock on a directory that is missing, and does not call that a second run',
      fn: async () => {
        await using directory = await scratch();
        const outcome = await attempt(join(directory.path, 'missing'));
        expect(outcome.lock).toEqual([]);
        expect(outcome.reason.includes('Another run')).toBe(false);
      },
    }),
  ],
});
