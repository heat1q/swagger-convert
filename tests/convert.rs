use assert_json_diff::assert_json_eq;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};
use swagger_convert::{include_json, spec::Swagger};
use testdir::testdir;
use utoipa::openapi::OpenApi;

#[test]
fn convert_basic_swagger_structure_should_succeed() {
    // given
    let swagger_path = PathBuf::from("tests/data/swagger.json");
    let file = File::open(swagger_path).unwrap();
    let mut buf = BufReader::new(file);
    let swagger: Swagger = serde_json::from_reader(&mut buf).unwrap();

    // when
    let openapi: OpenApi = swagger.into();

    // then
    let openapi_json = serde_json::to_value(openapi).unwrap();
    assert_json_eq!(include_json!(".././tests/data/openapi.json"), openapi_json);
}

#[test]
fn deserialize_to_file_should_succeed() {
    // given
    let swagger_path = PathBuf::from("tests/data/swagger.json");
    let file = File::open(swagger_path).unwrap();
    let mut buf = BufReader::new(file);
    let swagger: Swagger = serde_json::from_reader(&mut buf).unwrap();
    let dir = testdir!();

    // when
    let openapi: OpenApi = swagger.into();

    // then
    let openapi_json = serde_json::to_value(&openapi).unwrap();
    println!("sting: {}", openapi_json);
    let out_file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(dir.join("test.json"))
        .expect("Could not create/write target");
    let mut buf = BufWriter::new(out_file);
    serde_json::to_writer_pretty(&mut buf, &openapi).expect("Could not write merged file");
    // read from same file again
    let expected_file =
        File::open("tests/data/openapi.json").expect("Unable to open the openapi.json");
    let json: serde_json::Value =
        serde_json::from_reader(expected_file).expect("file should be proper JSON");

    assert_json_eq!(json, openapi_json);
}

