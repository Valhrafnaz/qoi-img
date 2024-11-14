use std::env;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::time::SystemTime;

use colors_transform::{Color, Hsl, Rgb};
use png;
use qoi::qoi_lib::*;

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
                    _ => panic!("unrecoverable for-loop failure"),
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
            println!("Error: {e:?}");
            return ();
        }
    };
    println!("Encode took: {} ms.", stop);
    write_to_file(img_bytes, "test").expect("Error writing file!");
}

fn demo() {
    let start = SystemTime::now();
    encode_checkerboard();
    let stop = match start.elapsed() {
        Ok(elapsed) => elapsed.as_millis(),
        Err(e) => {
            println!("Error: {e:?}");
            return ();
        }
    };
    println!("Encode took: {} ms.", stop);
    encode_debug();
}

//Attempts to encode given png image as second argument into qoi
fn encode(args: &Vec<String>) {
    //Path is fetched from arguments
    let path = &args[2];

    //Init png decoder, attempt to decode png into bitmap, throw error if unsuccessful
    let decoder = png::Decoder::new(File::open(path).unwrap());
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

    //encode generated bitmap
    if args.len() >= 4 {
        let filename: &String = &args[3];
        write_to_file(encode_from_image(img), filename).expect("ERROR: Can't write file.");
    } else {
        let mut filename = path.clone();
        for _i in 0..4 {
            filename.pop();
        }
        write_to_file(encode_from_image(img), filename.as_str()).expect("ERROR: Can't write file.");
    }
    println!("Encoding successful!");
}

fn decode(args: &Vec<String>) -> io::Result<()> {
    let mut path: String = String::new();
    if args.len() > 2 {
        path.push_str(args[2].as_str());
    } else {
        println!("ERROR: incorrect number of arguments! (specify file to decode!).");
        ()
    }

    let f: File = match File::open(path.as_str()) {
        Ok(f) => f,
        Err(e) => panic!("ERROR: {e:?}"),
    };
    let mut reader = BufReader::new(f);
    let mut bytes: Vec<u8> = Vec::new();

    reader.read_to_end(&mut bytes)?;

    match qoi::qoi_lib::decode(bytes) {
        Ok(_img) => println!("Decoding successful!"),
        Err(err) => panic!("ERROR: {err:?}"),
    }
    Ok(())
}

fn bench(args: &Vec<String>) {
    if args.len() < 4 {
        panic!("ERROR: invalid number of arguments!");
    }

    let start = SystemTime::now();
    encode(args);
    match start.elapsed() {
        Ok(elapsed) => println!("Encode took {} μs", elapsed.as_micros()),
        Err(e) => panic!("ERROR: {e:?}"),
    }
    let mut new_arg: Vec<String> = Vec::new();
    new_arg.push(String::from(""));
    new_arg.push(String::from(""));
    let mut to_push: String = args[3].clone();
    to_push.push_str(".qoi");
    new_arg.push(to_push);
    let start = SystemTime::now();
    decode(&new_arg).expect(
        "ERROR: Unspecified error during io-pipeline. Ensure file path is valid and can be read.",
    );
    match start.elapsed() {
        Ok(elapsed) => println!("Decode took {} μs", elapsed.as_micros()),
        Err(e) => panic!("ERROR: {e:?}"),
    }
}

fn main() {
    //Initialize logger
    init().expect("Failed to initialize logger.");

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("ERROR: no arguments send!");
    }

    match args[1].as_str() {
        "demo" => {
            demo();
        }
        //can only handle pngs for now
        "encode" => {
            encode(&args);
        }
        "decode" => {
            decode(&args).expect("ERROR: Unspecified error during io-pipeline. Ensure file path is valid and can be read.");
        }
        "bench" => {
            bench(&args);
        }
        "help" => {
            println!("qoi supports the following commands: \n encode [IMAGE] (encodes given png-encoded into .qoi) \n decode [IMAGE] decodes given .qoi to .png \n bench [INPUT] [OUTPUT] encodes input .png into .qoi with encoding speed measured in microseconds.")
        }
        _ => {
            panic!("Invalid arguments!")
        }
    }
}
