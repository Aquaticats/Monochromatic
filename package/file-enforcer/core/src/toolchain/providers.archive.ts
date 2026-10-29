/**
 The `archive` route: download to the toolchain root, verify SHA-256, extract
 or copy into `<root>/<id>/<version>/`, and restore launcher permissions.
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  createReadStream,
  createWriteStream,
} from 'node:fs';
import {
  copyFile,
  mkdir,
  readdir,
  stat,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import { pipeline, } from 'node:stream/promises';
import { createGunzip, } from 'node:zlib';
import {
  downloadVerified,
  extractArchive,
  makeExecutable,
} from './exec.ts';
import { toolInstallDir, } from './paths.ts';
import {
  artifactForHost,
  type ProviderContext,
} from './providers.context.ts';
import { l, } from '../logger.ts';

//region Archive provider

/**
 File name suffixes of payloads `tar` or `unzip` extract; any other payload
 is a single-file release asset.
 */
const ARCHIVE_SUFFIXES: readonly string[] = [
  '.tar.gz',
  '.tgz',
  '.tar.xz',
  '.tar.bz2',
  '.tar',
  '.zip',
];

/**
 Whether one character may appear in a download file name unchanged.

 @param char - Single character (code point) to classify

 @returns True for ASCII letters, digits, dot, underscore and hyphen

 @example
 ```ts
 isFileNameCharacter('a',); // true
 ```
 */
function isFileNameCharacter(char: string,): boolean {
  return ((char >= 'a') && (char <= 'z'))
    || ((char >= 'A') && (char <= 'Z'))
    || ((char >= '0') && (char <= '9'))
    || '._-'.includes(char,);
}

/**
 Replaces every run of characters outside letters, digits, dot, underscore
 and hyphen with one hyphen, so a tool id is safe inside a file name.

 @param text - Text to make file-name safe

 @returns The text with each run of unsafe characters collapsed to `-`

 @example
 ```ts
 fileNameSafe('cargo:fixture',); // 'cargo-fixture'
 ```
 */
function fileNameSafe(text: string,): string {
  return Array.from(
    text,
    function keepSafe(char,): string {
      return isFileNameCharacter(char,)
        ? char
        : '';
    },
  )
    .filter(
    function dropRepeatedGap(
      piece: string,
      index: number,
      pieces: readonly string[],
    ): boolean {
      return (piece !== '') || (pieces[index - 1] !== '');
    },
    )
    .map(
    function gapToHyphen(piece,): string {
      return piece === ''
        ? '-'
        : piece;
    },
    )
    .join('',);
}

/**
 Installs one `archive`-route tool:
 download to the toolchain root, verify SHA-256, extract into
 `<root>/<id>/<version>/`, and copy single-file payloads in place.

 @param context - Provider context

 @returns The install directory

 @example
 ```ts
 const installDir = await installArchive({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installArchive(
  context: ProviderContext,
): Promise<string> {
  /**
   Plan entry and path set this install reads.
   */
  const {
    entry,
    paths,
  } = context;
  /**
   Verified artifact for the host platform.
   */
  const artifact = artifactForHost(entry,);
  /**
   Download path inside the toolchain root.
   */
  const downloadPath = join(
    paths.toolchainRoot,
    'downloads',
    `${fileNameSafe(entry.id,)}-${
      entry.version
    }-${
      basename(artifact.url,)
    }`,
  );
  await downloadVerified({
    url: artifact.url,
    target: downloadPath,
    sha256: artifact.sha256,
  },);
  /**
   Machine install directory of this tool.
   */
  const installDir = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: entry.version,
  });
  await mkdir(
    installDir,
    { recursive: true },
  );
  if (ARCHIVE_SUFFIXES.some(
    function endsTheUrl(suffix,): boolean {
      return artifact.url
        .endsWith(suffix,);
    },
  )) {
    await extractArchive({
      archive: downloadPath,
      into: installDir,
    },);
    // Release zips may store every entry as mode 0644 (the llamacpp-rocm
    // bundles do), which would hide the launchers from `findBinUnder`:
    // restore the executable bit on the declared names before linking.
    await restoreExecBits({
      root: installDir,
      names: [
        ...entry.bins,
        ...Object.keys(entry.options
          .binAliases
          ?? {}),
      ],
    },);
    return installDir;
  }
  // Single-file payload (a standalone binary release asset).
  /**
   Destination file name: the tool's first declared bin.
   */
  const name = entry.bins[0] ?? basename(artifact.url,);
  /**
   Destination path of the payload inside the install directory.
   */
  const target = join(
    installDir,
    name,
  );
  // Gzip single-file payloads (taplo ships a .gz binary) must be
  // decompressed before the file can run.
  await (artifact.url
    .endsWith('.gz',)
    ? pipeline(
      createReadStream(downloadPath,),
      createGunzip(),
      createWriteStream(target,),
    )
    : copyFile(
      downloadPath,
      target,
    ));
  await makeExecutable(target,);
  return installDir;
}

//endregion Archive provider

//region Executable bits

/**
 Deepest directory level below the extracted root that is searched for
 launchers (bounded like `findBinUnder`).
 */
const EXEC_SCAN_MAX_DEPTH = 3;

/**
 One directory queued for the launcher search.
 */
type PendingDirectory = {
  readonly dir: string;
  readonly depth: number;
};

/**
 What one directory contributed to the launcher search.
 */
type DirectoryScan = {
  /**
   Declared names made executable in this directory.
   */
  readonly fixed: readonly string[];
  /**
   Child directories still inside the depth bound.
   */
  readonly subdirectories: readonly PendingDirectory[];
};

/**
 Inspects one directory: makes declared launchers executable and lists the
 child directories worth searching next.

 @param pending - Directory and its depth below the extracted root

 @param wanted - Declared launcher names to make executable

 @returns Fixed names and child directories; an unreadable directory
 contributes neither

 @example
 ```ts
 const scan = await scanDirectory({ pending: { dir: '/tmp/tool', depth: 0, }, wanted: new Set(['tool',],), },);
 ```
 */
async function scanDirectory(
  {
    pending,
    wanted,
  }: {
    readonly pending: PendingDirectory;
    readonly wanted: ReadonlySet<string>;
  },
): Promise<DirectoryScan> {
  /**
   Logger tagged for this directory scan.
   */
  const log = tagged({
    tag: scanDirectory.name,
    l,
  },);
  try {
    /**
     Child names in a stable order.
     */
    const children = (await readdir(pending.dir,))
      .toSorted();
    /**
     Per-child outcomes, in child order.
     */
    const scanned = await Promise.all(children.map(
      async function scanChild(child,): Promise<{
        readonly fixed?: string;
        readonly subdirectory?: PendingDirectory;
      }> {
        /**
         Absolute child path.
         */
        const childPath = join(
          pending.dir,
          child,
        );
        try {
          /**
           File kind of the child.
           */
          const kind = await stat(childPath,);
          if (kind.isDirectory()) {
            return pending.depth < EXEC_SCAN_MAX_DEPTH
              ? {
                subdirectory: {
                  dir: childPath,
                  depth: pending.depth + 1,
                },
              }
              : {};
          }
          if (kind.isFile() && wanted.has(child,)) {
            await makeExecutable(childPath,);
            return { fixed: child, };
          }
        } catch (error) {
          log.debug(`${childPath} is not a launcher candidate: ${caughtValueText(error,)}`,);
        }
        return {};
      },
    ),);
    return {
      fixed: scanned.flatMap(
        function fixedName(outcome,): readonly string[] {
          return outcome.fixed === undefined
            ? []
            : [outcome.fixed];
        },
      ),
      subdirectories: scanned.flatMap(
        function subdirectoryOf(outcome,): readonly PendingDirectory[] {
          return outcome.subdirectory === undefined
            ? []
            : [outcome.subdirectory];
        },
      ),
    };
  } catch (error) {
    log.debug(`${pending.dir} is unreadable and is skipped: ${caughtValueText(error,)}`,);
    return {
      fixed: [],
      subdirectories: [],
    };
  }
}

/**
 Restores the executable bit on every file named like a declared bin inside
 an extracted tree (release zips may store every entry as mode 0644, which
 would hide the launchers from `findBinUnder`).

 @param root - Extracted tree to search

 @param names - Declared bin names to fix

 @returns Names whose file was made executable

 @example
 ```ts
 const fixed = await restoreExecBits({ root: '/tmp/tool', names: ['tool',], },);
 ```
 */
export async function restoreExecBits(
  {
    root,
    names,
  }: {
    readonly root: string;
    readonly names: readonly string[];
  },
): Promise<readonly string[]> {
  /**
   Declared names to fix.
   */
  const wanted = new Set(names,);
  /**
   Names made executable.
   */
  const fixed: string[] = [];
  /**
   Directories of the level being searched next.
   */
  const queue: PendingDirectory[] = [{
    dir: root,
    depth: 0,
  }];
  while (queue.length > 0) {
    /**
     The whole current level, removed from the queue.
     */
    const level = queue.splice(0,);
    /**
     Scans of every directory in the level, in level order.
     */
    // oxlint-disable-next-line no-await-in-loop -- each level lists the directories found by the previous level
    const scans = await Promise.all(level.map(
      function scanLevelDirectory(pending,): Promise<DirectoryScan> {
        return scanDirectory({
          pending,
          wanted,
        },);
      },
    ),);
    for (const scan of scans) {
      fixed.push(...scan.fixed,);
      queue.push(...scan.subdirectories,);
    }
  }
  return fixed;
}

//endregion Executable bits
