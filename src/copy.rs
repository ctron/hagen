use fs_extra::dir::{create, create_all, get_dir_content2, CopyOptions, DirOptions};
use fs_extra::file;
use std::path::{Path, PathBuf};

pub type Result<T> = ::std::result::Result<T, Error>;

macro_rules! err {
    ($text:expr, $kind:expr) => {
        return Err(Error::new($kind, $text));
    };

    ($text:expr) => {
        err!($text, ErrorKind::Other)
    };
}

use fs_extra::error::{Error, ErrorKind};

/// Copies the directory contents from one place to another using recursive method.
/// This function will also copy the permission bits of the original files to
/// destionation files (not for directories).
///
/// # Errors
///
/// This function will return an error in the following situations, but is not limited to just
/// these cases:
///
/// * This `from` path is not a directory.
/// * This `from` directory does not exist.
/// * Invalid folder name for `from` or `to`.
/// * The current process does not have the permission rights to access `from` or write `to`.
///
/// # Example
/// ```rust,ignore
/// extern crate fs_extra;
/// use fs_extra::dir::copy;
///
/// let options = CopyOptions::new(); //Initialize default values for CopyOptions
/// // options.mirror_copy = true; // To mirror copy the whole structure of the source directory
///
///
/// // copy source/dir1 to target/dir1
/// copy("source/dir1", "target/dir1", &options)?;
///
/// ```
pub fn copy<P, Q>(from: P, to: Q, options: &CopyOptions) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();

    if !from.exists() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" does not exist or you don't have access!", msg);
            err!(&msg, ErrorKind::NotFound);
        }
        err!(
            "Path does not exist Or you don't have access!",
            ErrorKind::NotFound
        );
    }
    if !from.is_dir() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" is not a directory!", msg);
            err!(&msg, ErrorKind::InvalidFolder);
        }
        err!("Path is not a directory!", ErrorKind::InvalidFolder);
    }
    let mut to: PathBuf = to.as_ref().to_path_buf();
    if !options.copy_inside {
        if let Some(dir_name) = from.components().last() {
            to.push(dir_name.as_os_str());
        } else {
            err!("Invalid folder from", ErrorKind::InvalidFolder);
        }
    }

    let mut read_options = DirOptions::new();
    if options.depth > 0 {
        read_options.depth = options.depth;
    }

    let dir_content = get_dir_content2(from, &read_options)?;
    for directory in dir_content.directories {
        let tmp_to = Path::new(&directory).strip_prefix(from)?;
        let dir = to.join(&tmp_to);
        if !dir.exists() {
            if options.copy_inside {
                create_all(dir, false)?;
            } else {
                create(dir, false)?;
            }
        }
    }
    let mut result: u64 = 0;
    for file in dir_content.files {
        let to = to.to_path_buf();
        let tp = Path::new(&file).strip_prefix(from)?;
        let path = to.join(&tp);

        let file_options = file::CopyOptions {
            overwrite: options.overwrite,
            skip_exist: options.skip_exist,
            buffer_size: options.buffer_size,
        };
        let mut result_copy: Result<u64>;
        let mut work = true;

        while work {
            result_copy = file::copy(&file, &path, &file_options);
            match result_copy {
                Ok(val) => {
                    result += val;
                    work = false;
                }
                Err(err) => {
                    let err_msg = err.to_string();
                    err!(err_msg.as_str(), err.kind)
                }
            }
        }
    }
    Ok(result)
}
