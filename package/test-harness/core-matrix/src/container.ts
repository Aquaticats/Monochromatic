/**
 Container backend for matrix test execution.
 
 Builds shell commands and spawns podman containers for each
 combination in the test matrix.
 */

import {
  containerLimitArguments,
  type ContainerLimits,
} from '@monochromatic-dev/container-limit-core/ts';
import {
  logger as defaultLogger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import spawn from 'nano-spawn';
import {
  detectPackageManager,
  prerequisiteCommand,
  runtimeDependencyCommand,
  userCreationCommand,
} from './distro.ts';
import { pnpmStoreMountArguments, } from './pnpm-store-mount.ts';
import {
  runtimeExecCommand,
  runtimeInstallCommand,
} from './runtime.ts';
import type {
  Combination,
  ParsedOs,
} from './types.ts';

/**
 Parses an OS specification string into protocol and distro.
 
 @param os - OS specification with protocol prefix (e.g. `'container:ubuntu'`)
 
 @returns parsed protocol and distro
 
 @throws Error when the protocol prefix is missing or unrecognized
 
 @example
 ```ts
 parseOs('container:ubuntu');
 // \{ protocol: 'container', distro: 'ubuntu' \}
 
 parseOs('container:fedora:39');
 // \{ protocol: 'container', distro: 'fedora:39' \}
 ```
 */
export function parseOs(os: string,): ParsedOs {
  /**
   Split point between protocol prefix and distro; -1 signals missing prefix.
   */
  const colonIndex = os.indexOf(':',);

  if (colonIndex === (-1)) {
    throw new Error(
      `OS specification "${os}" must have a protocol prefix (e.g. "container:ubuntu")`,
    );
  }

  /**
   Captured separately so the protocol switch below reads as a plain comparison.
   */
  const protocol = os.slice(
    0,
    colonIndex,
  );
  /**
   Remaining suffix; may itself contain a `:` tag (e.g. `fedora:39`), kept intact.
   */
  const distro = os.slice(colonIndex + 1,);

  if (protocol === 'container') {
    return {
      protocol,
      distro,
    };
  }

  if (protocol === 'host') {
    return {
      protocol,
      distro,
    };
  }

  if (protocol === 'vm') {
    return {
      protocol,
      distro,
    };
  }

  throw new Error(
    `Unknown protocol "${protocol}" in OS specification "${os}". Supported: container, host, vm`,
  );
}

/**
 Builds the shell command to run inside a container for one combination.
 
 The command sequence:
 1.  Install prerequisites (curl, unzip, optionally sudo) via
     {@link prerequisiteCommand}
 2.  Install runtime system libraries (node needs `libatomic`) via
     {@link runtimeDependencyCommand}
 3.  Create non-root user if needed via {@link userCreationCommand}
 4.  Install the JS runtime via {@link runtimeInstallCommand}
 5.  Execute each test file via {@link runtimeExecCommand}
 
 For non-root users, writes a script to `/tmp/run-test.sh` and
 executes it via `sudo -u testuser -i` to avoid nested quoting issues.
 
 @param combination - Fully resolved combination
 
 @param monorepoRoot - Absolute path to the monorepo root on the host
 
 @returns shell command string for `sh -c`
 
 @throws Error when the runtime cannot run on the distro's libc
 
 @example
 ```ts
 buildContainerCommand({ combination: { file: '/test.ts', os: 'ubuntu', user: 'root', runtime: 'bun' }, monorepoRoot: '/repo' });
 ```
 */
export function buildContainerCommand({
  combination,
  monorepoRoot,
}: {
  readonly combination: Combination;
  readonly monorepoRoot: string;
},): string {
  /**
   Pre-parsed protocol+distro so the downstream lookups stay independent of string parsing.
   */
  const parsed = parseOs(combination.os,);
  /**
   Resolved package manager; gates the variant of every install command below.
   */
  const manager = detectPackageManager(parsed.distro,);
  /**
   Step 1 of the container script: install curl/unzip/sudo via the distro's package manager.
   */
  const prerequisites = prerequisiteCommand({
    manager,
    user: combination.user,
  },);
  /**
   Step 2: system libraries the runtime links against; empty for bun and deno.
   */
  const runtimeDependencies = runtimeDependencyCommand({
    manager,
    runtime: combination.runtime,
  },);
  /**
   Step 3: create the non-root test user when needed; empty string for root.
   */
  const userSetup = userCreationCommand({
    manager,
    user: combination.user,
  },);
  /**
   Step 4: install the JS runtime that will execute the test file.
   */
  const runtimeInstall = runtimeInstallCommand(combination.runtime,);
  /**
   System packages that must be installed as root before the runtime setup;
   empty for bun and deno, so their command chains stay unchanged.
   */
  const runtimeDependenciesList: readonly string[] = runtimeDependencies === ''
    ? []
    : [runtimeDependencies,];

  /**
   Path to the test file inside the container (mounted at /workspace).
   */
  const relativePath = combination.file
    .startsWith(monorepoRoot,)
    ? combination.file
      .slice(monorepoRoot.length
        + 1,)
    : combination.file;
  /**
   Container-side absolute path; the host's monorepo is mounted at /workspace.
   */
  const containerFilePath = `/workspace/${relativePath}`;
  /**
   Step 5: actual `runtime run <file>` invocation built from the resolved binary path.
   */
  const execCommand = runtimeExecCommand({
    runtime: combination.runtime,
    filePath: containerFilePath,
  },);

  if (combination.user
    === 'user') {
    /**
     Write a script to `/tmp/run-test.sh` to avoid nested quoting issues
     with `sudo -u`. Heredoc with quoted delimiter preserves all special characters.
     */
    const scriptLines = [
      '#!/bin/sh',
      'set -e',
      'cd /workspace',
      runtimeInstall,
      execCommand,
    ]
      .join('\n',);

    /**
     Heredoc terminator must occupy its own line, so the trailing `chmod` and
     `sudo` commands start after it rather than being swallowed as script text.
     */
    const runScriptPart = [
      `cat > /tmp/run-test.sh << 'TESTSCRIPT'`,
      scriptLines,
      'TESTSCRIPT',
      'chmod +x /tmp/run-test.sh',
      'sudo -u testuser -i /tmp/run-test.sh',
    ]
      .join('\n',);

    /**
     Non-root variant: heredoc the inner script so `sudo -u` does not need nested quoting.
     */
    const parts = [
      prerequisites,
      userSetup,
      ...runtimeDependenciesList,
      runScriptPart,
    ];

    return parts.join(' && ',);
  }

  /**
   Root variant: runtime install and exec happen inline; no user switching required.
   */
  const parts = [
    prerequisites,
    ...runtimeDependenciesList,
    runtimeInstall,
    'cd /workspace',
    execCommand,
  ];

  return parts.join(' && ',);
}

/**
 Image repositories for distros whose Docker Hub repository name differs from the distro name
 and that have no CPU-level ladder.
 A `Map` keeps lookups of names like `constructor` from reaching the prototype.
 */
const IMAGE_REPOSITORY_BY_DISTRO: ReadonlyMap<string, string> = new Map([
  [
    'cachyos',
    'docker.io/cachyos/cachyos',
  ],
],);

/**
 Resolves a distro name to a container image reference.
 Appends `:latest` if no tag is specified.
 
 A bare `cachyos` resolves to the CachyOS image the caller resolved for the host's CPU level
 (`cachyos-v4`, `cachyos-v3` or plain `cachyos`, see `resolveBaseImage` in `./cachyos-image.ts`).
 `cachyos:<tag>` is an explicit choice and resolves to the plain `cachyos/cachyos` repository
 with that tag, skipping the ladder. Every other distro name is used as the image name.
 
 @param distro - Distro name (e.g. `'ubuntu'`, `'fedora:39'`, `'cachyos'`)
 
 @param cachyosImage - Image for a bare `cachyos`, from `resolveBaseImage`; required only for that distro
 
 @returns container image reference (e.g. `'ubuntu:latest'`, `'fedora:39'`,
 `'docker.io/cachyos/cachyos-v4:latest'`)
 
 @throws Error when a bare `cachyos` is given without `cachyosImage`
 
 @example
 ```ts
 resolveImage({ distro: 'ubuntu', }); // 'ubuntu:latest'
 resolveImage({ distro: 'cachyos', cachyosImage: 'docker.io/cachyos/cachyos:latest', });
 ```
 */
export function resolveImage({
  distro,
  cachyosImage,
}: {
  readonly distro: string;
  readonly cachyosImage?: string;
},): string {
  /**
   Split point between distro name and tag; -1 signals no tag.
   */
  const colonIndex = distro.indexOf(':',);
  /**
   Distro name without its tag, the key into the repository map.
   */
  const baseName = colonIndex === (-1)
    ? distro
    : distro.slice(
      0,
      colonIndex,
    );

  if ((baseName === 'cachyos') && (colonIndex === (-1))) {
    if (cachyosImage === undefined) {
      throw new Error(
        'A bare "cachyos" distro needs the image chosen for the host CPU level: '
          + 'pass cachyosImage from resolveBaseImage (the matrix does this once at startup).',
      );
    }
    return cachyosImage;
  }

  /**
   Registry repository for distros whose image name differs from the distro name.
   */
  const repository = IMAGE_REPOSITORY_BY_DISTRO.get(baseName,);

  if (repository === undefined) {
    return colonIndex === (-1)
      ? `${distro}:latest`
      : distro;
  }

  return `${repository}${distro.slice(colonIndex,)}`;
}

/**
 Bounds of every matrix cell.

 2 GiB and 2 CPUs are the bound the cell has had since an unbounded cell grew to 42 GB RSS on 2026-10-26 and the kernel
 OOM killer ended a harness subprocess scope.
 2048 processes is what `podman run --help` prints as the default, so the cell's effective cap does not change and
 becomes explicit; cells install packages and start runtimes, which start many threads, and threads count against the
 limit.
 No bounded run of this workload has been measured.
 The one matrix cell reachable from a task today, `env-local-json`'s `test:container`, could not start its workload on
 2026-11-08T19:18:31-05:00: the container's node could not resolve a dependency because this checkout's `node_modules`
 links into `${HOME}/.local/share/pnpm/store`, which the cell's checkout mount did not contain. The cell now mounts that
 store read-only at the path the links name, and the workload ran on 2026-11-09T01:32:39Z in both combinations
 (`doc/handover/container-limits-everywhere.md`, section The shared pnpm store inside a container).
 */
export const CELL_LIMITS: ContainerLimits = {
  memory: '2g',
  cpus: 2,
  pidsLimit: 2_048,
};

/**
 Arguments after `podman` that run one matrix cell.

 The cell mounts the checkout at `/workspace` and the shared pnpm store at the absolute path this checkout's
 `node_modules` links name, read-only, so an installed package resolves inside the cell
 ({@link pnpmStoreMountArguments}). Labels stay off the cell: the store is a directory every worktree shares, so a
 `:Z` relabel would rewrite it, and an unlabelled read-only store mount is denied under an enforcing SELinux.

 @param image - fully tagged image reference

 @param monorepoRoot - absolute path to the monorepo root on the host

 @param command - single shell string passed to `sh -c`

 @returns `run` with the cell's limits and the lowest disk weight ahead of the mounts and the image

 @example
 ```ts
 const args = containerRunArguments({ image: 'docker.io/library/ubuntu:latest', monorepoRoot: '/repo', command: 'true' });
 ```
 */
export function containerRunArguments({
  image,
  monorepoRoot,
  command,
}: {
  readonly image: string;
  readonly monorepoRoot: string;
  readonly command: string;
},): readonly string[] {
  return [
    'run',
    '--rm',
    // AGENTS.md RXI bounds for host-exhausting runs, and rule HXR's lowest disk weight: a container sits in its own
    // cgroup scope and ignores the launcher's IOWeight.
    ...containerLimitArguments(CELL_LIMITS,),
    '--security-opt',
    'label=disable',
    '-v',
    `${monorepoRoot}:/workspace`,
    ...pnpmStoreMountArguments(),
    image,
    'sh',
    '-c',
    command,
  ];
}

/**
 Runs a single combination in a podman container.
 
 Mounts the monorepo root at `/workspace:Z` (SELinux relabel)
 and executes the command built by {@link buildContainerCommand} via `sh -c`.
 A run that exits successfully forwards its stderr to `l` as a warning, so the caller decides where it goes and
 whether it is shown at all; the return value carries its stdout.
 
 @param combination - Fully resolved combination to execute
 
 @param monorepoRoot - Absolute path to the monorepo root on the host
 
 @param cachyosImage - CachyOS image resolved once for the whole matrix, for a bare `cachyos` distro
 
 @param l - Logger; defaults to the shared logger
 
 @returns stdout from the container execution
 
 @throws Error with container stderr when the command fails
 
 @example
 ```ts
 const output = await runContainer({
   combination: {
     file: '/path/to/test.ts',
     os: 'container:ubuntu',
     user: 'root',
     runtime: 'bun',
   },
   monorepoRoot: '/var/home/user/Monochromatic',
 });
 ```
 */
export async function runContainer({
  combination,
  monorepoRoot,
  cachyosImage,
  l: parentLogger = defaultLogger,
}: {
  readonly combination: Combination;
  readonly monorepoRoot: string;
  readonly cachyosImage?: string;
  readonly l?: Logger;
},): Promise<string> {
  /**
   Logger tagged with this function, so a forwarded stderr line names the backend that ran.
   */
  const l: Logger = tagged({
    tag: runContainer.name,
    l: parentLogger,
  },);
  /**
   Pre-parsed OS so the vm-protocol guard and the image resolution can both reuse it.
   */
  const parsed = parseOs(combination.os,);

  if (parsed.protocol
    === 'vm')
    throw new Error('vm: protocol not yet implemented',);

  /**
   Fully tagged image reference suitable for `podman run`.
   */
  const image = resolveImage({
    distro: parsed.distro,
    ...(cachyosImage === undefined ? {} : { cachyosImage, }),
  },);
  /**
   Single shell string passed to `sh -c`; built ahead of time so the spawn call stays readable.
   */
  const command = buildContainerCommand({
    combination,
    monorepoRoot,
  },);

  /**
   Spawn result kept in a binding so stderr can be forwarded before returning stdout.
   */
  const result = await spawn(
    'podman',
    [
      ...containerRunArguments({
        image,
        monorepoRoot,
        command,
      },),
    ],
  );

  if (result.stderr
    !== '')
    l.warn(result.stderr,);

  return result.stdout;
}
