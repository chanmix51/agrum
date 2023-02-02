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
struct BikeStation {
    bike_station_id: i32,
    designation: String,
    coords: geo_types::Point<f64>,
    has_bank: bool,
}

struct BikeStationFinderService<'client> {
    db_client: &'client Client,
}

impl<'client> BikeStationFinderService<'client> {
    pub fn new(db_client: &'client Client) -> Self {
        Self { db_client }
    }

    pub async fn find_bike_station_id(&self, bike_station_id: u32) -> Result<Option<BikeStation>, Box<dyn Error>> {
        let sql = "select * from bike_station where bike_station_id = $1";
        let rows = self.db_client
            .query(&sql, &[bike_station_id]).await?;
    }
}
```