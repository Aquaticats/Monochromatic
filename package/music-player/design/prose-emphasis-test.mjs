import { globSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

// AGENTS.md rule WR3 allows bold emphasis only: never italics. This guard holds the package's authored text to it.
// Excluded on purpose, as for the dash guard: `_ds/` and `candidates/_ds/` are copied design-system bundles,
// `support.js` is the generated Design Component runtime, and `uploads/` holds files the user sent in.
const root = import.meta.dirname;
const authored = [
  ...globSync(['*.md', 'questions/*.template.html'], { cwd: root }),
  ...globSync(['*.dc.html', 'candidates/*.dc.html'], { cwd: root })
].sort();

const blank = value => value
  .split('\n')
  .map(line => ' '.repeat(line.length))
  .join('\n');

// Fenced blocks, code spans, styles and plain scripts hold code; `data-dc-script` blocks hold the rendered captions.
function proseMask(text, file) {
  let masked = text;
  if (file.endsWith('.md')) {
    masked = masked.replace(/^```[\s\S]*?^```/gmu, match => blank(match));
    return masked.replace(/`[^`\n]*`/gu, match => blank(match));
  }
  return masked.replace(/<(style|script)(?![^>]*data-dc-script)[^>]*>[\s\S]*?<\/\1>/giu, match => blank(match));
}

// A single asterisk pair holds italics; two hold bold and are what the rule asks for instead.
function italicSpans(line) {
  return [...line.matchAll(/(?<![\w*])\*([^*\s][^*\n]*?)\*(?![\w*])/gu)].map(match => match[0]);
}

function emphasisFindings(text, file) {
  const masked = proseMask(text, file);
  const findings = [];
  for (const [index, line] of masked.split('\n').entries()) {
    for (const span of italicSpans(line)) {
      findings.push({ file, line: index + 1, what: `italic ${span}`, text: line.trim() });
    }
    // One finding per element, then any tag left unpaired and any inline italic style.
    for (const element of line.matchAll(/<(em|i)\b[^>]*>[\s\S]*?<\/\1>/giu)) {
      findings.push({ file, line: index + 1, what: `italic element ${element[1]}`, text: line.trim() });
    }
    const withoutElements = line.replace(/<(em|i)\b[^>]*>[\s\S]*?<\/\1>/giu, '');
    if (/<\/?(?:em|i)\b/iu.test(withoutElements)) {
      findings.push({ file, line: index + 1, what: 'unpaired italic tag', text: line.trim() });
    }
    if (/font-style:\s*italic/iu.test(line)) {
      findings.push({ file, line: index + 1, what: 'italic style', text: line.trim() });
    }
  }
  return findings;
}

// A probe proves the scanner can see italics and reaches past them to bold before its silence counts.
const probe = emphasisFindings('# Report\nUse *this*, **that** and an <em>element</em>.\n', 'probe.md');
const probed = probe.map(item => item.what).join(', ');
if ((probe.length !== 2) || !probed.includes('italic *this*') || !probed.includes('italic element em')) {
  throw new Error(`Expected the emphasis scanner to find one italic span and one italic element, and no bold; it found ${probed}.`);
}

const findings = authored.flatMap(file => emphasisFindings(readFileSync(join(root, file), 'utf8'), file));
if (findings.length > 0) {
  const listed = findings.map(item => `${item.file}:${item.line}: ${item.what} in "${item.text}"`);
  throw new Error(`Design prose emphasizes with bold only (AGENTS.md WR3); found ${findings.length} italic spans:\n${listed.join('\n')}`);
}
console.log(`Design prose: no italic emphasis in ${authored.length} authored documents.`);
