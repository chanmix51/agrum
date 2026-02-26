// ---------------------------------------------------------------------------
// Company (pommr.company)
// ---------------------------------------------------------------------------

use agrum::{HydrationError, Projection, SqlEntity, Structure, Structured};
use tokio_postgres::Row;
use uuid::Uuid;

#[allow(unused_variables, dead_code)]
#[derive(Debug)]
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

#[allow(unused_variables, dead_code)]
#[derive(Debug)]
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

#[allow(unused_variables, dead_code)]
#[derive(Debug)]
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
