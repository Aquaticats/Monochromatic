// Opens the links added to the Optique documents after the first check and confirms each page shows the quoted
// text, then closes the browser. Prints one JSON line per link.
//
// Run: node verify-links-2.ts

import { spawnSync, } from 'node:child_process';

const COMMIT = '1d6e459442e92f83006a7c34e4f11f5018c9ab39';
const LINKS: { url: string; expect: string[]; }[] = [
  {
    url: `https://github.com/dahlia/optique/blob/${COMMIT}/packages/core/src/modifiers.ts#L2743-L2746`,
    expect: ['state: annotateFreshArray(context.state, [', '...(added ? context.state : context.state.slice(0, -1)),',],
  },
  {
    url: `https://github.com/dahlia/optique/blob/${COMMIT}/packages/core/src/primitives.ts#L2725`,
    expect: ['buffer: context.buffer.slice(i + 1),',],
  },
  {
    url: 'https://github.com/dahlia/optique/issues/1120#:~:text=was%20reported%20in%20Monochromatic%27s',
    expect: ['was reported in Monochromatic\'s', 'then reproduced locally',],
  },
];

function browser(args: string[],): string {
  const child = spawnSync('agent-browser', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 120000, },);

  return `${child.stdout}${child.stderr}`;
}

for (const { url, expect, } of LINKS) {
  browser(['open', url,],);
  browser(['wait', '3000',],);

  const text = browser(['eval', 'document.body.innerText',],);
  // The code view's highlighted lines: read which line numbers carry the highlight class, where the page has them.
  const highlighted = browser(['eval', 'JSON.stringify([...document.querySelectorAll(".highlighted-line, [class*=highlighted]")].length)',],).trim();
  const flat = text.replace(/\\n/g, ' ',).replace(/\s+/g, ' ',);

  console.log(JSON.stringify({ url, found: expect.map((needle,) => flat.includes(needle.replace(/\s+/g, ' ',),)), highlighted, length: text.length, },),);
}

console.log(browser(['close',],).trim(),);
