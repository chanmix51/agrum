// This code should be generated automatically.

use tokio_postgres::Client;

use std::error::Error;

use agrum::core::{SqlDefinition, SqlSource, Structure};

pub async fn database_setup(client: &Client) -> Result<(), Box<dyn Error>> {
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

pub async fn database_clean(client: &Client) -> Result<(), Box<dyn Error>> {
    let queries = &["drop schema if exists bike_station_app cascade"];

    for sql in queries {
        client.execute(sql.to_owned(), &[]).await?;
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct BikeStationTable;

impl SqlDefinition for BikeStationTable {
    fn expand(&self, _condition: &str) -> String {
        "bike_station_app.bike_station".to_owned()
    }
}

impl SqlSource for BikeStationTable {
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

#[derive(Debug, Default)]
pub struct StationMeasureTable;

impl SqlDefinition for StationMeasureTable {
    fn expand(&self, _condition: &str) -> String {
        "bike_station_app.station_measure".to_owned()
    }
}

impl SqlSource for StationMeasureTable {
    fn get_structure(&self) -> Structure {
        let mut structure = Structure::default();
        structure
            .set_field("station_measure_id", "public.uuid")
            .set_field("bike_station_id", "int")
            .set_field("probed_at", "timestamptz")
            .set_field("total_slots", "int2")
            .set_field("working_slots", "int2")
            .set_field("available_slots", "int2");

        structure
    }
}
