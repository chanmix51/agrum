use std::{any::Any, marker::PhantomData};

use agrum::{
    Projection, QueryBook, ReadQueryBook, SqlEntity, SqlQuery, Structure, Structured, Transaction,
    WhereCondition,
};
use futures_util::stream::StreamExt;
use uuid::Uuid;

mod model;
use crate::model::Contact;

mod pool;
use pool::get_pool;

pub struct CompanyShort {
    pub company_id: Uuid,
    pub name: String,
    pub contacts_nb: i64,
}

impl Structured for CompanyShort {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("company_id", "uuid"),
            ("name", "text"),
            ("contacts_nb", "integer"),
        ])
    }
}

impl SqlEntity for CompanyShort {
    fn get_projection() -> Projection<CompanyShort> {
        Projection::<CompanyShort>::new("company")
            .set_definition("contacts_nb", "count(contact.company_id)")
    }

    fn hydrate(row: &tokio_postgres::Row) -> Result<Self, agrum::HydrationError> {
        Ok(Self {
            company_id: row.get("company_id"),
            name: row.get::<_, String>("name"),
            contacts_nb: row.get("contacts_nb"),
        })
    }
}

pub struct ContactQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for ContactQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.contact"
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for ContactQueryBook<T> {}

impl<T: SqlEntity> Default for ContactQueryBook<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

struct CompanyShortQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for CompanyShortQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.company"
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for CompanyShortQueryBook<T> {}

impl<T: SqlEntity> CompanyShortQueryBook<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn get_sql_definition(&self) -> &'static str {
        r#"select {:projection:}
  from {:source:} as company
    left join {:contact_source:} as contact
      on company.company_id = contact.company_id
  where {:condition:}
  group by company.company_id"#
    }

    fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
        let contact_source = ContactQueryBook::<Contact>::default().get_sql_source();
        let mut query = SqlQuery::new(self.get_sql_definition());
        let (conditions, parameters) = conditions.expand();
        query
            .set_parameters(parameters)
            .set_variable("projection", &T::get_projection().to_string())
            .set_variable("source", self.get_sql_source())
            .set_variable("contact_source", contact_source)
            .set_variable("condition", &conditions);
        query
    }

    pub fn select_by_id<'a>(&self, id: &'a Uuid) -> SqlQuery<'a, T> {
        self.select(WhereCondition::new("company.company_id = $?", vec![id]))
    }
}

#[tokio::test]
async fn test_select_by_id() {
    let pool = get_pool().await;
    let mut connection = pool.get().await.unwrap();
    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
    let company_id = Uuid::parse_str("a7b5f2c8-8816-4c40-86bf-64e066a8db7a").unwrap();
    let query = CompanyShortQueryBook::<CompanyShort>::new().select_by_id(&company_id);
    assert!(query.get_parameters().len() == 1);
    assert_eq!(
        (query.get_parameters()[0] as &dyn Any)
            .downcast_ref::<Uuid>()
            .unwrap(),
        &company_id
    );
    assert_eq!(
        query.to_string(),
        r#"select company.company_id as company_id, company.name as name, count(contact.company_id) as contacts_nb
  from pommr.company as company
    left join pommr.contact as contact
      on company.company_id = contact.company_id
  where company.company_id = $1
  group by company.company_id"#
    );
    let mut results = transaction
        .query(query)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;
    assert_eq!(results.len(), 1);
    let company_short = results.pop().transpose().unwrap().unwrap();
    assert_eq!(company_short.company_id, company_id);
    assert_eq!(company_short.name, "first");
    assert_eq!(company_short.contacts_nb, 1);
    transaction.rollback().await.unwrap();
}
