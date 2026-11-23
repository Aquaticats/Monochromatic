# Triage Labels

The skills speak in terms of five canonical triage roles.
 This file maps those roles to the actual label strings used in this repo's issue tracker.

<table>
<thead>
<tr>
<th>Label in mattpocock/skills</th>
<th>Label in our tracker</th>
<th>Meaning</th>
</tr>
</thead>
<tbody>
<tr>
<td>`needs-triage`</td>
<td>`needs-triage`</td>
<td>Maintainer needs to evaluate this issue</td>
</tr>
<tr>
<td>`needs-info`</td>
<td>`needs-info`</td>
<td>Waiting on reporter for more information</td>
</tr>
<tr>
<td>`ready-for-agent`</td>
<td>`ready-for-agent`</td>
<td>Fully specified, ready for an AFK agent</td>
</tr>
<tr>
<td>`ready-for-human`</td>
<td>`ready-for-human`</td>
<td>Requires human implementation</td>
</tr>
<tr>
<td>`wontfix`</td>
<td>`wontfix`</td>
<td>Will not be actioned</td>
</tr>
</tbody>
</table>

When a skill mentions a role (e.g. "apply the AFK-ready triage label"),
 use the corresponding label string from this table.

Edit the right-hand column to match whatever vocabulary you actually use.

## Blocked work

A standing instruction of the owner's,
given 2026-11-23 when asked which label marks held work:
"Label:
blockedBy:<specific> .
Create new labels as you go.
In fact make this a standing instruction in issue-triage ."

So an issue that cannot be worked on now carries one `blockedBy:<specific>` label for each thing it waits on.

- `<specific>` names the blocker itself.
  When another issue blocks it,
  that is the issue's number,
  as in `blockedBy:6261`.
  Otherwise it is a short name in lowercase words joined by hyphens for the thing waited on,
  as in `blockedBy:zfs-route` or `blockedBy:owner-decision`.
- When the label does not exist yet,
  create it,
  with a description that says what is waited on and since when:
  `gh label create "blockedBy:<specific>" --description "<what it waits on>" --color d93f0b`.
  `gh label list --search blockedBy` prints the ones that exist;
  reuse one before making another for the same blocker.
- The issue's first line also says what blocks it,
  in words,
  with the owner's own words where the owner set the hold.
- A blocked issue does not carry `ready-for-agent`.
- When the blocker is gone,
  remove the label from every issue that carries it,
  give each its ordinary triage label,
  and delete a `blockedBy:` label that no issue carries any more.
