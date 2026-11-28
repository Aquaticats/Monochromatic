//! What:
//!  The controls of the route's own layout:
//!  a file that names the checkout's path and the two datasets,
//!  a file another account could write,
//!  and a text that is not one JSONC document.
//! Why:
//!  Every other machine fact is named through these three,
//!  and a file the route refuses must stop the gather with the file's own
//!  sentence rather than a report about names nobody read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRoutesLayout(places)).toMatchObject({ checkout, checkoutDataset, parentDataset });
//! expect(theRoutesLayout(places)).toThrow('the route\'s own layout could not be named');
//! ```

/// The places a fixture names.
use super::super::the_route_places::TheRoutePlaces;
/// The read a control judges.
use super::the_routes_layout;
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The three names a layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
/// The owner and mode bits a fixture's file is given.
use std::os::unix::fs::PermissionsExt;
/// The owned and borrowed paths these controls use.
use std::path::{
    Path,
    PathBuf,
};

/// The checkout dataset a control's file names.
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The container a control's file names.
const CONTAINER_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// What:
///  A file naming all three answers the three names it holds,
///  and the checkout's path is the one the file spells.
/// Why:
///  The mount table,
///  the walks and the delegation are all named through this answer,
///  so a control must see the file's own text and not a default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theRoutesLayout(places)).toMatchObject({ checkout, checkoutDataset: CHECKOUT_DATASET, parentDataset: CONTAINER_DATASET });
/// ```
#[test]
fn a_file_naming_three_answers_the_three_it_holds() {
    let root: PathBuf = fixture("layout-names-three");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let checkout: PathBuf = root.join("checkout");
    write_configuration(places.configuration.as_path(), checkout.as_path());
    let layout: SweepConfiguration = the_routes_layout(&places).expect("a file naming all three");
    assert_eq!(layout.checkout, checkout);
    assert_eq!(layout.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(layout.parent_dataset, CONTAINER_DATASET);
}

/// What:
///  A file another account could write stops the gather,
///  with the file's path in the sentence.
/// Why:
///  A configuration another account can rewrite names the datasets root acts
///  on,
///  so the route refuses it and the report names the file to repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theRoutesLayout(places)).toThrow(places.configuration);
/// ```
#[test]
fn a_file_another_account_could_write_is_refused_with_its_path() {
    let root: PathBuf = fixture("layout-writable");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    let checkout: PathBuf = root.join("checkout");
    write_configuration(places.configuration.as_path(), checkout.as_path());
    std::fs::set_permissions(
        places.configuration.as_path(),
        std::fs::Permissions::from_mode(0o666),
    )
    .expect("the file's mode");
    let why: String = the_routes_layout(&places).expect_err("a writable file is refused");
    assert!(
        why.contains(places.configuration.display().to_string().as_str()),
        "{why}"
    );
    assert!(why.contains("could not be named"), "{why}");
}

/// What:
///  A text that is not one JSONC document stops the gather,
///  with the file's path in the sentence.
/// Why:
///  A report about a layout read from a file nobody could parse would name
///  datasets the file never spelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theRoutesLayout(places)).toThrow('not one JSONC document');
/// ```
#[test]
fn a_text_that_is_not_jsonc_is_refused_with_its_path() {
    let root: PathBuf = fixture("layout-not-jsonc");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    write_configuration_text(places.configuration.as_path(), "{ this is not a document");
    let why: String = the_routes_layout(&places).expect_err("a broken document is refused");
    assert!(
        why.contains(places.configuration.display().to_string().as_str()),
        "{why}"
    );
}

/// One configuration naming the three facts of one fixture.
fn write_configuration(
    path: &Path,
    checkout: &Path,
) {
    let document: String = format!(
        "{{\n  \"cloneMounts\": {{\n    \"checkout\": \"{}\",\n    \"parent\": \"{CONTAINER_DATASET}\",\n    \"checkoutDataset\": \"{CHECKOUT_DATASET}\"\n  }}\n}}\n",
        checkout.display()
    );
    write_configuration_text(path, document.as_str());
}

/// One configuration file with the text the control chose.
fn write_configuration_text(
    path: &Path,
    text: &str,
) {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).expect("the configuration's folder");
    }
    std::fs::write(path, text.as_bytes()).expect("the configuration");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))
        .expect("the configuration's mode");
}
