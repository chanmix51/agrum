mod utils;

use utils::get_client;

use agrum::{
    core::{
        HydrationError, Projection, Provider, SourceAliases, SqlDefinition, SqlEntity,
        SqlQueryWithParameters, Structure, Structured, WhereCondition,
    },
    params,
};
use tokio_postgres::Row;

#[derive(Debug, PartialEq, Clone)]
struct Something {
    something_id: i32,
    thing: String,
}

impl Structured for Something {
    fn get_structure() -> Structure {
        Structure::new(&[("something_id", "int"), ("thing", "text")])
    }
}

impl SqlEntity for Something {
    fn hydrate(row: Row) -> Result<Self, HydrationError>
    where
        Self: Sized,
    {
        let something = Self {
            something_id: row.get("something_id"),
            thing: row.get("thing"),
        };

        Ok(something)
    }
}

#[derive(Debug, Clone)]
struct SomethingQuery {
    projection: Projection<Something>,
    source_aliases: SourceAliases,
}

impl Default for SomethingQuery {
    fn default() -> Self {
        Self {
            projection: Projection::default(),
            source_aliases: SourceAliases::new(&[("something", "something")]),
        }
    }
}

impl SqlDefinition for SomethingQuery {
    fn expand<'a>(&self, params: WhereCondition<'a>) -> SqlQueryWithParameters<'a> {
        let projection = self.projection.expand(&self.source_aliases);
        let (condition, params) = params.expand(&self.source_aliases);
        let sql = format!("select {projection} from (values (1, 'one'), (2, 'two'), (3, 'three')) as something (something_id, thing) where {condition}");

        (sql, params)
    }
}

#[tokio::test]
async fn main() {
    let client = get_client().await;
    let query = SomethingQuery::default();
    assert_eq!(
        "select something_id as something_id, thing as thing from (values (1, 'one'), (2, 'two'), (3, 'three')) as something (something_id, thing) where true".to_string(),
        query.expand(WhereCondition::default()).0
    );
    let provider: Provider<Something> = Provider::new(&client, Box::new(query.clone()));
    let condition = WhereCondition::new("{:something:}.something_id = $?", params![1]);

    let tuples = provider.fetch(condition).await.unwrap();
    let something = tuples
        .first()
        .expect("There should be an entity in the query result.");

    assert_eq!(
        &Something {
            something_id: 1,
            thing: "one".to_string(),
        },
        something
    );
}
