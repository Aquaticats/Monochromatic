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

## Records kept in the tree for a removed archive

0 of the removed archives are cited for something a reader can
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

## Archives that stayed tracked

Three archives stayed in the tree because removing them would have left a record the
owner has to decide about.
Each is a question,
not an oversight:

- `doc/audit/depot-native-buildkit-job-auth-2026-10-18.sources.json.gz`:
  one evidence file records 18768 bytes and sha256 e5d29d60...,
  the tracked blob is 128260 bytes and 73c1e958...
- `doc/audit/depot-owned-allocation-follow-on-2026-10-18.sources.json.gz`:
  one evidence file records 387034 bytes and sha256 a7a144cc...,
  the tracked blob is 392421 bytes and 87265177...
- `doc/audit/depot-logger-host-development-2026-10-17.evidence.json.gz`:
  its naming file,
  doc/audit/depot-final-recovery-2026-10-17.evidence.json,
  records its own bytes and sha256,
  so editing that file can never leave a matching digest

The three Java archives the owner allowlisted stay tracked as well:
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
and the two archives they concern stayed tracked.
