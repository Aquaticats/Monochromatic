import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { connect } from 'node:net';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  instanceOf,
  SERVICE_MODE_WORDS,
} from '../e2e/service-launch-fixture.ts';
import {
  assertServiceAnswers,
  makeService,
} from './native-perf-service.ts';

/**
 Private scratch parent shared with the benchmark itself, outside the repository.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 A fake service program: it records its arguments next to its socket and listens on the `--path` it is given, as the
 real service mode (`monochromatic-git-policy-cli service run`) does.
 */
const FAKE_SERVICE = `#!${process.execPath}
const { createServer } = require('node:net');
const { writeFileSync } = require('node:fs');
const arguments_ = process.argv.slice(2);
const socket = arguments_[arguments_.indexOf('--path') + 1];
writeFileSync(socket + '.arguments', JSON.stringify(arguments_));
createServer().listen(socket);
`;

/**
 A fake service program that ends at once, without listening.
 */
const FAILING_SERVICE = `#!${process.execPath}
process.exit(3);
`;

/**
 Make a scratch tree with the fake programs in it.

 @returns the tree, the fake service and the failing service; the tree removes itself when the caller's scope ends, so
   a failing assertion leaves no directory behind
 */
async function scratchTree(): Promise<{
  readonly root: string;
  readonly fake: string;
  readonly failing: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  /**
   The new tree.
   */
  const directory = await mkdtempDisposable(join(
    scratchParent,
    'ps-',
  ));
  /**
   Path of the tree, which the fixture paths are built from.
   */
  const root = directory.path;
  /**
   Path of the fake service.
   */
  const fake = join(
    root,
    'fake-service',
  );
  /**
   Path of the failing service.
   */
  const failing = join(
    root,
    'failing-service',
  );
  await writeFile(
    fake,
    FAKE_SERVICE,
  );
  await writeFile(
    failing,
    FAILING_SERVICE,
  );
  await chmod(
    fake,
    0o755,
  );
  await chmod(
    failing,
    0o755,
  );
  return {
    root,
    fake,
    failing,
    [Symbol.asyncDispose]: async function removeScratchTree(): Promise<void> {
      await directory.remove();
    },
  };
}

/**
 Whether something accepts connections on a socket path.

 @param socket - path of the unix socket

 @returns true when a connection was made
 */
async function accepts(socket: string): Promise<boolean> {
  return await new Promise<boolean>(function attempt(resolve) {
    /**
     The connection attempt.
     */
    const connection = connect(socket);
    connection.once(
      'connect',
      function connected() {
        connection.destroy();
        resolve(true);
      },
    );
    connection.once(
      'error',
      function refused() {
        resolve(false);
      },
    );
  });
}

/**
 Put a fake `git` into `root/bin` that stands for the wrapper's answer to `git monochromatic-git-policy-cli service status`.

 @param root - scratch root that holds `bin`

 @param status - exit status the fake wrapper ends with

 @param stderr - words it prints
 */
async function fakeWrapper({
  root,
  status,
  stderr,
}: {
  readonly root: string;
  readonly status: number;
  readonly stderr: string;
}): Promise<void> {
  await mkdir(
    join(
      root,
      'bin',
    ),
    { recursive: true, },
  );
  /**
   Path of the fake wrapper.
   */
  const program = join(
    root,
    'bin',
    'git',
  );
  await writeFile(
    program,
    `#!${process.execPath}
if (process.argv.slice(2).join(' ') !== 'monochromatic-git-policy-cli service status') process.exit(99);
if (!process.env.XDG_RUNTIME_DIR) process.exit(98);
process.stderr.write(${JSON.stringify(stderr)});
process.exit(${String(status)});
`,
  );
  await chmod(
    program,
    0o755,
  );
}

await describe({
  name: makeService.name,
  children: [
    it({
      name: 'serves each worktree under the instance name and socket the wrapper derives, and stops them all',
      fn: async () => {
        await using tree = await scratchTree();
        const { root, fake, } = tree;
        await mkdir(join(
          root,
          'home',
        ));
        const first = join(
          root,
          'first',
          '.git',
        );
        const second = join(
          root,
          'second',
          '.git',
        );
        await mkdir(
          first,
          { recursive: true, },
        );
        await mkdir(
          second,
          { recursive: true, },
        );
        const service = await makeService({
          program: fake,
          root,
          scratchParent: root,
        });
        await service.serve(first);
        await service.serve(second);
        const names = await Promise.all([
          first,
          second,
        ].map(function nameOf(gitDir) {
          return instanceOf(gitDir);
        }));
        expect(names[0]).not.toBe(names[1]);
        await Promise.all(names.map(async function checkServed(name) {
          const socket = join(
            service.runtime,
            'monochromatic-git-policy-cli',
            `${name}.sock`,
          );
          expect(await accepts(socket)).toBe(true);
          const recorded: unknown = JSON.parse(await readFile(
            `${socket}.arguments`,
            'utf8',
          ));
          expect(recorded).toEqual([
            ...SERVICE_MODE_WORDS,
            '--journal',
            join(
              root,
              'journal',
            ),
            '--path',
            socket,
            '--instance',
            name,
          ]);
        }));
        await service.stop();
        await expect(stat(service.runtime)).rejects.toThrow();
      },
    }),
    it({
      name: 'fails naming the worktree when its service ends without listening',
      fn: async () => {
        await using tree = await scratchTree();
        const { root, failing, } = tree;
        const gitDir = join(
          root,
          'repo',
          '.git',
        );
        await mkdir(
          gitDir,
          { recursive: true, },
        );
        const service = await makeService({
          program: failing,
          root,
          scratchParent: root,
        });
        await expect(service.serve(gitDir)).rejects.toThrow(gitDir);
        await service.stop();
      },
    }),
    it({
      name: 'fails before starting a service whose socket path a socket cannot bind',
      fn: async () => {
        await using tree = await scratchTree();
        const { root, fake, } = tree;
        const deep = join(
          root,
          'x'.repeat(100),
        );
        await mkdir(deep);
        const service = await makeService({
          program: fake,
          root,
          scratchParent: deep,
        });
        const gitDir = join(
          root,
          'repo',
          '.git',
        );
        await expect(service.serve(gitDir)).rejects.toThrow('longer than');
        await service.stop();
      },
    }),
  ],
});

await describe({
  name: assertServiceAnswers.name,
  children: [
    it({
      name: 'passes when the wrapper reports that the worktree service answers, and asks with the run runtime directory',
      fn: async () => {
        await using tree = await scratchTree();
        const { root, } = tree;
        await mkdir(join(
          root,
          'home',
        ));
        await fakeWrapper({
          root,
          status: 0,
          stderr: '',
        });
        await assertServiceAnswers({
          root,
          runtime: join(
            root,
            'rt',
          ),
          directory: root,
        });
      },
    }),
    it({
      name: 'fails naming the repository and the wrapper words when the service does not answer',
      fn: async () => {
        await using tree = await scratchTree();
        const { root, } = tree;
        await mkdir(join(
          root,
          'home',
        ));
        await fakeWrapper({
          root,
          status: 1,
          stderr: 'does not answer here',
        });
        /**
         Failure the refused answer raised, and nothing when it was accepted.
         */
        let failure: unknown;
        try {
          await assertServiceAnswers({
            root,
            runtime: join(
              root,
              'rt',
            ),
            directory: root,
          });
        } catch (error: unknown) {
          failure = error;
        }
        expect(failure).toBeInstanceOf(Error);
        expect((failure as Error).message).toContain(root);
        expect((failure as Error).message).toContain('does not answer here');
      },
    }),
  ],
});
