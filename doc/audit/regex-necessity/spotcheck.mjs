// Boundary check: read random filed issues back from GitHub and confirm the body's
// file, line, and pattern match the repository source at that location.
import { readFileSync, } from 'node:fs';
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
    'filed3.jsonl'
  ),
  'utf8'
)
  .split('\n')
  .filter((line) => line.trim() !== '')
  .map((line) => JSON.parse(line));
const payloadBySiteId = new Map(payloads.map((payload) => [
  payload.siteId,
  payload,
]));

// Deterministic spread across the batches rather than a random draw.
const picks = [
  0,
  37,
  74,
  111,
  148,
  185,
  222,
  259,
  ledger.length - 1,
]
  .filter((index) => index < ledger.length);

let checked = 0;
let mismatches = 0;

for (const index of picks) {
  const entry = ledger[index];
  if (entry === undefined) continue;
  const payload = payloadBySiteId.get(entry.siteId);
  if (payload === undefined) {
    console.log(`#${entry.number} has no payload`);
    mismatches += 1;
    continue;
  }
  const issue = JSON.parse(execFileSync(
    'gh',
    [
      'issue',
      'view',
      String(entry.number),
      '--json',
      'number,title,state,body,labels'
    ],
    {
      cwd: ROOT,
      encoding: 'utf8',
      timeout: 120_000,
    }
  ));
  // 1. The body must name the site's file and line.
  const namesSite = issue.body
    .includes(`${payload.repoPath}:${payload.line}`);
  // 2. The body must quote the pattern the harness extracted.
  const patternCore = payload.pattern === undefined ? '' : payload.pattern
    .replaceAll(
    '\n',
    ' '
    )
    .slice(
    0,
    40
    );
  const quotesPattern = (patternCore === '')
    || issue.body
      .includes(patternCore.slice(
        0,
        24
      ));
  // 3. The repository source at that line must contain the pattern's first literal.
  const sourceLines = readFileSync(
    join(
      ROOT,
      payload.repoPath
    ),
    'utf8'
  )
    .split('\n');
  const sourceLine = sourceLines[payload.line - 1] ?? '';
  const literalHint = /[A-Za-z0-9_]{3,}/.exec(patternCore)?.[0] ?? '';
  const sourceAgrees = (literalHint === '')
    || sourceLine.includes(literalHint)
    || issue.body
      .includes(sourceLine.trim()
        .slice(
        0,
        60
        ));
  // 4. Title and state.
  const titleOk = issue.title === payload.title;
  const labelsOk = issue.labels
    .every((label) => payload.labels
      .includes(label.name))
    && payload.labels
      .every((label) => issue.labels
        .map((entry2) => entry2.name)
        .includes(label));
  const ok = namesSite && quotesPattern
    && sourceAgrees
    && titleOk
    && labelsOk
    && (issue.state === 'OPEN');
  checked += 1;
  if (ok === false) mismatches += 1;
  console.log(`#${issue.number} ${ok ? 'OK' : 'MISMATCH'} namesSite=${namesSite} quotesPattern=${quotesPattern} sourceAgrees=${sourceAgrees} titleOk=${titleOk} labelsOk=${labelsOk} state=${issue.state}`);
  if (ok === false) {
    console.log(`   title: ${issue.title
      .slice(
        0,
        100
      )}`);
    console.log(`   expected: ${payload.title
      .slice(
        0,
        100
      )}`);
    console.log(`   source line: ${sourceLine.trim()
      .slice(
        0,
        100
      )}`);
  }
  execFileSync(
    'sleep',
    ['1', ]
  );
}

console.log(JSON.stringify(
  {
    checked,
    mismatches,
    ledgerEntries: ledger.length,
    payloads: payloads.length,
  },
  null,
  2
));
