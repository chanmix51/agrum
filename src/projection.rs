use std::{collections::HashMap, slice::Iter};

use crate::{structure::StructureField, Structure};

type SourceAliases = HashMap<String, String>;

/// Definition of a projection field.
#[derive(Debug, Clone)]
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
            definition: format!("{{:{}:}}.{}", source_name, field_name),
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

    /// Create the SQL definition of the projection.
    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        let mut definition = self.definition.clone();

        for (source_name, source_alias) in source_aliases {
            definition = definition.replace(&format!("{{:{}:}}", source_name), source_alias);
        }
        format!("{} as {}", definition, self.name)
    }
}

/// A Projection defines what is output from a query in order to hydrate a
/// [SQLEntity]
#[derive(Debug, Clone)]
pub struct Projection {
    structure: Structure,
    fields: Vec<ProjectionFieldDefinition>,
    source_aliases: SourceAliases,
}

impl Projection {
    pub fn from_structure(structure: Structure, source_name: &str) -> Self {
        let source_name = source_name.to_string();
        let fields = structure
            .get_definition()
            .iter()
            .map(|f| ProjectionFieldDefinition::from_structure_field(f, &source_name))
            .collect();
        let source_aliases = [(source_name.clone(), source_name)].into_iter().collect();

        Self {
            structure,
            fields,
            source_aliases,
        }
    }

    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        self.fields
            .iter()
            .map(|def| def.expand(source_aliases))
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn get_fields(&self) -> &[ProjectionFieldDefinition] {
        self.fields.iter().as_slice()
    }

    pub fn get_structure(&self) -> &Structure {
        &self.structure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_projection() -> Projection {
        let mut structure = Structure::new();
        structure
            .set_field("test_id", "int")
            .set_field("something", "text")
            .set_field("is_what", "bool");

        Projection::from_structure(structure, "alias")
    }

    #[test]
    fn test_expand() {
        let projection = get_projection();
        let mut source_aliases = HashMap::new();
        source_aliases.insert("alias".to_string(), "test_alias".to_string());

        assert_eq!(
            String::from("test_alias.test_id as test_id, test_alias.something as something, test_alias.is_what as is_what"),
            projection.expand(&source_aliases)
        );
    }
}
