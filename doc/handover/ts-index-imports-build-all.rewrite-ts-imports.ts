// One-shot, throwaway: collapse cross-package @monochromatic-dev import/export
// specifiers for ONE target package to its `/ts` index. Run per target so each
// commit is one logical unit (cli-git scoped-pathspec friendly).
//
//   bun mise.rewrite-ts-imports.ts <target-unscoped> [--dry]
//   bun mise.rewrite-ts-imports.ts module-test
//   bun mise.rewrite-ts-imports.ts module-logger --dry
//
// Safety: only rewrites specifiers in import/export context (preceded by a
// word-boundaried `from`, `import`, or `import(`), so bare string-literal
// package names (e.g. `const X = '@monochromatic-dev/git-policy-cli'`, oxlint rule
// config `package: '@monochromatic-dev/logger-core/types'`) are untouched.
// Skips files owned by the target itself (self-imports).
import { Glob } from 'bun';
import {
  readFileSync,
  writeFileSync,
  existsSync
} from 'node:fs';
import {
  dirname,
  join
} from 'node:path';

const target = process.argv[2];
const dry = process.argv
  .includes('--dry');
if ((!target) || target.startsWith('--')) throw new Error('usage: bun mise.rewrite-ts-imports.ts <target-unscoped> [--dry]');

// Archived beside doc/handover/ts-index-imports-build-all.md; the package/
// tree this scans lives two directories up from doc/handover/.
const root = join(
  import.meta.dir,
  '..',
  '..'
);
const scoped = `@monochromatic-dev/${target}`;
const canonical = `${scoped}/ts`;

// owner package name cache by directory
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

// import/export context + boundary-safe target match; replacement keeps the
// leading `from `/`import `/`import(` and the quote, swaps only the specifier.
const re = new RegExp(
  `(?<![A-Za-z0-9_$])(from|import)(\\s*\\(?\\s*)(['"])${scoped.replaceAll(
    /[.*+?^${}()|[\]\\]/g,
    String.raw`\$&`
  )}(?:/[^'"]*)?\\3`,
  'g',
);

const changed: string[] = [];
let totalHits = 0;
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
  let hits = 0;
  const next = text.replaceAll(
    re,
    (
      _m,
      kw: string,
      gap: string,
      q: string
    ) => {
      hits += 1;
      return `${kw}${gap}${q}${canonical}${q}`;
    }
  );
  if (next !== text) {
    totalHits += hits;
    changed.push(rel);
    if (!dry) writeFileSync(
      file,
      next
    );
  }
}

console.error(`${dry ? '[dry] ' : ''}target ${scoped}: ${totalHits} specifier(s) in ${changed.length} file(s)`);
for (const c of changed) console.log(c);
