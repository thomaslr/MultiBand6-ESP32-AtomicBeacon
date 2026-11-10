//! Hardware GPIO pin definitions for MultiBand6-ESP32-AtomicBeacon.
//!
//! Target Board: ESP32-C3 SuperMini / Custom SMT/DIY Carrier Boards.
//!
//! Pin Assignments:
//! - GPIO 1: RF Carrier Output (LEDC Channel 0 PWM). Non-strapping pin.
//! - GPIO 3: Manual Bedside Sync Button (Tactile button SW1 / External header J3, Active LOW).
//! - GPIO 8: Onboard Status LED (Active LOW: 255 = OFF in standby, breathing PWM when transmitting).
//! - GPIO 9: Onboard ESP32-C3 SuperMini BOOT Button (Active LOW, dual-mapped as manual sync button).

pub const PIN_ANTENNA: i32 = 1;
pub const PIN_SYNC_BUTTON: i32 = 3;
pub const PIN_STATUS_LED: i32 = 8;
pub const PIN_BOOT_BUTTON: i32 = 9;
