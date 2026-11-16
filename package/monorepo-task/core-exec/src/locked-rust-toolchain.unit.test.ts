/**
 Checks the reading of the Rust toolchain `toolchain.lock.json` records, and the `rustup` command line that installs it.

 The lock holds one dated nightly for every build of this repository, and the workflow step that installs it reads
 `.tools.rust.version` rather than a date typed into a workflow
 (`doc/handover/cli-git-rust-implementation.md`, owner decision of 2026-11-15: "Everything nightly."). A wrong or missing
 row must end the step with a message naming the row and the fix, and the toolchain name must reach `rustup` as one
 argument, never as a shell word that could be read as an option.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RUST_TOOLCHAIN_ROW,
  RustToolchainPinError,
  isInstalableToolchain,
  lockedRustToolchainVersion,
  rustupInstallArguments,
  rustupToolchainEnvironmentLine,
} from '../dist/final/node/testing.mjs';

//region Fixtures

/**
 A lock holding one dated nightly, in the shape the toolchain update writes.
 */
const lockOfTheDay = {
  tools: {
    rust: {
      request: 'rust@nightly',
      version: 'nightly-2026-11-15',
      manager: 'rustup',
    },
  },
};

/**
 A lock holding a release number instead of a nightly, which the field also allows.
 */
const lockOfARelease = { tools: { rust: { version: '1.99.0' } } };

/**
 The message a lock's refusal carried, so one loop can judge every shape the reading refuses.

 @param lock - The lock the reading must refuse

 @returns the refusal's message, empty when the reading accepted the lock

 @example
 ```ts
 const message = refusalOf({ lock: {} });
 ```
 */
function refusalOf({ lock, }: { readonly lock: unknown },): string {
  try {
    lockedRustToolchainVersion({ lock, },);
  }
  catch (error) {
    expect(error,).toBeInstanceOf(RustToolchainPinError,);
    const { message, } = error as Error;
    return message;
  }
  return '';
}

//endregion Fixtures

await describe({
  name: 'locked Rust toolchain',
  children: [
    it({
      name: 'reads the dated nightly the lock records',
      fn: async function readsTheNightly() {
        expect(lockedRustToolchainVersion({ lock: lockOfTheDay, },),).toBe('nightly-2026-11-15',);
      },
    },),
    it({
      name: 'reads a release number too, which the field allowed before the pin existed',
      fn: async function readsARelease() {
        expect(lockedRustToolchainVersion({ lock: lockOfARelease, },),).toBe('1.99.0',);
      },
    },),
    it({
      name: 'names the lock row in the message of every refusal, so the reader knows what to write',
      fn: async function refusalsNameTheRow() {
        /**
         Locks the reading must refuse, one per shape it cannot install.
         */
        const refused: readonly unknown[] = [
          'not an object',
          [],
          null,
          {},
          { tools: {}, },
          { tools: { rust: 'nightly-2026-11-15', }, },
          { tools: { rust: {}, }, },
          { tools: { rust: { version: 20_261_115, }, }, },
          { tools: { rust: { version: 'nightly', }, }, },
          { tools: { rust: { version: 'nightly-2026-11', }, }, },
          { tools: { rust: { version: 'nightly-2026-11-15-x86_64-unknown-linux-gnu', }, }, },
          { tools: { rust: { version: '', }, }, },
        ];
        for (const lock of refused) {
          const message = refusalOf({ lock, },);
          expect(message,).toContain(RUST_TOOLCHAIN_ROW,);
        }
      },
    },),
    it({
      name: 'installs the toolchain by the name the lock records, with the minimal profile and no self-update',
      fn: async function installsByTheRecordedName() {
        expect(rustupInstallArguments({ version: 'nightly-2026-11-15', },),)
          .toEqual([
            'toolchain',
            'install',
            'nightly-2026-11-15',
            '--profile',
            'minimal',
            '--no-self-update',
          ],);
      },
    },),
    it({
      name: 'hands the toolchain to the rest of the job as one RUSTUP_TOOLCHAIN assignment',
      fn: async function handsOverTheToolchain() {
        expect(rustupToolchainEnvironmentLine({ version: 'nightly-2026-11-15', },),)
          .toBe('RUSTUP_TOOLCHAIN=nightly-2026-11-15\n',);
      },
    },),
    it({
      name: 'accepts only the two forms rustup can be given as a named toolchain here',
      fn: async function namesTheAcceptedForms() {
        for (const accepted of ['nightly-2026-11-15', '1.99.0', '1.102.0',]) expect(isInstalableToolchain(accepted,),).toBe(true,);
        for (const refused of ['nightly', 'stable', 'nightly-2026-1-15', 'nightly-2026-11-15 ', ' 1.99.0', 'v1.99.0', '',]) expect(isInstalableToolchain(refused,),).toBe(false,);
      },
    },),
  ],
},);
