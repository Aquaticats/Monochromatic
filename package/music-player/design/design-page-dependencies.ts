/**
 One reading of what a music player design page asks the browser for: the files it names, the typefaces it sets,
 and which of those names answer inside this tree. Both offline guards read a page through this module, so the real
 pages and the fixtures that prove a guard can fail are read the same way.
 */

import { globSync, readFileSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';

/** Directory holding the pages, the Design Component runtime and the vendored builds. */
export const designRoot = import.meta.dirname;

/**
 Typefaces the design system ships beside the pages: one `@font-face` rule each in the Material Design 3 bundle's
 `tokens/fonts.css`. A page may name these and nothing it has to fetch.
 */
export const SHIPPED_FONT_FAMILIES = [
  'Roboto Flex',
  'Google Sans',
  'Google Sans Text',
  'Material Symbols',
] as const;

/** Families every browser resolves without a stylesheet or a font file. */
const SYSTEM_FONT_FAMILIES = [
  'system-ui',
  'ui-monospace',
  'ui-sans-serif',
  'ui-serif',
  'sans-serif',
  'serif',
  'monospace',
  'cursive',
  'fantasy',
] as const;

/** Every family a page may set without adding a dependency. */
export const ALLOWED_FONT_FAMILIES: ReadonlySet<string> = new Set<string>([
  ...SHIPPED_FONT_FAMILIES,
  ...SYSTEM_FONT_FAMILIES,
]);

/** Where a reference points: inside this tree, or at a scheme this tree cannot answer. */
export type ReferenceKind = 'local' | 'fragment' | 'embedded' | 'remote';

/** Pages under `root`, relative to it, in a stable order. */
export function designPages(root: string = designRoot,): readonly string[] {
  return globSync(
    [
      '*.dc.html',
      'candidates/*.dc.html',
    ],
    { cwd: root, },
  ).sort();
}

/** Every `src` and `href` value in a document, in source order. */
export function pageReferences(html: string,): readonly string[] {
  const pattern = /\b(?:src|href)\s*=\s*(?:"([^"]*)"|'([^']*)')/giu;
  return [...html.matchAll(pattern,)].map(function pick(match,): string {
    return match[1] ?? match[2] ?? '';
  },);
}

/** Every `url()` and `@import` target in a stylesheet, in source order. */
export function stylesheetReferences(css: string,): readonly string[] {
  const pattern
    = /url\(\s*(?:"([^"]*)"|'([^']*)'|([^)'"]*))\s*\)|@import\s+(?:"([^"]*)"|'([^']*)')/giu;
  return [...css.matchAll(pattern,)].map(function pick(match,): string {
    return (match[1] ?? match[2] ?? match[3] ?? match[4] ?? match[5] ?? '').trim();
  },);
}

/** Which tree answers a reference. */
export function classifyReference(reference: string,): ReferenceKind {
  if (reference.startsWith('#',)) {
    return 'fragment';
  }
  if (/^data:/iu.test(reference,)) {
    return 'embedded';
  }
  if (/^\/\//u.test(reference,) || /^[a-z][a-z\d+.-]*:/iu.test(reference,)) {
    return 'remote';
  }
  return 'local';
}

/**
 Absolute path a local reference names, read from the file holding it, because every page resolves its `src` and
 `href` values against its own directory.
 */
export function resolveReference(
  {
    fromFile,
    reference,
  }: {
    fromFile: string;
    reference: string;
  },
): string {
  return resolve(
    dirname(fromFile,),
    reference,
  );
}

/** Path a file is written as in a finding, relative to the tree the guard read. */
export function displayPath(
  {
    root,
    file,
  }: {
    root: string;
    file: string;
  },
): string {
  return relative(
    root,
    file,
  );
}

/** Every `font-family` declaration value in a document, without repeats. */
export function fontFamilyValues(html: string,): readonly string[] {
  const pattern = /(?:^|[^a-z-])font-family\s*:\s*([^;{}"]*)/giu;
  const values = new Set<string>();
  for (const match of html.matchAll(pattern,)) {
    values.add(match[1].trim(),);
  }
  return [...values].sort();
}

/**
 `font` shorthand declarations in a document that could hide a family. A shorthand hides the family inside a value
 this module does not parse, so a page carrying one is reported instead of read; the CSS-wide keywords set no
 family and are not reported.
 */
export function fontShorthandValues(html: string,): readonly string[] {
  const pattern = /(?:^|[^a-z-])font\s*:\s*([^;{}"]*)/giu;
  const keywords = new Set<string>([
    'inherit',
    'initial',
    'revert',
    'revert-layer',
    'unset',
  ],);
  return [...html.matchAll(pattern,)]
    .map(function pick(match,): string {
      return match[1].trim();
    },)
    .filter(function notKeyword(value,): boolean {
      return !keywords.has(value.toLowerCase(),);
    },);
}

/** Individual family names inside one `font-family` value. */
export function familyNames(value: string,): readonly string[] {
  return value
    .split(',',)
    .map(function unquote(part,): string {
      return part.trim().replace(
        /^["']|["']$/gu,
        '',
      ).trim();
    },)
    .filter(function present(name,): boolean {
      return name.length > 0;
    },);
}

/** Families a stylesheet defines with `@font-face`. */
export function facesDefinedBy(css: string,): readonly string[] {
  const pattern = /@font-face\s*\{[^}]*?font-family\s*:\s*([^;{}]*)/giu;
  return [...css.matchAll(pattern,)]
    .map(function pick(match,): string {
      return match[1].trim().replace(
        /^["']|["']$/gu,
        '',
      ).trim();
    },)
    .filter(function present(name,): boolean {
      return name.length > 0;
    },);
}

/** Text of a file, or `null` when the path is not a readable file. */
export function readText(file: string,): string | null {
  try {
    return readFileSync(
      file,
      'utf8',
    );
  }
  catch {
    return null;
  }
}
