/**
 Runtime installation scripts and execution commands.

 Each supported JS runtime has an install script (fetched via curl)
 and a command pattern for executing test files.
 */

import type { Runtime, } from './types.ts';

//region Runtime install scripts

/**
 Shell command installing the Node release `toolchain.lock.json` pins.

 Mirrors the root Dockerfile's verified pattern: `jq` reads the current
 platform's `linux-<arch>` artifact row from the mounted
 `/workspace/toolchain.lock.json`, the archive is fetched and its SHA-256
 checked against the lock before extraction into `$HOME/node`. Nothing here
 resolves a version ad hoc: the lock is the only version source.
 */
const NODE_LOCKED_INSTALL = [
  'cd /workspace',
  'node_platform=$(case "$(uname -m)" in aarch64) echo linux-arm64;; *) echo linux-x64;; esac)',
  'node_url="$(jq --raw-output --arg p "$node_platform" \'.tools.node.artifacts[$p].url\' toolchain.lock.json)"',
  'node_sum="$(jq --raw-output --arg p "$node_platform" \'.tools.node.artifacts[$p].sha256\' toolchain.lock.json)"',
  'curl --fail --silent --show-error --location --output /tmp/node.tar.gz "$node_url"',
  'echo "$node_sum  /tmp/node.tar.gz" | sha256sum --check',
  'mkdir -p "$HOME/node"',
  'tar --extract --file /tmp/node.tar.gz --directory "$HOME/node" --strip-components=1',
]
  .join(' && ',);

/**
 Install commands for each supported runtime.
 Bun and deno use their upstream installer scripts; node is the locked
 release from `toolchain.lock.json` (see {@link NODE_LOCKED_INSTALL}).

 Post-install, every runtime binary lives at a predictable path under `$HOME`
 (bun: `$HOME/.bun/bin/bun`, deno: `$HOME/.deno/bin/deno`,
 node: `$HOME/node/bin/node`).
 */
const RUNTIME_INSTALL: Record<Runtime, string> = {
  bun: 'curl -fsSL https://bun.sh/install | bash',
  deno: 'curl -fsSL https://deno.land/install.sh | sh',
  node: NODE_LOCKED_INSTALL,
};

/**
 Absolute path to the runtime binary after installation.
 Uses `$HOME` so it resolves correctly for both root and non-root users.
 */
const RUNTIME_BIN: Record<Runtime, string> = {
  bun: '$HOME/.bun/bin/bun',
  deno: '$HOME/.deno/bin/deno',
  node: '$HOME/node/bin/node',
};

/**
 Returns the shell command to install a JS runtime inside a container.

 @param runtime - Runtime to install

 @returns shell command string

 @example
 ```ts
 runtimeInstallCommand('bun');
 // 'curl -fsSL https://bun.sh/install | bash'

 runtimeInstallCommand('node');
 // 'cd /workspace && node_platform=... && node_url="$(jq ...)" && ... && tar --extract ...'
 ```
 */
export function runtimeInstallCommand(runtime: Runtime,): string {
  return RUNTIME_INSTALL[runtime];
}

/**
 Returns the shell command to execute a file with the given runtime.

 Every runtime runs its own locked binary under `$HOME`: node is the release
 `toolchain.lock.json` pins, extracted by {@link runtimeInstallCommand}.

 @param runtime - Runtime to use

 @param filePath - Absolute path to the file inside the container

 @returns shell command string

 @example
 ```ts
 runtimeExecCommand({ runtime: 'bun', filePath: '/workspace/test.ts' });
 // '$HOME/.bun/bin/bun run /workspace/test.ts'

 runtimeExecCommand({ runtime: 'node', filePath: '/workspace/test.ts' });
 // '$HOME/node/bin/node /workspace/test.ts'
 ```
 */
export function runtimeExecCommand({
  runtime,
  filePath,
}: {
  readonly runtime: Runtime;
  readonly filePath: string;
},): string {
  /**
   Absolute runtime binary inside the container.
   */
  const bin = RUNTIME_BIN[runtime];

  if (runtime === 'node')
    return `${bin} ${filePath}`;

  if (runtime === 'deno')
    return `${bin} run --allow-all ${filePath}`;

  return `${bin} run ${filePath}`;
}

//endregion Runtime install scripts
