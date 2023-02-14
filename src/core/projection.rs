use std::{collections::HashMap, marker::PhantomData};

use super::{structure::Structured, Structure};

//pub type SourceAliases = HashMap<String, String>;

pub struct SourceAliases {
    aliases: HashMap<String, String>,
}

impl SourceAliases {
    pub fn new(alias_def: &[(&str, &str)]) -> Self {
        let mut aliases: HashMap<String, String> = HashMap::new();

        for (name, alias) in alias_def {
            aliases.insert(name.to_string(), alias.to_string());
        }

        Self { aliases }
    }

    pub fn get_aliases(&self) -> &HashMap<String, String> {
        &self.aliases
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
#[derive(Debug, Clone)]
pub struct Projection<T>
where
    T: Structured,
{
    structure: Structure,
    fields: Vec<ProjectionFieldDefinition>,
    _phantom: PhantomData<T>,
}

impl<T> Default for Projection<T>
where
    T: Structured,
{
    fn default() -> Self {
        let mut fields: Vec<ProjectionFieldDefinition> = Vec::new();
        let structure = T::get_structure();

        for def in structure.get_fields() {
            let (name, _type) = def.dump();
            fields.push(ProjectionFieldDefinition {
                definition: name.to_owned(),
                name: name.to_owned(),
            });
        }

        Self {
            structure,
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<T> Projection<T>
where
    T: Structured,
{
    /// Replace a field definition. It panics if the field is not declared.
    pub fn set_definition(&mut self, name: &str, definition: &str) -> &mut Self {
        let definition = ProjectionFieldDefinition::new(definition, name);

        for field in self.fields.as_mut_slice() {
            if field.name == name {
                *field = definition;

                return self;
            }
        }

        panic!(
            "Field {name} not found in projection. Available fields: '{}'.",
            self.get_fields().join(", ")
        );
    }

    /// Return the projection SQL definition to be used in queries.
    pub fn expand(&self, source_aliases: &SourceAliases) -> String {
        let projection = self
            .fields
            .iter()
            .map(|def| def.expand())
            .collect::<Vec<String>>()
            .join(", ");

        source_aliases
            .get_aliases()
            .iter()
            .fold(projection, |projection, (name, alias)| {
                projection.replace(&format!("{{:{name}:}}"), alias)
            })
    }

    /// Return the field names list.
    pub fn get_fields(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.name.to_owned()).collect()
    }

    /// Return the underlying structure.
    pub fn get_structure(&self) -> &Structure {
        &self.structure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStructured;

    impl Structured for TestStructured {
        fn get_structure() -> Structure {
            Structure::new(&[
                ("test_id", "int"),
                ("something", "text"),
                ("is_what", "bool"),
            ])
        }
    }

    fn get_projection() -> Projection<TestStructured> {
        let mut projection = Projection::<TestStructured>::default();
        projection.set_definition("test_id", "{:alias:}.test_id");

        projection
    }

    #[test]
    fn test_expand() {
        let projection = get_projection();
        let source_aliases = SourceAliases::new(&[("alias", "test_alias")]);

        assert_eq!(
            String::from(
                "test_alias.test_id as test_id, something as something, is_what as is_what"
            ),
            projection.expand(&source_aliases)
        );
    }

    #[test]
    #[should_panic]
    fn test_unexistent_field() {
        let mut projection = get_projection();

        projection
            .set_definition("how_old", "age({:alias:}.born_at)")
            .set_definition("test_id", "{:alias:}.is_ok");
    }

    #[test]
    fn redefine_field() {
        let mut projection = get_projection();
        let source_aliases = SourceAliases::new(&[("alias", "test_alias")]);
        projection.set_definition("something", "initcap({:alias:}.something)");

        assert_eq!(
            String::from("test_alias.test_id as test_id, initcap(test_alias.something) as something, is_what as is_what"),
            projection.expand(&source_aliases)
        );
    }
}
