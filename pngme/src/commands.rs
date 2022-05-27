use crate::args::{DecodeArgs, EncodeArgs, PrintArgs, RemoveArgs};
use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::Png;
use crate::{Error, Result};
use clap::{arg, App, AppSettings, Arg, ArgMatches};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
pub struct PngMeCmd {
    arg_matches: ArgMatches,
}

impl PngMeCmd {
    pub fn new() -> Self {
        let matches = App::new("pngme")
            .about("A png secret message CLI")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::AllowExternalSubcommands)
            .setting(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
            .subcommand(
                App::new("encode")
                    .about("encode png secret message")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(arg!(<PATH> "png path").allow_invalid_utf8(true))
                    .arg(Arg::new("CHUNK_TYPE").required(true).help("chunk type (the first character must be uppercase for png can normal display)"))
                    .arg(Arg::new("DATA").required(true).help("secret message")),
            )
            .subcommand(
                App::new("decode")
                    .about("decode png secret message by type")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(arg!(<PATH> "png path").allow_invalid_utf8(true))
                    .arg(Arg::new("CHUNK_TYPE").required(true).help("chunk type (the first character must be uppercase for png can normal display)")),
                )
            .subcommand(
                App::new("remove")
                    .about("remove png secret message by type")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(arg!(<PATH> "png path").allow_invalid_utf8(true))
                    .arg(Arg::new("CHUNK_TYPE").required(true).help("chunk type (the first character must be uppercase for png can normal display)")),
                )
            .subcommand(
                App::new("print")
                    .about("print png all of the chunks")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(arg!(<PATH> "png path").allow_invalid_utf8(true))
                )
            .get_matches();
        PngMeCmd {
            arg_matches: matches,
        }
    }

    pub fn match_handler(&self) {
        match self.arg_matches.subcommand() {
            Some(("encode", sub_matches)) => {
                let paths = sub_matches.value_of_os("PATH").unwrap_or_default();
                // .map(PathBuf::from)
                // .collect::<Vec<_>>();
                let data = sub_matches.value_of("DATA").unwrap();
                let chunk_type = sub_matches.value_of("CHUNK_TYPE").unwrap();
                let p = PathBuf::from(paths);
                let arg = EncodeArgs {
                    path: p,
                    chunk_type: chunk_type.to_string(),
                    data: data.to_string(),
                };
                if let Err(e) = encode(arg) {
                    println!("{:?}", e.to_string());
                }
            }
            Some(("decode", sub_matches)) => {
                let paths = sub_matches.value_of_os("PATH").unwrap_or_default();
                // .map(PathBuf::from)
                // .collect::<Vec<_>>();
                let chunk_type = sub_matches.value_of("CHUNK_TYPE").unwrap();
                let p = PathBuf::from(paths);
                let arg = DecodeArgs {
                    path: p,
                    chunk_type: chunk_type.to_string(),
                };
                if let Err(e) = decode(arg) {
                    println!("{:?}", e.to_string());
                }
            }
            Some(("remove", sub_matches)) => {
                let paths = sub_matches.value_of_os("PATH").unwrap_or_default();
                // .map(PathBuf::from)
                // .collect::<Vec<_>>();
                let chunk_type = sub_matches.value_of("CHUNK_TYPE").unwrap();
                let p = PathBuf::from(paths);
                let arg = RemoveArgs {
                    path: p,
                    chunk_type: chunk_type.to_string(),
                };
                if let Err(e) = remove(arg) {
                    println!("{:?}", e.to_string());
                }
            }
            Some(("print", sub_matches)) => {
                let paths = sub_matches.value_of_os("PATH").unwrap_or_default();
                // .map(PathBuf::from)
                // .collect::<Vec<_>>();
                let p = PathBuf::from(paths);
                let arg = PrintArgs { path: p };
                if let Err(e) = print_chunks(arg) {
                    println!("{:?}", e.to_string());
                }
            }
            _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
        }
    }
}

/// Encodes a message into a PNG file and saves the result
pub fn encode(args: EncodeArgs) -> Result<()> {
    let mut png = Png::from_file(&args.path).unwrap();
    println!("{}", &png.as_bytes().len());
    // let data = fs::read(&args.path).unwrap();
    // let png = Png::try_from(data.as_ref()).unwrap();
    let chunk_type = ChunkType::from_str(args.chunk_type.as_str()).unwrap();
    let d = args.data.bytes().collect();
    let chunk = Chunk::new(chunk_type, d);
    png.append_chunk(chunk);
    println!("{}", &png.as_bytes().len());
    let res = fs::write(&args.path, png.as_bytes());
    if let Err(e) = res {
        println!("{:?}", e.to_string());
    }
    println!("ok");
    Ok(())
}

/// Searches for a message hidden in a PNG file and prints the message if one is found
pub fn decode(args: DecodeArgs) -> Result<()> {
    let png = Png::from_file(&args.path).unwrap();
    // let data = fs::read(&args.path).unwrap();
    // let png = Png::try_from(data.as_ref()).unwrap();
    let res = png.chunk_by_type(args.chunk_type.as_str());
    match res {
        None => println!("not exists"),
        Some(c) => println!("data: {}", c.data_as_string()?),
    }
    Ok(())
}

/// Removes a chunk from a PNG file and saves the result
pub fn remove(args: RemoveArgs) -> Result<()> {
    let mut png = Png::from_file(&args.path).unwrap();
    // let data = fs::read(&args.path).unwrap();
    // let png = Png::try_from(data.as_ref()).unwrap();
    let res = png.remove_chunk(args.chunk_type.as_str());
    match res {
        Err(e) => println!("remove error: {}", e.to_string()),
        Ok(data) => println!("remove chunk: {}", data.to_string()),
    }
    let res = fs::write(&args.path, png.as_bytes());
    if let Err(e) = res {
        println!("{:?}", e.to_string());
    }
    println!("ok");
    Ok(())
}

/// Prints all of the chunks in a PNG file
pub fn print_chunks(args: PrintArgs) -> Result<()> {
    let png = Png::from_file(&args.path).unwrap();
    println!("{}", png.to_string());
    Ok(())
}
