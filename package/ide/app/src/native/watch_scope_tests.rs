//! An ignored measurement:
//!  the inotify watches the window holds with many expanded folders,
//!  and how
//! stale a folder that changed while scrolled out of view is at the moment it is scrolled back in.
//! `inspect:watch-scope` runs it for the shipped scope (every expanded folder is watched) and for a
//! disposable copy that watches only the folders in the tree's viewport.

/// Tree-row lookup by label and the bounded wait for startup state.
use super::navigation_tests::row;
/// Shipped window setup shared with the watch tests.
use super::watch_tests::open;
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Real headless timers drive the same refresh timers as the shipped event loop;
/// `ComponentHandle` provides `hide` on the generated window and `Model` the tree's row count.
use slint::{ComponentHandle, Model, platform::update_timers_and_animations};
/// What:
///  `fs` reads `/proc` and writes fixtures;
///  `Duration`/`Instant` are a time span and a monotonic time.
/// Why:
///  The watch count comes from this process's own inotify descriptors,
///  as the kernel counts them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readdirSync, readFileSync } from 'node:fs';
/// ```
use std::{
    fs,
    time::{Duration, Instant},
};

/// Expanded folders in the fixture;
///  with three files each they fill several screens of the tree.
const FOLDERS: usize = 60;

/// Time between the change and scrolling its folder into view,
///  longer than a notified reread takes.
const DELAY: Duration = Duration::from_millis(300);

/// Count this process's inotify watches:
///  one `inotify wd:` line per watch in each inotify descriptor's fdinfo.
fn kernel_watches() -> usize {
    let mut total = 0;
    // What: `read_dir` lists `/proc/self/fd`; `flatten` skips entries that could not be read.
    // Why: Each descriptor that links to `anon_inode:inotify` is an inotify instance whose watches count.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const fd of readdirSync('/proc/self/fd')) if (readlinkSync(...) === 'anon_inode:inotify') total += countWatches(fd);
    // ```
    for entry in fs::read_dir("/proc/self/fd")
        .expect("list descriptors")
        .flatten()
    {
        let target = fs::read_link(entry.path()).unwrap_or_default();
        if target.as_os_str() != "anon_inode:inotify" {
            continue;
        }
        let info = format!("/proc/self/fdinfo/{}", entry.file_name().to_string_lossy());
        let text = fs::read_to_string(info).unwrap_or_default();
        for line in text.lines() {
            if line.starts_with("inotify wd:") {
                total += 1;
            }
        }
    }
    return total;
}

/// Run native timers for `span`.
fn run_for(span: Duration) {
    let start = Instant::now();
    while start.elapsed() < span {
        update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// For each trial:
///  scroll a folder out of view,
///  create a file in it,
///  wait `DELAY`,
///  scroll it back,
/// and time until the new row is in the tree;
///  zero means it was already there when the folder came into view.
#[test]
#[ignore = "measurement; run through inspect:watch-scope"]
fn watch_scope_reveal_staleness() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let displayed = fixture.path().join("view.txt");
    fs::write(&displayed, "view\n").expect("displayed source");
    for index in 0..FOLDERS {
        let folder = fixture.path().join(format!("folder-{index:02}"));
        fs::create_dir(&folder).expect("fixture folder");
        for file in 0..3 {
            fs::write(folder.join(format!("file-{index:02}-{file}.txt")), "").expect("folder file");
        }
    }
    let (window, _state, _timers) = open(fixture.path(), &displayed);
    for index in 0..FOLDERS {
        let label = format!("folder-{index:02}");
        {
            let mut waiting = Waiting::eventually();
            while !(row(&window, &label).is_some()) {
                waiting.step(NAVIGATION_UNREACHED);
            }
        }
        window.invoke_tree_activate(row(&window, &label).expect("folder row"));
        let child = format!("file-{index:02}-0.txt");
        {
            let mut waiting = Waiting::eventually();
            while !(row(&window, &child).is_some()) {
                waiting.step(NAVIGATION_UNREACHED);
            }
        }
    }
    run_for(Duration::from_millis(1500));
    let watches = kernel_watches();
    let mut stale_ms = Vec::new();
    let mut fresh = 0;
    let trials = 20;
    for trial in 0..trials {
        // What: `(trial * 7) % FOLDERS` steps through the folders in a spread-out order.
        // Why: Each trial uses another folder, alternating between the top and the bottom of the tree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const target = (trial * 7) % FOLDERS;
        // ```
        let target = (trial * 7) % FOLDERS;
        let total = window.get_tree_entries().row_count() as i32;
        // Scroll to the far end from the target, so the target's rows are out of view.
        let away = if target < FOLDERS / 2 { total - 1 } else { 0 };
        window.invoke_reveal_tree(away);
        run_for(Duration::from_millis(200));
        let name = format!("created-{trial:02}.txt");
        let folder = fixture.path().join(format!("folder-{target:02}"));
        fs::write(folder.join(&name), "").expect("change while out of view");
        run_for(DELAY);
        let label = format!("folder-{target:02}");
        let index = row(&window, &label).expect("target folder row") as i32;
        let revealed = Instant::now();
        window.invoke_reveal_tree(index + 1);
        if row(&window, &name).is_some() {
            fresh += 1;
            stale_ms.push(0);
            continue;
        }
        loop {
            update_timers_and_animations();
            if row(&window, &name).is_some() {
                break;
            }
            assert!(
                revealed.elapsed() < Duration::from_secs(5),
                "the created file did not appear within 5 s of scrolling to its folder"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        stale_ms.push(revealed.elapsed().as_millis());
    }
    println!(
        "{{\"case\":\"watch-scope\",\"folders\":{FOLDERS},\"kernel_watches\":{watches},\"delay_ms\":{},\"trials\":{trials},\"fresh_at_reveal\":{fresh},\"stale_ms\":{stale_ms:?}}}",
        DELAY.as_millis()
    );
    window.hide().expect("close measurement window");
}
