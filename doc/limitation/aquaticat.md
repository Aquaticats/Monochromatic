# Aquaticat limitations

Documenting failure patterns of the repository owner in the reviewer and delegator
role, so future delegations carry the right gates from the start.

## Client portfolio review coverage (2026-10, audited 2026-10-17)

**Goal:
** review and accept increments of a delegated implementation of a client's
portfolio, produced by GPT 6.1 Sol in the client's repository, whose identifier is
withheld from this public document, and hold the client relationship through review
rounds.
On 2026-10-17 an evidence-driven audit of a client complaint pack, kept in the folder
`Client comparison screenshots 2026OCT170133` beside the portfolio repository, showed
that every defect class the client photographed sat exactly where this reviewer's
coverage had stopped.

### Mistakes

#### Trusted a single delegate end to end without any independent check

The whole engagement, implementation, verification, and the fidelity claims made to the
client, was handed to GPT 6.1 Sol and accepted as produced.
No page ever received an independent comparison run by this reviewer or by a second
agent or model, and the handover's verification narrative was treated as ground truth
rather than as a claim to test.
The trust was never calibrated by sampling: picking one case-study page at one
reference width and diffing it against its reference would have taken minutes and would
have surfaced the drift in October, weeks before the client's pack arrived.
Every other mistake in this entry is a downstream shape of this one.

Correct behavior:
 trust but sample.
For any delegated fidelity claim, independently re-run at least one probe per page
family, or commission a second-opinion review from a different model, before accepting
the work as verified.
A delegate's handover is evidence about what the delegate checked, never evidence about
what the product is.

#### Accepted verification conclusions without auditing their coverage

The handover said checks passed, and that was read as the site matching its references.
The questions that would have caught the gap, which pages, which widths, which probe
classes, which positive controls, were not asked.
The three case-study pages never had matched-scale comparisons, and that survived
until client evidence arrived.

Correct behavior:
 for every delegated fidelity claim, audit the coverage before
accepting the conclusion: pages, widths, probe classes, positive controls.
Read the evidence list, not the verdict.

#### Waived the checkpoints that would have caught the gap

Per-section review of Home was waived by this reviewer's own instruction.
The case-study scope was accepted on interaction-focused checks, viewers, links,
targets, while the fidelity artifact for those pages was never requested.
The minimum-code priority traded away per-page comparison artifacts without naming
them as a risk.

Correct behavior:
 waive review of content as often as needed, but never waive the
existence of the comparison artifact; a page without one is not verifiable later.

#### Approved fluid wide-screen behavior without asking what wide means

The adaptation of Home to Tablet and Desktop, including the fluid growth variable and
the Desktop works width cap, was approved without declaring what windows wider than
the widest reference should show.
The resulting torn composition at 1920 pixels became the client's loudest complaint
class, and it was nobody's decision because it was never asked.

Correct behavior:
 any approval of fluid behavior past a reference frame includes a
declared above-reference behavior and one capture at the widest window the audience
actually uses.

#### Reviewed at reference widths and generalized the verdict

Review happened at 390, 744, and 1440 pixels, the widths the references exist at, and
"no difference" was true there for the compared pages.
That true statement was then applied to the whole site at every width, including the
client's 1920-pixel window and 393-point phone.

Correct behavior:
 every match claim carries its width and page set in the same
sentence, and at least one review round per release happens at the audience's real
viewport, not only at reference widths.

#### Kept no written review contract with the client

No agreed review widths, no agreed artifact format, no agreed language for defect
names.
Feedback arrived as unstructured WeChat complaints in the client's language while the
repository and correspondence ran in English, which let roughly fifty visible instances
accumulate before anyone grouped them into eight causes.

Correct behavior:
 agree review widths, pack format, and bilingual captions with the
client before the first review round, and send comparison packs before being asked.

#### Left the available target hardware unused

The ssh config names a Mac host under the alias `m1`, and this repository's rules
document its memory and storage limits, yet no verification round was routed through
it, neither by the delegate nor by this reviewer, who owns the config and never
connected to it for this project.
The client's two real environments therefore remained untested on any Apple stack
until her own screenshots arrived.
At audit time the host's hostname did not resolve, which explains why it stayed unused
on the audit night but not why the gap went unnoticed for weeks before that.

Correct behavior:
 device coverage belongs in the coverage matrix; when a target device
is reachable, a release is not verified until one round has run on it, and when it is
unreachable, that is raised as a blocker instead of being silently dropped from scope.

### What worked, for balance

Matched-scale reference comparisons were insisted upon for the pages that have them,
which is why Home, About, and Contact measure height-exact and why the audit had a
baseline to compare against.
Handover discipline made the entire later audit possible in one evening.
Commissioning the evidence-driven review, converting every client file, measuring at
the client's widths, and grouping the complaint into causes before replying, turned an
open-ended dispute into a finishable list.

### Lesson

Delegating implementation is not delegating verification ownership.
The owner's irreducible job is the coverage audit: which probes can fail, on which
pages, at which widths, against which source of truth.
Every gap in this engagement sat exactly where that audit was skipped, and every
strength sat exactly where it was performed.
Blind trust in a single delegate is the failure mode that made the skipping possible:
sampling one page at one width costs minutes, while client-discovered drift costs
nights and a relationship.
Future delegations start by writing the coverage matrix before the first line of
implementation, ship the matrix with the work, and include the owner's own sampled
probe results in the acceptance record.
