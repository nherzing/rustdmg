#![macro_use]

// macro_rules! b0 {
//     ($x:expr) => (($x >> 0) & 0x1);
// }

// macro_rules! b1 {
//     ($x:expr) => (($x >> 1) & 0x1);
// }

macro_rules! b2 {
    ($x:expr) => (($x >> 2) & 0x1);
}

// macro_rules! b3 {
//     ($x:expr) => (($x >> 3) & 0x1);
// }

// macro_rules! b4 {
//     ($x:expr) => (($x >> 4) & 0x1);
// }

// macro_rules! b5 {
//     ($x:expr) => (($x >> 5) & 0x1);
// }

// macro_rules! b6 {
//     ($x:expr) => (($x >> 6) & 0x1);
// }

macro_rules! b7 {
    ($x:expr) => (($x >> 7) & 0x1);
}
