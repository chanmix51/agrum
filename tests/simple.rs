use agrum::{Entity, Projection, Structure};

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
    fn make_projection(&self) -> Projection {
        let mut structure = Structure::new();
        structure
            .set_field("entity_id", "int")
            .set_field("content", "text")
            .set_field("has_thing", "bool")
            .set_field("something", "int");

        Projection::from_structure(structure, "whatever")
    }
}
