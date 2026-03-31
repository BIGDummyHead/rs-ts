use crate::import_data::ImportData;

/// # Type Kit
///
/// Represents the type and the imports it relies on.
#[derive(Debug)]
pub struct TypeKit {
    pub name: String,
    pub import: Option<ImportData>,
}

impl TypeKit {

    /// # No Import
    ///
    /// Creates a new type kit without import data
    pub fn no_import(name: &str) -> TypeKit {
        TypeKit {
            name: name.to_string(),
            import: None,
        }
    }

    /// # Import
    ///
    /// Creates a new type kit with required import data
    pub fn import(name: &str, data: ImportData) -> TypeKit {
        TypeKit {
            name: name.to_string(),
            import: Some(data),
        }
    }
}