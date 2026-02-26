use std::{collections::HashMap, marker::PhantomData};

use futures_util::stream::StreamExt;
use uuid::Uuid;

use agrum::{
    DeleteQueryBook, InsertQueryBook, QueryBook, ReadQueryBook, SqlEntity, SqlQuery, ToSqlAny,
    Transaction, UpdateQueryBook, WhereCondition,
};

mod model;
use model::{Address, Company, Contact};

mod pool;
use pool::get_pool;

const COMPANY_1_ID: &str = "a7b5f2c8-8816-4c40-86bf-64e066a8db7a";
const COMPANY_2_ID: &str = "dcce1188-66ad-48a1-bb41-756a48514ac4";
const ADDRESS_1_ID: &str = "ffb3ef0e-697d-4fba-bc4f-28317dc44626";
const ADDRESS_2_ID: &str = "af18dfe3-b189-4d80-bbc9-a90792d92143";
const CONTACT_1_ID: &str = "529fb920-6df7-4637-8f7f-0878ee140a0f";
const CONTACT_2_ID: &str = "99c4996c-b5a7-42bf-af8a-2df326722566";

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

impl<T: SqlEntity> AddressQueryBook<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn get_all<'a>(&self) -> SqlQuery<'a, T> {
        self.select(WhereCondition::default())
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for AddressQueryBook<T> {}
impl<T: SqlEntity> InsertQueryBook<T> for AddressQueryBook<T> {}
impl<T: SqlEntity> UpdateQueryBook<T> for AddressQueryBook<T> {}

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

impl<T: SqlEntity> UpdateQueryBook<T> for CompanyQueryBook<T> {}
impl<T: SqlEntity> InsertQueryBook<T> for CompanyQueryBook<T> {}

/* ---------------------------------------------------------------------------
 * ContactQueryBook
 * --------------------------------------------------------------------------- */
#[derive(Default)]
struct ContactQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}
impl<T: SqlEntity> ContactQueryBook<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: SqlEntity> QueryBook<T> for ContactQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.contact"
    }
}

impl<T: SqlEntity> InsertQueryBook<T> for ContactQueryBook<T> {}
impl<T: SqlEntity> DeleteQueryBook<T> for ContactQueryBook<T> {}
impl<T: SqlEntity> UpdateQueryBook<T> for ContactQueryBook<T> {}

/* ---------------------------------------------------------------------------
 * Test functions
 * --------------------------------------------------------------------------- */

#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_address_query_book() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;

    let query: SqlQuery<'_, Address> = AddressQueryBook::<Address>::new().get_all();
    let results = transaction
        .query(query)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;

    assert_eq!(results.len(), 2);
    let address = results[0].as_ref().unwrap();
    assert_eq!(address.address_id, Uuid::parse_str(ADDRESS_1_ID).unwrap());
    assert_eq!(address.label, "FIRST HQ");
    assert_eq!(address.company_id, Uuid::parse_str(COMPANY_1_ID).unwrap());
    assert_eq!(address.content, "3 rue de la marche");
    assert_eq!(address.zipcode, "57300");
    assert_eq!(address.city, "Mouzillon-Sur-Moselle");
    assert_eq!(
        address.associated_contact_id,
        Some(Uuid::parse_str(CONTACT_1_ID).unwrap())
    );

    let address = results[1].as_ref().unwrap();
    assert_eq!(address.address_id, Uuid::parse_str(ADDRESS_2_ID).unwrap());
    assert_eq!(address.label, "SECOND_HQ");
    assert_eq!(address.company_id, Uuid::parse_str(COMPANY_2_ID).unwrap());
    assert_eq!(address.content, "1, place du carré vert");
    assert_eq!(address.zipcode, "13820");
    assert_eq!(address.city, "Mingon-En-Provence");
    assert_eq!(
        address.associated_contact_id,
        Some(Uuid::parse_str(CONTACT_2_ID).unwrap())
    );
    transaction.rollback().await.unwrap();
}

#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_condition_company_id() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
    let company_id = Uuid::parse_str(COMPANY_1_ID).unwrap();
    let query = CompanyQueryBook::<Company>::new().get_from_id(&company_id);
    let company = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(company.company_id, Uuid::parse_str(COMPANY_1_ID).unwrap());
    assert_eq!(company.name, "first");
    assert_eq!(
        company.default_address_id,
        Uuid::parse_str(ADDRESS_1_ID).unwrap()
    );
    transaction.rollback().await.unwrap();
}

// The following test creates a company then an address, and sets the address as
// the default address of the company. It updates the company to set the
// address as the default address. It is possible since the company has a
// deferrable foreign key to the address.
#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_scenario_create_company() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;

    let company_query_book = CompanyQueryBook::<Company>::new();
    let default_address_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
    let query = company_query_book.insert(HashMap::from([
        ("name", &"test_name" as &dyn ToSqlAny),
        ("default_address_id", &default_address_id),
    ]));
    let company = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();

    let address_query_book = AddressQueryBook::<Address>::new();
    let query = address_query_book.insert(HashMap::from([
        ("label", &"test_label" as &dyn ToSqlAny),
        ("content", &"test_content"),
        ("zipcode", &"test_zipcode"),
        ("city", &"test_city"),
        ("company_id", &company.company_id),
    ]));
    let address = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();

    let query = company_query_book.update(
        HashMap::from([("default_address_id", &address.address_id as &dyn ToSqlAny)]),
        WhereCondition::new("company_id = $?", vec![&company.company_id]),
    );
    let company = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(company.default_address_id, address.address_id);

    // ↓ uncomment to commit the transaction and see the changes in the database
    //transaction.commit().await.unwrap();
    transaction.rollback().await.unwrap();
}

// This scenario creates a new contact, modify an adress to set this new contact
// as the default contact of the address, then modify the contact to set the
// address as the default address of the company. It is possible since the
// contact has a foreign key to the address. and then try do delete the contact.
// The transaction should fail because the contact is referenced by the address.
#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_scenario_create_contact() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;

    let company_id = Uuid::parse_str(COMPANY_1_ID).unwrap();
    let address_id = Uuid::parse_str(ADDRESS_1_ID).unwrap();

    let contact_query_book = ContactQueryBook::<Contact>::new();
    let query = contact_query_book.insert(HashMap::from([
        ("name", &"test_name" as &dyn ToSqlAny),
        ("email", &"test_email"),
        ("phone_number", &"test_phone_number"),
        ("company_id", &company_id),
    ]));
    let contact = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();

    let query = AddressQueryBook::<Address>::new().update(
        HashMap::from([(
            "associated_contact_id",
            &contact.contact_id as &dyn ToSqlAny,
        )]),
        WhereCondition::new("address_id = $?", vec![&address_id as &dyn ToSqlAny]),
    );
    let address = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(address.associated_contact_id, Some(contact.contact_id));

    let query = contact_query_book.delete(WhereCondition::new(
        "contact_id = $?",
        vec![&contact.contact_id],
    ));
    let _result = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap_err();
    transaction.rollback().await.unwrap();
}
