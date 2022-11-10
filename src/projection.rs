use std::collections::HashMap;

use crate::{structure::StructureField, Structure};

pub struct ProjectionFieldDefinition {
    source_name: String,
    definition: String,
    name: String,
    sql_type: String,
}

impl ProjectionFieldDefinition {
    pub fn from_structure_field(structure_field: &StructureField, source_name: &str) -> Self {
        let field = structure_field.dump();

        Self {
            source_name: source_name.to_string(),
            definition: format!("**.{}", field.0),
            name: field.0.to_string(),
            sql_type: field.1.to_string(),
        }
    }

    pub fn new(source_name: &str, definition: &str, name: &str, sql_type: &str) -> Self {
        Self {
            source_name: source_name.to_string(),
            definition: definition.to_string(),
            name: name.to_string(),
            sql_type: sql_type.to_string(),
        }
    }

    pub fn get_source_name(&self) -> &str {
        &self.source_name
    }

    pub fn expand(&self, source_alias: &str) -> String {
        format!(
            "{} as {}",
            self.definition.replace("**", source_alias),
            self.name
        )
    }
}
trait Projection {
    fn expand(&self, source_aliases: HashMap<String, String>) -> String;

    fn get_structure(&self) -> Structure;
}
