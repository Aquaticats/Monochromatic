/**
 Throwaway Phase-C helper: switch a package's `*.unit.test.ts` imports of its
 OWN sibling source modules from relative (`./foo.ts`) to by-name
 (`@monochromatic-dev/<pkg>`), so the tests exercise the BUILT dist after the
 package's `.` export is flipped from src to dist.
 
 Scope guard (deliberate): only rewrites a file that imports EXACTLY ONE own
 non-test sibling source module. Files with two or more such imports would
 collapse to duplicate by-name statements (an `eslint/no-duplicate-imports`
 violation needing a manual merge), so they are reported and SKIPPED for hand
 handling. Files with zero own-source imports (aggregators, already-by-name,
 node-builtin-only) are left untouched.
 
 Internal-symbol detection is delegated to the build oracle: if a switched
 symbol is not re-exported from the package index, `buildAndTest` fails loudly
 naming it, and that one import is reverted to source by hand.
 
 No regex: pure string scanning of each line's `from '...'` / `import '...'`
 specifier. Idempotent (a by-name specifier is not relative, so it is skipped).
 
 Usage: `bun mise.switch-tests-byname.ts <pkgDir> <pkgName> [--dry]`
   e.g. `bun mise.switch-tests-byname.ts package/or-throw/core @monochromatic-dev/or-throw-core`
 
 @module
 */

import { Glob, } from 'bun';
import {
  dirname,
  resolve,
} from 'node:path';
import { existsSync, } from 'node:fs';

const [pkgDir, pkgName, dryFlag,] = process.argv
  .slice(2,);

if ((!pkgDir) || (!pkgName)) {
  throw new Error('usage: bun mise.switch-tests-byname.ts <pkgDir> <pkgName> [--dry]',);
}

const dry = dryFlag === '--dry';

/**
 Extract the quoted module specifier from one import/export line.
 
 Returns the specifier text plus the byte offsets of the surrounding quotes,
 so the caller can splice a replacement without disturbing the rest of the line.
 
 @param line - One source line to inspect
 
 @returns Specifier and quote offsets, or null when the line has no `from '...'`
   / `import '...'` clause
 */
function extractSpecifier(line: string,): {
  spec: string;
  openQuote: number;
  closeQuote: number
} | null {
  const fromIdx = line.indexOf('from ',);
  const sideEffectIdx = line.trimStart()
    .startsWith('import ',)
    && line.includes('\'',)
    && (!line.includes('{',))
    ? line.indexOf('import ',)
    : -1;
  const anchor = fromIdx !== (-1) ? fromIdx : sideEffectIdx;
  if (anchor < 0) {
    return null;
  }
  const singleOpen = line.indexOf(
    '\'',
    anchor,
  );
  const doubleOpen = line.indexOf(
    '"',
    anchor,
  );
  const openQuote = (singleOpen !== (-1)) && ((doubleOpen === (-1)) || (singleOpen < doubleOpen)) ? singleOpen : doubleOpen;
  if (openQuote < 0) {
    return null;
  }
  const quoteChar = line[openQuote];
  const closeQuote = line.indexOf(
    quoteChar,
    openQuote + 1,
  );
  if (closeQuote === (-1)) {
    return null;
  }
  return {
    spec: line.slice(
      openQuote + 1,
      closeQuote,
    ),
    openQuote,
    closeQuote,
  };
}

/**
 Decide whether a specifier targets an own non-test sibling source module.
 
 @param spec - Module specifier text
 
 @param fileDir - Directory of the importing test file (for relative resolution)
 
 @returns True when spec is a relative `.ts` (not `.unit.test.ts`) resolving to
   an existing file
 */
function isOwnSourceImport({
  spec,
  fileDir,
}: {
  readonly spec: string;
  readonly fileDir: string
},): boolean {
  if (!spec.startsWith('.',)) {
    return false;
  }
  if ((!spec.endsWith('.ts',)) || spec.endsWith('.unit.test.ts',)) {
    return false;
  }
  return existsSync(resolve(
    fileDir,
    spec,
  ),);
}

const glob = new Glob('src/**/*.unit.test.ts',);
const switched: string[] = [];
const skippedMulti: string[] = [];

for await (const rel of glob.scan(pkgDir,)) {
  const filePath = resolve(
    pkgDir,
    rel,
  );
  const fileDir = dirname(filePath,);
  const text = await Bun.file(filePath,)
    .text();
  const lines = text.split('\n',);
  const ownSourceLineIndexes = lines.flatMap(function collectOwnSourceLine(
    line,
    index,
  ) {
    const extracted = extractSpecifier(line,);
    if (extracted && isOwnSourceImport({
      spec: extracted.spec,
      fileDir,
    },)) {
      return [index,];
    }
    return [];
  },);

  if (ownSourceLineIndexes.length === 0) {
    continue;
  }
  if (ownSourceLineIndexes.length > 1) {
    skippedMulti.push(`${rel} (${String(ownSourceLineIndexes.length,)} own-source imports — merge by hand)`,);
    continue;
  }

  const lineIndex = ownSourceLineIndexes[0]!;
  const line = lines[lineIndex]!;
  const extracted = extractSpecifier(line,)!;
  const rewritten = line.slice(
    0,
    extracted.openQuote + 1,
  ) + pkgName
    + line.slice(extracted.closeQuote,);
  lines[lineIndex] = rewritten;
  if (!dry) {
    await Bun.write(
      filePath,
      lines.join('\n',),
    );
  }
  switched.push(`${rel}: ${extracted.spec} -> ${pkgName}`,);
}

console.error(`${dry ? '[dry] ' : ''}switched ${String(switched.length,)} file(s) in ${pkgDir}:`,);
for (const entry of switched) {
  console.error(`  ${entry}`,);
}
if (skippedMulti.length > 0) {
  console.error(`SKIPPED ${String(skippedMulti.length,)} file(s) with multiple own-source imports (handle manually):`,);
  for (const entry of skippedMulti) {
    console.error(`  ${entry}`,);
  }
}
