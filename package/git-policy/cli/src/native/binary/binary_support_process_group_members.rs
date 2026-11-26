//! What:
//!  The processes still in a helper's process group,
//!  this process and the programs that started it,
//!  and the assertion that a control's group is empty after it ended its helper.
//! Why:
//!  A helper a control started in a group of its own takes its children into that group,
//!  and a child keeps its group even when it is reparented to the manager,
//!  so the group is where a stand-in that outlived its test is still visible
//!  (integration pass 31, 2026-11-14, its report's first finding).
//!  These reads live beside the fixture file rather than inside it,
//!  because the fixture file is over the repository's per-file line budget and they describe the process table,
//!  not the fixture's lifecycle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = procTable().filter((p) => p.pgrp === group && !ours.has(p.pid)); left.map((p) => [p.pid, p.cmdline]);
//! ```

/// What:
///  Every process still in `group`,
///  except this process and the programs that started it,
///  each with the command line that names it.
/// Why:
///   A helper that a control started in a group of its own takes its children into that group,
///  and a child keeps its group even when it is reparented to the manager,
///  so the group is where a stand-in that outlived its test is still visible
///  (integration pass 31,
///  2026-11-14,
///  its report's first finding).
///  A helper started outside a group of its own shares the group of the test,
///  so that group's other members are this process and its own starters,
///  which are not what a control is looking for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = procTable().filter((p) => p.pgrp === group && !ours.has(p.pid)); left.map((p) => [p.pid, p.cmdline]);
/// ```
pub fn processes_in_group(group: libc::pid_t) -> Vec<(u32, String)> {
    let ours: Vec<u32> = this_process_and_its_starters();
    let mut found: Vec<(u32, String)> = Vec::new();
    let Ok(table) = std::fs::read_dir("/proc") else {
        return found;
    };
    for entry in table.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty()
            || !name.bytes().all(|byte| {
                return byte.is_ascii_digit();
            })
        {
            continue;
        }
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if ours.contains(&pid) {
            continue;
        }
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        // The group is the fifth field of `stat`, after the program's name, which may hold spaces and brackets.
        let Some((_before, after_name)) = stat.rsplit_once(')') else {
            continue;
        };
        let fields: Vec<&str> = after_name.split_whitespace().collect();
        let Some(in_group) = fields.get(2).and_then(|text| {
            return text.parse::<libc::pid_t>().ok();
        }) else {
            continue;
        };
        if in_group != group {
            continue;
        }
        let Ok(bytes) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let said: String = String::from_utf8_lossy(bytes.as_slice()).replace('\0', " ");
        found.push((pid, said.trim_end().to_owned()));
    }
    return found;
}

/// What:
///  This process and the programs that started it, by process number,
///  up to the first one whose parent is outside them.
/// Why:
///   A helper started outside a group of its own shares the group of this process,
///  and a check that listed that group's members would answer with the test binary, the test runner and the shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours = [pid, ...ancestors(pid)]
/// ```
fn this_process_and_its_starters() -> Vec<u32> {
    let mut ours: Vec<u32> = vec![std::process::id()];
    let mut parent: u32 = parent_of(std::process::id());
    while parent != 0 && !ours.contains(&parent) {
        ours.push(parent);
        parent = parent_of(parent);
    }
    return ours;
}

/// The process number that started `pid`,
/// or 0 when it cannot be read.
fn parent_of(pid: u32) -> u32 {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return 0;
    };
    let Some((_before, after_name)) = stat.rsplit_once(')') else {
        return 0;
    };
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    return fields
        .get(1)
        .and_then(|text| {
            return text.parse::<u32>().ok();
        })
        .unwrap_or(0);
}

/// What:
///  Fails the control when any process of `group` is still there after the control ended its helper.
/// Why:
///  A helper that outlives its control keeps the `systemd-run` scope of the test alive,
///  and holds a process nothing will ever read;
///  a control that started one asserts this before it ends,
///  so such a leak fails a test instead of waiting for someone to notice it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(processesInGroup(group)).toEqual([])
/// ```
pub fn assert_nothing_in_group(group: libc::pid_t) {
    let left: Vec<(u32, String)> = processes_in_group(group);
    assert!(
        left.is_empty(),
        "processes of the group this control's helper was in are still there after the control ended it: {left:?}"
    );
}
