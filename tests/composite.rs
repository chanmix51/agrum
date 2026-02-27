use std::marker::PhantomData;

use agrum::{
    Projection, QueryBook, SqlEntity, SqlQuery, Structure, Structured, Transaction, WhereCondition,
};
use futures_util::stream::StreamExt;
use uuid::Uuid;

mod model;
use model::*;

mod pool;
use pool::get_pool;

const ADDRESS_1_ID: &str = "ffb3ef0e-697d-4fba-bc4f-28317dc44626";

pub struct AddressAggregateEntity {
    pub address_id: Uuid,
    pub label: String,
    pub company: Company,
    pub content: String,
    pub zipcode: String,
    pub city: String,
    pub contact: Option<Contact>,
}

impl SqlEntity for AddressAggregateEntity {
    fn get_projection() -> Projection<AddressAggregateEntity> {
        Projection::<AddressAggregateEntity>::new("address")
            .set_definition("company", "company")
            .set_definition("contact", "contact")
    }

    fn hydrate(row: &tokio_postgres::Row) -> Result<Self, agrum::HydrationError> {
        Ok(Self {
            address_id: row.get("address_id"),
            label: row.get("label"),
            company: row.get("company"),
            content: row.get("content"),
            zipcode: row.get("zipcode"),
            city: row.get("city"),
            contact: row.get("contact"),
        })
    }
}

impl Structured for AddressAggregateEntity {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("address_id", "uuid"),
            ("label", "text"),
            ("company", "pommr.company"),
            ("content", "text"),
            ("zipcode", "text"),
            ("city", "text"),
            ("contact", "pommr.contact"),
        ])
    }
}

pub struct AddressAggregateQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for AddressAggregateQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.address"
    }
}

impl<T: SqlEntity> Default for AddressAggregateQueryBook<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: SqlEntity> AddressAggregateQueryBook<T> {
    fn get_sql_definition(&self) -> &'static str {
        r#"select {:projection:}
from {:source:} as address
    inner join {:company_source:} as company
        on address.company_id = company.company_id
    left outer join {:contact_source:} as contact
        on address.associated_contact_id = contact.contact_id
where {:condition:}"#
    }

    pub fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
        let mut query = SqlQuery::new(self.get_sql_definition());
        let (conditions, parameters) = conditions.expand();
        query
            .set_parameters(parameters)
            .set_variable("projection", &T::get_projection().to_string())
            .set_variable("source", self.get_sql_source())
            .set_variable(
                "company_source",
                CompanyQueryBook::<T>::default().get_sql_source(),
            )
            .set_variable(
                "contact_source",
                ContactQueryBook::<T>::default().get_sql_source(),
            )
            .set_variable("condition", &conditions);
        query
    }
}

#[tokio::test]
#[ignore = "skipping database tests"]
async fn test_address_aggregate_query_book() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;

    let company_id = Uuid::parse_str(COMPANY_1_ID).unwrap();
    let query = AddressAggregateQueryBook::<AddressAggregateEntity>::default().select(
        WhereCondition::new("address.company_id = $?", vec![&company_id]),
    );
    let address: AddressAggregateEntity = transaction
        .query(query)
        .await
        .unwrap()
        .next()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(address.address_id, Uuid::parse_str(ADDRESS_1_ID).unwrap());
    assert_eq!(address.label, "FIRST HQ");
    assert_eq!(
        address.company.company_id,
        Uuid::parse_str(COMPANY_1_ID).unwrap()
    );
    assert_eq!(
        address.company.default_address_id,
        Uuid::parse_str(ADDRESS_1_ID).unwrap()
    );
    assert_eq!(
        address.contact.as_ref().unwrap().contact_id,
        Uuid::parse_str(CONTACT_1_ID).unwrap()
    );
}
