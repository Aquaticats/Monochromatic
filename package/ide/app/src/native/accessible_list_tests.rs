//! The tree rows,
//!  the search results,
//!  and the location list as an assistive tool reaches them through
//! Slint's element handles:
//!  each container's role,
//!  name,
//!  and item count,
//!  and each row's role,
//!  name,
//! index,
//!  and selected state,
//!  which must follow the open file and the arrow keys.

/// Finding exactly one element by its accessible label,
///  and counting the elements with one label.
use super::accessible_box_tests::{count, only};
/// One key press and release through the window,
///  and a bounded wait that names what did not happen.
use super::find_tests::key;
/// The bounded wait for native navigation state and the tree-row lookup.
use super::navigation_tests::row;
/// The search overlay's double-Shift opening.
use super::search_tests::open;
/// The 1100 by 660 sidebar fixture,
///  an exact window size,
///  and settled layout.
use super::sidebar_tests::{fixture, resize, settle};
/// The window type and the location row type generated from the shipped markup.
use super::{AppWindow, find_tests, search_tests, ui::ReferenceEntry};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// What:
///  `AccessibleRole` is the toolkit's enum of accessible roles;
///  `ElementHandle` is Slint's test-only
/// handle on one element;
///  `ElementQuery` filters the elements below a window step by step.
/// Why:
///  Rows are found by role and name together,
///  as an assistive tool lists the items of a list.
/// Gotcha:
///  The crate is internal to Slint and must have exactly the resolved Slint version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AccessibleRole, ElementHandle, ElementQuery } from 'slint-testing';
/// ```
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
/// Window ownership,
///  toolkit models,
///  and key names.
use slint::{ComponentHandle, Model, ModelRc, VecModel, platform::Key};
/// What:
///  `Rc` is a shared pointer for one thread (sibling `Arc` works across threads);
///  `Cell` holds a small
/// copied value that can change behind a shared pointer.
/// Why:
///  The location list's choose callback records an index that the test body reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::Cell, fs, rc::Rc};

/// What:
///  `query` is an element query already narrowed to a window or to one list;
///  `role` and `label` are
/// the wanted accessible role and name;
///  the answer is the one element that has both.
/// Why:
///  A row's text is also an element with the row's name,
///  and the tree and the search results can list
/// the same file name,
///  so a row is found by its role inside its own list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function matching(query: ElementQuery, role: AccessibleRole, label: string): ElementHandle;
/// ```
fn matching(query: ElementQuery, role: AccessibleRole, label: &str) -> ElementHandle {
    // What: `.to_string()` copies the borrowed label into an owned `String` (sibling `&str` borrows).
    // Why: The filter below is kept by the query, so it must own its copy of the label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wanted = label;
    // ```
    let wanted = label.to_string();
    // What: `match_accessible_role` and `match_predicate` keep the elements that pass;
    // `move |candidate| ...` is an arrow function that owns `wanted`; `is_some_and` is true for `Some`
    // holding a value that passes; `find_all()` collects the result into an array (`Vec`).
    // Why: Exactly one element may have this role and this name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = query.filter(e => e.accessibleRole === role && e.accessibleLabel === wanted);
    // ```
    let mut found = Vec::new();
    for candidate in query.match_accessible_role(role).find_all() {
        if has_label(&candidate, &wanted) {
            found.push(candidate);
        }
    }
    assert_eq!(
        found.len(),
        1,
        "assistive tools find {} elements of role {role:?} named {label:?}, not one",
        found.len()
    );
    // What: `found[0].clone()` copies the handle at index 0 out of the array.
    // Why: The array is dropped at the end of this function; the caller needs its own handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return found[0];
    // ```
    return found[0].clone();
}

/// Whether an element reads out exactly `wanted` as its accessible label.
fn has_label(candidate: &ElementHandle, wanted: &str) -> bool {
    return match candidate.accessible_label() {
        Some(text) => text == wanted,
        None => false,
    };
}

/// What:
///  `list: &ElementHandle` lends the list or tree;
///  `label` names one of its rows.
/// Why:
///  Rows are list items inside their own container.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function item(list: ElementHandle, label: string): ElementHandle;
/// ```
fn item(list: &ElementHandle, label: &str) -> ElementHandle {
    // What: `query_descendants()` starts a query over every element below this one.
    // Why: Only this list's rows are candidates.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return matching(descendants(list), AccessibleRole.ListItem, label);
    // ```
    return matching(list.query_descendants(), AccessibleRole::ListItem, label);
}

/// What:
///  `label` names a row of `list`;
///  `selected` and `index` are what it must report.
/// Why:
///  Every row of every list reports the same item properties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertRow(list: ElementHandle, label: string, selected: boolean, index: number): void;
/// ```
fn assert_row(list: &ElementHandle, label: &str, selected: bool, index: usize) {
    let row = item(list, label);
    // What: `.to_string()` copies the label for the filter, which the query keeps; `.len()` counts the matches.
    // Why: Only the row itself may carry its name; a text inside it with the same name is read twice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = descendants(list).filter(e => e.accessibleLabel === label).length;
    // ```
    let wanted = label.to_string();
    let mut named = 0;
    for candidate in list.query_descendants().find_all() {
        if has_label(&candidate, &wanted) {
            named += 1;
        }
    }
    assert_eq!(
        named, 1,
        "row {label:?} is named by {named} elements in its list, so its name would be read more than once"
    );
    // What: `Some(true)` is the present variant of `Option` holding `true`.
    // Why: Every row can be selected, and an assistive tool must be told so.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(row.accessibleItemSelectable, true);
    // ```
    assert_eq!(
        row.accessible_item_selectable(),
        Some(true),
        "row {label:?} is not reported as selectable"
    );
    assert_eq!(
        row.accessible_item_selected(),
        Some(selected),
        "row {label:?} does not report selected = {selected}"
    );
    assert_eq!(
        row.accessible_item_index(),
        Some(index),
        "row {label:?} does not report its position"
    );
}

/// What:
///  `role` and `label` name a list or tree;
///  `rows` is how many items it must report;
///  the answer is
/// the container,
///  for finding its rows.
/// Why:
///  The container's role,
///  name,
///  and item count tell an assistive tool what it is moving through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function container(window: AppWindow, role: AccessibleRole, label: string, rows: number): ElementHandle;
/// ```
fn container(window: &AppWindow, role: AccessibleRole, label: &str, rows: usize) -> ElementHandle {
    // What: `ElementQuery::from_root(window)` is a query over every element of the window.
    // Why: Lists and the tree are found anywhere in the window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = matching(descendants(window), role, label);
    // ```
    let found = matching(ElementQuery::from_root(window), role, label);
    assert_eq!(
        found.accessible_item_count(),
        Some(rows),
        "{label:?} does not report its row count"
    );
    return found;
}

/// The tree is named and counts its rows;
///  each row is a list item with its file name,
///  its position,
///  and
/// a selected state that marks the open file and moves when another file is opened through the row's
/// default action.
///  A directory row is expandable and its expand action expands it.
#[test]
fn tree_rows_report_role_name_position_and_the_open_file_as_selected() {
    // What: `tempfile::tempdir()` makes a disposable directory removed when the value is dropped.
    // Why: The tree lists a real project, which must never be the repository.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fixture = mkdtempSync(join(tmpdir(), 'tree-'));
    // ```
    let fixture = tempfile::tempdir().expect("disposable accessible tree project");
    // What: `fixture.path().join(..)` builds a path inside the directory; `fs::write` creates the file.
    // Why: Two files and one directory give a selected row, an unselected row, and an expandable row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(join(fixture, 'alpha.txt'), 'alpha\n');
    // ```
    fs::write(fixture.path().join("alpha.txt"), "alpha\n").expect("tree fixture");
    fs::write(fixture.path().join("beta.txt"), "beta\n").expect("tree fixture");
    fs::create_dir(fixture.path().join("nested")).expect("tree fixture directory");
    fs::write(fixture.path().join("nested").join("inner.txt"), "inner\n").expect("tree fixture");
    let reader = find_tests::reader(fixture.path(), "alpha.txt");
    // What: `&reader.window` borrows the window out of the reader record.
    // Why: The helpers take a borrowed window, and the reader keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = reader.window;
    // ```
    let window = &reader.window;
    resize(window, 1100.0, 660.0);
    // What: `|| return ...` is a zero-argument arrow function; `is_some()` is true for `Some`.
    // Why: The tree arrives from a worker after the window opens.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // waitUntil(() => row(window, 'beta.txt') !== undefined);
    // ```
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "beta.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    let rows = window.get_tree_entries().row_count();
    let tree = container(
        window,
        AccessibleRole::Tree,
        "Project directories and files",
        rows,
    );
    let nested = item(&tree, "nested");
    assert_eq!(
        nested.accessible_expandable(),
        Some(true),
        "the directory row is not reported as expandable"
    );
    assert_eq!(
        nested.accessible_expanded(),
        Some(false),
        "the closed directory row is reported as expanded"
    );
    assert_row(&tree, "alpha.txt", true, 0);
    assert_row(&tree, "beta.txt", false, 1);
    assert_row(&tree, "nested", false, 2);
    assert_eq!(
        item(&tree, "nested").accessible_description().as_deref(),
        Some("Directory"),
        "the directory row does not say it is a directory"
    );
    // Opening another file through the row's default action moves the selected state to its row.
    item(&tree, "beta.txt").invoke_accessible_default_action();
    {
        let mut waiting = Waiting::eventually();
        while !(item(&tree, "beta.txt").accessible_item_selected() == Some(true)) {
            waiting.step("opening beta.txt through its row did not select its row");
        }
    }
    assert_row(&tree, "alpha.txt", false, 0);
    assert_row(&tree, "beta.txt", true, 1);
    // The opened file has a slot badge; its row's description names the shortcut the badge shows.
    let badge = window
        .get_tree_entries()
        .row_data(row(window, "beta.txt").expect("beta row") as usize)
        .expect("beta entry")
        .recency;
    assert!(
        !badge.is_empty(),
        "positive control: the opened file has no slot badge"
    );
    assert_eq!(
        item(&tree, "beta.txt")
            .accessible_description()
            .unwrap_or_default()
            .as_str(),
        format!("Source file, Ctrl+{badge}").as_str(),
        "the row of a file with a slot badge does not name its shortcut"
    );
    // A handle does not keep its element alive, so the row is looked up again after the file switch.
    item(&tree, "nested").invoke_accessible_expand_action();
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "inner.txt").is_some()) {
            waiting.step("the expand action did not list the directory's file");
        }
    }
    settle(window);
    assert_eq!(
        item(&tree, "nested").accessible_expanded(),
        Some(true),
        "the expand action did not report the directory row as expanded"
    );
    window.hide().expect("close accessible tree window");
}

/// The search results are a named list that counts its rows;
///  each result is a list item with its path and
/// position,
///  and the selected state follows Down and Up.
#[test]
fn search_results_report_role_name_position_and_the_selected_result() {
    let fixture = tempfile::tempdir().expect("disposable accessible search project");
    // Names match the query and contents do not, so each file is one result with its own path.
    fs::write(fixture.path().join("needle-one.txt"), "first\n").expect("search fixture");
    fs::write(fixture.path().join("needle-two.txt"), "second\n").expect("search fixture");
    let reader = search_tests::reader(fixture.path());
    let window = &reader.window;
    resize(window, 1100.0, 660.0);
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "needle-two.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    open(window);
    only(window, "Search query").set_accessible_value("needle");
    {
        let mut waiting = Waiting::eventually();
        while !(!window.get_search_busy() && window.get_search_entries().row_count() == 2) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    let results = container(window, AccessibleRole::List, "Search results", 2);
    // What: `row_data(0)` returns an `Option` holding a copy of the first result; `.path` is its path.
    // Why: The order of the two results is the search's, so the expected names are read from its model.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = window.searchEntries[0].path;
    // ```
    let first = window
        .get_search_entries()
        .row_data(0)
        .expect("first search result")
        .path;
    let second = window
        .get_search_entries()
        .row_data(1)
        .expect("second search result")
        .path;
    assert_row(&results, &first, true, 0);
    assert_row(&results, &second, false, 1);
    key(window, Key::DownArrow);
    settle(window);
    assert_row(&results, &first, false, 0);
    assert_row(&results, &second, true, 1);
    key(window, Key::UpArrow);
    settle(window);
    assert_row(&results, &first, true, 0);
    assert_row(&results, &second, false, 1);
    window.invoke_search_dismiss();
    window.hide().expect("close accessible search list window");
}

/// The location list is a list named by its title that counts its rows;
///  each location is a list item with
/// its label and position,
///  the selected state follows Down,
///  and a row's default action chooses it.
#[test]
fn location_list_reports_role_name_position_and_the_selected_location() {
    let shared = fixture(6);
    let window = &shared.window;
    // What: `Rc::new(Cell::new(-1))` makes a shared, changeable number; `Rc::clone` copies the pointer;
    // `move |index|` is a callback that owns that copy.
    // Why: The callback records which location the default action chose.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const chosen = { current: -1 };
    // window.onReferencesChoose(index => { chosen.current = index; });
    // ```
    let chosen = Rc::new(Cell::new(-1));
    let observer = Rc::clone(&chosen);
    window.on_references_choose(move |index| {
        observer.set(index);
    });
    // What: `vec![...]` builds an array of location records; `.into()` converts each literal to Slint's
    // string type; `ModelRc::from(Rc::new(VecModel::from(..)))` wraps it as the model the window accepts.
    // Why: The list's contents come from the language server in the application; here they are fixed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.referencesEntries = [{ label: 'src/one.ts:3', detail: '' }, ...];
    // ```
    let entries = vec![
        ReferenceEntry {
            label: "src/one.ts:3".into(),
            detail: "".into(),
        },
        ReferenceEntry {
            label: "lib/two.d.ts:12".into(),
            detail: "Outside project".into(),
        },
    ];
    window.set_references_title("2 references".into());
    window.set_references_entries(ModelRc::from(Rc::new(VecModel::from(entries))));
    window.set_references_selected(0);
    window.set_references_open(true);
    window.invoke_focus_references();
    settle(window);
    let places = container(window, AccessibleRole::List, "2 references", 2);
    assert_eq!(
        count(window, "2 references"),
        1,
        "the location list's title is read besides the list's own name"
    );
    assert_row(&places, "src/one.ts:3", true, 0);
    assert_row(&places, "lib/two.d.ts:12", false, 1);
    assert_eq!(
        item(&places, "lib/two.d.ts:12")
            .accessible_description()
            .as_deref(),
        Some("Outside project"),
        "the location row does not report why it is special"
    );
    key(window, Key::DownArrow);
    settle(window);
    assert_row(&places, "src/one.ts:3", false, 0);
    assert_row(&places, "lib/two.d.ts:12", true, 1);
    item(&places, "src/one.ts:3").invoke_accessible_default_action();
    assert_eq!(
        chosen.get(),
        0,
        "the location row's default action did not choose it"
    );
    window.hide().expect("close accessible location window");
}
