//! Real pnpm's workspace override must load from the new worktree even when the home path is a symlink.
//! The package tarball, shim, store and both spellings of home all belong to the fixture.

use super::super::support::{bounded_for, give_service_environment, observe, wrapper_first_path};
use super::{Fixture, Observed, available_or_acknowledged, fixture, git, link_real, on_path, real_pnpm, remove, repository, stderr_of, wrapped};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

/// Write the workspace and seed its own global store through the physical home path.
/// The archive declares `proper-lockfile`; the install can only get it from the workspace's shim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = initRepository(); writeOverrideWorkspace(source); installOffline(source, physicalHome); commitFixture(source);
/// ```
fn installed_workspace(fixture: &Fixture, physical_home: &Path) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let archive_source: PathBuf = fixture.root.join("archive-source");
    std::fs::create_dir_all(archive_source.join("package")).expect("consumer package directory");
    std::fs::write(archive_source.join("package/package.json"), b"{\"name\":\"consumer\",\"version\":\"1.0.0\",\"type\":\"module\",\"main\":\"index.js\",\"dependencies\":{\"proper-lockfile\":\"1.0.0\"}}\n").expect("consumer manifest");
    std::fs::write(archive_source.join("package/index.js"), b"export { default } from 'proper-lockfile';\n").expect("consumer entry");
    let path = wrapper_first_path(fixture);
    let tar: Output = bounded_for(fixture, Path::new("/usr/bin/tar"), &path, "15")
        .args(["--create", "--gzip", "--file"]).arg(repo.join("consumer.tgz"))
        .arg("--directory").arg(archive_source).arg("package").output().expect("create package tarball");
    assert!(tar.status.success(), "{}", String::from_utf8_lossy(&tar.stderr));
    std::fs::create_dir(repo.join("shim")).expect("shim directory");
    std::fs::write(repo.join("shim/package.json"), b"{\"name\":\"proper-lockfile\",\"version\":\"1.0.0\",\"type\":\"module\",\"main\":\"index.js\"}\n").expect("shim manifest");
    std::fs::write(repo.join("shim/index.js"), b"export default import.meta.url;\n").expect("shim entry");
    std::fs::write(repo.join("package.json"), b"{\"private\":true,\"dependencies\":{\"consumer\":\"file:consumer.tgz\"}}\n").expect("workspace manifest");
    std::fs::write(repo.join("pnpm-workspace.yaml"), b"storeDir: ${HOME}/.local/share/pnpm/store\nvirtualStoreType: global\noverrides:\n  proper-lockfile: link:shim\n").expect("store and override settings");
    std::fs::write(repo.join(".gitignore"), b"node_modules/\n").expect("ignore install output");
    let install: Output = bounded_for(fixture, &fixture.root.join("bin/pnpm"), &path, "30")
        .env("HOME", physical_home).current_dir(&repo).args(["install", "--offline"]).output().expect("seed the fixture store");
    assert!(install.status.success(), "{}\n{}", String::from_utf8_lossy(&install.stdout), String::from_utf8_lossy(&install.stderr));
    git(fixture, &repo, &["add", "--", ".gitignore", "consumer.tgz", "package.json", "pnpm-workspace.yaml", "pnpm-lock.yaml", "shim"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=workspace"]);
    return repo;
}

/// The wrapper chooses the caller's pnpm, but a queued install inherits the service's home.
/// Both have the alias here; the seed install alone used the physical spelling.
/// The new worktree sits at another depth, so the copied entry cannot resolve and pnpm must write a new one.
/// A same-depth copy can reuse the source's slot even with a physical home; that is a different pnpm defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktree = wrappedAdd(source, { HOME: alias }); expect(resolve(linkInFixtureStore(worktree))).toBe(worktree.shim);
/// expect(importConsumer(worktree)).toBe(worktree.shimUrl);
/// ```
#[test]
fn real_pnpm_links_the_new_worktrees_override_with_a_linked_home() {
    let control: &str = "real_pnpm_links_the_new_worktrees_override_with_a_linked_home";
    let pnpm: Option<PathBuf> = real_pnpm();
    let node: Option<PathBuf> = on_path("node");
    if !available_or_acknowledged(pnpm.as_deref(), "pnpm", control) || !available_or_acknowledged(node.as_deref(), "node", control) {
        return;
    }
    let fixture: Fixture = fixture("repair-pnpm-linked-home");
    link_real(&fixture, "pnpm", pnpm.as_deref().expect("real pnpm"));
    link_real(&fixture, "node", node.as_deref().expect("real node"));
    let physical_home: PathBuf = fixture.root.join("physical/extra-level/home");
    let linked_home: PathBuf = fixture.root.join("home-link");
    std::fs::create_dir_all(&physical_home).expect("physical home");
    std::os::unix::fs::symlink(&physical_home, &linked_home).expect("linked home");
    give_service_environment(&fixture, "HOME", linked_home.to_str().expect("fixture home path"));
    let repo: PathBuf = installed_workspace(&fixture, &physical_home);
    let source: PathBuf = std::fs::canonicalize(repo.join("node_modules/consumer")).expect("source consumer");
    let source_link: PathBuf = source.with_file_name("proper-lockfile");
    let source_shim: PathBuf = std::fs::canonicalize(&source_link).expect("the seed install's link resolves");
    let topic: PathBuf = fixture.root.join("worktrees/topic");
    let added: Observed = observe(wrapped(&fixture).env("HOME", &linked_home).current_dir(&repo)
        .args(["worktree", "add", "-b", "topic", "../worktrees/topic"]), b"");
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let consumer: PathBuf = std::fs::canonicalize(topic.join("node_modules/consumer")).expect("consumer resolves from the new worktree");
    let store: PathBuf = std::fs::canonicalize(physical_home.join(".local/share/pnpm/store")).expect("fixture store");
    assert!(consumer.starts_with(&store), "consumer resolved outside the fixture store: {}", consumer.display());
    assert_ne!(consumer, source, "the new worktree reused the source's slot");
    assert_eq!(std::fs::canonicalize(source_link).expect("the source link still resolves"), source_shim);
    let link: PathBuf = consumer.with_file_name("proper-lockfile");
    let target: PathBuf = std::fs::read_link(&link).expect("override link in the fixture store");
    let resolved = std::fs::canonicalize(&link);
    let expected: PathBuf = std::fs::canonicalize(topic.join("shim")).expect("new worktree's shim");
    let imported: Output = bounded_for(&fixture, &fixture.root.join("bin/node"), &wrapper_first_path(&fixture), "15")
        .env("HOME", &linked_home).current_dir(&topic).args(["--input-type=module", "--eval",
        "import shim from 'consumer'; import { fileURLToPath } from 'node:url'; console.log(fileURLToPath(shim));"])
        .output().expect("load the installed consumer");
    remove(&fixture);
    let node_stderr = String::from_utf8_lossy(&imported.stderr);
    assert_eq!(resolved.ok().as_ref(), Some(&expected), "{} -> {} did not resolve to the new worktree's shim; Node said:\n{node_stderr}", link.display(), target.display());
    assert!(imported.status.success(), "{node_stderr}");
    assert_eq!(String::from_utf8_lossy(&imported.stdout).trim(), expected.join("index.js").to_str().expect("fixture path"));
}
