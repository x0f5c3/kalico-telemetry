import time
from machine import UART, USB_VCP

usb = USB_VCP()
uart = UART(0, 115200)

while True:
    if usb.any():
        uart.write(usb.read(usb.any()))
    if uart.any():
        usb.write(uart.read(uart.any()))
    time.sleep_ms(1)
