import time
from machine import UART, USB_VCP

usb = USB_VCP()
uart = UART(0, 115200)

while True:
    usb_data_available = usb.any()
    if usb_data_available:
        uart.write(usb.read(usb_data_available))
    if uart.any():
        usb.write(uart.read(uart.any()))
    time.sleep_ms(1)
