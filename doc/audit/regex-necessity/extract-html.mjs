// Extract <script> blocks from tracked HTML files into mirror .js files with a
// line-offset marker, so the same parser-backed no-regex scan covers them.
import {
  readFileSync,
  writeFileSync,
  mkdirSync,
} from 'node:fs';
import { execFileSync, } from 'node:child_process';
import {
  join,
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

const HTML_MIRROR = join(
  AUDIT,
  'mirror-html'
);

const tracked = execFileSync(
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
)
  .split('\0')
  .filter((entry) => entry !== '');

const htmlFiles = tracked.filter((entry) => entry.endsWith('.html') || entry.endsWith('.htm')
  || entry.endsWith('.vue')
  || entry.endsWith('.astro'));
const yamlFiles = tracked.filter((entry) => entry.endsWith('.yml') || entry.endsWith('.yaml'));
const kotlinFiles = tracked.filter((entry) => entry.endsWith('.kt') || entry.endsWith('.kts'));

const manifest = [];
let blocks = 0;

for (const file of htmlFiles) {
  let text;
  try {
    text = readFileSync(
      join(
        ROOT,
        file
      ),
      'utf8'
    );
  } catch (error) {
    continue;
  }
  // Walk <script ...> ... </script> spans, recording the 1-based line of each opener.
  let cursor = 0;
  let index = 0;
  while (cursor < text.length) {
    const open = text.indexOf(
      '<script',
      cursor
    );
    if (open === (-1)) break;
    const openEnd = text.indexOf(
      '>',
      open
    );
    if (openEnd === (-1)) break;
    const close = text.indexOf(
      '</script>',
      openEnd
    );
    if (close === (-1)) break;
    const body = text.slice(
      openEnd + 1,
      close
    );
    if (body.trim() !== '') {
      const startLine = text.slice(
        0,
        openEnd + 1
      )
        .split('\n')
        .length;
      const target = join(
        HTML_MIRROR,
        `${file.replaceAll(
          '/',
          '__'
        )}.${index}.js`
      );
      mkdirSync(
        dirname(target),
        { recursive: true, }
      );
      writeFileSync(
        target,
        body,
        'utf8'
      );
      manifest.push({
        source: file,
        block: index,
        startLine,
        target: target.slice(AUDIT.length + 1),
        kind: 'html-script',
      });
      blocks += 1;
    }
    index += 1;
    cursor = close + 9;
  }
}

// YAML: inline shell or script strings are not parseable as JS reliably, so record
// them for a text-level review instead of a parser scan.
const yamlHits = [];
for (const file of yamlFiles) {
  let lines;
  try {
    lines = readFileSync(
      join(
        ROOT,
        file
      ),
      'utf8'
    )
      .split('\n');
  } catch (error) {
    continue;
  }
  lines.forEach((
    line,
    position
  ) => {
    if (/new RegExp|\.replace\(\/|\.match\(\/|\.matchAll\(\/|\.test\(/.test(line)) {
      yamlHits.push({
        source: file,
        line: position + 1,
        text: line.trim()
          .slice(
          0,
          240
          ),
      });
    }
  });
}

writeFileSync(
  join(
    AUDIT,
    'html-manifest.json'
  ),
  JSON.stringify(
    {
      manifest,
      yamlHits,
      counts: {
        htmlFiles: htmlFiles.length,
        kotlinFiles: kotlinFiles.length,
        blocks,
        yamlHits: yamlHits.length,
      },
    },
    null,
    2
  ),
  'utf8'
);
console.log(JSON.stringify(
  {
    htmlFiles: htmlFiles.length,
    kotlinFiles: kotlinFiles.length,
    scriptBlocks: blocks,
    yamlHits: yamlHits.length,
  },
  null,
  2
));
