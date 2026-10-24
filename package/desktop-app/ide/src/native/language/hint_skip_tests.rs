//! A view whose hint request would ask only lines that stored answers of the same text revision asked about
//!  sends no request,
//!  and every other view sends one.
//!
//! The scripted server writes every request it receives to a report file,
//!  so these tests count the requests that reached it
//!  and read which lines each one asked.
//!  The view stands where the test scrolls it,
//!  and a request goes out once the lines have stayed the same for the settling time.

/// Fixtures,
///  the scripted server's definitions,
///  and the readiness wait.
use super::test_support::{LanguageReader, definitions, idle, project, reader, ready};
/// The stamp of the displayed text.
use crate::native::annotate::displayed;
/// A scroll the toolkit reports to the change handler.
use crate::native::overscan_tile_tests::scroll_to;
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// One code row is the least a line takes.
use ide_app::row_map::CODE_ROW;
/// The report is JSON lines.
use serde_json::Value;
/// External writes and the report path.
use std::{fs, path::PathBuf};

/// Lines of the fixture text.
const LINES: usize = 600;

/// Longer than the settling time of the hint request,
///  so a request that the view would send has gone out when this has passed.
const SETTLED_MS: u64 = 900;

/// The text:
/// `LINES` numbered lines of one length each.
fn numbered() -> String {
    let mut text = String::new();
    for index in 0..LINES {
        text.push_str(&format!("line {index:03} word\n"));
    }
    return text;
}

/// What:
///  A reader on the fixture text with the scripted server's report,
///  and the number of lines its view shows.
/// Why:
///  The ranges the tests scroll to depend on the view height.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scene = { reader: LanguageReader; report: string; view: number; project: Project };
/// ```
struct Scene {
    /// Removes the project when dropped.
    project: super::test_support::Project,
    /// The window with the language binding.
    reader: LanguageReader,
    /// The scripted server's report.
    report: PathBuf,
    /// Lines the view shows,
    ///  as the hint request counts them.
    view: usize,
}

/// What:
///  Open the fixture text with the scripted server answering its hint requests as `steps` says,
///  and wait until the requests of the start are over.
///  The empty `steps` answers every request.
/// Why:
///  Every test starts from a view at the top whose first request was answered or is the one `steps` scripts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scene(steps: string): Scene
/// ```
fn scene(steps: &str) -> Scene {
    let fixture = project(&[("main.scripted", &numbered())]);
    let report = fixture.outside.join("report.jsonl");
    let report_text = report.display().to_string();
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(&[("REPORT", &report_text), ("HINT_STEPS", steps)], None),
    );
    ready(&reader);
    let view = ((reader.window.get_viewport_height() / CODE_ROW).ceil() as usize).max(1);
    assert!(
        view >= 4 && view * 8 < LINES,
        "the window shows {view} lines; the tests need between 4 and {} to scroll within the text",
        LINES / 8
    );
    let opened = Scene {
        project: fixture,
        reader,
        report,
        view,
    };
    // The start asks once for the lines at the top.
    wait_requests(&opened, 1);
    idle(SETTLED_MS);
    assert_eq!(
        asked(&opened),
        [0],
        "the start asked for other lines than the top"
    );
    return opened;
}

/// What:
///  The first line of every hint request the scripted server received,
///  in the order it received them.
/// Why:
///  A request asks the lines from one view height above the first visible line,
///  so this names the view that sent it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function asked(scene: Scene): number[]
/// ```
fn asked(scene: &Scene) -> Vec<usize> {
    // An absent report is empty: the server creates it when it starts.
    let text = fs::read_to_string(&scene.report).unwrap_or_default();
    let mut lines = Vec::new();
    for line in text.lines() {
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if entry["received"] != "textDocument/inlayHint" {
            continue;
        }
        let start = entry["params"]["range"]["start"]["line"]
            .as_u64()
            .expect("the request names the first line it asks");
        lines.push(start as usize);
    }
    return lines;
}

/// Wait until the server received `count` hint requests.
fn wait_requests(scene: &Scene, count: usize) {
    let mut waiting = Waiting::language();
    while !(asked(scene).len() >= count) {
        waiting.step("the scripted server did not receive the expected hint request");
    }
}

/// Wait until the store holds an answer that asked the lines `first` up to `last`.
fn wait_answer(scene: &Scene, first: usize, last: usize) {
    let mut waiting = Waiting::language();
    loop {
        {
            let current = scene.reader.source.borrow();
            if let Some(held) = current.annotations.hints(displayed(&current))
                && held.first_line == first
                && held.last_line == last
            {
                return;
            }
        }
        waiting.step("the store did not take the expected answer");
    }
}

/// Scroll so that `line` is the first visible line.
fn show(scene: &Scene, line: usize) {
    let offset = scene.reader.source.borrow().row_map.block_top(line);
    scroll_to(&scene.reader.window, offset);
}

/// What:
///  Show `line`,
///  wait for the request of that view,
///  and wait for its answer to be stored.
///  The request must be the one after `before` earlier ones.
/// Why:
///  The tests build up the lines the answers cover one view at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function visit(scene: Scene, line: number, before: number): Promise<void>
/// ```
fn visit(scene: &Scene, line: usize, before: usize) {
    show(scene, line);
    wait_requests(scene, before + 1);
    wait_answer(scene, line - scene.view, line + 2 * scene.view);
}

/// A first line far from the top,
///  with room for the views a test scrolls through.
const FAR: usize = 200;

/// Two answered views leave the lines between them covered:
///  a view between them sends no request.
#[test]
fn a_view_inside_the_answered_lines_sends_no_request() {
    let scene = scene("");
    let view = scene.view;
    visit(&scene, FAR, 1);
    visit(&scene, FAR + view, 2);
    let before = asked(&scene);
    assert_eq!(before, [0, FAR - view, FAR]);
    // The lines of this view's request lie between the first view's request and the second one's.
    show(&scene, FAR + view / 2);
    idle(SETTLED_MS);
    assert_eq!(
        asked(&scene),
        before,
        "a view whose request asks only answered lines sent a request"
    );
}

/// A view whose request asks a line no answer asked about sends a request,
///  once its lines have stayed the same for the settling time.
#[test]
fn a_view_leaving_the_answered_lines_sends_a_request() {
    let scene = scene("");
    let view = scene.view;
    visit(&scene, FAR, 1);
    visit(&scene, FAR + view, 2);
    show(&scene, FAR + view / 2);
    idle(SETTLED_MS);
    // One line above the lines the first answer asked about.
    show(&scene, FAR - 1);
    wait_requests(&scene, 4);
    assert_eq!(
        asked(&scene),
        [0, FAR - view, FAR, FAR - 1 - view],
        "the view that leaves the answered lines did not send its request"
    );
    // Back inside what is answered now: no request.
    show(&scene, FAR + view / 2);
    idle(SETTLED_MS);
    assert_eq!(asked(&scene).len(), 4, "a view inside the answers sent a request");
}

/// An edit from outside makes another revision:
///  the lines answers asked about in the earlier revision do not count,
///  so the same view sends a request.
#[test]
fn an_edit_forgets_the_lines_the_earlier_revision_asked_about() {
    let scene = scene("");
    let view = scene.view;
    visit(&scene, FAR, 1);
    visit(&scene, FAR + view, 2);
    let revision = scene.reader.source.borrow().document.revision();
    fs::write(
        scene.project.root.join("main.scripted"),
        numbered() + "appended\n",
    )
    .expect("external change");
    let mut waiting = Waiting::language();
    while !(scene.reader.source.borrow().document.revision() != revision) {
        waiting.step("the reload did not arrive");
    }
    // The reload asks again for the lines of the view, and its answer is stored for the new revision.
    wait_answer(&scene, FAR, FAR + 3 * view);
    idle(SETTLED_MS);
    let before = asked(&scene).len();
    // These lines lie between the two answers of the earlier revision, which is gone.
    show(&scene, FAR + view / 2);
    wait_requests(&scene, before + 1);
}

/// A request that is still pending covers nothing:
///  the lines it asks do not make a later view's request unnecessary.
#[test]
fn a_pending_request_does_not_cover_its_lines() {
    // The server answers the start and the second request, and never answers the first view's.
    let scene = scene("answer,silent");
    let view = scene.view;
    show(&scene, FAR);
    wait_requests(&scene, 2);
    visit(&scene, FAR + view, 2);
    idle(SETTLED_MS);
    let before = asked(&scene);
    assert_eq!(before, [0, FAR - view, FAR]);
    // Between the two requests; only the answered one counts, and it does not reach these lines.
    show(&scene, FAR + view / 2);
    wait_requests(&scene, before.len() + 1);
}

/// A request the server failed covers nothing.
#[test]
fn a_failed_request_does_not_cover_its_lines() {
    let scene = scene("answer,error");
    let view = scene.view;
    show(&scene, FAR);
    wait_requests(&scene, 2);
    idle(SETTLED_MS);
    visit(&scene, FAR + view, 2);
    idle(SETTLED_MS);
    let before = asked(&scene);
    assert_eq!(before, [0, FAR - view, FAR]);
    show(&scene, FAR + view / 2);
    wait_requests(&scene, before.len() + 1);
}

/// A worker told to open the file again holds no earlier request:
///  the lines answers asked about before do not count.
#[test]
fn opening_the_file_again_forgets_the_lines_asked_about() {
    let scene = scene("");
    let view = scene.view;
    visit(&scene, FAR, 1);
    visit(&scene, FAR + view, 2);
    {
        let binding = scene.reader.binding.as_ref().expect("language binding");
        binding.language.borrow_mut().reopen = true;
    }
    // The next tick tells the worker about the displayed file again.
    let mut waiting = Waiting::language();
    while scene
        .reader
        .binding
        .as_ref()
        .is_some_and(|binding| binding.language.borrow().reopen)
    {
        waiting.step("the file was not displayed to the worker again");
    }
    idle(SETTLED_MS);
    let before = asked(&scene).len();
    show(&scene, FAR + view / 2);
    wait_requests(&scene, before + 1);
}
