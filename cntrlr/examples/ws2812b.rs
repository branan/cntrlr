// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

#![no_std]
#![no_main]

use bit_field::BitField;
use cntrlr::prelude::*;

#[derive(Clone, Copy, Default)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

const COLORS: [Color; 12] = [
    Color { r: 255, g: 0, b: 0 },
    Color {
        r: 191,
        g: 64,
        b: 0,
    },
    Color {
        r: 127,
        g: 128,
        b: 0,
    },
    Color {
        r: 64,
        g: 191,
        b: 0,
    },
    Color { r: 0, g: 255, b: 0 },
    Color {
        r: 0,
        g: 191,
        b: 64,
    },
    Color {
        r: 0,
        g: 127,
        b: 128,
    },
    Color {
        r: 0,
        g: 64,
        b: 191,
    },
    Color { r: 0, g: 0, b: 255 },
    Color {
        r: 64,
        g: 0,
        b: 191,
    },
    Color {
        r: 128,
        g: 0,
        b: 127,
    },
    Color {
        r: 191,
        g: 0,
        b: 64,
    },
];

#[derive(Default)]
struct Panel([[Color; 8]; 8]);

impl Panel {
    fn new() -> Self {
        Default::default()
    }

    fn render(&mut self, frame: usize) {
        for i in 0..8 {
            for j in 0..8 {
                let k = if i % 2 == 0 { j } else { 7 - j };
                let color = (i + k + frame) % COLORS.len();
                self.0[i][j] = COLORS[color]
            }
        }
    }

    async fn display<W: Write>(&mut self, writer: &mut W) -> Result<(), <W as Write>::Error> {
        for i in 0..8 {
            for j in 0..8 {
                write_pixel(&self.0[i][j], writer).await?;
            }
        }
        Ok(())
    }
}
async fn write_pixel<W: Write>(pixel: &Color, writer: &mut W) -> Result<(), <W as Write>::Error> {
    write_byte(pixel.g, writer).await?;
    write_byte(pixel.r, writer).await?;
    write_byte(pixel.b, writer).await?;
    Ok(())
}

async fn write_byte<W: Write>(byte: u8, writer: &mut W) -> Result<(), <W as Write>::Error> {
    write_bits(byte.get_bits(6..8), writer).await?;
    write_bits(byte.get_bits(4..6), writer).await?;
    write_bits(byte.get_bits(2..4), writer).await?;
    write_bits(byte.get_bits(0..2), writer).await?;
    Ok(())
}

async fn write_bits<W: Write>(bits: u8, writer: &mut W) -> Result<(), <W as Write>::Error> {
    let out_byte = match bits {
        0 => 0b11001110,
        1 => 0b10001110,
        2 => 0b11001100,
        3 => 0b10001100,
        _ => unreachable!(),
    };

    while writer.write(&[out_byte]).await? == 0 {}
    Ok(())
}

#[entry]
async fn main() -> ! {
    let mut panel = Panel::new();
    let mut frame = 0;
    serial_1()
        .enable_with_options(4_000_000, &[SerialOption::Invert(true)])
        .expect("Could not enable serial port");
    loop {
        let frame_start = millis();
        panel.render(frame);
        panel
            .display(&mut *serial_1())
            .await
            .expect("Failed to display frame");
        frame += 1;
        let frame_end = millis();
        let frame_len = frame_end.saturating_sub(frame_start);
        // Maximum of 10 FPS, or as fast as possible on slower micros.
        sleep_millis(100usize.saturating_sub(frame_len)).await;
    }
}
