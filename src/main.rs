#![deny(unsafe_op_in_unsafe_fn)]
mod fsevent;
mod fsevent_flags;

use fsevent::FsEvent;

use core_foundation::{
    array::{CFArrayCreate, CFArray},
    date::CFTimeInterval,
    mach_port::CFAllocatorRef,
    runloop::{kCFRunLoopDefaultMode, CFRunLoopGetCurrent, CFRunLoopRun},
    string::CFString,
    base::TCFType,
};
use fsevent_sys::{
    kFSEventStreamCreateFlagFileEvents, kFSEventStreamCreateFlagNoDefer,
    kFSEventStreamEventIdSinceNow, FSEventStreamContext, FSEventStreamCreate,
    FSEventStreamEventFlags, FSEventStreamEventId, FSEventStreamRef,
    FSEventStreamScheduleWithRunLoop, FSEventStreamStart,
};
use std::{
    ffi::CStr,
    ffi::{c_void, OsStr},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    ptr, slice,
};

type EventsCallback = Box<dyn FnMut(&[FsEvent]) + Send>;

extern "C" fn callback(
    stream: FSEventStreamRef,   // ConstFSEventStreamRef streamRef
    callback_info: *mut c_void, // void *clientCallBackInfo
    num_events: usize,          // size_t numEvents
    event_paths: *mut c_void,   // void *eventPaths
    event_flags: *const FSEventStreamEventFlags, // const FSEventStreamEventFlags eventFlags[]
    event_ids: *const FSEventStreamEventId, // const FSEventStreamEventId eventIds[]
) {
    let event_paths = unsafe { slice::from_raw_parts(event_paths as *const *const i8, num_events) };
    let event_flags =
        unsafe { slice::from_raw_parts(event_flags as *const FSEventStreamEventFlags, num_events) };
    let event_ids =
        unsafe { slice::from_raw_parts(event_ids as *const FSEventStreamEventId, num_events) };

    let events: Vec<_> = event_paths
        .iter()
        .zip(event_flags)
        .zip(event_ids)
        .map(|((&path, &flag), &id)| FsEvent::from_raw(path, flag, id))
        .collect();
    println!("events_paths: {:#?}", events);
}

fn listen() {
    let allocator: CFAllocatorRef = ptr::null();
    let mypath = CFString::new("/");
    let paths_to_watch = CFArray::from_CFTypes(&[mypath]);
    // could put stream-specific data here.
    let context: *const FSEventStreamContext = ptr::null();
    // Latency in seconds
    let latency: CFTimeInterval = 0.1; 

    // https://developer.apple.com/documentation/coreservices/1455376-fseventstreamcreateflags
    let stream: FSEventStreamRef = unsafe {
        FSEventStreamCreate(
            allocator as _,
            callback,
            context,
            paths_to_watch.as_concrete_TypeRef() as _,
            kFSEventStreamEventIdSinceNow,
            latency,
            kFSEventStreamCreateFlagNoDefer | kFSEventStreamCreateFlagFileEvents,
        )
    };
    println!("{:p}", stream);
    let run_loop = unsafe { CFRunLoopGetCurrent() };
    unsafe {
        FSEventStreamScheduleWithRunLoop(stream, run_loop as _, kCFRunLoopDefaultMode as _);
    };
    let result = unsafe { FSEventStreamStart(stream) };
    println!("start result: {}", result);
    unsafe { CFRunLoopRun() };
}

fn main() {
    listen();
}
