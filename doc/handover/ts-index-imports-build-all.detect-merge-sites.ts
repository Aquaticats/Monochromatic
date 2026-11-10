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

// The import statement grammar below is scanned character by character; no pattern is
// compiled. A whitespace character is one member of the ECMAScript `\s` set, and a
// word character is one member of the `\w` set, so both match what the grammar they
// replaced accepted.
const FROM_KEYWORD = 'from';
const IMPORT_KEYWORD = 'import';
const TYPE_KEYWORD = 'type';
const WORD_CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_';
const WHITESPACE_CHARACTERS = ' \t\n\v\f\r\u00a0\u1680\u2000\u2001\u2002\u2003\u2004'
  + '\u2005\u2006\u2007\u2008\u2009\u200a\u2028\u2029\u202f\u205f\u3000\ufeff';

function isWhitespaceCharacter(character: string): boolean {
  return (character !== '') && WHITESPACE_CHARACTERS.includes(character);
}

// The first index at or after `index` that holds a non-whitespace character, or the
// end of the text.
function skipWhitespace(
  text: string,
  index: number
): number {
  let cursor = index;
  while (isWhitespaceCharacter(text.charAt(cursor))) cursor += 1;
  return cursor;
}

// The content of a quoted run holding at least one character, `['"][^'"]+['"]`,
// starting at `index`; null when no such run starts there.
function quotedContentAt(
  text: string,
  index: number
): string | null {
  const opening = text.charAt(index);
  if ((opening !== '"') && (opening !== "'")) return null;
  let cursor = index + 1;
  while ((cursor < text.length) && (text.charAt(cursor) !== '"')
    && (text.charAt(cursor) !== "'")) cursor += 1;
  if (cursor === (index + 1)) return null;
  const closing = text.charAt(cursor);
  if ((closing !== '"') && (closing !== "'")) return null;
  return text.slice(
    index + 1,
    cursor
  );
}

// `^import\s+['"][^'"]+['"]`: an import with no bindings, so no `from` clause.
function isSideEffectImport(text: string): boolean {
  if (!text.startsWith(IMPORT_KEYWORD)) return false;
  const contentStart = skipWhitespace(
    text,
    IMPORT_KEYWORD.length
  );
  if (contentStart === IMPORT_KEYWORD.length) return false;
  return quotedContentAt(
    text,
    contentStart
  ) !== null;
}

// Whether a one-character string is a member of the boundary test's word set.
function isWordCharacter(character: string): boolean {
  return (character !== '') && WORD_CHARACTERS.includes(character);
}

// `^import\s+type\b`: a type-only import, from which no runtime binding comes.
function isTypeImport(text: string): boolean {
  if (!text.startsWith(IMPORT_KEYWORD)) return false;
  const typeStart = skipWhitespace(
    text,
    IMPORT_KEYWORD.length
  );
  if (typeStart === IMPORT_KEYWORD.length) return false;
  if (!text.startsWith(
    TYPE_KEYWORD,
    typeStart
  )) return false;
  return !isWordCharacter(text.charAt(typeStart + TYPE_KEYWORD.length));
}

// The specifier of the first `from\s*['"][^'"]+['"]` clause in `text`, or null.
// The clause is unanchored, so every `from` is tried in turn and the walk resumes
// after the keyword that failed.
function findFromSpecifier(text: string): string | null {
  let from = text.indexOf(FROM_KEYWORD);
  while (from !== -1) {
    const specifier = quotedContentAt(
      text,
      skipWhitespace(
        text,
        from + FROM_KEYWORD.length
      )
    );
    if (specifier !== null) return specifier;
    from = text.indexOf(
      FROM_KEYWORD,
      from + 1
    );
  }
  return null;
}

// The specifier of a bare `^import\s*['"][^'"]+['"]` clause at the start of `text`,
// or null. No whitespace is required, which is what separates it from the
// side-effect test.
function findBareImportSpecifier(text: string): string | null {
  if (!text.startsWith(IMPORT_KEYWORD)) return null;
  return quotedContentAt(
    text,
    skipWhitespace(
      text,
      IMPORT_KEYWORD.length
    )
  );
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
      while ((findFromSpecifier(stmt) === null) && (!isSideEffectImport(stmt.trimStart()))
        && (j < (lines.length
          - 1))) {
        j += 1;
        stmt += `\n${  lines[j]}`;
        if ((j - i) > 50) break;
      }
      const isType = isTypeImport(stmt.trimStart());
      const specifier = findFromSpecifier(stmt) ?? findBareImportSpecifier(stmt);
      if (specifier !== null) out.push({
        isType,
        specifier
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
