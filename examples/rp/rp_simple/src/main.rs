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


use {defmt_rtt as _, panic_probe as _};


bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});



#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    //
    // set up spi
    //
    
    info!("set up spi and io's for pcd8544 display");

    let used_spi = p.SPI1;
    let spi1_miso = p.PIN_12;
    let spi1_mosi = p.PIN_11;
    let spi1_sx_clk = p.PIN_10;
    let display_cs = p.PIN_13;
    let display_dc = p.PIN_14;

    let display_cs_output = Output::new(display_cs, Level::High); // Initially disable!
    let display_dc_output = Output::new(display_dc, Level::Low);

    // create spi bus, wrap it in a mutex, so others could access it, too.
    let spi_bus = Spi::new(used_spi, spi1_sx_clk, spi1_mosi, spi1_miso, p.DMA_CH0, p.DMA_CH1, Irqs, embassy_rp::spi::Config::default());
    let spi_bus_mutex = Mutex::<NoopRawMutex, _>::new(spi_bus);

    // create spi device 
    let spi_device = SpiDevice::new(&spi_bus_mutex, display_cs_output);

    // create display interface abstraction from SPI and DC
    let di = SPIInterface::new(spi_device, display_dc_output);

    // create display
    let mut display = Pcd8544Driver::new(di);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(300).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
