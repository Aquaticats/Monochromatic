/**
 Library surface of `@monochromatic-dev/vmsync-cli`, re-exported so tests import
 the built bundle instead of sibling source files.
 
 The `vmsync` executable lives in `./cli.ts`, and every symbol here is exposed
 for built-artifact verification rather than for consumers of this private
 package: the CLI is what ships, and these are the helpers its tests reach.
 
 @module
 */

/**
 Hypervisor detection for the running platform.
 
 @internal
 */
export { detectHypervisor, } from './config.ts';

/**
 The single statement of which characters a VM name may contain.
 
 @internal
 */
export { isVmNameChar, } from './config.ts';

/**
 Parsing of a `vmsync.jsonc` file, delegated to the shared JSONC parser.
 
 @internal
 */
export { parseVmsyncConfig, } from './config.ts';

/**
 The VM-name validation `import` relies on before it reads any image.
 
 @internal
 */
export { validateName, } from './config.ts';

/**
 The refusal a broken VM name throws, carrying the whole explanation.
 
 @internal
 */
export { InvalidVmNameError, } from './config.ts';

/**
 Where a VM name came from, which a refusal states.
 
 @internal
 */
export type { VmNameOrigin, } from './config.ts';

/**
 Path of the per-VM configuration file.
 
 @internal
 */
export { vmConfigPath, } from './config.ts';

/**
 Directory holding one VM's images and configuration.
 
 @internal
 */
export { vmDir, } from './config.ts';

/**
 The VM-name derivation `importImage` runs on an image path.
 
 @internal
 */
export { nameFromPath, } from './import.ts';

/**
 The memory-string parser `boot` runs before handing a domain to libvirt.
 
 @internal
 */
export { parseMemoryToBytes, } from './boot.ts';

/**
 The whitespace scan `qemu-img` runs over a `sha256sum` line.
 
 @internal
 */
export { firstWhitespaceIndex, } from './qemu-img.ts';

/**
 The `dd` argument list `nbd` builds for a block transfer.
 
 @internal
 */
export { ddTransferArgs, } from './nbd.ts';
