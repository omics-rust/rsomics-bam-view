use std::path::Path;
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-view"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn samtools_count(args: &[&str]) -> u64 {
    let out = Command::new("samtools")
        .arg("view")
        .args(args)
        .arg("-c")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).trim().parse().unwrap()
}

fn ours_count(args: &[&str]) -> u64 {
    let out = bin().args(args).arg("-c").arg(fixture()).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().parse().unwrap()
}

#[test]
fn count_matches_samtools() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    assert_eq!(ours_count(&[]), samtools_count(&[]));
}

#[test]
fn require_flags_matches_samtools() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    for f in ["1", "2", "4", "16", "64"] {
        assert_eq!(
            ours_count(&["-f", f]),
            samtools_count(&["-f", f]),
            "require-flags {f}"
        );
    }
}

#[test]
fn exclude_flags_matches_samtools() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    for f in ["4", "256", "1024"] {
        assert_eq!(
            ours_count(&["-F", f]),
            samtools_count(&["-F", f]),
            "exclude-flags {f}"
        );
    }
}

#[test]
fn min_mapq_matches_samtools() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    for q in ["1", "20", "60"] {
        assert_eq!(
            ours_count(&["--min-mapq", q]),
            samtools_count(&["-q", q]),
            "min-mapq {q}"
        );
    }
}
