use gpio_cdev::{errors::Error, Chip, LineRequestFlags};
use linux_embedded_hal::CdevPin;
use linux_embedded_hal::Delay as ExtDelay;
use std::thread;
use std::time::Duration;

pub fn get_gpio_pin(pin_number: u32, label: &str) -> Result<RbpiGpioPin, Error> {
    RbpiGpioPin::try_new(pin_number, label)
}

/// Linux CDev pin type

/// Newtype around [`gpio_cdev::LineHandle`] that implements the `embedded-hal` traits
///
/// [`gpio_cdev::LineHandle`]: https://docs.rs/gpio-cdev/0.2.0/gpio_cdev/struct.LineHandle.html
pub struct RbpiGpioPin(pub gpio_cdev::LineHandle, bool);

impl RbpiGpioPin {
    /// See [`gpio_cdev::Line::request`][0] for details.
    ///
    /// [0]: https://docs.rs/gpio-cdev/0.2.0/gpio_cdev/struct.Line.html#method.request
    pub fn try_new(pin_number: u32, label: &str) -> Result<Self, Error> {
        let mut chip = match Chip::new("/dev/gpiochip0") {
            Ok(chip) => chip,
            Err(e) => return Err(e),
        };
        let line = match chip.get_line(pin_number) {
            Ok(line) => line,
            Err(e) => return Err(e),
        };
        let handle = line.request(LineRequestFlags::OUTPUT, 0, label)?;
        let info = handle.line().info()?;
        Ok(RbpiGpioPin(handle, info.is_active_low()))
    }
}

impl embedded_hal::digital::v2::OutputPin for RbpiGpioPin {
    type Error = Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(0)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(1)
    }
}

impl embedded_hal::digital::v2::InputPin for RbpiGpioPin {
    type Error = gpio_cdev::errors::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        if !self.1 {
            self.0.get_value().map(|val| val != 0)
        } else {
            self.0.get_value().map(|val| val == 0)
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_high().map(|val| !val)
    }
}

impl core::ops::Deref for RbpiGpioPin {
    type Target = gpio_cdev::LineHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RbpiGpioPin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Delay {}

impl embedded_hal::blocking::delay::DelayUs<u16> for Delay {
    fn delay_us(&mut self, n: u16) {
        thread::sleep(Duration::new(0, n as u32 * 1000))
    }
}
