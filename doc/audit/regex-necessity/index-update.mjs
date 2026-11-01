// Build the corrected index issue body and the grouped issue list from the ledger,
// so every number in them is computed rather than typed.
import {
  readFileSync,
  writeFileSync,
  existsSync,
} from 'node:fs';
import { join, } from 'node:path';
import { homedir, } from 'node:os';

const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;
const GATE_ISSUE = 679;
const INDEX_ISSUE = 680;

const findings = JSON.parse(readFileSync(
  join(
    AUDIT,
    'to-file2.json'
  ),
  'utf8'
));
const payloads = JSON.parse(readFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  'utf8'
));
const ledger = existsSync(join(
  AUDIT,
  'filed2.jsonl'
))
  ? readFileSync(
    join(
      AUDIT,
      'filed2.jsonl'
    ),
    'utf8'
  )
    .split('\n')
    .filter((line) => line.trim() !== '')
    .map((line) => JSON.parse(line))
  : [];

const numberBySiteId = new Map(ledger.map((entry) => [
  entry.siteId,
  Number(entry.number),
]));
const payloadBySiteId = new Map(payloads.map((payload) => [
  payload.siteId,
  payload,
]));
/**
 Scope from a generated title. The scope itself can contain parentheses
 (`docs(handover)`), so the cut runs to the last `)` before the subject.
 */
function scopeOf(payload) {
  const start = payload.title
    .indexOf('(')
    + 1;
  const end = payload.title
    .indexOf('): drop the unnecessary regex');
  return end === (-1) ? payload.title
    .slice(start) : payload.title
      .slice(
      start,
      end
      );
}

const groups = new Map();
for (const payload of payloads) {
  const scope = scopeOf(payload);
  const list = groups.get(scope) ?? [];
  list.push(payload);
  groups.set(
    scope,
    list
  );
}

function counted(predicate) { return findings.filter(predicate)
  .length; }
const stats = {
  total: findings.length,
  active: counted((finding) => finding.exclusion === 'in-scope'),
  paused: counted((finding) => finding.exclusion === 'paused-package'),
  justified: counted((finding) => finding.covered),
  surfaces: findings.reduce(
    (
      accumulator,
      finding
    ) => {
      const key = finding.surface ?? 'js-ts';
      accumulator[key] = (accumulator[key] ?? 0) + 1;
      return accumulator;
    },
    {}
  ),
  shapes: findings.reduce(
    (
      accumulator,
      finding
    ) => {
      accumulator[finding.shape] = (accumulator[finding.shape] ?? 0) + 1;
      return accumulator;
    },
    {}
  ),
  filed: ledger.length,
  missing: payloads.filter((payload) => ! numberBySiteId.has(payload.siteId))
    .length,
};

// Issue list comment, grouped by scope, each line linking the issue to its site.
const closed = ledger.filter((entry) => ! payloadBySiteId.has(entry.siteId));
const listLines = [
  '# Filed site issues',
  '',
  `One issue per unnecessary regex site, grouped by package scope: ${payloads.length} live issues.`,
  '',
  `A further ${closed.length} issues were filed in the first round and then closed on provenance (${closed.map((entry) => `#${entry.number} \`${entry.repoPath}:${entry.line}\``)
    .join(', ')}): generated output whose template is filed separately, and user-provided reference material. The evidence is in each closing comment and in \`doc/audit/regex-necessity.md\`.`,
  '',
];
for (const [scope, list] of [...groups.entries()].sort((
  left,
  right
) => right.length - left.length)) {
  listLines.push(
    `## ${scope} (${list.length})`,
    ''
  );
  for (const payload of list.sort((
    left,
    right
  ) => left.repoPath
    .localeCompare(right.repoPath)
    || (left.line - right.line))) {
    const number = numberBySiteId.get(payload.siteId);
    const marker = number === undefined ? '(not filed)' : `#${number}`;
    listLines.push(`- ${marker} \`${payload.repoPath}:${payload.line}\` (${payload.shape})`);
  }
  listLines.push('');
}
writeFileSync(
  join(
    AUDIT,
    'index-comment.md'
  ),
  listLines.join('\n'),
  'utf8'
);

console.log(JSON.stringify(
  stats,
  null,
  2
));
console.log(
  'scopes:',
  groups.size
);
writeFileSync(
  join(
    AUDIT,
    'index-stats.json'
  ),
  JSON.stringify(
    {
      stats,
      scopes: [...groups.entries()].map(([scope, list]) => ({
        scope,
        count: list.length,
      })),
    },
    null,
    2
  ),
  'utf8'
);
