#!/usr/bin/env node
/**
 Reproduce issue 6293 in a container whose session bus and service manager are its own, and check what the fix
 promises:

 - a run given a runtime directory of its own still commits, asks no service manager anything, and leaves nothing
   under the account's state directory (its journal, its retry plans and its scratch files stay inside that runtime
   directory);
 - a push to an address that can never be a remote ends its retry plan at once, leaves one notice, and no later
   command prints it again;
 - the session whose own bus lives in its runtime directory is unaffected (case A, the control);
 - an account that named no runtime directory at all is unaffected (case C, the control).

The account's manager is a stand-in named `systemd-run` that records every call and then does what
`systemd-run --user --scope` does, and the bus is a `dbus-daemon` inside the container, so nothing here can reach
this machine's manager or session bus (the rules every Pi brief carries).

Two arms are how the defect was shown:
`--binary target/debug/monochromatic-git-policy-cli` built from the branch fails the proof before the fix, and the
same command after it passes. The arms are built outside this program with `mise run
//package/git-policy/cli:native:build`, and the executable is copied into the image, so the container needs no Rust.

Usage: `node bin/proof-private-runtime-directory.ts [--binary <path>] [--memory <size>] [--cpus <count>]
[--pids-limit <count>]`.
Every option overrides this proof's own bounds of 4 GiB, 4 CPUs and 512 processes.
 */
import {
  copyFile,
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  parseContainerLimitOptions,
  buildLimitArguments,
  runLimitArguments,
} from './container-limits.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';
import { pullCachyosLatest } from './native-test-base.ts';
import {
  proofScript,
  standInSystemdRun,
} from './proof-private-runtime-container-steps.ts';

/**
 Package directory this proof runs from.
 */
const packageDirectory = resolve(
  import.meta.dirname,
  '..',
);
/**
 Bounds this proof runs its container with when nothing overrides them: the case that commits needs four CPUs to
 start two services and three repositories, and the wrapper reads descriptors of this account's processes, which
 needs room in memory.
 */
const proofLimits = [
  '--memory',
  '4g',
  '--cpus',
  '4',
  '--pids-limit',
  '512',
];
/**
 The image this proof builds and runs.
 */
const image = 'localhost/cli-git-private-runtime-proof:latest';

/**
 The options this run reads: `--binary` and the container limits.

 @returns the executable to copy into the image and the bounds its container gets
 */
function readOptions(): {
  readonly executable: string;
  readonly limits: ReturnType<typeof parseContainerLimitOptions>['limits'];
} {
  /**
   Everything after `--binary` and the limit options, split apart.
   */
  const split = splitBinaryOption(
    process.argv
      .slice(2,),
  );
  /**
   Options left for the container limits.
   */
  const forTheLimits = split.rest;
  /**
   The bounds given on the command line, over this proof's own.
   */
  const parsed = parseContainerLimitOptions({
    options: [
      ...proofLimits,
      ...forTheLimits,
    ],
    acceptPidsLimit: true,
  });
  /**
   The arguments this proof has no use for.
   */
  const leftovers: readonly string[] = parsed.rest;
  if (leftovers.length > 0) {
    /**
     Those arguments as one line, quoted back in the refusal.
     */
    const unused: string = leftovers.join(' ');
    throw new NativeVerificationError(`this proof takes no argument these do not use: ${unused}`);
  }
  return {
    executable: split.executable,
    limits: parsed.limits,
  };
}

/**
 What:
  Where `--binary` points, and the arguments without that option and its value.
 Why:
  The executable is this proof's only own option, and every other argument belongs to the container's bounds, so one
  pass separates them without a mutable accumulator at a function's root.

 @param arguments_ - everything after the program's name

 @returns the wrapper this arm runs and the arguments the limits are read from
 */
function splitBinaryOption(arguments_: readonly string[],): {
  readonly executable: string;
  readonly rest: readonly string[];
} {
  /**
   Where the option stands, or nothing when it was not given.
   */
  if (!arguments_.includes('--binary',)) {
    return {
      executable: join(
        packageDirectory,
        'target',
        'debug',
        'monochromatic-git-policy-cli',
      ),
      rest: arguments_,
    };
  }
  /**
   Where the option stands.
   */
  const at = arguments_.indexOf('--binary',);
  /**
   The path the option names.
   */
  const named = arguments_[at + 1];
  if (named === undefined)
    throw new NativeVerificationError('--binary needs the path of a wrapper executable');
  return {
    executable: resolve(named,),
    rest: arguments_.filter(function withoutTheOption(
      _,
      index,
    ) {
      return (index !== at) && (index !== (at + 1));
    },),
  };
}

/**
 Build the image: Debian's Git, a D-Bus daemon, the stand-in and the executable under test as `git`.

 @param executable - the wrapper this arm runs

 @param limits - the container's bounds, whose build form takes no process cap

 @returns the build context path, kept so a failing run can be looked at
 */
async function buildTheImage({
  executable,
  limits,
}: {
  readonly executable: string;
  readonly limits: ReturnType<typeof parseContainerLimitOptions>['limits'];
},): Promise<string> {
  /**
   The build context: the executable, the stand-in, the steps and the Containerfile.
   */
  const context = join(
    packageDirectory,
    'node_modules',
    '.monochromatic',
    'git-policy',
    'private-runtime-proof',
  );
  await mkdir(
    context,
    { recursive: true },
  );
  /**
   Where the image reads the executable from.
   */
  const copied = join(
    context,
    'git-under-test',
  );
  // A copy of an earlier arm is read-only, and a build context that cannot be written again would keep that arm's
  // executable whatever this run was given.
  await rm(
    copied,
    { force: true },
  );
  await copyFile(
    executable,
    copied,
  );
  await writeFile(
    join(
      context,
      'stand-in-systemd-run',
    ),
    standInSystemdRun,
  );
  await writeFile(
    join(
      context,
      'repro-in-a-container.sh',
    ),
    proofScript,
  );
  await copyFile(
    join(
      packageDirectory,
      'e2e',
      'private-runtime.Containerfile',
    ),
    join(
      context,
      'Containerfile',
    ),
  );
  /**
   The CachyOS image this host's CPU runs, pulled first so the build names a resolved base and the image carries the
   same C library as the executable copied into it (AGENTS.md rule BXD).
   */
  const base = await pullCachyosLatest();
  await runCommand({
    command: 'podman',
    args: [
      'build',
      '--build-arg',
      `CACHYOS_IMAGE=${base.cachyosImage}`,
      ...buildLimitArguments(limits,),
      '--file',
      'Containerfile',
      '--tag',
      image,
      '.',
    ],
    cwd: context,
  },);
  return context;
}

/**
 Run the proof's three cases in a container with the given bounds.

 @param limits - the container's bounds

 @returns what the container printed
 */
async function runTheProof(limits: ReturnType<typeof parseContainerLimitOptions>['limits'],): Promise<string> {
  /**
   The container's whole output, so a failure can be read and quoted.
   */
  const ran = await runCommand({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...runLimitArguments({
        limits,
        withPidsLimit: true,
      },),
      image,
      '/bin/sh',
      '/proof/repro-in-a-container.sh',
    ],
    capture: true,
    allowFailure: true,
  },);
  return `${ran.stdout}${ran.stderr}`;
}

/**
 Print the proof's output and refuse when the container did not pass.

 @param output - what the container printed
 */
function judge(output: string,): void {
  process
    .stdout
    .write(output,);
  if (!output.includes('PROOF-PASSED'))
    throw new NativeVerificationError('the proof did not pass: the output above names every case that failed');
}

/**
 The executable this arm runs and the bounds its container gets.
 */
const options = readOptions();
/**
 The build context this arm's image was made from, printed so a failing run can be looked at.
 */
const context = await buildTheImage(options,);
console.log(`built ${image} from ${options.executable} (context ${context})`);
judge(await runTheProof(options.limits,),);
