#![no_std]

#[allow(unused_imports)]
use defmt::*;


use embedded_graphics::{
    pixelcolor::BinaryColor, 
    prelude::*
};

use display_interface::{
    AsyncWriteOnlyDataCommand, 
    DataFormat, DisplayError
};

const DISPLAY_WIDTH : u32 = 84;
const DISPLAY_HEIGHT : u32 = 48;
const BUFFER_SIZE : usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT / 8) as usize;

pub struct Pcd8544Driver<DELAY, DI, RESETPIN>{
    /// The framebuffer with one `u8` value per 8 vertical pixels.
    framebuffer: [u8; BUFFER_SIZE],
    display_interface : DI,
    reset_pin : RESETPIN,
    delay : DELAY
}

impl<DELAY, DI, RESETPIN> Pcd8544Driver<DELAY, DI, RESETPIN> 
where DI : AsyncWriteOnlyDataCommand,
    RESETPIN : embedded_hal_1::digital::OutputPin,
    DELAY : embedded_hal_async::delay::DelayNs
{
    pub fn new(delay: DELAY, display_interface : DI, reset_pin: RESETPIN ) -> Self{
        Pcd8544Driver { 
            framebuffer: [0u8; BUFFER_SIZE], 
            display_interface,
            reset_pin,
            delay
        }
    }


    /// reset and initialize. 
    pub async fn init(&mut self) -> Result<(), DisplayError> {

        self.reset().await;

        // chip active (PD=0); horizontal addressing mode (V = 0); use extended instruction set (H = 1)
        self.send_byte_command(consts::FUNCTION_SET + consts::EXTENDED_INSTRUCTION_SET).await?;
        // try 0xB1 (for 3.3V red SparkFun), 0xB8 (for 3.3V blue SparkFun), 0xBF if your display is too dark, or 0x80 to 0xFF if experimenting
        self.send_byte_command(consts::VOP + 0b00111000).await?;
        // temp coefficient (0)
        self.send_byte_command(consts::TEMP_COEFF).await?;
        // LCD bias mode 1:48
        self.send_byte_command(consts::BIAS + 0b011).await?;

        // we must send 0x20 before modifying the display control mode
        self.send_byte_command(consts::FUNCTION_SET).await?;
        // set display control to normal mode (pixels are on when data is 1), inverse mode=0x0D
        self.send_byte_command(consts::DISPLAY_CONTROL + consts::DISPLAY_CONF_NORMAL).await?;

        //self.clear();
        self.framebuffer.fill(0);
        self.flush().await?;

        Ok(())
    }


    /// helper: send a single byte as command. 
    async fn send_byte_command(&mut self, c : u8) -> Result<(), DisplayError> {
        self.display_interface.send_commands(DataFormat::U8(&[c])).await
    }

    /// reset
    async fn reset(&mut self){
        let _ = self.reset_pin.set_low();
        self.delay.delay_us(10).await;
        let _ = self.reset_pin.set_high();
        self.delay.delay_us(10).await;
    }

    /// send framebuffer to device.    
    pub async fn flush(& mut self) -> Result<(), DisplayError>{
        self.display_interface.send_data( DataFormat::U8(&self.framebuffer) ).await
    }
}

impl<DELAY, DI, RESETPIN> DrawTarget for Pcd8544Driver<DELAY, DI, RESETPIN> 
{
    type Color = BinaryColor;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>> {

        for Pixel(coord, color) in pixels.into_iter() {
            // Check if the pixel coordinates are out of bounds (negative or greater than
            // (DISPLAY_WIDTH,DISPLAY_HEIGHT)). `DrawTarget` implementation are required to discard any out of bounds
            // pixels without returning an error or causing a panic.
            if let Ok((x @ 0..=DISPLAY_WIDTH, y @ 0..=DISPLAY_HEIGHT)) = coord.try_into() {
                //
                // the following is very explicit. Optimization should
                // take care of it.
                //
                // Calculate the index in the framebuffer.
                let index = (x + (y / 8) * DISPLAY_WIDTH) as usize;
                // position of the bit to set/clear
                let bitpos = y % 8;
                // mask to mask the bit out of the olf value
                let bitmask : u8 = 1 << bitpos;
                // new bit value to set
                let bitvalue : u8 = (color.is_on() as u8) << bitpos;
                // and do it.
                let oldval = self.framebuffer[index];
                let newval: u8 = (oldval & !bitmask) | bitvalue;
                self.framebuffer[index] = newval;
            }
        }

        Ok(())            
    }
}

impl<DELAY, DI, RESETPIN> OriginDimensions for Pcd8544Driver<DELAY, DI, RESETPIN> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }
}


mod consts {
    #![allow(dead_code)]
    pub(crate) const X_ADDR: u8 = 0x80;
    pub(crate) const Y_ADDR: u8 = 0x40;
    pub(crate) const FUNCTION_SET: u8 = 0x20;
    pub(crate) const FUNCTION_SET_H_ADDRESSING: u8 = 0b00;
    pub(crate) const FUNCTION_SET_V_ADDRESSING: u8 = 0b10;
    pub(crate) const EXTENDED_INSTRUCTION_SET: u8 = 0x01;
    pub(crate) const VOP: u8 = 0x80;
    pub(crate) const TEMP_COEFF: u8 = 0b100;
    pub(crate) const BIAS: u8 = 0x10;
    pub(crate) const DISPLAY_CONTROL: u8 = 0x08;
    pub(crate) const DISPLAY_CONF_NORMAL: u8 = 0b100;
}
