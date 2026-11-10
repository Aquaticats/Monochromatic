/**
 Build arguments that hand a tool of `toolchain.lock.json` to a container build, for tools a Containerfile installs itself.

 Rationale: the Containerfiles of `package/terminal/app` and `package/music-player/desktop-app` once carried a hand-kept
 Zig version and checksums, and the Containerfiles of `package/ide/app` a hand-kept `cargo-about` version, so the images
 froze a tool that the lock moves every day. They now declare build arguments with no default, and their callers read the
 values here. The build context is a package directory, so the Containerfile itself cannot read the lock.
 */

import { readFile, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

/**
 Name of the lock file that records the tools.
 */
export const LOCK_FILE_NAME = 'toolchain.lock.json';

/**
 Tools whose lock row a Containerfile can consume.
 */
export const LOCKED_BUILD_ARGUMENT_TOOLS = [
  'zig',
  'cargo-about',
] as const;

/**
 A tool of {@link LOCKED_BUILD_ARGUMENT_TOOLS}.
 */
export type LockedBuildArgumentTool = (typeof LOCKED_BUILD_ARGUMENT_TOOLS)[number];

/**
 Lock platforms that carry a Zig archive, keyed by the Node `process.arch` of the host the build runs on.
 */
export const ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE = {
  x64: 'linux-x64',
  arm64: 'linux-arm64',
} as const;

/**
 Whether a value names a tool of {@link LOCKED_BUILD_ARGUMENT_TOOLS}.

 @param value - text a caller supplied as a tool name

 @returns `true` when the value is one of the tools

 @example
 ```ts
 isLockedBuildArgumentTool('zig',);
 // true
 ```
 */
export function isLockedBuildArgumentTool(value: unknown,): value is LockedBuildArgumentTool {
  return LOCKED_BUILD_ARGUMENT_TOOLS.some(function isValue(candidate: LockedBuildArgumentTool,): boolean {
    return candidate === value;
  },);
}

/**
 Key of each tool's row in the lock's `tools` table.
 */
const LOCK_ROW_KEY_BY_TOOL = {
  'zig': 'zig',
  'cargo-about': 'cargo:cargo-about',
} as const satisfies Readonly<Record<LockedBuildArgumentTool, string>>;

/**
 Thrown when no lock file is found above a directory, or the lock lacks what a build argument needs.
 */
export class LockedToolBuildArgumentsError extends Error {
  /**
   Builds the error.

   @param message - what was missing and where it was looked for
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'LockedToolBuildArgumentsError';
  }
}

/**
 Parses the lock's text.

 @param lockText - contents of `toolchain.lock.json`

 @returns the parsed document, whose shape is not yet checked

 @throws LockedToolBuildArgumentsError when the text is not JSON
 */
function parseLock(lockText: string,): unknown {
  try {
    return JSON.parse(lockText,);
  }
  catch (error) {
    throw new LockedToolBuildArgumentsError(`${LOCK_FILE_NAME} is not JSON: ${String(error,)}`,);
  }
}

/**
 Reads one property of a value that may not be an object.

 @param value - any parsed JSON value

 @param key - property name to read

 @returns the property, or `undefined` when the value is not an object
 */
function childOf({
  value,
  key,
}: {
  readonly value: unknown;
  readonly key: string;
},): unknown {
  return (((typeof value) === 'object') && (value !== null)) ? Reflect.get(
    value,
    key,
  ) : undefined;
}

/**
 Reads a property that must be a non-empty string.

 @param value - parsed JSON value that should hold the property

 @param key - property name to read

 @param description - where the property lives, named in the error

 @returns the property's text

 @throws LockedToolBuildArgumentsError when the property is missing, not a string, or empty
 */
function nonEmptyStringOf({
  value,
  key,
  description,
}: {
  readonly value: unknown;
  readonly key: string;
  readonly description: string;
},): string {
  /**
   The property, `undefined` when the value has none.
   */
  const property: unknown = childOf({
    value,
    key,
  },);
  if (((typeof property) !== 'string') || (property === '')) {
    throw new LockedToolBuildArgumentsError(`${LOCK_FILE_NAME} has no ${description} string`,);
  }
  return property;
}

/**
 Finds the row of one tool in the lock's text.

 @param lockText - contents of `toolchain.lock.json`

 @param tool - tool whose row is read

 @returns the row, whose shape is not yet checked

 @throws LockedToolBuildArgumentsError when the text is not JSON
 */
function rowOfTool({
  lockText,
  tool,
}: {
  readonly lockText: string;
  readonly tool: LockedBuildArgumentTool;
},): unknown {
  return childOf({
    value: childOf({
      value: parseLock(lockText,),
      key: 'tools',
    },),
    key: LOCK_ROW_KEY_BY_TOOL[tool],
  },);
}

/**
 Reads the `cargo-about` build argument out of the lock's text.

 The `cargo-about` row has no archive (its route installs from crates.io), so it yields only `CARGO_ABOUT_VERSION`.

 @param lockText - contents of `toolchain.lock.json`

 @returns lines of the form `NAME=value`, ready for `podman build --build-arg`

 @throws LockedToolBuildArgumentsError when the text is not JSON or the row lacks a non-empty version

 @example
 ```ts
 cargoAboutBuildArgumentsFromText('{"tools":{"cargo:cargo-about":{"version":"0.9.3"}}}',);
 // ['CARGO_ABOUT_VERSION=0.9.3']
 ```
 */
export function cargoAboutBuildArgumentsFromText(lockText: string,): readonly string[] {
  return [
    `CARGO_ABOUT_VERSION=${
      nonEmptyStringOf({
        value: rowOfTool({
          lockText,
          tool: 'cargo-about',
        },),
        key: 'version',
        description: `tools.${LOCK_ROW_KEY_BY_TOOL['cargo-about']}.version`,
      },)
    }`,
  ];
}

/**
 Finds the lock platform whose Zig archive a host needs.

 @param hostArchitecture - Node `process.arch` of the host the build runs on

 @returns the lock platform, for example `linux-x64`

 @throws LockedToolBuildArgumentsError when the lock has no Zig archive for the architecture
 */
function zigLockPlatformOf(hostArchitecture: string,): string {
  /**
   Lock platform by architecture, as a map that answers for any text.
   */
  const platformByArchitecture = new Map<string, string>(Object.entries(ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE,),);
  /**
   Lock platform of the architecture, absent for a host the lock has no Zig archive for.
   */
  const platform = platformByArchitecture.get(hostArchitecture,);
  if (platform === undefined) {
    throw new LockedToolBuildArgumentsError(
      `${LOCK_FILE_NAME} has no Zig archive for host architecture ${hostArchitecture}, only for: ${
        [...platformByArchitecture.keys(),].join(', ',)
      }`,
    );
  }
  return platform;
}

/**
 Reads the Zig build arguments out of the lock's text: `ZIG_ARCHIVE_URL` and `ZIG_ARCHIVE_SHA256`, the archive of the
 host's platform, which the Containerfile downloads and checks.

 @param lockText - contents of `toolchain.lock.json`

 @param hostArchitecture - Node `process.arch` of the host the build runs on

 @returns lines of the form `NAME=value`, ready for `podman build --build-arg`

 @throws LockedToolBuildArgumentsError when the text is not JSON, the host has no archive, or the archive lacks a needed non-empty string

 @example
 ```ts
 zigBuildArgumentsFromText({
   lockText: '{"tools":{"zig":{"artifacts":{"linux-x64":{"url":"https://example.invalid/zig.tar.xz","sha256":"abc"}}}}}',
   hostArchitecture: 'x64',
 },);
 // ['ZIG_ARCHIVE_URL=https://example.invalid/zig.tar.xz', 'ZIG_ARCHIVE_SHA256=abc']
 ```
 */
export function zigBuildArgumentsFromText({
  lockText,
  hostArchitecture,
}: {
  readonly lockText: string;
  readonly hostArchitecture: string;
},): readonly string[] {
  /**
   Lock platform whose archive is read.
   */
  const platform = zigLockPlatformOf(hostArchitecture,);
  /**
   The archive of the platform, `undefined` along any missing step.
   */
  const archive: unknown = childOf({
    value: childOf({
      value: rowOfTool({
        lockText,
        tool: 'zig',
      },),
      key: 'artifacts',
    },),
    key: platform,
  },);
  /**
   Where the lock says the archive of this platform lives.
   */
  const archiveDescription = `tools.${LOCK_ROW_KEY_BY_TOOL.zig}.artifacts.${platform}`;
  return [
    `ZIG_ARCHIVE_URL=${
      nonEmptyStringOf({
        value: archive,
        key: 'url',
        description: `${archiveDescription}.url`,
      },)
    }`,
    `ZIG_ARCHIVE_SHA256=${
      nonEmptyStringOf({
        value: archive,
        key: 'sha256',
        description: `${archiveDescription}.sha256`,
      },)
    }`,
  ];
}

/**
 Reads the build arguments of one tool out of the lock's text.

 @param lockText - contents of `toolchain.lock.json`

 @param tool - tool whose row is read

 @param hostArchitecture - Node `process.arch` of the host the build runs on; only the Zig row depends on it

 @returns lines of the form `NAME=value`, ready for `podman build --build-arg`

 @throws LockedToolBuildArgumentsError when the text is not JSON or the row lacks a needed non-empty string

 @example
 ```ts
 lockedToolBuildArgumentsFromText({
   lockText: '{"tools":{"cargo:cargo-about":{"version":"0.9.3"}}}',
   tool: 'cargo-about',
   hostArchitecture: 'x64',
 },);
 // ['CARGO_ABOUT_VERSION=0.9.3']
 ```
 */
export function lockedToolBuildArgumentsFromText({
  lockText,
  tool,
  hostArchitecture,
}: {
  readonly lockText: string;
  readonly tool: LockedBuildArgumentTool;
  readonly hostArchitecture: string;
},): readonly string[] {
  if (tool === 'cargo-about') {
    return cargoAboutBuildArgumentsFromText(lockText,);
  }
  return zigBuildArgumentsFromText({
    lockText,
    hostArchitecture,
  },);
}

/**
 Reads the lock in one directory.

 @param directory - directory that may hold `toolchain.lock.json`

 @returns the lock's text, or the empty string when the directory has no lock
 */
async function readLockTextIn(directory: string,): Promise<string> {
  try {
    return await readFile(
      join(
        directory,
        LOCK_FILE_NAME,
      ),
      'utf8',
    );
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT')) {
      return '';
    }
    throw error;
  }
}

/**
 Reads the build arguments of one tool from the nearest lock at or above a directory.

 @param directory - directory being tried

 @param startDirectory - directory the search began in, named in the error when nothing is found

 @param tool - tool whose row is read

 @param hostArchitecture - Node `process.arch` of the host the build runs on

 @returns lines of the form `NAME=value`

 @throws LockedToolBuildArgumentsError when no ancestor holds a lock or the lock lacks what the tool needs
 */
async function readFromNearestLock({
  directory,
  startDirectory,
  tool,
  hostArchitecture,
}: {
  readonly directory: string;
  readonly startDirectory: string;
  readonly tool: LockedBuildArgumentTool;
  readonly hostArchitecture: string;
},): Promise<readonly string[]> {
  /**
   Text of the lock in this directory, empty when there is none.
   */
  const lockText = await readLockTextIn(directory,);
  if (lockText !== '') {
    return lockedToolBuildArgumentsFromText({
      lockText,
      tool,
      hostArchitecture,
    },);
  }

  /**
   Next directory up, equal to this one at the filesystem root.
   */
  const parent = dirname(directory,);
  if (parent === directory) {
    throw new LockedToolBuildArgumentsError(`no ${LOCK_FILE_NAME} in ${startDirectory} or any directory above it`,);
  }
  return readFromNearestLock({
    directory: parent,
    startDirectory,
    tool,
    hostArchitecture,
  },);
}

/**
 Reads the build arguments of one tool from the nearest lock above a directory.

 @param startDirectory - directory to start from, usually the caller's working directory

 @param tool - tool whose row is read

 @param hostArchitecture - Node `process.arch` of the host the build runs on; only the Zig row depends on it

 @returns lines of the form `NAME=value`

 @throws LockedToolBuildArgumentsError when no ancestor holds a lock or the lock lacks what the tool needs

 @example
 ```ts
 await readLockedToolBuildArguments({ startDirectory: process.cwd(), tool: 'cargo-about', hostArchitecture: process.arch, },);
 // ['CARGO_ABOUT_VERSION=0.9.3']
 ```
 */
export function readLockedToolBuildArguments({
  startDirectory,
  tool,
  hostArchitecture,
}: {
  readonly startDirectory: string;
  readonly tool: LockedBuildArgumentTool;
  readonly hostArchitecture: string;
},): Promise<readonly string[]> {
  return readFromNearestLock({
    directory: startDirectory,
    startDirectory,
    tool,
    hostArchitecture,
  },);
}
