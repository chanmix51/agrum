use std::{collections::HashMap, slice::Iter};

use crate::{structure::StructureField, Structure};

type SourceAliases = HashMap<String, String>;

/// Definition of a projection field.
pub struct ProjectionFieldDefinition {
    /// SQL definition of the field, usally a field name but can be any SQL operation of function.
    /// Note that the expression "**" is replaced by the source name when expanded. By example:
    /// `**.thing_id` will turn into `my_source.thing_id` (if source name is
    /// `my_source` obviously) when expanded. `sum(**.field)` will turn to `sum(my_source.field)`
    definition: String,

    /// Output field name
    name: String,

    /// SQL type of the output field
    sql_type: String,
}

impl ProjectionFieldDefinition {
    /// Create field definition from a field structure.
    pub fn from_structure_field(structure_field: &StructureField, source_name: &str) -> Self {
        let (field_name, field_type) = structure_field.dump();

        Self {
            definition: format!("{{:{}:}}*.{}", source_name, field_name),
            name: field_name.to_string(),
            sql_type: field_type.to_string(),
        }
    }

    /// Instanciate field definition.
    pub fn new(definition: &str, name: &str, sql_type: &str) -> Self {
        Self {
            definition: definition.to_string(),
            name: name.to_string(),
            sql_type: sql_type.to_string(),
        }
    }

    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        let mut definition = self.definition.clone();

        for (source_name, source_alias) in source_aliases {
            definition = definition.replace(&format!("{{:{}:}}", source_name), source_alias);
        }
        format!("{} as {}", definition, self.name)
    }
}

trait Projection {
    fn get_fields(&self) -> Iter<&ProjectionFieldDefinition>;

    fn expand(&self, source_aliases: &SourceAliases) -> String {
        self.get_fields()
            .map(|def| def.expand(source_aliases))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn get_structure(&self) -> Structure;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProjection {
        fields: Vec<ProjectionFieldDefinition>,
        source_aliases: SourceAliases,
    }

    impl Projection for TestProjection {
        fn get_fields(&self) -> Iter<&ProjectionFieldDefinition> {
            todo!()
        }

        fn get_structure(&self) -> Structure {
            todo!()
        }
    }
}
