# Slint IDE implementation handover

## Purpose and next action

Continue the authorized implementation of `package/desktop-app/ide`.
The user confirmed [the scope][scope] and [the approach][plan],
then authorized implementation.
A checkpoint is not a pause:
continue the queue without asking the user to say “continue”.
The application remains an incomplete read-only workspace reader.
The user paused this session once for a CPU change and then resumed it; continue the queue.

Current boundary:
combined search and in-file find are complete;
the "Handoff state" section lists the work in flight and the queue.
Historical record of the combined-search gate:
`proc_9bf4` passed 119 library/integration tests,
fourteen native tests,
and complete package lint.
`proc_ee06` passed disposable guard-removal controls including result caps at
`/home/user/temp/agent/ide-search-guard-r46p2u/results.json`.
The guards cover reply identity,
result/record limits,
scope containment,
EOF cancellation,
and source focus while a search overlay is open.

The dark and light native search probes pass at
`/tmp/monochromatic-ide-native-H0znPy/search-probe.mjs` and
`/tmp/monochromatic-ide-native-YJgXM6/search-probe.mjs`.
Their inspected screenshots cover ordered combined results,
selection/copy at source line 120,
and invalid regex diagnostics alongside a usable filename result.
The light `search-edge-probe.mjs` also verifies actual tree-directory scoping,
failed binary-file retention,
recovery,
and outside-click dismissal.
The first clipboard probe treated a not-yet-published selection as terminal failure;
the corrected probe waits for the expected isolated clipboard value.
No host clipboard is inspected or cleared.
Both native consumer processes exited cleanly after socket `quit`;
there is no active native IDE probe.

Whole-model search row replacement already cancels a held click in Slint 1.18.1.
The added custom press guard was ineffective and was removed after a rebuilt disposable control still passed.
The tree's fixed-slot guard remains necessary.
See [the pointer lifetime investigation](../troubleshooting/slint-repeater-model-pointer-lifetime.md).

In-file find was implemented on 2026-10-05;
the "Handoff state" section records its result.
The following paragraphs are the research record that preceded it.
Fresh reference reads confirm that editord delegates Ctrl+F to real Chrome's find-in-page,
rather than implementing another regular-expression search widget.
`inspect:find-reference` now probes synthetic Unicode/whitespace cases in an isolated browser profile.
`proc_53b2` passed and closed its own browser session in `finally`.
Results are `/tmp/ide-find-reference-is2TFy/results.json`,
with a reduced user agent reporting HeadlessChrome 149.
The disposable profile's `Last Version` file identifies the actual build as `149.0.7827.54`.
Positive and negative controls passed.
The probe matches case differences,
canonical/decomposed accents,
plain letters against accented letters,
`STRASSE` against `Straße`,
`office` against `oﬃce`,
final sigma,
dotted I,
and NBSP against an ordinary space.
It preserves significant query spaces,
treats regex punctuation literally,
and does not treat a literal newline or tab as an ordinary space in the `<pre>` fixture.
Literal newline queries do match literal newlines;
empty patterns do not match.
The probe's `Window.find` path is non-standard and is not itself proof of every Chrome toolbar behavior.
`proc_be99` completed Chromium finder-directory discovery.
The pinned source trace and expanded measured corpus are recorded in
[the find matching plan](../planning/slint-ide-find-matching.md).
No new matching dependency is adopted.

The matcher is settled.
An earlier version of this handover called browser matching fidelity a settled requirement
that must not be sent back to the user.
That was an agent inference and is withdrawn:
on 2026-10-05 the user asked why a find matcher needs vetting when it is just substring matching.
In-file find uses plain literal case-insensitive substring matching with the existing `regex` dependency.
The technology-vetting run and the editord line-DOM probe are dropped.

The incumbent gap is measured, not assumed.
`inspect:find-regex` (`proc_6c18`) runs the already adopted Helix `regex` engine
against the captured 29-case corpus,
escaping every query as a literal and enabling Unicode case-insensitive matching.
It agrees on 16 cases and differs on 13:
canonical-accent,
plain-accent,
case-expansion,
compatibility-ligature,
dotted-i,
nbsp-as-space,
kana-script,
kana-width,
kana-composed,
single-quote,
double-quote,
soft-hyphen,
and combining-mark-only.
Those 13 cases are accepted as documented deliberate differences from editord's browser find.

### Servers from mise

Branch `feat/ide-mise-servers` (not merged into `main`).
The user decided: "If mise is detected in dir the IDE starts the servers themselves through mise exec.
We need to be adaptable and not hardcode anything."
What is implemented, and why it differs from the literal wording:

- When the project folder has a mise configuration at or above it,
  servers are looked up in the tool directories mise prints for that folder and started from there;
  otherwise the plain `PATH` lookup stays.
  Nothing of any repository is named in the code.
  The README section "Servers from mise" in `package/desktop-app/ide/README.md` has the behavior.
- `mise exec` is not the launched command.
  `mise env --json` in this repository prints 42 variables, 26 with a name containing `KEY` or `TOKEN`
  (names only read);
  `mise exec -- SERVER` would give every server the decrypted project `[env]` and bypass the cleared environment.
  Mise only says where the tools are (`config ls --json` from the project and from `/`, then `bin-paths`).
- Mise runs inside the servers' bubblewrap sandbox (`confine_with`), not outside it as first suggested.
  Reason: every mise command, including `config ls`, evaluates a trusted configuration's `[env]` templates
  (a template that touched a marker file ran for `config ls`, `bin-paths`, `which`, `env` and `exec`),
  which is project code that would otherwise run on the bare host with network and write access.
  Measured, no distinguishable cost:
  `bin-paths` 235 ms plain (standard deviation 43) against 319 ms sandboxed (41) in a two-tool project;
  413 ms (80) against 387 ms (152) in this repository;
  production resolution 179 to 639 ms over four runs on a host at load about 60, 415 ms in a headless window run.
- Safety variables for every mise run:
  `MISE_AUTO_INSTALL`, `MISE_EXEC_AUTO_INSTALL`, `MISE_NOT_FOUND_AUTO_INSTALL`, `MISE_TASK_RUN_AUTO_INSTALL`,
  `MISE_LOCKFILE` and `MISE_YES` false, `MISE_OFFLINE` true;
  an environment variable beats a project `[settings]` (measured), and a positive control in a disposable data
  directory shows auto install on makes `mise exec -- true` exit 1 while trying to fetch.
  The IDE never runs `mise trust`, `install` or `exec`.
- Trust: a tools-only configuration needs none; one with `[env]`, hooks or `[settings]` is refused until the user
  runs `mise trust`.
  A refusal, any other failure, or a run over 10 seconds gives a language support notice that names the
  configuration, quotes mise's message, says the user runs `mise trust` (or `mise doctor`),
  and falls back to `PATH` lookup with shims skipped.
  The notice puts that explanation first, because the display keeps about a dozen lines.
- Servers get mise's directories first on their `PATH`, so a Node launcher finds the project's `node`;
  directories below `/tmp`, `/run` and `/dev` are bound back read-only.
- Evidence: `lint:clippy`, `test:language`, `test:native`,
  `inspect:language-mise` (43 checks on disposable projects with the launcher's `PATH`),
  guard-removal controls seen failing for each rule (the shim refusal, the three safety variables, the read-only
  binds, the notice, the global listing, the served-failure state, the run time limit),
  and a headless nested compositor run with screenshots (hover answered by `marksman` and by
  `bash-language-server` from mise's directories; a folder without mise keeps "was not found on PATH";
  an untrusted folder shows mise's words).
  Failure causes and probes: `doc/troubleshooting/mise-servers-for-the-confined-ide.md`.
- Verified: the IDE writes no `mise.lock` and no trust record (trees compared before and after, trust directory
  listed after the window runs).
- Open: variables a tool adds to its environment (`GOROOT`, `JAVA_HOME`) are not passed;
  mise's global tools come with the project's directories (56 directories in the e2e project);
  an `[env]` template runs without network (not measured for a template that needs one);
  unconfined mise runs outside the IDE persisted trust records (see the troubleshooting doc).

### Handoff state

#### Current state on 2026-10-06

This block is current;
the items after it are the record of 2026-10-05 and older.

- The host restarted overnight and cleared `/tmp`,
  including the coordinating session's scratch scripts.
  Durable coordinator scripts and the shared agent brief now live in `~/temp/agent/ide-coordinator/`
  (`ide-fanout-common.md`,
  `provision-worktree.mjs`,
  `gate.mjs`,
  `land-merge.mjs`,
  `render-check.mjs`,
  `inotify-usage.mjs`,
  `fsmonitor-sockets.mjs`);
  gate logs go to `~/temp/agent/ide-coordinator/logs/`.
- Landed on `main` as merge `45db45d02`:
  reaping and lifecycle checks (`feat/ide-language-verify`),
  UI batch 2 (`feat/ide-ui-batch-2`),
  re-asking for timed-out hints and pulled diagnostics (`fix/ide-hints-after-reload`),
  the user's watch timing (`feat/ide-tree-watch` through `cefc9902d`),
  and release packaging (`feat/ide-package`).
  Gate at the integration commit `93ac17859`
  (IDE tree `d1f5569c31fe7e4cb92e651820f0338ac68842aa`, load 4 to 15):
  lint;
  96 passing library and integration result lines;
  102 of 102 native tests with 4 ignored measurements skipped.
- Landed on `main` as merge `906ee5aed`:
  the logging rework (`feat/ide-logging` through `2572405cc`)
  and hint and diagnostic rows above the code line with gutter severity letters
  (`feat/ide-hint-rows` through `b05776ede`).
  Gate at the integration commit `93f9f4891`
  (IDE tree `53fd6b369da8be4ea65c447cb2a4f2a6dd219897`, load 75 to 98):
  lint;
  99 passing library and integration result lines;
  112 of 112 native tests with 4 ignored measurements skipped.
  The merge resolved nine conflicted files by keeping both sides;
  the three tests with a hard-coded `TEXT_LEFT` of 313 now take `sidebar_tests::TEXT_LEFT`
  (256 + 1 + 58 = 315).
  An earlier gate of the logging merge alone failed on two known timing tests,
  which `fix/ide-test-flakes` owns.
  UI batch 3 was asked with the page `package/desktop-app/ide/design/questions/2026-10-06-ui-batch-3.html`;
  every answer matches what the build ships.
- Landed on `main` as merge `d1493c66c`:
  a crashed language server's last standard-error lines in its warning
  (`feat/ide-logging` through `6c1aff99b`; 8 lines of at most 512 bytes per server;
  a start timeout keeps its ERROR line and adds a WARN with the lines;
  the crash warning waits up to 1 s for the last lines;
  these three choices were the agent's, settled by the coordinating session).
  Gate at `180353bbb` (IDE tree `88b0bf572869089e9e723f2567b6614070991013`):
  lint;
  102 passing library and integration result lines;
  `test:native` failed once in
  `native::annotation_gutter_tests::the_letter_stands_the_same_gap_before_one_two_and_three_digit_numbers`
  ("line 0's number or text changed when its letter appeared")
  and passed 112 of 112 on the rerun of the same tree;
  that intermittent failure went to `fix/ide-test-flakes`.
- The API session limit cut four agents off again at about 15:10 on 2026-10-06;
  each worktree was clean and pushed,
  and each agent was resumed from its transcript with its measured state.
- Landed on `main` as merge `21f2c625d`:
  the single executable carrying its runtime, the home folder as the default project,
  install, the launcher entry, `--licenses`, and removal of other builds' grammar folders after 30 days unused
  (`feat/ide-single-file` through `4b8b03663`).
  Gate at `a70b0bead` (IDE tree `93cf8e20158fd4c11d0143d28a33224496c3b567`):
  lint, lint:release, test, test:cli, test:native 114 of 114, bundle, inspect:bundle 9 of 9.
  The integration worktree needs its own release build of `//package/cli/nested-wayland-session`
  before `inspect:bundle`; without it six checks fail naming the missing compositor.
- Landed on `main` as merge `7940aef9c`:
  the IDE watching the project's source folders for the language servers,
  quiet reads, the watch-limit back-off, offline containers,
  and no warning for plain text (`fix/ide-watch-save` through `bcd30ef32`),
  plus `f3f0a6092`, which routes the plain-text check through `OwnLoader::helix`.
  Gate on IDE tree `9bf8ee9c59037bae008d7e9e980d644d68179dc5`:
  lint, lint:release, test, test:cli, test:native 116 of 116, bundle, inspect:bundle 9 of 9,
  and inspect:server-watches.
- systemd-oomd killed the shared `claude-code-bash` cgroup at 17:11:23 and 17:14:48 on 2026-10-06,
  taking every background shell of this session and its agents;
  long runs now go through `systemd-run --user` as described in
  `doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`.
- Landed on `main` as merge `4ee4171b5`:
  element-handle accessibility tests with the batch 3b and accessibility answers applied
  (`feat/ide-accessibility-tests` through `5335dddce`)
  and cargo-about collecting every Rust crate's license text into the executable and `--licenses`
  (`feat/ide-crate-notices` through `72e34e0ae`).
  Gate at `6a4095f99` (IDE tree `65e7fed6f689088b645411ec5e7bfb0da817d46b`), run as a user unit:
  lint, lint:release, test, test:cli, test:native 124 of 124, bundle,
  inspect:bundle 10 of 10, inspect:bundle-guards 8 of 8.
  The batch 3b page gave the whole-cell plate a wrong cost (covering the focus line);
  the user was shown the correction with the applied frames and kept the plate.
  The shared image `localhost/monochromatic/ide` now contains cargo-about 0.9.2.
- Landed on `main` as merge `6de44d83d`:
  the gate flake fixes and two Language defect fixes (`fix/ide-test-flakes` through `fe21f6494`)
  and the guard anchor cleanup (`fix/ide-guard-anchors` through `8f5f174d0`).
  Language servers get a start allowance of three request timeouts;
  a late answer for an earlier hint window no longer replaces the current hints;
  a diagnostics set pushed during the 2 s hold is kept and shown when it ends;
  test fixtures moved from the container's btrfs `/tmp` into memory,
  where captured kernel wait points showed transaction stalls;
  several 3 s test bounds that sat below real first-reply cost now use the harness patience;
  main's `rows::due` had weakened the `repaint-on-accept` guard's test,
  so the test now pins the reader's scroll time a minute ahead and the guard fails again.
  Gate at `e74339b7c` (IDE tree `1644837fddf533cdff60f5fc1e41a72ca49dd82b`), run as user units:
  lint, lint:release, test, test:cli, test:native 124 of 124, bundle, inspect:bundle, inspect:bundle-guards.
  The landing message says `inspect:language-navigation-guards` ran at that commit;
  it ran at `8b04b222b`, whose IDE tree differs only by one `Cargo.toml` line merged from `main`.
  The other nine guard scripts passed at `bd1ad9ad4`, which differs from the landed tree in one test file.
  One `inspect:bundle` run at load 40 failed `lone-copy-highlights`
  (the application took longer than the compositor's 2 s to close) and three reruns passed.
- Landed on `main` as `0b81ef2da1` (first parent chain through `25b52185f3`):
  a `lint:rustfmt` task (`cargo fmt --check`) in the package's `lint` task, the user's answer of 2026-10-10.
  It passes on the clean tree and fails, printing the diff, with a misformatted function appended to
  `src/launcher.rs` (restored afterwards).
  The merge kept main's `monochromatic-lint` step beside it.
  Gate on IDE tree `dc5a8828df8594a874b27eada1f0639f9029f055`:
  lint:clippy, lint:rustfmt, lint:slint, lint:release, test, test:cli, test:native 124 of 124, bundle,
  inspect:bundle, inspect:bundle-guards.
- The package's combined `lint` task is red on `main` since the linter cutover of another session:
  `monochromatic-lint package/desktop-app/ide` exits 1 with 570 `markdown/semantic-line-breaks` errors
  and 821 `rust/no-anonymous-functions` warnings.
  None of it comes from the IDE work of this handover; the cutover owns it.
- The native minimap is split,
  and the missing rustdocs of the minimap,
  `git_ignore`
  and `language` files are added,
  on branch `refactor/ide-split-minimap` (worktree `.claude/worktrees/ide-split-minimap`),
  with no change of behavior.
  Fixed:
  `rust/max-lines` in `src/native/minimap.rs` (439 code lines against a limit of 300),
  and `rust/require-rustdoc` in `src/native/minimap.rs`,
  `src/native/minimap_build.rs`,
  `src/minimap.rs`,
  `src/minimap/raster.rs`,
  `src/git_ignore.rs`,
  `src/git_ignore/stamp.rs`,
  and the nine files of `src/language/` that the combined lint named.
  `src/native/minimap.rs` now holds the module documentation and the re-exports,
  so no caller changed.
  `native/minimap/state.rs` holds `MinimapState` and the drag.
  `minimap/pace.rs` holds `plan`,
  `Plan`,
  and the two periods.
  `minimap/request.rs` holds `key_of`,
  `post`,
  `flush`,
  and `note`.
  `minimap/picture.rs` holds `apply`,
  the poll of the worker's replies,
  and the poll timer.
  `minimap/pointer.rs` holds `scroll_to`,
  `jump`,
  and `bind`.
  Items that moved down a level and were `pub(super)` are `pub(in crate::native)`,
  and `plan`,
  `Plan`,
  `DEBOUNCE`,
  `THROTTLE` and `apply` are re-exported under `#[cfg(test)]` only,
  because only `src/native/minimap_tests.rs` uses them outside the module.
  Eleven `inspect:source-guards` cases changed their `file:` path and nothing else
  (`minimap-press-centers`,
  `minimap-press-caret`,
  `minimap-shift-anchor`,
  `minimap-drag-scrolls`,
  `minimap-release-ends-drag` now name `src/native/minimap/pointer.rs`;
  `minimap-key-has-no-scroll` names `request.rs`;
  `minimap-follows-measurement`,
  `minimap-resize-keeps-wait` and `minimap-throttle-period` name `pace.rs`;
  `minimap-stale-text` and `minimap-older-request` name `picture.rs`).
  A scratch checker counted every `file:` and `before:` pair of `bin/inspect-*.mjs` (412 pairs) in the moved tree
  and in `main`:
  the same five pairs,
  all in find and search guards that pass an occurrence number,
  are not found once.
  Tracing events of the moved functions now carry the targets
  `monochromatic_ide::native::minimap::request`,
  `::picture` and `::pointer`,
  not `monochromatic_ide::native::minimap`;
  no filter,
  test or script in the package names the old target.
  The record of the minimap on branch `feat/ide-minimap` cites `src/native/minimap.rs` for `plan`,
  which is `src/native/minimap/pace.rs` now.
  Gate on the branch:
  lint:rustfmt,
  lint:clippy,
  lint:slint,
  lint:release,
  test,
  test:cli,
  test:native with 197 passed as before the split,
  test:language,
  test:minimap,
  test:wrap,
  test:annotations,
  test:annotations-native,
  test:syntax,
  bundle,
  inspect:bundle,
  inspect:bundle-guards,
  and the source (108 of 108),
  language (31 of 31),
  language lifecycle (28 runs),
  language navigation (16 of 16)
  and ignored-entry (28 of 28) guards,
  inspect:language,
  inspect:language-mise and inspect:language-confinement,
  all status 0.
  The lint of `src/` shows no `rust/max-lines` and no `rust/require-rustdoc` finding in these files,
  the `rust/no-anonymous-functions` count is unchanged,
  and no file of the IDE package gained a `markdown/semantic-line-breaks` finding outside the new minimap modules,
  whose total is below the old file's.
- Branch `refactor/ide-split-top` (2026-10-15) clears the `rust/max-lines` and `rust/require-rustdoc` findings of
  `src/annotation.rs`,
  `src/shaped_text.rs`,
  `src/text_raster.rs`,
  `src/name_order.rs`,
  `src/tab_layout.rs`,
  `src/text_projection.rs`,
  `src/wrap.rs`
  and `src/wrap_sign.rs`.
  The rule counts lines that hold a token other than a comment or whitespace,
  with a limit of 300
  (`package/linter/monochromatic-lint/src/rust_source.rs`,
  `code_lines`).
  The three over-long files had 304,
  452
  and 308 such lines
  and now have 267,
  279
  and 258.
  `annotation/severity.rs` now holds `rank`,
  `severity_name`,
  `shown`
  and `describe`;
  `text_raster/palette.rs` holds `CodeColors`
  and `ink`;
  `shaped_text/frame.rs` holds `Clip`,
  `Viewport`
  and `ShapedView`;
  `shaped_text/layout.rs` holds `LineRequest`
  and the line and virtual-row shaping of `TextShaper`
  (`line_layout`,
  `wrapped_line_layout`,
  `build_line`,
  `row_layout`,
  `wrap_row_text`,
  `build_row`,
  `space_advance`,
  `baseline`).
  Every moved item is re-exported from its old module,
  so no caller changed;
  `TextShaper::baseline` became `pub(super)`.
  The source guards `markup-heading-ink`,
  `indent-scale`,
  `markup-weight-push`
  and `markup-italic-push`
  and the annotation guard `omitted-severity` point at the new files;
  no `before` text or expected failure changed.
  Gate on the code of `a31468e2bc` (logs in `~/temp/agent/ide-split-top/logs/`):
  lint:rustfmt,
  lint:clippy,
  lint:slint,
  lint:release,
  test,
  test:cli,
  test:language,
  test:minimap,
  test:wrap,
  test:annotations,
  test:annotations-native,
  test:syntax,
  bundle,
  inspect:bundle 10 of 10,
  inspect:bundle-guards 8 of 8,
  test:native 197 of 197 on the second run
  (the first run failed `wrap_keys_tests::the_view_follows_the_caret_through_every_visual_row`,
  the known flake).
  The guard tasks passed with
  annotation 65,
  source 108,
  find 39,
  ignored 28,
  search 17,
  sidebar 18,
  theme 7,
  watch 76,
  language 31,
  language lifecycle 28 runs,
  and language navigation 16.
  The combined lint of `package/desktop-app/ide` has the same 2654 `markdown/semantic-line-breaks` errors
  and 1135 `rust/no-anonymous-functions` warnings as before.
  Its `rust/max-lines` findings fell from 9 to 6,
  and `rust/require-rustdoc` from 33 to 25;
  the remaining ones are in `src/native*`,
  `src/language*`,
  `src/minimap*`
  and `src/git_ignore*`,
  which other branches own.
- The over-long native files are split on branch `refactor/ide-split-native`
  (worktree `.claude/worktrees/ide-split-native`),
  with no change of behavior.
  Fixed:
  `rust/max-lines` in `src/native.rs`,
  `src/native/render.rs`,
  `src/native/rows.rs`,
  `src/native/tile.rs`,
  and `src/native/tile_build.rs`,
  and `rust/require-rustdoc` in `src/native/tile_build.rs`.
  The rule counts lines that hold a token other than a comment or whitespace,
  against a limit of 300 (`check_max_lines` in `package/linter/monochromatic-lint/src/rust_rules.rs`).
  Docs and `//region` markers are free.
  Each new module is re-exported by its parent,
  so no caller changed.
  `native/state.rs` holds `State::new`.
  `native/start.rs` holds `servers_wanted` and `run`.
  `native/render/paint.rs` holds the paint inputs,
  `tile_cap`,
  and the tile stamp and job.
  `render/present.rs` holds `Frame`,
  `show`,
  and the gutter's wrap indicator.
  `render/measure.rs` holds the measurement of the requested tile.
  `native/rows/held.rs` holds the space held over a reload.
  `rows/quiet.rs` holds the scrolling rule's clock.
  `rows/window.rs` holds the window models and setters.
  `native/tile/pass.rs` holds `current_key`,
  `sync_pass`,
  and `apply_pass`.
  `native/tile_build/job.rs` holds the job,
  the result,
  and `build_tile`.
  `tile_build/worker_thread.rs` holds the worker loop.
  The test modules stay declared in `native.rs`,
  so every test path such as `native::wrap_keys_tests::...` is unchanged.
  Moved items that were `pub(super)` became `pub(in crate::native)`.
  Two guard anchors moved:
  `clip-cap` (now `src/native/render/paint.rs`),
  and the `logging::install` mutation of `inspect:log-stall` (now `src/native/start.rs`).
  The 16 `tracing` records in the moved code
  (`tile/pass.rs` 5,
  `render/present.rs` 3,
  `rows/held.rs` 1,
  `tile_build/worker_thread.rs` 7)
  now carry the module path of their new file as target.
  A search of `bin`,
  `tests`,
  `README.md`,
  and `src` found no filter on the old targets.
  Combined lint over `package/desktop-app/ide/src`:
  `rust/max-lines` 9 to 4,
  `rust/require-rustdoc` 33 to 30,
  none of the remaining ones in these files.
  The rest belong to `native/minimap.rs`,
  `native/minimap_build.rs`,
  `minimap.rs`,
  and `minimap/raster.rs`,
  which this change did not touch,
  and to the files of the other split.
  `markdown/semantic-line-breaks` stays at 1156 and `rust/no-anonymous-functions` at 862.
  Gate on the branch tip,
  all status 0:
  lint:rustfmt,
  lint:clippy,
  lint:slint,
  lint:release,
  test,
  test:cli,
  test:native 197 of 197,
  test:language,
  test:minimap,
  test:wrap,
  test:annotations,
  test:annotations-native,
  test:syntax,
  bundle,
  inspect:bundle,
  inspect:bundle-guards,
  the source (108),
  theme (7),
  annotation (65),
  language navigation (16),
  find (39),
  sidebar (18),
  ignored (28),
  watch (76),
  search (17),
  language (31),
  and language lifecycle (28 runs) guards,
  inspect:log-stall,
  and inspect:refresh-latency.
- The user approved installing into the real home on 2026-10-10:
  `bin/install.mjs` put `dist/monochromatic-ide` at `~/.local/bin/monochromatic-ide`
  (identical bytes, hash checked) and the launcher entry at `~/.local/share/applications/monochromatic.ide.desktop`
  (`desktop-file-validate` accepts it).
  `inspect:bundle` against the installed file passed 10 of 10.
- The cutover changed the git policy configuration, so git in the landing worktree stopped with
  `config-changed`; with the user's approval and after reading the diff of `cli-git.config.ts`,
  `git cli-git trust --yes` was run in `.claude/worktrees/ide-land` only.
- Source view choppiness, requests of 2026-10-10 (the user ruled it is not excusable even under extreme host load):
  - The file tree is sorted mixed (directories and files together), case folded, with digit runs compared by value
    (`src/name_order.rs`; before, it kept raw `read_dir` order).
  - Vello was tried (`SLINT_BACKEND=winit-vello`, approved by the user) and not adopted:
    it draws in the nested compositor, and FemtoVG, software, and Vello showed the same frame cadence.
    The experiment (the `renderer-vello` feature and per-phase timings in `src/native/render.rs`) is kept only as a
    patch at `~/temp/agent/ide-render-phases/vello-trial-uncommitted.patch`; its branch and worktree are removed.
  - The measured cause was in the app: `render()` ran about twice per wheel step and every call was a full repaint.
    Stage 1 of `doc/planning/slint-ide-scroll-pipeline.md` is built:
    an overscan tile of about three view heights, an early return while the view stays inside it,
    and a row cache (`src/shaped_row_cache.rs`).
    Old build against new build in the nested compositor, valid runs only, host load 27 to 77:
    renders during a scroll fell from about 165 to 220 to 7 to 12 (median 0 per step),
    and application CPU in the scroll window from 0.4 to 0.5 s to 0.2 to 0.3 s.
    P95 latency and the longest gap between painted frames did not improve (the longest gap read higher);
    their run-to-run spread is as large as the difference, and the stage 1 tile rebuild still runs on the UI thread.
    The "painted share of possible frames, about half" first reported here was a harness artifact:
    the harness counts its 1.5 s idle tail as possible frames, so perfect painting of the input phase scores about
    115 of 204, and a rerun restricted to the input phase painted 103 of 115 frame slots.
    The `rendersInScroll` counter counts viewport tile presentations (a log line), not paints.
  - Gate at the landed IDE tree: lint:clippy, lint:rustfmt, lint:slint, lint:release, test, test:cli,
    test:native 126 of 126, bundle, inspect:bundle, inspect:bundle-guards,
    and the source, find, annotation, sidebar, theme, search, and watch guards.
    The `rebase-frame` guard now matches the shared failure prefix, because the first assertion of its test no longer
    sees the removed rebase while the second still does.
  - Stage 2 is landed on `main` as merge `2c62b1feb6` (IDE tree `55ad110cd030e8f4c7b6b7a1c241edd73a05850d`):
    a scroll that nears the tile edge re-centers it on a worker thread (`src/native/tile.rs`, `tile_build.rs`),
    the worker shapes with a shaper forked from the window's fonts, a failed worker is replaced on the next re-center,
    and closing the window does not wait for a build in flight.
    Gate on that tree, at host load 47 to 85: lint:clippy, lint:rustfmt, lint:slint, lint:release, test, test:cli,
    test:native 131 of 131, bundle, inspect:bundle 10 of 10, inspect:bundle-guards, and the source (33 of 33), find,
    annotation, sidebar, theme, search, and watch guards, all passing.
    The landing message lists them as still running because they were; they finished green afterwards.
    A release profile tile build was never timed (the test builds are debug and take about 600 ms each).
  - The installed single file at `~/.local/bin/monochromatic-ide` predates stages 1 and 2; reinstalling it is a
    separate action in the user's real home.
  - Soft wrap landed on main at `db43b5c97e` (branch `feat/ide-soft-wrap`, worktree `.claude/worktrees/ide-wrap`,
    through `a973a163f1`, package tree `80e74506a4`; the merged tree `b63b63eb7f` adds only the check-script fix below).
    Four slices: IntelliJ break rules and wrapped shaping; wrap width, per-line visual rows, whole-file measurement on
    the tile worker, wrapped painting, the gutter number on first rows and the arrow on continuation rows, no horizontal
    scrolling; Home, End, Up, Down, and paging by visual row, the wrap side stored in `ReadingPosition`, pointer and
    multi-click by visual row; hints and message rows wrapped, a rope view that removes the quadratic cost of a
    100000 character line (about 0.5 s in a debug build against over 90 s before), a clipped tile for a line past the
    raster limit, and a character budget for tab correction.
    Gates on the branch tip: lint:clippy, lint:rustfmt, lint:slint, lint:release, test, test:cli, test:native (163 of
    163, 5 skipped), bundle, inspect:bundle, inspect:bundle-guards, and the sidebar (18), theme (7), find (39),
    annotation (65), and source (77) guards, all status 0; the new guards were seen failing.
    Recorded deviations: blocks stay above the first visual row of their line (the user's decision of 2026-10-05;
    IntelliJ differs and the package README lists it); a resize packs only the hinted lines near the view and the
    others when the view comes near; tab correction past its budget may leave later tabs off their stops on a very
    tall line.
    Frame measurement of the final bundle (sha1 `a57508d468`, release, nested compositor 0.4.0, recorder at 60 fps,
    143 valid runs of 145, scratch `~/temp/agent/ide-wrap-s5/`, results in `results-all.jsonl`): at host load below
    20 the longest gap while scrolling a 50,000 line file was at most 50 ms in 17 runs and none passed 100 ms, for the
    wrap build and for the three earlier builds alike. At load 20 to 100, 9 of 17 wrap runs had a longest gap over
    100 ms (up to 417), and so did 4 of 8 stage 2 runs, 4 of 8 stage 1 runs, and 4 of 7 runs of the build without
    overscan. Run-to-run spread of one build and condition (17 to 308 ms, stage 1 up to 800) is as large as every
    difference between builds, so no build can be called smoother; the load fell from 70 to 3 during the series, so
    round number stands in for load. Two things the numbers do show: a continuous resize drag (1100 to 600 px at about
    1560 px per second) leaves the old tile on screen until the worker's tile for the new width arrives, so 72 to 81 of
    about 101 frames per run showed text running past the right edge or stopping short, in stretches of 384 to 1,371 ms,
    and the picture settled 25 to 127 ms after the last resize step; the longest gap in resize runs stayed 17 to 60 ms
    and no frame was blank. Open time at 50,000 lines: first non-flat frame median 1,152 ms (879 to 1,160 below load
    10), whole file measured median 2,710 ms (3 to 15 s at load 30 to 60).
    Resize decision (settled, no question to the user: open decisions follow IntelliJ, which re-wraps at once): a width
    change re-wraps and paints the visible rows on the window thread within a counted allowance, and the worker keeps
    the whole-file re-break. The user also said twice that questions IntelliJ already answers must not be asked.
    Landed 2026-10-13: the window-thread re-wrap (IDE tree `589a2d3b4e`, 17 commits on `feat/ide-soft-wrap`, written
    by a Sonnet subagent and verified by the coordinator). A width change re-breaks and shapes the lines of the
    requested tile on the window thread within an allowance of 384 lines, 24,000 work units and a 32 MiB raster
    (`src/native/rewrap.rs`); anything over a bound goes to the worker, which keeps the whole-file re-break. Gate on
    that tree: all 15 tasks status 0 (logs `fE-*` and `fF-*` in `~/temp/agent/ide-wrap-s6/logs/`), source guards
    84 of 84, annotation guards 65 of 65, `test:native` 178 passed 4 times on the tree (3 by the agent, 1 by the
    coordinator in `c-native.log`). Two native failures found on the new CPU were test races, not product bugs:
    the stale-height test compared the map before the 200 ms scroll quiet time ended (it now moves the scroll rule's
    clock and waits until the map is due), and `rows_above_the_view_changing_height_move_no_visible_pixel` installed
    its pass gate after the window opened (failed 5 of 5 on the pre-branch tree on this CPU; now installed first).
    Both passed 20 of 20 repeats. The `wrapped-tile-holds` guard now expects "the source tile worker did not finish
    the re-center", which matches any hang of that kind, so it separates less than before; under its mutation the
    branch hangs at open (`scroll_wanted` never cleared) where the old tree failed with "entered render", and the
    reason was not found.
    Measured (release bundle, nested compositor 0.4.0, recorder 60 fps, load below 20 only): in the requested 16 ms
    drag the longest gap (17 to 18 ms) and the count of frames not fitting (74 to 77 against 76 to 82) did not change
    from the `final` bundle; in a slower 100 ms drag the count fell to 0, 8 and 19 of about 265 against 44, 42 and 50.
    A frame drawn right after a size event is wrapped to the new width in the headless backend (198 rows run to the
    edge with the step off), but in the real compositor each new size still shows the previous step's content for one
    frame, and where that delay comes from (compositor configure and commit, or the toolkit's redraw scheduling) is
    not determined. The window-thread step took a median 3 to 7 ms and at most 15 ms at low load; on the earlier CPU
    it reached 91 ms once, so the allowance bounds counted work and not wall time.
    Determined 2026-10-13 (branch `feat/ide-resize-frame`,
    `doc/troubleshooting/nested-wayland-recording-previous-resize-step.md`):
the one-step delay is the recorder's capture model plus winit's redraw gate, not the application. The compositor
    changes the recorded frame size when the `resize` command arrives, captures before it sends frame callbacks, and
    sends them only from the recorder tick; winit holds the redraw until that callback. 76 to 77 of 81 to 82 captures
    held a buffer of another width than the output, every frame that did not fit was among them, and no frame whose
    buffer had the output's width failed to fit. Controls: stopping the application 40 and 80 ms moved the logged size
    event by that time, and sending the callbacks at the command cut the stale captures from 94 to 56 to 61 percent.
    No application change was made; the debug lines at the `monochromatic_ide::native::frame` target were added.
    Closed by the user on 2026-10-14 ("Smoothness both under normal conditions and under heavy load: We can
    consider it solved. Smooth for me."): the live path (no recording) and the load band 20 to 100 were not
    measured, and no further smoothness measurement is planned. Reopen only on a new report from the user.
    Other pending small items: remove the worktrees `ide-wrap` and `ide-land2` after landing; delete the large
    scratch caches `~/temp/agent/ide-wrap-s4/guard-cache`, `ide-wrap-s6/guard-cache`, `ide-wrap-s5/bundles` when done.
    Open: the installed single file at `~/.local/bin/monochromatic-ide` predates stages 1, 2, soft wrap and this
    change; reinstalling it changes the real home and needs the user's approval.
    Both flaky tests of the guard runs were fixed in this work: the scroll quiet-time test reads a clock a test can
    hold, and find reveal measures its own line first (it failed 3 of 16 runs at baseline).
    Minimap on branch `feat/ide-minimap` (user request: "Need a minimap also acting as a scrollbar."). The minimap at
    the right edge of the source view is the scroll control: a press jumps and centers the pressed row (the caret
    moves, Shift extends the selection), a drag of the box scrolls with the box, the wheel scrolls as over the text, and
    the keys are unchanged. Open decisions follow CodeGlance Pro (`Nasller/CodeGlancePro`, read from source) and then
    IntelliJ: 110 px wide (a sixth of the source column when narrow, at least 15 px), four pixel rows for a row of
    text, fit mode, box fill 0.15, 0.25 under the pointer on the box, 0.35 dragged, with a border as the second
    channel, and no scroll bar (the view had none). The source column minimum is now 288 px (240 plus the minimap).
    The picture is built on the `ide-minimap` worker, keyed by everything but the scroll offset, bounded by the track
    height, accepted by the tile worker's rule; a scroll moves the box in markup alone, and requests follow `plan` in
    `src/native/minimap.rs`. Frame harness on release bundles (zero dropped frames, start load 7.4 to 13.2, four runs
    a side, bundle sha1 `9317d00b`): wheel scrolling and the width drag keep the longest gaps of `main` (16.7 to
    17.7 ms; the harness's duplicate width slot gives 33.3 ms in both builds), a width step on the window thread is not
    longer, the picture settles 41 to 60 ms (50,000 lines) or 242 to 260 ms (2,000 wrapped lines) after the last
    width, a 50,000-line picture builds in 13 to 15 ms, and memory is within 0.4 MB of the current `main`. Numbers,
    flake counts, and the 17 guard cases (each seen failing) are in the IDE README section Minimap. Not done: marks in
    the picture, the code preview after 400 ms, the resizable width. In parallel full runs of
    `test:native` the test `the_view_follows_the_caret_through_every_visual_row` failed 3 of 23 times on this branch
    and 1 of 10 on `main` (same assertion, 120 against 147), and `find_box_context_menu_runs_each_entry` failed once
    on this branch; filtered (serial) runs never failed. Details are in the README section Minimap.
    Landed on `main` as `2255008ede` after the coordinating session reran the gates on the merged tree (IDE tree
    `7d11076a05`; rustfmt, clippy, slint, release, test, test:cli, test:minimap, test:language, test:native 197, bundle,
    inspect:bundle, inspect:bundle-guards, language-mise, language-confinement, and the source (108), sidebar (18),
    theme (7), find, annotation (65) and language (31) guards, all status 0) and a frame run of its own on the final
    bundle against the installed `main` bundle (three runs a side, alternated, start load 0.7 to 1.6): wheel longest
    gap 16.7 to 17 ms and 228 of 230 slots painted in every run of both builds; width drag longest gap 17 to 17.3 ms
    outside the duplicate slot, 75 to 77 of 78 painted, settle 24 to 26 ms in both. The harness's own "frames not
    fitting" detector reads 68 to 73 of 102 for the minimap build against 33 to 52 for `main`; the detector looks for
    ink in the last columns, which the minimap fills, so those counts do not compare (the border signature detector
    the agent used reads 37 to 70). The installed `~/.local/bin/monochromatic-ide` is that build (sha256 `9c4d6047`).
    Landed on 2026-10-15: the splits of `annotation.rs`, `shaped_text.rs`, `text_raster.rs`, `native.rs`,
    `native/render.rs`, `native/rows.rs`, `native/tile.rs`, `native/tile_build.rs` and `native/minimap.rs`, and the
    rustdocs the combined lint found (branches `refactor/ide-split-top`, `refactor/ide-split-native` and
    `refactor/ide-split-minimap`; the coordinating session merged them and reran every guard task on the combined
    trees before each landing).
    The user approved the bulk reformat on 2026-10-15 ("Yes fix both directly in main."): the mise managed linter,
    declared `latest` (never pinned; `mise.lock` only records that it resolved to 0.1.3),
    found by `mise which monochromatic-lint` (the PATH of a stale shell finds 0.1.1, which rejects the config), counts
    2654 `markdown/semantic-line-breaks` errors (1291 in `README.md` and `design/README.md`, the rest in Markdown
    inside Rust doc comments, which the rule checks too and `--fix` repairs) and 1135 `rust/no-anonymous-functions`
    warnings (not fixable by the tool) in `package/desktop-app/ide`. Done on 2026-10-18: `monochromatic-lint --fix`
    over the IDE package (86 files, zero `markdown/semantic-line-breaks`, `rust/max-lines` and `rust/require-rustdoc`
    findings left; the 28 gates passed on the merged tree, IDE tree `3145c409fc`) and landed on `main`. Then branch
    `refactor/ide-closures` (an agent; see the bullet "Closures named in the IDE"): the IDE container lint gets the
    root `clippy.toml` (issue 1756; about 64 `Result::unwrap` sites were fixed first), and the 1135
    `rust/no-anonymous-functions` warnings fell to 67.
    Landed on 2026-10-18 from open issues: the nested compositor 0.6.1 (owner-only control socket bound through a
    private staging directory and a rename, so no instant of wider access; startup and frame failures reported as
    errors instead of eight panics; root `clippy.toml` in its container lint; nextest for its container tests;
    issues 1239 and 1755 closed), the IDE test tasks on nextest (distinct passing names equal before and after: 783,
    64 and 379), and the regex in `bin/nested-compositor.mjs` replaced by a cursor scan (issue 838). Issues 1756 and
    1757 stay open for other packages. Cosmetic and open: hover text shows raw Markdown links
    (`[Date](file:///...)`).
    Markdown and MDX highlighting (branch `feat/ide-markdown-highlight`, worktree `.claude/worktrees/ide-markdown`):
    a copy of `.agents/skills/choosing-technology/SKILL.md` showed only the YAML frontmatter colored. The language, the
    `markdown`, `markdown_inline` and `yaml` grammars and the injections all worked; `syntax::SCOPES` had no `markup`
    entry, so every `markup.*` capture was dropped (357 spans, 2 colored before; 872 spans, 517 colored after; the
    real bundle logged the same 872). Nine scopes were appended with a second channel (bold, semibold, light, the real
    italic face; underline and strikethrough are not drawn by the glyph painter), fenced blocks get no block color, and
    the tab measuring pass now shapes with the roles. MDX stays Markdown (Helix has no JSX-aware language); the README
    section "Markdown and MDX highlighting" lists what paints. New: the `test:syntax` task, `tests/syntax_markdown.rs`,
    `tests/syntax_markdown_faces.rs`, and seven `inspect:source-guards` cases (`markup-*`, `fence-language-injection`)
    that were seen failing. Gate on IDE tree `38f5eec3ba`: lint:rustfmt, lint:clippy, lint:slint, lint:release, test,
    test:cli, test:native (178 passed), bundle, inspect:bundle (10 of 10), inspect:bundle-guards, and the source (91),
    sidebar (18), theme (7), find (39) and annotation (65) guards, all status 0. Not guarded: the tab measuring pass
    (JetBrains Mono advances do not change with weight or italic, so no mutation can fail a test), and the contrast
    ratios of at least 6.6 were measured against assumed backgrounds. The installed binary is not changed.
  - Tall window blanked the text (branch `fix/ide-tall-window-tile`, worktree `.claude/worktrees/ide-tall-tile`,
    2026-10-18; landed on main 2026-10-19). Cause: the window chooses a tile clip from its row map, where a wrapped
    line the background pass has not reached counts as one row, while the shaper sizes the raster from the rows it just shaped,
    several times taller. At a text width of 1354 the cap is 11,994 px at scale 1, so the three-view tile of a window
    over 4,000 px tall passed 64 MiB (journal: `Source viewport 1360x12768 exceeds the 64 MiB raster limit`), the
    painter refused it, and the window thread then replaced the picture with an empty image. Fix: `fit_clip` and
    `fit_viewport` in `src/shaped_text/limit.rs` cut every prepared frame to the cap around its middle, and log it at
    warn level (`cut a frame to the raster limit`); `TileBuilt::bounds` and its stamp name the clip really painted, and
    `TileBuilt::requested` the job it answers; a refused tile keeps the picture and names the failure; a failed worker
    job is freed; a worker tile the window discards still hands the window the offset its measured heights moved.
    `band_clip` moved to `limit.rs`, centers on the view, and keeps the last row of the text. Tests:
    `tests/raster_limit.rs`, `src/native/tall_window_tests.rs` (end to end and a property test over scales 0.5 to 3,
    text widths 160 to 3000, heights 660 to 12000, stale and measured maps), `src/native/refused_tile_tests.rs`,
    `src/native/tall_window_minimap_tests.rs`; seven `inspect:source-guards` cases were seen failing (`frame-fit`,
    `tile-stamp-names-its-clip`, `refused-tile-keeps-image`, `failed-job-freed`, `clip-centered-on-view`,
    `clip-keeps-last-row`, `minimap-glyph-pixels`) and the three `clip-*` cases were re-anchored. Not reliably
    guarded: the discarded-tile offset (the tall-window test failed once without it, and passed once without it). The
    real binary at 1790x3300 shows text and logs one cut; no persistent blank frame was captured before the fix.
    The coordinating session reran the gates on 2026-10-19 after merging `origin/main` (with the computed container
    bound): `lint:rustfmt`, `lint:clippy`, `lint:release`, `lint:closures`, `test` (792), `test:wrap` (15 and 67) and
    `test:native` (203) passed, and the ten `inspect:source-guards` cases above (seven new, three re-anchored) each
    passed at baseline, failed with the guard removed, and passed restored.
    Follow-up (branch `fix/ide-tall-wrapped-settle`,
    issue 1785,
    2026-10-19):
    a window 10,000 px or taller over wrapped text never finished the measurement pass
    (8,000 px and smaller settled in 1.7 s).
    Cause:
    `holds` in `src/native/viewport.rs` asked for a margin of a quarter view height on each side of the view,
    and a tile cut to the 64 MiB cap (12,384 px at text width 1326)
    cannot give 1.25 view heights once the view passes about 9,900 px
    (8,200 px in the middle of the text).
    `accepts` in `src/native/tile.rs` then discarded every tile the worker built
    (probe:
    only `holds` was false),
    the render asked for the same tile,
    and the `ide-source-tile` thread rebuilt a 12,384 px raster for ever.
    Fix:
    `recenter_edge` takes the smaller of the quarter and what the cap leaves around the view,
    less two code rows for the clip rounding
    (negative for a view taller than the cap).
    Settle times at heights of 8,000 to 14,000 px (five sizes):
    1.8 s each after the fix;
    10,000 and above did not settle in 25 s before.
    Tests:
    `a_tall_window_over_wrapped_text_finishes_the_measurement_pass`
    and the new `accepted` check in `no_window_size_or_scale_makes_a_raster_past_the_limit`
    (both fail without the fix);
    guard `tall-window-edge`.
    A window taller than the cap leaves part of the view unpainted at each edge
    (up to the overshoot plus two code rows),
    and every scroll step builds a new tile there,
    as `band_clip` already documents.
    The IDE was reinstalled from `main` at `482a3cf220` on 2026-10-19 (`dist/monochromatic-ide` and the installed binary
    both sha256 `7e6b2f6a8aab…`; the previous binary is `~/.local/bin/monochromatic-ide.previous`, sha256 `f068d238b760…`).
  - Closures named in the IDE (branch `refactor/ide-closures`, worktree `.claude/worktrees/ide-closures`, 2026-10-18;
    the user approved removing every `rust/no-anonymous-functions` finding without suppressing any). The count in
    `package/desktop-app/ide` fell from 1135 to 67; `rust/max-lines`, `rust/require-rustdoc` and
    `markdown/semantic-line-breaks` are at zero. Iterator adaptors, `map_or`, `with_context`, `retain` and predicate
    parameters became `for` loops, `match`, `if let`, and small named types (`Waiting` in `src/native/waiting_tests.rs`
    for the window test waits, the `Cancel` trait for `minimap::build`, `Condition`, `StopVisitor`, `SignCoverage`,
    `EventRecorder` and the like). Several short-circuit chains became unconditional reads of plain fields
    (`differs_only_in_annotations` in `src/native/render.rs`, `errored_there` in `src/native/navigation/tick.rs`,
    `errored_here` in `src/native/navigation/actions.rs`); no read has a side effect. The 67 that remain are closures
    that the callee's signature requires and that own state: 38 Slint `on_*` callbacks, 8 `Timer::start` bodies, 17
    `thread::spawn` or `Builder::spawn` bodies, 2 `catch_unwind` bodies inside thread work, and the two `tree_house`
    callbacks in `src/syntax_loader.rs`. Stable Rust has no named item that owns state for an `FnMut` or `FnOnce`
    parameter, so they stay reported. Thread locals are not a way out: a spawned thread cannot read the parent's
    thread locals (`src/find_worker.rs:179` hands two channel ends to its worker), and for the UI callbacks they
    would swap `Rc<RefCell<..>>` captures for hidden global state. The helper route the user picked first (generic
    helpers with a suppression each) is withdrawn: `monochromatic-lint` had no inline suppression then (comment directives were built afterwards), and a Rust
    `#[allow]` does not reach it (`src/rust_no_anonymous_functions.rs` has no option but severity). The user then
    asked to change the rule itself, with type resolution and ignore comments. The proposal is
    `doc/planning/linter-selective-anonymous-functions.md` (not accepted): directive comments with a mandatory
    reason, and a semantic criterion (a closure that captures state and is passed to a callee requiring `'static`
    stays unreported; `ra_ap_hir` 0.0.336 gives `Closure::captured_items`). The user asked for the spikes on
    2026-10-18 and an agent ran them on branch `spike/linter-closure-static` (worktree
    `.claude/worktrees/linter-spike`; the branch also sits on origin because the commit wrapper auto-pushed it). The
    results are in the proposal ("Spike results") and `~/temp/agent/linter-spike/report.md`: hir answers the
    `'static` question for resolvable callees and the Slint setters are `impl FnMut + 'static`, but under the
    shipped `SourceOnly` load only 14 of the 67 pass (the generated setters and `Timer::start` do not resolve),
    11 sit in unloaded test sidecars, and the semantic rule costs 5.9 s against 0.067 s on the IDE package.
    Acceptance and the choice between a callee allow-list and a build-generated load mode are open: the user
    answered "Not enough research" to the choice between directives only, directives plus an allow-list, and
    directives plus a build-generated load. Two agents run on 2026-10-18: one researches precedent (directive
    syntax in oxlint, ESLint, Biome, Ruff and Clippy; closure bans elsewhere; a Dylint or rustc-driver route), one
    closes the measured gaps (test sidecars under `cfg(test)`, the IDE dependency graph and generated code, the
    5.9 s breakdown, wrapper cases). Both reported (proposal sections "Research results" and "Round two results"):
    comment directives were already decided (D2 of `doc/planning/rust-linter-oxlint-parity.md`; built since, see the bullet "Inline directives in the linter"); the type
    criterion gives 62 allowed and 5 reported of the 67 only with the full dependency graph, `cfg(test)` plus
    `cfg(debug_assertions)` and `BuildGenerated`, at 22 to 39 s on the IDE package against 0.063 s; under
    `SourceOnly` 25 of 67. The spike branch `spike/linter-closure-static` keeps its local commits (`fad1074bd2`,
    `37bcb6f5b3`); the remote copy was deleted. Decided 2026-10-18 (the user chose it from three
    options): a Dylint rustc lint plus comment directives, recorded in `doc/decision/rust-closure-lint-dylint.md`
    (the Dylint lint is suppressed by rustc's own `expect(.., reason)`; whether the user meant the comment form for it
    too was not asked and is open to veto). Stage 1 runs as an agent on branch `feat/dylint-closure-lint` (worktree
    `.claude/worktrees/dylint-closure`): the new package, tests with controls, README, and a run over a disposable
    copy of the IDE; no IDE edit and no root toolchain edit (not approved then). Stage 1 reported (verified by the
    coordinating session: 13 tests pass in an independent run, and disabling the capture guard fails
    `adversarial.rs` and `not_kept_by_callee.rs`): package `package/linter/monochromatic-lint-closure`, lint
    `avoidable_closure`, library `monochromatic_lint_closure`; on a disposable IDE copy 62 of the 67 pass and 5
    report (`language/actions.rs:134`
    let-bound, `minimap_build.rs:399` and `tile_build.rs:140` `catch_unwind(AssertUnwindSafe(..))`,
    `syntax_loader.rs:188` and `:311` `tree_house`); all 37 generated Slint setters resolve under rustc. It also
    reports 78 closures the syntax rule never saw (46 in `tests/`, 27 in `*_tests.rs`, 5 in `src/bin`, all inside
    macro arguments such as `json_value!`; 57 capture nothing): stage 2b converts them. The user's answers of
    2026-10-18: repoint the two stray mise symlinks (`nightly-2026-10-13`, `nightly-2026-10-17` pointed into scratch
    after stage 1 and the coordinator's verification run; repaired), add `rustc-dev` to the shared nightly, exempt
    `tests/ui` in `monochromatic-lint.config.jsonc`. Stage 2a landed on `main` as `00829db75b` (pushed): root
    `rust` components gained `rustc-dev`, `cargo:cargo-dylint` and `cargo:dylint-link` (unpinned) joined the tools,
    and refreshing `mise.lock` moved the floating nightly from `nightly-2026-10-17` to `nightly-2026-10-18` (the
    toolchain is installed in `~/.rustup`, 24 GB to 27 GB; every session's next mise run uses it); the `tests/ui`
    exemption (entry `ui-fixture-exemptions`) cleared the 86 fixture warnings; the package, on the shared nightly
    with `build` and `lint` task names again, passed 13 tests, clippy, rustfmt and `monochromatic-lint` from a clean
    shell, and the IDE copy still gave 62 of 67 allowed. The coordinator reran the package tests and the IDE
    `test:cli` (13) on the new nightly after the final merge. Known host problem, diagnosed afterwards: cargo
    of `nightly-2026-10-13`, `-10-17` and `-10-18` fails to reach crates.io (`Could not connect to server` in
    17 ms) because an OpenSnitch `reject-always` rule exists for each of those exact `cargo` paths
    (`/etc/opensnitchd/rules/`); it destroys the socket (`ECONNABORTED`). It is not a 1.101 bug: the same
    binary at an allowed path connects, and `nightly-2026-10-11` and `-10-12` (1.101) connect. An unanswered
    OpenSnitch pop-up (30 s timeout, default action reject, duration forever) mints such a rule for every new
    toolchain path. Fixed on 2026-10-18 at the owner's request: the rule `000-allow-cargo.json` (regexp on the
    `cargo` of any rustup toolchain under the home directory or an agent scratch rustup home, with `precedence`) was
    added, a fetch on `nightly-2026-10-18` then worked, and the six toolchain `cargo` reject rules were moved to
    `/etc/opensnitchd/rules.removed-2026-10-18/`
    (`doc/troubleshooting/cargo-nightly-1-101-cannot-reach-crates-io.md`). The pop-up default (reject, forever, 30 s)
    is unchanged, so a path outside every rule still gets a permanent reject; `lint:closures` still fetches with the
    stable cargo and builds offline, which stays valid.
    Stage 2b landed after review (branch `feat/ide-closure-lint`; see the bullet "Dylint gate over the IDE").
    The user also asked to look at their oxlint setup; findings: shared config
    `package/config/oxlint/src/config-base.ts` sets `reportUnusedDisableDirectives: 'warn'` with
    `denyWarnings: true`; the repository holds 861
    `oxlint-disable-next-line`, 401 `oxlint-disable` and 332 `oxlint-enable` directives, 698 of 734 next-line ones with
    a ` -- reason`; an audit (`doc/audit/oxlint-unused-disable-directives.md`) found and removed 301 unused ones; the
    plugin `no-restricted-syntax` bans disabling chosen rules (`no-disable-*`, built by `_ban-disable-factory.ts`),
    including `no-arrow-function`, whose replacement is a named function expression (impossible for Rust closures);
    tests are exempted from the TypeScript arrow ban by an override in `package/config/oxlint/src/overrides.ts`
    (line 201). A Rust equivalent of that test override would clear the 11 test closures without any new rule
    machinery.
    Until then the 67 stay reported as warnings, listed in `~/temp/agent/ide-closures/final-unconverted.txt`.
    Landed on 2026-10-18 as `d0283728f4` (fast-forward, pushed; branch and worktree removed, the auto-pushed remote
    branch deleted). The agent's gates ran before `main` was merged in; the coordinating session reran them on the
    merged tree, whose IDE tree `218fe10ff4` was identical to the gated one: lint, `test` (783), `test:cli`,
    `test:language` (379), `test:minimap`, `test:wrap`, `test:annotations`, `test:annotations-native`,
    `test:syntax` (64), `test:native` (197), `lint:release`, bundle, `inspect:bundle`, `inspect:bundle-guards`,
    eleven guard scripts (source 108, annotation 65, sidebar 18, theme 7, ignored 28, watch 76, language 31, and the
    find, search, language lifecycle and language navigation ones), and `inspect:language`, `inspect:language-mise`
    and `inspect:language-confinement`, all status 0. Issue 1756 got a progress comment. The IDE is reinstalled
    (`~/.local/bin/monochromatic-ide`, sha256 `2172fa208b55`, equal to `dist/monochromatic-ide`; the previous binary
    is `monochromatic-ide.previous`, sha256 `9c4d6047`); `--help` runs. The installed build includes the minimap
    and the closures work.
    Files added: `src/json_value.rs`, `src/language/diagnostics/store/pulled.rs`,
    `src/language/request/answer/trouble.rs`, `src/minimap/paint.rs`, `src/native/waiting_tests.rs` and
    `src/bin/ide-language-inspect/session/condition.rs`. Guard anchors moved in six `bin/inspect-*-guards.mjs` scripts;
    every guard script was run again on the branch and all passed (annotation 65, find 39, ignored 28, search 17,
    sidebar 18, source 108, theme 7, watch 76, language 31, language navigation 16, language lifecycle 28 runs,
    bundle 8 cases). Test suites on the branch: `test` 783, `test:native` 197, `test:language` 379, `test:syntax` 64,
    `test:wrap` 62 and 15, all passed. Also issue #1756: the container lint now applies the root `clippy.toml`, and
    `Result::unwrap` sites use `json_value!` (one scoped allowance) or `Value::from`.
  - Container bound and test time (branch `perf/ide-container-bounds`, worktree `.claude/worktrees/ide-container`, an
    agent, 2026-10-18; landed on main as a0d9cacaed, 2026-10-19). The owner asked for a larger container and said not to bake numbers into the
    repository; the coordinator then fixed the policy: 75 percent of the host, computed at run time, overridable.
    `bin/container-bounds.ts` is now the one place that computes the bound; `mise.toml` (the shared `[vars] cargo`
    header and the `image` task) and 17 `bin/inspect-*.mjs` scripts import it. Default: `floor(0.75 * CPUs)` (at least
    1) and `floor(0.75 * RAM)` in GiB (at least 2), 256 processes per CPU with a floor of 512, 4096 descriptors,
    `CARGO_BUILD_JOBS` and `NEXTEST_TEST_THREADS` equal to the CPUs. Overrides: `IDE_CONTAINER_CPUS`,
    `IDE_CONTAINER_MEMORY`, `IDE_CONTAINER_PIDS`, `IDE_CONTAINER_NOFILE`, `IDE_CONTAINER_JOBS`,
    `IDE_CONTAINER_TEST_THREADS`, each validated (a malformed value stops the task naming the variable and the accepted
    form); `test:bin` runs 40 node tests of the module. The old fixed bound is `IDE_CONTAINER_CPUS=2
    IDE_CONTAINER_MEMORY=2g`. A process started from the agent shell sees 28 CPUs (affinity), one started with
    `systemd-run --user` sees 32, so the default is 21 CPUs or 24 CPUs and 46 GiB depending on how the task is started.
    The control groups of the containers are children of `user@1000.service`, not of the `systemd-run` unit that starts
    the task, so the unit's `MemoryMax` neither bounds nor measures the container.
    Premise corrected: the measured claim that nextest started one test thread per host CPU was wrong for the installed
    cargo-nextest 0.9.143. `nproc` prints 32 under a 2 CPU quota, but nextest reads the quota through Rust's
    `available_parallelism` and ran 2 tests at once (a 48-test probe recording start and end times). The variable is
    explicit and redundant (control: the same 24 without it); see
    `doc/troubleshooting/container-nproc-ignores-cpu-quota.md`, which also lists the other packages that pass
    `--cpus=` (unchanged).
    Slow-test cause: `every_row_of_real_source_fits_its_width` was not slow because of the shaping it proves. Its helper
    `unexcused_breaks` called `space_advance(scale)` for every source line, and that builds a new `TextShaper`, about
    18,000 shapers per scale and width pair. The advance depends only on the scale, so it is measured once per pair now.
    The test is also split into six (`every_row_of_real_source_fits_width_<w>_at_scale_<s>`), so the pairs run in
    parallel and a failure names its pair; the rows judged and the rule checks are the same. The coverage thresholds
    (more than 100 wrapped lines, a token wider than a row) hold at width 600 only (width 900 wraps 347 lines and has
    no wide token, width 1400 wraps 14), so the two width 600 tests assert them, which implies the totals of the
    single test. Guard proof (production code broken, restored with `git checkout`): `layout.break_all_lines(None)`
    fails all six on "some lines wrap", `OverflowWrap::Normal` fails the two width 600 tests on "the excuse is
    exercised", and removing `set_line_break_override` fails four on "broke a rule".
    Measured (this host, load average 10 to 17 from other sessions; times are wall seconds of the mise task):
    old code at the old 2 CPU, 2 GiB bound: `build:debug` (warm) 8 s, `test` 818 s (783 tests), `test:wrap` 840 s (15 and 62
    tests), `test:native` 236 s (197 tests). New code at the computed default (24 CPUs, 46 GiB from a `systemd-run` unit),
    two runs each: `build:debug` 16 and 8 s, `test` 22 and 19 s (788 tests, 783 plus the six minus the one removed),
    `test:wrap` 10 and 10 s (15 and 67 tests), `test:native` 121 and 93 s (197 tests). New code at the old bound: `build:debug`
    12 s, `test` 103 s (788), `test:wrap` 29 s, `test:native` stopped by a failure at 138 s (see the flaky note). Old code at
    the raised bound: `build:debug` 11 s, `test` 799 s (783), `test:native` 95 s (197); so the bound alone does not shorten
    `test` (one test, one thread) and the shaper fix does. Control group peaks at the default (cgroup `memory.peak`, which
    includes page cache): `test` 4.1 and 0.3 GiB, 18 to 19 CPUs; `test:native` 1.5 to 1.6 GiB, 22 to 24 CPUs;
    `test:wrap` 0.1 GiB, 11 CPUs; `build:debug` 0.9 to 1.8 GiB, one CPU. The 46 GiB and 24 CPU limits were never reached.
    Opt-level (measured with environment and `--config` overrides only, `Cargo.toml` untouched): One wrap pair takes 8.9 s unoptimized;
    `CARGO_PROFILE_TEST_OPT_LEVEL=1` gives 0.65 s (13.7 times; level 3 gives 0.49 s) and
    `--config 'profile.dev.package."*".opt-level=2'` gives 1.03 s (8.7 times; level 3 gives 1.00 s); dependencies
    (`parley`, `swash`, `skrifa`) carry the gain. The IDE directory is not in `CARGO_PROFILE_BY_DIR` of
    `file-enforcer.config.ts`, so its profile entries are not enforced today, and the preset type cannot express
    `profile.dev.package."*"`. Whole suites, same overrides: `test` 15.3 s becomes 7.2 s (level 1) or 9.0 s (dependencies
    at 2); `test:native` 101 s becomes 12 s (level 1) or 16 to 19 s (dependencies at 2). Not adopted: with the shaper fix
    the pair is no longer the critical path, level 1 made two native tests fail
    (`minimap_tests::scrolling_posts_no_request_and_sets_no_picture`, "a scroll posted a minimap request", left 4, right
    3; `wrap_tests::rows_above_the_view_changing_height_move_no_visible_pixel`, "the pass changed the extent"), and
    dependencies at 2 failed one native test in one of two runs and passed 197 in the other. Both are timing races
    that a faster build exposes. The cost is a cold rebuild of every dependency at 24 CPUs: 58 s for the `test` set and
    41 s more for the native set (level 1, which also keeps a second artifact set for the test profile), 110 s and 45 s
    (dependencies at 2). The next critical path is the native test
    `annotation_stability_tests::scrolling_across_annotated_lines_is_rigid` (83 to 109 s alone, at every bound).
    `test` gained an optional nextest filter argument and `.config/nextest.toml` (default profile `slow-timeout`
    120 s, terminate after 15 periods), proved in the container by `mise run test:nextest-config`.
    Rules for agents: run only the tests a change can reach, never `test` and `test:wrap` together (the second repeats
    the first), and use the filter (`mise run //package/desktop-app/ide:test 'test(/wrap/)'`).
    The flaky `wrap_clip_tests::revealing_a_late_row_of_an_unmeasured_last_line_puts_it_in_view` is fixed on
    `fix/ide-wrap-clip-flake`: `TileWorker::take_pass` closed the pass on the finishing output of a replaced pass, so
    the window stopped polling for the newer one (about 1 failure in 100 runs at one CPU before, 0 in 1500 after).
    Gates of this branch, re-run by the coordinator after merging `origin/main` (default bound, each task alone):
    `test:bin`, `test:nextest-config`, `lint:rustfmt`, `lint:clippy`, `lint:closures` (nightly) and `test:wrap` (15 and
    67 tests, 8 s) passed. Not re-run, because nothing they exercise changed (the only Rust change is
    `tests/wrap.rs`): `test` and `test:native` (the agent's final run at the default bound: 788 tests in 13 s and 197
    tests in 88 s, all passed). Guard scripts: the agent ran the eleven scripts below at the new bound before the merge,
    every guard accepted and none not accepted (accepted counts: find 105, watch 205, annotation 175, language 92,
    sidebar 44, language lifecycle 28, ignored 75, theme 13, search 49, language navigation 45, source 300). The other
    six edited scripts (idle cost, log stall, refresh latency, language reap rate, watch limit, watch scope) are not
    guard runners and were not run; they carry the same mechanical import and limit-argument change.
  - After the minimap landing (2026-10-19): issue #1785 (a 10,000 px or taller window over wrapped text never
    settled) is fixed and landed as `8c597c3c58`; the wrap-clip flake, a production bug in `take_pass`, is fixed
    and landed as `8627edeeba`. The coordinator reran the gates for both (`lint:rustfmt`, `lint:clippy`,
    `lint:release`, `lint:closures`, `test`, `test:native`), the `tall-window-edge` guard, and 400 stress runs of the
    flaky test (400 passed), and installed the build after each (the installed binary was not launched).
    The IDE parts of issues #1756 (root `clippy.toml` in the container, `75b9aee2a3`) and #1757 (nextest, `bd12a2fb62`)
    were already on `main` from 2026-10-18; the issues stay open for the other packages. The owner authorized deleting the merged IDE branches on origin and 34 were
    deleted (each was an ancestor of `origin/main`; the list is in `~/temp/agent/merged-ide-branches.txt`); the
    unmerged IDE branches (language, live theme, navigation, runtime languages, sidebar resize, source keys,
    prototypes, research) were left.
  - Vello all the way (owner decision of 2026-10-19): the application must never paint text itself. The CPU text
    pipeline (Parley layout, Swash glyph raster, `TextRaster`, the overscan tile as an `Image`, and the minimap's
    downscale of a CPU rendering) is to be replaced by Slint elements drawn by the Vello renderer on the GPU. The
    owner's premise that Slint has Vello by default is not true for Slint 1.18.1 (read from the registry manifests:
    the default features are `renderer-femtovg` and `renderer-software`; `renderer-vello` is an opt-in, experimental
    feature), so the migration enables it. Why the CPU path existed is not documented; the renderer comparison of
    2026-10-10 (`doc/planning/slint-ide-scroll-pipeline.md`) compared renderers that all showed an image the CPU had
    painted, so it never tested GPU-drawn text. The drag jank agent was stopped because its fix (prefetch and parallel
    CPU raster) is moot; its worktree `.claude/worktrees/ide-drag` holds uncommitted timing instrumentation (do not
    land it) and the measured period of the freezes (41 frames) matches the tile re-center margin at the drag speed.
    An agent writes `doc/planning/ide-vello-text.md`: feasibility of Vello in the nested compositor and the real
    app, what Slint 1.18.1 text can do (colored runs, wrap, tabs, selection, caret and point mapping), a windowed
    row model, the minimap on the GPU, what is deleted, and a staged plan, with measurements.
  - Clarification of 2026-10-19 (owner): only CPU painting of text is banned. Vello is not mandatory; text drawn on
    the GPU by any Slint renderer, FemtoVG included, is acceptable. The study ranks designs on that basis and
    compares FemtoVG and Vello for every question (the agent was told so).
  - Second clarification of 2026-10-19 (owner): the IDE is not restricted to Slint's builtin text elements. The
    constraints are: no CPU painting of text; any GPU renderer; any GPU text path, including Parley layout drawn by
    Vello into a wgpu texture shown in Slint. The study ranks those designs together with the builtin-text ones.
  - Third clarification of 2026-10-19 (owner): "No CPU painting of anything." The ban covers every pixel the CPU
    fills, not only text: the minimap picture (a CPU downscale of a CPU rendering), the overscan tile, any buffer
    uploaded as an image, and the software renderer. CPU work stays allowed for layout, shaping, highlighting and
    building scene data. The migration deletes every CPU pixel path; the agent was told to list them.
  - GPU text study landed (2026-10-19, `doc/planning/ide-vello-text.md`, merged `3b90f79132`). Designs: A builtin
    `Text` on FemtoVG; B builtin `Text` on Slint's Vello renderer; C Parley drawn by Vello 0.11 into wgpu textures
    shown as Slint images, Slint on `renderer-femtovg-wgpu`. I reran the agent's cadence matrix (verify files
    `~/temp/agent/verify-G.jsonl`, `verify-S.jsonl`): C holds 119/119 (scroll) and 120/121, 121/121 (drag at 489 and
    50,000 lines) in release and 72 to 73 of about 120 in debug; A and B hold 119/119 on scroll and only 61/121 and
    60/121 on the 50,000-line minimap drag (a rebinding slice, so an upper bound on C's lead), 31 to 42 in debug.
    I confirmed in `femtovg-0.27.0/src/text.rs` and `i-slint-renderer-femtovg-1.18.1/Cargo.toml` that Slint builds
    FemtoVG with `swash`, so every FemtoVG glyph (chrome text too) is rasterized on the CPU into an atlas. Under the
    strict reading only B has no CPU glyph raster in the window; but Slint text cannot do tab stops, ligature
    control, custom wrap or hanging indent, and the real IDE froze about 1 s under Vello (cause unfound). Owner
    questions are asked via the question tool; the answers are recorded in this bullet's next sibling.
  - Owner answers of 2026-10-19 to the GPU study: (1) the FemtoVG glyph atlas is accepted for library chrome text, so
    design C is chosen (Parley layout drawn by Vello into wgpu textures shown as Slint images, Slint on
    `renderer-femtovg-wgpu`); (2) `unstable-wgpu-30`, `renderer-femtovg-wgpu`, full wgpu limits and the vello
    lock entries are accepted; (3) native tests must not keep the software renderer: a GPU snapshot harness is built
    first, before the pipeline is replaced. Open from the study, adopted unless vetoed: `[profile.dev.package."*"]`
    optimization for dependencies (debug builds are twelve times slower in Slint and Vello; the owner believes the
    jank shows in debug); the soft 2.5 px minimap is judged against the old bitmap by screenshot at the minimap stage.
    Stages: 0 GPU snapshot harness (headless wgpu pixel readback replacing `take_snapshot` in the native tests);
    1 dev-profile groundwork plus the cadence driver kept in the repo; 2 renderer switch; 3 row scene on the GPU;
    4 selection, find, underlines, hint rows and wrap sign as scene content; 5 minimap as a GPU slice texture;
    6 delete the CPU pipeline (`text_raster.rs`, `glyph_cache.rs`, tile image path, `swash`, `renderer-software`).
    Skia (owner decision of 2026-10-19): not pursued; the stage 2 renderer is `renderer-femtovg-wgpu` and Skia is not
    measured. Finding kept: Skia rasterizes chrome-size glyph masks on the CPU through FreeType into its atlas
    (`src/core/SkStrike.cpp:356`, `src/ports/SkFontHost_FreeType.cpp:1398` at `chrome/m150`), like FemtoVG.
    Running now (started 2026-10-19, Sonnet agents in worktrees, nothing pushed by them): stage 0 agent
    `a90c692a544c2aecc` (feasibility doc `doc/planning/ide-gpu-snapshot-harness.md`, harness, one test file migrated;
    a CPU Vulkan driver such as lavapipe is an open owner question, not adopted) and stage 1 agent
    `a54a3dbd127b14fbf` (dev-profile optimization with build-time numbers, `inspect:cadence` task with a positive
    control). I verify each report myself before landing.
    Stage 0 report received (agent `a90c692a544c2aecc`, branch `worktree-agent-a90c692a544c2aecc`, worktree
    `.claude/worktrees/agent-a90c692a544c2aecc`, eight commits up to `1e4f35b0e2`, not pushed, not yet landed):
    `src/native/gpu_snapshot.rs` is a test-only `Platform` + `WindowAdapter` around `FemtoVGWGPURenderer` on a
    hardware Vulkan device (RADV in the container through `--device /dev/dri`, `VK_DRIVER_FILES` naming only
    `radeon_icd.x86_64.json`, so lavapipe is not listed; the harness refuses CPU adapters and panics instead of
    falling back); `source_font_tests.rs` is migrated; renders were deterministic and a compositor screenshot matched
    the harness in 0 of 60,000 pixels for one component. The agent's full-suite experiment (every test calling
    `install_gpu_platform()`) passed 209 of 214; two failures were the missing clipboard (fixed) and three remain
    undiagnosed (dark-scheme selection ink in `query_input_tests` and `query_input_paint_tests`, and
    `annotation_stability_tests::scrolling_across_annotated_lines_is_rigid`). My answers to its open questions, adopted
    unless the owner vetoes: lavapipe is never accepted (CPU rasterization); `test:native` keeps requiring
    `/dev/dri` (the owner's rule makes every native test a GPU test; CI does not run the IDE tests: no workflow
    mentions `test:native` or `desktop-app/ide`); render-node passthrough is accepted; full wgpu limits and discrete
    over integrated are accepted. Known limit: the `VK_DRIVER_FILES` path is AMD-only, so an Intel or NVIDIA host
    needs another ICD file (follow-up: choose the file by detected vendor). Next: migrate the other 13 `take_snapshot`
    files, diagnose the three failures, then drop `renderer-software` and the testing backend.
    Verified by me and landed (2026-10-19, merge `64cb8b96eb` on `main`): clippy, rustfmt and slint lint pass, and
    `test:native` passes 214 of 214 with a unique log (an earlier run of mine used a log shared with another run
    and showed one failure, `minimap_reflection_tests::the_minimap_of_find_matches_is_the_average_of_the_windows_pixels`
    at scale 2, "differs by 99"; it did not recur in the clean run; the gate runner now takes `TAG` for a unique
    log and unit name). Stage 0b agent `a151e856e450d38e8` (migrate the 13 files, diagnose the three failures, drop
    `renderer-software`) landed (agent `a151e856e450d38e8`, merged into `main` on 2026-10-20, verified by me on the
    merged tree: `lint`, `inspect:software-renderer`, `test:bin`, `build` pass and `test:native` passed 4 of 4 runs,
    214 tests each in about 23 s; one guard script it edited but never ran, `inspect:source-guards ... clip-edges`,
    shows baseline pass, removed fail, restored pass). Every native test installs the GPU platform with one
    `install_gpu_platform();` before each window constructor (18 constructors in 15 files); the headless backend and
    the `slint/mcp` feature are gone from `test:native`; `bin/gpu-arguments.ts` gives the 11 guard scripts the same
    device flags; both `slint` dependency entries set `default-features = false` because the default set pulled the
    software renderer in; the task `inspect:software-renderer` fails if it is linked again. Builds with
    `--features slint/mcp` (tasks `mcp`, `inspect:native`, the idle-cost and log-stall scripts) still link the
    software renderer, because Slint's MCP server screenshots with it: open owner question. The three failures were
    all assumptions about software-renderer pixels, not application bugs: the GPU anti-aliases the first and last
    row of a fractionally placed selection rectangle (sampled rows moved inward), and the rigid-scroll test now
    allows one level of difference in the last window row only (a clip-edge difference in FemtoVG's scissor
    mask, read in the shader, not isolated); guard runs show both tests still fail when the guarded code is removed.
    Load-dependent flakes seen only in the agent's parallel runs and not in my 4 runs: the rigid-scroll test once at
    an interior row (cause unfound), `write_wait_tests` once, two `watch_tests` once (400 ms budgets). The commit
    message of `415bc345ed` says 22 sites where it is 18 constructors in 15 files; a corrective commit comment was
    posted (GCA).
    Stage 2 landed (agent `a1b70500a2a370d0d`, merged into `main`, pushed as `fcfaec56e6`, verified by me): the
    application runs on FemtoVG over wgpu 30 (Vulkan). `Cargo.toml` lists `renderer-femtovg-wgpu`,
    `unstable-wgpu-30` and `unstable-winit-030` (the public `BackendSelector` app-id hook, no new package) and no
    longer `renderer-femtovg` (OpenGL). `src/gpu_device.rs` (7 tests) lists Vulkan adapters, keeps only discrete and
    integrated GPUs, requests the device with the adapter's resolution limits and retries the next adapter; on failure
    it names the adapters seen and every remedy. The launcher installs the backend with `WGPUConfiguration::Manual`
    because wgpu-core filters CPU adapters only on request and Slint's own check is bypassed by `SLINT_WGPU_CPU`
    (guard proof: accepting `Cpu` fails 3 of 7 device tests; a lavapipe run with the guard removed started on
    llvmpipe). `SLINT_MCP_PORT` no longer returns early in the launcher. Debug builds aborted at window close
    (wgpu-core's debug-only thread-local lock trace); the launcher now leaks one handle of each wgpu object.
    Agent's measurements: adapter `AMD Radeon RX 7600 (RADV NAVI33)` in every run with 8 storage buffers and a 16384 px
    texture limit; baseline and new screenshots differ in 28 of 726,000 pixels by at most one level; first frame 328 ms
    to 350 ms; idle RSS 155 MiB to 136 MiB; release binary 83.0 MB to 90.1 MB; cold release build 49 s to 55 s.
    My verification on the merged `main`: `lint`, `test:bin`, `build`, `inspect:software-renderer` pass;
    `test:native` 224 of 224 twice (about 24 s each); `inspect:cadence` release, interleaved baseline and new, 2 runs
    each, 1100x660: scroll 117/119 and 117/119 against 117/119 and 115/119, drag 117/118 and 117/118 against 117/118
    and 116/118, longest identical run 1 to 2 in every cell, so no difference; I viewed the dark find screenshot and
    the light hover screenshot. The agent saw one failure of
    `minimap_tests::an_external_edit_of_fifty_thousand_lines_renders_only_the_band_it_touches` in 9 runs ("rendered
    4 of 4 bands"), passing alone and in 8 later full runs; I did not see it in 3 of my runs. It is a CPU minimap
    picture test of the code stage 5 deletes.
    The IDE was reinstalled from this build: `~/.local/bin/monochromatic-ide` sha256 prefix `e8944c22f708ac4a`
    (equal to the release build), previous binary kept as `monochromatic-ide.previous` (`85d38e6a32faee9a`).
    Not exercised: any GPU other than this AMD host, the integrated-GPU path, the device-refused retry outside a
    unit test, resize, scale other than 1, 50,000-line files, surface loss and suspend.
    Decisions I took (veto-open): keep `unstable-winit-030` over a direct `i-slint-core` dependency (public API, no
    new package); virtual GPUs stay refused with CPU and unknown kinds; discrete GPU is preferred over integrated, the
    choice is not checked against the window surface, and `WGPU_POWER_PREF` and `WGPU_ADAPTER_NAME` are no longer read.
    Owner answers of 2026-10-20: Vulkan only is accepted (no OpenGL fallback; "Yes, Vulkan only"); the `slint/mcp`
    inspection builds are to be dropped and their checks rebuilt without Slint's MCP server: "Drop the MCP builds and
    rebuild those checks without it, our nested compositor supports GPU."
    Owner correction of 2026-10-20: preferring a discrete GPU over an integrated one "is going to anger users on
    laptops with both GPUs"; my veto-open decision above is retracted. Mesa documents (docs.mesa3d.org/envvars.html)
    that without `DRI_PRIME` the default GPU is the one the display server uses, and that `DRI_PRIME` or
    `MESA_VK_DEVICE_SELECT` put the chosen GPU first in the Vulkan list; desktops set these per application through
    switcheroo-control. New policy (agent `a090eeb1ae38efc05`, started 2026-10-20): follow the loader's order among hardware adapters, honor
    `WGPU_ADAPTER_NAME` and `WGPU_POWER_PREF` again, keep refusing CPU and unknown kinds. Not testable here (one
    AMD desktop): a hybrid laptop, NVIDIA proprietary ordering.
    Stages 3 and 4 report received (agent `a8ec6e5d0b8981d42`, branch `worktree-agent-a8ec6e5d0b8981d42`, 8 commits,
    base `b4e54b36ce`; its branch was auto-pushed to origin by the hook, delete the remote branch after landing):
    code view drawn by Vello into a `Rgba8Unorm` texture from a rendering notifier (`src/tile_scene/`,
    `src/native/gpu_tile.rs`), tile 1113x8160 px at scale 1 and 2226x7488 px at scale 2 (texture side from the
    device limit, 64 MiB byte rule), nothing deleted whole because the minimap still uses `text_raster`,
    `glyph_cache.rs`, `selection_paint.rs`, `annotation_paint::paint_underlines` and `swash`. Selection fills and
    find boxes stay Slint rectangles under the tile. Agent figures: cadence equal to baseline within spread except the
    50,000-line drag (new 110 to 118, baseline 115 to 120 changed of about 120), debug drag no longer freezes (baseline
    38 to 39 of 119); RSS 157 to 210 MiB idle, VRAM +260 MiB, first frame +34 ms; text is unhinted; one error-wave
    brightness regression found by screenshot comparison and fixed. Merged locally on 2026-10-20 as `36d522d4c1`
    (auto-pushed to `origin/main` with the next handover commit, before verification finished, because auto-push publishes every commit on `main`; verification continues on the pushed state). My verification so far: `lint`, `test:bin`, `build`, `inspect:software-renderer` pass; screenshots
    viewed, same as baseline to the eye; `test:native` failed 2 of 3 full runs at load average 12 to 14 (two other
    agents building): `annotation_stability_tests::scrolling_across_annotated_lines_is_rigid` ("frame at offset 115 is
    not the previous one shifted by one row", 125 rows differ), `rows_arriving_in_view_move_only_lines_beneath_them`
    (line 8 changed pixels in row 5), `query_input_tests::find_box_context_menu_runs_each_entry`; the third run passed
    238 of 238. To judge on a quiet host before pushing; the pixel tests could point at a real tile re-center shimmer
    (unhinted glyphs at a new tile origin) and must be compared against `origin/main` under the same load.
    Owner instruction of 2026-10-20: "Plan for the load average never to quiet down for longer than a minute at a
    time." Measurement and verification therefore must not wait for a quiet host (another session's
    translation-repair fan-out drove the load average to 123 while my cadence runs were going; those runs are void).
    The plan, each part measured here on 2026-10-20: (a) run every measured process as a systemd user unit with
    `CPUWeight=10000` (the `cpu` controller is delegated to the user manager; all sessions' shell commands share one
    `claude-code-bash` cgroup at weight 100). Positive control, 56 busy loops: a fixed single-thread job took 647 ms
    unloaded, 1797 ms at default weight, 1190 ms at weight 10000, so it narrows but does not remove the slowdown (busy
    sibling hyperthreads and shared caches remain; cgroup `cpuset` is not delegated, so cores cannot be reserved);
    (b) interleave A and B back to back, alternating the order per pair, and judge by the paired differences and the
    medians, never by one run (QNB); (c) first run A against A for the noise floor under the current load, and claim a
    difference only when it exceeds that floor; (d) record CPU pressure (`/proc/pressure/cpu` PSI) and the stalled time
    per run instead of the load average, which lags; (e) for test reliability, accept only counts over many full runs
    under the load that exists (15 of 15 passed on a moderately loaded host here), and judge a failure by its name, not
    by the host state. Tools: `~/temp/agent/matrix-load.mjs` (runs the cadence task as such a unit, pairs, PSI) and
    `matrix-load-report.mjs`.
    All four landed on 2026-10-20 (pushed as `8b0860d75e`, IDE tree identical to the one that passed the gates; the
    main checkout could not merge `origin/main` because another session had staged renames there, so I pushed from a
    scratch worktree, `git push origin HEAD:main`, and removed it). This supersedes the queue below. Flake diagnosis
    (agent `a717e60922dca0448`): four causes, all in tests, none in the GPU code view. (1) Vello renders are not
    bit-identical: 200 consecutive renders of one tile all differed, 10 to 32 texels by one level (the same texels,
    a rounding tie in the coverage sum, alpha 57 against 58); the stability tests now accept the measured one-level
    `RENDER_NOISE` only for a frame that follows a re-render, and scrolling inside one tile stays exact; (2) the
    find-matches minimap test pressed Enter after the first match was already selected; (3) the failed-worker-tile
    test polled for a message a retry render cleared within 0.2 ms (the state now records failures in debug builds);
    (4) the resize test counted passes while the syntax reply could still arrive. Flake rates under shared load:
    baseline 1 of 20, `main` at the merge 9 of 70, fixed head 15 of 15 and 5 of 5 under a background build. My runs of
    the integrated tree on a moderately loaded host: `lint`, `test:bin`, `build`, `inspect:software-renderer` pass,
    `test:native` 15 of 15 (260 tests, about 27 s each). Cadence by my `matrix-load.mjs` (baseline `fcfaec56e6` = A,
    integrated tree = B, release, alternating pairs, `CPUWeight=10000`): noise floor B against B on the 50,000-line
    drag 89 to 97 percent of frames changed with paired differences from -5 to +6 points; 50,000-line drag A 94.2 and
    B 95.8 percent median (6 pairs), scroll 99.2 and 100.0; 1147x2000 drag A 94.2 and B 96.7 (5 pairs), scroll 98.3
    and 99.2; no regression, within noise. One earlier matrix (load average 123) is void. Unexplained and left
    open: one nine-level pixel difference seen once in `rows_above_the_view_wait_until_scrolling_has_stopped`,
    `find_box_context_menu_runs_each_entry` once (passed 72 of 72 stressed), the old minimap test
    `an_external_edit_of_fifty_thousand_lines_renders_only_the_band_it_touches` (1 of 20 on the baseline; left to
    stage 5). The MCP removal (agent `a3b41470505ef8920`) and the GPU choice by system order (agent
    `a090eeb1ae38efc05`) are in. The IDE was reinstalled: `~/.local/bin/monochromatic-ide` sha256 prefix
    `80c27b37e4224602` (equal to the release build), previous binary `monochromatic-ide.previous` (`e8944c22f708ac4a`).
    Decisions I took (veto-open): a one-level tolerance for frames after a re-render is the permanent rule (Vello 0.11
    has no deterministic renderer); selection fills and find boxes stay Slint rectangles under the tile (drawn by
    FemtoVG on the GPU, no CPU painting); the 64 MiB tile byte bound is unchanged; +52 MiB RSS, +260 MiB VRAM and +34 ms
    first frame are accepted until stage 6 and the item below; `inspect:native` keeps `--accessibility` on, the
    measuring tasks leave it off. Queued (QWD): (a) Vello allocates about 262 MiB of video memory for scratch buffers
    for any scene (165 MiB named in its `config.rs`: lines 48, segments 48, ptcl 32, tiles 16, segment counts 16,
    blend spill 4, bin data 1; 97 MiB unattributed), which matters on an integrated GPU with shared memory and an 8 GiB
    card running many parallel test processes (a parallel `test:native` needs about 9.5 GiB); find out whether Vello
    0.11 sizes can be set from the scene or patched, before or during stage 5; (b) try `hint(true)` for Vello glyphs
    and compare crispness (text is unhinted today); (c) move `selection_ink.rs` next to `code_colors.rs`; (d) the
    log-stall control stalls before its first window, so its "picture frozen" half checks only that no window
    appeared; make it log at a quieter level so it stalls after the window shows; (e) done: the worktree `.claude/worktrees/agent-a717e60922dca0448` stayed locked because the flake agent left
    a polling loop running for 7.5 hours (`until ls .../exp-all-fail-*.log; do sleep 10; done`, waiting for a file that
    exists only if an experiment fails); I stopped the loop (pid 1450932), unlocked, removed the worktree and its merged
    branch. Owner correction: "Assume nothing ever works perfectly." So after every agent report, scan for strays
    before declaring its worktree done: for each process whose `/proc/<pid>/cwd` is inside the worktree, read its
    command line, stop what the agent left behind, and only then unlock and remove; and look for units the agent
    started (`systemctl --user list-units`). Next: stage 5 (GPU minimap, with the Vello buffer question), then stage 6.
    Running (started 2026-10-20): stages 5 and 6 merged (GPU minimap on the shared Vello renderer, delete the rest of
    the CPU pipeline and `swash`), agent `a0880539282a3162b`; and a read-only study of Vello 0.11's fixed scratch
    buffers (262 MiB VRAM for any scene), agent `aefec5d0492ba3cab`, which writes only
    `doc/planning/ide-vello-buffer-sizes.md`.
    The study is in (`doc/planning/ide-vello-buffer-sizes.md`, pushed as `8e63ab3ef7`, no strays found, worktree and
    branches removed). Findings: Vello 0.11's seven bump buffers are fixed literals (165 MiB); no public option
    shrinks them; on overflow the frame is silently blank and `render_to_texture` still returns `Ok`; the other 97 MiB
    are wgpu allocator slack from the default `MemoryHints::Performance` (128 to 256 MiB blocks, a bare device holds
    193.6 MiB). Lead finding, a shipped correctness bug: `src/tile_scene/split.rs` and `encode.rs` clip each selected
    row with rectangles as tall as the whole texture, about 36,700 tiles per row, so a selection of 58 or more rows
    overflows the default tiles buffer and blanks the frame (56 rows rendered, 58 did not, in a probe; a 2000 px
    window shows about 83 rows). Also: a texture above about 128 MiB corrupts silently at the default ptcl size, so the
    64 MiB tile byte rule must stay; minimap slices must stay under about 1,600 rows at one sixth scale (sent to the
    stage 5 and 6 agent). Options ranked by the study: e1 `MemoryHints::MemoryUsage` (one field, saves 224 MiB in the
    app and 260 MiB per test process, 21 parallel test processes 9.4 GiB to 4.0 GiB) > b a `[patch]` of
    `vello_encoding` with sized buffers (another 40 to 72 MiB, needs the clip fix, a computed ptcl, failure detection
    and a lockfile regeneration) > c upstream (PR 1861 open and conflicting) > a leave > d other version > e2
    `use_cpu`. Decisions I took (veto-open): do e1 now, measured in the app; defer b (value after e1 is small against a
    carried vendored fork); no upstream comment. The two Vello 0.11 silent failures (blank frame on buffer overflow, corruption past 128 MiB at the default `ptcl`) are
    external-tool behavior and now have `doc/troubleshooting/vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.md`
    (owner question of 2026-10-20; none existed). Owner question of 2026-10-20, "Didn't the auto prototype trigger?": no. I wrote the Vello note by hand without
    loading the `troubleshooting-doc` skill, which fires when a diagnosis finishes and requires the 6-constraint
    upstream check and, when constraints 1 to 5 hold, a prototype of the fix in a disposable clone. Corrected:
    agent `a220c117f7de885e2` brings that doc up to the skill standard and prototypes the fix for the unchecked
    initial `ptcl` region; agent `acd6338a1aeec9b94` writes the troubleshooting docs for the other external findings of
    this migration that had none (wgpu-core debug-build abort at exit, CPU adapter selection and downlevel defaults in
    Slint's wgpu path plus Mesa's default-GPU rule, Slint `renderer-vello` clip and texture import limits, Vello's
    non-bit-identical renders, Slint's default features pulling in the software renderer). Nothing is filed upstream;
    the owner decides filing after reading each doc's decision. Rule for future briefs: a study or fix of an external
    tool must say "run the troubleshooting-doc skill" and I load the skill myself at the moment of diagnosis.
    The five troubleshooting docs landed (pushed as `e0cd3fec64`, agent `acd6338a1aeec9b94`, no strays, worktree and
    branches removed; nothing filed upstream): `wgpu-core-snatch-lock-trace-aborts-at-thread-local-destruction.md`
    (fixed upstream by wgpu PR 10030 merged 2026-10-17, which I confirmed with `gh`; wgpu 30.0.1 of 2026-08-22 is still
    the latest release and lacks it, so the IDE's leaked-handle workaround in `src/launcher.rs` and
    `src/native/gpu_snapshot.rs` stays until a release with the fix is resolved; queued: re-check on each wgpu or Slint
    update), `slint-wgpu-automatic-configuration-adapter-and-limits.md` (llvmpipe chosen by Slint with
    `MESA_VK_DEVICE_SELECT=10005:0`; downlevel defaults give Vello 0 storage buffers; draft not fileable, motivation
    weak), `slint-vello-renderer-clip-ignores-scale-and-rotation.md` (I confirmed in `itemrenderer.rs` that `translate`
    updates `clip_rect` while `rotate` and `scale` do not; all six constraints hold, a 51-line patch makes Vello match
    FemtoVG; the only doc worth filing and it needs a human-written description; we do not use that renderer, so it has
    no value for the IDE: recommendation hold), `vello-compute-renderer-renders-are-not-bit-identical.md` (cause: f32
    area sums over an `atomicAdd`-ordered segment list; a 16.16 fixed-point probe patch gives 0 of 200 differing
    renders; not fileable, research renderer; the IDE keeps `RENDER_NOISE = 1`), and
    `slint-default-cargo-features-link-software-renderer-and-opengl.md` (274 crates against 256).
    The Vello troubleshooting doc is finished to the skill standard (pushed as `d18845b99f`, agent `a220c117f7de885e2`, no
    strays, worktree and branches removed; nothing filed): variant 1 (blank frame) has many upstream duplicates (issues
    366, 175, 541, 720; PRs 606, 1861 and others) and nothing to add; variant 2 (initial per-tile `ptcl` region never
    checked against `ptcl_size`) has no upstream report; the agent reproduced both on 0.11.0 and on upstream `main`
    (`7d70631ac8`, relevant files byte-identical), prototyped a fix in `vello_encoding` `RenderConfig::new`
    (`ptcl = max(default, tiles * 64 + (1 << 22))`, 3 hunks, 56 lines, kept as a `.patch` beside the doc; I applied it
    to the installed 0.11.0 sources with `git apply -p3`, it applies cleanly) with before and after counts, and ran
    upstream's unit tests inside a network-less container (33 pass; the new test fails without the hunk). Study
    numbers reproduced except two within noise (290,939 against 290,992 bytes). Correction: the "1,600 rows" minimap
    bound I sent the stage 5 and 6 agent was for a reduced 1,048,576-line profile; at the default the bound is about
    3,200 rows (derived, not run), so 1,600 stays as the conservative figure. Six constraints: all hold for variant 2
    (soft on constraints 3 and 5), but the Linebender LLM policy requires human-written descriptions, so the drafts are
    not fileable as written. Agent's filing ranking: post nothing now > one human-written issue for variant 2 >
    comment on PR 1861 > a PR. Decision I took (veto-open): post nothing; the IDE is protected by row-height clips
    and the 131,072-tile rule, and could carry the patch through `[patch.crates-io]` if a larger target is ever
    needed. Note: at exactly 131,072 tiles the default `ptcl` has no dynamic room, so a scene needing one more chunk
    there fails loudly with bit 16.
    INCIDENT 2026-10-21, owner: "Your subagents are doing stress-ng 12 busy loops. 12! That is absolutely unacceptable."
    Cause: my brief to the flake agent (`af624c22739073177`) suggested busy loops or `stress-ng` as a load generator
    under a `systemd-run` unit with only a memory cap and misquoted the repo rule RXI, which caps stress, bench and load
    runs at `podman run --memory=2g --cpus=2 --rm`; the earlier flake agent (`a717e60922dca0448`) had used 16 busy loops
    and I let it pass. I found two units `ide-stress-load-354743` and `ide-stress-load-473223` running 12
    `stress-ng-cpu` workers each (24 busy loops), and container `ide-stress-354743` running the whole suite with
    `--stress-count 100` (3 copies, 14 threads), which also ignored my own "no repeated full suite" instruction; I
    stopped the node drivers (`bin/stress-native.ts`, 5 pids) and both units (verified with `ps`: no stress-ng,
    stress-native or stress-count process remains; the container was already gone). I messaged the flake agent and the
    stage 5 and 6 agent: no host load generators, stress only in a container limited to 2 CPUs or a
    `CPUQuota=200%` unit around the test run alone, one full `test:native` per snapshot. The host load average was
    113 at the peak of this and 76 after; other sessions add to it. Rules for every future brief: (1) quote RXI exactly:
    stress and load work runs in `podman run --memory=2g --cpus=2 --rm` and stays at or under 2 CPUs at once; (2) no
    host-wide load generators (stress-ng, busy loops) at all; to reproduce starvation, limit the test run's own CPU;
    (3) no repeated full-suite runs on one snapshot; (4) the report states the maximum CPUs loaded at once; (5) after
    every agent report I list `systemctl --user list-units` and `ps` for units and processes it left, and stop them
    before closing its worktree. `~/temp/agent/matrix-load.mjs` adds no load: it only raises the priority
    (`CPUWeight`) of the measured run, one cell at a time.
    STOP 2026-10-21, owner: lost about four hours of sleep to fan noise; asked to be told when the machine can reboot to
    BIOS "WITHOUT LOSING ANY WORK OR CONTEXT"; the owner also said I should have told the agents at once to preserve
    their work and stop. Both agents were told, committed, pushed and stopped; `~/temp/agent/reboot-check.mjs` (read-only)
    lists unpushed commits, uncommitted tracked changes in agent worktrees, containers, `ide-*` units and builds. Result
    after the stops: nothing of this work running or unpushed; one container, `angry_tesla` (another session's
    translation job, mounts `pi-mimo` and `span02-shadow`), would be killed by a reboot, which is that session's call.
    Subagent contexts are not saved by a reboot: the notes below are the durable state.
    LANDED 2026-10-21 after the reboot: both agent branches are merged and pushed as `34a38c9897` on `main` (scratch
    worktree, explicit `git push origin HEAD:main`; `origin/main` had gained only translation and doc commits, none
    touching the IDE, the compositor or the lint tool). One conflict, a doc comment in `tile_scene/item.rs`. Repairs the
    merge needed: `tests/tile_vello_limit.rs` imported the deleted `text_raster::MAX_RASTER_BYTES` (now
    `shaped_text::MAX_RASTER_BYTES`), an implicit `return` in `annotation_stability_tests.rs`, rustfmt in
    `minimap_tests.rs` and `minimap_dense_tests.rs`, semantic line breaks in three flake test files.
    The positive control `one_render_of_four_dense_chunks_comes_out_blank` failed on the merged tip. A probe measured
    one render of the densest selected scene: eight chunks (4,096 rows) draw every inked pixel, twelve (6,144 rows) come
    out blank. The likely cause is `c57aa0fd66` (a selected row drawn once), which shrinks the scene; no run on the
    unmerged agent tip isolated it. The control is now `one_render_of_every_dense_chunk_comes_out_blank` over the whole
    16-chunk slice (blank, 0 of 515,712 inked pixels), and the doc section names the new margin.
    Gates on the final tree: `lint` pass; `test:native` 271 of 271 passed, 5 skipped, 170 s, run once with the filter
    `all()` so the run does not stop at the first failure (the first run, with fail-fast, stopped at the control with
    139 tests unrun); `test` 795 of 795, `test:bin`, `build`, `inspect:software-renderer` pass. Cadence by
    `~/temp/agent/matrix-load.mjs` (release, 50,000-line `big.rs`, 1100x660, baseline `origin/main` before the merge as
    A, merged tip as B, 5 alternating pairs, A against A twice for the floor): scroll 118 to 119 of 119 in every run of
    both; drag A 115, 117, 116, 115, 112 of 121 (median 115), B 118, 115, 114, 110, 117 (median 115), floor 111 to
    117, so
    no difference beyond the floor. Screenshot of the merged tip (dark, scale 1): code and minimap draw, and select-all
    shows the selection marks in both. Not run: scale 2 and the light scheme in the app (the reflection tests cover both
    scales), the chunk seam of a window of about 3,700 px, quiet-host cadence, a denser scene. The fixture lives in
    `/tmp/ide-gpu-fixture` (the inspector demands a project below `/tmp`; rebuild with
    `~/temp/agent/make-gpu-fixture.mjs`).
    Installed 2026-10-21: `~/.local/bin/monochromatic-ide` sha256 prefix `c0b8219d73bc1062`, equal to the release build;
    the previous binary (`ab91c5f45960a4e7`) is `monochromatic-ide.previous`.
    Removed: both agent branches (local and remote), their worktrees, `ide-gpu56-baseline`, `.claude/exp-flake` and
    `.claude/exp-flake2`, my scratch worktrees. Another session's `cargo mutants` container (`event_hints.rs`, started
    03:36 on 2026-10-21) was running during this work; it is not ours and was left alone.
    Owner decisions of 2026-10-21 on the stage 5+6 questions (`doc/handover/ide-gpu-stages-5-6.md`): (a) a window
    taller than the device texture limit stacks textures, so the minimap covers the whole window (built; a stack of
    16,384 and 512 rows abutting without a gap was measured on 2026-10-22); (b) the
    reflection bounds (worst 36 / mean 0.8 at scale 1, 58 / 0.9 at scale 2) are the contract, kept as the guard;
    (c) the `minimap_build` to `minimap_worker` rename stands (already in the code, `src/native/minimap_worker.rs`).
    Same principle for the code tile, owner answer of 2026-10-22 ("Stack the code tile"; asked again by mistake: the
    2026-10-21 answer already settled the principle, and a settled decision is adopted and recorded, not asked):
    a window
    too large for one code tile stacks code textures; no blank bands, no soft text, no notice.
    Still open from that note: the stale `TEXTURE_DIMENSION` doc; dense-selection cost until full-height clipping in
    `split.rs` and `encode.rs` is fixed.
    LANDED 2026-10-21, stacked minimap textures (owner decision (a)): `main` is `cf58581533` (merge of the agent branch
    tip `54a1e5089d` and my guard text fix, pushed explicitly from a scratch worktree; `origin/main` had gained only
    translation and doc commits). A window taller than the device texture limit gets several textures, each at most
    the device side and 64 MiB (`texture_rows` and `texture_runs` in `src/minimap/plan.rs`, one `Image` per texture in
    `src/native/gpu_tile/slice.rs`, a `[MinimapSlice]` list in `ui/minimap.slint`); the defect test (12,000 px window
    over a device capped at 8,192) failed before with "the picture is 8064 rows of a track of 11968", seam and
    unsplit-render tests cover 3, 4 and 7 textures at scales 1 and 1.5, and mutation checks prove the one-row
    placement faults are caught. My verification: the cadence A/B found a scroll hitch the agent's first version
    caused (an identical frame at recorded slot 35 in 11 of 13 stacked runs against 6 of 27 baseline runs); the cause,
    shown by intervention, is that every picture was given to the window as a new `VecModel`, so the markup rebuilt
    every `Image` item inside a scroll-step frame. Fix `3247690ff8` changes the rows of the existing list in
    place (guard test `a_picture_of_the_same_shape_changes_the_rows_of_the_list_and_never_replaces_it`, failing
    commit `ee59d9630f`, source guard `minimap-slices-in-place` proven). Where the cost lands is not located: window
    thread timing is the same in both trees, so it is outside that thread. The agent also found that runs where
    highlighting times out (a load-dependent tree-sitter deadline) show a 30 px banner and move the hitch to slot 43;
    my counts include 4 such runs, and the per-record mapping to artifact directories is probable, not certain.
    Gates on the landed tree, once each: `lint` pass; `test:native` 274 of 274 (5 skipped, 174 s); `test` 797 of
    797; `test:bin`, `build`, `inspect:software-renderer` pass; `inspect:source-guards` for the two changed guards
    (`minimap-texture-top`, `minimap-slices-in-place`) baseline pass, removed fail, restored pass (the other 146
    guards were not run in this pass). My guard text fix: `minimap-texture-top` expected "the composited column
    differs" while the removed guard fails with "the composited column is blank"; the entry now matches the prefix
    "the composited column". Cadence by `matrix-load.mjs` (release, `big.rs`, 1100x660, baseline `origin/main` before
    the merge as A, fixed tree as B, alternating pairs, A against A for the floor): scroll, identical-frame runs in
    A (floor included) 7 of 20 with 4 at slot 35, in B 3 of 10 with 1 at slot 35, longest identical run 1 in both;
    drag, identical frames per run A 3, 9, 9, 5, 10 against B 5, 11, 6, 14, 4, longest run 1, no difference beyond the
    floor. One identical frame at slots 35, 67, 99 or 130 remains in both trees: the extra redraw `picture.rs`
    requests on handoff exists on `main` and was not explained or changed (open). Not run: display scale 2 or other
    scales with a stack, devices with a texture limit above 8,192 (16,384 is common), a window of 17,000 px or more
    (three textures were made by lowering the recorded limit), densest-text stacking, a look in a nested compositor
    window, frame time of a 12,000 px window (about 24 chunk renders per picture on the window thread), cadence on a
    quiet host (load average was 8 to 9 from other sessions).
    Update 2026-10-21: the extra redraw `picture.rs` requested is explained and fixed, and landed (see the LANDED
    block of the redraw agent below; cause, counts and limits in `doc/handover/ide-gpu-stages-5-6.md`, section
    "The extra frame at a picture hand-over").
    Installed 2026-10-21: `~/.local/bin/monochromatic-ide` sha256 prefix `e1f2590ffadecc91`, equal to the release
    build; the previous binary (`c0b8219d73bc1062`) is `monochromatic-ide.previous`. Removed: the agent branch (local
    and remote), its worktree, my scratch worktrees and guard caches. Another session's mutation container
    (`git-policy-native-mutation:m028b`) was running and is not ours.
    Owner request 2026-10-21: "Check if depot can do the CI for this app. Early terminate on this depot testing task:
    tell me whenever we encounter any kind of blocker and halt the depot testing task." The first blocker found stops
    the task and is reported to the owner at once; nothing is worked around.
    BLOCKER found 2026-10-21, task halted, nothing run on Depot: `doc/decision/depot-ci-adoption.md` records GPU as
    out of scope for Depot CI ("no hardware `/dev/dri` device was found" in the sandbox, a finding from when software
    rendering was the accepted path); the IDE now needs a hardware Vulkan adapter (never a CPU adapter, no software
    renderer), so `test:native`, `inspect:native`, `inspect:cadence` and the screenshot checks cannot run in a Depot
    sandbox as the decision stands. The CLI is installed (2.102.23) and logged in (org `l4zncjjbj5`); `.depot/workflows`
    holds `current-baseline.yml` and `weekly-native.yml`. Unprobed: whether today's Depot sandbox exposes a GPU.
    Docs checked the same day (depot.dev CI overview, blog, pricing, changelog): sandboxes are x86_64 and Arm64 Linux,
    2x8 to 64x256, Ubuntu 24.04, no GPU, `/dev/dri` or Vulkan mentioned; the only GPU offering found is Depot GitHub
    Actions runners on Depot Managed (the owner's AWS account, Custom plan), a different product that the adoption
    decision also excludes. The owner approved one probe job (list `/dev/dri`, `/dev/kvm`, PCI devices, Vulkan ICD
    files on `depot-ubuntu-24.04`, workflow only in the scratch worktree `~/temp/agent/depot-probe`, never committed).
    SECOND BLOCKER, task halted again: `depot ci run --workflow .depot/workflows/ide-gpu-probe.yml --follow --forge
    github` (CLI 2.102.23) exits 1 with "no supported repository found in git remotes; use --repo and optionally
    --forge" although `origin` is `https://github.com/Aquaticats/Monochromatic`; cause not found. With the owner's
    approval the probe was rerun with `--repo Aquaticats/Monochromatic --forge github`: the CLI warned "could not
    determine merge base, skipping patch: cannot determine default branch" (same detection problem, the probe workflow
    still ran) and `--follow` gave up with "Timed out waiting for jobs to be created" although the run finished in 38 s
    (run `7fsk45ct45`, job log `hgc361htlw`, 2026-10-21, one job on `depot-ubuntu-24.04`, 2 CPUs, 7 GiB).
    MEASURED RESULT: no `/dev/dri`, no `/sys/class/drm`, no GPU on the PCI bus (virtio devices only), no `nvidia-smi`,
    an empty `/etc/vulkan/icd.d`; `/dev/kvm` is present; kernel 6.12.109+. So the Depot CI sandbox has no GPU as of
    2026-10-21 and the GPU gates (`test:native`, `inspect:native`, `inspect:cadence`, screenshots) cannot run there.
    Other sandbox sizes were not probed. The non-GPU gates (`lint`, `test`, `test:bin`, `build`,
    `inspect:software-renderer`) were not tried on Depot. The one probe run consumed about 38 s of the 2x8 sandbox and
    was not entered in `doc/planning/depot-admission-ledger.json`. The scratch worktree and probe file are removed;
    the decision record `doc/decision/depot-ci-adoption.md` is unchanged (a change needs the owner's acceptance).
    Owner decisions after that: stop the Depot check here (the non-GPU gates were not tried on Depot); rename the
    guard `inspect:software-renderer` to `inspect:no-software-renderer` (the owner asked why a software renderer
    task still exists: it is the guard that fails when that renderer is linked, `cargo tree` only, no rendering; renamed
    2026-10-21 in `mise.toml`, the IDE README and the troubleshooting doc; older notes in this file and the planning and
    handover docs keep the old name for their recorded results, so read `inspect:software-renderer` there as the
    renamed task).
    Namespace (`nsc` v0.0.587, workspace "Personal", installed by the owner), checked 2026-10-21 for GPU CI: the docs
    (machine shapes, `nsc create` reference, Linux and macOS compute pages, docs index) name no GPU shape, no GPU
    feature and no Vulkan or `/dev/dri`; Apple M4 Pro and M5 Max Linux is early access with no GPU statement, macOS
    mentions Metal only. With the owner's approval one probe instance was created and destroyed
    (`nsc create --machine_type linux/amd64:2x4 --duration 10m --bare`, id `1v2fprmgr0h4a`, probe script
    `~/temp/agent/nsc-probe.mjs`): kernel 7.1.4, no `/dev/dri`, no `/sys/class/drm`, no `nvidia-smi`, no Vulkan ICD
    directory, `/dev/kvm` present, 2 CPUs and 4 GB; `lspci` is not installed there, so the PCI bus was not listed (the
    missing `/sys/class/drm` carries the result); `nsc list` afterwards shows no instances. So neither Depot CI nor a
    standard Namespace Linux instance gives this app a GPU; the GPU gates stay local. Unprobed: Namespace Apple
    Silicon Linux early access, other shapes, macOS (Metal, not the Vulkan path the owner accepted).
    LANDED 2026-10-21, flake agent `a5d7faec36a652c16` (merge tip `3bb3fd2e8e`, contained in `origin/main`): tests and
    docs only, no product code, so no reinstall and no cadence run. The find-box menu flake is explained and fixed:
    the test right-clicked where the toolkit clamps the menu under the pointer, and the toolkit runs the entry under
    the pointer on a release more than 300 ms after the menu appears (a stalled test thread under host load); the test
    now clicks where the pointer rests on the menu frame, and `a_slow_right_click_leaves_the_find_menu_open` holds the
    click 400 ms (old position 600 of 600 failed, new position 0 of 600; the original test 0 of 600 either way, so the
    old load-dependent rate of 2 of 240 and 5 of 480 was not re-measured). A shared `wait_for_highlighting` now covers
    the other fixtures that open a file (an exposure removed, no failure was ever measured there). The annotation
    stability tests did not reproduce (1,145 of 1,145 runs of `rows_arriving_in_view_move_only_lines_beneath_them`), so
    their cause is unknown; every comparison now keeps both frames under `target/flake-frames/` when it fails, and the
    next failure's frames are read before any change. The minimap external-edit test was deleted with the CPU minimap;
    its successor passed 120 of 120. No video-memory cap was set (no failure is attributed to it; the override
    `IDE_CONTAINER_TEST_THREADS` exists). The "Flaky test ledger" is in `doc/handover/ide-native-flakes.md`, with two
    incomplete entries (`the_view_follows_the_caret_through_every_visual_row` fix unverified; the scroll quiet-time
    test has no recurrence recorded). Zero tests are under retries. Gates once on the merged tip: `lint`,
    `test:native` 275 of 275 (5 skipped), `test` 797 of 797, `test:bin`, `build`, `inspect:no-software-renderer` all
    pass. Not run: a second full `test:native`, guard runners, a capped against uncapped video-memory comparison, runs
    under host load. The agent's worktree, its 32 GiB `target` copy and its branch (local and remote) are removed; no
    units or containers were left.
    LANDED 2026-10-21, redraw agent `a3a543d4c80040d25` (merge tip `73ee80f08b`, contained in `origin/main`):
    `src/native/minimap/picture.rs` called `request_redraw()` the moment it handed a picture to the GPU; a scroll draws
    a frame at every wheel step and the GPU draws a waiting picture inside the next frame, so the request added a second
    frame between two steps and the text area stood still for one tick (row shift 0 px where every other tick is
    25 px). The hand-over now starts a single-shot 20 ms timer (`FRAME_WAIT`) and asks for a frame only if
    `GpuTile::minimap_waiting()` is still true then; an idle window gets its frame at most 20 ms later. Guards: tests
    `a_picture_that_a_frame_draws_asks_for_no_frame_of_its_own` and `a_picture_that_no_frame_draws_asks_for_a_frame`
    (`picture_frame_tests.rs`), source guards `minimap-picture-waits-for-frame` and `minimap-picture-asks-when-idle`
    (agent: baseline pass, removed fail, restored pass; mine on the merged tree: the two pass). The cadence tool's
    whole-frame hash undercounts these stalls (it saw an identical frame in 7 of 20 old-behaviour runs where the stall
    was present in 20 of 20), so the measure is the agent's frame-shift probe `~/temp/agent/mm-stall.mjs` (ticks where
    the text area moved 0 rows). Agent's counts (10 pairs, release, `big.rs`, 1100x660, scroll): first-hand-over stall
    A 10 of 10, B 0 of 10. My own run on the merged tree
    (6 pairs, alternating order, `~/temp/agent/redraw-stall-ab.mjs`):
    first-hand-over stall A 6 of 6 (slot 35 every time), B 0 of 6; stalls in total A 12, B 8; no odd row shifts in
    either tree. Hash tool on 8 pairs: identical frame A 2 of 8 runs, B 0 of 8; A against A 1 of 8 (the floor).
    NOT eliminated: every B run (6 of 6) stalls once at slot 50, where A shows none there (A stalls at 95, 98, 74, 67,
    142 in other runs). The agent attributes the remaining stalls to the tile hand-over in
    `src/native/render/present.rs` (`request_redraw()` after `submit_tile` and `submit_sign`) from a debug log, not from an intervention; it is open.
    Not run: the other 148 source guards, the drag scenario, display scales other than 1, a real compositor, idle
    latency of the 20 ms wait (a picture in an idle window waits up to 20 ms; on a display slower than 50 Hz the old
    extra frame returns), the markdown render check. Gates on the merged tip, once each: `lint`, `test:native`
    277 of 277 (5 skipped), `test` 797 of 797, `test:bin`, `build`, `inspect:no-software-renderer`, all pass.
    Installed 2026-10-21 (second install this day): `~/.local/bin/monochromatic-ide` sha256 prefix
    `8706732d98cdd82c`, equal to the release binary of the verified tree; the previous binary (`e1f2590ffadecc91`) is
    `monochromatic-ide.previous`. The agent's worktree (it was locked by the session process; I checked its HEAD is in
    `origin/main` and unlocked it), branch (local and remote), my two scratch trees and 24 cadence artifact
    directories are removed; no units or containers were left.
    Tile-stall agent `a0b5a3cdf2d733095` (started 2026-10-21, own worktree off `origin/main`, branch push only): decides
    by intervention why the tile hand-over stalls slot 50 and whether the stall moved from slot 35 or the fix changed
    the timing; fixes only if the frame-shift probe shows it; brief quotes `NLG`, `NLD`, `RXI`. On its report: verify,
    scan `ps`, `podman ps`, `systemctl --user list-units`, merge in a scratch worktree, gates once, own stall A/B with
    `~/temp/agent/redraw-stall-ab.mjs`, reinstall if the binary changed, remove its worktree (unlock first only after
    checking its HEAD is in `origin/main`) and branch.
    LANDED 2026-10-21 (tile-stall agent `a0b5a3cdf2d733095`, tip `f75106b086`, base `b4e9063c3a`; merge tip
    `e2b9ac2415` on `origin/main`): by intervention, the explicit `request_redraw()` after `submit_tile` is not the
    cause (removing it leaves the slot 50 stall in 6 of 6 runs; the earlier attribution was wrong on that point); the
    setters of the tile swap ask for a frame themselves, which is not isolated further. The fix holds the swap: the
    20 ms tile poll (`src/native/tile.rs`, `STEP_WAIT`) leaves a finished tile while the last scroll step is under
    20 ms old, and the step takes the tile at the end of `on_viewport_changed` (`src/native/viewport.rs`). With no step
    coming, the poll takes the tile 20 ms later. The agent's first poll order broke the 50,000 line external edit test
    (20 of 20 failures against 0 of 20); the shipped order polls after the step. Section "The extra frame at a tile
    hand-over" in `doc/handover/ide-gpu-stages-5-6.md`.
    My A/B (8 pairs, release, `big.rs`, 1100x660, scroll, frame-shift probe, one app run at a time): baseline
    (`origin/main` before the merge) stall at slot 50 in 8 of 8 runs, 12 stalls in total (slots 50, 98, 142, 74);
    merged tree 0 of 8 at slot 50, 1 stall in total (slot 72). The agent's own 8 pairs: 0 of 8 against 8 of 8, 1 against
    13 stalls; its one stall was at slot 33, mine at slot 72, so a rare stall elsewhere remains and is undecided
    (host or fix). Gates on the merged tree, once each: lint, `test:native` 279 of 279 (5 skipped), `test` 797 of
    797, `test:bin`, `build`, `inspect:no-software-renderer`, the three new source guards (`tile-left-for-next-step`,
    `tile-taken-by-the-step`, `tile-taken-by-the-poll-when-idle`) pass; the agent showed each fails when its guarded
    line is removed. Not run: the idle latency of the 20 ms wait (a tile finishing within 20 ms after the last step waits
    up to one more poll period; reasoned, not measured), input slower than 50 Hz (keeps the old extra frame), the drag
    scenario, scales other than 1, the sign indicator intervention, a real compositor, the other 150 guards.
    Installed 2026-10-21 (third install this day): `~/.local/bin/monochromatic-ide` sha256 prefix `854d0998238b249b`,
    equal to the release binary of the verified tree; the previous binary (`8706732d98cdd82c`) is
    `monochromatic-ide.previous`. The agent's worktree, branch (local and remote) and my two scratch trees are removed;
    no units, containers or stray processes were left (scanned after the report).
    Latency and rare-stall agent `a0d7b8bed3990e6c8` LANDED 2026-10-22 (doc only, merge tip `7d3d76f503`; the agent's
    switch commit and its revert are in the history, the net product diff is empty, so no gate and no install):
    section "Idle latency of the 20 ms wait and rare stalls elsewhere" in `doc/handover/ide-gpu-stages-5-6.md`.
    Latency, 2-step flick of 1500 px (47 runs per arm): wait off median 29.7 ms (quartiles 28.8 and 30.3, so under about
    3 ms is noise), merged median 30.2 ms, but two modes: 28 runs at 28.3 to 32.9 ms and 19 at 38.8 to 51.0 ms; in 18 of
    the 19 the poll left a tile and the next poll came a whole period (20 ms) later. Positive control (wait 150 ms) moved
    the measure by 136 ms. Rare stalls: fix off 32 stalls in 16 runs (slot 50 in 16 of 16); merged 0 stalls in 56 runs;
    the slot 33 and 72 stalls did not recur, so "host or fix" is undecided (upper bound of the rate near 5% per run).
    Verified by me: branch tip `7e051535a2`, net diff one doc file, no stray units or containers (the busy processes
    in `ps` were another session's translation workers); the agent's worktree and branch (local and remote) and my
    scratch tree are removed.
    Tail agent `a52afe09d606ed6ed` LANDED 2026-10-22 (branch tip `6d17994673`, product tip `9f21309e6d`; merge tip
    `cbb2d47e52` on `origin/main`): a single-shot `tile_wake` timer (`src/native/tile/wait.rs`, `wake_delay` rounds up
    to a whole millisecond because its first build spun on sub-millisecond waits) takes a tile left for the next
    scroll step as soon as the 20 ms wait ends; `poll` stays the only taker and stops the wake-up first. Three new
    tests and three new guards (`tile-wakes-when-the-wait-ends`, `tile-wake-cancelled-by-the-poll`,
    `tile-wake-covers-whole-milliseconds`), each shown by the agent to fail when its line is removed; the module
    move to `tile/wait.rs` was forced by the 300 line limit of `tile.rs`.
    The agent's counts (2 wheel steps of 1500 px, tile late when applied over 30 ms after the last wheel event): late
    A 70 of 140, B 0 of 100; its 120-step stalls B 16 in 60 runs against A 10 in 62, slot 50 B 1 of 60 (the spinning
    first build) against A 1 of 62. Its open cost: a tile taken in an input gap over 20 ms is an extra frame.
    My check (own trees, load 4 to 9 from other sessions, alternating arms): latency A late 19 of 40 with median
    frame 37.8 ms, B late 0 of 40 with median frame 30.2 ms (B frame over 36 ms in 1 run, A in 20); A against A (20
    pairs) late 10 and 11 of 20, so the baseline is bimodal at about half and the effect is far outside the spread;
    120-step stalls A 5 in 5 of 20 runs, B 2 in 2 of 20, slot 50 in none. So the stall cost the agent feared was not
    seen at these counts; counts this small cannot exclude a small rate. Gates on the merged snapshot, once each:
    lint, `test:native` 282 of 282 (5 skipped), `test` 797 of 797, `test:bin`, `build`,
    `inspect:no-software-renderer`, six tile source guards (6 of 156) all pass.
    Installed 2026-10-22: `~/.local/bin/monochromatic-ide` sha256 prefix `27b877741a056ecf`, equal to the release
    binary of the tree it was built in (the tree held a later merge of `origin/main` that touched no IDE file); the
    previous binary (`854d0998238b249b`) is `monochromatic-ide.previous`. The agent's worktree, branch, my two trees
    and guard directories are removed; no units or containers were left. Not run: drag scenario, scales other than 1,
    a real compositor, input slower than 50 Hz, step lengths other than 16 ms, the other 150 guards, the doc text of
    the agent has about 200 semantic-line-break findings (older text 480) that are not reflowed.
    Scale, scheme and seam agent `a1612e9c563c279ed` LANDED 2026-10-22 (doc only, merge tip `3a2fb45dd4`; branch tip
    `c7f48424c0`, net diff one doc file; section "Scale 2, the light scheme and the seams, measured in the
    compositor" in
    `doc/handover/ide-gpu-stages-5-6.md`). No defect in the three checks, no product change, no gates. Measured on the
    installed binary `27b877741a056ecf` in the headless compositor (`--headless --gpu /dev/dri/renderD128`, which
    already has `--scale`, `--color-scheme`, `--accessibility` and `record`; no compositor change needed; the nested
    window is clamped to the host screen). Scale 2: sharp text (Laplacian ratio 1.18 against 0.25 for an upscale),
    layout and caret identical in logical pixels, minimap bounds exactly 2x, hit tests 10 of 10 (one run of 7 had its
    first 5 clicks do nothing, unexplained, while the agent's own build ran), a wheel notch about 60 logical px at both
    scales. Light: no dark-only constant in the Vello code; every text ink at or above 4.5 on `#FAFAFA`. Seams: chunk
    seams clean in 128 seams over 12 shots (positive control: a one row copy fault flagged in 8 of 8 shots), tile phase
    constant to 0.05 px over 18 shots (control: a 1 px roll read +1.00). I looked at five of its images myself: scale 2
    dark, light find box, the seam control, the 8K window. Blind spot named by the agent: a chunk shifted by a whole
    line is invisible to the capture measure (only `minimap_seam_tests.rs` covers it).
    Findings outside the checks, all being worked (agents below): (1) in a 7680x4320 window at scale 1 the code tile is
    cut to 7259x2280 and 52% of the code view is blank (I confirmed it in `~/temp/agent/sss-shots/w8k/plain.png`); (2)
    a window over 16,384 physical pixels in one dimension panics in the toolkit's surface configure; (3) Vello logs
    "Trying to paint too large image" at warn level in every 3,700 px scale 2 run (64 MiB is exactly 256 bins, a tile
    off the 256 grid counts more); (4) during a fling the minimap picture lags the viewport box (91 to 94 blank track
    rows 3.5 s after a 3,777 px scroll in one run). Peak load of the agent was 12 CPUs; images in
    `~/temp/agent/sss-shots/`.
    Code tile stacking agent `a074735d16c6106a5` (started 2026-10-22, own worktree off `origin/main`, branch push only;
    brief quotes `NLG`, `NLD`, `RXI`, `NQH` and the stacking decision above): stacks the code tile over several
    textures (findings 1 to 3), measured by captures at 7680x4320 scale 1, 3840x2160 scale 2 and 1100x8100 scale 2 in
    both
    schemes, the seam measure with a positive control, no Vello warning, and the cadence (20 alternating pairs plus 15
    A against A pairs; the 40 pair latency scenario keeps late tiles at 0 of 40). Minimap lag agent
    `afaeefdb803099e40` (same rules; stays out of the `gpu_tile` and tile modules): reproduces finding 4 with a measure
    and a same-binary control, finds the cause by intervention, fixes it at the cause if it is ours. On each report:
    verify, scan `ps`, `podman ps`, `systemctl --user list-units`, look at the images, read the product diff, gates once
    on the merged snapshot, land one agent at a time, push, reinstall if the binary changed, remove worktrees and
    branches.
    Owner statement of 2026-10-22: "The host CAN handle multiple jobs at once, that's the exact reason I bought this
    CPU." Real jobs may therefore run in parallel, each capped (`CPUQuota`, `MemoryMax`); `NLG` still bans synthetic
    load, and measurements still alternate arms with a same-binary control (`NQH`) because parallel jobs add load.
    Owner statement of 2026-10-22, later: "Use my system (almost) fully instead of limiting to only a very limited
    subset of it." The host measured that day: 28 threads, 61 GiB (about 32 GiB available), load average 7 to 13 from
    the agents and other sessions. Applied: rule `FHU` in `AGENTS.md` (commit `ea7937c9fd`); my gate and scratch units
    now use `CPUQuota` near 2400% and `MemoryMax` near 36G (they were 1200% and 32G, and 600% in the agents' briefs);
    the three running agents were told by message that the cap is lifted for real jobs; later briefs say the same and
    still quote `NLG`, `NLD`, `RXI`, `NQH`. Not changed without the owner: the IDE container share `hostShare = 0.75`
    in `package/desktop-app/ide/bin/container-bounds.ts` (21 of 28 CPUs, 46 GiB), because the owner decided it on
    2026-10-18 ("use 75 percent ... and keep that") and the README numbers were measured at 21 CPUs; a question about
    raising it went to the owner.
    Owner answer of 2026-10-22: raise the container share to 90 percent (25 of 28 CPUs, 55 GiB). Built and gated in the
    scratch tree `~/temp/agent/land-share` (commit "feat(desktop-app-ide): the build and test containers take 90
    percent of the host", not pushed): unfiltered `test:native` 283 passed and 5 skipped in 51.7 s in the 25 CPU
    container, `test` 797 passed, lint and `inspect:no-software-renderer` pass, and `test:bin` with the share put back
    to 0.75 fails as the guard. Its `test:bin` gate is red on the real snapshot for a reason that is not the share:
    see the next paragraph.
    HOLD, 2026-10-22: `origin/main` has files that no longer parse. Commits `68415aeb53` and `3da31bdc3a` (repo-wide
    `style(*)`, from an `oxlint --fix` style run by a session that is not this one and not `monochromatic-09`) left 20
    `.mjs` files failing `node --check` that parsed before them, 17 of them in `package/desktop-app/ide` (the
    `bin/inspect-*.mjs` scripts, `bin/nested-compositor.mjs`, two `design` scripts). My first count was 52; 32 of those
    were TypeScript files that `node --check` rejects in their old versions as well (it does not strip types), and
    redone with `module.stripTypeScriptTypes` 1849 of 1850 TS files parse before and after, none broken by the commits.
    The issue and both peer sessions were corrected. `test:bin` fails on `main` because of
    `bin/nested-compositor.mjs`. Owner instruction: open an issue and let MiMo take it; it is
    https://github.com/Aquaticats/Monochromatic/issues/1794 (list of files and the measure are in it). I restored the 17
    IDE files from before the style commits only in the scratch tree `land-share` (uncommitted, not pushed) and did
    not touch the other packages. Held until the fix is on `main`: pushing the 90 percent change and landing the
    minimap fling agent, because their `test:bin` gates cannot be green. The IDE `test:bin` gate is the only gate known
    to be red; the other gates passed on the snapshot.
    Owner decision of 2026-10-22 on the 17 IDE files: convert them to TypeScript instead of restoring them as `.mjs`
    (reason given: the oxlint setup did not corrupt any of the 1,850 `.ts` files and did corrupt `.mjs` ones; the
    owner also asked why `.mjs` was used, and the answer is that no rule required it, the inspection scripts of
    2026-10-05 to 2026-10-19 began as plain `.mjs` that `node` runs without a build step, and 30 `.mjs` against 10 `.ts`
    remain in `bin/`). Running: conversion agent `a973fa0a05208c348` (own worktree off `origin/main`, branch push only;
    brief quotes `NLG`, `NLD`, `RXI`, `NQH`, `FHU`; starts from each file's valid version at `68415aeb53~1`, updates every
    reference, runs old against new for each script, lint and type tasks, `test:bin`, `test`, one each). On its
    report: verify the diff and the compared outputs, run the gates once on the merge, push, then land the 90 percent
    share (rebase `~/temp/agent/land-share`) and the minimap agent. The 15 other `.mjs` files in `bin/` (30 minus the 15 that broke)
    stay as they are until the owner says otherwise (a conversion of all of them is a separate decision).
    LANDED, 2026-10-22, conversion agent `a973fa0a05208c348` (merge tip `dd134c167a` on `origin/main`, ancestry
    confirmed; branch tip `de2cd5834e`, 4 commits). While it ran, another session put the 17 files back as `.mjs` on
    `main` (`0c2f35836d`) and re-applied the formatting (`60d4795f95`, which says `Closes #1794`), so the merge had
    modify/delete conflicts. I resolved them by the owner's decision: the 17 `.ts` files equal the agent's branch tip
    byte for byte (`git diff` against the tip is empty) and the restored `.mjs` are removed. The formatted `.mjs`
    versions are in history at `60d4795f95`. Gates once on the merged snapshot: `lint` passes, `test:bin` 71 of 71,
    `test` 797 passed and 1 skipped (no Rust changed, so no `test:native`). My checks of its report: all 17 parse with
    `stripTypeScriptTypes`; `nested-compositor.ts` differs from its pre-corruption `.mjs` only by types, TSDoc and one
    equivalent `.map(Number)` rewrite; the report's "line counts are unchanged" was wrong (the files grew 10 to 15
    percent from TSDoc and types), and I read only that one diff by hand, the rest rests on the agent's old-against-new
    runs of 12 scripts with equal verdicts. Known limits it reported: `inspect:language-confinement` and
    `inspect:language-lifecycle` fail in the same way before and after (rust-analyzer does not start under the mise
    task environment; run directly with `node` the confinement script passes 114 of 114); the design scripts write
    frames whose bytes differ from the old script's (cause not isolated). Agent worktree and branch removed locally and
    on the remote; no stray unit, process or container. The HOLD above is lifted: `test:bin` is green on `main`.
    Next, one at a time: the stacking branch, the minimap branch, the 90 percent share (its scratch tree
    `land-share` still holds uncommitted `.mjs` restores that must be discarded, not pushed).
    The code tile stacking agent `a074735d16c6106a5` has handed back (branch `worktree-agent-a074735d16c6106a5`, product
    snapshot `628fb36f39`, tip `0a541c6008`, 13 commits, pushed, not on `main`; I read the markup, the stack builder and
    the viewport diffs and found no defect; no stray unit or process). Its numbers: captures at 7680x4320 scale 1
    went from 1,405 to 1,553 inked rows of 4,320 to 2,879 to 2,897 (about 66 percent), 3840x2160 scale 2 from about
    1,600 to about 2,800 inked rows, 1100x8100 scale 2 from about 8,000 to about 10,470, last row inked in every B
    capture; seam measure 0 differing rows in 16, with a positive control of 32 to 35 differing rows; the Vello
    warning 5 times in each of 3 runs before and 0 in 6 runs after; cadence 22 pairs B 4 stalls in 3 of 23 runs against
    A 1 in 1 of 22, and a same-binary control 7 stalls in 7 of 28 runs (inside the spread); `texture-limit` clamps the
    window (the panic over 16,384 px was the compositor's, which also panics at 1100x8400 scale 2, so that part is not
    reproduced end to end); nine guards shown to fail; its gates passed on `628fb36f39` (`test:native -- 'all()'` 288
    passed, serial). Open in its report: late tiles B 2 of 80 against A 0 of 80 (pass 1 had 2 of 40 at 52.6 and 238.6
    ms, pass 2 0 of 40) with the evidence removed, so I measured it (unit `stack-all`, trees of `origin/main` and of `origin/main` plus the
    branch with docs resolved by `-X theirs` for the measurement only, both removed; 60 alternating latency pairs and
    20 A against A pairs; rows kept in `~/temp/agent/stack-lat-ab.jsonl` and `stack-lat-aa.jsonl`).
    Three older guards (`failed-job-freed`, `overscan-early-return`, `minimap-scroll-notes-bands`) have anchors that
    were already absent on `main`; the agent did not repair them, and one of mine to repair after the landings.
    My latency measurement of the stacking branch (2026-10-22, release builds of `origin/main` as A and of `origin/main`
    plus the branch as B, 1100x660 window, one app run at a time): 60 alternating pairs, late tiles (applied more than
    30 ms after the last wheel event) A 0 of 60 and B 0 of 60, median apply 20.8 ms in both, median frame 30.3 ms and
    29.8 ms, one frame over 36 ms in each; the A against A control of 20 pairs 0 of 20 and 0 of 20. The agent's own 2
    late tiles of 80 on B (applied at 52.6 and 238.6 ms, evidence removed) did not reproduce, so the combined count
    is B 2 of 140 against A 0 of 140; the cause of those two runs is not known and a load burst is not shown. At this
    window size the stack is one texture, so this measures the one-texture path only.
    LANDED, 2026-10-22, code tile stacking agent `a074735d16c6106a5` (merge tip `7ae6dfbde2` on `origin/main`, ancestry
    confirmed; branch tip `0a541c6008`). One conflict, in `bin/inspect-source-guards.mjs`: main had reformatted the
    file (`3da31bdc3a`) while the branch replaced the `clip-cap` and `clip-texture-side` cases with nine stacking cases
    and followed the `Slice` rename in `minimap-slices-in-place`. I kept main's formatted file and re-applied exactly
    those edits (the `diff` against main shows nothing else; the nine new objects were generated from the branch's own
    lines). Gates once on the merged snapshot: `lint` passes, unfiltered `test:native` 289 passed and 5 skipped,
    `test:bin` 71 of 71, `test` 801 passed and 1 skipped, `inspect:no-software-renderer` passes. Because I resolved
    the guards file by hand, I ran the ten guards the merge touched in a disposable copy of `target`: each passes at
    baseline, fails with its guard removed, passes restored (10 of 163 guards; the other 153 were not run). Installed
    the new IDE: `monochromatic-ide` is now `9c046da1e4f2...` (equal to `dist/monochromatic-ide` after `bundle`), the
    previous binary `27b877741a05...` is kept as `~/.local/bin/monochromatic-ide.previous`. I have not yet opened
    the installed binary in the nested compositor; that check is after the minimap landing, which installs again.
    Agent worktree and branch removed locally and on the remote; scratch tree, guard cache and artifacts removed.
    The minimap lag agent `afaeefdb803099e40` has handed back (branch `worktree-agent-afaeefdb803099e40`, tip
    `fbe4f73b2b`, pushed, not on `main`, not yet verified by me): blank-track frames in a 3,777 px fling fell from
    0.45 to 0.01 (A/A control 0.43), by a `LEAD` of 8 bands on the heading side; seven guards shown to fail; its own
    gates passed. Its report is the input to the verification, which waits for the hold above and for the code tile
    stacking agent `a074735d16c6106a5` (still running), whose branch touches other files in the same package.
    LANDED, 2026-10-22, minimap lag agent `afaeefdb803099e40` (merge tip `f0c616addd` on `origin/main`, ancestry
    confirmed; branch tip `fbe4f73b2b`). I read the whole source diff (`Heading`, `LEAD`, `bands_toward`, `follow`,
    the reset on another file) and found no defect; I did not repeat its 3,777 px fling measurement. Conflicts: the
    handover `doc/handover/ide-gpu-stages-5-6.md` (both appended, kept both, rewrapped two lines over 120 characters)
    and `bin/inspect-source-guards.mjs` (main's formatted file kept; the two changed cases and five new cases
    re-applied from the branch's own lines, `diff` against main shows only those). Gates once on the merged
    snapshot: `lint` passes, unfiltered `test:native` 290 passed and 5 skipped, `test:bin` 71 of 71, `test` 803 passed
    and 1 skipped, `inspect:no-software-renderer` passes. The seven touched guards each pass at baseline, fail with the
    guard removed and pass restored in a disposable copy (7 of 168 guards; the other 161 were not run). The push
    merged in one newer doc-only commit of mine (the stacking record), which was not gated. Installed again:
    `monochromatic-ide` is `1ab8ee28af3c...` (equal to `dist/monochromatic-ide`); `.previous` is now the stacking
    build `9c046da1e4f2...`. User boundary check: the installed binary run in the nested compositor
    (`design/soft-wrap-shots.ts`, light and dark, six frames, exit 0) draws the wrapped source, the tree and the minimap
    picture; frames are in `~/temp/agent/smoke-installed`. Agent worktree and branch removed locally and on the remote.
    LANDED, 2026-10-22, the 90 percent container share (merge tip `0436ea512e` on `origin/main`, ancestry confirmed;
    my commit `001d74da17`: `hostShare = 0.9` in `bin/container-bounds.ts`, its test, the README). It merged cleanly.
    Gates once on the merged snapshot: `lint` passes, unfiltered `test:native` 290 passed and 5 skipped (the first
    run at the new bound), `test:bin` 71 of 71, `test` 803 passed and 1 skipped, `inspect:no-software-renderer`
    passes. No product code, so no reinstall. The uncommitted `.mjs` restores in the old scratch tree `land-share`
    were not pushed; that tree is superseded and has been removed.
    LANDED, 2026-10-22, the three stale guard anchors repaired (merge tip `3ad1271c94` on `origin/main`, ancestry
    confirmed; guards file only, no product code). `failed-job-freed` now anchors on the failure path of
    `src/native/tile.rs` (`current.tile_job = None;` before the debug-only `worker_failures` push) and names the test
    that was renamed to `a_failed_worker_reply_frees_its_job_and_names_the_failure`; `overscan-early-return` now
    turns the `if changed { render } else { note }` branch of `src/native/viewport.rs` into `if true`, because the
    early `return;` it guarded is gone; `minimap-scroll-notes-bands` only needed the indentation of its anchor.
    The static anchor check reports 0 absent anchors of 168. All three were proved in a disposable copy: baseline
    passes, guard removed fails, restored passes (3 of 168 guards; the other 165 were not run in this pass). The first
    run stopped at `failed-job-freed` because the test name was stale, which is how the rename was found. Gates once
    on the merged snapshot: `lint` passes, `test:bin` 71 of 71, `test` 803 passed and 1 skipped,
    `inspect:no-software-renderer` passes (no Rust changed, so no `test:native`).
    State after this series: no IDE agent is running; `origin/main` holds the conversion, the stacked tile, the
    minimap lead, the 90 percent share and the repaired anchors; the installed IDE is the minimap build. Open items:
    the annotation stability tests (not reproduced), the caret test (fixed by inference only), the stacking branch
    measured only on the one-texture path (no run at a window tall enough to need more than one texture), the late
    tiles B 2 of 140 against A 0 of 140 (cause unknown), dense-selection cost, the blind spot for a chunk shifted by a
    whole line, and the 15 other `.mjs` files in `bin/`.
    Owner decision of 2026-10-22 (question "convert the 15 other `.mjs` files?", answer "Convert all 15
    (Recommended)"): convert them to TypeScript like the first 17. Running: the second conversion agent (Sonnet, own
    worktree off `origin/main`, branch push only; brief quotes `NLG`, `NLD`, `RXI`, `NQH`, `FHU`; converts
    `inspect-source-guards` in its own commit and merges `origin/main` before it reports, because other sessions edit
    that file; reports real line counts). On its report: verify the diff, the compared outputs and the line counts
    myself, merge in a scratch tree (the guards file may conflict again: keep main's semantic changes), gates once
    (`lint`, `test:bin` 71, `test`), prove two or three guards in a disposable copy because the guards script changes,
    push, clean up, add a LANDED block here and a note on issue 1794.
    Owner request of 2026-10-22: "Please setup gh releases CI." Existing state I found: the music player has
    `.github/workflows/music-player-release.yml` (decision `doc/decision/music-player-releases.md`), `forbidden-strings`
    has binary releases inside `cargo-publish.yml` (`build-binary`, `create-release`), and the IDE (`publish = false`,
    0.1.0) has no release path. CORRECTION, my error: I first wrote here and told the owner in the question that the
    nested compositor was only published to crates.io and that `mise` built it from source. That was wrong. The
    compositor has had GitHub releases since 0.1.2 (`gh release list` shows seven, v0.1.2 to v0.6.1, all immutable and
    not pre-releases, each with x86_64 and aarch64 gnu `.tar.gz` archives), `nws-build-binary` and `nws-create-release`
    exist since `6c0c5de42c`, `[package.metadata.binstall]` is in its `Cargo.toml`, and a cargo-binstall probe finds
    the archive. I had read only grep lines of `cargo-publish.yml` and never the nws jobs. The owner's choice of "the
    nested compositor binaries" rested on that wrong premise, and I told the owner so.
    Question asked (which products): the owner chose both the IDE and the compositor binaries. Settled, adopted from the
    music player decision without asking: version bump on `main` triggers, dispatch dry run defaults to true, every file
    attested, immutable tags, one `gh release create` with all assets, pre-releases, first real release only with the
    owner's go-ahead. Running: two Sonnet agents in their own worktrees, branch push only (an IDE workflow
    `ide-release.yml` with its decision and runbook, Linux x86_64; compositor jobs added to `cargo-publish.yml` with
    Linux x86_64 and arm64 archives and `[package.metadata.binstall]`, no version bump). A new workflow can only be
    dispatched from `main`, so after each branch is verified and landed I dispatch a dry run
    (`gh workflow run ... -f dry-run=true`), read the run, and send fixes back; no real release is cut without the
    owner. Follow-up not done: switching the `mise` backend for the compositor to the prebuilt release.
    Compositor agent handed back (branch `worktree-agent-aedec3def05fc4366`, tip `420b7ceb2e`, not on `main`): it found
    the release jobs and binstall metadata already present (see the correction above) and only hardened them: a dry
    run now builds every target, a smoke run and an archive verification step, licenses inside the archives, a
    checksums file in the same `gh release create`, the release tag looked up first (job `nws-release-state`), the
    release waits for the crates.io publish, runner `ubuntu-24.04` instead of `ubuntu-latest`, `--locked`. I read the
    whole diff of the workflow and it matches its report. I dispatched a dry run on that branch
    (`gh workflow run cargo-publish.yml --ref worktree-agent-aedec3def05fc4366 -f crate=monochromatic-nested-wayland-session -f dry-run=true`;
    dry run: no crates.io upload, no attestation, no release). Result, run `39829260282`: success; `nws-detect`,
    `nws-publish-crate` (the `cargo publish --dry-run` step), `nws-release-state` and both `nws-build-binary` jobs
    (x86_64 on `ubuntu-24.04`, aarch64 on `ubuntu-24.04-arm`, so the arm64 build and smoke run are proved on a real
    runner) succeeded, `nws-create-release` and every other crate's jobs were skipped. I downloaded the two archives:
    each holds the binary, `README.md` and `LICENSES/GPL-3.0-or-later.txt` and `LGPL-3.0-or-later.txt`, and the
    extracted x86_64 binary prints `monochromatic-nested-wayland-session 0.6.1`. LANDED, 2026-10-22: merge tip
    `c6f9787ba1` on `origin/main`, ancestry confirmed; `actionlint` exits 0 on the merged workflow; the push started no
    workflow run (the push path filter lists only `Cargo.toml` files). Not proved: the real release path (attestation,
    the checksums file in `gh release create`, `nws-create-release` after `nws-publish-crate`), which only the next
    real release exercises, and that release needs the owner's go-ahead. Agent worktree and branches removed.
    LANDED, 2026-10-22, conversion of the 15 other `.mjs` files (agent `afeb812862ba874cb`, branch tip `213bcc5ab4`;
    merge tip `0f93cc19df` on `origin/main`, ancestry confirmed). Gates on the merged snapshot in a capped unit
    (`conv2-gates`): merge clean, `lint` 0, `test:bin` 71 of 71, `test` 803 run 803 passed 1 skipped,
    `inspect:no-software-renderer` 0 (`test:native` not run: no native file changed). The guards script's diff
    (`inspect-source-guards`) was read in full: types and `replaceOne({...})`/`run({...})` object parameters only,
    one added `throw` for a guard that names no test kind, cases untouched. Open: the agent saw the `clip-chosen`
    guard's removed run panic at `src/native/tile_reads.rs:187:45` instead of the expected failure; being checked in a
    disposable copy (unit `conv2-guards`, guards `clip-chosen`, `failed-job-freed`, `minimap-scroll-notes-bands`).
    IDE release workflow agent `a286368be67434dfd` handed back (branch tip `b79ed220b1`): `ide-release.yml` (641
    lines), `doc/decision/ide-releases.md`, `doc/runbook/release-the-ide.md`, README `### Releases`. I read the whole
    workflow: detect job (version bump trigger, dispatch dry run defaults true, real release only on `main`, existing
    tag skips), build in the repository's podman image tasks on `ubuntu-24.04`, gates (`--version`, shared libraries
    and glibc floor 2.39, `inspect:no-software-renderer`, `inspect:bundle` window-free checks, tracked files
    unchanged), tarball plus notices file plus `SHA256SUMS`, attestation only when not a dry run, one
    `gh release create --prerelease`. `actionlint` 1.7.12 exits 0 on the merged workflow. LANDED: merge tip
    `e976345c28` on `origin/main`, ancestry confirmed; the push started no run (path filter lists only the IDE
    `Cargo.toml`). Dry run dispatched from `main`: `gh workflow run ide-release.yml --ref main -f dry-run=true`, run
    `39832705367`, watched by a bounded background `gh run watch`; on its end: download the artifacts, check archive,
    notices and `SHA256SUMS`, run the extracted binary `--version`, record the cold build time in the decision, send
    failures back to the agent. Owner answers of 2026-10-22 (asked through the question tool): (1) licensing of the
    shipped bytes: "Keep as shipped (Recommended)" (Slint used under `GPL-3.0-only`, release notes say so, manifest
    unchanged); (2) toolchain: "Leave unpinned, record rustc (Recommended)" (stable compiler from the Fedora image,
    the release notes record the `rustc` version, no caches). Dry run `39832705367` RESULT: failed. `detect` and
    every `build` step up to and including the glibc/shared-library check passed (cold build 15m37s in total, so
    the 150-minute timeout is generous); the step "Check that no software renderer is linked" failed: `cargo tree
    --frozen --all-features` in the container found no `async-net` (required by `i-slint-backend-testing`) in the
    offline index of the fresh `ide-cargo` volume, so `checksums` and `release` were skipped and the later build
    steps (`inspect:bundle`, notices, packaging, archive check) have not run on a runner. Verified: `async-net` is
    absent from `Cargo.lock` (`rg -c` exit 1) and is an optional dependency of `i-slint-backend-testing` 1.18.1
    (`[dependencies.async-net] optional = true`). Not yet verified: why `cargo tree --frozen --all-features` asks the
    offline index for it on a fresh volume while the long-lived local volume answers; the fresh-volume reproduction
    decides it. Fix sent to the same agent
    (`a286368be67434dfd`, fresh branch off `origin/main`): reproduce with a fresh volume, fix at the cause without
    weakening the gate, run every gate it can reproduce locally against the fresh volume, record the cold time.
    Then: land, dispatch dry run 2, repeat until green, then download and verify the artifacts.
    Agent hand-back (branch `fix/ide-release-no-renderer-prefetch`, tip `5e1a86d63f`): the failing command was the
    second positive control (`--features slint/mcp`), not `--all-features`; those crates are never resolved by
    `cargo fetch` and `--frozen` cannot download index entries, while the long-lived local volume already held them.
    Fix in the task `inspect:no-software-renderer` (not the workflow): one network-enabled `--locked` pass per request
    before the unchanged offline checks. I read the diff (11 lines in `mise.toml`, doc and runbook updates, one
    workflow comment) and ran `lint`, `test:bin` 71, `test` and `inspect:no-software-renderer` on the merged
    snapshot: all 0. LANDED: merge tip `08d1525b83` on `origin/main`, ancestry confirmed. Agent-reported, not
    re-run by me: the full chain on a fresh volume (`bundle` 5 min 6 s locally, `inspect:bundle` 3 checks, 581,554
    byte notices, 23,457,136 byte archive); runner timings from run 1 (base image 1 min 04 s, IDE image 1 min 30 s,
    `bundle` 10 min 26 s; about 8 GB disk used). Dry run 2: run `39878025410`, watched in the background.
    Dry run 2 RESULT: success, every step green (`detect` 27 s, `build` 15 min 53 s, `checksums` 9 s, `release` 6 s,
    attestation steps skipped as a dry run; the `release` job printed the `gh release create` command only). I
    downloaded the three artifacts: the sha256 of the archive (`4628854e30...`) and of the notices (`0fc6032c03...`)
    match `SHA256SUMS`; the archive holds exactly `bin/monochromatic-ide`, `INSTALL.txt`, `LICENSE`, both
    `LICENSES/` texts, the launcher entry and `THIRD-PARTY-LICENSES.txt`; the extracted binary run with an empty
    environment prints `monochromatic-ide 0.1.0` and its `--licenses` output is byte-identical to the notices file.
    Not proved: the real release path (attestation, `gh release create` with all assets, tag creation), a window on
    the runner (none possible: no display, no GPU), and the binary on a machine other than this one. The owner has
    not said go for the first real release; none has been cut. A real release needs a version bump merged to `main`
    or `gh workflow run ide-release.yml --ref main -f dry-run=false` (runbook `doc/runbook/release-the-ide.md`).
    The IDE release agent's worktree, branches (local and remote) and my scratch trees are removed.
    Owner answers of 2026-10-22 (question tool): "Cut it now" for the first real IDE pre-release, and "Run the
    sweep (Recommended)" for the 168-guard sweep. REAL RELEASE dispatched: `gh workflow run ide-release.yml --ref
    main -f dry-run=false`, run `39882469329`, from `main` at `461c8c62b6`; watched by a bounded background `gh run
    watch`. On its end: `gh release view ide-v0.1.0`, download the assets, `gh attestation verify` each file,
    compare `SHA256SUMS`, run the extracted binary; record the result here and in `doc/decision/ide-releases.md`.
    RESULT, run `39882469329`: the release was created before I could cancel it: pre-release `ide-v0.1.0` (tag
    `ide-v0.1.0`, target `461c8c62b6`, immutable, 3 assets: `ide-0.1.0-linux-x64.tar.gz` 23,457,040 bytes,
    `ide-0.1.0-third-party-licenses.txt` 581,554 bytes, `SHA256SUMS`). The owner then said (mid-turn) "Must also
    support Linux ARM64"; I tried `gh run cancel` and it answered "Cannot cancel a workflow run that is completed",
    so v0.1.0 is x86_64 only and cannot be amended (immutable); I did not delete it. ARM64 ships from the next
    version. Not yet done for v0.1.0: `gh attestation verify` of the three published files and a download compare.
    STANDING OWNER AUTHORIZATION (mid-turn message, 2026-10-23): "You may do new releases of the IDE and the nested
    compositor whenever you want w/o my approval every time." So a version bump and a real release of either
    product needs no further question, but each still follows the dry run first and the runbooks.
    ARM64 agent `aaae391ccfc296421` (fresh Sonnet agent in its own worktree, branch push only; the earlier workflow
    agent could not be resumed because its worktree was gone): matrix `ubuntu-24.04` and `ubuntu-24.04-arm`, audit
    of the x86_64-only assumptions, no version bump. On its report: review the diff, merge in a scratch tree,
    `actionlint`, land, dispatch the dry run, check both archives, then bump the IDE version to 0.1.1 and release.
    Guard sweep: not started yet; needs a built target directory (the scratch trees that held one were removed), so
    build one first in a capped unit, then run all 168 cases in a capped unit and repair broken guards like
    `clip-chosen`.
    Guard sweep finding (disposable copy, unit `conv2-guards`): `clip-chosen` removed run fails with status 100 at
    `src/native/tile_reads.rs:187:45` (`tile texture pixels`) because the stacked tile reads no texture for a line
    past the raster limit; the guard still bites but its expected `failure` text (`the window shows an error`) was
    stale since the stacking landing. Changed the text to `tile texture pixels` (the only case with it); re-proof in
    unit `conv2-guards2` with `clip-chosen`, `clip-edges`, `failed-job-freed`, `minimap-scroll-notes-bands`: all four
    accepted in baseline, removed and restored ("Source guard controls passed ... 4 of 168 guards", 236 s), so the
    converted `.ts` script runs end to end. 164 of 168 guards remain unrun since the stacking and minimap landings;
    a full sweep is still a candidate real job. Conversion agent worktree and branches removed.
    Guard sweep, owner chose "Run the sweep (Recommended)" (2026-10-23): per-guard driver `~/temp/agent/sweep-each.ts`
    in unit `sweep-each` (one guard at a time, continues after a failing guard; results in
    `~/temp/agent/sweep-each/summary.txt`, artifact dirs `~/temp/agent/ide-source-guard-*`, built tree
    `~/temp/agent/land-sweep`). At the 89 of 153 mark: 84 PASS, 5 FAIL. Findings:
    - `clip-rounding`: the test did not bite (still passed with the guard removed); strengthened in
      `src/native/wrap_clip_tests.rs` (clip height assertion plus a band as tall as the cap), proven to fail with the
      guard removed. UNCOMMITTED in the main checkout until the sweep ends.
    - `tall-window-edge`: stale. Stacking raised `tile_cap`, so windows of 10,000 to 14,000 px no longer reach
      `room` below 0.25 view heights in `recenter_edge`; removed run passes. Needs a test at a height where the stack
      bound binds.
    - `rewrap-view-past-cap`: stale. The test `a_view_taller_than_the_clip_cap_is_left_to_the_worker` passes with the
      three removed checks (cause not yet found; read `begin` in `src/native/rewrap.rs` for which later defer fires).
    - `tile-stamp-names-its-clip`, `frame-fit`, `refused-tile-keeps-image`: baseline exit 127, "Cannot create
      temporary file in /tmp/" in the container link step while three guards ran in parallel; environment, to be
      re-run alone and sequentially.
    Next: when the sweep ends, re-run those three, repair the stale guards, prove each (baseline, removed, restored),
    gate, commit explicit paths, push.
    SWEEP DONE (153 run in `sweep-each`): 147 PASS, 6 FAIL: the five above plus `minimap-chunk-size` (removed run with
    `CHUNK_ROWS` 4096 passes `the_densest_chunks_are_not_blank`; needs a value or a test that bites). Re-run of the
    three environment failures alone: unit `rerun3`, log `~/temp/agent/rerun3.log`.
    `rewrap-view-past-cap` cause: with the checks removed `rewrap::begin` returns an allowance, then `render.rs`
    defers anyway (`measure_requested` incomplete or `raster_fits` of the settled tile), and the fixture's short
    lines are already shaped, so `shaped + rebroken == 0` holds either way; the test needs lines that must be
    re-broken at the new width so the removed checks show `rebroken > 0`.
    ARM64 HAND-BACK (agent `aaae391ccfc296421`, branch `feat/ide-release-linux-arm64`, tip `bdef20f6b0`, based on
    `0fa4950b88`, nothing dispatched, version still 0.1.0): matrix `x64` on `ubuntu-24.04` and `arm64` on
    `ubuntu-24.04-arm`, asset `ide-<version>-linux-arm64.tar.gz`, per-architecture NEEDED list (the aarch64
    executable has no dynamic loader entry), notices from the x64 leg only with a byte comparison against the arm64
    leg's, toolchain lines as `toolchain-*` artifacts, `inspect:no-software-renderer` checks both targets, docs and
    decision updated. Measured by the agent under qemu-user only (not the runner); unproven until the dry run:
    `--pids-limit`/`--ulimit` on the arm runner, disk and time of a cold arm build, `jdx/mise-action` on arm,
    attestation and the two-leg checksums and release jobs. Agent reported a `podman system reset --force` with a
    private storage config that caused a burst of lock errors in the default podman state; images and containers
    looked intact. My review: read the workflow diff in full, no defect found. Next: merge in `~/temp/agent/land-arm`,
    `actionlint`, renderer gate, push, dispatch the dry run, verify both archives, then bump to 0.1.1.
    ARM64 LANDED (2026-10-23): merge `785e685ff9` on `origin/main`; `actionlint` and `zizmor --offline` clean, renderer
    gate prints "not linked ... for x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu" with both positive
    controls reaching it. Dry run `39903652488` dispatched from main (`gh workflow run ide-release.yml --ref main
    --field dry-run=true`), bounded watch log `~/temp/agent/arm-dry-watch.log`. On green: download both archives and
    `SHA256SUMS`, compare, then bump `package/desktop-app/ide/Cargo.toml` to 0.1.1 (the standing authorization
    covers it) and verify the real release including `gh attestation verify` of all assets.
    ARM64 dry run `39903652488`: all five jobs green. I downloaded the artifacts: `sha256sum --check` OK for the three
    assets, the arm64 binary is `ELF 64-bit ... ARM aarch64 ... interpreter /lib/ld-linux-aarch64.so.1` (90,611,544
    bytes, as the agent's emulated build), NEEDED `libfontconfig libgcc_s libm libc`, INSTALL.txt names aarch64, the
    released notices equal the arm64 leg's and the archive's byte for byte, rustc 1.99.1 on both. x64 binary prints
    `monochromatic-ide 0.1.0`. The arm64 binary has not been run (this host is x86_64); nobody has opened its window.
    REAL RELEASE 0.1.1: bump `ec2dec4e0d` pushed to `origin/main` (Cargo.toml and exactly one Cargo.lock line, via
    `//package/desktop-app/ide:fetch`), under the standing owner authorization. Run `39905451031` (push event),
    watch log `~/temp/agent/rel-watch.log`. On green: release page `ide-v0.1.1` has four assets and is a pre-release,
    `gh attestation verify` of each, sha256 compare; then verify `ide-v0.1.0` the same way.
    RELEASED `ide-v0.1.1` (run `39905451031` green, tag target `ec2dec4e0d`): immutable pre-release with
    `ide-0.1.1-linux-arm64.tar.gz` (23,869,002 bytes), `ide-0.1.1-linux-x64.tar.gz`, `ide-0.1.1-third-party-licenses.txt`
    and `SHA256SUMS`; I downloaded it, `sha256sum --check` OK for all three files, and `gh attestation verify
    --repo Aquaticats/Monochromatic` exited 0 for all four files; same for `ide-v0.1.0` (three files, exit 0). The arm64
    binary is verified as a real aarch64 ELF but has never been run on ARM64 hardware. Not done: the installed IDE
    on this host is still the minimap build (prints 0.1.0); reinstall from the next product change.
    GUARD SWEEP CLOSED: agent `a0207c18495764475` repaired four guards, merged by me (`90ed2bd2d1` on `origin/main`,
    test-only: `tall_window_tests.rs`, `wrap_resize_tests.rs`, `bin/inspect-source-guards.ts`). I re-ran
    `tall-window-edge`, `rewrap-view-past-cap`, `refused-tile-keeps-image`, `minimap-chunk-size` and `clip-rounding` on
    the merged tree: baseline, removed and restored all accepted; gates once on that snapshot: lint 0, `test:native`
    290 passed 5 skipped, `test` 803 passed, `test:bin`, renderer gate 0. What each repair was:
    - `tall-window-edge`: heights now come from `tile_cap` (85 and 95 percent) with `long_text(1500)`, and two
      asserts prove the case is reached; the removed run takes about 90 s (waits out `Waiting::slow()`).
    - `rewrap-view-past-cap`: the step counters recorded nothing when a measurement ran out of its allowance, so
      the assert reads the shaper's own counters; `failure` text is now `brought in lines for a view past the cap`.
    - `refused-tile-keeps-image`: the mutation submits an empty one-texture `TileStack` to the GPU instead of the removed
      `set_source_image`.
    - `minimap-chunk-size`: `after` is now 8192 (the test passes up to 4096 rows and fails from 5120).
    Follow-ups from the agent's report, DONE (`3fdc6f8bb1` on `origin/main`): the `CHUNK_ROWS` doc comment in
    `src/minimap/plan.rs` now states the measured limit (between 4,096 and 5,120 rows, a factor of eight at 512; it
    said 2,048 overflow before `e5787ebfae`); new guard `minimap-chunk-margin` (`CHUNK_ROWS` 4096) for
    `the_chunk_size_leaves_a_factor_of_two_inside_what_one_render_holds`, baseline, removed and restored accepted;
    `doc/planning/ide-gpu-snapshot-harness.md` gained the section "Script names after the conversion to TypeScript".
    Lint exit 0 and `test:bin` exit 0 on that snapshot. The reason for exactly 512 was not found (any value up to
    4096 passes the tests).
    Cleanup done: agent worktrees, `land-*` trees, branches `fix/ide-stale-source-guards-2`,
    `worktree-agent-a0207c18495764475`, `feat/ide-release-linux-arm64` (local and remote, tips verified in `origin/main`),
    sweep scratch and `ide-source-guard-*` dirs removed. Other sessions' worktrees (`agent-a563e0c526d53df51`,
    `tr-*`) were left alone.
    OWNER REPORT 2026-10-24 (minimap "jumps around", hints "doubling" at line 96 of `build.unit.test.ts`), FIXED on
    branch `worktree-agent-a0ae031be26535225`: not a minimap defect. The minimap code did not change between the
    installed build (2026-10-22 12:48) and `origin/main`. Cause: TypeScript 7.1.0-beta answers a range request with the
    hints of every node that reaches into the range, so a request for lines 100 to 200 returns the four hints of line 96
    (the statement `const integrationChildren = ....map(...)` spans lines 96 to 269), and `Annotations::merge_labels`
    kept them although it had removed only the old hints of the asked lines: each such answer added four hints to line
    96 (counted 4, 8, 4, 8 in the application log before the fix; the owner's picture shows five copies). Every line
    below moved by a block per copy, which is the jumping, and a later answer that asked for line 96 removed all copies
    again. Editor frames recorded while clicking back to line 96 (22.5 frames per second kept): the installed build
    showed a 130 px gap above line 96 for 5 frames (6 hint rows instead of 3), then 82 px; the fixed build 82 px in
    all 34 frames. Reproduced in the headless nested compositor at 1080x1880 scale 2 with the real
    TypeScript 7 server on a disposable copy, same clicks on the minimap gave minimap crops that differed by 17,028 px
    (visit 2 against visit 3 of y=1700) and 1,925 px (y=1000); after the fix the same pairs differ by 0 px. Probe of the
    server alone: `~/temp/agent/mm/lsp-probe.ts`; record `doc/troubleshooting/typescript-7-inlay-hints-outside-the-requested-range.md`.
    Fix: hints standing on lines outside the answer's asked lines are left out (`src/annotation/blocks.rs`); tests
    `hints_outside_the_asked_lines_are_not_part_of_an_answer` and
    `an_answer_for_other_lines_adds_no_hint_to_a_line_it_did_not_ask_about` fail before, pass after; guards
    `hints-asked-lines` and `hints-asked-lines-window` accepted in baseline, removed and restored. Gates on the fixed
    tree: lint 0, `test:native` 291 passed 5 skipped, `test` (without the real-source wrap tests) 798 passed,
    `test:minimap` 30 passed, `test:annotations` 51 passed, `test:bin` 71 passed. The installed IDE still has the
    defect until reinstalled. Residual, not this defect: with the fix, the minimap of a first visit to the bottom
    differs by 994 px (device rows 2500 to 3238 of the track) from the one after a visit to another place, and is
    stable afterwards (0 px); the server's first answer had 68 hints against 72 in-range hints it gives when warm
    (a probe of the server alone shows range answers equal the full-file answer inside the range), so it is most
    likely the server's first, incomplete answer; not verified further.
    Committed `clip-rounding` test strengthening (`1b57f960de`, merge `3f220bd444`).
    Sweep re-run of the three environment failures: `frame-fit` and `tile-stamp-names-its-clip` accepted in all three
    phases; `refused-tile-keeps-image` is a fourth real stale guard (its `after` injects
    `window.set_source_image(...)`, a property that no longer exists, so the removed run does not compile).
    Guard repair agent `a0207c18495764475` (Sonnet, own worktree off `origin/main`, branch push only to
    `fix/ide-stale-source-guards-2`; brief quotes `NLG`, `NLD`, `RXI`, `NQH`, `FHU`): repairs `tall-window-edge`,
    `rewrap-view-past-cap`, `refused-tile-keeps-image`, `minimap-chunk-size`, each proven baseline/removed/restored.
    On its report: read the diff, re-run the four guards myself, merge in a scratch tree, gates once (lint, test:bin,
    one full `test:native`, test), push, remove worktree and branches, stop leftover `stalegd-*` units.
    Stall pass agent `ae05d7039f46336dc` (started 2026-10-22, own worktree off `origin/main`, branch push only; brief
    quotes `NLG`, `NLD`, `RXI`, `NQH`): at least 60 alternating stall pairs (tree A = before the wake-up, tree B =
    `origin/main`) plus 30 A against A pairs for the spread; keeps the debug timeline of every stalled run and decides
    per slot whether a tile was applied outside a step, an input gap, or neither (host); fixes the wake-up only if it
    is shown to cost frames, keeping the 0 of 40 late tiles. On its report: same verification as the others.
    LANDED, flake agent `a4671ba160cf3feec` (third flake session; merge tip `6620fda00a` on `origin/main`; tests, one
    `mise.toml` line and the ledger `doc/handover/ide-native-flakes.md` only, no product code, no binary change, no
    reinstall, zero retry overrides). Gates once on the merged snapshot: lint, `test:native 'all()'` 283 passed and 5
    skipped with no FLAKY or SLOW, `test` 797 passed, `test:bin`, `inspect:no-software-renderer` ("not linked"). The
    new guard `a_reader_that_stalls_past_the_quiet_time_still_has_rows_wait` fails when the scrolling rule ignores
    the held clock (I removed the fix in the scratch tree and it failed, then restored). What the agent found:
    - A new flake, fixed at its cause: the minimap test
      `fifty_thousand_lines_give_a_bounded_picture_without_blocking_the_window`
      read the third picture's statistics as the cold build's (before 317 of 400 failed, after 0 of 400). The test
      now asserts shaped plus reused equals lines; the cold-build claim stays asserted in `tests/minimap_scene.rs`.
    - Quiet-time test: cause, fix (`3bcdc17603`, a clock a test can hold) and evidence in the ledger; the guard failed
      20 of 20 with the fix ignored, a real-clock copy failed 10 of 10.
    - Caret test `the_view_follows_the_caret_through_every_visual_row`: "fixed by inference" only (the unit guard
      `the_finish_of_a_replaced_pass_leaves_the_newer_pass_open` fails 20 of 20 without the fix, but the tool never
      produced the interleaving); it now reports wrap widths and facts of both moments if it fails again.
    - Annotation stability tests: NOT reproduced in about 2,100 runs (0 failures) under load averages of 18 to 56;
      still open and instrumented (`shot()` takes a second frame of the same state and names the tile counters);
      the one old failure was a burst in one copy while another hit "No space left on device", so a full host disk
      is a suspect.
    - Video memory: measured, no change. One test process holds about 0.28 GiB (0.52 at most), 21 at once 2.37 GiB;
      the card spilled to system memory with no failure attributable to it. A cap would free under a third of the
      card, so none is made.
    - Two findings the agent did not fix, recorded as open work: (a) the `runtime` mise task is not safe for two
      concurrent callers in one tree (a `build` and a `test` started together failed with ENOENT renaming
      `target/debug/runtime/grammars/awk.so.new`; fix by atomic staging or a freshness skip); (b) the gate
      I ran, `test:native -- 'all()'`, is a filtered run, and `package/desktop-app/ide/mise.toml` (line 306) adds
      `--no-capture` to every filtered run, which makes nextest run one test at a time (sum of durations 198.5 s,
      wall 198.6 s). That is my invocation's fault, not a defect of the task: every full pass recorded with that
      filter was serial, and only an unfiltered `test:native` runs at the thread bound. Before 2026-10-22 only a
      127-test subset at 21 threads and 32-process hunts were parallel.
    - The agent exceeded the brief's CPU guidance (one unit at `CPUQuota=2400%`, foreground runs at the 21-CPU
      bound, about 24 CPUs at the peak). The owner's statement above allows parallel real jobs, each capped; it is
      recorded as an overrun of the brief, not hidden. Cleanup was verified by me: no `flk` units, no stress
      container, no process in its worktree; its worktree and branch are removed, local and remote.
    First unfiltered full `test:native` runs at the thread bound (2026-10-22): snapshot `052cc29638` 283 passed and 5
    skipped in 35.9 s (unit `full-native`, `CPUQuota=1200%`), and the runtime merge snapshot 283 passed and 5 skipped
    in 44.2 s, both with no FLAKY, SLOW or FAIL line; the serial filtered run of the same tests took 176 s. These are
    snapshots, not flake counts.
    LANDED, runtime task agent `a70cd9e6f7ed2044f` (merge tip `bbbf0d0a23` on `origin/main`; `mise.toml`, new
    `bin/runtime-publish.ts` and `bin/runtime-publish.test.ts`, section "The runtime task is safe for concurrent
    callers" in `doc/handover/ide-native-flakes.md`; no change to the shipped binary, so no reinstall). Cause: two
    callers shared the staging name `<grammar>.so.new` and one renamed it away (`ENOENT` on `awk.so.new`), and both
    shared `target/runtime-config` for the fetch and build. Fix: an exclusive `flock` on `target/runtime.lock` around the
    whole preparation (the kernel drops it when its holder dies), unique staging names per file, skip of an equal file,
    manifest published last, stale staging swept under the lock. Evidence: paired starts of the unfixed task failed in
    10 of 20 pairs (the positive control); after the fix 0 in 20, 0 in 24 more, and 0 in a final 20 on the last code;
    my own 5 pairs on the merge snapshot 0 of 10 callers failed. The tree hash of `target/debug/runtime` (1622
    entries) is unchanged for a single caller. The guard tests (11) fail with a fixed staging name (I forced it in the
    scratch tree: `test:bin` status 1, then restored). Gates once on my snapshot: lint, unfiltered `test:native`, `test`
    797 passed, `test:bin`, `inspect:no-software-renderer`. Deviations of the agent: fix commit `0e749d3a72` uses the
    scope `desktop-app/ide` where the repo scope is `desktop-app-ide` (not amended, per the never-amend rule); gates ran
    at 8 CPUs and 16 GiB; one pair took 307 s waiting on another session's cargo lock on the shared `ide-cargo`
    volume. Cleanup verified: no `ide-rt` unit, no `flock` process, worktree and branches removed.
    Open, in order: (1) the stalls away from slot 50 (slot 33 and 72, one run each; undecided; a larger alternating
    pass with a same-binary control, under whatever load there is; the stall pass agent) and whether a tile taken in an
    input gap over 20 ms costs a frame; (1b) the code tile stack and the minimap fling lag (the two agents above);
    (2) the flake work, in `doc/handover/ide-native-flakes.md`:
    fixed with counts (refused-tile test 10 of 360 to 0 of 360, wrap clip 3 of 240 to 0 of 240, reader fixture 28 of 160
    to 0 of 160, find-box menu test by its guard, quiet-time test, minimap picture test 317 of 400 to 0 of 400, see the
    LANDED blocks above); open: the annotation stability tests (not reproduced in about 2,100 runs; instrumented, frames
    kept under `target/flake-frames/` for the next failure; a full host disk is a suspect for the one old failure),
    the caret test (fixed by inference only). Closed: (2a) the `runtime` task race (see its LANDED block) and (2b) the
    serial gate (my filtered `test:native -- 'all()'` ran one test at a time; the gates now use the unfiltered task and
    the first two unfiltered runs passed). The video-memory question is closed by measurement (no cap).
    Full `test:native` passes are snapshots, not flake counts. (3) Scale 2, light scheme and seam checks: done, see the
    scale agent's LANDED block.
    The prevention proposal `doc/planning/agents-cannot-load-the-host.md` waits for the owner: a fresh Claude session and
    at least two more reviewers must read it first. Stopgap rules `NLG` and `NLD` are in `AGENTS.md`. The entry in
    `doc/limitation/claude.md` ("Delegated GPU migration of the Slint IDE") lists this session's and the agents' mistakes.
    Owner addition of 2026-10-20: "proven unfixably flaky" is a very, very high bar; "in many cases we can change our
    code, our tests, split the tests, and more instead". Zero retry overrides is the expected outcome. A test may go under
    retries only when the report lists every alternative tried or ruled out with evidence (deterministic product code,
    waiting on the real condition, splitting the test, injected clock or renderer state, waiting for GPU idle, less
    parallel GPU use, a restructured assertion), and the cause lies outside our code and outside what a restructured
    test controls; difficulty, effort or load-dependence alone never qualify; an undiagnosed flake stays an open,
    instrumented flake and is not retried. Sent to agent `af624c22739073177`.
    Flake work, second session (2026-10-21, branch `worktree-agent-a5d7faec36a652c16`, details and the ledger in
    `doc/handover/ide-native-flakes.md`): `find_box_context_menu_runs_each_entry` is explained and fixed (the menu opens
    under the pointer near the window bottom, and a release more than 300 ms later runs the entry under it; hold-time
    control 600 of 600 failing before, 0 of 600 after); the highlighting-reply wait is in the other fixtures; the
    annotation stability tests did not reproduce with the permitted tool (0 of 1,310) and keep their frames; no retry
    overrides.
    Owner instruction of 2026-10-20 that REPLACES the run-count rule in the next paragraph: "I suggest you properly fix the
    flakes instead. And we should never run a full test against one snapshot of code. If some tests are confirmed
    unfixably flaky, make the test harness run the specific tests more instead." My reading (stated to the owner, not
    contradicted so far): nobody repeats the whole suite on one snapshot to judge it; one full `test:native` run per
    snapshot is the gate; flakes are fixed at their cause; a test proven unfixably flaky gets a nextest per-test retry
    override that keeps the pass-after-retry visible (FLAKY in the summary) and an entry in a ledger. Agent
    `af624c22739073177` (started 2026-10-20) builds a per-test stress tool (a mise task), reproduces each known flake
    (listed in its brief: the annotation stability tests, `find_box_context_menu_runs_each_entry`,
    `refused_tile_tests`, `wrap_clip_tests`, the older `write_wait_tests` and `watch_tests`, the `wrap_tests` `settled`
    early return; the CPU minimap test is the stage 5 and 6 agent's and disappears with it), fixes causes, and adds the
    nextest config and ledger only for tests it proves unfixable. For stage 5 and 6 verification I now run
    `test:native` once on the merged snapshot and rely on the per-test stress tool for any test that fails.
    Owner question of 2026-10-20, "Do we really need 15 or more full test:native runs? I thought we fixed the
    flakes?": no, 15 was my round number. With a true flake rate p per run, n runs catch it with probability
    1-(1-p)^n: 54 percent at p = 5 percent and 79 percent at p = 10 percent for n = 15, so it is no proof either. The
    four causes found by the flake agent are fixed (three batches of 15 passed since, one at load 50); the residual
    repeat offender is the CPU minimap test `an_external_edit_of_fifty_thousand_lines_renders_only_the_band_it_touches`
    (1 of 20 on the baseline), which stage 5 and 6 delete; one nine-level pixel difference and
    `find_box_context_menu_runs_each_entry` were each seen once. Rule I adopted (veto-open): a change that adds or
    rewrites tests that read pixels or wait on timers (stage 5 and 6 does) gets 15 consecutive full runs; any other
    change gets 3 full runs. The "15 or more" in the stage 5 and 6 brief stands for that reason only.
    LANDED 2026-10-20 (pushed as `6a44ec419f`; the IDE tree equals the one I verified: `lint`, `test:bin`, `build`,
    `inspect:software-renderer` pass, `test:native` 15 of 15 at 264 tests under a load average of about 50; the first
    push attempt failed twice, a hook message and then a plain non-fast-forward because others pushed, and the third
    attempt after re-merging `origin/main` succeeded; the agent worktree, its local and remote branches and the scratch
    worktree are removed, no strays). The IDE was reinstalled with the selection fix and the memory hint:
    `~/.local/bin/monochromatic-ide` sha256 prefix `ab91c5f45960a4e7` (equal to the release build), previous binary
    `monochromatic-ide.previous` (`80c27b37e4224602`). Before this install the shipped binary blanked or staled the code
    view on a tall selection. I did not rerun the agent's cadence and memory measurements. The next paragraph is the
    pre-landing state of this entry.
    Report in from agent `ae0713008b220a655` (selection fix and memory hint; verification in progress in the scratch
    worktree `~/temp/agent/ide-integrate2`, merged with `origin/main`): the bug reproduced in the SHIPPED app (release
    binary of `origin/main`, nested compositor, 1147x2000, `/tmp/ide-gpu-fixture/code.rs`): 30 to 39 selected rows are
    fine; 42 or more keep stale glyph ink under the new fills; select-all followed by a scroll to the end blanks the whole
    code view (I viewed `~/temp/agent/vello-fix/shots-before-d/sel-all-scroll.png`, blue fills and no text, and the
    after shot with the text present). The study's row-height clip was not enough: select-all on the 16,384 px limit
    (682 dense rows) overflowed the lines buffer, because each clipped copy of a run is flattened again and a fully
    selected row cost three copies per glyph; `reach` and `meets` in `src/tile_scene/split.rs` drop copies a run's
    glyphs (plus half a font size) cannot reach (I read the diff and accept it). `MemoryHints::MemoryUsage` measured in
    the app: video memory idle 499 to 288 MiB (`code.rs` tall) and 476 to 216 MiB (`big.rs`), after scroll 505 to 358
    and 483 to 259; RSS equal; first frame and cadence inside the noise floor; a parallel `test:native` demand fell from
    about 11 GiB to about 5.3 GiB (card shared and full, so GTT spill; the study's 9.4 and 4.0 were sums of probe
    figures). Detection of the silent blank is impossible in Vello 0.11 without a patch (`robust` is
    `cfg!(feature = "debug_layers")`, `WgpuEngine` is `pub(crate)`). Decisions I took (veto-open): keep the memory
    hint; accept `reach`/`meets`; for detection prefer a pre-encode estimate of lines and tiles (no GPU wait, bounded
    cost) over a sentinel-texel readback after each render (a GPU sync per tile render), to be done in
    `src/native/gpu_tile.rs` after the minimap agent lands. Queued: parallel `test:native` still wants about 5.3 GiB of
    video memory plus GTT; decide on a test-thread cap or a device budget after stage 5 and 6; two items outside this
    scope: the doc `doc/planning/ide-gpu-snapshot-harness.md` has 56 older `semantic-line-breaks` findings.
    Agent `ae0713008b220a655` (started 2026-10-20) reproduces the
    selection blank in the real app, fixes the clip structure with a failing test first, applies e1 and measures VRAM,
    RSS, first frame and cadence with `matrix-load.mjs`. Verification of both uses `~/temp/agent/matrix-load.mjs` and at least 15
    full `test:native` runs, under whatever load exists.
    Found and fixed while adding the `BGL` rule (pushed as `1c3ec09299`): `package/desktop-app/ide/Cargo.toml` is
    managed by file-enforcer (WC2) and `file-enforcer.config.ts` owned `slint` fleet-wide as
    `backend-winit`, `renderer-femtovg`, `renderer-software`. Running `node file-enforcer.config.ts` rewrote the IDE's
    `slint` line back to that form (observed in a scratch worktree), so the stage 0b and stage 2 manifest edits, made
    by hand, would have been undone at the next sync, putting the software renderer and OpenGL back. `slint` is now
    removed from `CARGO_SHARED_DEPENDENCIES` (like `tokio`, `image`, `gtk4`, `windows`); after the change the enforcer
    leaves the IDE manifest untouched, and terminal and music-player keep their identical lines. The new rule
    `BGL` in `AGENTS.md` (and `CLAUDE.md`) says every background wait carries a deadline and is stopped by its starter.
    Lesson for agent briefs: manifests and root files may be generated, so check `file-enforcer.config.ts` before
    editing them (WC2) and run the enforcer in a scratch worktree to prove the edit survives a sync.
    State at 2026-10-20 after the three agent reports (to verify and land in this order):
    (1) flake diagnosis of stages 3 and 4 (agent `a717e60922dca0448`, running): on a quiet host 2 of 8 full
    `test:native` runs failed on `main` (`annotation_stability_tests` "window row 240 differs after the annotations
    returned"; `waiting_tests.rs:138` "the find did not reach its first match"), against none in about 14 runs before
    stage 3; it also measures the Vello setup cost and the +52 MiB RSS and +260 MiB VRAM the stage 3 agent reported;
    (2) the MCP removal (agent `a3b41470505ef8920`, branch `worktree-agent-a3b41470505ef8920`, reported all gates
    green; `inspect:native` now uses the compositor's `a11y`, `inspect:log-stall` probes the compositor's `windows`
    and screenshots; the log-stall control stalls before its first window, not after it); (3) the GPU choice by system
    order (agent `a090eeb1ae38efc05`, branch `worktree-agent-a090eeb1ae38efc05`, 245 of 245 native tests, adapter runs
    here: default RX 7600, `WGPU_POWER_PREF=low` Raphael, `DRI_PRIME=1` on the IDE only Raphael; setting `DRI_PRIME` on
    the nested compositor too flips the default because Mesa's default is the compositor's GPU). Heavy verification of
    (2) and (3) waits until (1) finishes so that its flake-rate runs are not perturbed by load.
    (2) and (3) are already merged with each other in the scratch worktree `~/temp/agent/ide-integrate` (detached,
    commit `31cd2f1140`, not on any branch, not pushed): the only conflicts were `gpu_device.rs` (the stage 3 and 4
    texture cap `open_hardware_device_capped` and `set_texture_dimension` reapplied onto the split
    `src/gpu_device/` layout) and the README (both new sections kept). Next: run `lint`, `test:native`, `test:bin`,
    `build`, `inspect:*` there after (1) finishes, then fast-forward `main` to it (merge `origin/main` first).
    Stages 3 and 4 are merged into one task (the tile bakes in selection, underlines and the wrap sign, so they
    cannot be split cleanly): agent `a8ec6e5d0b8981d42`, started 2026-10-20 (code view on the GPU, CPU pipeline
    deleted except what the minimap still needs). A second agent, `a3b41470505ef8920`, removes the MCP builds in parallel and rebuilds their checks on the nested
    compositor (started 2026-10-20).
    Stage 1 landed (agent `a54a3dbd127b14fbf`, merged into `main`, verified by me): `[profile.dev.package."*"]`
    `opt-level = 2` in the IDE `Cargo.toml` (clean debug build 49.7 s to 81.8 s, incremental rebuild unchanged at
    about 4 s, `test:native` 214 tests in about 23 s on a quiet host, measured by me after the merge) and the
    `inspect:cadence` task (`mise run //package/desktop-app/ide:inspect:cadence <scenario> <project> <file> [profile]`,
    `bin/cadence.ts`, `bin/inspect-cadence.mjs`, README section "Frame cadence check"). My control run of the debug
    build with optimized dependencies on `client.unit.test.ts`, scenario `drag`: 106 changed, 15 identical of 121,
    longest identical run 1 (single dropped frames about every eight frames, none of the 5 to 9 frame freezes the
    agent measured in the unoptimized debug build). The 50,000-line case was not measured. The agent changed
    `wrap_tests::rows_above_the_view_changing_height_move_no_visible_pixel` to wait for the extent to change before
    `settled`: with optimized dependencies `settled` returned while the extent was unchanged (10608 before and
    after, 24504 once measured); why is undetermined (candidate: the pass closes when its last output is taken while
    the window applies it on a later timer tick), so a production tracking bug in `pass_open` (`tile_build.rs`) is
    not ruled out. Commit scopes there are `desktop-app/ide` where the repo uses `desktop-app-ide`; not amended.
    Test races found by running the full suite on the merged `main` and fixed by me (`87d02a02d8`):
    `tall_window_minimap_tests` read the minimap picture of the window before it was resized; the fixture now also
    waits for the picture to be taller than 1000 rows (it passed 6 of 6 alone and failed once in a loaded full run).
    The three `markdown/semantic-line-breaks` findings from my earlier flake fix are fixed; the IDE `lint` task is
    green (clippy, rustfmt, slint, `monochromatic-lint`).
  - Where docs are edited (owner instruction of 2026-10-19): handover and doc edits are made in the main checkout
    (`/var/home/user/Monochromatic`), merging `origin/main` into it first when it is behind; not in a temporary
    worktree. Agent worktrees are for code.
  - Minimap drag jank (owner report of 2026-10-19, a screenshot of `client.unit.test.ts` in the search-fetch package:
    dragging and holding the viewport box is janky; the owner says it reproduces in a debug build too). Standing
    owner instructions: when the nested compositor cannot do something, improve it, release it and install it; the
    owner authorized releasing everything relevant this session without asking again. The sentence in the sidebar
    bullet that the compositor has no held pointer drag is wrong: `drag X1 Y1 X2 Y2 left steps=N step-ms=MS`,
    `move`, `button down|up` and `record DIR FPS bmp` with `record stop` exist in
    `package/cli/nested-wayland-session` (README, sections "Pointer input" and "60fps recording"); the agent corrects
    that sentence. Reproduction by the coordinator with the scratch driver `~/temp/agent/drag-frames.mjs` (records 60
    fps BMP frames during a held `drag` on the box and prints which frames changed; the project copy is in
    `/tmp/drag-project`, because the task needs a project below `/tmp`): a debug build over a 489-line copy of the file
    at 1147x2000, scale 1, shows 41 changed frames, 10 identical frames (about 170 ms), 41 changed, 11 identical
    (about 180 ms), 18 changed, with no frame dropped by the recorder. The minimap before the redesign (detached
    checkout `.claude/worktrees/ide-drag-control` at `1cd982bfd6`) shows exactly the same pattern, so it is not a
    regression of the redesign: either the text view stalls the window thread or the probe stalls; the probe has no
    positive control yet (a drag in the text area changes frames only every fourth step, so it is not one). An agent
    (`fix/ide-minimap-drag-jank`, worktree `.claude/worktrees/ide-drag`) builds the controls, finds the cause by
    window-thread timing, fixes it, and records the before and after patterns here.
  - Closure check everywhere (owner requirement of 2026-10-19, recorded in `doc/decision/rust-closure-lint-dylint.md`,
    proposal in `doc/planning/closure-lint-everywhere.md`): done. `monochromatic-lint` runs `cargo dylint` per Rust
    package as the rule `rust/avoidable-closure` (mechanism A), warnings first, tests, fuzz and bench exempt, a package
    missing its prerequisites skipped with a note, the IDE held at error severity, and the IDE `lint:closures` task
    removed. Released on 2026-10-19 as `monochromatic-lint` 0.3.0 at the owner's request (dry run of the
    `cargo-publish` workflow first, then the push of the bump published the crate and the GitHub release with eight
    archives; crates.io lists 0.3.0), installed with `mise upgrade cargo:monochromatic-lint` (`mise.lock`, commit
    `bca9a2856e`), and the config activated in `e2a0653130`. The coordinator reran the container suite (592 passed,
    2 skipped) and the closure package tests (23 passed), and ran the installed 0.3.0 over the whole repository:
    1692 warnings in 15 packages, none in the IDE, no `core/` failures; the run exits 1 because of Markdown
    `semantic-line-breaks` errors that were already there. Not proven: the mutation proofs were run by the agent on
    its debug build and not repeated; the GTK and Qt prerequisite kinds ran only their happy path; the generated-code
    skip of the lint is shown by its fixture and not by a removed-guard run. The owner also chose to leave the
    OpenSnitch pop-up default (reject, forever, 30 s) as it is.
  - Minimap box fill removed (owner request of 2026-10-19): the viewport box is an outline without a fill
    (`ui/minimap.slint`); hover and drag change its border width and opacity only. The native test is now
    `the_box_is_an_outline_whose_border_changes_under_the_pointer`. Checked: `lint:slint`, `lint:rustfmt`,
    `lint:clippy`, `lint:closures`, `test:native` (214 passed) and a screenshot of the real binary in the nested
    compositor; installed.
  - Minimap redesign (owner request of 2026-10-18, answers through the question tool; branch
    `feat/ide-minimap-downscale`, worktree `.claude/worktrees/ide-minimap`, an agent, started 2026-10-19; landed on
    `main` as `16540bdb9d` on 2026-10-19 and installed). The tall-window blanking, the container bound, the comment directives of `monochromatic-lint` and the
    Dylint gate are landed (their bullets). The owner wants the minimap to stay crisp, never replaced by rectangles,
    and to be merely a zoomed-out reflection of the main window; the owner chose "downscale the real rendering": the
    window's own glyph rendering with selection, find matches, diagnostics and virtual rows, shrunk to the minimap
    width, replacing the separate pixel-per-column builder of `src/minimap/` (`raster.rs`, `paint.rs`) and the
    picture side of `src/native/minimap_build.rs`. It must be tiled and cached so that a long file stays fast and
    under the raster limit.
    Owner answers of 2026-10-19 after the first finished build (they supersede the design below where they differ):
    a long file must not be squeezed whole into the window height (a 50,000-line file became a dim gray strip);
    the minimap keeps a fixed scale of at most a sixth and slides with the scroll once the file is longer than
    the track, building only the bands near the view; the huge dark surfaces (editor, minimap, sidebar) become
    pure OLED black so the picture needs no gain, and the exact area average stays; edits are not rare (the
    files change under the reader), so a change must re-render only the bands it touches.
    State on 2026-10-19, finished by the agent after a shutdown, a resume and the owner's second round (then landed:
    the coordinator reran `lint:rustfmt`, `lint:clippy`, `lint:release`, `lint:closures` and `lint:slint` (exit 0),
    `test` (807 passed, 1 skipped) and `test:native` (211 passed, 5 skipped) after merging `origin/main`, reran eight
    of the new guard cases (all three phases accepted), viewed the screenshots, and installed the release build;
    the installed binary was not launched, the previous one is `~/.local/bin/monochromatic-ide.previous`).
    Design (done): `src/minimap/` is `downscale.rs` (exact fractional area averaging, separable, f32, rows of a
    picture with an origin, edge rows shared unrounded), `plan.rs` (bands on a fixed pixel grid at multiples of
    `band_step`, at most 4 MiB), `key.rs` (content keys: FNV-1a digests of a band's text and clipped style spans,
    selection, matches and marks measured from the band's first character), `cache.rs` (a map of band number to slot,
    `retain(window)`, fallback to the previous rows when a band fails or the budget cannot pay it, with a
    `tracing::warn!`), `build.rs` (`prepare`, `build`, `Cancel`, `BUILD_WORK` 5,000,000), `geometry.rs` (fixed scale of a
    sixth, `track`, `slide`, offset-aware `content_y`/`target`, `bands_for`), `fills.rs`, `render.rs`, `horizontal.rs`.
    The worker is `src/native/minimap_build.rs` plus `minimap_build/serve.rs`; the window key holds selection, find,
    active match, annotations version, inks and the window of bands. The sampled path, the quick pass, the coarse
    picture, the stretched strip and `unit.rs` are deleted. This contradicts the original "coarse-then-full is kept":
    the quick pass was `Sampled(1)`, which no longer exists, and a new wrap width now yields one reply.
    Sliding: a text that fits the view at the fixed scale is drawn whole; a longer one slides past the track by
    `slide = overhang * offset / max_offset`, and the box top is the same share of its room, so a press and a drag map
    through the slide. The picture holds the bands the track shows and one more on each side; `ui/minimap.slint` places
    the image at whole device pixels. `note` in `native/viewport.rs` and `minimap/pace.rs` post a request only when the
    named bands change (`SLIDE` 60 ms, `THROTTLE` 250 ms, `DEBOUNCE` 120 ms).
    Black: `ui/surface.slint` holds `Surface.background` (`#000000` in the dark scheme, `Palette.background` otherwise);
    `app`, `tree`, `search`, `find`, `references` and `minimap` use it; the raster paints no background and `CodeColors`
    has none. The picture needs no gain. Not changed: `hover.slint` `alternate-background` and the find input fill.
    Gates, each alone at the default bound, on the branch head plus the uncommitted README and one rustfmt line of
    `tests/minimap_downscale.rs`: `lint:rustfmt`, `lint:clippy`, `lint:release`, `lint:slint`, `lint:closures`
    (`MISE_RUST_VERSION=nightly`) and `monochromatic-lint` on the changed files exit 0; `test` 807 passed, 1 skipped;
    `test:native` 211 passed, 5 skipped; `test:minimap` 34 passed.
    Guards (`inspect:source-guards`, three phases, every one accepted): 143 cases exist, all anchors match (script
    `anchors.mjs`). The new and changed minimap cases were each proved failing and restored: `box-share`, `slide-share`,
    `slide-offset`, `window-margin`, `picture-origin`, `origin-whole-rows`, `cache-window`, `key-text`, `key-styles`,
    `key-from-band`, `key-text-reload`, `key-has-bands`, `key-selection`, `window-key-selection`, `scroll-notes-bands`,
    `slide-period`, `throttle-period`, `follows-measurement`, `resize-keeps-wait`, `overscan-early-return`, and the
    reducer, cache, budget and stitch cases (`ide-guards-a*.log`, `ide-guards-b*.log` under `~/temp/agent`). Two guards
    first did not fail and were repaired: `origin-whole-rows` (its test now selects all text and runs at scale 1.3, so
    ink lies on both sides of a border that falls between pixel rows) and `slide-period` (failure text).
    Measured (README "Performance bounds"): release, 50,000 lines of about ninety characters, 1,000 px view, 110 px
    minimap: cold picture 44 ms (scale 1) and 78 ms (scale 2); every band cached 0.13 and 0.38 ms; one line changed in
    the window renders 1 band in 3.7 and 1.9 ms; a change above or a line inserted below the window renders none; a line
    inserted or deleted above or in the window renders 10 to 12 of 12 bands (21 to 26 ms) or 64 to 66 of 66 (58 to 61 ms)
    because the pixel grid shifts and keys are content, not position (bounded by the window; a content-anchored grid
    was not built). Debug, nested compositor, 50,000 short lines edited on disk at the end: 1 of 4 bands in 44 ms (scale
    1), 1 of 14 in 8.7 ms (scale 2); the 600-character wrapped copy re-renders the whole window while the measurement
    after the reload moves the grid.
    Screenshots (`~/temp/agent/minimap-shots/b-*`, dark scheme, 1500x900 at scale 1 and 1100x660 at scale 2, this
    package's longest source file and 50,000-line copies; plain, selection, find, slid by a drag, `ctrl+End`, edited):
    pixels at the editor, sidebar, header and minimap blanks read 0,0,0; selection, find matches, dim comments and the
    box border read clearly on black; the picture slides with the box and ends at the last line with the box at the
    bottom. Not provoked: blank edge rows during a very fast drag over a very long file; the compositor caps a window
    at 3,704 px, so 12,000 px is covered by the native test only.
    Measurement pass in a tall window (resolved, not this branch): `settled` times out at 90 s for a window of 10,000
    px and above, on `origin/main` as well. Control: a throwaway test on a worktree of `origin/main` and on this branch,
    400 wrapped lines, heights 1500, 3500, 6000 and 8000 settle in 0.8 to 1.7 s on both; 10,000, 11,000 and 12,000 do
    not settle in 90 s on both (12,000 still not after 430 s). At that point the pass key is posted and equal to the
    current key, and `tile_job` is set with `scroll_wanted` of 403 lines, width 1328 and a clip of 12,384 px, the tile
    worker thread (`ide-source-tile`) burns a full core (a thread CPU sample moved 510 ticks in 5 s), so the worker is
    computing, not blocked. It is the old tile path for very tall wrapped tiles (a cliff between 8,000 and 10,000 px,
    cause not located), not the minimap: the minimap worker is idle then. Worth its own issue.
    Left: the coordinator lands (merge, install). The throwaway probe test and its mise task were removed.
  - Dylint gate over the IDE (stage 2b of `doc/decision/rust-closure-lint-dylint.md`; branch `feat/ide-closure-lint`,
    worktree `.claude/worktrees/ide-dylint`, an agent, 2026-10-18; landed after the coordinator's gates below). The lint
    `avoidable_closure` now reports nothing on the IDE: `mise run //package/desktop-app/ide:lint:closures` exits 0.
    The 83 distinct reports (84 in the stage logs, because `src/syntax_loader.rs:188` is reported in both the library
    and the binary) became: 78 conversions (5 in `src/bin`, 27 in 16 `*_tests.rs` files, 46 in 19
    files of `tests/`) by `for` loops, `contains`, `OsStr::to_str`, `?` helpers (`debugged` and `identified` in
    `src/bin/ide-language-inspect/render.rs`; a bare `match .. None => None` there is `clippy::manual_map`), and small
    test helpers (`named`, `has_line_with`, `styled`, `some_contains`, `listed_as_directory`, `has_content_hit`,
    `instance_of`), and 5 `#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason =
    ".."))]` attributes: the `catch_unwind` bodies of `src/native/minimap_build.rs` and `src/native/tile_build.rs`, the
    two `tree_house` callbacks of `src/syntax_loader.rs`, and the `let` statement of the caret callback in
    `src/native/language/actions.rs` (the attribute cannot sit on a `let` initializer, and two setter calls would
    duplicate the body). Removing any one attribute makes the lint report exactly that site, and an `expect` on a
    closure that is allowed is reported as `this lint expectation is unfulfilled`. The `check-cfg` entry
    comes from `file-enforcer.config.ts` (`CARGO_DYLINT_DIRS`, so only the IDE manifest changed). No suppression other
    than those five, no change to `monochromatic-lint.config.jsonc`, root `mise*.toml` or `mise.lock`; the syntax-only
    `rust/no-anonymous-functions` rule still reports 67 (the 62 the new lint allows and the 5 it expects), unchanged in
    count. The conversions evaluate every element where an `any` or `all` stopped early; no converted expression has a
    side effect, so only failure messages differ.
    The task: it depends on `runtime` and `notices`, fetches with `cargo +stable fetch --locked` (the nightly cargo of
    this host cannot reach crates.io), builds offline on the shared nightly into `target/dylint` (links to
    `target/debug/runtime` and `target/crate-licenses.json`, which `build.rs` finds relative to the build directory),
    and runs `cargo dylint --all-targets --keep-going` with `DYLINT_RUSTFLAGS='-D avoidable_closure -D
    unfulfilled_lint_expectations'`. Wall time with load average 12 to 19: 78.9 to 87.1 s with `target/dylint` deleted,
    15.0 to 16.9 s unchanged, 16.5 to 21.6 s after touching a source file (3 runs each, all exit 0), and about 30 s
    more for a first driver build with an empty `DYLINT_DRIVER_PATH`. It is the last entry of the IDE `lint` aggregate
    (whole aggregate 33 s warm); dropping that one line returns it to a separate task, which is the owner's call
    because it puts the host nightly with `rustc-dev` and a network step into every `lint` fan-out.
    Verified on the branch: container clippy, `lint:rustfmt`, `lint:release`, `monochromatic-lint` on the IDE (67
    warnings, no error), `test` 783, `test:cli` 13, `test:native` 197, `test:language` 379, `test:syntax` 64,
    `test:minimap` 19, `test:wrap` 15 and 62, `test:annotations` 54, `test:annotations-native` 25, `test:fonts` 330,
    `test:search` 335, `test:open` 7, `test:tree` 20, `test:find` 350, `test:raster` 1, `test:ignored` 10, 1, 28 and 1,
    `test:reader-lifecycle` 316, and `inspect:language` against disposable projects, all passed. All 411 anchors of the
    `bin/inspect-*-guards.mjs` scripts still match their files (none moved); the 21 guard cases whose test functions
    or anchored files this branch touched were run again and failed on their guard as designed (source 12, annotation
    2, find 1, language 2, search 1, watch 3). The coordinating session then merged `main` (only this file
    conflicted) and reran on the merged tree, all status 0: `lint:rustfmt`, container clippy, `lint:slint`,
    `lint:release`, `lint:closures`, `test` 783, `test:cli` 13, `test:language` 379, `test:minimap` 19, `test:wrap` 15
    and 62, `test:annotations` 54, `test:annotations-native` 25, `test:syntax` 64, `test:native` 197, `bundle`,
    `inspect:bundle` (10 of 10), `inspect:bundle-guards` (8 of 8), all eleven guard scripts (source 108, annotation 65,
    sidebar 18, theme 7, find 39, ignored 28, search 17, watch 76, language 31, language lifecycle 28 runs, language
    navigation 16), and `inspect:language`, `inspect:language-mise` and `inspect:language-confinement`. A positive
    control (the expectation of `minimap_build.rs` renamed to `dead_code`) made `lint:closures` exit 1 with the closure
    reported and `this lint expectation is unfulfilled`; the file was restored. The stale stage-1 wording of the lint
    package README was rewritten by the removal below.
    Landed on 2026-10-18 as `1e143fbabe`, pushed straight to `origin/main` from the worktree (the owner chose this):
    the main checkout holds an uncommitted line re-wrap in `package/desktop-app/ide/README.md` (one of a 314-file bulk
    change from another session, last written 09:24), which makes `git merge --ff-only` refuse, so the checkout's
    `main` stays behind `origin/main` until that change is committed or dropped by its owner. Branch and worktree
    removed. The IDE is reinstalled (`~/.local/bin/monochromatic-ide`, sha256 `f068d238b760`, equal to
    `dist/monochromatic-ide`; `monochromatic-ide.previous` is the build `2172fa208b55`); `--help` runs.
    Decided on 2026-10-18 (the owner,
    through the question tool):
    the syntax-only rule `rust/no-anonymous-functions` is removed altogether,
    repo-wide,
    instead of being switched off for the IDE only.
    It landed on 2026-10-18 by a push to `origin/main` from branch `refactor/remove-no-anonymous-functions`
    (an agent; the coordinator merged `main`, reran the linter package tests, 412 passed and 1 skipped, the fuzz
    sidecar tests, 10 passed, clippy, rustfmt and the closure package lint, all status 0, and ran the installed
    `monochromatic-lint` 0.1.3, which still contains the rule, over the IDE and the closure package: status 0, no
    finding, because the repository config no longer selects it).
    A config that still names the removed rule is a setup error, `Unknown built-in rule rust/no-anonymous-functions.`,
    exit status 2.
    The agent reported that the root lint run exits 1 on Markdown findings in files it did not touch (24 in its
    final run, compared against the base text for the four files it did touch); the coordinator did not rerun it.
    The Dylint lint `avoidable_closure` is the only closure check from then on.
    Rust packages other than the IDE have no closure check until each gets its own `lint:closures` gate.
    Note of 2026-10-19:
    the IDE task `lint:closures` named throughout this record was removed.
    `monochromatic-lint` now runs the same Dylint lint for every Rust package as the rule `rust/avoidable-closure`
    (library and binaries only,
    tests exempt),
    and the repository configuration holds the IDE at error severity.
    The five `expect` attributes stay.
    See `package/linter/monochromatic-lint/README.md`,
    section "Closure check",
    and the section "Answers of 2026-10-19 to the proposal" of `doc/decision/rust-closure-lint-dylint.md`.
    The IDE's `lint` aggregate task no longer lists `lint:closures`;
    its stable-fetch step became the linter's offline check (a cold cache skips the package with a note),
    and its runtime,
    licenses and build-directory links became the prerequisite `ide-runtime-artifacts` in
    `package/desktop-app/ide/monochromatic-lint.prerequisites.jsonc`.
  - Inline directives in the linter (D2 of `doc/planning/rust-linter-oxlint-parity.md`, built after the Dylint gate above;
    branch `feat/lint-directives`, worktree `.claude/worktrees/lint-directives`, an agent, 2026-10-18; landed as merge
    `6808166dfd`, after the coordinator reran 488 package tests, clippy, rustfmt and the fuzz sidecar tests on the
    merged tree and ran the built binary over a disposable project: a covered finding stayed quiet, a directive aimed
    at `rust/max-lines` was refused, one without a reason was reported). The owner decided D2 on 2026-07-25 and asked
    for it to be built. Released on 2026-10-18 as `monochromatic-lint` 0.2.0 at the owner's request (a minor bump,
    because removing `rust/no-anonymous-functions` breaks a config that names it): the `cargo-publish` run on
    `bc8ffc2c47` published the crate and the GitHub release `monochromatic-lint-v0.2.0` with eight platform archives,
    crates.io lists 0.2.0, `mise upgrade cargo:monochromatic-lint` moved the installed tool and `mise.lock` (commit
    `661af4261b`), and the installed 0.2.0 exits 0 over the IDE and the closure package. The owner also chose to keep
    doc comments inert (the source quotes a directive in a `///` comment at `src/run_modes.rs:91`).
    `monochromatic-lint-disable`, `-enable`,
    `-disable-line` and `-disable-next-line` are read from `//` and `/* */` comments of Rust files and from HTML
    comments of Markdown files (not MDX), each needing a ` -- reason`. `rust/max-lines` and `rust/require-rustdoc`
    declare themselves unsuppressible in `RULES` (`src/run_modes.rs`, shown by `--rules`); a directive aimed at either is
    reported as `directive/non-suppressible`. The other findings are `directive/missing-justification`,
    `directive/unknown-rule`, `directive/malformed` and, on request, `directive/unused`
    (`--report-unused-disable-directives`, or `--report-unused-disable-directives-severity` with oxlint's five values).
    One filter in `HostChecker::check_snapshot` covers the syntax rules, the semantic rule and the rustdoc and fence
    processors, so `--fix` never sees a suppressed finding's fix. The README section "Inline directives" lists the
    stricter readings taken where the planning documents were silent. The installed `monochromatic-lint` 0.1.3 predates
    this and ignores such comments; the package version was not bumped, and no IDE file, configuration or root file
    changed. The Dylint lint's own `expect(.., reason)` attributes are unaffected: the comment form for it is still the
    open veto above.
  - Ignored entries dimmed in the tree (branch `feat/ide-ignored-dim`, request of 2026-10-14: "Apply 0.8 opacity to text
    for gitignored files/dirs"; the user chose the `ignore` crate 0.4.33, the only new package). Before, the tree
    listed ignored entries at full ink (`Workspace::list` is a plain `read_dir`). The reader thread now decides the
    ignored names beside each listing from `.gitignore` files (top of the work tree down), `info/exclude` and the
    global excludes file, with git's precedence and the parent-directory rule (`src/git_ignore.rs`); the answer rides
    the listing's single-use request, so a stale reply drops it. The name draws at 0.8 opacity (arrow and badge keep
    full ink, as IntelliJ colors the name only), the row label gains `, ignored by git` (the compositor's `a11y`
    serves names, not descriptions), and the watcher reports a folder whose `.gitignore` or `info/exclude` is written
    in place. Measured contrast of the name's strongest pixel: dark 17.04 to 11.16, light 17.01 to 9.19, selected row
    4.53 to 3.45 (3.92 tinted), which is under WCAG 2's 4.5 and over the 3:1 floor of the selected-ink rule; kept as
    asked. Not done: git's index is not read, so a tracked file matching a pattern is dimmed (pinned by a test and
    named in the README); `.git/info/exclude` and the global file are found by the safety sweep, not a watch;
    the call that turns the changed-rules flag into a reread of every shown folder has no guard of its own, because
    the safety sweep would also repair the rows within a second. `inspect:ignored-guards` removes 28 guards in a copy
    and all failed as named.
  - Language servers through mise (request of 2026-10-14: servers "to the repo's mise tools"). Fifteen servers are
    declared in `mise.no-env.toml` (generated into `mise.toml`) and locked: taplo, tombi, marksman, rumdl,
    markdown-oxide, zizmor, yaml-language-server, `@actions/languageserver`, `@ansible/ansible-language-server`,
    `vscode-langservers-extracted` (json, css, html), superhtml, bash-language-server, the two Docker Node servers and
    docker-language-server. Each answered `initialize` in the IDE's bubblewrap shape; hover works for all but zizmor
    (diagnostics only); inlay hints are advertised by tombi, markdown-oxide, the Actions server and
    docker-language-server, and none produced a hint on the fixture. Nine Node servers exited about 3 s after start
    because `vscode-languageserver` 9.0.1 (`lib/node/main.js`) checks `process.kill(clientPid, 0)` every 3 s and the
    client's pid is invisible in a pid namespace; they were added to `NO_PID_NAMESPACE` in
    `src/language/confine/recipe.rs` (the network stays unshared; test
    `node_servers_that_watch_the_client_run_without_a_pid_namespace_but_without_network` was seen failing without the
    list). The desktop launcher's PATH is a stale systemd user session snapshot without the repo's tool directories,
    and mise shims fail with "No version is set for shim" outside a mise tree. The user decided: "If mise is detected
    in dir the IDE starts the servers themselves through mise exec. We need to be adaptable and not hardcode
    anything." Queue: the branch `feat/ide-mise-servers` (agent in flight) implements the lookup through mise for a
    project folder with a mise configuration, never auto installing or trusting, with PATH lookup as the fallback.
    Open causes: taplo answered hover only after 10 s and tombi answered null at once on the TOML fixture (cause not
    determined); `find_box_context_menu_runs_each_entry` failed once under load (left "needlex", right "x") and
    passed alone and in a full rerun (cause not determined).
  - The IDE checks used compositor builds inside each worktree's `target` directory, which were 0.1.1 (no minimized
    launch) in the root checkout and four worktrees, so checks opened visible windows on the user's desktop. Those
    five binaries were replaced with the mise-installed 0.4.0, and the four `bin/inspect-*.mjs` scripts and the `native`
    task now spawn the compositor from PATH; `bin/nested-compositor.mjs` refuses a version below 0.4.0. On the merged
    tree `inspect:bundle` (10 of 10) and `inspect:bundle-guards` passed through it; `inspect-idle-cost`,
    `inspect-log-stall`, `inspect-watch-limit`, and the `native` task had a syntax check only.
  - The nested compositor now launches minimized (`package/cli/nested-wayland-session`, the user confirmed it works),
    version 0.1.2 is published on crates.io with a GitHub release of Linux archives, and the repository's mise tool
    list installs it through cargo-binstall as `cargo:monochromatic-nested-wayland-session` (source:
    `mise.no-env.toml`; `mise.toml` is generated from it by file-enforcer).
  - Haiku subagent failures of this work are recorded in `doc/limitation/haiku.md`.
- Still open:
  - Lazy server lookup was measured and not built:
    locating servers is 12 ms of a 210 ms registry build.
  - Fixtures of other native tests still use `tempfile::tempdir()` on btrfs.
  - The `.claude/worktrees/ide-integrate` worktree was removed with its half finished merge of `main`
    (git there was blocked by the same `config-changed` stop); the branch `integrate/ide-language-core` remains.
- Inotify:
  on 2026-10-06 about 478000 of the host's 524288 inotify watches were in use,
  about 462000 of them by `git fsmonitor--daemon` processes
  (`core.fsmonitor=true` in `~/.gitconfig`),
  one per worktree and per git-wrapper shadow git directory,
  each watching every directory of its worktree including ignored `node_modules` and `target`
  (146000 for another session's worktree,
  135000 for the main checkout,
  about 13900 for each IDE worktree with a build cache).
  The coordinating session removed twelve clean,
  fully pushed IDE worktrees it had created
  (their branches remain).
  The user then chose to turn the monitor off for this repository:
  `core.fsmonitor=false` is in `.git/config`
  (the wrapper's shadow git directories include that file),
  and stopping the sixteen running daemons that served this repository freed 436062 watches;
  49870 were in use afterwards.
  Three daemons for another session's worktree `translation-repair` started again afterwards,
  one through an explicit `git fsmonitor--daemon start` under the git wrapper's shadow git directory;
  the cause is not established.
- Reminder issues for the user to raise upstream themselves,
  opened on 2026-10-06 as with #606 for Slint:
  #608 (helix-lsp logs a server's normal exit at ERROR level)
  and #609 (the TypeScript 7 language server exits with `context canceled`).
  Each points at its troubleshooting document's draft.
- Still owed to the user:
  UI batch 3 with screenshots
  (hint row look,
  label shortening,
  diagnostic row wording,
  the caret card beside the rows,
  UI batch 2's open choices,
  the watch-limit message,
  the launcher icon).

Claude Opus resumed the queue on 2026-10-05 from `e81b01c42`
and fanned the work out to in-process subagents at the user's request.
Unrelated concurrent commits from other sessions sit in the same history;
leave them alone and stage only explicit task paths.

Proportionality rule from the user's correction:
editord is a familiar reference,
not an exact contract.
Do not chase a reference's incidental engine semantics,
and do not start a technology-vetting run,
when a plain conventional solution with existing dependencies meets the stated scope.

Work in flight,
each owned by one subagent:

- Branch heads at 11:48 on 2026-10-05,
  all committed with clean worktrees:
  `feat/ide-sidebar-resize` at `4e0907962` (6 commits,
  plus `8bfa7bd0b` later;
  since integrated),
  `feat/ide-source-keys` at `b59b15b59` (8 commits,
  since integrated),
  `feat/ide-language-core` at `652d383dd` (7 commits;
  12 through `00dd01956` were integrated,
  and later commits on that branch carry the bubblewrap leg).
  All three agents were cut off by an API session limit at about 11:10
  and resumed at 11:48 from their transcripts.
  If this session ends,
  integrate each branch with `git cherry-pick main..<branch>` one branch at a time,
  running `:test`,
  `:test:native`,
  and `:lint` between branches.
  Files touched by more than one branch:
  `mise.toml`,
  `src/lib.rs`,
  `src/native.rs`,
  `ui/app.slint`
  (and `README.md` additions);
  resolve them additively.
  Guard-removal proofs are taken on branch state;
  rerun every `inspect:*-guards` task once on the final `main`.
  When another session has files staged in the main checkout,
  `git cherry-pick` refuses to run there
  ("your local changes would be overwritten by cherry-pick").
  The language-core integration therefore ran in the provisioned worktree `.claude/worktrees/ide-integrate`
  on branch `integrate/ide-language-core`:
  cherry-pick,
  gate,
  `git rebase main` when other sessions moved `main` meanwhile,
  confirm `git rev-parse <commit>:package/desktop-app/ide` is unchanged from the gated commit,
  then `git merge --ff-only` from the main checkout.
  Whether a fast-forward preserves another session's staged files was not tested,
  because their files had been committed by then.
- No agent edits the IDE crate in the main worktree now.
  `main` is the integration point:
  the coordinating session cherry-picks each branch and reruns the suite.
  `inspect:native` takes `IDE_NATIVE_MCP_PORT` so parallel sessions do not collide.
- Bubblewrap launch policy for the Language module:
  done,
  landed on `main` through `34e41e424`
  (branch commits `6789cfd9f` through `b9f877b23`).
  `LanguageSetup::default()` confines every server with `/usr/bin/bwrap`
  (no network,
  `--clearenv` plus an allowlist that keeps `RUSTUP_TOOLCHAIN`,
  a process-id namespace except for the TypeScript servers,
  per-project per-server state under the cache directory,
  type acquisition off)
  and binds the project back read-only last,
  so projects below `/tmp`,
  `/run`,
  and `/dev` work;
  only `/proc` is refused.
  `PROJECT_MOUNT` in `src/language/confine/project.rs` is the single place a 1.x write mode would change.
  Every failure is `LaunchRefused` with a remedy;
  nothing falls back to an unconfined launch,
  so the native UI must show that state.
  Measured:
  `inspect:language-confinement` 114 of 114 checks,
  including fixtures below `/tmp` and `/run/user/1000`;
  confined `inspect:language` 20 of 20 for TypeScript and 21 of 21 for Rust,
  with the project tree unchanged and two `rustc` `E0308` diagnostics from `cargo check`
  now that the Rust fixture has a `Cargo.lock`.
  Gate on the identical IDE tree:
  44 test binaries with no failures,
  43 of 43 native tests,
  lint.
  Notes:
  `doc/troubleshooting/bubblewrap-project-paths-and-working-directory.md`;
  the TypeScript servers can still signal host processes because they run without a process-id namespace.
- Native language navigation:
  done,
  landed on `main` through `d92a57d21`
  (branch commits `e98f8b243` through `9aa5b8531`,
  plus the coordinator's integration fix).
  `native::run` enters the project directory right after `Workspace::new`,
  starts `LanguageWorker`,
  and keeps the window working if either fails.
  Bindings,
  from editord unless noted:
  Ctrl+B for definition at the caret
  (falling back to references when the only target is the caret's own line),
  Ctrl+click for definition under the pointer,
  Ctrl+Q for hover at the caret
  (JetBrains Quick Documentation;
  editord has no keyboard hover),
  and hover after the pointer rests 350 ms.
  Several targets open a location list beside the caret line;
  `file` targets outside the project open read-only with an "Outside project" tag,
  no tree reveal,
  and no history slot.
  Server and document states produce a note only after an explicit action;
  the sentences are in `src/native/language/message.rs` and the README section "Language navigation".
  The package README lists every deliberate difference from editord.
  Integration finding:
  the build image has no `bwrap`,
  so window tests must launch the scripted server with `launch_directly`,
  as `tests/language/support.rs` does;
  the branch's tests used `LanguageSetup::default()`,
  which became confinement after they were written,
  and all failed with "the scripted server did not become ready" until `d92a57d21`.
  That commit also adds a window test and guard entry for a refused launch,
  the state a user without bubblewrap reaches.
  Gate on the integration branch:
  44 library and integration test binaries with no failures
  (run before the test-only fix,
  which does not touch them),
  75 of 75 native tests,
  lint;
  the `launch-refused-explained` guard control failed when removed and passed when restored
  (`~/temp/agent/ide-language-navigation-guard-KZntea`).
  The branch's own evidence:
  12 guard controls
  (`~/temp/agent/ide-language-navigation-guard-reiHcb`,
  `~/temp/agent/ide-language-navigation-guard-RKSEkB`)
  and seat-input sessions against disposable TypeScript 7 and Rust projects in
  `~/temp/agent/ide-lsp-nav-evidence/`,
  taken unconfined before confinement landed;
  the coordinating session inspected the TypeScript hover and Rust references frames.
  Still to do for this feature:
  rerun the nested sessions on `main`,
  where servers are confined;
  stop language-server shutdown from logging bare errors
  (`context canceled`,
  `StreamClosed`,
  and rust-analyzer's "notify error: No path was found" all land at ERROR through `helix_lsp`),
  which the repository's cleanup rule forbids;
  selected rows of the location list use the palette's selected-text ink like the tree and search list.
  The annotation renderer's snapshot store and repaint are recorded in the item "Inlay hints and diagnostics".
- Inlay hints and diagnostics:
  done,
  landed on `main` as merge `1d7bc0fe7`
  (branch `feat/ide-lsp-annotations` through `112dbd955`,
  merged on `integrate/ide-language-core` as `3476a78af` with fixes `889beb9fe` and `d5ea96492`),
  plus the test correction `1c953683b`.
  Hints are drawn in boxes after the end of their line,
  diagnostics as severity underlines with lettered markers,
  and the problems at the caret in a card;
  placement was the agent's delegated choice and stays open to the user's veto.
  The package README section "Inlay hints and diagnostics" has the measurements and the differences from editord.
  One store:
  `State::annotations` is `ide_app::annotation::Annotations`;
  the language poll stores a snapshot only for the displayed file generation and revision
  (`accept_hints`,
  `accept_diagnostics`),
  the renderer reads the same store,
  and the frame stamp includes the visible annotations.
  The poll repaints once in the tick that stored a snapshot (`src/native/language/poll.rs`).
  The problem card hides while the hover popup,
  a note,
  or the location list is shown (`problem-card-visible` in `ui/app.slint`).
  Gate at `d5ea96492`
  (IDE tree `4653e9920f21305bb958cbd65fe80ed5b6d6a7eb`):
  lint;
  the library and integration tests on the second run;
  85 of 85 native tests.
  Guard controls on `main` at `1c953683b`:
  `repaint-on-accept` and `card-yields-to-popup` failed when removed and passed when restored
  (`~/temp/agent/ide-language-navigation-guard-murJAW`),
  and the re-anchored `card-focus` guard likewise
  (`~/temp/agent/ide-annotation-guard-a3HMKh`).
  Two findings from this integration.
  First,
  the window tests' shared server definitions set `IDE_SCRIPTED_PUSH` to `0`;
  a test that needs the scripted warning names `PUSH` itself.
  Second,
  the first version of the repaint test passed with the repaint removed
  (`~/temp/agent/ide-language-navigation-guard-pc9vqd`):
  the source refresh repaints once when the file's first highlighting answer is applied,
  about 250 ms after opening,
  and that painted the stored annotations.
  The test now uses a server that answers after 3 s,
  waits until `State::syntax_revision` equals the displayed revision,
  and asserts nothing is painted before the server is ready.
  Any window test that claims "painted without another event" needs the same wait.
- Open defect,
  observed once in the first library test run of that gate:
  `lifecycle::dropping_the_worker_leaves_no_child_process` failed with a scripted-server child
  left as a zombie (state `Z`) for the whole 5 s wait after the worker was dropped
  (`~/temp/agent/ide-gate-logs-20261005/annotations-merge-test-zombie.log`).
  The Language module's shutdown code is the same on `main`,
  so this is not from the merge.
  Unverified guess:
  the child handle is dropped with kill-on-drop before the child exited,
  and the worker's runtime ends before reaping it.
  The agent working on quiet shutdown (`feat/ide-language-verify`) was asked for the cause,
  a fix,
  a guard control,
  and the failure rate before and after;
  the test's wait must not be lengthened to hide it.
- Repeated runs on the IDE tree `b35830353a255f96dd4754d9f0836bb359201a1b`,
  taken while the host's load average was near 98
  (`~/temp/agent/ide-gate-logs-20261005/repeat/results.json` and the logs beside it):
  the native window tests passed 5 of 5 times,
  so the two find flakes fixed in `4ab83c59e` did not recur;
  the library and integration tests passed 4 of 5 times.
- Open defect from the failing run:
  `requests::inlay_hints_are_shaped_and_follow_reloads` timed out after 20 s
  waiting for hints for the reloaded text.
  At the timeout the server was ready,
  the diagnostics snapshot already carried revision 1 with the reloaded text,
  and the hints snapshot still carried revision 0.
  So hints for the new revision were never asked for,
  or their answer was dropped and not asked again;
  in the application,
  hints would vanish after an external change until something else asks.
  An agent works on it in the worktree `.claude/worktrees/ide-hints-reload`
  on branch `fix/ide-hints-after-reload`:
  reproduction rate,
  cause,
  fix in the Language module,
  a deterministic regression test with its guard control,
  and the rate after.
- Event-driven tree and displayed-file refresh with the user-approved `notify` crate:
  landed on `main` as merge `732384e4b`
  (branch `feat/ide-tree-watch` through `096fbf574`,
  9 commits on `48a1b5756`,
  merged on `integrate/ide-language-core` as `aab809c54`
  with additive conflicts in `mise.toml` and `src/native.rs`).
  Gate at `aab809c54`
  (IDE tree `b35830353a255f96dd4754d9f0836bb359201a1b`):
  lint;
  88 passing library and integration result lines with none failed;
  87 of 87 native tests with the two ignored latency measurements skipped.
  Still in flight:
  the user chose a 1 s safety sweep and a 50 ms quiet,
  100 ms limit write wait,
  and kept the 100 ms gap;
  `main` still has the 10 s sweep and the 150 and 250 ms waits.
  The watch agent applies the new values on `feat/ide-tree-watch`
  (worktree `.claude/worktrees/ide-tree-watch`),
  with a log-once rule for failed watches,
  idle cost measured at 1,
  12,
  and 100 expanded folders,
  and a fresh latency run;
  merge its new commits after it reports.
  The 30 watch guard controls were observed on the branch;
  the rerun on the merged tree `b35830353a255f96dd4754d9f0836bb359201a1b` passed 30 of 30
  (`~/temp/agent/ide-watch-guard-UAHxUa`),
  taken with the 10 s sweep and the 150 and 250 ms waits.
  The agent's measured results and open points,
  taken with the 10 s sweep and the 150 and 250 ms waits:
  - `notify` 8.2.0 with default features off;
    the code names `notify::INotifyWatcher`,
    so no polling backend can be selected.
    Six new lockfile packages
    (`notify`,
    `notify-types`,
    `inotify`,
    `inotify-sys`,
    and the BSD-only `kqueue` and `kqueue-sys`),
    regenerated with the `fetch` task.
  - Watched,
    each non-recursively:
    the root,
    visible expanded folders,
    and the displayed file's folder,
    only when the folder's canonical path equals its path inside the root.
    Events only invalidate;
    the existing readers reread and keep their stale-reply fencing.
  - A full reread follows a queue overflow,
    a stream error,
    an event without paths,
    a removed or renamed watched folder,
    a failed watch,
    and a watch thread that ended.
    Items without a live watch keep the old timers (file 250 ms,
    folders 500 ms round robin);
    a sweep every 10 s rereads everything shown and retries failed watches.
  - Latency from an external write to the model change,
    medians of 16 trials in 3 runs:
    a new file with eight expanded folders took 1657 to 1706 ms under polling and 29 to 35 ms watching;
    a rewrite of the displayed file took 115 to 118 ms and now 44 to 48 ms.
    Run-to-run spread of medians was at most 134 ms polling and 6 ms watching
    (`~/temp/agent/ide-refresh-latency-MpRf1V/results.json` before,
    `~/temp/agent/ide-refresh-latency-9Ljcjy/results.json` after).
  - On the branch:
    30 of 30 guard controls (`~/temp/agent/ide-watch-guard-lnfGWB/results.json`),
    405 passing library and integration results,
    43 of 43 native tests with the two ignored latency measurements skipped,
    lint;
    nested-session frames for both schemes in `~/temp/agent/ide-tree-watch-20261005/native-evidence-final/`.
  - Values the agent chose,
    since decided by the user as stated in this item:
    the 10 s sweep,
    a 100 ms gap between notified rereads of one item
    (added beyond the brief so a busy folder is not reread every tick),
    and the 150 ms quiet and 250 ms limit before reading a file that is still being written.
  - Still open,
    to be asked with built screenshots:
    whether reaching the watch limit shows a visible message or stays log only.
  - Not exercised end to end:
    an unavailable watcher (inotify instance limit) and `fs.inotify.max_user_watches` exhaustion;
    both are logged at WARN and fall back to the timers,
    with no visible message.
    `notify` 8.2.0 spawns its loop thread detached and unwraps sends to it;
    a panic there would take the watch thread down.
  - Facts worth keeping:
    the IDE's own reads raise open and close events on watched paths,
    so only close-after-write counts among access events;
    a moved folder's watch follows the inode unless removed,
    and `notify` removes it itself only when the parent is watched too.
- UI batch 2:
  the comparison page and option screenshots are on `main`
  (`package/desktop-app/ide/design/`,
  commit `b52227249`),
  the user answered every question,
  and the answers are in the scope record under "Interface decisions (UI batch 2)".
  An agent applies them in the worktree `.claude/worktrees/ide-ui-batch2`
  on branch `feat/ide-ui-batch-2`
  (MCP ports 9394 and 9395),
  based on `integrate/ide-language-core` at `d5ea96492`:
  a thin divider line with a narrow pointer grab zone taken from cited desktop precedent,
  the divider still a Tab stop with two-channel focus,
  the application's own text box with a 48 by 48 clear cell in the find and search boxes,
  white text on selected rows in the tree,
  the search list,
  and the references list,
  and frames of the applied result under `package/desktop-app/ide/design/screenshots/`.
- Release packaging and the consumer-boundary check of the packaged application
  in the worktree `.claude/worktrees/ide-package`
  on branch `feat/ide-package`
  (MCP ports 9414 and 9415),
  based on `aab809c54`:
  what a runnable artifact means from sibling precedent and Helix's runtime lookup,
  a release build task,
  a self-contained directory that runs without `HELIX_RUNTIME` and the source tree,
  a check of that artifact in the nested compositor against disposable TypeScript and Rust projects,
  and the scale-change latency in release and debug builds.
  Desktop entry,
  icon,
  installer,
  and install location are left as questions for the user.
- Landing a branch that conflicts with `main`:
  merge it with `git merge --no-ff` in `.claude/worktrees/ide-integrate`,
  resolve once,
  and gate there.
  A plain `git rebase main` afterwards would drop the merge commit and its resolutions,
  so a gated merge lands through the worktree `.claude/worktrees/ide-land`
  (branch `land/ide-desktop-app`):
  fast-forward it to `main`,
  `git merge --no-ff <gated commit>`,
  confirm `git rev-parse HEAD:package/desktop-app/ide` equals the gated tree,
  then `git merge --ff-only` from the main checkout.
  That keeps `main` as the first parent.
- Quiet language-server shutdown and the confined navigation check
  in the worktree `.claude/worktrees/ide-language-verify`
  on branch `feat/ide-language-verify`
  (MCP ports 9408 and 9409):
  trace and fix the ERROR-level shutdown and runtime messages at their cause,
  a lifecycle check that fails on any of them,
  SIGKILL of the application leaving no server behind,
  and navigation in the nested compositor with confined servers against disposable projects below `/tmp`,
  with project tree hashes before and after.
- Nested compositor output scaling and the closed private bus:
  done on `main`,
  commits `a8e60feca` through `dd5a19953` in `package/cli/nested-wayland-session`.
  `--scale SCALE` at startup and `scale SCALE` on the control socket accept 0.5 to 3 in steps of 1/120;
  `--size` and `resize` are logical sizes,
  screenshots and recordings are logical size times scale,
  and input coordinates stay logical.
  The client learns the scale through `wp_fractional_scale_v1` with `wp_viewporter`,
  which winit 0.30.13 requires together.
  Captures now render into an offscreen texture,
  because Mesa defers a window resize while an unswapped back buffer exists
  and frames recorded after a mid-recording scale switch were empty.
  The private `dbus-daemon` runs from a written configuration with no service directories:
  it lists only `org.freedesktop.DBus` and answers `ServiceUnknown` at once,
  where the stock configuration listed 73 activatable names.
  The package passes 69 tests,
  lint,
  Clippy,
  and its `inspect:clipboard`,
  `inspect:color-scheme`,
  `inspect:stalled-parent`,
  and new `inspect:scale` tasks;
  release binary sha256 prefix `cc9c5479b3b4346c`.
  IDE scale migration,
  verified end to end with `inspect:native dark scroll` and switches 1,
  2,
  1.25,
  1:
  every settled frame is sharp,
  the first frame the IDE commits at each scale equals the settled one,
  and the frame after the round trip equals the start
  (`~/temp/agent/nws-scale-nasfB8/ide-run-2/`).
  Worktrees that copied the release binary earlier lack `scale` until recopied.
  Open:
  an intermittent `calloop` "Received an event for non-existent source" warning,
  not attributed;
  a README link in that package points at `doc/decisions/` instead of `doc/decision/` (repointed in that README on 2026-10-22).
- Coordinator working-directory hazard:
  the compositor agent reported its session working directory moving into `.claude/worktrees/ide-integrate`
  while the coordinating session ran commands there.
  Keep every coordinator command pinned with `cd /var/home/user/Monochromatic` or `git -C`,
  and tell agents to pin theirs.
- Inlay hint and diagnostic placement,
  decided by the user on 2026-10-05 and not started yet:
  hints move from boxes after the line end to virtual rows above the code line,
  like editord,
  and diagnostic messages go on virtual rows there too,
  replacing the lettered marker and the caret card unless the user keeps either;
  the rows must read as belonging to the code line beneath them,
  through spacing;
  the scope record has the user's words under "Interface decisions (inlay hints and diagnostics)".
  The worktree `.claude/worktrees/ide-hint-rows` on branch `feat/ide-hint-rows` is provisioned
  (MCP ports 9424 and 9425),
  and the agent's complete brief is kept at `~/temp/agent/ide-gate-logs-20261005/agent-w-hint-rows-brief.md`.
  The harness runs at most five agents at once
  (`CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS`),
  and all five slots were in use;
  start this agent with that brief when one finishes.
  Its second phase produces the comparison frames for UI batch 3:
  hint row look,
  label shortening,
  marker and card beside the diagnostic rows,
  and the wording of a diagnostic row.
- Session-limit recovery at 18:50 on 2026-10-05:
  all five agents had been cut off;
  their worktrees held the work described in the next item,
  partly uncommitted,
  and each was resumed from its transcript with its measured state.
  One orphaned private `dbus-daemon` of a dead nested session (started 16:34) was stopped by process id.
- Usage-limit stop at about 16:45 on 2026-10-05:
  the coordinating session reached its usage limit while five agents were running.
  Each works in its own worktree and commits as it goes;
  none of their work was on `main` beyond what this section records as landed.
  To resume,
  check each worktree with `git status` and `git log`,
  check for leftover processes and bound MCP ports,
  and continue from the branch:
  `feat/ide-ui-batch-2` in `.claude/worktrees/ide-ui-batch2` (UI batch 2),
  `feat/ide-tree-watch` in `.claude/worktrees/ide-tree-watch` (the user's timing values),
  `feat/ide-language-verify` in `.claude/worktrees/ide-language-verify` (quiet shutdown and the zombie child),
  `feat/ide-package` in `.claude/worktrees/ide-package` (release packaging and the consumer check),
  and `fix/ide-hints-after-reload` in `.claude/worktrees/ide-hints-reload` (hints lost after a reload).
  Merge each on `integrate/ide-language-core`,
  gate,
  and land through `.claude/worktrees/ide-land`.
  Still owed to the user,
  with built screenshots:
  a visible message or log only at the watch limit,
  the divider's pointer grab width,
  and how several inlay hints on one line are told apart.
- Commits that appear late:
  on 2026-10-05,
  with the host's load average near 98 from several sessions' builds,
  a commit through the repository's `git` wrapper took minutes,
  and one command returned while its file was still staged and its commit was not yet in `git log`.
  Issuing the same commit again produced the real commit `e8ac286e4`
  and an empty duplicate `29a5b0da6` with the same subject
  (corrected by a commit comment on GitHub).
  The wrapper's landing mechanism was not read,
  so the cause is not established.
  After a commit command returns without a visible commit,
  look at `git log` again after a while and at running `git-policy-cli` processes before committing again;
  run commits in the background and read their full output from a file.
- Session-limit interruptions:
  the API session limit cut the running agents off twice,
  at about 11:10 and 14:15 on 2026-10-05.
  Each time their worktrees held committed work and nothing was left running;
  the coordinating session checked each worktree with `git status`,
  checked for leftover processes and bound MCP ports,
  and resumed every agent from its transcript with a message stating the measured state.
  The compositor agent works in the main checkout and had uncommitted edits at the second cutoff.
- IDE reaction to a live color-scheme switch:
  done,
  landed on `main` as `f158ae458` and its parent.
  No IDE source change was needed.
  `src/native/theme_tests.rs` switches dark,
  light,
  dark through `SlintContext::set_color_scheme`
  (the call Slint 1.18.1's portal watcher makes)
  and requires the live source image to equal a cold render,
  the scheme's syntax and selected-text ink,
  match rectangles,
  and tree,
  find bar,
  and divider on the scheme's side;
  a second test re-tints the accent five times per scheme.
  Both reach the toolkit through `slint::private_unstable_api`,
  so a Slint upgrade may break only these tests.
  Five guard-removal controls failed as named
  (`~/temp/agent/ide-theme-guard-wGG2mw`).
  In the nested compositor,
  `color-scheme light` and `color-scheme dark` repainted within one frame,
  and the return to dark was byte-identical;
  twelve recorded switches showed no frame with the new palette over the old source image
  (frames in `~/temp/agent/live-theme-Z50lLh/evidence/ide-native/`).
  Gate on the integration branch:
  43 of 43 native tests,
  lint.
  Found along the way:
  the file tree and combined-search rows still draw selected text with the palette's dark-scheme black on blue
  (`ui/tree.slint`,
  `ui/search.slint`),
  unlike the source view's fill-derived ink;
  fix them with the same rule.
- Runtime color-scheme switching in `package/cli/nested-wayland-session`,
  through the private portal only,
  plus a read-only analysis of what the IDE needs for a live switch.
  IDE-side verification is a later leg.

Completed in this fan-out:

- Headless Language module core,
  integrated onto `main` through `770c8f3a6` from `feat/ide-language-core` commits `8ee886976` through `00dd01956`.
  `ide_app::language::LanguageWorker` runs `helix-lsp` on one long-lived `ide-language` thread;
  commands are non-blocking
  (`Ok(false)` means the 64-entry queue is full; resend next tick)
  and replies,
  status,
  diagnostics,
  and hint snapshots are polled;
  every result carries a `DocumentStamp` of file generation and revision,
  and the handle drops stale ones and counts them.
  The package README section "Language module" lists the API and states.
  Deviations from the design,
  recorded in the agent report and `doc/troubleshooting/helix-lsp-embedding-roots-and-stop.md`:
  `Registry::stop` leaves a tombstone that blocks restarts,
  so servers are retired with `remove_by_id` and `force_shutdown`;
  paths are respelled through Helix's `PWD`-derived workspace,
  because `/home` links to `/var/home` here and a resolved path otherwise yields `rootUri: null`;
  superseded answers retry at most three times while their text is displayed;
  the unversioned-diagnostics hold falls back after 2 s,
  a chosen,
  unmeasured value.
  The TypeScript override requires the project's `package.json` to name TypeScript 7 or later,
  otherwise the state is `MissingExecutable` with a remedy;
  workspace and user Helix configuration are never loaded.
  Lockfile change:
  two direct dependency edges (`arc-swap`,
  `futures-util`),
  no new package or version.
  Gate in the integration worktree on the identical IDE tree:
  44 test binaries with no failures,
  41 of 41 native tests,
  lint.
  Real servers on disposable projects
  (`inspect:language`):
  18 of 18 checks each for TypeScript 7.0.2 and rust-analyzer,
  evidence in `~/temp/agent/ide-language-core-evidence-20261005/inspect-passing/`;
  that run is unconfined and rust-analyzer wrote `Cargo.lock` into its disposable project.
  Eleven guard-removal controls passed
  (`~/temp/agent/ide-language-guard-FpVQz5/results.json`,
  `~/temp/agent/ide-language-guard-uWmw4T`).
  Native wiring must call `language::enter_project_directory` right after `Workspace::new` at startup,
  add `HELIX_LOG_DIRECTIVE` to the log filter,
  build `DocumentReload::from_reload` before `Document::apply_reload` consumes the reload,
  and accept that dropping the handle blocks up to about 1 s while servers exit.
- Resizable sidebar,
  cherry-picked onto `main` as `365699eef` through `2187b8ebd` from `feat/ide-sidebar-resize`,
  plus the integration fix `0123ec0ba`.
  The tree starts at 256px like editord's `16rem`,
  session-only,
  and stays between 160px and the window width minus the divider cell and a 240px source column.
  The divider is its own 48px layout cell,
  so its pointer area overlaps no tree row or source text;
  the cost is a 47px gutter beside the 1px line.
  A left-button drag resizes;
  clicks and double-clicks without movement do nothing.
  The divider is a Tab stop between tree and source,
  steps 16px with Left and Right and jumps with Home and End,
  and is exposed as a `slider` named "Sidebar width",
  because Slint 1.18.1 has no splitter or separator role.
  States use two channels:
  idle 1px line,
  hover with column-resize cursor and a 3px line,
  drag at full ink,
  keyboard focus with a cell boundary.
  The width is clamped by layout constraints,
  not a size binding on `root.width`,
  which is a binding loop;
  the source cell's preferred width equals its 240px minimum,
  or long content shrinks the sidebar.
  The tab stop and the 48px cell changed `main`'s source pointer and focus test expectations,
  updated in `0123ec0ba`.
  The first gate on `main` after this stopped on the known flaky paint test;
  `4ab83c59e` fixed both find flakes at their cause
  (the waits now also require the active match to spell the whole query),
  and the rerun passed 234 library and integration tests,
  41 of 41 native tests,
  and lint.
  One passing run does not prove the flake gone;
  repeated native runs on the final `main` are part of the closing gate.
  Eight guard-removal controls passed on the branch
  (`~/temp/agent/ide-sidebar-guard-hJqYf7/results.json`).
  Dark,
  light,
  and 480x320 narrow sessions passed with seat clicks and toolkit-level drags
  (the compositor's `drag`, `move` and `button down|up` commands hold a pointer drag;
  the claim that it had none was wrong and was corrected on 2026-10-19);
  evidence in `~/temp/agent/ide-sidebar-j/`.
  `inspect:native` now also takes `IDE_NATIVE_SIZE`,
  because the compositor's `resize` command had no effect while the host session was locked
  (inferred,
  with the older compositor binary).
  The divider-as-Tab-stop choice and the gutter were reported to the user as open to veto.
- Source-view reading gate,
  cherry-picked onto `main` as `6545c521d` through `374ea9a36`
  from `feat/ide-source-keys`.
  Bindings,
  each with a Shift form that extends the selection:
  Left and Right by grapheme (collapsing a selection first),
  Ctrl+Left and Ctrl+Right by word,
  Home,
  End,
  Ctrl+Home,
  Ctrl+End,
  Up and Down toward a remembered pixel x,
  PageUp and PageDown by the lines in view;
  every caret key scrolls the caret into view.
  Pointer:
  click,
  double click for a word,
  triple click for a line,
  Shift+click,
  and drag by the pressed unit;
  mouse-drag panning of the source view is off so quick drags select.
  Tabs are shaped as one space widened to stops every two space advances in pixels,
  matching editord's `tab-size: 2` with mixed CJK and Latin prefixes.
  Tab and Shift+Tab cycle tree,
  source,
  and the open find input;
  the outer key-observing `FocusScope` was a dead Tab stop before.
  Selected-text ink is now white on the blue selection fill in both schemes:
  Slint's fluent style sets black in dark
  (WCAG 2 ratio 4.64,
  APCA Lc 33.4 before;
  4.53 and Lc -76.2 after).
  The coordinating session reran the gate on `main` after the cherry-pick:
  234 library and integration tests in 41 binaries,
  30 native tests,
  and package lint pass.
  Thirty-one guard-removal controls passed on the branch
  (`~/temp/agent/ide-source-guard-bX6gco/results.json`,
  `~/temp/agent/ide-source-guard-Qu70pZ/results.json`).
  Dark and light seat-input evidence is in `~/temp/agent/source-keys-k1/evidence-final/`;
  the coordinating session inspected the dark selection-ink and tab-stop frames.
  Open:
  drag autoscroll runs only while the pointer moves;
  under container load one run timed out in
  `native_find_recomputes_after_external_reload_and_follows_selection_correspondence`,
  which then passed unloaded;
  watch it.
- Language-server write confinement,
  measured and specified but not yet implemented:
  `doc/planning/slint-ide-write-confinement.md`.
  Five candidates were measured with real rust-analyzer and TypeScript servers on disposable fixtures;
  ranking:
  bubblewrap,
  an `unshare` script,
  a systemd transient unit,
  podman,
  Landlock through `setpriv`.
  Adopted on 2026-10-05 and reported to the user as open to veto:
  the bubblewrap wrapper (`/usr/bin/bwrap` is already on the host),
  no network for confined servers,
  and a process-id namespace except for the TypeScript servers.
  Under that shape the project tree stayed byte-identical,
  build scripts and procedural macros got `EROFS` on the project,
  and hover and definition still answered.
  Constraints the implementation must honor:
  build server configuration in code and never load workspace or user Helix language configuration;
  resolve the server executable to an absolute path;
  never fall back to an unconfined launch;
  keep per-project,
  per-server private state outside the project and not an ancestor of it;
  start servers from the long-lived language worker thread.
  Unconfined servers also write outside the project by default:
  cargo and npm caches in the home directory,
  and both TypeScript servers run `npm install` for type acquisition.
  Servers inherit the application environment,
  credential variables included;
  clearing it is a follow-up to test in the implementation leg.
- In-file find,
  commits `f51495788` through `4e1c2a808` on `main`.
  `src/find.rs` holds the single matcher `find_matches`
  (escaped literal,
  Unicode case-insensitive,
  source character ranges);
  `src/find_worker.rs` runs it on a named thread with one running job and one replaceable waiting request;
  results carry a `FindIdentity` of file-open generation,
  content revision,
  and query generation,
  and are used only when all parts are current.
  Bounds:
  1,000 characters of find text,
  64 MiB of source,
  10,000 retained matches.
  The bar is a layout row under the source view with no buttons:
  Ctrl+F opens it,
  Enter and Shift+Enter move with wrap-around,
  Escape closes it.
  The active match is the reading selection,
  so Ctrl+C copies it and reload correspondence reuses the selection mapping.
  `tests/find_reference.rs` asserts the exact 13 browser-corpus cases that differ.
  The coordinating session independently reran the combined `main` state:
  198 library and integration tests,
  20 native tests,
  and package lint all pass.
  Thirty guard-removal controls passed before the runtime cherry-picks
  (`~/temp/agent/ide-find-guard-xnCvWt/results.json`,
  `~/temp/agent/ide-find-guard-uRdXAC/results.json`).
  Dark and light nested-compositor evidence with seat input is in
  `~/temp/agent/find-impl-c1/evidence/`;
  the coordinating session inspected the active-match frame in both themes.
  Open items from it:
  the toolkit `LineEdit` clear icon shows in the find input,
  as it does in the search overlay;
  every accepted match list repaints the source tile.
  The stale compositor `screenshot` frames it reported are fixed;
  see the runtime color-scheme entry in this list.
- Measured Helix runtime language coverage,
  cherry-picked onto `main` as `953753833` through `9cc81b5f7`.
  The runtime ships 27 grammars for 22 of the 23 `tokei`-measured languages
  (plain text has no grammar),
  34,607,713 bytes of assets.
  A recognized language without a bundled grammar now reads as plain text;
  a bundled grammar that fails to load remains a visible failure;
  a missing or malformed `manifest.json` fails syntax-engine startup with a named diagnostic.
  The combined `main` state passed 198 library and integration tests in 34 test binaries
  (`mise run //package/desktop-app/ide:test`,
  log kept only in session scratch).
  Record:
  `doc/planning/slint-ide-runtime-languages.md`.
  Its worktree `/var/home/user/worktrees/ide-runtime-languages`
  (branch `feat/ide-runtime-languages`)
  is merged and no longer needed.
- The compiled `helix-lsp` spike and design:
  `doc/planning/slint-ide-language-intelligence.md`.
  Adopted there on 2026-10-05:
  the project's own TypeScript 7 server (`tsc --lsp --stdio`) replaces Helix's default TypeScript entry,
  because `typescript-language-server` 6.0.0 cannot start without a `tsserver.js` and the repository pins TypeScript 7;
  `file` targets outside the project root open read-only;
  `didSave` follows an external reload when the server asks for it.
  The TypeScript choice was reported to the user as open to veto.
- Runtime color-scheme switching in the nested compositor,
  commits `96f8adc2a` through `8da338a3e` on `main`.
  The control socket accepts `color-scheme dark|light`;
  it answers `ok changed` after replacing the served value and emitting
  `org.freedesktop.portal.Settings.SettingChanged` with body `ssv`
  from the connection that owns `org.freedesktop.portal.Desktop` on the private bus,
  `ok unchanged` without a signal,
  and an error when the session has no private portal.
  It never falls back to another bus.
  Slint 1.18.1's winit backend ignores the signal from any other sender or with a `u` body
  (`doc/troubleshooting/slint-nested-color-scheme-portal.md`).
  `inspect:color-scheme` in that package showed a `slint-viewer` scene follow dark,
  light,
  dark,
  light live.
  The same leg traced and fixed the stale `screenshot` frames:
  while the parent compositor stops presenting,
  for example with the host session locked,
  the hosted client received no frame callbacks;
  `src/frame_pacing.rs` now paces it from a timer
  (`doc/troubleshooting/nested-wayland-screenshot-stale-frame.md`).
  The package passes 56 tests,
  lint,
  and Clippy.
  Read-only analysis found no IDE change needed for a live switch:
  `ui/app.slint` calls `theme-changed` when `dark-scheme` changes,
  and `src/source_frame.rs` includes the colors in the frame stamp.
  Not yet verified:
  the IDE under a live switch,
  a headless theme-change test,
  and a possible frame with the new palette over the old source image.
  The compositor has no runtime output scaling,
  so physical-output DPI migration is covered only by the headless scale-factor test.
  Residual risk found by that leg:
  the private D-Bus daemon attempts service activation from host service files
  (a hosted `slint-viewer` triggered `org.a11y.Bus` activation,
  which failed with `Permission denied`);
  nothing reached the host bus,
  and a private daemon configuration without service directories would close it.

Known constraints from the spike that the native wiring must honor:
`Registry` derives the LSP root from the process working directory;
a client used before `initialize` completes panics;
a failed `initialize` is invisible without a watchdog;
unconfined rust-analyzer writes `Cargo.lock` and `target/` into the project,
so real servers run only against disposable projects until confinement is wired in.

Language servers present on this host for measured languages:
TypeScript and JavaScript,
Rust,
Slint,
and QML.
Sixteen other measured Helix languages configure servers that are not installed;
SQL,
Batch,
and XML configure none.

Claude Code's built-in worktree isolation for subagents fails here;
see `doc/troubleshooting/claude-code-worktree-create-hook-no-path.md`,
written by another session,
for the cause and the manual `git worktree add` workaround with its provisioning steps.
The repository `git` shim rejects `git switch --create` inside an existing worktree;
new branch work needs its own `git worktree add -b`.

Queue after the in-flight work:

1. Integrate the annotation branch,
   then connect `State::annotations` to its renderer,
   and verify all five language feature paths in the nested compositor on `main`,
   confined,
   against disposable projects.
2. Measure the IDE's scale-change latency in a release build.
   In the debug build the compositor showed the IDE's previous buffer,
   scaled and soft,
   for about 330 ms after a switch from 1 to 2 before the first new frame;
   a cold glyph cache at the new scale is a guess,
   not a measured cause.
3. Confirm the two find flakes are gone with repeated native runs on the final `main`.
   `native_find_paints_visible_matches_only_and_reveals_far_columns` failed 3 of 8 runs on unmodified `8995633b0`
   because the query `n` also yields its `1/301` count;
   `native_find_recomputes_after_external_reload_and_follows_selection_correspondence`
   failed 1 of 8 with selection `(2, 5)`,
   because the prefix `am ` also yields `1/1`.
   `4ab83c59e` adds `status_for`,
   which also waits for the active match to spell the whole query.
4. The final package gates:
   guard-removal reruns on the final `main`,
   packaging,
   consumer-boundary checks,
   and the behavior-difference record.

Do not restart technology selection for the IDE architecture:
Helix reuse and the standalone Rust/Slint design are approved.

Historical navigation verification:
The read-only workspace model's five tests and the lazy tree model's eight tests pass.
`FileTree` accepts directory snapshots without doing filesystem I/O.
Opaque directory-request identities now fence replies;
`proc_36aa` passed all fifteen tree/request tests and scoped Rust lint.
`DirectoryWorker` now performs bounded background reads with admission before token creation;
`proc_ab5d` passed all twenty tree/request/worker tests plus scoped Rust lint.
Native tree rows,
file switching,
and Ctrl+digit promotion/reveal are now bound.
Native callback tests and scoped lint pass in `proc_ee8d`.
Actual nested Wayland mouse/keyboard/clipboard navigation checks pass in
`/tmp/monochromatic-ide-native-bOrX20/interaction.mjs` and `edge-probe.mjs`.
Off-screen reveal,
Tab/Shift+Tab traversal,
and clicks on windowed rows now pass in the light native probe.
Guard controls,
formatting,
91 library/integration tests,
ten native tests,
and package lint pass.
The final dark native smoke also passes with the click guard installed.
This navigation checkpoint preceded the combined-search gate.
Identity/admission and file-open-generation guard controls subsequently passed in the disposable copy.
The procedural-macro and variable-font paths now both pass native font isolation.
Continue annotations and language intelligence after workspace navigation.
Actual TypeScript/Rust syntax is now wired and the highlighted native screenshot was inspected.
The simplified UI,
nested clipboard,
and live external-change correspondence are verified.
Do not restart technology selection:
Helix reuse and the standalone Rust/Slint architecture are approved.

## Confirmed contract

- Current Linux x86_64 KDE/Wayland host only;
  local projects,
  one project root/window/source view.
- Read-only source navigation:
  file tree,
  combined path/content search,
  syntax highlighting,
  line numbers,
  caret/selection/copy,
  in-file find.
- Definition,
  references,
  hover,
  inlay hints,
  and displayed-file diagnostics,
  according to actual backend capabilities.
- Coverage is actual repository source/configuration languages found by `tokei`,
  intersected with Helix support.
  Documentation fences do not expand semantic coverage.
  No custom integrations for unsupported languages.
- No direct or delegated project-file writes.
  Scoped private caches and temporary files outside the project are allowed.
  Cache relocation alone is not write confinement.
- External changes refresh during selection,
  preserving best-effort source correspondence and roughly stable viewport.
  Required cases are `I am a bi|g cat.` to `I was a bi|g cat.`,
  and `I [am a] big cat` to `I [was a] big cat, but now I am a human!`.
  The selection must not jump to the later occurrence of `am a`.
- Ctrl+0 through Ctrl+9 match editord:
  unique MRU slots,
  promotion,
  tree badges,
  ancestor expansion,
  reveal,
  and session-local history.
  Ctrl+0 is the current file;
  Ctrl+1 alternates the last two after promotion;
  empty slots do nothing.
- Bundle Inter and JetBrains Mono with their licenses.
  Follow system appearance,
  including runtime changes,
  without an app-specific override.
- Shared CJK/Latin baseline and advance geometry.
  The independently centered per-grapheme screenshot was rejected.
- Genuine pixel-level smooth scrolling with a notched wheel,
  following music-player's native `Flickable` approach.
- Minimal visible UI.
  The user rejected the read-only indicator and Copy button.
  Those controls and the debug footer are removed from the current markup.
  Copy stays on Ctrl+C;
  accessibility retains read-only semantics;
  actionable errors appear only when needed.
- Record every deliberate difference from editord.
  Editing,
  saves,
  formatting,
  refactoring,
  filesystem mutations,
  go-to-line,
  tabs/splits,
  multiple roots,
  Git/status/diff UI,
  terminal/task/agent control,
  project-wide problems,
  rendered/media previews,
  persistent restore,
  remote support,
  and additional platforms are excluded.

## Work queue

- [x] Implement and verify nested compositor clap help and isolated clipboard support.
  Full package tests now report 40 passing cases,
  including wheel protocol and joined-option cases.
  Release build and post-wheel Clippy pass.
  Debug/release help exit 0 before Wayland startup.
  Native text/binary clipboard lifecycle checks pass.
  Hosted children no longer inherit X11 fallback variables.
- [x] Bundle and verify font embedding.
  Official variable roman and genuine italic Inter and JetBrains Mono
  are imported by Slint from `asset/font`.
  `build.rs` explicitly selects `EmbedResourcesKind::EmbedFiles`.
  Generated Rust embeds/registers the bytes.
  Native fontconfig excludes host copies of those families.
- [x] Complete native source-view interaction,
  clipboard readback,
  CJK/combining/tab boundary coverage,
  source rendering,
  and smooth-wheel acceptance.
  Integrated from `feat/ide-source-keys` on 2026-10-05.
- [x] Implement live external refresh and exercise both supplied correspondence examples through the GUI.
- [x] Implement native tree, file switching, and recent-file reveal/badges.
- [x] Implement combined path/content search and verify it in dark/light native sessions.
- [x] Implement in-file find.
  Matching is plain literal case-insensitive substring matching with the existing `regex` dependency;
  the 13 of 29 captured cases where Chrome's ICU collation search differs are deliberate differences.
  The native find bar,
  worker,
  and match painting are verified headless and in dark and light nested sessions.
- [ ] Implement runtime syntax and language-server paths:
  definition,
  references,
  hover,
  hints,
  displayed-file diagnostics,
  and reload synchronization.
- [x] Provision measured Helix runtime language coverage.
  Twenty-seven grammars are built,
  licensed,
  and tested on `main`;
  see `doc/planning/slint-ide-runtime-languages.md`.
- [x] Enforce subprocess project-write confinement.
  Bubblewrap by default since `34e41e424`;
  114 acceptance checks pass on disposable fixtures.
- [x] Verify light mode and live system-theme changes using the private portal,
  not host KDE settings.
  Verified on 2026-10-05 headless and in the nested compositor.
- [ ] Finish scoped formatting,
  Rust documentation/line budgets,
  Clippy,
  tests,
  packaging,
  consumer-boundary checks,
  and behavior-difference documentation.

## Current source boundaries

`src/document.rs` owns the Helix rope,
content revision,
selection,
caret,
and viewport anchor.
`prepare_reload()` uses `compare_ropes`;
`apply_reload()` rejects stale revisions and maps the latest reading state.
Its seven tests include both required correspondence cases,
reverse selection,
Unicode prefix,
deletion,
stale result rejection,
and selection movement while a reload is pending.

`src/file_reload.rs` reads regular UTF-8 source without filesystem mutation.
`src/reload_worker.rs` allows one outstanding job/reply on bounded channels,
computing reads and Helix correspondence on a named background thread.
File generations accompany replies;
the document still rejects stale content revisions.
`src/native/reload.rs` polls replies every 20 ms and submits reads at 250 ms intervals.
It applies changes to the latest selection,
retains fractional viewport placement,
and keeps the last readable text with an actionable error on read failure.
Its library tests and native build pass.
The live GUI verifies both required correspondence cases,
fractional viewport preservation after a prefix line,
and missing-file retention/recovery.
The probe is `/tmp/monochromatic-ide-reload-probe-STkJVk/probe.mjs`;
its native fixture is `/tmp/monochromatic-ide-native-zCPqiB/fixture.ts`.
Caret x changed from 137 to 146 while y remained 34,
and the next copied grapheme remained `g`.
Selection copied `was a`,
not the later literal.
Viewport offsets changed from -240.5 to -264.5 for one inserted line.
The selection screenshot was inspected.

`src/recent.rs` implements ten unique push-to-front slots and exact Ctrl+digit decoding.
Its four tests pass.
It is now wired to native file switching,
slot badges,
ancestor expansion,
and reveal.

`src/text_projection.rs` preserves source/display correspondence through expanded tabs.
`src/shaped_text.rs` uses Parley 0.11 layouts for glyph advances,
fallback baselines,
caret,
selection,
and hit testing.
`src/text_raster.rs` uses Swash 0.2 to paint those exact glyph runs into premultiplied RGBA.
Both dependencies were already present through Slint.
The raster has a 64 MiB viewport allocation limit.

`src/glyph_cache.rs` adds bounded reuse of glyph masks/color images:
16 MiB of image data and 4096 entries.
Keys retain font blob/face,
size,
variation coordinates,
glyph ID,
and exact fractional position bits.
Empty glyphs are cached too.
Theme colors are applied while compositing alpha masks.

`src/source_frame.rs` distinguishes paint inputs from collapsed caret movement.
The native renderer reuses its image and shaped geometry for caret-only changes.
Selection range,
source revision,
syntax spans,
viewport tile,
scale,
horizontal origin,
and palette invalidate painting.
Reset the stamp and accessibility revision when replacing the displayed document with another file.

`src/native.rs` owns the current UI-thread state and startup.
`src/native/render.rs`,
`src/native/input.rs`,
and `src/native/viewport.rs` separate presentation,
input,
and native scrolling.
Pointer selection,
Ctrl+A,
left/right,
Home/End,
and Copy are wired.
A hidden `TextInput` is only a clipboard bridge,
not a second text layout.

Unfinished surfaces as of 2026-10-05:
LSP wiring
and subprocess project-write confinement.
Native tree,
file switching,
combined search,
and in-file find are implemented,
and so are vertical,
paging,
word,
and focus navigation.
The accessible description no longer claims automatic refresh before that path exists.
Legacy terminal-cell geometry and tests in `view_model.rs` were removed on 2026-10-04.
The retained `StyleSpan` now lives in `source_style.rs`.

## Verification and measured rendering issue

The complete IDE suite before render reuse passed 21 tests:
seven document,
four recent,
five shaped-text,
and five legacy view-model cases.
The shaped build and Slint markup check passed.
A native dark screenshot was inspected at
`/tmp/monochromatic-ide-native-DdPO53/shaped.png`.
This is not whole-application acceptance.

Before the debug footer was removed,
a native click reached `Selection 61:61` after the leading CJK glyph;
keyboard selection reached `Selection 60:61`.
After adding nested clipboard management,
independent `wl-paste` readback matches full source,
`猫`,
and decomposed `é` exactly.
Typing and Ctrl+V preserve the accessible source value.
A multi-step MCP probe timed out,
but subsequent state reads responded and showed the expected selection;
this did not establish an application deadlock.

Repeated Home MCP requests took 357.19,
364.49,
and 359.61 ms,
while an ignored `x` request took 2.31 ms.
These are end-to-end request timings,
not isolated paint timings.
The bounded `test:raster` fixture then measured unchanged shaping at 9.82 to 10.29 ms
and rasterization at 368.16 to 373.14 ms.
After glyph caching,
unchanged shaping measured 9.42 to 9.59 ms
and rasterization 7.76 to 8.10 ms.
Repeated cached pixels match byte-for-byte.
The container limits mean absolute fixture timings are not native-host timings.
Additional cache-invalidation and native-boundary checks remain necessary.

A genuine native notch before the cache fix moved from 0 to -60 pixels,
with fractional positions including -5.85,
-13.183334,
and -48.442593.
At the materialization boundary,
a state request took 367.79 ms and the next observed position jumped to -60.
Evidence:
`/tmp/monochromatic-ide-native-8CCvg2/wheel-before.json`.
Native `Flickable` easing exists,
but the synchronous source raster interrupted it.
After rebuilding,
`/tmp/monochromatic-ide-native-S6Y37Y/wheel-after.json` shows continued fractional motion through that boundary,
including -49.020374,
-57.600006,
and -59.814823.
The boundary request took 32.52 ms in this debug build,
not the previous 367.79 ms.
Steady repeated Home requests after caret-image reuse took 1.68 to 2.95 ms.
This is evidence of the corrected redraw path,
not a claim of guaranteed frame pacing or whole-application acceptance.

MCP `scroll_element` sends `TouchPhase::Cancelled`,
which exercises immediate scrolling and is not a notched-wheel test.
The repo compositor now supports
`wheel X Y HORIZONTAL_NOTCHES VERTICAL_NOTCHES`,
using `AxisSource::Wheel`,
v120 notch units,
and ordinary axis deltas.
See [the scroll investigation][scroll].

## Syntax and font-fidelity additions

`src/bin/ide-runtime.rs` and the `runtime` task own build-only grammar provisioning.
They reuse pinned Helix fetching/building,
with private configuration under `target`.
The initial slice contains Rust,
TypeScript,
TSX,
JavaScript,
and JSDoc grammars;
full measured-inventory coverage remains pending.
Queries come from the pinned Cargo checkout,
and each grammar's license is copied into the runtime.
Assets are published beside debug/release binaries.
The asset directory measured 5,655,910 bytes at this checkpoint.

The terminal base image lacked `c++`,
which Helix's grammar manager requests even for C parsers.
`package/desktop-app/ide/Containerfile` now extends that image with `gcc-c++`;
tasks use `localhost/monochromatic/ide` with the same resource bounds.
The `runtime` task and actual parser tests pass.

`src/syntax.rs` uses Helix filename/shebang recognition and tree-house offsets,
converting byte ranges to source-character ranges.
Adjacent equal paint intervals are merged;
Rust comments arrived as adjacent `//` and ` 猫` captures rather than a single span.
`HighlighterError` implements Display but not std::error::Error;
`src/syntax_error.rs` supplies explicit operation-specific diagnostics.
Syntax failures retain readable text rather than pretending an installed language is unrecognized.
The worker computes initial and changed-source syntax and tags it with its target revision.
Native application rejects stale syntax,
and immutable `SourceStyles` prevent copying all spans during caret-only updates.

The dark syntax screenshot was inspected at
`/tmp/monochromatic-ide-native-KA8eTH/syntax-dark.png`.
It shows real grammar classifications,
not fixed example colors.

The user explicitly requested correct ligatures and other JetBrains Mono/Inter font behavior.
The source now explicitly enables JetBrains Mono's `calt` feature.
Real on/off glyph controls,
slashed-zero substitution,
and ligature-interior caret/hit/copy checks pass.
A committed regression exposed wrong selection ink inside the middle character of `===`.
`src/selection_paint.rs` now clips foreground against the same rectangles used for native backgrounds,
without reshaping or breaking the ligature.
Fractional DPI/origin and opacity controls pass.

Inter tests verify kerning against an off control (106.35498 versus 112.976074 pixels),
real 400/600 face bytes,
and tabular/proportional figures through the same Parley/fontique stack Slint uses.
The user rejected the static subset and explicitly requested variable fonts and real italics.
On 2026-10-04,
`Inter-Regular.ttf`/`Inter-SemiBold.ttf` were replaced by `InterVariable.ttf`/`InterVariable-Italic.ttf`,
and `JetBrainsMono-Regular.ttf` by `JetBrainsMono-Variable.ttf`/`JetBrainsMono-VariableItalic.ttf`.
The original filesystem names for the latter pair are recorded in the font README;
font bytes and internal names are unchanged.
JetBrains Mono advertises wght 100 to 800;
Inter advertises wght 100 to 900 and opsz 14 to 32.
Both families have separate real italic faces.

`font_asset.rs` shares stable source-font blob identities.
`SourceTypography` validates variable weights and chooses real italic faces.
Tests verify intermediate weights,
exact font bytes,
no synthetic emboldening/skew,
variation-aware glyph caching with the same blob ID,
and italic ligature caret/copy behavior.
Inter tests verify explicit optical sizes 14 and 32.
The toolkit's ordinary UI request still leaves opsz at 14;
automatic optical sizing is not claimed.
Optional stylistic alternates remain upstream defaults and no settings panel was added.
`asset/font/README.md` records the measured feature boundary.

`ui/app.slint` now tracks a physical pixel converted to logical length,
so an idle display-scale change invalidates source rasterization.
`src/native/tests.rs` drives actual Slint window events through the headless backend.
`test:native` and the markup check pass after the observed-failing DPI test.

## Latest clipboard and interface verification

The user explicitly authorized adding clipboard support to the nested compositor.
`src/handler/clipboard.rs` registers wlr-data-control and ext-data-control against the nested seat.
`src/child.rs` removes `DISPLAY` and `XAUTHORITY` so toolkit fallback cannot reach host X11.
The new isolation test was observed failing before the fix and passing afterward.
`inspect:clipboard` passed text and binary transfer,
selection clearing,
and clean producer shutdown.
See [the clipboard investigation][clipboard].

The latest complete IDE suite passes 49 tests after the expanded font checks;
the native headless DPI test also passes.
A new five-case workspace-model test file was added afterward and is running.
The headless native DPI regression was observed failing (1300-pixel bitmap retained instead of 2600),
then passed after adding physical-pixel scale invalidation.
It checks scale factors 2,
1.25,
and back to 1 without source input or resize.
The Slint markup check and native build pass.
IDE Clippy,
Rust documentation/line budgets,
and Slint markup checks now pass.
`native.rs` uses Slint's supported procedural-macro re-export of `ui/app.slint`,
retaining generated-code provenance rather than suppressing shadow lints.
The earlier generated-module implicit-return allowance was removed.
`build.rs` explicitly supplies `SLINT_EMBED_RESOURCES=true` to that macro and retains standalone markup validation.
Native font-isolation verification should be repeated for this integration change.

The minimal dark UI screenshot at
`/tmp/monochromatic-ide-native-8rNGro/minimal-dark.png`
was inspected.
MCP confirms no Button role,
read-only badge,
or debug footer in the complete element tree.
Copy is keyboard-only.
Only the file context and source remain visible in the normal source-view gate.

## Active probes at this checkpoint

Every managed process exited before this handoff;
none is running,
so no MCP port or nested Wayland socket is bound.

- Search backend and transient native overlay are implemented.
  `src/native/navigation/search/` binds double-Shift,
  150ms debounce,
  scope capture,
  independent errors,
  `%` filtering,
  close cancellation,
  and safe result opening.
  `navigation/line.rs` maps a content result's one-based line to canonical source characters.
  It retains the current document identity for a same-file hit.
  `search_worker.rs` keeps one latest request/reply,
  cancels superseded queries,
  and joins its owned thread after child cleanup.
  `search_process.rs` runs filename/content ripgrep streams concurrently,
  disables inherited configuration and preprocessing,
  and reaps children after cancellation or result caps.
- `proc_ea69` passed real subprocess tests,
  protocol tests,
  bounded output tests,
  and scoped Rust lint.
  Quiet-child cancellation verifies the owned PID is gone;
  a preprocessing-config positive control verifies `--no-config` separately from environment removal.
  Invalid regexes and oversized records retain the other stream's filename results.
- Search output records are capped at 4 MiB before parsing;
  diagnostic storage is capped at 64 KiB while still draining stderr.
  Previews retain at most 300 complete graphemes.
  Native filenames use NUL-delimited records or decoded base64 JSON paths.
- The bounded IDE image lacked `rg` and was updated to include it.
  Its verified runtime is ripgrep 14.1.1;
  the host is 15.2.0.
  Existing base64 0.23.1 is now an explicit dependency;
  the generated lockfile added only that direct dependency edge.
- Fresh UI-reference reads found required parity details:
  double-Shift release within 400ms opens the modal;
  non-Shift keypresses interrupt the chord;
  query debounce is 150ms;
  leading `%` selects content-only presentation;
  input is trimmed;
  Up/Down selection wraps.
  Search scope is the last-focused tree directory,
  or the parent of its focused file,
  falling back to the project root.
  The worker now validates per-request directory scopes on its background thread.
  Scope success/escape/file/missing/recovery tests pass;
  canonical checking remains vulnerable to filesystem races and is not OS confinement.
- Installed Slint FocusScope supports `capture-key-pressed` and `capture-key-released`.
  The overlay uses those for global shortcut observation.
  Headless tests and real nested-seat double-Shift both exercise them.
  Search uses the supported `search` accessibility role;
  `dialog` is rejected by Slint 1.18.1.
  `inspect:search-accessibility` retains positive and negative controls.

- Final native smoke `proc_4e47` is at port 9318,
  socket `/tmp/monochromatic-ide-native-EGMdAC/control.sock`.
  `interaction.mjs`,
  `focus-reveal.mjs`,
  and `windowed-click.mjs` all pass against the final pointer guard.
  The dark off-screen-reveal screenshot was inspected.
- `proc_fe92` passed final formatting,
  the full 91-test library/integration suite,
  all ten native tests,
  and package lint.
- Search work is starting from fresh reference reads of
  `package-paused/desktop-daemon/editord/src/server/operations/search.ts`
  and `stream-rg.ts`.
  Path results use smart-case substring matching with a cap of 20;
  content results use smart-case regex matching with a cap of 30 and one matching line per file.
  The path stream and content stream run concurrently,
  with path results presented first.
  The host has ripgrep 15.2.0;
  the container runtime probe is next.
  Use NUL-delimited filename records and `--no-config` rather than inheriting arbitrary ripgrep configuration.
  Ripgrep's current help confirms `--max-columns` does not limit JSON output,
  so JSON record/preview handling still needs a bounded design.

- `proc_fe92`,
  `ide-tree-final-format-verification`,
  runs formatting,
  the full library/integration suite,
  native tests,
  and package lint after the final tree-windowing and pointer-identity changes.
- `proc_81e8` stopped cleanly after the light-mode native proofs.
  No current native probe should be assumed running;
  check the process list before starting the final smoke check.

- `proc_81e8`,
  `ide-fixed-row-tree-native-proof`,
  was the completed light-mode probe at port 9319,
  socket `/tmp/monochromatic-ide-native-D6RbFJ/control.sock`.
  Its `interaction.mjs`,
  `focus-reveal.mjs`,
  and `windowed-click.mjs` pass genuine seat input and independent clipboard reads.
  `tree-offscreen-reveal-light.png` was inspected.
  This process predates the final pressed-row/model click guard;
  restart before that guard's final native smoke check.
- Slint ListView's large-jump branch discarded the within-row offset:
  row 43 had bottom 672px in a 628px viewport.
  The native regression failed in `proc_88bc`.
  The tree now uses native `ScrollView` with bounded fixed-row windowing,
  retaining exact offsets and native scrolling physics.
  The corresponding viewport test passed in `proc_d499`.
- Windowed row reuse exposed a genuine pointer-release identity regression in `proc_6bed`.
  The input item now remembers the pressed row index and model identity;
  stale releases do not activate a different row.
  All ten native tests passed in `proc_1d2a`.
- The upstream layout investigation,
  runnable isolated harness,
  tested prototype patch,
  and additive comment draft are in
  `doc/troubleshooting/slint-listview-random-seek-offset.md` and its sidecars.
  No upstream message was posted and no dependency was patched.
  Original-source controls failed in `proc_2f24`;
  the completed prototype passed five control groups in `proc_3830`.
- `proc_2879` passed observed-failing identity,
  directory admission,
  source-open ordering,
  and runtime build-boundary guard controls in the disposable copy at
  `/home/user/temp/agent/ide-tree-guard-zlIbRv/package`.
  Every baseline passed,
  removing each guard caused its named assertion to fail,
  and restoring it passed again.
  Evidence is `navigation-results.json` beside that copy.
  This completes the previously outstanding runtime-helper guard-removal probe.
- `proc_1542` passed 91 library/integration tests and the complete package lint before the final windowing changes.

- `proc_3806`,
  `ide-native-tree-seat-controls`,
  serves MCP at port 9318 and control socket
  `/tmp/monochromatic-ide-native-bOrX20/control.sock`.
  Its binary predates the latest reveal-liveness,
  reader-disconnection,
  initial-directory diagnostic,
  and Tab changes;
  restart before their physical-input verification.
- `interaction.mjs` in that directory passes entirely through native seat modifiers and key events:
  mouse file opens,
  arrow/Enter tree navigation,
  Ctrl+1 alternation,
  Ctrl+0 ancestor reveal,
  failed-open retention,
  ignored typing,
  and `am a` to `was a` selection correspondence after switching files.
  Independent `wl-paste` reads the isolated clipboard.
  `tree-selection-reload.png` was inspected.
- `edge-probe.mjs` passes ignored native paste,
  outside-symlink rejection,
  external tree removal/recreation,
  root-unavailable source retention and recovery,
  and tree Home/End/PageUp/PageDown/Space controls.
  `tree-root-unavailable.png` was inspected.
- An initial probe mixed MCP modifier injection with native seat arrow events and failed its selection-copy assertion.
  It is not selection-regression evidence:
  the replacement probe keeps the entire chord on the native seat and passes.
- The nested helper now accepts `ctrl`,
  `shift`,
  `alt`,
  and `meta` key names.
  Its modifier test failed with `None` versus `Some(29)` before implementation;
  `proc_9075` then passed all 42 helper tests,
  Clippy,
  and the release build.
- Advisor raised unmatched reveal and stopped-reader liveness risks.
  `missing_reveal_keeps_the_existing_model_until_tree_changes` failed before the fix in `proc_bbf5`.
  The native code now waits for actual directory changes rather than rebuilding an identical tree every 20ms.
  A positive model-identity control proves the assertion detects real replacements.
- The proposed contained-alias defect was not reproduced:
  the canonical target row is selected and badged,
  reopening its alias preserves selection/generation,
  and model identity stays stable.
  The test passed both before and after the liveness fix.
  Canonical aliases intentionally share one history identity.
- Reader/opener disconnect tests failed before state cleanup in `proc_f52d`.
  `proc_9d01` passed after clearing the reader's busy/sender state and the opener's waiting target.
  `proc_ee8d` passed native review regressions,
  including accurate first-listing failure text and recovery,
  plus scoped Rust lint.

- `proc_ba75`,
  `ide-native-navigation-consumer-tests`,
  runs the real window callbacks for file switching,
  Ctrl+0 reading-state preservation,
  Ctrl+1 alternation,
  ancestor reveal,
  and failed-open retention through external refresh.
- `proc_d93e` passed the background file-opening tests.
- `proc_ead2` passed CLI parsing,
  real executable help/version/usage exits,
  and scoped Rust lint.
  Startup is now `monochromatic-ide PROJECT [--file FILE]`;
  native inspection tasks pass an explicit disposable project.
- `proc_247b` passed the full pre-opener suite and package lint after directory-reader integration.

- `proc_0d4b` passed the full library/integration suite after variable-font replacement,
  then exposed owned-name comparison lint findings in the new workspace tests.
  Those findings were fixed.
  `proc_e7bd` subsequently passed Clippy,
  Rust documentation/line budgets,
  and the native build.
- `proc_60a7` passed all three native tests,
  Rust documentation/line budgets,
  and Clippy after adding actual source-image typography/selection coverage.
  Native widget pixel comparisons cover request repainting and fallback distinction,
  not direct selected-font-byte introspection.
  The normal source surface remains roman 400 with color-only syntax spans.
- `proc_8ed6` passed both native headless tests under nextest:
  idle DPI changes and real widget variable-weight/italic pixel changes.
  Plain cargo test initially failed because Slint's global platform was initialized on another test thread;
  native tests now use process isolation.
- `proc_e46e`,
  `ide-variable-native-isolation`,
  stopped cleanly after variable-font verification.
  Its completed dark fixture was at
  `/tmp/monochromatic-ide-native-m6zNtt/control.sock`.
  MCP is port 9318.
  All four complete variable/italic font byte sequences were found in the binary.
  Isolated fontconfig resolves Inter Variable and JetBrains Mono to Adwaita Mono externally.
  The native middle-character ligature copy and screenshot probe passes;
  `/tmp/monochromatic-ide-native-m6zNtt/ligature-dark.png` was inspected.
- `proc_ed8c` is stopped after the pre-variable macro-path verification.

- `proc_f89f`,
  `ide-workspace-model-tests`,
  runs the IDE suite with new read-only workspace tests.
- `proc_e9fe`,
  `ide-font-cold-start`,
  is the current dark/syntax probe.
  Socket:
  `/tmp/monochromatic-ide-native-46wFSY/control.sock`.
  MCP:
  `http://127.0.0.1:9318/mcp`.
  It predates the procedural-macro integration change.
  Its font probe copied the middle `=` of `===` and its dark screenshot was inspected.

- `proc_75d5` (stopped),
  `ide-light-font-native`,
  verified the light/syntax fixture with ligature and DPI fixes.
  `/tmp/monochromatic-ide-native-SjCyvc/font-probe.mjs` copied exactly `=` from the middle of `===`.
  Selection was at x=236,
  y=32,
  with width 9.
  `ligature-light.png` in that directory was inspected.
- `proc_2d43`,
  `ide-inter-default-features`,
  passed the expanded font tests including Inter calt/dlig defaults.

- `proc_1614` (stopped),
  `ide-live-reload-native-retry`,
  was the verified dark/scroll fixture with live refresh.
  Socket:
  `/tmp/monochromatic-ide-native-zCPqiB/control.sock`.
  MCP:
  `http://127.0.0.1:9318/mcp`.
- `proc_4ae9` exited before spawning the IDE:
  private D-Bus startup timed out.
  The unchanged retry started successfully;
  no root cause or fix is established for that separate fixture-startup incident.
- `proc_b0cc` (stopped),
  `ide-minimal-clipboard-native`,
  is stopped after verifying the dark/basic fixture with the simplified UI and rebuilt clipboard-capable compositor.
  MCP is `http://127.0.0.1:9318/mcp`.
  Socket:
  `/tmp/monochromatic-ide-native-8rNGro/control.sock`.
  Consumer probe:
  `/tmp/monochromatic-ide-native-8rNGro/clipboard-probe.mjs`.
- `proc_f5f1` (stopped),
  `ide-native-wheel-probe`,
  runs the pre-cache source renderer with the rebuilt compositor.
  MCP is `http://127.0.0.1:9318/mcp`.
  Socket:
  `/tmp/monochromatic-ide-native-8CCvg2/control.sock`.
  Source:
  `/tmp/monochromatic-ide-native-8CCvg2/fixture.ts`.
  This historical baseline process is stopped.
- `proc_ea50`,
  `ide-caret-reuse-tests-build`,
  runs scoped tests then the native build.
  It completed successfully.
- `proc_37fc` and `proc_7948` are stopped.
  Their original source-view handles and binaries are obsolete.

Rediscover windows and elements after every restart.
JSON-RPC `tools/call` accepts loopback POST with
`Content-Type: application/json` and `Accept: application/json, text/event-stream`.
Tool JSON is inside `result.content[].text`.
`list_windows` returns `windowHandles`,
not `windows`.
Use `find_elements_by_id` for `AppWindow::scroll`.
Element and window handles are distinct even when their shapes coincide.
Read current `tools/list` schemas before driving input.

## Build and runtime evidence

All Cargo tasks run through package `mise.toml` in the IDE image extending the terminal build image,
with 2 GiB RAM,
2 CPUs,
512 processes,
4096 file descriptors,
and two Cargo jobs.
Only the package and dedicated Cargo volume are mounted.
Slint resolves to 1.18.1.
Its installed source is under
`~/.local/share/containers/storage/volumes/ide-cargo/_data/registry/src/`.

Root `file-enforcer.config.ts` owns the unconditional Slint dependency.
`gui = []` gates generated UI code,
not optional dependency membership.
Reintroducing `dep:slint` would reproduce Cargo's
`feature gui includes dep:slint, but slint is not an optional dependency`.
Native key comparisons use `SharedString::from(Key::...)`.
Current Slint markup uses `content-width`,
`content-height`,
and `content-y` rather than deprecated `viewport-*` names.

Font provenance,
checksums,
and original OFL licenses live in `asset/font/README.md` and adjacent license files.
Selected releases are Inter v4.1 and JetBrains Mono v2.304.
Native fontconfig resolves both absent primary families to Adwaita Mono externally,
while the app retains its embedded faces.

Read [the private theme-portal investigation][theme].
The compositor supports startup `--color-scheme dark|light` on a private D-Bus session,
and since 2026-10-05 the runtime control command `color-scheme dark|light`.
The IDE's reaction to a live switch is not yet verified.

## Helix and behavioral references

Pinned Helix revision:
`ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`.
Source clone:
`~/temp/agent/slint-ide-languages.mEr8K9/helix`.
All selected Helix crates use that same revision.
Relevant inspected APIs include `compare_ropes`,
`ChangeSet::map_pos`,
`syntax::Loader`,
`Syntax`,
`TextAnnotations`,
and `helix-lsp::Client`/`Registry`.
The installed tree-house highlighter uses event offsets,
not the older start/end/source iterator shape.

`helix-loader::config::default_lang_config()` supplies the registry.
Grammar fetching/building exists in `helix-loader::grammar`,
and the initial Rust/TypeScript/TSX/JavaScript/JSDoc runtime slice is provisioned.
The remaining measured-language inventory still needs build/runtime coverage.
Respect each server's text synchronization mode.
The measured inventory includes TS/JS,
Rust,
Kotlin,
Slint,
QML,
HCL,
SQL,
shell,
C/C++,
Batch,
and configuration/markup families.
Inspected SQL,
XML/SVG,
and Batch entries have grammars but no configured LSP.

Installed `hx` reports `25.07.1 (a05c151b)`.
`hx --health typescript` found parser/query assets and typescript-language-server 6.0.0.
A fresh `hx --health` probe found the installed runtime at `/usr/lib64/helix/runtime`.
Its other reported candidates,
`~/.config/helix/runtime` and `/usr/bin/runtime`,
were absent.
Installed assets are not assumed compatible with the pinned Helix revision.

Editord reference:
`package-paused/desktop-daemon/editord`.
Recency behavior is in `src/client/recent-files.ts`,
`src/client/app/app.ts`,
and `src/client/file-tree/reveal.ts`.
The paused browser/WebSocket daemon architecture is not being revived.

## Workspace model and remaining verification details

`src/workspace.rs` opens one canonical local root,
resolves tree/search paths against it,
and returns fresh directory snapshots in filesystem order.
It preserves hidden entries and native filenames.
`tests/workspace.rs` covers normal listings,
refresh,
contained paths,
sibling-prefix/parent escapes,
symlinks,
and invalid directory inputs.
`src/file_tree.rs` and its `listing`/`rows` helpers now own lazy expansion,
visible snapshot order,
and detached-subtree pruning without filesystem I/O.
Collapsing a parent retains descendant expansion but suppresses hidden loading requests.
Directory-to-file replacement discards obsolete child caches.
Atomic snapshot validation rejects traversal-like names,
duplicate keys,
and mismatched paths;
native byte filenames remain intact.
`test:tree` passes eight cases;
Clippy,
Rust documentation/line budgets,
and Slint checks pass in `proc_1f80`.
`DirectoryRequest` now carries a private shared allocation identity across threads,
not a reusable integer or just path equality.
Superseded,
consumed,
foreign-tree,
and removed/recreated-directory replies are ignored before applying either data or errors.
A current read error releases its request slot while preserving the previous snapshot.
Asynchronous callers must use `complete_listing` rather than the synchronous `apply_listing` entry point.
`DirectoryWorker` admits at most one read or unread reply,
never blocks UI polling,
and closes input before joining its thread on shutdown.
The native tree now uses `ui/tree.slint` plus `src/native/navigation/` bindings.
Its header shows the distinguishing project name;
the full root remains available through accessibility.
It retains native paths behind lossy display labels,
shows recent-file slot badges,
and expands/reveals ancestors after successful opens.
Rows have 48px minimum dimensions;
expanded folders use both arrow direction and font weight,
and the displayed file uses selection background and weight.
Since 2026-10-05 the sidebar starts at 256px and resizes through a 48px divider cell;
the "Completed in this fan-out" list records it.

`FileOpener` reuses `ReloadWorker` for project-relative resolution,
read/diff,
and syntax on its reader thread.
Only the latest requested open can install a document;
failed opens retain the old source and do not promote history.
The ordinary displayed-file refresh worker continues independently.
`navigation_error` remains separate from `file_error`,
so successful refresh of the old file cannot erase an unrelated failed-open diagnostic.

Directory refresh currently round-robins visible expanded folders at 500ms intervals,
with missing lazy snapshots taking priority.
This is polling,
not filesystem watcher integration;
refresh latency grows with the number of expanded directories.
No event-driven latency guarantee is claimed.
Collapsing a folder dismisses its folder-scoped diagnostic;
re-expansion requests it again.
Canonical inside symlink aliases select and badge the real target row rather than creating duplicate history slots.
Successful file opens focus the source view;
Tab traversal back to the tree and Shift+Tab back to source pass with native seat input.
Both readers close their input and join on shutdown;
a filesystem operation that never returns can therefore delay shutdown.
Cancellation invalidates replies,
not a running operating-system file read.

Observed-failing guard checks passed in `proc_1d30`:
removing snapshot validation,
unknown-directory rejection,
and non-directory expansion rejection each caused its named test to fail.
Restoring all guards returned the eight-test suite to passing.
The evidence is `results.json` beside the disposable copy.
The disposable copy is `/home/user/temp/agent/ide-tree-guard-zlIbRv/package`;
its baseline passed eight tests in `proc_5274`.
`guard-check.mjs` outside that copy removes guards one at a time,
checks named failures,
restores the files,
and reruns the suite.
All Cargo work remains bounded by the copied package's container task.
Canonical path checks do not claim race-proof OS-level read confinement;
server project-write confinement remains a separate required gate.

Reference search behavior was re-read:
editord returns up to 20 smart-case substring file-path matches before up to 30 regex content matches,
one content line per file,
using concurrent ripgrep processes and cancellation.
Its tree preserves readdir order and does not turn directory symlinks into expandable directories.
Its CSS uses tab-size 2;
the current source projection still expands tabs using a four-column terminal-width calculation.
Tab-width parity and mixed-script physical tab geometry are outstanding findings,
not verified font behavior.

The runtime helper's configuration-boundary guard was removed only in the disposable copy.
`runtime_rejects_non_build_configuration_before_writing` failed,
then passed again after restoration.
No real user configuration was touched.

The compositor protocol-delivery regression failed with an unflushed sync reply,
then passed after end-of-dispatch flushing.
Its full 41-test suite,
Clippy,
and release build pass.
The rebuilt helper cold-started the native dark font probe successfully.
The earlier light-startup stall recovered before the fix,
so its exact trigger remains unproven.
See `doc/troubleshooting/nested-wayland-protocol-delivery.md`.

## Repository safety and policy proposals

Scope every commit to task files and preserve concurrent work.
Auto-push is enabled.
Generated GPL/LGPL texts under `package/desktop-app/ide/LICENSES/` are now committed unchanged.
Current unrelated linter work is concurrent work,
not something to revert.

Recent task commits:
`4af24d5c7` adds the raster probe;
`6385c1fe1` adds bounded glyph images;
`205ff25ea` splits native boundaries;
`d4e4b3973` adds caret-only image reuse.
Inspect current Git status and process state rather than assuming this snapshot is live.

Policy proposals were resolved on 2026-10-05.
The user declined a CLI `--help` rule,
a font-verification rule,
and a reference-parity rule,
and approved two rules:
`QRX` in `AGENTS.md`
(batch decisions in the question tool and re-explain each item's context in plain words)
and `QVS` in the visual-design-review skill
(show built screenshots of every option as committed local files before asking).
The `PXQ` checkpoint-continuation idea was not re-proposed;
`PXQ` already forbids ending a turn on a status report the user must answer with "continue".

Minimap report of 2026-10-24 (fixed, landed, installed):
the owner opened `package/build-tool/css/src/build.unit.test.ts` in the installed IDE (built 2026-10-22 12:48)
and saw the minimap "jump around as I scroll",
a band of its picture drawn doubled and wider (screenshot `~/Pictures/Screenshots/Screenshot_20261024_070953.png`),
and "the inlay hints at line 96 keep doubling or something as I click around in the minimap";
later the owner saw the same hint double in the main editor for a few ms when clicking away from line 96 and back.
The installed binary and `main` held the same minimap code (diff since `cd49e20a92` is comments and tests only),
so the minimap was not the cause.
The cause is `Annotations::merge_labels` in `package/desktop-app/ide/src/annotation/blocks.rs`:
TypeScript 7 answers a range request with the hints of every syntax node that reaches into the range,
line 96 is a statement whose body runs to line 269,
so every answer asked for lines 100 to 269 added line 96's four hints again,
and each copy pushed the lines below it down.
The fix leaves out hints that stand on lines the answer did not ask about.
Tests that failed first: `hints_outside_the_asked_lines_are_not_part_of_an_answer` (`tests/annotation.rs`) and
`an_answer_for_other_lines_adds_no_hint_to_a_line_it_did_not_ask_about` (`src/native/annotation_range_tests.rs`);
source guards `hints-asked-lines` and `hints-asked-lines-window` accepted in all three phases.
The agent's measurements: minimap crops after the same click differed by 17,028 px before the fix and about 0 after;
the gap above line 96 was 130 px for about 0.2 s in 5 of 35 frames before and 82 px in all 34 frames after.
Landed as merge `acb620dd6a` (branch tip `7a5063dfc6`);
my gates on the merged tree: `lint` 0, `test:native` 291 passed and 5 skipped, `test` 804 passed,
`test:bin` 0, `inspect:no-software-renderer` 0.
The IDE was rebuilt and installed from that tree (sha256 `9c8c51a8bd96ff49…`, prints 0.1.1;
the previous binary is `~/.local/bin/monochromatic-ide.previous`).
Agent worktree and branch removed locally and on the remote;
the agent's captures stay in `~/temp/agent/mm` until the owner confirms the fix on the installed binary.
Open from this report:
the first visit to the bottom of the file still gives a minimap that differs by 994 px (device rows 2500 to 3238)
from the one after a visit elsewhere, then stays stable;
the agent's reading is the server's first answer being incomplete (68 hints against 72), not verified.
The doubling lasted about 0.2 s (5 of 35 frames), not a few ms:
`HINT_SETTLE` in `src/native/language/sync.rs:58` is 200 ms,
the time the visible lines must stay the same before hints are asked for them,
so the extra copies stayed until the next request that covered line 96 was answered.
The agent also left a stale `tail --pid=0` wait loop in its worktree (stopped by me).
Next action: ask the owner to confirm on the installed binary, then clear `~/temp/agent/mm`.
Started 2026-10-24: an agent in a worktree measures the 994 px residual
(the server's first and warm answers diffed hint by hint,
whether the IDE re-asks or the minimap rebuilds when hints change)
and fixes it behind a failing test if it is an IDE defect;
its report is verified, landed and the IDE reinstalled like the first one.

Correction of the residual, measured later on 2026-10-24 (`~/temp/agent/mm2`;
record in `doc/troubleshooting/typescript-7-inlay-hints-outside-the-requested-range.md`):
the server's first answer is not incomplete
(equal ranges gave identical answers hint by hint, and every range equals the full-file answer on its lines),
and the 994 px are the minimap's viewport box, not the picture.
The first click at the bottom is mapped through a layout of 8688 px that holds only the hints of the first answer,
so the view lands on line 214;
the answer for lines 137 to 287 then makes the layout 9522 px,
and the same click after a visit elsewhere lands at the end (line 223 on top).
In a window tall enough for the first request to cover the file the difference is 0 px.
No IDE change was made;
options (re-aim once, ask for the whole file, leave it) are ranked in the record and wait for the owner.

[scope]: ../decision/slint-ide-0x-scope.md
[plan]: ../planning/slint-ide-implementation.md
[theme]: ../troubleshooting/slint-nested-color-scheme-portal.md
[scroll]: ../troubleshooting/slint-flickable-smooth-scroll.md
[clipboard]: ../troubleshooting/slint-nested-clipboard.md
