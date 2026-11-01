# Fastly CLI 16.1.0 `profile list` prints every stored API token unmasked, and its own deprecation notice names a replacement that does not

**Date**:
 2026-10-09.
**Subject**:
 `fastly profile list` writes the stored API token for every profile to
standard output in cleartext,
with no masking and no confirmation,
and in
`--json` mode writes the entire token map including refresh tokens.
The
command is deprecated in favour of `fastly auth list`,
which prints no secret
at all.
Discovered when the token for a live account landed in an agent
transcript during
[`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md)
work.

## Symptom

```text
$ fastly profile list
DEPRECATED: This command will be removed in a future release. Use 'fastly auth list' instead.

INFO: Default profile highlighted in red.

<profile-name>

Default: true
Email: <account-email>
Token: <32-character API token, in full>
SSO: true
Account ID: <customer-id>
Label: N/A (<account-email>)
```

The token is printed complete.
It is not truncated,
not masked with a prefix,
and not gated behind a confirmation or a `--show-token` flag.
The output goes
to standard output,
so any redirect,
pipe,
`tee`,
terminal recorder,
or agent
tool call that captures stdout captures the credential.

The command's own first line tells the reader to use a different command,
which
makes the leak easy to hit by accident:
a caller looking for "which profiles
exist" reaches for `profile list` because the name describes that,
reads the
deprecation notice only after the secret is already on the terminal,
and has no
reason to expect a listing command to print a credential.

In JSON mode the exposure is wider.
`fastly profile list --json` writes the
whole `Auth.Tokens` map,
which for an SSO profile includes the long-lived
`access_token` and `refresh_token` alongside the legacy API token.

## Root cause

Cloned per this repository's convention:

```bash
gh repo clone fastly/cli "${HOME}/temp/agent/fastly-cli-2026-10-09" -- --depth 1
```

At commit `bb93017`,
`pkg/commands/profile/list.go:79` formats the token
field directly into the output writer:

```go
text.Output(out, "%s: %s", style("Token"), at.Token)
```

`style` is a colouriser passed by the caller,
`text.BoldRed` for the default
profile and `text.Bold` for the others,
so the styling argument decorates the
value rather than redacting it.
Nothing between the config struct and the
writer transforms `at.Token`.

`pkg/commands/profile/list.go:39` is the JSON path,
and it serialises the
entire token collection:

```go
if ok, err := c.WriteJSON(out, c.Globals.Config.Auth.Tokens); ok {
```

`pkg/commands/profile/list.go:32` emits the deprecation notice,
and it fires
only when neither `--quiet` nor `--json` is set:

```go
text.Deprecated("This command will be removed in a future release. Use 'fastly auth list' instead.\n\n")
```

So in `--json` mode the caller gets the wider exposure and no notice.

The named replacement does not print secrets.
`pkg/commands/auth/list.go:73`
is the whole of its per-profile output:

```go
text.Output(out, "%s%s (%s)%s%s\n", marker, name, info, reauthStr, expiryStr)
```

`marker` is `* ` or two spaces,
`name` is the profile name,
`info` is the
token type or the account email,
and the last two are a re-authentication flag
and an expiry summary.
No token field is read.

The likely reason `profile list` prints the token at all is that a human
setting up a second machine wants to copy it.
That use case is already served,
twice over,
by commands in the replacement family,
which makes the unmasked
listing an inconsistency rather than a considered design.
`fastly auth --help`
in 16.1.0 lists:

```text
token   Output the active API token (for use in shell substitutions)
show    Show details for a stored token
```

and `pkg/commands/auth/show.go:30` registers an explicit opt-in for the value:

```go
c.CmdClause.Flag("reveal", "Show the full token value (use with care)").BoolVar(&c.reveal)
```

`pkg/commands/auth/show.go:109` to `pkg/commands/auth/show.go:116` masks
unless that flag is set:

```go
if c.reveal {
	text.Output(out, "Token: %s\n", entry.Token)
} else {
	if len(entry.Token) > 8 {
		text.Output(out, "Token: %s...%s\n", entry.Token[:4], entry.Token[len(entry.Token)-4:])
	} else {
		text.Output(out, "Token: ****\n")
	}
}
```

So the codebase already contains both the deliberate-retrieval command and the
mask-by-default shape.
The deprecated command has neither.

## Verification

Fastly CLI 16.1.0,
installed through `mise` at
`~/.local/share/mise/installs/npm-fastly-cli/latest/bin/fastly`.
Source read at
commit `bb93017` of `fastly/cli`.
Measured on 2026-10-09 against a live
account with one SSO profile.

Reproduce without printing the secret by measuring lengths instead:

```bash
fastly profile list 2>/dev/null | awk -F': ' '/^Token:/ {print "token chars:", length($2)}'
fastly auth list    2>/dev/null | grep -c -i 'token:' || echo "auth list prints no token field"
```

The first prints a non-zero character count.
The second prints `0` and the
fallback line.

### Catalog: commands that print the credential

- `fastly profile list`
- `fastly profile list --json`,
   which additionally exposes the SSO
  `access_token` and `refresh_token`

### Catalog: commands that do not

- `fastly auth list`,
   the documented replacement
- `fastly whoami`,
   which prints the account display name and email only
- `fastly domain list`,
   `fastly service list`,
   and the other resource
  listings

## Verified workarounds

### 1. Use `fastly auth list`

It answers the question `profile list` appears to answer,
which profiles exist
and which is default,
and adds expiry and re-authentication status that
`profile list` omits.

Tradeoff:
none for inventory purposes.
It does not print the token,
so a
workflow that was copying the token out of `profile list` needs a different
source.

### 2. Read the token from the config file when the value is actually needed

`~/.config/fastly/config.toml` holds it under `[auth.tokens.<profile>]` as
`token`.
Read it in-process and never echo it:

```bash
node --input-type=module -e '
import {readFileSync} from "node:fs";
import {homedir} from "node:os";
const t = readFileSync(homedir() + "/.config/fastly/config.toml", "utf8");
const token = t.match(/^\s*token\s*=\s*"([^"]+)"/m)[1];
console.log("token loaded, length", token.length);'
```

Tradeoff:
it depends on the config file's layout,
which is versioned by the
`config_version` key at the top of the file and can change.
It keeps the secret
out of stdout,
which is the point.

### 3. Rotate after an exposure

A token that reached a transcript,
a log,
or a CI artifact should be treated
as compromised and replaced through the Fastly control panel's API token page.

Tradeoff:
every automation using the old token must be updated in the same
window.

## What does not work

- **Relying on the deprecation notice as a warning.**
   It names a
   replacement;
   it does not say the current command prints a secret,
   and
   it is emitted after the header rather than before the token line.
- **`--quiet` as a redaction mechanism.**
   The guard at
  `pkg/commands/profile/list.go:31` wraps only the `text.Deprecated` call.
   The `display` calls that print the token are unguarded,
   so `--quiet`
   removes the notice and keeps the secret.
- **Assuming the JSON path is safer because it is machine-readable.**
   It
   serialises strictly more secrets,
   including the SSO refresh token,
   and
   skips the deprecation notice.
- **Treating this as an agent-specific hazard.**
   Any shell history logger,
   terminal multiplexer capture,
   or CI step that records stdout has the same
   exposure.

## Upstream filing decision

`.out-of-scope/` was checked:
eleven files,
none matching Fastly,
a CDN
provider,
a credential-handling bug class,
or a deprecated-command defect.
The
two whose text contains the word "token",
`low-impact-typescript-formatting.md`
and `pi-gpt55-long-context.md`,
use it in unrelated senses,
respectively
lexer-level formatting and model context length.

The upstream tracker was searched for a duplicate before drafting,
with
`gh search issues` and `gh search prs` against `fastly/cli` for
"profile list token",
"token cleartext",
"mask token",
"print api token",
and
"redact token".
All seven queries returned nothing.
Two positive controls
confirm the search itself works rather than failing silently:
`gh search issues --repo fastly/cli "domain"` returns five hits,
and `"token"`
returns five more.
Note that `--state all` is rejected by these commands,
which
accept only `open` or `closed`,
so the searches ran with no state filter and
cover both.

The nearest threads are related context and not duplicates.
[fastly/cli#1709](https://github.com/fastly/cli/issues/1709)
asks for a token-only output mode to replace the deprecated
`fastly profile token`,
citing
[fastly/cli#1708](https://github.com/fastly/cli/issues/1708),
and a maintainer
replies that it is in progress in
[fastly/cli#1690](https://github.com/fastly/cli/pull/1690).
All three are about the ergonomics of retrieving a token deliberately.
None is
about a listing command emitting credentials the caller did not ask for,
and
none mentions `profile list`.
Nothing additive belongs on those threads,
 so no
comment draft is kept for them.

An earlier version of this section ended at "do not file" and recorded
constraint 6 as "No".
That audit was incomplete:
 it was written before the
auto-prototype requirement in
`.agents/skills/troubleshooting-doc/SKILL.md`
had been read,
 and it treated `SECURITY.md`'s private reporting channel as a
constraint-4 failure when that channel is scoped to security vulnerabilities.
Both are corrected below,
 and the decision changed.

1.  **Is it really upstream's fault?**
    Yes.
    The earlier reading,
     that
    printing a stored credential is a design choice with a foreseeable failure
    mode,
     does not survive the successor command's source.
    `pkg/commands/auth/show.go:109`
    masks by default behind an explicit `--reveal`
    documented "use with care",
     and `fastly auth token` exists for the
    deliberate copy-it-to-another-machine case.
    The deprecated command is
    inconsistent with the family replacing it,
     not an alternative expression of
    the same intent.
    Suppressing the deprecation notice under `--json` while
    that mode serialises strictly more credentials is a second,
     separate
    inconsistency,
     and `text.Deprecated` writes to `os.Stderr`
    (`pkg/text/text.go:229`),
     so the suppression was not even buying clean
    stdout.
2.  **Can upstream fix it?**
    Yes,
     in `fastly/cli`,
     and the fix is
    prototyped under constraint 6.
3.  **Are they supporting this use case?**
    Yes.
    `fastly auth list` is
    maintained,
     documented as the replacement,
     and prints strictly more useful
    status than the command it replaces.
    `ISSUES.md` lists "A command produces
    incorrect output or wrong results" and "Commands or flags don't work as
    documented" as bug types,
     and this is the first:
     the command's own
    deprecation notice points at a replacement that behaves differently on the
    same field.
4.  **Would the repo welcome our contribution?**
    Yes,
     through
    the public tracker.
    `CONTRIBUTING.md:4` invites opening an issue to discuss
    before a pull request.
    `SECURITY.md` routes **security issues** to a
    private process at
    <https://www.fastly.com/security/report-security-issue>;
    this finding is not
    a vulnerability.
    No attacker and no privilege boundary is involved:
     the
    command prints the invoking user's own stored credential to that user's own
    stdout.
    The defect is unmasked output where the sibling command masks,
     so
    the bug channel is the right one.
    Checked `CONTRIBUTING.md`,
     `ISSUES.md`,
    and `SECURITY.md` for a ban on external or AI-assisted reports;
     none was
    found.
    Were this filed as a pull request rather than an issue,
     it would
    disclose AI assistance in preparing the patch.
5.  **Will they likely fix it?**
    Plausible,
     which the skill treats as a
    soft yes rather than a fail.
    Countervailing signal:
     the command is
    deprecated and slated for removal,
     and #1690 is actively reworking the
    `profile` to `auth` migration,
     so removal may overtake a patch.
    Supporting
    signal:
     #1709 shows maintainers engaging on exactly this command family,
    and the fix reuses a shape already present in `auth/show.go`.
    Per the
    skill,
     absence of a definitive signal is not a fail.
6.  **Have we prototyped a minimal fix?**
    Yes.
    Constraints 1 to 5 hold or
    sorta-hold,
     so the auto-prototype requirement fires.

    Cloned into a fresh private directory,
     not a reused clone:

    ```bash
    mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX"
    gh repo clone fastly/cli cli -- --depth 1
    git remote set-url --push origin DISABLED
    ```

    Origin verified as `https://github.com/fastly/cli.git` at `bb93017`,
     the
    commit this document cites,
     before any edit.

    The fix adds a `--reveal` flag carrying `auth show`'s exact help text,
    masks the token in the human-readable path with `auth/show.go`'s existing
    shape,
     redacts `Token`,
     `AccessToken`,
     and `RefreshToken` in the `--json`
    path through a copy that leaves the caller's config untouched,
     and drops the
    `--json` condition from the deprecation notice.
    Two files change,
    `pkg/commands/profile/list.go` and its existing test,
     which encoded the
    leaking behaviour and now also asserts the reveal path.

    The diff is
    [`fastly-cli-profile-list-prints-api-token.patch`](fastly-cli-profile-list-prints-api-token.patch),
    216 lines across the two files.
    It was confirmed to forward-apply to
    pristine `bb93017` bytes extracted with `git show`,
     and the applied result
    was confirmed byte-identical to the tree the tests ran against.

    Verification ran the package's own harness rather than a full build,
     with no
    ambient credentials involved:
     the scenarios copy `testdata/config.toml` into
    a disposable root and point `ConfigPath` at it,
     and its tokens are the
    fixtures `123` and `456`.

    ```bash
    go test ./pkg/commands/profile/ -run TestProfileList -count=1
    ```

    Pre-patch,
     with the original `list.go` and the updated test,
     four scenarios
    fail and the failure output is the leak itself:

    ```text
    --- FAIL: TestProfileList/validate_listing_profiles_works
        "...Email: foo@example.com\nToken: 123\nSSO: false..." doesn't contain
        "foo\n\nDefault: true\nEmail: foo@example.com\nToken: ****"
    --- FAIL: TestProfileList/validate_--reveal_prints_the_full_token_values
        want no error, have "error parsing arguments: unknown long flag '--reveal'"
    --- FAIL: TestProfileList/validate_listing_profiles_with_--json_displays_data_correctly
        "{...\"token\": \"456\"...}" doesn't contain "\"token\": \"****\""
    --- FAIL: TestProfileList/validate_--json_with_--reveal_still_displays_the_full_token_values
        want no error, have "error parsing arguments: unknown long flag '--reveal'"
    ```

    Post-patch:

    ```text
    ok  	github.com/fastly/cli/pkg/commands/profile	0.117s
    ```

    The whole package passes,
     `go vet ./pkg/commands/profile/` is clean,
     and
    `go build ./...` succeeds,
     so nothing unrelated broke.

Decision:
 all six constraints hold,
 so the draft below is fileable as-is.
It
has **not** been filed.
Opening an issue on a third-party tracker is an
external action,
 and this repository's agent guidance requires authorization
for that rather than treating it as implied by writing the draft.

~~~md
Title: `fastly profile list` prints stored API tokens unmasked, while its own named replacement masks them

Labels: bug

**Version**

Fastly CLI version 16.1.0
Source read at commit bb93017.

**What happened**

`fastly profile list` writes the full stored API token for every profile to stdout, unmasked and behind no flag. `pkg/commands/profile/list.go:79`:

    text.Output(out, "%s: %s", style("Token"), at.Token)

The replacement its own deprecation notice names does not. `pkg/commands/auth/list.go:73` prints only name, type or email, re-auth flag, and expiry.

Two related inconsistencies compound it:

- `--json` serialises the whole token map, `pkg/commands/profile/list.go:39`, exposing the SSO `access_token` and `refresh_token` as well as the API token.
- The deprecation notice is suppressed under `--json`, `pkg/commands/profile/list.go:31`, so that mode drops the warning while widening the exposure. `text.Deprecated` writes to `os.Stderr` (`pkg/text/text.go:229`), so the suppression does not buy clean stdout either.

`pkg/commands/auth/show.go` already establishes the intended shape for this field: mask by default, reveal only behind `--reveal`, whose help text reads "Show the full token value (use with care)" (`pkg/commands/auth/show.go:30`, `:109`-`:116`). And `fastly auth token` already covers the deliberate retrieval case. So the deprecated command is inconsistent with the family replacing it.

The practical cost is that any redirect, pipe, `tee`, terminal recorder, or automation harness capturing stdout captures a live credential, from a command whose name and help text promise an inventory.

**Reproduction**

Measures the exposure without printing the secret:

    fastly profile list 2>/dev/null | awk -F': ' '/^Token:/ {print "token chars:", length($2)}'
    fastly auth list    2>/dev/null | grep -c -i 'token:' || echo "auth list prints no token field"

Observed on 16.1.0 with one stored profile: the first prints a non-zero character count, the second prints `0` and the fallback line.

**Suggested fix**

Mirror `pkg/commands/auth/show.go` in `pkg/commands/profile/list.go`:

1. Add a `reveal bool` field and register `--reveal` with the same help text as `auth show`.
2. In `display`, mask `at.Token` unless `reveal`, reusing the `first4...last4` / `****` shape from `pkg/commands/auth/show.go:112`-`:115`.
3. In `Exec`, redact `Token`, `AccessToken`, and `RefreshToken` on a copy of the map before `WriteJSON`, so the caller's config is not mutated.
4. Drop `!c.JSONOutput.Enabled` from the deprecation-notice guard.

The existing `TestProfileList` scenarios encode the current behaviour and need their expectations updated, plus one `--reveal` and one `--json --reveal` scenario.

A patch implementing exactly this, with pre-patch and post-patch `go test ./pkg/commands/profile/ -run TestProfileList -count=1` output, is available on request. Prepared with AI assistance; the reproduction and the test runs above were executed against a real install and a real clone at bb93017.
~~~

The token exposed during the investigation that produced this document was a
legacy API token for a live account,
 32 characters.
It reached an agent
transcript rather than a public artifact.
Rotation was recommended to the
maintainer and is their decision,
since it requires updating every automation
that uses it.

## References

- `fastly/cli` source,
  read at commit `bb93017`:
  <https://github.com/fastly/cli>
- The prototyped fix recorded under constraint 6:
  [`fastly-cli-profile-list-prints-api-token.patch`](fastly-cli-profile-list-prints-api-token.patch)
- Fastly security issue reporting process,
  which `SECURITY.md` names as the
  channel for security vulnerabilities.
   Not the channel for this finding,
  for the reason given under constraint 4:
  <https://www.fastly.com/security/report-security-issue>
- Nearest upstream threads on the `profile` to `auth` migration,
  read in full
  and found not to be duplicates:
  <https://github.com/fastly/cli/issues/1709>,
  <https://github.com/fastly/cli/issues/1708>,
  <https://github.com/fastly/cli/pull/1690>
- Fastly CLI reference index:
  <https://www.fastly.com/documentation/reference/cli/>
- Related:
  [`fastly-api-reads-report-live-resources-absent.md`](fastly-api-reads-report-live-resources-absent.md),
  the API-side findings from the same investigation,
  and
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md),
  the task this surfaced during.
