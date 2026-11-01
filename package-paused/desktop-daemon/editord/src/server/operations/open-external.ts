/**
 * External application launch operations.
 *
 * Opens a terminal at a directory path or opens a file in the default application.
 * On Linux, delegates to `@monochromatic-dev/terminal-exec-cli` which resolves
 * the preferred terminal and passes `--dir=` so the emulator opens in the correct directory.
 * Uses `open` on macOS, `start` on Windows.
 */

import { dirname, } from 'node:path';
import { platform, } from 'node:process';

import { launchTerminal, } from '@monochromatic-dev/terminal-exec-cli';

import { assertWithinRoot, } from './assert-within-root.ts';
import { spawnDetached, } from './spawn-detached.ts';

/**
 * Opens a terminal emulator at the given directory path.
 * Uses `terminal-exec` conventions per platform.
 *
 * @param rootDir - absolute root directory for path containment
 *
 * @param path - directory path to open in a terminal
 *
 * @throws when the path escapes root or no terminal emulator is found
 *
 * @example
 * ```ts
 * await openInTerminal({ rootDir: '/home/user/project', path: '/home/user/project/src/main.ts', });
 * ```
 */
export async function openInTerminal(
  {
    rootDir,
    path,
  }: {
    readonly rootDir: string;
    readonly path: string;
  },
): Promise<void> {
  /**
   * Path validated to live inside `rootDir`; resolved absolute form passed to the OS launcher.
   */
  const absolutePath = assertWithinRoot({
    rootDir,
    path,
  },);
  /**
   * Local alias of `process.platform`; lets each branch read like a plain comparison.
   */
  const currentPlatform = platform;

  if (currentPlatform === 'linux')
    await launchTerminal({ dir: absolutePath, },);
  else if (currentPlatform === 'darwin') {
    await spawnDetached({
      command: 'open',
      args: [
        '-a',
        'Terminal',
        absolutePath,
      ],
      cwd: absolutePath,
    },);
  }
  else if (currentPlatform === 'win32') {
    await spawnDetached({
      command: 'cmd',
      args: [
        '/c',
        'start',
        'cmd',
      ],
      cwd: absolutePath,
    },);
  }
  else {
    throw new Error(`unsupported platform: ${currentPlatform}`,);
  }
}

/**
 * Opens a file in its default application.
 * Uses `xdg-open` on Linux, `open` on macOS, `start` on Windows.
 *
 * @param rootDir - absolute root directory for path containment
 *
 * @param path - file path to open
 *
 * @throws when the path escapes root or the open command fails
 *
 * @example
 * ```ts
 * await openInDefaultApp({ rootDir: '/home/user/project', path: '/home/user/project/src/main.ts', });
 * ```
 */
export async function openInDefaultApp(
  {
    rootDir,
    path,
  }: {
    readonly rootDir: string;
    readonly path: string;
  },
): Promise<void> {
  /**
   * Path validated to live inside `rootDir`; resolved absolute form passed to the OS launcher.
   */
  const absolutePath = assertWithinRoot({
    rootDir,
    path,
  },);
  /**
   * Local alias of `process.platform`; lets each branch read like a plain comparison.
   */
  const currentPlatform = platform;

  /**
   * Parent directory of the target; used as the spawn `cwd` so relative resources resolve correctly.
   */
  const dir = dirname(absolutePath,);

  if (currentPlatform === 'linux') {
    await spawnDetached({
      command: 'xdg-open',
      args: [absolutePath,],
      cwd: dir,
    },);
  }
  else if (currentPlatform === 'darwin') {
    await spawnDetached({
      command: 'open',
      args: [absolutePath,],
      cwd: dir,
    },);
  }
  else if (currentPlatform === 'win32') {
    await spawnDetached({
      command: 'cmd',
      args: [
        '/c',
        'start',
        '',
        absolutePath,
      ],
      cwd: dir,
    },);
  }
  else {
    throw new Error(`unsupported platform: ${currentPlatform}`,);
  }
}
