#!/usr/bin/env node
/**
 `locked-rust-toolchain`: prints the Rust toolchain `toolchain.lock.json` records, so callers can hand it to `podman build`.

 Usage: `locked-rust-toolchain [--build-argument]` prints `RUST_TOOLCHAIN=<name>` with the option and the bare name without it.
 The lock is the nearest one above the working directory; a missing lock or Rust entry exits 1 and says so on stderr.
 */

import {
  LockedRustToolchainError,
  RUST_TOOLCHAIN_BUILD_ARGUMENT_NAME,
  readLockedRustToolchain,
} from '../locked-rust-toolchain.ts';

try {
  /**
   Toolchain the nearest lock records.
   */
  const toolchain = await readLockedRustToolchain(process.cwd(),);
  /**
   Whether the caller wants `RUST_TOOLCHAIN=<name>` for `podman build --build-arg`.
   */
  const asBuildArgument = process.argv
    .includes('--build-argument',);
  process.stdout
    .write(asBuildArgument ? `${RUST_TOOLCHAIN_BUILD_ARGUMENT_NAME}=${toolchain}\n` : `${toolchain}\n`,);
}
catch (error) {
  if (!(error instanceof LockedRustToolchainError)) {
    throw error;
  }
  process.stderr
    .write(`${error.message}\n`,);
  process.exitCode = 1;
}
