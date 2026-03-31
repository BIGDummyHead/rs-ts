/// # Import Data
/// 
/// Represents data that will be used to identify where an Import comes from for a TypeScript folder.
#[derive(Debug)]
pub struct ImportData {
    pub type_name: String,
    pub file_name: String,
}

impl ImportData {
    pub fn as_string(&self) -> String {
        format!("import {} from './{}';", self.type_name, self.file_name)
    }
}