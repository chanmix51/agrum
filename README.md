# Agrum

Agrum is a crate designed to make the SQL code maintainable and testable while letting developpers to focus on the business value of the queries.

This library is still in early development stage, means it is **not production ready**.

## What is Agrum?

The ideas behind Agrum are:
- mitigating the [Object-Relational impedance mismatch](https://en.wikipedia.org/wiki/Object%E2%80%93relational_impedance_mismatch) by using a Projection led mechanism
- turning the database into a composable set of data sources

### Projection led framework

A SQL query (mostly `SELECT`) is a declarative chain of transformations. Projection is done by explicitely stating how output values are mapped. If the code allows to tune the SQL projection it maps how entities are hydrated from SQL queries.

```rust
let sql = #"
select
  customer_id,
  name,
  age(datebirth) as age
from customer
where customer_id = *?"#;

let results = client.query(sql, &[customer_id])
```

The code above is fragile since a change either in the database or the application model would require all the queries to be modified.

```rust
let projection = get_projection();
let sql = #"select {projection} from customer where customer_id = *?"#;
```

The code above delegates the projection management to a `get_projection` function. It sets in a unique place the projection definition for all queries on the `customer` table. It is testable. It makes SQL queries easier to write.

### Modular data sources

Most of the time the source of SQL query data are tables but it can also be views, functions, sub queries, fixed sets (VALUES). In a way, data source is always a SET of data, a table being a persistent SET. Since a query can be used as a data source (being a programmable set), it is possible to chain data transformations by chaining queries exactly the same way it is done in a SQL `WITH` statement. Agrum aims at leveraging this behavior to use SQL as a mapping layer between data collection (tables) and the Rust code.

## State of development

As June 2025, Agrum is in early stage. The projection mechanism works but it still needs to be hardened on real world situations. The usability is still poor, it is very verbose as anything needs to be defined by hand, an annotation mechanism would ease a lot that part and is still to be done.

## How to work with Agrum?

Agrum organizes the database code in 3 layers:

 * a business oriented layer that defines what queries need to be performed and make easy to define and test conditions and parameters (clear code easy to modify)
 * a query layer that defines SQL queries which is separated into projections and definitions that can be unit tested (not often modified)
 * a database layer that hydrates Rust structures from SQL results (low level, isolated)

Determine what SQL query you want to issue using your favorite SQL client (not to say `psql`). Once you know exactly the SQL query you need, put it as a test for the `SqlDefinition` you will create. This SQL definition will be split in two responsibilities:
 * the projection (the `SELECT` part) that is required to build the Rust structure that will hold the data
 * the conditions (the `WHERE` part) that can vary using the same SQL query to represent different data contexts

The same query can be used to hydrate several kinds of Rust structures. The same query for the same structure can be used with different set of conditions.

As an example, we can imagine the list of objects for sale. There are 3 different views of those objects: private view with full details and price, public with less details and small for compact lists. The query remains the same: get the list of objects that are still for sale and on display but the projection of the query changes.

### Writing a data book (data repository)

```rust
struct WhateverEntityDataBook<'client> {
    provider: Provider<'client, WhateverEntity>,
}

impl<'client> WhateverEntityDataBook<'client> {
    pub fn new(provider: Provider<'client, WhateverEntity>) -> Self {
        Self { provider }
    }

    pub async fn fetch_all(&self) -> Result<Vec<WhateverEntity>, Box<dyn Error + Sync + Send>> {
        self.provider.fetch(WhereCondition::default()).await
    }

    pub async fn fetch_by_id(
        &self,
        thing_id: i32,
    ) -> Result<Option<WhateverEntity>, Box<dyn Error + Sync + Send>> {
        let condition = WhereCondition::new("thing_id = $?", params![thing_id]);
        let entity = self.provider.fetch(condition).await?.pop();

        Ok(entity)
    }
}
```

### Writing a SQL definition 


