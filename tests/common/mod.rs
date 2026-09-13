use std::io::{Cursor, Write};

use zip::{write::FileOptions, ZipWriter};

const INDEX: &str = include_str!("../fixtures/mrpack/index.json");

pub fn mrpack(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = FileOptions::default();

    writer
        .start_file("modrinth.index.json", options)
        .expect("starts the index");
    writer
        .write_all(INDEX.as_bytes())
        .expect("writes the index");

    for (name, bytes) in entries {
        writer.start_file(*name, options).expect("starts an entry");
        writer.write_all(bytes).expect("writes an entry");
    }

    writer.finish().expect("finishes the zip").into_inner()
}
