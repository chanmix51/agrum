use agrum::Entity;

struct WhateverEntity {
    entity_id: u32,
    content: String,
    has_thing: bool,
    something: Option<i64>,
}

impl Entity for WhateverEntity {
    fn hydrate(row: postgres::Row) -> Result<Self, agrum::HydrationError>
    where
        Self: Sized,
    {
        Ok(Self {
            entity_id: row.get("entity_id"),
            content: row.get("content"),
            has_thing: row.get("has_thing"),
            something: row.get("something"),
        })
    }
}
