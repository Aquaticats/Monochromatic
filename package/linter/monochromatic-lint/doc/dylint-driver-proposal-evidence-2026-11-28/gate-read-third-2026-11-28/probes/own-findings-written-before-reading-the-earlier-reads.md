# My own findings, written before I opened the first read, the second read or the report

Written between the clock reads 2026-11-28T07:24:33-05:00 and 2026-11-28T07:27:14-05:00
(the time first written here was an estimate, corrected after the clock was read; the file carries a second section, below).
Same-vendor second pass (Claude Sonnet); unapproved and non-final.
This file exists so the order the brief demands (own findings first) is on disk.
It is a working list; `review.md` is the deliverable and may refine or drop items here.

## Read so far

- The brief, whole.
- The proposal whole, at `e63bf58c19` (copied to scratch with `git show`).
- The evidence folder at that commit (extracted with `git archive`): M7, M8, M10, M11 runs and scripts, the M2 and M5 logs that carry the `E0603` and `E0463` errors, `machine/machine-state.txt` not yet.
- `package/linter/monochromatic-lint-closure/{Cargo.toml,.cargo/config.toml,README.md}`, `toolchain.plan.jsonc` rust row, `package/mpm-shadow/cli/src/toolchain/install_rust_android.rs`.

## Candidate findings (to verify before they go in the review)

1. Answer I says the plan's `rust` row "already carries that list, so `toolchain:install` puts them back".
   The installer (`install_rust_android.rs`) runs `rustup toolchain install --no-self-update <channel>` and
   `rustup component add <c> --toolchain <channel>`, with `channel` the row's `packageId`, which is `{version}`, the dated name.
   The row's own comment says "Floating nightly channel, resolved to the dated toolchain string".
   The host's state (floating `nightly` without components, `nightly-2026-11-16` with them) is what that code produces.
   So the row never puts components on the floating channel, and "What changes for good: nothing in the repository" for I is not shown.
2. M8's compiler is a stale copy: the floating folder was deleted and the dated 2026-11-17 install copied over it.
   The only kept run that added components to a real floating channel (M2b) ran on `c82ce1563` (2026-11-27) and ended in `E0603`.
   The Decision 1 criterion is "the state a fresh machine and a runner have"; a fresh machine today resolves a nightly at or after `c82ce1563`.
   So the evidence for I's delivery is a state a fresh machine cannot reach, and the evidence for the state it can reach is a failure.
3. Under Mechanism T (the proposal's own first-ranked mechanism) the variable is set, so "I needs neither the file nor the variable" is not a difference.
   The ranking text also says "Nothing in one ranking is a reason against an answer in another" while Decision 1's criterion is stated as "no exported variable", which is Decision 2's subject.
4. The 91 s against 123 s is one run of each, on different compilers, in different containers; it is a cost, not a delivery, and no spread was measured.
5. Answer I does not say what the result is when the library fails to compile (status 2 for every Rust package, not a skip) or who meets it.
6. M7 cites `log/dated-components.txt` for "the two Android targets"; that log lists six components and no Android target.
7. M11's folder is named "does-not-install-an-absent-toolchain" but its log shows rustup installed the toolchain; the folder has no `log/`; M11 has no section under "What this round measured";
   the log never lists which components were installed ("downloading 8 components" only); the `rustc --version` arm ran after `rustup show` had already installed the toolchain, so it cannot tell whether `rustc` alone installs.
8. M10 prints no linter digest although the proposal says "printed in each `run.txt`"; M10 arm B's skip names the floating driver directory (the original library), not the dated one the arm's label suggests.
9. Stale labels: "Design S" (line 2175) for SH; "Answers F and P" (line 2275); options M and E and J, D1, D2, D3 appear in the combinations and question text with no description of M or E anywhere in the file.
10. The questions begin past line 1890 of 2521; the file opens with a purpose paragraph and a revision history, not with what is asked and recommended.
11. Answer C says "the measured traps a fresh image meets are ... a writable temporary `TMPDIR`" while the L5 reply in the same file says the cause of M1's refusal "was not established".
12. Option F under I ("makes the first run offline-sufficient") is stated as a result; no kept run is an offline driver build with a pre-filled registry under I; it rests on 0 online `git` executions and an older record.
13. Z over A rests on "the paused installer"; Decision 7 says the pause is the session's reading of a comment, not the owner's words, and asks what it covers.

## Added between the clock reads 2026-11-28T07:29:18-05:00 and 2026-11-28T07:32:29-05:00 (estimate corrected), after the doc fetches and probes, still before opening the three earlier files

Confirmed by source and by fetched docs (each has a kept probe or a quoted line in review.md):

- Item 1 is confirmed.
  `model.rs` `expand_version` replaces `{version}` in `packageId` and in every `env` value;
  `rustup_channel()` returns `packageId`;
  `install_rust_android.rs` runs `rustup toolchain install --no-self-update <channel>` and `rustup component add <c> --toolchain <channel>` with that channel.
  The lock holds `"version":"nightly-2026-11-16"`, `"packageId":"nightly-2026-11-16"`, `env.RUSTUP_TOOLCHAIN` the same.
  So the plan row installs a dated toolchain with the components and never touches the floating `nightly` channel.
  The cleaner (`lean_rustup_toolchains.rs`) says "The repository installs a dated nightly per `toolchain:update`" and "channel names are never touched".
  Answer I as written ("the plan row already carries that list, so `toolchain:install` puts them back", "nothing in the repository", "a channel that moves daily") does not describe this repository.
- Item 3 is confirmed.
  Today `https://static.rust-lang.org/dist/channel-rust-nightly.toml` is the 2026-11-28 nightly with rustc `c82ce1563 2026-11-27`, the compiler on which M2b and M5 got `E0603`.
  The 2026-11-17 nightly is `5df8c02af` (M8 compiler), the 2026-11-16 nightly is `5f4effd57` (lock).
  Dylint has no release after 6.4.0 (2026-11-10).
  Upstream PR 2203, merged 2026-11-24, "keeps the driver and linting utility compatible with nightly toolchains that moved ErrorOutputType out of rustc_session::config", is on main (51 commits past v6.4.0) and unreleased.
- Item 5 is confirmed by source: a failed `cargo dylint` run becomes a `processing_failure` on the package `Cargo.toml`, "The result is incomplete, so the run cannot be trusted".
- The daily-update record (`doc/planning/toolchain-always-latest.md`) already says how a package a nightly breaks is noticed (before and after pass, newly red gets an issue, "failed then failed is already red and is never reported again", and it names the closure library as the known "Wait for Dylint" red, E0797 and E0277 with 6.3.2).
  The proposal cites none of that for Answer I.
- rustup: 1.28.0 removed auto-install, 1.28.1 restored it by default; issue 4836 (milestone 1.30.0) says `rustup show` will no longer auto-install from 1.30;
  the supported way to install what a toolchain file names is `rustup toolchain install` with no arguments (1.28.0 changelog).
  M11 measured only `rustup show active-toolchain` on 1.29.3.
- Issue 6316 comment: it says "the project answered" the quoted sentence; the proposal says "the coordinating session wrote" it.
  All comments on that issue carry the one account `Aquaticat`.
- `.cache/toolchain-bin` links were rewritten at 2026-11-28 07:22:34 -05:00, so the installer (or whatever writes the links) ran in the main checkout today; Decision 3 ranks Z over A on "the paused installer".
- Dylint "How Dylint works", Limitations: Dylint builds the linted package with the toolchain that built the library, so under I every linted package must compile on the floating nightly, not the lock nightly. The proposal states only the library side.
- Label collision: finding names M1 to M8 in the reads section collide with measurement names M1 to M11.
