// Throwaway: for a target package, simulate collapsing its cross-package
// imports to `<target>/ts`, then report files that would end up with DUPLICATE
// top-level import statements (same specifier AND same type-ness), which the
// oxlint `import/no-duplicates` rule (prefer-inline:false) flags. A value import
// and a type import from the same source are NOT duplicates and are not reported.
//
//   bun mise.detect-merge-sites.ts <target-unscoped>
import { Glob } from 'bun';
import {
  readFileSync,
  existsSync
} from 'node:fs';
import {
  dirname,
  join
} from 'node:path';

const target = process.argv[2];
if ((!target) || target.startsWith('--')) throw new Error('usage: bun mise.detect-merge-sites.ts <target-unscoped>');

// Archived beside doc/handover/ts-index-imports-build-all.md; the package/
// tree this scans lives two directories up from doc/handover/.
const root = join(
  import.meta.dir,
  '..',
  '..'
);
const scoped = `@monochromatic-dev/${target}`;

const ownerCache = new Map<string, string | null>();
function ownerOf(file: string): string | null {
  let d = dirname(file);
  const seen: string[] = [];
  while (d.startsWith(root)) {
    if (ownerCache.has(d)) {
      const cached = ownerCache.get(d)!;
      for (const s of seen) ownerCache.set(
        s,
        cached
      );
      return cached;
    }
    seen.push(d);
    const pj = join(
      d,
      'package.json'
    );
    if (existsSync(pj)) {
      const name = JSON.parse(readFileSync(
        pj,
        'utf8'
      ))
        .name
        ?? null;
      for (const s of seen) ownerCache.set(
        s,
        name
      );
      return name;
    }
    d = dirname(d);
  }
  for (const s of seen) ownerCache.set(
    s,
    null
  );
  return null;
}

// Parse top-level import statements. Each statement starts with `import` (maybe
// `import type`), may span multiple lines, ends at the line containing `from '...';`
// or (side-effect import) `import '...';`. Returns {isType, specifier} per stmt.
type ImportStmt = {
  isType: boolean;
  specifier: string
};
function parseImports(text: string): ImportStmt[] {
  const lines = text.split('\n');
  const out: ImportStmt[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trimStart();
    if (trimmed.startsWith('import ') || trimmed.startsWith('import{')
      || (trimmed === 'import')
      || trimmed.startsWith('import type')) {
      // collect until we see a closing `from '...'` or a bare `import '...'`
      let stmt = line;
      let j = i;
      while ((!/from\s*['"][^'"]+['"]/.test(stmt)) && (!/^import\s+['"][^'"]+['"]/.test(stmt.trimStart()))
        && (j < (lines.length
          - 1))) {
        j += 1;
        stmt += `\n${  lines[j]}`;
        if ((j - i) > 50) break;
      }
      const isType = /^import\s+type\b/.test(stmt.trimStart());
      const m = (/from\s*['"]([^'"]+)['"]/.exec(stmt)) ?? (/^import\s*['"]([^'"]+)['"]/.exec(stmt));
      if (m) out.push({
        isType,
        specifier: m[1]
      });
      i = j + 1;
      continue;
    }
    i += 1;
  }
  return out;
}

function collapse(spec: string): string {
  if ((spec === scoped) || spec.startsWith(`${scoped  }/`)) return `${scoped}/ts`;
  return spec;
}

const mergeFiles: {
  file: string;
  valueCount: number;
  typeCount: number
}[] = [];
for (const rel of new Glob('package/**/*.{ts,tsx,mts}').scanSync(root)) {
  if (rel.includes('/dist/') || rel.includes('/node_modules/')) continue;
  const file = join(
    root,
    rel
  );
  if (ownerOf(file) === scoped) continue;
  const text = readFileSync(
    file,
    'utf8'
  );
  if (!text.includes(scoped)) continue;
  const imports = parseImports(text);
  let typeCount = 0;
  let valueCount = 0;
  for (const imp of imports) {
    if (collapse(imp.specifier) === `${scoped}/ts`) {
      if (imp.isType) typeCount += 1;
      else valueCount += 1;
    }
  }
  // eslint/no-duplicate-imports (the active rule) counts a separate `import type`
  // and value `import` from the same source as duplicates, so any file with 2+
  // collapsed imports (value+type combined) is a merge site.
  if ((valueCount + typeCount) > 1) mergeFiles.push({
    file: rel,
    valueCount,
    typeCount
  });
}

console.error(`target ${scoped}: ${mergeFiles.length} file(s) with duplicate import statements after collapse`);
for (const m of mergeFiles) console.log(`  value=${m.valueCount} type=${m.typeCount}  ${m.file}`);
