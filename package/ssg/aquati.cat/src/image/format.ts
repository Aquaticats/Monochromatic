/**
 Image format conversion script.
 
 Converts non-AVIF raster images (PNG, JPEG, TIFF, WebP) to AVIF
 counterparts using sharp. Only converts files that do not already
 have a sibling `.avif` file on disk.
 
 Run via `mise run format:images`.
 */
import type { ReadonlyDeep, } from 'type-fest';
import {
  basename,
  dirname,
  extname,
  join,
} from 'node:path';

import readdir from 'tiny-readdir-glob';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { maybeConvert, } from './convert.ts';

 // module boundary marker

/**
 Tagged logger for the image format conversion subsystem.
 */
const l = tagged({
  tag: 'format:images',
  l: logger,
},);

/**
 Glob pattern matching raster image extensions eligible for AVIF conversion.
 
 Covers all raster formats sharp (libvips) can decode, derived from the
 `ImageType` enum in sharp's `src/common.h`. Excludes vector (SVG),
 document (PDF), and specialty formats (FITS, DCRAW, VIPS, MAGICK, OpenSlide).
 */
const RASTER_GLOB =
  '**/*.{png,jpg,jpeg,tif,tiff,webp,gif,heic,heif,jxl,jp2,j2k,jpx,ppm,pgm,pbm,pfm,exr,hdr}';

/**
 Directories to scan for raster images.
 */
const SCAN_DIRS = [
  'src/content',
  'public',
];

//region Top-level conversion pipeline: scans directories and converts raster images to AVIF

l.info('scanning for raster images',);

/**
 Scan results from all directories, fetched concurrently.
 */
const scanResults = await Promise.all(
  SCAN_DIRS.map(function scanDir(dir,) {
    return readdir(`${dir}/${RASTER_GLOB}`,);
  },),
);

/**
 Conversion tasks for all discovered raster images across all scanned directories.
 */
const tasks = scanResults.flatMap(function buildTasks(
  result: ReadonlyDeep<(typeof scanResults)[number]>,
) {
  return result.files
    .map(function createTask(filePath,) {
    /**
     Base filename without extension, used to derive the AVIF output path.
     */
      const nameWithoutExt = basename(
        filePath,
        extname(filePath,),
      );
    /**
     Target AVIF path sitting alongside the source raster image.
     */
      const avifPath = join(
        dirname(filePath,),
        `${nameWithoutExt}.avif`,
      );
      return maybeConvert({
        filePath,
        avifPath,
        l,
      },);
    },);
},);

/**
 Settled conversion outcomes used to tally converted vs skipped counts.
 */
const results = await Promise.all(tasks,);
/**
 Number of images that were newly converted to AVIF.
 */
const converted = results.filter(Boolean,)
  .length;
/**
 Number of images skipped because an AVIF counterpart already existed.
 */
const skipped = results.length
  - converted;

l.info(`done: ${converted} converted, ${skipped} skipped`,);

//endregion Top-level conversion pipeline
