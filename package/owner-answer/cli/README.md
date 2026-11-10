# @monochromatic-dev/owner-answer-cli

Record the owner's answers in the repository's questions file,
in the owner's own words,
and take the answered questions out of its open list.

The questions file,
`doc/handover/questions-for-the-owner.md`,
says it is re-read before every update and that an answered question leaves it.
This command does that edit from one answers file,
so the owner's words are pasted rather than retyped from a summary,
and so a heading that no longer exists stops the run instead of leaving a half-edited file.

It replaces the coordinator's scratch `record-*.ts` scripts,
listed in `doc/planning/scratch-scripts-folded-into-packages.md`.

## Usage

```bash
# Print what the answers file would remove and where it would insert.
owner-answer --answers answers.json

# Rewrite the questions file.
owner-answer --answers answers.json --apply
```

The run exits `0` when it finished,
and `2` when an input cannot be read or a named part of the questions file is not there.
An unreachable part is never skipped:
the file is written only after every removal and the insertion have been found.

## The answers file

```json
{
  "file": "doc/handover/questions-for-the-owner.md",
  "record": "## Answered in a message, recorded 2026-11-10T08:14:32-05:00\n\n### Container base stays CachyOS\n\nThe owner wrote \"Okay, cachyOS is fine then.\"\n",
  "anchor": "## Answered in the question tool, recorded",
  "remove": {
    "sections": ["### Is the container base too heavy?"],
    "bullets": ["- The btrfs host run needs someone to start it"],
    "lineRuns": [["  A question that runs over two lines,", "  asked in the open list."]]
  }
}
```

- `file` is the questions file to rewrite,
  as a path from where the command runs.
- `record` is the block to insert,
  verbatim.
  Its last line may be empty,
  which a written-as-template block ends with;
  that line is dropped so the block joins the file cleanly.
- `anchor` is the heading the block goes before,
  matched as a line prefix.
- `remove.sections` names headings of questions the answers settle.
  Each removes its heading through the line before the next `###` or `##` heading.
- `remove.bullets` names the first line of a bullet in the open list.
  Each removes that line and every following line indented by two spaces.
- `remove.lineRuns` names exact consecutive lines,
  for a question that is not a section or a bullet.
  Each line is compared byte for byte.

Every list is optional,
and each name is matched as a prefix except `lineRuns`.

## Development

```bash
mise run //package/owner-answer/cli:buildAndTest
mise run //package/owner-answer/cli:lint
mise run //package/owner-answer/cli:lint:types
```

The suites drive the line operations and the run against disposable files under the test scratch root.
No suite touches `doc/handover/questions-for-the-owner.md`.
