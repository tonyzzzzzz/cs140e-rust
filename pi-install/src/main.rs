extern crate core;

use bytes::{BufMut, BytesMut};
use clap::Parser;
use constants::BOOT_OP::{PUT_CODE, PUT_PROG_INFO};
use constants::{ARM_BASE, BOOT_OP, UART_BAUD_RATE};
use serialport::Error;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser, Debug)]
struct PiInstall {
    #[arg(short, long, default_value_t = UART_BAUD_RATE)]
    baud: u32,

    #[arg(short, long, default_value = "0x8000")]
    addr: String,

    #[arg(short, long, default_value = "true")]
    last: bool,

    #[arg(short, long, default_value = "false")]
    first: bool,

    #[arg(short, long)]
    device: Option<PathBuf>,

    kernel: PathBuf,
}

fn find_serial_port(first: bool) -> Option<PathBuf> {
    let ports = serialport::available_ports().expect("Failed to enumerate serial ports");

    let mut metadata = ports
        .iter()
        .filter(|x| {
            let serialport::SerialPortType::UsbPort(port_info) = &x.port_type else {
                return false;
            };
            port_info.vid == 0x10c4 && port_info.pid == 0xea60
        })
        .map(|x| {
            (
                fs::metadata(x.port_name.clone()).unwrap(),
                x.port_name.clone(),
            )
        })
        .collect::<Vec<_>>();

    metadata.sort_by(|a, b| a.0.modified().unwrap().cmp(&b.0.modified().unwrap()));

    if first {
        return metadata.first().map(|x| PathBuf::from(x.1.clone()));
    }

    metadata.last().map(|x| PathBuf::from(x.1.clone()))
}

fn open_serial_port(
    port_name: &PathBuf,
    baud_rate: u32,
) -> Result<Box<dyn serialport::SerialPort>, Error> {
    let mut last_error = None;
    for i in 0..5 {
        match serialport::new(port_name.to_string_lossy(), baud_rate)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_secs(5))
            .open()
        {
            Ok(port) => return Ok(port),
            Err(e) => {
                last_error = Some(e.clone());
                println!("Failed to open port {:?}: {}", port_name, e);
            }
        }
    }

    Err(last_error.unwrap())
}

fn get_op(buf: &[u8; 4]) -> Option<BOOT_OP> {
    let op_code = u32::from_le_bytes(*buf);

    BOOT_OP::from_u32(op_code)
}

fn read_code(file: &PathBuf) -> Result<Vec<u8>, Error> {
    let mut f = fs::File::open(file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    let args = PiInstall::parse();

    /*
       OPEN THE PORT
    */
    let ports = serialport::available_ports().expect("Failed to enumerate serial ports");
    for p in ports {
        println!("{:?}", p);
    }

    let port_name = match args.device {
        Some(p) => p,
        None => find_serial_port(args.last).expect("Cannot find any serial port"),
    };

    let mut port = open_serial_port(&port_name, args.baud).expect("Failed to open serial port");

    println!("Using port {:?}", port_name);

    /*
       READ PROGRAM
    */
    let program = read_code(&args.kernel).expect(format!("Failed to read program at {}", args.kernel.display()).as_str() );
    let crc_32 = crc32fast::hash(&program);
    let nbytes = program.len();
    println!("Program size: {} bytes, crc32={:x}", nbytes, crc_32);

    /*
       MAIN LOOP
    */
    let mut get_prog_handled = false;
    let mut buf = [0u8; 4];
    while let Ok(()) = port.read_exact(&mut buf) {
        match get_op(&buf) {
            Some(BOOT_OP::GET_PROG_INFO) => {
                if get_prog_handled {
                    continue;
                }

                println!("Sending program info...");

                let mut write_buf = BytesMut::with_capacity(4 * 4);
                write_buf.put_u32_le(PUT_PROG_INFO.into());
                write_buf.put_u32_le(ARM_BASE);
                write_buf.put_u32_le(nbytes as u32);
                write_buf.put_u32_le(crc_32);
                port.write_all(&write_buf)
                    .expect("Failed to write program info");

                get_prog_handled = true;
            }
            Some(BOOT_OP::GET_CODE) => {

                println!("Sending code...");

                // Check CRC32 is the same
                port.read_exact(&mut buf)
                    .expect("Failed to read CRC32 echo");

                let crc_32_echo = u32::from_le_bytes(buf);
                assert_eq!(crc_32_echo, crc_32);

                let mut write_buf = BytesMut::with_capacity(4 + nbytes);
                write_buf.put_u32_le(PUT_CODE.into());
                write_buf.put_slice(&program);
                port.write_all(&write_buf).expect("Failed to write code");
            }
            Some(BOOT_OP::BOOT_ERROR) => panic!("Boot failed"),
            Some(BOOT_OP::BOOT_SUCCESS) => {
                println!("Boot successful, starting...");
                port.set_timeout(Duration::from_hours(1)).expect("Failed to set timeout");
                break;
            },
            Some(BOOT_OP::PRINT_STRING) => {
                port.read_exact(&mut buf)
                    .expect("Failed to read string length");

                let n_read = u32::from_le_bytes(buf);
                let mut string = vec![0u8; n_read as usize];
                port.read_exact(&mut string).expect("Failed to read string");

                println!("PI: {}", String::from_utf8(string).unwrap());
            }
            Some(x) => println!("Unimplemented OP: {:?}", x),
            None => {
                println!("Unknown op code discarding: {:?}", buf);
                port.read_exact(&mut [0u8; 1])
                    .expect("Failed to skip a byte");
            }
        }
    }

    println!("Starts to print output from PI:");

    let mut read_buf = [0u8; 1024];
    while let Ok(n) = port.read(&mut read_buf) {
        if n == 0 {
            sleep(Duration::from_micros(1000));
            continue
        }

        let output = String::from_utf8_lossy(&read_buf[..n]);
        print!("{}", output);
    }

}
