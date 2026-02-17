use std::{any::Any, env, marker::PhantomData};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use futures_util::stream::StreamExt;
use tokio_postgres::NoTls;
use uuid::Uuid;

use agrum::{
    QueryBook, ReadQueryBook, SqlEntity, SqlQuery, Structured, Transaction, WhereCondition,
};

mod model;
use model::{Address, Company, Contact};

/* ---------------------------------------------------------------------------
 * AddressQueryBook
 * --------------------------------------------------------------------------- */
struct AddressQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for AddressQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.address"
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for AddressQueryBook<T> {}

impl<T: SqlEntity> AddressQueryBook<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn get_all<'a>(&'a self) -> SqlQuery<'a, T> {
        self.select(WhereCondition::default())
    }
}

/* ---------------------------------------------------------------------------
 * CompanyQueryBook
 * --------------------------------------------------------------------------- */
struct CompanyQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for CompanyQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.company"
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for CompanyQueryBook<T> {}

impl<T: SqlEntity> CompanyQueryBook<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn get_from_id<'a>(&self, id: &'a Uuid) -> SqlQuery<'a, T> {
        self.select(WhereCondition::new("company_id = $?", vec![id]))
    }
}

/* ---------------------------------------------------------------------------
 * ContactQueryBook
 * --------------------------------------------------------------------------- */
#[derive(Default)]
struct ContactQueryBook;

impl QueryBook<Contact> for ContactQueryBook {
    fn get_sql_source(&self) -> &'static str {
        "pommr.contact"
    }
}

impl ContactQueryBook {
    pub fn insert<'a>(&self, contact: &'a Contact) -> SqlQuery<'a, Contact> {
        let sql = r#"insert into {:source:} ({:structure:})
  values ($1, $2, $3, $4, $5)
  returning {:projection:}"#;
        let mut query = SqlQuery::new(sql);
        query
            .add_parameter(&contact.contact_id)
            .add_parameter(&contact.name)
            .add_parameter(&contact.email)
            .add_parameter(&contact.phone_number)
            .add_parameter(&contact.company_id)
            .set_variable("source", self.get_sql_source())
            .set_variable(
                "structure",
                &Contact::get_structure().get_names().join(", "),
            );
        query
    }
}

/* ---------------------------------------------------------------------------
 * Test functions
 * --------------------------------------------------------------------------- */

async fn get_pool() -> Pool<PostgresConnectionManager<NoTls>> {
    // Load .env if present; existing env vars override .env values
    let _ = dotenvy::dotenv();
    let pg_dsn = match env::var("PG_DSN").ok().filter(|s| !s.is_empty()) {
        Some(dsn) => dsn,
        None => panic!("PG_DSN is not set (set it in the environment or in a .env file)"),
    };
    let pg_mgr =
        PostgresConnectionManager::new_from_stringlike(pg_dsn, tokio_postgres::NoTls).unwrap();

    Pool::builder().build(pg_mgr).await.unwrap()
}

#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_address_query_book() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
    let query_book = AddressQueryBook::<Address>::new();
    let query = query_book.get_all();
    let results = transaction.query(query).await.unwrap();
    let results = results.collect::<Vec<_>>().await;

    assert_eq!(results.len(), 2);
    let address = results[0].as_ref().unwrap();

    assert_eq!(
        address.address_id,
        Uuid::parse_str("ffb3ef0e-697d-4fba-bc4f-28317dc44626").unwrap()
    );
    assert_eq!(address.label, "FIRST HQ");
    assert_eq!(
        address.company_id,
        Uuid::parse_str("a7b5f2c8-8816-4c40-86bf-64e066a8db7a").unwrap()
    );
    assert_eq!(address.content, "3 rue de la marche");
    assert_eq!(address.zipcode, "57300");
    assert_eq!(address.city, "Mouzillon-Sur-Moselle");
    assert_eq!(
        address.associated_contact_id,
        Some(Uuid::parse_str("529fb920-6df7-4637-8f7f-0878ee140a0f").unwrap())
    );

    let address = results[1].as_ref().unwrap();
    assert_eq!(
        address.address_id,
        Uuid::parse_str("af18dfe3-b189-4d80-bbc9-a90792d92143").unwrap()
    );
    assert_eq!(address.label, "SECOND_HQ");
    assert_eq!(
        address.company_id,
        Uuid::parse_str("dcce1188-66ad-48a1-bb41-756a48514ac4").unwrap()
    );
    assert_eq!(address.content, "1, place du carré vert");
    assert_eq!(address.zipcode, "13820");
    assert_eq!(address.city, "Mingon-En-Provence");
    assert_eq!(
        address.associated_contact_id,
        Some(Uuid::parse_str("99c4996c-b5a7-42bf-af8a-2df326722566").unwrap())
    );
}

#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_condition_company_id() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
    let company_id = Uuid::parse_str("a7b5f2c8-8816-4c40-86bf-64e066a8db7a").unwrap();
    let query = CompanyQueryBook::<Company>::new().get_from_id(&company_id);
    let results = transaction
        .query(query)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;
    assert_eq!(results.len(), 1);

    let company = results[0].as_ref().unwrap();
    assert_eq!(
        company.company_id,
        Uuid::parse_str("a7b5f2c8-8816-4c40-86bf-64e066a8db7a").unwrap()
    );
    assert_eq!(company.name, "first");
    assert_eq!(
        company.default_address_id,
        Uuid::parse_str("ffb3ef0e-697d-4fba-bc4f-28317dc44626").unwrap()
    );
}

#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_insert_contact() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
    let company_id = Uuid::parse_str("a7b5f2c8-8816-4c40-86bf-64e066a8db7a").unwrap();
    let query = CompanyQueryBook::<Company>::new().get_from_id(&company_id);
    let company = transaction
        .query(query)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .pop()
        .transpose()
        .unwrap()
        .unwrap();
    let contact = Contact {
        contact_id: Uuid::new_v4(),
        name: "John Doe".to_string(),
        email: Some("john.doe@example.com".to_string()),
        phone_number: Some("1234567890".to_string()),
        company_id: company.company_id,
    };

    let query = ContactQueryBook {}.insert(&contact);
    assert_eq!(
        query.to_string(),
        r#"insert into pommr.contact (contact_id, name, email, phone_number, company_id)
  values ($1, $2, $3, $4, $5)
  returning contact_id as contact_id, name as name, email as email, phone_number as phone_number, company_id as company_id"#
    );
    let parameters = query.get_parameters();
    assert_eq!(parameters.len(), 5);
    assert_eq!(
        (parameters[0] as &dyn Any).downcast_ref::<Uuid>().unwrap(),
        &contact.contact_id
    );
    assert_eq!(
        (parameters[1] as &dyn Any)
            .downcast_ref::<String>()
            .unwrap(),
        &contact.name
    );
    assert_eq!(
        (parameters[2] as &dyn Any)
            .downcast_ref::<Option<String>>()
            .unwrap(),
        &contact.email
    );
    assert_eq!(
        (parameters[3] as &dyn Any)
            .downcast_ref::<Option<String>>()
            .unwrap(),
        &contact.phone_number
    );
    assert_eq!(
        (parameters[4] as &dyn Any).downcast_ref::<Uuid>().unwrap(),
        &contact.company_id
    );

    let results = transaction
        .query(query)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;
    assert_eq!(results.len(), 1);

    let contact = results[0].as_ref().unwrap();
    assert_eq!(contact.name, "John Doe");
    assert_eq!(contact.email, Some("john.doe@example.com".to_string()));
    assert_eq!(contact.phone_number, Some("1234567890".to_string()));
    assert_eq!(contact.company_id, company.company_id);
    transaction.rollback().await.unwrap();
}
