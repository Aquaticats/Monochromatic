#!/usr/bin/env node
/**
 Signs the container image with cosign and pushes it to GHCR.
 Separate from `build-and-import.ts` because cosign signatures are OCI artifacts
 that must be pushed to a registry; they cannot exist in local podman storage alone.
 
 Run: mise run //package/vm-builder/cli:push
 
 Prerequisites:
 - Image already built via `mise run //package/vm-builder/cli:run`
 - `cosign` installed
 - `podman login ghcr.io` completed
 - Cosign key pair at `package/config/cosign/`
 */
import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import { spawn as nodeSpawn, } from 'node:child_process';
import { once, } from 'node:events';
import { join, } from 'node:path';

/**
 Local image tag to push.
 */
const IMAGE_TAG = 'localhost/monochromatic-dev:latest';

/**
 GHCR image reference. Change this to match your GitHub username/org.
 */
const GHCR_TAG = 'ghcr.io/aquaticat/monochromatic-dev:latest';

/**
 Absolute path to the monorepo root.
 */
const MONOREPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/**
 Path to the cosign private key for image signing.
 */
const COSIGN_KEY = join(
  MONOREPO_ROOT,
  'package',
  'config',
  'cosign',
  'cosign.key',
);

/**
 Spawns a command with inherited stdio so output streams to the terminal in real time.
 
 @param cmd - Executable name
 
 @param args - Arguments passed to the command
 
 @throws When the command exits with a non-zero code
 
 @example
 ```ts
 await run({ cmd: 'podman', args: ['push', 'ghcr.io/...'] });
 ```
 */
async function run(
  {
    cmd,
    args,
  }: {
    readonly cmd: string;
    readonly args: readonly string[];
  },
): Promise<void> {
  /**
   Spawned child process with inherited stdio; awaited via `once(child, 'close')` for the exit code.
   */
  const child = nodeSpawn(
    cmd,
    [...args,],
    { stdio: 'inherit', },
  );
  await once(
    child,
    'close',
  );
  if (child.exitCode !== 0)
    throw new Error(`${cmd} exited with code ${String(child.exitCode,)}`,);
}

/**
 Tags the local {@link IMAGE_TAG} image as {@link GHCR_TAG} and pushes it.
 */
async function pushImage(): Promise<void> {
  console.log(`[vm-builder] tagging ${IMAGE_TAG} as ${GHCR_TAG}...`,);
  await run({
    cmd: 'sudo',
    args: [
      'podman',
      'tag',
      IMAGE_TAG,
      GHCR_TAG,
    ],
  },);

  console.log(`[vm-builder] pushing ${GHCR_TAG}...`,);
  await run({
    cmd: 'sudo',
    args: [
      'podman',
      'push',
      GHCR_TAG,
    ],
  },);
}

/**
 Signs the pushed image with cosign using {@link COSIGN_KEY}.
 The signature is stored as an OCI artifact in the same {@link GHCR_TAG} repository.
 */
async function signImage(): Promise<void> {
  console.log('[vm-builder] signing image with cosign...',);
  await run({
    cmd: 'sudo',
    args: [
      'env',
      'COSIGN_PASSWORD=',
      'cosign',
      'sign',
      '--key',
      COSIGN_KEY,
      '--tlog-upload=false',
      GHCR_TAG,
    ],
  },);
}

await pushImage();
await signImage();

console.log(`[vm-builder] pushed and signed ${GHCR_TAG}`,);
