// Opens the two outside links this recheck added to the troubleshooting document and confirms each page shows the
// quoted text, then closes the browser. Prints one JSON line per link.
//
// Run: node verify-new-link.ts > keep/verify-new-link.jsonl

import { spawnSync, } from 'node:child_process';

const LINKS = [
  {
    url: 'https://github.com/dahlia/optique/blob/d26f1a244d4434f9dbfeb0317f6db32533cb8c32/packages/core/src/primitives.ts#L1251',
    expect: ['buffer: context.buffer.slice(2),',],
  },
  {
    url: 'https://github.com/denoland/deno/issues/37030#:~:text=will%20continue%20maintain%20Deno%20until%20October%202027',
    expect: ['will continue maintain Deno until October 2027', 'No new feature work',],
  },
];

function browser(args: string[],): string {
  const child = spawnSync('agent-browser', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 120000, },);

  return `${child.stdout}${child.stderr}`;
}

for (const { url, expect, } of LINKS) {
  browser(['open', url,],);
  browser(['wait', '4000',],);

  const flat = browser(['eval', 'document.body.innerText',],).replace(/\\n/gu, ' ',).replace(/\\"/gu, '"',).replace(/\s+/gu, ' ',);

  console.log(JSON.stringify({ url, found: expect.map((needle,) => flat.includes(needle,)), length: flat.length, },),);
}

console.log(browser(['close',],).trim(),);
