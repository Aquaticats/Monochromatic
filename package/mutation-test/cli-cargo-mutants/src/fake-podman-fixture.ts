import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  mkdir,
  mkdtemp,
  readdir,
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
 Image ID that the stand-in `podman image inspect` reports for the tested image: 64 lowercase hexadecimal digits.
 */
export const fakeTestedImageId = 'a'.repeat(64);

/**
 Image ID that the stand-in `podman build --iidfile` writes for the campaign image.
 */
export const fakeCampaignImageId = 'b'.repeat(64);

/**
 Container ID that the stand-in `podman create` prints.
 */
export const fakeContainerId = 'c'.repeat(64);

/**
 Stand-in for `podman`: records every call, answers the few calls a campaign makes, and starts nothing.
 It copies the Containerfile out of every build context before the caller removes the context.
 */
const fakePodman = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.FAKE_DIR;
const args = process.argv.slice(2);
const calls = path.join(directory, 'calls.jsonl');
fs.appendFileSync(calls, JSON.stringify(args) + '\\n');
const number = fs.readFileSync(calls, 'utf8').trim().split('\\n').length;
if (args[0] === 'image' && args[1] === 'inspect') {
  process.stdout.write('${fakeTestedImageId}\\n');
} else if (args[0] === 'build') {
  const context = args[args.length - 1];
  fs.copyFileSync(path.join(context, 'Containerfile'), path.join(directory, 'containerfile.' + number));
  const at = args.indexOf('--iidfile');
  if (at >= 0) fs.writeFileSync(args[at + 1], 'sha256:${fakeCampaignImageId}\\n');
} else if (args[0] === 'create') {
  fs.writeFileSync(path.join(directory, 'created.json'), JSON.stringify(args));
  process.stdout.write('${fakeContainerId}\\n');
} else if (args[0] === 'container' && args[1] === 'inspect') {
  const created = JSON.parse(fs.readFileSync(path.join(directory, 'created.json'), 'utf8'));
  const at = created.indexOf('${fakeCampaignImageId}');
  process.stdout.write(JSON.stringify(created.slice(at + 1)) + '\\n');
} else if (args[0] === 'cp') {
  fs.mkdirSync(args[2], { recursive: true });
}
`;

/**
 Stand-in for `cargo`: records its call with a leading marker and answers `mutants --list` with one line per `--file`
 scope, or two fixed lines for an unscoped listing.
 */
const fakeCargo = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
fs.appendFileSync(path.join(process.env.FAKE_DIR, 'calls.jsonl'), JSON.stringify(['<cargo>', ...args]) + '\\n');
const files = args.flatMap((argument, index) => (args[index - 1] === '--file' ? [argument] : []));
const lines = files.length === 0 ? ['zz:1:1: unscoped one', 'zz:2:1: unscoped two'] : files.map((file) => file + ':1:1: replace f in ' + file);
process.stdout.write(lines.join('\\n') + '\\n');
`;

/**
 Stand-in for the installed \`cargo-mutants\` executable, copied into campaign images and hashed into manifests.
 */
const fakeCargoMutantsBytes = 'fake cargo-mutants executable\n';

/**
 Fixture holding the stand-in programs and the working directory a script under test runs in.
 */
export type FakePodman = AsyncDisposable & {
  readonly directory: string;
  readonly workingDirectory: string;
  readonly environment: Readonly<Record<string, string>>;
};

/**
 What a script under test did.
 */
export type ScriptRun = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
  readonly calls: readonly (readonly string[])[];
  readonly containerfiles: readonly string[];
  readonly evidenceFiles: Readonly<Record<string, string>>;
};

/**
 Create the stand-in programs and a clean working directory.

 @returns the fixture; declare it with `await using`
 @example
 ```ts
 await using fake = await createFakePodman();
 ```
 */
export async function createFakePodman(): Promise<FakePodman> {
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
    'cargo-mutants-podman-',
  ));
  /**
   Directory the script runs in, so evidence lands here and not in a package.
   */
  const workingDirectory = join(
    directory,
    'work',
  );
  /**
   Directory standing in for the cargo home.
   */
  const cargoHome = join(
    directory,
    'cargo',
  );
  await mkdir(
    join(
      cargoHome,
      'bin',
    ),
    { recursive: true },
  );
  await mkdir(
    workingDirectory,
    { recursive: true },
  );
  await mkdir(
    join(
      directory,
      'path',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      directory,
      'path',
      'podman',
    ),
    fakePodman,
    { mode: ownerOnlyMode },
  );
  await writeFile(
    join(
      directory,
      'path',
      'cargo',
    ),
    fakeCargo,
    { mode: ownerOnlyMode },
  );
  await writeFile(
    join(
      cargoHome,
      'bin',
      'cargo-mutants',
    ),
    fakeCargoMutantsBytes,
  );
  return {
    directory,
    workingDirectory,
    environment: {
      PATH: `${join(
        directory,
        'path',
      )}:${process.env['PATH'] ?? ''}`,
      FAKE_DIR: directory,
      CARGO_HOME: cargoHome,
      HOME: directory,
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
 Read every file under a directory into a map keyed by the path relative to it.

 @param root - directory to read
 @param relative - path below the root that is being read
 @returns file texts by relative path
 @example
 ```ts
 await readTree({ root: '/evidence', relative: '' });
 ```
 */
async function readTree({
  root,
  relative,
}: {
  readonly root: string;
  readonly relative: string;
}): Promise<Readonly<Record<string, string>>> {
  /**
   Entries of the directory.
   */
  const entries = await readdir(
    join(
      root,
      relative,
    ),
    { withFileTypes: true },
  );
  /**
   Maps of each entry, files as one pair and directories as their subtree.
   */
  const maps = await Promise.all(entries.map(async function readEntry(entry) {
    /**
     Path of the entry below the root.
     */
    const path = join(
      relative,
      entry.name,
    );
    return entry.isDirectory()
      ? readTree({
        root,
        relative: path,
      })
      : {
        [path]: await readFile(
          join(
            root,
            path,
          ),
          'utf8',
        ),
      };
  }));
  return Object.assign(
    {},
    ...maps,
  ) as Readonly<Record<string, string>>;
}

/**
 Run a Node script with the stand-in programs first on `PATH` and read back everything it left.

 @param fake - fixture
 @param script - absolute path of the script
 @param args - arguments after the script path
 @param extraEnvironment - environment added to the fixture's
 @returns exit status, output, the recorded podman calls, the Containerfiles built, and the evidence files written
 @example
 ```ts
 await runScript({ fake, script: '/p/bin/mutate.ts', args: [], extraEnvironment: {} });
 ```
 */
export async function runScript({
  fake,
  script,
  args,
  extraEnvironment,
}: {
  readonly fake: FakePodman;
  readonly script: string;
  readonly args: readonly string[];
  readonly extraEnvironment: Readonly<Record<string, string>>;
}): Promise<ScriptRun> {
  /**
   The running script.
   */
  const child = spawn(
    'node',
    [script, ...args],
    {
      cwd: fake.workingDirectory,
      env: {
        ...fake.environment,
        ...extraEnvironment,
      },
      stdio: ['ignore', 'pipe', 'pipe'],
    },
  );
  /**
   Output chunks by stream.
   */
  const chunks: Record<'stdout' | 'stderr', Buffer[]> = {
    stdout: [],
    stderr: [],
  };
  child.stdout.on(
    'data',
    function collectOut(chunk: Buffer) {
      chunks.stdout.push(chunk);
    },
  );
  child.stderr.on(
    'data',
    function collectError(chunk: Buffer) {
      chunks.stderr.push(chunk);
    },
  );
  /**
   Arguments of the close event.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Names in the fixture directory, which show whether podman was ever called.
   */
  const names = await readdir(fake.directory);
  /**
   Recorded podman calls, one JSON array per line; absent when the script never called podman.
   */
  const callText = names.includes('calls.jsonl')
    ? await readFile(
      join(
        fake.directory,
        'calls.jsonl',
      ),
      'utf8',
    )
    : '';
  /**
   Names of the Containerfiles the fake copied, in build order.
   */
  const containerfileNames = names
    .filter(function isContainerfile(name) {
      return name.startsWith('containerfile.');
    })
    .toSorted(function byNumber(first, second) {
      return Number(first.split('.')[1]) - Number(second.split('.')[1]);
    });
  /**
   Evidence files below the working directory's `target`, when any were written.
   */
  const evidenceFiles = (await readdir(fake.workingDirectory)).includes('target')
    ? await readTree({
      root: join(
        fake.workingDirectory,
        'target',
      ),
      relative: '',
    })
    : {};
  return {
    status: typeof closed[0] === 'number' ? closed[0] : 1,
    stdout: Buffer.concat(chunks.stdout)
      .toString('utf8'),
    stderr: Buffer.concat(chunks.stderr)
      .toString('utf8'),
    calls: callText === ''
      ? []
      : callText.trim()
        .split('\n')
        .map(function parseCall(line) {
          return JSON.parse(line) as readonly string[];
        }),
    containerfiles: await Promise.all(containerfileNames.map(async function readContainerfile(name) {
      return readFile(
        join(
          fake.directory,
          name,
        ),
        'utf8',
      );
    })),
    evidenceFiles,
  };
}
