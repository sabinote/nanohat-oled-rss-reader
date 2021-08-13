mod nanohat;
mod rss;

use std::error::Error;
use rusttype::{Font, Scale};
use image::{DynamicImage, GrayImage, Luma};
use imageproc::drawing::draw_text_mut;
use i2cdev::linux::LinuxI2CDevice;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c)?;
    let mut oled = nanohat::oled::NanoHatOLED::open(i2cdev)?;

    let xml = reqwest::get("https://news.yahoo.co.jp/rss/topics/top-picks.xml")
        .await?
        .text()
        .await?;
    let rss = rss::RSS::new(&xml)?;
    let items = rss.channel.items;
    
    let font = {
        let v =  Vec::from(include_bytes!("./font/misaki_gothic.ttf") as &[u8]);
        Font::try_from_vec(v).unwrap()
    };

    let mut img = GrayImage::new(128, 64);
    draw_text_mut(&mut img, Luma([255]), 0, 0, Scale{x:8.0, y:8.0}, font, rss.channel.title);
    oled.draw_image(DynamicImage::ImageLuma8(img), 0, 0)?;
    Ok(())
}
