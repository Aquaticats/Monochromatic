/**
 Build script: generates a single self-contained HTML doodle widget.
 
 Reads default SVG backgrounds for each page, reads the pre-bundled
 client JS from tsdown output, then assembles HTML/CSS/JS into a
 single file. SVG backgrounds are embedded unmodified; white fill
 removal for user-uploaded SVGs happens at runtime via size-based
 detection in the client background module.
 
 Requires `mise run build:js:client` to have run first so
 `dist/client/main.js` exists.
 */
import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { renderPage, } from './page.ts';
import { resolveSourceUrl, } from './source-url.ts';
import { renderStyles, } from './styles.ts';



/**
 Absolute path to this package's root directory
 */
const PACKAGE_DIR: string = new URL(
  '..',
  import.meta.url,
)
  .pathname;

/**
 Output directory for the generated site
 */
const DIST_DIR = join(
  PACKAGE_DIR,
  'dist',
  'final',
);

console.error('[doodle-widget] building...',);

/**
 SVG backgrounds passed through unmodified; the bundled SVGs already
 have transparent backgrounds (`fill:none`). White fill removal only
 applies to user-uploaded SVGs at runtime via {@link setSvgBackground},
 which uses size-based detection to target actual backgrounds.
 */
const svgBackgrounds = [
  await readFile(
    join(
      PACKAGE_DIR,
      'src',
      'asset',
      'output_1.svg',
    ),
    'utf8',
  ),
  await readFile(
    join(
      PACKAGE_DIR,
      'src',
      'asset',
      'output_2.svg',
    ),
    'utf8',
  ),
];

/**
 Minified CSS stylesheet, rendered by {@link renderStyles}
 */
const css = renderStyles();

/**
 Client-side canvas drawing and background management script, pre-bundled by tsdown
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
 Source code URL resolved from git remote and package.json by {@link resolveSourceUrl}
 */
const sourceUrl = await resolveSourceUrl(PACKAGE_DIR,);

/**
 Complete self-contained HTML document, assembled by {@link renderPage}
 */
const html = renderPage({
  css,
  js,
  svgBackgrounds,
  sourceUrl,
},);

await mkdir(
  DIST_DIR,
  { recursive: true, },
);
await writeFile(
  join(
    DIST_DIR,
    'index.html',
  ),
  html,
  'utf8',
);

console.error(`[doodle-widget] wrote ${
  join(
    DIST_DIR,
    'index.html',
  )
}`,);
