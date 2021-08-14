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
    categories: Vec<(String, String)>,
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
            ("主要".to_string(),"https://news.yahoo.co.jp/rss/topics/top-picks.xml".to_string()),
            ("国内".to_string(),"https://news.yahoo.co.jp/rss/topics/domestic.xml".to_string()),
            ("国際".to_string(),"https://news.yahoo.co.jp/rss/topics/world.xml".to_string()),
            ("経済".to_string(),"https://news.yahoo.co.jp/rss/topics/business.xml".to_string()),
            ("エンタメ".to_string(),"https://news.yahoo.co.jp/rss/topics/entertainment.xml".to_string()),
            ("スポーツ".to_string(),"https://news.yahoo.co.jp/rss/topics/sports.xml".to_string()),
            ("IT".to_string(),"https://news.yahoo.co.jp/rss/topics/it.xml".to_string()),
            ("科学".to_string(),"https://news.yahoo.co.jp/rss/topics/science.xml".to_string()),
            ("地域".to_string(),"https://news.yahoo.co.jp/rss/topics/local.xml".to_string()),
        ],
        display_range: 0..8,
        selected: 0,
    };
    let mut state = State::Categories;

    let mut img = GrayImage::new(128, 64);
    for (i, (s, _)) in category_pane.categories[category_pane.display_range].iter().enumerate() {
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


    while let Ok(pressed) = button.pressed().await {
        match state {
            State::Categories => {
                match pressed {
                    [true, _, _] => {
                        if category_pane.selected < 7 {
                            let mut img = GrayImage::new(128, 8);
                            let i = category_pane.display_range.nth(category_pane.selected).unwrap();
                            draw_text_mut(&mut img, Luma([255]), 0, 0, Scale{x:8.0, y:8.0}, &font, &category_pane.categories[i]);
                            oled.draw_image(&img, 0, category_pane.selected as u8);

                            let mut img = GrayImage::new(128, 8);
                            category_pane.selected += 1;
                            let i = category_pane.display_range.nth(category_pane.selected).unwrap();
                            draw_text_mut(&mut img, Luma([0]), 0, 0, Scale{x:8.0, y:8.0}, &font, &category_pane.categories[i]);
                            oled.draw_image(&img, 0, category_pane.selected as u8);
                        }
                    },
                    _ => {
                        ()
                    }
                }
            },
            _ => unimplemented!(),
        }
    }
    Ok(())
}
