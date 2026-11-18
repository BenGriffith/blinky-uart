#![no_std]
#![no_main]

use blinky_uart::{BlinkStats, LedState};
use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::uart;
use embassy_rp::{bind_interrupts, dma};
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blink-uart example"),
    embassy_rp::binary_info::rp_program_description!(c"test"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await;
}

// fn write_led_stats(
//     uart: &mut Uart<'_, Blocking>,
//     count: &usize,
//     on_time: &Duration,
//     off_time: &Duration,
// ) {
//     let mut msg: heapless::String<64> = heapless::String::new();
//     core::write!(
//         &mut msg,
//         "LED Count: {}, On Time: {}, Off Time: {}\r\n",
//         count,
//         on_time.as_millis(),
//         off_time.as_millis(),
//     )
//     .unwrap();
//     uart.blocking_write(msg.as_bytes()).unwrap();
// }

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let fw = aligned_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let nvram = aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        dma::Channel::new(p.DMA_CH0, Irqs),
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let delay = Duration::from_millis(250);
    let mut counter: usize = 0;

    let config = uart::Config::default();
    let mut uart = uart::Uart::new_blocking(p.UART0, p.PIN_0, p.PIN_1, config);
    loop {
        // uart.blocking_write("hello there\r\n".as_bytes()).unwrap();
        // uart.blocking_write("boom goes the dynamite!\r\n".as_bytes())
        // .unwrap();
        // cortex_m::asm::delay(1_000_000);

        // cycle blink mode every N blinks (slow, fast, burst)
        info!("led on!");
        let led_on_start = Instant::now();
        control.gpio_set(0, LedState::On.as_bool()).await;
        Timer::after(delay).await;
        let on_time = Instant::now() - led_on_start;

        info!("led off!");
        let led_off_start = Instant::now();
        control.gpio_set(0, LedState::Off.as_bool()).await;
        Timer::after(delay).await;
        let off_time = Instant::now() - led_off_start;

        // write_led_stats(&mut uart, &counter, &on_time, &off_time);
        // counter = counter.wrapping_add(1);
        let blink_stats = BlinkStats::new(counter, on_time, off_time);
        blink_stats.current_blink(&mut uart);
        counter = counter.wrapping_add(1);
    }
}
