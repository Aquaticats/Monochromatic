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

## Records kept in the tree for a removed archive

1 of the removed archives are cited for something a reader can
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

### `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence/`

Every member the five documents cite by folder or by name,
byte for byte except six markdown members the repository linter re-wrapped.

Cut from `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip` at commit `a6783a44ae7bdd43f9cd6995fcb6416eea1250d1`.
The archive is 631659 bytes,
SHA-256 `0f2a70bd4504b1c8f58cb5234feb5354e5fa490fa7933019106195517d892352`.

Kept members,
at their archive-relative paths under that folder.
A member the linter re-wrapped carries the digest it had in the archive first
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
  in the tree 33486 bytes,
  SHA-256 `282823b0bbd7c3449f1835e8f20acb345647c5c18a719a02692bf8910adcd25a`.
- `results/report-tables.txt`:
  31570 bytes,
  SHA-256 `5da58e4a90dfee289578a1c32c9ecc5b8d5b6287f8d663143619bb43779ba8e6`.
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
