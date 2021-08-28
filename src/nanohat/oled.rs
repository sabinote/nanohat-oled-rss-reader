use i2cdev::core::I2CDevice;
use image::imageops::{dither, BiLevel};
use image::{DynamicImage, GenericImageView, GrayImage};
use std::error::Error;
use std::io;

pub struct NanoHatOLED<T>
where
    T: I2CDevice + Sized,
    T::Error: 'static,
{
    i2cdev: T,
}
impl<T> NanoHatOLED<T>
where
    T: I2CDevice + Sized,
{
    pub fn open(mut i2cdev: T) -> Result<Self, T::Error> {
        let commands = [
            0xAE, //display off
            0x00, //set lower column address
            0x10, //set higher column address
            0x40, //set display start line
            0xB0, //set page address
            0x81, //set contrast control
            0xCF, 0xA1, //set segment re-map
            0xA6, //set normal display
            0xA8, //set multiplex ratio
            0x3F, 0xC8, //Set COM OutputScan Direction
            0xD3, //set display offset
            0x00, 0xD5, //set display clock divide ratio/ oscillator frequency
            0x80, 0xD9, //set pre-charge period
            0xF1, 0xDA, //set COM pins
            0x12, 0xDB, //set vcomh
            0x40, 0x8D, //set charge pump enable
            0x14, 0x20, //set horizontal mode
            0x00, 0xAF, //display on
        ];
        Self::send_commands(&mut i2cdev, &commands)?;
        Ok(Self { i2cdev })
    }
    fn send_commands(i2cdev: &mut T, commands: &[u8]) -> Result<(), T::Error> {
        i2cdev.smbus_write_i2c_block_data(0x00, commands)?;
        Ok(())
    }
    fn send_data(i2cdev: &mut T, data: &[u8]) -> Result<(), T::Error> {
        for chunk in data.chunks(32) {
            i2cdev.smbus_write_i2c_block_data(0x40, chunk)?;
        }
        Ok(())
    }

    pub fn set_draw_range(&mut self, x: u8, y: u8, w: u8, h: u8) -> Result<(), T::Error> {
        let commands = [0x21, x, x + w - 1, 0x22, y, y + h - 1];
        Self::send_commands(&mut self.i2cdev, &commands)?;
        Ok(())
    }

    pub fn reset_draw_range(&mut self) -> Result<(), T::Error> {
        self.set_draw_range(0, 0, 128, 8)?;
        Ok(())
    }

    pub fn draw_image(&mut self, img: &GrayImage, x: u8, page_y: u8) -> Result<(), Box<dyn Error>> {
        let w = img.width();
        let (h, rem) = {
            let h = img.height();
            (h / 8, h % 8)
        };

        if x as u32 + w > 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The Image's dimensions are too large",
            )
            .into());
        } else if rem != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The Image's height is invalid",
            )
            .into());
        } else if page_y as u32 + h > 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The Image's dimensions are too large",
            )
            .into());
        } else {
            /*do nothing*/
        }

        let mut img = img.clone();
        dither(&mut img, &BiLevel);

        let height_pixel_offsets = [0, 8, 16, 24, 32, 40, 48, 56];
        let data = height_pixel_offsets
            .iter()
            .take(h as usize)
            .map(|offset| {
                (0..w)
                    .map(|x| {
                        (0..8).rev().fold(0u8, |mut bits, y| {
                            let px = img.get_pixel(x as u32, offset + y);
                            bits <<= 1;
                            bits |= if px[0] == 255 { 1 } else { 0 };
                            bits
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .flat_map(|v| v)
            .collect::<Vec<_>>();
        self.set_draw_range(x, page_y, w as u8, h as u8)?;
        Self::send_data(&mut self.i2cdev, &data)?;
        Ok(())
    }

    pub fn clear(&mut self, x: u8, y: u8, w: u8, h: u8) -> Result<(), Box<dyn Error>> {
        if x + w > 128 {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "The range is too large").into(),
            );
        } else if y + h > 8 {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "The range is too large").into(),
            );
        } else if w == 0 {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "The width must not be zero").into(),
            );
        } else if h == 0 {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "The height must not be zero").into(),
            );
        } else {
            /*do nothing*/
        }
        self.set_draw_range(x, y, w, h)?;
        let data = (0..)
            .take(w as usize * h as usize)
            .map(|_| 0x00)
            .collect::<Vec<_>>();
        Self::send_data(&mut self.i2cdev, &data)?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;
    use i2cdev::linux::LinuxI2CDevice;
    #[test]
    fn open_test() {
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        assert!(NanoHatOLED::open(i2cdev).is_ok());
    }

    #[test]
    fn draw_image_test1() {
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        let mut oled = NanoHatOLED::open(i2cdev).unwrap();
        let img_128x64 = GrayImage::new(128, 64);
        assert!(oled.draw_image(&img_128x64, 0, 0).is_ok());
        assert!(oled.draw_image(&img_128x64, 1, 0).is_err());
        assert!(oled.draw_image(&img_128x64, 0, 1).is_err());
        assert!(oled.draw_image(&img_128x64, 128, 0).is_err());
        assert!(oled.draw_image(&img_128x64, 0, 8).is_err());
    }
    #[test]
    fn draw_image_test2() {
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        let mut oled = NanoHatOLED::open(i2cdev).unwrap();
        let img_8x8 = GrayImage::new(8, 8);
        assert!(oled.draw_image(&img_8x8, 120, 0).is_ok());
        assert!(oled.draw_image(&img_8x8, 120, 7).is_ok());
        assert!(oled.draw_image(&img_8x8, 0, 0).is_ok());
        assert!(oled.draw_image(&img_8x8, 0, 7).is_ok());
    }
    #[test]
    fn draw_image_test3() {
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        let mut oled = NanoHatOLED::open(i2cdev).unwrap();
        let img_8x9 = GrayImage::new(8, 9);
        assert!(oled.draw_image(&img_8x9, 0, 0).is_err());
    }
    #[test]
    fn draw_image_test4() {
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        let mut oled = NanoHatOLED::open(i2cdev).unwrap();
        let img_10x8 = GrayImage::new(10, 8);
        assert!(oled.draw_image(&img_10x8, 0, 0).is_ok());
    }
    #[test]
    fn clear() {
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        let mut oled = NanoHatOLED::open(i2cdev).unwrap();

        assert!(oled.clear(0, 0, 128, 8).is_ok());
        assert!(oled.clear(0, 0, 129, 8).is_err());
        assert!(oled.clear(0, 0, 128, 9).is_err());
        assert!(oled.clear(129, 0, 1, 1).is_err());
        assert!(oled.clear(0, 8, 1, 1).is_err());
    }
}
