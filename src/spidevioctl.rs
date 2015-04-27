// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

use ioctl;
use std::mem;
use std::io;
use std::os::unix::io::RawFd;

// Constants extracted from linux/spi/spidev.h
bitflags! {
    flags SpiModeFlags: u8 {
        const SPI_CPHA = 0x01,
        const SPI_CPOL = 0x02,
        const SPI_MODE_0 = 0x00,
        const SPI_MODE_1 = SPI_CPHA.bits,
        const SPI_MODE_2 = SPI_CPOL.bits,
        const SPI_MODE_3 = (SPI_CPOL.bits | SPI_CPHA.bits),
    }
}

bitflags! {
    flags SpidevOptionFlags: u8 {
        const SPI_CS_HIGH = 0x04,
        const SPI_LSB_FIRST = 0x08,
        const SPI_3WIRE = 0x10,
        const SPI_LOOP = 0x20,
        const SPI_NO_CS = 0x40,
        const SPI_READY = 0x80,
    }
}

const SPI_IOC_MAGIC: u8 = 'k' as u8;

const SPI_IOC_NR_TRANSFER: u8 = 0;
const SPI_IOC_NR_MODE: u8 = 1;
const SPI_IOC_NR_LSB_FIRST: u8 = 2;
const SPI_IOC_NR_BITS_PER_WORD: u8 = 3;
const SPI_IOC_NR_MAX_SPEED_HZ: u8 = 4;
const SPI_IOC_NR_MODE32: u8 = 5;

/// Structure that is used when performing communication
/// with the kernel.
struct spi_ioc_transfer {
    pub tx_buf: u64,
    pub rx_buf: u64,
    
    pub len: u32,

    // optional overrides
    pub speed_hz: u32,
    pub delay_usecs: u16,
    pub bits_per_word: u8,
    pub cs_change: u8,
    pub pad: u32,
}

/// Representation of a spidev transfer that is shared
/// with external users
#[derive(Default)]
pub struct SpidevTransfer {
    tx_buf: Option<Box<[u8]>>,
    rx_buf: Option<Box<[u8]>>,
    len: u32,
    speed_hz: u32,
    delay_usecs: u16,
    bits_per_word: u8,
    cs_change: u8,
    pad: u32,
}

impl SpidevTransfer {
    pub fn read(length: u32) -> SpidevTransfer {
        let rx_buf_vec: Vec<u8> = Vec::with_capacity(length as usize);
        SpidevTransfer {
            tx_buf: None,
            rx_buf: Some(rx_buf_vec.into_boxed_slice()),
            len: length as u32,
            ..Default::default()
        }
    }

    pub fn write(tx_buf: &[u8]) -> SpidevTransfer {
        let rx_buf_vec: Vec<u8> = Vec::with_capacity(tx_buf.len());
        let mut tx_buf_vec: Vec<u8> = Vec::with_capacity(tx_buf.len());
        tx_buf_vec.clone_from_slice(tx_buf);
        SpidevTransfer {
            tx_buf: Some(tx_buf_vec.into_boxed_slice()),
            rx_buf: Some(rx_buf_vec.into_boxed_slice()),
            len: tx_buf.len() as u32,
            ..Default::default()
        }
    }

}

fn spidev_ioc_read<T>(fd: RawFd, nr: u8) -> io::Result<T> {
    let size: u16 = mem::size_of::<T>() as u16;
    let op = ioctl::op_read(SPI_IOC_MAGIC, nr, size);
    unsafe { ioctl::read(fd, op) }
}

fn spidev_ioc_write<T>(fd: RawFd, nr: u8, data: &T) -> io::Result<()> {
    let size: u16 = mem::size_of::<T>() as u16;
    let op = ioctl::op_write(SPI_IOC_MAGIC, nr, size);
    unsafe { ioctl::write(fd, op, data) }
}

pub fn get_mode(fd: RawFd) -> io::Result<u8> {
    // #define SPI_IOC_RD_MODE _IOR(SPI_IOC_MAGIC, 1, __u8)
    spidev_ioc_read::<u8>(fd, SPI_IOC_NR_MODE)
}

pub fn set_mode(fd: RawFd, mode: SpiModeFlags) -> io::Result<()> {
    // #define SPI_IOC_WR_MODE _IOW(SPI_IOC_MAGIC, 1, __u8)
    spidev_ioc_write(fd, SPI_IOC_NR_MODE, &mode.bits)
}

pub fn get_lsb_first(fd: RawFd) -> io::Result<bool> {
    // #define SPI_IOC_RD_LSB_FIRST _IOR(SPI_IOC_MAGIC, 2, __u8)
    Ok(try!(spidev_ioc_read::<u8>(fd, SPI_IOC_NR_LSB_FIRST)) != 0)
}

pub fn set_lsb_first(fd: RawFd, lsb_first: bool) -> io::Result<()> {
    // #define SPI_IOC_WR_LSB_FIRST _IOW(SPI_IOC_MAGIC, 2, __u8)
    let lsb_first_value: u8 = if lsb_first { 1 } else { 0 };
    spidev_ioc_write(fd, SPI_IOC_NR_LSB_FIRST, &lsb_first_value)
}

pub fn get_bits_per_word(fd: RawFd) -> io::Result<u8> {
    // #define SPI_IOC_RD_BITS_PER_WORD _IOR(SPI_IOC_MAGIC, 3, __u8)
    spidev_ioc_read::<u8>(fd, SPI_IOC_NR_BITS_PER_WORD)
}

pub fn set_bits_per_word(fd: RawFd, bits_per_word: u8) -> io::Result<()> {
    // #define SPI_IOC_WR_BITS_PER_WORD _IOW(SPI_IOC_MAGIC, 3, __u8)
    spidev_ioc_write(fd, SPI_IOC_NR_BITS_PER_WORD, &bits_per_word)
}

pub fn get_max_speed_hz(fd: RawFd) -> io::Result<u32> {
    // #define SPI_IOC_RD_MAX_SPEED_HZ _IOR(SPI_IOC_MAGIC, 4, __u32)
    spidev_ioc_read::<u32>(fd, SPI_IOC_NR_MAX_SPEED_HZ)
}

pub fn set_max_speed_hz(fd: RawFd, max_speed_hz: u32) -> io::Result<()> {
    // #define SPI_IOC_WR_MAX_SPEED_HZ _IOW(SPI_IOC_MAGIC, 4, __u32)
    spidev_ioc_write(fd, SPI_IOC_NR_MAX_SPEED_HZ, &max_speed_hz)
}

pub fn xfer(fd: RawFd, tx_buf: &[u8]) {}
