#!/usr/bin/env node
// Probe editord's browser-provided find reference using only synthetic text and a disposable browser profile.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, join } from 'node:path';
import { pathToFileURL } from 'node:url';

const directory = mkdtempSync(join(tmpdir(), 'ide-find-reference-'));
const config = join(directory, 'agent-browser.json');
const profile = join(directory, 'profile');
const fixture = join(directory, 'fixture.html');
mkdirSync(profile);
writeFileSync(config, '{}');
writeFileSync(fixture, '<!doctype html><meta charset="utf-8"><meta name="color-scheme" content="light dark"><title>Disposable find reference</title><pre id="sample"></pre>');
const environment = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith('AGENT_BROWSER_')));
const options = ['--config', config, '--namespace', basename(directory), '--session', basename(directory), '--profile', profile, '--headed', 'false', '--no-webmcp', '--engine', 'chrome', '--json'];
// What the browser reports for one case: whether Window.find found the query, and what it selected.
interface FindRow { name: string; found: boolean }
interface FindOutput { userAgent: string; results: FindRow[] }
const call = (args: readonly string[]): { result?: FindOutput } => {
  const output = execFileSync('agent-browser', [...options, ...args], { env: environment, encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 });
  const result = JSON.parse(output);
  if (!result.success) throw new Error('Browser reference command failed: ' + JSON.stringify(result));
  return result.data;
};
const cases = [
  { name: 'positive-literal', text: 'a needle here', query: 'needle' },
  { name: 'negative-literal', text: 'ordinary text', query: 'needle' },
  { name: 'ascii-case', text: 'Needle', query: 'NEEDLE' },
  { name: 'regex-metacharacters', text: 'a [bracket] b', query: '[' },
  { name: 'significant-spaces', text: 'a cat b', query: ' cat ' },
  { name: 'cjk', text: 'a 猫 b', query: '猫' },
  { name: 'canonical-accent', text: 'cafe\u0301', query: 'café' },
  { name: 'plain-accent', text: 'café', query: 'cafe' },
  { name: 'case-expansion', text: 'Straße', query: 'STRASSE' },
  { name: 'compatibility-ligature', text: 'o\ufb03ce', query: 'office' },
  { name: 'sigma', text: 'ς', query: 'Σ' },
  { name: 'dotted-i', text: 'İ', query: 'i' },
  { name: 'newline-as-space', text: 'one\ntwo', query: 'one two' },
  { name: 'literal-newline', text: 'one\ntwo', query: 'one\ntwo' },
  { name: 'tab-as-space', text: 'one\ttwo', query: 'one two' },
  { name: 'nbsp-as-space', text: 'one\u00a0two', query: 'one two' },
  { name: 'empty-pattern', text: 'a needle here', query: '' },
  { name: 'kana-script', text: 'か', query: 'カ' },
  { name: 'kana-width', text: 'ｶ', query: 'カ' },
  { name: 'kana-small', text: 'ゃ', query: 'や' },
  { name: 'kana-voiced', text: 'が', query: 'か' },
  { name: 'kana-composed', text: 'か\u3099', query: 'が' },
  { name: 'kana-halfwidth-voiced', text: 'ｶﾞ', query: 'ガ' },
  { name: 'single-quote', text: 'can’t', query: "can't" },
  { name: 'double-quote', text: '“text”', query: '"text"' },
  { name: 'soft-hyphen', text: 'co\u00adoperate', query: 'cooperate' },
  { name: 'soft-hyphen-only', text: 'cooperate', query: '\u00ad' },
  { name: 'half-expansion', text: 'ß', query: 's' },
  { name: 'combining-mark-only', text: 'cafe\u0301', query: '\u0301' },
];
try {
  call(['open', pathToFileURL(fixture).href]);
  const script = `(() => {
    if (typeof window.find !== 'function') throw new Error('This browser does not expose Window.find');
    const sample = document.querySelector('#sample');
    const results = [];
    for (const item of ${JSON.stringify(cases)}) {
      sample.textContent = item.text;
      const selection = window.getSelection();
      selection.removeAllRanges();
      const range = document.createRange();
      range.selectNodeContents(sample);
      range.collapse(true);
      selection.addRange(range);
      const found = window.find(item.query, false, false, false, false, false, false);
      results.push({ ...item, found, selected: selection.toString(), startUtf16: selection.anchorOffset, endUtf16: selection.focusOffset });
    }
    return { userAgent: navigator.userAgent, results };
  })()`;
  const output = call(['eval', script]);
  writeFileSync(join(directory, 'results.json'), JSON.stringify(output, null, 2));
  const result = (output.result ?? output) as FindOutput;
  const rowNamed = (name: string): FindRow => {
    const row = result.results.find(item => item.name === name);
    if (row === undefined) throw new Error('The browser reported no result for ' + name);
    return row;
  };
  assert.equal(rowNamed('positive-literal').found, true);
  assert.equal(rowNamed('negative-literal').found, false);
  console.log(JSON.stringify({ directory, result }, null, 2));
} finally {
  call(['close']);
}
