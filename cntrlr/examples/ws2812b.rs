// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

// This provides an example of a framebuffer for a panel of WS2812B
// programmable RGB LEDs. It uses a UART to generate the bit pattern
// used on the chip.
//
// The assumed panel layout is a zig zag, with each row being
// continuous and each consecutive row reversing direction, as below:
//    PANEL_LENGTH
// /---------------\
// -O-O-O-O-O-O-O-O \
//                | |
//  O-O-O-O-O-O-O-O |
//  |               |
//  O-O-O-O-O-O-O-O |
//                | |
//  O-O-O-O-O-O-O-O |
//  |               | PANEL_HEIGHT
//  O-O-O-O-O-O-O-O |
//                | |
//  O-O-O-O-O-O-O-O |
//  |               |
//  O-O-O-O-O-O-O-O |
//                | |
//  O-O-O-O-O-O-O-O /
//  |
//
// The size of the panel can be adjusted with the PANEL_LENGTH and
// PANEL_HEIGHT constants.
//
// To change the pattern, modify the `render` function. This function
// is responsible for knowing the layout of your LEDs in the
// chain. The default implementation renders a diagonal rainbow
// pattern which cycles based on the frame number.
//
// The `display` function is responsible for sending the pixel data
// over the UART.
//
// Wiring:
// 0 -> WS2812B data in
//
// Note that USB power is typically insufficient for large strings of
// WS2812Bs. When operating your LEDs and board with separate power
// supplies, be sure to connect the grounds together.

#![no_std]
#![no_main]

const PANEL_LENGTH: usize = 8;
const PANEL_HEIGHT: usize = 8;

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
struct Panel<const L: usize, const H: usize>
where
    [[Color; L]; H]: Default,
{
    buffer: [[Color; L]; H],
}

impl<const L: usize, const H: usize> Panel<L, H>
where
    [[Color; L]; H]: Default,
{
    fn new() -> Self {
        Self {
            buffer: [[COLORS[0]; L]; H],
        }
    }

    fn render(&mut self, frame: usize) {
        for (i, row) in self.buffer.iter_mut().enumerate() {
            for (j, pixel) in row.iter_mut().enumerate() {
                let j = if i % 2 == 0 { j } else { 7 - j };
                let color = (i + j + frame) % COLORS.len();
                *pixel = COLORS[color]
            }
        }
    }

    async fn display<W: Write>(&mut self, writer: &mut W) -> Result<(), <W as Write>::Error> {
        for row in self.buffer.iter() {
            for pixel in row.iter() {
                write_pixel(*pixel, writer).await?;
            }
        }
        Ok(())
    }
}

async fn write_pixel<W: Write>(pixel: Color, writer: &mut W) -> Result<(), <W as Write>::Error> {
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
    let mut panel = Panel::<PANEL_LENGTH, PANEL_HEIGHT>::new();
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
