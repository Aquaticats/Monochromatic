// Add the surfaces the JS/TS scan could not reach: Kotlin, HTML script blocks, and
// one YAML inline script. Also split the incidental escape regex out of the
// module/es regexp-tooling exemption.
import {
  readFileSync,
  writeFileSync,
  existsSync,
} from 'node:fs';
import {
  join,
  dirname,
} from 'node:path';
import {
  splitPattern,
  tokenize,
  deriveSketch,
} from './pattern-analysis.mjs';

const ROOT = '/var/home/user/Monochromatic';
const AUDIT = '/home/user/temp/agent/regex-audit';

const allFindings = JSON.parse(readFileSync(
  join(
    AUDIT,
    'audit.json'
  ),
  'utf8'
));

// Sites reclassified on individual review: producing or accepting a RegExp is the
// contract, but an incidental regex beside that contract is not.
const FLIP_TO_UNNECESSARY = [
  {
    match: (finding) => (finding.repoPath === 'package-paused/module/es/src/types/t object/t regexp/f/t string/r s/p n/index.ts')
      && (finding.line === 62),
    splitNote: 'The enclosing `toRegExp` must return a `RegExp`, so the `RegExp(escapedPattern,)` construction at line 67 stays and no issue is filed for it. This site is the incidental escape pass that feeds it: replacing each metacharacter with a backslash-prefixed copy is a character walk over a 15-character alphabet, and it needs no pattern to do that.',
  },
  {
    match: (finding) => (finding.repoPath === 'package/test-harness/core-expect/src/expect-matchers-core.ts')
      && (finding.line === 294),
    splitNote: 'The `.toMatch` contract is `string | RegExp`, and the RegExp branch genuinely needs chai\'s regex matcher. This site is the string branch: `new RegExp(expected,)` compiles the author\'s string so chai can match it. Jest documents a string argument to `toMatch` as substring inclusion, which `includes` answers directly, so the compilation is unnecessary and currently deviates from Jest for strings containing metacharacters (`toMatch(\'a.c\')` matches `\'abc\'` today). Changing the string branch is a behaviour change for exactly those inputs, so it needs an owner decision and a test that pins whichever semantics are wanted.',
  },
];

const findings = [];
for (const finding of allFindings) {
  const flip = FLIP_TO_UNNECESSARY.find((entry) => entry.match(finding));
  if (flip !== undefined) {
    findings.push({
      ...finding,
      necessary: false,
      necessaryReason: '',
      exclusion: finding.repoPath
        .startsWith('package-paused/') ? 'paused-package' : 'in-scope',
      splitNote: flip.splitNote,
      surface: 'js-ts',
      detector: null,
    });
    continue;
  }
  if ((finding.necessary === false)
    && ((finding.exclusion === 'in-scope') || (finding.exclusion === 'paused-package'))) {
    findings.push(finding);
  }
}
const htmlSites = JSON.parse(readFileSync(
  join(
    AUDIT,
    'html-sites.json'
  ),
  'utf8'
));

const packageCache = new Map();
function packageContextOf(repoPath) {
  if (packageCache.has(repoPath)) return packageCache.get(repoPath);
  let cursor = dirname(join(
    ROOT,
    repoPath
  ));
  let miseDir = null;
  let manifestDir = null;
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
  let scope = null;
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
  if (scope === null) {
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

function linesOf(repoPath) {
  try {
    return readFileSync(
      join(
        ROOT,
        repoPath
      ),
      'utf8'
    )
      .split('\n');
  } catch (error) {
    return [];
  }
}

function symbolAt(
  lines,
  line
) {
  const patterns = [
    /(?:export\s+)?(?:async\s+)?function\s*\*?\s*([A-Za-z0-9_$]+)/,
    /(?:private|public|internal)?\s*(?:fun|val|var)\s+([A-Za-z0-9_$]+)/,
    /(?:export\s+)?const\s+([A-Za-z0-9_$]+)\s*(?::[^=]+)?=\s*(?:async\s*)?(?:\(|function)/,
    /(?:export\s+)?const\s+([A-Za-z0-9_$]+)\s*(?::[^=]+)?=/,
    /function\s+([A-Za-z0-9_$]+)\s*\(/,
  ];
  for (let index = line - 2; (index >= 0) && (index >= (line
    - 120)); index -= 1) {
    const text = lines[index] ?? '';
    for (const pattern of patterns) {
      const match = pattern.exec(text);
      if (match !== null) return match[1];
    }
  }
  return '(module scope)';
}

/**
 Unwrap a Kotlin `Regex("...")` construction into the regex source its string
 literal denotes, so the pattern tokenizer sees the real body.
 */
function kotlinPatternBody(pattern) {
  const start = pattern.indexOf('"');
  const end = pattern.lastIndexOf('"');
  if ((start === (-1)) || (end <= start)) return null;
  return pattern.slice(
    start + 1,
    end
  )
    .replaceAll(
    String.raw`\\`,
    '\\'
    );
}

function analyse(pattern) {
  const kotlinBody = pattern.startsWith('Regex(') ? kotlinPatternBody(pattern) : null;
  if (kotlinBody !== null) {
    const derived = deriveSketch({
      atoms: tokenize(kotlinBody),
      flags: '',
      dynamic: false,
    });
    return {
      ...derived,
      flags: '',
      dynamic: false,
    };
  }
  const {
    body,
    flags,
    dynamic,
  } = splitPattern(pattern);
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
  return {
    ...derived,
    flags,
    dynamic,
  };
}

/**
 Provenance exclusions found by tracing generators and directory ownership rather
 than by a filename marker. Each entry records the evidence that decided it.
 */
const PROVENANCE_EXCLUSIONS = [
  {
    match: (site) => site.repoPath === 'package/music-player/design/questions/template-editor.html',
    reason: 'generated output: `package/music-player/design/template-editor.mjs:121` writes this 18 MB file from `template-editor.template.html`, which is scanned separately and carries its own issues. Fixing a generated file means fixing its template.',
  },
  {
    match: (site) => site.repoPath
      .includes('/uploads/'),
    reason: 'user-provided reference material: `package/music-player/design/HANDOFF.md:3208` describes `uploads/` as "the user\'s own mockup, screenshots and reference images", so it is audit input rather than first-party code.',
  },
];

const excludedByProvenance = [];

const added = [];

// Kotlin: the audit's JS detector cannot see these, so they were enumerated by a
// corrected text search over tracked .kt and .kts files and read individually.
const kotlinSites = [
  {
    repoPath: 'package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/TrackTemplate.kt',
    line: 989,
    pattern: 'Regex("0[xX][0-9a-fA-F]+")',
    note: 'hexadecimal numeric literal for the template expression evaluator',
  },
  {
    repoPath: 'package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/TrackTemplate.kt',
    line: 992,
    pattern: 'Regex("0[oO][0-7]+")',
    note: 'octal numeric literal for the template expression evaluator',
  },
  {
    repoPath: 'package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/TrackTemplate.kt',
    line: 995,
    pattern: 'Regex("0[bB][01]+")',
    note: 'binary numeric literal for the template expression evaluator',
  },
  {
    repoPath: 'package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/TrackTemplate.kt',
    line: 998,
    pattern: String.raw`Regex("[+-]?(Infinity|(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?)")`,
    note: 'decimal numeric literal with optional sign and exponent',
  },
  {
    repoPath: 'package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/SearchMatcher.kt',
    line: 54,
    pattern: String.raw`Regex("\\s+")`,
    note: 'whitespace run used to split a search query into terms',
  },
];

for (const site of kotlinSites) {
  const lines = linesOf(site.repoPath);
  const derived = analyse(site.pattern);
  const context = packageContextOf(site.repoPath);
  added.push({
    ...site,
    column: 1,
    offset: 0,
    length: 0,
    siteId: `${site.repoPath}:${site.line}:kotlin`,
    tracked: true,
    kind: 'literal',
    message: 'Kotlin Regex construction',
    spanText: (lines[site.line - 1] ?? '').trim()
      .slice(
      0,
      300
      ),
    sourceLine: (lines[site.line - 1] ?? '').trim()
      .slice(
      0,
      300
      ),
    covered: false,
    directiveKind: null,
    justification: null,
    exclusion: 'in-scope',
    scope: context.scope,
    packageDir: context.packageDir,
    symbol: symbolAt(
      lines,
      site.line
    ),
    necessary: false,
    necessaryReason: '',
    surface: 'kotlin',
    detector: "a corrected text search over tracked `.kt` and `.kts` files (`rg -n --no-ignore -g '*.kt' -g '*.kts' -e 'Regex' -e 'toRegex' -e 'Pattern.compile'`), then each hit read individually. The repo's `no-restricted-syntax/no-regex` rule is a JavaScript rule and cannot see Kotlin, and `package/rust-linter-plugin` has no regex rule either, so no gate covers this surface.",
    note: site.note,
    shape: derived.shape,
    sketch: derived.sketch,
    grammar: derived.grammar,
    flags: derived.flags,
    dynamic: derived.dynamic,
  });
}

// HTML script blocks: extracted into mirror .js files and scanned with the same
// parser-backed rule, then mapped back by block start line.
for (const site of htmlSites) {
  const provenance = PROVENANCE_EXCLUSIONS.find((entry) => entry.match(site));
  if (provenance !== undefined) {
    excludedByProvenance.push({
      repoPath: site.repoPath,
      line: site.line,
      reason: provenance.reason,
    });
    continue;
  }
  const lines = linesOf(site.repoPath);
  const derived = analyse(site.pattern);
  const context = packageContextOf(site.repoPath);
  const paused = site.repoPath
    .startsWith('package-paused/');
  added.push({
    ...site,
    siteId: `${site.repoPath}:${site.offset}:${site.length}`,
    spanText: (lines[site.line - 1] ?? '').trim()
      .slice(
      0,
      300
      ),
    sourceLine: (lines[site.line - 1] ?? '').trim()
      .slice(
      0,
      300
      ),
    covered: false,
    directiveKind: null,
    justification: null,
    exclusion: paused ? 'paused-package' : 'in-scope',
    scope: context.scope,
    packageDir: context.packageDir,
    symbol: symbolAt(
      lines,
      site.line
    ),
    necessary: false,
    necessaryReason: '',
    surface: 'html-script',
    detector: "the repo's own `no-restricted-syntax/no-regex` rule, run over each tracked HTML file's `<script>` blocks extracted into mirror `.js` files (102 blocks from 162 files) and mapped back by block start line. HTML is outside the oxlint gate's file set, so no justification was ever required here.",
    shape: derived.shape,
    sketch: derived.sketch,
    grammar: derived.grammar,
    flags: derived.flags,
    dynamic: derived.dynamic,
  });
}

// YAML inline script: one hit, in a paused package's compose file.
const yamlSite = {
  repoPath: 'package-paused/dev-script/inference-canary-viewer/docker-compose.yml',
  line: 55,
  column: 1,
  offset: 0,
  length: 0,
  siteId: 'package-paused/dev-script/inference-canary-viewer/docker-compose.yml:55:yaml',
  tracked: true,
  kind: 'string-method',
  message: 'String#replace() with an inline regex in an inline bun script',
  pattern: String.raw`/^\/$/`,
  surface: 'yaml-inline',
  detector: "a text search for regex call shapes over tracked `.yml` and `.yaml` files, then each hit read individually. Inline shell and script strings in compose files are outside every lint gate.",
  note: 'inline bun HTTP handler rewriting a root pathname to index.html',
  exclusion: 'paused-package',
  covered: false,
  directiveKind: null,
  justification: null,
  necessary: false,
  necessaryReason: '',
};
{
  const lines = linesOf(yamlSite.repoPath);
  const derived = analyse(yamlSite.pattern);
  const context = packageContextOf(yamlSite.repoPath);
  yamlSite.sourceLine = (lines[54] ?? '').trim()
    .slice(
    0,
    300
    );
  yamlSite.spanText = yamlSite.sourceLine;
  yamlSite.scope = context.scope;
  yamlSite.packageDir = context.packageDir;
  yamlSite.symbol = 'compose service command';
  yamlSite.shape = derived.shape;
  yamlSite.sketch = derived.sketch;
  yamlSite.grammar = derived.grammar;
  yamlSite.flags = derived.flags;
  yamlSite.dynamic = derived.dynamic;
  added.push(yamlSite);
}

// The two reclassifications above are applied where the findings are loaded, so
// every retained necessary site has been read individually rather than exempted by
// a directory prefix alone.

const all = [
  ...findings,
  ...added
];
writeFileSync(
  join(
    AUDIT,
    'to-file2.json'
  ),
  JSON.stringify(
    all,
    null,
    2
  ),
  'utf8'
);
writeFileSync(
  join(
    AUDIT,
    'provenance-exclusions.json'
  ),
  JSON.stringify(
    excludedByProvenance,
    null,
    2
  ),
  'utf8'
);
console.log(JSON.stringify(
  {
    excludedByProvenance: excludedByProvenance.length,
    previousUnnecessary: findings.length,
    added,
    total: all.length,
    bySurface: all.reduce(
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
  },
  null,
  2
));
