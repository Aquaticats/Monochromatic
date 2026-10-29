import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, 'node_modules', '.monochromatic', 'music-player-design');
makeScratchBase(scratchBase, { recursive: true });


//region Test the actual inline classifier in a disposable Node consumer, never mutate committed HTML
const html = readFileSync(
  join(
    process.cwd(),
    'questions',
    'keyboard-map.prototype.html'
  ),
  'utf8'
);
const program = (/<script>([\s\S]*?)<\/script>/.exec(html))?.[1];
if (!program) throw new Error('Expected one keyboard-map program.');
const boundary = program.indexOf('//region Authored transitions');
if (boundary === (-1)) throw new Error('Keyboard-map classifier boundary differs.');
const source = program.slice(
  0,
  boundary
);
const mutations = {
  'without-player-owner': "  if (request.owner !== 'player') return 'unbound';",
  'without-composition-owner': "  if (request.composing) return 'owner';",
  'without-popup-owner': "  if (request.inPopup) return request.key === 'Escape' ? 'close-popup' : 'owner';",
};
let candidate = source;
const mutation = process.argv[2];
if (mutation !== undefined) {
  const needle = mutations[mutation];
  if ((!needle) || (source.split(needle)
    .length
    !== 2)) throw new Error('Unknown mutation or changed keymap guard target.');
  candidate = source.replace(
    needle,
    ''
  );
}
const cases = `
const base = { platform: 'windows-linux', key: ' ', ctrlKey: false, metaKey: false,
  altKey: false, shiftKey: false, repeat: false, owner: 'player', inPopup: false,
  inSearch: false, composing: false };
if (shortcutAction({ ...base, owner: 'surface' }) !== 'unbound') throw new Error('Non-player key escaped owner boundary.');
if (shortcutAction({ ...base, key: 'f', ctrlKey: true, composing: true }) !== 'owner') throw new Error('Composition key escaped its owner.');
if (shortcutAction({ ...base, inPopup: true }) !== 'owner') throw new Error('Popup key escaped its owner.');
const checks = [
  ['toggle', {}], ['owner', { owner: 'editor' }], ['owner', { owner: 'control' }],
  ['ignore-repeat', { repeat: true }], ['search', { key: 'f', ctrlKey: true }],
  ['picker', { key: 'o', ctrlKey: true }], ['settings', { key: 's', ctrlKey: true, altKey: true }],
  ['previous', { key: 'ArrowLeft', ctrlKey: true }], ['next', { key: 'ArrowRight', ctrlKey: true }],
  ['owner', { key: 'ArrowLeft', ctrlKey: true, owner: 'editor' }], ['unbound', { key: 'ArrowDown' }],
  ['unbound', { platform: 'mac', key: 'ArrowRight', ctrlKey: true }],
  ['search', { platform: 'mac', key: 'f', metaKey: true }],
  ['picker', { platform: 'mac', key: 'o', metaKey: true }],
  ['settings', { platform: 'mac', key: ',', metaKey: true }],
  ['reveal', { platform: 'mac', key: 'l', metaKey: true }],
  ['previous', { platform: 'mac', key: '[', metaKey: true, shiftKey: true }],
  ['next', { platform: 'mac', key: ']', metaKey: true, shiftKey: true }],
  ['close-popup', { key: 'Escape', inPopup: true }],
  ['owner', { key: 'f', ctrlKey: true, inPopup: true }],
  ['back', { key: 'Escape', owner: 'editor', inSearch: true }],
  ['reveal', { key: 'g', ctrlKey: true }], ['mode', { key: 'm', ctrlKey: true }],
  ['unbound', { platform: 'mac', key: 'm', metaKey: true }],
  ['unbound', { key: 'f', ctrlKey: true, metaKey: true }],
  ['unbound', { key: 'f', ctrlKey: true, shiftKey: true }],
];
for (const [expected, changes] of checks) {
  const actual = shortcutAction({ ...base, ...changes });
  if (actual !== expected) throw new Error('Shortcut classification differs: ' + expected + ' / ' + actual);
}
let rejected = false;
try { shortcutAction({ ...base, platform: 'unknown' }); }
catch (error) { if (!String(error).includes('Unknown keyboard-map platform.')) throw error; rejected = true; }
if (!rejected) throw new Error('Unknown keyboard platform was accepted.');
console.log('Keymap classifier and ownership tests passed.');
`;
const fixture = mkdtempSync(join(
  scratchBase,
  'keymap-scope-'
));
try {
  const path = join(
    fixture,
    'consumer.mjs'
  );
  writeFileSync(
    path,
    candidate + cases
  );
  const result = spawnSync(
    process.execPath,
    [path],
    {
      encoding: 'utf8',
      maxBuffer: 1_024 * 1_024
    }
  );
  if (result.stdout) console.log(result.stdout
    .trim());
  if (result.status !== 0) throw new Error(`Keymap consumer failed: ${  result.stderr}`);
} finally {
  rmSync(
    fixture,
    {
      recursive: true,
      force: true
    }
  );
}
//endregion
