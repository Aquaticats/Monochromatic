// Opens each external link the input-trace troubleshooting doc cites in agent-browser and checks that the page shows
// the quoted text, then closes the browser. Prints one JSON line per link.
//
// Run: node verify-links.ts

import { spawnSync, } from 'node:child_process';

const COMMIT = '1d6e459442e92f83006a7c34e4f11f5018c9ab39';
const MAIN = '84f9571069584d8e36a3a4cfbaebf8a9a61fc645';
const LINKS: { url: string; expect: string[]; }[] = [
  {
    url: `https://github.com/dahlia/optique/blob/${COMMIT}/packages/core/src/input-trace.ts#L140-L144`,
    expect: ['const copy = new Map(this.#entries);', 'return new InputTraceImpl(copy);',],
  },
  {
    url: `https://github.com/dahlia/optique/blob/${COMMIT}/packages/core/src/primitives.ts#L162-L176`,
    expect: ['function recordTrace<TState>(', 'const trace = context.exec.trace.set(context.exec.path, entry);',],
  },
  {
    url: 'https://github.com/dahlia/optique/issues/1120#:~:text=The%20separate%20whole%2DMap%20copy,outside%20this%20issue%27s%20scope',
    expect: ['is present in both 1.3.6 and', 'outside this issue\'s scope',],
  },
  {
    url: 'https://github.com/dahlia/optique/pull/1121#:~:text=without%20timing%20thresholds',
    expect: ['without timing thresholds', 'the separator case remains slower than 1.3.6',],
  },
  {
    url: `https://github.com/dahlia/optique/blob/${MAIN}/AI_POLICY.md?plain=1#L14-L18`,
    expect: ['Pull requests created in any way by AI can only be for accepted issues.', 'open', 'a discussion or attach it to an existing discussion.',],
  },
  {
    url: `https://github.com/dahlia/optique/blob/${MAIN}/AI_POLICY.md?plain=1#L25-L27`,
    expect: ['human-in-the-loop', 'been reviewed and edited by a human before submission.',],
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
  const location = browser(['eval', 'location.href',],).trim();
  // GitHub's code view renders a non-breaking or split layout; compare with whitespace collapsed.
  const flat = text.replace(/\\n/g, ' ',).replace(/\s+/g, ' ',);

  console.log(JSON.stringify({
    url,
    landed: location.slice(0, 160,),
    found: expect.map((needle,) => flat.includes(needle.replace(/\s+/g, ' ',),)),
    length: text.length,
  },),);
}

console.log(browser(['close',],).trim(),);
