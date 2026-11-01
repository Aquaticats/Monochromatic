//! What:
//!  Fresh in-memory workspace for fuzzing the real semantic session.
//! Why:
//!  No arbitrary fuzz bytes enter rust-analyzer's fixture directive syntax or cause Cargo/subprocess execution.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Initialize fixed project metadata once per draw, then submit arbitrary source only as a text overlay.
//! ```

/// Import the production session boundary.
use monochromatic_lint::rust_semantic_session::RustSemanticSession;
/// Import the database and its fixed fixture constructor.
use ra_ap_hir::EditionedFileId;
use ra_ap_ide_db::RootDatabase;
use ra_ap_test_fixture::WithFixture;
/// Import owned file identities and virtual paths.
use ra_ap_vfs::{FileExcluded, FileId, Vfs, VfsPath};

/// Build one fixed,
///  dependency-free crate;
///  the source supplied by the fuzzer is never fixture metadata.
pub fn semantic_session() -> RustSemanticSession {
    let (database, editioned): (RootDatabase, EditionedFileId) =
        RootDatabase::with_single_file("//- /main.rs\nfn main() {}\n");
    let file: FileId = editioned.file_id(&database);
    let path: VfsPath = VfsPath::new_real_path(String::from("/main.rs"));
    let mut files: Vfs = Vfs::default();
    files.set_file_contents(
        path.clone(),
        Some(Vec::<u8>::from("fn main() {}\n".as_bytes())),
    );
    let (actual, excluded): (FileId, FileExcluded) =
        files.file_id(&path).expect("fixed file exists");
    // The identities come from separate typed builders; validate their association rather than assuming it.
    assert_eq!(actual, file);
    assert_eq!(excluded, FileExcluded::No);
    return RustSemanticSession::from_workspace(database, files, None);
}
