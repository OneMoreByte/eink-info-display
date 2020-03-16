extern crate sysfs_gpio;
extern crate spidev;

use std::io;
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;

use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use sysfs_gpio::{Direction, Pin};

/* 169.254.186.222
 * Get temp
 * Power on Tcon (COG)
 *  Power display
 *  wait 5ms

 *  RES <- 1
 *  wait 5ms
 *  RES <- 0
 *  wait 10ms
 *  RES <- 1
 *  SPI(0x00, 0x0E)
 *  wait 5ms
 * Send temp to Tcon
 * Send image data to COG
 *   left to right
 *   top to down
 *   400 x 300
 * Send power settings to DC/DC
 * Check BUSY == 1
 * Send update command
 * Wait for Busy == 1
 * Turn off DC/DC
 *
 *
 * Max Clock is 10MHz
 */

 /*
 * GPIO 0-11 -> SPI
 * GPIO17 -> ECS
 * GPIO27 -> D/C
 * GPIO22 -> SRCS
 * GPIO23 -> BUSY
 * GPIO24 -> RST
 * GPIO25 -> ENA
 *
 *
 */

struct EinkDisplay {
    power: Pin,
    cs: Pin,
    dc: Pin,
    busy: Pin,
    reset: Pin,
    enable: Pin,
    spi: Spidev,
    max_x: usize,
    max_y: usize,
}

// Do not update more than once every 180 seconds or you may permanently damage the display

impl EinkDisplay {
    fn create_spi() -> io::Result<Spidev> {
        let mut spi = Spidev::open("/dev/spidev0.0")?;
        let options = SpidevOptions::new()
             .bits_per_word(8)
             .max_speed_hz(10_000_000)
             .mode(SpiModeFlags::SPI_MODE_0)
             .build();
        spi.configure(&options)?;
        Ok(spi)
    }

    fn power_on() {
        five_ms = time::Duration::from_millis(5);
        ten_ms = time::Duration::from_millis(10);
        // Set pin direction
        power.set_direction(Direction::High)?;
        reset.set_direction(Direction::Low)?;
        cs.set_direction(Direction::Low)?;
        dc.set_direction(Direction::Low)?;
        busy.set_direction(Direction::In)?;
        // Enable pin
        power.export();
        reset.export();
        cs.export();
        dc.export();
        busy.export();
        // Power on Tcon (COG)
        thread::sleep(five_ms);
        reset.set_value(1);
        thread::sleep(five_ms);
        reset.set_value(0);
        thread::sleep(ten_ms);
        reset.set_value(1);
        thread::sleep(five_ms);
        spi.write(&[0x00, 0x0E])?;
        thread::sleep(five_ms);
        // Input Temp TODO: real data
        spi.write(&[0xe5, 0x14])?;
        // Active Temp
        if max_x == 400 && max_y == 300 {
            // Why is 4.2in special?
            spi.write(&[0xe0, 0x02])?;
        } else {
            spi.write(&[0xcf, 0x02])?;
        }
        // Panel settings
        spi.write(&[0x00, 0x0F, 0x89])?;
    }

    // Step 5 from "Application Notes for small size Spectra & Yellow EPD with iTC
    // Shut down DC
    // Also unexports to clean up.
    fn power_off() {
        spi.write(&[0x02, 0x02]);
        wait_busy(1);
        reset.set_direction(Direction::In)?;
        cs.set_value(0);
        dc.set_value(0);
        power.set_value(0);
        busy.set_direction(Direction::Low)?;
        thread::sleep(time::Duration::from_millis(150));
        power.unexport();
        reset.unexport();
        cs.unexport();
        dc.unexport();
        busy.unexport();
    }


    fn send_image(image: Image) {
        let layers = image.layers();
        for l in layers.iter() {
            spi.write(&[l.copy()]);
            cs.set_value(1);

            dc.set_value(1);

            cs.set_value(0);
            spi.write(&image.layer_to_byte_array(l));
            dc.set_value(0);
        }
        wait_busy(1);
        // Power on command
        spi.write(&[0x04, 0x04]);
        wait_busy(0);
        // Display refresh
        spi.write(&[0x12, 0x12]);
        wait_busy(1);
    }

    fn wait_busy(state: u8) {
        loop {
            let val = busy.get_value()?;
            if val == state {
                break;
            }
            thread::sleep(time::Duration::from_millis(5));
        }
    }

    fn new(x: u8, y: u8) -> EinkDisplay {
        EinkDisplay {
            spi: create_spi(spi_path),
            cs: Pin::new(17),
            dc: Pin::new(27),
            //srcs: Pin::new(22),
            busy: Pin::new(23),
            reset: Pin::new(24),
            enable: Pin::new(25),

        }
    }

}


fn main() {
    let x: usize = 400;
    let y: usize = 300;
    let display = EinkDisplay::new(x, y);

    println!("Hello, world!");
}
