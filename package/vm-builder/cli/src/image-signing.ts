/**
 Publishes the built image to GHCR and signs the pushed digest with cosign.

 The steps live here rather than in `sign-and-push.ts` because that script runs them when it is imported, so a unit test
 could not reach the argument lists without a registry login and the signing key.
 */
import { join, } from 'node:path';

import {
  DEV_VM_IMAGE_TAG,
  DEV_VM_REGISTRY_IMAGE_TAG,
} from './dev-vm.ts';
import {
  execStreaming,
  type StreamingCommandRunner,
} from './exec.ts';

/**
 Locates the cosign key pair the config package ships.

 @param monorepoRoot - absolute path of the repository root

 @returns Absolute path of the private key `cosign sign` reads

 @example
 ```ts
 const keyPath = cosignKeyPath({ monorepoRoot: '/repo', },);
 ```
 */
export function cosignKeyPath(
  {
    monorepoRoot,
  }: {
    readonly monorepoRoot: string;
  },
): string {
  return join(
    monorepoRoot,
    'package',
    'config',
    'cosign',
    'cosign.key',
  );
}

/**
 Tags the local build under the registry name, pushes it, and signs the pushed digest.

 The signature is an OCI artifact beside the image, so it can only exist after the image is pushed. `COSIGN_PASSWORD=` is
 set empty because the key pair is unencrypted.

 @param keyPath - Absolute path of the cosign private key

 @param runner - Command runner; tests pass one that records the calls instead of pushing or signing

 @throws `SubprocessError` from {@link execStreaming} when the tag, the push or the signature fails

 @example
 ```ts
 await pushAndSignImage({ keyPath: '/repo/package/config/cosign/cosign.key', },);
 ```
 */
export async function pushAndSignImage(
  {
    keyPath,
    runner = execStreaming,
  }: {
    readonly keyPath: string;
    readonly runner?: StreamingCommandRunner;
  },
): Promise<void> {
  console.log(`[vm-builder] tagging ${DEV_VM_IMAGE_TAG} as ${DEV_VM_REGISTRY_IMAGE_TAG}...`,);
  await runner({
    cmd: 'sudo',
    args: [
      'podman',
      'tag',
      DEV_VM_IMAGE_TAG,
      DEV_VM_REGISTRY_IMAGE_TAG,
    ],
  },);

  console.log(`[vm-builder] pushing ${DEV_VM_REGISTRY_IMAGE_TAG}...`,);
  await runner({
    cmd: 'sudo',
    args: [
      'podman',
      'push',
      DEV_VM_REGISTRY_IMAGE_TAG,
    ],
  },);

  console.log('[vm-builder] signing image with cosign...',);
  await runner({
    cmd: 'sudo',
    args: [
      'env',
      'COSIGN_PASSWORD=',
      'cosign',
      'sign',
      '--key',
      keyPath,
      '--tlog-upload=false',
      DEV_VM_REGISTRY_IMAGE_TAG,
    ],
  },);
}
