/**
 Two file trees read by the offline guards: one carrying every defect a guard exists for, and one carrying none.
 A guard that reports nothing on the first, or something on the second, fails the test rather than passing quietly.
 */

import { createHash } from 'node:crypto';
import {
  mkdirSync,
  mkdtempSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join } from 'node:path';

import { designRoot } from './design-page-dependencies.ts';

/** A file tree the guards read the way they read the committed one. */
type Fixture = {
  root: string;
  files: Record<string, string>;
};

/** Digest the vendored-build record states for a fixture file. */
function digestOf(content: string,): string {
  return `sha384-${createHash('sha384',).update(content,).digest('base64',)}`;
}

/** Digest the rejected fixture states, so a swapped build is caught rather than matched. */
const WRONG_DIGEST = `sha384-${'A'.repeat(86)}`;

/** A runtime that asks this tree for its build, as the committed copies do. */
const ACCEPTED_RUNTIME = `(function () {
  var VENDOR_BASE = "./vendor/";
  var REACT_BUILD = "react-18.3.1.production.min.js";
  function vendorScriptFor(build) { return new URL(build, document.baseURI).href; }
  document.head.appendChild(Object.assign(document.createElement("script"), { src: vendorScriptFor(REACT_BUILD) }));
})();
`;

/** Scratch directory the fixtures live in, created under the package's gitignored tree. */
export function scratchDirectory(): string {
  const base = join(
    designRoot,
    'node_modules',
    '.monochromatic',
    'music-player-design',
  );
  mkdirSync(
    base,
    { recursive: true, },
  );
  return mkdtempSync(join(
    base,
    'offline-pages-',
  ),);
}

/** Writes a fixture tree and answers its root. */
function writeFixture(base: string, fixture: Fixture,): string {
  const root = join(
    base,
    fixture.root,
  );
  for (const [path, content] of Object.entries(fixture.files,)) {
    const file = join(
      root,
      path,
    );
    mkdirSync(
      dirname(file,),
      { recursive: true, },
    );
    writeFileSync(
      file,
      content,
      'utf8',
    );
  }
  return root;
}

/** Control trees, written under `base`. */
export function writeControls(base: string,): {
  accepted: string;
  rejected: string;
} {
  const build = 'console.log("react");\n';
  const record = {
    builds: [
      {
        file: 'react-18.3.1.production.min.js',
        package: 'react',
        version: '18.3.1',
        license: 'MIT',
        integrity: digestOf(build,),
      },
    ],
    runtimes: [
      'support.js',
      'candidates/support.js',
    ],
  };
  const accepted = writeFixture(
    base,
    {
      root: 'accepted',
      files: {
        'vendor/builds.json': JSON.stringify(
          record,
          null,
          2,
        ),
        'vendor/react-18.3.1.production.min.js': build,
        'support.js': ACCEPTED_RUNTIME,
        'candidates/support.js': ACCEPTED_RUNTIME,
        'tokens.css': "@font-face{font-family:'Roboto Flex';src:url(\"./roboto-flex.woff2\")}\n",
        'roboto-flex.woff2': 'font',
        'clean.dc.html': `<!DOCTYPE html>
<html><head><link rel="stylesheet" href="./tokens.css"></head>
<body><x-dc><div style="font-family:'Roboto Flex',system-ui,sans-serif">clean</div></x-dc></body></html>
`,
      },
    },
  );
  const rejected = writeFixture(
    base,
    {
      root: 'rejected',
      files: {
        'vendor/builds.json': JSON.stringify(
          {
            ...record,
            builds: [
              {
                ...record.builds[0],
                integrity: WRONG_DIGEST,
              },
            ],
          },
          null,
          2,
        ),
        'vendor/react-18.3.1.production.min.js': build,
        'vendor/stray-1.0.0.js': 'not recorded\n',
        'support.js': `${ACCEPTED_RUNTIME}// https://unpkg.com/react@18.3.1/umd/react.production.min.js\n`,
        'candidates/support.js': `${ACCEPTED_RUNTIME}// the copies drift\n`,
        'tokens.css': "@font-face{font-family:'Roboto Flex';src:url(\"../fonts/absent.woff2\")}\n",
        'absent-page.dc.html': `<!DOCTYPE html>
<html><head><link rel="stylesheet" href="./absent.css"></head>
<body><x-dc><div style="font-family:sans-serif">absent</div></x-dc></body></html>
`,
        'remote.dc.html': `<!DOCTYPE html>
<html><head><link href="https://fonts.googleapis.com/css2?family=Roboto&display=swap" rel="stylesheet"></head>
<body><x-dc><div style="font-family:sans-serif">remote</div></x-dc></body></html>
`,
        'unshipped.dc.html': `<!DOCTYPE html>
<html><head><link rel="stylesheet" href="./tokens.css"></head>
<body><x-dc><div style="font-family:'Noto Sans',sans-serif">unshipped</div></x-dc></body></html>
`,
        'undeclared.dc.html': `<!DOCTYPE html>
<html><body><x-dc><div style="font-family:'Google Sans Text',sans-serif">undeclared</div></x-dc></body></html>
`,
        'shorthand.dc.html': `<!DOCTYPE html>
<html><head><link rel="stylesheet" href="./tokens.css"></head>
<body><x-dc><div style="font:500 14px/1.4 'Roboto Flex',sans-serif">shorthand</div></x-dc></body></html>
`,
      },
    },
  );
  return {
    accepted,
    rejected,
  };
}
