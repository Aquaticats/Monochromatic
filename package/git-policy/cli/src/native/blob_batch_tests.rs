//! Controls for batch parsing and loading against real Git.

use super::*;
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// Owned IDs from literals.
fn ids(items: &[&str]) -> Vec<String> {
    let mut owned: Vec<String> = Vec::new();
    for item in items {
        owned.push(String::from(*item));
    }
    return owned;
}

/// Headers need three fields,
///  a blob and a canonical size.
#[test]
fn headers_are_strict() {
    assert_eq!(
        parse_batch_header("abc blob 3"),
        Ok((String::from("abc"), 3))
    );
    assert_eq!(
        parse_batch_header("abc blob 0"),
        Ok((String::from("abc"), 0))
    );
    assert_eq!(
        parse_batch_header("abc missing"),
        Err(String::from(
            "Git blob batch returned malformed header: abc missing"
        ))
    );
    assert_eq!(
        parse_batch_header("abc blob 3 x"),
        Err(String::from(
            "Git blob batch returned malformed header: abc blob 3 x"
        ))
    );
    assert_eq!(
        parse_batch_header("abc tree 3"),
        Err(String::from("Git blob batch returned tree for abc."))
    );
    for size in ["03", "", "+3", "-3", "3.0", "9007199254740992", "x"] {
        assert_eq!(
            parse_batch_header(format!("abc blob {size}").as_str()),
            Err(format!(
                "Git blob batch returned invalid size for abc: {size}"
            )),
            "{size}"
        );
    }
    assert_eq!(
        parse_batch_header("abc blob 9007199254740991").map(second),
        Ok(9_007_199_254_740_991)
    );
}

/// The size of a parsed header.
fn second(header: (String, usize)) -> usize {
    return header.1;
}

/// Output is read by size,
///  never by delimiters inside content.
#[test]
fn output_is_read_by_size() {
    let output: &[u8] = b"a blob 3\nx\ny\nb blob 0\n\n";
    let parsed: HashMap<String, Vec<u8>> =
        parse_batch_output(output, ids(&["a", "b"]).as_slice()).expect("parsed");
    assert_eq!(parsed.get("a").map(Vec::as_slice), Some(&b"x\ny"[..]));
    assert_eq!(parsed.get("b").map(Vec::as_slice), Some(&b""[..]));
    assert_eq!(
        parse_batch_output(b"", ids(&["a"]).as_slice()),
        Err(String::from("Git blob batch omitted header for a."))
    );
    assert_eq!(
        parse_batch_output(b"b blob 1\nx\n", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned b while a was requested."
        ))
    );
    assert_eq!(
        parse_batch_output(b"a blob 5\nx\n", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned truncated content for a."
        ))
    );
    assert_eq!(
        parse_batch_output(b"a blob 1\nxy", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned truncated content for a."
        ))
    );
    assert_eq!(
        parse_batch_output(b"a blob 1\nx", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned truncated content for a."
        ))
    );
    assert_eq!(
        parse_batch_output(b"a blob 1\nx\nextra", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned unexpected trailing bytes."
        ))
    );
    assert_eq!(
        parse_batch_output(b"\xff blob 1\nx\n", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned a header that is not UTF-8 for a."
        ))
    );
    assert_eq!(
        parse_batch_output(b"a blob 18446744073709551615\n", ids(&["a"]).as_slice()),
        Err(String::from(
            "Git blob batch returned invalid size for a: 18446744073709551615"
        ))
    );
    assert_eq!(parse_batch_output(b"", &[]), Ok(HashMap::new()));
}

/// Real Git supplies every distinct blob,
///  including ones only a private store holds.
#[test]
fn real_git_supplies_blobs() {
    let root: PathBuf = fixture("blob-batch-real");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"alpha\n").expect("file");
    let alpha: String = git_text(repo.as_path(), &["hash-object", "-w", "a.txt"]);
    let context: crate::transaction_git::GitContext = git_context();
    let loaded: HashMap<String, Vec<u8>> = load_blob_batch(
        &context,
        repo.as_path(),
        &[alpha.clone(), alpha.clone()],
        None,
    )
    .expect("loaded");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded.get(&alpha).map(Vec::as_slice), Some(&b"alpha\n"[..]));
    assert_eq!(
        load_blob_batch(&context, repo.as_path(), &[], None),
        Ok(HashMap::new())
    );
    // A private store with alternates reaches both its own and the real objects.
    let private: PathBuf = root.join("objects");
    std::fs::create_dir_all(private.join("info")).expect("private store");
    std::fs::write(
        private.join("info").join("alternates"),
        format!("{}\n", repo.join(".git").join("objects").display()),
    )
    .expect("alternates");
    let mut write: crate::transaction_git::GitRequest =
        crate::transaction_git::GitRequest::new(repo.as_path(), &["hash-object", "-w", "--stdin"]);
    write.object_directory = Some(private.clone());
    write.input = Some(b"private\n".to_vec());
    let private_oid: String = String::from_utf8(
        crate::transaction_git::run_git_checked(&context, &write)
            .expect("private object")
            .stdout,
    )
    .expect("UTF-8")
    .trim()
    .to_string();
    let both: HashMap<String, Vec<u8>> = load_blob_batch(
        &context,
        repo.as_path(),
        &[private_oid.clone(), alpha.clone()],
        Some(private.as_path()),
    )
    .expect("both");
    assert_eq!(
        both.get(&private_oid).map(Vec::as_slice),
        Some(&b"private\n"[..])
    );
    assert!(
        load_blob_batch(
            &context,
            repo.as_path(),
            std::slice::from_ref(&private_oid),
            None
        )
        .is_err()
    );
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    assert_eq!(
        load_blob_batch(&context, repo.as_path(), std::slice::from_ref(&tree), None),
        Err(format!("Git blob batch returned tree for {tree}."))
    );
    let unstartable: crate::transaction_git::GitContext = crate::transaction_git::GitContext {
        real_git: root.join("no-git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    assert!(
        load_blob_batch(
            &unstartable,
            repo.as_path(),
            std::slice::from_ref(&alpha),
            None
        )
        .expect_err("no Git")
        .starts_with("git cat-file --batch could not start: ")
    );
    assert!(load_blob_batch(&context, root.join("missing").as_path(), &[alpha], None).is_err());
    remove(root.as_path());
}
