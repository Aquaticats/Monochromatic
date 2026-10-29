# forbidden-strings 0.4.1 built-in `hashicorp-tf-password` matches synthetic fixture tokens in JSON receipts, blocking every merge that brings the file in

## Symptom

A merge commit that brings `doc/audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json`
into a line of history that lacked it is refused by the commit guard,
 even though the file was already on `origin/main`:

```text
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward","policyId":"security/forbidden-strings","severity":"error","code":"security/forbidden-strings/forbidden-string","message":"Forbidden string matched at line 2883 (rule hashicorp-tf-password).","path":"doc/audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json","fix":"none"}
```

The same finding repeats for lines 2900 and 2917.
Made from the branch's side (`git merge origin/main` in a branch worktree),
 the commit is refused before it lands (`trigger: pre-forward`).
Made from `main`'s side,
 the commit lands,
 but the post-commit pass reports the same three lines and holds the automatic push:

```text
{"schemaVersion":1,"sequence":3,"type":"commit-landed","oid":"19f6aa50ecb3bf2a88bbaec55a89fb5f305b04f8","outcome":"post-commit-blocked","message":"Commit 19f6aa50ecb3bf2a88bbaec55a89fb5f305b04f8 remains local; post-commit gate blocked automatic backup."}
```

The matched values are not credentials.
Each is the synthetic token a test case selected,
 forwarded as a WebDAV password to a `.invalid` endpoint:

```text
doc/audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json:2861 (after the workaround)
"selectedSyntheticToken": "fixture-explicit-token",
"forwarded": {
  "RUSTC_WRAPPER": "[home]/temp/agent/depot-cargo-wrapper-boundary-zCmXni/bin/sccache",
  "SCCACHE_WEBDAV_ENDPOINT": "https://fixture-cache.invalid",
  "SCCACHE_WEBDAV_USERNAME": "fixture-owned-org",
  "SCCACHE_WEBDAV_PASSWORD": "<same as selectedSyntheticToken>"
}
```

The password line is shown here as it reads after the workaround;
 before it,
 the value repeated the selected token.
This case's token is the 22-character one the rule never reported;
 the three reported cases used shorter tokens of the same shape.

## Root cause

### The rule is a Terraform rule that runs on every file

The built-in rule is ported from betterleaks,
 `package/cli/forbidden-strings/data/builtin-rules.txt:407-411`:

```text
# package/cli/forbidden-strings/data/builtin-rules.txt:407
==> hashicorp-tf-password <==
# Identified a HashiCorp Terraform password field, risking unauthorized infrastructure configuration and security breaches.
# NOTE: upstream restricts this rule to files matching: (?i)\.(?:tf|hcl)$
#       Forbidden-strings has no per-rule path scoping; the rule fires on every scanned file.
/(?:(?:(?:administrator_login_password)|(?:Administrator_Login_Password)|(?:ADMINISTRATOR_LOGIN_PASSWORD))|(?:(?:password)|(?:Password)|(?:PASSWORD)))(?:[ \t\w.-]{0,20})[\s'"]{0,3}(?:=|>|(?::{1,3}=)|(?:\|\|)|:|(?:=>)|(?:\?=)|,)[`'"\s=]{0,5}(?:"[a-zA-Z0-9=_\-]{8,20}")(?:(?:\\?['"`])|[\s;]|(?:\\[nNrR])|$)/
```

A JSON key ending in `PASSWORD`,
 a colon,
 and a quoted value of 8 to 20 characters from `[a-zA-Z0-9=_\-]` satisfies it.
Three of the receipt's four non-empty values were 17 or 19 characters long;
 the fourth,
 at 22,
 is longer than the rule's 20-character limit and was never reported.
The scanner states both missing features in `package/cli/forbidden-strings/README.md:91` and `:95`:
 no per-rule path scoping and no per-rule allowlists.

### A merge is scanned against every parent

`forbidden-strings` is registered for every content trigger,
 including manual push,
 `package/git-policy/cli/src/native/policy_trigger.rs:138-145`:

```rust
// package/git-policy/cli/src/native/policy_trigger.rs:138
        PolicyId::ForbiddenStrings => {
            return &[
                Trigger::PreForward,
                Trigger::PostCommit,
                Trigger::ManualPush,
                Trigger::DirectCheck,
            ];
        }
```

and the manual-push scope is each new commit's own delta against all of its parents,
 `package/git-policy/cli/SPEC.md:3068-3070`:

```md
<!-- package/git-policy/cli/SPEC.md:3068 -->
Scan each newly published commit's own delta against its parents:
every parent for merges,
the whole tree only for parentless commits,
```

So any true merge of a line that lacks the file with one that has it carries the file in its delta against the parent that lacked it,
 whichever side the merge is made from,
 and there is no skip for a blob the remote already holds.
The guard skips explicit dry runs (`package/git-policy/cli/SPEC.md:3079`),
 so a `git push --dry-run` cannot preview the refusal.

## Verification

Scanner build under test:
 `forbidden-strings 0.4.1`
 (`package/cli/forbidden-strings/target/release/forbidden-strings --help`;
 the source tree reads `0.5.0`).
The harness `s245/split/scan-control.ts` in the session's scratch directory ran this from an empty directory,
 so that only the built-in rules load:

```sh
# from an empty directory, once per input
forbidden-strings --builtin-rules --name-path doc/audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json <file>
```

Results on 2026-10-19:

- Original blob `9e10ec7fb05db0959b851a00f5ddc390bc7c0028` (as of `origin/main` `331cc20126`):
   exit `1`,
   findings at lines 2883,
   2900 and 2917,
   rule `hashicorp-tf-password`.
- Redacted file:
   exit `0`.

Values that match the rule:
 any quoted run of 8 to 20 characters drawn only from letters,
 digits,
 `=`,
 `_` and `-`,
 such as a hyphenated three-word fixture token.
Values that do not:
 shorter than 8 or longer than 20 such characters,
 or containing any other character,
 such as the `<`,
 `>` and spaces of `<same as selectedSyntheticToken>`.

## Verified workarounds

### Replace the fixture value with a placeholder that names its source

Applied on 2026-10-19 on the owner's choice,
 to make the merge of `translation-repair-rebased` into `main` pushable:
 each non-empty `SCCACHE_WEBDAV_PASSWORD` value in the receipt became `<same as selectedSyntheticToken>`.
The rewrite refused to run unless every replaced value equalled the `selectedSyntheticToken` of its own case,
 so the receipt still says what was forwarded.
The empty value of the case that forwarded nothing is unchanged.

Tradeoffs:

- The receipt is no longer the campaign's byte-exact output.
   The original stays in history as blob `9e10ec7fb05db0959b851a00f5ddc390bc7c0028`
   (`git show 331cc20126:doc/audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json`).
- Three other receipts pin the original as an input,
   by SHA-256 `5fd82136c26b007aba1dc7c7484d9bb974ac84bf568738e649a3b09a923fd72b` or the blob id above:
   `depot-owned-allocation-ledger-2026-10-18.evidence.json`,
   `depot-rust-legacy-input-version-joins-2026-10-18.evidence.json`
   and `depot-rust-legacy-owner-search-2026-10-18.evidence.json`.
   Those pins were left as written:
   they describe the input those audits read,
   which is the original blob,
   not the current file.
- A future campaign that writes fixture tokens shaped like this into a receipt trips the same rule;
   the producer should write the placeholder form at capture time.

## What does not work

- `--no-enforce-*` flags.
   None reaches `security/forbidden-strings`
   ([forbidden-strings-appendix-self-match.md](forbidden-strings-appendix-self-match.md) enumerates them).
- Making the merge from the other side.
   It moves the refusal from before the commit to after it,
   because the scan covers every parent.
- Checking with `git push --dry-run` first.
   Explicit dry runs bypass the manual-push policies,
   so the dry run passes and proves nothing.

## Upstream filing decision

Nothing is filed:
 both the scanner and the commit guard are repository-owned
 (`package/cli/forbidden-strings`,
 `package/git-policy/cli`),
 and the upstream betterleaks rule is not at fault,
 because it is scoped to `.tf` and `.hcl` files,
 as `builtin-rules.txt:409` records.
Walking the six constraints for the ported rule:

1. Upstream's fault:
    no;
    the over-match comes from this repository's port dropping the path scope.
2. Upstream could fix it:
    nothing to fix upstream.
3. Supported use case:
    upstream supports the rule only for Terraform files.
4. Contribution welcome:
    not assessed,
    since there is nothing to contribute.
5. Likely fix:
    not applicable.
6. Prototype:
    none,
    for the same reason.

The durable local fix is per-rule path scoping or an allowance mechanism in the scanner,
 which the README lists as missing;
 adding it changes a security scanner's skip logic and was not chosen for this incident.

## Update 2026-10-29: a merge of content a parent already holds is no longer refused

The refusal in this document came from the scan reading a merge commit's delta against every parent.
A file that one parent lacks and the other holds is an addition against the first parent,
so a merge that only joined two lines of history rescanned a file that was already on `origin/main`.

Fixed in the source of `monochromatic-git-policy-cli` (`src/native/candidate_known_content.rs`):
the policy no longer reads the bytes of a candidate whose exact blob any parent commit's tree already holds.
That covers the pre-forward scan of a commit (against its recorded base),
the post-commit scan of a landed merge (against every parent),
and the manual-push scan (against the parents the remote already had).
The merge described here would have passed,
because the receipt's blob was already in the other parent.
`package/git-policy/cli/SPEC.md` ("Content already in history") has the rule and its security reasoning.
The merge controls are in `src/native/policy_forbidden_strings_known_content_tests.rs`;
the one for a merge whose result holds a file that only one parent has failed on unchanged code before the rule existed.

What this does not change:

- The fix takes effect only in an installed wrapper built from a release that carries it.
- Content that is new in any way is still scanned.
   A receipt whose bytes change,
   or a first commit that adds such a receipt,
   still trips the over-matching rule,
   so the producer should still write the placeholder form at capture time.
- Redacting the receipt remains the way to make changed content pass.
