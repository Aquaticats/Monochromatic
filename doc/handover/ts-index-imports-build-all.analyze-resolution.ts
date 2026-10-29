// One-shot, throwaway: the Phase-C success-criterion gate.
//
//   bun doc/handover/ts-index-imports-build-all.analyze-resolution.ts
//
// For every `@monochromatic-dev/<pkg>[/subpath]` specifier in import/export
// context under package/**, resolve the subpath against the TARGET package's
// `exports` and classify the resolved file as `src` or `dist`. Reports any
// CROSS-package (importer != target), NON-exempt import that still resolves to
// `dist`. The locked task is done when that count is 0.
//
// Self-imports are EXCLUDED: the by-name lib smoke tests deliberately import a
// package's own `.` (built dist) to exercise the bundle, e.g. build-tool-css's
// build.unit.test.ts importing '@monochromatic-dev/css-cli'. Those are
// the intended dist resolutions and not violations.
import { Glob } from 'bun';
import {
  readFileSync,
  existsSync
} from 'node:fs';
import {
  dirname,
  join
} from 'node:path';

// Archived beside doc/handover/ts-index-imports-build-all.md; the package/
// tree this scans lives two directories up from doc/handover/.
const root = join(
  import.meta.dir,
  '..',
  '..'
);
const SCOPE = '@monochromatic-dev/';

// Targets whose imports are not rewritten and which produce no source-to-dist
// transformation (plan "Exemptions" + hook-types decided exempt this session).
// An import of one of these is never a violation regardless of where it lands.
const EXEMPT_EXACT = new Set([
  'config-typescript',
  'config-dprint',
  'config-stylelint',
  'config-cosign',
  'config-tofu',
  'config-dotfiles',
  'config-tsdown',
  'config-oxlint',
  'config-oxlint-no-restricted-syntax',
  'config-oxlint-stylistic',
  'config-oxlint-tsdoc',
  'claude-code-plugin-source',
  'claude-code-plugin-hook-type',
]);
function isExemptTarget(unscoped: string): boolean {
  if (EXEMPT_EXACT.has(unscoped)) return true;
  if (unscoped.startsWith('test-fixture-')) return true;
  if (unscoped.startsWith('shim-')) return true;
  if (unscoped.startsWith('stub-')) return true;
  return false;
}

// --- Build name -> { dir, exports } map from every package.json under package/.
type Pkg = {
  name: string;
  dir: string;
  exports: unknown
};
const pkgByName = new Map<string, Pkg>();
for (const rel of new Glob('package/**/package.json').scanSync(root)) {
  if (rel.includes('/node_modules/') || rel.includes('/dist/')) continue;
  const file = join(
    root,
    rel
  );
  const json = JSON.parse(readFileSync(
    file,
    'utf8'
  ));
  if (((typeof json.name) === 'string')
    && json.name
      .startsWith(SCOPE)) {
    pkgByName.set(
      json.name,
      {
        name: json.name,
        dir: dirname(file),
        exports: json.exports
      }
    );
  }
}

// owner package name cache by directory (which package a source file lives in).
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

// Resolve one exports subkey ('.' | './ts' | './ts/foo.ts' | './scope') to its
// string target, expanding a single '*' for the ./ts/* pattern. Conditional
// objects collapse to default||node||import||types (the value a bundler picks).
function condTarget(v: unknown): string | null {
  if ((typeof v) === 'string') return v;
  if (v && ((typeof v) === 'object')) {
    const o = v as Record<string, unknown>;
    for (const k of [
      'default',
      'node',
      'import',
      'types'
    ]) {
      if ((typeof o[k]) === 'string') return o[k] as string;
    }
  }
  return null;
}
function resolveExport(
  exportsObj: unknown,
  subkey: string
): string | null {
  if ((!exportsObj) || ((typeof exportsObj) !== 'object')) {
    // string exports shorthand only matches '.'
    return subkey === '.' ? condTarget(exportsObj) : null;
  }
  const map = exportsObj as Record<string, unknown>;
  if (subkey in map) return condTarget(map[subkey]);
  // wildcard patterns, e.g. './ts/*'
  for (const [pat, val] of Object.entries(map)) {
    if (!pat.includes('*')) continue;
    const [pre, post] = pat.split('*');
    if (subkey.startsWith(pre) && subkey.endsWith(post)
      && (subkey.length >= (pre.length
        + post.length))) {
      const star = subkey.slice(
        pre.length,
        subkey.length - post.length
      );
      const tv = condTarget(val);
      return tv === null ? null : tv.replace(
        '*',
        star
      );
    }
  }
  return null;
}

// Map a bare/subpath specifier to its exports subkey: '@m/pkg' -> '.',
// '@m/pkg/ts' -> './ts', '@m/pkg/ts/x.ts' -> './ts/x.ts'.
function subkeyFor(
  unscoped: string,
  specifier: string
): string {
  const rest = specifier.slice(SCOPE.length + unscoped.length); // '' | '/ts' | '/ts/x.ts'
  return rest === '' ? '.' : `.${rest}`;
}

// import/export-context specifier scanner (same boundary rule as the rewriter).
const re = new RegExp(
  `(?<![A-Za-z0-9_$])(?:from|import)\\s*\\(?\\s*['"](${SCOPE.replaceAll(
    /[.*+?^${}()|[\]\\]/g,
    String.raw`\$&`
  )}[^'"]+)['"]`,
  'g',
);

type Finding = {
  readonly file: string;
  readonly importer: string;
  readonly specifier: string;
  readonly subkey: string;
  readonly resolved: string | null;
  readonly bucket: string
};
const findings: Finding[] = [];
const names = [...pkgByName.keys()].map(n => n.slice(SCOPE.length))
  .toSorted((
    a,
    b
  ) => b.length - a.length);

for (const rel of new Glob('package/**/*.{ts,tsx,mts,cts}').scanSync(root)) {
  if (rel.includes('/dist/') || rel.includes('/node_modules/')) continue;
  const file = join(
    root,
    rel
  );
  const text = readFileSync(
    file,
    'utf8'
  );
  if (!text.includes(SCOPE)) continue;
  const importer = ownerOf(file);
  for (const m of text.matchAll(re)) {
    const specifier = m[1];
    const tail = specifier.slice(SCOPE.length);
    // longest matching package name is the target
    const unscoped = names.find(n => (tail === n) || tail.startsWith(`${n}/`));
    if (!unscoped) { findings.push({
      file: rel,
      importer: importer ?? '?',
      specifier,
      subkey: '?',
      resolved: null,
      bucket: 'unknown-target'
    });
      continue; }
    const targetName = SCOPE + unscoped;
    const isSelf = importer === targetName;
    const exempt = isExemptTarget(unscoped);
    const subkey = subkeyFor(
      unscoped,
      specifier
    );
    const resolved = resolveExport(
      pkgByName.get(targetName)!
        .exports,
      subkey
    );
    let bucket: string;
    if (exempt) bucket = 'exempt-target';
    else if (isSelf) bucket = 'self-import';
    else if (resolved === null) bucket = 'UNRESOLVED';
    else if (resolved.includes('/dist/')) bucket = 'CROSS->DIST';
    else if (resolved.includes('/src/')) bucket = 'cross->src';
    else bucket = `cross->other(${resolved})`;
    findings.push({
      file: rel,
      importer: importer ?? '?',
      specifier,
      subkey,
      resolved,
      bucket
    });
  }
}

const byBucket = new Map<string, Finding[]>();
for (const f of findings) (byBucket.get(f.bucket)
  ?? byBucket.set(
    f.bucket,
    []
  )
    .get(f.bucket)!).push(f);

console.log(`total @monochromatic-dev import-context specifiers: ${findings.length}\n`);
for (const [bucket, fs] of [...byBucket.entries()].toSorted((
  a,
  b
) => b[1]
  .length
  - a[1]
    .length)) {
  console.log(`  ${bucket}: ${fs.length}`);
}

const violations = byBucket.get('CROSS->DIST') ?? [];
const unresolved = byBucket.get('UNRESOLVED') ?? [];
const unknown = byBucket.get('unknown-target') ?? [];
console.log('\n=== SUCCESS CRITERION: cross-package, non-exempt, non-self imports resolving to dist ===');
console.log(`CROSS->DIST violations: ${violations.length}`);
for (const v of violations) console.log(`  ${v.importer}  <-  ${v.specifier}  (${v.subkey} -> ${v.resolved})  [${v.file}]`);
if (unresolved.length > 0) {
  console.log(`\nUNRESOLVED (should be 0; broken imports):`);
  for (const v of unresolved) console.log(`  ${v.importer}  <-  ${v.specifier}  (${v.subkey})  [${v.file}]`);
}
if (unknown.length > 0) {
  console.log(`\nunknown-target (specifier matched no package name):`);
  for (const v of unknown) console.log(`  ${v.specifier}  [${v.file}]`);
}

// Also surface any self->dist (the by-name smokes) so the exclusion is auditable.
const selfDist = (byBucket.get('self-import') ?? []).filter(f => (f.resolved ?? '').includes('/dist/'));
console.log(`\nself-import -> dist (the intentional by-name lib smokes, excluded): ${selfDist.length}`);
for (const v of selfDist) console.log(`  ${v.importer} self-imports ${v.specifier}  [${v.file}]`);

const pass = (violations.length === 0) && (unresolved.length === 0)
  && (unknown.length === 0);
console.log(`\n${pass ? 'PASS' : 'FAIL'}: criterion ${pass ? 'met' : 'NOT met'}.`);
