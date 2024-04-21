// RCL -- A reasonable configuration language.
// Copyright 2024 Ruud van Asseldonk

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! Smith is a fuzzer that generates likely-interesting RCL expressions.
//!
//! See also the [`rcl_fuzz::smith`] module.

#![no_main]

use libfuzzer_sys::{fuzz_mutator, fuzz_target};
use rcl_fuzz::smith::SynthesizedProgram;
use rcl_fuzz::uber::fuzz_main;
use tinyrand::wyrand::Wyrand;
use tinyrand::{RandRange, Seeded};

fuzz_target!(|input: SynthesizedProgram| {
    fuzz_main(input.mode, &input.program);
});

/// Helper that implements a custom libFuzzer mutator.
struct Mutator<'a> {
    data: &'a mut [u8],
    size: usize,
    max_size: usize,
    rng: Wyrand,
}

impl<'a> Mutator<'a> {
    /// Generate a uniform random byte.
    fn gen_byte(&mut self) -> u8 {
        self.rng.next_range(0..0x100_u16) as u8
    }

    /// Return the byte offset of an arbitrary instruction in the buffer.
    fn gen_instruction_index(&mut self) -> Option<usize> {
        // Subtract 1 so we are sure to have an index of a full 2-byte instruction,
        // not a trailing 1-byte leftover.
        let i = std::cmp::min(self.size - 1, self.max_size - 1) / 2;

        if i == 0 {
            None
        } else {
            Some(self.rng.next_range(0..i) * 2)
        }
    }

    /// Return an arbitrary index into the data buffer.
    fn gen_data_index(&mut self) -> usize {
        // Bias indices towards the end of the data; the instructions are at the
        // start and auxiliary data is at the end. Instructions are 2 bytes, so
        // if we delete one byte in the middle then the part after it becomes
        // meaningless (they might still be valid instructions, but it's not a
        // small mutation). We should have more luck deleting in e.g. a string
        // at the end.
        let n = std::cmp::min(self.size, self.max_size);
        match self.rng.next_range(0..3_u16) {
            0 => n - 1,
            1 => self.rng.next_range((n / 2)..n),
            2 => self.rng.next_range(0..n),
            _ => unreachable!(),
        }
    }

    /// Generate a random valid opcode.
    fn gen_opcode(&mut self) -> u8 {
        loop {
            let opcode = self.gen_byte();
            if rcl_fuzz::smith::parse_opcode(opcode).is_some() {
                return opcode;
            }
        }
    }

    /// Generate an instruction argument.
    fn gen_argument(&mut self) -> u8 {
        // We bias the argument towards smaller numbers, because often they are
        // lengths or indexes into the stack, and those are all small.
        match self.rng.next_range(0..4_u16) {
            0 => 0,
            1 => 1,
            2 => self.rng.next_range(0..10_u16) as u8,
            3 => self.gen_byte(),
            _ => unreachable!(),
        }
    }

    fn mutate(&mut self) {
        // Some mutations don't succeed. For example, if the input is 1 byte,
        // we can't generate an instruction index. So try up to 8 times to get
        // a working mutation.
        for _ in 0..8 {
            let mutation = match self.rng.next_range(0..10_u16) {
                0 => self.insert_instruction(),
                1 => self.remove_instruction(),
                2 => self.replace_instruction(),
                3 => self.swap_instructions(),
                4 => self.increment_argument(),
                5 => self.decrement_argument(),
                6 => self.replace_argument(),
                7 => self.append_byte(),
                8 => self.remove_byte(),
                9 => self.mutate_libfuzzer(),
                _ => unreachable!(),
            };
            if mutation.is_some() {
                break;
            }
        }
    }

    fn insert_instruction(&mut self) -> Option<()> {
        if self.size + 2 >= self.max_size {
            return None;
        }

        let i = self.gen_instruction_index().unwrap_or(0);

        // Move everything behind the insertion place one instruction ahead.
        self.data.copy_within(i..self.data.len() - 2, i + 2);

        // Then insert the new instruction.
        self.data[i] = self.gen_opcode();
        self.data[i + 1] = self.gen_argument();
        self.size += 2;

        Some(())
    }

    fn remove_instruction(&mut self) -> Option<()> {
        let i = self.gen_instruction_index()?;

        // Move everything back one place.
        self.data.copy_within(i + 2.., i);
        self.size -= 2;

        Some(())
    }

    fn replace_instruction(&mut self) -> Option<()> {
        let i = self.gen_instruction_index()?;
        self.data[i] = self.gen_opcode();
        self.data[i + 1] = self.gen_argument();
        Some(())
    }

    fn swap_instructions(&mut self) -> Option<()> {
        let i = self.gen_instruction_index()?;
        let j = self.gen_instruction_index()?;
        if i == j {
            return None;
        }
        self.data.swap(i, j);
        self.data.swap(i + 1, j + 1);
        Some(())
    }

    fn increment_argument(&mut self) -> Option<()> {
        let i = self.gen_instruction_index()?;
        self.data[i + 1] = self.data[i + 1].saturating_add(1);
        Some(())
    }

    fn decrement_argument(&mut self) -> Option<()> {
        let i = self.gen_instruction_index()?;
        self.data[i + 1] = self.data[i + 1].saturating_sub(1);
        Some(())
    }

    fn replace_argument(&mut self) -> Option<()> {
        let i = self.gen_instruction_index()?;
        self.data[i + 1] = self.gen_argument();
        Some(())
    }

    fn append_byte(&mut self) -> Option<()> {
        if self.size >= self.data.len()
            || self.max_size >= self.data.len()
            || self.size + 1 >= self.max_size
        {
            return None;
        }
        // Bias values towards 0 or printable ASCII, the auxiliary data at the
        // end is often used for indices or strings.
        let b = match self.rng.next_range(0..2u16) {
            0 => 0,
            1 => self.rng.next_range(0x20..0x7f_u16) as u8,
            2 => self.gen_byte(),
            _ => unreachable!(),
        };
        self.data[self.size] = b;
        self.size += 1;
        Some(())
    }

    fn remove_byte(&mut self) -> Option<()> {
        let i = self.gen_data_index();
        self.data.copy_within(i + 1.., i);
        self.size -= 1;
        Some(())
    }

    fn mutate_libfuzzer(&mut self) -> Option<()> {
        // To avoid getting stuck in a monoculture with our own mutations,
        // call the upstream mutator occasionally.
        self.size = libfuzzer_sys::fuzzer_mutate(self.data, self.size, self.max_size);
        Some(())
    }
}

fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
    let rng = Wyrand::seed(seed as u64);
    let mut mutator = Mutator {
        data,
        size,
        max_size,
        rng,
    };
    mutator.mutate();
    mutator.size
});
