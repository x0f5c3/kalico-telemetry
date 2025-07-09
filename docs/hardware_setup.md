# Hardware Setup

This project communicates with external devices over a UART interface. If your
host machine lacks a serial port you can repurpose a Raspberry Pi Pico as a
USB-to-UART bridge. Two firmware options are provided:

* **MicroPython**: `uart_bridge.py` – quick to flash and use.
* **Rust**: `pico_uart_bridge/` – built with the `rp-pico` crate for a fully
  native implementation.

Both reside in [`pico_bridge/`](../pico_bridge). Follow the README there to
flash either implementation. Once running, connect the device's TX/RX pins to
the Pico's UART0 pins (GP0 and GP1) and use the USB connection to talk to the
target device.
