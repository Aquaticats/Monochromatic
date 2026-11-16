// Opens the public reproduction repository's page and the Deno tracker link the README now carries, and prints what
// the rendered pages hold, so the README's new text is checked as rendered and its link is checked to land on the
// quoted words.
//
// Run: node check-live-readme.ts

import { spawnSync, } from 'node:child_process';

function browser(args: readonly string[],): string {
  const child = spawnSync('agent-browser', args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, },);

  return `${child.stdout}${child.stderr}`.trim();
}

const README_PAGE = 'https://github.com/Aquaticat/optique-parse-cost-reproduction/blob/main/README.md';
const DENO_LINK =
  'https://github.com/denoland/deno/issues/37030#:~:text=will%20continue%20maintain%20Deno%20until%20October%202027';

browser(['open', README_PAGE,],);
browser(['wait', '3000',],);

const readme = browser([
  'eval',
  `(() => {
    const article = document.querySelector('article') ?? document.body;
    const text = article.innerText;
    const link = [...article.querySelectorAll('a')].find((a) => a.textContent.includes('October 2027'));
    return JSON.stringify({
      holdsFigureKinds: text.includes('Every figure in this file is a count'),
      holdsDenoNote: text.includes('Deno is here because a source checkout runs on it without a build'),
      linkText: link ? link.textContent : null,
      linkTarget: link ? link.href : null,
      literalBrackets: /\\[will maintain/.test(text),
      headings: [...article.querySelectorAll('h1,h2')].map((h) => h.textContent.trim()),
    });
  })()`,
],);

console.log(JSON.stringify({ page: 'readme', result: readme, },),);

browser(['open', DENO_LINK,],);
browser(['wait', '3000',],);

const deno = browser([
  'eval',
  `(() => JSON.stringify({
    title: document.title,
    holdsQuote: document.body.innerText.includes('will continue maintain Deno until October 2027'),
  }))()`,
],);

console.log(JSON.stringify({ page: 'deno issue', result: deno, },),);
console.log(JSON.stringify({ closed: browser(['close',],), },),);
