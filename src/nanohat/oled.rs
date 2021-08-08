use i2cdev::core::I2CDevice;

pub struct NanoHatOLED<T>
where
    T: I2CDevice + Sized,
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
    fn send_commands(dev: &mut T, commands: &[u8]) -> Result<(), T::Error> {
        dev.smbus_write_i2c_block_data(0x00, commands)?;
        Ok(())
    }
    fn send_data(dev: &mut T, data: &[u8]) -> Result<(), T::Error> {
        for chunk in data.chunks(32) {
            dev.smbus_write_i2c_block_data(0x40, chunk)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(target_os = "linux")]
    fn staging_oled_open_test() {
        use i2cdev::linux::LinuxI2CDevice;
        let i2cdev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c).unwrap();
        assert!(NanoHatOLED::open(i2cdev).is_ok());
    }
}
