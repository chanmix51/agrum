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
        let bike_station = rows.first()
            .map(|row| BikeStation { 
                bike_station_id: row.get("bike_station_id"),
                designation: row.get("designation"),
                coords: row.get("coords"),
                has_bank: row.get("has_bank"),
                });
        
        Ok(bike_station)
    }
}
```

She stars at the code she just wrote liking how elegantly the Rust language allowed to express what she wants, dealing with error and memory management in a concise way. She very likes the part where she says « take the first row of the result and if it exists, convert it to a `BikeStation` instance.
Inserting this piece of code in her existing project is a very easy task and she goes to bed with the feeling of a day that saw things well done.