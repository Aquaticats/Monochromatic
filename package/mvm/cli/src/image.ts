/**
 Image download and caching for cloud images and ISOs.
 
 @module
 */

import {
  mkdir,
  unlink,
  writeFile as fsWriteFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { IMAGES_DIR, } from './config.ts';
import { pathExists, } from './path-exists.ts';
import { writeWithProgress, } from './download-progress.ts';
import {
  type ImageSpec,
  VIRTIO_WIN_FILENAME,
  VIRTIO_WIN_URL,
} from './registry.ts';
import { spawn, } from './spawn.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Downloads a file from a URL to a destination path if not already cached.
 Shows download progress on stderr when fetching.
 
 @param destPath - Destination file path
 
 @param tag - Logger tag for status messages
 
 @param url - URL to download from
 
 @returns Absolute path to the downloaded file
 
 @throws Error when the download fails
 
 @example
 ```ts
 const path = await downloadIfMissing({
   url: 'https://example.com/image.qcow2',
   destPath: '/home/user/.local/share/mvm/images/image.qcow2',
   tag: 'ensureImage',
 });
 ```
 */
async function downloadIfMissing({
  destPath,
  tag,
  url,
}: {
  readonly destPath: string;
  readonly tag: string;
  readonly url: string;
},): Promise<string> {
  /**
   Logger scoped to the caller's tag so download progress is attributed to the right caller.
   */
  const rl = tagged({
    tag,
    l,
  },);

  if (await pathExists(destPath,)) {
    rl.info(`using cached file ${destPath}`,);
    return destPath;
  }

  rl.info(`downloading to ${destPath}`,);
  await mkdir(
    IMAGES_DIR,
    { recursive: true, },
  );

  /**
   HTTP response for the source URL; consumed by {@link writeWithProgress} to stream the body to disk.
   */
  const response = await fetch(url,);
  if (!response.ok) {
    throw new Error(
      `failed to download from ${url}: ${response.status} ${response.statusText}`,
    );
  }

  await writeWithProgress({
    destPath,
    response,
    rl,
  },);
  return destPath;
}

/**
 Ensures a cloud image or evaluation ISO is cached locally, downloading it if missing.
 Shows download progress on stderr when fetching.
 
 @param spec - Image specification from the registry
 
 @returns Absolute path to the cached image file
 
 @throws Error when the download fails
 
 @example
 ```ts
 const imagePath = await ensureImage(IMAGES['ubuntu']);
 // => /home/user/.local/share/mvm/images/noble-server-cloudimg-amd64.img
 
 const isoPath = await ensureImage(IMAGES['windows']);
 // => /home/user/.local/share/mvm/images/windows-server-2025-eval.iso
 ```
 */
export function ensureImage(spec: ImageSpec,): Promise<string> {
  return downloadIfMissing({
    destPath: join(
      IMAGES_DIR,
      spec.fileName,
    ),
    tag: ensureImage.name,
    url: spec.url,
  },);
}

/**
 Ensures the virtio-win ISO is cached locally, downloading it if missing.
 The virtio-win ISO contains VirtIO storage/network drivers and the QEMU
 guest agent installer, required for Windows template creation.
 
 @returns Absolute path to the cached virtio-win ISO
 
 @throws Error when the download fails
 
 @example
 ```ts
 const virtioPath = await ensureVirtioWin();
 // => /home/user/.local/share/mvm/images/virtio-win.iso
 ```
 */
export function ensureVirtioWin(): Promise<string> {
  return downloadIfMissing({
    destPath: join(
      IMAGES_DIR,
      VIRTIO_WIN_FILENAME,
    ),
    tag: ensureVirtioWin.name,
    url: VIRTIO_WIN_URL,
  },);
}

/**
 Cached filename for the WinFsp MSI under `~/.local/share/mvm/images/`.
 */
const WINFSP_FILENAME = 'winfsp.msi';

/**
 Ensures the WinFsp MSI is cached locally, downloading it if missing.
 WinFsp (Windows File System Proxy) is required by VirtioFsSvc to mount
 virtiofs shares as drive letters on Windows guests.
 
 @returns Absolute path to the cached WinFsp MSI
 
 @throws Error when the download fails or the latest version cannot be resolved
 
 @example
 ```ts
 const winfspPath = await ensureWinFsp();
 // => /home/user/.local/share/mvm/images/winfsp.msi
 ```
 */
export async function ensureWinFsp(): Promise<string> {
  /**
   Logger scoped to this call so the multi-step WinFsp fetch is namespaced.
   */
  const rl = tagged({
    tag: ensureWinFsp.name,
    l,
  },);

  /**
   Cached MSI location; the early-exit short-circuit uses it before any network IO.
   */
  const destPath = join(
    IMAGES_DIR,
    WINFSP_FILENAME,
  );

  if (await pathExists(destPath,)) {
    rl.info(`using cached WinFsp MSI at ${destPath}`,);
    return destPath;
  }

  await mkdir(
    IMAGES_DIR,
    { recursive: true, },
  );

  rl.info('resolving latest WinFsp release...',);

  /**
   Resolve the latest version tag via GitHub redirect.
   */
  const redirectResponse = await fetch(
    'https://github.com/winfsp/winfsp/releases/latest',
    { redirect: 'manual', },
  );
  /**
   Redirect target carrying the resolved version in its `/tag/<version>` suffix.
   */
  const location = redirectResponse.headers
    .get('location',);
  if (location === null)
    throw new Error('failed to resolve latest WinFsp release',);
  /**
   Version tag from the redirect URL (e.g. "v2.1").
   */
  const [, version,] = location.split('/tag/',);
  if (version === undefined)
    throw new Error(`unexpected redirect URL: ${location}`,);

  /**
   Fetch release metadata to find the actual MSI asset name (includes build number).
   */
  const releaseResponse = await fetch(
    `https://api.github.com/repos/winfsp/winfsp/releases/tags/${version}`,
    { headers: { Accept: 'application/vnd.github+json', }, },
  );
  if (!releaseResponse.ok) {
    throw new Error(
      `failed to fetch WinFsp release metadata: ${releaseResponse.status}`,
    );
  }
  /**
   Parsed release payload narrowed to the asset list; only `assets` is read.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- GitHub API response
  const release = await releaseResponse.json() as {
    readonly assets: readonly {
      readonly name: string;
      readonly browser_download_url: string;
    }[];
  };
  /**
   Matching MSI asset; the released zip carries a per-build filename so it cannot be hard-coded.
   */
  const msiAsset = release.assets
    .find(function findMsi(a,) {
      return (a.name
        .endsWith('.msi',))
        && (!a.name
          .includes('tests',));
    },);
  if (msiAsset === undefined)
    throw new Error(`no MSI asset found in WinFsp release ${version}`,);

  rl.info(`downloading ${msiAsset.browser_download_url}`,);

  /**
   HTTP response for the MSI asset; body is buffered into memory then written to disk.
   */
  const msiResponse = await fetch(msiAsset.browser_download_url,);
  if (!msiResponse.ok) {
    throw new Error(
      `failed to download WinFsp: ${msiResponse.status} ${msiResponse.statusText}`,
    );
  }

  await fsWriteFile(
    destPath,
    new Uint8Array(await msiResponse.arrayBuffer(),),
  );
  rl.info(`WinFsp MSI cached at ${destPath}`,);
  return destPath;
}

/**
 Ensures the mise Windows exe is cached locally.
 Downloads the zip from the latest GitHub release, extracts mise.exe,
 and caches it in the images directory.
 
 Current template creation does not embed this file in Autounattend;
 Windows lifecycle tests install mise at runtime instead.
 
 @returns Absolute path to the cached mise.exe
 
 @throws Error when the download or extraction fails
 
 @example
 ```ts
 const misePath = await ensureMiseWindows();
 // => /home/user/.local/share/mvm/images/mise.exe
 ```
 */
export async function ensureMiseWindows(): Promise<string> {
  /**
   Logger scoped to this call so the multi-step mise fetch is namespaced.
   */
  const rl = tagged({
    tag: ensureMiseWindows.name,
    l,
  },);

  /**
   Cached mise.exe location; the early-exit short-circuit uses it before any network IO.
   */
  const destPath = join(
    IMAGES_DIR,
    'mise.exe',
  );

  if (await pathExists(destPath,)) {
    rl.info(`using cached mise.exe at ${destPath}`,);
    return destPath;
  }

  await mkdir(
    IMAGES_DIR,
    { recursive: true, },
  );

  rl.info('resolving latest mise release...',);

  /**
   Resolve the latest version tag via GitHub redirect.
   */
  const redirectResponse = await fetch(
    'https://github.com/jdx/mise/releases/latest',
    { redirect: 'manual', },
  );
  /**
   Redirect target carrying the resolved version in its `/tag/<version>` suffix.
   */
  const location = redirectResponse.headers
    .get('location',);
  if (location === null)
    throw new Error('failed to resolve latest mise release',);
  /**
   Version tag from the redirect URL (e.g. "v2026.3.17").
   */
  const [, version,] = location.split('/tag/',);
  if (version === undefined)
    throw new Error(`unexpected redirect URL: ${location}`,);

  /**
   Download URL for the Windows x64 zip.
   */
  const zipUrl =
    `https://github.com/jdx/mise/releases/download/${version}/mise-${version}-windows-x64.zip`;
  rl.info(`downloading ${zipUrl}`,);

  /**
   HTTP response for the mise zip; body is buffered into memory then written to disk for unzip.
   */
  const zipResponse = await fetch(zipUrl,);
  if (!zipResponse.ok) {
    throw new Error(
      `failed to download mise: ${zipResponse.status} ${zipResponse.statusText}`,
    );
  }

  /**
   Write zip to disk, extract mise.exe, clean up the zip.
   */
  const zipPath = join(
    IMAGES_DIR,
    'mise-windows-x64.zip',
  );
  await fsWriteFile(
    zipPath,
    new Uint8Array(await zipResponse.arrayBuffer(),),
  );

  await spawn({
    command: 'unzip',
    args: [
      '-o',
      '-j',
      zipPath,
      'mise/bin/mise.exe',
      '-d',
      IMAGES_DIR,
    ],
    notFoundRemedy:
      'mvm unpacks mise for the Windows template with `unzip`; install the host package that provides it.',
  },);

  await unlink(zipPath,);
  rl.info(`mise.exe cached at ${destPath}`,);
  return destPath;
}
