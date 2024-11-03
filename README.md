# Setup
This setup assumes a clean install of Raspberry Pi OS. For the Raspberry Pi Zero W 2, it is highly recommended that you install the Lite version of Raspberry Pi OS, and configure the swap to be 512MB at least.

* Edit the swap file config. `sudo nano /etc/dphys-swapfile`
* Update the `CONF_SWAPFILE` to atleast `512` for the Raspberry Pi Zero W 2.
* `sudo dphys-swapfile setup` to take the new setup.
* `sudo reboot` to make sure all apps get the new swap space.

# Tool Chain
As always with a new install, please `sudo apt update` and `sudo apt upgrade` to make sure you have the lastest versions.

We use rustup to install the rust compiler and cargo:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
(Use the default settings across the board.)

You also want to install git. `sudo apt install git` -- You might need it.

# Hardware
* UART connections are:
  * UART TX BCM Pin 14 / Physical Pin 8 for TX (Orange)
  * UART RX BCM Pin 15 / Physical Pin 10 for RX (Yellow).
* TOUCH GPIO BCM Pin 4 / Physical Pin 7 (Blue).
* GROUND is Physical Pin 14 (Black).
* 3V3 is Physical Pain 17 (Red).

![GPIO](https://www.raspberrypi.com/documentation/computers/images/GPIO-Pinout-Diagram-2.png)