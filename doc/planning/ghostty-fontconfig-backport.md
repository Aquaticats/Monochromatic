# Backport the Ghostty font-language fix while Terra 1.4.0 is blocked

## Problem

The installed Ghostty `1.3.1-3.fc45` crashes when looking up `U+23BF` (`⎿`).
[The diagnosis](../troubleshooting/ghostty-fontconfig-language-type.md)
establishes a string passed as a Fontconfig language-set pointer.
The upstream fix is in Ghostty 1.4.0,
but [Terra's Fedora 45 publication path is blocked](../troubleshooting/ghostty-terra-package-version-lag.md).

The user said:
"Yes but it looks like Ghostty 1.4 isn't coming.
At least here."
This proposal addresses that availability constraint.
It is not authorization to replace the installed terminal.

## Proposed repair

Backport upstream PR `14853`,
merge commit `8112f7676007b7823798f8a1259ab274058e8bf6`,
onto Ghostty `v1.3.1`,
commit `332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28`.
The upstream patch changes `src/font/DeferredFace.zig` and `src/font/discovery.zig`.
It preserves the tagged Fontconfig value and branches between language sets and strings.
It does not change the compiler requirement.

On 2026-11-01,
the exact `gh pr diff 14853 --repo ghostty-org/ghostty` output passed
`git apply --check` against the clean local `v1.3.1` clone.
This confirms textual applicability only.
No third-party source was edited.
The repository lock already contains Zig `0.15.2`,
which matches `v1.3.1`'s declared compiler requirement.

## Validation required

- Build baseline and patched copies in disposable,
  isolated environments using the locked compiler and system Fontconfig linkage.
- Establish that the baseline reproduces the installed package's crash.
- Require the patched `A`,
  `⎿`,
  spinner,
  emoji,
  and CJK lookups to resolve with correct presentation.
- Exercise a real Ghostty renderer and the original Claude startup in an isolated process.
- Check font configuration and style selection,
  since attempted explicit-font workarounds separately returned `FontconfigNoMatch`.
- Keep installation and launcher changes separate from experimental builds.

## Tradeoffs and limits

This route avoids waiting for Terra's Zig 0.17 package work.
It also creates a locally maintained terminal build until an upstream package replaces it.
Patch applicability does not establish successful compilation,
runtime correctness,
or compatibility with Fedora 45's GTK and font libraries.
The installed terminal and its settings are unchanged.

The temporary Fontconfig override was rejected:
it avoided the segfault but failed to resolve `⎿` in the default presentation.
No working configuration-only repair has been established.
