# EPD driver

EPD = Electronic Paper Display

NOTE: This is a personal POC project.

## How to use

```rust
    let spi = Spi::new(
        p.SPI2,
        p.PB10,
        p.PC3,
        p.PC2, // not used
        NoDma,
        NoDma,
        Hertz(1_000_000),
        embassy_stm32::spi::Config::default(),
    );

    let cs = Output::new(p.PC7, Level::Low, Speed::VeryHigh);
    let dc = Output::new(p.PC9, Level::High, Speed::VeryHigh);
    let rst = Output::new(p.PA11, Level::Low, Speed::VeryHigh);
    let busy = Input::new(p.PG9, Pull::None);

    let di = EPDInterface::new(spi, dc, cs, rst, busy);
    let mut display: TriColorEPD<_, DisplaySize400x300, SSD1619A> = TriColorEPD::new(di);

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
