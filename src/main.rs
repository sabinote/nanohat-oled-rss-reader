mod nanohat;
mod rss;

use std::error::Error;
use std::ops::Range;
use rusttype::{Font, Scale};
use image::{DynamicImage, GrayImage, Luma, GenericImage};
use image::imageops::colorops::invert;
use imageproc::drawing::draw_text_mut;
use i2cdev::linux::LinuxI2CDevice;


struct CategoryPane {
    categories: Vec<(&str, &str)>,
    display_range: Range<usize>,
    selected: usize,
}

enum State {
    Categories,
    Titles,
    Details,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c)?;
    let mut oled = nanohat::oled::NanoHatOLED::open(i2cdev)?;
    let mut button = nanohat::button::Button::open("/dev/gpiochip0")?;
    let font = {
        let v =  Vec::from(include_bytes!("font/misaki_gothic.ttf") as &[u8]);
        Font::try_from_vec(v).unwrap()
    };

    let mut category_pane = CategoryPane {
        categories: vec![
            ("主要","https://news.yahoo.co.jp/rss/topics/top-picks.xml"),
            ("国内","https://news.yahoo.co.jp/rss/topics/domestic.xml"),
            ("国際","https://news.yahoo.co.jp/rss/topics/world.xml"),
            ("経済","https://news.yahoo.co.jp/rss/topics/business.xml"),
            ("エンタメ","https://news.yahoo.co.jp/rss/topics/entertainment.xml"),
            ("スポーツ","https://news.yahoo.co.jp/rss/topics/sports.xml"),
            ("IT","https://news.yahoo.co.jp/rss/topics/it.xml"),
            ("科学","https://news.yahoo.co.jp/rss/topics/science.xml"),
            ("地域","https://news.yahoo.co.jp/rss/topics/local.xml"),
        ],
        display_range: 0..8,
        selected: 0,
    };
    let mut state = State::Categories;

    let mut img = GrayImage::new(128, 64);
    for (i, (s, _)) in category_pane.categories[display_range].enumerate() {
        if category_pane.selected == i {
            let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
            invert(&mut sub);
            draw_text_mut(&mut img, Luma([0]), 0, (i * 8) as u32, Scale{x:8.0, y:8.0}, &font, s);
        }
        else {
            draw_text_mut(&mut img, Luma([255]), 0, (i * 8) as u32, Scale{x:8.0, y:8.0}, &font, s);
        }
    }
    oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
    Ok(())
}
