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
Rust: joining, filtering or sorting data in a performance wise way. Furthermore,
being a declarative languge makes SQL a very solid error prone language: once a
SQL query is marked 'good to go' it normally won't break nor fail (check your
NULLs handling). This means the most you ask the database, the less you have to
implement and test in Rust, leting data intensive operations as close to the
metal as possible. 

The ideas behind Agrum are:
 * mitigate the [Object-Relational impedance mismatch](https://en.wikipedia.org/wiki/Object%E2%80%93relational_impedance_mismatch) by using a Projection led mechanism
 * let developers write the SQL they want
 * have a testable database access layer
 * make queries eventually re-usable for different entities
 * turn the database into a composable set of data sources

 ## What does working with Agrum look like?

For now, Agrum is still under heavy work and is not production ready. Examples
below show the state of the crate as is.

### QueryBooks

 QueryBooks are responsible of building the queries that will be sent to the
 database server.  The main idea behind query books is to manipulate SQL query
 templates where the projection and the conditions are dynamically expanded.
 
 It is a way of making the database layer testable: be able to
 test what query is sent to the server for each particular business demand. 
 It uses the `WhereCondition` internally to ease conditions composition.

 With the service layer shown above, the corresponding QueryBook would be:

 ```rust
#[derive(Default)]
struct CompanyQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for CompanyQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "some_schema.company" // fully qualified name of the table
    }
}

impl<T: SqlEntity> CompanyQueryBook<T> {
   pub fn get_by_id<'a>(&self, company_id: &'a Uuid) -> SqlQuery<'a, T> {
      self.select(WhereCondition::new("company_id = $?", vec![company_id]))
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

Since pretty much of the `SELECT`, `INSERT`, `DELETE` or `UPDATE` sql statements used in
development are simple queries they are available as traits. But the QueryBook patern makes
Agrum able to hold complex queries (see below).

### Conditions

Conditions designate the `where` part of SQL queries. It is handled by the
`WhereCondition` structure that holds the different SQL conditions and their
associated parameters.
 
```rust
let condition = WhereCondition::new("stuff_id=$?", vec![&1_u32])
    .and_where(
        WhereCondition::where_in("substuff_id", vec![&20_u32, &23, &42])
            .or_where(WhereCondition::new("is_alone", vec![]))
        );
// will expand to `stuff_id=$? and (substuff_id in ($?, $?, $?) or is_alone)`
```

### SQL Entities

SQL entities are entities returned by the queries. This means they are tied to a
particular projection. The entities associated with the Query Book presented
above are the following:

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

### Nested SQL entities

Because Postgres used to be an object oriented database, it is possible to nest
entities. In Postgres, defining a table is the same as defining a new type hence
it is possible to create fields with these new types which are just composite
types. For example it is perfectly fine to do this in Postgres'SQL:

```SQL
select
  company.company_id    as company_id,
  company.name          as name,
  address               as default_address -- <-+ address is the type specified by the join below
from company                               --   | it will return the composite type
  join address                             -- <-+
    on address.address_id = company.default_address_id
where company.company_id = '…'::uuid;
```

The output is:

```
             company_id               │ name  │                                                                default_address                                                                 
──────────────────────────────────────┼───────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 a7b5f2c8-8816-4c40-86bf-64e066a8db7a │ first │ (ffb3ef0e-697d-4fba-bc4f-28317dc44626,"FIRST HQ",a7b5f2c8-8816-4c40-86bf-64e066a8db7a,"3 rue de la marche",57300,Mouzillon-Sur-Moselle,529fb92…
                                      │       │…0-6df7-4637-8f7f-0878ee140a0f)
(1 row)
```

in other words, `select my_table from my_table` means: «give me all the records with the
type `my_table`».

Use of the crate `postgres-types` allow to map composite types to Rust structs.
This means we can declare aggregates entities and fetch them directly from the
database. 

```rust
#[derive(Debug, ToSql, FromSql)]
#[postgres(name="address")]
pub struct Address {
    pub address_id: Uuid,
    pub content: String,
    pub zipcode: String,
    pub default_company_id: Uuid
}

pub struct Company {
    pub company_id: Uuid,
    pub name: String,
    pub default_address: Address, // ← Address struct here
}

impl SqlEntity for Company {
    fn get_projection() -> Projection<Company> {
        Projection::<Company>::new("company")               // ← declare an alias for company fields
            .set_definition("default_address", "address")   // ← replace de SQL definition of address
    }

    fn hydrate(row: &tokio_postgres::Row) -> Result<Self, agrum::HydrationError> {
        Ok(Self {
            company_id: row.get("company_id"),
            name: row.get("name"),
            default_address: row.get("default_address"),    // ← postgres-types will do its magic
        })
    }
}

impl Structured for Company {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("company_id", "uuid"),
            ("name", "text"),
            ("default_address", "pommr.address"),
        ])
    }
}

#[derive(Default)]
pub struct CompanyAggregateQueryBook<T: SqlEntity> {
    _phantom: PhantomData<T>,
}

impl<T: SqlEntity> QueryBook<T> for CompanyAggregateQueryBook<T> {
    fn get_sql_source(&self) -> &'static str {
        "some_schema.address"
    }
}

impl<T: SqlEntity> AddressAggregateQueryBook<T> {
    fn get_sql_definition(&self) -> &'static str {
        r#"select {:projection:}
from {:source:} as company
    inner join {:address_source:} as address
        on address.company_id = company.company_id
where {:condition:}"#
    }

    pub fn select<'a>(&self, conditions: WhereCondition<'a>) -> SqlQuery<'a, T> {
        let mut query = SqlQuery::new(self.get_sql_definition());
        let (conditions, parameters) = conditions.expand();
        query
            .set_parameters(parameters)
            .set_variable("projection", &T::get_projection().to_string())
            .set_variable("source", self.get_sql_source())
            .set_variable("address_source", AddressQueryBook::<T>::default().get_sql_source())
            .set_variable("condition", &conditions);
        query
    }
}
```

## Testing queries

The QueryBook patern makes it easy to test the resulting query (or parts of it).

```rust
let query = SomethingQueryBook<Something>::default().getForbiddenCategories();

assert_eq!(Some("category_id in ($?, $?, $?)"), query.get_variables().get("condition"))
assert_eq!(3, query.get_parameters().len())
```