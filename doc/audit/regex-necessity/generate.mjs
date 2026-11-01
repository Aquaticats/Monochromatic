// Generate one issue payload per unnecessary-regex site, plus the gate-coverage issue.
import {
  readFileSync,
  writeFileSync,
  mkdirSync,
  existsSync,
} from 'node:fs';
import {
  join,
  dirname,
} from 'node:path';
import {
  rebuttalFor,
  SAFETY_ONLY_FALLBACK,
} from './rebuttals.mjs';

const ROOT = '/var/home/user/Monochromatic';
const AUDIT = '/home/user/temp/agent/regex-audit';
const OUT = join(
  AUDIT,
  'issues'
);

const findings = JSON.parse(readFileSync(
  join(
    AUDIT,
    'to-file2.json'
  ),
  'utf8'
));

mkdirSync(
  OUT,
  { recursive: true, }
);

function packageDirOf(repoPath) {
  let dir = dirname(join(
    ROOT,
    repoPath
  ));
  while (dir.startsWith(ROOT)) {
    if (existsSync(join(
      dir,
      'package.json'
    ))) return dir.slice(ROOT.length + 1);
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}

const COMPLEXITY = {
  'prefix': 'One prefix comparison, O(k) in the pattern length, the same order as the anchored pattern.',
  'suffix': 'One suffix comparison, O(k) in the pattern length. Same order as the anchored pattern, without the compilation.',
  'substring': 'One `indexOf` pass, O(n) worst case, the same order as the unanchored pattern.',
  'exact-equality': 'One equality comparison, O(k) in the literal length, the same order as the anchored pattern.',
  'replace-all-literal': 'The string form of `replaceAll` (or `split`/`join`) is a single linear pass with no engine, same order as the `/g` pattern.',
  'word-boundary-substring': 'Linear: each candidate occurrence resumes after the previous one, and the flanking check is O(1) per candidate.',
  'class-run-scan': 'One linear pass over the input, character by character, with no backtracking state to reason about.',
  'fixed-length-class-run': 'A length compare plus one linear character walk: O(n) in the input, and the repetition count becomes an explicit number in code.',
  'literal-class-sequence': 'One linear pass that consumes each literal and each character run in order: O(n), no backtracking state, and each step is visible in the code.',
  'capture': 'One linear pass with a moving cursor; each `indexOf` resumes where the previous slice ended, so the total is O(n) rather than a rescan per extracted field.',
  'alternation': 'One pass matching the candidates in order, or O(candidates x n) with `some(includes)`. For a fixed small candidate list that is the same linear order as the alternation.',
  'dynamic-construction': 'One linear scan over text the caller already holds, the same order as the pattern match. This audit measured no compilation-cost win (1000 iterations of `new RegExp(source)` plus a test took 32.6 ms against 32.3 ms for a literal, inside run-to-run spread), so the case here is necessity, not speed.',
  'lookbehind-scan': 'One forward pass carrying the escape-parity state, so O(n) overall. A backward walk per candidate would not be O(1) when the preceding escape run is unbounded, which is why the replacement tracks parity forward.',
  'scanner': 'One linear pass with explicit branches in place of engine dispatch.',
};

/**
 Render text as a Markdown code span, widening the delimiter so a backtick inside
 the pattern cannot terminate the span early.
 */
function codeSpan(text) {
  let longest = 0;
  let current = 0;
  for (const char of text) {
    if (char === '`') {
      current += 1;
      longest = Math.max(
        longest,
        current
      );
      continue;
    }
    current = 0;
  }
  const fence = '`'.repeat(longest + 1);
  const pad = text.startsWith('`') || text.endsWith('`') ? ' ' : '';
  return `${fence}${pad}${text}${pad}${fence}`;
}

function languageOf(finding) {
  const surface = finding.surface ?? 'js-ts';
  if (surface === 'kotlin') return 'kotlin';
  if (surface === 'yaml-inline') return 'yaml';
  if (surface === 'html-script') return 'html';
  const {repoPath} = finding;
  if (repoPath.endsWith('.ts') || repoPath.endsWith('.tsx')) return 'ts';
  return 'js';
}

const ROLLDOWN_COORDINATION = [
  'Anchored protocol prefix test for node builtins',
  'Anchored literal name test keeping the electron runtime',
];

const TITLE_LIMIT = 140;
const TITLE_PREFIX = 'refactor(%s): drop the unnecessary regex at ';

/**
 Longest trailing path suffix that keeps the title within the readable width.
 `index.ts` alone is ambiguous in the type-encoded `module/es` tree, so the
 suffix widens toward the repo root until the next segment would not fit.
 */
function pathSuffixFor(
  finding,
  budget
) {
  const segments = finding.repoPath
    .split('/');
  let chosen = null;
  for (let count = 1; count <= segments.length; count += 1) {
    const candidate = `${segments.slice(-count)
      .join('/')}:${finding.line}`;
    if (candidate.length > budget) break;
    chosen = candidate;
  }
  return chosen ?? `${segments.at(-1)}:${finding.line}`;
}

/**
 Short readable form of a pattern, for disambiguating titles when one line carries
 several distinct regex sites (chained calls report one diagnostic each at the same
 start position).
 */
function patternTag(finding) {
  const cleaned = finding.pattern
    .replaceAll(
    '`',
    "'"
    )
    .replaceAll(
    '\n',
    ' '
    )
    .trim();
  return cleaned.length <= 26 ? cleaned : `${cleaned.slice(
    0,
    26
  )}...`;
}

function titleFor(
  finding,
  group
) {
  const prefix = TITLE_PREFIX.replace(
    '%s',
    finding.scope
  );
  const suffix = pathSuffixFor(
    finding,
    TITLE_LIMIT - prefix.length
  );
  const tag = group === null ? '' : ` [${patternTag(finding)}]`;
  const budget = TITLE_LIMIT - tag.length;
  if (`${prefix}${suffix}`.length <= budget) return `${prefix}${suffix}${tag}`;
  // The path suffix has to shrink to make room for the tag.
  const base = `${finding.repoPath
    .slice(finding.repoPath
      .lastIndexOf('/')
      + 1)}:${finding.line}`;
  return `${prefix}${base}${tag}`;
}

function shapeSentence(finding) {
  const map = {
    'prefix': 'the pattern is a literal prefix test, which `startsWith` answers directly',
    'suffix': 'the pattern is a literal suffix test, which `endsWith` answers directly',
    'substring': 'the pattern is a literal substring test, which `includes` answers directly',
    'exact-equality': 'the pattern is an anchored literal, which an equality compare answers directly',
    'replace-all-literal': 'the pattern is a literal global replacement, which the string form of `replaceAll` answers directly',
    'word-boundary-substring': 'the pattern is a literal with word boundaries, which an `indexOf` walk plus a flanking-character check answers directly',
    'class-run-scan': 'the pattern is a literal-and-character-run shape, which one linear character walk answers directly',
    'fixed-length-class-run': 'the pattern is a fixed-count character run, which a length compare plus a character walk answers directly',
    'literal-class-sequence': 'the pattern is an ordered sequence of literals and character runs, which one linear walk answers directly',
    'capture': 'the pattern extracts fields between literal delimiters, which a cursor of `indexOf` and `slice` calls answers directly',
    'alternation': 'the pattern is an alternation over a fixed candidate set, which an array of literal tests answers directly',
    'dynamic-construction': 'the pattern is compiled at runtime from a string the caller already holds, so the condition can be tested without compiling anything',
    'lookbehind-scan': 'the pattern encodes a backward condition with a lookbehind, which a backward character walk from each candidate answers directly',
    'scanner': 'the pattern is a small token grammar, which a single-pass scanner answers directly',
  };
  return map[finding.shape] ?? 'the same condition is expressible with string APIs';
}

/**
 Sites whose gate status was measured individually rather than inferred.
 */
const GATE_OVERRIDES = new Map([
  [
    'package/git-policy/cli/bin/build-git-test-image.mjs:61',
    'Unlike the other unjustified sites, this one IS reached by the gate and is currently red: `mise run //package/git-policy/cli:lint:oxlint` reports 75 errors and 65 warnings (measured 2026-10-12), and exactly one of them is this `no-restricted-syntax(no-regex)` error. The task then fails for an unrelated reason (a tsgo panic inside `plugin-prefer-readonly-parameter-type`), so the package lint has not been green. This contradicts the "sweep complete ... green for every non-paused package" status recorded in `doc/handover/no-regex.md`.'
  ],
]);

/**
 Why the no-regex gate never reached this site, stated with the deciding evidence.
 */
function gateNote(
  finding,
  taskPrefix,
  gateIssue
) {
  const override = GATE_OVERRIDES.get(`${finding.repoPath}:${finding.line}`);
  if (override !== undefined) return `${override} The gate-coverage half of this finding is tracked in #${gateIssue}.`;
  const surface = finding.surface ?? 'js-ts';
  if (surface === 'kotlin') {
    return `This site carries no disable justification because no gate can ask for one: \`no-restricted-syntax/no-regex\` is a JavaScript rule, and \`package/rust-linter-plugin\` has no regex rule, so Kotlin regex use is unpolicied repo-wide. The five Kotlin sites this audit found are all in \`package/music-player/android-app\`. The gate-coverage half of this finding is tracked in #${gateIssue}.`;
  }
  if (surface === 'html-script') {
    return `This site carries no disable justification because the oxlint gate does not lint HTML: the file set is JS and TS, so a \`<script>\` block inside a tracked \`.html\` file is outside every rule, not only \`no-regex\`. The gate-coverage half of this finding is tracked in #${gateIssue}.`;
  }
  if (surface === 'yaml-inline') {
    return `This site carries no disable justification because inline script strings inside a compose file are outside every lint gate. The gate-coverage half of this finding is tracked in #${gateIssue}.`;
  }
  const tail = ` The gate-coverage half of this finding is tracked in #${gateIssue}.`;
  if (finding.repoPath
    .startsWith('doc/')) {
    return `This site carries no disable justification because the oxlint gate never reaches it: no package task covers \`doc/\`, since the lint fanout visits only \`//package/\` tasks (recorded in \`monochromatic-lint.config.jsonc\`, block \`old-gate-scope\`).${tail}`;
  }
  if (finding.repoPath
    .endsWith('.js')
    || finding.repoPath
      .endsWith('.cjs')) {
    return `This site carries no disable justification because the oxlint gate never reaches it: \`package/config/oxlint/src/config-base.ts:63\` lists \`**/*.js\` and \`**/*.cjs\` in \`ignorePatterns\`, so the file is excluded from every lint run.${tail}`;
  }
  return `This site carries no disable justification because the oxlint gate never reaches it: \`mise tasks ls --all\` lists no \`${taskPrefix ?? '//<none>'}:lint:oxlint\` task for the owning package (positive control: \`//package/pi-plugin/auto-mode:lint:oxlint\` and \`//package/ssg/aquati.cat:lint:oxlint\` both exist, so the probe can show a task when one is defined).${tail}`;
}

const TASK_LIST = readFileSync(
  join(
    AUDIT,
    'tasks.txt'
  ),
  'utf8'
);

const JS_DETECTOR = "the repo's own `no-restricted-syntax/no-regex` rule (`package/oxlint-plugin/no-restricted-syntax/src/rule/no-regex.ts`), run over a mirror tree whose suppression directives were neutralised at equal byte length, so justified sites are enumerated too";

/**
 Whether a package actually defines a task, checked against a full
 `mise tasks ls --all` dump rather than assumed from the task templates.
 The match is anchored on the task-name column, because a substring test reports
 `test` as present for a package that only defines `test:unit`.
 */
function taskExists(
  packageDir,
  task
) {
  if (packageDir === null) return false;
  const wanted = `//${packageDir}:${task}`;
  return TASK_LIST.split('\n')
    .some((line) => {
      const name = /^[^\s]+/.exec(line);
      return (name !== null) && (name[0] === wanted);
    });
}

function verificationSteps(
  finding,
  gateIssue
) {
  const dir = finding.packageDir ?? null;
  const steps = [];
  if (dir === null) {
    steps.push('- [ ] This file is a repo-level script with no package task: run it against its documented input and compare the output with the pre-change run.');
    return steps;
  }
  steps.push(taskExists(
    dir,
    'lint:types'
  )
    ? `- [ ] \`mise run //${dir}:lint:types\` passes.`
    : `- [ ] \`//${dir}\` defines no \`lint:types\` task (checked against \`mise tasks ls --all\`), so type-check through whatever task the package does define, or record why none applies.`);
  if ((finding.surface ?? 'js-ts') === 'kotlin') {
    steps.push(
      taskExists(
        dir,
        'lint:detekt'
      )
        ? `- [ ] \`mise run //${dir}:lint:detekt\` passes (this is the Kotlin gate that does exist for this package).`
        : `- [ ] \`//${dir}\` defines no \`lint:detekt\` task either, so record how the change is verified.`,
      `- [ ] The Android module's own test task runs the equivalence fixtures, or the issue records why it cannot.`
    );
    return steps;
  }
  const testTask = [
    'test:unit',
    'test',
    'test:container'
  ].find((task) => taskExists(
    dir,
    task
  ));
  steps.push(
    testTask === undefined
      ? `- [ ] \`//${dir}\` defines no test task (checked against \`mise tasks ls --all\`): put the equivalence fixtures where they can actually run, or record in this issue why the site stays untested.`
      : `- [ ] \`mise run //${dir}:${testTask}\` passes.`,
    taskExists(
      dir,
      'lint:oxlint'
    )
      ? `- [ ] \`mise run //${dir}:lint:oxlint\` is clean with the regex gone${finding.covered ? ' and the disable comment removed' : ''}.`
      : `- [ ] \`//${dir}\` defines no \`lint:oxlint\` task, so no gate will confirm the regex is gone: the coverage fix is #${gateIssue}, and until it lands, verify by re-running the audit enumeration on this file.`
  );
  return steps;
}

function bodyFor(
  finding,
  {
    indexIssue,
    gateIssue,
  }
) {
  const packageDir = finding.packageDir ?? null;
  const taskPrefix = packageDir === null ? null : `//${packageDir}`;
  const lines = [];
  lines.push(
    '> **Generated by AI during the repo-wide regex-necessity audit.** Method, counts, exclusions and measurements: `doc/audit/regex-necessity.md`.',
    '',
    '## Agent Brief',
    '',
    '**Category:** enhancement',
    '',
    `**Summary:** \`${finding.repoPath}:${finding.line}\` uses ${codeSpan(finding.pattern)} in \`${finding.symbol}\`, and ${shapeSentence(finding)}. Under the audit criterion, a regex is unnecessary when the same feature is implementable without regex at similar or better time complexity, so this site qualifies.`,
    '',
    '**Site**',
    '',
    `- File: \`${finding.repoPath}:${finding.line}\` (column ${finding.column})`,
    `- Enclosing declaration: \`${finding.symbol}\``,
    `- Package scope: \`${finding.scope}\`${packageDir === null ? ' (repo-level script, no package task prefix)' : ` (\`${packageDir}\`)`}`
  );
  const surface = finding.surface ?? 'js-ts';
  lines.push(
    `- Reported form: ${surface === 'kotlin' ? 'a Kotlin `Regex(...)` construction' : surface === 'yaml-inline' ? 'an inline regex inside a script string' : finding.kind === 'literal' ? 'a regex literal' : finding.kind === 'constructor' ? 'a `RegExp` constructor call' : `a string method taking an inline regex (\`${finding.message
      .split(' with ')[0]}\`)`}`,
    `- Detector: ${finding.detector ?? JS_DETECTOR}`,
    '',
    `\`\`\`${languageOf(finding)}`,
    `// ${finding.repoPath}:${finding.line}`,
    finding.sourceLine === '' ? finding.spanText : finding.sourceLine,
    '```',
    '',
    '**What the pattern matches**',
    '',
    finding.grammar === '' ? 'A dynamically constructed pattern; the source is assembled at runtime from values the caller already holds.' : finding.grammar,
    ''
  );
  if (finding.splitNote !== undefined) {
    lines.push(
      '**Why this site is separate from its neighbours**',
      '',
      finding.splitNote,
      ''
    );
  }
  if (finding.note !== undefined) {
    lines.push(
      `**Role in the file:** ${finding.note}`,
      ''
    );
  }
  lines.push(
    '**Why it is unnecessary**',
    '',
    `Replacement shape: ${finding.sketch
      .replace(
        /\.$/,
        ''
      )}.`,
    '',
    `Complexity: ${COMPLEXITY[finding.shape] ?? COMPLEXITY.scanner}`
  );
  if (finding.covered === false) {
    lines.push(
      '',
      gateNote(
        finding,
        taskPrefix,
        gateIssue
      )
    );
  }
  lines.push('');
  if (finding.covered) {
    const rebuttal = rebuttalFor(finding.justification) ?? SAFETY_ONLY_FALLBACK;
    lines.push(
      '**The recorded justification does not establish necessity**',
      '',
      `The site carries a \`${finding.directiveKind}\` disable for \`no-restricted-syntax/no-regex\`:`,
      '',
      `> ${(finding.justification ?? '').replaceAll(
      '\n',
      ' '
    )}`,
      '',
      rebuttal,
      ''
    );
  }
  if (finding.exclusion === 'paused-package') {
    lines.push(
      '**Paused package**',
      '',
      'This file is in `package-paused/`, which `monochromatic-lint.config.jsonc` and `package/config/oxlint/src/config-base.ts` both exclude from the lint scope, and whose README records that development is paused pending the no-regex sweep in `doc/handover/no-regex.md`. Resuming the package is a separate decision from this refactor; the site is filed so the deferral is visible per site rather than only in a README paragraph.',
      ''
    );
  }
  if (ROLLDOWN_COORDINATION.some((marker) => (finding.justification ?? '').includes(marker))
    || (finding.repoPath === 'package/config/rolldown/src/package-externals.ts')) {
    lines.push(
      '**This site cannot be fixed alone**',
      '',
      "Rolldown's `external` accepts a predicate as the whole option value (`ExternalOption = StringOrRegExp | StringOrRegExp[] | ExternalOptionFunction`), not as an entry inside the string-or-RegExp array. Migrating these patterns therefore means replacing the array with one shared predicate, which touches every config listing `/^node:/u` or an electron name pattern, plus `anchoredExternal` and `escapePackageNameForRegExp` in `package/config/rolldown/src/package-externals.ts`. The issues in this group must land as one change: whichever is picked up first should do the whole migration or link the others, rather than editing the same option independently.",
      ''
    );
  }
  lines.push(
    '**Acceptance criteria**',
    '',
    "- [ ] Replace the regex with the string or scanner form named here, behaviour-preserving: the current implementation is the spec. The replacement shape in this issue is a proposal derived from the pattern syntax, not a verified equivalent, so prove it with differential fixtures that run both forms over the same inputs, covering the flags this pattern carries, its capture order, its failure paths, and adversarial input from the site's real input source, before deleting the pattern."
  );
  if (finding.covered) {
    lines.push('- [ ] Remove the scoped `no-restricted-syntax/no-regex` disable comment, so the gate covers the site again instead of recording an exception.');
  }
  for (const step of verificationSteps(
    finding,
    gateIssue
  )) {
    lines.push(step);
  }
  if (finding.exclusion === 'paused-package') {
    lines.push('- [ ] If the package is still paused, record the change in its README status block rather than unpausing the package as a side effect.');
  }
  lines.push(`- [ ] Audit index: #${indexIssue}.`);
  return lines.join('\n');
}

function labelsFor(finding) {
  const labels = ['enhancement'];
  const easy = new Set([
    'prefix',
    'suffix',
    'substring',
    'exact-equality',
    'replace-all-literal',
    'word-boundary-substring',
    'class-run-scan',
    'fixed-length-class-run',
    'lookbehind-scan'
  ]);
  labels.push(easy.has(finding.shape) ? 'difficulty:easy' : 'difficulty:medium');
  if (finding.exclusion === 'paused-package') {
    labels.push('needs-triage');
    // The recorded deferral label names webapp-forge and paper2vn work explicitly;
    // other paused packages get needs-triage so the owner decides the status.
    if (finding.repoPath
      .includes('webapp-forge')
      || finding.repoPath
        .includes('paper2vn')
      || finding.repoPath
        .includes('messages-demo')) {
      labels.push('deferred-new-packages');
    }
  } else {
    labels.push('needs-triage');
  }
  if (finding.repoPath
    .startsWith('package/music-player/')) labels.push('app:music-player');
  return labels;
}

// Placeholder numbers, rewritten once the index and gate issues exist.
const INDEX_PLACEHOLDER = 'INDEX';
const GATE_PLACEHOLDER = 'GATE';

/**
 Sites sharing a file and line are distinct regex uses (chained calls, or several
 literals on one source line), so their titles need a discriminator.
 */
const lineGroupSizes = new Map();
for (const finding of findings) {
  const key = `${finding.repoPath}:${finding.line}`;
  lineGroupSizes.set(
    key,
    (lineGroupSizes.get(key) ?? 0) + 1
  );
}

const payloads = [];
for (const finding of findings) {
  const groupKey = `${finding.repoPath}:${finding.line}`;
  const group = (lineGroupSizes.get(groupKey) ?? 0) > 1 ? groupKey : null;
  // Chained calls share a line and column, so the slug carries the span identity
  // enumeration deduped on; otherwise distinct sites overwrite one payload file.
  const slug = `${finding.repoPath
    .replaceAll(
      '/',
      '__'
    )}:${finding.line}:${finding.column}:${finding.offset}-${finding.length}`;
  payloads.push({
    file: join(
      OUT,
      `${slug.replaceAll(
        ' ',
        '_'
      )}.json`
    ),
    title: titleFor(
      finding,
      group
    ),
    labels: labelsFor(finding),
    body: bodyFor(
      finding,
      {
        indexIssue: INDEX_PLACEHOLDER,
        gateIssue: GATE_PLACEHOLDER,
      }
    ),
    repoPath: finding.repoPath,
    line: finding.line,
    column: finding.column,
    offset: finding.offset,
    length: finding.length,
    // Surfaces enumerated without a span (Kotlin, inline YAML) carry their own
    // identity from supplement.mjs; offset and length are zero there, so the
    // computed form would collide across every site in one file.
    siteId: finding.siteId ?? `${finding.repoPath}:${finding.offset}:${finding.length}`,
    shape: finding.shape,
    exclusion: finding.exclusion,
    covered: finding.covered,
  });
}

for (const payload of payloads) {
  writeFileSync(
    payload.file,
    JSON.stringify(
      {
        title: payload.title,
        labels: payload.labels,
        body: payload.body,
      },
      null,
      2
    ),
    'utf8'
  );
}
writeFileSync(
  join(
    AUDIT,
    'payloads.json'
  ),
  JSON.stringify(
    payloads.map(({
      file,
      title,
      labels,
      repoPath,
      line,
      column,
      offset,
      length,
      siteId,
      shape,
      exclusion,
      covered,
    }) => ({
      file,
      title,
      labels,
      repoPath,
      line,
      column,
      offset,
      length,
      siteId,
      shape,
      exclusion,
      covered,
    })),
    null,
    2
  ),
  'utf8'
);

console.log(
  'payloads written:',
  payloads.length
);
console.log(
  'title length max:',
  Math.max(...payloads.map((payload) => payload.title
    .length))
);
console.log(
  'body length max:',
  Math.max(...payloads.map((payload) => payload.body
    .length))
);
console.log(
  'body length median:',
  payloads.map((payload) => payload.body
    .length)
    .sort((
      left,
      right
    ) => left - right)[Math.floor(payloads.length / 2)]
);
