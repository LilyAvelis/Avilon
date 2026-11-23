use std::{
    fs,
    hash::DefaultHasher,
    hash::Hasher,
    io,
    path::{Path, PathBuf},
};

pub(super) const PREVIEW: &[u8] = include_bytes!("../../assets/animation/spectre-360.jpg");
const PAGE: &[u8] = include_bytes!("../../assets/animation/spectre-360.html");

pub(super) fn open() -> io::Result<()> {
    let folder = std::env::temp_dir().join("Mira").join("Animations");
    open_browser(&materialize(&folder)?)
}

fn materialize(folder: &Path) -> io::Result<PathBuf> {
    let mut revision = DefaultHasher::new();
    revision.write(PAGE);
    let path = folder.join(format!("spectre-360-{:016x}.html", revision.finish()));
    if fs::read(&path).is_ok_and(|bytes| bytes == PAGE) {
        return Ok(path);
    }
    fs::create_dir_all(folder)?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let write = fs::write(&temporary, PAGE).and_then(|()| fs::rename(&temporary, &path));
    if let Err(error) = write {
        let _ = fs::remove_file(temporary);
        if !fs::read(&path).is_ok_and(|bytes| bytes == PAGE) {
            return Err(error);
        }
    }
    Ok(path)
}

#[cfg(windows)]
fn open_browser(path: &Path) -> io::Result<()> {
    use std::{os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};

    let path: Vec<_> = path.as_os_str().encode_wide().chain([0]).collect();
    let verb: Vec<_> = "open".encode_utf16().chain([0]).collect();
    // Opening a local HTML document through Windows invokes the user's file association.
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            verb.as_ptr(),
            path.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    if result > 32 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(result as i32))
    }
}

#[cfg(not(windows))]
fn open_browser(path: &Path) -> io::Result<()> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(path)
        .spawn()
        .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_packaged_page_and_repairs_incomplete_cache() -> io::Result<()> {
        let folder = std::env::temp_dir().join(format!("mira-spectre-test-{}", std::process::id()));
        let path = materialize(&folder)?;
        assert_eq!(fs::read(&path)?, PAGE);
        assert!(str::from_utf8(PAGE).unwrap().contains("Луч Spectre"));
        fs::write(&path, b"incomplete")?;
        assert_eq!(materialize(&folder)?, path);
        assert_eq!(fs::read(&path)?, PAGE);
        fs::remove_file(path)?;
        fs::remove_dir(folder)
    }
}
