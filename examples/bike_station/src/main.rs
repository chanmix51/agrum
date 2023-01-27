use agrum::core::{
    HydrationError, Projection, Provider, SourceAliases, SqlDefinition, SqlEntity, SqlSource,
    Structure, WhereCondition,
};
use std::error::Error;
use tokio_postgres::{Client, NoTls};

async fn database_setup(client: &Client) -> Result<(), Box<dyn Error>> {
    let queries = &[
        "create schema bike_station_app",
        "create table bike_station_app.bike_station (bike_station_id serial primary key, coords point not null, name text not null unique, has_bank bool not null default false)",
        "create table bike_station_app.station_measure (station_measure_id uuid primary key default uuid_generate_v4(), bike_station_id int not null references bike_station_app.bike_station (bike_station_id), probed_at timestamptz not null default now(), total_slots smallint not null check(total_slots >= 0), working_slots smallint not null check(working_slots >= 0), available_slots smallint not null check(available_slots >= 0))",
        "insert into bike_station_app.bike_station (coords, name, has_bank) values ('(47.220448, -1.554602)', '50 ôtages', true), ('(47.222699, -1.552424)', 'maquis de saffré', false), ('(47.224533, -1.553717)', 'île de versailles', false), ('(47.223557, -1.557789)', 'bellamy', false)",
        "insert into bike_station_app.station_measure (bike_station_id, probed_at, total_slots, working_slots, available_slots) values (1, '2022-12-02 13:33:47+00', 25, 23, 21), (1 ,'2022-12-02 13:37:03+00', 25, 23, 18), (1,'2022-12-02 13:37:03+00',25,23,18), (1,'2022-12-02 13:40:19+00',25,23,17), (1,'2022-12-02 13:43:35+00',25,23,21), (1,'2022-12-02 13:46:51+00',25,23,20)",
        "insert into bike_station_app.station_measure (bike_station_id, probed_at, total_slots, working_slots, available_slots) values (2,'2022-12-02 13:33:13+00',15,14,10), (2,'2022-12-02 13:36:39+00',15,14,9), (2,'2022-12-02 13:40:05+00',15,14,9), (2,'2022-12-02 13:43:31+00',15,14,9), (2,'2022-12-02 13:46:57+00',15,14,10)",
        "insert into bike_station_app.station_measure (bike_station_id, probed_at, total_slots, working_slots, available_slots) values (3,'2022-12-02 13:33:03+00',20,18,3), (3,'2022-12-02 13:36:24+00',20,18,5), (3,'2022-12-02 13:39:45+00',20,18,4), (3,'2022-12-02 13:43:06+00',20,18,6), (3,'2022-12-02 13:46:27+00',20,18,5)",
        "insert into bike_station_app.station_measure (bike_station_id, probed_at, total_slots, working_slots, available_slots) values (4,'2022-12-02 13:33:53+00',15,15,9), (4,'2022-12-02 13:37:06+00',15,15,11), (4,'2022-12-02 13:40:19+00',15,15,13), (4,'2022-12-02 13:43:32+00',15,15,12), (4,'2022-12-02 13:46:45+00',15,15,11)",
    ];
    for sql in queries {
        client.execute(sql.to_owned(), &[]).await?;
    }

    Ok(())
}

async fn database_clean(client: &Client) -> Result<(), Box<dyn Error>> {
    let queries = &["drop schema if exists bike_station_app cascade"];

    for sql in queries {
        client.execute(sql.to_owned(), &[]).await?;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct BikeStationTable;

impl SqlDefinition for BikeStationTable {
    fn expand(&self, _condition: String) -> String {
        "bike_station_app.bike_station".to_owned()
    }
}

impl SqlSource for BikeStationTable {
    fn get_definition(&self) -> &dyn SqlDefinition {
        self
    }

    fn get_structure(&self) -> Structure {
        let mut structure = Structure::default();
        structure
            .set_field("bike_station_id", "int")
            .set_field("name", "text")
            .set_field("coords", "point")
            .set_field("has_bank", "bool");

        structure
    }
}

#[derive(Debug)]
struct ShortBikeStation {
    bike_station_id: i32,
    name: String,
    coords: geo_types::Point<f64>,
    total_slots: i16,
    working_slots: i16,
    available_slots: i16,
    has_bank: bool,
}

impl SqlEntity for ShortBikeStation {
    fn hydrate(row: tokio_postgres::Row) -> Result<Self, HydrationError>
    where
        Self: Sized,
    {
        let entity = Self {
            bike_station_id: row.get("bike_station_id"),
            name: row.get("name"),
            coords: row.get("coords"),
            total_slots: row.get("total_slots"),
            working_slots: row.get("working_slots"),
            available_slots: row.get("available_slots"),
            has_bank: row.get("has_bank"),
        };

        Ok(entity)
    }

    fn get_structure() -> Structure {
        let mut structure = Structure::default();
        structure
            .set_field("bike_station_id", "int")
            .set_field("name", "text")
            .set_field("coords", "point")
            .set_field("total_slots", "int")
            .set_field("working_slots", "int")
            .set_field("available_slots", "int");

        structure
    }
}

struct ShortBikeStationDefinition {
    projection: Projection,
}

impl ShortBikeStationDefinition {
    pub fn new() -> Box<Self> {
        let mut projection =
            Projection::from_structure(BikeStationTable::default().get_structure(), "station");
        projection
            .add_field("{:measure:}.total_slots", "total_slots")
            .add_field("{:measure:}.working_slots", "working_slots")
            .add_field("{:measure:}.available_slots", "available_slots");

        Box::new(Self { projection })
    }
}

impl SqlDefinition for ShortBikeStationDefinition {
    fn expand(&self, condition: String) -> String {
        let source_aliases = SourceAliases::new(vec![
            ("station", "bike_station"),
            ("measure", "last_measure"),
        ]);
        let projection = self.projection.expand(&source_aliases);
        let sql = r#"
select
  {projection}
from bike_station_app.bike_station as bike_station
  inner join lateral (
    select total_slots, working_slots, available_slots
    from bike_station_app.station_measure as station_measure
    where station_measure.bike_station_id = bike_station.bike_station_id
    order by probed_at desc
    limit 1
    ) as last_measure on true
where {condition}"#;

        sql.replace("{projection}", &projection)
            .replace("{condition}", &condition)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (client, connection) = tokio_postgres::connect(
        "host=postgres.lxc user=greg application_name=bike-station",
        NoTls,
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    database_clean(&client).await.unwrap();
    println!("database cleaned successfuly");
    database_setup(&client).await.unwrap();
    println!("database created successfuly");
    let provider: Provider<ShortBikeStation> =
        Provider::new(&client, ShortBikeStationDefinition::new());

    let rows = provider.find(WhereCondition::default()).await.unwrap();
    for row in rows {
        println!("ROW = '{:?}'.", row);
    }
}
