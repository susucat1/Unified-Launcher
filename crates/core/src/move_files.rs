use std::fs;
use std::io::{self, Result};
use std::path::Path;

pub fn move_files(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if !to.exists() {
        fs::create_dir_all(to)?;
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source = entry.path();
        let target = to.join(entry.file_name());

        if fs::rename(&source, &target).is_ok() {
            continue;
        }

        if file_type.is_symlink() {
            move_symlink(&source, &target)?;
        } else if file_type.is_dir() {
            fs::create_dir_all(&target)?;
            move_files(&source, &target)?;
            fs::remove_dir(&source)?;
        } else {
            fs::copy(&source, &target)?;
            fs::remove_file(&source)?;
        }
    }

    Ok(())
}

fn move_symlink(source: &Path, target: &Path) -> Result<()> {
    let link_target = fs::read_link(source)?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(link_target, target)?;
    }

    #[cfg(windows)]
    {
        if source.is_dir() {
            std::os::windows::fs::symlink_dir(link_target, target)?;
        } else {
            std::os::windows::fs::symlink_file(link_target, target)?;
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Symlinks are not supported on this platform",
        ));
    }

    fs::remove_file(source)?;
    Ok(())
}
