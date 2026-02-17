# Agrum

Agrum is a database access layers designed to make the SQL code maintainable and
testable while letting developpers to focus on the business value of the
queries.

This library is still in early development stage, means it is **not production
ready**. If you are looking for a mature solution, have a look at
[Elephantry](https://elephantry.github.io/)

## What is Agrum?

Relational databases are a very handy way to store data because they handle
correctly atomic concurrent transactions. Furthermore the SQL language proposes
a vast and useful set of features that would be very tiedous to re-implement in
Rust: filtering, sorting, joining. Being a declarative languge makes SQL a very
solid error prone language: once a SQL query is marked 'good to go' it normally
won't break nor fail (check your NULLs handling). This means the most you ask
the database, the less you have to implement and test in Rust making data
intensive operations close to the metal. 

The ideas behind Agrum are:
 * mitigating the [Object-Relational impedance mismatch](https://en.wikipedia.org/wiki/Object%E2%80%93relational_impedance_mismatch) by using a Projection led mechanism
 * make developers able to write the SQL they want
 * have a testable database access layer
 * turning the database into a composable set of data sources
 * make queries eventually re-usable for different entities

 # What does working with Agrum look like?

For now, Agrum is still under heavy work and is not even close to be production
ready. Examples below show the state of the crate as is.

 ## Service layer

 The role of the service layer is to serve controlers' demands while ensuring
 the consistency of the state they are responsible of.

 ```rust
pub async fn get_company_by_id(&self, company_id: &Uuid) -> Result<Option<Company>> {
   let mut connection = self.pool.get()?;
   let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
   let query = CompanyQueryBook::get_by_id(company_id);
   transaction
      .query(query)
      .await?
      .collect::<Vec<_>>()
      .await
      .pop()
      .transpose()
   // transaction is dropped here => rollback
   // connection is dropped here and returns to the pool
}
 ```

 ## QueryBooks

 QueryBooks are responsible of building the query that will be sent to the
 database server. It is a way of making the database layer testable: be able to
 test what query is sent to the server for each particular business demand. 
 It uses the `WhereCondition` internally to hold the query parameters.

 With the service layer shown above, the corresponding QueryBook would be:

 ```rust
struct CompanyQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for CompanyQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "pommr.company" // fully qualified name of the table
    }
}

impl<T: SqlEntity> CompanyQueryBook<T> {
   pub fn new() -> Self {
      Self {
          _phantom: PhantomData,
      }
   }

   pub fn get_by_id<'a>(&self, id: &'a Uuid) -> SqlQuery<'a, T> {
      self.select(WhereCondition::new("company_id = $?", vec![id]))
   }

   // This method could be brung by the `ReadQueryBook` trait, it is implemented
   // here for the sake of showing the internals.
   pub fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T>
      let mut query = SqlQuery::new("select {:projection:} from {:source:} where {:condition:}");
      let (conditions, parameters) = conditions.expand();
      query
         .set_variable("projection", &T::get_projection().expand())
         .set_variable("source", self.get_sql_source())
         .set_variable("condition", &conditions.to_string())
         .set_parameters(parameters);

      query
   }
}
 ```

The main idea behinq query books is to manipulate a SQL query template where the
projection and the conditions are dynamically expanded.

Since pretty much of the `SELECT`, `INSERT` or `UPDATE` sql statements used in development are simple queries they can be implemented as trait but it makes Agrum able to hold complex queries (see below).

## Conditions

Conditions designate the `where` part of SQL queries. It is handled by the
`WhereCondition` structure that holds the different SQL conditions and their
associated parameters.
 
```rust
let condition = WhereCondition::new("stuff_id=$?", vec![&1_u32])
   .and_where(WhereCondition::where_in("substuff_id", vec![&20_u32, &23, &42]).or_where(WhereCondition::new("is_alone", vec![])));
// will expand to `stuff_id=$1 and (substuff_id in ($2, $3, $4) or is_alone)`
```

## SQL Entities

SQL entities are entities returned by the queries. This means they are tied to a particular projection. The entities associated with the Query Book presented above is the following:

```rust
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
```

Ideally, the goal would be something as simple as:

```rust
#[derive(SqlEntity)]
pub struct Company {
    pub company_id: Uuid,
    pub name: String,
    pub default_address_id: Uuid,
}
```