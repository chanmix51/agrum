use std::{error::Error};

use tokio_postgres::{Client, Row};

use crate::core::{SqlEntity, Structured, Structure, SqlDefinition, Provider, WhereCondition};

use super::*;

pub struct DatabaseInfo {
    pub name: String,
    pub owner: String,
    pub encoding: String,
    pub size: String,
    pub description: String,
}

impl SqlEntity for DatabaseInfo {
    fn hydrate(row: Row) -> Result<Self, core::HydrationError>
        where
            Self: Sized {
        let entity = Self {
            name: row.get("name"),
            owner: row.get("owner"),
            encoding: row.get("encoding"),
            size: row.get("size"),
            description: row.get("description"),
        };

        Ok(entity)
    }
}

impl Structured for DatabaseInfo {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("name", "text"),
            ("owner", "text"),
            ("encoding", "text"),
            ("size", "text"),
            ("description", "text"),
        ])
    }
}


#[derive(Default)]
struct DatabaseInfoDefinition;

impl SqlDefinition for DatabaseInfoDefinition {
    fn expand(&self, condition: &str) -> String {
        format!(r#"
select
  db.datname as name,
  pg_catalog.pg_get_userbyid(db.datdba) as owner,
  pg_catalog.pg_encoding_to_char(db.encoding) as encoding,
  case
    when pg_catalog.has_database_privilege(db.datname, 'CONNECT')
      then pg_catalog.pg_size_pretty(pg_catalog.pg_database_size(db.datname))
    else 'No Access'
  end as size,
  pg_catalog.shobj_description(db.oid, 'pg_database') as description
from pg_catalog.pg_database as db
where {condition}
order by 1;"#)
    }
 }

#[derive(Debug)]
pub struct SchemaInfo {
    pub name: String,
    pub relations: i64,
    pub owner: String,
    pub description: Option<String>,
}

impl SqlEntity for SchemaInfo {
    fn hydrate(row: Row) -> Result<Self, core::HydrationError>
        where
            Self: Sized {
        let schema_info = Self {
            name: row.get("name"),
            relations: row.get("relations"),
            owner: row.get("owner"),
            description: row.get("description"),
        };

        Ok(schema_info)
    }
}

impl Structured for SchemaInfo {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("name", "text"),
            ("relations", "int"),
            ("owner", "text"),
            ("description", "text")
        ])
    }
}

#[derive(Default)]
struct SchemaInfoDefinition;

impl SqlDefinition for SchemaInfoDefinition {
    fn expand(&self, condition: &str) -> String {
        format!(r#"
select
  n.nspname     as "name",
  count(c)      as "relations",
  o.rolname     as "owner",
  d.description as "description"
from pg_catalog.pg_namespace n
  left join pg_catalog.pg_description d
    on n.oid = d.objoid
  left join pg_catalog.pg_class c on
    c.relnamespace = n.oid and c.relkind in ('r', 'v')
  join pg_catalog.pg_roles o
    on n.nspowner = o.oid
where {condition}
group by 1, 3, 4
order by 1 asc;"#)
    }
}

pub struct Inspector<'client> {
    client: &'client Client,
}

impl<'client> Inspector<'client> {
    pub fn new(client: &'client Client) -> Self {
        Self { client }
    }

    fn get_dbinfo_provider(&self) -> Provider<DatabaseInfo> {
        Provider::new(
            &self.client,
            Box::new(DatabaseInfoDefinition::default())
            )
    }

    pub async fn get_database_list(&self) -> Result<Vec<DatabaseInfo>, Box<dyn Error>> {
        self.get_dbinfo_provider()
            .find(WhereCondition::default())
            .await
    }

    pub async fn get_db_info(&self, name: &str) -> Result<Option<DatabaseInfo>, Box<dyn Error>> {
        let condition = WhereCondition::new("datname = $?", params![name]);
        let rows = self.get_dbinfo_provider()
            .find(condition)
            .await?;

        Ok(rows.into_iter().next())
    }

    pub async fn get_schema_list(&self) -> Result<Vec<SchemaInfo>, Box<dyn Error>> {
        let condition = WhereCondition::new("n.nspname !~ $?", params!["^pg_"])
            .and_where(WhereCondition::new("n.nspname != $?", params!["information_schema"]));

        self.get_all_schemas(condition).await
    }

    pub async fn get_all_schemas(&self, condition: WhereCondition<'_>) -> Result<Vec<SchemaInfo>, Box<dyn Error>> {
        self.get_schema_provider()
            .find(condition)
            .await
    }

    fn get_schema_provider(&self) -> Provider<SchemaInfo> {
        Provider::new(
            &self.client,
            Box::new(SchemaInfoDefinition::default())
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_database_list() {
        let definition = DatabaseInfoDefinition::default();
        let query = r#"
select
  db.datname as name,
  pg_catalog.pg_get_userbyid(db.datdba) as owner,
  pg_catalog.pg_encoding_to_char(db.encoding) as encoding,
  case
    when pg_catalog.has_database_privilege(db.datname, 'CONNECT')
      then pg_catalog.pg_size_pretty(pg_catalog.pg_database_size(db.datname))
    else 'No Access'
  end as size,
  pg_catalog.shobj_description(db.oid, 'pg_database') as description
from pg_catalog.pg_database as db
where CONDITION
order by 1;"#;

        assert_eq!(
            query,
            definition.expand("CONDITION")
            );
    }
}
