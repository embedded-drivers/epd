//! Command Table

#[repr(u8)]
pub enum Command {
    /// Set the number of gate.
    ///
    /// <<A:u8, 0:b5, B:b3>>
    DriverOutputControl = 0x01,
    /// Set Gate driving voltage
    ///
    /// <<0:b3, A:b5, 0:b4, B:b4>>
    GateDrivingVoltageControl = 0x03,
    /// Set Source output voltage
    ///
    /// LUT byte 30, the content of source level,
    ///
    /// <<0:b3, A:b5>>
    SourceDrivingVoltageControl = 0x04,
    /// Deep Sleep mode Control
    ///
    /// <<0:b7, A:b1>>
    ///
    /// ## A
    /// A=0, Normal Mode [POR]
    /// A=1, Enter Deep Sleep Mode
    DeepSleepMode = 0x10,
    /// Define data entry sequence
    /// <<0:b5, A:b3>>
    ///
    /// ## A[1:0]
    /// - 00 – Y decrement, X decrement,
    /// - 01 – Y decrement, X increment,
    /// - 10 – Y increment, X decrement,
    /// - 11 – Y increment, X increment [POR]
    ///
    /// ## A[2]
    /// - AM = 0, the address counter is updated in the X direction. [POR]
    /// - AM = 1, the address counter is updated in the Y direction.
    DataEntryModeSetting = 0x11,
    SwReset = 0x12,
    /// <<A:u8, b:b4, 0:b4>>
    TemperatureSensorControl = 0x1a,
    /// Activate Display Update Sequence
    ///
    /// The Display Update Sequence Option is located at R22h
    MasterActivation = 0x20,

    DisplayUpdateControl1 = 0x21,
    /// Display Update Sequence Option:
    /// Enable the stage for Master Activation.
    ///
    /// Enable Clock Signal
    /// Then Enable Analog
    /// No Use
    /// Then Load LUT
    /// Then INIITIAL DISPLAY
    /// Then PATTERN DISPLAY
    /// Then Disable Analog
    /// Then Disable OSC
    DisplayUpdateControl2 = 0x22,

    PanelBreakDetection = 0x23,

    /// Data entries will be written into the RAM until another command is written.
    /// Address pointers will advance accordingly.
    WriteRam = 0x24,
    /// Write VCOM register.
    ///
    /// A[7:0] = 00h [POR]
    ///
    /// | A[7:0] | VCOM | A[7:0] | VCOM |
    /// | ------ | ---- | ------ | ---- |
    /// | 0Fh    | -0.2 | 5Ah    | -1.7 |
    /// | 14h    | -0.3 | 5Fh    | -1.8 |
    /// | 19h    | -0.4 | 64h    | -1.9 |
    /// | 1Eh    | -0.5 | 69h    | -2   |
    /// | 23h    | -0.6 | 6Eh    | -2.1 |
    /// | 28h    | -0.7 | 73h    | -2.2 |
    /// | 2Dh    | -0.8 | 78h    | -2.3 |
    /// | 32h    | -0.9 | 7Dh    | -2.4 |
    /// | 37h    | -1   | 82h    | -2.5 |
    /// | 3Ch    | -1.1 | 87h    | -2.6 |
    /// | 41h    | -1.2 | 8Ch    | -2.7 |
    /// | 46h    | -1.3 | 91h    | -2.8 |
    /// | 4Bh    | -1.4 | 96h    | -2.9 |
    /// | 50h    | -1.5 | 9Bh    | -3   |
    /// | 55h    | -1.6 |        |      |
    WriteVcomRegister = 0x2c,
    /// Panel-Break flag, Chip ID
    StatusBitRead = 0x2f,
    /// Write LUT register from MCU interface [30 bytes]
    /// (excluding the VSH/VSL and Dummy bit)
    WriteLutRegister = 0x32,
    /// Set number of dummy line period.
    /// LUT byte 29, the content of dummy line.
    ///
    /// A[6:0]: Number of dummy line period in term of TGate
    ///
    /// Default: 0x06
    ///
    /// Driver: 0x1a, 4 dummy lines per gate
    ///
    /// Available setting 0 to 127.
    SetDummyLinePeriod = 0x3a,
    /// Set Gate line width (TGate) A[3:0] Line width in us.
    /// LUT byte 31, the content of gate line width.
    ///
    /// A[3:0]: Line width in us, 0 to 8
    ///
    /// Default: 0x0b = 0b1011, TGate = 78us
    /// Driver: 0x08, 2us/line
    ///
    /// NOTE: Default value will give 50Hz Frame frequency under 6 dummy line pulse setting.
    SetGateLineWidth = 0x3b,
    /// Select border waveform for VBD.
    BorderWaveformControl = 0x3c,
    /// Specify the start/end positions of the window address in the X direction by an address unit.
    ///
    /// x point must be the multiple of 8 or the last 3 bits will be ignored
    SetRamXAddressStartEndPosition = 0x44,
    /// Specify the start/end positions of the window address in the Y direction by an address unit.
    SetRamYAddressStartEndPosition = 0x45,
    SetRamXAddressCounter = 0x4e,
    SetRamYAddressCounter = 0x4f,
}
