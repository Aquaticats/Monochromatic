//! Registry assembly on disposable project roots:
//!  overrides,
//!  availability,
//!  and the launch seam.

use super::{LanguageSetup, Languages, Unavailable, typescript};
use crate::language::launch::{LaunchRequest, ServerLaunch, launch_directly};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

/// A scripted language whose server program is `/bin/sh`,
///  which exists wherever tests run.
const SCRIPTED: &str = r#"
[language-server.scripted-ls]
command = "sh"
args = ["-c", "exit 0"]
environment = { KEY = "value" }

[language-server.absent-ls]
command = "definitely-not-installed-language-server"

[[language]]
name = "scripted"
scope = "source.scripted"
file-types = ["scripted"]
roots = []
language-servers = ["scripted-ls", "absent-ls"]
"#;

fn project() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("project");
    let root = directory.path().canonicalize().expect("canonical project");
    return (directory, root);
}

fn install_typescript(root: &Path, version: &str) {
    let package = root.join("node_modules/typescript");
    fs::create_dir_all(package.join("bin")).expect("package directory");
    fs::write(
        package.join("package.json"),
        format!(r#"{{ "name": "typescript", "version": "{version}" }}"#),
    )
    .expect("manifest");
    fs::write(package.join("bin/tsc"), "#!/usr/bin/env node\n").expect("launcher");
    fs::set_permissions(package.join("bin/tsc"), fs::Permissions::from_mode(0o755))
        .expect("launcher mode");
}

fn scripted_setup() -> LanguageSetup {
    return LanguageSetup {
        extra_languages: Some(SCRIPTED.to_string()),
        ..LanguageSetup::unconfined()
    };
}

/// Server names Helix would start for a file,
///  read from the registry Helix itself uses.
fn startable(languages: &Languages, file: &str) -> Vec<String> {
    let loader = languages.loader.load();
    let language = loader
        .language_for_filename(Path::new(file))
        .expect("known language");
    let mut names = Vec::new();
    for features in &loader.language(language).config().language_servers {
        names.push(features.name.clone());
    }
    return names;
}

#[test]
fn typescript_family_is_served_by_the_projects_own_server() {
    let (_directory, root) = project();
    install_typescript(&root, "7.0.2");
    let languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    for language in ["typescript", "tsx", "javascript", "jsx"] {
        assert_eq!(
            languages.configured(language),
            [typescript::SERVER.to_string()],
            "{language} is not served by the TypeScript 7 server"
        );
    }
    assert_eq!(languages.unavailable(typescript::SERVER), None);
    let launch = languages.launch(typescript::SERVER).expect("launch");
    assert_eq!(
        launch.command,
        root.join("node_modules/typescript/bin/tsc")
            .to_str()
            .expect("Unicode path")
    );
    assert_eq!(launch.args, ["--lsp", "--stdio"]);
    let settings = launch.settings.as_ref().expect("settings");
    assert_eq!(
        settings["typescript"]["inlayHints"]["parameterNames"]["enabled"],
        "all"
    );
    assert_eq!(
        settings["typescript"]["inlayHints"]["variableTypes"]["enabled"],
        true
    );
    assert_eq!(startable(&languages, "main.ts"), [typescript::SERVER]);
}

#[test]
fn project_without_typescript_shows_the_missing_executable_reason() {
    let (_directory, root) = project();
    let languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    let Some(Unavailable::Missing(reason)) = languages.unavailable(typescript::SERVER) else {
        panic!("the TypeScript server was not reported missing");
    };
    assert!(
        reason.contains("node_modules/typescript/bin/tsc"),
        "{reason}"
    );
    assert!(
        reason.contains("TypeScript 7 or later is required"),
        "{reason}"
    );
    assert!(
        startable(&languages, "main.ts").is_empty(),
        "Helix could still start a server whose program is missing"
    );
    assert_eq!(languages.configured("typescript"), [typescript::SERVER]);
}

#[test]
fn project_typescript_before_version_seven_is_missing_too() {
    let (_directory, root) = project();
    install_typescript(&root, "5.9.3");
    let languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    let Some(Unavailable::Missing(reason)) = languages.unavailable(typescript::SERVER) else {
        panic!("TypeScript 5 was accepted as a language server");
    };
    assert_eq!(
        reason,
        &format!(
            "the project's TypeScript 5.9.3 in {} has no built-in language server; version 7 or later is required",
            root.display()
        )
    );
}

/// A document in a package below the root,
///  with the folder it lives in.
fn nested_document(root: &Path) -> PathBuf {
    let folder = root.join("package/app/src");
    fs::create_dir_all(&folder).expect("package source folder");
    return folder.join("main.ts");
}

#[test]
fn typescript_is_found_from_the_documents_folder_upward_inside_the_project() {
    let (_directory, root) = project();
    install_typescript(&root, "7.0.2");
    let document = nested_document(&root);
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    assert!(
        !languages
            .refresh("typescript", Some(&document))
            .expect("refresh"),
        "the root's install is the one both lookups find"
    );
    let launch = languages.launch(typescript::SERVER).expect("launch");
    assert_eq!(
        launch.command,
        root.join("node_modules/typescript/bin/tsc")
            .to_str()
            .expect("Unicode path")
    );
}

#[test]
fn typescript_is_found_for_a_project_folder_that_holds_no_install_of_its_own() {
    // The project folder is above the repository, like a home folder: the install is two folders down.
    let (_directory, root) = project();
    let repository = root.join("repository");
    install_typescript(&repository, "7.0.2");
    let document = repository.join("package/app/src/main.ts");
    fs::create_dir_all(document.parent().expect("folder")).expect("source folder");
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    assert!(matches!(
        languages.unavailable(typescript::SERVER),
        Some(Unavailable::Missing(_))
    ));
    assert!(
        languages
            .refresh("typescript", Some(&document))
            .expect("refresh"),
        "the install below the project folder was not found from the document's folder"
    );
    assert_eq!(languages.unavailable(typescript::SERVER), None);
    let launch = languages.launch(typescript::SERVER).expect("launch");
    assert_eq!(
        launch.command,
        repository
            .join("node_modules/typescript/bin/tsc")
            .to_str()
            .expect("Unicode path")
    );
}

#[test]
fn the_nearest_typescript_install_decides_and_an_older_one_is_not_skipped() {
    let (_directory, root) = project();
    install_typescript(&root, "7.0.2");
    let package = root.join("package/app");
    install_typescript(&package, "5.9.3");
    let document = nested_document(&root);
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    assert!(
        languages
            .refresh("typescript", Some(&document))
            .expect("refresh"),
        "the package's TypeScript 5 was skipped for the root's TypeScript 7"
    );
    let Some(Unavailable::Missing(reason)) = languages.unavailable(typescript::SERVER) else {
        panic!("the package's TypeScript 5 was skipped for the root's TypeScript 7");
    };
    assert!(
        reason.contains("TypeScript 5.9.3") && reason.contains("package/app"),
        "{reason}"
    );
    // Back at the root the root's own install serves again.
    let at_root = root.join("main.ts");
    assert!(
        languages
            .refresh("typescript", Some(&at_root))
            .expect("refresh")
    );
    assert_eq!(languages.unavailable(typescript::SERVER), None);
}

#[test]
fn the_registry_follows_the_launcher_to_another_install_of_version_seven() {
    // Both installs are usable, so availability never changes; only the launcher's location does.
    let (_directory, root) = project();
    install_typescript(&root, "7.0.2");
    install_typescript(&root.join("package/app"), "7.1.0");
    let document = nested_document(&root);
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    assert!(
        languages
            .refresh("typescript", Some(&document))
            .expect("refresh"),
        "the registry kept the launcher of the other install"
    );
    let launch = languages.launch(typescript::SERVER).expect("launch");
    assert_eq!(
        launch.command,
        root.join("package/app/node_modules/typescript/bin/tsc")
            .to_str()
            .expect("Unicode path")
    );
}

#[test]
fn a_typescript_install_above_the_project_is_named_but_never_used() {
    let (_directory, outer) = project();
    install_typescript(&outer, "7.0.2");
    let root = outer.join("package/app");
    fs::create_dir_all(&root).expect("project folder");
    let document = root.join("src/main.ts");
    fs::create_dir_all(document.parent().expect("folder")).expect("source folder");
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    languages
        .refresh("typescript", Some(&document))
        .expect("refresh");
    let Some(Unavailable::Missing(reason)) = languages.unavailable(typescript::SERVER) else {
        panic!("an install outside the project was used");
    };
    assert!(
        reason.contains(
            &outer
                .join("node_modules/typescript/bin/tsc")
                .display()
                .to_string()
        ),
        "{reason}"
    );
    assert!(
        reason.contains("open the folder that holds it as the project"),
        "{reason}"
    );
    assert!(
        reason.contains("TypeScript 7 or later is required"),
        "{reason}"
    );
    assert!(languages.launch(typescript::SERVER).is_none());
}

#[test]
fn a_document_outside_the_project_is_looked_up_at_the_project_root_only() {
    let (_directory, root) = project();
    let (_other, elsewhere) = project();
    install_typescript(&elsewhere, "7.0.2");
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    languages
        .refresh("typescript", Some(&elsewhere.join("src/main.ts")))
        .expect("refresh");
    assert!(
        matches!(
            languages.unavailable(typescript::SERVER),
            Some(Unavailable::Missing(_))
        ),
        "an install beside a document outside the project was used"
    );
}

#[test]
fn registry_is_rebuilt_when_a_missing_program_appears() {
    let (_directory, root) = project();
    let mut languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    assert!(!languages.refresh("typescript", None).expect("refresh"));
    install_typescript(&root, "7.0.2");
    assert!(
        languages.refresh("typescript", None).expect("refresh"),
        "an installed server stayed missing"
    );
    assert_eq!(languages.unavailable(typescript::SERVER), None);
    assert_eq!(startable(&languages, "main.ts"), [typescript::SERVER]);
    fs::remove_dir_all(root.join("node_modules")).expect("remove dependencies");
    assert!(languages.refresh("typescript", None).expect("refresh"));
    assert!(languages.unavailable(typescript::SERVER).is_some());
}

#[test]
fn missing_program_is_reported_and_never_handed_to_helix() {
    let (_directory, root) = project();
    let languages = Languages::new(&root, scripted_setup()).expect("registry");
    assert_eq!(
        languages.configured("scripted"),
        ["scripted-ls".to_string(), "absent-ls".to_string()]
    );
    assert_eq!(
        languages.unavailable("absent-ls"),
        Some(&Unavailable::Missing(
            "the program 'definitely-not-installed-language-server' for absent-ls was not found on PATH"
                .to_string()
        ))
    );
    assert_eq!(startable(&languages, "file.scripted"), ["scripted-ls"]);
    let launch = languages.launch("scripted-ls").expect("launch");
    assert!(
        Path::new(&launch.command).is_absolute(),
        "the server program was not resolved to an absolute path: {}",
        launch.command
    );
    assert!(launch.command.ends_with("/sh"));
    assert_eq!(
        launch.environment.get("KEY").map(String::as_str),
        Some("value")
    );
}

/// A policy that wraps the server,
///  as a confining launcher would.
fn wrapping_policy(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    let mut launch = launch_directly(request)?;
    let mut args = vec![
        "--state".to_string(),
        request
            .state_root
            .as_ref()
            .expect("state root")
            .display()
            .to_string(),
        "--project".to_string(),
        request.project_root.display().to_string(),
        "--".to_string(),
        launch.command.clone(),
    ];
    args.append(&mut launch.args);
    launch.command = "/usr/bin/env".to_string();
    launch.args = args;
    launch
        .environment
        .insert("WRAPPED".to_string(), request.server.clone());
    launch.settings = Some(serde_json::json!({ "overridden": true }));
    return Ok(launch);
}

#[test]
fn launch_policy_output_is_what_helix_is_given() {
    let (_directory, root) = project();
    let setup = LanguageSetup {
        launch: wrapping_policy,
        state_root: Some(PathBuf::from("/var/tmp/ide-state")),
        extra_languages: Some(SCRIPTED.to_string()),
        ..LanguageSetup::unconfined()
    };
    let languages = Languages::new(&root, setup).expect("registry");
    let loader = languages.loader.load();
    let definition = loader
        .language_server_configs()
        .get("scripted-ls")
        .expect("definition");
    assert_eq!(definition.command, "/usr/bin/env");
    assert_eq!(definition.args[0], "--state");
    assert_eq!(definition.args[1], "/var/tmp/ide-state");
    assert_eq!(definition.args[3], root.display().to_string());
    assert_eq!(definition.args[4], "--");
    assert!(
        Path::new(&definition.args[5]).is_absolute() && definition.args[5].ends_with("/sh"),
        "the program after the separator is not absolute: {}",
        definition.args[5]
    );
    assert_eq!(&definition.args[6..], ["-c", "exit 0"]);
    assert_eq!(
        definition.environment.get("WRAPPED").map(String::as_str),
        Some("scripted-ls")
    );
    assert_eq!(
        definition.config,
        Some(serde_json::json!({ "overridden": true }))
    );
    assert!(
        matches!(
            languages.unavailable("absent-ls"),
            Some(Unavailable::Missing(_))
        ),
        "a wrapper hid a missing server program"
    );
}

/// A policy that cannot confine anything.
fn refusing_policy(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    return Err(format!("confinement is unavailable for {}", request.server));
}

#[test]
fn refused_launch_fails_closed() {
    let (_directory, root) = project();
    let setup = LanguageSetup {
        launch: refusing_policy,
        state_root: None,
        extra_languages: Some(SCRIPTED.to_string()),
        ..LanguageSetup::unconfined()
    };
    let languages = Languages::new(&root, setup).expect("registry");
    assert_eq!(
        languages.unavailable("scripted-ls"),
        Some(&Unavailable::Refused(
            "confinement is unavailable for scripted-ls".to_string()
        ))
    );
    assert!(languages.launch("scripted-ls").is_none());
    assert!(
        startable(&languages, "file.scripted").is_empty(),
        "a refused server could still be started without its wrapper"
    );
}

#[test]
fn workspace_helix_configuration_is_never_loaded() {
    let (_directory, root) = project();
    fs::create_dir(root.join(".helix")).expect("workspace configuration directory");
    fs::write(
        root.join(".helix/languages.toml"),
        "[language-server.rust-analyzer]\ncommand = \"/project/supplied/program\"\n",
    )
    .expect("workspace configuration");
    let languages = Languages::new(&root, LanguageSetup::unconfined()).expect("registry");
    let loader = languages.loader.load();
    let definition = loader
        .language_server_configs()
        .get("rust-analyzer")
        .expect("definition");
    assert_ne!(
        definition.command, "/project/supplied/program",
        "a project-supplied server command was loaded"
    );
}

#[test]
fn malformed_extra_definitions_stop_the_module() {
    let (_directory, root) = project();
    let setup = LanguageSetup {
        extra_languages: Some("[[language".to_string()),
        ..LanguageSetup::unconfined()
    };
    let Err(error) = Languages::new(&root, setup) else {
        panic!("malformed definitions were accepted");
    };
    assert!(
        error
            .to_string()
            .contains("Cannot parse the extra language definitions"),
        "{error}"
    );
}
