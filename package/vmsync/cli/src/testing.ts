/**
 Built entry the unit tests import.

 The package's only entry is the `vmsync` bin, which parses argv and dispatches, so
 no test can load it. This module re-exports exactly the behavior the unit suites
 drive, so they load what this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the bin bundle. Nothing in this package imports it, and `vmsync` never loads it.

 @module
 */

//region Boot: memory grammar and the boot command

/**
 Human-readable memory string to byte count, and the boot command itself.

 @internal
 */
export {
  bootVm,
  parseMemoryToBytes,
} from './boot.ts';

//endregion Boot

//region Configuration: names, paths, and the JSONC manifest

/**
 Name validation, directory and manifest paths, JSONC stripping, and the
 hypervisor probe.

 @internal
 */
export {
  detectHypervisor,
  isVmNameChar,
  stripJsoncComments,
  validateName,
  vmConfigPath,
  vmDir,
} from './config.ts';

//endregion Configuration

//region NBD: the dd argument vector for a region transfer

/**
 The argument vector a region transfer hands `dd`.

 @internal
 */
export { ddTransferArgs, } from './nbd.ts';

//endregion NBD

//region qemu-img: the whitespace scanner the checksum parser walks with

/**
 First whitespace offset in a `sha256sum` output line, or the string length.

 @internal
 */
export { firstWhitespaceIndex, } from './qemu-img.ts';

//endregion qemu-img

//region Import: the VM name an image path yields

/**
 Guest VM name derived from an image file path.

 @internal
 */
export { nameFromPath, } from './import.ts';

//endregion Import
