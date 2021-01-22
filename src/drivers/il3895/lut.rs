//! Waveform Look Up Table(LUT), which defines the display driving waveform settings.
//!
//! # LUT Table
//!
//! phase0A phase0B phase1A phase1B phase2A phase2B phase3A phase3B phase4A phase4B
//!
//! VS[n-XY], TP[n#], RP[n]
//!
//! - The phase period defined as TP[n#] * T_FRAME, where TP[n#] range from 0 to 31 (5 bits)
//!   - TP[n#] = 0 indicates phase skipped.
//! - The Repeat counter defined as RP[n], which represents repeating TP[nA] and TP[nB].
//!   - RP[n] = 0 indicates run time =1, where RP[n] range from 0 to 63.
//! - Source Voltage Level: VS[n#-XY] is constant in each phase.
//! - VS[n-XY] indicates the voltage in phase n for transition from X to Y:
//!   - X, Y: H, L
//!   - 00 – VSS
//!   - 01 – VSH
//!   - 10 – VSL
//!   - 11 - ? HiZ

/*
const unsigned char EPD_2IN13_lut_full_update[] = {
    0x22, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x11,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E,
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00
};

const unsigned char EPD_2IN13_lut_partial_update[] = {
    0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00
};

0x0F, 0x01
0b000_01111, 0b000_00001
*/

/// LUT for full update.
#[rustfmt::skip]
pub const LUT_FULL_UPDATE: [u8; 30] = [
    // VS, voltage in phase n
    0x22, 0x55,
    0xAA, 0x55,
    0xAA, 0x55,
    0xAA, 0x11,
    0x00, 0x00,
    // padding
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // RP, TP
    0x1E, 0x1E,
    0x1E, 0x1E,
    0x1E, 0x1E,
    0x1E, 0x1E,
    0x01, 0x00,
    // padding
    0x00, 0x00, 0x00, 0x00,
];

/// LUT for partial update.
#[rustfmt::skip]
pub const LUT_PARTIAL_UPDATE: [u8; 30] = [
    // VS, voltage in phase n
    // <<VS[0A-HH]:2/binary, VS[0A-HL]:2/binary, VS[0A-LH]:2/binary, VS[0A-LL]:2/binary>>
    // <<VS[0B-HH]:2/binary, VS[0B-HL]:2/binary, VS[0B-LH]:2/binary, VS[0B-LL]:2/binary>>
    // HL: white to black
    // LH: black to white
    // e.g. 0x18 = 0b00_01_10_00
    0x18, 0x00, // phase 0
    0x00, 0x00, // phase 1
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00, // phase 4
    // padding
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // RP, repeat counter, 0 to 63, 0 means run time = 1
    // TP, phase period, 0 to 31
    // <<RP[0]_L:3/binary, TP[0A]:5/binary>>
    // <<RP[0]_H:3/binary, TP[0B]:5/binary>>
    0x0F, 0x01, // phase 0
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00,
    0x00, 0x00, // phase 4
    // padding
    0x00, 0x00, 0x00, 0x00
];
