//! Controls for the question a commit asks its service before it builds anything:
//!  a drawn tree is handed over,
//!  a service that holds none answers so,
//!  a service that says nothing costs the deadline and no more,
//!  a socket nobody listens on costs no wait at all,
//!  and an answer naming a tree that is not this worktree's drawn one is refused.

use super::*;
use crate::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use crate::service_protocol::{ServiceMessage, encode_service};
use crate::shadow_drawn::drawn_root;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::transaction_git::GitContext;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What a stand-in service answers.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Behaviour {
    /// It greets and then answers with this tree.
    Hands(PathBuf),
    /// It greets and then says it holds none.
    HoldsNone,
    /// It greets and then says nothing at all.
    SaysNothing,
    /// It accepts the connection and closes it without a greeting.
    ClosesAtOnce,
}

/// What:
///  A stand-in service at a socket of its own that answers one question,
///  and the socket's path.
/// Why:
///  The ask is one exchange,
///  so a control needs a service that greets,
///  reads the question and answers it,
///  and one that does either badly.
///  The socket sits in a short folder because one socket address holds 107 bytes and a fixture root does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const socket = aStandIn('a-control', Behaviour.HoldsNone);
/// ```
fn a_stand_in(name: &str, behaviour: Behaviour) -> PathBuf {
    let folder: PathBuf = std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("a home directory")
        .join("temp/agent")
        .join(format!("ask{}", std::process::id()))
        .join(name);
    std::fs::create_dir_all(folder.as_path()).expect("the short folder");
    crate::test_support::track_for_this_thread(folder.parent().expect("the folder's parent").to_path_buf().as_path());
    let socket: PathBuf = folder.join("service.sock");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("a stand-in socket");
    std::thread::spawn(move || {
        let Ok((stream, _address)) = listener.accept() else { return };
        let mut writer = stream.try_clone().expect("the second half of the connection");
        if behaviour == Behaviour::ClosesAtOnce {
            return;
        }
        let welcome: String = format!("{}\n", encode_service(&ServiceMessage::Welcome { version: crate::service_protocol::PROTOCOL_VERSION, release: Some(String::from("test")) }));
        if writer.write_all(welcome.as_bytes()).is_err() {
            return;
        }
        let mut reader: BufReader<std::os::unix::net::UnixStream> = BufReader::new(stream);
        let mut line: String = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }
        let answer: Option<String> = match behaviour {
            Behaviour::Hands(tree) => Some(encode_service(&ServiceMessage::Drawn { tree })),
            Behaviour::HoldsNone => Some(encode_service(&ServiceMessage::NoShadow)),
            Behaviour::SaysNothing | Behaviour::ClosesAtOnce => None,
        };
        if let Some(text) = answer {
            let _sent = writer.write_all(format!("{text}\n").as_bytes());
            return;
        }
        // A service that says nothing keeps its connection open, as a stopped one does: closing it would be an answer,
        //  and the wait this control measures is the deadline and not a close.
        std::thread::sleep(DRAWN_ANSWER_DEADLINE * 4);
    });
    return socket;
}

/// The facts one invocation's shadow is made from.
fn template_of(repo: &Path) -> crate::shadow_create::ShadowTemplate {
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo, false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo, layout).expect("capture");
    return crate::shadow_create::ShadowTemplate::of(&context, &capture.layout).expect("template");
}

/// A repository with one commit on `main`,
/// and the scratch tree name a commit of it would ask with.
fn repository_with_a_scratch(root: &Path) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let scratch: PathBuf = repo.join(".git").join("monochromatic-git-policy-cli-scratch").join("monochromatic-git-policy-cli-service-one-0");
    std::fs::create_dir_all(scratch.as_path()).expect("the commit's scratch tree");
    return (repo, scratch);
}

/// What:
///  A service that drew a tree ahead of the commit hands it over,
///  and the commit reads the path it was given.
#[test]
fn a_drawn_tree_is_handed_over() {
    let root: PathBuf = fixture("shadow-ask-drawn");
    let (repo, scratch) = repository_with_a_scratch(root.as_path());
    let template: crate::shadow_create::ShadowTemplate = template_of(repo.as_path());
    let drawn: PathBuf = drawn_root(&template).join("drawn-00000000000000000000000000000001");
    std::fs::create_dir_all(drawn.as_path()).expect("a drawn tree");
    let socket: PathBuf = a_stand_in("drawn", Behaviour::Hands(drawn.clone()));
    assert_eq!(ask_for_a_drawn_shadow(&Bind::Path(socket), &template, scratch.as_path()), Some(drawn));
    remove(root.as_path());
}

/// What:
///  A service that holds no drawn tree answers so,
///  and the commit is left to take the pool's member or make its own creation.
#[test]
fn a_service_that_holds_none_leaves_the_commit_to_the_pool() {
    let root: PathBuf = fixture("shadow-ask-none");
    let (repo, scratch) = repository_with_a_scratch(root.as_path());
    let template: crate::shadow_create::ShadowTemplate = template_of(repo.as_path());
    let socket: PathBuf = a_stand_in("none", Behaviour::HoldsNone);
    assert_eq!(ask_for_a_drawn_shadow(&Bind::Path(socket), &template, scratch.as_path()), None);
    remove(root.as_path());
}

/// What:
///  A service that says nothing costs the deadline and no more,
///  and the commit goes on to its own path.
/// Why:
///  A service stopped at any instruction greets nothing,
///  and a commit that waited for one would wait for as long as it stayed stopped.
#[test]
fn a_service_that_says_nothing_costs_the_deadline_and_no_more() {
    let root: PathBuf = fixture("shadow-ask-silent");
    let (repo, scratch) = repository_with_a_scratch(root.as_path());
    let template: crate::shadow_create::ShadowTemplate = template_of(repo.as_path());
    let socket: PathBuf = a_stand_in("silent", Behaviour::SaysNothing);
    let began: Instant = Instant::now();
    assert_eq!(ask_for_a_drawn_shadow(&Bind::Path(socket), &template, scratch.as_path()), None);
    let waited: Duration = began.elapsed();
    assert!(waited >= DRAWN_ANSWER_DEADLINE, "the deadline is what ended the wait, and it took {waited:?}");
    assert!(waited < DRAWN_ANSWER_DEADLINE * 10, "the wait is the deadline and not a wait of its own: {waited:?}");
    remove(root.as_path());
}

/// What:
///  A socket nobody listens on is answered without the ask,
///  and a service that closes the connection at once is no service.
#[test]
fn a_socket_nobody_listens_on_or_one_that_closes_is_answered_without_a_tree() {
    let root: PathBuf = fixture("shadow-ask-gone");
    let (repo, scratch) = repository_with_a_scratch(root.as_path());
    let template: crate::shadow_create::ShadowTemplate = template_of(repo.as_path());
    let missing: PathBuf = root.join("nothing-here.sock");
    assert_eq!(ask_for_a_drawn_shadow(&Bind::Path(missing), &template, scratch.as_path()), None);
    let socket: PathBuf = a_stand_in("closes", Behaviour::ClosesAtOnce);
    assert_eq!(ask_for_a_drawn_shadow(&Bind::Path(socket), &template, scratch.as_path()), None);
    remove(root.as_path());
}

/// What:
///  An answer that names a tree outside this worktree's drawn folder is refused,
///  and the commit takes its own path.
/// Why:
///   A commit removes the tree it was handed when the service did not take it,
///  so a path from the socket that is not one of the wrapper's drawn trees of this worktree
///  must never reach that removal.
#[test]
fn an_answer_naming_another_tree_is_refused() {
    let root: PathBuf = fixture("shadow-ask-elsewhere");
    let (repo, scratch) = repository_with_a_scratch(root.as_path());
    let template: crate::shadow_create::ShadowTemplate = template_of(repo.as_path());
    let elsewhere: PathBuf = root.join("drawn-00000000000000000000000000000002");
    std::fs::create_dir_all(elsewhere.as_path()).expect("a tree of another place");
    let socket: PathBuf = a_stand_in("elsewhere", Behaviour::Hands(elsewhere));
    assert_eq!(ask_for_a_drawn_shadow(&Bind::Path(socket), &template, scratch.as_path()), None);
    remove(root.as_path());
}
