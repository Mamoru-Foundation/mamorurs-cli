// static SERVICES: &[&str] = &[
//     "./proto/mamoru-chain/proto/validationchain/validationchain/tx.proto",
//     "./proto/mamoru-chain/proto/validationchain/validationchain/query.proto",
// ];

// static INCLUDES: &[&str] = &["./proto/mamoru-chain/proto/", "./proto/"];

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let builder = tonic_build::configure()
//         .build_server(false)
//         .build_client(true)
//         .extern_path(".cosmos", "::cosmrs::proto::cosmos")
//         .include_file("includes.rs");

//     builder.compile(SERVICES, INCLUDES)?;

//     Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
