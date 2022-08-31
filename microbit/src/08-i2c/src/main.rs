#![no_main]
#![no_std]

use cortex_m_rt::entry;
use heapless::Vec;
use core::{
    fmt::Write,
    str::from_utf8,
};
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

use microbit::{
    // I2C
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,

    // UART
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

use lsm303agr::{
    AccelOutputDataRate, MagOutputDataRate, Lsm303agr,
};

mod serial_setup;
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();
    let mut serial = {
        let serial = uarte::Uarte::new(board.UARTE0, board.uart.into(), Parity::EXCLUDED, Baudrate::BAUD115200);
        UartePort::new(serial)
    };

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100)};

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let mut bytes: Vec<u8, 32> = Vec::new();

    loop {
        let b = nb::block!(serial.read()).unwrap();
        if b == 13 {
            write!(serial, "\r\n").unwrap();
            // Interpret buffer as command.
            let cmd = from_utf8(bytes.as_ref()).unwrap();
            match cmd {
                "acc" => {
                    while !(sensor.accel_status().unwrap().xyz_new_data) {};
                    let data = sensor.accel_data().unwrap();
                    write!(serial, "Acceleration: x: {}, y: {}, z: {}\r\n", data.x, data.y, data.z).unwrap();
                },
                "mag" => {
                    while !(sensor.mag_status().unwrap().xyz_new_data) {};
                    let data = sensor.mag_data().unwrap();
                    write!(serial, "Magnetic field: x: {}, y: {}, z: {}\r\n", data.x, data.y, data.z).unwrap();
                }
                x => {
                    if x != "" {
                        write!(serial, "Unknown command: {}\r\n", x).unwrap()
                     }
                }
            };
            bytes.clear();
        } else {
            write!(serial, "{}", b as char).unwrap();
            bytes.push(b).unwrap_or_else(|_| { rprintln!("Buffer full!") });
        }
    }
}
