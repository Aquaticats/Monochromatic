import { globSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

// AGENTS.md rule WR2 bans em dashes, en dashes and their ASCII substitutes from prose, and this guard holds the
// package's authored text to it. Excluded on purpose: `_ds/` and `candidates/_ds/` are copied design-system bundles,
// `support.js` is the generated Design Component runtime, and `uploads/` holds files the user sent in, one of which
// parses filename separators as data.
const root = import.meta.dirname;
const authored = [
  ...globSync(['*.md', 'evidence/**/*.md', 'questions/*.template.html'], { cwd: root }),
  ...globSync(['*.dc.html', 'candidates/*.dc.html', 'candidates/*.js'], { cwd: root })
    .filter(file => !file.endsWith('/support.js')),
  'Decisions.dc.html'
].sort();

const blank = value => value
  .split('\n')
  .map(line => ' '.repeat(line.length))
  .join('\n');

// Fenced blocks, inline code spans, styles and plain scripts hold code, not prose; `data-dc-script` blocks hold the
// rendered captions, so they stay in scope.
function proseMask(text, file) {
  let masked = text;
  if (file.endsWith('.md')) {
    masked = masked.replace(/^```[\s\S]*?^```/gmu, match => blank(match));
    masked = masked.replace(/`[^`\n]*`/gu, match => blank(match));
  } else if (file.endsWith('.html')) {
    masked = masked.replace(/<(style|script)(?![^>]*data-dc-script)[^>]*>[\s\S]*?<\/\1>/giu, match => blank(match));
  }
  return masked;
}

function dashFindings(text, file) {
  const masked = proseMask(text, file);
  const findings = [];
  for (const [index, line] of text.split('\n').entries()) {
    const visible = masked.split('\n')[index] ?? '';
    for (const match of visible.matchAll(/[—–]/gu)) {
      findings.push({
        file,
        line: index + 1,
        column: match.index + 1,
        character: match[0] === '—' ? 'em dash' : 'en dash',
        text: line.trim()
      });
    }
  }
  return findings;
}

// A probe proves the scanner can see what it forbids before its silence is trusted.
const probe = dashFindings('# Report — one, two\nand 2–3 more.\n', 'probe.md');
if (probe.length !== 2 || (probe[0].line !== 1) || (probe[1].line !== 2)) {
  throw new Error(`Expected the dash scanner to find one em dash and one en dash in the probe; it found ${probe.length}.`);
}

const findings = authored.flatMap(file => dashFindings(readFileSync(join(root, file), 'utf8'), file));
if (findings.length > 0) {
  const listed = findings.map(item => `${item.file}:${item.line}:${item.column}: ${item.character} in "${item.text}"`);
  throw new Error(`Design prose keeps no em or en dash (AGENTS.md WR2); found ${findings.length}:\n${listed.join('\n')}`);
}
console.log(`Design prose: no em or en dash in ${authored.length} authored documents.`);
