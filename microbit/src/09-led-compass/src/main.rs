#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod calibration;
use crate::calibration::{
    // calc_calibration,
    calibrated_measurement,
    CALIBRATED_VALUE,
};
mod led;
use crate::led::{
    Direction,
    direction_to_led,
};

use microbit::{display::blocking::Display, hal::Timer};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate, Measurement};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    // let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    // let calibration = Calibration(Measurement(-7932, -4710, 38996), Measurement(1118, 1070, 1138), 78535);
    let calibration = CALIBRATED_VALUE;
    rprintln!("Calibration: {:?}", calibration);
    rprintln!("Calibration done, entering busy loop");

    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}
        let mut data = sensor.mag_data().unwrap();
        data = calibrated_measurement(data, &calibration);
        let dir = match (data.x > 0, data.y > 0) {
            (true, true) => Direction::NorthEast,
            (true, false) => Direction::SouthEast,
            (false, true) => Direction::NorthWest,
            (false, false) => Direction::SouthWest,
        };
        let led_matrix = direction_to_led(dir);
        display.show(&mut timer, led_matrix, 100);
        let x = data.x as f32;
        let y = data.y as f32;
        let z = data.z as f32;
        rprintln!("Amplitude: {}", libm::sqrtf(x*x + y*y + z*z));
    }
}
