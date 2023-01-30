use std::collections::{hash_map::Iter, HashMap};

use super::{structure::StructureField, Structure};

#[derive(Debug, Clone)]
pub struct SourceAliases {
    aliases: HashMap<String, String>,
}

impl SourceAliases {
    pub fn new(aliases: Vec<(&str, &str)>) -> Self {
        let aliases: HashMap<String, String> = aliases
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        Self { aliases }
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.aliases.iter()
    }
}

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
}

impl ProjectionFieldDefinition {
    /// Create field definition from a field structure.
    pub fn from_structure_field(structure_field: &StructureField, source_name: &str) -> Self {
        let (field_name, _field_type) = structure_field.dump();

        Self {
            definition: format!("{{:{}:}}.{}", source_name, field_name),
            name: field_name.to_string(),
        }
    }

    /// Instanciate field definition.
    pub fn new(definition: &str, name: &str) -> Self {
        Self {
            definition: definition.to_string(),
            name: name.to_string(),
        }
    }

    /// Create the SQL definition of the projection.
    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        let mut definition = self.definition.clone();

        for (source_name, source_alias) in source_aliases.iter() {
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
}

impl Projection {
    pub fn from_structure(structure: Structure, source_name: &str) -> Self {
        let fields = structure
            .get_fields()
            .iter()
            .map(|f| ProjectionFieldDefinition::from_structure_field(f, source_name))
            .collect();

        Self { structure, fields }
    }

    pub fn set_field(&mut self, field_definition: &str, field_alias: &str) -> &mut Self {
        let definition = ProjectionFieldDefinition::new(field_definition, field_alias);

        for field in self.fields.as_mut_slice() {
            if &field.name == field_alias {
                *field = definition;

                return self;
            }
        }
        self.fields.push(definition);

        self
    }

    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        self.fields
            .iter()
            .map(|def| def.expand(source_aliases))
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn get_fields(&self) -> &[ProjectionFieldDefinition] {
        self.fields.as_slice()
    }

    pub fn get_structure(&self) -> &Structure {
        &self.structure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_projection() -> Projection {
        let mut structure = Structure::default();
        structure
            .set_field("test_id", "int")
            .set_field("something", "text")
            .set_field("is_what", "bool");

        Projection::from_structure(structure, "alias")
    }

    #[test]
    fn test_expand() {
        let projection = get_projection();
        let source_aliases = SourceAliases::new(vec![("alias", "test_alias")]);

        assert_eq!(
            String::from("test_alias.test_id as test_id, test_alias.something as something, test_alias.is_what as is_what"),
            projection.expand(&source_aliases)
        );
    }

    #[test]
    fn test_set_field() {
        let mut projection = get_projection();
        let source_aliases = SourceAliases::new(vec![("alias", "test_alias")]);

        projection
            .set_field("age({:alias:}.born_at)", "how_old")
            .set_field("{:alias:}.is_ok", "is_ok");

        assert_eq!(
            String::from("test_alias.test_id as test_id, test_alias.something as something, test_alias.is_what as is_what, age(test_alias.born_at) as how_old, test_alias.is_ok as is_ok"),
            projection.expand(&source_aliases)
        );
    }

    #[test]
    fn redefine_field() {
        let mut projection = get_projection();
        let source_aliases = SourceAliases::new(vec![("alias", "test_alias")]);
        projection.set_field("initcap({:alias:}.something)", "something");

        assert_eq!(
            String::from("test_alias.test_id as test_id, initcap(test_alias.something) as something, test_alias.is_what as is_what"),
            projection.expand(&source_aliases)
        );
    }
}
