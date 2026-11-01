#!/usr/bin/env node
// Write the third-party license notices of one music player binary as one plain-text file.
// Decision and allow-list: doc/decision/music-player-third-party-licenses.md.
//
// Usage (--crate names the folder of the crate whose about.toml lists the targets and accepted licenses; it defaults
// to the current folder, and every relative path is resolved against the current folder):
//   node <this file> --product desktop --output target/THIRD-PARTY-LICENSES.txt [--features alsa-fallback]
//   node <this file> --product android --crate rust --output <file> \
//     --android-libraries <licensee artifacts.json> --android-library-notices <collected NOTICE files>
// The mise tasks `notices` of desktop-app and android-app run it.
//
// What it does, in order, stopping with an error at the first failure:
//   1. `cargo fetch --locked` (network), so every crate of every listed target has its source on disk.
//   2. `cargo-about generate --format json --frozen --fail`: one license per crate, chosen from the accepted list in
//      about.toml in its order of preference, with the crate's own license file when it ships one. A crate whose
//      license is not accepted or cannot be determined stops it. cargo-about 0.9.2 drops the text of a LicenseRef-*
//      license from its output (doc/troubleshooting/cargo-about-licenseref-text-is-dropped.md), so the desktop
//      Slint crates, which are accepted under LicenseRef-Slint-Royalty-free-2.0, are found by their manifest license
//      and their text is added from notices/slint-royalty-free-2.0.md once every crate's own file matches it.
//   3. For every target in about.toml, `cargo tree --edges normal,no-proc-macro`: every registry crate that target
//      links into the binary must be among the crates cargo-about covered; the counts are printed.
//   4. The file: the program's own license texts, the bundled assets, the Rust standard library, the Android libraries
//      when given, and every Rust crate, grouped by license text.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import {
  dirname,
  join,
  resolve
} from 'node:path';
import { parseArgs } from 'node:util';

//region Arguments and paths
const { values: options } = parseArgs({
  options: {
    product: { type: 'string' },
    output: { type: 'string' },
    crate: {
      type: 'string',
      default: '.'
    },
    features: {
      type: 'string',
      default: ''
    },
    'android-libraries': { type: 'string' },
    'android-library-notices': { type: 'string' },
    'skip-fetch': {
      type: 'boolean',
      default: false
    },
  },
});
if ((options.product !== 'desktop') && (options.product !== 'android')) {
  throw new Error(`--product must be desktop or android, got ${  JSON.stringify(options.product)}`);
}
if (!options.output) throw new Error('--output names the notice file to write');
if ((options.product === 'android') && (!(options['android-libraries'] && options['android-library-notices']))) {
  throw new Error('--product android needs --android-libraries and --android-library-notices (the Gradle outputs)');
}
if ((options.product !== 'android') && (options['android-libraries'] || options['android-library-notices'])) {
  throw new Error('--android-libraries and --android-library-notices apply only to --product android');
}
const crateDirectory = resolve(options.crate);
const scriptDirectory = dirname(new URL(import.meta.url).pathname);
const musicPlayer = resolve(
  scriptDirectory,
  '..',
  '..'
);
const repository = resolve(
  musicPlayer,
  '..',
  '..'
);
const output = resolve(options.output);
const featureArgs = options.features ? [
  '--features',
  options.features
] : [];
const newline = String.fromCharCode(10);
//endregion

//region Commands
function run(
  command,
  args,
  extra = {}
) { return execFileSync(
  command,
  args,
  {
    cwd: crateDirectory,
    encoding: 'utf8',
    maxBuffer: 256 * 1_024
      * 1_024,
    ...extra
  }
); }

// cargo-about comes from the repository's mise tools (mise.lock records the version).
let aboutVersion = '';
try {
  aboutVersion = run(
    'cargo-about',
    ['--version'],
    { stdio: [
      'ignore',
      'pipe',
      'pipe'
    ] }
  )
    .trim();
} catch (error) {
  throw new Error(
    `cargo-about is not on PATH (${
       String(error.message)
         .split(newline)[0]
       }). `
      + `Install the repository tools with \`mise install cargo:cargo-about\` and run this through its mise task.`,
    { cause: error },
  );
}
console.log(`Using ${  aboutVersion}`);
if (!options['skip-fetch']) run(
  'cargo',
  [
    'fetch',
    '--locked'
  ],
  { stdio: 'inherit' }
);
//endregion

//region cargo-about
const aboutConfig = readFileSync(
  join(
    crateDirectory,
    'about.toml'
  ),
  'utf8'
);
const targetsMatch = aboutConfig.match(/^targets = \[([^\]]*)\]/m);
if (!targetsMatch) throw new Error(`about.toml in ${  crateDirectory
   } has no targets list`);
const targets = [...targetsMatch[1]
  .matchAll(/"([^"]+)"/g)].map(match => match[1]);
const aboutJson = `${output  }.about.json`;
mkdirSync(
  dirname(output),
  { recursive: true }
);
rmSync(
  aboutJson,
  { force: true }
);
// --frozen: the locked graph and no network. --fail: an unaccepted or unknown license stops it.
run(
  'cargo-about',
  [
    '--color',
    'never',
    'generate',
    '--format',
    'json',
    '--frozen',
    '--fail',
    '--config',
    'about.toml',
    ...featureArgs,
    '--output-file',
    aboutJson,
  ],
  { stdio: 'inherit' }
);
const about = JSON.parse(readFileSync(
  aboutJson,
  'utf8'
));
rmSync(
  aboutJson,
  { force: true }
);
if ((!Array.isArray(about.licenses)) || (about.licenses
  .length
  === 0)) throw new Error('cargo-about wrote no license text');
function crateKey(item) { return `${item.name  } ${  item.version}`; }
const coveredByAbout = new Set(about.licenses
  .flatMap(license => license.used_by
    .map(used => crateKey(used.crate))));
//endregion

//region Slint crates (desktop only)
// The accepted license of the Slint crates, which cargo-about accepts but gives no text for. A crate is a Slint crate
// when its manifest offers this license and cargo-about listed no text for it; any other crate without a text, or a
// Slint crate whose own license file differs from the vendored one, stops the task.
const slintLicenseId = 'LicenseRef-Slint-Royalty-free-2.0';
const slintLicensePath = join(
  musicPlayer,
  'desktop-app',
  'notices',
  'slint-royalty-free-2.0.md'
);
// SHA-256 of the license file in every Slint crate 1.18.1 (LICENSES/LicenseRef-Slint-Royalty-free-2.0.md), identical to
// the file on slint-ui/slint master on 2026-10-18. A changed file stops the task until it is reviewed.
const slintLicenseChecksum = '5167f5056e850419106ab6265efbdca7cba4d99c849d1445ca0bbf6a1e2315fe';
const slintCrates = new Map();
for (const item of about.crates) {
  const key = crateKey(item.package);
  if (coveredByAbout.has(key)) continue;
  const offered = String(item.license ?? '')
    .split(/\s+(?:OR|AND)\s+/)
    .map(part => part.replaceAll(
      /[()]/g,
      ''
    )
      .trim());
  if ((options.product !== 'desktop') || (!offered.includes(slintLicenseId))) {
    throw new Error(
      `cargo-about gave no license text for ${  key
         } (license ${
         JSON.stringify(item.license)
         }), and it is `
        + `not a desktop Slint crate; check about.toml`,
    );
  }
  slintCrates.set(
    key,
    join(
      dirname(item.package
        .manifest_path
        ?? ''),
      'LICENSES',
      `${slintLicenseId  }.md`
    )
  );
}
if ((options.product === 'desktop') && (slintCrates.size === 0)) {
  throw new Error(`No Slint crate found: the desktop program must carry ${  slintLicenseId
     } and its text`);
}
const slintLicenseText = options.product === 'desktop' ? readFileSync(
  slintLicensePath,
  'utf8'
) : '';
if (options.product === 'desktop') {
  if (createHash('sha256')
    .update(slintLicenseText)
    .digest('hex')
    !== slintLicenseChecksum) {
    throw new Error(`${slintLicensePath  } does not match the checksum recorded in this script; review it first`);
  }
  for (const [key, file] of slintCrates) {
    if ((!existsSync(file)) || (createHash('sha256')
      .update(readFileSync(file))
      .digest('hex')
      !== slintLicenseChecksum)) {
      throw new Error(`${key  } does not ship ${
         file
         } with the vendored license text; review the new Slint license first`);
    }
  }
}
const covered = new Set([
  ...coveredByAbout,
  ...slintCrates.keys()
]);
//endregion

//region Coverage check against cargo tree
const linkedUnion = new Set();
for (const target of targets) {
  const tree = run(
    'cargo',
    [
      'tree',
      '--frozen',
      '--edges',
      'normal,no-proc-macro',
      '--target',
      target,
      ...featureArgs,
      '--prefix',
      'none',
      '--format',
      '{p}',
    ],
    { stdio: [
      'ignore',
      'pipe',
      'inherit'
    ] }
  );
  // A path dependency prints its folder after the version: it is this repository's own code, not a third party's.
  const linked = [...new Set(tree.split(newline)
    .filter(line => line && (!line.includes(' (/')))
    .map(line =>
      line.replace(
        / \(\*\)$/,
        ''
      )
        .replace(
        / \(.*\)$/,
        ''
        )
        .replace(
        ' v',
        ' '
        )
    ))].sort();
  const missing = linked.filter(key => !covered.has(key));
  if (missing.length > 0) {
    throw new Error(`${target  } links crates the notice does not cover: ${  missing.join(', ')}`);
  }
  for (const key of linked) linkedUnion.add(key);
  console.log(`${target  }: ${
     linked.length
     } third-party crates linked, all covered`);
}
console.log(
  `cargo-about covered ${
     covered.size
     } crates (${
     about.licenses
       .length
     } license texts); ${
     linkedUnion.size
     } of them are linked into a binary of some target; the other ${
     covered.size - linkedUnion.size
     } are reached only through procedural macros, which run while compiling`,
);
//endregion

//region Text helpers
function rule(character) { return character.repeat(78); }
function heading(title) { return [
  rule('='),
  title,
  rule('='),
  ''
].join(newline); }
function subheading(title) { return [
  title,
  rule('-'),
  ''
].join(newline); }
function readText(path) { return readFileSync(
  path,
  'utf8'
)
  .replaceAll(
  '\r\n',
  newline
  )
  .trimEnd(); }
// The Rust toolchain's own notice for its standard library is HTML; keep its text and decode the entities it uses.
function htmlToText(html) { return html
  .replaceAll(
  /<(head|script|style)[\s\S]*?<\/\1>/g,
  ''
  )
  .replaceAll(
  /<br\s*\/?>/g,
  newline
  )
  .replaceAll(
  /<\/(p|h[1-6]|li|pre|div|tr)>/g,
  newline
  )
  .replaceAll(
  /<[^>]+>/g,
  ''
  )
  .replaceAll(
  /&#(\d+);/g,
  (
    _,
    code
  ) => String.fromCodePoint(Number(code))
  )
  .replaceAll(
  '&lt;',
  '<'
  )
  .replaceAll(
  '&gt;',
  '>'
  )
  .replaceAll(
  '&quot;',
  '"'
  )
  .replaceAll(
  '&#39;',
  "'"
  )
  .replaceAll(
  '&amp;',
  '&'
  )
  .split(newline)
  .map(line => line.trimEnd())
  .join(newline)
  .replaceAll(
  /\n{3,}/g,
  newline + newline
  )
  .trim(); }
//endregion

//region Sections
const productVersion = readFileSync(
  join(
    musicPlayer,
    'desktop-app',
    'Cargo.toml'
  ),
  'utf8'
)
  .match(/^\[package\][\s\S]*?^version = "([^"]+)"/m)?.[1];
if (!productVersion) throw new Error('No [package] version in package/music-player/desktop-app/Cargo.toml');
const sections = [];
const title = options.product === 'desktop'
  ? `Music Player ${  productVersion  } for Windows, macOS and Linux: license notices`
  : `Music Player ${  productVersion
     } for Android: license notices`;
const intro = options.product === 'desktop'
  ? [
    'This file lists the licenses of Music Player and of every third-party component built into the program:',
    'the program\'s own license, the typeface and icons it draws with, the Rust standard library, and every Rust',
    'crate (library) compiled into it. `music-player --licenses` prints the same text.',
    '',
    'Music Player is licensed under the GNU Lesser General Public License, version 3 or later. The program is built',
    'with the Slint user interface toolkit, which this project uses under the Slint Royalty-free Desktop, Mobile,',
    'and Web Applications License, version 2.0, one of the three licenses Slint offers. Its text, and the crates it',
    'covers, are in the Slint section below; the Licenses page in Settings shows the "Made with Slint" widget that',
    'license asks for. The complete source code of each release is the repository tree at its tag',
    '(https://github.com/Aquaticats/Monochromatic, tag',
    `music-player-v${  productVersion
       }), and the source of every crate below is published on crates.io under the`,
    'name and version given.',
    '',
    'One notice file covers the programs for every platform, so it also names crates that only another platform\'s',
    'program contains.',
  ]
  : [
    'This file lists the licenses of Music Player for Android and of every third-party component built into the',
    'app: the app\'s own license, the icons it draws, the Android libraries it includes, the Rust standard library',
    'and every Rust crate (library) compiled into its native library (libmusicplayer_native.so).',
    'The app carries the same text as assets/THIRD-PARTY-LICENSES.txt inside the APK.',
    '',
    'Music Player is licensed under the GNU Lesser General Public License, version 3 or later. The complete source',
    'code of each release is the repository tree at its tag (https://github.com/Aquaticats/Monochromatic, tag',
    `music-player-v${  productVersion
       }); the source of every crate below is published on crates.io under the`,
    'name and version given, and the Android libraries are published on Google\'s Maven repository and Maven',
    'Central under the coordinates given.',
  ];
sections.push(
  [
    heading(title),
    ...intro,
    ''
  ].join(newline),
  [
    heading('Music Player: GNU Lesser General Public License, version 3 or later'),
    'Copyright (C) the Monochromatic authors.',
    '',
    readText(join(
      repository,
      'LICENSE'
    )),
    '',
    subheading('The GNU General Public License, version 3, which the license above supplements'),
    readText(join(
      repository,
      'LICENSES',
      'GPL-3.0-or-later.txt'
    )),
    '',
  ].join(newline)
);

const apacheText = readText(join(
  repository,
  'LICENSES',
  'Apache-2.0.txt'
));
const bundled = [heading('Assets drawn by the program')];
if (options.product === 'desktop') {
  bundled.push(
    subheading('Inter typeface (ui/fonts/InterVariable.ttf), SIL Open Font License 1.1'),
    readText(join(
      musicPlayer,
      'desktop-app',
      'LICENSES',
      'OFL-1.1.txt'
    )),
    '',
  );
}
// Both products draw license in their Licenses row; only the order of the names differs.
const iconNames = options.product === 'android'
  ? 'queue_music, play_circle, autoplay, license, check and close'
  : 'queue_music, play_circle, autoplay, check, close and license';
bundled.push(
  subheading('Material Symbols icons, Apache License 2.0'),
  `The icons ${  iconNames  } (Outlined style, weight 400, grade 0, optical size`,
  '24) come from https://github.com/google/material-design-icons at commit',
  '49d4db35df873165d6bd6ba09b063c7dafbac2f4, published by Google under the Apache License, Version 2.0.',
  'The program embeds their outlines as vector path data, with each outline moved onto a 24 by 24 grid.',
  '',
  apacheText,
  '',
);
sections.push(bundled.join(newline));

if (options.product === 'desktop') {
  // Slint: the toolkit the desktop programs draw their window with. The text is the one every Slint crate ships.
  sections.push([
    heading(`Slint (${  slintCrates.size
       } crates): Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0`),
    `The desktop programs use Slint under the license below (SPDX identifier ${  slintLicenseId
       }), one of the`,
    'three licenses its crates offer. Section 2 asks the Application to display the AboutSlint widget in an About',
    'screen or dialog; the Licenses page of Settings does that. The crates it covers, each of which ships this',
    `exact text as LICENSES/${  slintLicenseId  }.md:`,
    '',
    ...[...slintCrates.keys()].sort(),
    '',
    subheading(`Text of ${  slintLicenseId}`),
    slintLicenseText.replaceAll(
      '\r\n',
      newline
    )
      .trimEnd(),
    '',
  ].join(newline));
}

const sysroot = run(
  'rustc',
  [
    '--print',
    'sysroot'
  ],
  { stdio: [
    'ignore',
    'pipe',
    'inherit'
  ] }
)
  .trim();
const stdNotice = join(
  sysroot,
  'share',
  'doc',
  'rust',
  'COPYRIGHT-library.html'
);
if (!existsSync(stdNotice)) throw new Error(`The Rust toolchain at ${  sysroot
   } has no ${
   stdNotice}`);
const rustcVersion = run(
  'rustc',
  ['--version'],
  { stdio: [
    'ignore',
    'pipe',
    'inherit'
  ] }
)
  .trim();
sections.push([
  heading('The Rust standard library'),
  `Compiled into the program from ${  rustcVersion  }. This is the notice that toolchain ships`,
  '(share/doc/rust/COPYRIGHT-library.html), converted to text. It names the licenses of the standard library and of',
  'the crates the standard library is built from, with their notices.',
  '',
  htmlToText(readFileSync(
    stdNotice,
    'utf8'
  )),
  '',
].join(newline));

if (options.product === 'desktop') {
  // The x86_64-pc-windows-gnu program links the MinGW-w64 startup code and runtime libraries statically (its release
  // check refuses any MinGW runtime DLL import). Their texts are kept in notices/ beside this script's package.
  sections.push([
    heading('MinGW-w64 runtime (in the Windows x64 program built with MinGW-w64 only)'),
    'The windows-x64-gnu program is linked with the MinGW-w64 C runtime startup code and libraries, which are',
    'linked into it statically. The GCC support library (libgcc) linked with it is covered by the GCC Runtime Library',
    'Exception, which asks for no notice.',
    '',
    subheading('MinGW-w64 runtime (mingw-w64 13.0.0, file COPYING)'),
    readText(join(
      musicPlayer,
      'desktop-app',
      'notices',
      'mingw-w64-runtime.txt'
    )),
    '',
    subheading('winpthreads (mingw-w64-libraries/winpthreads/COPYING)'),
    readText(join(
      musicPlayer,
      'desktop-app',
      'notices',
      'winpthreads.txt'
    )),
    '',
  ].join(newline));
}
//endregion

//region Android libraries
if (options.product === 'android') {
  const artifacts = JSON.parse(readFileSync(
    resolve(options['android-libraries']),
    'utf8'
  ));
  if ((!Array.isArray(artifacts)) || (artifacts.length === 0)) throw new Error(`No Android library in ${  options['android-libraries']}`);
  const byLicense = new Map();
  for (const artifact of artifacts) {
    const ids = (artifact.spdxLicenses ?? []).map(license => license.identifier);
    if ((ids.length === 0) || ((artifact.unknownLicenses ?? []).length > 0)) {
      throw new Error(`Android library ${  artifact.groupId  }:${  artifact.artifactId  } has no SPDX license`);
    }
    const key = ids.join(' OR ');
    byLicense.set(
      key,
      [
        ...(byLicense.get(key) ?? []),
        `${artifact.groupId  }:${
           artifact.artifactId
           }:${
           artifact.version}`
      ]
    );
  }
  const libraryTexts = { 'Apache-2.0': apacheText };
  const androidSection = [heading(`Android libraries (${  artifacts.length
     } artifacts)`)];
  for (const [license, coordinates] of [...byLicense].sort()) {
    const text = libraryTexts[license];
    if (!text) throw new Error(`No license text for the Android library license ${  license
       }; add it to the notices`);
    androidSection.push(
      subheading(`Under ${  license  }: ${  coordinates.length  } artifacts`),
      ...coordinates.sort(),
      '',
      text,
      '',
    );
  }
  // Apache License 2.0 section 4(d): NOTICE files inside the libraries travel with the app. APK packaging drops
  // META-INF/NOTICE* files, so the Gradle task collectAndroidLibraryNotices copies them out of the resolved release
  // libraries first; an empty file means none of them carries one.
  const libraryNotices = readText(resolve(options['android-library-notices']));
  androidSection.push(
    subheading('NOTICE files carried by the Android libraries'),
    libraryNotices.length > 0 ? libraryNotices : 'None of the Android libraries carries a NOTICE file.',
    '',
  );
  sections.push(androidSection.join(newline));
}
//endregion

//region Rust crates
const crateSection = [heading(
  `Rust crates (${  covered.size
     } crates, ${
     about.licenses
       .length
       + (slintCrates.size > 0 ? 1 : 0)
     } license texts)`,
)];
crateSection.push(
  'Each entry below is one license text and the crates it covers. When a crate offers several licenses, the first of',
  `these is the one listed: ${
     [...aboutConfig.matchAll(/^ {2}"([^"]+)",$/gm)].map(match => match[1])
       .join(', ')
     }.`,
  '',
  ...(slintCrates.size > 0
    ? [
      `The ${  slintCrates.size
         } Slint crates are covered by the Slint section above, with the text of their license.`,
      ''
    ]
    : []),
);
const sortedLicenses = [...about.licenses].sort((
  left,
  right
) =>
  left.id
    .localeCompare(right.id)
    || crateKey(left.used_by[0]
      .crate)
      .localeCompare(crateKey(right.used_by[0]
        .crate))
);
for (const license of sortedLicenses) {
  const crates = [...new Set(license.used_by
    .map(used => crateKey(used.crate)))].sort();
  const source = license.source_path
    ? `Text from the crate file ${
       license.source_path
         .split(/[\\/]/)
         .slice(-2)
         .join('/')}`
    : 'Text: the standard text of this license (no crate file was recognized)';
  crateSection.push(
    subheading(`Rust crates under ${  license.name  } (${  license.id  }): ${  crates.length  } crates`),
    ...crates,
    '',
    source,
    '',
    license.text
      .replaceAll(
      '\r\n',
      newline
      )
      .trim(),
    '',
  );
}
sections.push(crateSection.join(newline));
//endregion

//region Write
const text = sections.join(newline) + newline;
const partial = `${output  }.partial`;
writeFileSync(
  partial,
  text
);
renameSync(
  partial,
  output
);
console.log(`Wrote ${  output  }: ${  Buffer.byteLength(text)  } bytes`);
//endregion
