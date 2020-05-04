extern crate sysfs_gpio;
extern crate spidev;
extern crate image;

use std::io;
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;

use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use sysfs_gpio::{Direction, Pin};
use image::{GenericImage};
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


struct EinkImage {
    bw_layers: [[u8; 15000]; 2],
    //cw_layers: [[u8; 15000]; 2],
    image: GenericImage,
    color: bool,
    max_x: usize,
    max_y: usize,
}

impl EinkImage {
    fn new(image: GenericImage) -> EinkImage {
        let (x, y) = image.dimensions() as (usize, usize);
        EinkImage {
            bw_layers: get_bw_layers(image),
            //cw_layers: get_cw_layers(image),
            image: image,
            color: false,
            max_x: x,
            max_y: y
        }

    }

    fn get_layers(image: GenericImage) -> [[u8; 15000]; 2] {
        let (max_x, max_y) = image.dimensions();
        let layers: [[u8; 15000]; 2];
        let dark: [u8; 15000];
        let red: [u8; 15000];
        let temp: [u8; 8];
        let i = 0;
        for y in 0..max_y {
            for x in 0..(max_x/8) {
                // Break 8 pixels into a u8 where each bit is 1/0
                let msg: u8 = 0;
                for n in 0..7 {
                    let pixel = image.get_pixel_mut((x*8)+n, y);
                    if pixel.get_luma() <  u8::MAX/2 {
                        msg += 1 * 2u8.pow(n);
                    }
                }
                dark[i] = msg;
                red[i] = 0;
                i += 1;
            }
        }
        layers[0] = dark;
        layers[1] = red;
        return layers;
    }

    /*fn get_cw_layers(image: GenericImage) -> [[u8; 15000]; 2] {
        let dark: [u8; 15000];
    }*/
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
        self.power.set_direction(Direction::High);
        self.reset.set_direction(Direction::Low);
        self.cs.set_direction(Direction::Low);
        self.dc.set_direction(Direction::Low);
        self.busy.set_direction(Direction::In);
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
        self.spi.write(&[0x00, 0x0E]);
        thread::sleep(five_ms);
        // Input Temp TODO: real data
        self.spi.write(&[0xe5, 0x14]);
        // Active Temp
        if self.max_x == 400 && self.max_y == 300 {
            // Why is 4.2in special?
            self.spi.write(&[0xe0, 0x02]);
        } else {
            self.spi.write(&[0xcf, 0x02]);
        }
        // Panel settings
        self.spi.write(&[0x00, 0x0F, 0x89]);
    }

    // Step 5 from "Application Notes for small size Spectra & Yellow EPD with iTC
    // Shut down DC
    // Also unexports to clean up.
    fn power_off(&self) {
        self.spi.write(&[0x02, 0x02]);
        self.wait_busy(1);
        self.reset.set_direction(Direction::In);
        self.cs.set_value(0);
        self.dc.set_value(0);
        self.power.set_value(0);
        self.busy.set_direction(Direction::Low);
        thread::sleep(time::Duration::from_millis(150));
        self.power.unexport();
        self.reset.unexport();
        self.cs.unexport();
        self.dc.unexport();
        self.busy.unexport();
    }


    fn send_image(&self, data: [[u8; 15000]; 2]) {
        let layer_id: [u8; 2] = [0x10, 0x13];
        for layer in 0..1 {
            self.spi.write(&[layer_id[layer]]);
            self.cs.set_value(1);
            self.dc.set_value(1);
            self.cs.set_value(0);
            self.spi.write(&data[layer]);
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
            let val = self.busy.get_value().unwrap();
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
            power: Pin::new(18),
            max_x: x,
            max_y: y,
        }
    }

}


fn main() {
    let x: usize = 400;
    let y: usize = 300;
    let raw_image = image::open("test-image.png").unwrap();
    let image: EinkImage = EinkImage::new(raw_image)
    let display = EinkDisplay::new(x, y);

    println!("Hello, world!");
}
