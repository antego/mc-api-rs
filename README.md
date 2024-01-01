# Linux Media Controller API bindings for Rust

The experimental bindings for the Media Controller API (MC API). The main motivation behind creating these bindings is to be able to get the media device information from the programs written in Rust without directly touching the IOCTL APIs.

You can read more about the Media Controller API [here](https://docs.kernel.org/userspace-api/media/mediactl/media-controller.html).

## Use at your own risk

Only a small subset of MC API was tested. Some of the bindings might not work. Currently there are no automated tests.

## Usage

1. Add dependency to the `Cargo.toml`
```
[dependencies]
mc-api = { git = "https://github.com/antego/mc-api-rs" }
```
2. Use the bindings to get the information about a media device
```rust
let topology = mc_api_rs::get_topology(Path::new("/dev/media3"));
println!("result: {:#?}", topology);
```
