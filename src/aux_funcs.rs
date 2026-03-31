use crate::TYPES_FOLDER;
use std::fs;
use std::fs::File;
use std::io::Write;

/// Write content to a file, panics upon not being able to open or write.
pub fn write_to_file(path: &str, content: &[u8]) {
    File::create(&path)
        .unwrap_or_else(|e| panic!("{e}, could not open"))
        .write_all(&content)
        .unwrap_or_else(|e| panic!("{e}, could not write"));
}

/// # Write Type to File
///
/// Takes the type name (ident) and the output to write to the file content.
///
/// ## Panics
///
/// Panics if the file cannot be opened or written to.
pub fn write_type_to_file(ident: String, output: &[u8]) -> () {
    let folder = TYPES_FOLDER.trim();

    let created_type_path = format!("{}/{}.ts", folder, ident);

    let _ = fs::create_dir(folder);

    write_to_file(&created_type_path, output);
}
