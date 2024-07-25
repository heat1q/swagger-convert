use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use spec::Swagger;
use utoipa::openapi::OpenApi;

use swagger_convert::spec;

fn main() {
    let mut cmd = Command::new("swagger-convert")
        .about("Convert Swagger 2.0 specs into OpenAPI 3.0.")
        .arg(
            Arg::new("swagger")
                .help("Path to swagger 2.0 spec")
                .value_hint(clap::ValueHint::FilePath),
        )
        .arg(
            Arg::new("out")
                .short('o')
                .long("out")
                .default_value("./openapi.json")
                .help("Output OpenAPI file path")
                .value_hint(clap::ValueHint::FilePath),
        );

    let help = cmd.render_help();
    if let Err(err) = parse_args(cmd) {
        let err = err.context("failed to parse arguments");
        println!("{help}\nerror: {err:#}");
    }
}

fn parse_args(cmd: Command) -> Result<()> {
    let matches = cmd.try_get_matches()?;

    let swagger_path = matches
        .get_one::<String>("swagger")
        .ok_or_else(|| anyhow!("missing swagger path"))?;
    let openapi_path = matches.get_one::<String>("out").unwrap();

    let file = File::open(swagger_path)?;
    let mut buf = BufReader::new(file);
    let swagger: Swagger = serde_json::from_reader(&mut buf)?;
    let openapi: OpenApi = swagger.into();

    println!("Writing OpenAPI file to {openapi_path:?}");
    let out_file = File::options()
        .create_new(true)
        .write(true)
        .open(openapi_path)?;
    let mut buf = BufWriter::new(out_file);
    serde_json::to_writer_pretty(&mut buf, &openapi)?;

    Ok(())
}
