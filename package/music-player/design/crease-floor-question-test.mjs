import { spawnSync } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, 'node_modules', '.monochromatic', 'music-player-design');
mkdirSync(scratchBase, { recursive: true });

//region The E2 floor review's count is the cohort it actually embeds, as the build and the validate message both say
const root = process.cwd();
const fixture = mkdtempSync(join(scratchBase, 'crease-floor-review-'));
const questions = join(fixture, 'question');
const render = join(questions, 'render');
mkdirSync(render, { recursive: true });
copyFileSync(join(root, 'crease-floor-question.mjs'), join(fixture, 'crease-floor-question.mjs'));
copyFileSync(join(root, 'question', 'crease-floor.template.html'), join(questions, 'crease-floor.template.html'));
const builder = join(fixture, 'crease-floor-question.mjs');
const output = join(questions, 'crease-floor-review.html');
// A message may name its cohort as digits or as the word for it; both are checked against the built page.
const COUNT_WORDS = ['zero', 'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight', 'nine', 'ten', 'eleven', 'twelve'];
const names = value => [String(value), COUNT_WORDS[value]]
  .filter(name => name !== undefined);
const namesCount = (message, count) => names(count).some(name => message.includes(name));
// Every capture the committed template can name is copied, so the fixture builds the real cohort.
const floors = ['7p5', '14', '20'];
const states = ['player', 'empty', 'results'];
for (const floor of floors) {
  for (const state of states) {
    const name = `search-e2-complete-${floor}-${state}-inner-light-s200.png`;
    copyFileSync(join(root, 'question', 'render', name), join(render, name));
  }
}
function invoke(command) {
  return spawnSync(process.execPath, [builder, command], {
    cwd: fixture,
    encoding: 'utf8',
    maxBuffer: 2 * 1_024
      * 1_024
  });
}
// The cohort size comes from the built page, never from a number written into this test.
const embeddedCount = () => new Set(
  [...readFileSync(output, 'utf8').matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/gu)]
    .map(match => match[1])
).size;
try {
  const build = invoke('build');
  if (build.status !== 0) throw new Error(`E2 fixture build failed: ${build.stderr}`);
  const count = embeddedCount();
  if (count !== floors.length * states.length) {
    throw new Error(`E2 review embeds ${count} distinct captures; the fixture copies ${floors.length * states.length}.`);
  }
  const built = `${count} native player/Search E2 states`;
  if (!(namesCount(build.stdout, count) && build.stdout.includes('native player/Search E2 states'))) {
    throw new Error(`Build message does not name the ${count} captures it embeds: ${build.stdout.trim()}`);
  }
  const untouched = invoke('validate');
  if (untouched.status !== 0) throw new Error(`Committed E2 review did not validate: ${untouched.stderr}`);
  writeFileSync(output, `${readFileSync(output, 'utf8')} `);
  const changed = invoke('validate');
  if (changed.status === 0) throw new Error('Changed E2 review was not rejected.');
  // The thrown sentence only: node's frame also quotes the source line and its directory.
  const diagnostic = /^Error: (.*)$/mu.exec(changed.stderr)?.[1] ?? changed.stderr.trim();
  if (!(namesCount(diagnostic, count) && diagnostic.includes('sanitized native captures'))) {
    throw new Error(`Validate message does not name the ${count} captures it compares: ${diagnostic}`);
  }
  console.log(`E2 floor review: ${count} embedded captures are named by the build message and by the validate message.`);
} finally {
  rmSync(fixture, { recursive: true, force: true });
}
//endregion
