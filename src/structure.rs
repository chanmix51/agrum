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
#[derive(Debug, Clone)]
pub struct Structure {
    fields: Vec<StructureField>,
}

impl Default for Structure {
    fn default() -> Self {
        Self { fields: Vec::new() }
    }
}

impl Structure {
    pub fn set_field(&mut self, name: &str, sql_type: &str) -> &mut Self {
        let name = name.to_string();
        let sql_type = sql_type.to_string();

        let definition = StructureField {
            name: name,
            sql_type,
        };
        self.fields.push(definition);

        self
    }

    pub fn get_definition(&self) -> &Vec<StructureField> {
        &self.fields
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn use_structure() {
        let structure = {
            let mut structure = Structure::default();
            structure
                .set_field("a_field", "a_type")
                .set_field("another_field", "another_type");

            structure
        };

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
            structure.get_definition()
        );
    }
}
