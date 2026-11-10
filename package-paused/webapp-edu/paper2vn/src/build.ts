/**
 Build script: assembles the self-contained paper2vn HTML.
 
 Reads the bundled client JavaScript, the placeholder sprite manifest,
 and the per-locale i18n bundle, then renders the HTML shell with
 everything inlined so the output works from `file://` or any static host.
 
 Requires `mise run build:js:client` to have run first so
 `dist/client/main.js` exists.
 */
import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l as packageLogger, } from './log.ts';
import { renderPage, } from './page.ts';
import { renderStyles, } from './styles.ts';

/**
 Logger for this build script, tagged below the package root logger.
 */
const l = tagged({
  tag: 'build',
  l: packageLogger,
},);

/**
 Absolute path to this package's root directory
 */
const PACKAGE_DIR: string = new URL(
  '..',
  import.meta.url,
)
  .pathname;

/**
 Output directory for the generated single-file app
 */
const DIST_DIR = join(
  PACKAGE_DIR,
  'dist',
  'final',
);

l.info('building...',);

/**
 Pre-bundled client SPA, produced by tsdown
 */
const js = await readFile(
  join(
    PACKAGE_DIR,
    'dist',
    'client',
    'main.js',
  ),
  'utf8',
);

/**
 Sprite pack manifest read as raw text so it can be embedded as a
 JSON island. Reading instead of importing keeps TS from inferring
 the parsed-object shape.
 */
const spritePackManifest = await readFile(
  join(
    PACKAGE_DIR,
    'src',
    'assets',
    'sprites',
    'manifest.json',
  ),
  'utf8',
);

/**
 Minified CSS stylesheet
 */
const css = renderStyles();

/**
 Complete self-contained HTML document
 */
const html = renderPage({
  css,
  js,
  spritePackManifest,
},);

await mkdir(
  DIST_DIR,
  { recursive: true, },
);
/**
 Final HTML output path inside the dist directory.
 */
const outPath = join(
  DIST_DIR,
  'index.html',
);
await writeFile(
  outPath,
  html,
  'utf8',
);

l.info(`wrote ${outPath}`,);
