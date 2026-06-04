#![no_std]
#![no_main]

//use core::time::Duration;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, dma};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1};

use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Level, Output};

// SPI stuff
use embassy_rp::spi::Spi;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;

// Mutex stuff.
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;

use display_interface_spi::SPIInterface;
use pcd8544::Pcd8544Driver;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_5X8, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});



#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_22, Level::Low);

    //
    // set up spi
    //
    
    info!("set up spi and io's for pcd8544 display");

    let used_spi = p.SPI0;
    let spi1_miso = p.PIN_16;
    let spi1_mosi = p.PIN_19;
    let spi1_sx_clk = p.PIN_18;
    let display_cs = p.PIN_17;
    let display_dc = p.PIN_20;
    let display_reset = p.PIN_21;

    let display_cs_output = Output::new(display_cs, Level::High); // Initially disable!
    let display_dc_output = Output::new(display_dc, Level::Low);
    let display_reset_output = Output::new(display_reset, Level::High);

    // create spi bus, wrap it in a mutex, so others could access it, too.
    let spi_bus = Spi::new(used_spi, spi1_sx_clk, spi1_mosi, spi1_miso, p.DMA_CH0, p.DMA_CH1, Irqs, embassy_rp::spi::Config::default());
    let spi_bus_mutex = Mutex::<NoopRawMutex, _>::new(spi_bus);

    // create spi device 
    let spi_device = SpiDevice::new(&spi_bus_mutex, display_cs_output);

    // create display interface abstraction from SPI and DC
    let display_interface = SPIInterface::new(spi_device, display_dc_output);

    // create display_driver
    let mut display = Pcd8544Driver::new(embassy_time::Delay, display_interface, display_reset_output);

    display.init().await.unwrap();

    // create a text style.
    let text_style = MonoTextStyleBuilder::new()
    .font(&FONT_6X10)
    .text_color(BinaryColor::On).
    background_color(BinaryColor::Off)
    .build();

    // draw some text
    Text::with_baseline("Hello World", Point::new(0, 0), text_style, Baseline::Top).draw(&mut display).unwrap();
    // and flush.
    display.flush().await.unwrap();

    let mut string_buf = [0u8; 64];

    let mut counter = 0;
    loop {
        display.flush().await.unwrap();
        //info!("led on!");
        led.set_high();
        Timer::after_millis(300).await;

        //info!("led off!");
        led.set_low();
        Timer::after_millis(300).await;

        let s = format_no_std::show(&mut string_buf, format_args!("COUNTER = {}", counter)).unwrap();
        // draw the counter value
        Text::with_baseline(s, Point::new(0, 16), text_style, Baseline::Top).draw(&mut display).unwrap();
        // and flush.
        display.flush().await.unwrap();

        counter+=1;

    }
}
