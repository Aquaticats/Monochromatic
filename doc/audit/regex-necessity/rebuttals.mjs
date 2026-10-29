// Rebuttals for each distinct disable justification carried by an in-scope site.
// Matched by a distinctive substring of the justification text. Every measured
// claim here comes from evidence2.mjs (corrected probes) or evidence.mjs.
const ROLLDOWN_EXTERNAL = "rolldown's `external` accepts a plain string or a predicate, not only a RegExp: `type ExternalOption = StringOrRegExp | StringOrRegExp[] | ExternalOptionFunction` with `type ExternalOptionFunction = (id: string, parentId: string | undefined, isResolved: boolean) => NullValue<boolean>` (`node_modules/rolldown/dist/shared/define-config-CGhm2DU-.d.mts:3415-3417`, read in this tree). The predicate replaces the whole option value rather than one array entry, so this site has to migrate together with the other configs that list the same patterns.";

const THROW_STRING = "this repo's expect accepts a string here: `toThrow: (expected?: string | RegExp | (abstract new(...args: never) => unknown),) => void` (`package/test-harness/core-expect/src/expect-matchers.ts:91-93`), and the implementation passes the value straight to chai's `.throw` (`expect-matchers-core.ts:131-146`), where a string argument is a message-inclusion check. So the string form removes the regex from the feature entirely, unlike `.toMatch`, whose string form compiles `new RegExp(expected)` at `expect-matchers-core.ts:294`.";

const RULES = [
  {
    key: 'secret-detection patterns; the regex literals ARE the rule set',
    text: '"The regex literals ARE the rule set" answers what the patterns are, not why they need regex semantics. Every shape in this block is a literal token followed by a bounded run from a small alphabet (`ghp_` plus at least 36 of `[A-Za-z0-9_]`, `AKIA` plus exactly 16 of `[0-9A-Z]`), or a keyword with word boundaries. Each is one linear scan: `indexOf` the literal, walk while the character satisfies the class, compare the count against the bound. The exported type is `readonly RegExp[]` and consumers call `.test()`, so the replacement is an array of predicates with the same call shape. Note that `readonly RegExp[]` is this repo\'s own annotation, not an external contract, so it does not by itself establish necessity. The rest of the justification is a safety argument ("no nested quantifiers", "input is bounded by the surrounding scanner"), which is not the question the rule asks. Two patterns carry the `i` flag (`SECRET_VAR_PATTERN`, `SECRET_PATH_PATTERN`), so their replacements must fold case explicitly.',
  },
  {
    key: 'preview extractor strips markdown structural markup',
    text: 'The linearity claim survives measurement, so this issue argues necessity only. Corrected probe: on a single trailing unmatched opener the fence pattern takes 0.1 ms over 128 KB, and on many matched pairs 0.4 ms over 128 KB, because a lazy `A[\\s\\S]*?A` with the global flag consumes through each terminator and only a trailing unmatched opener costs one extra full scan. Necessity is still unmet: each of the eight chained passes is either a character-class run collapse or a literal-delimiter pair, both single-pass scans, and eight chained `replaceAll` calls mean eight passes over the excerpt where one scanner pass can do all of them.',
  },
  {
    key: 'Anchored protocol prefix test for node builtins',
    text: `\`/^node:/u.test(id)\` is exactly \`id.startsWith('node:')\`: no character class, no quantifier, no anchor semantics beyond the prefix. The justification offers input bounds and backtracking safety, neither of which bears on whether a prefix test needs a pattern engine. ${  ROLLDOWN_EXTERNAL  } The same literal is duplicated across six config files while \`package/config/rolldown/src/package-externals.ts\` already owns the externals helpers.`,
  },
  {
    key: 'Fragment key parser; inputs are canonical keys',
    text: 'A canonical, slash-delimited key is the textbook `split` case: split on `/`, compare the fixed segments, and take the captures as the remaining segments. That is linear like the pattern, and it states the segment contract in code instead of encoding it inside an anchored pattern with two negated classes. The justification argues bounds and the absence of nested quantifiers, which is a safety claim, not a necessity claim.',
  },
  {
    key: 'anchored package-name matchers consumed by tsdown',
    text: "This repo already recorded that tsdown's `alwaysBundle` accepts plain strings: non-`/`-fenced entries go through picomatch or exact match, verified against `node_modules/tsdown/dist/general-D3muxt2f.mjs:14-30` (`doc/handover/no-regex.md`, section \"Critical conventions captured during this session\"). The string form has to be chosen per entry, because the patterns here are not all the same shape: `/^debug$/u` is exact and becomes `'debug'`, `/^micromark-/u` is a hyphenated prefix and becomes `'micromark-*'`, and `/^micromark/u` is a bare prefix that also matches names such as `micromarkish`, so a single exact string would narrow it. tsdown is not installed in this tree because the package is paused, so re-verify each string form against the installed tsdown when the package resumes.",
  },
  {
    key: 'Anchored prefix/exact match on package names',
    text: "Same as the messages-demo tsdown matchers: this repo recorded that `alwaysBundle` accepts plain strings (picomatch or exact match) in `doc/handover/no-regex.md`, verified against `node_modules/tsdown/dist/general-D3muxt2f.mjs:14-30`. `/^@monochromatic-dev\\//u` is a scoped prefix, `/^lezer-/u` a hyphenated prefix, and `/^jspdf$/u` an exact name, so each needs its own string or glob form rather than one shape for all. Re-verify against the installed tsdown when this paused package resumes.",
  },
  {
    key: "The 'u' flag is omitted deliberately: the pattern interpolates the dynamic name identifier",
    text: 'Corrected probe against all four production patterns and six identifiers (`describe`, `ALL_CAPS`, `_under`, `a1`, `café`, `a$b`), with a positive control confirming that six of the combinations do match: no result differed between `new RegExp(source)` and `new RegExp(source, \'u\')`. So the claim that u-mode "could alter how that interpolated value matches" is not supported by these fixtures; the honest statement is narrower, that no difference was observed for parser-produced identifiers. Two further points. First, the same probe shows a separate latent defect: an identifier containing `$` (a legal identifier character, and a regex anchor when interpolated) never matches in either mode, so `a$b` is unclassifiable regardless of flags. Second, "simpler than parsing a second AST" is a code-length argument, while the rule message asks for an index scan, parser, or string API: each pattern is a keyword at line start plus an identifier compare, so one linear pass over the already-cached source text replaces four pattern constructions per lookup. The re-export matcher does have a measured cost problem, on the input family the earlier probe missed: with many unterminated `export {` openers, `export\\s*\\{[^}]*\\bNAME\\b[^}]*\\}\\s*from\\s*[\'"]` takes 10.4 ms over 15 KB, 28.7 ms over 30 KB, 118 ms over 60 KB and 460 ms over 120 KB (three runs per size, about four times per doubling, so quadratic), because each opener rescans the brace-free remainder.',
  },
  {
    key: 'Anchored literal name test keeping the electron runtime external',
    text: `\`/^electron(\\/|$)/u.test(id)\` is \`id === 'electron' || id.startsWith('electron/')\`. ${  ROLLDOWN_EXTERNAL}`,
  },
  {
    key: 'Anchored literal name test keeping the electron runtime and its packager external',
    text: `\`/^(electron|@electron\\/packager)(\\/|$)/u.test(id)\` is a two-element array test: \`EXTERNAL_NAMES.some((name) => id === name || id.startsWith(name + '/'))\`. ${  ROLLDOWN_EXTERNAL}`,
  },
  {
    key: 'source-code tokenizer scanning project files for `icon(',
    text: 'This is the shape the rule message names directly ("an index scan, parser, or string API"): walk the source once, skip `//` and `/* */` spans with `indexOf`, and at each `icon(` occurrence read the quoted name and check the `[a-z][a-z0-9_]` alphabet character by character. Same linear complexity, and comment skipping becomes explicit instead of a second lazy `[\\s\\S]*?` pattern that can also match across a string literal. The justification argues bounds and linearity, which is a safety claim.',
  },
  {
    key: 'asserts the buffer table rejects each out-of-range changeset',
    text: `The justification itself calls this a "literal phrase match", which is a substring test on the thrown message. ${  THROW_STRING  } "No backtracking" is a safety claim about a pattern that has no metacharacters at all.`,
  },
  {
    key: 'canonical first-unescaped-quote scan',
    text: 'The justification concedes the point: "expressing the escape-counting rule via index walk is significantly more code and equally bounded". Equal bounds means equal complexity, and code length is not the criterion. The rule is a forward pass carrying escape parity: count the run of consecutive backslashes before each quote candidate and treat the quote as effective when that count is even, so the input is visited once with no lookbehind engine and no `(?:\\\\\\\\)*` group to reason about.',
  },
  {
    key: 'canonical unescaped-quote enumeration',
    text: 'Same as the `hasQuotedSyntax` sites: the justification concedes the index walk is "equally bounded" and objects only that it is more code. Escape-parity counting is a forward pass carrying the parity state; the `/g` `matchAll` form additionally resumes from `lastIndex` per quote, so the scan form is no worse and keeps the parity rule visible.',
  },
  {
    key: 'fingerprinted-filename anchored suffix matchers',
    text: 'Both patterns are constant-length checks over a basename: find the last `.`, find the `.` before it, test that the slice between them is exactly 10 characters each within `0-9a-f`, and test the tail for a dot (and for the `.zst` suffix in the second pattern). That is linear over a filesystem-length name, and it turns the `{10}` repetition count into an explicit length compare the fingerprinter and the matcher must agree on.',
  },
  {
    key: 'Extracts S3 pagination control fields',
    text: 'A single-element XML field is `indexOf(\'<IsTruncated>\')`, `indexOf(\'</IsTruncated>\', from)` and a `slice`: one linear pass with a moving cursor, no engine. "Non-greedy single-capture pattern with no alternation prevents catastrophic backtracking" is a safety argument; it also leaves the semantics implicit, since `.` does not cross newlines without the `s` flag, so this pattern silently fails on a pretty-printed body where the scan form would find the field.',
  },
  {
    key: 'Streaming XML extractor over an S3 ListObjectsV2 response body',
    text: 'The `/g` form is a cursor loop: `indexOf(\'<Key>\', cursor)`, `indexOf(\'</Key>\', from)`, push the slice, advance. Linear in the body, one pass, and unlike `(.+?)` it does not accept a `<Key>` whose value contains `<`. The replacement must also keep the non-overlap rule: the cursor advances past the closing tag, not past the opening one. The bounded-`maxKeys` and no-alternation claims are safety arguments, not necessity.',
  },
  {
    key: 'URL path parser; input is a request path',
    text: 'A request pathname is a slash-delimited segment list: `pathname.split(\'/\')` plus fixed-segment compares and a digit check on the issue number. Linear, and it names each segment (owner, repo, number) instead of leaving them as numbered captures. The trailing `/?` optional slash becomes one explicit `at(-1) === \'\'` check.',
  },
  {
    key: 'builds a malformed-but-base64url-shaped fixture',
    text: `The \`/=+$/u\` strip is a backward walk from the end while the character is \`=\`, then a \`slice\`; the phrase assertion is a substring test on the thrown message. ${  THROW_STRING}`,
  },
  {
    key: 'builds a base64url-shaped fixture with a non-numeric',
    text: `Same as the sibling fixture site: \`/=+$/u\` is a backward walk over trailing \`=\` characters plus a \`slice\`, and the error-message assertion is a substring test. ${  THROW_STRING}`,
  },
  {
    key: 'canonical UUIDv4 format validator',
    text: 'The 8-4-4-4-12 contract is a length check plus per-position character checks: hex digits at the digit positions, a literal `-` at indices 8, 13, 18 and 23, `4` at index 14, and one of `89ab` at index 19. Linear over a 36-character input, and it states the contract position by position, which is what the test asserts. The pattern also carries the `i` flag, so the replacement must accept both hex cases. "No nested quantifiers, linear matching" is a safety claim.',
  },
  {
    key: 'Two replaceAll passes normalize whitespace in joined PDF page text',
    text: '"Character-class and quantifier regex are the clearest forms" is a brevity argument. Collapsing horizontal-whitespace runs to one space and capping newline runs at two are both single-pass character walks with a counter; the replacement is linear like the regex and does not allocate an intermediate string per run.',
  },
  {
    key: 'Two anchored regex strip the Markdown JSON code fence',
    text: '"Regex is the clearest way to express the optional `json` tag" is a brevity argument, and the branch is two `startsWith` tests: `startsWith(\'```json\')` consumes 7 characters, `startsWith(\'```\')` consumes 3, otherwise nothing; then trim. The suffix is `endsWith(\'```\')` plus a trim. Linear and branch-explicit.',
  },
  {
    key: 'anchored at `$` to parse the trailing `owner/repo',
    text: 'Two problems, not one. (1) Necessity: the slug is the last `owner/repo` pair before an optional `.git`, which is an `endsWith(\'.git\')` strip, two `lastIndexOf(\'/\')` calls and a host check on `github.com`; linear over a bounded URL. (2) A dotted-repository gap: the `[^/.]+` class excludes dots from the repository name, so the pattern returns no match for real GitHub repositories whose names contain a dot. Measured `null` for `git@github.com:owner/repo.js.git`, `https://github.com/owner/dot.repo`, and `git@github.com:owner/repo.name.git`, while `gh api repos/socketio/socket.io` confirms dotted names are real. The traced consequence: `getGithubSlug` feeds `resolveGitDatesContext().githubSlug`, which `git-dates.ts:446` documents as "required when shallow" and `git-dates.ts:358` interpolates into `repos/<slug>/commits?path=...`, so a dotted-name remote leaves the shallow-clone date path without a slug and the per-file dates fall back to whatever the ABSENT branch yields. Fixing that gap is a behaviour change and is deliberately separate from the behaviour-preserving regex removal this issue asks for.',
  },
  {
    key: 'SVG element tokenizer scoped to two literal tag names',
    text: 'The claim "Linear: every char is visited at most twice across the alternation" does not hold for a backtracking engine, and both sides are now measured on the same synthetic adversarial input (a string of `"<rect "` repeated with no `/>` terminator anywhere, which is the shape a truncated export produces; this is a constructed worst case, not an observed Figma truncation). Regex, one `exec` call, three runs per size: 9.3 ms at 15996 characters, 37 ms at 31998, 148 ms at 63996, 589 ms at 127998, about four times per doubling, so quadratic, with roughly 3% spread. The proposed cursor replacement (`indexOf(\'<rect\')` or `indexOf(\'<path\')`, then `indexOf(\'/>\', from)`, advancing past the terminator) on the same input: 0.1 ms, 0 ms, 0.1 ms, 0.2 ms. Each unterminated opener makes the lazy `[^>]*?` scan reach end of input before failing, which is where the quadratic term comes from. The replacement must keep two properties the pattern has: the tag match ends at a name boundary, and the attribute slice cannot cross `>`.',
  },
  {
    key: 'canonical SVG `translate(N)` shape parser',
    text: 'The pattern is unanchored, so it finds `translate(...)` anywhere inside the attribute value; a `startsWith` test would narrow it. The equivalent scan is `indexOf(\'translate(\')`, then `indexOf(\')\', from)`, then a digit and optional-fraction walk over the slice. Linear in the attribute length like the pattern, and it makes the accepted number shape explicit instead of leaving it to `(?:\\.\\d+)?`.',
  },
  {
    key: 'SVG path tokenizer; the alternation either matches one command letter',
    text: 'The alternation is one command letter or one signed decimal, which is a single character-class dispatch inside a `for` loop over the `d` string: linear, one pass, no per-token engine dispatch. It also keeps the tokenizer state (implicit command repetition, which the consumer currently tracks separately) in the same loop as the token boundaries. The pattern is global and unanchored, so the loop must skip separators (commas and whitespace) the way the engine currently does implicitly.',
  },
  {
    key: 'binding names are parser-produced identifiers',
    text: '`/^[A-Z][A-Z0-9_]*$/u` over a parser-produced identifier is two character-range compares in a linear walk: the first character is within `A-Z`, and every remaining character is within `A-Z`, `0-9` or `_`. The grammar is already fixed by the parser, so the pattern adds nothing but a construction.',
  },
  {
    key: 'Anchored IPv4 token classification',
    text: 'The regex is unnecessary either way, but the justification mislabels it. The function documents its own contract as "True when token is digits/dots or contains IPv6 colon", and the implementation matches that contract, so this is a lexical classifier rather than an IPv4 validator; calling it "Anchored IPv4 token classification" overstates it. Measured, `/^[0-9.]+$/u` accepts `...`, `.`, `1..2`, `1.2.3.4.5` and `999.999.999.999`, and rejects `dns.google`. The replacement is one linear character scan testing digits and dots, which is what the code means today; adding octet counting or range checks would be a behaviour change and is not part of this issue. The "avoids rescanning giant AllowedIPs values" claim is a performance argument for a per-token test that a character loop answers in the same single pass.',
  },
  {
    key: 'format check on `host:port`',
    text: 'The assertion is "host is `127.0.0.1` and the port is all digits": split at the last `:`, compare the host literal, and walk the port characters. Linear over a bounded address string, and the failure output can name which half mismatched instead of reporting that a whole pattern failed.',
  },
  {
    key: 'Escaping metacharacters in a short bare package name',
    text: `This escape pass exists only to feed the \`new RegExp\` two sites below. With a predicate form of rolldown's \`external\` there is no pattern to escape, so the function becomes dead code rather than needing a faster implementation. ${  ROLLDOWN_EXTERNAL}`,
  },
  {
    key: 'Dynamic source is one metacharacter-escaped bare package name',
    text: `The constructed pattern expresses "this bare name, or any subpath under it", which is \`id === name || id.startsWith(name + '/')\`. ${  ROLLDOWN_EXTERNAL  } The same file already has a string-based \`matchesBundlePattern\` for the glob form, so the predicate form is the incumbent idiom here, not a new one.`,
  },
  {
    key: 'Test assertion matches a short literal substring in a thrown error message; expect API requires a RegExp',
    text: `The stated reason is incorrect for this repo, and the site uses \`.toThrow\`, not \`.toMatch\`. ${  THROW_STRING  } The pattern \`/closed/u\` has no metacharacters, so the string form asserts the same thing and removes the regex from the feature.`,
  },
  {
    key: String.raw`Unicode property class \p{Upper}`,
    text: 'The claim "no string API exposes the Unicode upper-letter set" is true only for exact property semantics. Case mapping is a string API carrying the same Unicode data, and for a single grapheme `char.toLowerCase() !== char && char.toUpperCase() === char` is an O(1) test. Corrected probe over 15 samples (ASCII, Greek sigma forms, dotted I, two modifier letters, a Roman numeral, a digit, punctuation): the uppercase comparison showed no divergence in this sample, and the lowercase comparison diverged on U+1D2C and U+1D43. A 15-sample probe cannot establish that those are the only divergences, so this site is recorded as a judgement call rather than a settled finding: an exact property-table lookup is a regex-free implementation at the same complexity, and the choice between a table and case mapping is an owner decision about which semantics the classifier wants.',
  },
  {
    key: String.raw`Unicode property class \p{Lower}`,
    text: 'The claim "no string API exposes the Unicode lower-letter set" is true only for exact property semantics. Case mapping is a string API carrying the same Unicode data, and for a single grapheme `char.toUpperCase() !== char && char.toLowerCase() === char` is an O(1) test. Corrected probe over 15 samples: the lowercase comparison diverged on U+1D2C and U+1D43 (category Lm), where `\\p{Lower}` matches and the case-mapping test does not; the uppercase comparison showed no divergence in the same sample. A 15-sample probe cannot establish that those are the only divergences, so this is a judgement call: an exact property-table lookup is a regex-free implementation at the same complexity, and whether modifier letters must classify as lowercase is an owner decision, not something the current justification states.',
  },
  {
    key: 'assertion verifies absence of base64url-forbidden chars',
    text: 'A three-character forbidden set is one linear character scan (`some` over the string, or an `indexOf` per forbidden character on a fixed-length cursor). "Bounded literal class, no backtracking" is a safety claim about a pattern with no quantifier.',
  },
  {
    key: 'trailing `=` padding strip on a fixed-length base64 string',
    text: 'The `/=+$/u` strip is a backward walk from the end while the character is `=`, then a `slice`; equivalently, `while (text.endsWith(\'=\'))`. Linear over a fixed-length base64 string, and the padding count becomes explicit instead of implied by the quantifier.',
  },
  {
    key: 'anchored prefix on one line of bounded length; matches CommonMark fence opener',
    text: 'The CommonMark opener test is: count leading spaces, fail at four, then `startsWith(\'```\')` or `startsWith(\'~~~\')` at that offset. Linear over one line, and it states the 0 to 3 space rule as a count instead of a `{0,3}` repetition.',
  },
  {
    key: 'extracts the chunk seq from a test fixture URL',
    text: '`/\\/chunks\\/(\\d+)/u` over a fixture URL is `indexOf(\'/chunks/\')` plus a digit walk from that offset, then `Number` on the slice. Linear, and the "single bounded capture" becomes a named slice. The pattern is unanchored, so the scan searches anywhere in the URL rather than testing its start.',
  },
  {
    key: 'parses "chunk N of M" from server-rendered nav fragment',
    text: 'The phrase parse is `indexOf(\'chunk\')`, skip whitespace, skip a digit run, expect ` of `, then read the captured digit run: one linear pass over a bounded fragment, with the two numbers as named slices instead of positional captures.',
  },
  {
    key: 'Anchored literal-prefix strip on a bounded request pathname',
    text: 'A literal-prefix strip is `startsWith(\'/_fragments/\')` plus a `slice` of the known prefix length: one comparison and one allocation, no pattern construction, and no quantifier to reason about. "One-shot match, no quantifier means linear time" is a safety claim about a pattern whose only content is a literal.',
  },
];

/**
 Rebuttal text for a justification, or null when the justification is not one of
 the recorded ones.
 */
export function rebuttalFor(justification) {
  if ((justification === null) || (justification === undefined)
    || (justification === '')) return null;
  for (const rule of RULES) {
    if (justification.includes(rule.key)) return rule.text;
  }
  return null;
}

export const SAFETY_ONLY_FALLBACK = 'The recorded justification argues that matching is safe (bounded input, no nested quantifiers, no backtracking). Safety is not the question the rule asks, and it is not the audit criterion: the question is whether a regex-free implementation exists at similar or better time complexity. For this pattern it does, so the justification does not establish necessity.';

export { RULES };
