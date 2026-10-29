# @monochromatic-dev/open-code-review-issue-cli

Create GitHub Issues from validated OpenCodeReview structured output.
The command ingests review output only;
it never launches OpenCodeReview.

## Safety model

Issue creation is explicit and create-only.
The command does not update,
reopen,
close,
or deduplicate Issues from earlier runs.
A repeated input can create duplicates.

Security-classified findings are disclosure-gated.
An OpenCodeReview `security` category is not proof that public disclosure is safe.
Follow [`SECURITY.md`](../../../SECURITY.md) for unresolved suspected vulnerabilities.

Issue bodies retain OpenCodeReview content as active GitHub Markdown.
Headings,
HTML,
links,
mentions,
and fences can affect rendering or notifications.
Code-derived title fallbacks can expose source or secrets in Issue titles and notifications.
Review every preview before publication.

## Prerequisites

- Node.js supported by the workspace.
- GitHub CLI 2.97.0 or newer on `PATH`.
- An authenticated GitHub CLI session with access to the destination repository.
- OpenCodeReview structured JSON or JSONL produced separately.

## Accepted input

Every invocation requires exactly one positional input.
Interactive mode accepts either:

- shell-quoted complete JSON from `ocr review --format json`,
  `ocr scan --format json`,
  or `ocr session comments --json`;
- a named file containing one of those shapes or an OCR session JSONL transcript.

Non-interactive mode requires the named-file form.
Named files must be strict UTF-8 without a byte-order mark.
`-` never means standard input.
Piped,
redirected,
and TTY-pasted standard input are never ingestion sources.
Omitting the positional input is command misuse and exits with status `2` before any prompt.

## Modes

Exactly one mode is required.

### Interactive

```bash
open-code-review-issue --interactive ./review.json \
  --repo https://github.com/OWNER/NAME
```

Short form `-i` is available.
To pass generated JSON as one shell-safe positional argument:

```bash
open-code-review-issue --interactive "$(ocr review --format json)" \
  --repo https://github.com/OWNER/NAME
```

A literal JSON argument must quote the complete JSON value:

```bash
open-code-review-issue --interactive \
  '{"status":"complete","comments":[]}' \
  --repo https://github.com/OWNER/NAME
```

Interactive mode requires TTY standard input and TTY standard output.
TTY input is used only for finding selection and explicit decisions.
It presents ordinary and security findings separately,
requires explicit disclosure decisions,
and confirms the final batch before mutation.

### Non-interactive preview

```bash
open-code-review-issue --non-interactive ./review.json \
  --repo https://github.com/OWNER/NAME
```

Preview mode performs no Issue creation and prints one exact JSON publication plan.

### Non-interactive publication

Create only when no security-gated finding exists:

```bash
open-code-review-issue --non-interactive --apply ./review.json \
  --repo https://github.com/OWNER/NAME
```

Create only non-security findings:

```bash
open-code-review-issue --non-interactive --apply --non-security-only ./review.json \
  --repo https://github.com/OWNER/NAME
```

Create all findings after asserting that every security finding is safe to disclose publicly:

```bash
open-code-review-issue --non-interactive --apply --all ./review.json \
  --repo https://github.com/OWNER/NAME
```

`--all` authorizes public disclosure.
Do not use it for an unresolved suspected vulnerability.

## Finding OCR input

Generate a named JSON file with:

```bash
ocr review --format json > review.json
ocr scan --format json > scan.json
```

Inspect saved sessions or export one session's comments with:

```bash
ocr session list --json
ocr session comments --json <session-id> > comments.json
```

OCR session transcripts are commonly persisted at:

```text
~/.opencodereview/sessions/<encoded-repo-path>/<session-id>.jsonl
```

When positional input is omitted or a named path does not exist,
the command prints these generation instructions.
It also scans that sessions root without reading transcript contents
and suggests the most recently modified JSONL path when one exists.

## Repository selection

`--repo` accepts only `https://github.com/OWNER/NAME`.
Without it,
the command infers a repository only when the working directory is exactly a Git worktree root.
A subdirectory or non-worktree directory requires `--repo`.

## Exit statuses

- `0` for success or clean pre-publication cancellation;
- `1` for validation,
  preflight,
  GitHub,
  partial-publication,
  or handled interruption failures;
- `2` for command invocation misuse;
- `130` for a forced second publication interrupt.

## Development

```bash
mise run //package/open-code-review-issue/cli:buildAndTest
mise run //package/open-code-review-issue/cli:lint
mise run //package/open-code-review-issue/cli:lint:types
```

The integration suite invokes the built binary with disposable fixtures and a fake `gh` executable.
Live verification is not a routine mise task because it mutates an explicitly authorized disposable repository.
