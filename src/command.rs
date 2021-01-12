//! Command Table

#[repr(u8)]
pub enum Command {
    /// Set the number of gate
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

    WriteRam = 0x24,

    WriteVcomRegister = 0x2c,
    /// Panel-Break flag, Chip ID
    StatusBitRead = 0x2f,
    /// Write LUT register from MCU interface [30 bytes]
    /// (excluding the VSH/VSL and Dummy bit)
    WriteLutRegister = 0x32,
    /// LUT byte 29, the content of dummy line.
    SetDummyLinePeriod = 0x3a,
    /// LUT byte 31, the content of gate line width,
    SetGatelineWidth = 0x3b,
    BorderWaveformControl = 0x3c,
    /// Specify the start/end positions of the window address in the X direction by an address unit.
    ///
    /// x point must be the multiple of 8 or the last 3 bits will be ignored
    SetRamXAaddressStartEndPosition = 0x44,
    /// Specify the start/end positions of the window address in the Y direction by an address unit.
    SetRamYAaddressStartEndPosition = 0x45,
    SetRamXAddressCounter = 0x4e,
    SetRamYAddressCounter = 0x4f,
}
