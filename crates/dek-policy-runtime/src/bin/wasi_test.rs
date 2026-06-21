#![allow(unused)]
use std::io::Cursor;
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

fn main() {
    let input_str = r#"{"allow": true}"#;
    let stdin = ReadPipe::from(input_str);
    let stdout = WritePipe::new_in_memory();

    let mut builder = WasiCtxBuilder::new();
    builder.stdin(Box::new(stdin.clone()));
    builder.stdout(Box::new(stdout.clone()));

    let wasi = builder.build();
    let bytes = stdout.try_into_inner().unwrap().into_inner();
    println!("bytes len: {}", bytes.len());
}
