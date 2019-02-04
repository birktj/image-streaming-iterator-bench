mod allocator;

#[global_allocator]
static A: allocator::Counter = allocator::Counter;

use std::io::Read;
use std::fs::File;
use image::{Pixel, ColorType, ImageDecoder, Rgb, Rgba, Luma, Bgr, ImageBuffer};

fn pixels_dynamic<D: ImageDecoder>(decoder: D) -> PixelsDynamic<D::Reader> {
    let colortype = decoder.colortype();
    let buffer = vec![0; DynamicPixel::bytes(&colortype) as usize];
    let reader = decoder.into_reader().unwrap();

    PixelsDynamic {
        reader,
        buffer,
        colortype
    }
}

struct PixelsDynamic<R> {
    reader: R,
    buffer: Vec<u8>,
    colortype: ColorType,
}

impl<R: Read> Iterator for PixelsDynamic<R> {
    type Item = DynamicPixel; // Maybe Result<Box<dyn Pixels>> instead?

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.read_exact(&mut self.buffer).ok()?;
        Some(DynamicPixel::from_slice(&self.colortype, &self.buffer))
    }
}

fn pixels_rgb8<R: Read>(reader: R) -> PixelsRGB8<R> {
    PixelsRGB8 {
        reader
    }
}

struct PixelsRGB8<R> {
    reader: R,
}

impl<R: Read> Iterator for PixelsRGB8<R> {
    type Item = Rgb<u8>; // Maybe Result<Box<dyn Pixels>> instead?

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 3];
        self.reader.read_exact(&mut buffer).ok()?;
        Some(Rgb {data: buffer})
    }
}

enum DynamicPixel {
    Rgb8(Rgb<u8>),
    Rgba8(Rgba<u8>),
    Luma8(Luma<u8>),
    Bgr8(Bgr<u8>),
}

impl DynamicPixel {
    fn bytes(colortype: &ColorType) -> u8 {
        match colortype {
            ColorType::RGB(8) => 3,
            ColorType::RGBA(8) => 4,
            ColorType::Gray(8) => 1,
            ColorType::BGR(8) => 3,
            _ => panic!("Unsuported"),
        }
    }

    fn from_slice(colortype: &ColorType, slice: &[u8]) -> DynamicPixel {
        match colortype {
            ColorType::RGB(8) => DynamicPixel::Rgb8(Rgb::from_slice(slice).to_owned()),
            ColorType::RGBA(8) => DynamicPixel::Rgba8(Rgba::from_slice(slice).to_owned()),
            ColorType::Gray(8) => DynamicPixel::Luma8(Luma::from_slice(slice).to_owned()),
            ColorType::BGR(8) => DynamicPixel::Bgr8(Bgr::from_slice(slice).to_owned()),
            _ => panic!("Unsuported"),
        }
    }
}

impl DynamicPixel {
    fn to_rgb(&self) -> Rgb<u8> {
        match self {
            DynamicPixel::Rgb8(x) => x.to_rgb(),
            DynamicPixel::Rgba8(x) => x.to_rgb(),
            DynamicPixel::Luma8(x) => x.to_rgb(),
            DynamicPixel::Bgr8(x) => x.to_rgb(),
        }
    }
}

fn pixel_sum<I: Iterator<Item = Rgb<u8>>>(iter: I) -> [u32; 3] {
    iter.fold([0; 3], |mut acc, x| {
        acc[0] += x.data[0] as u32;
        acc[1] += x.data[1] as u32;
        acc[2] += x.data[2] as u32;
        acc
    })
}

fn pixel_sum_borrowed<'a, I: Iterator<Item = &'a Rgb<u8>>>(iter: I) -> [u32; 3] {
    iter.fold([0; 3], |mut acc, x| {
        acc[0] += x.data[0] as u32;
        acc[1] += x.data[1] as u32;
        acc[2] += x.data[2] as u32;
        acc
    })
}

fn bench<F: Fn()>(name: &str, f: F) {
    println!("{}:", name);
    allocator::reset_allocator();
    let t0 = std::time::SystemTime::now();
    f();
    let mem_max = allocator::get_max_allocated();
    let duration = t0.elapsed().unwrap();

    println!("time used: {:?}, memory allocated (in kb): {}\n", duration, mem_max/1024);
}

fn load_image_incremental_static(path: &str) {
    let file = File::open(path).unwrap();
    let image = image::png::PNGDecoder::new(file).unwrap();

    let pixels = pixels_rgb8(image.into_reader().unwrap());

    println!("pixel sum: {:?}", pixel_sum(pixels));
}

fn load_image_incremental_dynamic(path: &str) {
    let file = File::open(path).unwrap();
    let image = image::png::PNGDecoder::new(file).unwrap();

    let pixels = pixels_dynamic(image);

    println!("pixel sum: {:?}", pixel_sum(pixels.map(|x| x.to_rgb())));
}

fn load_image_buffer(path: &str) {
    let file = File::open(path).unwrap();
    let image = image::png::PNGDecoder::new(file).unwrap();
    
    let (width, height) = image.dimensions();

    let buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width as u32, height as u32, image.read_image().unwrap()).unwrap();

    let pixels = buffer.pixels();

    println!("pixel sum: {:?}", pixel_sum_borrowed(pixels));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img_path = &args[1];

    bench("Buffer", || {
        load_image_buffer(img_path);
    });

    bench("Incremental static", || {
        load_image_incremental_static(img_path);
    });

    bench("Incremental dynamic", || {
        load_image_incremental_dynamic(img_path);
    });
}
