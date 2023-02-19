#![no_std]
#![no_main]

use embedded_hal::digital::v2::{OutputPin, PinState};
use panic_halt as _;
use rp_pico::{self as bsp, entry, hal};

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    uart::UartConfig,
    watchdog::Watchdog,
};

use core::fmt::Write;
use embedded_hal::timer::CountDown;
use fugit::ExtU32;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Configure the Timer peripheral in count-down mode
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut count_down = timer.count_down();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();
    let mut led_red = pins.gpio5.into_push_pull_output();
    let mut led_yellow = pins.gpio9.into_push_pull_output();
    let mut led_green = pins.gpio13.into_push_pull_output();

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_mode::<hal::gpio::FunctionUart>(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_mode::<hal::gpio::FunctionUart>(),
    );
    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(UartConfig::default(), clocks.peripheral_clock.freq())
        .unwrap();

    uart.write_full_blocking(b"UART example\r\n");

    let mut value = 0u32;
    loop {
        led_green
            .set_state(if value & 1 == 1 {
                PinState::High
            } else {
                PinState::Low
            })
            .unwrap();
        led_yellow
            .set_state(if value & 2 == 2 {
                PinState::High
            } else {
                PinState::Low
            })
            .unwrap();
        led_red
            .set_state(if value & 4 == 4 {
                PinState::High
            } else {
                PinState::Low
            })
            .unwrap();
        writeln!(uart, "value: {value:02}\r").unwrap();
        led_pin.set_high().unwrap();
        count_down.start(500.millis());
        let _ = nb::block!(count_down.wait());
        led_pin.set_low().unwrap();
        count_down.start(500.millis());
        let _ = nb::block!(count_down.wait());
        value += 1;
    }
}

// End of file
