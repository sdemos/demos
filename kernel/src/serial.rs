//! serial driver

use spin::Mutex;
use x86_64::instructions::port::Port;

pub static COM1: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3f8));

pub unsafe fn init() {
    COM1.lock().init();
}

pub struct SerialPort {
    /// data register. when receiving, data is read from here. when writing, it
    /// is written here. when DLAB is set to 1, this is the least significant
    /// byte of the divisor value.
    data: Port<u8>,
    /// interrupt enable register. if interrupts are enabled, it seems they will
    /// be identified in the fifo_ctl register. when DLAB is set to 1, this is
    /// the most significant byte of the divisor value.
    int_en: Port<u8>,
    /// FIFO control register. enables/disables fifo and controls the size of
    /// the queue. also contains interrupt identification if interrupts are
    /// enabled.
    fifo_ctl: Port<u8>,
    /// line control register. this contains a bunch of important bits for
    /// various serial settings, notably controlling character length, stop
    /// bits, parity, and enabling/disabling DLAB.
    line_ctl: Port<u8>,
    /// modem control register. it's not clear to me what this is used for, but
    /// the initialization sequence detailed on the osdev wiki writes to it, so
    /// here we are.
    modem_ctl: Port<u8>,
    /// line status register. probably holds information on the current line
    /// status.
    line_sts: Port<u8>,
    /// modem status register. doesn't seem useful for me?
    modem_sts: Port<u8>,
    // there is also a scratch register.
}

impl SerialPort {
    /// new creates a new SerialPort object. it DOES NOT initialize the serial
    /// port. consumers must call the init function on startup before using the
    /// serial port.
    pub const fn new(base: u16) -> Self {
        SerialPort {
            data:      Port::new(base),
            int_en:    Port::new(base + 1),
            fifo_ctl:  Port::new(base + 2),
            line_ctl:  Port::new(base + 3),
            modem_ctl: Port::new(base + 4),
            line_sts:  Port::new(base + 5),
            modem_sts: Port::new(base + 6),
        }
    }

    /// init initializes the serial port by writing to a variety of control
    /// registers. it sets the port to 38400 baud, with 8 bit character length,
    /// no parity, and one stop bit. it also disables interrupts and configures
    /// the uart to set up a 14 bit FIFO. there are a few other things it sets
    /// that I don't understand yet.
    ///
    /// TODO: all these settings should be set with bitflags instead of as
    /// opaque hex values. it might also be useful to allow configuration.
    pub unsafe fn init(&mut self) {
        self.int_en.write(0x00);
        self.line_ctl.write(0x80);
        self.data.write(0x03);
        self.int_en.write(0x00);
        self.line_ctl.write(0x03);
        self.fifo_ctl.write(0xC7);
        self.modem_ctl.write(0x0B);
        self.int_en.write(0x01);
    }

    fn input_empty(&self) -> bool {
        unsafe {
            self.line_sts.read() & 1 == 0
        }
    }

    pub fn receive(&mut self) -> u8 {
        while self.input_empty() {}
        unsafe {
            self.data.read()
        }
    }

    fn output_empty(&self) -> bool {
        unsafe {
            self.line_sts.read() & 0x20 != 0
        }
    }

    pub fn send(&mut self, data: u8) {
        while !self.output_empty() {}
        unsafe {
            self.data.write(data);
        }
    }
}
