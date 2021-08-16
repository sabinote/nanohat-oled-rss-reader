mod nanohat;
mod rss;

use i2cdev::linux::LinuxI2CDevice;
use image::imageops::colorops::invert;
use image::{DynamicImage, GenericImage, GrayImage, Luma};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::error::Error;
use std::ops::Range;

struct CategoryPane {
    categories: Vec<(String, String)>,
    display_range: Range<usize>,
    selected: usize,
}

struct TitlePane {
    items: Vec<rss::Item>,
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
        let v = Vec::from(include_bytes!("font/misaki_gothic.ttf") as &[u8]);
        Font::try_from_vec(v).unwrap()
    };

    let mut category_pane = CategoryPane {
        categories: vec![
            (
                "主要".to_string(),
                "https://news.yahoo.co.jp/rss/topics/top-picks.xml".to_string(),
            ),
            (
                "国内".to_string(),
                "https://news.yahoo.co.jp/rss/topics/domestic.xml".to_string(),
            ),
            (
                "国際".to_string(),
                "https://news.yahoo.co.jp/rss/topics/world.xml".to_string(),
            ),
            (
                "経済".to_string(),
                "https://news.yahoo.co.jp/rss/topics/business.xml".to_string(),
            ),
            (
                "エンタメ".to_string(),
                "https://news.yahoo.co.jp/rss/topics/entertainment.xml".to_string(),
            ),
            (
                "スポーツ".to_string(),
                "https://news.yahoo.co.jp/rss/topics/sports.xml".to_string(),
            ),
            (
                "IT".to_string(),
                "https://news.yahoo.co.jp/rss/topics/it.xml".to_string(),
            ),
            (
                "科学".to_string(),
                "https://news.yahoo.co.jp/rss/topics/science.xml".to_string(),
            ),
            (
                "地域".to_string(),
                "https://news.yahoo.co.jp/rss/topics/local.xml".to_string(),
            ),
        ],
        display_range: 0..8,
        selected: 0,
    };

    let mut title_pane = TitlePane {
        items: Vec::new(),
        display_range: 0..1,
        selected: 0,
    };

    let mut state = State::Categories;

    let mut img = GrayImage::new(128, 64);
    for (i, (s, _)) in category_pane.categories
        [category_pane.display_range.start..category_pane.display_range.end]
        .iter()
        .enumerate()
    {
        if category_pane.selected == i {
            let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
            invert(&mut sub);
            draw_text_mut(
                &mut img,
                Luma([0]),
                0,
                (i * 8) as u32,
                Scale { x: 8.0, y: 8.0 },
                &font,
                s,
            );
        } else {
            draw_text_mut(
                &mut img,
                Luma([255]),
                0,
                (i * 8) as u32,
                Scale { x: 8.0, y: 8.0 },
                &font,
                s,
            );
        }
    }
    oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;

    while let Ok(pressed) = button.pressed().await {
        match state {
            State::Categories => {
                match pressed {
                    [true, false, false] => {
                        if category_pane.selected < 7 {
                            let mut img = GrayImage::new(128, 8);
                            let i = category_pane.display_range.start + category_pane.selected;
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &category_pane.categories[i].0,
                            );
                            //oled.set_draw_range(0, category_pane.selected as u8, 128, 8)?;
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                category_pane.selected as u8,
                            )?;

                            let mut img = GrayImage::new(128, 8);
                            invert(&mut img);
                            category_pane.selected += 1;
                            let i = category_pane.display_range.start + category_pane.selected;
                            draw_text_mut(
                                &mut img,
                                Luma([0]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &category_pane.categories[i].0,
                            );
                            //oled.set_draw_range(0, category_pane.selected as u8, 128, 8)?;
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                category_pane.selected as u8,
                            )?;
                        } else if category_pane.display_range.end < category_pane.categories.len() {
                            let mut img = GrayImage::new(128, 64);
                            let start = category_pane.display_range.start + 1;
                            let end = category_pane.display_range.end + 1;
                            category_pane.display_range = start..end;
                            let i = 7;
                            category_pane.selected = 7;
                            for (i, (s, _)) in
                                category_pane.categories[start..end].iter().enumerate()
                            {
                                if category_pane.selected == i {
                                    let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                                    invert(&mut sub);
                                    draw_text_mut(
                                        &mut img,
                                        Luma([0]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        s,
                                    );
                                } else {
                                    draw_text_mut(
                                        &mut img,
                                        Luma([255]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        s,
                                    );
                                }
                            }
                            oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                        } else {
                        }
                    }
                    [false, false, true] => {
                        if category_pane.selected > 0 {
                            let mut img = GrayImage::new(128, 8);
                            let i = category_pane.display_range.start + category_pane.selected;
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &category_pane.categories[i].0,
                            );
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                category_pane.selected as u8,
                            )?;

                            let mut img = GrayImage::new(128, 8);
                            invert(&mut img);
                            category_pane.selected -= 1;
                            let i = category_pane.display_range.start + category_pane.selected;

                            draw_text_mut(
                                &mut img,
                                Luma([0]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &category_pane.categories[i].0,
                            );
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                category_pane.selected as u8,
                            )?;
                        } else if category_pane.display_range.start > 0 {
                            let mut img = GrayImage::new(128, 64);
                            let start = category_pane.display_range.start - 1;
                            let end = category_pane.display_range.end - 1;
                            category_pane.display_range = start..end;
                            let i = 0;
                            category_pane.selected = 0;
                            for (i, (s, _)) in
                                category_pane.categories[start..end].iter().enumerate()
                            {
                                if category_pane.selected == i {
                                    let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                                    invert(&mut sub);
                                    draw_text_mut(
                                        &mut img,
                                        Luma([0]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        s,
                                    );
                                } else {
                                    draw_text_mut(
                                        &mut img,
                                        Luma([255]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        s,
                                    );
                                }
                            }
                            oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                        }
                    }
                    [false, true, false] => {
                        let i = category_pane.display_range.start + category_pane.selected;
                        let url = &category_pane.categories[i].1;
                        let s = reqwest::get(url).await?.text().await?;
                        let rss = rss::RSS::new(&s)?;

                        let end = {
                            let len = rss.channel.items.len();
                            if len > 8 {
                                8
                            } else {
                                len
                            }
                        };
                        title_pane = TitlePane {
                            items: rss.channel.items,
                            display_range: 0..end,
                            selected: 0,
                        };

                        let mut img = GrayImage::new(128, 64);
                        for (i, item) in title_pane.items
                            [title_pane.display_range.start..title_pane.display_range.end]
                            .iter()
                            .enumerate()
                        {
                            if title_pane.selected == i {
                                let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                                invert(&mut sub);
                                draw_text_mut(
                                    &mut img,
                                    Luma([0]),
                                    0,
                                    (i * 8) as u32,
                                    Scale { x: 8.0, y: 8.0 },
                                    &font,
                                    &item.title,
                                );
                            } else {
                                draw_text_mut(
                                    &mut img,
                                    Luma([255]),
                                    0,
                                    (i * 8) as u32,
                                    Scale { x: 8.0, y: 8.0 },
                                    &font,
                                    &item.title,
                                );
                            }
                        }
                        state = State::Titles;
                        oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                    }
                    _ => (),
                }
            }
            State::Titles => {
                match pressed {
                    [true, false, false] => {
                        if title_pane.selected < 7 {
                            let mut img = GrayImage::new(128, 8);
                            let i = title_pane.display_range.start + title_pane.selected;
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &title_pane.items[i].title,
                            );
                            //oled.set_draw_range(0, category_pane.selected as u8, 128, 8)?;
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                title_pane.selected as u8,
                            )?;

                            let mut img = GrayImage::new(128, 8);
                            invert(&mut img);
                            title_pane.selected += 1;
                            let i = title_pane.display_range.start + title_pane.selected;
                            draw_text_mut(
                                &mut img,
                                Luma([0]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &title_pane.items[i].title,
                            );
                            //oled.set_draw_range(0, category_pane.selected as u8, 128, 8)?;
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                title_pane.selected as u8,
                            )?;
                        } else if title_pane.display_range.end < title_pane.items.len() {
                            let mut img = GrayImage::new(128, 64);
                            let start = title_pane.display_range.start + 1;
                            let end = title_pane.display_range.end + 1;
                            title_pane.display_range = start..end;
                            let i = 7;
                            title_pane.selected = 7;
                            for (i, item) in title_pane.items[start..end].iter().enumerate() {
                                if title_pane.selected == i {
                                    let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                                    invert(&mut sub);
                                    draw_text_mut(
                                        &mut img,
                                        Luma([0]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        &item.title,
                                    );
                                } else {
                                    draw_text_mut(
                                        &mut img,
                                        Luma([255]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        &item.title,
                                    );
                                }
                            }
                            oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                        } else {
                        }
                    }
                    [false, false, true] => {
                        if title_pane.selected > 0 {
                            let mut img = GrayImage::new(128, 8);
                            let i = title_pane.display_range.start + title_pane.selected;
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &title_pane.items[i].title,
                            );
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                title_pane.selected as u8,
                            )?;

                            let mut img = GrayImage::new(128, 8);
                            invert(&mut img);
                            title_pane.selected -= 1;
                            let i = title_pane.display_range.start + title_pane.selected;

                            draw_text_mut(
                                &mut img,
                                Luma([0]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &title_pane.items[i].title,
                            );
                            oled.draw_image(
                                &DynamicImage::ImageLuma8(img),
                                0,
                                title_pane.selected as u8,
                            )?;
                        } else if title_pane.display_range.start > 0 {
                            let mut img = GrayImage::new(128, 64);
                            let start = title_pane.display_range.start - 1;
                            let end = title_pane.display_range.end - 1;
                            title_pane.display_range = start..end;
                            let i = 0;
                            title_pane.selected = 0;
                            for (i, item) in title_pane.items[start..end].iter().enumerate() {
                                if title_pane.selected == i {
                                    let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                                    invert(&mut sub);
                                    draw_text_mut(
                                        &mut img,
                                        Luma([0]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        &item.title,
                                    );
                                } else {
                                    draw_text_mut(
                                        &mut img,
                                        Luma([255]),
                                        0,
                                        (i * 8) as u32,
                                        Scale { x: 8.0, y: 8.0 },
                                        &font,
                                        &item.title,
                                    );
                                }
                            }
                            oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                        }
                    }
                    [true, false, true] => {
                        let mut img = GrayImage::new(128, 64);
                        for (i, (s, _)) in category_pane.categories
                            [category_pane.display_range.start..category_pane.display_range.end]
                            .iter()
                            .enumerate()
                        {
                            if category_pane.selected == i {
                                let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                                invert(&mut sub);
                                draw_text_mut(
                                    &mut img,
                                    Luma([0]),
                                    0,
                                    (i * 8) as u32,
                                    Scale { x: 8.0, y: 8.0 },
                                    &font,
                                    s,
                                );
                            } else {
                                draw_text_mut(
                                    &mut img,
                                    Luma([255]),
                                    0,
                                    (i * 8) as u32,
                                    Scale { x: 8.0, y: 8.0 },
                                    &font,
                                    s,
                                );
                            }
                        }
                        oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                        state = State::Categories;
                    }
                    [false, true, false] => {
                        let i = title_pane.display_range.start + title_pane.selected;
                        let s = &title_pane.items[i].description;

                        let (mut v, s, _) = s.chars().fold(
                            (Vec::new(), String::new(), 0),
                            |(mut v, mut s, mut column_count), c| {
                                let width = if c.is_ascii() { 4 } else { 8 };
                                if column_count + width > 128 {
                                    v.push(s);
                                    s = String::new();
                                    s.push(c);
                                    column_count = width;
                                } else {
                                    s.push(c);
                                    column_count += width;
                                }
                                (v, s, column_count)
                            },
                        );
                        v.push(s);
                        let mut img = GrayImage::new(128, 64);
                        for (i, s) in v.into_iter().enumerate() {
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                (i * 8) as u32,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &s,
                            );
                        }
                        oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                        state = State::Details;
                    }
                    _ => (),
                }
            }
            State::Titles => match pressed {
                [true, false, true] => {
                    let mut img = GrayImage::new(128, 64);
                    for (i, item) in title_pane.items
                        [title_pane.display_range.start..title_pane.display_range.end]
                        .iter()
                        .enumerate()
                    {
                        if title_pane.selected == i {
                            let mut sub = img.sub_image(0, (i * 8) as u32, 128, 8);
                            invert(&mut sub);
                            draw_text_mut(
                                &mut img,
                                Luma([0]),
                                0,
                                (i * 8) as u32,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &item.title,
                            );
                        } else {
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                (i * 8) as u32,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &item.title,
                            );
                        }
                    }
                    state = State::Titles;
                    oled.draw_image(&DynamicImage::ImageLuma8(img), 0, 0)?;
                }
                _ => (),
            },
            _ => unimplemented!(),
        }
    }
    Ok(())
}
