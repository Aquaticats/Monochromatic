// Evidence measurements for the regex-necessity audit. Each block answers one
// factual claim made by a disable justification. Bounded inputs only.
const results = [];
function record(
  name,
  value
) {
  results.push({
    name,
    value,
  });
  console.log(`${name}: ${JSON.stringify(value)}`);
}

// Claim A (package/wg/cli-quicker/src/config-parse-values.ts:128):
// "Anchored IPv4 token classification".
const IP_PATTERN = /^[0-9.]+$/u;
const ipSamples = [
  '198.245.51.147',
  '...',
  '1.2.3.4.5.6.7.8.9',
  '999.999.999.999',
  '.',
  '1..2',
  '0.0.0.0'
];
record(
  'A. /^[0-9.]+$/ accepts these non-IPv4 strings',
  ipSamples.filter((sample) => IP_PATTERN.test(sample) && (sample !== '198.245.51.147')
    && (sample !== '0.0.0.0'))
);

// Claim B (prefer-describe-function-ref-name.ts:158): re-export matcher
// `export\s*\{[^}]*\bNAME\b[^}]*\}\s*from\s*['"]` is safe/linear.
function timeIt(
  label,
  fn
) {
  const start = process.hrtime
    .bigint();
  fn();
  const end = process.hrtime
    .bigint();
  return {
    label,
    ms: Number(end - start) / 1e6,
  };
}
const NAME = 'zzzMissingName';
const reexportPattern = new RegExp(`export\\s*\\{[^}]*\\b${NAME}\\b[^}]*\\}\\s*from\\s*['"]`);
const reexportTimings = [];
for (const size of [
  32_000,
  64_000,
  128_000,
  256_000
]) {
  // A brace-free region the two unbounded negated classes must rescan, ending with
  // an `export {` that never closes before end of input (the failure path).
  const content = `export { aaaaaa } from './x.ts';\n${  'a'.repeat(size)  }\nexport { bbbbbb`;
  reexportTimings.push(timeIt(
    `n=${size}`,
    () => reexportPattern.test(content)
  ));
}
record(
  'B. reexport matcher timings (ms) on growing brace-free input',
  reexportTimings
);

// Claim C (markdown-stream.ts:285 and parse-svg.ts:146): lazy `[\s\S]*?` /
// `[^>]*?` between delimiters is "linear".
const fencePattern = /```[\s\S]*?```/gu;
const fenceTimings = [];
for (const size of [
  16_000,
  32_000,
  64_000,
  128_000
]) {
  // Many unmatched openers: each one starts a lazy scan that runs to end of input.
  const content = '```'.repeat(Math.floor(size / 3));
  fenceTimings.push(timeIt(
    `n=${size}`,
    () => content.replaceAll(
      fencePattern,
      ' '
    )
  ));
}
record(
  'C. fence-strip replaceAll timings (ms) on unmatched openers',
  fenceTimings
);

const elementPattern = /<(?<tag>rect|path)\s+(?<attrs>[^>]*?)\/>/gu;
const elementTimings = [];
for (const size of [
  16_000,
  32_000,
  64_000,
  128_000
]) {
  // `<rect ` openers with no `/>` terminator: lazy class scan restarts per opener.
  const content = '<rect '.repeat(Math.floor(size / 6));
  let count = 0;
  elementTimings.push(timeIt(
    `n=${size}`,
    () => {
      for (let match = elementPattern.exec(content); match !== null; match = elementPattern.exec(content)) count += 1;
    }
  ));
  elementPattern.lastIndex = 0;
}
record(
  'C2. svg element tokenizer timings (ms) on unterminated openers',
  elementTimings
);

// Claim D (module/es \p{Upper} / \p{Lower}): "no string API exposes the
// Unicode upper-letter set". Compare case-mapping against the property classes.
const upper = /\p{Upper}/v;
const lower = /\p{Lower}/v;
const samples = [
  'A',
  'a',
  'ǅ',
  'Ǆ',
  'ǆ',
  'Ⅻ',
  '٠',
  '_',
  '1',
  'Σ',
  'ς',
  'İ',
  'ı',
  'ᵃ',
  'ᴬ'
];
const divergence = samples.map((sample) => ({
  sample,
  propertyUpper: upper.test(sample),
  caseMappedUpper: (sample.toLowerCase() !== sample) && (sample.toUpperCase() === sample),
  propertyLower: lower.test(sample),
  caseMappedLower: (sample.toUpperCase() !== sample) && (sample.toLowerCase() === sample),
}))
  .filter((row) => (row.propertyUpper !== row.caseMappedUpper) || (row.propertyLower !== row.caseMappedLower));
record(
  String.raw`D. \p{Upper}/\p{Lower} vs case-mapping divergence on 15 samples`,
  divergence
);

// Claim E (prefer-describe-function-ref-name.ts): omitting `u` is required
// because the interpolated identifier could match differently under u-mode.
const identifiers = [
  'describe',
  'ALL_CAPS',
  '$dollar',
  '_under',
  'a1',
  'CamelCase'
];
const content = `export const describe = 1;\nexport function ALL_CAPS() {}\n// $dollar _under a1 CamelCase\n`;
const flagDifference = identifiers.map((name) => {
  const without = new RegExp(`(?:^|\\n)export\\s+const\\s+${name}\\b`).test(content);
  const withU = new RegExp(
    `(?:^|\\n)export\\s+const\\s+${name}\\b`,
    'u'
  ).test(content);
  return {
    name,
    without,
    withU,
    differs: without !== withU,
  };
})
  .filter((row) => row.differs);
record(
  'E. identifier interpolation: u-flag changes result for these names',
  flagDifference
);

// Claim F (ssg/aquati.cat/src/lib/git-dates.ts:199): the slug pattern parses
// `owner/repo[.git]` from SSH and HTTPS remotes.
const slugPattern = /github\.com[:/](?<slug>[^/]+\/[^/.]+)(?:\.git)?$/u;
const remotes = [
  'git@github.com:Aquaticat/Monochromatic.git',
  'https://github.com/Aquaticat/Monochromatic.git',
  'git@github.com:owner/repo.js.git',
  'https://github.com/owner/dot.repo',
  'git@github.com:owner/repo.name.git',
];
record(
  'F. github slug pattern results',
  remotes.map((remote) => ({
    remote,
    slug: slugPattern.exec(remote)
      ?.groups
      ?.slug
      ?? null,
  }))
);

console.log('\n--- summary ---');
console.log(JSON.stringify(
  results.map((entry) => entry.name),
  null,
  2
));
