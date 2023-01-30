use agrum::core::{
    HydrationError, Projection, Provider, SourceAliases, SqlDefinition, SqlEntity, SqlSource,
    Structure, WhereCondition,
};
use std::collections::HashMap;
use tokio_postgres::{Client, NoTls};

mod tables;
use tables::{BikeStationTable, StationMeasureTable};

#[derive(Debug)]
struct ShortBikeStation {
    bike_station_id: i32,
    name: String,
    coords: geo_types::Point<f64>,
    distance: f64,
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
            distance: row.get("distance"),
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
            .set_field("distance", "float")
            .set_field("coords", "point")
            .set_field("total_slots", "int")
            .set_field("working_slots", "int")
            .set_field("available_slots", "int");

        structure
    }
}

struct FindShortBikeStationAround {
    sources: HashMap<String, Box<dyn SqlSource>>,
}

impl FindShortBikeStationAround {
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

    fn get_projection(&self) -> Projection {
        let structure = self.sources.get("bike_station").unwrap().get_structure();
        let mut projection = Projection::from_structure(structure, "station");
        projection
            .add_field(
                "({:station:}.coords <-> parameters.current_position) * 113432",
                "distance",
            )
            .add_field("{:measure:}.total_slots", "total_slots")
            .add_field("{:measure:}.working_slots", "working_slots")
            .add_field("{:measure:}.available_slots", "available_slots");

        projection
    }
}

impl SqlDefinition for FindShortBikeStationAround {
    fn expand(&self, condition: String) -> String {
        let source_aliases = SourceAliases::new(vec![
            ("station", "bike_station"),
            ("measure", "last_measure"),
        ]);
        let projection = self.get_projection().expand(&source_aliases);
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
order by distance asc"#;

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

struct ShortBikeStationsAroundProvider<'client>(Provider<'client, ShortBikeStation>);

impl<'client> ShortBikeStationsAroundProvider<'client> {
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
    let provider =
        ShortBikeStationsAroundProvider(Provider::new(&client, FindShortBikeStationAround::new()));
    /*
       for row in rows {
           println!("ROW = '{:?}'.", row);
       }
    */
}
