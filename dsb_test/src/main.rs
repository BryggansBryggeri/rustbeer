#[cfg(target_arch = "x86_64")]
use bryggio_lib::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use bryggio_lib::hardware::rbpi as hardware_impl;
use bryggio_lib::sensor::cool_ds18b20::find_devices_std;
fn main() {
    let one_wire_pin = hardware_impl::get_gpio_pin(4, "1w").expect("Could not get GPIO pin");
    let mut delay = hardware_impl::Delay {};
    find_devices_std(&mut delay, one_wire_pin);
}
