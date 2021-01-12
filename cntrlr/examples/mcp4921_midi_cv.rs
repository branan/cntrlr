// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

// This example reads MIDI commands from serial_1, and outputs pitch
// and velocity to a pair of SPI DACs, using hardware chip
// selects. With appropriate scaling, these outputs could be used as
// control voltages for a synthesizer.
//
// To build on the Teensy LC, hadware chip select should be disabled.
//
// Required components:
// * 2x Motorola MCP4921 SPI DAC
// * 1x MIDI-compatible optoisolator
// * 1x 5-pin DIN connector
//
// Wiring:
// 0 -> Optoisolator collector & 3.3V
// 10 -> DAC 0 chip select
// 15 -> DAC 1 chip select
// 11 -> DAC 0&1 serial in
// 13 -> DAC 0&1 serial clock
// 16 -> DAC 0&1 latch
// Optoisolator inputs across MIDI connector pins 4+5

#![no_std]
#![no_main]
#![allow(dead_code)]

use bit_field::BitField;
use cntrlr::prelude::*;

// Per the MIDI specification
const MIDI_BAUD: usize = 31250;
// The MCP4921 supports up to 20MHz, but on a breadboard we should be
// conservative.
const SPI_BAUD: usize = 2_000_000;
// Pins 10 and 15 are hardware CS on all Teensies except for the LC as
// well as the Red-V
const PITCH_DAC_CS: usize = 10;
const VELOCITY_DAC_CS: usize = 15;
// DAC load is done by GPIO
const DAC_LOAD: usize = 16;

enum MidiMessage {
    NoteOn(u8, u8),
    NoteOff(u8, u8),
}

async fn get_midi_message<R: Read + 'static>(
    reader: &mut R,
) -> Result<MidiMessage, <R as Read>::Error> {
    loop {
        let mut byte: u8 = 0;
        reader.read_exact(core::slice::from_mut(&mut byte)).await?;
        match byte.get_bits(4..8) {
            8 => {
                let mut data: [u8; 2] = [0, 0];
                reader.read_exact(&mut data).await?;
                return Ok(MidiMessage::NoteOff(data[0], data[1]));
            }
            9 => {
                let mut data: [u8; 2] = [0, 0];
                reader.read_exact(&mut data).await?;
                return Ok(MidiMessage::NoteOn(data[0], data[1]));
            }
            _ => {
                // Ignore any unrecognized messages and wait for the next one
            }
        }
    }
}

async fn write_dac<S: Spi>(spi: &mut S, value: u8, cs: usize) -> Result<(), <S as Spi>::Error> {
    // Upscale 7-bit MIDI value to 12-bit DAC. Upper nybble is control
    // (see datasheet for details).
    let mut control: u16 = 0x3000;
    control.set_bits(5..12, value.get_bits(0..7) as u16);
    control.set_bits(0..5, value.get_bits(2..7) as u16);

    spi.transfer(SPI_BAUD, cs, 16)
        .await?
        .write_all(&control.to_be_bytes())
        .await?;

    Ok(())
}

async fn write_note<S: Spi>(spi: &mut S, pitch: u8, velocity: u8) -> Result<(), <S as Spi>::Error> {
    write_dac(spi, pitch, PITCH_DAC_CS).await?;
    write_dac(spi, velocity, VELOCITY_DAC_CS).await?;
    spi.flush().await?;
    digital_write(DAC_LOAD, false);
    digital_write(DAC_LOAD, true);
    Ok(())
}

#[cfg(any(board = "teensy_30", board = "teensy_32", board = "teensy_35"))]
#[entry]
async fn main() -> ! {
    serial_1()
        .enable(MIDI_BAUD)
        .expect("Could not initialize MIDI serial port");
    spi_1()
        .enable_with_options(&[
            SpiOption::HardwareCs(PITCH_DAC_CS),
            SpiOption::HardwareCs(VELOCITY_DAC_CS),
        ])
        .expect("Could not enable DAC SPI port");

    pin_mode(DAC_LOAD, PinMode::Output);
    digital_write(DAC_LOAD, true);

    write_note(&mut *spi_1(), 0, 0)
        .await
        .expect("Could not write note do DACs");
    let mut last_pitch: u8 = 0;
    loop {
        match get_midi_message(&mut *serial_1())
            .await
            .expect("Error reading MIDI message")
        {
            MidiMessage::NoteOn(pitch, velocity) => {
                last_pitch = pitch;
                write_note(&mut *spi_1(), pitch, velocity)
                    .await
                    .expect("Could not write note do DACs");
            }
            MidiMessage::NoteOff(pitch, _velocity) => {
                if last_pitch == pitch {
                    write_note(&mut *spi_1(), pitch, 0)
                        .await
                        .expect("Could not write note do DACs");
                }
            }
        }
    }
}

#[cfg(not(any(board = "teensy_30", board = "teensy_32", board = "teensy_35")))]
#[entry]
async fn main() -> ! {
    core::future::pending().await
}
