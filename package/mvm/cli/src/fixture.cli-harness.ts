/**
 Test helpers that run the built `mvm` executable against stand-in host tools.

 Each helper works in a fresh temporary directory, so tests using them touch
 neither the real home directory nor real VMs and can run side by side.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import spawn, { SubprocessError, } from 'nano-spawn';

//region Paths

/**
 Built `mvm` executable under test.
 */
export const CLI_PATH: string = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'node',
  'cli.mjs',
);

/**
 Source of the stand-in `virsh`.
 */
export const FAKE_VIRSH_PATH: string = join(
  import.meta.dirname,
  'fixture.fake-virsh.ts',
);

//endregion Paths

//region Sandbox

/**
 One test's private world: a home directory, a directory for stand-in
 executables, and the fake virsh's state directory.
 */
export type Sandbox = AsyncDisposable & {
  /**
   Directory for stand-in executables; the only entry of the child's `PATH` unless a test adds more.
   */
  readonly bin: string;
  /**
   Arguments of every fake virsh invocation so far, oldest first.
   */
  readonly calls: () => Promise<readonly (readonly string[])[]>;
  /**
   Home directory handed to the child, so mvm's data directory is disposable.
   */
  readonly home: string;
  /**
   State directory of the fake virsh; holds `scenario.json`, `state.json` and `calls.jsonl`.
   */
  readonly state: string;
  /**
   Directory mvm keeps VM data in, inside the disposable home.
   */
  readonly vms: string;
};

/**
 Creates a sandbox and writes the fake virsh's scenario into it.

 @param scenario - What the fake virsh is asked to do; see `fixture.fake-virsh.ts`

 @returns Sandbox removed when its scope ends

 @example
 ```ts
 await using sandbox = await createSandbox({ domains: [{ name: 'mvm-dev', state: 'running' }] });
 ```
 */
export async function createSandbox(
  scenario: Readonly<Record<string, unknown>>,
): Promise<Sandbox> {
  /**
   Root of everything this sandbox creates.
   */
  const root = await mkdtemp(join(
    tmpdir(),
    'mvm-cli-test-',
  ),);
  /**
   Directory for stand-in executables.
   */
  const bin = join(
    root,
    'bin',
  );
  /**
   Disposable home directory.
   */
  const home = join(
    root,
    'home',
  );
  /**
   State directory of the fake virsh.
   */
  const state = join(
    root,
    'virsh-state',
  );
  /**
   mvm's VM directory inside the disposable home.
   */
  const vms = join(
    home,
    '.local',
    'share',
    'mvm',
    'vms',
  );
  await Promise.all([
    mkdir(bin,),
    mkdir(
      vms,
      { recursive: true, },
    ),
    mkdir(state,),
  ],);
  await writeFile(
    join(
      state,
      'scenario.json',
    ),
    JSON.stringify(scenario,),
  );
  return {
    bin,
    async calls() {
      /**
       Call log written by the fake virsh, one JSON array per line.
       */
      const log = await readFile(
        join(
          state,
          'calls.jsonl',
        ),
        'utf8',
      );
      return log
        .split('\n',)
        .filter(function isCall(line,) {
          return line !== '';
        },)
        .map(function parseCall(line,): readonly string[] {
          /**
           One logged invocation, typed as unknown until its elements are checked.
           */
          const parsed: unknown = JSON.parse(line,);
          return Array.isArray(parsed,)
            ? parsed.filter(function isText(element: unknown,): element is string {
              return (typeof element) === 'string';
            },)
            : [];
        },);
    },
    home,
    state,
    vms,
    async [Symbol.asyncDispose]() {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Environment that points mvm at the fake virsh through its command variable.

 @param sandbox - Sandbox whose state directory the fake uses

 @returns Variables to pass to the child

 @example
 ```ts
 await runCli({ args: ['list'], env: fakeVirshEnv(sandbox), sandbox });
 ```
 */
export function fakeVirshEnv(sandbox: Sandbox,): Readonly<Record<string, string>> {
  return {
    MVM_FAKE_VIRSH_DIR: sandbox.state,
    MVM_VIRSH_COMMAND: JSON.stringify([
      process.execPath,
      FAKE_VIRSH_PATH,
    ],),
  };
}

/**
 Writes a stand-in executable into the sandbox's `bin` directory.
 The file starts with a line naming this Node executable, so it runs without `env` or `node` on `PATH`.

 @param body - JavaScript the executable runs

 @param name - File name, the command name tests invoke

 @param sandbox - Sandbox receiving the executable

 @example
 ```ts
 await writeExecutable({ body: 'process.exitCode = 3;', name: 'qemu-img', sandbox });
 ```
 */
export async function writeExecutable({
  body,
  name,
  sandbox,
}: {
  readonly body: string;
  readonly name: string;
  readonly sandbox: Sandbox;
},): Promise<void> {
  /**
   Path of the new executable.
   */
  const path = join(
    sandbox.bin,
    name,
  );
  await writeFile(
    path,
    `#!${process.execPath}\n${body}\n`,
  );
  await chmod(
    path,
    0o700,
  );
}

/**
 Installs a stand-in `flatpak` that knows one application and runs its `virsh` as the fake virsh.
 Every invocation is appended to `flatpak-calls.jsonl` in the fake virsh's state directory.

 @param sandbox - Sandbox receiving the executable

 @example
 ```ts
 await installFakeFlatpak(sandbox);
 ```
 */
export async function installFakeFlatpak(sandbox: Sandbox,): Promise<void> {
  await writeExecutable({
    body: [
      'const { appendFileSync } = require(\'node:fs\');',
      'const args = process.argv.slice(2);',
      `appendFileSync(${
        JSON.stringify(join(
          sandbox.state,
          'flatpak-calls.jsonl',
        ),)
      }, JSON.stringify(args) + '\\n');`,
      'if (args[0] === \'info\') {',
      '  process.exitCode = args.at(-1) === \'org.virt_manager.virt-manager\' ? 0 : 1;',
      '}',
      'else if (args[0] === \'run\' && args[1] === \'--command=virsh\' && args[2] === \'org.virt_manager.virt-manager\') {',
      `  process.env.MVM_FAKE_VIRSH_DIR = ${JSON.stringify(sandbox.state,)};`,
      '  process.argv = [process.argv[0], process.argv[1], ...args.slice(3)];',
      `  import(${JSON.stringify(pathToFileURL(FAKE_VIRSH_PATH,)
        .href,)});`,
      '}',
      'else {',
      String.raw`  process.stderr.write('fake flatpak: unsupported invocation\n');`,
      '  process.exitCode = 1;',
      '}',
    ].join('\n',),
    name: 'flatpak',
    sandbox,
  },);
}

//endregion Sandbox

//region Running the CLI

/**
 What one run of the built `mvm` produced.
 */
export type CliResult = {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Runs the built `mvm` with a private home and a `PATH` holding only the sandbox's stand-ins.

 @param args - Arguments after `mvm`

 @param env - Extra variables for the child, such as the fake virsh's

 @param sandbox - Sandbox supplying the home directory and `PATH`

 @returns Exit status and both output streams

 @example
 ```ts
 const result = await runCli({ args: ['list'], sandbox });
 ```
 */
export async function runCli({
  args,
  env = {},
  sandbox,
}: {
  readonly args: readonly string[];
  readonly env?: Readonly<Record<string, string>>;
  readonly sandbox: Sandbox;
},): Promise<CliResult> {
  try {
    /**
     Completed run with exit status 0.
     */
    const result = await spawn(
      process.execPath,
      [
        CLI_PATH,
        ...args,
      ],
      {
        env: {
          HOME: sandbox.home,
          MVM_QEMU_IMG_COMMAND: '',
          MVM_VIRSH_COMMAND: '',
          PATH: sandbox.bin,
          ...env,
        },
      },
    );
    return {
      exitCode: 0,
      stderr: result.stderr,
      stdout: result.stdout,
    };
  }
  catch (error: unknown) {
    if (!(error instanceof SubprocessError))
      throw error;

    return {
      exitCode: error.exitCode ?? 1,
      stderr: error.stderr,
      stdout: error.stdout,
    };
  }
}

//endregion Running the CLI
