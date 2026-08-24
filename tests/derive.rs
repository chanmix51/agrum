use agrum::{Projection, SqlEntity, Structured};
use uuid::Uuid;

#[derive(SqlEntity)]
#[allow(dead_code)]
struct Company {
    company_id: Uuid,
    name: String,
    default_address_id: Uuid,
}

#[test]
fn derive_sql_entity_provides_structure_and_default_projection() {
    let structure = Company::get_structure();
    let fields: Vec<_> = structure
        .get_fields()
        .iter()
        .map(|field| field.dump())
        .collect();

    assert_eq!(
        fields,
        vec![
            ("company_id", "uuid"),
            ("name", "text"),
            ("default_address_id", "uuid"),
        ]
    );

    let expected = Projection::<Company>::default();
    let projection = Company::get_projection();

    assert_eq!(projection.to_string(), expected.to_string());
    assert_eq!(
        projection.get_structure().get_fields(),
        expected.get_structure().get_fields()
    );
}
