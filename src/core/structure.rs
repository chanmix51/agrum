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

#[cfg(test)]
mod tests {

    use super::*;

    fn get_structure() -> Structure {
        let mut structure = Structure::default();
        structure
            .set_field("a_field", "a_type")
            .set_field("another_field", "another_type");

        structure
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
