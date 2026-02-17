use std::marker::PhantomData;

use super::{SqlEntity, Structure};

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
    T: SqlEntity,
{
    structure: Structure,
    fields: Vec<ProjectionFieldDefinition>,
    _phantom: PhantomData<T>,
}

impl<T> Default for Projection<T>
where
    T: SqlEntity,
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
    T: SqlEntity,
{
    /// Replace a field definition. It panics if the field is not declared.
    pub fn set_definition(mut self, name: &str, definition: &str) -> Self {
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
    pub fn expand(&self) -> String {
        self.fields
            .iter()
            .map(|def| def.expand())
            .collect::<Vec<String>>()
            .join(", ")
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
    use tokio_postgres::Row;

    use crate::Structured;

    use super::*;

    #[allow(dead_code)]
    struct TestSqlEntity {
        test_id: i64,
        something: String,
        is_what: bool,
    }

    impl Structured for TestSqlEntity {
        fn get_structure() -> Structure {
            Structure::new(&[
                ("test_id", "int"),
                ("something", "text"),
                ("is_what", "bool"),
            ])
        }
    }

    impl SqlEntity for TestSqlEntity {
        fn hydrate(row: &Row) -> Result<Self, crate::HydrationError>
        where
            Self: Sized,
        {
            let entity = Self {
                test_id: row.get("test_id"),
                something: row.get("something"),
                is_what: row.get("is_what"),
            };

            Ok(entity)
        }

        fn get_projection() -> Projection<TestSqlEntity> {
            Projection::<TestSqlEntity>::default()
        }
    }

    #[test]
    fn test_expand() {
        let projection = TestSqlEntity::get_projection();

        assert_eq!(
            String::from("test_id as test_id, something as something, is_what as is_what"),
            projection.expand()
        );
    }

    #[test]
    #[should_panic]
    fn test_unexistent_field() {
        let _projection = TestSqlEntity::get_projection()
            .set_definition("how_old", "age({:alias:}.born_at)")
            .set_definition("test_id", "{:alias:}.is_ok");
    }

    #[test]
    fn redefine_field() {
        let projection =
            TestSqlEntity::get_projection().set_definition("something", "initcap(something)");

        assert_eq!(
            String::from("test_id as test_id, initcap(something) as something, is_what as is_what"),
            projection.expand()
        );
    }
}
