use agrum::{Entity, Projection, Provider, Structure};
use postgres::Client;

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

    fn get_structure() -> Structure {
        let mut structure = Structure::new();
        structure
            .set_field("entity_id", "int")
            .set_field("content", "text")
            .set_field("has_thing", "bool")
            .set_field("something", "int");

        structure
    }
}

struct WhateverProvider<'client> {
    structure: Structure,
    projection: Projection,
    pg_client: &'client Client,
}

impl<'client> WhateverProvider<'client> {
    pub fn new(pg_client: &'client Client) -> Self {
        let structure = WhateverEntity::get_structure();
        let projection = Projection::from_structure(structure.clone(), "whatever");

        Self {
            structure,
            projection,
            pg_client,
        }
    }
}

impl<'client> Provider<'client> for WhateverProvider<'client> {
    type Entity = WhateverEntity;

    fn get_client(&'client self) -> &'client mut Client {
        &mut self.pg_client
    }

    fn get_definition(&self) -> String {
        todo!()
    }

    fn get_projection(&self) -> &Projection {
        todo!()
    }
}
