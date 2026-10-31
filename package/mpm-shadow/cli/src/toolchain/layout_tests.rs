//! Unit tests of the scopes,
//! the install layout and the binary search,
//! all against injected environments and
//! disposable directories.

use std::fs;
use std::path::{Path, PathBuf};

use super::fixtures::sample_lock;
use super::layout::{
    applies_to_family, bin_search_dirs, find_bin_under, find_tool_bin, install_dir_name, is_executable,
    manager_native_bin_dirs, platform_id, tool_install_dir,
};
use super::model::{LockedTool, OsFamily};
use super::scope::{EnvMap, ScopeKind, find_repo_root, global_scope, home_dir, repo_scope, resolve_scope};
use crate::test_scratch::scratch;

fn sample_tool(id: &str) -> LockedTool {
    let lock = sample_lock();
    return lock.tool(id).expect("the sample lock holds the tool").clone();
}

fn write_executable(path: &Path) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("make the directory");
    fs::write(path, "#!/bin/sh\n").expect("write the file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

#[test]
fn platform_ids_name_os_cpu_and_musl() {
    assert_eq!(platform_id("linux", "x86_64", false), "linux-x64");
    assert_eq!(platform_id("linux", "aarch64", false), "linux-arm64");
    assert_eq!(platform_id("linux", "x86_64", true), "linux-x64-musl");
    assert_eq!(platform_id("linux", "aarch64", true), "linux-arm64-musl");
    assert_eq!(platform_id("macos", "aarch64", false), "macos-arm64");
    assert_eq!(platform_id("macos", "aarch64", true), "macos-arm64");
    assert_eq!(platform_id("windows", "x86_64", false), "windows-x64");
    assert_eq!(platform_id("freebsd", "riscv64", false), "freebsd-riscv64");
}

#[test]
fn install_directory_names_collapse_runs_of_unusual_characters() {
    assert_eq!(install_dir_name("node"), "node");
    assert_eq!(install_dir_name("cargo:fd-find"), "cargo-fd-find");
    assert_eq!(install_dir_name("npm:@actions/languageserver"), "npm-actions-languageserver");
    assert_eq!(install_dir_name("llama.cpp"), "llama.cpp");
}

#[test]
fn a_tool_version_installs_below_its_own_directory() {
    let root = Path::new("/data/toolchain");
    assert_eq!(tool_install_dir(root, "node", "26.13.0"), PathBuf::from("/data/toolchain/node/26.13.0"));
    assert_eq!(tool_install_dir(root, "node", ""), PathBuf::from("/data/toolchain/node"));
}

#[test]
fn the_os_filter_admits_the_listed_families_only() {
    assert!(applies_to_family(&None, Some(OsFamily::Windows)));
    assert!(applies_to_family(&Some(vec![OsFamily::Linux]), Some(OsFamily::Linux)));
    assert!(!applies_to_family(&Some(vec![OsFamily::Linux]), Some(OsFamily::Macos)));
    assert!(!applies_to_family(&Some(vec![OsFamily::Linux]), None));
}

#[test]
fn the_repository_scope_puts_plan_lock_and_payloads_where_the_ide_and_ci_expect() {
    let env = EnvMap::from_pairs(&[("HOME", "/home/someone")]);
    let scope = repo_scope(Path::new("/work/tree"), &env).expect("a scope");
    assert_eq!(scope.kind, ScopeKind::Repo);
    assert_eq!(scope.plan_file, PathBuf::from("/work/tree/toolchain.plan.jsonc"));
    assert_eq!(scope.lock_file, PathBuf::from("/work/tree/toolchain.lock.json"));
    assert_eq!(scope.exposure_dir, PathBuf::from("/work/tree/.cache/toolchain-bin"));
    assert_eq!(scope.exposure_dir_relative, ".cache/toolchain-bin");
    assert_eq!(scope.link_dir, PathBuf::from("/work/tree/.cache/cli-git-bin"));
    assert_eq!(scope.toolchain_root, PathBuf::from("/home/someone/.local/share/monochromatic-dev/toolchain"));
    assert_eq!(scope.toolchain_bin_dir, PathBuf::from("/home/someone/.local/share/monochromatic-dev/toolchain/bin"));
}

#[test]
fn xdg_variables_move_the_data_and_config_directories() {
    let env = EnvMap::from_pairs(&[("HOME", "/h"), ("XDG_DATA_HOME", "/data"), ("XDG_CONFIG_HOME", "/config")]);
    let repo = repo_scope(Path::new("/w"), &env).expect("a scope");
    assert_eq!(repo.toolchain_root, PathBuf::from("/data/monochromatic-dev/toolchain"));
    let global = global_scope(&env).expect("a scope");
    assert_eq!(global.kind, ScopeKind::Global);
    assert_eq!(global.root, PathBuf::from("/config/monochromatic-dev/toolchain"));
    assert_eq!(global.plan_file, PathBuf::from("/config/monochromatic-dev/toolchain/toolchain.plan.jsonc"));
    assert_eq!(global.toolchain_root, PathBuf::from("/data/monochromatic-dev/toolchain-global"));
    assert_eq!(global.exposure_dir, PathBuf::from("/data/monochromatic-dev/toolchain-global/exposure"));
    assert_eq!(global.exposure_dir_relative, "exposure");
    assert_eq!(global.link_dir, PathBuf::from("/h/.local/bin"));
}

#[test]
fn an_empty_variable_counts_as_unset() {
    let env = EnvMap::from_pairs(&[("HOME", "/h"), ("XDG_DATA_HOME", "")]);
    let scope = repo_scope(Path::new("/w"), &env).expect("a scope");
    assert_eq!(scope.toolchain_root, PathBuf::from("/h/.local/share/monochromatic-dev/toolchain"));
}

#[test]
fn a_missing_home_is_a_configuration_error_naming_both_variables() {
    let error = home_dir(&EnvMap::default()).expect_err("no home");
    assert!(error.message.contains("HOME"), "{error}");
    assert!(error.message.contains("USERPROFILE"), "{error}");
}

#[test]
fn the_repository_root_is_found_from_a_subdirectory_or_the_override() {
    let directory = scratch();
    let root = directory.path().join("tree");
    let nested = root.join("package").join("deep");
    fs::create_dir_all(&nested).expect("make directories");
    fs::write(root.join("toolchain.plan.jsonc"), "{}").expect("write the plan");
    let env = EnvMap::from_pairs(&[("HOME", "/h")]);
    assert_eq!(find_repo_root(&nested, &env).expect("found"), root);
    let overridden = env.with("MONOCHROMATIC_WORKTREE_ROOT", "/elsewhere");
    assert_eq!(find_repo_root(&nested, &overridden).expect("overridden"), PathBuf::from("/elsewhere"));
    let lonely = directory.path().join("lonely");
    fs::create_dir_all(&lonely).expect("make the directory");
    let error = find_repo_root(&lonely, &env).expect_err("no plan anywhere above");
    assert!(error.message.contains("toolchain.plan.jsonc"), "{error}");
}

#[test]
fn resolve_scope_picks_global_or_repository() {
    let directory = scratch();
    let env = EnvMap::from_pairs(&[("HOME", "/h")]);
    assert_eq!(resolve_scope(true, directory.path(), &env).expect("global").kind, ScopeKind::Global);
    let rooted = env.with("MONOCHROMATIC_WORKTREE_ROOT", directory.path().to_str().expect("utf-8"));
    assert_eq!(resolve_scope(false, directory.path(), &rooted).expect("repo").kind, ScopeKind::Repo);
}

#[test]
fn manager_directories_follow_the_environment() {
    let env = EnvMap::from_pairs(&[
        ("HOME", "/h"),
        ("CARGO_HOME", "/cargo"),
        ("UV_TOOL_BIN_DIR", "/uv"),
        ("PNPM_HOME", "/pnpm"),
        ("npm_config_prefix", "/npm"),
    ]);
    let scope = repo_scope(Path::new("/w"), &env).expect("a scope");
    let dirs = manager_native_bin_dirs(&scope, &env);
    assert_eq!(dirs[0], scope.exposure_dir);
    assert_eq!(dirs[1], scope.toolchain_bin_dir);
    assert_eq!(dirs[2], PathBuf::from("/cargo/bin"));
    assert_eq!(dirs[3], PathBuf::from("/uv"));
    assert_eq!(dirs[4], PathBuf::from("/pnpm/bin"));
    assert_eq!(dirs[5], PathBuf::from("/npm/bin"));
}

#[test]
fn each_route_searches_its_own_directories() {
    let env = EnvMap::from_pairs(&[("HOME", "/h")]);
    let scope = repo_scope(Path::new("/w"), &env).expect("a scope");
    let root = scope.toolchain_root.clone();
    assert_eq!(bin_search_dirs(&sample_tool("cargo:fd-find"), &scope, &env, &[]), vec![PathBuf::from("/h/.cargo/bin")]);
    assert_eq!(bin_search_dirs(&sample_tool("node"), &scope, &env, &[]), vec![root.join("node").join("1.2.3")]);
    let extra = vec![PathBuf::from("/extra")];
    assert_eq!(
        bin_search_dirs(&sample_tool("pnpm"), &scope, &env, &extra),
        vec![scope.toolchain_bin_dir.clone(), PathBuf::from("/h/.local/bin"), PathBuf::from("/extra")]
    );
    assert_eq!(
        bin_search_dirs(&sample_tool("npm:wrangler"), &scope, &env, &extra),
        vec![PathBuf::from("/extra"), PathBuf::from("/h/.local/share/pnpm"), root.join("node")]
    );
    assert_eq!(bin_search_dirs(&sample_tool("pipx:slopo"), &scope, &env, &extra), vec![PathBuf::from("/h/.local/bin"), PathBuf::from("/extra")]);
    assert_eq!(
        bin_search_dirs(&sample_tool("rust"), &scope, &env, &[]),
        vec![PathBuf::from("/h/.cargo/bin"), PathBuf::from("/h/.rustup/toolchains")]
    );
    let android = bin_search_dirs(&sample_tool("android-sdk"), &scope, &env, &[]);
    assert_eq!(android[0], root.join("android-sdk").join("23.0").join("cmdline-tools").join("23.0").join("bin"));
    assert_eq!(android[1], root.join("android-sdk").join("23.0").join("platform-tools"));
}

#[test]
fn a_non_executable_file_or_a_directory_is_not_a_binary() {
    let directory = scratch();
    let plain = directory.path().join("plain");
    fs::write(&plain, "data").expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&plain, fs::Permissions::from_mode(0o644)).expect("chmod");
        assert!(!is_executable(&plain));
    }
    assert!(!is_executable(directory.path()));
    assert!(!is_executable(&directory.path().join("missing")));
    let runnable = directory.path().join("runnable");
    write_executable(&runnable);
    assert!(is_executable(&runnable));
}

#[test]
fn the_search_finds_a_binary_nested_in_an_archive_layout_and_prefers_the_shallower_one() {
    let directory = scratch();
    write_executable(&directory.path().join("node-v1").join("bin").join("node"));
    assert_eq!(find_bin_under(directory.path(), "node"), Some(directory.path().join("node-v1").join("bin").join("node")));
    write_executable(&directory.path().join("a").join("node"));
    assert_eq!(find_bin_under(directory.path(), "node"), Some(directory.path().join("a").join("node")));
    assert_eq!(find_bin_under(directory.path(), "absent"), None);
}

#[test]
fn the_search_stops_below_the_depth_limit() {
    let directory = scratch();
    write_executable(&directory.path().join("a").join("b").join("c").join("d").join("tool"));
    assert_eq!(find_bin_under(directory.path(), "tool"), None);
    write_executable(&directory.path().join("a").join("b").join("c").join("tool"));
    assert!(find_bin_under(directory.path(), "tool").is_some());
}

#[test]
fn find_tool_bin_looks_in_the_route_directories_of_the_scope() {
    let directory = scratch();
    let home = directory.path().join("home");
    let env = EnvMap::from_pairs(&[("HOME", home.to_str().expect("utf-8"))]);
    let scope = repo_scope(&directory.path().join("work"), &env).expect("a scope");
    assert_eq!(find_tool_bin(&sample_tool("cargo:fd-find"), "fd", &scope, &env, &[]), None);
    write_executable(&home.join(".cargo").join("bin").join("fd"));
    assert_eq!(find_tool_bin(&sample_tool("cargo:fd-find"), "fd", &scope, &env, &[]), Some(home.join(".cargo").join("bin").join("fd")));
}
