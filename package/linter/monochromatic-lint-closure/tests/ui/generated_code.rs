// edition:2024
// Policy: a closure is reported only in the package's own source. A file that a build script wrote below `OUT_DIR`
// and a file outside the package directory are skipped, because nobody can rewrite them by hand. The same closure
// written in a file of the package is the positive control: it is reported.

#[path = "../outside/generated.rs"]
mod outside;

#[path = "auxiliary/generated_in_out_dir.rs"]
mod generated_in_out_dir;

fn own_source_closure_is_reported() -> bool {
    // verdict: reported
    return "a1".bytes().all(|byte| byte.is_ascii_digit());
}

fn main() {
    let _ = outside::outside_capture_free();
    let _ = outside::outside_borrowing(3);
    let _ = generated_in_out_dir::generated_capture_free();
    let _ = own_source_closure_is_reported();
}
