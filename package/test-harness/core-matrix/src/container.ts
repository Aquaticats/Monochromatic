/**
 Container backend for matrix test execution.
 
 Builds shell commands and spawns podman containers for each
 combination in the test matrix.
 */

import spawn from 'nano-spawn';
import {
  detectPackageManager,
  prerequisiteCommand,
  runtimeDependencyCommand,
  userCreationCommand,
} from './distro.ts';
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
 Image repositories for distros whose Docker Hub repository name differs from the distro name.
 Per the owner's 2026-10-29 container base distro decision
 (`doc/decision/container-base-distro.md`), CachyOS resolves to the unpinned `latest` tag
 of the plain `cachyos/cachyos` image, which runs on any x86-64 CPU
 (`cachyos/cachyos-v3` needs AVX2 and fails on CPUs without it).
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
 Distros listed in `IMAGE_REPOSITORY_BY_DISTRO` resolve to their registry repository
 (keeping an explicit tag); every other distro name is used as the image name.
 
 @param distro - Distro name (e.g. `'ubuntu'`, `'fedora:39'`, `'cachyos'`)
 
 @returns container image reference (e.g. `'ubuntu:latest'`, `'fedora:39'`,
 `'docker.io/cachyos/cachyos:latest'`)
 
 @example
 ```ts
 resolveImage('ubuntu'); // 'ubuntu:latest'
 resolveImage('cachyos'); // 'docker.io/cachyos/cachyos:latest'
 ```
 */
export function resolveImage(distro: string,): string {
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
  /**
   Registry repository for distros whose image name differs from the distro name.
   */
  const repository = IMAGE_REPOSITORY_BY_DISTRO.get(baseName,);

  if (repository === undefined)
    return colonIndex === (-1) ? `${distro}:latest` : distro;

  return colonIndex === (-1)
    ? `${repository}:latest`
    : `${repository}${distro.slice(colonIndex,)}`;
}

/**
 Runs a single combination in a podman container.
 
 Mounts the monorepo root at `/workspace:Z` (SELinux relabel)
 and executes the command built by {@link buildContainerCommand} via `sh -c`.
 
 @param combination - Fully resolved combination to execute
 
 @param monorepoRoot - Absolute path to the monorepo root on the host
 
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
}: {
  readonly combination: Combination;
  readonly monorepoRoot: string;
},): Promise<string> {
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
  const image = resolveImage(parsed.distro,);
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
      'run',
      '--rm',
      // AGENTS.md RXI bounds for host-exhausting runs: an unbounded cell grew to
      // 42 GB RSS on 2026-10-26 and the kernel OOM killer terminated a harness
      // subprocess scope. Every cell stays within these bounds.
      '--memory=2g',
      '--cpus=2',
      '-v',
      `${monorepoRoot}:/workspace:Z`,
      image,
      'sh',
      '-c',
      command,
    ],
  );

  if (result.stderr
    !== '')
    console.error(result.stderr,);

  return result.stdout;
}
