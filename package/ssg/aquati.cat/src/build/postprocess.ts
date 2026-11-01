/**
 Post-processing stage: pagefind indexing + content-hash fingerprinting.
 
 Orchestrates two independent post-site operations so they overlap
 wherever they can:
 
 - `pagefind --site dist/` reads every `dist/**\/*.html` and writes its
   index to `dist/pagefind/`.
 - Fingerprinting renames asset files with content hashes and rewrites
   references in HTML, CSS, and `manifest.webmanifest`.
 
 Fingerprinting has three dependency-ordered phases (CSS references fonts):
 1. Hash leaf assets (images, fonts, JS, PDFs, favicons): no outgoing references
 2. Rewrite CSS font `url()` with hashed names from phase 1, then hash the CSS
 3. Rewrite all HTML files and `manifest.webmanifest` with the complete replacement map
 
 Phases 1 and 2 touch binary leaf assets and `styles.css`: disjoint
 from the HTML pagefind reads. Those phases run **concurrently with
 pagefind** via `Promise.all`. Phase 3 modifies the same HTML files
 pagefind reads, so it runs **after** pagefind completes.
 
 Phase 3 uses basename-level `replaceAll` rather than HTML parsing.
 This works because the h-html template system produces predictable
 output, and basename replacement handles both absolute (`/inter.woff2`)
 and relative (`../glass-collection.avif`) paths uniformly.
 
 Excluded from fingerprinting: HTML (entry points), `pagefind/` (manages
 its own hashing), MDX source files, `robots.txt`, RSS feeds,
 `manifest.webmanifest` (rewritten but not renamed). Also excludes
 `node_modules/`, hidden directories, and build artifacts
 (`*.tsbuildinfo`, `*.jsonl`) that may exist inside `dist/`.
 
 Run via `mise run build:postprocess` or `node src/build/postprocess.ts`.
 */
import {
  readFile,
  rename,
  unlink,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import spawn from 'nano-spawn';
import readdir from 'tiny-readdir-glob';

import { sha256, } from '../lib/cache-hash.ts';

import {
  insertHash,
  sha256Buffer,
} from './fingerprint-naming.ts';
import { isLeafExcluded, } from './postprocess-excludes.ts';
import { DIST, } from './write-page.ts';

 // module boundary marker

/**
 Tagged logger for the postprocess pipeline.
 */
const l = tagged({
  tag: 'postprocess',
  l: logger,
},);

/* oxlint-disable no-restricted-syntax/no-regex -- fingerprinted-filename anchored suffix matchers; the input is a basename (bounded by filesystem name length) and both patterns anchor at `$`. `{10}` is a constant repetition count and `[^.]+` is linear with no nesting; no backtracking risk. */
/**
 Regex matching a previously fingerprinted filename.
 Matches `name.{10 hex chars}.ext` patterns.
 */
const STALE_HASH_PATTERN = /\.[0-9a-f]{10}\.[^.]+$/u;

/**
 Regex matching a previously fingerprinted filename with zstd compression.
 Matches `name.{10 hex chars}.ext.zst` patterns.
 */
const STALE_HASH_ZST_PATTERN = /\.[0-9a-f]{10}\.[^.]+\.zst$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Sentinel returned by {@link fingerprintCss} when no `styles.css` exists to
 fingerprint (already renamed). A genuine `Symbol` rather than
 `null`/`undefined`, which the `no-nullish-union` rule rejects.
 */
const CSS_ABSENT: unique symbol = Symbol('stylesheet asset missing before fingerprinting',);

//region Phase 1: fingerprint leaf assets

/**
 Fingerprints all leaf assets (files with no outgoing references to other hashable assets).
 
 @param files - pre-filtered file paths to fingerprint
 
 @returns replacement map of original basenames to hashed basenames
 
 @example
 ```ts
 const replacements = await fingerprintLeafAssets({ files: leafAssetFiles });
 // Map { 'inter.woff2' => 'inter.a1b2c3d4ef.woff2', ... }
 ```
 */
async function fingerprintLeafAssets(
  { files, }: { readonly files: readonly string[]; },
): Promise<Map<string, string>> {
  /**
   Original-to-hashed basename map built up across the per-file fan-out.
   */
  const replacements = new Map<string, string>();

  await Promise.all(files.map(async function fingerprintFile(filePath,) {
    /**
     File bytes read once for both the hash and the eventual rename.
     */
    const content = await readFile(filePath,);
    /**
     Content-addressed hash spliced into the renamed filename.
     */
    const hash = sha256Buffer(content,);
    /**
     Original basename used as the map key and rename source.
     */
    const original = basename(filePath,);
    /**
     Hashed basename produced by {@link insertHash}.
     */
    const hashed = insertHash({
      name: original,
      hash,
    },);
    /**
     Absolute rename target sharing the original directory.
     */
    const hashedPath = join(
      dirname(filePath,),
      hashed,
    );
    await rename(
      filePath,
      hashedPath,
    );
    replacements.set(
      original,
      hashed,
    );
    l.info(`${original} → ${hashed}`,);
  },),);

  return replacements;
}

//endregion Phase 1

//region Phase 2: fingerprint CSS

/**
 Rewrites font references in CSS and fingerprints the CSS file.
 
 Reads `styles.css`, applies existing replacements (font filenames from phase 1),
 computes a content hash of the updated CSS, writes the hashed file,
 and deletes the original.
 
 @param distDir - path to the dist output directory
 
 @param replacements - phase 1 replacement map (read-only; CSS entry returned, not added)
 
 @returns `styles.css` to hashed-basename mapping for the caller to record, or
 {@link CSS_ABSENT} when the CSS file is absent (already fingerprinted)
 
 @example
 ```ts
 const cssEntry = await fingerprintCss({ distDir: 'dist', replacements });
 ```
 */
async function fingerprintCss(
  {
    distDir,
    replacements,
  }: {
    readonly distDir: string;
    readonly replacements: ReadonlyMap<string, string>;
  },
): Promise<{
  readonly name: string;
  readonly hashedName: string;
} | typeof CSS_ABSENT> {
  /**
   Absolute path of the pre-fingerprinted CSS file shared by readCss and the rename.
   */
  const cssPath = join(
    distDir,
    'styles.css',
  );

  /**
   Reads the CSS file, returning {@link CSS_ABSENT} when missing (already fingerprinted).
   
   @returns CSS file contents, or {@link CSS_ABSENT} when the file does not exist
   */
  async function readCss(): Promise<string | typeof CSS_ABSENT> {
    try {
      return await readFile(
        cssPath,
        'utf8',
      );
    }
    catch (error) {
      if ((Error.isError(error,)) && ('code' in error)
        && (error.code
          === 'ENOENT')) {
        l.info(
          'styles.css not found, skipping CSS fingerprinting (already fingerprinted?)',
        );
        return CSS_ABSENT;
      }
      throw error;
    }
  }

  /**
   Pre-fingerprint CSS body; {@link CSS_ABSENT} when the file is already renamed.
   */
  const initialCss = await readCss();
  if (initialCss === CSS_ABSENT)
    return CSS_ABSENT;

  /**
   CSS body rewritten via replacements before hashing.
   */
  const cssContent = [...replacements,].reduce(
    function applyReplacement(
      acc: string,
      [original, hashed,]: readonly [
        string,
        string,
      ],
    ) {
      return acc.replaceAll(
        original,
        hashed,
      );
    },
    initialCss,
  );

  /**
   Content-addressed hash spliced into the renamed CSS filename.
   */
  const hash = sha256(cssContent,);
  /**
   Hashed basename produced by {@link insertHash}.
   */
  const hashedName = insertHash({
    name: 'styles.css',
    hash,
  },);
  /**
   Absolute path used as the write target before the original is unlinked.
   */
  const hashedPath = join(
    distDir,
    hashedName,
  );
  await writeFile(
    hashedPath,
    cssContent,
    'utf8',
  );
  await unlink(cssPath,);
  l.info(`styles.css → ${hashedName}`,);
  return {
    name: 'styles.css',
    hashedName,
  };
}

//endregion Phase 2

//region Phase 3: rewrite references

/**
 Rewrites asset references in HTML files and manifest.webmanifest.
 
 Performs basename-level string replacement so both absolute (`/inter.woff2`)
 and relative (`../glass-collection.avif`) references are updated.
 
 @param distDir - path to the dist output directory
 
 @param replacements - complete replacement map from phases 1 and 2
 
 @example
 ```ts
 await rewriteReferences({ distDir: 'dist', replacements });
 ```
 */
async function rewriteReferences(
  {
    distDir,
    replacements,
  }: {
    readonly distDir: string;
    readonly replacements: ReadonlyMap<string, string>;
  },
): Promise<void> {
  /**
   HTML files discovered by globbing dist for rewrite candidates.
   */
  const htmlResult = await readdir(`${distDir}/**/*.html`,);
  /**
   Combined HTML and manifest targets for the rewrite fan-out.
   */
  const rewriteTargets = [
    ...htmlResult.files,
    join(
      distDir,
      'manifest.webmanifest',
    ),
  ];

  await Promise.all(rewriteTargets.map(async function rewriteFile(filePath,) {
    /**
     Initial file body before replacements are applied.
     */
    const initialContent = await readFile(
      filePath,
      'utf8',
    );
    /**
     File body after every replacement has been applied.
     */
    const content = [...replacements,].reduce(
      function applyReplacement(
        acc: string,
        [original, hashed,]: readonly [
          string,
          string,
        ],
      ) {
        return acc.replaceAll(
          original,
          hashed,
        );
      },
      initialContent,
    );
    await writeFile(
      filePath,
      content,
      'utf8',
    );
  },),);

  l.info(`rewrote references in ${rewriteTargets.length} files`,);
}

//endregion Phase 3

//region Stale cleanup

/**
 Removes previously fingerprinted files from dist.
 
 Deletes any file matching the `name.{10 hex chars}.ext` pattern,
 including `.zst` compressed companions, to prevent accumulation
 across rebuilds.
 
 @param staleFiles - file paths matching the stale fingerprint pattern
 
 @example
 ```ts
 await cleanStaleFingerprints({ staleFiles: [...] });
 ```
 */
async function cleanStaleFingerprints(
  { staleFiles, }: { readonly staleFiles: readonly string[]; },
): Promise<void> {
  if (staleFiles.length
    > 0) {
    await Promise.all(staleFiles.map(function deleteStale(filePath,) {
      return unlink(filePath,);
    },),);
    l.info(`cleaned ${staleFiles.length} stale fingerprinted files`,);
  }
}

//endregion Stale cleanup

//region Pagefind

/**
 Runs `pagefind --site <distDir>` as a child process.
 
 Pagefind reads every `<distDir>/**\/*.html`, extracts indexable text,
 and writes its index into `<distDir>/pagefind/`. It does not touch
 any asset hrefs in HTML (those live in `<link>`/`<script>`/`<img>`,
 which are not part of pagefind's indexed text), so it can safely
 overlap with fingerprint phases 1 and 2 (leaf-asset renaming + CSS
 rewriting) which never touch HTML.
 
 Each stdout line is forwarded through the tagged logger at info level
 so pagefind's section headers, index counts, and timing reach the build
 log. stderr lines are forwarded at warn level (pagefind emits `Note:`
 diagnostics and hard errors there). Non-zero exit throws.
 
 @param distDir - path to the dist output directory to index
 
 @example
 ```ts
 await runPagefind({ distDir: DIST });
 ```
 */
async function runPagefind(
  { distDir, }: { readonly distDir: string; },
): Promise<void> {
  /**
   Sub-tagged logger so pagefind output is attributable in the build log.
   */
  const pl = tagged({
    tag: 'pagefind',
    l,
  },);
  /**
   Child process handle used to stream stdout and stderr concurrently.
   */
  const subprocess = spawn(
    'pagefind',
    [
      '--site',
      distDir,
    ],
  );
  await Promise.all([
    (async function forwardStdout(): Promise<void> {
      for await (const line of subprocess.stdout)
        pl.info(line,);
    })(),
    (async function forwardStderr(): Promise<void> {
      for await (const line of subprocess.stderr)
        pl.warn(line,);
    })(),
    subprocess,
  ],);
  pl.info('indexed',);
}

//endregion Pagefind

//region Main pipeline

l.info('starting',);

/**
 Single scan of dist, partitioned into stale fingerprinted files and leaf assets.
 */
const fullScan = await readdir(`${DIST}/**/*`,);

/**
 Previously fingerprinted files to clean up before re-fingerprinting.
 */
const staleFiles = fullScan
  .files
  .filter(function isStale(filePath,) {
  /**
   Basename used for the stale-fingerprint pattern check, regardless of subdirectory depth.
   */
    const name = basename(filePath,);
    return STALE_HASH_PATTERN.test(name,)
      || STALE_HASH_ZST_PATTERN
        .test(name,);
  },);

await cleanStaleFingerprints({ staleFiles, },);

/**
 Leaf assets eligible for fingerprinting (excludes HTML, CSS, pagefind, etc.).
 */
const leafAssetFiles = fullScan
  .files
  .filter(function isLeafAsset(filePath,) {
  /**
   Basename used for the stale-fingerprint pattern check independent of the full path.
   */
    const name = basename(filePath,);
    if (STALE_HASH_PATTERN.test(name,)
      || STALE_HASH_ZST_PATTERN
        .test(name,))
      return false;
    return !isLeafExcluded(filePath,);
  },);

/**
 Runs fingerprint phases 1 and 2 together: hash leaf assets, then
 rewrite and hash `styles.css`. Returns the replacements map so phase
 3 can consume it.
 
 Split out so it can be awaited in a `Promise.all` with {@link runPagefind};
 neither branch touches HTML, so they race on disjoint files.
 
 @returns basename → hashed-basename map covering every hashed asset
 
 @example
 ```ts
 const replacements = await fingerprintAssets();
 ```
 */
async function fingerprintAssets(): Promise<Map<string, string>> {
  /**
   Hashed leaf-asset replacements; passed to CSS rewriting and later returned to phase 3.
   */
  const replacements = await fingerprintLeafAssets({
    files: leafAssetFiles,
  },);
  l.info(`phase 1: fingerprinted ${replacements.size} leaf assets`,);

  /**
   `styles.css` mapping returned by phase 2, or {@link CSS_ABSENT} when the CSS file is absent.
   */
  const cssEntry = await fingerprintCss({
    distDir: DIST,
    replacements,
  },);
  if (cssEntry !== CSS_ABSENT) {
    replacements.set(
      cssEntry.name,
      cssEntry.hashedName,
    );
  }
  l.info('phase 2: fingerprinted CSS',);

  return replacements;
}

/**
 Phases 1+2 (fingerprint leaf assets + CSS) run in parallel with
 pagefind. Phase 3 (HTML rewrite) is sequenced strictly after both
 because it modifies the same HTML files pagefind is reading.
 */
const [replacements,] = await Promise.all([
  fingerprintAssets(),
  runPagefind({ distDir: DIST, },),
],);

await rewriteReferences({
  distDir: DIST,
  replacements,
},);
l.info('phase 3: rewrote references',);

l.info(`done: ${replacements.size} assets fingerprinted`,);

//endregion Main pipeline
