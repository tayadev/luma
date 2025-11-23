use std::process::Command;

fn main() {
    // Get the git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Set environment variable for use in main.rs
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Re-run if git HEAD changes (this is a file that points to the current ref)
    println!("cargo:rerun-if-changed=.git/HEAD");

    // Re-run if the git index changes (when commits are made)
    println!("cargo:rerun-if-changed=.git/index");
}
