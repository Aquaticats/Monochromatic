# Git policy cli a new build and a running service share one protocol number

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 43631 to 43636 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  so each starts again on the new program at its next connection.
  They were stopped because the service's messages
  changed between the two commits (thirty files under `src/native/service/`,
  the push programs held in memory) while
  `PROTOCOL_VERSION` stayed 12 and the release stayed `0.6.1-alpha.44`,
  so a new client and an old service accept each
  other by number and speak different messages;
  a release changes the release string,
  so this bites only two builds of
  one unreleased version.
  Read before the switch:
  the journal's change is additive (an intent without `transports` reads
  as none chosen);
  the provisioning code is unchanged and the unit's text names the program link,
  so the switch asks for
