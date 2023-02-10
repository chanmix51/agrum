use std::collections::{hash_map::Iter, HashMap};

use super::Structure;

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
    definition: String,

    /// Output field name
    name: String,
}

impl ProjectionFieldDefinition {
    /// Instanciate field definition.
    pub fn new(definition: &str, name: &str) -> Self {
        Self {
            definition: definition.to_string(),
            name: name.to_string(),
        }
    }

    /// Create the SQL definition of the projection.
    pub fn expand(&self) -> String {
        let definition = self.definition.clone();

        format!("{} as {}", definition, self.name)
    }
}

/// A Projection defines what is output from a query in order to hydrate a
/// [SQLEntity]
#[derive(Debug, Clone, Default)]
pub struct Projection {
    structure: Structure,
    fields: Vec<ProjectionFieldDefinition>,
}

impl Projection {
    pub fn new(field_list: Vec<(&str, &str, &str)>) -> Self {
        let mut projection = Self::default();

        for (name, definition, sql_type) in field_list {
            projection.set_field(name, definition, sql_type);
        }

        projection
    }

    pub fn set_field(&mut self, name: &str, definition: &str, sql_type: &str) -> &mut Self {
        let definition = ProjectionFieldDefinition::new(definition, name);

        for field in self.fields.as_mut_slice() {
            if field.name == name {
                *field = definition;

                return self;
            }
        }
        self.fields.push(definition);
        self.structure.set_field(name, sql_type);

        self
    }

    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        let projection = self
            .fields
            .iter()
            .map(|def| def.expand())
            .collect::<Vec<String>>()
            .join(", ");

        source_aliases
            .iter()
            .fold(projection, |projection, source_alias| {
                projection.replace(&format!("{{:{}:}}", source_alias.0), source_alias.1)
            })
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
        let field_list: Vec<(&str, &str, &str)> = [
            ("test_id", "{:alias:}.test_id", "int"),
            ("something", "something", "text"),
            ("is_what", "is_what", "bool"),
        ]
        .to_vec();

        Projection::new(field_list)
    }

    #[test]
    fn test_expand() {
        let projection = get_projection();
        let source_aliases = SourceAliases::new(vec![("alias", "test_alias")]);

        assert_eq!(
            String::from(
                "test_alias.test_id as test_id, something as something, is_what as is_what"
            ),
            projection.expand(&source_aliases)
        );
    }

    #[test]
    fn test_set_field() {
        let mut projection = get_projection();
        let source_aliases = SourceAliases::new(vec![("alias", "test_alias")]);

        projection
            .set_field("how_old", "age({:alias:}.born_at)", "interval")
            .set_field("is_ok", "{:alias:}.is_ok", "bool");

        assert_eq!(
            String::from("test_alias.test_id as test_id, something as something, is_what as is_what, age(test_alias.born_at) as how_old, test_alias.is_ok as is_ok"),
            projection.expand(&source_aliases)
        );
    }

    #[test]
    fn redefine_field() {
        let mut projection = get_projection();
        let source_aliases = SourceAliases::new(vec![("alias", "test_alias")]);
        projection.set_field("something", "initcap({:alias:}.something)", "text");

        assert_eq!(
            String::from("test_alias.test_id as test_id, initcap(test_alias.something) as something, is_what as is_what"),
            projection.expand(&source_aliases)
        );
    }
}
