#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use fugit::RateExtU32;
use heapless::spsc::Queue;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal::{self as hal, pac, Clock};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

// Types for UART peripheral
use hal::gpio::bank0::{Gpio0, Gpio1};
use hal::gpio::{FunctionUart, Pin, PullNone};

// Aliases to keep things tidy
type UartPins = (
    Pin<Gpio0, FunctionUart, PullNone>,
    Pin<Gpio1, FunctionUart, PullNone>,
);
type Uart = hal::uart::UartPeripheral<hal::uart::Enabled, pac::UART0, UartPins>;

static USB_BUS: Mutex<RefCell<Option<UsbBusAllocator<hal::usb::UsbBus>>>> =
    Mutex::new(RefCell::new(None));
static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<hal::usb::UsbBus>>>> =
    Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<SerialPort<hal::usb::UsbBus>>>> =
    Mutex::new(RefCell::new(None));
static UART: Mutex<RefCell<Option<Uart>>> = Mutex::new(RefCell::new(None));

// Queues for moving bytes between USB and UART
static mut USB_TO_UART: Queue<u8, 256> = Queue::new();
static mut UART_TO_USB: Queue<u8, 256> = Queue::new();

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            hal::uart::UartConfig::new(115_200u32.Hz(), hal::uart::DataBits::Eight, None, hal::uart::StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart.enable_rx_interrupt();
    uart.enable_tx_interrupt();

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    critical_section::with(|cs| {
        USB_BUS.borrow(cs).replace(Some(usb_bus));
    });

    let bus_ref = critical_section::with(|cs| USB_BUS.borrow(cs).as_ref().unwrap().clone());

    let serial = SerialPort::new(&bus_ref);
    critical_section::with(|cs| {
        USB_SERIAL.borrow(cs).replace(Some(serial));
    });

    let strings = [
        StringDescriptors::default()
            .manufacturer("kalico")
            .product("Pico UART Bridge")
            .serial_number("0001"),
    ];

    let usb_dev = UsbDeviceBuilder::new(&bus_ref, UsbVidPid(0x1209, 0x0001))
        .strings(&strings)
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    critical_section::with(|cs| {
        USB_DEVICE.borrow(cs).replace(Some(usb_dev));
        UART.borrow(cs).replace(Some(uart));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
        pac::NVIC::unmask(pac::Interrupt::UART0_IRQ);
    }

    // Main loop sleeps; all work happens in interrupts
    loop {
        cortex_m::asm::wfe();
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    critical_section::with(|cs| {
        let dev_ref = USB_DEVICE.borrow(cs).as_mut().unwrap();
        let serial_ref = USB_SERIAL.borrow(cs).as_mut().unwrap();

        if dev_ref.poll(&mut [serial_ref]) {
            let mut buf = [0u8; 64];
            if let Ok(count) = serial_ref.read(&mut buf) {
                if count > 0 {
                    for &b in &buf[..count] {
                        if let Some(q) = USB_TO_UART.enqueue(b).ok() {
                            let _ = q; // suppress unused warning
                        }
                    }
                }
            }

            while let Some(b) = unsafe { UART_TO_USB.dequeue() } {
                let _ = serial_ref.write(&[b]);
            }
        }
    });
    cortex_m::asm::sev();
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn UART0_IRQ() {
    critical_section::with(|cs| {
        if let Some(uart) = UART.borrow(cs).as_mut() {
            while let Ok(b) = uart.read() {
                if let Some(q) = UART_TO_USB.enqueue(b).ok() {
                    let _ = q;
                }
            }

            while let Some(b) = unsafe { USB_TO_UART.dequeue() } {
                if uart.write(b).is_err() {
                    USB_TO_UART.enqueue(b).ok();
                    break;
                }
            }
        }
    });
    cortex_m::asm::sev();
}

