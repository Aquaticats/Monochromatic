// Replaces the timing part of the section "Prototypes for the two further copies" in the input-trace troubleshooting
// document with timing-section.md, after applying the corrections listed here to that draft, and adds the link
// definition the new text uses. The part runs from the heading "#### Timing" up to the heading
// "#### Where the agent's report and the recheck differ".
//
// Run: node splice-timing-section.ts

import { readFileSync, writeFileSync, } from 'node:fs';

const DOCUMENT = '/var/mnt/encrypted/repository/Monochromatic/doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md';
const START = '#### Timing\n';
const END = '#### Where the agent\'s report and the recheck differ\n';

function replaceOnce(text: string, find: string, replace: string,): string {
  if (text.split(find,).length !== 2) throw new Error(`not found exactly once: ${find.slice(0, 70,)}`,);

  return text.replace(find, replace,);
}

let section = readFileSync(`${import.meta.dirname}/timing-section.md`, 'utf8',);

// Headings may go four levels deep, so the parts of the timing are headings of the same level as its method.
for (
  const [from, to,] of [
    ['#### Timing\n', '#### Timing: how it was measured\n',],
    ['##### A run of positional arguments\n', '#### Timing: a run of positional arguments\n',],
    ['##### Options and arguments alternating\n', '#### Timing: options and arguments alternating\n',],
    ['##### A parser shaped like `git commit`\n', '#### Timing: a parser shaped like `git commit`\n',],
    ['##### The same copy written with `concat`\n', '#### Timing: the same copy written with `concat`\n',],
    ['##### For scale only\n', '#### Timing: absolute times, for scale only\n',],
  ] as const
) section = replaceOnce(section, from, to,);

section = replaceOnce(
  section,
  'At 2,000 and 4,000 tokens it reads between 0.78 and 0.94 on the three runtimes,\nwith two of thirty pairs slower,\nwhich is near the spread of identical code.\nOn Bun it gains about half of what it gains on Node.',
  'At 2,000 and 4,000 tokens it reads between 0.72 and 0.94 on the three runtimes,\nwith three of thirty pairs slower,\nwhich is near the spread of identical code.\nOn Bun the gain is smaller than on Node:\n0.43 against 0.16 at 32,000 tokens.',
);
section = replaceOnce(
  section,
  'At 12,000 tokens and below,\nwhere the buffer fix does not take effect,\nthe same ratio is inside the spread.',
  'At 12,000 tokens and below,\nwhere the buffer fix does not take effect,\nthere is no such loss.',
);
section = replaceOnce(
  section,
  '   no arm differs from the base by more than the spread on any runtime,\n   with or without `--`,\n   except thresholds-at-zero on Deno,',
  '   no arm differs from the base by more than the spread on any runtime\n   (with and without `--` on Deno,\n   without on Node and Bun),\n   except thresholds-at-zero on Deno,',
);
section = replaceOnce(
  section,
  'Against the state fix it is level up to 8,000 tokens and then falls behind,',
  'Against the state fix it is level or ahead up to 4,000 tokens and then falls behind,',
);

const document = readFileSync(DOCUMENT, 'utf8',);
const start = document.indexOf(START,);
const end = document.indexOf(END,);

if (start === -1 || end === -1 || end < start || document.indexOf(START, start + 1,) !== -1) throw new Error('the timing part was not found once',);

const linkAnchor = '[option buffer]: ';
let result = `${document.slice(0, start,)}${section}${document.slice(end,)}`;

result = replaceOnce(
  result,
  linkAnchor,
  `[deno state]: https://github.com/denoland/deno/issues/37030#:~:text=will%20continue%20maintain%20Deno%20until%20October%202027\n${linkAnchor}`,
);

writeFileSync(DOCUMENT, result,);
console.log(JSON.stringify({ replacedCharacters: end - start, newCharacters: section.length, },),);
