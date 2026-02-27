#![allow(unused_variables, dead_code)]
// ---------------------------------------------------------------------------
// Company (pommr.company)
// ---------------------------------------------------------------------------

use std::marker::PhantomData;

use agrum::{
    DeleteQueryBook, HydrationError, InsertQueryBook, Projection, QueryBook, ReadQueryBook,
    SqlEntity, SqlQuery, Structure, Structured, UpdateQueryBook, WhereCondition,
};
use postgres_types::{FromSql, ToSql};
use tokio_postgres::Row;
use uuid::Uuid;

pub const COMPANY_1_ID: &str = "a7b5f2c8-8816-4c40-86bf-64e066a8db7a";
pub const COMPANY_2_ID: &str = "dcce1188-66ad-48a1-bb41-756a48514ac4";
pub const ADDRESS_1_ID: &str = "ffb3ef0e-697d-4fba-bc4f-28317dc44626";
pub const ADDRESS_2_ID: &str = "af18dfe3-b189-4d80-bbc9-a90792d92143";
pub const CONTACT_1_ID: &str = "529fb920-6df7-4637-8f7f-0878ee140a0f";
pub const CONTACT_2_ID: &str = "99c4996c-b5a7-42bf-af8a-2df326722566";

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "company")]
pub struct Company {
    pub company_id: Uuid,
    pub name: String,
    pub default_address_id: Uuid,
}

impl SqlEntity for Company {
    fn get_projection() -> Projection<Company> {
        Projection::default()
    }

    fn hydrate(row: &Row) -> Result<Self, HydrationError> {
        Ok(Self {
            company_id: row.get("company_id"),
            name: row.get("name"),
            default_address_id: row.get("default_address_id"),
        })
    }
}

impl Structured for Company {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("company_id", "uuid"),
            ("name", "text"),
            ("default_address_id", "uuid"),
        ])
    }
}

// ---------------------------------------------------------------------------
// Address (pommr.address)
// ---------------------------------------------------------------------------

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "address")]
pub struct Address {
    pub address_id: Uuid,
    pub label: String,
    pub company_id: Uuid,
    pub content: String,
    pub zipcode: String,
    pub city: String,
    pub associated_contact_id: Option<Uuid>,
}

impl SqlEntity for Address {
    fn get_projection() -> Projection<Address> {
        Projection::default()
    }

    fn hydrate(row: &Row) -> Result<Self, HydrationError> {
        Ok(Self {
            address_id: row.get("address_id"),
            label: row.get("label"),
            company_id: row.get("company_id"),
            content: row.get("content"),
            zipcode: row.get("zipcode"),
            city: row.get("city"),
            associated_contact_id: row.get("associated_contact_id"),
        })
    }
}

impl Structured for Address {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("address_id", "uuid"),
            ("label", "text"),
            ("company_id", "uuid"),
            ("content", "text"),
            ("zipcode", "text"),
            ("city", "text"),
            ("associated_contact_id", "uuid"),
        ])
    }
}

// ---------------------------------------------------------------------------
// Contact (pommr.contact)
// ---------------------------------------------------------------------------

#[derive(Debug, FromSql, ToSql)]
#[postgres(name = "contact")]
pub struct Contact {
    pub contact_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub company_id: Uuid,
}

impl SqlEntity for Contact {
    fn get_projection() -> Projection<Contact> {
        Projection::default()
    }

    fn hydrate(row: &Row) -> Result<Self, HydrationError> {
        Ok(Self {
            contact_id: row.get("contact_id"),
            name: row.get("name"),
            email: row.get("email"),
            phone_number: row.get("phone_number"),
            company_id: row.get("company_id"),
        })
    }
}

impl Structured for Contact {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("contact_id", "uuid"),
            ("name", "text"),
            ("email", "text"),
            ("phone_number", "text"),
            ("company_id", "uuid"),
        ])
    }
}

/* ---------------------------------------------------------------------------
 * AddressQueryBook
 * --------------------------------------------------------------------------- */
pub struct AddressQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for AddressQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.address"
    }
}

impl<T: SqlEntity> Default for AddressQueryBook<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: SqlEntity> AddressQueryBook<T> {
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
pub struct CompanyQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for CompanyQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.company"
    }
}

impl<T: SqlEntity> Default for CompanyQueryBook<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: SqlEntity> CompanyQueryBook<T> {
    pub fn get_from_id<'a>(&self, id: &'a Uuid) -> SqlQuery<'a, T> {
        self.select(WhereCondition::new("company_id = $?", vec![id]))
    }
}

impl<T: SqlEntity> ReadQueryBook<T> for CompanyQueryBook<T> {}
impl<T: SqlEntity> UpdateQueryBook<T> for CompanyQueryBook<T> {}
impl<T: SqlEntity> InsertQueryBook<T> for CompanyQueryBook<T> {}

/* ---------------------------------------------------------------------------
 * ContactQueryBook
 * --------------------------------------------------------------------------- */
pub struct ContactQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> Default for ContactQueryBook<T> {
    fn default() -> Self {
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
