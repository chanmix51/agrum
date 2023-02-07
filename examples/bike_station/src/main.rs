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

    fn sql_projection() -> Projection {
        let mut projection = Projection::default();
        projection
            .set_field("bike_station_id", "{:station:}.bike_station_id", "int")
            .set_field("name", "initcap({:station:}.name)", "text")
            .set_field("coords", "{:station:}.coords", "point")
            .set_field(
                "distance_m",
                "floor(sin(radians({:station:}.coords <-> parameters.current_position)) * 6431000)::int",
                "int",
            )
            .set_field("total_slots", "{:measure:}.total_slots", "int")
            .set_field("working_slots", "{:measure:}.working_slots","int")
            .set_field("available_slots", "{:measure:}.available_slots", "int")
            .set_field("has_bank", "{:station:}.has_bank", "bool");

        projection
    }
}

struct FindShortBikeStationAroundDefinition {
    sources: HashMap<String, Box<dyn SqlSource>>,
}

impl FindShortBikeStationAroundDefinition {
    pub fn new() -> Box<Self> {
        let mut sources: HashMap<String, Box<dyn SqlSource>> = HashMap::new();
        sources.insert(
            "bike_station".to_string(),
            Box::new(BikeStationTable::default()),
        );
        sources.insert(
            "station_measure".to_string(),
            Box::new(StationMeasureTable::default()),
        );

        Box::new(Self { sources })
    }
}

impl SqlDefinition for FindShortBikeStationAroundDefinition {
    fn expand(&self, condition: String) -> String {
        let source_aliases = SourceAliases::new(vec![
            ("station", "bike_station"),
            ("measure", "last_measure"),
        ]);
        let projection = ShortBikeStation::sql_projection().expand(&source_aliases);
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

        sql.replace("{projection}", &projection)
            .replace("{condition}", &condition)
            .replace(
                "{station}",
                &self
                    .sources
                    .get("bike_station")
                    .unwrap()
                    .get_definition()
                    .expand(String::new()),
            )
            .replace(
                "{measure}",
                &self
                    .sources
                    .get("station_measure")
                    .unwrap()
                    .get_definition()
                    .expand(String::new()),
            )
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
