mod lzw;
use clap::Parser;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;
use tiff;
use tiff::decoder::DecodingResult;
use tiff::encoder::colortype;
use tiff::ColorType;

#[derive(Parser)]
struct Cli {
    #[arg(short)]
    input_file: PathBuf,

    #[arg(short)]
    output_file: PathBuf,

    #[arg(long, default_value_t = true)]
    compress: bool,

    #[arg(long, default_value_t = false)]
    decompress: bool,
}

fn run_decompressor(cli: &Cli) -> Result<(), Error> {
    let mut input_f = File::open(cli.input_file.to_str().unwrap())?;
    let mut archive = Vec::new();
    input_f.read_to_end(&mut archive)?;
    let (data, dim) = lzw::decompress(&archive);

    let output_f = File::create(cli.output_file.to_str().unwrap())?;
    let mut encoder = tiff::encoder::TiffEncoder::new(output_f).unwrap();
    if archive[0] == 0 {
        encoder
            .write_image::<colortype::Gray8>(dim.0, dim.1, &data)
            .unwrap();
    } else {
        encoder
            .write_image::<colortype::RGB8>(dim.0, dim.1, &data)
            .unwrap();
    }

    return Ok(());
}

fn run_compressor(cli: &Cli) -> Result<(), Error> {
    let input_f = File::open(cli.input_file.to_str().unwrap())?;
    let mut decoder = tiff::decoder::Decoder::new(&input_f).unwrap();
    let img_coded = decoder.read_image();
    let dim = decoder.dimensions().unwrap();
    match img_coded {
        Ok(DecodingResult::U8(data)) => {
            let archive = match decoder.colortype() {
                Ok(ColorType::RGB(_)) => lzw::compress_rgb(&data, dim),
                Ok(ColorType::Gray(_)) => lzw::compress_gray(&data, dim),
                _ => panic!("unsupported colortype"),
            };
            let mut output_f = File::create(cli.output_file.to_str().unwrap())?;
            output_f.write_all(&archive)?;
        }
        _ => {
            panic!("something went wrong");
        }
    }

    return Ok(());
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let result = if cli.decompress {
        run_decompressor(&cli)
    } else {
        run_compressor(&cli)
    };

    if let Err(error) = result {
        match error.kind() {
            ErrorKind::NotFound => println!("Указанный файл не найден"),
            ErrorKind::AlreadyExists => println!("Указанный файл уже существует"),
            _ => println!("Произошла непредвиденная ошибка"),
        };
    }

    Ok(())
}
