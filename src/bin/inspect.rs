use agrum::inspector::Inspector;
use clap::{Parser, Subcommand};
use tokio_postgres::NoTls;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type UnitResult = Result<()>;

trait OutputBuffer {
    fn add_line(&mut self, line: String);

    fn flush(self) -> Vec<String>;
}

#[derive(Debug, Default)]
struct LineOutputBuffer {
    lines: Vec<String>,
}

impl OutputBuffer for LineOutputBuffer {
    fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }

    fn flush(self) -> Vec<String> {
        self.lines
    }
}

/// Database inspector program
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct CommandLine {
    /// Database source definition ie `pgsql://user:pass@host.com:5432/db_name`.
    #[arg(long, env = "AGRUM_DSN")]
    dsn: String,

    /// inspecto command
    #[command(subcommand)]
    command: InspectorCommandChoice,
}

impl CommandLine {
    async fn execute(&self, output: &mut (dyn OutputBuffer)) -> UnitResult {
        self.command.execute(&self.dsn, output).await
    }
}

#[derive(Debug, Subcommand)]
enum InspectorCommandChoice {
    /// List information about the databases
    List,

    /// Show information about the database
    Show,

    /// Schemas subcommands
    Schema {
        #[command(subcommand)]
        command: InspectorSchemaSubCommandChoice,
    },
}

impl InspectorCommandChoice {
    async fn execute(&self, dsn: &str, output: &mut dyn OutputBuffer) -> UnitResult {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {e}");
            }
        });
        let inspector = Inspector::new(&client);

        match self {
            Self::List => {
                for db in inspector.get_database_list().await? {
                    output.add_line(format!("{}", db.name));
                    output.add_line(format!("    owner:         {}", db.owner));
                    output.add_line(format!("    encoding:      {}", db.encoding));
                    output.add_line(format!("    size:          {}", db.size));
                    output.add_line(format!("    description:   {}", db.name));
                }

                Ok(())
            }
            Self::Show => {
                /*
                let db_name = dsn_info
                    .database
                    .clone()
                    .ok_or_else(|| -> Box<dyn std::error::Error> {
                        format!("No database given in DSN '{dsn_info:?}'.").into()
                    })?
                    .to_owned();
                let db_info = inspector.get_db_info(&db_name).await?;
                */

                Ok(())
            }
            Self::Schema { command } => command.execute(&inspector, output).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum InspectorSchemaSubCommandChoice {
    /// List the schemas in the database.
    Schemas,

    /// Show informations about a given schema.
    Schema {
        /// Schema name
        schema_name: String,

        #[command(subcommand)]
        command: InspectorTableSubCommand,
    },
}

impl InspectorSchemaSubCommandChoice {
    pub async fn execute(
        &self,
        inspector: &Inspector<'_>,
        output: &mut dyn OutputBuffer,
    ) -> UnitResult {
        match self {
            Self::Schemas => {
                let schemas = inspector.get_schema_list().await?;

                for schema_info in schemas {
                    output.add_line(format!("name: {}", schema_info.name));
                    output.add_line(format!("    relations:     {}", schema_info.relations));
                    output.add_line(format!("    owner:         {}", schema_info.owner));
                    output.add_line(format!(
                        "    description:   {}",
                        match schema_info.description {
                            Some(v) => v,
                            None => String::new(),
                        }
                    ));
                }

                Ok(())
            }
            Self::Schema {
                schema_name,
                command,
            } => command.execute(inspector, schema_name, output).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum InspectorTableSubCommand {
    /// List the tables in a schema
    Relations,

    /// Show table details
    Relation {
        /// relation name
        relation_name: String,
    },
}

impl InspectorTableSubCommand {
    pub async fn execute(
        &self,
        inspector: &Inspector<'_>,
        schema_name: &str,
        output: &mut dyn OutputBuffer,
    ) -> UnitResult {
        todo!()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> UnitResult {
    let args = CommandLine::parse();
    let mut output = LineOutputBuffer::default();
    let res = args.execute(&mut output).await;

    if let Err(e) = res {
        return Err(format!("error: {e}").into());
    } else {
        for line in output.flush() {
            println!("{line}");
        }
    }

    Ok(())
}
