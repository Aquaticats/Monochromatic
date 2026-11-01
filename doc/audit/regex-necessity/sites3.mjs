// Re-extract sites with byte-exact spans, dedupe chained calls, and pull the
// regex literal that belongs to each reported call (the last literal in its span).
import {
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { execFileSync, } from 'node:child_process';
import { join, } from 'node:path';

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

const RULE = 'no-restricted-syntax/no-regex';

const raw = JSON.parse(readFileSync(
  join(
    AUDIT,
    'scan.json'
  ),
  'utf8'
));
const trackedText = execFileSync(
  'git',
  [
    '-C',
    ROOT,
    'ls-files',
    '-z'
  ],
  {
    encoding: 'utf8',
    maxBuffer: 1 << 28,
  }
);
const tracked = new Set(trackedText.split('\0')
  .filter((entry) => entry !== ''));

const bufferCache = new Map();
function bufferOf(repoPath) {
  if (bufferCache.has(repoPath)) return bufferCache.get(repoPath);
  let buffer = Buffer.alloc(0);
  try {
    buffer = readFileSync(join(
      ROOT,
      repoPath
    ));
  } catch (error) {
    buffer = Buffer.alloc(0);
  }
  bufferCache.set(
    repoPath,
    buffer
  );
  return buffer;
}
const lineCache = new Map();
function linesOf(repoPath) {
  if (lineCache.has(repoPath)) return lineCache.get(repoPath);
  const lines = bufferOf(repoPath)
    .toString('utf8')
    .split('\n');
  lineCache.set(
    repoPath,
    lines
  );
  return lines;
}

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
 Every regex literal in a source slice, as { start, text }, scanning left to right.
 A `/` opens a literal when the previous significant character can precede one.
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

function directiveRanges(lines) {
  const ranges = [];
  let openBlock = null;
  for (let index = 0; index < lines.length; index += 1) {
    const text = lines[index];
    const lineNumber = index + 1;
    const disableMatch = /oxlint-disable(-next-line|-line)?\b([^\n]*)/.exec(text);
    if (disableMatch !== null) {
      const verb = disableMatch[1] ?? '';
      const [rulesPart, justificationPart] = (disableMatch[2] ?? '').split('--');
      const ruleText = (rulesPart ?? '').replaceAll(
        '*/',
        ' '
      )
        .trim();
      const rules = ruleText === '' ? [] : ruleText.split(/[,\s]+/)
        .filter((entry) => entry !== '');
      const justification = (justificationPart ?? '').replaceAll(
        '*/',
        ' '
      )
        .trim();
      if ((rules.length === 0) || rules.some((entry) => entry === RULE)) {
        if (verb === '-next-line') ranges.push({
          firstLine: lineNumber + 1,
          lastLine: lineNumber + 1,
          kind: 'next-line',
          justification,
        });
        else if (verb === '-line') ranges.push({
          firstLine: lineNumber,
          lastLine: lineNumber,
          kind: 'line',
          justification,
        });
        else openBlock = {
          firstLine: lineNumber,
          kind: 'block',
          justification,
        };
      }
    }
    const enableMatch = /oxlint-enable\b([^\n]*)/.exec(text);
    if ((enableMatch !== null) && (openBlock !== null)) {
      const ruleText = (enableMatch[1] ?? '').replaceAll(
        '*/',
        ' '
      )
        .trim();
      const rules = ruleText === '' ? [] : ruleText.split(/[,\s]+/)
        .filter((entry) => entry !== '');
      if ((rules.length === 0) || rules.some((entry) => entry === RULE)) {
        ranges.push({
          ...openBlock,
          lastLine: lineNumber,
        });
        openBlock = null;
      }
    }
  }
  if (openBlock !== null) ranges.push({
    ...openBlock,
    lastLine: lines.length,
  });
  return ranges;
}

const rangeCache = new Map();
function rangesOf(repoPath) {
  if (rangeCache.has(repoPath)) return rangeCache.get(repoPath);
  const ranges = directiveRanges(linesOf(repoPath));
  rangeCache.set(
    repoPath,
    ranges
  );
  return ranges;
}

// Dedupe on the exact span: chained calls share a start offset but differ in length.
const seen = new Set();
const sites = [];
for (const diagnostic of raw.diagnostics ?? []) {
  const span = diagnostic.labels?.[0]
    ?.span;
  if (span === undefined) continue;
  const repoPath = diagnostic.filename
    .replace(
    /^mirror\//,
    ''
    );
  const key = `${repoPath}:${span.offset}:${span.length}`;
  if (seen.has(key)) continue;
  seen.add(key);
  const buffer = bufferOf(repoPath);
  const spanText = buffer.subarray(
    span.offset,
    span.offset + span.length
  )
    .toString('utf8');
  const literals = regexLiteralsIn(spanText);
  const isConstructor = spanText.startsWith('new RegExp') || spanText.startsWith('RegExp(');
  const ownLiteral = isConstructor ? spanText : (literals.at(-1)
    ?.text
    ?? spanText);
  const lines = linesOf(repoPath);
  const covering = rangesOf(repoPath)
    .find((range) => (span.line >= range.firstLine) && (span.line <= range.lastLine));
  sites.push({
    repoPath,
    line: span.line,
    column: span.column,
    offset: span.offset,
    length: span.length,
    tracked: tracked.has(repoPath),
    kind: diagnostic.message
      .startsWith('Regex literal')
      ? 'literal'
      : diagnostic.message
        .startsWith('RegExp constructor')
        ? 'constructor'
        : 'string-method',
    message: diagnostic.message,
    spanText: spanText.replaceAll(
      '\n',
      ' '
    )
      .slice(
      0,
      400
      ),
    pattern: ownLiteral.replaceAll(
      '\n',
      ' '
    )
      .slice(
      0,
      400
      ),
    sourceLine: (lines[span.line - 1] ?? '').trim()
      .slice(
      0,
      400
      ),
    covered: covering !== undefined,
    directiveKind: covering?.kind ?? null,
    justification: covering?.justification ?? null,
  });
}

writeFileSync(
  join(
    AUDIT,
    'sites3.json'
  ),
  JSON.stringify(
    sites,
    null,
    2
  ),
  'utf8'
);
console.log(
  'unique sites:',
  sites.length
);
console.log(
  'tracked:',
  sites.filter((site) => site.tracked)
    .length
);
console.log(
  'covered by a directive:',
  sites.filter((site) => site.covered)
    .length
);
