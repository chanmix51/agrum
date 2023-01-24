use tokio_postgres::{Client, NoTls};

use std::error::Error;

async fn database_setup(client: &Client) -> Result<(), Box<dyn Error>> {
    let queries = &[
        "create schema bike_station_app",
        "create table bike_station_app.bike_station (bike_station_id serial primary key, coords point not null, name text not null unique, has_bank bool not null default false)",
        "create table bike_station_app.station_measure (station_measure_id uuid primary key default uuid_generate_v4(), station_id int not null references bike_station_app.bike_station (bike_station_id), probed_at timestamptz not null default now(), total_slots smallint not null check(total_slots >= 0), working_slots smallint not null check(working_slots >= 0), available_slots smallint not null check(available_slots >= 0))",
    ];
    for sql in queries {
        client.execute(sql.to_owned(), &[]).await?;
    }

    Ok(())
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
    database_setup(&client).await.unwrap();
}
