// Mirror the repo into a scratch tree with lint-suppression directives neutralized
// (same byte length, so line/column positions are preserved), so the no-regex rule
// reports every regex site including ones already carrying a disable justification.
import {
  readdirSync,
  statSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import {
  join,
  relative,
  dirname,
} from 'node:path';

import { homedir, } from 'node:os';
import { fileURLToPath, } from 'node:url';

/**
 Repository root, derived from this file's committed location rather than a
 hardcoded path, so the harness runs from any clone.
 */
const ROOT = fileURLToPath(new URL(
  '../../../..',
  import.meta.url
))
  .replace(
  /\/$/,
  ''
  );

/**
 Scratch working directory. The harness writes mirrors and ledgers here, never
 into the repository. Override with REGEX_AUDIT_WORK.
 */
const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;

const MIRROR = '${AUDIT}/mirror';

const SKIP_DIRS = new Set([
  'node_modules',
  '.git',
  'target',
  'dist',
  'bundle',
  'coverage',
  '.cache',
  '.slopo.local.dir',
  'playwright-report',
  'test-results',
  '.pnpm-store',
  'vendor',
  '.idea',
  '.serena',
  '.claude',
  '.crush',
  '.factory',
  '.fallow',
  '.mise',
  '.changeset',
  '.opencodereview',
  '.pi',
]);

const EXTENSIONS = new Set([
  '.ts',
  '.tsx',
  '.mts',
  '.cts',
  '.js',
  '.jsx',
  '.mjs',
  '.cjs'
]);

const collected = [];
let skippedBinary = 0;

function walk(dir) {
  let entries;
  try {
    entries = readdirSync(
      dir,
      { withFileTypes: true }
    );
  } catch (error) {
    console.error(`unreadable dir ${dir}: ${String(error)}`);
    return;
  }
  for (const entry of entries) {
    const full = join(
      dir,
      entry.name
    );
    if (entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) {
      if (SKIP_DIRS.has(entry.name)) continue;
      walk(full);
      continue;
    }
    if (!entry.isFile()) continue;
    const dot = entry.name
      .lastIndexOf('.');
    const extension = dot === (-1) ? '' : entry.name
      .slice(dot);
    if (!EXTENSIONS.has(extension)) continue;
    collected.push(full);
  }
}

walk(ROOT);

let mirrored = 0;
const neutralizeLog = [];
for (const file of collected) {
  let text;
  try {
    text = readFileSync(
      file,
      'utf8'
    );
  } catch (error) {
    skippedBinary += 1;
    continue;
  }
  let changed = 0;
  const neutralized = text
    .replaceAll(
    'oxlint-disable',
    'oxlint-xisable'
    )
    .replaceAll(
    'oxlint-enable',
    'oxlint-xnable'
    )
    .replaceAll(
    'eslint-disable',
    'eslint-xisable'
    )
    .replaceAll(
    'eslint-enable',
    'eslint-xnable'
    );
  changed = neutralized === text ? 0 : 1;
  if (changed === 1) neutralizeLog.push(relative(
    ROOT,
    file
  ));
  const target = join(
    MIRROR,
    relative(
      ROOT,
      file
    )
  );
  mkdirSync(
    dirname(target),
    { recursive: true }
  );
  writeFileSync(
    target,
    neutralized,
    'utf8'
  );
  mirrored += 1;
}

writeFileSync(
  '${AUDIT}/manifest.json',
  JSON.stringify(
    {
      fileCount: collected.length,
      mirrored,
      skippedBinary,
      neutralizedFiles: neutralizeLog,
    },
    null,
    2
  ),
  'utf8',
);
console.log(`candidate files: ${collected.length}`);
console.log(`mirrored: ${mirrored}`);
console.log(`files with neutralized directives: ${neutralizeLog.length}`);
