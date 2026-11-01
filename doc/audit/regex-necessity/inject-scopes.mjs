// Inject the computed per-scope counts into the index issue body.
import {
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { homedir, } from 'node:os';

const AUDIT = process.env
  .REGEX_AUDIT_WORK
  ?? `${homedir()}/temp/agent/regex-audit`;

const payloads = JSON.parse(readFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  'utf8'
));
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
  groups.set(
    scope,
    (groups.get(scope) ?? 0) + 1
  );
}
const rows = [...groups.entries()].sort((
  left,
  right
) => (right[1] - left[1])
  || left[0]
    .localeCompare(right[0]));

const list = rows.map(([scope, count]) => `- \`${scope}\`: ${count}`)
  .join('\n');
const section = `**Site issues by scope**\n\n${rows.length} scopes, ${payloads.length} issues. The per-issue list, with each issue number beside its site, is a comment on this issue.\n\n${list}\n`;

const bodyPath = join(
  AUDIT,
  'index-issue-v2.md'
);
const body = readFileSync(
  bodyPath,
  'utf8'
);
const start = body.indexOf('**Site issues by scope**');
const end = body.indexOf('**Reproduction and status of this index**');
if ((start === (-1)) || (end === (-1))) throw new Error('index body markers not found');
writeFileSync(
  bodyPath,
  `${body.slice(
    0,
    start
  )}${section}\n${body.slice(end)}`,
  'utf8'
);
console.log(`injected ${rows.length} scopes, ${payloads.length} issues`);
