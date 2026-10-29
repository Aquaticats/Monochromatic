/**
 Container mount points and baked-image paths shared by host and
 container code.
 
 @example
 ```ts
 import { WORK_MOUNT } from './mounts.ts';
 ```
 */

/**
 Read-only repository mount inside the shard container.
 */
export const SOURCE_MOUNT: string = '/src-ro';

/**
 Writable tmpfs work tree inside the shard container.
 */
export const WORK_MOUNT: string = '/work';

/**
 Report mount the container writes its shard report into.
 */
export const REPORT_MOUNT: string = '/out';

/**
 Manifest mount the host places the shard manifest into (read-only).
 */
export const MANIFEST_MOUNT: string = '/manifest';

/**
 Baked repository root inside the runtime image.
 */
export const BAKED_ROOT: string = '/baked';

/**
 Container-side entrypoint executed from baked source with plain node.
 */
export const BAKED_ENTRYPOINT: string = '/baked/package/mutation-test/cli/src/container/main.ts';
