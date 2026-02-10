use macros::{enum_ptr, enum_u32};
use crate::memory::{dev_barrier, gcc_mb};
use crate::println;

const MBOX_CHANNEL: u32 = 8;
const GPU_MEM_OFFSET: u32 = 0x40000000;

enum_ptr! {
    enum MBOX_REG {
        READ = 0x2000B880,
        STATUS = 0x2000B898,
        WRITE = 0x2000B8A0,
    }
}

enum_u32! {
    enum MBOX_STATUS {
        FULL = (1 << 31),
        EMPTY = (1 << 30),
    }
}

#[repr(C, align(16))]
struct Align16<T>(pub T);

#[repr(C, align(16))]
struct MailBoxMsg<const N: usize> {
    buf_size: u32,
    buf_req_resp_code: u32,
    tag_id: u32,
    tag_value_buffer_size: u32,
    tag_req_resp_code: u32,
    value_buf: [u8; N],
    end_tag: u32
}

impl<const N: usize> MailBoxMsg<N> {
    fn new(tag_id: u32) -> MailBoxMsg<N> {
        let buf_size = size_of::<MailBoxMsg<N>>() as u32;

        MailBoxMsg {
            buf_size,
            buf_req_resp_code: 0,
            tag_id,
            tag_value_buffer_size: N as u32,
            tag_req_resp_code: 0,
            value_buf: [0u8; N],
            end_tag: 0
        }
    }

    fn as_ptr(&self) -> *const u32 {
        self as *const MailBoxMsg<N> as *const u32
    }

    fn get_value(&self) -> &[u8] {
        self.value_buf.as_ref()
    }
}

fn mbox_write(data: *const u32) {
    // Check alignment
    let data_addr = data as u32;
    assert_eq!(data_addr % 16, 0);

    dev_barrier();

    unsafe {
        // Wait until not full
        while MBOX_REG::STATUS.as_ptr::<u32>().read_volatile() & MBOX_STATUS::FULL.val() != 0 {}

        // Write the data
        MBOX_REG::WRITE.as_mut_ptr::<u32>().write_volatile(data_addr | GPU_MEM_OFFSET | MBOX_CHANNEL);
    }

    dev_barrier();
}

fn mbox_read() -> u32 {
    dev_barrier();

    let v: u32;

    unsafe {
        // Wait until not empty
        while MBOX_REG::STATUS.as_ptr::<u32>().read_volatile() & MBOX_STATUS::EMPTY.val() != 0 {}

        // Read from mailbox
        v = MBOX_REG::READ.as_ptr::<u32>().read_volatile();

        // Check channel
        assert_eq!(v & 0xf, MBOX_CHANNEL);
    }

    dev_barrier();

    v
}

pub fn mbox_send<const N: usize>(msg: &MailBoxMsg<N>) {
    let data = msg.as_ptr();

    // unsafe {
    //     for i in 0..8 {
    //         println!("{}: {:08x}", i, data.add(i).read_volatile());
    //     }
    // }

    gcc_mb();
    mbox_write(data);
    mbox_read();
    gcc_mb();

    // unsafe {
    //     for i in 0..8 {
    //         println!("{}: {:08x}", i, data.add(i).read_volatile());
    //     }
    // }

    unsafe {
        let result = data.add(1).read_volatile();
        assert_eq!(result, 0x80000000);
    }
}

pub fn mbox_get_serial_num() -> u64 {
    let msg = MailBoxMsg::<8>::new(0x00010004);

    mbox_send(&msg);

    let result = msg.get_value();
    u32::from_le_bytes(result[0..4].try_into().unwrap()) as u64
}

pub fn mbox_get_model() -> u32 {
    let msg = MailBoxMsg::<4>::new(0x00010001);

    mbox_send(&msg);

    let result = msg.get_value();
    u32::from_le_bytes(result[0..4].try_into().unwrap())
}

pub fn mbox_get_revision() -> u32 {
    let msg = MailBoxMsg::<4>::new(0x00010002);
    mbox_send(&msg);
    let result = msg.get_value();
    u32::from_le_bytes(result[0..4].try_into().unwrap())
}

pub fn mbox_get_memory() -> u32 {
    let msg = MailBoxMsg::<8>::new(0x00010005);
    mbox_send(&msg);
    let result = msg.get_value();
    u32::from_le_bytes(result[4..].try_into().unwrap())
}

pub fn mbox_get_temperature() -> u32 {
    let msg = MailBoxMsg::<8>::new(0x0003000a);
    mbox_send(&msg);
    let result = msg.get_value();
    u32::from_le_bytes(result[4..].try_into().unwrap())
}