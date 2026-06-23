use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use fs2::FileExt;
use tempfile::NamedTempFile;

pub fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;
    temp.as_file().write_all(contents)?;
    temp.persist(path).map_err(|err| err.error)?;
    Ok(())
}

pub fn locked_append_line(path: &Path, line: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Windows cannot exclusive-lock files opened append-only; read+write + seek works.
    #[cfg(windows)]
    {
        use std::io::Seek;
        let mut file =
            OpenOptions::new().create(true).read(true).write(true).truncate(false).open(path)?;
        file.seek(io::SeekFrom::End(0))?;
        file.lock_exclusive()?;
        let result = writeln!(file, "{line}");
        file.unlock()?;
        result
    }

    #[cfg(not(windows))]
    {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.lock_exclusive()?;
        let result = writeln!(file, "{line}");
        file.unlock()?;
        result
    }
}
