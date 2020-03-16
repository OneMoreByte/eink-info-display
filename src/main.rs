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

struct Layer {
    id: u8,
    byte_array: [u8; 15000],
    max_x: usize,
    max_y: usize,
}

impl Layer {
    fn get_byte_array() ->  [u8; max_x * max_y] {
        let mut array = [u8; max_x * max_y];
        for i in 0..(max_x * max_y) {
            array[i] = self.byte_array[i];
        }

        array
    }

    fn new() -> Layer {

    }
}


struct EinkImage {
    layers: [Layer; 2],
    image: GenericImage,
    max_x: usize,
    max_y: usize,
}

impl EinkImage {
    fn new(image: GenericImage, color_layer: Option<bool>) -> EinkImage {


    }
}


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

    fn power_on(&self) {
        five_ms = time::Duration::from_millis(5);
        ten_ms = time::Duration::from_millis(10);
        // Set pin direction
        self.power.set_direction(Direction::High)?;
        self.reset.set_direction(Direction::Low)?;
        self.cs.set_direction(Direction::Low)?;
        self.dc.set_direction(Direction::Low)?;
        self.busy.set_direction(Direction::In)?;
        // Enable pin
        self.power.export();
        self.reset.export();
        self.cs.export();
        self.dc.export();
        self.busy.export();
        // Power on Tcon (COG)
        thread::sleep(five_ms);
        self.reset.set_value(1);
        thread::sleep(five_ms);
        self.reset.set_value(0);
        thread::sleep(ten_ms);
        self.reset.set_value(1);
        thread::sleep(five_ms);
        self.spi.write(&[0x00, 0x0E])?;
        thread::sleep(five_ms);
        // Input Temp TODO: real data
        self.spi.write(&[0xe5, 0x14])?;
        // Active Temp
        if self.max_x == 400 && self.max_y == 300 {
            // Why is 4.2in special?
            self.spi.write(&[0xe0, 0x02])?;
        } else {
            self.spi.write(&[0xcf, 0x02])?;
        }
        // Panel settings
        self.spi.write(&[0x00, 0x0F, 0x89])?;
    }

    // Step 5 from "Application Notes for small size Spectra & Yellow EPD with iTC
    // Shut down DC
    // Also unexports to clean up.
    fn power_off(&self) {
        self.spi.write(&[0x02, 0x02]);
        self.wait_busy(1);
        self.reset.set_direction(Direction::In)?;
        self.cs.set_value(0);
        self.dc.set_value(0);
        self.power.set_value(0);
        self.busy.set_direction(Direction::Low)?;
        thread::sleep(time::Duration::from_millis(150));
        self.power.unexport();
        self.reset.unexport();
        self.cs.unexport();
        self.dc.unexport();
        self.busy.unexport();
    }


    fn send_image(&self, image: EinkImage) {
        let layers = &image.layers;
        for l in layers.iter() {
            self.spi.write(&[l.id.copy()]);
            self.cs.set_value(1);
            self.dc.set_value(1);
            self.cs.set_value(0);
            self.spi.write(&l.get_byte_array());
            self.dc.set_value(0);
        }
        self.wait_busy(1);
        // Power on command
        self.spi.write(&[0x04, 0x04]);
        self.wait_busy(0);
        // Display refresh
        self.spi.write(&[0x12, 0x12]);
        self.wait_busy(1);
    }

    fn wait_busy(&self, state: u8) {
        loop {
            let val = self.busy.get_value()?;
            if val == state {
                break;
            }
            thread::sleep(time::Duration::from_millis(5));
        }
    }

    fn new(x: usize, y: usize) -> EinkDisplay {
        EinkDisplay {
            spi: create_spi(spi_path),
            cs: Pin::new(17),
            dc: Pin::new(27),
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
