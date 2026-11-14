# Removed archives and where history holds each

Not final and not approved:
written by the run that removed the archives,
from its own measurements.

## What this record is

On 2026-11-13 the owner ordered that no archive stays tracked:
"Git ignore all common archive formats (gz,
zip,
tar) and remove anything already
committed from the repo.
If something would break,
represent the data another way."
Asked how the data is kept,
the owner answered "Trim to what is cited.
It doesn't discard
the evidence for good because we are not going to rewrite history."
So the raw bulk left the tree,
Git history keeps every archive whole,
and what a document cites is kept beside it as plain text.

This record is the one place that resolves a pointer to a removed archive.
For each archive it gives the bytes and SHA-256 the archive had as tracked,
the commit that last changed it,
and the command that prints it from history.
Every document that names a removed archive still names it,
and now says beside the name
that the archive left the tree and where this record resolves it.

No history was rewritten:
no filter,
no force push,
nothing rebased.
A commit below is a commit of the branch that removed the archive,
and `git show <commit>:<path>` reads the whole archive from any checkout.

## How a document points at this record

In Markdown,
one note follows the paragraph that names the archive:

```text
`<archive file name>` left the tree on 2026-11-13.
`doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the
command that prints it.
```

Where records were kept for a document,
the note adds the line
"Its cited records are in `<path>`.".
A link whose target was a removed archive became its label plus the same path in a code
span,
so no page renders a dead link,
and a reference definition that pointed at one was
inlined where it was used.

In JSON the note is a sibling field of the entry that names the archive,
or of the entry
that records the digest of a file this removal edited.
It is always called `inTreeNow` and always holds an array of notes,
one per path the entry names,
each with the same fields:

- `measuredOn`:
  the day the note was written,
  `2026-11-13`.
- `bytes`:
  the file's bytes in the tree now,
  `null` when the path left the tree.
- `sha256`:
  the file's SHA-256 in the tree now,
  `null` when the path left the tree.
- `removedFromTree`:
  whether this removal took the path out of the tree.
- `historyCommand`:
  the command that prints a removed archive from history,
  else `null`.
- `record`:
  this record's path.
- `keptAs`:
  the plain file that holds what the documents cite,
  else `null`.
- `note`:
  a sentence for the few records that need one,
  else `null`.

A `note` explains a pair the seven other fields cannot:
a record whose pair is an earlier version of the path,
naming both pairs and the commits
that hold them;
a file that records its own digest,
whose pair can never be current;
or a member that is not kept,
and why.
Where such a note is about a file that records its own digest,
`bytes` and `sha256` are
`null`,
because writing a current pair into that file changes it.

The recorded `bytes` and `sha256` of an entry are left as they were:
they describe the file when the evidence was recorded,
and this note describes the tree
now.
A JSON entry that is a verbatim copy of a provider record,
a GitHub issue or comment or a captured script,
never gains a field:
its note sits on the nearest enclosing entry this repository wrote.
These are the sites where that happened,
listed so the reader can find the note:

- `doc/audit/depot-toml-reconciliation-2026-10-17.evidence.json`,
  field `/continuation/pilotRecovery/issueComment` names `doc/audit/depot-toml-mutation-native-2026-10-17.evidence.json.gz`;
  the note is on `/continuation/pilotRecovery`.
- `doc/audit/depot-toml-reconciliation-2026-10-17.evidence.json`,
  field `/historicalIndependentCoverage/closureIssue/receipt/issue/comments/0` names `doc/audit/depot-toml-full-mutation-2026-10-17.evidence.zip`;
  the note is on `/historicalIndependentCoverage/closureIssue/receipt/issue`.
- `doc/audit/depot-toml-reconciliation-2026-10-17.evidence.json`,
  field `/historicalIndependentCoverage/closureIssue/receipt/issue/comments/0` names `doc/audit/depot-toml-full-reconciliation-2026-10-17.evidence.json.gz`;
  the note is on `/historicalIndependentCoverage/closureIssue/receipt/issue`.
- `doc/audit/depot-toml-reconciliation-2026-10-17.evidence.json`,
  field `/issues/3` names `doc/audit/depot-toml-mutation-native-2026-10-17.evidence.json.gz`;
  the note is on `the document root`.
- `doc/audit/depot-toml-reconciliation-2026-10-17.evidence.json`,
  field `/mutationRunnerSpawnControl/sourceIssue/issue` names `doc/audit/depot-toml-spawn-control-2026-10-17.evidence.zip`;
  the note is on `/mutationRunnerSpawnControl/sourceIssue`.
- `doc/audit/depot-toml-reconciliation-2026-10-17.evidence.json`,
  field `/sourceMethodLookupIncident/issue` names `doc/audit/depot-toml-mutation-native-2026-10-17.evidence.json.gz`;
  the note is on `/sourceMethodLookupIncident`.

## Planning records, `doc/planning/depot-*`

39 archives left the tree in this family.

- `doc/planning/depot-complete-coverage-android-junit-execution.json.gz`:
  19294 bytes,
  SHA-256 `3014921f12d9ca8f84927c93e633376e8ee698118d82630b3846307f6cab758d`,
  last changed in commit `a1def554867e9f8c66c8f538bac9edb5f441bb85`,
  printed by `git show a1def554867e9f8c66c8f538bac9edb5f441bb85:doc/planning/depot-complete-coverage-android-junit-execution.json.gz`.
- `doc/planning/depot-complete-coverage-android-receipts.json.gz`:
  96498 bytes,
  SHA-256 `b517d9c2ff726f96c1afedc8c5bbb5ed2a389125ccc41913a1c3292bcf7d85e1`,
  last changed in commit `a92139ee4aeca633580b6d810640433fee664888`,
  printed by `git show a92139ee4aeca633580b6d810640433fee664888:doc/planning/depot-complete-coverage-android-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-android-reproducibility.json.gz`:
  18889 bytes,
  SHA-256 `04d241cdbe765a92c434dd808dd9c13a96931e92ca4d806fca66decf3fe1b57e`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-android-reproducibility.json.gz`.
- `doc/planning/depot-complete-coverage-artifact-browser-receipts.json.gz`:
  8319868 bytes,
  SHA-256 `27f3d6eab5756c66207278995784ccb21b24d16200abce96c98c6bd62e6d7d74`,
  last changed in commit `13d793bf83f61691b141750eed5bd3ff32ff3927`,
  printed by `git show 13d793bf83f61691b141750eed5bd3ff32ff3927:doc/planning/depot-complete-coverage-artifact-browser-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-commands.json.gz`:
  194168 bytes,
  SHA-256 `4da3df7425e8fe039f862be5696a749e116595a64b8aab29751083f8992d7de8`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-commands.json.gz`.
- `doc/planning/depot-complete-coverage-file-enforcer-git-config.json.gz`:
  70499 bytes,
  SHA-256 `559172ce0a2c8ebee3e4acb320eba40b259ca25d72440d15e9fa2a9e3834b251`,
  last changed in commit `e913e0c6bf4708f09ab24ae6b785cb06c525c4b8`,
  printed by `git show e913e0c6bf4708f09ab24ae6b785cb06c525c4b8:doc/planning/depot-complete-coverage-file-enforcer-git-config.json.gz`.
- `doc/planning/depot-complete-coverage-finite-owner-receipts.json.gz`:
  391550 bytes,
  SHA-256 `e93b3f330656a5f7c7c7c629d915cc70e0dda8a1bfdc8fb6ad826556bba264c7`,
  last changed in commit `e4f516b1efecf2ee377ece0ddbaab8627f85604b`,
  printed by `git show e4f516b1efecf2ee377ece0ddbaab8627f85604b:doc/planning/depot-complete-coverage-finite-owner-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-harnesses.json.gz`:
  189868 bytes,
  SHA-256 `be7f1316c675028655c2f1c7f84a119043732f4fcde21fb867d9849131050b83`,
  last changed in commit `785a1d164b44050ae3a8d51f1002eb7846c9a5e0`,
  printed by `git show 785a1d164b44050ae3a8d51f1002eb7846c9a5e0:doc/planning/depot-complete-coverage-harnesses.json.gz`.
- `doc/planning/depot-complete-coverage-matched-12127814164.zip`:
  283403 bytes,
  SHA-256 `83163ac9274414e736478531ea56bca6d001b22f1b8a019901c37b5058a54d67`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-matched-12127814164.zip`.
- `doc/planning/depot-complete-coverage-matched-12129071058.zip`:
  283582 bytes,
  SHA-256 `624cea56d39c4b5dd10a38b3f00618e13de3c4387fba6d4b964fc665fa177254`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-matched-12129071058.zip`.
- `doc/planning/depot-complete-coverage-matched-receipts.json.gz`:
  263704 bytes,
  SHA-256 `12fa72bf11bfbfa44d59927c928e0a3284f8ac762b887944a67ad28104c263b1`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-matched-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-module-12128501990.zip`:
  738730 bytes,
  SHA-256 `bd7ef7c0cd3ee98a9819d0833cecba1acb0f171c180e80f94c66acceafe25979`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-module-12128501990.zip`.
- `doc/planning/depot-complete-coverage-module-12128625660.zip`:
  613215 bytes,
  SHA-256 `c4f272a7cb596ba6f3b2b8d8f02da7a6370950d3984527455fac07a0a7ff2b46`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-module-12128625660.zip`.
- `doc/planning/depot-complete-coverage-module-receipts.json.gz`:
  282355 bytes,
  SHA-256 `404f23b4383b66949d0fee790cb715b48745ac0379e36e095c016bce0a049470`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-module-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-node-default-pure-12130024032.zip`:
  323469 bytes,
  SHA-256 `942c1f3bb48bb7193042bd851eca936582f61099c51941652952d5ee6365ac85`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-default-pure-12130024032.zip`.
- `doc/planning/depot-complete-coverage-node-default-pure-12130643908.zip`:
  467590 bytes,
  SHA-256 `c58ea57a791771e71c688ec6c1d139920226b0eb23ebbde71e3ed4abeb07a273`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-default-pure-12130643908.zip`.
- `doc/planning/depot-complete-coverage-node-default-pure-receipts.json.gz`:
  269643 bytes,
  SHA-256 `21b922f16581e24940d44bc7e42f7f46296ae0ccbbe6899d1ba8d7fb1aba0e8e`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-default-pure-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-node-host-12133008592.zip`:
  1130359 bytes,
  SHA-256 `93c619710d1c7d5e65f5d69f6f098d3e6e45dc45a8746c7f575b0276ae59ca12`,
  last changed in commit `785a1d164b44050ae3a8d51f1002eb7846c9a5e0`,
  printed by `git show 785a1d164b44050ae3a8d51f1002eb7846c9a5e0:doc/planning/depot-complete-coverage-node-host-12133008592.zip`.
- `doc/planning/depot-complete-coverage-node-host-12133191644.zip`:
  132753 bytes,
  SHA-256 `9fcb7a0185590d9527b8fb2aea3b7c78e0a1a86b2d6bdda3f31539e69dd17e98`,
  last changed in commit `785a1d164b44050ae3a8d51f1002eb7846c9a5e0`,
  printed by `git show 785a1d164b44050ae3a8d51f1002eb7846c9a5e0:doc/planning/depot-complete-coverage-node-host-12133191644.zip`.
- `doc/planning/depot-complete-coverage-node-host-receipts.json.gz`:
  2320902 bytes,
  SHA-256 `cb941774b973a6faae6ec9f3f78ee0f65ab7a85a377d2f58e19cacb7046cf412`,
  last changed in commit `4289ddfb1e59140adf137b5c91031535b8336e27`,
  printed by `git show 4289ddfb1e59140adf137b5c91031535b8336e27:doc/planning/depot-complete-coverage-node-host-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-node-process-12130894667.zip`:
  854996 bytes,
  SHA-256 `fc86829147a0f861c8d31bc4f07b5ffe41746116d12d7ac7b751c224b553d806`,
  last changed in commit `3875a4ee9f857898191fb07473b00a608232f55a`,
  printed by `git show 3875a4ee9f857898191fb07473b00a608232f55a:doc/planning/depot-complete-coverage-node-process-12130894667.zip`.
- `doc/planning/depot-complete-coverage-node-process-12131308314.zip`:
  853811 bytes,
  SHA-256 `f54995cbfd17b37f75947ce8921c1d3ea8360f4df391ddacb77640a28d21e433`,
  last changed in commit `3875a4ee9f857898191fb07473b00a608232f55a`,
  printed by `git show 3875a4ee9f857898191fb07473b00a608232f55a:doc/planning/depot-complete-coverage-node-process-12131308314.zip`.
- `doc/planning/depot-complete-coverage-node-process-receipts.json.gz`:
  284172 bytes,
  SHA-256 `4cdd02706562f9dc7d30db96cb41561374e793712b7c02bc3eb6e33bde4b5211`,
  last changed in commit `3875a4ee9f857898191fb07473b00a608232f55a`,
  printed by `git show 3875a4ee9f857898191fb07473b00a608232f55a:doc/planning/depot-complete-coverage-node-process-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-node-static-12129574586.zip`:
  1472088 bytes,
  SHA-256 `0813cca85cb87526d76479a600a8502cdc656130438b343f270e9b9fde647aa7`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-static-12129574586.zip`.
- `doc/planning/depot-complete-coverage-node-static-12129764494.zip`:
  878520 bytes,
  SHA-256 `337e24c8a54e655a56547b21ae5db7426d86481d9947243ac306112531c9485c`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-static-12129764494.zip`.
- `doc/planning/depot-complete-coverage-node-static-12129804795.zip`:
  661231 bytes,
  SHA-256 `5779436f80e77811d8f0bc702a9128589e47212479fca5c0636d65ea5b5b37a9`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-static-12129804795.zip`.
- `doc/planning/depot-complete-coverage-node-static-12129992871.zip`:
  694654 bytes,
  SHA-256 `9d091206bc138fa79238c33a1c2549b1053f523cb388de76627ba802b9cafbd0`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-static-12129992871.zip`.
- `doc/planning/depot-complete-coverage-node-static-receipts.json.gz`:
  568755 bytes,
  SHA-256 `c82ed36cfcf5c5653727a191d5f939c9f79257edb39a1cd3601581dbf9eb9420`,
  last changed in commit `aad6071135cd14b7201a8502cb062d429c97431c`,
  printed by `git show aad6071135cd14b7201a8502cb062d429c97431c:doc/planning/depot-complete-coverage-node-static-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-packages.json.gz`:
  1143058 bytes,
  SHA-256 `67b48d151904ab676116033a1866ffbde9cf48d7ac9c896f19d34185af5587d5`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-packages.json.gz`.
- `doc/planning/depot-complete-coverage-pilot-12127344658.zip`:
  75356 bytes,
  SHA-256 `f63fd6ff48c0767286dc55a6ca229fff05441f2611d3874e00ae3cd259b38a79`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-pilot-12127344658.zip`.
- `doc/planning/depot-complete-coverage-pilot-12127851283.zip`:
  84871 bytes,
  SHA-256 `d82ef3475fc03d269ff48e06a3f58372cd06f72f72d506fe04157b3184e43156`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-pilot-12127851283.zip`.
- `doc/planning/depot-complete-coverage-pilot-receipts.json.gz`:
  8701 bytes,
  SHA-256 `ae463bdd483be9bd572f8ddcd90ef7c47f2eb4b56c49e9906ec86b2e25632022`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-pilot-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-source.json.gz`:
  1036943 bytes,
  SHA-256 `4709d2fa872a5f85dc43fba3d899397dd6f59e5917c143e48c22baf0e1f17fd9`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-source.json.gz`.
- `doc/planning/depot-complete-coverage-specifications.json.gz`:
  407870 bytes,
  SHA-256 `d4bfb8338f24404809e8ebbb3dcf826553a59ba40cd3c016b2388fecbaf6b001`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-specifications.json.gz`.
- `doc/planning/depot-complete-coverage-tree.json.gz`:
  470740 bytes,
  SHA-256 `454274c9bb5253b6403a5951987db5d5915e968c79a9f3567054e78f0ef89020`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-tree.json.gz`.
- `doc/planning/depot-complete-coverage-typescript-fuzz-12134106934.zip`:
  183724 bytes,
  SHA-256 `0f8a23ad24fc94ef3fac95ec8be759566f7ddf52132a5e93de3d57b82e2a63fa`,
  last changed in commit `3cf6175eaa66af95032cccccfce2fdfa1946f474`,
  printed by `git show 3cf6175eaa66af95032cccccfce2fdfa1946f474:doc/planning/depot-complete-coverage-typescript-fuzz-12134106934.zip`.
- `doc/planning/depot-complete-coverage-typescript-fuzz-receipts.json.gz`:
  163263 bytes,
  SHA-256 `7fde34982edba1b1086cd1a8c3a1e80da2461dc7231ba171b17bce1d8e180bfb`,
  last changed in commit `3cf6175eaa66af95032cccccfce2fdfa1946f474`,
  printed by `git show 3cf6175eaa66af95032cccccfce2fdfa1946f474:doc/planning/depot-complete-coverage-typescript-fuzz-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-wg-companion-receipts.json.gz`:
  138936 bytes,
  SHA-256 `a31c47e47dcae8ea4619eb3c8b14e6eda24a67b4815b028b9976797a71344dae`,
  last changed in commit `f99173f641e92ab891076630cd0c96c45ea5d986`,
  printed by `git show f99173f641e92ab891076630cd0c96c45ea5d986:doc/planning/depot-complete-coverage-wg-companion-receipts.json.gz`.
- `doc/planning/depot-complete-coverage-workflows.json.gz`:
  48100 bytes,
  SHA-256 `735f99a61ef4cad3892da1001b2c69a763e1943fa711187c529356b4ecf0ed89`,
  last changed in commit `9d19767001013fb3d7cdee990a3a1135ab301689`,
  printed by `git show 9d19767001013fb3d7cdee990a3a1135ab301689:doc/planning/depot-complete-coverage-workflows.json.gz`.

## Chromium disk-responsiveness records

22 archives left the tree in this family.

- `doc/troubleshooting/chromium-disk-responsiveness/candidates.jsonl.gz`:
  7169576 bytes,
  SHA-256 `f0d1d30edc245d9017817199fa363488b6bf76a55af519ebffb27e003ac1fe4a`,
  last changed in commit `7467f5f7c72086fe8137a29c20235d11fc6310ca`,
  printed by `git show 7467f5f7c72086fe8137a29c20235d11fc6310ca:doc/troubleshooting/chromium-disk-responsiveness/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/dependencies-candidates.jsonl.gz`:
  298922 bytes,
  SHA-256 `0784e9fa0c5b52a98251d96b73a4d09490d2369ace88570392f9a504c89d3a8d`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/dependencies-candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_angle/candidates.jsonl.gz`:
  2302 bytes,
  SHA-256 `6bb4f853f91e927dda6141cb5a5a7ab4cb3096c30253d0cdff73c158eaaf2a22`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_angle/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_dawn/candidates.jsonl.gz`:
  692 bytes,
  SHA-256 `e9568fccadadfe646dd91fc5b0c94a8e0743db6bec478e0b659a3fc8c128fe2a`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_dawn/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_icu/candidates.jsonl.gz`:
  345 bytes,
  SHA-256 `c8fa1109c54987bf6409a133f669cb817e1d8bf64fa9fc61c8fa2e829ad3dd2b`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_icu/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_leveldatabase_src/candidates.jsonl.gz`:
  20 bytes,
  SHA-256 `59869db34853933b239f1e2219cf7d431da006aa919635478511fabbfc8849d2`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_leveldatabase_src/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_pdfium/candidates.jsonl.gz`:
  268 bytes,
  SHA-256 `fb80a55167ffe91f8b51dd5339dda22ed60746fdf156948c9a882af07fc5805e`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_pdfium/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_skia/candidates.jsonl.gz`:
  992 bytes,
  SHA-256 `000d3a046a1c22f07922f38fd7434375e0a7d39a5be3a61d33e3c23406fcd234`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_skia/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_sqlite_src/candidates.jsonl.gz`:
  1261 bytes,
  SHA-256 `b81bc4028e950f16cd9129888e01ed3eeb2dba729872fb3d6aa2cecd0bc95870`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_sqlite_src/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_webrtc/candidates.jsonl.gz`:
  752 bytes,
  SHA-256 `4bd2d966b79c63bdc0ad30aa8ca20fd91ddd8c0a6e34891e192b7d7b0442b4f0`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/third_party_webrtc/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/v8/candidates.jsonl.gz`:
  385 bytes,
  SHA-256 `887ff114ca940e3f7375ac25da1c0290f1de491448491209f390ad9c7f157c72`,
  last changed in commit `c8dce0da318b4767349da177bf59f7963bf078d5`,
  printed by `git show c8dce0da318b4767349da177bf59f7963bf078d5:doc/troubleshooting/chromium-disk-responsiveness/dependencies/loaders/v8/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_angle/candidates.jsonl.gz`:
  13471 bytes,
  SHA-256 `4dd82eceb2fdb9463cd68072758b0981dd4b55badef0ef5be0822520d08307da`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_angle/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_dawn/candidates.jsonl.gz`:
  32052 bytes,
  SHA-256 `466cc50cdf0fe0ca4887d191c9ae18a7e11cab7f7009e1619d1fe8bfe7be70df`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_dawn/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_icu/candidates.jsonl.gz`:
  16602 bytes,
  SHA-256 `06ef194be8dc6cadbe068e105919b7c4f6922c0bd536d427f489d4e492575919`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_icu/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_leveldatabase_src/candidates.jsonl.gz`:
  6890 bytes,
  SHA-256 `d149f49aad090b63eec48da9016684328ac3c73ecb8c747b2e5e897fa4b8b2fc`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_leveldatabase_src/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_pdfium/candidates.jsonl.gz`:
  5054 bytes,
  SHA-256 `e85287851c8d63f979101cac40a396970eba27b819d3eda879580c198f204889`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_pdfium/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_skia/candidates.jsonl.gz`:
  52366 bytes,
  SHA-256 `eb807ae474d8e3358c9b90563d8c8ddacd1351d0059ca6e90187e95ae610cbec`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_skia/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_sqlite_src/candidates.jsonl.gz`:
  99166 bytes,
  SHA-256 `a615fe012cc57c66a248c27a48a94f1a7aed8166247b76b3cf9a83a8b7c06fec`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_sqlite_src/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_webrtc/candidates.jsonl.gz`:
  27464 bytes,
  SHA-256 `30a3538ffce99c58c1403f23d3b17fbe3cd9a35ce98fb1bf5792c38de134348c`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/third_party_webrtc/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/dependencies/v8/candidates.jsonl.gz`:
  37663 bytes,
  SHA-256 `f77c27335b784833e3364a454dde0443554932bda57258682eb65e52a36a8238`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/dependencies/v8/candidates.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/loader-raw.jsonl.gz`:
  22509 bytes,
  SHA-256 `797127c51ff0a4342654aaeea9a42a0e8c7d0cd0d991d52d21106edcaf8a8cc1`,
  last changed in commit `f776290a3bbfa31d5f76a266717e5135060b9f2f`,
  printed by `git show f776290a3bbfa31d5f76a266717e5135060b9f2f:doc/troubleshooting/chromium-disk-responsiveness/loader-raw.jsonl.gz`.
- `doc/troubleshooting/chromium-disk-responsiveness/supplemental-raw.jsonl.gz`:
  7050252 bytes,
  SHA-256 `ebcc6b24b53b98979eaf801f747cb2f23c8f3962a5439cc4a50caef3a3aa00a1`,
  last changed in commit `7467f5f7c72086fe8137a29c20235d11fc6310ca`,
  printed by `git show 7467f5f7c72086fe8137a29c20235d11fc6310ca:doc/troubleshooting/chromium-disk-responsiveness/supplemental-raw.jsonl.gz`.

## The optique research record

1 archives left the tree in this family.

- `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`:
  631659 bytes,
  SHA-256 `0f2a70bd4504b1c8f58cb5234feb5354e5fa490fa7933019106195517d892352`,
  last changed in commit `a6783a44ae7bdd43f9cd6995fcb6416eea1250d1`,
  printed by `git show a6783a44ae7bdd43f9cd6995fcb6416eea1250d1:doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`.
  Its cited records are kept in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence/`.

## The Firefox disk-responsiveness audit

4 archives left the tree in this family.

- `doc/audit/firefox-esr-disk-responsiveness.browser.json.gz`:
  51260 bytes,
  SHA-256 `2c5de6807a774e097b9fc0accdcfc9382197a1a6e4b88b04bdf9817665a92124`,
  last changed in commit `b1b7669f03151c1f7417452987d4850923469eeb`,
  printed by `git show b1b7669f03151c1f7417452987d4850923469eeb:doc/audit/firefox-esr-disk-responsiveness.browser.json.gz`.
- `doc/audit/firefox-esr-disk-responsiveness.candidates.jsonl.gz`:
  14511953 bytes,
  SHA-256 `8f2edf54fa33e11cf832316a0abade2b34ed3cfe2d5c70e3cc628f1f8eff5f4f`,
  last changed in commit `21f33ad9a32b541819f3491a9a10600adb8fab1d`,
  printed by `git show 21f33ad9a32b541819f3491a9a10600adb8fab1d:doc/audit/firefox-esr-disk-responsiveness.candidates.jsonl.gz`.
- `doc/audit/firefox-esr-disk-responsiveness.inventory.json.gz`:
  14843647 bytes,
  SHA-256 `2c190ba306eee8fb61c271ffabc48cfcc47f8b5bb9ba073186392ebe9ed2fb17`,
  last changed in commit `21f33ad9a32b541819f3491a9a10600adb8fab1d`,
  printed by `git show 21f33ad9a32b541819f3491a9a10600adb8fab1d:doc/audit/firefox-esr-disk-responsiveness.inventory.json.gz`.
- `doc/audit/firefox-esr-disk-responsiveness.sources.json.gz`:
  52384 bytes,
  SHA-256 `2cc8f8102945a906e4256a26a7dead8246d0af235d69d8af861657ab18a13e44`,
  last changed in commit `b1b7669f03151c1f7417452987d4850923469eeb`,
  printed by `git show b1b7669f03151c1f7417452987d4850923469eeb:doc/audit/firefox-esr-disk-responsiveness.sources.json.gz`.

## Audit records, `doc/audit/depot-*`

70 archives left the tree in this family.

- `doc/audit/depot-arm64-functional-2026-10-18.sources.json.gz`:
  86403 bytes,
  SHA-256 `003e3155561cea4636cb927d1709e5e157fa27fa0995e49083632266558390d5`,
  last changed in commit `20abb5fa5b96353b646a2d25e0c8eca13c6ab49d`,
  printed by `git show 20abb5fa5b96353b646a2d25e0c8eca13c6ab49d:doc/audit/depot-arm64-functional-2026-10-18.sources.json.gz`.
- `doc/audit/depot-buildkit-platform-campaign-2026-10-18.sources.json.gz`:
  3573335 bytes,
  SHA-256 `cd9fff9610f40cbfd39ce9292ef104f8b7b7ef16e61d5c834fbcc71b7f78c40f`,
  last changed in commit `16d4af7a756729fdbeb440825883f84f62d1c0b3`,
  printed by `git show 16d4af7a756729fdbeb440825883f84f62d1c0b3:doc/audit/depot-buildkit-platform-campaign-2026-10-18.sources.json.gz`.
  Its cited records are kept in `doc/audit/depot-buildkit-platform-campaign-2026-10-18.sources.cited-records.json`.
  What the documents cite is kept,
  trimmed to the two Dockerfile records;
  the `depot-cargo-wrapper-boundary.json` record the platform ledger names is not,
  because the forbidden-strings guard refused the file that carried its
  `SCCACHE_WEBDAV_PASSWORD` fixture markers.
- `doc/audit/depot-cache-integration-document-source-review-2026-10-18.evidence.json.gz`:
  8210 bytes,
  SHA-256 `f92c3eb1f7e87bd59c393faa0df7a1f9e42f762bb287cbdb8697154b0c690f6d`,
  last changed in commit `1b28fb2da949ced0d57fdcaf19d47d5f07b20390`,
  printed by `git show 1b28fb2da949ced0d57fdcaf19d47d5f07b20390:doc/audit/depot-cache-integration-document-source-review-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-cache-trust-scope-2026-10-18.sources.json.gz`:
  1641463 bytes,
  SHA-256 `cab47536cdfa53aa741420880ea2a89000c3490f443ec9e577ee28a83279a1fa`,
  last changed in commit `77833ff47edd8ce593611a5e4cc017d6a3e51607`,
  printed by `git show 77833ff47edd8ce593611a5e4cc017d6a3e51607:doc/audit/depot-cache-trust-scope-2026-10-18.sources.json.gz`.
- `doc/audit/depot-complete-coverage-desktop-receipts.json.gz`:
  22756 bytes,
  SHA-256 `8b9be6f1b2cfb29540226d0a8c4b520d867d8d6a24b0eddda516198b26b26885`,
  last changed in commit `ab400e8cb74148a62eb567ccf1c2ebee850a1582`,
  printed by `git show ab400e8cb74148a62eb567ccf1c2ebee850a1582:doc/audit/depot-complete-coverage-desktop-receipts.json.gz`.
- `doc/audit/depot-complete-coverage-external-generation-receipts.json.gz`:
  1910 bytes,
  SHA-256 `899467f33d5eccfb01a1f053db71c773bf4406db86887131698ab59193f67c6b`,
  last changed in commit `ab400e8cb74148a62eb567ccf1c2ebee850a1582`,
  printed by `git show ab400e8cb74148a62eb567ccf1c2ebee850a1582:doc/audit/depot-complete-coverage-external-generation-receipts.json.gz`.
- `doc/audit/depot-complete-coverage-packed-consumer-receipts.json.gz`:
  1701669 bytes,
  SHA-256 `3a050fdd4e1dc025c0cff57efd77f056504b7fada677e25158f6b50b233a467b`,
  last changed in commit `898fd6ab03be33c0e478115671cf82f71fbf46c3`,
  printed by `git show 898fd6ab03be33c0e478115671cf82f71fbf46c3:doc/audit/depot-complete-coverage-packed-consumer-receipts.json.gz`.
- `doc/audit/depot-complete-coverage-source-feature-assertions.json.gz`:
  3341685 bytes,
  SHA-256 `11e25bc960076fd330bb923c3bd0aa586b52ec47824c7633198d1a9bd7910204`,
  last changed in commit `ab400e8cb74148a62eb567ccf1c2ebee850a1582`,
  printed by `git show ab400e8cb74148a62eb567ccf1c2ebee850a1582:doc/audit/depot-complete-coverage-source-feature-assertions.json.gz`.
- `doc/audit/depot-container-prerequisites-2026-10-18.sources.json.gz`:
  15117 bytes,
  SHA-256 `f713b99f1f6842cdf206ca2f447af351d6d05ff2ba638633c7a90f7ebadf8a76`,
  last changed in commit `922faafef162af0f153f1b83e1d761b210651f1a`,
  printed by `git show 922faafef162af0f153f1b83e1d761b210651f1a:doc/audit/depot-container-prerequisites-2026-10-18.sources.json.gz`.
- `doc/audit/depot-current-baseline-build-2026-10-18.log.gz`:
  19469 bytes,
  SHA-256 `a8aefe633a69a43eafb37147bd3cb0923dc2d7154a07816f1cfb2231fe289796`,
  last changed in commit `ac99991c648e7f703369ffb5acc7207ea020ff30`,
  printed by `git show ac99991c648e7f703369ffb5acc7207ea020ff30:doc/audit/depot-current-baseline-build-2026-10-18.log.gz`.
- `doc/audit/depot-current-baseline-ci-39197594870-2026-10-18.log.gz`:
  194192 bytes,
  SHA-256 `c85527ec93557fa0d07411b49be56161b26965fa3966986c7b77bb6d51ef4049`,
  last changed in commit `331cc20126d9e529d3f4b49c4828d21a5b6912c5`,
  printed by `git show 331cc20126d9e529d3f4b49c4828d21a5b6912c5:doc/audit/depot-current-baseline-ci-39197594870-2026-10-18.log.gz`.
- `doc/audit/depot-current-baseline-static-2026-10-18.log.gz`:
  822925 bytes,
  SHA-256 `790bf4d6ac1e97256ab5b03e61a4196fb0d6db2f571c56f1d83674015939e192`,
  last changed in commit `f9c7bd60825ae51fcf511cd0384dc26f2af8a06f`,
  printed by `git show f9c7bd60825ae51fcf511cd0384dc26f2af8a06f:doc/audit/depot-current-baseline-static-2026-10-18.log.gz`.
- `doc/audit/depot-current-baseline-unit-2026-10-18.log.gz`:
  160177 bytes,
  SHA-256 `0cd3ee73d1d1cc46ecb388ff9402643efd8a79fc9c87eb5bbebefef799848823`,
  last changed in commit `ac99991c648e7f703369ffb5acc7207ea020ff30`,
  printed by `git show ac99991c648e7f703369ffb5acc7207ea020ff30:doc/audit/depot-current-baseline-unit-2026-10-18.log.gz`.
- `doc/audit/depot-current-host-controls-admission-2026-10-18.evidence.json.gz`:
  25554 bytes,
  SHA-256 `c25fc9c62a39f31922736e61ba53fe71217c96e0c77497a613543962331a5ef3`,
  last changed in commit `046596174a061c31e4e2b8e8aaf7eca6565384a8`,
  printed by `git show 046596174a061c31e4e2b8e8aaf7eca6565384a8:doc/audit/depot-current-host-controls-admission-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-current-host-first-handoff-2026-10-18.evidence.json.gz`:
  121319 bytes,
  SHA-256 `c7d97ade72711bc3386591871419757f97f05afb5055f03b8e671dfeadd1cf85`,
  last changed in commit `1a562991f6d24f8284b931c7913fb3c30c5f3467`,
  printed by `git show 1a562991f6d24f8284b931c7913fb3c30c5f3467:doc/audit/depot-current-host-first-handoff-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-current-host-gnu-owner-2026-10-18.evidence.json.gz`:
  2046665 bytes,
  SHA-256 `19e672e8a099c63b4a070e4619f7c6bea8b844629d80a91cb5f8f88b4c7c2372`,
  last changed in commit `208b7da9a5532a2c4b136cc285f4e165a9696911`,
  printed by `git show 208b7da9a5532a2c4b136cc285f4e165a9696911:doc/audit/depot-current-host-gnu-owner-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-current-host-owner-comparison-2026-10-18.evidence.json.gz`:
  14207447 bytes,
  SHA-256 `c11ce61387c8675f99c303cad6093d444a13bc7c42adf24eb77f2bdd1f1677b7`,
  last changed in commit `c7d1c45a007d773dbd491bd9228611095286bfcc`,
  printed by `git show c7d1c45a007d773dbd491bd9228611095286bfcc:doc/audit/depot-current-host-owner-comparison-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-current-host-owner-complete-2026-10-18.evidence.json.gz`:
  14201235 bytes,
  SHA-256 `d29c7887cf4df674346d850db3f2186345a55cbc53bbacd98ef33c7977221a35`,
  last changed in commit `c7d1c45a007d773dbd491bd9228611095286bfcc`,
  printed by `git show c7d1c45a007d773dbd491bd9228611095286bfcc:doc/audit/depot-current-host-owner-complete-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-expanded-graph-conditional-waiting-value-2026-10-18.sources.json.gz`:
  5704 bytes,
  SHA-256 `254a0f69763e8cccfe6bd0f4d2785e757c8d4fde48e02edf232715a11876c5e5`,
  last changed in commit `655373399364dbfc4d3a6fcb9f2e35b99d8248a0`,
  printed by `git show 655373399364dbfc4d3a6fcb9f2e35b99d8248a0:doc/audit/depot-expanded-graph-conditional-waiting-value-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-github-repetition-2026-10-18.sources.json.gz`:
  154139 bytes,
  SHA-256 `f3fb9967cba82508ccf37968ea772c97e2363d39a61df7d2d69ee66d9b9e0a9b`,
  last changed in commit `05d02428a0a0322ab4267a7d522c593317541699`,
  printed by `git show 05d02428a0a0322ab4267a7d522c593317541699:doc/audit/depot-expanded-graph-github-repetition-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-native-correction-2026-10-18.sources.json.gz`:
  173906 bytes,
  SHA-256 `86b8686ea7e3496cdc9246fa2947b51968350f5031cff535e97ccceb36fb8ac9`,
  last changed in commit `f53c920cb24157db59873d3b24aaf51a493d6e32`,
  printed by `git show f53c920cb24157db59873d3b24aaf51a493d6e32:doc/audit/depot-expanded-graph-native-correction-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-native-parity-review-2026-10-18.sources.json.gz`:
  118332 bytes,
  SHA-256 `63ce6da88139b3b1941f38f015e630d6d4c91f38c07a96fc49fa3b001959c7cd`,
  last changed in commit `e0790e738da7e1c33e49fa200e9fc5105d11503a`,
  printed by `git show e0790e738da7e1c33e49fa200e9fc5105d11503a:doc/audit/depot-expanded-graph-native-parity-review-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-parity-review-2026-10-18.sources.json.gz`:
  154233 bytes,
  SHA-256 `117d2c7103877917eecc037f73ce013766b5f778b56324540de2f34f31aeaaca`,
  last changed in commit `088d45e80e2d29548541813f23821e8d16fa1290`,
  printed by `git show 088d45e80e2d29548541813f23821e8d16fa1290:doc/audit/depot-expanded-graph-parity-review-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-podman-preflight-2026-10-18.sources.json.gz`:
  18551 bytes,
  SHA-256 `122ef60af29ddb193445fead9080bac530ab5cb18879690c1b471154d05ce97c`,
  last changed in commit `5edbdadcc278cb95484aa1d4033495317513c75c`,
  printed by `git show 5edbdadcc278cb95484aa1d4033495317513c75c:doc/audit/depot-expanded-graph-podman-preflight-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-preflight-2026-10-18.sources.json.gz`:
  1096903 bytes,
  SHA-256 `4f173723e155cff0ae91c7c3c794c9ceeed930603f2960bb03b94cc87a233d01`,
  last changed in commit `eec08fd2264de03236d1953ec2cce7fa5841fff8`,
  printed by `git show eec08fd2264de03236d1953ec2cce7fa5841fff8:doc/audit/depot-expanded-graph-preflight-2026-10-18.sources.json.gz`.
- `doc/audit/depot-expanded-graph-setup-failure-2026-10-18.sources.json.gz`:
  2184 bytes,
  SHA-256 `caf8e72e975d2bba895c4e734f617d16ea3c2de4b6b13d06d633d987e4a06f01`,
  last changed in commit `eabb44a5e6ac03320cbbb488ed643c2d995e2877`,
  printed by `git show eabb44a5e6ac03320cbbb488ed643c2d995e2877:doc/audit/depot-expanded-graph-setup-failure-2026-10-18.sources.json.gz`.
- `doc/audit/depot-fast-subset-baseline-39199930641-2026-10-18.log.gz`:
  100237 bytes,
  SHA-256 `75f5c277894782b3727b463678e0580405769f4115c05db5a5eafe993ee9a36f`,
  last changed in commit `31b65e086e384a7fe9f1e9008d75aab2794ad9f1`,
  printed by `git show 31b65e086e384a7fe9f1e9008d75aab2794ad9f1:doc/audit/depot-fast-subset-baseline-39199930641-2026-10-18.log.gz`.
- `doc/audit/depot-gnu-design-terminal-2026-10-18.sources.json.gz`:
  2082916 bytes,
  SHA-256 `068977eea116b0c033c8a7ba286e776c38c0bf1f40ccfb26a528e818e6ed71c6`,
  last changed in commit `8d80a5ed1d4dbcbdb6d58b281437d40a0d85b146`,
  printed by `git show 8d80a5ed1d4dbcbdb6d58b281437d40a0d85b146:doc/audit/depot-gnu-design-terminal-2026-10-18.sources.json.gz`.
- `doc/audit/depot-image-list-empty-output-2026-10-18.sources.json.gz`:
  12512 bytes,
  SHA-256 `537a75bf0542f03cca5ea03fcdf0299f979aac67a5365d5691790c0d4a0ec617`,
  last changed in commit `d5e3e0e9101560fc716b19b967004e00ddfb60e0`,
  printed by `git show d5e3e0e9101560fc716b19b967004e00ddfb60e0:doc/audit/depot-image-list-empty-output-2026-10-18.sources.json.gz`.
- `doc/audit/depot-included-allowance-2026-10-18.sources.json.gz`:
  112370 bytes,
  SHA-256 `197dac272cc047780d092b6b3740c61091805a0aa7b87211cd4406ec1ba784c0`,
  last changed in commit `1434118fe0bb7bff146caa5cf31a291f1a857c3e`,
  printed by `git show 1434118fe0bb7bff146caa5cf31a291f1a857c3e:doc/audit/depot-included-allowance-2026-10-18.sources.json.gz`.
- `doc/audit/depot-job-policy-history-2026-10-18.archives.json.gz`:
  5899 bytes,
  SHA-256 `c96e692b6dacebaffac76c045c46286fbd49b92262b91b7395dd0bec1597ac6c`,
  last changed in commit `8b358eaa515d03e99f26e4b765592d3fbfbf0244`,
  printed by `git show 8b358eaa515d03e99f26e4b765592d3fbfbf0244:doc/audit/depot-job-policy-history-2026-10-18.archives.json.gz`.
- `doc/audit/depot-live-public-pricing-2026-10-18.evidence.json.gz`:
  8976 bytes,
  SHA-256 `c52a7c7cbbbae7719035a881194bdda8a07e1810f3743b83893cef6323095825`,
  last changed in commit `6daf14ae5a00cc41d19bc777531cce12267ef15f`,
  printed by `git show 6daf14ae5a00cc41d19bc777531cce12267ef15f:doc/audit/depot-live-public-pricing-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-local-figma-authored-consumer-2026-10-18.sources.json.gz`:
  998 bytes,
  SHA-256 `e2eb57e29c36e4785c360132f5659260ef74d7a39d41327909852be507ed1b7c`,
  last changed in commit `5a52544d1e65e8f29de9ce315e0698965b532c26`,
  printed by `git show 5a52544d1e65e8f29de9ce315e0698965b532c26:doc/audit/depot-local-figma-authored-consumer-2026-10-18.sources.json.gz`.
- `doc/audit/depot-local-node-capture-correction-2026-10-18.sources.json.gz`:
  9905 bytes,
  SHA-256 `6bc7ac7872561855779aa9b126b397e662de6bb1aa8454bb6bf29157a680b5c4`,
  last changed in commit `9412e49ee6259e4e9cacfc0b36d27660882d081c`,
  printed by `git show 9412e49ee6259e4e9cacfc0b36d27660882d081c:doc/audit/depot-local-node-capture-correction-2026-10-18.sources.json.gz`.
- `doc/audit/depot-local-node-counterfactual-2026-10-18.sources.json.gz`:
  8408 bytes,
  SHA-256 `bf7243ebbd570e912bf84fdca493c0c5a6ffda3de40211a9abdba97edc6a482d`,
  last changed in commit `1b29e57f37ea1a698501870026d7de1f07ebdf5d`,
  printed by `git show 1b29e57f37ea1a698501870026d7de1f07ebdf5d:doc/audit/depot-local-node-counterfactual-2026-10-18.sources.json.gz`.
- `doc/audit/depot-logger-host-development-2026-10-17.evidence.json.gz`:
  426661 bytes,
  SHA-256 `adc0a5e7a443843c07362fb479e5f7d951a71bcd6996e4d6166ea6de80eeb0ef`,
  last changed in commit `9c4121a4a43255323184d203dddf4740137323a8`,
  printed by `git show 9c4121a4a43255323184d203dddf4740137323a8:doc/audit/depot-logger-host-development-2026-10-17.evidence.json.gz`.
  Its only naming file,
  `doc/audit/depot-final-recovery-2026-10-17.evidence.json`,
  records at `/commonManifestTransportControls` its own digest
  `1a6f6aaee60910a6fe44c653308648ba8a19abf2dc3713ebe05e227ff1390369`
  of the 1043537 bytes at commit `8c70741076b02238bc9dab9fc10443dd9aea77e2`.
  Its note says that pair is not and cannot be current:
  a note that carries a file's own digest changes the file.
- `doc/audit/depot-material-owner-input-payload-identities-2026-10-18.evidence.json.gz`:
  211223 bytes,
  SHA-256 `6435ee9f846e5f964ef1ad408cc540adb053d8a37d95bde7264766aa7a87459e`,
  last changed in commit `8ccda67a087a67dd64922899f6ed559bc08718be`,
  printed by `git show 8ccda67a087a67dd64922899f6ed559bc08718be:doc/audit/depot-material-owner-input-payload-identities-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-material-owner-primary-2026-10-18.sources.json.gz`:
  533479 bytes,
  SHA-256 `253cca9bfa20a4bbc767bc0e09d47f659d84b7196dafe7ca8f847335bf3d3050`,
  last changed in commit `20d75368570ee045ddff43d67aef42fd245cce4b`,
  printed by `git show 20d75368570ee045ddff43d67aef42fd245cce4b:doc/audit/depot-material-owner-primary-2026-10-18.sources.json.gz`.
- `doc/audit/depot-mise-locked-affected-2026-10-18.sources.json.gz`:
  164869 bytes,
  SHA-256 `e49d561d5d1f2c3562c3df01db4eacd79865141ea1cf88e42b867dfb91eb4e84`,
  last changed in commit `a9541b020cd9c7a17fd37ff8f63696ff67c59cee`,
  printed by `git show a9541b020cd9c7a17fd37ff8f63696ff67c59cee:doc/audit/depot-mise-locked-affected-2026-10-18.sources.json.gz`.
- `doc/audit/depot-mvm-guest-files-2026-10-18.originals.json.gz`:
  63665 bytes,
  SHA-256 `6f0bf7c1ea1891b24d8702b373fc9c30b8eb885a0827e6e056cb483cac0183d9`,
  last changed in commit `c65e3b8eeb0f1bc0e06bf555048b85635ec59202`,
  printed by `git show c65e3b8eeb0f1bc0e06bf555048b85635ec59202:doc/audit/depot-mvm-guest-files-2026-10-18.originals.json.gz`.
- `doc/audit/depot-mvm-kvm-2026-10-18.archives.json.gz`:
  181117 bytes,
  SHA-256 `9ec38d2f8c3605e9400b0960a24d29196ad012c19f89fb9ce042c4b8e8d2a38f`,
  last changed in commit `f86e1ee1da76ae8bd4a6a08a6fd30fac238e5265`,
  printed by `git show f86e1ee1da76ae8bd4a6a08a6fd30fac238e5265:doc/audit/depot-mvm-kvm-2026-10-18.archives.json.gz`.
- `doc/audit/depot-native-baseline-2ls35xfpds-2026-10-19.log.gz`:
  131149 bytes,
  SHA-256 `6b4b5a581914ea3577a7334542048d6d0a9def769d7d93cb5dae2d2f2226fa06`,
  last changed in commit `eedccaa1afc057e114201ff2ed8055f420088957`,
  printed by `git show eedccaa1afc057e114201ff2ed8055f420088957:doc/audit/depot-native-baseline-2ls35xfpds-2026-10-19.log.gz`.
- `doc/audit/depot-native-buildkit-job-auth-2026-10-18.sources.json.gz`:
  128260 bytes,
  SHA-256 `73c1e95822919d9029144fd6892ecb5a9273f244db3ea39be634eac7e7c4b8a3`,
  last changed in commit `ba8e7e6ce3d68c72ba298efeeb80724fd144ad55`,
  printed by `git show ba8e7e6ce3d68c72ba298efeeb80724fd144ad55:doc/audit/depot-native-buildkit-job-auth-2026-10-18.sources.json.gz`.
  One record names an earlier version of this path:
  18768 bytes and SHA-256
  `e5d29d600540f529c333ec3bb0bf259dde04d2b1fd76af16428d58c03142a2a2`,
  held by commit `c1cef256ba096dca3879b4d9f04e633b0fe7b4a2`.
  That record is `/outOfSnapshotFollowOns/1/sourceAdmission/sourceArchive` in
  `doc/audit/depot-owned-allocation-ledger-2026-10-18.evidence.json`,
  and the note beside it says the same.
  The version tracked last is the pair above.
- `doc/audit/depot-native-ci-document-source-review-2026-10-18.evidence.json.gz`:
  13893 bytes,
  SHA-256 `672193a5ca02e030240664f0812f7b612e7d02b263158b36728346a8c7a29f7e`,
  last changed in commit `d63caec30a413e3ace1d352241059928e9482870`,
  printed by `git show d63caec30a413e3ace1d352241059928e9482870:doc/audit/depot-native-ci-document-source-review-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-native-common-before-negative-2026-10-17.evidence.zip`:
  60781 bytes,
  SHA-256 `927514e26567d5ffc53d11e3bbcd03eb140bc00722652c73b07ce5b91e610463`,
  last changed in commit `9c112c357a1f3403d50db060725ca777a285e448`,
  printed by `git show 9c112c357a1f3403d50db060725ca777a285e448:doc/audit/depot-native-common-before-negative-2026-10-17.evidence.zip`.
- `doc/audit/depot-native-orchestration-2026-10-18.archives.json.gz`:
  16062 bytes,
  SHA-256 `afab7c02d27a17c0e8ac5466fa10a6a12a270a15e77148a3bc7a1e066f99eca4`,
  last changed in commit `afd2f8efac0b53e563aeb19e5de31b81877f435b`,
  printed by `git show afd2f8efac0b53e563aeb19e5de31b81877f435b:doc/audit/depot-native-orchestration-2026-10-18.archives.json.gz`.
- `doc/audit/depot-native-snapshot-absence-2026-10-18.sources.json.gz`:
  39722 bytes,
  SHA-256 `9267311d49c9e029fec418aeb9d2af89ee77ddccaa947db08df60fbed80ffafc`,
  last changed in commit `49e5c84d1b8066bc9d974742bffbff684e67179a`,
  printed by `git show 49e5c84d1b8066bc9d974742bffbff684e67179a:doc/audit/depot-native-snapshot-absence-2026-10-18.sources.json.gz`.
- `doc/audit/depot-native-ssh-acceptance-2026-10-18.sources.json.gz`:
  2342905 bytes,
  SHA-256 `8c503d374116bb787f8eb2dd395db70654a822e887631cda94f859e471968923`,
  last changed in commit `3adb2516b16afcddc27429024090889b7ec3d3bb`,
  printed by `git show 3adb2516b16afcddc27429024090889b7ec3d3bb:doc/audit/depot-native-ssh-acceptance-2026-10-18.sources.json.gz`.
- `doc/audit/depot-native-ssh-correction-2026-10-18.sources.json.gz`:
  329845 bytes,
  SHA-256 `11118675368dcfecfcba9d6f3d3cee8856e6d6c9c1709eca9f78dc00d59b806a`,
  last changed in commit `a90cb3ef079b2ca868556b8bfc1fe72705c50af6`,
  printed by `git show a90cb3ef079b2ca868556b8bfc1fe72705c50af6:doc/audit/depot-native-ssh-correction-2026-10-18.sources.json.gz`.
- `doc/audit/depot-owned-allocation-final-2026-10-18.sources.json.gz`:
  386093 bytes,
  SHA-256 `98d63cf22dfc333016187266983e0f1512d89b401f5c1d88859d65cb314cc813`,
  last changed in commit `24ce11c66aeddc012458b05b10e6b940343469f4`,
  printed by `git show 24ce11c66aeddc012458b05b10e6b940343469f4:doc/audit/depot-owned-allocation-final-2026-10-18.sources.json.gz`.
- `doc/audit/depot-owned-allocation-follow-on-2026-10-18.sources.json.gz`:
  392421 bytes,
  SHA-256 `872651779e81a7ae6627c10c7392151fbe435cb97be456a8e409428ee6ca1b6d`,
  last changed in commit `3cac1b657b38a1194adb59b1b143a4f7552947a6`,
  printed by `git show 3cac1b657b38a1194adb59b1b143a4f7552947a6:doc/audit/depot-owned-allocation-follow-on-2026-10-18.sources.json.gz`.
  One record names an earlier version of this path:
  387034 bytes and SHA-256
  `a7a144cce9c0798f5e9a578e23d153f0375772b59fd21dde61edce2beacb8ae2`,
  held by commit `39ef85953417eb49705aef425c07d29e5da4e455`.
  That record is `/input/sourceArchive` in
  `doc/audit/depot-owned-allocation-follow-on-independent-2026-10-18.evidence.json`,
  and the note beside it says the same.
  The version tracked last is the pair above.
- `doc/audit/depot-owned-allocation-follow-on-independent-2026-10-18.sources.json.gz`:
  398912 bytes,
  SHA-256 `9a71dca1550e93b8a4cfebd18c28e53ad29347a93a1f31f6a93cddd2aa6903a5`,
  last changed in commit `846d2c13016c642da994b952f320988d17993940`,
  printed by `git show 846d2c13016c642da994b952f320988d17993940:doc/audit/depot-owned-allocation-follow-on-independent-2026-10-18.sources.json.gz`.
- `doc/audit/depot-owned-allocation-ledger-2026-10-18.sources.json.gz`:
  8006342 bytes,
  SHA-256 `17ddab66fd96b7fc0b6267405b3cf13ffe669262a2909025265190263130b1fa`,
  last changed in commit `9341e215df4df5a1462d9a39fa0462a357a4c779`,
  printed by `git show 9341e215df4df5a1462d9a39fa0462a357a4c779:doc/audit/depot-owned-allocation-ledger-2026-10-18.sources.json.gz`.
- `doc/audit/depot-physical-test-shards-2026-10-18.archives.json.gz`:
  37381 bytes,
  SHA-256 `00d0a6c49fecef1874b044527658e17a2c2ad0cce90a8cf88811d46d8ad96991`,
  last changed in commit `b2b1aa427b624eba065acfd5b3192eb4a98307a7`,
  printed by `git show b2b1aa427b624eba065acfd5b3192eb4a98307a7:doc/audit/depot-physical-test-shards-2026-10-18.archives.json.gz`.
- `doc/audit/depot-platform-surface-sources-2026-10-18.evidence.json.gz`:
  3883626 bytes,
  SHA-256 `d3d907ba12236df20900f65ef7109ac2b90c9a9a1ad71a03f1ae3b1bad2ca890`,
  last changed in commit `337b2a09e3db2be07ff84b75797544197ade066e`,
  printed by `git show 337b2a09e3db2be07ff84b75797544197ade066e:doc/audit/depot-platform-surface-sources-2026-10-18.evidence.json.gz`.
- `doc/audit/depot-process-timeouts-2026-10-18.archives.json.gz`:
  4979 bytes,
  SHA-256 `ae7580dcf37185b3422befd024f4f1fd7fa342e79f3c4d92c3dffafceb899d4e`,
  last changed in commit `d43f5c400319d6f5f33b919dccc3817aaf070aa9`,
  printed by `git show d43f5c400319d6f5f33b919dccc3817aaf070aa9:doc/audit/depot-process-timeouts-2026-10-18.archives.json.gz`.
- `doc/audit/depot-remaining-local-unit-2026-10-18.sources.json.gz`:
  5327 bytes,
  SHA-256 `22cecfc123460fbc5529654a5e622e0ae6e72d93249290b0f46d0cc6c8d2504f`,
  last changed in commit `594497b184ccb18726303e2fbabccd58c82a7422`,
  printed by `git show 594497b184ccb18726303e2fbabccd58c82a7422:doc/audit/depot-remaining-local-unit-2026-10-18.sources.json.gz`.
- `doc/audit/depot-reuse-lock-portability-2026-10-18.receipts.json.gz`:
  45098 bytes,
  SHA-256 `2e2a331991486c50af1fac361f04f99dcd964c4deb7b4b4dc599cd936b5f5609`,
  last changed in commit `c2263298da6eb50370ee9f1da7649d86ccad3047`,
  printed by `git show c2263298da6eb50370ee9f1da7649d86ccad3047:doc/audit/depot-reuse-lock-portability-2026-10-18.receipts.json.gz`.
- `doc/audit/depot-runtime-pr-2026-10-18.archives.json.gz`:
  5262 bytes,
  SHA-256 `fa455b4dd47abbfa7dce33d590a6ee1b0edf9d9d3657d35a829c0c6a1a062c1c`,
  last changed in commit `e2dc6e8413ad4dca242bf6713d82df9690c98688`,
  printed by `git show e2dc6e8413ad4dca242bf6713d82df9690c98688:doc/audit/depot-runtime-pr-2026-10-18.archives.json.gz`.
- `doc/audit/depot-rust-coverage-2026-10-18.catalog.json.gz`:
  538476 bytes,
  SHA-256 `6dda590c2b8cfe131ade20f2368cc366f2ef92e294a6b380b4f9ac5c8327d1ff`,
  last changed in commit `0c80930f4d1a714087b619644d3e19173d608a28`,
  printed by `git show 0c80930f4d1a714087b619644d3e19173d608a28:doc/audit/depot-rust-coverage-2026-10-18.catalog.json.gz`.
- `doc/audit/depot-rust-coverage-2026-10-18.receipts.json.gz`:
  11483893 bytes,
  SHA-256 `a6ef79bb57f66ffb891ff54dba0aa2744eb435bb6735a323cd2ccecfe51f650f`,
  last changed in commit `f4ea52b61326f9b00d7a0971b1e467aeb7c159ff`,
  printed by `git show f4ea52b61326f9b00d7a0971b1e467aeb7c159ff:doc/audit/depot-rust-coverage-2026-10-18.receipts.json.gz`.
  Its cited records are kept in `doc/audit/depot-rust-coverage-2026-10-18.receipts.cited-records.json`.
- `doc/audit/depot-rust-legacy-owner-search-2026-10-18.sources.json.gz`:
  14951 bytes,
  SHA-256 `957707da9ae62aea184452d44c749ae026f5acce24b01e67aaba81467738a05e`,
  last changed in commit `d09cf5f8d73910d88a5aac14ae50e3cfcc138505`,
  printed by `git show d09cf5f8d73910d88a5aac14ae50e3cfcc138505:doc/audit/depot-rust-legacy-owner-search-2026-10-18.sources.json.gz`.
- `doc/audit/depot-semantic-package-feature-assertion-manifest-2026-10-18.json.gz`:
  2484600 bytes,
  SHA-256 `722162765d5f8eaa65b5beac7d7c648624cf633894ef182b0049f95f82d2592f`,
  last changed in commit `5c28fbf04670f0c5afd9f195f4be5dbf2d1977fc`,
  printed by `git show 5c28fbf04670f0c5afd9f195f4be5dbf2d1977fc:doc/audit/depot-semantic-package-feature-assertion-manifest-2026-10-18.json.gz`.
- `doc/audit/depot-semgrep-invocation-2026-10-18.sources.json.gz`:
  12731 bytes,
  SHA-256 `7f63cb6479bccb4295cad18cfc553c6d7e9e98c69a7a27f840fd79a926d00015`,
  last changed in commit `ed03473f9dfbe354df788f274e3c539e93be4762`,
  printed by `git show ed03473f9dfbe354df788f274e3c539e93be4762:doc/audit/depot-semgrep-invocation-2026-10-18.sources.json.gz`.
- `doc/audit/depot-service-containers-2026-10-18.archives.json.gz`:
  20148 bytes,
  SHA-256 `1d548493228ec9950b9de34d9b70b1107734a72a600e7ee9a7ad3218fc1a6b20`,
  last changed in commit `0a8a2979e091970f4480c38544978c19ace59f93`,
  printed by `git show 0a8a2979e091970f4480c38544978c19ace59f93:doc/audit/depot-service-containers-2026-10-18.archives.json.gz`.
- `doc/audit/depot-tests-identity-adapter-2026-10-18.sources.json.gz`:
  256546 bytes,
  SHA-256 `3b4525feba218b90c267124259fb9ad3eb8ab6701f98c38a36ff3ba2c40962bb`,
  last changed in commit `e212b700ab7e1e80c9bee61e800da42889dd458d`,
  printed by `git show e212b700ab7e1e80c9bee61e800da42889dd458d:doc/audit/depot-tests-identity-adapter-2026-10-18.sources.json.gz`.
- `doc/audit/depot-toml-full-mutation-2026-10-17.evidence.zip`:
  12057855 bytes,
  SHA-256 `31e45cf2ad99de72d9253c70e1057f050f985c617c1d69a2ee3402f5ec1fa5e3`,
  last changed in commit `eb5da82579aa99ab8a4486262736fd05f5b6d603`,
  printed by `git show eb5da82579aa99ab8a4486262736fd05f5b6d603:doc/audit/depot-toml-full-mutation-2026-10-17.evidence.zip`.
  Its cited records are kept in `doc/audit/depot-toml-full-mutation-2026-10-17.evidence.cited-records.md`.
- `doc/audit/depot-toml-full-reconciliation-2026-10-17.evidence.json.gz`:
  710921 bytes,
  SHA-256 `7bbdb669aec26a41aec88713f77ad39311cc5777caa112d06099c8eaea279c39`,
  last changed in commit `eb5da82579aa99ab8a4486262736fd05f5b6d603`,
  printed by `git show eb5da82579aa99ab8a4486262736fd05f5b6d603:doc/audit/depot-toml-full-reconciliation-2026-10-17.evidence.json.gz`.
- `doc/audit/depot-toml-mutation-native-2026-10-17.evidence.json.gz`:
  296300 bytes,
  SHA-256 `baa57122526e9d5093b6b8e18e208bd492680b45b54dc10ac2aa213fe96e8b1d`,
  last changed in commit `893b26838e79ac5af4a204db6cfad823de4f0c9b`,
  printed by `git show 893b26838e79ac5af4a204db6cfad823de4f0c9b:doc/audit/depot-toml-mutation-native-2026-10-17.evidence.json.gz`.
- `doc/audit/depot-toml-spawn-control-2026-10-17.evidence.zip`:
  100312 bytes,
  SHA-256 `88073f4fd7a6e22b0c268e63d70eca7e89b01448174917b712adc34a98bcac12`,
  last changed in commit `5d3d964bf514a741fc664fd5401340082a908e7c`,
  printed by `git show 5d3d964bf514a741fc664fd5401340082a908e7c:doc/audit/depot-toml-spawn-control-2026-10-17.evidence.zip`.
  Its cited records are kept in `doc/audit/depot-toml-spawn-control-2026-10-17.evidence.cited-records.md`.

## Records kept in the tree for a removed archive

5 of the removed archives are cited for something a reader can
only check by opening the archive:
a figure it would have to recompute,
a member path with what it holds,
or a record it names.
For those,
the records that back the statements are kept as plain text beside the
documents,
and every other removed archive is named only as the place the raw
material was kept,
which this record resolves.

### `doc/audit/depot-buildkit-platform-campaign-2026-10-18.sources.cited-records.json`

The two `.files[]` records the documents cite by commit and path,
with their bodies:
`ci-context/Dockerfile` at commit `0b471538cd`,
and `ci-node26-context/Dockerfile` at commit `7590bcde15`,
whose SHA-256 is the one the document names.
The archive's `depot-cargo-wrapper-boundary.json` record is not kept:
its `SCCACHE_WEBDAV_PASSWORD` fixture marker keys match the forbidden-strings rule `hashicorp-tf-password`,
and the guard refused the file that carried them;
read that record from the archive with the print command.

Cut from `doc/audit/depot-buildkit-platform-campaign-2026-10-18.sources.json.gz` at commit `16d4af7a756729fdbeb440825883f84f62d1c0b3`.
The archive is 3573335 bytes,
SHA-256 `cd9fff9610f40cbfd39ce9292ef104f8b7b7ef16e61d5c834fbcc71b7f78c40f`.

### `doc/audit/depot-rust-coverage-2026-10-18.receipts.cited-records.json`

The extension object the research record walks to by `run.run_id`,
and the `selected` entry it names.

Cut from `doc/audit/depot-rust-coverage-2026-10-18.receipts.json.gz` at commit `f4ea52b61326f9b00d7a0971b1e467aeb7c159ff`.
The archive is 11483893 bytes,
SHA-256 `a6ef79bb57f66ffb891ff54dba0aa2744eb435bb6735a323cd2ccecfe51f650f`.

### `doc/audit/depot-toml-full-mutation-2026-10-17.evidence.cited-records.md`

The two fixture members the audit records with `error: embedded-json-syntax`,
as text.

Cut from `doc/audit/depot-toml-full-mutation-2026-10-17.evidence.zip` at commit `eb5da82579aa99ab8a4486262736fd05f5b6d603`.
The archive is 12057855 bytes,
SHA-256 `31e45cf2ad99de72d9253c70e1057f050f985c617c1d69a2ee3402f5ec1fa5e3`.

### `doc/audit/depot-toml-spawn-control-2026-10-17.evidence.cited-records.md`

The two fixture members the audit records with `error: embedded-json-syntax`,
as text.

Cut from `doc/audit/depot-toml-spawn-control-2026-10-17.evidence.zip` at commit `5d3d964bf514a741fc664fd5401340082a908e7c`.
The archive is 100312 bytes,
SHA-256 `88073f4fd7a6e22b0c268e63d70eca7e89b01448174917b712adc34a98bcac12`.

### `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence/`

Every member the five documents cite by folder or by name,
byte for byte except the six that the repository own guards changed:
five markdown members re-wrapped and given fence labels,
and one text file whose final newline was made canonical.

Cut from `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip` at commit `a6783a44ae7bdd43f9cd6995fcb6416eea1250d1`.
The archive is 631659 bytes,
SHA-256 `0f2a70bd4504b1c8f58cb5234feb5354e5fa490fa7933019106195517d892352`.

Kept members,
at their archive-relative paths under that folder.
A member the repository changed carries the digest it had in the archive first
and the digest it has in the tree second.

- `agent-scaling-report.md`:
  21653 bytes,
  SHA-256 `384ce4184137ad4bc2d892870dad886c0cccd6fd815aefeac03497bd78a23a32`;
  in the tree 22381 bytes,
  SHA-256 `6817eb48633dd90f45652783163b03e321ec4d4cc57df7ff08bd483cf9a94e2e`.
- `coordinator-check/literal-no-match-rounds.jsonl`:
  1134 bytes,
  SHA-256 `571bba4495f840dece8c266779cc677b3f74c790d46a2ac2cac276da004c833c`.
- `coordinator-check/one-typo-rounds.jsonl`:
  782 bytes,
  SHA-256 `0babc707f261529bf489832c0ed856d162a739de8b77fce823f4fe7f5927ba57`.
- `coordinator-check/paired-statistics.ts`:
  3568 bytes,
  SHA-256 `d460eeb0e52bdb1884b5d6bd1d55ee2e654b5d268f73227328d4993669c8e0bc`.
- `coordinator-check/reproducer-rounds.jsonl`:
  2794 bytes,
  SHA-256 `50b006101d01cc8f561187266e74057f6ba7ea6243224c43e8c7694b8bb60eb0`.
- `coordinator-check/separator-variant-rounds.jsonl`:
  840 bytes,
  SHA-256 `73d31bdd0e152bef92ed9b47442fac4ce23b325db91757a47372eb1d74f7daaf`.
- `coordinator-check/token-quadratic-rounds.jsonl`:
  2549 bytes,
  SHA-256 `6589386a465cde20a99bd2cac05f51ca111a2c672a370f9c9150846a1ef9df00`.
- `measurements-2026-11-13/Containerfile.older-lines`:
  1611 bytes,
  SHA-256 `c398f08360128d5c1979aed266f93c9917a02f69a03e14b8c95424c2869b6bd1`.
- `measurements-2026-11-13/Containerfile.upstream-arms`:
  1212 bytes,
  SHA-256 `c0a1004464360fd413f1b559001f0de2112cb9532e1a5fe9257a93e8027ed40f`.
- `measurements-2026-11-13/Containerfile.work-current-bun`:
  617 bytes,
  SHA-256 `a57c494aaef7f76932ec252e99344fbcc3b17e868b29c9831b4efb8898099f8f`.
- `measurements-2026-11-13/analyse-profile.ts`:
  3556 bytes,
  SHA-256 `5e6cc9c74cad539e419db136ee0eacd814297b8b21f34f343499a2225c8268ad`.
- `measurements-2026-11-13/bun-suite-comparison-on-current-bun.jsonl`:
  1529 bytes,
  SHA-256 `0035820ca72bf750b976072cb3a9cff2c228dba5227f18e2381230bff61a04df`.
- `measurements-2026-11-13/bun-suite-comparison.jsonl`:
  1612 bytes,
  SHA-256 `fcef8ce929ba52578e943748c168433d410c4c67eabbb9a04034e202889e492b`.
- `measurements-2026-11-13/bun-suite-with-and-without-the-patch.ts`:
  3257 bytes,
  SHA-256 `ea5a22c80308311e502b444803b0bc0565780b01231162cd34669595651772b9`.
- `measurements-2026-11-13/close-review-2-gaps.jsonl`:
  1751 bytes,
  SHA-256 `98fc6131c23a71d23d854a6cb677c81c09fd06e5ffb1b94261d797eded1daaa7`.
- `measurements-2026-11-13/close-review-2-gaps.ts`:
  2757 bytes,
  SHA-256 `ff71e22d2dbaae15f4dbd3102106aced82ef2ef25f3e23ae20a0d5b917191784`.
- `measurements-2026-11-13/cross-runtime/Containerfile`:
  2951 bytes,
  SHA-256 `e44583b79f46bdddabf26ceb6444005e720069faf7de4545808675406c323c81`.
- `measurements-2026-11-13/cross-runtime/deno-variants.log`:
  1350 bytes,
  SHA-256 `e1e8851091dbbcd338f90c099083a5f6c4902f03bc87fce7f2efec314bc8a242`.
- `measurements-2026-11-13/cross-runtime/results.jsonl`:
  40699 bytes,
  SHA-256 `e0eac76163bfda55e42dbc77c892920a1cfd73a23d3f9e7bfec8fabe7230888e`.
- `measurements-2026-11-13/cross-runtime/run-matrix.ts`:
  4035 bytes,
  SHA-256 `03b8690daa1f49febf0eafef99d9816d7fe9bd0bc314afa5a5ce6a807dfe4e36`.
- `measurements-2026-11-13/cross-runtime/summarise.ts`:
  3459 bytes,
  SHA-256 `842fd7cc3c2b305193a519de2ddd91ea379fb22aa789d68fd54a3b327db6a742`.
- `measurements-2026-11-13/cross-runtime/summary.jsonl`:
  5524 bytes,
  SHA-256 `7bd600ffa0929f3920ddc7804e07ff9285db2dd073e3eff5b2dd30c8f56ebff2`.
- `measurements-2026-11-13/older-lines-results.jsonl`:
  28274 bytes,
  SHA-256 `5c3f560cab43909ec0f580ab457e4491b2e14909af0c222f0f4f613eca6fe32c`.
- `measurements-2026-11-13/patched-profile-2.log`:
  3433 bytes,
  SHA-256 `08b346871a37668da806176c6357d8dabc5e8ddb54f4c3cc6c8d3bb265cdb15c`.
- `measurements-2026-11-13/patched-profile.log`:
  1303 bytes,
  SHA-256 `8d47e2b152b1ca0ba5fc17d736d3c558411eeba70d608ac60b1739ae87fbee0d`.
- `measurements-2026-11-13/prototype-recheck-2.jsonl`:
  1922 bytes,
  SHA-256 `48adcc9088d8fd3d8b909de275478619b30c42938793b19a1da67851f1931bef`.
- `measurements-2026-11-13/prototype-recheck.jsonl`:
  3401 bytes,
  SHA-256 `6949aa6823ac6a93cb27a684f1e0c1bebe9edff8484a1f709f88c64d164704be`.
- `measurements-2026-11-13/recheck-2026-11-13.ts`:
  4320 bytes,
  SHA-256 `12662a969b3259f659dde195cc11f50e8393116607853873ea4b11285a603621`.
- `measurements-2026-11-13/recheck-after-test-removal.ts`:
  3802 bytes,
  SHA-256 `c7385431ae9295679da5e7c275d21f7dc0b5e484a613edae14da395b768fc990`.
- `measurements-2026-11-13/reproduction-run-43443075690.jsonl`:
  993 bytes,
  SHA-256 `0d635edb9e97254ca6715cc2afa0d8223ab749b260f065041ec3cdb67bf582fb`.
- `measurements-2026-11-13/run-arms.ts`:
  2115 bytes,
  SHA-256 `0bf1826d0a7d4a0e59db6faadcff49d3ada941d298341232a6fd3827f2414b87`.
- `measurements-2026-11-13/run-bun-comparison-on-current-bun.ts`:
  2194 bytes,
  SHA-256 `6b36d54936ed2008e75c76f573edc0d0360010573af6863e3611990aafa21c65`.
- `measurements-2026-11-13/run-older-lines.ts`:
  1621 bytes,
  SHA-256 `ae38fa03a5e346debea9d7b4ea9aff894d70bcdf1fee4ad44004e1f9b50dc1f6`.
- `measurements-2026-11-13/stage-upstream-arms.ts`:
  2869 bytes,
  SHA-256 `d23c7f6f854dd21bd9b604d1102681517e018bee62bb2fe2fa5ed5c97075c9e2`.
- `measurements-2026-11-13/summarise-arms.ts`:
  2916 bytes,
  SHA-256 `b130d2687e252f3c7da282916b0056678490e8bc5017ba14922c3ce965fbbc0f`.
- `measurements-2026-11-13/unpatched-profile.log`:
  1529 bytes,
  SHA-256 `b6dd11eccd6babb2ec2e4bc15967a251e62e6553b78dcd6f7f6739c425ab674b`.
- `measurements-2026-11-13/upstream-arms-results.jsonl`:
  61036 bytes,
  SHA-256 `134b050e43c03e40170bb7d29a3bf95405166e992e0c7f4ffc2b288e7a59f831`.
- `measurements-2026-11-13/upstream-arms-summary.jsonl`:
  2920 bytes,
  SHA-256 `0d906a996579113089f36f146240f1e7d7dd6afe6e23e197a2690c95952b2d42`.
- `measurements-2026-11-13/verify-links-2.jsonl`:
  550 bytes,
  SHA-256 `d4adbe0b9301875efe90e7609b9af1d012e98ef4b6760a8b83d699058445a368`.
- `measurements-2026-11-13/verify-links-2.ts`:
  1876 bytes,
  SHA-256 `3b126a435178ddbcb133cfb36d598fd4a9e159a3ed26f9d04774db8009c5767b`.
- `measurements-2026-11-13/verify-links.jsonl`:
  1603 bytes,
  SHA-256 `0487218af42740b2032c369ce4f7684243a7c80500ce14bb0a44e8c1de6f302a`.
- `measurements-2026-11-13/verify-links.ts`:
  2585 bytes,
  SHA-256 `33c905bee60458ece6c5a8e7eaf4e1f12c76e81ac8ff8a2ca6c548ba6dd34baa`.
- `pull-request-1121-on-node-and-bun-2026-11-13/.containerignore`:
  22 bytes,
  SHA-256 `1ce1c451161f5829c029aceeb9ffacfe7cfc0a371de259e8321da812bde19148`.
- `pull-request-1121-on-node-and-bun-2026-11-13/Containerfile`:
  1662 bytes,
  SHA-256 `bc215b497e5a12f60316a00708ddaf986cdf9fe2fcbe29518ca554a3016fb697`.
- `pull-request-1121-on-node-and-bun-2026-11-13/add-evidence.ts`:
  3857 bytes,
  SHA-256 `6cde1d58ff30fa00ed032452ab6e938a34a1af7cec86166f89de288285b5efec`.
- `pull-request-1121-on-node-and-bun-2026-11-13/keep/arm-package-build.jsonl`:
  189 bytes,
  SHA-256 `8518a0ea82487bdbaf8e22357aa19f1808287822beaf94b5c74a0b7da762b79b`.
- `pull-request-1121-on-node-and-bun-2026-11-13/keep/staging.json`:
  738 bytes,
  SHA-256 `0d5f2355b4994140b913f9f32866e9aecddb0f061270ee636d3e91e265df6f32`.
- `pull-request-1121-on-node-and-bun-2026-11-13/keep/timing-rounds.jsonl`:
  52655 bytes,
  SHA-256 `0e0bb8ca8937811a70b9386da10742fad7e16850bc90ec2e7f0a28e46e14bc0a`.
- `pull-request-1121-on-node-and-bun-2026-11-13/keep/timing-summary.txt`:
  4132 bytes,
  SHA-256 `49ce895322da1397b3b5d5f324da679461aa86a1b8efe2b5c34263dea3be24f5`.
- `pull-request-1121-on-node-and-bun-2026-11-13/run-pull-request-1121-on-node-and-bun.ts`:
  5357 bytes,
  SHA-256 `e864b7cd73f9373f09ef77aff460243900868f26d4d7567cf2f65e3e84ab8e4a`.
- `pull-request-1121-on-node-and-bun-2026-11-13/run-timing-inside.ts`:
  2507 bytes,
  SHA-256 `906ff62979137ed2f4c5699eef1bfa2ae332218701dbb8ff57e065f49796ab93`.
- `pull-request-1121-on-node-and-bun-2026-11-13/summarise.ts`:
  3489 bytes,
  SHA-256 `fc875f96decc8c67f6b87ca457edfb32013e1156df426c90e1645ed1ccec7b30`.
- `recast-to-ratios-2026-11-13/check-live-readme.log`:
  776 bytes,
  SHA-256 `a791d438a62b9a61dc526c1ac5fee4589fc3cd20aba655eee013923cd1423214`.
- `recast-to-ratios-2026-11-13/check-live-readme.ts`:
  2051 bytes,
  SHA-256 `50ac4d5058400a38f1286d2d9119a90aade0b76a9d0bac5bab3b93196bfa5684`.
- `recast-to-ratios-2026-11-13/keep/length-sweep.txt`:
  6599 bytes,
  SHA-256 `746643227feb019b67329d96962bdb0090a760dae23deb86582095c1b7e77f38`.
- `recast-to-ratios-2026-11-13/show-length-sweep.ts`:
  1418 bytes,
  SHA-256 `cdf0053c361d07cef8a0d86cdc9ca6ac8b062ac402d1ccc7c53f5c43fcc69d6d`.
- `results-scaling/fit-scaling.txt`:
  9172 bytes,
  SHA-256 `8573bc29e7f2565f0800b98fd7ba99d2ce2b1e0f46ab167eb8711ec089d47271`.
- `results-scaling/fit.json`:
  54531 bytes,
  SHA-256 `89c2f13eba257107e554073e3139c98b5f9f5beb7210fc6dd6419f51f21efbc3`.
- `results-scaling/summarise-search.txt`:
  294 bytes,
  SHA-256 `37fc472fa880edaea181c519c2e7b4274d0008ff29cc1a1aec086ffa9957991b`.
- `results/behavior-comparison.json`:
  12964 bytes,
  SHA-256 `9c758c79d6792972f77c6eca9167be65eb60624b54c7cd5e8217d492b4af1f23`.
- `results/behavior.jsonl`:
  137261 bytes,
  SHA-256 `ed08a7cfe2b8e1db1f7c9b5e5ac2648f92b1f96812c4ebb78de8e4f5e1f5533d`.
- `results/behavior.log`:
  0 bytes,
  SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `results/compare-behavior.txt`:
  3671 bytes,
  SHA-256 `ca0e07c0a4310247808cb54ff64afb53f66c75d024c166010745577c62b785a2`.
- `results/profiles.jsonl`:
  10518 bytes,
  SHA-256 `03297d3b471f310e1fc95d10a5f50ec69ced7e27bddd5e8832791f642c75d3c4`.
- `results/profiles.log`:
  213 bytes,
  SHA-256 `c99a42291f6323a33fc2a172da3c3efe5a69ae5928258530eb22dbb7babc5521`.
- `results/report-tables.md`:
  31570 bytes,
  SHA-256 `5da58e4a90dfee289578a1c32c9ecc5b8d5b6287f8d663143619bb43779ba8e6`;
  in the tree 33485 bytes,
  SHA-256 `47547d6e9b1f8b4db50ec3d54c9a76626523ec1747e3f83731fe039644f68150`.
- `results/report-tables.txt`:
  31570 bytes,
  SHA-256 `5da58e4a90dfee289578a1c32c9ecc5b8d5b6287f8d663143619bb43779ba8e6`;
  in the tree 31569 bytes,
  SHA-256 `d05e32a7ebf65434eea68b8bbd1f4ab36002c2fde026c093922de2bf0d438686`.
- `results/run-meta.json`:
  1051 bytes,
  SHA-256 `0df5914aca8e78aaf283882dde6d649e40b59e7820cffb464b2b47ede42e435f`.
- `results/runs.jsonl`:
  460706 bytes,
  SHA-256 `0993201ebeea28d6aa18fb5b093ad7cef20028a12a27e3a3cfed6f4c65388c8b`.
- `results/runs.log`:
  97751 bytes,
  SHA-256 `d9f57793f3688e5c3cc282dfe5da67bc3efef91c312985f794b4a60fa7ad980e`.
- `results/spread-summary.json`:
  4676 bytes,
  SHA-256 `7116008bf053bc6ebd3290846e463ed7f606c58ec4e521cab99b270b9ae4ddd9`.
- `results/spread.jsonl`:
  165253 bytes,
  SHA-256 `dcf00decd56525d50d1d120b52083bb077a54517858e102492e942794a35e053`.
- `results/spread.log`:
  37166 bytes,
  SHA-256 `73a88e6cc30981f9e7a64a4a622ab110c867bd517488d7da14d53c863011f9db`.
- `results/spread.txt`:
  1711 bytes,
  SHA-256 `e94b4885db656a4401e867c1db73a79e3a46f5bafeca201cdb0ac95d2e1bbad3`.
- `results/summarise.txt`:
  19572 bytes,
  SHA-256 `70a26fa3fa906ebbc379d2ab191c009a33fc0e58e58c28ba09d30e48c689d510`.
- `results/summary.json`:
  101537 bytes,
  SHA-256 `b1dc11fbf1f27a6fa862494746771021789a4c589e5c9c9533d32254f4a5afd5`.
- `two-remaining-copies-recheck-2026-11-13/Containerfile.annotated`:
  735 bytes,
  SHA-256 `3e767600df7aab20158640fe13ae8c63f21a4a8ab762a83c87e848b180b1fdc3`.
- `two-remaining-copies-recheck-2026-11-13/Containerfile.kept-patch`:
  827 bytes,
  SHA-256 `2e0780e7a129e37cd09760571bf3f2193e5611f0f146a754bedaec8e808eb7cb`.
- `two-remaining-copies-recheck-2026-11-13/Containerfile.mixed-threshold`:
  521 bytes,
  SHA-256 `fbdae552307c3c05b997f2ece9664f1bbf506fce2044f010dfc7f9aef6c3c601`.
- `two-remaining-copies-recheck-2026-11-13/Containerfile.node-and-bun`:
  1046 bytes,
  SHA-256 `c72fb5dd90fb7f4058cd7adc4286b609e6fcf39b1375aeb6640ff33b986acede`.
- `two-remaining-copies-recheck-2026-11-13/Containerfile.recheck`:
  1203 bytes,
  SHA-256 `8732d8c46470a599d12444b6dcc0423a16314a592a3e6d23e20f316b01e6851c`.
- `two-remaining-copies-recheck-2026-11-13/add-recheck-evidence.ts`:
  6088 bytes,
  SHA-256 `37865a1c749b9a31838bf07bf8e003edf208e0c834291735dd9f4299f4f30e77`.
- `two-remaining-copies-recheck-2026-11-13/compact-timing-view.ts`:
  1811 bytes,
  SHA-256 `12e77e7dcec57dd3da15b2ed90c3d701a84a0eece39c2376928572e2f2841fcb`.
- `two-remaining-copies-recheck-2026-11-13/derive-views-from-two-arm.ts`:
  1275 bytes,
  SHA-256 `1c69362d0bdea13f1f19200f37cfaadad8228ea3b2e69832d4a5ab12a45da88f`.
- `two-remaining-copies-recheck-2026-11-13/image/annotated-scaling.ts`:
  3253 bytes,
  SHA-256 `871e018e755902500c49b7c8efece08e099ed3d4f62a6dddae784c6ae7aa9517`.
- `two-remaining-copies-recheck-2026-11-13/image/mixed-scaling.ts`:
  3310 bytes,
  SHA-256 `2a7b31a74979863784f27baaabc7b459b27911035df2fb0f6b7194ff24b2f084`.
- `two-remaining-copies-recheck-2026-11-13/image/run-annotated.ts`:
  1178 bytes,
  SHA-256 `59ff07277f8cb2d9d2f486cedec18e1de3a523219c255baac2ec7f620e781837`.
- `two-remaining-copies-recheck-2026-11-13/image/run-kept-patch-timing.ts`:
  1953 bytes,
  SHA-256 `749243556507226c2a8fccbc5b80407bd04754e01636a704968f6043782136be`.
- `two-remaining-copies-recheck-2026-11-13/image/run-mixed-threshold.ts`:
  1799 bytes,
  SHA-256 `e031d3648989401d6e57a12f271bc4ff900a56a0bf4b8396d7e813bc29e020f9`.
- `two-remaining-copies-recheck-2026-11-13/image/run-node-and-bun-timing.ts`:
  3452 bytes,
  SHA-256 `b5e6ed4c0333972d0c0cf0fbfc1041e4ae8d5c48404b3fb7119ac8ef3dc7bb4b`.
- `two-remaining-copies-recheck-2026-11-13/image/run-timing.ts`:
  4520 bytes,
  SHA-256 `4ef11d2de3e6b6294543f34e366e877c1ae1bdda095847e747887a578fc74a05`.
- `two-remaining-copies-recheck-2026-11-13/image/view-facts.ts`:
  3848 bytes,
  SHA-256 `35c02b60ec8610735e108e9daa32ad5548a799e4457e6b39a7c74ac996a0d64b`.
- `two-remaining-copies-recheck-2026-11-13/keep/annotated-parse-timing.jsonl`:
  12058 bytes,
  SHA-256 `208b49d6671e6348b6ee5f48f5a61d096ef00532ea4db5f1bf4c8d6506502737`.
- `two-remaining-copies-recheck-2026-11-13/keep/arm-packages-build.jsonl`:
  710 bytes,
  SHA-256 `4a2fe69e468bff7b0c9b4fec6b98471ed7ea3557893fefc61a8a1608f4becf03`.
- `two-remaining-copies-recheck-2026-11-13/keep/behaviour-probes-on-all-arms.jsonl`:
  12815 bytes,
  SHA-256 `fd1f7177fc8c54094a6c21d4e8148b04e1e2dfa6dddd3fb7115cd20f3a274034`.
- `two-remaining-copies-recheck-2026-11-13/keep/behaviour-probes-on-the-kept-patch.jsonl`:
  11644 bytes,
  SHA-256 `18b6e1ce55cc20571823a3f6243e172a5c913c99a57fb76273e3b44a725f5506`.
- `two-remaining-copies-recheck-2026-11-13/keep/derive-views-from-two-arm.jsonl`:
  265 bytes,
  SHA-256 `1df032ec9192cb5722462b3d0385ed18c3e1208eb60e015e2e184ad3c36f78fd`.
- `two-remaining-copies-recheck-2026-11-13/keep/freeze-aftermath.jsonl`:
  9848 bytes,
  SHA-256 `7fb98505a71bff75986cca702281d44834a4c382ddcfb2a686d41c440e356e51`.
- `two-remaining-copies-recheck-2026-11-13/keep/make-final-multiple-arm.jsonl`:
  1847 bytes,
  SHA-256 `8809b27a32d293a5eae5f96db9240793d40e71b9f2364637e4ff7de63e355362`.
- `two-remaining-copies-recheck-2026-11-13/keep/multiple-state-fix-with-recheck-changes.patch`:
  52514 bytes,
  SHA-256 `2dfa46ae66f18982b185ba29fd141a5274c8b674f608185f776bd14b32b3e6e3`.
- `two-remaining-copies-recheck-2026-11-13/keep/one-bun-test-file-repeated-on-two-arms.jsonl`:
  1624 bytes,
  SHA-256 `d45520330dae63ce5900d523433221a1e6569c687fe882a40e77e323bb81a8e4`.
- `two-remaining-copies-recheck-2026-11-13/keep/stage-arms-for-recheck.jsonl`:
  5206 bytes,
  SHA-256 `743dc23a1af69b3cad80380e9bce13fc8888e318c6b8cf7edfac78dc5f1ba637`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-round-1-large-base.jsonl`:
  1514 bytes,
  SHA-256 `0c84287f04e9cae6864e0a0fce941ef9b48e1a934b11ea965f47b2b71153c3e8`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-round-1.jsonl`:
  34810 bytes,
  SHA-256 `8a7d236c511a7350d416eacc97711ec2314a758bc79c1ad88408d28d05cd496f`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-round-kept-patch.jsonl`:
  32625 bytes,
  SHA-256 `0c93523b918850726188dfd0ab32d2fc902e131d3ed4385725bb45486e789b87`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-round-mixed-threshold.jsonl`:
  20903 bytes,
  SHA-256 `a6dceef02c1b3007c77678b8696c59b95984beb0f062594ef8187773763425a7`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-round-node-and-bun.jsonl`:
  113492 bytes,
  SHA-256 `2980adcb28c0cc2271f0b0a42fbb1cb3da541118911eb81e8efe0b6db4bb02f4`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-rounds-2-to-5.jsonl`:
  134237 bytes,
  SHA-256 `0fbbeae1dca534c377d1b4968645bd72cb507118bedfe4eed48f115c0c6a67c5`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-summary.jsonl`:
  100638 bytes,
  SHA-256 `1ec1fe54cc2a412eb44c8c6d65a357df13239e8ddd72bf6aa35dd8e79178bb83`.
- `two-remaining-copies-recheck-2026-11-13/keep/timing-summary.txt`:
  48105 bytes,
  SHA-256 `c03f416e9f8aa1e464302142cbdb63ec7ec400f4f84f3dc2982e689fe0634aea`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-the-final-multiple-arm-second-run.jsonl`:
  3709 bytes,
  SHA-256 `1ed3ea0a72be7dac3f84cf8c83600ec0bacfbef065b7b6e73804539e7029987b`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-the-final-multiple-arm-third-run-and-concat.jsonl`:
  5050 bytes,
  SHA-256 `3e3ed582541d2af58cc542b784faaaa37490266c5e846402d7305f4719edaaef`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-the-final-multiple-arm-with-the-freeze-refusal.jsonl`:
  4335 bytes,
  SHA-256 `75a9474b1beb0fa437d83030ff97893cdae231f5986eab34069efc02a89a8c1a`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-the-final-multiple-arm.jsonl`:
  4078 bytes,
  SHA-256 `85eb5f4863205e42fc77a4a8b66d3dc592647e1d7c73402e7143621dd17eea2b`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-the-kept-patch-and-with-its-threshold-at-two.jsonl`:
  6109 bytes,
  SHA-256 `0fafcd4a86f7a2216681cb097451ea1619b925881838efddb0c9356b410caa8a`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-the-kept-patch-with-its-length-descriptor-fixed.jsonl`:
  6481 bytes,
  SHA-256 `436517971ebc0ee753859d98e0e414acf1e2e2710e8b3176c04523216ab9a8a4`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-on-three-runtimes.jsonl`:
  6504 bytes,
  SHA-256 `5acc7ecbd228171831fd4891575a4f07adeb2a95bfbf832ad7aa89164a3363c1`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-core-suite-with-views-from-two-items.jsonl`:
  927 bytes,
  SHA-256 `59d3df5e71b0c0c9caed739fa18f057feea967a5e2934a5fb01f943b21455b8c`.
- `two-remaining-copies-recheck-2026-11-13/keep/upstream-state.jsonl`:
  3814 bytes,
  SHA-256 `2f4464673f0f64219158bc757d9ec18e37dc16775eb9a6e290e52cd798f58635`.
- `two-remaining-copies-recheck-2026-11-13/keep/verify-new-link.jsonl`:
  335 bytes,
  SHA-256 `d5e771698739e164092207eea9de0293ead003eea94fc1e6d8efed57d2395e99`.
- `two-remaining-copies-recheck-2026-11-13/keep/view-facts.jsonl`:
  14303 bytes,
  SHA-256 `3856165a49973993d09db440bd568c3422e72b541bcdd828d7fc1e461f905115`.
- `two-remaining-copies-recheck-2026-11-13/keep/write-aftermath.jsonl`:
  55496 bytes,
  SHA-256 `74424850c72ac52c5abb33b5024b5a3eb3e98ff6f57392ac600b3e34edf7e1f6`.
- `two-remaining-copies-recheck-2026-11-13/make-final-multiple-arm.ts`:
  21322 bytes,
  SHA-256 `c9ebbe678796c69ec7b11f76a0d6ef326b0ebce8aa093d473c76576e05c13f65`.
- `two-remaining-copies-recheck-2026-11-13/probe/behaviour-probe.ts`:
  11097 bytes,
  SHA-256 `3b7623f2647ccd248bc369fcb135071979c114d2f619391cc53430d907e9e469`.
- `two-remaining-copies-recheck-2026-11-13/probe/compare-builds-in-container.ts`:
  7422 bytes,
  SHA-256 `b9465f4f60ea0eb7a9d740ed7b44f26b714056e61e52cc5b2ba315bbd1f9603d`.
- `two-remaining-copies-recheck-2026-11-13/probe/freeze-aftermath.ts`:
  2676 bytes,
  SHA-256 `375574eb9abb00beffdcf5e8e6d083f20c032dc29fa39b1cc8c1ed98929647d3`.
- `two-remaining-copies-recheck-2026-11-13/probe/long-input-probe.ts`:
  13013 bytes,
  SHA-256 `63682fd77f306569ffe322300b3d560d2bc2a2652cea493a2758a21bb541355e`.
- `two-remaining-copies-recheck-2026-11-13/probe/write-aftermath.ts`:
  3317 bytes,
  SHA-256 `6ea46d30845ea000fd14c3799cb02bd9f028f640c6400d797a873b49cb2ebb91`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/changelog/consumed-argument-buffer.md`:
  228 bytes,
  SHA-256 `f1b5585509579f468c175d5388e2362c9035a4e8bc63dd4eeab146b20cf671b3`;
  in the tree 232 bytes,
  SHA-256 `0178a34067d81e09d694fe084845ac469db8dc5c053200d746105542d8908b6b`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/changelog/multiple-item-state-sharing.md`:
  259 bytes,
  SHA-256 `45f976f257fbe6af33fcea0eff9d4061b3e368fd3a1028a8e372dab64bb71044`;
  in the tree 263 bytes,
  SHA-256 `c4ae00765e0a1ec67502c10b20cf060e206c7188ba95373c8a33dc4c70d9db48`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/commit-messages/01-array-view.txt`:
  1281 bytes,
  SHA-256 `b5c93d6a665cee05ac886df0402c4a39f43aa764650285264ad044d92ec244c5`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/commit-messages/02-multiple-state.txt`:
  1632 bytes,
  SHA-256 `0213f38fddcbcd3695cd5cd6fa3151b9ffcdee7dd8d0f65dc5163b4fe52632a8`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/commit-messages/03-input-buffer.txt`:
  1826 bytes,
  SHA-256 `62d4dff23c1c04b33d780845beb3f9d31406de2a395171a55a9db2bceb8c2da2`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/diffs/01-array-view.patch`:
  10932 bytes,
  SHA-256 `c3cada50158e159120ac57931ef792a42e2c612051ae782258f0bf1902550c4e`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/diffs/02-multiple-state.patch`:
  35309 bytes,
  SHA-256 `634a9f94d965592ca0c900e4795645bcf43a64e980bd80f098f1cdb67ad1b76c`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/diffs/03-input-buffer.patch`:
  15795 bytes,
  SHA-256 `9366a81cb40d2fa646da916efbf2848e86b6ee985a562e5607572c5440202c4d`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/diffs/04-both-combined.patch`:
  62036 bytes,
  SHA-256 `0a0420fd24696d1580d6a0f9964e241ba5b47cc2d3efeb53dfce21c423c23a52`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/measurements/matrix-summary.jsonl`:
  5966 bytes,
  SHA-256 `37241edc351810c97a019c66ff0ace4646387ac5d1c69fd45cbc3e57dd400564`.
- `two-remaining-copies-recheck-2026-11-13/prototype-agent/report.md`:
  29918 bytes,
  SHA-256 `ae27c78bba644eaec450688d4cfe18a1e538bedd891e9490307f9f67c4ef70f0`;
  in the tree 31526 bytes,
  SHA-256 `32fccc2b1d92dd6bbf44b4b15e09bf599c3fac1c775e3088cbf5af751620780e`.
- `two-remaining-copies-recheck-2026-11-13/read-upstream-state.ts`:
  2296 bytes,
  SHA-256 `1db9c526c29d5ef15b617eedc924115675b0bccba767af083e1c9eb090060b14`.
- `two-remaining-copies-recheck-2026-11-13/run-annotated-check.ts`:
  1201 bytes,
  SHA-256 `96e15d50359e0b23467c85d203da8110e661a926b3e05076e5a384ddad15b87d`.
- `two-remaining-copies-recheck-2026-11-13/run-freeze-aftermath.ts`:
  2188 bytes,
  SHA-256 `967abb597d60586d825f0a1e1beaf5b2b3467c41045e9218b6deec6100e26f65`.
- `two-remaining-copies-recheck-2026-11-13/run-in-recheck-image.ts`:
  1247 bytes,
  SHA-256 `bc1a4123cc2a6d62b1542db1addc9c8b9ad42d96fa3ffcfb812b25386cf45193`.
- `two-remaining-copies-recheck-2026-11-13/run-kept-patch-timing-check.ts`:
  1672 bytes,
  SHA-256 `06c62d55278f3d3465788b97a63277c802a9432838834954e759765bf412eba8`.
- `two-remaining-copies-recheck-2026-11-13/run-mixed-threshold-check.ts`:
  1358 bytes,
  SHA-256 `8ce4034b0cea1a69e71dc04c3c206f3580da7680ed064aae31ec5fd1ae998af5`.
- `two-remaining-copies-recheck-2026-11-13/run-node-and-bun-timing-check.ts`:
  2536 bytes,
  SHA-256 `a7baa60212bbaec8f28ba221a8ce1026b97fda29c8dafcffc4b3f869500a3bea`.
- `two-remaining-copies-recheck-2026-11-13/run-probes-on-the-kept-patch.ts`:
  1448 bytes,
  SHA-256 `6c518981a17159983f30c2c6c8b57b7c5b385ddde24a2d85e2efdbf34503881c`.
- `two-remaining-copies-recheck-2026-11-13/run-remaining-timing-rounds.ts`:
  1415 bytes,
  SHA-256 `4cd84634344ca5c10f087a75221ea55bfe53d6b5e7d47e2ed00ecac47c083588`.
- `two-remaining-copies-recheck-2026-11-13/run-repeat-one-bun-test-file.ts`:
  1221 bytes,
  SHA-256 `0f0cb5e18a93e7e5c5d6373b050549ecdf04e043357d8beb1fca286e73ccff30`.
- `two-remaining-copies-recheck-2026-11-13/run-suites.ts`:
  1636 bytes,
  SHA-256 `86a05823a429e987ec907af9030ed149b497c95bb199357e26ed140a696e0894`.
- `two-remaining-copies-recheck-2026-11-13/run-view-facts.ts`:
  1461 bytes,
  SHA-256 `81d5259e9b5db9b6bd12fbb1e66f14e44d0c2e3c49a7b886ac2c433730522e77`.
- `two-remaining-copies-recheck-2026-11-13/splice-timing-section.ts`:
  3845 bytes,
  SHA-256 `af5b362b1d1bb3b09129a8b010a6d2bad9c9463254d73f53639ac659a0f39332`.
- `two-remaining-copies-recheck-2026-11-13/stage-arms-for-recheck.ts`:
  8313 bytes,
  SHA-256 `247ab396f1adf7e19e0cd1c803b840d2bdb188b47a730e8f1096e19a7846f340`.
- `two-remaining-copies-recheck-2026-11-13/suite/build-arm-packages-inside.ts`:
  2005 bytes,
  SHA-256 `e4bd2abb58cdca3f1543272e486e8733de9c0c0aa1e3c3b075f1376e6e302c23`.
- `two-remaining-copies-recheck-2026-11-13/suite/repeat-one-bun-test-file.ts`:
  1871 bytes,
  SHA-256 `13935d2839a24f4b9f8d6f3266dfbf0387a0308fbbb41d2cb21c87c555a40024`.
- `two-remaining-copies-recheck-2026-11-13/suite/run-suites-inside.ts`:
  8939 bytes,
  SHA-256 `3c668dde8e80e1649d861824816a50f2d82038a104de254b7915b4b1931bad2c`.
- `two-remaining-copies-recheck-2026-11-13/summarise-timing.ts`:
  4679 bytes,
  SHA-256 `67b9f8855a6d9b36cabd01b536907575310688dc92758680f848eda42cae0332`.
- `two-remaining-copies-recheck-2026-11-13/verify-new-link.ts`:
  1344 bytes,
  SHA-256 `00e0bb7121ab7d48912b523c91a023810f0ebc63b9ad93578f601eb62839e5a5`.

## Archives that stayed tracked

None.
The four archives the first pass stopped on were settled on 2026-11-13:
two because a record disagreed with the blob it names,
one because its naming file records its own digest,
and one because the guard refused the record it was kept for.
Each left the tree in the second pass,
and the note beside the record that raised
the question says what it found.

The three Java archives the owner allowlisted stay tracked:
`package/intellij-plugin/islands-black/islands-black.jar`,
`package/linter/kotlin/gradle/wrapper/gradle-wrapper.jar` and
`package/music-player/android-app/gradle/wrapper/gradle-wrapper.jar`.
`.gitignore` ignores `*.jar` and re-includes those three beside the pattern.

## What this record does not say

It does not claim the tree holds every byte of a removed archive:
history does,
and the command above each archive reads it.
It does not claim a recorded digest that did not match the tree before this removal now
matches:
those pairs are as they were,
and the archives they concern stayed tracked.
