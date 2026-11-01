// Repair titles and bodies for the issues affected by post-filing corrections:
// title disambiguation for sites sharing a file and line, doc scope naming, and the
// HTML sites whose own regex literal was extracted after they were filed.
import {
  readFileSync,
  writeFileSync,
  appendFileSync,
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
const DELAY_MS = 1_000;
const DONE_FILE = join(
  AUDIT,
  'repaired-titles.jsonl'
);

const payloads = JSON.parse(readFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  'utf8'
));
const payloadBySiteId = new Map(payloads.map((payload) => [
  payload.siteId,
  payload,
]));
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

const already = new Set();
try {
  for (const line of readFileSync(
    DONE_FILE,
    'utf8'
  )
    .split('\n')) {
    if (line.trim() !== '') already.add(JSON.parse(line)
      .siteId);
  }
} catch (error) {
  console.error(`no prior repair ledger: ${String(error)
    .slice(
      0,
      100
    )}`);
}

function sleep(ms) { return new Promise((resolve) => setTimeout(
  resolve,
  ms
)); }
const bodyPath = join(
  AUDIT,
  'current-body.md'
);

let repaired = 0;
let skipped = 0;
let failed = 0;

for (const entry of ledger) {
  const payload = payloadBySiteId.get(entry.siteId);
  if (payload === undefined) {
    // The site was excluded after filing (provenance), so its issue is closed
    // separately rather than retitled.
    skipped += 1;
    continue;
  }
  const needsBody = payload.repoPath
    .endsWith('.html');
  const needsTitle = payload.title !== entry.title;
  if (((needsBody || needsTitle) === false) || already.has(entry.siteId)) {
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
    String(entry.number),
    '--title',
    payload.title,
    '--body-file',
    bodyPath,
  ];
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
    repaired += 1;
    appendFileSync(
      DONE_FILE,
      `${JSON.stringify({
        siteId: entry.siteId,
        number: entry.number,
        title: payload.title,
      })}\n`,
      'utf8'
    );
    console.log(`repaired #${entry.number} ${needsTitle ? 'title ' : ''}${needsBody ? 'body' : ''} ${payload.repoPath}:${payload.line}`);
  } catch (error) {
    failed += 1;
    console.error(`FAILED #${entry.number}: ${String(error)
      .slice(
        0,
        200
      )}`);
  }
  await sleep(DELAY_MS);
}

console.log(`DONE repaired=${repaired} skipped=${skipped} failed=${failed}`);
