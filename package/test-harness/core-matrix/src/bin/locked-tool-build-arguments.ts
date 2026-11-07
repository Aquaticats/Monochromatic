#!/usr/bin/env node
/**
 `locked-tool-build-arguments`: prints the build arguments that hand a tool of `toolchain.lock.json` to `podman build`.

 Usage: `locked-tool-build-arguments <zig|cargo-about>` prints one `NAME=value` line per build argument:
 `ZIG_ARCHIVE_URL` and `ZIG_ARCHIVE_SHA256` (the archive of the host's platform) for `zig`, `CARGO_ABOUT_VERSION` for `cargo-about`.
 The lock is the nearest one above the working directory; a missing lock, an unknown tool, a host architecture without a Zig archive,
 or a row without the needed value exits 1 and says so on stderr.
 */

import {
  LOCKED_BUILD_ARGUMENT_TOOLS,
  LockedToolBuildArgumentsError,
  isLockedBuildArgumentTool,
  readLockedToolBuildArguments,
} from '../locked-tool-build-arguments.ts';

try {
  /**
   Tool the caller named as the only argument, absent when none was named.
   */
  const toolName = process.argv
    .at(2,);
  if (!isLockedBuildArgumentTool(toolName,)) {
    throw new LockedToolBuildArgumentsError(
      `name the tool as the only argument, one of: ${LOCKED_BUILD_ARGUMENT_TOOLS.join(', ',)}`,
    );
  }

  /**
   Build arguments the nearest lock records for the tool, for the architecture of the host the build runs on.
   */
  const buildArguments = await readLockedToolBuildArguments({
    startDirectory: process.cwd(),
    tool: toolName,
    hostArchitecture: process.arch,
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
