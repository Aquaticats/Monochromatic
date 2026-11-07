#!/usr/bin/env node
/**
 `locked-tool-build-arguments`: prints the build arguments that hand a tool of `toolchain.lock.json` to `podman build`.

 Usage: `locked-tool-build-arguments <zig|cargo-about>` prints one `NAME=value` line per build argument:
 `ZIG_ARCHIVE_URL` and `ZIG_ARCHIVE_SHA256` (the archive of the host's platform) for `zig`, `CARGO_ABOUT_VERSION` for `cargo-about`.
 The lock is the nearest one above the working directory; a missing lock, an unknown tool, a host without a Zig archive,
 or a row without the needed value exits 1 and says so on stderr.
 */

import {
  LOCKED_BUILD_ARGUMENT_TOOLS,
  LockedToolBuildArgumentsError,
  ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE,
  readLockedToolBuildArguments,
} from '../locked-tool-build-arguments.ts';

try {
  /**
   Tool the caller named, `undefined` when none was named.
   */
  const toolName = process.argv[2];
  /**
   The named tool when it is one this bin knows, `undefined` otherwise.
   */
  const tool = LOCKED_BUILD_ARGUMENT_TOOLS.find((candidate,) => candidate === toolName);
  if (tool === undefined) {
    throw new LockedToolBuildArgumentsError(
      `name the tool as the only argument, one of: ${LOCKED_BUILD_ARGUMENT_TOOLS.join(', ',)}`,
    );
  }

  /**
   Lock platform of the host the build runs on, `undefined` for a host with no Zig archive in the lock.
   */
  const platform = Reflect.get(
    ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE,
    process.arch,
  ) as string | undefined;
  /**
   Build arguments the nearest lock records for the tool.
   */
  const buildArguments = await readLockedToolBuildArguments({
    startDirectory: process.cwd(),
    tool,
    platform,
  },);
  process.stdout
    .write(`${buildArguments.join('\n',)}\n`,);
}
catch (error) {
  if (!(error instanceof LockedToolBuildArgumentsError)) {
    throw error;
  }
  process.stderr
    .write(`${error.message}\n`,);
  process.exitCode = 1;
}
