# forbidden-strings 0.3.0: a literal rule body in the appendix matches itself, failing every later commit that touches the file

## A note on how this document spells things

Every banned identifier below is written with one character bracketed,
 `Oxc[C]omment` for the identifier
`Oxc` followed by `Comment`.
 That is not a typo and not the identifier's real spelling:
 it is the same
workaround the document describes,
 applied to the document.
 Spelling the identifiers plainly made this file fail
the very rules it is about,
 on three lines at once,
 which is the most direct demonstration of the problem
available.
 Remove the brackets when copying anything from here into a real file or command.

## Symptom

Editing `forbidden-strings.append.txt` fails the commit on a line the edit never touched:

```text
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward",
"policyId":"security/forbidden-strings","severity":"error",
"code":"security/forbidden-strings/forbidden-string",
"message":"Forbidden string matched at line 138 (rule crn-renamed-oxc-comment).",
"path":"forbidden-strings.append.txt","fix":"none"}
```

Line 138 was the body of the `crn-renamed-oxc-comment` rule itself,
 a bare literal.
 The commit being attempted
appended an unrelated rule further down the file.

`git commit --no-enforce-security` does not help;
 no such escape exists for this policy.
 The `--no-enforce-*`
flags `cli-git` accepts are `--no-enforce-only`,
 `--no-enforce-bulk-add`,
 `--no-enforce-require-root`,
`--no-enforce-worktree` and `--no-enforce-worktree-branch`,
 none of which reaches
`security/forbidden-strings`.

## Root cause

The scanner has no notion of a file that is exempt from scanning,
 so it reads its own rule file as ordinary
content.
 Stated in the tool's own README,
 `package/forbidden-strings/cli/README.md:95`:

```md
- **Per-rule path scoping**.
   Every rule runs against every non-skipped file;
   the scanner
  cannot apply rule X only to YAML files.
- **Per-rule allowlists**.
   No way to say "rule X but skip when it matches in path Y".
```

The policy is registered without any path exclusion,
 `cli-git.config.ts:27`:

```ts
    'security/forbidden-strings': [
      'error',
      {
        executable: './package/forbidden-strings/cli/target/release/forbidden-strings',
        builtinRules: true,
      },
    ],
```

So a rule whose body is the bare text `Oxc[C]omment` is satisfied by its own definition.
 Nothing detects this at
authoring time,
 because the policy runs against staged paths:
 the file passes until something stages it,
 and
then it fails for reasons unrelated to the change.

An earlier reading of this was wrong and is worth naming:
 the first assumption was that the newly added rule was
malformed and had broken section parsing for the rules above it.
 It had not.
 The failing rule was years older
than the edit,
 and the added section was last in the file,
 so it could not affect anything above it.
 The tell was
that the reported line number pointed at existing content.

## Verification

Version under test:
 `forbidden-strings 0.3.0`,
 built at
`package/forbidden-strings/cli/target/release/forbidden-strings`.

Harness,
 run against a disposable directory rather than the repository:

```bash
D=$(mktemp --directory "${HOME}/temp/agent/fsprobe.XXXXXXXX")
cat forbidden-strings.append.txt forbidden-strings.append.local.txt > "$D/rules.txt"
printf 'const Oxc[C]omment = 1;\n' > "$D/hit-one.ts"
printf 'const ParsedComment = 3;\n' > "$D/clean.ts"
for f in hit-one clean; do
  printf '%-12s ' "$f"
  ./package/forbidden-strings/cli/target/release/forbidden-strings \
    --rules "$D/rules.txt" "$D/$f.ts" > /dev/null 2>&1 \
    && echo "exit 0 (no violation)" || echo "exit 1 (violation)"
done
printf '%-12s ' "appendix"
./package/forbidden-strings/cli/target/release/forbidden-strings \
  --rules "$D/rules.txt" forbidden-strings.append.txt > /dev/null 2>&1 \
  && echo "exit 0 (no violation)" || echo "exit 1 (violation)"
rm --recursive --force "$D"
```

Patterns that self-match,
 so the appendix fails on its own content,
 shown here unbracketed in description only:

-   a bare literal body,
     the identifier `Oxc[C]omment` written with no brackets at all;
-   a regex body whose pattern text contains the string it matches,
     an alternation of the same identifiers
    written with no brackets.

Patterns that do not self-match,
 while matching the same target text,
 and these are literal:

-   `/Oxc[C]omment/`;
-   `/(?:INERT_MEMBERS_BY_INTERFAC[E])|(?:isInertCollectionMembe[r])/`;
-   any anchored pattern whose anchor cannot hold inside a comment line,
     such as the `^CODE:` shortcode rules,
    whose per-code rationale comments are written as `# INF: ...` and so never match at line start.

After rewriting both self-matching rules,
 the harness prints `exit 1` for each banned identifier,
 `exit 0` for
each replacement identifier,
 and `exit 0` for the appendix itself.

## Verified workarounds

Bracket one character of the pattern so the pattern text no longer matches the pattern:

```diff
 ==> crn-renamed-oxc-comment <==
 # Renamed to ParsedComment when mutation-test moved from oxc-parser to
 # yuku-parser; the old name misattributes the producer of the comment shape.
-Oxc[C]omment      <- was the bare identifier, unbracketed
+/Oxc[C]omment/    <- now a regex, bracketed exactly as shown
```

Tradeoff:
 the rule body no longer reads as the plain string it bans,
 so a reader has to know the convention to
see what is banned.
 That cost is paid once per rule and is why the appendix header now states the convention
above the first affected section.
 It also means a rule can no longer be copied verbatim out of the file as a
search term.

Applies to the checked-in appendix and equally to `forbidden-strings.append.local.txt`,
 whose bare-literal
sections have the same exposure the moment that file is scanned.

## What does not work

-   `git commit --no-enforce-security`.
     Not a flag;
     the policy has no escape hatch.
     Confirmed by enumerating the
    `--no-enforce-*` strings in `package/git-policy/cli/src`.
-   Writing the new rule as a regex while leaving older literal rules alone.
     The trip is on pre-existing content,
    so any edit to the file still fails until every self-matching rule is fixed.
-   Moving the rule to `forbidden-strings.append.local.txt`.
     That file is gitignored,
     so a `CRN` reservation put
    there never reaches other clones,
     and the same self-match applies to it anyway.
-   Relying on the policy to catch this at authoring time.
     It runs against staged paths,
     so an unstaged appendix
    edit looks fine and fails later.

## Upstream filing decision

`forbidden-strings` is a repository-owned package under `package/forbidden-strings/cli`,
 not third-party,
 so the
six-constraint upstream check does not apply and there is nothing to file on an external tracker.
 The
convention-level fix is landed;
 the tool-level fix is recorded below as a change this repository could make to
its own code.

The durable improvement would be for the scanner,
 or the `cli-git` policy wrapping it,
 to skip the resolved rules
file and its appendixes when walking.
 That removes the need for the bracketing convention entirely.
 It is not
attempted here because it changes a security scanner's skip logic,
 which deserves its own decision about whether
a file being a rule source should ever exempt its content:
 a rule file that legitimately contains a real secret
as a pattern is exactly the case the current behaviour catches.
