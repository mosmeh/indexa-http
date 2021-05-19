mod indexa_config;
mod routes;

use actix_web::{middleware, web, App, HttpServer};
use indexa::database::Database;
use std::path::PathBuf;
use structopt::{clap::AppSettings, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(
    author = env!("CARGO_PKG_AUTHORS"),
    rename_all = "kebab-case",
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    setting(AppSettings::AllArgsOverrideSelf)
)]
pub struct Opt {
    /// Address to listen on
    #[structopt(short, long, default_value = "127.0.0.1:8080")]
    addr: String,

    /// Number of threads to use.
    ///
    /// Defaults to the number of available CPUs - 1.
    #[structopt(short, long)]
    threads: Option<usize>,

    /// Location of the config file.
    #[structopt(short = "C", long)]
    config: Option<PathBuf>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let opt = Opt::from_args();
    let mut indexa_config = indexa_config::read_config(opt.config.as_ref()).unwrap();
    indexa_config.flags.merge_opt(&opt);

    let db_location = indexa_config
        .database
        .location
        .expect("Could not determine the location of database file. Please edit the config file.");

    let database: Database =
        bincode::deserialize(&std::fs::read(db_location)?).expect("Failed to load database");
    let database = web::Data::new(database);

    rayon::ThreadPoolBuilder::new()
        .num_threads(indexa_config.flags.threads)
        .build_global()
        .expect("Could not build thread pool");

    eprintln!("Listening on http://{}", opt.addr);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(database.clone())
            .configure(routes::configure)
    })
    .bind(opt.addr)?
    .run()
    .await
}
