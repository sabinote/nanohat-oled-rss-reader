use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use i2cdev::core::*;

pub struct NanoHatOLED{
    dev: LinuxI2CDevice,
}
impl NanoHatOLED {
    pub fn open() -> Result<Self, LinuxI2CError> {
        let mut dev = LinuxI2CDevice::new("/dev/i2c-0", 0x3c)?;
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