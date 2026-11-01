// Repair issue labels: `gh issue edit` applies removals after additions, so the
// reconcile pass dropped every label it also listed for removal. This pass reads
// each issue's current labels and adds only what is missing.
import {
  readFileSync,
  existsSync,
} from 'node:fs';
import { execFileSync, } from 'node:child_process';
import { join, } from 'node:path';
import { homedir, } from 'node:os';

const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;
const ROOT = process.env
  .REGEX_AUDIT_REPO
  ?? readFileSync(
    join(
      AUDIT,
      'repo-root.txt'
    ),
    'utf8'
  )
    .trim();
const DELAY_MS = 900;

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

function sleep(ms) { return new Promise((resolve) => setTimeout(
  resolve,
  ms
)); }

let repaired = 0;
let ok = 0;
let failed = 0;

for (const entry of ledger) {
  const payload = payloadBySiteId.get(entry.siteId);
  if (payload === undefined) {
    console.error(`no payload for ${entry.siteId} (#${entry.number})`);
    failed += 1;
    continue;
  }
  let current = [];
  try {
    current = JSON.parse(execFileSync(
      'gh',
      [
        'issue',
        'view',
        String(entry.number),
        '--json',
        'labels',
        '--jq',
        '[.labels[].name]'
      ],
      {
        cwd: ROOT,
        encoding: 'utf8',
        timeout: 120_000,
      }
    ));
  } catch (error) {
    console.error(`cannot read labels for #${entry.number}: ${String(error)
      .slice(
        0,
        160
      )}`);
    failed += 1;
    continue;
  }
  const missing = payload.labels
    .filter((label) => current.includes(label) === false);
  if (missing.length === 0) {
    ok += 1;
    continue;
  }
  const args = [
    'issue',
    'edit',
    String(entry.number),
  ];
  for (const label of missing) args.push(
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
        timeout: 120_000,
      }
    );
    repaired += 1;
    console.log(`repaired #${entry.number} +${missing.join(',')}`);
  } catch (error) {
    failed += 1;
    console.error(`FAILED #${entry.number}: ${String(error)
      .slice(
        0,
        160
      )}`);
  }
  await sleep(DELAY_MS);
}

console.log(`DONE repaired=${repaired} alreadyOk=${ok} failed=${failed} total=${ledger.length}`);
