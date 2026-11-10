import { spawnSync } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import { usesClipboardApi } from './command-question.markup-scan.mjs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(
  import.meta.dirname,
  'node_modules',
  '.monochromatic',
  'music-player-design'
);
mkdirSync(
  scratchBase,
  { recursive: true }
);

const root = process.cwd();
const scriptPath = join(
  root,
  'command-question.mjs'
);

//region A page that reaches for the clipboard is refused, and the scan reads the same literals the pattern did
// Every case names what the markup carries and what the withdrawn pattern answered for it, so the scan is held to
// the pattern's behaviour rather than to a tidier rule: an unanchored match, a run of any length, and any quote.
const clipboardCases = [
  ['a clipboard object', 'document.title = navigator.clipboard', true],
  ['the legacy clipboard object', 'window.clipboardData.getData', true],
  ['a quoted copy command', "<button onclick=\"execCommand('copy')\">", true],
  ['a copy command in double quotes', '<button onclick=\'execCommand("copy")\'>', true],
  ['a copy command after a space', "<button onclick=\"execCommand( 'copy')\">", true],
  ['a copy command after a newline and a tab', "<a onclick=\"execCommand(\n\t'copy')\">", true],
  ['a copy command inside a longer word', "<a onclick=\"myexecCommand('copy')\">", true],
  ['a copy command with a capital letter', "<a onclick=\"runExecCommand('copy')\">", false],
  ['a command that merely starts with copy', "<a onclick=\"execCommand('copyy')\">", true],
  ['an unquoted copy command', "<a onclick=\"execCommand(copy)\">", false],
  ['a paste command', "<a onclick=\"execCommand('paste')\">", false],
  ['a quoted space before copy', "<a onclick=\"execCommand(' copy')\">", false],
  ['a clipboard word without its object', 'navigator clipboardDataTitle', true],
  ['markup that mentions neither', '<figure class="capture"><button type="button">Preview</button></figure>', false],
  ['nothing at all', '', false],
];
for (const [name, markup, expected] of clipboardCases) {
  const found = usesClipboardApi(markup);
  if (found !== expected) throw new Error(`The clipboard scan reports ${found} for ${name}: ${JSON.stringify(markup)}`);
}

// The site string is the one issue #787 records. It has to stay out of the script, and the search has to be able to
// find it at all, so the control runs the same search over a page that carries it.
const CLIPBOARD_SITE = String.raw`/navigator\.clipboard|clipboardData|execCommand\(\s*['"]copy/`;
const siteSearchControl = `if (${CLIPBOARD_SITE}.test(html)) throw`;
if (!siteSearchControl.includes(CLIPBOARD_SITE)) throw new Error('The site search cannot see the site string it looks for.');
const script = readFileSync(
  scriptPath,
  'utf8'
);
if (script.includes(CLIPBOARD_SITE)) throw new Error(`command-question.mjs still carries the filed clipboard pattern: ${CLIPBOARD_SITE}`);
//endregion

//region The validator still refuses every page the swapped guard refused
// A disposable copy of the questionnaire builds its page from the committed template, captures and reference, so the
// validator runs against real markup and every mutation is one addition to that page.
const fixture = mkdtempSync(join(
  scratchBase,
  'command-question-'
));
const questions = join(
  fixture,
  'questions'
);
const archive = join(
  questions,
  'archive'
);
const render = join(
  questions,
  'render'
);
const evidence = join(
  questions,
  'evidence'
);
for (const folder of [
  archive,
  render,
  evidence
]) mkdirSync(
  folder,
  { recursive: true }
);
for (const file of [
  'command-question.mjs',
  'command-question.markup-scan.mjs'
]) copyFileSync(
  join(root, file),
  join(fixture, file)
);
copyFileSync(
  join(root, 'questions', 'archive', 'command-igr-rejected.template.html'),
  join(archive, 'command-igr-rejected.template.html')
);
copyFileSync(
  join(root, 'questions', 'evidence', 'command-md3-baseline-header-comparison.png'),
  join(evidence, 'command-md3-baseline-header-comparison.png')
);
// Every capture the committed template names is copied, so the fixture embeds the real cohort.
for (const name of readdirSync(join(root, 'questions', 'render'))) {
  if (name.startsWith('command-md3-') && name.endsWith('.png')) {
    copyFileSync(
      join(root, 'questions', 'render', name),
      join(render, name)
    );
  }
}
const output = join(
  archive,
  'command-igr-rejected.html'
);
function invoke(verb) {
  return spawnSync(
    process.execPath,
    [
      'command-question.mjs',
      verb
    ],
    {
      cwd: fixture,
      encoding: 'utf8',
      maxBuffer: 64 * 1_024
        * 1_024
    }
  );
}
try {
  const build = invoke('build');
  if (build.status !== 0) throw new Error(`The questionnaire fixture did not build: ${build.stderr}`);
  const pristine = readFileSync(
    output,
    'utf8'
  );
  const accepted = invoke('validate');
  if (accepted.status !== 0) throw new Error(`The built questionnaire did not validate: ${accepted.stderr}`);
  // Each mutation adds markup the guard refuses; the built page is restored before the next one.
  const mutations = [
    [
      'a clipboard object',
      '<p data-probe="navigator.clipboard"></p>'
    ],
    [
      'a copy command',
      '<p data-probe="execCommand(\n  \'copy\')"></p>'
    ],
  ];
  for (const [name, markup] of mutations) {
    const mutated = pristine.replace(
      '</body>',
      `${markup}</body>`
    );
    if (mutated === pristine) throw new Error('The fixture page lost its closing body tag.');
    writeFileSync(
      output,
      mutated
    );
    const refused = invoke('validate');
    if (refused.status === 0) throw new Error(`The validator accepted a page carrying ${name}.`);
    // One sentence covers the external-resource guard and the clipboard guard, so the message is the pair's.
    if (!refused.stderr.includes('depends on an external resource or clipboard API')) {
      throw new Error(`The validator refused ${name} with an unexpected diagnostic: ${refused.stderr}`);
    }
    writeFileSync(
      output,
      pristine
    );
  }
  console.log(`Command questionnaire: ${clipboardCases.length} clipboard readings hold, the filed pattern is gone, and ${mutations.length} added clipboard reach is refused.`);
} finally {
  rmSync(
    fixture,
    { recursive: true, force: true }
  );
}
//endregion
