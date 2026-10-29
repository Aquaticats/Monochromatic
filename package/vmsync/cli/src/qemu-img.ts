/**
 Typed wrapper around the `qemu-img` command-line tool.
 Provides functions for image inspection, conversion, overlay management,
 and block map retrieval needed for incremental sync.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { spawn, } from './spawn.ts';
import type {
  QemuImgInfo,
  QemuMapRegion,
} from './types.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

/**
 Retrieves format and geometry information for a disk image.
 
 @param imagePath - Absolute path to the disk image
 
 @returns Parsed {@link QemuImgInfo} from `qemu-img info`
 
 @throws Error when the image is unreadable or format is unsupported
 
 @example
 ```ts
 const info = await imageInfo('/path/to/disk.qcow2');
 console.log(info.format, info['virtual-size']);
 ```
 */
export async function imageInfo(imagePath: string,): Promise<QemuImgInfo> {
  /**
   Function-tagged logger so traces show which qemu-img call produced each line.
   */
  const rl = tagged({
    tag: imageInfo.name,
    l,
  },);
  rl.info(`inspecting ${imagePath}`,);

  /**
   Raw JSON string from qemu-img.
   */
  const raw = await spawn({
    command: 'qemu-img',
    args: [
      'info',
      '--output=json',
      imagePath,
    ],
  },);

  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- trusted qemu-img JSON output
  return JSON.parse(raw,) as QemuImgInfo;
}

/**
 Converts a disk image between formats.
 
 @param sourcePath - Path to the source image
 
 @param sourceFormat - Format of the source image (e.g. 'qcow2', 'raw', 'vhdx')
 
 @param targetPath - Path for the output image
 
 @param targetFormat - Desired output format
 
 @throws Error when conversion fails (e.g. unsupported format, disk full)
 
 @example
 ```ts
 await convert({
   sourcePath: '/tmp/disk.raw',
   sourceFormat: 'raw',
   targetPath: '/data/disk.qcow2',
   targetFormat: 'qcow2',
 });
 ```
 */
export async function convert(
  {
    sourcePath,
    sourceFormat,
    targetPath,
    targetFormat,
  }: {
    readonly sourcePath: string;
    readonly sourceFormat: string;
    readonly targetPath: string;
    readonly targetFormat: string;
  },
): Promise<void> {
  /**
   Function-tagged logger so traces show which qemu-img call produced each line.
   */
  const rl = tagged({
    tag: convert.name,
    l,
  },);
  rl.info(`${sourceFormat} -> ${targetFormat}: ${sourcePath} -> ${targetPath}`,);

  await spawn({
    command: 'qemu-img',
    args: [
      'convert',
      '-f',
      sourceFormat,
      '-O',
      targetFormat,
      '-p',
      sourcePath,
      targetPath,
    ],
  },);

  rl.info('conversion complete',);
}

/**
 Creates a qcow2 overlay backed by an existing base image.
 The overlay stores only blocks that differ from the base,
 enabling cheap change detection after a boot session.
 
 @param overlayPath - Path for the new overlay file
 
 @param backingPath - Absolute path to the base qcow2 image
 
 @throws Error when overlay creation fails
 
 @example
 ```ts
 await createOverlay({
   overlayPath: '/data/alpine/overlay.qcow2',
   backingPath: '/data/alpine/base.qcow2',
 });
 ```
 */
export async function createOverlay(
  {
    overlayPath,
    backingPath,
  }: {
    readonly overlayPath: string;
    readonly backingPath: string;
  },
): Promise<void> {
  /**
   Function-tagged logger so traces show which qemu-img call produced each line.
   */
  const rl = tagged({
    tag: createOverlay.name,
    l,
  },);
  rl.info(`creating overlay backed by ${backingPath}`,);

  await spawn({
    command: 'qemu-img',
    args: [
      'create',
      '-f',
      'qcow2',
      '-b',
      backingPath,
      '-F',
      'qcow2',
      overlayPath,
    ],
  },);

  rl.info(`overlay created at ${overlayPath}`,);
}

/**
 Returns the block allocation map for an image, showing which regions
 are allocated at each depth in the backing chain.
 Regions with `depth === 0` are data written to the topmost overlay.
 
 @param imagePath - Path to a qcow2 overlay (must have a backing file for meaningful depth info)
 
 @returns array of {@link QemuMapRegion} descriptors
 
 @example
 ```ts
 const regions = await blockMap('/data/alpine/overlay.qcow2');
 const changed = regions.filter(r => r.depth === 0 && r.data);
 ```
 */
export async function blockMap(imagePath: string,): Promise<readonly QemuMapRegion[]> {
  /**
   Function-tagged logger so traces show which qemu-img call produced each line.
   */
  const rl = tagged({
    tag: blockMap.name,
    l,
  },);
  rl.info(`reading block map for ${imagePath}`,);

  /**
   Raw JSON array string from qemu-img map.
   */
  const raw = await spawn({
    command: 'qemu-img',
    args: [
      'map',
      '--output=json',
      imagePath,
    ],
  },);

  /**
   Parsed region array.
   */
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- trusted qemu-img JSON output
  const regions = JSON.parse(raw,) as readonly QemuMapRegion[];
  /**
   Count of overlay-level (depth 0) regions containing data.
   */
  const changedCount = regions
    .filter(
    function isOverlayData(r,) {
      return (r.depth
        === 0) && r
          .data;
    },
    )
    .length;

  rl.info(
    `${String(regions.length,)} regions total, ${
      String(changedCount,)
    } changed in overlay`,
  );
  return regions;
}

/**
 Commits (merges) an overlay's changes into its backing file.
 After commit, the overlay's changes are part of the base image
 and the overlay file is no longer needed.
 
 @param overlayPath - Path to the qcow2 overlay to commit
 
 @throws Error when commit fails (e.g. backing file is read-only)
 
 @example
 ```ts
 await commitOverlay('/data/alpine/overlay.qcow2');
 ```
 */
export async function commitOverlay(overlayPath: string,): Promise<void> {
  /**
   Function-tagged logger so traces show which qemu-img call produced each line.
   */
  const rl = tagged({
    tag: commitOverlay.name,
    l,
  },);
  rl.info(`committing overlay ${overlayPath} into backing file`,);

  await spawn({
    command: 'qemu-img',
    args: [
      'commit',
      overlayPath,
    ],
  },);

  rl.info('overlay committed',);
}

/**
 Locates the exclusive end of the leading non-whitespace run in `text`.
 
 Scans once from offset 0, stopping at the first space, tab, newline, or
 carriage return; returns `text.length` when no whitespace is present.
 Single linear pass, stack-safe on arbitrarily long input.
 
 @param text - string to scan from offset 0
 
 @returns index of first whitespace character, or `text.length`
 
 @example
 ```ts
 firstWhitespaceIndex('abcdef  file'); // 6
 firstWhitespaceIndex('nowhitespace'); // 12
 ```
 */
export function firstWhitespaceIndex(text: string,): number {
  /**
   Scan cursor; advances past each leading non-whitespace character in one linear pass.
   */
  let cursor = 0;
  while (cursor < text
    .length) {
    /**
     Char under the cursor; stops the scan at the first whitespace (space, tab, newline, or carriage return).
     */
    const c = text.charAt(cursor,);
    if (' \t\n\r'.includes(c,))
      break;
    cursor += 1;
  }
  return cursor;
}

/**
 Computes the SHA-256 checksum of a disk image file.
 Uses `sha256sum` on Linux and `certutil` on Windows.
 
 @param imagePath - Absolute path to the image file
 
 @returns Hex-encoded SHA-256 hash prefixed with `sha256:`
 
 @example
 ```ts
 const hash = await checksum('/data/alpine/base.qcow2');
 // "sha256:abcdef1234..."
 ```
 */
export async function checksum(imagePath: string,): Promise<string> {
  /**
   Function-tagged logger so traces show which qemu-img call produced each line.
   */
  const rl = tagged({
    tag: checksum.name,
    l,
  },);
  rl.info(`computing sha256 for ${imagePath}`,);

  if (process.platform
    === 'win32') {
    /**
     certutil output includes the hash on its own line.
     */
    const raw = await spawn({
      command: 'certutil',
      args: [
        '-hashfile',
        imagePath,
        'SHA256',
      ],
    },);
    /**
     Second line of certutil output is the hex hash.
     */
    const hash = nonNullishOrThrow(
      raw.split('\n',)[1],
    )
      .trim();
    return `sha256:${hash}`;
  }

  /**
   sha256sum output: "<hash>  <filename>".
   */
  const raw = await spawn({
    command: 'sha256sum',
    args: [imagePath,],
  },);
  /**
   Hash portion before the filename separator.
   */
  const hash = raw.slice(
    0,
    firstWhitespaceIndex(raw,),
  );
  return `sha256:${hash}`;
}
