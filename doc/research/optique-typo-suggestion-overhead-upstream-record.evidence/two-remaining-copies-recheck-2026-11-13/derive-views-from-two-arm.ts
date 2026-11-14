// Derives one more diagnostic arm from the staged arm `both`: both thresholds set to two, the smallest value at which
// the prototype's own tests, which size themselves as threshold minus one and threshold plus a few, stay meaningful.
// States of three or more items and buffers with two or more tokens left are then views.
//
// Run: node derive-views-from-two-arm.ts

import { cpSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';

const ARMS = `${import.meta.dirname}/arms`;
const SOURCE = 'packages/core/src';

rmSync(`${ARMS}/views-from-two`, { recursive: true, force: true, },);
cpSync(`${ARMS}/both`, `${ARMS}/views-from-two`, { recursive: true, },);

for (
  const [file, find,] of [
    ['internal/persistent-vector.ts', 'export const VIEW_THRESHOLD = 1024;',],
    ['internal/input-buffer-view.ts', 'export const VIEW_THRESHOLD = 16384;',],
  ] as const
) {
  const path = `${ARMS}/views-from-two/${SOURCE}/${file}`;
  const text = readFileSync(path, 'utf8',);

  if (text.split(find,).length !== 2) throw new Error(`${file} does not hold the threshold line once`,);

  writeFileSync(path, text.replace(find, 'export const VIEW_THRESHOLD = 2;',),);
  console.log(JSON.stringify({ file, replaced: find, with: 'export const VIEW_THRESHOLD = 2;', },),);
}
