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
    let sx_miso = p.PIN_12;
    let sx_mosi = p.PIN_11;
    let sx_clk = p.PIN_10;
    let sx_csn = p.PIN_13;
    let sx_dc = p.PIN_14;

    let sx_cs_output = Output::new(sx_csn, Level::High); // Initially disable!
    let sx_dc_output = Output::new(sx_dc, Level::Low);

    // create spi bus, wrap it in a mutex.
    let spi_bus = Spi::new(used_spi, sx_clk, sx_mosi, sx_miso, p.DMA_CH0, p.DMA_CH1, Irqs, embassy_rp::spi::Config::default());
    let spi_bus_mutex = Mutex::<NoopRawMutex, _>::new(spi_bus);

    // create spi device 
    let spi_device = SpiDevice::new(&spi_bus_mutex, sx_cs_output);


    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(300).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
