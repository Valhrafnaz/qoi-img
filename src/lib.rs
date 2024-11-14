//! # qoi_img
//! `qoi_img` is a bad, from-scratch implementation of the decoder and encoder for the `.qoi` file format as described as on [qoiformat.org](https://qoiformat.org/qoi-specification.pdf).
//! This crate should not be published as better crates are available, e.g. [rapid-qoi](https://crates.io/crates/rapid-qoi).
#![allow(dead_code, unused_variables)]
pub mod qoi_lib {

    use log::{debug, info, Level, LevelFilter, Record, SetLoggerError};
    use std::fmt;
    use std::fs::*;
    use std::io::prelude::*;
    

    use array_init;

    //Custom error for custom error handling
    #[derive(Debug, Clone, PartialEq)]
    pub enum ImgError {
        DataError,
        PixelNumberError,
        DecodeError,
        HeaderError,
    }
    //inherit from base Error
    impl std::error::Error for ImgError {}

    //Output for error handling
    impl fmt::Display for ImgError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ImgError::DataError => {
                    write!(f, "invalid number of bytes (must be devisible by 4)")
                }
                ImgError::PixelNumberError => {
                    write!(f, "number of pixels does not match height and width params")
                }
                ImgError::DecodeError => write!(f, "decoder failed to construct valid image"),
                ImgError::HeaderError => write!(f, "not a valid QOI file header"),
            }
        }
    }

    //boilerplate implementation of the log crate
    struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= Level::Debug
        }
        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                eprintln!("{} - {}", record.level(), record.args());
            }
        }
        fn flush(&self) {}
    }
    //logging boilerplate
    static LOGGER: SimpleLogger = SimpleLogger;
    /// Initialises the logger provided by [log](https://crates.io/crates/log)
    /// # Example
    ///
    /// ```
    /// # use std::error::Error;
    /// # use crate::qoi::qoi_lib::*;
    /// # fn main() -> Result<(), Box<ImgError>> {
    /// init().expect("Failed to initialize logger");
    /// #
    /// # Ok(())
    /// #
    /// # }
    /// ```
    ///
    /// If you want to pass the error on replace the `println!`:
    ///
    /// ```
    /// # use std::error::Error;
    /// # use crate::qoi::qoi_lib::*;
    /// # fn main() -> Result<(), ImgError> {
    /// match init() {
    /// Ok(()) => (),
    /// Err(e) => println!("Logger failed to initialize!")
    /// }
    /// #
    /// # Ok(())
    /// #
    /// # }
    /// ```
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
    }

    /// Custom image struct, which is used to store decoded data. Used by [encode_from_image] to encode the necessary data in bytes. Requires a Vector over [Pixel] values, `Vec<Pixel>`,
    /// which can be generated by [`self::new`] if given byte data. Otherwise, [self.pixels] must be given filled vector.
    /// `height` and `width` are given as u32 (note that qoi encoding does not guarantee functionality for images containing over 4000000 pixels.)
    /// `channels` specifies the number of channels 3 (RGB)  or 4 (RBGA).
    /// `colorspace` specifies whether sRGB or all linear channels are used (0,1);
    /// # Examples
    /// Create a new image via constructor [`Image::new()`];
    /// ```rust
    /// # use std::error::Error;
    /// # use crate::qoi::qoi_lib::*;
    /// # fn main() -> Result<(), Box<ImgError>> {
    ///
    /// let pixels: Vec<u8> = vec![0;1024*1024*4];
    /// let height: u32 = 1024;
    /// let width: u32 = 1024;
    /// let channels: u8 = 4;
    /// let colorspace: u8 = 0;
    /// let img: Image = Image::new(pixels, height, width, channels, colorspace)?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Alternatively, [`Image::from_pixels()`] can be used to create an image from pixel values.
    pub struct Image {
        pixels: Vec<Pixel>,
        height: u32,
        width: u32,
        channels: u8,
        colorspace: u8,
    }

    impl Image {
        //Image constructor, expects an array of u8 pixels values in order, left to right, top to bottom.
        pub fn new(
            data: Vec<u8>,
            height: u32,
            width: u32,
            channels: u8,
            colorspace: u8,
        ) -> Result<Image, ImgError> {
            let alpha: bool;
            if channels == 4 {
                alpha = true;
            } else {
                alpha = false;
            }
            let pixels: Vec<Pixel> = match Image::pixels_from_bytes(data, alpha) {
                Ok(out) => out,
                Err(error) => return Err(error),
            };
            if pixels.len() == (height * width) as usize {
                let out: Image = Image {
                    pixels,
                    height,
                    width,
                    channels,
                    colorspace,
                };
                Ok(out)
            } else {
                Err(ImgError::PixelNumberError)
            }
        }

        pub fn from_pixels(
            pixels: Vec<Pixel>,
            height: u32,
            width: u32,
            channels: u8,
            colorspace: u8,
        ) -> Image {
            let img = Image {
                pixels,
                height,
                width,
                channels,
                colorspace,
            };
            img
        }

        //Expects pixel data in order left to right, top to bottom, with values for rgba in sequential order
        fn pixels_from_bytes(data: Vec<u8>, alpha: bool) -> Result<Vec<Pixel>, ImgError> {
            let mut pixels: Vec<Pixel> = Vec::with_capacity(data.len() / 4);
            if alpha {
                if data.len() % 4 == 0 {
                    for i in 0..data.len() / 4 {
                        pixels.push(Pixel {
                            r: data[i * 4],
                            g: data[i * 4 + 1],
                            b: data[i * 4 + 2],
                            a: data[i * 4 + 3],
                        });
                    }
                    Ok(pixels)
                } else {
                    Err(ImgError::DataError)
                }
            } else {
                if data.len() % 4 == 0 {
                    for i in 0..data.len() / 3 {
                        pixels.push(Pixel {
                            r: data[i * 3],
                            g: data[i * 3 + 1],
                            b: data[i * 3 + 2],
                            a: 255,
                        });
                    }
                    Ok(pixels)
                } else {
                    Err(ImgError::DataError)
                }
            }
            
        }
        pub fn pixels_to_bytes(&self) -> Vec<u8> {
            let mut buf: Vec<u8> = Vec::with_capacity(self.height as usize * self.width as usize * 4 as usize);
            for pixel in &self.pixels {
                buf.push(pixel.r);
                buf.push(pixel.g);
                buf.push(pixel.b);
                buf.push(pixel.a);
            }
            return buf;
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Pixel {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    }

    #[derive(Debug, PartialEq)]
    pub enum ChunkType {
        Run,
        Index,
        Luma,
        Diff,
        RGB,
        RGBA,
    }

    impl Pixel {
        pub fn new(r: u8, g: u8, b: u8, a: u8) -> Pixel {
            Pixel { r, g, b, a }
        }
        fn equals(&self, other: &Pixel) -> bool {
            if (self.r == other.r)
                && (self.g == other.g)
                && (self.b == other.b)
                && (self.a == other.a)
            {
                true
            } else {
                false
            }
        }

        fn equals_rgb(&self, other: &Pixel) -> bool {
            if (self.r == other.r) && (self.g == other.g) && (self.b == other.b) {
                true
            } else {
                false
            }
        }

        //self = curr pixel, other = prev pixel
        pub fn determine_chunk(
            &self,
            other: &Pixel,
            buffer: &Vec<Pixel>,
        ) -> (ChunkType, Option<(u8, u8, u8)>) {
            if self.equals(&other) {
                return (ChunkType::Run, None);
            }

            if self.equals(&buffer[color_hash(&self) as usize]) {
                return (ChunkType::Index, Some((color_hash(&self), 0, 0)));
            }

            if self.a != other.a {
                return (ChunkType::RGBA, None);
            }

            let diff_tuple: (i16, i16, i16) = self.diff(other);
            let dr: i16 = diff_tuple.0;
            let dg: i16 = diff_tuple.1;
            let db: i16 = diff_tuple.2;

            if (dr > -3 && dr < 2) && (dg > -3 && dg < 2) && (db > -3 && db < 2) {
                let dr: u8 = (dr + DIFF_BIAS as i16) as u8;
                let dg: u8 = (dg + DIFF_BIAS as i16) as u8;
                let db: u8 = (db + DIFF_BIAS as i16) as u8;
                return (ChunkType::Diff, Some((dr, dg, db)));
            } else if (dg > -33 && dg < 32)
                && ((dr - dg) > -9)
                && ((dr - dg) < 8)
                && ((db - dg) > -9)
                && ((db - dg) < 8)
            {
                let dg_out: u8 = (dg + LUMA_BIAS_G as i16) as u8;
                let dr_dg: u8 = (dr - dg + LUMA_BIAS_RB as i16) as u8;
                let db_dg: u8 = (db - dg + LUMA_BIAS_RB as i16) as u8;
                return (ChunkType::Luma, Some((dg_out, dr_dg, db_dg)));
            } else {
                return (ChunkType::RGB, None);
            }
        }
        pub fn diff(&self, other: &Pixel) -> (i16, i16, i16) {
            let mut dr: i16;
            let dr_inv: i16;
            let mut dg: i16;
            let dg_inv: i16;
            let mut db: i16;
            let db_inv: i16;

            dr = self.r.wrapping_sub(other.r) as i16;
            dr_inv = other.r.wrapping_sub(self.r) as i16;

            if dr.abs() > dr_inv.abs() {
                dr = dr_inv;
                dr = -dr;
            }

            dg = self.g.wrapping_sub(other.g) as i16;
            dg_inv = other.g.wrapping_sub(self.g) as i16;

            if dg.abs() > dg_inv.abs() {
                dg = dg_inv;
                dg = -dg;
            }

            db = self.b.wrapping_sub(other.b) as i16;
            db_inv = other.b.wrapping_sub(self.b) as i16;

            if db.abs() > db_inv.abs() {
                db = db_inv;
                db = -db;
            }

            (dr, dg, db)
        }
    }

    //Definition of header bytes
    struct Header {
        magic: [char; 4], //magic bytes "qoif"
        width: u32,       //image width in pixels (BE)
        height: u32,      //image height in pixels (BE)
        channels: u8,     // 3 = RGB, 4 = RBGA
        colorspace: u8,   // 0 = sRGB with linear alpha, 1 = all channels linear
    }

    impl Header {
        fn convert_to_bytestream(&self) -> [u8; 14] {
            let mut out: [u8; 14] = [0; 14];

            //First, set magic bytes
            out[0] = self.magic[0] as u8;
            out[1] = self.magic[1] as u8;
            out[2] = self.magic[2] as u8;
            out[3] = self.magic[3] as u8;

            //split width and height into 8-bit chunks
            let width_bytes = self.width.to_be_bytes();
            let height_bytes = self.height.to_be_bytes();

            out[4] = width_bytes[0];
            out[5] = width_bytes[1];
            out[6] = width_bytes[2];
            out[7] = width_bytes[3];
            out[8] = height_bytes[0];
            out[9] = height_bytes[1];
            out[10] = height_bytes[2];
            out[11] = height_bytes[3];

            //Set information bits
            out[12] = self.channels;
            out[13] = self.colorspace;

            out
        }
    }

    //Definition of End of Stream bytes
    #[derive(Debug)]
    struct End {
        bytes: [u8; 8],
    }
    impl End {
        fn new() -> End {
            End {
                bytes: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
            }
        }
    }

    //chunks as defined in the QOI spec
    const QOI_OP_RGB: u8 = 0b1111_1110;
    const QOI_OP_RGBA: u8 = 0b1111_1111;
    const QOI_OP_RUN: u8 = 0b1100_0000;
    const QOI_OP_INDEX: u8 = 0b0000_0000;
    const QOI_OP_DIFF: u8 = 0b0100_0000;
    const QOI_OP_LUMA: u8 = 0b1000_0000;

    //Biases as defined in the QOI spec
    const RUN_BIAS: u8 = 1;

    const DIFF_BIAS: u8 = 2;

    const LUMA_BIAS_G: u8 = 32;
    const LUMA_BIAS_RB: u8 = 8;

    //hash function for assigning buffer indices to stored pixels
    fn color_hash(pixel: &Pixel) -> u8 {
        let store: u32 =
            pixel.r as u32 * 3 + pixel.g as u32 * 5 + pixel.b as u32 * 7 + pixel.a as u32 * 11;
        (store % 64) as u8
    }

    pub fn encode_from_image(img: Image) -> Vec<u8> {
        let mut prev_pixel: Pixel = Pixel {
            r: 0u8,
            b: 0u8,
            g: 0u8,
            a: 255u8,
        };

        let mut prev_buffer: Vec<Pixel> = Vec::with_capacity(64);

        for i in 0..64 {
            let pix: Pixel = Pixel {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            };
            prev_buffer.push(pix);
        }

        let mut encoded_bytes: Vec<u8> = Vec::new();
        let mut run: u64 = 0;

        let head = Header {
            magic: ['q', 'o', 'i', 'f'],
            width: img.width,
            height: img.height,
            channels: img.channels,
            colorspace: img.colorspace,
        };
        let head_stream = head.convert_to_bytestream();

        for i in head_stream {
            encoded_bytes.push(i);
        }

        let mut counter: u64 = 0;

        for pixel in img.pixels {
            counter += 1;
            let chunk: (ChunkType, Option<(u8, u8, u8)>) =
                pixel.determine_chunk(&prev_pixel, &prev_buffer);
            if chunk == (ChunkType::Run, None) {
                run += 1;
                prev_pixel = pixel.clone();
                continue;
            }
            if run > 0 {
                if run > 62 {
                    while run > 0 {
                        if run / 62 > 0 {
                            encoded_bytes.push(QOI_OP_RUN | (62 - RUN_BIAS));
                            run -= 62;
                        } else if run % 62 > 0 {
                            let run_remainder: u8 = run.try_into().unwrap();
                            encoded_bytes.push(QOI_OP_RUN | (run_remainder - RUN_BIAS));
                            run = 0;
                        } else {
                            break;
                        }
                    }
                } else {
                    let run8: u8 = run.try_into().unwrap();
                    encoded_bytes.push(QOI_OP_RUN | (run8 - RUN_BIAS));
                    run = 0;
                }
            }


            match chunk {
                (ChunkType::Index, Some((index, irr1, irr2))) => {
                    encoded_bytes.push(QOI_OP_INDEX | index);
                    prev_pixel = pixel;
                }
                (ChunkType::Diff, Some((dr, dg, db))) => {
                    let mut out: u8 = 0b0000_0000;
                    out = out | db;
                    out = out | (dg << 2);
                    out = out | (dr << 4);
                    encoded_bytes.push(QOI_OP_DIFF | out);
                    prev_pixel = pixel.clone();
                    prev_buffer[color_hash(&pixel) as usize] = pixel;
                }
                (ChunkType::Luma, Some((dg, dr_dg, db_dg))) => {
                    let mut out: [u8; 2] = [0b0000_0000; 2];
                    out[0] |= dg;
                    out[0] |= QOI_OP_LUMA;
                    out[1] |= db_dg;
                    out[1] |= dr_dg << 4;
                    encoded_bytes.push(out[0]);
                    encoded_bytes.push(out[1]);
                    prev_pixel = pixel.clone();
                    prev_buffer[color_hash(&pixel) as usize] = pixel;
                }
                (ChunkType::RGB, None) => {
                    encoded_bytes.push(QOI_OP_RGB);
                    encoded_bytes.push(pixel.r);
                    encoded_bytes.push(pixel.g);
                    encoded_bytes.push(pixel.b);
                    prev_pixel = pixel.clone();
                    prev_buffer[color_hash(&pixel) as usize] = pixel;
                }
                (ChunkType::RGBA, None) => {
                    if (pixel.a as i16 - prev_pixel.a as i16) == 0i16 {
                        //this should never be reached, but it is
                        encoded_bytes.push(QOI_OP_RGB);
                        encoded_bytes.push(pixel.r);
                        encoded_bytes.push(pixel.g);
                        encoded_bytes.push(pixel.b);
                        prev_pixel = pixel.clone();
                        prev_buffer[color_hash(&pixel) as usize] = pixel;
                    } else {
                        encoded_bytes.push(QOI_OP_RGBA);
                        encoded_bytes.push(pixel.r);
                        encoded_bytes.push(pixel.g);
                        encoded_bytes.push(pixel.b);
                        encoded_bytes.push(pixel.a);
                        prev_pixel = pixel.clone();
                        prev_buffer[color_hash(&pixel) as usize] = pixel;
                    }
                }
                _ => panic!(
                    "Critical error at encoding stage: Illegal output from difference function."
                ),
            }
        }

        if run > 0 {
            if run > 62 {
                while run > 0 {
                    if run / 62 > 0 {
                        encoded_bytes.push(QOI_OP_RUN | (62 - RUN_BIAS));
                        run -= 62;
                    } else if run % 62 > 0 {
                        let run_remainder: u8 = run.try_into().unwrap();
                        encoded_bytes.push(QOI_OP_RUN | (run_remainder - RUN_BIAS));
                        run = 0;
                    } else {
                        break;
                    }
                }
            } else {
                let run8: u8 = run.try_into().unwrap();
                encoded_bytes.push(QOI_OP_RUN | (run8 - RUN_BIAS));
                // run = 0;
            }
        }

        let end_bytes = End::new();
        for i in end_bytes.bytes {
            encoded_bytes.push(i)
        }

        info!("Number of pixels processed: {}.", counter);
        info!(
            "Number of bytes in encoding: {:?}.",
            encoded_bytes.len() - 22
        );
        info!(
            "Compression rate: {:.2}%.",
            (1.0 - (encoded_bytes.len() - 22) as f64 / (counter * 4) as f64) * 100.0
        );

        encoded_bytes
    }

    pub fn write_to_file(bytes: Vec<u8>, filename: &str) -> std::io::Result<()> {
        let mut file_path: String = String::from(filename);
        file_path.push_str(".qoi");

        let mut buffer = File::create(file_path)?;
        let mut pos = 0;

        while pos < bytes.len() {
            let bytes_written = buffer.write(&bytes[pos..])?;
            pos += bytes_written;
        }
        Ok(())
    }

    fn read_header(bytes: &[u8]) -> Result<(u32, u32, u8, u8), ImgError> {
        if bytes[0] == 'q' as u8
            && bytes[1] == 'o' as u8
            && bytes[2] == 'i' as u8
            && bytes[3] == 'f' as u8
        {
            let mut width: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
            let mut height: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
            width |= ((bytes[4] as u32) << 24) as u32;
            width |= ((bytes[5] as u32) << 16) as u32;
            width |= ((bytes[6] as u32) << 8) as u32;
            width |= (bytes[7]) as u32;
            height |= ((bytes[8] as u32) << 24) as u32;
            height |= ((bytes[9] as u32) << 16) as u32;
            height |= ((bytes[10] as u32) << 8) as u32;
            height |= (bytes[11]) as u32;
            return Ok((width, height, bytes[12], bytes[13]));
        } else {
            return Err(ImgError::HeaderError);
        }
    }

    fn read_tag(tag: u8) -> Result<ChunkType, ImgError> {
        if tag == QOI_OP_RGB {
            return Ok(ChunkType::RGB);
        }
        if tag == QOI_OP_RGBA {
            return Ok(ChunkType::RGBA);
        }
        if (tag & 0b1100_0000) == QOI_OP_DIFF {
            return Ok(ChunkType::Diff);
        }
        if (tag & 0b1100_0000) == QOI_OP_INDEX {
            return Ok(ChunkType::Index);
        }
        if (tag & 0b1100_0000) == QOI_OP_LUMA {
            return Ok(ChunkType::Luma);
        }
        if (tag & 0b1100_0000) == QOI_OP_RUN {
            return Ok(ChunkType::Run);
        }
        return Err(ImgError::DecodeError);
    }

    fn dec_rgb(bytes: &[u8], alpha: u8) -> Pixel {
        let pixel: Pixel = Pixel::new(bytes[1], bytes[2], bytes[3], alpha);
        pixel
    }

    fn dec_rgba(bytes: &[u8]) -> Pixel {
        let pixel: Pixel = Pixel::new(bytes[1], bytes[2], bytes[3], bytes[4]);
        pixel
    }

    fn dec_diff(byte: u8, prev_pixel: &Pixel) -> Pixel {
        let dr: u8;
        let dg: u8;
        let db: u8;

        dr = (byte & 0b00110000) >> 4;
        dg = (byte & 0b00001100) >> 2;
        db = byte & 0b00000011;

        let r: u8 = prev_pixel.r.wrapping_add(dr);
        let g: u8 = prev_pixel.g.wrapping_add(dg);
        let b: u8 = prev_pixel.b.wrapping_add(db);

        let r: u8 = r.wrapping_sub(DIFF_BIAS);
        let b: u8 = b.wrapping_sub(DIFF_BIAS);
        let g: u8 = g.wrapping_sub(DIFF_BIAS);

        let pixel: Pixel = Pixel::new(r, g, b, prev_pixel.a);
        pixel
    }

    fn dec_luma(bytes: &[u8], prev_pixel: &Pixel) -> Pixel {
        let dr: u8;
        let dr_dg: u8;
        let db_dg: u8;
        let dg: u8;
        let db: u8;

        dg = bytes[0] & 0b00111111;
        dr_dg = (bytes[1] & 0b11110000) >> 4;
        db_dg = bytes[1] & 0b00001111;
        dr = dr_dg + dg;
        db = db_dg + dg;

        let r: u8 = prev_pixel.r.wrapping_add(dr);
        let g: u8 = prev_pixel.g.wrapping_add(dg);
        let b: u8 = prev_pixel.b.wrapping_add(db);

        let r: u8 = r.wrapping_sub(LUMA_BIAS_RB + LUMA_BIAS_G);
        let g: u8 = g.wrapping_sub(LUMA_BIAS_G);
        let b: u8 = b.wrapping_sub(LUMA_BIAS_RB + LUMA_BIAS_G);

        let pixel: Pixel = Pixel::new(r, g, b, prev_pixel.a);
        pixel
    }

    fn dec_run() {}

    pub fn decode(mut bytes: Vec<u8>) -> Result<Image, ImgError> {
        let width: u32;
        let height: u32;
        let channels: u8;
        let colorspace: u8;

        let mut prev_pixel: Pixel = Pixel {
            r: 0u8,
            g: 0u8,
            b: 0u8,
            a: 255u8,
        };

        let mut prev_buffer: [Pixel; 64] = array_init::array_init(|_| Pixel::new(0, 0, 0, 0));

        match read_header(&bytes[0..14]) {
            Ok((w, h, ch, c)) => {
                width = w;
                height = h;
                channels = ch;
                colorspace = c;
            }
            Err(err) => {
                return Err(err);
            }
        }
        let mut pixels: Vec<Pixel> = Vec::with_capacity((width * height * 4) as usize);

        if bytes[bytes.len() - 1] == 1 {
            for i in 2..9 {
                if bytes[bytes.len() - i] != 0 {
                    debug!("Ending bytes not present.");
                    return Err(ImgError::DecodeError);
                }
            }
            for i in 0..8 {
                bytes.pop();
            }
        } else {
            debug!("Ending bytes not present.");
            return Err(ImgError::DecodeError);
        }

        let mut i: usize = 14;

        while i < bytes.len() {
            match read_tag(bytes[i]) {
                Ok(tag) => match tag {
                    ChunkType::RGB => {
                        let dec_pix: Pixel = dec_rgb(&bytes[i..i + 4], prev_pixel.a);
                        prev_pixel = dec_pix.clone();
                        prev_buffer[color_hash(&dec_pix) as usize] = dec_pix.clone();
                        pixels.push(dec_pix);
                        i += 3;
                    }
                    ChunkType::RGBA => {
                        let dec_pix: Pixel = dec_rgba(&bytes[i..i + 5]);
                        prev_pixel = dec_pix.clone();
                        prev_buffer[color_hash(&dec_pix) as usize] = dec_pix.clone();
                        pixels.push(dec_pix);
                        i += 4;
                    }
                    ChunkType::Diff => {
                        let dec_pix: Pixel = dec_diff(bytes[i], &prev_pixel);
                        prev_pixel = dec_pix.clone();
                        prev_buffer[color_hash(&dec_pix) as usize] = dec_pix.clone();
                        pixels.push(dec_pix);
                    }
                    ChunkType::Index => {
                        let dec_pix: Pixel = prev_buffer[bytes[i] as usize];
                        prev_pixel = dec_pix.clone();
                        prev_buffer[color_hash(&dec_pix) as usize] = dec_pix.clone();
                        pixels.push(dec_pix);
                    }
                    ChunkType::Luma => {
                        let dec_pix: Pixel = dec_luma(&bytes[i..i + 2], &prev_pixel);
                        prev_pixel = dec_pix.clone();
                        prev_buffer[color_hash(&dec_pix) as usize] = dec_pix.clone();
                        pixels.push(dec_pix);
                        i += 1;
                    }
                    ChunkType::Run => {
                        let length: u8 = (bytes[i] & 0b00111111) + RUN_BIAS;
                        for j in 0..length {
                            pixels.push(prev_pixel.clone());
                        }
                        prev_buffer[color_hash(&prev_pixel) as usize] = prev_pixel.clone();
                    }
                },
                Err(err) => return Err(err),
            }
            i += 1;
        }

        if pixels.len() as u32 != height * width {
            debug!("h*w: {}", height * width);
            debug!("n pixels: {}", pixels.len());
            return Err(ImgError::DecodeError);
        }

        let img = Image::from_pixels(pixels, height, width, channels, colorspace);
        Ok(img)
    }

    #[cfg(test)]
    mod tests {

        use png::ColorType;

        use super::*;
        use std::io;
        use std::io::{BufReader, Read};
        use std::path::*;

        #[test]
        fn diff_test() {
            init().expect("Logger initialisation failed!");
            let pix1: Pixel = Pixel::new(0, 0, 0, 255);
            let pix2: Pixel = Pixel::new(255, 255, 255, 255);

            let pix3: Pixel = Pixel::new(155, 155, 155, 255);
            let pix4: Pixel = Pixel::new(160, 160, 160, 255);

            assert_eq!(pix1.diff(&pix2), (1, 1, 1));
            assert_eq!(pix2.diff(&pix1), (-1, -1, -1));
            assert_eq!(pix4.diff(&pix3), (5, 5, 5));
            assert_eq!(pix3.diff(&pix4), (-5, -5, -5));
        }

        #[test]
        fn qoi_to_qoi_test() -> io::Result<()> {
            //Open path to test images
            let path: &Path = Path::new("./qoi_test_images/");
            let dir: ReadDir = match path.read_dir() {
                Ok(d) => d,
                Err(e) => panic!("Error reading path {e:?}"),
            };
            //Loop over files in directory, attempt to decode .qoi images and reencode 
            for entry in dir {

                let file_path = match entry {
                    Ok(d) => d.path(),
                    Err(e) => panic!("Non-functional dir entry! \n {e:?}")
                };
                let file_path_str = match file_path.to_str() {
                    Some(s) => s,
                    None => ""
                };
                if !(file_path_str.contains(".qoi")) {
                    continue;
                }
                
                let file = match File::open(&file_path) {
                    Ok(f) => f,
                    Err(e) => panic!("Error reading file with path {:?}", file_path_str),
                };
                let mut reader = BufReader::new(file);
                let mut bytes: Vec<u8> = Vec::new();

                reader.read_to_end(&mut bytes)?;

                let output_image: super::Image;
                match super::decode(bytes) {
                    Ok(img) => output_image = img,
                    Err(err) => panic!("Image decode failed for {:?}" , file_path.to_str())
                }
                let mut name = match file_path.file_name() {
                    Some(s) => match s.to_str() {
                        Some(ss) => ss,
                        None => panic!("File Name Error!")
                    },
                    None => panic!("File Name Error!"),
                };
                name = match name.strip_suffix(".qoi") {
                    Some(n) => n,
                    None => name,
                };
                write_to_file(encode_from_image(output_image), name).expect("Writing image failed!");
            }
            
            Ok(())
        }

        #[test]
        fn png_to_qoi_test() -> io::Result<()> {
            //Open path to test images
            let path: &Path = Path::new("./qoi_test_images/");
            let dir: ReadDir = match path.read_dir() {
                Ok(d) => d,
                Err(e) => panic!("Error reading path {e:?}"),
            };
            //Loop over files in directory, attempt to decode png and encode as qoi, compare to qoi
            for entry in dir {

                let file_path = match entry {
                    Ok(d) => d.path(),
                    Err(e) => panic!("Non-functional dir entry! \n {e:?}")
                };
                let file_path_str = match file_path.to_str() {
                    Some(s) => s,
                    None => ""
                };
                if !(file_path_str.contains(".png")) {
                    continue;
                }
                println!("{:}",file_path_str);
                let file = match File::open(&file_path) {
                    Ok(f) => f,
                    Err(e) => panic!("Cannot read file! \n {e:?}")
                };
                let decoder = png::Decoder::new(file);
                let mut reader = match decoder.read_info() {
                    Ok(reader) => reader,
                    Err(e) => panic!("ERROR: couldn't decode file: {e:}"),
                };
                //read image metadata
                let width: u32 = reader.info().width;
                let height: u32 = reader.info().height;
                //for now: hardcoded to 4
                let channels = match reader.info().color_type {
                    ColorType::Rgb => 3,
                    ColorType::Rgba => 4,
                    _ => panic!("ERROR: Incompatible png file!")
                };

                //create buffer matching the size of png-decoder output, writing size to output
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = match reader.next_frame(&mut buf) {
                    Ok(i) => i,
                    Err(e) => panic!("ERROR: {e:?}"),
                };
                let bytes = &buf[..info.buffer_size()];
                let byte_vec: Vec<u8> = bytes.to_vec();

                //create bitmap data from raw byte vector
                let img: Image = match Image::new(byte_vec, height, width, channels, 0) {
                    Ok(image) => image,
                    Err(err) => panic!("Problem generating image: {:?}", err),
                };

                let encoded_buffer = super::encode_from_image(img);
                let mut name =  match file_path.file_name() {
                    None => panic!("whoops!"),
                    Some(n) => match n.to_str() {
                        None => panic!("im shiddin"),
                        Some(s) => s, 
                    },
                };
                name = match name.strip_suffix(".png") {
                    Some(n) => n,
                    None => name,
                };
                write_to_file(encoded_buffer,name ).expect("Can't write resulting file!");
            }
            
            Ok(())
        }

        #[test]
        fn tag_test() {
            //init().expect("Logger initialisation failed!");
            let test_rgb: u8 = 0b1111_1110;
            let test_rgba: u8 = 0b1111_1111;
            let test_luma: u8 = 0b1011_1010;
            let test_run: u8 = 0b1110_1101;
            let test_diff: u8 = 0b0110_1010;
            let test_index: u8 = 0b0010_1010;

            assert_eq!(Ok(ChunkType::RGB), super::read_tag(test_rgb));
            assert_eq!(Ok(ChunkType::RGBA), super::read_tag(test_rgba));
            assert_eq!(Ok(ChunkType::Luma), super::read_tag(test_luma));
            assert_eq!(Ok(ChunkType::Diff), super::read_tag(test_diff));
            assert_eq!(Ok(ChunkType::Index), super::read_tag(test_index));
            assert_eq!(Ok(ChunkType::Run), super::read_tag(test_run));
        }

        #[test]
        fn sub_decoders_test() {
            //init().expect("Logger initialisation failed!");
            let pix: Pixel = Pixel {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            };
            let prev: Pixel = Pixel {
                r: 1,
                g: 1,
                b: 1,
                a: 255,
            };
            let byte: u8 = 0b01000000;

            assert_eq!(pix, dec_diff(byte, &prev));

            let pix: Pixel = Pixel {
                r: 17,
                g: 22,
                b: 28,
                a: 100,
            };
            let prev: Pixel = Pixel {
                r: 10,
                g: 10,
                b: 10,
                a: 100,
            };
            let byte: [u8; 2] = [0b10101100, 0b00111110];

            assert_eq!(pix, dec_luma(&byte[0..2], &prev));
        }
    }
}
