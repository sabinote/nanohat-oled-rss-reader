use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use i2cdev::core::*;

pub struct NanoHatOLED{
    dev: LinuxI2CDevice,
}
impl NanoHatOLED {
    pub fn open() -> Result<Self, LinuxI2CError> {
        let mut dev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c)?;
        let commands = [
            0xAE, //display off
            0x00, //set lower column address
            0x10, //set higher column address
            0x40, //set display start line
            0xB0, //set page address
            0x81, //set contrast control
            0xCF,
            0xA1, //set segment re-map
            0xA6, //set normal display
            0xA8, //set multiplex ratio
            0x3F,
            0xC8, //Set COM OutputScan Direction
            0xD3, //set display offset
            0x00,
            0xD5, //set display clock divide ratio/ oscillator frequency
            0x80,
            0xD9, //set pre-charge period
            0xF1,
            0xDA, //set COM pins
            0x12, 
            0xDB, //set vcomh
            0x40,
            0x8D, //set charge pump enable
            0x14,
            0x20, //set horizontal mode
            0x00,
            0xAF, //display on
        ];
        Self::send_commands(&mut dev, &commands)?;
        Ok(
            Self {
                dev,
            }
        )
    }
    fn send_commands(dev: &mut LinuxI2CDevice, commands: &[u8]) -> Result<(), LinuxI2CError> {
        dev.smbus_write_i2c_block_data(0x00, commands)?;
        Ok(())
    }
    fn send_data(dev: &mut LinuxI2CDevice, data: &[u8]) -> Result<(), LinuxI2CError> {
        for chunk in data.chunks(32) {
            dev.smbus_write_i2c_block_data(0x40, chunk)?;
        }
        Ok(())
    }
}