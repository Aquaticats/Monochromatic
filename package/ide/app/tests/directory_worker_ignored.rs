//! The reader decides ignored entries beside each listing,
//!  from the project's own rule files,
//!  and a changed rule file reaches the rows.

/// Disposable fixture directories,
///  in memory when the system has one.
mod memory_fixture;

/// The consumer combines the read-only workspace,
///  UI-thread tree,
///  background reader,
///  and its matcher.
use ide_app::{
    directory_worker::DirectoryWorker, file_tree::FileTree, git_ignore::GlobalSource,
    workspace::Workspace,
};
/// Fixture files are private;
///  worker waits have an explicit deadline.
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

/// Write a file,
///  creating its folders.
fn put(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().expect("parent folder")).expect("fixture folder");
    fs::write(path, text).expect("fixture file");
}

/// One tree and one reader over a fixture,
///  with the global excludes file fixed to none.
struct Session {
    /// Canonical project root.
    root: std::path::PathBuf,
    /// The UI-side tree.
    tree: FileTree,
    /// The background reader.
    worker: DirectoryWorker,
}

/// Open a session on the folder.
fn open(folder: &Path) -> Session {
    let workspace = Workspace::new(folder).expect("workspace");
    let root = workspace.root().to_path_buf();
    let worker = DirectoryWorker::with_global_ignores(workspace, GlobalSource::File(None))
        .expect("directory worker");
    return Session {
        tree: FileTree::new(&root),
        root,
        worker,
    };
}

/// A session reads a directory and applies the reply.
impl Session {
    /// List one directory now and wait for its reply.
    fn list(&mut self, directory: &Path) {
        assert!(
            self.worker
                .request(&mut self.tree, directory)
                .expect("request"),
            "the reader is free"
        );
        let start = Instant::now();
        while self.worker.is_busy() {
            self.worker.poll(&mut self.tree).expect("reply");
            assert!(
                start.elapsed() < Duration::from_secs(3),
                "directory worker did not reply"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// Expand a directory row and list it.
    fn expand(&mut self, name: &str) {
        let path = self.root.join(name);
        self.tree.toggle(&path).expect("expand");
        self.list(&path);
    }

    /// The visible rows that are marked ignored,
    ///  as paths relative to the root.
    fn dimmed(&self) -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();
        for row in self.tree.rows() {
            if row.ignored {
                paths.push(
                    row.entry
                        .path
                        .strip_prefix(&self.root)
                        .expect("inside the root")
                        .display()
                        .to_string(),
                );
            }
        }
        return paths;
    }
}

/// Patterns,
///  a negation,
///  a nested file,
///  and an ignored directory with files inside reach the visible rows.
#[test]
fn listed_rows_carry_what_the_rule_files_say() {
    let fixture = memory_fixture::directory("ide-ignored-rows-");
    let root = fixture.path();
    fs::create_dir(root.join(".git")).expect("git folder");
    put(
        root,
        ".gitignore",
        "*.log\n!important.log\nbuild/\n/anchored.txt\n",
    );
    put(root, "src/.gitignore", "*.tmp\n");
    for name in [
        "app.log",
        "important.log",
        "main.rs",
        "anchored.txt",
        "sub/anchored.txt",
        "build/out.o",
        "src/lib.rs",
        "src/scratch.tmp",
        "scratch.tmp",
    ] {
        put(root, name, "x");
    }
    let mut session = open(root);
    let top = session.root.clone();
    session.list(&top);
    for directory in ["build", "src", "sub"] {
        session.expand(directory);
    }
    assert_eq!(
        session.dimmed(),
        vec![
            "anchored.txt",
            "app.log",
            "build",
            "build/out.o",
            "src/scratch.tmp"
        ],
        "file, anchored and directory patterns dim; the negation, the root's scratch.tmp, sub/anchored.txt and plain files stay; \
         the ignored directory's contents are dimmed too"
    );
}

/// Without a `.git` above the project nothing is dimmed,
///  whatever a `.gitignore` says.
#[test]
fn outside_a_git_work_tree_nothing_is_dimmed() {
    let fixture = memory_fixture::directory("ide-ignored-none-");
    let root = fixture.path();
    put(root, ".gitignore", "*.log\n");
    put(root, "app.log", "x");
    let mut session = open(root);
    let top = session.root.clone();
    session.list(&top);
    assert_eq!(session.tree.rows().len(), 2);
    assert!(session.dimmed().is_empty());
}

/// Editing `.gitignore` between two listings changes the rows of that listing,
///  and raises the flag that rereads the directories below.
#[test]
fn a_changed_gitignore_changes_the_rows_and_raises_the_reread_flag() {
    let fixture = memory_fixture::directory("ide-ignored-edit-");
    let root = fixture.path();
    fs::create_dir(root.join(".git")).expect("git folder");
    put(root, ".gitignore", "*.log\n");
    put(root, "app.log", "x");
    put(root, "notes.txt", "x");
    put(root, "sub/deep.txt", "x");
    let mut session = open(root);
    let top = session.root.clone();
    session.list(&top);
    session.expand("sub");
    assert_eq!(session.dimmed(), vec!["app.log"]);
    assert!(
        !session.tree.take_ignore_sources_changed(),
        "first listings raise nothing"
    );
    // Longer than before, so the length alone moves the stamp even within one timestamp tick.
    put(root, ".gitignore", "*.txt\n# edited\n");
    session.list(&top);
    assert_eq!(
        session.dimmed(),
        vec!["notes.txt"],
        "the root listing follows the new rules at once"
    );
    assert!(
        session.tree.take_ignore_sources_changed(),
        "the stamp of the root's .gitignore moved, so the shown directories below must be listed again"
    );
    // sub/deep.txt still shows the old answer until its directory is listed again.
    assert!(!session.dimmed().contains(&"sub/deep.txt".to_string()));
    let sub = session.root.join("sub");
    session.list(&sub);
    assert_eq!(session.dimmed(), vec!["notes.txt", "sub/deep.txt"]);
}
