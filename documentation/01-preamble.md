# Agrum

Efficiently using SQL from Rust in a maintainable manner.

## Using SQL in Rust

Mixing SQL in imperative code language has always been difficult some even call this an “impedance mismatch”. Agrum pretends to be an alternative approach about this problem.

Let's travel through this problem in the shoes of Julia who is writing an application to allow her to find the nearest bike stations when she's out in the city of Nantes. She downloads the data from the official operator website and opens a connection to Postgres:

```sql
create table bike_station (
    bike_station_id int     not null primary key,
    designation     text    not null unique,
    coords          point   not null,
    has_bank        bool    not null
);
```
She knows that more data will come later but this would be the smallest yet valuable sample of application she thought. She opens her favorite code environment and in a brand new Rust project, she starts her POC:

```rust
use tokio_postgres::Row;

#[derive(Debug)]
struct BikeStation {
    bike_station_id: i32,
    designation: String,
    coords: geo_types::Point<f64>,
    has_bank: bool,
}

impl BikeStation {
    pub fn hydrate(row: &Row) -> Self {
        Self {
            bike_station_id: row.get("bike_station_id"),
            designation: row.get("designation"),
            coords: row.get("coords"),
            has_bank: row.get("has_bank"),
        }
    }
}
```

Now she has got a `BikeStation` entity with a method that will spawn instances from SQL results. The service definition is also pretty straightforward:

```rust
use tokio_postgres::Client;

use std::error::Error;

struct BikeStationFinderService<'client> {
    db_client: &'client Client,
}

impl<'client> BikeStationFinderService<'client> {
    pub fn new(db_client: &'client Client) -> Self {
        Self { db_client }
    }

    pub async fn find_bike_station_id(
        &self,
        bike_station_id: u32,
    ) -> Result<Option<BikeStation>, Box<dyn Error>> {
        let sql = "select * from bike_station where bike_station_id = $1";
        let rows = self.db_client.query(sql, &[&bike_station_id]).await?;
        let maybe_entity = rows.first().map(BikeStation::hydrate);

        Ok(maybe_entity)
    }
}
```

She is always a bit uncomfortable when she has to use lifetimes but in this case, she has to store a borrow of the database connection in the service in order to let other services do queries without having to open a connection every single time. She just tells the compiler the service lifetime must overlap client's lifetime completely and things go without much hassle. She runs this code which works. It is always noticeable when more than 20 lines of code works at first run. It is late, Julia yawns and stars at the code, liking how elegantly Rust allows to express what to be done in a safe and consise way. She then shuts down here computer with the feeling of achieving a good day.

## Database coupling

When she wakes up, Julia has mixed thoughts about pieces of Rust code and SQL. As she sips her tea, she tries to figure out what puzzle her. She wakes up her computer and the code she found nice the night before appeared less handsome. Her first concerns are the coupling the SQL query she wrote induces between the Rust code and the underneath SQL structure. Even though the `select *` looked very handy at first glance, adapting to any table structure change, it now appears as a golden handcuff which locks the `BikeStation` entity representation to the `bike_station` table. This means that instead of representing a structure that serves business purposes, it is a technical representation of a piece of the database. This little star cuts her off from one of the most powerful feature of SQL: the projection.

```rust
    pub async fn find_bike_station_id(
        &self,
        bike_station_id: u32,
    ) -> Result<Option<BikeStation>, Box<dyn Error>> {
        let sql = "select bike_station_id, designation, coords, has_bank from bike_station where bike_station_id = $1";
        let rows = self.db_client.query(sql, &[&bike_station_id]).await?;
        let maybe_entity = rows.first().map(BikeStation::hydrate);

        Ok(maybe_entity)
    }
```

The projection is the definition of the fields output by a query. This definition involves the comprehensive list of the fields and how they are computed : this is the `SELECT` part of the query.

Now Julia has this in mind, she recalls thait the aim of her project is to give her a list of bike stations near of where she is. So she adds a new property to `BikeStation` entity: `distance_m` to display the distance in meters. In some cases, for instance when querying stations by id, this data is not relevant and shall be set to null, so she chooses to set this property as an option:

```rust
#[derive(Debug)]
struct BikeStation {
    bike_station_id: i32,
    designation: String,
    coords: geo_types::Point<f64>,
    has_bank: bool,
    distance_m: Option<i32>,
}
```

As soon as she does that, rust-analyzer complains that her `hydrate` method lacks this field. “Nice” she thougts, noting for herself to arrage her code in a manner the compiler helps her as much as she could. While saying that, she tries her API, fetching a station by its identifier but the server crashes because of course there is no such field returned by the query. She fixes that by adding this new field as null in the query's projection:

```rust
    pub async fn find_bike_station_id(
        &self,
        bike_station_id: u32,
    ) -> Result<Option<BikeStation>, Box<dyn Error>> {
        let sql = r#"
select
  bike_station_id,
  designation,
  coords,
  has_bank,
  null::int as distance_m
from bike_station
where bike_station_id = $1"#;
        let rows = self.db_client.query(sql, &[&bike_station_id]).await?;
        let maybe_entity = rows.first().map(BikeStation::hydrate);

        Ok(maybe_entity)
    }
}
```

As she is about to add a new `find_nearest_bike_station` method, she figures out this projection will have to be managed, at least, in two different places. So she creates a projection struct dedicated to `BikeStation` entities.

For every query, she could download the full list of bike stations, compute their distance from her position in Rust and return the result but that seemed a bit clumsy. If Postgres projection is that great, would it be possible to compute the distance directly from the database and the sort the results by distance? She browses the PostgreSQL's documentation and very quickly finds that, since she uses the `point` geometric type, she is able to use a lot of operators, one of them being the [`distance` operator]()