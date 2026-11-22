//! What:
//!  Controls of the wake a released held push gives the service:
//!  who is asked,
//!  and that a caller with no service asks nobody.
//! Why:
//!  The step reaches a socket and a manager in production,
//! so the seam is what a control can check:
//! the line of a held
//!  record must read the same whether or not a service answers,
//! and nothing may reach a real service from a control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const woken: string[] = []; heldPushNotices(gitDir, record, { wake: (dir) => woken.push(dir) });
//! ```

use super::{NothingWoken, WakeTheService};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// What:
///  The wake a caller with no service gives asks nothing,
/// which is what a command with no reachable service leaves undone.
/// Why:
///   A released record stays in `pending` in that case,
/// and the next service to look makes it,
/// so the silence loses
///  nothing.
#[test]
fn a_caller_with_no_service_asks_nobody() {
    let woken: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    struct Recorder<'a>(&'a RefCell<Vec<PathBuf>>);
    impl WakeTheService for Recorder<'_> {
        fn wake(&self, git_dir: &Path) {
            self.0.borrow_mut().push(git_dir.to_path_buf());
        }

        fn forget_programs(&self, _git_dir: &Path, _records: &[String]) {
            panic!("this control has no record that can no longer run");
        }
    }
    NothingWoken.wake(Path::new("/repository/.git"));
    assert!(woken.borrow().is_empty(), "nothing was recorded, because nothing was asked");
    Recorder(&woken).wake(Path::new("/repository/.git"));
    assert_eq!(woken.borrow().len(), 1, "the seam asks whatever it holds");
}

/// What:
///  A caller that gives up a push record's values asks the service to forget them by that record's id,
///  and a caller with no service asks nobody.
/// Why:
///   The values live in the service's memory alone,
/// and a record a command removes here leaves nothing it could learn the ids from
///  (`service_push_programs_held_in_memory`).
#[test]
fn a_caller_tells_the_service_which_records_can_no_longer_run() {
    let forgotten: RefCell<Vec<(PathBuf, Vec<String>)>> = RefCell::new(Vec::new());
    struct Recorder<'a>(&'a RefCell<Vec<(PathBuf, Vec<String>)>>);
    impl WakeTheService for Recorder<'_> {
        fn wake(&self, _git_dir: &Path) {}

        fn forget_programs(&self, git_dir: &Path, records: &[String]) {
            self.0.borrow_mut().push((git_dir.to_path_buf(), records.to_vec()));
        }
    }
    let records: Vec<String> = vec![String::from("push-0123456789abcdef0123456789abcdef01234567-aaaa")];
    NothingWoken.forget_programs(Path::new("/repository/.git"), records.as_slice());
    assert!(forgotten.borrow().is_empty(), "nothing was recorded, because nothing was asked");
    Recorder(&forgotten).forget_programs(Path::new("/repository/.git"), records.as_slice());
    assert_eq!(forgotten.borrow().len(), 1, "the seam asks whatever it holds");
    assert_eq!(forgotten.borrow()[0].1, records, "the record that can no longer run is the one named");
}

/// What:
///  A command that gives up the values of a settled record writes nothing about the worktree when the manager says this
///  worktree's service does not run,
///  and provisioning the same route does write the program link.
/// Why:
///   The seam asked the manager before it provisioned anything,
///  and the order it replaced provisioned first:
/// that wrote the unit links of a worktree a command only wanted to tell nothing
///  (the low a Claude Sonnet review of this branch left).
///  No control constructed `AskTheService`,
///  so putting the old order back left every control green
///  (`closure3-mutation-the-seam-with-provision-before-the-ask.log` in the review's folder).
///  The route is handed in for that reason:
///  the seam reads the process environment otherwise,
///  and this control proves the order with no session bus and no manager.
///  The positive half is the route's own provisioning,
///  which writes the program link before it reaches for the bus,
///  so the assertion above can fail.
#[test]
fn a_forget_for_a_service_that_is_not_running_provisions_nothing() {
    use crate::service_address_of_a_worktree::socket_address_of;
    use crate::service_instance::{longest_runtime_directory, program_link};
    use crate::service_provision::{ProvisionHost, provision};
    use crate::service_route::Route;
    use crate::test_support::{fixture, remove, runtime_directory_parent, runtime_directory_under};
    let root: PathBuf = fixture("forget-no-service");
    let instance: String = String::from("0123456789abcdef0123456789abcdef");
    let longest: usize = longest_runtime_directory(instance.as_str());
    // The runtime directory stays inside the longest one a socket path carries, so provisioning refuses nothing about it.
    let tag: &str = "fg";
    let parent: PathBuf = runtime_directory_parent("gf", longest.saturating_sub(1 + tag.len()));
    let runtime: PathBuf = runtime_directory_under(parent.as_path(), parent.as_os_str().len() + 1 + tag.len(), tag);
    let git_dir: PathBuf = root.join("repository/.git/worktrees/topic");
    std::fs::create_dir_all(git_dir.as_path()).expect("the worktree's Git directory");
    let program: PathBuf = root.join("programs/monochromatic-git-policy-cli");
    std::fs::create_dir_all(program.parent().expect("the program's folder")).expect("the program's folder");
    std::fs::write(program.as_path(), b"program").expect("the wrapper's own executable");
    let host: ProvisionHost = ProvisionHost {
        // No bus listens here, so the manager answers nothing and this worktree's service is not running.
        bus: runtime.join("bus"),
        runtime_dir: runtime,
        user_unit_dir: Some(root.join("config/systemd/user")),
        service_binary: program,
        git: PathBuf::from("/usr/bin/git"),
        instance,
        git_dir: git_dir.clone(),
        retirement_wait: std::time::Duration::from_millis(1),
    };
    let route: Route = Route {
        address: socket_address_of(host.runtime_dir.as_path(), host.instance.as_str()),
        provision: Some(host.clone()),
    };
    let records: Vec<String> = vec![String::from("push-0123456789abcdef0123456789abcdef01234567-aaaa")];
    super::forget_programs_through(&route, git_dir.as_path(), records.as_slice());
    assert!(
        !program_link(host.runtime_dir.as_path()).exists(),
        "the manager said this worktree's service does not run, so nothing of its units was written: {}",
        program_link(host.runtime_dir.as_path()).display()
    );
    // The positive half: the same route provisioned writes the program link before it asks the manager, so the assertion
    //  above is one that can fail. A bus that is not there is what the provisioning refuses at, after that write.
    let _refused: Result<(), crate::service_messages::ClientFailure> = provision(&host);
    assert!(
        program_link(host.runtime_dir.as_path()).exists(),
        "provisioned, this route writes the program link, so the order control's assertion can fail"
    );
    remove(root.as_path());
}
