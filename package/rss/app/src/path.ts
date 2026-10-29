import { join, } from 'node:path';

/**
 Absolute path to the directory storing ignore JSONL files.
 Used by server routes and ignore content reading to persist and filter "ignored" items.
 Resolved relative to the source directory (one level up to package root, then into `ignore/`).
 */
export const IGNORE_PATH: string = join(
  import.meta.dirname,
  '..',
  'ignore',
);
