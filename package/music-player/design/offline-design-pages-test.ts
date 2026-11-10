/**
 Proves the music player's design pages answer from this tree alone: no page names a scheme this tree cannot serve,
 every local name resolves to a committed file, the Design Component runtime loads its vendored builds, and every
 typeface a page sets is one the design system ships.
 
 The browser half of the same promise, that each page renders with no request leaving this tree, lives in
 `offline-design-pages-render-test.ts`.
 */

import { createHash } from 'node:crypto';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { dirname, join } from 'node:path';

import {
  ALLOWED_FONT_FAMILIES,
  SHIPPED_FONT_FAMILIES,
  classifyReference,
  designPages,
  designRoot,
  displayPath,
  facesDefinedBy,
  familyNames,
  fontFamilyValues,
  fontShorthandValues,
  pageReferences,
  readText,
  resolveReference,
  stylesheetReferences,
} from './design-page-dependencies.ts';

//region Reading the tree

/** Record naming every vendored build, its provenance record and the runtime copies that load them. */
const BUILD_RECORD = join(
  'vendor',
  'builds.json',
);

/** One vendored build as the record states it. */
type BuildRecord = {
  builds?: {
    file?: string;
    integrity?: string;
    license?: string;
    package?: string;
    version?: string;
  }[];
  runtimes?: string[];
};

/** The vendored-build record, or `null` when the tree holds none a reader can parse. */
function readBuildRecord({ root, }: { root: string; },): BuildRecord | null {
  const text = readText(join(
    root,
    BUILD_RECORD,
  ),);
  if (text === null) {
    return null;
  }
  try {
    const parsed = JSON.parse(text,) as unknown;
    return typeof parsed === 'object' && parsed !== null ? parsed as BuildRecord : null;
  }
  catch {
    return null;
  }
}

/** Names of the vendored builds the record lists. */
function recordedBuildNames(record: BuildRecord | null,): readonly string[] {
  const builds = record?.builds ?? [];
  return builds.map(function name(build,): string {
    return String(build.file,);
  },);
}

/** A reference to a file, without the query string or fragment a browser never sends to the server. */
function withoutQuery(reference: string,): string {
  return reference.split('#',)[0].split('?',)[0];
}

//endregion Reading the tree

//region Guards

/**
 No page names a host. A page may embed data or link a fragment, because both answer without a network; anything
 carrying a scheme or a protocol-relative host is a dependency this tree cannot serve.
 */
function externalReferenceFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  for (const page of designPages(root,)) {
    const html = readText(join(
      root,
      page,
    ),) ?? '';
    for (const reference of pageReferences(html,)) {
      if (classifyReference(reference,) === 'remote') {
        findings.push(`${page}: names ${reference}`);
      }
    }
  }
  return findings;
}

/** Every local name a page makes, and every local name a linked stylesheet makes, resolves to a file in the tree. */
function unresolvedReferenceFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const visited = new Set<string>();
  const pending: string[] = designPages(root,).map(function absolute(page,): string {
    return join(
      root,
      page,
    );
  },);
  while (pending.length > 0) {
    const file = pending.shift();
    if (file === undefined || visited.has(file,)) {
      continue;
    }
    visited.add(file);
    const text = readText(file,) ?? '';
    const references = file.endsWith('.css',)
      ? stylesheetReferences(text,)
      : pageReferences(text,);
    for (const reference of references) {
      if (classifyReference(reference,) !== 'local') {
        continue;
      }
      const target = resolveReference({
        fromFile: file,
        reference: withoutQuery(reference,),
      },);
      if (!existsSync(target,)) {
        findings.push(`${displayPath({
          root,
          file,
        },)}: ${reference} is not in this tree`);
        continue;
      }
      if (target.endsWith('.css',)) {
        pending.push(target,);
      }
    }
  }
  return findings;
}

/**
 Every vendored build the record names is present with the digest the record states, and no file sits in `vendor/`
 with no record saying where it came from.
 */
function vendorRecordFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const record = readBuildRecord({ root, },);
  if (record === null) {
    return [`${BUILD_RECORD} is missing or unparseable, so no vendored build has a recorded provenance`];
  }
  const names = new Set<string>(recordedBuildNames(record,),);
  for (const build of record.builds ?? []) {
    const name = String(build.file,);
    const file = join(
      root,
      'vendor',
      name,
    );
    if (!existsSync(file,)) {
      findings.push(`vendor/${name} is recorded and missing`);
      continue;
    }
    const digest = `sha384-${createHash('sha384',).update(readFileSync(file,),).digest('base64',)}`;
    if (digest !== build.integrity) {
      findings.push(`vendor/${name} is ${digest}, and the record states ${String(build.integrity,)}`);
    }
    for (const field of [
      'license',
      'package',
      'version',
    ] as const) {
      if ((build[field] ?? '').length === 0) {
        findings.push(`vendor/${name} records no ${field}`);
      }
    }
  }
  const vendorDirectory = join(
    root,
    'vendor',
  );
  if (existsSync(vendorDirectory,)) {
    for (const entry of readdirSync(vendorDirectory,)) {
      if (entry !== 'builds.json' && !names.has(entry,)) {
        findings.push(`vendor/${entry} is committed with no entry in ${BUILD_RECORD}`);
      }
    }
  }
  for (const runtime of record.runtimes ?? []) {
    if (!existsSync(join(
      root,
      String(runtime,),
    ),)) {
      findings.push(`${String(runtime,)} is recorded as a runtime copy and missing`);
    }
  }
  return findings;
}

/**
 The runtime asks this tree for React, ReactDOM and Babel, names every build file the record lists, keeps no
 integrity or cross-origin attribute that a local script does not need and `file://` cannot satisfy, and keeps its
 copies identical apart from the one line saying where `vendor/` sits from that copy.
 */
function runtimeBuildFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const record = readBuildRecord({ root, },);
  if (record === null) {
    return [`${BUILD_RECORD} is missing or unparseable, so no runtime copy is known`];
  }
  const builds = recordedBuildNames(record,);
  const copies = (record.runtimes ?? []).map(function read(name,): {
    name: string;
    text: string;
  } {
    return {
      name: String(name,),
      text: readText(join(
        root,
        String(name,),
      ),) ?? '',
    };
  },);
  if (copies.length < 2) {
    findings.push(`${BUILD_RECORD} names ${copies.length} runtime copies; this guard compares at least two`);
  }
  const normalized = new Set<string>();
  for (const copy of copies) {
    if (copy.text.length === 0) {
      findings.push(`${copy.name}: is recorded and unreadable`);
      continue;
    }
    for (const match of copy.text.matchAll(/https?:\/\/[^\s"')]+/gu,)) {
      findings.push(`${copy.name}: loads ${match[0]} over the network`);
    }
    for (const build of builds) {
      if (!copy.text.includes(build,)) {
        findings.push(`${copy.name}: never asks this tree for ${build}`);
      }
    }
    for (const attribute of [
      'integrity',
      'crossOrigin',
    ]) {
      if (copy.text.includes(attribute,)) {
        findings.push(`${copy.name}: keeps ${attribute}, which a local script does not need and file:// cannot satisfy`);
      }
    }
    normalized.add(copy.text.replace(
      /var VENDOR_BASE = "[^"]*";/u,
      'var VENDOR_BASE = "<vendor base>";',
    ),);
  }
  if (normalized.size > 1) {
    findings.push('the runtime copies differ beyond their vendor base');
  }
  return findings;
}

/**
 A page sets only a typeface this tree ships or one the browser already has, and a shipped typeface it sets is
 defined by a stylesheet it links, so no family silently falls back.
 */
function shippedFontFindings({ root, }: { root: string; },): readonly string[] {
  const findings: string[] = [];
  const shipped: ReadonlySet<string> = new Set(SHIPPED_FONT_FAMILIES,);
  for (const page of designPages(root,)) {
    const file = join(
      root,
      page,
    );
    const html = readText(file,) ?? '';
    for (const shorthand of fontShorthandValues(html,)) {
      findings.push(`${page}: sets font: ${shorthand}, which hides its family from this guard`);
    }
    const named = new Set<string>();
    for (const value of fontFamilyValues(html,)) {
      for (const name of familyNames(value,)) {
        named.add(name,);
      }
    }
    for (const name of named) {
      if (!ALLOWED_FONT_FAMILIES.has(name,)) {
        findings.push(`${page}: sets ${name}, which this tree does not ship`);
      }
    }
    const defined = new Set<string>();
    for (const reference of pageReferences(html,)) {
      const path = withoutQuery(reference,);
      if (classifyReference(reference,) !== 'local' || !path.endsWith('.css',)) {
        continue;
      }
      const css = readText(resolveReference({
        fromFile: file,
        reference: path,
      },),) ?? '';
      for (const family of facesDefinedBy(css,)) {
        defined.add(family,);
      }
    }
    for (const name of named) {
      if (shipped.has(name,) && !defined.has(name,)) {
        findings.push(`${page}: sets ${name} and links no stylesheet defining it`);
      }
    }
  }
  return findings;
}

//endregion Guards

//region Control fixtures

/** A file tree the guards read the way they read the committed one. */
type Fixture = {
  root: string;
  files: Record<string, string>;
};

/** Scratch directory the fixtures live in, created under the package's gitignored tree. */
function scratchDirectory(): string {
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

/** Digest the vendored-build record states for a fixture file. */
function digestOf(content: string,): string {
  return `sha384-${createHash('sha384',).update(content,).digest('base64',)}`;
}

/** A runtime that asks this tree for its build, as the committed copies do. */
const ACCEPTED_RUNTIME = `(function () {
  var VENDOR_BASE = "./vendor/";
  var REACT_BUILD = "react-18.3.1.production.min.js";
  function vendorScriptFor(build) { return new URL(build, document.baseURI).href; }
  document.head.appendChild(Object.assign(document.createElement("script"), { src: vendorScriptFor(REACT_BUILD) }));
})();
`;

/** Digest the rejected fixture states, so a swapped build is caught rather than matched. */
const WRONG_DIGEST = `sha384-${'A'.repeat(86)}`;

/** Fixtures proving each guard reports the defect it exists for, and accepts a tree that has none. */
function fixtures(base: string,): {
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
        'candidates/support.js': `${ACCEPTED_RUNTIME}// copies drift\n`,
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

//endregion Control fixtures

//region Report

/**
 Guards this file holds the tree to. A control fixture carries one defect per control entry, so a guard that stops
 reporting its defect fails this file rather than passing quietly.
 */
const CHECKS = [
  {
    name: 'no-external-reference',
    run: externalReferenceFindings,
  },
  {
    name: 'references-resolve',
    run: unresolvedReferenceFindings,
  },
  {
    name: 'vendor-builds-recorded',
    run: vendorRecordFindings,
  },
  {
    name: 'runtime-loads-vendored-builds',
    run: runtimeBuildFindings,
  },
  {
    name: 'shipped-fonts-only',
    run: shippedFontFindings,
  },
] as const;

/** One defect the control fixture carries, and the guard that must name it. */
const CONTROLS = [
  {
    defect: 'fonts.googleapis.com',
    run: externalReferenceFindings,
  },
  {
    defect: 'absent.css is not in this tree',
    run: unresolvedReferenceFindings,
  },
  {
    defect: '../fonts/absent.woff2 is not in this tree',
    run: unresolvedReferenceFindings,
  },
  {
    defect: 'and the record states',
    run: vendorRecordFindings,
  },
  {
    defect: 'committed with no entry',
    run: vendorRecordFindings,
  },
  {
    defect: 'loads https://unpkg.com',
    run: runtimeBuildFindings,
  },
  {
    defect: 'differ beyond their vendor base',
    run: runtimeBuildFindings,
  },
  {
    defect: 'sets Noto Sans, which this tree does not ship',
    run: shippedFontFindings,
  },
  {
    defect: 'sets Google Sans Text and links no stylesheet defining it',
    run: shippedFontFindings,
  },
  {
    defect: 'hides its family from this guard',
    run: shippedFontFindings,
  },
] as const;

const base = scratchDirectory();
try {
  const control = fixtures(base,);
  const failures: string[] = [];
  for (const entry of CONTROLS) {
    const reported = entry.run({ root: control.rejected, },);
    if (!reported.some(function mentions(finding,): boolean {
      return finding.includes(entry.defect,);
    },)) {
      failures.push(`control: ${entry.defect} went unreported in ${reported.length} findings`);
    }
  }
  const acceptedFindings = CHECKS.flatMap(function controls(check,): readonly string[] {
    return check.run({ root: control.accepted, },);
  },);
  if (acceptedFindings.length > 0) {
    failures.push(`control: a clean tree reported ${acceptedFindings.join('; ')}`);
  }

  const findings = CHECKS.flatMap(function read(check,): readonly string[] {
    return check.run({ root: designRoot, },).map(function label(finding,): string {
      return `${check.name}: ${finding}`;
    },);
  },);

  const lines = [
    ...failures,
    ...findings,
  ];
  if (lines.length > 0) {
    throw new Error(`Offline design pages: ${lines.length} findings\n${lines.join('\n')}`);
  }
  console.log(
    `Offline design pages: ${designPages(designRoot,).length} pages and ${CHECKS.length} guards, each proven able to fail.`,
  );
}
finally {
  rmSync(
    base,
    { recursive: true, force: true, },
  );
}

//endregion Report
