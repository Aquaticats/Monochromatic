// HTML script-block diagnostics report the whole call expression, so the regex
// belonging to each site has to be pulled out of the span the same way sites3.mjs
// does for JS and TS: the last regex literal inside the span.
import {
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { homedir, } from 'node:os';

const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;

const REGEX_PRECEDERS = new Set([
  '(',
  ',',
  '=',
  ':',
  '[',
  '!',
  '&',
  '|',
  '?',
  '{',
  '}',
  ';',
  '+',
  '-',
  '*',
  '%',
  '~',
  '^',
  '<',
  '>',
  ''
]);

/**
 Every regex literal in a source slice, scanning left to right. A `/` opens a
 literal when the previous significant character can precede one.
 */
function regexLiteralsIn(text) {
  const found = [];
  let index = 0;
  let previousSignificant = '';
  while (index < text.length) {
    const char = text[index];
    if ((char === '"') || (char === "'")
      || (char === '`')) {
      const quote = char;
      index += 1;
      while ((index < text.length) && (text[index] !== quote)) index += text[index] === '\\' ? 2 : 1;
      index += 1;
      previousSignificant = 'x';
      continue;
    }
    if ((char === '/') && (text[index + 1] === '/')) {
      while ((index < text.length) && (text[index] !== '\n')) index += 1;
      continue;
    }
    if ((char === '/') && (text[index + 1] === '*')) {
      const end = text.indexOf(
        '*/',
        index + 2
      );
      index = end === (-1) ? text.length : end + 2;
      continue;
    }
    if ((char === '/') && REGEX_PRECEDERS.has(previousSignificant)) {
      let cursor = index + 1;
      let inClass = false;
      let closed = false;
      while (cursor < text.length) {
        const current = text[cursor];
        if (current === '\\') {
          cursor += 2;
          continue;
        }
        if (current === '[') inClass = true;
        else if (current === ']') inClass = false;
        else if ((current === '/') && (!inClass)) {
          closed = true;
          break;
        } else if ((current === '\n') && (!inClass)) break;
        cursor += 1;
      }
      if (closed) {
        let end = cursor + 1;
        while ((end < text.length) && 'dgimsuvy'.includes(text[end])) end += 1;
        found.push({
          start: index,
          text: text.slice(
            index,
            end
          ),
        });
        index = end;
        previousSignificant = 'x';
        continue;
      }
    }
    if (char.trim() !== '') previousSignificant = char;
    index += 1;
  }
  return found;
}

// Rebuild the HTML site list from the scan output so each site keeps the mirror
// block it came from, then resolve its own regex literal out of the span.
const scan = JSON.parse(readFileSync(
  join(
    AUDIT,
    'scan-html.json'
  ),
  'utf8'
));
const manifest = JSON.parse(readFileSync(
  join(
    AUDIT,
    'html-manifest.json'
  ),
  'utf8'
));
const blockByTarget = new Map(manifest.manifest
  .map((entry) => [
    entry.target,
    entry,
  ]));

const sites = [];
for (const diagnostic of scan.diagnostics ?? []) {
  const span = diagnostic.labels?.[0]
    ?.span;
  if (span === undefined) continue;
  const entry = blockByTarget.get(diagnostic.filename);
  if (entry === undefined) continue;
  sites.push({
    repoPath: entry.source,
    line: (entry.startLine + span.line) - 1,
    column: span.column,
    offset: span.offset,
    length: span.length,
    kind: diagnostic.message
      .startsWith('Regex literal')
      ? 'literal'
      : diagnostic.message
        .startsWith('RegExp constructor')
        ? 'constructor'
        : 'string-method',
    message: diagnostic.message,
    tracked: true,
    surface: 'html-script',
    target: diagnostic.filename,
  });
}
const blockCache = new Map();

let fixed = 0;
let unresolved = 0;
for (const site of sites) {
  const target = site.target ?? null;
  if (target === null) {
    unresolved += 1;
    continue;
  }
  if (!blockCache.has(target)) {
    // oxlint spans are UTF-8 byte offsets, so slice bytes and decode, never code units.
    blockCache.set(
      target,
      readFileSync(join(
        AUDIT,
        target
      ))
    );
  }
  const buffer = blockCache.get(target);
  const spanText = buffer.subarray(
    site.offset,
    site.offset + site.length
  )
    .toString('utf8');
  const isConstructor = spanText.trimStart()
    .startsWith('new RegExp')
    || spanText.trimStart()
      .startsWith('RegExp(');
  const literals = regexLiteralsIn(spanText);
  const own = isConstructor ? spanText.trim() : literals.at(-1)
    ?.text;
  if (own === undefined) {
    unresolved += 1;
    continue;
  }
  site.spanText = spanText.replaceAll(
    '\n',
    ' '
  )
    .slice(
    0,
    300
    );
  site.pattern = own.replaceAll(
    '\n',
    ' '
  )
    .slice(
    0,
    300
    );
  fixed += 1;
}

writeFileSync(
  join(
    AUDIT,
    'html-sites.json'
  ),
  JSON.stringify(
    sites,
    null,
    2
  ),
  'utf8'
);
console.log(JSON.stringify(
  {
    sites: sites.length,
    fixed,
    unresolved,
  },
  null,
  2
));
