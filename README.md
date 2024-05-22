# EPD driver

EPD = Electronic Paper Display

NOTE: This is a personal POC project.

## Design

- Normal B/W Driver
- Tri-Color Driver if supported
- Fast Driver
- Gray Scale Drivers

Refer [List of Displays](https://github.com/CursedHardware/epd-datasheet/blob/master/epd-display.csv) to see which driver should be used.

## How to use

```rust
    let spi = Spi::new_txonly(
        p.SPI2,
        p.PB10,
        p.PC3,
        NoDma,
        NoDma,
        Hertz(8_000_000),
        embassy_stm32::spi::Config::default(),
    );

    let dc = Output::new(p.PC9, Level::High, Speed::VeryHigh);
    let rst = Output::new(p.PA11, Level::Low, Speed::VeryHigh);
    let busy = Input::new(p.PG9, Pull::None);

    let di = EpdInterface::new(spi, cs, dc, rst, busy);

    display.init(&mut delay);

    // draw display here

    display.display_frame();
```

## Presets

```rust
// 2in9, 296x128
// FPC: FPC-7519
// SSD1680
```
