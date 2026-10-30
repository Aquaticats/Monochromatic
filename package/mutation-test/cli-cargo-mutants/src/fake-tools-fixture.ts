import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Scratch parent of every fixture directory; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Stand-in for the `node` program: counts its calls, records its arguments NUL-separated, and prints the retry marker
 for the first `FAKE_FAIL_UNTIL` calls.
 */
const fakeNode = `#!/bin/sh
count=$(cat "$FAKE_DIR/count" 2>/dev/null || echo 0)
count=$((count + 1))
echo "$count" > "$FAKE_DIR/count"
printf '%s\\0' "$@" > "$FAKE_DIR/args.$count"
if [ "$count" -le "$FAKE_FAIL_UNTIL" ]; then echo 'FAILED   Unmutated baseline'; fi
`;

/**
 Stand-in for the `rg` program that supports the one call shape the shell body makes:
 `rg --quiet --fixed-strings -- <text> <file>`.
 */
const fakeRg = `#!/bin/sh
shift 3
grep --quiet --fixed-strings -- "$1" "$2"
`;

/**
 Directory holding the stand-in programs and the files they write; disposing it removes the directory.
 */
export type FakeTools = AsyncDisposable & {
  readonly directory: string;
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Create stand-in `node` and `rg` programs in a fresh scratch directory.

 @param failUntil - number of leading `node` calls whose log shows the retry marker
 
 @returns the directory and the environment that puts it first on `PATH`; declare it with `await using`
 
 @example
 ```ts
 await using tools = await createFakeTools(0);
 ```
 */
export async function createFakeTools(failUntil: number): Promise<FakeTools> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   Fresh directory for this fixture.
   */
  const directory = await mkdtemp(join(
    scratchParent,
    'cargo-mutants-fake-',
  ));
  for (const [name, text] of [
    [
      'node',
      fakeNode
    ],
    [
      'rg',
      fakeRg
    ],
  ] as const) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- two small files, written in order
    await writeFile(
      join(
        directory,
        name,
      ),
      text,
    );
    // oxlint-disable-next-line eslint/no-await-in-loop -- two small files, written in order
    await chmod(
      join(
        directory,
        name,
      ),
      ownerOnlyMode,
    );
  }
  return {
    directory,
    environment: {
      PATH: `${directory}:${process.env
        .PATH
        ?? ''}`,
      FAKE_DIR: directory,
      FAKE_FAIL_UNTIL: String(failUntil),
    },
    async [Symbol.asyncDispose]() {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Read how often the stand-in `node` ran.

 @param tools - fixture
 
 @returns the call count; the stand-in writes it on its first call, so a run that never reached it fails the read
 
 @example
 ```ts
 await readCallCount(tools);
 ```
 */
export async function readCallCount(tools: FakeTools): Promise<number> {
  return Number(await readFile(
    join(
      tools.directory,
      'count',
    ),
    'utf8',
  ));
}

/**
 Read the arguments of one stand-in `node` call.

 @param tools - fixture
 
 @param call - one-based call number
 
 @returns the arguments exactly as the program received them
 
 @example
 ```ts
 await readCallArguments({ tools, call: 1 });
 ```
 */
export async function readCallArguments({
  tools,
  call,
}: {
  readonly tools: FakeTools;
  readonly call: number;
}): Promise<readonly string[]> {
  /**
   NUL-terminated record of the call.
   */
  const record = await readFile(
    join(
      tools.directory,
      `args.${call}`,
    ),
    'utf8',
  );
  return record.split('\0')
    .slice(
    0,
    -1,
    );
}
