use super::n64::R4300;
use emu::bus::be::{Device, Reg32};
use mips64::Cop0;

use bit_field::BitField;
use bitflags::bitflags;
use slog;

bitflags! {
    pub struct IrqMask: u32 {
        const SP =            0b00000001;
        const SI =            0b00000010;
        const AI =            0b00000100;
        const VI =            0b00001000;
        const PI =            0b00010000;
        const DP =            0b00100000;
    }
}

#[derive(DeviceBE)]
pub struct Mi {
    #[reg(offset = 0x08, readonly)]
    irq_ack: Reg32,

    #[reg(offset = 0x0C, wcb)]
    irq_mask: Reg32,

    logger: slog::Logger,
}

impl Mi {
    pub fn new(logger: slog::Logger) -> Box<Mi> {
        Box::new(Mi {
            irq_ack: Reg32::default(),
            irq_mask: Reg32::default(),
            logger,
        })
    }

    pub fn set_irq_line(&mut self, mask: IrqMask, status: bool) {
        let old = self.irq_ack.get();
        let new = if status {
            old | mask.bits()
        } else {
            old & !mask.bits()
        };
        self.irq_ack.set(new);
        if old != new {
            info!(self.logger, "changed IRQ ack"; "irq" => ?IrqMask::from_bits(new));

            // FIXME: this reentrancy will eventually panic (CPU -> BUS -> Device -> CPU).
            // Find out a way to fix this.
            if new != 0 {
                R4300::get_mut().cop0.set_hwint_line(0, true);
            } else {
                R4300::get_mut().cop0.set_hwint_line(0, false);
            }
        }
    }

    fn cb_write_irq_mask(&mut self, old: u32, new: u32) {
        let mut mask = old;
        for i in 0..12 {
            if new.get_bit(i) {
                mask.set_bit(i / 2, i % 2 != 0);
            }
        }
        self.irq_mask.set(mask);
        info!(self.logger, "changed IRQ mask"; "irq" => ?IrqMask::from_bits(mask));
    }
}
