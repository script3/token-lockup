#![no_std]

pub mod contract;
mod errors;
mod storage;
mod types;
mod validation;

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod testutils;
