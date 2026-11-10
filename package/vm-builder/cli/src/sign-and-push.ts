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

import { DEV_VM_REGISTRY_IMAGE_TAG, } from './dev-vm.ts';
import {
  cosignKeyPath,
  pushAndSignImage,
} from './image-signing.ts';

/**
 Absolute path to the monorepo root, which holds the cosign key pair.
 */
const MONOREPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

await pushAndSignImage({ keyPath: cosignKeyPath({ monorepoRoot: MONOREPO_ROOT, },), },);

console.log(`[vm-builder] pushed and signed ${DEV_VM_REGISTRY_IMAGE_TAG}`,);
