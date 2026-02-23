# What Agrum Tackles

This document explains the main problems the Agrum project addresses and how the design solves them, using examples from `tests/simple.rs`.

---

## 1. Projection-led framework to handle changes

**Problem:** When the schema or the shape of data changes, you want one place to define “what goes in and what comes out” of a query, so that SELECT lists, RETURNING clauses, and hydration stay in sync without scattering column names and types everywhere.

**Approach:** The framework is **projection-led**: each entity defines a **Structure** (column names and SQL types) and a **Projection** (what is actually selected and under which names). The projection is derived from the structure by default but can be overridden per query. All query building (SELECT, INSERT … RETURNING) uses these definitions, so schema changes are reflected by updating the entity once.

### Structure and default projection

Each entity implements `Structured` and `SqlEntity`. The structure is the single source of truth for column names and types:

```rust
// From tests/model.rs
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
```

The default projection is “each structure column as itself”, so it stays aligned with the structure. Queries are built from it (e.g. `{:projection:}` in SQL).

### One template for both column list and RETURNING

Inserts use the **structure** for the column list and the **projection** for what comes back, so the same entity definition drives both:

```78:89:tests/simple.rs
    pub fn insert<'a>(&self, contact: &'a Contact) -> SqlQuery<'a, Contact> {
        let sql = r#"insert into {:source:} ({:structure:})
  values ($1, $2, $3, $4, $5)
  returning {:projection:}"#;
        let mut query = SqlQuery::new(sql);
        query.add_parameter(&contact.contact_id)
            .add_parameter(&contact.name)
            // ...
            .set_variable("source", &self.get_sql_source())
            .set_variable("structure", &Contact::get_structure().get_names().join(", "));
        query
    }
```

- **`{:structure:}`** → column names for the INSERT (from `get_structure().get_names()`).
- **`{:projection:}`** → RETURNING clause (from `Contact::get_projection().expand()`), so the returned row matches what `hydrate` expects.

When you add or rename a column, you change the entity’s structure (and optionally its projection); the insert template and hydration stay correct.

### Customising the projection per query

Projections can be overridden for specific queries (e.g. computed or renamed columns) via `set_definition`, while still staying tied to the entity’s shape. So “projection-led” means: the default shape comes from the structure, and any variation is explicit in the projection, making schema evolution and query variants manageable in one place.

---

## 2. Testability of the queries

**Problem:** You want to verify that the right SQL and parameters are built without hitting the database, so tests are fast, deterministic, and not tied to DB state.

**Approach:** Queries are first-class values. You can inspect the final SQL and all parameters before execution. Tests can assert on the exact string and on each parameter.

### Asserting on the full SQL

After building a query, you can render it to a string and assert on it:

```173:175:tests/simple.rs
    let query = ContactQueryBook::default().insert(&contact);
    assert_eq!(query.to_string(),r#"insert into pommr.contact (contact_id, name, email, phone_number, company_id)
  values ($1, $2, $3, $4, $5)
  returning contact_id as contact_id, name as name, email as email, phone_number as phone_number, company_id as company_id"#);
```

So you lock the expected SQL (including projection and structure) in a test without running it.

### Asserting on parameters

Parameters are accessible for inspection. The test checks count and values (including types) via downcast:

```176:182:tests/simple.rs
    let parameters = query.get_parameters();
    assert_eq!(parameters.len(), 5);
    assert_eq!((parameters[0] as &dyn Any).downcast_ref::<Uuid>().unwrap(), &contact.contact_id);
    assert_eq!((parameters[1] as &dyn Any).downcast_ref::<String>().unwrap(), &contact.name);
    assert_eq!((parameters[2] as &dyn Any).downcast_ref::<Option<String>>().unwrap(), &contact.email);
    // ...
```

So query building (SQL + parameters) is fully testable without a database. Integration tests can then run the same queries against a real DB when needed.

---

## 3. Easy way to fetch structures from queries

**Problem:** You want to get typed structs (e.g. `Address`, `Company`, `Contact`) from the database with minimal boilerplate: no hand-written column lists or manual row mapping scattered across the codebase.

**Approach:** Each entity defines its **structure**, **projection**, and **hydration**. Query books build queries from that definition; the transaction runs the query and returns a stream of hydrated entities. Call sites just work with Rust structs.

### Defining the entity once

The entity ties together structure, projection, and row → struct mapping:

```42:83:tests/model.rs
pub struct Address {
    pub address_id: Uuid,
    pub label: String,
    pub company_id: Uuid,
    // ...
}

impl SqlEntity for Address {
    fn get_projection() -> Projection<Address> {
        Projection::default()
    }

    fn hydrate(row: &Row) -> Result<Self, HydrationError> {
        Ok(Self {
            address_id: row.get("address_id"),
            label: row.get("label"),
            // ...
        })
    }
}

impl Structured for Address {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("address_id", "uuid"),
            ("label", "text"),
            // ...
        ])
    }
}
```

So “fetch structures” means: the shape of the result is defined once on the entity; queries and hydration both use it.

### Query books and execution

Query books use the entity’s projection and source to build SELECTs (or other operations). The transaction runs the query and returns a stream of hydrated entities:

```111:119:tests/simple.rs
async fn test_address_query_book(pool: &Pool<PostgresConnectionManager<NoTls>>) {
    let mut connection = pool.get().await.unwrap();

    let transaction = Transaction::start(connection.transaction().await.unwrap()).await;
    let query_book = AddressQueryBook::<Address>::new();
    let query = query_book.get_all();
    let results = transaction.query(query).await.unwrap();
    let results = results.collect::<Vec<_>>().await;
```

Results are already `Result<Address>`; the caller just uses the struct:

```122:129:tests/simple.rs
    assert_eq!(results.len(), 2);
    let address = results[0].as_ref().unwrap();

    assert_eq!(address.address_id, Uuid::parse_str("ffb3ef0e-697d-4fba-bc4f-28317dc44626").unwrap());
    assert_eq!(address.label, "FIRST HQ");
    assert_eq!(address.company_id, Uuid::parse_str("a7b5f2c8-8816-4c40-86bf-64e066a8db7a").unwrap());
    // ...
```

The same pattern works for conditional queries (e.g. by `company_id`) and for INSERT … RETURNING: you get back a stream of the same entity type. So “easy way to fetch structures” means: define the entity once, then use query books and `transaction.query(...)` to get typed structs from the database without repeating column names or mapping logic at the call site.

---

## Summary

| Problem | Approach |
|--------|----------|
| **Projection-led handling of changes** | Structure + Projection per entity; INSERT uses `{:structure:}` and `{:projection:}` so schema changes are done in one place and RETURNING matches hydration. |
| **Testability of queries** | Queries expose `to_string()` and `get_parameters()`, so tests can assert on SQL and parameters without a database. |
| **Easy fetch of structures** | Entity defines structure, projection, and `hydrate`; query books build from that; `transaction.query()` returns a stream of typed entities. |

The examples in `tests/simple.rs` (and the entity definitions in `tests/model.rs`) are the main reference for these three ideas.
