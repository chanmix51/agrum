use std::marker::PhantomData;

use agrum::{
    Projection, QueryBook, ReadQueryBook, SqlEntity, SqlQuery, Structured, Transaction,
    WhereCondition,
};
use futures_util::stream::StreamExt;
use uuid::Uuid;

mod pool;
use pool::get_pool;

const COMPANY_1_ID: &str = "a7b5f2c8-8816-4c40-86bf-64e066a8db7a";
const COMPANY_2_ID: &str = "dcce1188-66ad-48a1-bb41-756a48514ac4";
const ADDRESS_1_ID: &str = "ffb3ef0e-697d-4fba-bc4f-28317dc44626";
const ADDRESS_2_ID: &str = "af18dfe3-b189-4d80-bbc9-a90792d92143";

#[derive(SqlEntity)]
struct Company {
    company_id: Uuid,
    name: String,
    default_address_id: Uuid,
}

struct CompanyQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> Default for CompanyQueryBook<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: SqlEntity> QueryBook<T> for CompanyQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.company"
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for CompanyQueryBook<T> {}

impl<T: SqlEntity> CompanyQueryBook<T> {
    fn get_all<'a>(&self) -> SqlQuery<'a, T> {
        self.select(WhereCondition::default())
    }
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

#[tokio::test]
#[ignore = "skipping database tests"]
async fn derive_sql_entity_hydrates_company_from_database() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;

    let query = CompanyQueryBook::<Company>::default().get_all();
    let results = transaction
        .query(query)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;

    assert_eq!(results.len(), 2);
    let companies: Vec<_> = results.into_iter().map(|row| row.unwrap()).collect();

    let first = companies
        .iter()
        .find(|company| company.name == "first")
        .unwrap();
    assert_eq!(first.company_id, Uuid::parse_str(COMPANY_1_ID).unwrap());
    assert_eq!(
        first.default_address_id,
        Uuid::parse_str(ADDRESS_1_ID).unwrap()
    );

    let second = companies
        .iter()
        .find(|company| company.name == "second")
        .unwrap();
    assert_eq!(second.company_id, Uuid::parse_str(COMPANY_2_ID).unwrap());
    assert_eq!(
        second.default_address_id,
        Uuid::parse_str(ADDRESS_2_ID).unwrap()
    );

    transaction.rollback().await.unwrap();
}
