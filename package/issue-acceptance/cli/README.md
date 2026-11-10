# @monochromatic-dev/issue-acceptance-cli

Settle GitHub issues against what a branch actually holds.

A session reads an issue,
finds what fixed it,
and then has to say so on the issue without guessing.
This command takes that work as data:
one plan file names each issue,
the checks that decide it,
and the words the issue receives.
The command decides every check on the branch first,
and writes on GitHub only when asked.

It replaces the coordinator's scratch scripts that closed issues one round at a time,
listed in `doc/planning/scratch-scripts-folded-into-packages.md`.

## Commands

```bash
# Decide every issue of a plan and print each verdict; write nothing.
issue-acceptance settle --plan plan.json

# Settle the issues whose checks hold.
issue-acceptance settle --plan plan.json --apply

# Print one issue whole, with its comments.
issue-acceptance show --repo Aquaticats/Monochromatic 1234

# Print what each issue asked for, so a closing comment can be checked against it.
issue-acceptance show --repo Aquaticats/Monochromatic 1234 1235 --acceptance

# Group open bug and ready-for-agent issues by the first package each names.
issue-acceptance ready-bugs --repo Aquaticats/Monochromatic --out grouping.json
```

`settle` exits `0` when every check held and every write succeeded,
`1` when a check did not hold or a GitHub call failed,
and `2` when the invocation or the plan is wrong.

## The plan

A plan is JSON.
Every key is required except `commitCorrections`,
and a key this schema does not define fails the run,
so a misspelled `checks` cannot silently settle nothing.

```json
{
  "repository": "Aquaticats/Monochromatic",
  "checkout": "/var/mnt/encrypted/repository/Monochromatic",
  "branch": "main",
  "issues": [
    {
      "issue": 1251,
      "action": "close",
      "checks": [
        { "kind": "absent", "pattern": "for \\(let ", "paths": ["package/cloudflare-worker/rand/src"] },
        { "kind": "onBranch", "commit": "2e00b12885" }
      ],
      "comment": "Fixed on main as 2e00b12885: a one-word draw now reduces by remainder."
    }
  ],
  "commitCorrections": [
    { "commit": "2b309f8f25", "body": "Correction to this commit's body: it names only the test." }
  ]
}
```

`action` is `close` or `comment`.
`comment` leaves the issue open,
which is how a partly done issue keeps its remaining work in view.

### Checks

Every check is one question about the branch,
and each returns the words a failure needs.

- `absent`:
  the pattern matches nowhere under `paths`.
  Paths are Git pathspecs,
  so `:!*.test.ts` narrows them and `src/*.test.ts` widens one file to many.
- `present`:
  the pattern matches somewhere under `paths`.
- `exactly`:
  the matches under `paths` number exactly `count`.
- `atLeast`:
  the matches under `paths` number at least `count`.
- `exists`:
  the branch tracks `path`.
- `gone`:
  the branch no longer tracks `path`.
- `fileContains`:
  the text of `path` holds `text`.
- `fileLacks`:
  the text of `path` does not hold `text`,
  or the branch has no such path.
- `pathMatching`:
  no tracked path matches the pattern,
  which is how a renamed-away file is proved gone.
- `onBranch`:
  `commit` is an ancestor of the branch.

Patterns for the count checks are POSIX extended expressions,
which `git grep -E` evaluates;
`pathMatching` compiles its pattern as a JavaScript expression.

## Safety

- Without `--apply`,
  nothing is written.
  The run decides every check and prints `would close` or `DOES NOT HOLD`.
- With `--apply`,
  the issue's state is read first.
  An issue that is not `OPEN` is left alone and named.
- The branch is proved readable before any verdict:
  a branch that tracks no path fails the whole run,
  because that means the checkout or the branch name is wrong.
- Every child process has a fixed deadline,
  supplies no inherited standard input,
  and receives its arguments as a vector,
  never a shell string.
- An issue is settled once:
  a plan that names one issue twice is refused.

## Why some regular expressions carry a suppression

Three patterns are regular expressions because a scan would be longer and no faster.
Each carries a scoped `oxlint-disable` naming its bound,
as `AGENTS.md` rule `LN3` asks,
and this section is the record it asks for:

- `src/ready-bugs.ts`,
  `PAUSED_WORDS` and `PACKAGE_PATH`:
  one match over one issue's title or body,
  bounded by that issue.
  No nested quantifier,
  so matching stays linear.
- `src/show.ts`,
  `ACCEPTANCE_HEADING`:
  one anchored prefix test over one heading line of an issue body.
- `src/verdict.ts`,
  the `pathMatching` compile:
  the plan supplies the pattern at run time,
  so it cannot be a literal.
  The pattern comes from a reviewed plan file and runs over the branch's tracked path list.
- `src/settle.ts` and `src/verdict.ts` carry a file-scope `no-await-in-loop` suppress
  because a plan is applied one entry at a time:
  the ordering is the design,
  and `doc/lint/sequential-awaits.md` records why.

## Development

```bash
mise run //package/issue-acceptance/cli:buildAndTest
mise run //package/issue-acceptance/cli:lint
mise run //package/issue-acceptance/cli:lint:types
```

The unit suites drive the real child-process boundary with fake `git` and `gh` executables
that record the argument vectors they receive,
and drive the settlement itself with a fake branch reader and a fake tracker.
No suite reaches a real repository or a real issue.
