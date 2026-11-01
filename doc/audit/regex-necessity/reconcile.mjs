// Reconcile: correct the already-filed issues in place, then file the remaining
// sites. Resume-safe via a ledger keyed by span identity.
import {
  readFileSync,
  writeFileSync,
  appendFileSync,
  existsSync,
} from 'node:fs';
import { execFileSync, } from 'node:child_process';
import { join, } from 'node:path';

import { homedir, } from 'node:os';
import { fileURLToPath, } from 'node:url';

/**
 Repository root, derived from this file's committed location rather than a
 hardcoded path, so the harness runs from any clone.
 */
const ROOT = fileURLToPath(new URL(
  '../../../..',
  import.meta.url
))
  .replace(
  /\/$/,
  ''
  );

/**
 Scratch working directory. The harness writes mirrors and ledgers here, never
 into the repository. Override with REGEX_AUDIT_WORK.
 */
const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;

const LEDGER = join(
  AUDIT,
  'filed2.jsonl'
);
const GATE_ISSUE = '679';
const INDEX_ISSUE = '680';
const DELAY_MS = 1_200;
const OLD_LABELS = [
  'ready-for-agent',
  'enhancement',
  'difficulty:easy',
  'difficulty:medium',
  'difficulty:hard',
  'app:music-player',
  'deferred-new-packages',
  'needs-triage'
];

const payloads = JSON.parse(readFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  'utf8'
));
const previous = readFileSync(
  join(
    AUDIT,
    'filed.jsonl'
  ),
  'utf8'
)
  .split('\n')
  .filter((line) => line.trim() !== '')
  .map((line) => JSON.parse(line));

const done = new Set();
if (existsSync(LEDGER)) {
  for (const line of readFileSync(
    LEDGER,
    'utf8'
  )
    .split('\n')) {
    if (line.trim() === '') continue;
    try {
      done.add(JSON.parse(line)
        .siteId);
    } catch (error) {
      console.error(`unreadable ledger line: ${String(error)}`);
    }
  }
}

// Pair each filed issue with its corrected payload on repoPath plus line; the
// earlier run filed every site at a given line, so the group sizes match.
const byLine = new Map();
for (const record of previous) {
  const key = `${record.repoPath}:${record.line}`;
  const list = byLine.get(key) ?? [];
  list.push(record);
  byLine.set(
    key,
    list
  );
}
for (const list of byLine.values()) list.sort((
  left,
  right
) => (left.title < right.title ? -1 : 1));

const payloadByLine = new Map();
for (const payload of payloads) {
  const key = `${payload.repoPath}:${payload.line}`;
  const list = payloadByLine.get(key) ?? [];
  list.push(payload);
  payloadByLine.set(
    key,
    list
  );
}
for (const list of payloadByLine.values()) list.sort((
  left,
  right
) => (left.column - right.column) || (left.offset - right.offset));

function sleep(ms) { return new Promise((resolve) => setTimeout(
  resolve,
  ms
)); }

function bodyOf(payload) {
  const record = JSON.parse(readFileSync(
    payload.file,
    'utf8'
  ));
  return record.body
    .replaceAll(
    '#INDEX',
    `#${INDEX_ISSUE}`
    )
    .replaceAll(
    '#GATE',
    `#${GATE_ISSUE}`
    );
}

function run(
  args,
  label
) {
  for (const attempt of [
    1,
    2,
    3
  ]) {
    try {
      return execFileSync(
        'gh',
        args,
        {
          cwd: ROOT,
          encoding: 'utf8',
          timeout: 180_000,
        }
      )
        .trim();
    } catch (error) {
      const message = String(error)
        .slice(
        0,
        200
        );
      if (attempt === 3) {
        console.error(`FAILED ${label} :: ${message}`);
        return null;
      }
      console.error(`retry ${attempt} ${label} :: ${message}`);
      execFileSync(
        'sleep',
        [String(15 * attempt), ]
      );
    }
  }
  return null;
}

let edited = 0;
let created = 0;
let skipped = 0;
let failed = 0;
const bodyPath = join(
  AUDIT,
  'current-body.md'
);

for (const [key, filedList] of byLine) {
  const candidates = payloadByLine.get(key) ?? [];
  for (const [index, filed] of filedList.entries()) {
    const payload = candidates[index];
    if (payload === undefined) {
      console.error(`no payload for filed issue ${filed.url}`);
      failed += 1;
      continue;
    }
    if (done.has(payload.siteId)) {
      skipped += 1;
      continue;
    }
    const number = filed.url
      .split('/')
      .pop();
    writeFileSync(
      bodyPath,
      bodyOf(payload),
      'utf8'
    );
    const args = [
      'issue',
      'edit',
      number,
      '--body-file',
      bodyPath,
    ];
    for (const label of OLD_LABELS) args.push(
      '--remove-label',
      label
    );
    for (const label of payload.labels) args.push(
      '--add-label',
      label
    );
    const result = run(
      args,
      `edit #${number}`
    );
    if (result === null) {
      failed += 1;
      continue;
    }
    edited += 1;
    appendFileSync(
      LEDGER,
      `${JSON.stringify({
        siteId: payload.siteId,
        action: 'edited',
        number,
        url: filed.url,
        repoPath: payload.repoPath,
        line: payload.line,
        title: payload.title,
      })}\n`,
      'utf8'
    );
    console.log(`edited ${edited} #${number} ${payload.repoPath}:${payload.line}`);
    await sleep(DELAY_MS);
  }
}

const filedKeys = new Set(byLine.keys());
for (const payload of payloads) {
  if (filedKeys.has(`${payload.repoPath}:${payload.line}`)) continue;
  if (done.has(payload.siteId)) {
    skipped += 1;
    continue;
  }
  writeFileSync(
    bodyPath,
    bodyOf(payload),
    'utf8'
  );
  const record = JSON.parse(readFileSync(
    payload.file,
    'utf8'
  ));
  const args = [
    'issue',
    'create',
    '--title',
    record.title,
    '--body-file',
    bodyPath,
  ];
  for (const label of payload.labels) args.push(
    '--label',
    label
  );
  const url = run(
    args,
    `create ${payload.repoPath}:${payload.line}`
  );
  if ((url === null) || (url === '')) {
    failed += 1;
    continue;
  }
  created += 1;
  appendFileSync(
    LEDGER,
    `${JSON.stringify({
      siteId: payload.siteId,
      action: 'created',
      number: url.split('/')
        .pop(),
      url,
      repoPath: payload.repoPath,
      line: payload.line,
      title: record.title,
    })}\n`,
    'utf8'
  );
  console.log(`created ${created} ${url} ${payload.repoPath}:${payload.line}`);
  await sleep(DELAY_MS);
}

console.log(`DONE edited=${edited} created=${created} skipped=${skipped} failed=${failed}`);
