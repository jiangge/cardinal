mod event;
mod event_flag;
mod event_stream;
mod utils;

pub use event::FsEvent;
pub use event_flag::{EventAction, EventFlag, ScanType};
pub use event_stream::{EventStream, spawn_event_watcher};
pub use fsevent_sys::FSEventStreamEventId;
pub use utils::{current_event_id, dev_of_path, event_id_to_timestamp};
