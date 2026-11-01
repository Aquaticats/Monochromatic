# Pi Advisor collapsed output can show only the Markdown title

## Symptom

Collapsed Advisor tool output sometimes appears to contain only:

```text
## Advisor review
```

The provider response can still contain more text.
The visible one-line symptom happens when the response starts with Markdown headings and the tool row is collapsed.

## Root cause

The collapsed renderer intentionally shows a header plus one summary line.
`package/pi-plugin/advisor/src/rendering-summary.ts:47` calls `firstAdvisoryLine` when `expanded` is false:

```typescript
const firstLine = firstAdvisoryLine(text,);
```

Before the fix,
 `firstAdvisoryLine` returned the first non-empty line.
For Markdown-shaped advisor responses,
 that first line was often the document title,
 `## Advisor review`,
not an actual finding.

The fixed implementation keeps the collapsed behavior but skips Markdown ATX headings before choosing the summary line.
`package/pi-plugin/advisor/src/rendering-summary.ts:96` normalizes non-empty lines
and chooses the first non-heading body line:

```typescript
return nonEmptyLines
  .find(function keepBodyLine(line,) {
    return !isMarkdownHeadingLine(line,);
  },)
  ?? nonEmptyLines.at(0,)
  ?? '(advisor returned no text)';
```

`package/pi-plugin/advisor/src/rendering-summary.ts:194` detects heading lines
by checking leading hash markers followed by a space:

```typescript
const firstNonMarkerIndex = firstNonHeadingMarkerIndex(trimmedLine,);
return (firstNonMarkerIndex > 0)
  && (trimmedLine.at(firstNonMarkerIndex,) === MARKDOWN_HEADING_SEPARATOR);
```

## Verification

Version under test:
 commit `058b16777`.

The regression test is `package/pi-plugin/advisor/src/rendering-summary.unit.test.ts`.
It reproduces the old behavior with this input:

```markdown
## Advisor review

### Flawed assumptions

1. **Assumption** Check the active model identity.
```

The red run before the fix failed with the expected symptom:

```text
AssertionError: expected '## Advisor review' to equal '1. **Assumption** Check the active mo…'
```

The fixed run passes:

```bash
mise run //package/pi-plugin/advisor:test:unit package/pi-plugin/advisor/src/rendering-summary.unit.test.ts
```

```text
[firstAdvisoryLine] PASS skips markdown headings before body text, keeps markdown heading when it is the only text
```

Full package verification also passed:

```bash
mise run //package/pi-plugin/advisor:test:unit
mise run //package/pi-plugin/advisor:lint:types
mise run //package/pi-plugin/advisor:lint:oxlint
mise run //package/pi-plugin/advisor:verify:extension
```

## Verified workaround

Expand the Advisor tool result in the TUI.
Expanded rendering returns the full text path from `renderAdvisorSummary`,
 so the hidden body is visible.

## What does not work

Rerunning Advisor does not address this display issue when the next response starts with the same heading shape.
The provider call may have succeeded;
 the collapsed renderer chose a low-information first line.

Treating the result as an empty provider response is also wrong.
The failing regression shows non-empty body text was present but not selected for the collapsed line.

## Upstream filing decision

This is repo-local Pi Advisor behavior,
 not an upstream pi-coding-agent bug.
No upstream issue is appropriate.
