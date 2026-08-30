use std::process::Command;

fn command_output(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn main() {
    // Falls back to "unknown" when building outside a git checkout (e.g. a
    // Docker build context with no `.git` directory).
    let git_sha =
        command_output("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let rustc_version =
        command_output("rustc", &["--version"]).unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_SHA={git_sha}");
    println!("cargo:rustc-env=BUILT_AT={}", chrono::Utc::now().to_rfc3339());
    println!("cargo:rustc-env=RUSTC_VERSION={rustc_version}");

    // Re-run when HEAD moves, but don't fail the build without a `.git` dir.
    println!("cargo:rerun-if-changed=.git/HEAD");
}
