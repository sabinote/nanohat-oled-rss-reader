mod nanohat;
mod rss;

use i2cdev::linux::LinuxI2CDevice;
use image::imageops::colorops::invert;
use image::imageops::overlay;
use image::{DynamicImage, GenericImage, GenericImageView, GrayImage, Luma};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::error::Error;
use std::ops::Range;

struct CategoryPane {
    categories: Vec<GrayImage>,
    urls: Vec<&'static str>,
    start_i: usize,
    selected: usize,
}

struct TitlePane {
    titles: Vec<GrayImage>,
    descriptions: Vec<String>,
    start_i: usize,
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

    let categories = [
        "主要",
        "国内",
        "国際",
        "経済",
        "エンタメ",
        "スポーツ",
        "IT",
        "科学",
        "地域",
    ]
    .into_iter()
    .map(|category| {
        let mut img = GrayImage::new(128, 8);
        draw_text_mut(
            &mut img,
            Luma([255]),
            0,
            0,
            Scale { x: 8.0, y: 8.0 },
            &font,
            category,
        );
        img
    })
    .collect::<Vec<_>>();
    let urls = vec![
        "https://news.yahoo.co.jp/rss/topics/top-picks.xml",
        "https://news.yahoo.co.jp/rss/topics/domestic.xml",
        "https://news.yahoo.co.jp/rss/topics/world.xml",
        "https://news.yahoo.co.jp/rss/topics/business.xml",
        "https://news.yahoo.co.jp/rss/topics/entertainment.xml",
        "https://news.yahoo.co.jp/rss/topics/sports.xml",
        "https://news.yahoo.co.jp/rss/topics/it.xml",
        "https://news.yahoo.co.jp/rss/topics/science.xml",
        "https://news.yahoo.co.jp/rss/topics/local.xml",
    ];
    assert_eq!(categories.len(), urls.len());

    let img = categories.iter().take(8).enumerate().fold(
        GrayImage::new(128, 64),
        |mut img, (i, page)| {
            if i == 0 {
                let mut inverted = page.clone();
                invert(&mut inverted);
                overlay(&mut img, &inverted, 0, (i * 8) as u32);
            } else {
                overlay(&mut img, page, 0, (i * 8) as u32);
            }
            img
        },
    );
    oled.draw_image(&img, 0, 0)?;

    let mut category_pane = CategoryPane {
        categories: categories,
        urls: urls,
        start_i: 0,
        selected: 0,
    };

    let mut title_pane = TitlePane {
        titles: Vec::new(),
        descriptions: Vec::new(),
        start_i: 0,
        selected: 0,
    };

    let mut state = State::Categories;

    while let Ok(pressed) = button.pressed().await {
        match state {
            State::Categories => match pressed {
                [true, false, false] => {
                    if category_pane.selected < 7 {
                        let i = category_pane.start_i + category_pane.selected;
                        let img = category_pane.categories.get(i).unwrap();
                        oled.draw_image(img, 0, category_pane.selected as u8)?;
                        category_pane.selected += 1;
                        i += 1;
                        let mut img = category_pane.categories.get(i).unwrap().clone();
                        invert(&mut img);
                        oled.draw_image(&img, 0, category_pane.selected as u8)?;
                    } else {
                        category_pane.start_i += 1;
                        let img = category_pane
                            .categories
                            .iter()
                            .skip(category_pane.start_i)
                            .enumerate()
                            .fold(GrayImage::new(128, 64), |mut img, (i, page)| {
                                if i == 7 {
                                    let mut inverted = page.clone();
                                    invert(&mut inverted);
                                    overlay(&mut img, &inverted, 0, (i * 8) as u32);
                                } else {
                                    overlay(&mut img, page, 0, (i * 8) as u32);
                                }
                                img
                            });
                        oled.draw_image(&img, 0, 0)?;
                    }
                }
                [false, false, true] => {
                    if category_pane.selected > 0 {
                        let i = category_pane.start_i + category_pane.selected;
                        let img = category_pane.categories.get(i).unwrap();
                        oled.draw_image(img, 0, category_pane.selected as u8)?;
                        category_pane.selected -= 1;
                        i -= 1;
                        let mut img = category_pane.categories.get(i).unwrap().clone();
                        invert(&mut img);
                        oled.draw_image(&img, 0, category_pane.selected as u8)?;
                    } else {
                        category_pane.start_i -= 1;
                        let img = category_pane
                            .categories
                            .iter()
                            .skip(category_pane.start_i)
                            .enumerate()
                            .fold(GrayImage::new(128, 64), |mut img, (i, page)| {
                                if i == 0 {
                                    let mut inverted = page.clone();
                                    invert(&mut inverted);
                                    overlay(&mut img, &inverted, 0, (i * 8) as u32);
                                } else {
                                    overlay(&mut img, page, 0, (i * 8) as u32);
                                }
                                img
                            });
                        oled.draw_image(&img, 0, 0)?;
                    }
                }
                [false, true, false] => {
                    let i = category_pane.start_i + category_pane.selected;
                    let url = category_pane.urls.get(i).unwrap();
                    let s = reqwest::get(*url).await?.text().await?;
                    let rss = rss::RSS::new(&s)?;

                    let titles = rss
                        .channel
                        .items
                        .iter()
                        .map(|item| {
                            let mut img = GrayImage::new(128, 8);
                            draw_text_mut(
                                &mut img,
                                Luma([255]),
                                0,
                                0,
                                Scale { x: 8.0, y: 8.0 },
                                &font,
                                &item.title,
                            );
                            img
                        })
                        .collect::<Vec<_>>();

                    let descriptions = rss
                        .channel
                        .items
                        .into_iter()
                        .map(|item| item.description)
                        .collect::<Vec<_>>();

                    let mut img = titles.iter().take(8).enumerate().fold(
                        GrayImage::new(128, 64),
                        |mut img, (i, page)| {
                            if i == 0 {
                                let mut inverted = page.clone();
                                invert(&mut inverted);
                                overlay(&mut img, &inverted, 0, (i * 8) as u32);
                            } else {
                                overlay(&mut img, page, 0, (i * 8) as u32);
                            }
                            img
                        },
                    );
                    oled.draw_image(&img, 0, 0)?;
                    state = State::Titles;
                    title_pane = TitlePane {
                        titles: titles,
                        descriptions: descriptions,
                        start_i: 0,
                        selected: 0,
                    };
                }
                _ => (),
            },
            State::Titles => match pressed {
                [true, false, false] => {
                    if title_pane.selected < 7 {
                        let i = title_pane.start_i + title_pane.selected;
                        let img = title_pane.titles.get(i).unwrap();
                        oled.draw_image(img, 0, title_pane.selected as u8)?;
                        title_pane.selected += 1;
                        i += 1;
                        let mut img = title_pane.titles.get(i).unwrap().clone();
                        invert(&mut img);
                        oled.draw_image(&img, 0, title_pane.selected as u8)?;
                    } else {
                        title_pane.start_i += 1;
                        let img = title_pane
                            .titles
                            .iter()
                            .skip(title_pane.start_i)
                            .enumerate()
                            .fold(GrayImage::new(128, 64), |mut img, (i, page)| {
                                if i == 7 {
                                    let mut inverted = page.clone();
                                    invert(&mut inverted);
                                    overlay(&mut img, &inverted, 0, (i * 8) as u32);
                                } else {
                                    overlay(&mut img, page, 0, (i * 8) as u32);
                                }
                                img
                            });
                        oled.draw_image(&img, 0, 0)?;
                    }
                }
                [false, false, true] => {
                    if title_pane.selected > 0 {
                        let i = title_pane.start_i + title_pane.selected;
                        let img = title_pane.titles.get(i).unwrap();
                        oled.draw_image(img, 0, title_pane.selected as u8)?;
                        title_pane.selected -= 1;
                        i -= 1;
                        let mut img = title_pane.titles.get(i).unwrap().clone();
                        invert(&mut img);
                        oled.draw_image(&img, 0, title_pane.selected as u8)?;
                    } else {
                        title_pane.start_i -= 1;
                        let img = title_pane
                            .titles
                            .iter()
                            .skip(title_pane.start_i)
                            .enumerate()
                            .fold(GrayImage::new(128, 64), |mut img, (i, page)| {
                                if i == 0 {
                                    let mut inverted = page.clone();
                                    invert(&mut inverted);
                                    overlay(&mut img, &inverted, 0, (i * 8) as u32);
                                } else {
                                    overlay(&mut img, page, 0, (i * 8) as u32);
                                }
                                img
                            });
                        oled.draw_image(&img, 0, 0)?;
                    }
                }
                [true, false, true] => {
                    let img = category_pane
                        .categories
                        .iter()
                        .skip(category_pane.start_i)
                        .enumerate()
                        .fold(GrayImage::new(128, 64), |mut img, (i, page)| {
                            if i == category_pane.selected {
                                let mut inverted = page.clone();
                                invert(&mut inverted);
                                overlay(&mut img, &inverted, 0, (i * 8) as u32);
                            } else {
                                overlay(&mut img, page, 0, (i * 8) as u32);
                            }
                            img
                        });
                    oled.draw_image(&img, 0, 0)?;
                    state = State::Categories;
                }
                [false, true, false] => {
                    let i = title_pane.start_i + title_pane.selected;
                    let s = title_pane.descriptions.get(i).unwrap();

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
                    oled.draw_image(&img, 0, 0)?;
                    state = State::Details;
                }
                _ => (),
            },
            State::Details => match pressed {
                [true, false, true] => {
                    let mut img = title_pane
                        .titles
                        .iter()
                        .skip(title_pane.start_i)
                        .take(8)
                        .enumerate()
                        .fold(GrayImage::new(128, 64), |mut img, (i, page)| {
                            if i == title_pane.selected {
                                let mut inverted = page.clone();
                                invert(&mut inverted);
                                overlay(&mut img, &inverted, 0, (i * 8) as u32);
                            } else {
                                overlay(&mut img, page, 0, (i * 8) as u32);
                            }
                            img
                        });
                    oled.draw_image(&img, 0, 0)?;
                    state = State::Titles;
                }
                _ => (),
            },
            _ => unimplemented!(),
        }
    }
    Ok(())
}
