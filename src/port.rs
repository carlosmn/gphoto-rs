use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem;

use ::libc::c_void;

/// Types of ports.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum PortType {
    /// Serial port.
    Serial,

    /// USB port.
    USB,

    /// Disk or local mountpoint.
    Disk,

    /// PTP or IP connection.
    PTPIP,

    /// Direct I/O on a USB mass storage device.
    Direct,

    /// USB mass storage raw SCSI port.
    SCSI,

    /// Unknown port type.
    Other,
}

/// A structure describing a port.
///
/// ## Example
///
/// A `Port` object can be used to report information about a camera's connection:
///
/// ```no_run
/// let mut context = gphoto::Context::new().unwrap();
/// let mut camera = gphoto::Camera::new().unwrap();
/// camera.init(&mut context).unwrap();
/// let port = camera.port();
///
/// println!("port type = {:?}", port.port_type());
/// println!("port name = {:?}", port.name());
/// println!("port path = {:?}", port.path());
/// ```
///
/// The above example may print something like the following:
///
/// ```text
/// port type = USB
/// port name = "Universal Serial Bus"
/// port path = "usb:020,007"
/// ```
pub struct Port<'a> {
    // GPPortInfo is a typedef for a pointer. Lifetime is needed because it borrows data owned by
    // the Camera struct.
    inner: ::gphoto2::GPPortInfo,
    __phantom: PhantomData<&'a c_void>,
}

impl<'a> Port<'a> {
    /// Returns the type of the port.
    pub fn port_type(&self) -> PortType {
        let mut port_type = mem::MaybeUninit::uninit();

        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_port_info_get_type(self.inner, port_type.as_mut_ptr()));
        }

        match unsafe { port_type.assume_init() } {
            ::gphoto2::GP_PORT_SERIAL          => PortType::Serial,
            ::gphoto2::GP_PORT_USB             => PortType::USB,
            ::gphoto2::GP_PORT_DISK            => PortType::Disk,
            ::gphoto2::GP_PORT_PTPIP           => PortType::PTPIP,
            ::gphoto2::GP_PORT_USB_DISK_DIRECT => PortType::Direct,
            ::gphoto2::GP_PORT_USB_SCSI        => PortType::SCSI,
            ::gphoto2::GP_PORT_NONE | _        => PortType::Other,
        }
    }

    /// Returns the name of the port.
    pub fn name(&self) -> Cow<str> {
        let mut name = mem::MaybeUninit::uninit();

        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_port_info_get_name(self.inner, name.as_mut_ptr()));
            String::from_utf8_lossy(CStr::from_ptr(name.assume_init()).to_bytes())
        }
    }

    /// Returns the path of the port.
    pub fn path(&self) -> Cow<str> {
        let mut path = mem::MaybeUninit::uninit();

        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_port_info_get_path(self.inner, path.as_mut_ptr()));
            String::from_utf8_lossy(CStr::from_ptr(path.assume_init()).to_bytes())
        }
    }
}

#[doc(hidden)]
pub fn from_libgphoto2(_camera: & ::camera::Camera, ptr: ::gphoto2::GPPortInfo) -> Port {
    Port {
        inner: ptr,
        __phantom: PhantomData,
    }
}

/// A structure representing a list of PortInfo structures
#[repr(transparent)]
pub struct PortList(*mut ::gphoto2::GPPortInfoList);

impl Drop for PortList {
    fn drop(&mut self) {
        unsafe {
            ::gphoto2::gp_port_info_list_free(self.0);
        }
    }
}

impl PortList {
    /// Allocate a new list
    pub fn new() -> ::Result<Self> {
        let mut list = mem::MaybeUninit::uninit();
        try_unsafe!(::gphoto2::gp_port_info_list_new(list.as_mut_ptr()));
        let list = unsafe { list.assume_init() };

        Ok(PortList(list as *mut _))
    }

    /// Searches the system for io-drivers and appends them to the list. You
    /// would normally call this function once after PortList::new() and
    /// then use this list in order to supply gp_port_set_info with parameters
    /// or to do auto detection.
    pub fn load(&mut self) -> ::Result<()> {
        try_unsafe!(::gphoto2::gp_port_info_list_load(self.as_mut_ptr()));

        Ok(())
    }

    /// Looks for an entry in the list with the exact given name.
    ///
    /// Returns the index of the entry or an error
    pub fn lookup_name(&mut self, name: &str) -> ::Result<usize> {
        let cname = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_BAD_PARAMETERS)),
        };
        let idx = match unsafe {
            ::gphoto2::gp_port_info_list_lookup_name(self.as_mut_ptr(), cname.as_ptr())
        } {
            idx if idx >= 0 => idx,
            err => return Err(::error::from_libgphoto2(err)),
        };

        Ok(idx as usize)
    }

    /// Looks for an entry in the list with the supplied path. If no exact match
    /// can be found, a regex search will be performed in the hope some driver
    /// claimed ports like "serial:*".
    ///
    /// Returns the index of the entry or an error
    pub fn lookup_path(&mut self, path: &str) -> ::Result<usize> {
        let cpath = match CString::new(path) {
            Ok(s) => s,
            Err(_) => return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_BAD_PARAMETERS)),
        };
        let idx = match unsafe {
            ::gphoto2::gp_port_info_list_lookup_path(self.as_mut_ptr(), cpath.as_ptr())
        } {
            idx if idx >= 0 => idx,
            err => return Err(::error::from_libgphoto2(err)),
        };

        Ok(idx as usize)
    }

    /// Return a mutable underlying pointer
    fn as_mut_ptr(&mut self) -> *mut ::gphoto2::GPPortInfoList {
        self.0
    }

    /// Get the amount of entries in the list
    pub fn len(&mut self) -> usize {
        let l = unsafe { ::gphoto2::gp_port_info_list_count(self.0) };

        if l < 0 {
            panic!();
        }

        l as usize
    }
}
