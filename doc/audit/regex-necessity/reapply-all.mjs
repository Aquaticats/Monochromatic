// Deterministic re-apply: every live issue gets its current payload title and body,
// so the tracker matches the harness output exactly after any correction round.
import {
  readFileSync,
  writeFileSync,
  appendFileSync,
  existsSync,
} from 'node:fs';
import { execFileSync, } from 'node:child_process';
import { join, } from 'node:path';
import { homedir, } from 'node:os';

const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;
const ROOT = readFileSync(
  join(
    AUDIT,
    'repo-root.txt'
  ),
  'utf8'
)
  .trim();
const GATE_ISSUE = '679';
const INDEX_ISSUE = '680';
const OUT_LEDGER = join(
  AUDIT,
  'filed3.jsonl'
);
const DELAY_MS = 1_000;

const payloads = JSON.parse(readFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  'utf8'
));
const ledger = readFileSync(
  join(
    AUDIT,
    'filed2.jsonl'
  ),
  'utf8'
)
  .split('\n')
  .filter((line) => line.trim() !== '')
  .map((line) => JSON.parse(line));
const numberBySiteId = new Map(ledger.map((entry) => [
  entry.siteId,
  Number(entry.number),
]));

const done = new Set();
if (existsSync(OUT_LEDGER)) {
  for (const line of readFileSync(
    OUT_LEDGER,
    'utf8'
  )
    .split('\n')) {
    if (line.trim() !== '') done.add(JSON.parse(line)
      .siteId);
  }
}

function sleep(ms) { return new Promise((resolve) => setTimeout(
  resolve,
  ms
)); }
const bodyPath = join(
  AUDIT,
  'current-body.md'
);

let applied = 0;
let skipped = 0;
let failed = 0;
let orphaned = 0;

for (const payload of payloads) {
  const number = numberBySiteId.get(payload.siteId);
  if (number === undefined) {
    console.error(`no issue number for ${payload.siteId}`);
    orphaned += 1;
    continue;
  }
  if (done.has(payload.siteId)) {
    skipped += 1;
    continue;
  }
  const record = JSON.parse(readFileSync(
    payload.file,
    'utf8'
  ));
  const body = record.body
    .replaceAll(
    '#INDEX',
    `#${INDEX_ISSUE}`
    )
    .replaceAll(
    '#GATE',
    `#${GATE_ISSUE}`
    );
  writeFileSync(
    bodyPath,
    body,
    'utf8'
  );
  const args = [
    'issue',
    'edit',
    String(number),
    '--title',
    record.title,
    '--body-file',
    bodyPath,
  ];
  for (const label of payload.labels) args.push(
    '--add-label',
    label
  );
  try {
    execFileSync(
      'gh',
      args,
      {
        cwd: ROOT,
        encoding: 'utf8',
        timeout: 180_000,
      }
    );
    applied += 1;
    appendFileSync(
      OUT_LEDGER,
      `${JSON.stringify({
        siteId: payload.siteId,
        number,
        title: record.title,
        url: `https://github.com/Aquaticat/Monochromatic/issues/${number}`,
        repoPath: payload.repoPath,
        line: payload.line,
        shape: payload.shape,
        exclusion: payload.exclusion,
        covered: payload.covered,
      })}\n`,
      'utf8'
    );
    if ((applied % 25) === 0) console.log(`applied ${applied} (last #${number})`);
  } catch (error) {
    failed += 1;
    console.error(`FAILED #${number} ${payload.repoPath}:${payload.line} :: ${String(error)
      .slice(
        0,
        200
      )}`);
  }
  await sleep(DELAY_MS);
}

console.log(`DONE applied=${applied} skipped=${skipped} failed=${failed} orphaned=${orphaned}`);
