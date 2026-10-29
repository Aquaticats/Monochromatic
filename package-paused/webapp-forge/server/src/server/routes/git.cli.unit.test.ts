/**
 * Real-CLI verification of the smart-HTTP git server: spawn the system
 * `git` binary against a `serve()`-bound port and exercise clone / push
 * end-to-end.
 *
 * The existing `git.unit.test.ts` covers wire framing through h3's
 * in-process `app.fetch()`. This file is the user-boundary check the
 * Phase 2 plan calls for: the actual git CLI must speak our wire
 * protocol successfully.
 *
 * Three scenarios:
 *
 * 1. Tiny-file clone roundtrip; push, then clone elsewhere, then diff.
 * 2. 5 MB binary blob roundtrip; byte-for-byte equality after pack apply.
 * 3. 100-ref batched push; single push delivers every ref intact.
 *
 * Subprocess hardening:
 *
 * - `/usr/bin/git` is invoked by absolute path; the workspace ships a
 *   `cli-git` wrapper at `node_modules/.bin/git` that adds
 *   `require-root` and `atomic-push` semantics we do not want here.
 * - `PATH` is pinned to `/usr/bin:/bin` so any nested git invocation
 *   (hooks, helpers) cannot reach the wrapper either.
 * - `GIT_CONFIG_GLOBAL` and `GIT_CONFIG_SYSTEM` are silenced so the
 *   user's host config cannot influence the test.
 * - `GIT_PROTOCOL=version=0` because `iso-server-advertisement.ts`
 *   advertises v0/v1 capabilities only; modern git would otherwise try
 *   v2 first and rely on graceful fallback.
 */

import { BYTES_PER_MIB, } from '@monochromatic-dev/const-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import { randomBytes, } from 'node:crypto';
import {
  mkdtemp,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  H3,
  serve,
} from 'h3';
import spawn from 'nano-spawn';

/** Concrete type of the value `serve()` returns; not exported by h3 directly. */
type ServeHandle = ReturnType<typeof serve>;

Reflect.set(
  process.env,
  'DB_PATH',
  ':memory:',
);

/** Bare-gitdir root shared by every test in this file. */
const gitdirRoot = await mkdtemp(join(
  scratchRoot(),
  'forge-git-cli-root-',
),);

process.env.WEBAPP_FORGE_GITDIR_ROOT = gitdirRoot;

const gitRoutesMod = await import('./git.ts');

const {
  gitInfoRefsHandler,
  gitReceivePackHandler,
  gitUploadPackHandler,
} = gitRoutesMod;

/** Absolute path to the system git binary, bypassing the workspace wrapper. */
const SYSTEM_GIT = '/usr/bin/git';

/** Target size for the large-blob test (5 MiB). */
const FIVE_MIB: number = 5 * BYTES_PER_MIB;

/** Number of refs in the batched-push test. */
const BATCH_REF_COUNT = 100;

/** Pad width for batch ref names so they sort lexicographically. */
const BATCH_NAME_PAD = 3;

/** Default per-test timeout: server boot + git CLI takes time. */
const DEFAULT_TEST_TIMEOUT_MS = 30_000;

/**
 * Subprocess environment used for every git invocation. The empty
 * `HOME` placeholder gets replaced per-call with the per-test workdir
 * so any auxiliary git data lands inside the test sandbox.
 */
const BASE_GIT_ENV: Readonly<Record<string, string>> = {
  PATH: '/usr/bin:/bin',
  LANG: 'C',
  GIT_TERMINAL_PROMPT: '0',
  GIT_CONFIG_GLOBAL: '/dev/null',
  GIT_CONFIG_SYSTEM: '/dev/null',
  GIT_AUTHOR_NAME: 'CLI Test',
  GIT_AUTHOR_EMAIL: 'cli-test@example.invalid',
  GIT_COMMITTER_NAME: 'CLI Test',
  GIT_COMMITTER_EMAIL: 'cli-test@example.invalid',
  GIT_PROTOCOL: 'version=0',
};

/**
 * `await using` wrapper so the server's `close(true)` runs even if a
 * test throws. h3 does not re-export the underlying srvx `Server`
 * type, so we use {@link ServeHandle} as a stand-in.
 */
class DisposableServer implements AsyncDisposable {
  /** Bound server handle returned by `serve()`. */
  readonly server: ServeHandle;

  /**
   * @param server - the bound srvx server
   */
  constructor({ server, }: { readonly server: ServeHandle; },) {
    this.server = server;
  }

  /**
   * Closes the server and severs in-flight connections. Bound via
   * {@link Symbol.asyncDispose} so `await using` triggers it.
   */
  async [Symbol.asyncDispose](): Promise<void> {
    await this.server.close(true,);
  }
}

/**
 * Boots the smart-HTTP git server on a random port, returning the
 * public base URL plus a disposable handle for cleanup.
 *
 * @returns base URL (no trailing slash) plus async-disposable
 *
 * @example
 * ```ts
 * await using booted = await startServer();
 * await runGit({ cwd, args: ['clone', `${booted.base}/alice/demo.git`, target] });
 * ```
 */
async function startServer(): Promise<{
  readonly base: string;
  readonly disposable: DisposableServer;
}> {
  const app = new H3();
  app.get(
    '/:owner/:repo/info/refs',
    gitInfoRefsHandler,
  );
  app.post(
    '/:owner/:repo/git-upload-pack',
    gitUploadPackHandler,
  );
  app.post(
    '/:owner/:repo/git-receive-pack',
    gitReceivePackHandler,
  );
  const server = serve(
    app,
    {
      port: 0,
      hostname: '127.0.0.1',
      silent: true,
    },
  );
  await server.ready();
  const { url, } = server;
  if (url === undefined)
    throw new Error('serve(): server.url unavailable after ready()',);
  const base = url.endsWith('/',)
    ? url.slice(
      0,
      -1,
    )
    : url;
  return {
    base,
    disposable: new DisposableServer({ server, },),
  };
}

/**
 * Runs `git ARGS...` in `cwd` with the hardened test env, returning
 * `stdout` as a string. Throws on non-zero exit.
 *
 * @param row - inputs
 *
 * @returns subprocess stdout
 *
 * @example
 * ```ts
 * const refs = await runGit({ cwd: '/tmp/x', args: ['ls-remote', remoteUrl] });
 * ```
 */
async function runGit(row: {
  readonly cwd: string;
  readonly args: readonly string[];
},): Promise<string> {
  const { stdout, } = await spawn(
    SYSTEM_GIT,
    row.args,
    {
      cwd: row.cwd,
      env: {
        ...BASE_GIT_ENV,
        HOME: row.cwd,
      },
    },
  );
  return stdout;
}

/**
 * Initialises a fresh non-bare client repo at a new tmpdir, ready for
 * commits.
 *
 * @returns absolute path of the new repo
 *
 * @example
 * ```ts
 * const repo = await initClientRepo();
 * await writeFile(join(repo, 'README'), 'hi');
 * ```
 */
async function initClientRepo(): Promise<string> {
  const repo = await mkdtemp(join(
    scratchRoot(),
    'forge-git-cli-client-',
  ),);
  await runGit({
    cwd: repo,
    args: ['init', '--initial-branch=main',],
  },);
  return repo;
}

/**
 * Stages every change, commits with the given message, and returns
 * the resulting commit OID.
 *
 * @param row - inputs
 *
 * @returns the new commit OID
 */
async function commitAll(row: {
  readonly repo: string;
  readonly message: string;
},): Promise<string> {
  await runGit({
    cwd: row.repo,
    args: ['add', '-A',],
  },);
  await runGit({
    cwd: row.repo,
    args: ['commit', '-m', row.message,],
  },);
  const sha = await runGit({
    cwd: row.repo,
    args: ['rev-parse', 'HEAD',],
  },);
  return sha.trim();
}

/**
 * Returns the path of an empty tmpdir suitable as a clone target.
 *
 * @returns absolute path
 */
async function freshCloneTarget(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'forge-git-cli-clone-',
  ),);
}

await describe({
  name: 'routes/git real CLI',
  concurrency: 1,
  timeout: DEFAULT_TEST_TIMEOUT_MS,
  children: [
    it({
      name: 'clones back a small file pushed by the system git CLI',
      timeout: DEFAULT_TEST_TIMEOUT_MS,
      async fn() {
        const booted = await startServer();
        await using _shutdown = booted.disposable;
        const remote = `${booted.base}/alice/clone-roundtrip.git`;

        const repo = await initClientRepo();
        const filePath = join(
          repo,
          'hello.txt',
        );
        const payload = 'hello from the cli verification\n';
        await writeFile(
          filePath,
          payload,
        );
        await commitAll({
          repo,
          message: 'init',
        },);
        await runGit({
          cwd: repo,
          args: ['remote', 'add', 'origin', remote,],
        },);
        await runGit({
          cwd: repo,
          args: ['push', '-u', 'origin', 'main',],
        },);

        const cloneTarget = await freshCloneTarget();
        await runGit({
          cwd: cloneTarget,
          args: ['clone', remote, 'cloned',],
        },);

        const cloned = await readFile(
          join(
            cloneTarget,
            'cloned',
            'hello.txt',
          ),
          'utf8',
        );
        expect(cloned,).toBe(payload,);
      },
    },),
    it({
      name: 'roundtrips a 5 MiB binary blob byte-for-byte',
      timeout: DEFAULT_TEST_TIMEOUT_MS,
      async fn() {
        const booted = await startServer();
        await using _shutdown = booted.disposable;
        const remote = `${booted.base}/alice/large-blob.git`;

        const repo = await initClientRepo();
        const blob = randomBytes(FIVE_MIB,);
        await writeFile(
          join(
            repo,
            'blob.bin',
          ),
          blob,
        );
        await commitAll({
          repo,
          message: 'add 5 MiB blob',
        },);
        await runGit({
          cwd: repo,
          args: ['remote', 'add', 'origin', remote,],
        },);
        await runGit({
          cwd: repo,
          args: ['push', '-u', 'origin', 'main',],
        },);

        const cloneTarget = await freshCloneTarget();
        await runGit({
          cwd: cloneTarget,
          args: ['clone', remote, 'cloned',],
        },);

        const cloned = await readFile(join(
          cloneTarget,
          'cloned',
          'blob.bin',
        ),);
        expect(cloned.byteLength,).toBe(blob.byteLength,);
        // Byte-for-byte equality without piping a 5 MiB diff through
        // chai's deep-equal (which would format mismatches expensively).
        expect(Buffer.from(cloned,).equals(blob,),).toBe(true,);
      },
    },),
    it({
      name: 'accepts 100 ref updates in a single batched push',
      timeout: DEFAULT_TEST_TIMEOUT_MS,
      async fn() {
        const booted = await startServer();
        await using _shutdown = booted.disposable;
        const remote = `${booted.base}/alice/many-refs.git`;

        const repo = await initClientRepo();
        await writeFile(
          join(
            repo,
            'README',
          ),
          'batched push fixture\n',
        );
        const baseSha = await commitAll({
          repo,
          message: 'fixture',
        },);

        // One `update-ref --stdin` invocation creates all 100 refs
        // atomically; faster and clearer intent than 100 `git branch`
        // calls.
        const stdinScript = `${
          Array
            .from(
              { length: BATCH_REF_COUNT, },
              function buildLine(_unused, index,) {
                const ordinal = String(index + 1,).padStart(
                  BATCH_NAME_PAD,
                  '0',
                );
                return `create refs/heads/batch-${ordinal} ${baseSha}`;
              },
            )
            .join('\n',)
        }\n`;

        await spawn(
          SYSTEM_GIT,
          ['update-ref', '--stdin',],
          {
            cwd: repo,
            env: {
              ...BASE_GIT_ENV,
              HOME: repo,
            },
            stdin: { string: stdinScript, },
          },
        );

        await runGit({
          cwd: repo,
          args: ['remote', 'add', 'origin', remote,],
        },);
        await runGit({
          cwd: repo,
          args: ['push', 'origin', '--all',],
        },);

        // `ls-remote` exercises the upload-pack info/refs route just
        // like a clone would, but without writing the pack; it's the
        // tightest check that all 100 refs surfaced.
        const lsRemote = await runGit({
          cwd: repo,
          args: ['ls-remote', '--heads', remote,],
        },);
        const lines = lsRemote.split('\n',).filter(function notBlank(line,) {
          return line.length > 0;
        },);
        // Count rows whose ref is one of the batched names.
        const batched = lines.filter(function isBatched(line,) {
          return line.includes('refs/heads/batch-',);
        },);
        expect(batched.length,).toBe(BATCH_REF_COUNT,);
      },
    },),
  ],
},);
