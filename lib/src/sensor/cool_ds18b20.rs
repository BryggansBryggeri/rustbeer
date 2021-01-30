/// DS18B20 temperature sensor
///
/// [Datasheet](https://datasheets.maximintegrated.com/en/ds/DS18B20.pdf)
///
use core::fmt::{Debug, Write};
use ds18b20::{Ds18b20, Resolution, SensorData};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use one_wire_bus::{OneWire, OneWireResult};
use std::collections::HashMap;

pub fn find_devices_std<P, E>(delay: &mut impl DelayUs<u16>, one_wire_pin: P)
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: Debug,
{
    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();
    println!("Looking for devices...");
    for device_address in one_wire_bus.devices(false, delay) {
        // The search could fail at any time, so check each result. The iterator automatically
        // ends after an error.
        let device_address = device_address.unwrap();

        // The family code can be used to identify the type of device
        // If supported, another crate can be used to interact with that device at the given address
        println!(
            "Found device at address {:?} with family code: {:#x?}",
            device_address,
            device_address.family_code()
        );
    }
}

pub fn find_devices<P, E>(delay: &mut impl DelayUs<u16>, tx: &mut impl Write, one_wire_pin: P)
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: Debug,
{
    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();
    for device_address in one_wire_bus.devices(false, delay) {
        // The search could fail at any time, so check each result. The iterator automatically
        // ends after an error.
        let device_address = device_address.unwrap();

        // The family code can be used to identify the type of device
        // If supported, another crate can be used to interact with that device at the given address
        writeln!(
            tx,
            "Found device at address {:?} with family code: {:#x?}",
            device_address,
            device_address.family_code()
        )
        .unwrap();
    }
}

pub fn get_temperature<P, E>(
    delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
    one_wire_bus: &mut OneWire<P>,
) -> OneWireResult<HashMap<u64, SensorData>, E>
where
    P: OutputPin<Error = E> + InputPin<Error = E>,
    E: Debug,
{
    // initiate a temperature measurement for all connected devices
    ds18b20::start_simultaneous_temp_measurement(one_wire_bus, delay)?;

    // wait until the measurement is done. This depends on the resolution you specified
    // If you don't know the resolution, you can obtain it from reading the sensor data,
    // or just wait the longest time, which is the 12-bit resolution (750ms)
    Resolution::Bits12.delay_for_measurement_time(delay);

    // TODO: no_std?
    let mut measurements = HashMap::new();

    // iterate over all the devices, and report their temperature
    let mut search_state = None;
    while let Some((device_address, state)) =
        one_wire_bus.device_search(search_state.as_ref(), false, delay)?
    {
        search_state = Some(state);
        if device_address.family_code() != ds18b20::FAMILY_CODE {
            // skip other devices
            continue;
        }
        // You will generally create the sensor once, and save it for later
        let sensor = Ds18b20::new(device_address)?;

        // contains the read temperature, as well as config info such as the resolution used
        let sensor_data = sensor.read_data(one_wire_bus, delay)?;
        // TODO: Impl Hash for device address.
        measurements.insert(device_address.0, sensor_data);
    }
    Ok(measurements)
}

fn int_tuple_to_float(integer: i16, decimal: i16) -> f32 {
    integer as f32 + decimal as f32 / 10_000.0
}
