# No commit hold between a release dispatch and its publication

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 2367 to 2387 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

**"No commit goes to `main` between the dispatch and the publication" was a rule of this session's own making,
and it is withdrawn.**
The owner,
2026-11-13:
"that's backwards and implies a defect in our process or CI or whatever."
It first appears in this document on 2026-11-12 (the sections on 0.6.1-alpha.14 and 0.6.1-alpha.15) with no failure recorded as its reason,
and this session told the other session on 2026-11-13 that such a commit "has broken a release before",
which nothing in this document supports.
What the release is built to do,
read from its source:

- `.github/workflows/crate-release.yml` (its header,
  lines 12 to 19) and `package/crate-release/cli/src/push-decision.ts` (line 173 on):
  "main moves every few minutes and a release takes a long while",
  so when `main` moved while the release ran,
  the bump commit is sent again on the new head when none of the paths the release depends on changed,
  and nothing is pushed when one did.
  A commit of documentation or of another package during a release is therefore expected and harmless.
- `cargo-publish.yml` is started on `main` by a dispatch,
  and a run's commit is fixed when the run is made,
  so a commit after that moment cannot change what is built.
- One gap the source shows and no run has shown:
  the publish is started with `--ref main` a moment after the bump is pushed,
  and nothing in it checks that the commit it runs on is the bump commit.
  A commit to the crate's own paths landing in that moment would be published without the release's checks.
  Filed as issue 6294,
  not worked around by holding commits.
- On this machine,
  a commit made on the local `main` while the remote gained the bump commit is a commit behind the remote,
  like any other.
  Tried at once:
  the commit that recorded this (`9dc6859dd5`,
  11:18:02,
  0.9 s) was made while the remote's `main` stood at the bump `1d0a829ac7`;
  the wrapper merged the two as `52c16b435f` ("Merge origin/main into main") at 11:18:03 and pushed,
  and local and remote were equal ten seconds later.
  The release went on undisturbed.
- The other session was told the hold is withdrawn and why.
