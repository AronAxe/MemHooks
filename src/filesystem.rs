//! Filesystem safety shared by parsing and maintenance.
use crate::parser::ParseError;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::Path;

pub(crate) fn path_error(path: &Path, code: &str, message: impl Into<String>) -> ParseError {
    ParseError {
        code: code.into(),
        path: path.to_path_buf(),
        message: message.into(),
        line: None,
        column: None,
    }
}

pub(crate) fn regular_file_exists(path: &Path) -> Result<bool, ParseError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => Err(path_error(
            path,
            "MH029",
            "hook and lock paths must be regular files, not symlinks or special files",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(path_error(
            path,
            "MH000",
            format!("could not inspect file: {error}"),
        )),
    }
}

pub(crate) fn open_regular_file(path: &Path, writable: bool) -> Result<File, ParseError> {
    regular_file_exists(path)?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(writable)
        .create(writable)
        .truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_NONBLOCK also prevents a concurrently substituted FIFO from hanging an agent.
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path).map_err(|error| {
        path_error(
            path,
            "MH000",
            format!("could not open file safely: {error}"),
        )
    })?;
    let metadata = file.metadata().map_err(|error| {
        path_error(
            path,
            "MH000",
            format!("could not inspect opened file: {error}"),
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(path_error(
            path,
            "MH029",
            "opened hook or lock is not a regular file",
        ));
    }
    Ok(file)
}
