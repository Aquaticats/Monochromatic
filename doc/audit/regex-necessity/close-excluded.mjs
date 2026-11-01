// Close the issues filed for sites later excluded on provenance: generated output
// and user-provided reference material.
import {
  readFileSync,
  writeFileSync,
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
const payloads = JSON.parse(readFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  'utf8'
));
const liveSiteIds = new Set(payloads.map((payload) => payload.siteId));

const templateIssues = ledger
  .filter((entry) => entry.repoPath
    .endsWith('template-editor.template.html'))
  .map((entry) => `#${entry.number}`);

const GENERATED_COMMENT = `This site is in generated output, so there is nothing to change here.

\`package/music-player/design/template-editor.mjs:121\` writes \`package/music-player/design/questions/template-editor.html\` (18.6 MB) from \`template-editor.template.html\`. The regex sites in the template are filed separately (${templateIssues.join(', ')}), and fixing those removes these copies on the next generator run.

Closing as a duplicate of the template issues. The exclusion and its evidence are recorded in \`doc/audit/regex-necessity.md\` and in the harness's provenance exclusions (\`doc/audit/regex-necessity/supplement.mjs\`).`;

const UPLOADS_COMMENT = `This file is user-provided reference material rather than first-party code.

\`package/music-player/design/HANDOFF.md:3208\` describes \`uploads/\` as "the user's own mockup, screenshots and reference images". The audit population is first-party code the repo owns and can refactor, so this site is excluded.

Closing as wontfix. The exclusion and its evidence are recorded in \`doc/audit/regex-necessity.md\` and in the harness's provenance exclusions (\`doc/audit/regex-necessity/supplement.mjs\`).`;

function sleep(ms) { return new Promise((resolve) => setTimeout(
  resolve,
  ms
)); }
const bodyPath = join(
  AUDIT,
  'close-comment.md'
);

let closed = 0;
let failed = 0;

for (const entry of ledger) {
  if (liveSiteIds.has(entry.siteId)) continue;
  const isUploads = entry.repoPath
    .includes('/uploads/');
  const comment = isUploads ? UPLOADS_COMMENT : GENERATED_COMMENT;
  const label = isUploads ? 'wontfix' : 'duplicate';
  writeFileSync(
    bodyPath,
    comment,
    'utf8'
  );
  try {
    execFileSync(
      'gh',
      [
        'issue',
        'comment',
        String(entry.number),
        '--body-file',
        bodyPath,
      ],
      {
        cwd: ROOT,
        encoding: 'utf8',
        timeout: 120_000,
      }
    );
    execFileSync(
      'gh',
      [
        'issue',
        'edit',
        String(entry.number),
        '--add-label',
        label,
      ],
      {
        cwd: ROOT,
        encoding: 'utf8',
        timeout: 120_000,
      }
    );
    execFileSync(
      'gh',
      [
        'issue',
        'close',
        String(entry.number),
        '--reason',
        isUploads ? 'not planned' : 'duplicate',
      ],
      {
        cwd: ROOT,
        encoding: 'utf8',
        timeout: 120_000,
      }
    );
    closed += 1;
    console.log(`closed #${entry.number} (${label}) ${entry.repoPath}:${entry.line}`);
  } catch (error) {
    failed += 1;
    console.error(`FAILED #${entry.number}: ${String(error)
      .slice(
        0,
        200
      )}`);
  }
  await sleep(900);
}

console.log(`DONE closed=${closed} failed=${failed}`);
