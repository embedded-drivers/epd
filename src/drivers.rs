use crate::interface::{self, DisplayInterface};
use embedded_hal::blocking::delay::DelayUs;

// TOOD: add profile support
pub trait Driver {
    type Error;

    /// Wake UP and init
    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error>;

    // also set ram pos
    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error>;

    // fn set_ram_pos<DI: DisplayInterface>(&mut self, di: &mut DI, x: u16, y: u16);

    fn update_frame<DI: DisplayInterface>(
        di: &mut DI,
        channel: usize,
        buffer: &[u8],
    ) -> Result<(), Self::Error>;

    fn load_lut<DI: DisplayInterface>(&mut self, _di: &mut DI) -> Result<(), Self::Error> {
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;

    fn sleep<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Red/Black/White. 400 source outputs, 300 gate outputs
pub struct SSD1619A;

impl Driver for SSD1619A {
    type Error = interface::DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 200_000, 200_000);
        di.busy_wait();

        di.send_command(0x12)?; //swreset
        di.busy_wait();

        // 3
        // Set analogue then digital block control
        di.send_command_data(0x74, &[0x54])?;
        di.send_command_data(0x7e, &[0x3b])?;

        di.send_command_data(0x2b, &[0x03, 0x63])?; // reduce glitch under ACVCOM

        di.send_command_data(0x0c, &[0x8b, 0x9c, 0x96, 0x0f])?; // soft start setting

        di.send_command_data(0x01, &[0x2b, 0x01, 0x00])?; // Driver Output Control - set mux as 300

        di.send_command_data(0x11, &[0x00])?; // data entry mode

        // 0x44, 0x45, ram x,y start,end

        di.send_command_data(0x03, &[0x20])?; // Gate Driving Voltage Control

        // A[7:0] = 41h [POR], VSH1 at 15V
        // B[7:0] = A8h [POR], VSH2 at 5V.
        // C[7:0] = 32h [POR], VSL at -15V
        //di.send_command_data(0x04, &[0x4b, 0xce, 0x3a]); // Source Driving Voltage Control

        //di.send_command_data(0x3A, &[0x21]); // dummy line, 0 to 127
        //di.send_command_data(0x3B, &[0x06]); // gate width

        // 0b10_00_00 , VCOM, black
        // 0b11_00_00, HiZ
        // 0b01_00_00, VSS
        di.send_command_data(0x3C, &[0x01])?; // border wavefrom, HIZ

        di.send_command_data(0x18, &[0x80])?;
        // load temperature and waveform setting.
        // di.send_command_data(0x22, &[0xb1])?;

        di.send_command_data(0x22, &[0xb9])?;

        di.send_command(0x20)?;
        di.busy_wait();

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        // Set RAM X - address Start / End position
        di.send_command_data(0x44, &[0x00, ((x - 1) >> 3) as u8])?;

        // Set RAM Y - address Start / End position
        di.send_command_data(
            0x45,
            &[0x00, 0x00, ((y - 1) >> 8) as u8, ((y - 1) & 0xff) as u8],
        )?;

        Ok(())
    }

    fn update_frame<DI: DisplayInterface>(
        di: &mut DI,
        channel: usize,
        buffer: &[u8],
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

        if channel == 0 {
            di.send_command_data(0x24, buffer)?;
        } else if channel == 1 {
            di.send_command_data(0x26, buffer)?;
        } else {
            // error
        }

        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xf7])?; // Display Update Control 2
        di.send_command(0x20)?; // master activation
        di.busy_wait();
        Ok(())
    }
}

pub struct SSD1608;

impl Driver for SSD1608 {
    type Error = interface::DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        const EPD_WIDTH: u32 = 400;
        const EPD_HEIGHT: u32 = 300;

        // TODO: reset
        // Driver Output control
        di.send_command_data(
            0x01,
            &[(EPD_HEIGHT - 1) as u8, ((EPD_HEIGHT - 1) >> 8) as u8, 0],
        )
        .unwrap();

        // Booster Enable with Phase 1, Phase 2 and Phase 3 for soft start current setting.
        // di.send_command_data(0x0c, &[0xd7, 0xd6, 0x9d]).unwrap();

        // write VCOM reg
        di.send_command_data(0x2c, &[0x7c]).unwrap(); //a8

        // Set dummy line period
        di.send_command_data(0x3a, &[0x1a]).unwrap();
        // Set Gate line width
        di.send_command_data(0x3b, &[0x08]).unwrap();

        // Border Waveform Control
        // 00 VSS => 相当于无电场
        // 01 VSH => very black
        // 10 VSL => gray?
        // 11 HiZ => no change
        di.send_command_data(0x3c, &[0b1_1_10_00_00]).unwrap(); // border waveform control

        // Data Entry mode,
        // Y increment, X increment
        // address counter is updated in the X direction. [POR]
        di.send_command_data(0x11, &[0x03]).unwrap();

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        const EPD_WIDTH: u32 = 400;
        const EPD_HEIGHT: u32 = 300;

        // set ram x start/end
        di.send_command_data(0x44, &[0, (EPD_WIDTH >> 3) as u8])
            .unwrap();
        // set ram y start/end
        di.send_command_data(0x45, &[0, 0, EPD_HEIGHT as u8, (EPD_HEIGHT >> 8) as u8])
            .unwrap();
        Ok(())
    }

    fn update_frame<DI: DisplayInterface>(
        di: &mut DI,
        channel: usize,

        buffer: &[u8],
    ) -> Result<(), Self::Error> {
        let x = 0;
        //di.cmd_with_data(0x4E, &[(x >> 3) as u8]);
        //di.cmd_with_data(0x4f, &[y as u8, (y >> 8) as u8]);

        // set cursor
        di.send_command(0x24); // Write RAM
        di.send_data(buffer);
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        todo!()
    }
}
