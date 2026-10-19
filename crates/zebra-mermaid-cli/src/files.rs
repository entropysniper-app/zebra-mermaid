//! Output files and batch planning.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::Failure;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Writes `bytes` to a temporary file next to `path` and moves it into place. Without
/// `force`, an existing file is left alone and reported; the check and the move are one step
/// (a hard link), so a file that appears meanwhile is not replaced either.
pub fn write_atomic(path: &Path, bytes: &[u8], force: bool) -> Result<(), Failure> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let name = path.file_name().ok_or_else(|| Failure::io("io", format!("{} is not a file path", path.display())))?;
    if !force && path.exists() {
        return Err(Failure::io("exists", format!("{} exists; use --force to replace it", path.display())));
    }
    let temp = dir.join(format!(".{}.{}.{}.tmp", name.to_string_lossy(), std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
    let written = (|| -> std::io::Result<()> {
        let mut f = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        f.write_all(bytes)?;
        f.sync_all()
    })();
    if let Err(e) = written {
        let _ = fs::remove_file(&temp);
        return Err(Failure::io("io", format!("cannot write {}: {e}", path.display())));
    }
    let moved = if force {
        fs::rename(&temp, path)
    } else {
        match fs::hard_link(&temp, path) {
            Ok(()) => fs::remove_file(&temp),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temp);
                return Err(Failure::io("exists", format!("{} exists; use --force to replace it", path.display())));
            }
            // A file system without hard links: the existence check above has to do.
            Err(_) => fs::rename(&temp, path),
        }
    };
    moved.map_err(|e| {
        let _ = fs::remove_file(&temp);
        Failure::io("io", format!("cannot write {}: {e}", path.display()))
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub input: PathBuf,
    pub output: PathBuf,
}

fn is_diagram(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("mmd") || e.eq_ignore_ascii_case("mermaid"))
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        // Symbolic links to directories are not followed, so a link cannot make a cycle.
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            walk(&path, out)?;
        } else if is_diagram(&path) && fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false) {
            out.push(path);
        }
    }
    Ok(())
}

/// The absolute form of a path that may not exist yet.
fn absolute(path: &Path) -> PathBuf {
    if let Ok(p) = fs::canonicalize(path) {
        return p;
    }
    let base = std::env::current_dir().unwrap_or_default();
    let joined = if path.is_absolute() { path.to_path_buf() } else { base.join(path) };
    match (joined.parent(), joined.file_name()) {
        (Some(parent), Some(name)) => absolute(parent).join(name),
        _ => joined,
    }
}

/// Lists the inputs and where each one's SVG goes, refusing overlapping input and output
/// directories and two inputs that would write the same file.
pub fn plan_batch(inputs: &[PathBuf], out_dir: &Path) -> Result<Vec<Job>, Failure> {
    let out_abs = absolute(out_dir);
    let mut jobs = Vec::new();
    for input in inputs {
        let meta = fs::metadata(input).map_err(|e| Failure::io("io", format!("cannot read {}: {e}", input.display())))?;
        if meta.is_dir() {
            let in_abs = absolute(input);
            if out_abs == in_abs || out_abs.starts_with(&in_abs) || in_abs.starts_with(&out_abs) {
                return Err(Failure::usage(format!(
                    "--out-dir {} overlaps the input directory {}; choose a directory outside it",
                    out_dir.display(),
                    input.display()
                )));
            }
            let mut found = Vec::new();
            walk(input, &mut found).map_err(|e| Failure::io("io", format!("cannot list {}: {e}", input.display())))?;
            for file in found {
                let rel = file.strip_prefix(input).unwrap_or(&file).to_path_buf();
                jobs.push(Job { output: out_dir.join(rel).with_extension("svg"), input: file });
            }
        } else {
            let name = input.file_name().ok_or_else(|| Failure::usage(format!("{} is not a file", input.display())))?;
            jobs.push(Job { output: out_dir.join(name).with_extension("svg"), input: input.clone() });
        }
    }
    let mut seen: HashMap<PathBuf, &Path> = HashMap::new();
    for job in &jobs {
        if let Some(other) = seen.insert(absolute(&job.output), &job.input) {
            return Err(Failure::usage(format!(
                "{} and {} would both write {}",
                other.display(),
                job.input.display(),
                job.output.display()
            )));
        }
        if absolute(&job.input) == absolute(&job.output) {
            return Err(Failure::usage(format!("{} would overwrite its own input", job.output.display())));
        }
    }
    if jobs.is_empty() {
        return Err(Failure::usage("no .mmd or .mermaid files found"));
    }
    Ok(jobs)
}
