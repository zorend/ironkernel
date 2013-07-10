#[allow(ctypes)];
#[no_std];
#[no_core];

use drivers::cga;
use drivers::keyboard;
use drivers::pic;

pub mod zero;
mod drivers {
    pub mod cga;
    pub mod keyboard;
    pub mod pic;
}

#[inline]
pub fn size_of_val<T>(_val: *mut T) -> uint {
    unsafe { zero::size_of::<T>() }
}

#[packed]
struct idt_reg {
    size: u16,
    addr: *mut [idt_entry, ..256],
}

static Present: u8 = 1 << 7;
static PM32Bit: u8 = 1 << 3;

#[packed]
struct idt_entry {
    addr_lo: u16,
    sel: u16,
    zero: u8,
    flags: u8,
    addr_hi: u16
}

fn idt_entry(proc: u32, sel: u16, flags: u8) -> idt_entry {
    idt_entry {
        addr_lo: (proc & 0xffff) as u16,
        sel: sel,
        zero: 0,
        flags: flags | 0b110,
        addr_hi: (proc >> 16) as u16
    }
}

#[no_mangle]
extern "C" fn keyup(code: u32) { }

pub static ascii_table: &'static str = "\
\x00\x1B1234567890-=\x08\
\tqwertyuiop[]\n\
\x00asdfghjkl;'`\
\x00\\zxcvbnm,./\x00\
*\x00 ";

fn keydown(code: u32) {
    // mutable statics are incorrectly dereferenced in PIC!
    static mut pos: u32 = 0;

    if(code & (1 << 7) == 0) {
        unsafe {
            let char = ascii_table[code];
            if char == 8 && pos > 0 {
                pos -= 1;
                (*cga::screen)[pos] &= 0xff00;
            } else if char == '\n' as u8 {
                pos += 80 - pos % 80;
            } else {
                (*cga::screen)[pos] |= char as u16;
                pos += 1;
            }
        }
    }
}

#[no_mangle]
pub unsafe fn main() {
    cga::clear_screen(cga::LightRed);
    // invalid deref when &fn?
    keyboard::callback = keyboard::Some(keydown);

    let idt = 0x100000 as *mut [idt_entry, ..256];

    (*idt)[keyboard::IRQ] = idt_entry(keyboard::isr_addr(), 1 << 3, PM32Bit | Present);

    let idt_table = 0x100800 as *mut idt_reg;
    *idt_table = idt_reg {
        addr: idt,
        size: size_of_val(idt) as u16
    };

    pic::remap();
    pic::enable(keyboard::IRQ);

    asm!("
        lidt [$0]
        sti"
        :: "n"(idt_table) :: "intel");
}
