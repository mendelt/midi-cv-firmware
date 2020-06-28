#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use ad5668::AD5668;
use cortex_m_rt::entry;
use embedded_midi::MidiIn;
use nb::block;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
    spi::{Mode, NoMiso, Phase, Polarity, Spi},
};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // Configure pins for serial rx/tx
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let tx_pin = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx_pin = gpioa.pa3;

    // Configure serial
    let mut usart = Serial::usart2(
        dp.USART2,
        (tx_pin, rx_pin),
        &mut afio.mapr,
        Config::default().baudrate(31250.bps()).parity_none(),
        clocks,
        &mut rcc.apb1,
    );

    // Configure Midi
    let (mut tx, mut rx) = usart.split();
    let mut midi_in = MidiIn::new(rx);

    // Configure the pins for SPI2
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let spi2_mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let spi2_sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let spi2_cs = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);

    // Configure SPI
    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let spi = Spi::spi2(
        dp.SPI2,
        (spi2_sck, NoMiso, spi2_mosi),
        spi_mode,
        100.khz(),
        clocks,
        &mut rcc.apb1,
    );
    let mut dac = AD5668::new(spi, spi2_cs);

    loop {
        let event = block!(midi_in.read());
    }
}
