//! Board specific functionality for the Teensy 3.6

use core::sync::atomic::AtomicUsize;

pub mod digital;
pub mod io;

static CPU_FREQ: AtomicUsize = AtomicUsize::new(0);
static BUS_FREQ: AtomicUsize = AtomicUsize::new(0);

use crate::runtime::unused_interrupt;

/// The Teensy 3.6 interrupt table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_36", link_section = ".__CNTRLR_INTERRUPTS")]
#[cfg_attr(board = "teensy_36", export_name = "__cntrlr_interrupts")]
pub static INTERRUPTS: [unsafe extern "C" fn(); 100] = [
    unused_interrupt,  // 000
    unused_interrupt,  // 001
    unused_interrupt,  // 002
    unused_interrupt,  // 003
    unused_interrupt,  // 004
    unused_interrupt,  // 005
    unused_interrupt,  // 006
    unused_interrupt,  // 007
    unused_interrupt,  // 008
    unused_interrupt,  // 009
    unused_interrupt,  // 010
    unused_interrupt,  // 011
    unused_interrupt,  // 012
    unused_interrupt,  // 013
    unused_interrupt,  // 014
    unused_interrupt,  // 015
    unused_interrupt,  // 016
    unused_interrupt,  // 017
    unused_interrupt,  // 018
    unused_interrupt,  // 019
    unused_interrupt,  // 020
    unused_interrupt,  // 021
    unused_interrupt,  // 022
    unused_interrupt,  // 023
    unused_interrupt,  // 024
    unused_interrupt,  // 025
    unused_interrupt,  // 026
    unused_interrupt,  // 027
    unused_interrupt,  // 028
    unused_interrupt,  // 029
    unused_interrupt,  // 030
    io::serial_1_intr, // 031
    unused_interrupt,  // 032
    io::serial_2_intr, // 033
    unused_interrupt,  // 034
    io::serial_3_intr, // 035
    unused_interrupt,  // 036
    io::serial_4_intr, // 037
    unused_interrupt,  // 038
    unused_interrupt,  // 039
    unused_interrupt,  // 040
    unused_interrupt,  // 041
    unused_interrupt,  // 042
    unused_interrupt,  // 043
    unused_interrupt,  // 044
    unused_interrupt,  // 045
    unused_interrupt,  // 046
    unused_interrupt,  // 047
    unused_interrupt,  // 048
    unused_interrupt,  // 049
    unused_interrupt,  // 050
    unused_interrupt,  // 051
    unused_interrupt,  // 052
    unused_interrupt,  // 053
    unused_interrupt,  // 054
    unused_interrupt,  // 055
    unused_interrupt,  // 056
    unused_interrupt,  // 057
    unused_interrupt,  // 058
    unused_interrupt,  // 059
    unused_interrupt,  // 060
    unused_interrupt,  // 061
    unused_interrupt,  // 062
    unused_interrupt,  // 063
    unused_interrupt,  // 064
    unused_interrupt,  // 065
    io::serial_5_intr, // 066
    unused_interrupt,  // 067
    unused_interrupt,  // 068
    unused_interrupt,  // 069
    unused_interrupt,  // 070
    unused_interrupt,  // 071
    unused_interrupt,  // 072
    unused_interrupt,  // 073
    unused_interrupt,  // 074
    unused_interrupt,  // 075
    unused_interrupt,  // 076
    unused_interrupt,  // 077
    unused_interrupt,  // 078
    unused_interrupt,  // 079
    unused_interrupt,  // 080
    unused_interrupt,  // 081
    unused_interrupt,  // 082
    unused_interrupt,  // 083
    unused_interrupt,  // 084
    unused_interrupt,  // 085
    unused_interrupt,  // 086
    unused_interrupt,  // 087
    unused_interrupt,  // 088
    unused_interrupt,  // 089
    unused_interrupt,  // 090
    unused_interrupt,  // 091
    unused_interrupt,  // 092
    unused_interrupt,  // 093
    unused_interrupt,  // 094
    unused_interrupt,  // 095
    unused_interrupt,  // 096
    unused_interrupt,  // 097
    unused_interrupt,  // 009
    unused_interrupt,  // 099
];
