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
import {
  hasPreselectedRadio,
  occurrencesOf,
  pngPayloads,
  usesClipboardApi,
} from './command-question.markup-scan.mjs';

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
const PNG_PREFIX = 'data:image/png;base64,';

/** Reports one value pair when a reading and its expectation disagree. */
function fail(name, found, expected) {
  throw new Error(`The scan reads ${JSON.stringify(found)} for ${name}; the withdrawn pattern read ${JSON.stringify(expected)}.`);
}

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
  if (found !== expected) fail(name, found, expected);
}
//endregion

//region Every embedded raster payload is read, in page order, and stops at its alphabet's edge
const payloadCases = [
  ['a page without a raster', '<figure><img alt="none"></figure>', []],
  ['one payload', `<img src="${PNG_PREFIX}AAAA">`, ['AAAA']],
  ['two payloads', `<img src="${PNG_PREFIX}AA=="><img src="${PNG_PREFIX}BB+/">`, ['AA==', 'BB+/']],
  ['a payload that ends the page', `${PNG_PREFIX}CCCC`, ['CCCC']],
  ['a prefix with no payload after it', `${PNG_PREFIX}</p><img src="${PNG_PREFIX}DDDD">`, ['DDDD']],
  ['a payload stopped by a character base64 never carries', `${PNG_PREFIX}EE-f`, ['EE']],
  ['adjacent prefixes in separate markup', `${PNG_PREFIX}FF>${PNG_PREFIX}GG`, ['FF', 'GG']],
  // A prefix's own letters are base64 characters, so one issued inside a run continues that run.
  ['a prefix that continues a base64 run', `${PNG_PREFIX}FF${PNG_PREFIX}GG`, ['FFdata']],
  ['nothing at all', '', []],
];
for (const [name, markup, expected] of payloadCases) {
  const found = pngPayloads(markup);
  if (JSON.stringify(found) !== JSON.stringify(expected)) fail(name, found, expected);
}
//endregion

//region A radio that arrives checked is refused, and the scan reads one tag as the pattern did
// The two attributes have to share one tag and keep their order, and `checked` is a substring there, so a name that
// merely contains it counts, exactly as it did under the pattern.
const preselectCases = [
  ['a checked radio', '<input type="radio" name="inapp" value="I1" checked>', true],
  ['a radio that is not checked', '<input type="radio" name="inapp" value="I1">', false],
  ['a checked checkbox', '<input type="checkbox" name="aside" checked>', false],
  ['checked before the radio type', '<input checked type="radio" name="inapp">', false],
  ['a checked control in the next tag', '<input type="radio" name="inapp"><input checked>', false],
  ['a checked radio after another input', '<input type="text"><input type="radio" checked>', true],
  ['a tag whose attribute value closes it early', '<input data-probe=">" type="radio" checked>', false],
  ['a checked name that only contains the word', '<input type="radio" data-checked="no">', true],
  ['an uppercase input tag', '<INPUT type="radio" checked>', false],
  ['a radio without its closing bracket', '<input type="radio" checked', true],
  ['markup without an input', '<fieldset><legend>Placement</legend></fieldset>', false],
  ['nothing at all', '', false],
];
for (const [name, markup, expected] of preselectCases) {
  const found = hasPreselectedRadio(markup);
  if (found !== expected) fail(name, found, expected);
}
//endregion

//region A literal's occurrences are counted as a global match counted them
const occurrenceCases = [
  ['a literal that repeats', '<input type="radio"><input type="radio"><input type="radio">', 'type="radio"', 3],
  ['a literal that does not appear', '<fieldset></fieldset>', 'type="radio"', 0],
  ['overlapping occurrences', 'aaaa', 'aa', 2],
  ['a literal at both edges', 'xaxax', 'xa', 2],
  ['a literal longer than the text', 'ab', 'abc', 0],
  ['nothing at all', '', 'type="radio"', 0],
];
for (const [name, text, literal, expected] of occurrenceCases) {
  const found = occurrencesOf(text, literal);
  if (found !== expected) fail(name, found, expected);
}
let emptyLiteralRefused = false;
try {
  occurrencesOf('anything', '');
} catch {
  emptyLiteralRefused = true;
}
if (!emptyLiteralRefused) throw new Error('occurrencesOf counted an empty literal instead of refusing it.');
//endregion

//region The script carries none of the patterns the audit filed against it
// Each entry is the site string its issue records. The control runs the same search over a page that carries the
// first site, so an empty result means the pattern is gone rather than that the search reads nothing.
const FILED_SITES = [
  [String.raw`/navigator\.clipboard|clipboardData|execCommand\(\s*['"]copy/`, '#787'],
  [String.raw`/data:image\/png;base64,([A-Za-z0-9+/=]+)/g`, '#788 and #789'],
  [String.raw`/<input[^>]*type="radio"[^>]*checked/`, '#790'],
  [String.raw`/type="radio"/g`, '#791'],
  [String.raw`/<fieldset>/g`, '#792'],
];
const siteSearchControl = `if (${FILED_SITES[0][0]}.test(html)) throw`;
if (!siteSearchControl.includes(FILED_SITES[0][0])) throw new Error('The site search cannot see the site string it looks for.');
const script = readFileSync(
  scriptPath,
  'utf8'
);
for (const [site, issues] of FILED_SITES) {
  if (script.includes(site)) throw new Error(`command-question.mjs still carries the pattern filed as ${issues}: ${site}`);
}
//endregion

//region The validator still refuses every page the swapped guards refused
// A disposable copy of the questionnaire builds its page from the committed template, captures and reference, so the
// validator runs against real markup and every mutation is one change to that page.
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
  // Each mutation adds markup the guard refuses or rewrites one reading of the page, and names the sentence that has
  // to come back. The built page is restored before the next one.
  const mutations = [
    {
      name: 'a clipboard object',
      mutate: (page) => page.replace(
        '</body>',
        '<p data-probe="navigator.clipboard"></p></body>'
      ),
      diagnostic: 'depends on an external resource or clipboard API',
    },
    {
      name: 'a copy command',
      mutate: (page) => page.replace(
        '</body>',
        '<p data-probe="execCommand(\n  \'copy\')"></p></body>'
      ),
      diagnostic: 'depends on an external resource or clipboard API',
    },
    {
      name: 'a raster prefix that lost its comma',
      mutate: (page) => page.replace(
        PNG_PREFIX,
        'data:image/png;base64'
      ),
      diagnostic: 'data URLs',
    },
    {
      name: 'a preselected radio',
      mutate: (page) => page.replace(
        '</body>',
        '<input type="radio" name="probe" value="P1" checked></body>'
      ),
      diagnostic: 'must not preselect a choice',
    },
    {
      name: 'one radio choice too many',
      mutate: (page) => page.replace(
        '</body>',
        '<input type="radio" name="probe" value="P1"></body>'
      ),
      diagnostic: 'seven radio choices',
    },
    {
      name: 'one fieldset too many',
      mutate: (page) => page.replace(
        '</body>',
        '<fieldset></fieldset></body>'
      ),
      diagnostic: 'seven radio choices',
    },
  ];
  for (const { name, mutate, diagnostic } of mutations) {
    const mutated = mutate(pristine);
    // A mutation that rewrites nothing would pass the check below for the wrong reason.
    if (mutated === pristine) throw new Error(`The fixture page cannot carry ${name}.`);
    writeFileSync(
      output,
      mutated
    );
    const refused = invoke('validate');
    if (refused.status === 0) throw new Error(`The validator accepted a page carrying ${name}.`);
    if (!refused.stderr.includes(diagnostic)) {
      throw new Error(`The validator refused ${name} with an unexpected diagnostic: ${refused.stderr}`);
    }
    writeFileSync(
      output,
      pristine
    );
  }
  console.log(`Command questionnaire: ${clipboardCases.length} clipboard, ${payloadCases.length} payload, ${preselectCases.length} preselect and ${occurrenceCases.length} count readings hold, ${FILED_SITES.length} filed patterns are gone, and ${mutations.length} changed pages are refused.`);
} finally {
  rmSync(
    fixture,
    { recursive: true, force: true }
  );
}
//endregion
