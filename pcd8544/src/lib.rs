#![no_std]

use defmt::*;


use embedded_graphics::{
    pixelcolor::BinaryColor, 
    prelude::*
};

use display_interface::{
    AsyncWriteOnlyDataCommand, 
    DataFormat
};

const DISPLAY_WIDTH : u32 = 84;
const DISPLAY_HEIGHT : u32 = 48;
const BUFFER_SIZE : usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT / 8) as usize;

pub struct Pcd8544Driver<DI>{
    /// The framebuffer with one `u8` value per 8 vertical pixels.
    framebuffer: [u8; BUFFER_SIZE],
    comm : DI
}

impl<DI> Pcd8544Driver<DI> 
where DI : AsyncWriteOnlyDataCommand
{
    pub fn new(comm : DI ) -> Self{
        Pcd8544Driver { 
            framebuffer: [0u8; BUFFER_SIZE], 
            comm 
        }
    }
    
    pub async fn flush(& mut self){
        self.comm.send_data( DataFormat::U8(&self.framebuffer) ).await.unwrap()
    }
}

impl<DI> DrawTarget for Pcd8544Driver<DI> 
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

impl<DI> OriginDimensions for Pcd8544Driver<DI> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }
}