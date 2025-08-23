use bincode::{Decode, Encode};
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, Metadata},
    io::{Error, ErrorKind},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::UNIX_EPOCH,
};

#[derive(Serialize, Encode, Debug)]
pub struct Node {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
    pub name: String,
    pub metadata: Option<NodeMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone, Copy)]
pub struct NodeMetadata {
    pub r#type: NodeFileType,
    pub ctime: Option<u64>,
    pub mtime: Option<u64>,
    pub size: u64,
}

impl From<Metadata> for NodeMetadata {
    fn from(metadata: Metadata) -> Self {
        Self::new(&metadata)
    }
}

impl NodeMetadata {
    fn new(metadata: &Metadata) -> Self {
        let r#type = metadata.file_type().into();
        let ctime = metadata
            .created()
            .ok()
            .and_then(|x| x.duration_since(UNIX_EPOCH).ok())
            .map(|x| x.as_secs());
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|x| x.duration_since(UNIX_EPOCH).ok())
            .map(|x| x.as_secs());
        let size = metadata.size();
        Self {
            r#type,
            ctime,
            mtime,
            size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone, Copy)]
#[repr(u8)]
pub enum NodeFileType {
    // File occurs a lot, assign it to 0 for better compression ratio(I guess... maybe useful).
    File = 0,
    Dir = 1,
    Symlink = 2,
    Unknown = 3,
}

impl From<fs::FileType> for NodeFileType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_file() {
            NodeFileType::File
        } else if file_type.is_dir() {
            NodeFileType::Dir
        } else if file_type.is_symlink() {
            NodeFileType::Symlink
        } else {
            NodeFileType::Unknown
        }
    }
}

#[derive(Default, Debug)]
pub struct WalkData {
    pub num_files: AtomicUsize,
    pub num_dirs: AtomicUsize,
    ignore_directory: Option<PathBuf>,
    /// If set, metadata will be collected for each file node(folder node will get free metadata).
    need_metadata: bool,
}

impl WalkData {
    pub const fn new(path: PathBuf, need_metadata: bool) -> Self {
        Self {
            num_files: AtomicUsize::new(0),
            num_dirs: AtomicUsize::new(0),
            ignore_directory: Some(path),
            need_metadata,
        }
    }
}

pub fn walk_it(dir: &Path, walk_data: &WalkData) -> Option<Node> {
    walk(dir, walk_data)
}

fn walk(path: &Path, walk_data: &WalkData) -> Option<Node> {
    if walk_data.ignore_directory.as_deref() == Some(path) {
        return None;
    }
    // doesn't traverse symlink
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => Some(metadata),
        // If it's not found, we definitely don't want it.
        Err(e) if e.kind() == ErrorKind::NotFound => return None,
        // If it's permission denied or something, we still want to insert it into the tree.
        Err(e) => {
            if handle_error_and_retry(&e) {
                // doesn't traverse symlink
                path.symlink_metadata().ok()
            } else {
                None
            }
        }
    };
    let children = if metadata.as_ref().map(|x| x.is_dir()).unwrap_or_default() {
        walk_data.num_dirs.fetch_add(1, Ordering::Relaxed);
        let read_dir = fs::read_dir(&path);
        match read_dir {
            Ok(entries) => entries
                .into_iter()
                .par_bridge()
                .filter_map(|entry| {
                    match &entry {
                        Ok(entry) => {
                            if walk_data.ignore_directory.as_deref() == Some(path) {
                                return None;
                            }
                            // doesn't traverse symlink
                            if let Ok(data) = entry.file_type() {
                                if data.is_dir() {
                                    return walk(&entry.path(), walk_data);
                                } else {
                                    walk_data.num_files.fetch_add(1, Ordering::Relaxed);
                                    let name = entry
                                        .path()
                                        .file_name()
                                        .map(|x| x.to_string_lossy().into_owned())
                                        .unwrap_or_default();
                                    return Some(Node {
                                        children: vec![],
                                        name,
                                        metadata: walk_data
                                            .need_metadata
                                            .then_some(entry)
                                            .and_then(|entry| {
                                                // doesn't traverse symlink
                                                entry.metadata().ok().map(NodeMetadata::from)
                                            }),
                                    });
                                }
                            }
                        }
                        Err(failed) => {
                            if handle_error_and_retry(failed) {
                                return walk(path, walk_data);
                            }
                        }
                    }
                    None
                })
                .collect(),
            Err(failed) => {
                if handle_error_and_retry(&failed) {
                    return walk(path, walk_data);
                } else {
                    vec![]
                }
            }
        }
    } else {
        walk_data.num_files.fetch_add(1, Ordering::Relaxed);
        vec![]
    };
    let name = path
        .file_name()
        .map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_default();
    Some(Node {
        children,
        name,
        metadata: metadata.map(NodeMetadata::from),
    })
}

fn handle_error_and_retry(failed: &Error) -> bool {
    failed.kind() == std::io::ErrorKind::Interrupted
}
