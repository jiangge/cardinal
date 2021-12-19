use super::fsevent_flags::EventFlags;

use fsevent_sys::FSEventStreamEventId;

use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

#[derive(Debug)]
pub struct FsEvent {
    pub path: PathBuf,
    pub flag: EventFlags,
    pub id: FSEventStreamEventId,
}

impl FsEvent {
    pub(crate) fn from_raw(path: *const i8, flag: u32, id: u64) -> Self {
        let path = unsafe { CStr::from_ptr(path) };
        let path = OsStr::from_bytes(path.to_bytes());
        let path = PathBuf::from(path);
        let flag = EventFlags::from_bits_truncate(flag);
        FsEvent { path, flag, id }
    }
}
