use crate::println;
#[allow(unused_imports)]
use crate::scheduler;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn set_masks(primary: u8, secondary: u8) {
    // SAFETY: writing the mask registers only enables or disables IRQ lines;
    // the PIC was initialized in `lib::init` and every unmasked line has a
    // handler in the IDT. Interrupts are off so a handler cannot take the
    // PICS lock in between.
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        PICS.lock().write_masks(primary, secondary)
    });
}

pub fn unmask_timer() {
    set_masks(0b1111_1110, 0b1111_1111);
}

pub fn mask_all() {
    set_masks(0b1111_1111, 0b1111_1111);
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// SAFETY: vectors 32..48 do not overlap the CPU exceptions (0..32), and this
// is the only `ChainedPics` driving ports 0x20/0xA0.
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!("Page fault exception");
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    crate::scheduler::tick();

    // SAFETY: called once, at the end of the timer handler, for the vector
    // that is being serviced.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), layouts::Azerty, HandleControl::Ignore)
        );
    }

    let mut port = Port::new(0x60);
    // SAFETY: port 0x60 is the PS/2 data port; reading it in the keyboard
    // IRQ handler consumes the pending scancode and has no other effect.
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    crate::shell::add_key(character);
                }
                DecodedKey::RawKey(_key) => {}
            }
        }
    }

    // SAFETY: called once, at the end of the keyboard handler, for the vector
    // that is being serviced.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
