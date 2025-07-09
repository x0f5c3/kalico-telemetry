# Pico USB-to-UART Bridge

This folder contains example firmware for turning a Raspberry Pi Pico into a
USB-to-UART adapter. Two implementations are provided:

* `uart_bridge.py` &ndash; a MicroPython script that forwards data between the
  USB CDC interface and UART0.
* `pico_uart_bridge/` &ndash; a Rust program using the `rp-pico` board support
  crate. It runs USB and UART handling in interrupts for smooth throughput.

Both allow you to connect a device's UART output to a host PC via the Pico's USB
port.

## Flashing Instructions

1. Install MicroPython on the Pico using the official instructions from
   [raspberrypi.com](https://www.raspberrypi.com/documentation/microcontrollers/micropython.html).
2. Install `mpremote` from PyPI: `pip install mpremote`.
3. Connect the Pico via USB while holding the BOOTSEL button and copy the
   `uart_bridge.py` script as `main.py`:

   ```bash
   mpremote connect <port> fs cp uart_bridge.py :main.py
   ```

Replace `<port>` with the serial port of your board (for example `/dev/ttyACM0`).
After reset, the Pico will forward traffic between the USB serial connection and
its UART0 pins (GP0/GP1).

### Rust firmware

1. Install the `thumbv6m-none-eabi` target: `rustup target add thumbv6m-none-eabi`.
2. Build the firmware:

   ```bash
   cargo build --release \
     --manifest-path pico_bridge/pico_uart_bridge/Cargo.toml \
     --target thumbv6m-none-eabi
   ```

3. Convert the resulting ELF to a UF2 file and flash it:

   ```bash
   elf2uf2-rs target/thumbv6m-none-eabi/release/pico_uart_bridge pico_uart_bridge.uf2
   cp pico_uart_bridge.uf2 /media/$USER/RPI-RP2/
   ```

The Rust firmware behaves the same as the MicroPython version, forwarding bytes
between USB and UART0.
