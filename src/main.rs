#[derive(Debug)]
enum ConnectionType {
    Usb,
    Bluetooth,
}

struct ControllerRawState {
    up: bool,
    up_left: bool,
    up_right: bool,
    down: bool,
    down_left: bool,
    down_right: bool,
    left: bool,
    right: bool,
    triangle: bool,
    circle: bool,
    x: bool,
    square: bool,
    touch_click: bool,
    l1: bool,
    r1: bool,
    l2: bool,
    r2: bool,
    options: bool,
    share: bool,
    l3: bool,
    r3: bool,
    touch_1: bool,
    touch_2: bool,
    left_axis_x: u8,
    left_axis_y: u8,
    right_axis_x: u8,
    right_axis_y: u8,
    l2_trigger: u8,
    r2_trigger: u8,
    touch_timestamp: u8,
    gyro_timestamp: u16,
    gyro_x: u16,
    gyro_y: u16,
    gyro_z: u16,
    accel_x: u16,
    accel_y: u16,
    accel_z: u16,
    touch_1_x: u16,
    touch_1_y: u16,
    touch_2_x: u16,
    touch_2_y: u16,
}

impl std::fmt::Display for ControllerRawState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

fn main() -> ! {
    let api = hidapi::HidApi::new().unwrap();

    let (vid, pid) = (1356, 2508);
    let device = match api.open(vid, pid) {
        Ok(device) => device,
        Err(e) => {
            println!("Error opening device: {:#?}", e);
            panic!();
        }
    };

    let mut report_buffer = [0u8; 78];

    // device.read returns Ok(size) where size is the size of the report
    // written on report_buffer
    let report_size = device.read(&mut report_buffer);

    // The type of connection is discovered based on the size
    // of the report written on buf
    let conn_type = match report_size {
        Ok(78) => ConnectionType::Bluetooth,
        Ok(64) => ConnectionType::Usb,
        Ok(_) => panic!(),
        Err(_) => panic!(),
    };

    println!("Connected with {:#?}", conn_type);

    loop {
        device
            .read(&mut report_buffer)
            .expect("Failed to get report.");

        let raw_state = match conn_type {
            ConnectionType::Bluetooth => decode_bluetooth(report_buffer),
            ConnectionType::Usb => decode_usb(report_buffer),
        };

        std::thread::sleep(std::time::Duration::from_millis(4));
    }
}

fn decode_usb(buf: [u8; 78]) -> ControllerRawState {
    ControllerRawState {
        up: buf[5] == 0b0000,
        up_left: buf[5] == 0b0111,
        up_right: buf[5] == 0b0001,
        down: buf[5] == 0b0100,
        down_left: buf[5] == 0b0101,
        down_right: buf[5] == 0b0011,
        left: buf[5] == 0b0110,
        right: buf[5] == 0b0010,
        triangle: (buf[5] & 0b10000000) == 128,
        circle: (buf[5] & 0b01000000) == 64,
        x: (buf[5] & 0b00100000) == 32,
        square: (buf[5] & 0b00010000) == 16,
        touch_click: (buf[7] & 0b00000010) == 2,
        l1: (buf[6] & 0b00000001) == 1,
        r1: (buf[6] & 0b00000010) == 2,
        l2: (buf[6] & 0b00000100) == 4,
        r2: (buf[6] & 0b00001000) == 8,
        options: (buf[6] & 0b00100000) == 32,
        share: (buf[6] & 0b00010000) == 16,
        l3: (buf[6] & 0b01000000) == 64,
        r3: (buf[6] & 0b10000000) == 128,
        touch_1: (buf[35] & 0b10000000) == 0,
        touch_2: (buf[39] & 0b10000000) == 0,
        left_axis_x: buf[1],
        left_axis_y: buf[2],
        right_axis_x: buf[3],
        right_axis_y: buf[4],
        l2_trigger: buf[8],
        r2_trigger: buf[9],
        touch_timestamp: buf[34],
        gyro_timestamp: u16::from_le_bytes([buf[10], buf[11]]),
        gyro_x: u16::from_le_bytes([buf[13], buf[14]]),
        gyro_y: u16::from_le_bytes([buf[15], buf[16]]),
        gyro_z: u16::from_le_bytes([buf[17], buf[18]]),
        accel_x: u16::from_le_bytes([buf[19], buf[20]]),
        accel_y: u16::from_le_bytes([buf[21], buf[22]]),
        accel_z: u16::from_le_bytes([buf[23], buf[24]]),

        // Touch position data is organized like this:
        // -----------------------------------------
        //                v-----x_pos----v
        // |0000|0000|0000|0000|0000|0000|0000|0000|
        //  ^----y_pos----^              ^--zeros--^
        // -----------------------------------------
        //
        // So to read it:
        //    - shift to the correct location
        //    - mask the last 3 bytes
        //    - store the result in a u16
        touch_1_x: {
            let data = u32::from_le_bytes([0, buf[36], buf[37], buf[38]]);
            ((data >> 8) & 0x00000FFF) as u16
        },
        touch_1_y: {
            let data = u32::from_le_bytes([0, buf[36], buf[37], buf[38]]);
            ((data >> 20) & 0x00000FFF) as u16
        },
        touch_2_x: {
            let data = u32::from_le_bytes([0, buf[40], buf[41], buf[42]]);
            ((data >> 8) & 0x00000FFF) as u16
        },
        touch_2_y: {
            let data = u32::from_le_bytes([0, buf[40], buf[41], buf[42]]);
            ((data >> 20) & 0x00000FFF) as u16
        },
    }
}

fn decode_bluetooth(buf: [u8; 78]) -> ControllerRawState {
    ControllerRawState {
        up: buf[7] == 0b0000,
        up_left: buf[7] == 0b0111,
        up_right: buf[7] == 0b0001,
        down: buf[7] == 0b0100,
        down_left: buf[7] == 0b0101,
        down_right: buf[7] == 0b0011,
        left: buf[7] == 0b0110,
        right: buf[7] == 0b0010,
        triangle: (buf[7] & 0b10000000) == 128,
        circle: (buf[7] & 0b01000000) == 64,
        x: (buf[7] & 0b00100000) == 32,
        square: (buf[7] & 0b00010000) == 16,
        touch_click: (buf[9] & 0b00000010) == 2,
        l1: (buf[8] & 0b00000001) == 1,
        r1: (buf[8] & 0b00000010) == 2,
        l2: (buf[8] & 0b00000100) == 4,
        r2: (buf[8] & 0b00001000) == 8,
        options: (buf[8] & 0b00100000) == 32,
        share: (buf[8] & 0b00010000) == 16,
        l3: (buf[8] & 0b01000000) == 64,
        r3: (buf[8] & 0b10000000) == 128,
        touch_1: (buf[37] & 0b10000000) == 0,
        touch_2: (buf[41] & 0b10000000) == 0,
        left_axis_x: buf[3],
        left_axis_y: buf[4],
        right_axis_x: buf[5],
        right_axis_y: buf[6],
        l2_trigger: buf[10],
        r2_trigger: buf[11],
        touch_timestamp: buf[37],
        gyro_timestamp: u16::from_le_bytes([buf[12], buf[13]]),
        gyro_x: u16::from_le_bytes([buf[15], buf[16]]),
        gyro_y: u16::from_le_bytes([buf[17], buf[18]]),
        gyro_z: u16::from_le_bytes([buf[19], buf[20]]),
        accel_x: u16::from_le_bytes([buf[21], buf[22]]),
        accel_y: u16::from_le_bytes([buf[23], buf[24]]),
        accel_z: u16::from_le_bytes([buf[25], buf[26]]),

        // Touch position data is organized like this:
        // -----------------------------------------
        //                v-----x_pos----v
        // |0000|0000|0000|0000|0000|0000|0000|0000|
        //  ^----y_pos----^              ^--zeros--^
        // -----------------------------------------
        //
        // So to read it:
        //    - shift to the correct location
        //    - mask the last 3 bytes
        //    - store the result in a u16
        touch_1_x: {
            let data = u32::from_le_bytes([0, buf[38], buf[39], buf[40]]);
            ((data >> 8) & 0x00000FFF) as u16
        },
        touch_1_y: {
            let data = u32::from_le_bytes([0, buf[38], buf[39], buf[40]]);
            ((data >> 20) & 0x00000FFF) as u16
        },
        touch_2_x: {
            let data = u32::from_le_bytes([0, buf[42], buf[43], buf[44]]);
            ((data >> 8) & 0x00000FFF) as u16
        },
        touch_2_y: {
            let data = u32::from_le_bytes([0, buf[42], buf[43], buf[44]]);
            ((data >> 20) & 0x00000FFF) as u16
        },
    }
}
