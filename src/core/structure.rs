/// SQL field structure.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructureField {
    /// Name of the field.
    name: String,

    /// SQL type of the field.
    sql_type: String,
}

impl StructureField {
    pub fn new(name: &str, sql_type: &str) -> Self {
        Self {
            name: name.to_string(),
            sql_type: sql_type.to_string(),
        }
    }

    pub fn dump(&self) -> (&str, &str) {
        (&self.name, &self.sql_type)
    }
}
/// Structure of a SQL tuple.
#[derive(Debug, Clone, Default)]
pub struct Structure {
    fields: Vec<StructureField>,
}

impl Structure {
    /// Create a new instance of Structure from a slice of tuples.
    pub fn new(field_definitions: &[(&str, &str)]) -> Self {
        let mut fields: Vec<StructureField> = Vec::new();

        for (name, sql_type) in field_definitions {
            fields.push(StructureField::new(name, sql_type));
        }

        Self { fields }
    }

    pub fn set_field(&mut self, name: &str, sql_type: &str) -> &mut Self {
        let name = name.to_string();
        let sql_type = sql_type.to_string();

        let definition = StructureField { name, sql_type };
        self.fields.push(definition);

        self
    }

    pub fn get_fields(&self) -> &Vec<StructureField> {
        &self.fields
    }

    pub fn get_names(&self) -> Vec<&str> {
        let names: Vec<&str> = self.fields.iter().map(|f| f.name.as_str()).collect();

        names
    }
}

pub trait Structured {
    fn get_structure() -> Structure;
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_structure() -> Structure {
        Structure::new(&[("a_field", "a_type"), ("another_field", "another_type")])
    }

    #[test]
    fn use_structure() {
        let structure = get_structure();

        assert_eq!(
            &[
                StructureField {
                    name: "a_field".to_string(),
                    sql_type: "a_type".to_string()
                },
                StructureField {
                    name: "another_field".to_string(),
                    sql_type: "another_type".to_string()
                }
            ]
            .to_vec(),
            structure.get_fields()
        );
    }

    #[test]
    fn get_names() {
        let structure = get_structure();
        assert_eq!(vec!["a_field", "another_field"], structure.get_names());
    }
}
