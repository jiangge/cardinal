extern crate cardinal;

use anyhow::{Context, Result};
use cardinal::fs_entry::DiskEntry;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

fn main() -> Result<()> {
    let time = std::time::Instant::now();
    let hierarchy = DiskEntry::from_fs(Path::new("/"));
    println!("elapsed: {}s", time.elapsed().as_secs_f32());

    let file = File::create("target/fs.db").context("open hierarchy file failed.")?;
    let mut file = BufWriter::new(file);

    let time = std::time::Instant::now();
    bincode::encode_into_std_write(hierarchy, &mut file, bincode::config::standard())
        .context("write hierarchy to file failed.")?;
    println!("elapsed: {}s", time.elapsed().as_secs_f32());

    /*
    cardinal::init_sdk();
    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
        let events = cardinal::take_fs_events();
        if !events.is_empty() {
            println!("{:#?}", events);
        }
    }
    */
    Ok(())
}
