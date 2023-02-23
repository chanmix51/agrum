use comfy_table::Table;
use geo_types::Coord;
use tokio_postgres::{Client, NoTls};

use std::collections::HashMap;

use agrum::core::*;

mod tables;
use tables::{BikeStationTable, StationMeasureTable};

#[derive(Debug)]
struct ShortBikeStation {
    bike_station_id: i32,
    name: String,
    coords: geo_types::Point<f64>,
    distance_m: i32,
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
            distance_m: row.get("distance_m"),
            total_slots: row.get("total_slots"),
            working_slots: row.get("working_slots"),
            available_slots: row.get("available_slots"),
            has_bank: row.get("has_bank"),
        };

        Ok(entity)
    }
}

impl Structured for ShortBikeStation {
    fn get_structure() -> Structure {
        let structure = Structure::new(&[
            ("bike_station_id", "int"),
            ("name", "text"),
            ("coords", "point"),
            ("distance_m", "int"),
            ("total_slots", "int"),
            ("working_slots", "int"),
            ("available_slots", "int"),
            ("has_bank", "bool"),
        ]);

        structure
    }
}

struct FindShortBikeStationAroundDefinition {
    sources: SourcesCatalog,
    projection: Projection<ShortBikeStation>,
}

impl FindShortBikeStationAroundDefinition {
    pub fn new() -> Box<Self> {
        let mut sources: SourcesCatalog = SourcesCatalog::new(HashMap::new());
        sources
            .add_source("bike_station", Box::new(BikeStationTable::default()))
            .add_source("station_measure", Box::new(StationMeasureTable::default()));

        let projection = Projection::<ShortBikeStation>::default();
        projection
            .set_definition("distance_m",  "floor(sin(radians({:station:}.coords <-> parameters.current_position)) * 6431000)::int")
            .set_definition("bike_station_id", "{:station:}.bike_station_id");

        Box::new(Self {
            sources,
            projection,
        })
    }
}

impl SqlDefinition for FindShortBikeStationAroundDefinition {
    fn expand(&self, condition: &str) -> String {
        let source_aliases =
            SourceAliases::new(&[("station", "bike_station"), ("measure", "last_measure")]);
        let sql = r#"
with parameters as (select $1::point as current_position, $2::float as search_radius)
select
  {projection}
from {station} as bike_station
  cross join parameters
  inner join lateral (
    select total_slots, working_slots, available_slots
    from {measure} as station_measure
    where station_measure.bike_station_id = bike_station.bike_station_id
    order by probed_at desc
    limit 1
    ) as last_measure on true
where {condition}
order by distance_m asc"#;

        sql.replace("{projection}", &self.projection.expand(&source_aliases))
            .replace("{condition}", &condition)
            .replace("{station}", &self.sources.expand("bike_station", ""))
            .replace("{measure}", &self.sources.expand("station_measure", ""))
    }
}

struct ShortBikeStationsAroundFinder<'client>(Provider<'client, ShortBikeStation>);

impl<'client> ShortBikeStationsAroundFinder<'client> {
    pub fn new(client: &'client Client) -> Self {
        Self(Provider::new(
            &client,
            FindShortBikeStationAroundDefinition::new(),
        ))
    }

    pub async fn find_short_stations_around(
        &self,
        position: &geo_types::Point,
        search_radius: f64,
    ) -> Vec<ShortBikeStation> {
        let condition = WhereCondition::new(
            "circle(parameters.current_position, parameters.search_radius) @> coords",
            vec![position, &search_radius],
        );

        self.0.find(condition).await.unwrap()
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
    tables::database_clean(&client).await.unwrap();
    println!("database cleaned successfuly");
    tables::database_setup(&client).await.unwrap();
    println!("database created successfuly");
    let finder = ShortBikeStationsAroundFinder::new(&client);
    let rows = finder
        .find_short_stations_around(
            &geo_types::Point(Coord {
                x: 47.22226827156824,
                y: -1.5541691161031304,
            }),
            0.0035,
        )
        .await;

    let mut table = Table::new();
    let headers: Vec<&str> = vec![
        "station ID",
        "name",
        "distance (m)",
        "coordinates",
        "slots",
        "online slots",
        "available slots",
        "has bank",
    ];
    table.set_header(headers);

    for row in rows {
        let display: Vec<String> = vec![
            format!("{}", &row.bike_station_id),
            row.name.to_owned(),
            format!("{}", &row.distance_m),
            format!("{:?}", &row.coords),
            format!("{}", &row.total_slots),
            format!("{}", &row.working_slots),
            format!("{}", &row.available_slots),
            if row.has_bank {
                "YES".to_string()
            } else {
                "NO".to_string()
            },
        ];
        table.add_row(display);
    }

    println!("{table}");
}
