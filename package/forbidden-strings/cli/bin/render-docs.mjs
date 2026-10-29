#!/usr/bin/env node
/** Render only scanner documentation through the repository's installed CommonMark-to-HTML-tree pipeline. */
import {
  mkdir,
  readFile,
  readdir,
  writeFile
} from 'node:fs/promises';
import {
  resolve,
  join
} from 'node:path';
import { pathToFileURL } from 'node:url';

/** Resolve the incumbent pnpm-installed renderer without introducing or selecting another dependency. */
async function installed(name) {
  const store = resolve(
    import.meta.dirname,
    '../../../../node_modules/.pnpm'
  );
  const entries = (await readdir(store)).filter(entry => entry.startsWith(`${name}@`));
  if (entries.length !== 1) throw new Error(`Expected one installed ${name} renderer, found ${entries.length}.`);
  return await import(pathToFileURL(join(
    store,
    entries[0],
    'node_modules',
    name,
    'index.js'
  ))
    .href);
}

/** Render the documentation and retain the HTML element tree plus readable rendered text. */
async function main() {
  const { unified } = await installed('unified');
  const { default: parse } = await installed('remark-parse');
  const { default: html } = await installed('remark-rehype');
  const processor = unified()
    .use(parse)
    .use(html);
  const repository = resolve(
    import.meta.dirname,
    '../../../..'
  );
  const evidence = resolve(
    import.meta.dirname,
    '../target/verification/docs'
  );
  await mkdir(
    evidence,
    { recursive: true }
  );
  for (const file of [
    'package/forbidden-strings/cli/README.md',
    'package/forbidden-strings/cli.fuzz/README.md',
    'doc/handover/scanner-native-verification.md'
  ]) {
    const source = await readFile(
      join(
        repository,
        file
      ),
      'utf8'
    );
    const rendered = processor.runSync(processor.parse(source));
    const pending = [rendered];
    const text = [];
    let headings = 0;
    while (pending.length > 0) {
      const node = pending.pop();
      if (node.type === 'text') {
        if (node.value
          .includes('**')) throw new Error(`Unrendered emphasis delimiter in ${file}: ${node.value}`);
        text.push(node.value);
      }
      if ((node.type === 'element') && /^h[1-4]$/u.test(node.tagName)) headings += 1;
      if (node.children) pending.push(...[...node.children].reverse());
    }
    if (headings === 0) throw new Error(`No rendered headings in ${file}.`);
    const filename = file.replaceAll(
      '/',
      '_'
    );
    await writeFile(
      join(
        evidence,
        `${filename}.hast.json`
      ),
      `${JSON.stringify(
        rendered,
        null,
        2
      )  }\n`
    );
    await writeFile(
      join(
        evidence,
        `${filename}.txt`
      ),
      `${text.join('')  }\n`
    );
    console.log(`${file}: rendered ${headings} headings; no literal bold delimiters.`);
  }
}

await main();
