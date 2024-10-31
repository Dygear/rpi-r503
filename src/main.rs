use std::{time::Duration, u8};

use rppal::{gpio::{Gpio, Trigger}, uart::{Parity, Uart}};
use r503::{Color, Identifier, Instruction, LightPattern};
use heapless::Vec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // BCM Pin 4 / Physical Pin 7 is connected to the touch sensor on our r503 module.
    let mut touch = Gpio::new()?.get(4)?.into_input();
    match touch.set_interrupt(Trigger::Both, Some(Duration::from_millis(16))) {
        Ok(..) => println!("Touch event registered."),
        Err(e) => eprintln!("Touch event failed: {e:?}")
    };

    // There is only one UART interface on the GPIO pins;
    // BCM Pin 14 / Physical Pin 8 is TX, and BCM Pin 15 / Physical Pin 10 is RX.
    // The figerprint sensor itself has a buadrate of 57600, although that can be increased.
    let mut uart = Uart::new(57600, Parity::None, 8, 1)?;

    // Set Fingerprint sensor color to default white.
    let mut data_buf: Vec<u8, 32> = heapless::Vec::new();
    let _ = data_buf.push(LightPattern::Breathing.into());  // Breathing Light
    let _ = data_buf.push(0xFF);                            // 0 = Very Fast, 255 = Very Slow; Max Time = 5 Seconds.
    let _ = data_buf.push(Color::White.into());             // colour=Red, Blue, Purple
    let _ = data_buf.push(0x00);                            // times=Infinite
    let send_buf = send(
        Identifier::Command,
        Instruction::AuraLedConfig,
        Some(data_buf)
    );
    let data_write: [u8; 16] = send_buf.clone().into_array().unwrap();
    match uart.write(&data_write) {
        Ok(..) => (), // No News is Good News.
        Err(e) => eprintln!("Write error: {:?}", e),
    }

    loop {
        let edge = match touch.poll_interrupt(true, Some(Duration::from_secs(60))) {
            Ok(Some(event)) => event.trigger,
            Ok(None) => continue, // Timeout reached.
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let color = match edge {
            Trigger::RisingEdge  => Color::Blue,
            Trigger::FallingEdge => Color::Cyan,
            _ => unreachable!()
        };

        let mut data_buf: Vec<u8, 32> = heapless::Vec::new();

        // Set the data first, because the length is dependent on that.
        // However, we write the length bits before we do the data.
        data_buf.clear();
        let _ = data_buf.push(LightPattern::Breathing.into());  // Breathing Light
        let _ = data_buf.push(0xFF);                            // 0 = Very Fast, 255 = Very Slow; Max Time = 5 Seconds.
        let _ = data_buf.push(color.clone().into());            // colour=Red, Blue, Purple
        let _ = data_buf.push(0x00);                            // times=Infinite

        let send_buf = send(
            Identifier::Command,
            Instruction::AuraLedConfig,
            Some(data_buf)
        );

        // Send command buffer.
        let data_write: [u8; 16] = send_buf.clone().into_array().unwrap();
        println!("{:?}", data_write);
        match uart.write(&data_write) {
            Ok(..) => (), // No News is Good News.
            Err(e) => eprintln!("Write error: {:?}", e),
        }

        // Read command buffer.
        let mut buffer = [0u8; 16];
        match uart.read(&mut buffer) {
            Ok(len) => println!("Read {len} bytes, {buffer:?}"),
            Err(e) => eprintln!("Read error: {e}")
        };
    }
}

fn send(pid: Identifier, command: Instruction, data: Option<Vec<u8, 32>>) -> Vec<u8, 256> {
    let mut send_buf: Vec<u8, 256> = heapless::Vec::new();

    // Start    2 bytes Fixed value of 0xEF01; High byte transferred first.
    let _ = send_buf.extend_from_slice(&r503::HEADER.to_be_bytes()[..]);

    // Address  4 bytes Default value is 0xFFFFFFFF, which can be modified by command.
    //                  High byte transferred first and at wrong adder value, module
    //                  will reject to transfer.
    let _ = send_buf.extend_from_slice(&r503::ADDRESS.to_be_bytes()[..]);

    // PID      1 byte  01H Command packet;
    //                  02H Data packet; Data packet shall not appear alone in executing
    //                      processs, must follow command packet or acknowledge packet.
    //                  07H Acknowledge packet;
    //                  08H End of Data packet.
    let _ = send_buf.extend_from_slice(&[pid.into()]);

    // LENGTH   2 bytes Refers to the length of package content (command packets and data packets)
    //                  plus the length of Checksum (2 bytes). Unit is byte. Max length is 256 bytes.
    //                  And high byte is transferred first.
    if let Some(data) = &data {
        let len: u16 = (1 + data.len() + 2).try_into().unwrap();
        let _ = send_buf.extend_from_slice(&len.to_be_bytes()[..]);
    };

    // DATA     -       It can be commands, data, command’s parameters, acknowledge result, etc.
    //                  (fingerprint character value, template are all deemed as data);

    // DATA
    let _ = send_buf.push(command.into());

    // DATA
    if let Some(data) = &data {
        let _ = send_buf.extend_from_slice(data);
    }

    // SUM      2 bytes The arithmetic sum of package identifier, package length and all package
    //                  contens. Overflowing bits are omitted. high byte is transferred first.
    let chk = r503::compute_checksum(send_buf.clone());
    let _ = send_buf.extend_from_slice(&chk.to_be_bytes()[..]);

    send_buf
}