// Final audit pipeline: group sites, decide necessity, derive scope and replacement
// sketch, and emit the per-site finding records used to file issues.
import {
  readFileSync,
  writeFileSync,
  existsSync,
} from 'node:fs';
import {
  join,
  dirname,
  relative,
} from 'node:path';
import {
  splitPattern,
  tokenize,
  deriveSketch,
} from './pattern-analysis.mjs';

const ROOT = '/var/home/user/Monochromatic';
const AUDIT = '/home/user/temp/agent/regex-audit';

const sites = JSON.parse(readFileSync(
  join(
    AUDIT,
    'sites3.json'
  ),
  'utf8'
));

const lineCache = new Map();
function linesOf(repoPath) {
  if (lineCache.has(repoPath)) return lineCache.get(repoPath);
  let lines = [];
  try {
    lines = readFileSync(
      join(
        ROOT,
        repoPath
      ),
      'utf8'
    )
      .split('\n');
  } catch (error) {
    lines = [];
  }
  lineCache.set(
    repoPath,
    lines
  );
  return lines;
}

/**
 Nearest enclosing declaration, preferring a function, class or test block over an
 inner binding, and never reading the site's own line. A module-level constant is
 useful context; an inner local binding is not, so indented bindings are skipped.
 */
function enclosingSymbol(
  lines,
  line
) {
  const blockPattern = /(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s*\*?\s*([A-Za-z0-9_$]+)/;
  const testPattern = /^\s*(?:it|test|describe)\(\s*\{?\s*name:\s*[`'"]([^`'"]+)/;
  const classPattern = /(?:export\s+)?(?:abstract\s+)?class\s+([A-Za-z0-9_$]+)/;
  const arrowPattern = /^(?:export\s+)?const\s+([A-Za-z0-9_$]+)\s*(?::[^=]+)?=\s*(?:async\s*)?(?:\(|function)/;
  const moduleConstPattern = /^(?:export\s+)?const\s+([A-Za-z0-9_$]+)\s*(?::[^=]+)?=/;
  const start = line - 2;
  for (let index = start; (index >= 0) && (index >= (start
    - 250)); index -= 1) {
    const text = lines[index] ?? '';
    const block = blockPattern.exec(text) ?? testPattern.exec(text)
      ?? classPattern.exec(text)
      ?? arrowPattern.exec(text);
    if (block !== null) return block[1];
  }
  for (let index = start; (index >= 0) && (index >= (start
    - 250)); index -= 1) {
    const binding = moduleConstPattern.exec(lines[index] ?? '');
    if (binding !== null) return binding[1];
  }
  return '(module scope)';
}

const packageCache = new Map();

/**
 Resolve the owning package: the nearest ancestor with a `mise.toml` gives the
 task prefix, and the scope name comes from that directory's `package.json` when
 both sit in the same directory, otherwise from the path (the convention existing
 refactor issues use, such as `music-player-design`).
 */
function packageContextOf(repoPath) {
  if (packageCache.has(repoPath)) return packageCache.get(repoPath);
  const start = dirname(join(
    ROOT,
    repoPath
  ));
  let miseDir = null;
  let manifestDir = null;
  let cursor = start;
  while (cursor.startsWith(ROOT) && (cursor !== ROOT)) {
    if ((miseDir === null) && existsSync(join(
      cursor,
      'mise.toml'
    ))) miseDir = cursor;
    if ((manifestDir === null) && existsSync(join(
      cursor,
      'package.json'
    ))) manifestDir = cursor;
    if ((miseDir !== null) && (manifestDir !== null)) break;
    const parent = dirname(cursor);
    if (parent === cursor) break;
    cursor = parent;
  }
  function relative(dir) { return dir === null ? null : dir.slice(ROOT.length + 1); }
  const packageDir = relative(miseDir) ?? relative(manifestDir);
  let scope;
  if ((manifestDir !== null) && (manifestDir === miseDir)) {
    try {
      const {name} = JSON.parse(readFileSync(
        join(
          manifestDir,
          'package.json'
        ),
        'utf8'
      ));
      scope = ((typeof name) === 'string') && (name !== '') ? name.replace(
        '@monochromatic-dev/',
        ''
      ) : null;
    } catch (error) {
      scope = null;
    }
  }
  if ((scope === undefined) || (scope === null)) {
    if (packageDir === null) {
      // `doc/<family>/...` takes the family as its scope; a file directly under
      // `doc/` has no family segment, so it is plain `docs`.
      const segments = repoPath.split('/');
      scope = repoPath.startsWith('doc/')
        ? (segments.length > 2 ? `docs(${segments[1]})` : 'docs')
        : 'repo';
    } else {
      scope = packageDir.replace(
        /^package\//,
        ''
      )
        .replaceAll(
        '/',
        '-'
        );
    }
  }
  const context = {
    scope,
    packageDir,
  };
  packageCache.set(
    repoPath,
    context
  );
  return context;
}

// Sites where a RegExp value or arbitrary user-supplied pattern semantics IS the
// feature, so no regex-free implementation exists at any complexity.
const NECESSARY_FILES = new Set([
  'package/test-fixture/oxlint-no-restricted-syntax/src/valid/no-regex.ts',
  'package/test-fixture/oxlint-no-restricted-syntax/src/invalid/no-regex.ts',
  'package/or-throw/core/src/regexp-or-throw.unit.test.ts',
  'package/or-throw/core/src/regexp-or-throw.unit.test.js',
  'package/test-harness/core-expect/src/expect.unit.test.ts',
  'package/test-harness/core-expect/src/expect-matchers-core.ts',
  'package/test-harness/core/src/expect.unit.test.js',
  'package/test-harness/core/src/sinon.unit.test.ts',
  'package/test-harness/core/src/sinon.unit.test.js',
  'package/upstream-fork/pify/src/key-filter.unit.test.ts',
  'package/upstream-fork/pify/src/pify-options.ts',
  'package/upstream-fork/pify.fuzz/src/pify-arbitrary.ts',
  'package/upstream-fork/pify.fuzz/src/spec-fixtures.ts',
  'package/upstream-fork/pify.fuzz/src/coverage-driver.ts',
  'package/dependency-catalog/cli-cube/src/script/filter.ts',
  'package/watch-restart/cli/src/cli-helpers.ts',
]);
const NECESSARY_PREFIXES = [
  'package-paused/module/es/src/types/t object/t regexp/',
  'package-paused/module/es/src/types/t string/f/t string/trim/w/t object/t regexp/',
  'package/upstream-fork/deepmerge-ts.fuzz/',
];
const NECESSARY_REASONS = {
  'package/test-fixture/oxlint-no-restricted-syntax/src/valid/no-regex.ts': 'lint-rule fixture: the literals are the input the no-regex rule must accept',
  'package/test-fixture/oxlint-no-restricted-syntax/src/invalid/no-regex.ts': 'lint-rule fixture: the literals are the input the no-regex rule must report',
  'package/or-throw/core/src/regexp-or-throw.unit.test.ts': 'test of regExpOrThrow: the RegExp instances are the values under test',
  'package/or-throw/core/src/regexp-or-throw.unit.test.js': 'test of regExpOrThrow: the RegExp instances are the values under test',
  'package/test-harness/core-expect/src/expect.unit.test.ts': 'tests the .toMatch/.toThrow RegExp branch: the literal is the branch input',
  'package/test-harness/core/src/expect.unit.test.js': 'tests the .toMatch/.toThrow RegExp branch: the literal is the branch input',
  'package/test-harness/core-expect/src/expect-matchers-core.ts': 'Jest-compatible toMatch(string) must compile the author pattern to a RegExp for chai .match',
  'package/test-harness/core/src/sinon.unit.test.ts': 'expect.stringMatching(pattern: RegExp) accepts only a RegExp',
  'package/test-harness/core/src/sinon.unit.test.js': 'expect.stringMatching(pattern: RegExp) accepts only a RegExp',
  'package/upstream-fork/pify/src/key-filter.unit.test.ts': 'upstream pify include/exclude contract is Array<string | RegExp>; these exercise the RegExp branch',
  'package/upstream-fork/pify/src/pify-options.ts': 'verbatim upstream pify 6.1.0 default exclude RegExp: the differential oracle must see the same RegExp',
  'package/upstream-fork/pify.fuzz/src/pify-arbitrary.ts': 'differential fuzz inputs must be RegExp values to reach upstream pattern.test(key)',
  'package/upstream-fork/pify.fuzz/src/spec-fixtures.ts': 'copies a generated RegExp per side so /g lastIndex cannot leak between runs',
  'package/upstream-fork/pify.fuzz/src/coverage-driver.ts': 'needs a RegExp value to reach the regex branch of member selection',
  'package/dependency-catalog/cli-cube/src/script/filter.ts': 'compiles an arbitrary user-typed /.../ pattern: the feature is regex semantics on user input',
  'package/watch-restart/cli/src/cli-helpers.ts': 'compiles an arbitrary user CLI pattern: the feature is regex semantics on user input',
};

function necessity(site) {
  if (NECESSARY_FILES.has(site.repoPath)) {
    return {
      necessary: true,
      reason: NECESSARY_REASONS[site.repoPath] ?? 'RegExp value is the feature',
    };
  }
  if (NECESSARY_PREFIXES.some((prefix) => site.repoPath
    .startsWith(prefix))) {
    return {
      necessary: true,
      reason: 'RegExp instances are the data or API subject in this tree',
    };
  }
  return {
    necessary: false,
    reason: '',
  };
}

function exclusionOf(site) {
  if (site.tracked === false) return 'untracked';
  if (site.repoPath
    .endsWith('.generated.ts')
    || site.repoPath
      .includes('/canary-lint/')) return 'generated-or-corpus';
  if (site.repoPath
    .includes('upstream-conf/')) return 'vendored-upstream';
  if (site.repoPath
    .startsWith('package-paused/')) return 'paused-package';
  if (site.repoPath
    .startsWith('package-deprecated/')
    || site.repoPath
      .startsWith('.out-of-scope/')) return 'out-of-scope-tree';
  return 'in-scope';
}

const findings = [];
for (const site of sites) {
  const exclusion = exclusionOf(site);
  const decision = necessity(site);
  const lines = linesOf(site.repoPath);
  const context = packageContextOf(site.repoPath);
  const {
    body,
    flags,
    dynamic,
  } = splitPattern(site.pattern);
  const derived = dynamic
    ? deriveSketch({
      atoms: [],
      flags,
      dynamic: true,
    })
    : deriveSketch({
      atoms: tokenize(body),
      flags,
      dynamic: false,
    });
  findings.push({
    ...site,
    exclusion,
    scope: context.scope,
    packageDir: context.packageDir,
    symbol: enclosingSymbol(
      lines,
      site.line
    ),
    necessary: decision.necessary,
    necessaryReason: decision.reason,
    shape: derived.shape,
    sketch: derived.sketch,
    grammar: derived.grammar,
    flags,
    dynamic,
  });
}

writeFileSync(
  join(
    AUDIT,
    'audit.json'
  ),
  JSON.stringify(
    findings,
    null,
    2
  ),
  'utf8'
);

const inScopeUnnecessary = findings.filter((finding) => (finding.exclusion === 'in-scope') && (finding.necessary === false));
const pausedUnnecessary = findings.filter((finding) => (finding.exclusion === 'paused-package') && (finding.necessary === false));
const counts = {};
for (const finding of findings) {
  const key = `${finding.exclusion}/${finding.necessary ? 'necessary' : 'unnecessary'}`;
  counts[key] = (counts[key] ?? 0) + 1;
}
console.log(JSON.stringify(
  {
    total: findings.length,
    counts,
    activeIssuesToFIle: inScopeUnnecessary.length,
    pausedIssuesToFIle: pausedUnnecessary.length,
    activeFiles: new Set(inScopeUnnecessary.map((finding) => finding.repoPath)).size,
  },
  null,
  2
));
writeFileSync(
  join(
    AUDIT,
    'to-file.json'
  ),
  JSON.stringify(
    [
      ...inScopeUnnecessary,
      ...pausedUnnecessary
    ],
    null,
    2
  ),
  'utf8'
);
const byShape = {};
for (const finding of [
  ...inScopeUnnecessary,
  ...pausedUnnecessary
]) byShape[finding.shape] = (byShape[finding.shape] ?? 0) + 1;
console.log(JSON.stringify(
  byShape,
  null,
  2
));
const justified = [
  ...inScopeUnnecessary,
  ...pausedUnnecessary
].filter((finding) => finding.covered);
console.log(
  'with a disable justification needing rebuttal:',
  justified.length
);
console.log(
  'distinct justifications:',
  new Set(justified.map((finding) => (finding.justification ?? '').slice(
    0,
    80
  ))).size
);
