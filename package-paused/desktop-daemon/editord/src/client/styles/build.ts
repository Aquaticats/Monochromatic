/**
 * Build script that evaluates h-css style modules and writes `dist/client/global.css`.
 *
 * Concatenates all global (non-shadow-DOM) style modules into a single CSS file
 * loaded by `index.html` via `<link>`. Run via `mise run build:css`.
 *
 * @example
 * ```sh
 * bun src/client/styles/build.ts
 * ```
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  findRootCached,
  packageNamed,
} from '@monochromatic-dev/fs-path-core/ts';

import { STYLES as CTX_MENU_STYLES, } from '../context-menu/context-menu.styles.ts';
import { STYLES as TOAST_STYLES, } from '../toast/toast.styles.ts';
import { STYLES as FAB_STYLES, } from './fullscreen-fab.styles.ts';
import { STYLES as RESET_STYLES, } from './reset.ts';
import { STYLES as THEME_STYLES, } from './theme.ts';

export {};

/**
 * Package root: the nearest ancestor of this file whose manifest names this package.
 */
const PACKAGE_ROOT = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/desktop-daemon-editord',),
},);

/**
 * Output directory under the package root.
 */
const OUT_DIR = join(
  PACKAGE_ROOT,
  'dist/client',
);

/**
 * Output file path.
 */
const OUT_FILE = join(
  OUT_DIR,
  'global.css',
);

/**
 * Combined CSS from all global style modules.
 */
const css = THEME_STYLES + RESET_STYLES
  + TOAST_STYLES
  + FAB_STYLES
  + CTX_MENU_STYLES;

await mkdir(
  OUT_DIR,
  { recursive: true, },
);
await writeFile(
  OUT_FILE,
  css,
  'utf8',
);
