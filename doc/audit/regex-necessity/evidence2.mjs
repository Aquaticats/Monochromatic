// Corrected evidence probes. Each replaces an earlier probe whose input did not
// exercise the case it claimed to test.
function timeIt(fn) {
  const start = process.hrtime
    .bigint();
  fn();
  return Number(process.hrtime
    .bigint()
    - start) / 1e6;
}
function series(
  label,
  build,
  run
) {
  const rows = [];
  for (const size of [
    16_000,
    32_000,
    64_000,
    128_000
  ]) {
    const input = build(size);
    const times = [
      1,
      2,
      3
    ].map(() => timeIt(() => run(input)));
    rows.push({
      chars: input.length,
      ms: times.map((value) => Number(value.toFixed(1))),
    });
  }
  console.log(`${label}: ${JSON.stringify(rows)}`);
  return rows;
}

// Probe 1 (corrected): the re-export matcher with MANY unterminated `export {`
// openers, so each one scans the brace-free remainder. The earlier probe put the
// filler before a single opener, which is linear by construction.
const NAME = 'zzzMissing';
const reexport = new RegExp(`export\\s*\\{[^}]*\\b${NAME}\\b[^}]*\\}\\s*from\\s*['"]`);
series(
  'P1 re-export matcher, many unterminated openers',
  (size) => 'export { aaaaaa '.repeat(Math.floor(size / 17)),
  (input) => reexport.test(input)
);

// Probe 2 (corrected): the fence pattern. A lazy A..A with a global flag consumes
// through each terminator, so the worst case is one trailing unmatched opener, not
// one rescan per opener. Measured to state the bound honestly.
const fence = /```[\s\S]*?```/gu;
series(
  'P2 fence strip, single trailing unmatched opener',
  (size) => `\`\`\`${'a'.repeat(size)}`,
  (input) => input.replaceAll(
    fence,
    ' '
  )
);
series(
  'P2b fence strip, many matched pairs',
  (size) => '```a```'.repeat(Math.floor(size / 7)),
  (input) => input.replaceAll(
    fence,
    ' '
  )
);

// Probe 3 (corrected): the u-flag claim, tested against the four production
// patterns with identifiers that actually match, plus an interior-$ identifier
// that cannot match in either mode.
const patterns = {
  fn: (name) => `(?:^|\\n)export\\s+(?:async\\s+)?function\\s*\\*?\\s+${name}\\b`,
  cls: (name) => `(?:^|\\n)export\\s+class\\s+${name}\\b`,
  konst: (name) => `(?:^|\\n)export\\s+const\\s+${name}\\b`,
  reexp: (name) => `export\\s*\\{[^}]*\\b${name}\\b[^}]*\\}\\s*from\\s*['"]`,
};
const cases = [
  {
    name: 'describe',
    content: 'export const describe = 1;\nexport function describe2() {}\nexport { describe } from "./a.ts";\n',
  },
  {
    name: 'ALL_CAPS',
    content: 'export const ALL_CAPS = 1;\nexport class ALL_CAPS {}\n',
  },
  {
    name: '_under',
    content: 'export const _under = 1;\n',
  },
  {
    name: 'a1',
    content: 'export function a1() {}\n',
  },
  {
    name: 'café',
    content: 'export const café = 1;\n',
  },
  {
    name: 'a$b',
    content: 'export const a$b = 1;\n',
  },
];
const flagRows = [];
for (const kind of Object.keys(patterns)) {
  for (const test of cases) {
    const source = patterns[kind](test.name);
    let plain;
    let unicode;
    let error = null;
    try {
      plain = new RegExp(source).test(test.content);
      unicode = new RegExp(
        source,
        'u'
      ).test(test.content);
    } catch (caught) {
      error = String(caught)
        .slice(
        0,
        80
        );
    }
    flagRows.push({
      pattern: kind,
      name: test.name,
      plain,
      unicode,
      differs: plain !== unicode,
      error,
    });
  }
}
console.log(`P3 u-flag differences: ${JSON.stringify(flagRows.filter((row) => row.differs || (row.error !== null)))}`);
console.log(`P3 matching cases (positive control): ${JSON.stringify(flagRows.filter((row) => row.plain === true)
  .map((row) => `${row.pattern}/${row.name}`))}`);
console.log(`P3 interior-$ identifier results: ${JSON.stringify(flagRows.filter((row) => row.name === 'a$b'))}`);

// Probe 4 (corrected): the SVG tokenizer worst case, with the input construction
// stated exactly, and the proposed replacement measured on the same input.
const element = /<(?<tag>rect|path)\s+(?<attrs>[^>]*?)\/>/gu;
function scanReplacement(content) {
  // The linear replacement the issues propose: cursor over indexOf.
  let cursor = 0;
  let found = 0;
  while (cursor < content.length) {
    const rect = content.indexOf(
      '<rect',
      cursor
    );
    const path = content.indexOf(
      '<path',
      cursor
    );
    const start = rect === (-1) ? path : path === (-1) ? rect : Math.min(
      rect,
      path
    );
    if (start === (-1)) break;
    const end = content.indexOf(
      '/>',
      start
    );
    if (end === (-1)) break;
    found += 1;
    cursor = end + 2;
  }
  return found;
}
series(
  'P4 svg regex on unterminated openers',
  (size) => '<rect '.repeat(Math.floor(size / 6)),
  (input) => {
    element.lastIndex = 0;
    return element.exec(input);
  }
);
series(
  'P4b svg cursor replacement on the same input',
  (size) => '<rect '.repeat(Math.floor(size / 6)),
  (input) => scanReplacement(input)
);

// Probe 5: \p{Upper} and \p{Lower} against case mapping, reported separately so
// the divergence is attributed to the right comparison.
const upper = /\p{Upper}/v;
const lower = /\p{Lower}/v;
const samples = [
  'A',
  'a',
  'Ǆ',
  'ǅ',
  'ǆ',
  'ᵃ',
  'ᴬ',
  'Σ',
  'ς',
  'İ',
  'ı',
  'Ⅻ',
  '0',
  '_',
  '·'
];
const rows = samples.map((sample) => ({
  sample,
  cp: `U+${sample.codePointAt(0)
    .toString(16)
    .toUpperCase()
    .padStart(
      4,
      '0'
    )}`,
  upperProperty: upper.test(sample),
  upperCaseMapped: (sample.toLowerCase() !== sample) && (sample.toUpperCase() === sample),
  lowerProperty: lower.test(sample),
  lowerCaseMapped: (sample.toUpperCase() !== sample) && (sample.toLowerCase() === sample),
}));
console.log(`P5 upper divergences: ${JSON.stringify(rows.filter((row) => row.upperProperty !== row.upperCaseMapped))}`);
console.log(`P5 lower divergences: ${JSON.stringify(rows.filter((row) => row.lowerProperty !== row.lowerCaseMapped))}`);

// Probe 6: the IPv4 classifier against its own documented contract.
const ipPattern = /^[0-9.]+$/u;
const tokens = [
  '198.245.51.147',
  '...',
  '.',
  '1..2',
  '1.2.3.4.5',
  '999.999.999.999',
  'dns.google'
];
console.log(`P6 digits-or-dots classifier: ${JSON.stringify(tokens.map((token) => ({
  token,
  accepted: ipPattern.test(token),
})))}`);
