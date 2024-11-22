
use clap::{Args,Parser, Subcommand};
use std::fs::File;
use std::io::{BufReader, Read};
use std::process;
use std::time::SystemTime;

use colors_transform::{Color, Hsl, Rgb};
use png;
use qoi::qoi_lib::*;
use log::{error,info};

fn encode_checkerboard() {
    let mut pixels: Vec<Pixel> = Vec::with_capacity(64 * 64);
    let red: u8 = 150;
    let green: u8 = 0;
    let blue: u8 = 150;
    //row iterator
    for i in 0..64 {
        //column iterator
        for j in 0..64 {
            //if row is 0..16, 32..48
            if (i / 16) == 0 || (i / 16) == 2 {
                //if column is 0..16, 32..48
                if (j / 16) == 0 || (j / 16) == 2 {
                    let push_pix: Pixel = Pixel::new(red, green, blue, 255);
                    pixels.push(push_pix);
                } else {
                    let push_pix: Pixel = Pixel::new(255, 255, 255, 255);
                    pixels.push(push_pix);
                }
            } else {
                if (j / 16) == 1 || (j / 16) == 3 {
                    let push_pix: Pixel = Pixel::new(red, green, blue, 255);
                    pixels.push(push_pix);
                } else {
                    let push_pix: Pixel = Pixel::new(255, 255, 255, 255);
                    pixels.push(push_pix);
                }
            }
        }
    }

    let img: Image = Image::from_pixels(pixels, 64, 64, 4, 0);
    write_to_file(encode_from_image(img), "checkerboard").expect("Error writing file!");
}

fn encode_debug() {
    let mut img_data: Vec<u8> = Vec::new();
    //row iterator
    for i in 0..1024 {
        //cell iterator
        for j in 0..1024 {
            //subpixel iterator
            for k in 0..4 {
                let rgb: Hsl = Hsl::from(0.3515625 * j as f32, 100.0, 50.0);
                let rgb: Rgb = rgb.to_rgb();
                let alpha: f64 = -(255.0 / 1024.0) * (i as f64) + 255.0;
                match k {
                    0 => img_data.push(rgb.get_red() as u8),
                    1 => img_data.push(rgb.get_green() as u8),
                    2 => img_data.push(rgb.get_blue() as u8),
                    3 => img_data.push(alpha as u8),
                    _ => error!("unrecoverable for-loop failure"),
                }
            }
        }
    }
    let img: Image = match Image::new(img_data, 1024, 1024, 4, 0) {
        Ok(image) => image,
        Err(err) => panic!("Problem generating image: {:?}", err),
    };
    let start = SystemTime::now();
    let img_bytes: Vec<u8> = encode_from_image(img);
    let stop = match start.elapsed() {
        Ok(elapsed) => elapsed.as_millis(),
        Err(e) => {
            error!("Error: {e:?}");
            return ();
        }
    };
    info!("Encode took: {} ms.", stop);
    write_to_file(img_bytes, "test").expect("Error writing file!");
}

fn demo() {
    let start = SystemTime::now();
    encode_checkerboard();
    let stop = match start.elapsed() {
        Ok(elapsed) => elapsed.as_millis(),
        Err(e) => {
            error!("Error: {e:?}");
            return ();
        }
    };
    println!("Encode took: {} ms.", stop);
    encode_debug();
}

//Attempts to encode given png image as second argument into qoi
fn encode(in_path: &str, out_path: &str) {

    //Init png decoder, attempt to decode png into bitmap, throw error if unsuccessful
    let file:File = File::open(in_path).unwrap_or_else(|e| {
        println!("Error: {:?}", e.to_string());
        process::exit(1);
    });
    let decoder = png::Decoder::new(file);
    let mut reader = match decoder.read_info() {
        Ok(reader) => reader,
        Err(e) => panic!("ERROR: couldn't read file: {e:}"),
    };

    //read image metadata
    let width: u32 = reader.info().width;
    let height: u32 = reader.info().height;
    //for now: hardcoded to 4
    let channels: u8 = 4;

    //create buffer matching the size of png-decoder output, writing size to output
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = match reader.next_frame(&mut buf) {
        Ok(i) => i,
        Err(e) => panic!("ERROR: {e:?}"),
    };

    //convert buffer into vector
    let bytes = &buf[..info.buffer_size()];
    let byte_vec: Vec<u8> = bytes.to_vec();

    //create bitmap data from raw byte vector
    let img: Image = match Image::new(byte_vec, height, width, channels, 0) {
        Ok(image) => image,
        Err(err) => panic!("Problem generating image: {:?}", err),
    };

    //in case out_path is erroneously passed with suffix
    let filename = match out_path.strip_suffix(".png") {
        Some(s) => s,
        None => out_path
    };

    write_to_file(encode_from_image(img), filename).expect("ERROR: Can't write file.");
    info!("Encoding successful!");
}


fn decode(path: &str) -> Result<Image, std::io::Error> {
    let f: File = match File::open(path) {
        Ok(f) => f,
        Err(e) => panic!("ERROR: {e:?}"),
    };
    let mut reader = BufReader::new(f);
    let mut bytes: Vec<u8> = Vec::new();

    reader.read_to_end(&mut bytes)?;

    match qoi::qoi_lib::decode(bytes) {
        Ok(img) => {
            info!("Decoding successful!");
            return Ok(img);
        },
        Err(err) => panic!("ERROR: {err:?}"),
    }
}

fn bench(input: &str, output: Option<String>) {
    
    let start = SystemTime::now();
    let out_path = match output {
        Some(s) => s,
        None => input.strip_suffix(".png").unwrap_or(input).to_owned()
    };

    encode(input, &out_path);

    match start.elapsed() {
        Ok(elapsed) => {
            if elapsed.as_millis() == 0 {
                println!("Encode took {:?} μs to complete", elapsed.as_micros());
            } else if elapsed.as_millis() > 999 {
                println!("Encode took {:.3} s to complete", elapsed.as_secs_f32());
            } else {
                println!("Encode took {:?} ms to complete", elapsed.as_millis());
            }
        },
        
        Err(e) => panic!("ERROR: {e:?}"),
    }
    let start = SystemTime::now();
    let mut out_path: String = out_path.to_owned();
    if !out_path.contains(".qoi") {
        out_path.push_str(".qoi");
    }
    match decode(&out_path) {
        Ok(img) => {
            //Never fails as long as memory does not corrupt thanks to above push_str op.
            let png_path = out_path.strip_suffix(".qoi").unwrap();
            img.write_png(&png_path);
        },
        Err(e) => panic!("Error: {e:?}")
    }
    match start.elapsed() {
        Ok(elapsed) => {
            if elapsed.as_millis() == 0 {
                println!("Encode took {:?} μs to complete", elapsed.as_micros());
            } else if elapsed.as_millis() > 999 {
                println!("Encode took {:.3} s to complete", elapsed.as_secs_f32());
            } else {
                println!("Encode took {:?} ms to complete", elapsed.as_millis());
            }
        },
        
        Err(e) => panic!("ERROR: {e:?}"),
    }
}

#[derive(Parser)]
#[command(name = "QOI Image Transcoder")]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)] 
struct Cli {
    /// Specify log verbosity, respects multiple applications.
    #[arg(short,long, action = clap::ArgAction::Count)]
    verbose: Option<u8>,

    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Encode given [IMAGE] from { png } to qoi. 
    Encode(EncodeArgs),
    /// Decode given qoi to specified [FORMAT].
    Decode(DecodeArgs),
    /// Benchmark en- and decoder by passing in [IMAGE] and optionally specifying [OUTPUT] file.
    Bench(BenchArgs),
    /// Demo the application.
    Demo {
    }
}

#[derive(Args)]
struct BenchArgs {
    /// File to be encoded.
    #[arg(short,long)]
    input: String,
    /// Optional output path.
    #[arg(short,long)]
    output: Option<String>
}

#[derive(Args)]
struct DecodeArgs {
    /// Qoi file to be decoded
    #[arg(short,long)]
    input: String,
    /// Format to transcode into
    #[arg(short,long)]
    format: String,
    /// Optional file path
    #[arg(short,long)]
    output: Option<String>
}

#[derive(Args)]
struct EncodeArgs {
    // File to be encoded
    #[arg(short,long)]
    input: String,
    // Optional output path
    #[arg(short,long)]
    output: Option<String>
}

fn main() {
    let cli: Cli = Cli::parse();
    

    match &cli.command {
        Commands::Bench(args) => {
            bench(&args.input, args.output.clone());
        },
        Commands::Decode(args)=> {
            if args.format != "png" {
                panic!("Unsupported output format!")
            } else {
                let img = match decode(&args.input) {
                    Ok(i) => i,
                    Err(e) => panic!("Error: {e:?}")
                };
                let out_path = match &args.output {
                    Some(s) => s,
                    None => &args.input 
                };
                img.write_png(&out_path);
            }
        },
        Commands::Encode(args) => {
            let out_path = match &args.output {
                Some(s) => s,
                None => args.input.strip_suffix(".png").unwrap_or_else(||{
                    println!("Error: Could not construct output arg from input arg. Please provide explicitly");
                    process::exit(1);
                })
            };

            encode(&args.input, &out_path);
        },
        Commands::Demo {  } => demo()
    }
}
