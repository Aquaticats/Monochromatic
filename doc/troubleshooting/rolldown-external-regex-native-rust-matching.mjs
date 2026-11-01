// Probe for doc/troubleshooting/rolldown-external-regex-native-rust-matching.md.
//
// Verifies against the installed rolldown (native Rust binding):
// 1. an `external` array of RegExps and an equivalent `ExternalOptionFunction`
//    predicate produce byte-identical output, so the predicate offers no
//    functional gain;
// 2. the predicate form invokes the JS callback once per external check
//    (counted here), each invocation being one napi `ThreadsafeFunction`
//    round trip from a Rust tokio task to the JS main thread, while the
//    RegExp form is matched inside Rust with zero JS involvement;
// 3. the wall-time penalty of the predicate form scales with the number of
//    external checks.
//
// Run from anywhere inside the repository (rolldown resolves from the
// workspace root):
//
//   node doc/troubleshooting/rolldown-external-regex-native-rust-matching.mjs [moduleCount] [runsPerBatch]
//
// Defaults: moduleCount 400, runsPerBatch 5. The 2000-module fixture used
// for the recorded measurements takes a few seconds per build.
import {
  mkdtempSync,
  writeFileSync,
  mkdirSync
} from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const require = createRequire(import.meta.url);
const { rolldown } = await import(pathToFileURL(require.resolve('rolldown'))
  .href);

const root = mkdtempSync(join(
  tmpdir(),
  'rolldown-external-probe-'
));
const builtins = [
  'node:path',
  'node:fs',
  'node:os',
  'node:util',
  'node:events',
  'node:stream',
  'node:crypto',
  'node:url',
  'node:assert',
  'node:buffer',
];
const moduleCount = Number(process.argv[2] ?? 400);
const runsPerBatch = Number(process.argv[3] ?? 5);

mkdirSync(
  join(
    root,
    'src'
  ),
  { recursive: true }
);
for (let i = 0; i < moduleCount; i += 1) {
  const imports = Array.from(
    { length: 10 },
    (
      _,
      j
    ) => `import * as b${j} from '${builtins[(i + j) % builtins.length]}';`,
  );
  const next = (i + 1) < moduleCount
    ? `import * as v from './mod${i + 1}.ts';`
    : 'const v = { x0: 0 };';
  writeFileSync(
    join(
      root,
      'src',
      `mod${i}.ts`
    ),
    [
      ...imports,
      next,
      `export const x${i} = ${i} + Object.keys(v).length;`,
      'void b0;'
    ].join('\n'),
  );
}
writeFileSync(
  join(
    root,
    'src',
    'main.ts'
  ),
  [
    "import { app } from 'electron';",
    "import { ipcMain } from 'electron/main';",
    "import * as p from 'node:path';",
    "import { x0 } from './mod0.ts';",
    'export default [app, ipcMain, p, x0];',
    '',
  ].join('\n'),
);

// The two forms under comparison. The RegExp array is what the repository's
// rolldown configs use; the predicate is the audit-proposed replacement with
// the same matching semantics.
const arrayForm = [
  /^node:/u,
  /^electron(\/|$)/u
];
let callbackInvocations = 0;
function fnForm(id) {
  callbackInvocations += 1;
  return id.startsWith('node:') || (id === 'electron')
    || id.startsWith('electron/');
}

async function build(external) {
  const bundle = await rolldown({
    input: [join(
      root,
      'src/main.ts'
    )],
    platform: 'node',
    external,
    cwd: root,
  });
  const out = await bundle.generate({ format: 'es' });
  await bundle.close();
  return out.output
    .map((chunk) => chunk.code)
    .join('\n');
}

// Equivalence: identical output, externals preserved.
const codeArray = await build(arrayForm);
const invocationsBeforeBench = callbackInvocations;
const codeFn = await build(fnForm);
console.log(`fixture modules: ${moduleCount}`);
console.log(`callback invocations for one predicate-form build: ${callbackInvocations - invocationsBeforeBench}`);
console.log(`outputs identical: ${codeArray === codeFn}`);
console.log(`electron kept external: ${codeArray.includes("from 'electron'") || codeArray.includes('from "electron"')}`);
console.log(`electron/main kept external: ${codeArray.includes('electron/main')}`);

// Timing: two interleaved batches per form so run-to-run drift shows up
// inside each form's spread rather than between the forms.
for (const [name, form] of [
  [
    'array(RegExp)',
    arrayForm
  ],
  [
    'fn(predicate)',
    fnForm
  ],
  [
    'array(RegExp)',
    arrayForm
  ],
  [
    'fn(predicate)',
    fnForm
  ],
]) {
  const times = [];
  for (let run = 0; run < runsPerBatch; run += 1) {
    const start = performance.now();
    await build(form);
    times.push(performance.now() - start);
  }
  times.sort((
    a,
    b
  ) => a - b);
  const median = times[times.length >> 1];
  console.log(
    `${name}: median=${median.toFixed(1)}ms min=${times[0]
      .toFixed(1)}ms max=${times.at(-1)
        .toFixed(1)}ms`,
  );
}
console.log(`total predicate callback invocations: ${callbackInvocations}`);
