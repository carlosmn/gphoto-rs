use std::ffi::{CString};
use std::fs::File;
use std::mem;
use std::path::Path;
use std::convert::TryFrom;

use std::os::unix::prelude::*;


/// A trait for types that can store media.
pub trait Media {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut ::gphoto2::CameraFile;
}


/// Media stored as a local file.
pub struct FileMedia {
    file: *mut ::gphoto2::CameraFile,
}

impl Drop for FileMedia {
    fn drop(&mut self) {
        unsafe {
            ::gphoto2::gp_file_unref(self.file);
        }
    }
}

impl FileMedia {
    /// Creates a new file that stores media.
    ///
    /// This function creates a new file on disk. The file will start out empty.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the file can not be created:
    ///
    /// * `FileExists` if the file already exists.
    pub fn create(path: &Path) -> ::Result<Self> {
        use ::libc::{O_CREAT,O_EXCL,O_RDWR};

        let cstr = match CString::new(path.as_os_str().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_BAD_PARAMETERS))
        };

        let fd = unsafe { ::libc::open(cstr.as_ptr(), O_CREAT|O_EXCL|O_RDWR, 0o644) };
        if fd < 0 {
            return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_FILE_EXISTS));
        }

        let mut ptr = unsafe { mem::uninitialized() };

        match unsafe { ::gphoto2::gp_file_new_from_fd(&mut ptr, fd) } {
            ::gphoto2::GP_OK => {
                Ok(FileMedia { file: ptr })
            },
            err => {
                unsafe {
                    ::libc::close(fd);
                }

                Err(::error::from_libgphoto2(err))
            }
        }
    }

    /// Create a new FileMedia to store data in memory
    pub fn new() -> ::Result<Self> {
        let mut file = mem::MaybeUninit::uninit();
        match unsafe { ::gphoto2::gp_file_new(file.as_mut_ptr()) } {
            ::gphoto2::GP_OK => {
                Ok(FileMedia { file: unsafe { file.assume_init() } })
            },
            err => {
                Err(::error::from_libgphoto2(err))
            }
        }
    }
}

impl Media for FileMedia {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut ::gphoto2::CameraFile {
        self.file
    }
}

impl TryFrom<File> for FileMedia {
    type Error = crate::Error;

    fn try_from(f: File) -> ::Result<Self> {
        let mut ptr = mem::MaybeUninit::uninit();

        match unsafe { ::gphoto2::gp_file_new_from_fd(ptr.as_mut_ptr(), f.into_raw_fd()) } {
            ::gphoto2::GP_OK => {
                Ok(FileMedia { file: unsafe { ptr.assume_init() } })
            },
	    err => {
                Err(::error::from_libgphoto2(err))
	    }
        }
    }
}

impl TryFrom<RawFd> for FileMedia {
    type Error = crate::Error;

    /// Try to convert from a RawFd
    ///
    /// It is important to make sure the descriptor will live long enough
   fn try_from(fd: RawFd) -> ::Result<Self> {
        let mut ptr = mem::MaybeUninit::uninit();

        match unsafe { ::gphoto2::gp_file_new_from_fd(ptr.as_mut_ptr(), fd) } {
            ::gphoto2::GP_OK => {
                Ok(FileMedia { file: unsafe { ptr.assume_init() } })
            },
	    err => {
                Err(::error::from_libgphoto2(err))
	    }
        }
    }
}
