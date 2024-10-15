use device_driver::{implement_device, AddressableDevice, Register, RegisterDevice};
use embedded_hal::i2c::{Error, ErrorKind, I2c};
use registers::PsSt;

const VCNL36825T_ADDRESS: u8 = 0x60;
const VCNL36825T_ID: u16 = 0x0026;

#[derive(Debug)]
pub enum PSError {
    InvalidID,
    I2CError(ErrorKind),
}
pub struct VCNL36825T<I2C> {
    i2c: I2C,
}

impl<I2C> AddressableDevice for VCNL36825T<I2C> {
    type AddressType = u8;
}

impl<I2C: I2c> RegisterDevice for VCNL36825T<I2C> {
    //type Error = I2C::Error;
    type Error = PSError;

    fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &device_driver::bitvec::prelude::BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
            let mut buffer = [0u8; 3];
            buffer[0] = address;
            buffer[1..].copy_from_slice(&data.as_raw_slice()[..SIZE_BYTES]);
            self.i2c.write(VCNL36825T_ADDRESS, &buffer).map_err(|e| PSError::I2CError(e.kind()))
    }

    fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut device_driver::bitvec::prelude::BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let mut buffer = [0u8; 2];
        self.i2c.write_read(VCNL36825T_ADDRESS, &[address], &mut buffer).map_err(|e| PSError::I2CError(e.kind()))?;
        data.as_raw_mut_slice()[..SIZE_BYTES].copy_from_slice(&buffer[..SIZE_BYTES]);
        Ok(())
    }
}

impl<I2C: I2c + Default> Default for VCNL36825T<I2C> {
    fn default() -> Self {
        Self::new(I2C::default()).unwrap()
    }
}

impl<I2C: I2c> VCNL36825T<I2C>
{
    pub fn new(i2c: I2C) -> Result<Self, PSError> {
        let mut vcnl36825t = VCNL36825T {
            i2c
        };
        if vcnl36825t.id().read().unwrap().device_id() != VCNL36825T_ID {
            return Err(PSError::InvalidID);
        }
        vcnl36825t.ps_thdl().clear()?;
        Ok(vcnl36825t)
    }

    pub fn destroy(self) -> I2C {
        self.i2c
    }

    pub fn power_on(&mut self) -> Result<(), PSError> {
        self.ps_conf_1().write(|w| w.res_1(1).res_2(1))?;
        self.ps_conf_2().write(|w| w.ps_st(PsSt::Stop))?;
        self.ps_conf_1().write(|w| w.ps_on(true).ps_cal(true).res_1(1).res_2(1))?;
        self.ps_conf_2().write(|w| w.ps_st(PsSt::Start))
    }
}

pub mod registers {
    use super::*;
    implement_device!(
        impl<I2C> VCNL36825T<I2C> {
            register PS_CONF1 {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x00;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x01, 0x00];
                res1: u8 = 0..1,
                ps_on: bool = 1,
                ps_cal: bool = 7,
                res2: u8 = 9..10,

            },
            register PS_CONF2 {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x03;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x01, 0x00];

                ps_st: u8 as enum PsSt {
                    Start = 0,
                    Stop = 1,
                } = 0..1,
                ps_smart_pers: bool = 1,
                ps_int: u8 as enum PsInt {
                    IntOff = 0,
                    IntOn = 1,
                } = 2..4,
                ps_pers: u8 as enum PsPers {
                    Pers1 = 0,
                    Pers2 = 1,
                    Pers3 = 2,
                    Pers4 = 3,
                } = 4..6,
                ps_period: u8 as enum PsPeriod {
                    Period10ms = 0,
                    Period20ms = 1,
                    Period40ms = 2,
                    Period80ms = 3,
                } = 6..8,
                ps_hg: bool = 10,
                ps_itb: u8 as enum PsItb {
                    Itb25us = 0,
                    Itb50us = 1,
                } = 11..12,
                ps_mps: u8 as enum PsMps {
                    Mps1 = 0,
                    Mps2 = 1,
                    Mps4 = 2,
                    Mps8 = 3,
                } = 12..14,
                ps_it: u8 as enum PsIt {
                    It1T = 0,
                    It2T = 1,
                    It4T = 2,
                    It8T = 3,
                } = 14..16,
            },
            register PS_CONF3 {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x04;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];

                ps_sp_int: bool = 2,
                ps_forcenum: u8 as enum PsForcenum {
                    OneCycle = 0,
                    TwoCycle = 1,
                } = 4..5,
                ps_trig: u8 as enum PsTrig {
                    NoPSActive = 0,
                    OneTimeCycle = 1,
                } = 5..6,
                ps_af: u8 as enum PsAF {
                    AutoMode = 0,
                    ForceMode = 1,
                } = 6..7,
                i_vcsel: u8 as enum IVcsel {
                    I10mA = 2,
                    I12mA = 3,
                    I14mA = 4,
                    I16mA = 5,
                    I18mA = 6,
                    I20mA = 7,
                } = 8..12,
                ps_hd: u8 as enum PsHd {
                    HD12Bits = 0,
                    HD16Bits = 1,
                } = 12..13,
                ps_sc: u8 as enum PsSc {
                    SunlightCancellationDisable = 0,
                    SunlightCancellationEnable = 7,
                } = 13..16,
            },
            register PS_THDL {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x05;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];
                value: u16 = 0..12,
            },
            register PS_THDH {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x06;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];
                value: u16 = 0..12,
            },
            register PS_CANC {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x07;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];
                value: u16 = 0..12,
            },
            register PS_CONF4 {
                type RWType = ReadWrite;
                const ADDRESS: u8 = 0x08;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];

                ps_ac_int: bool = 0,
                ps_ac_trig: bool = 2,
                ps_ac: bool = 3,
                ps_ac_num: u8 as enum PsAcNum {
                    Num1 = 0,
                    Num2 = 1,
                    Num4 = 2,
                    Num8 = 3,
                } = 4..6,
                ps_ac_period: u8 as enum PsAcPeriod {
                    Period3ms = 0,
                    Period6ms = 1,
                    Period12ms = 2,
                    Period24ms = 3,
                } = 6..8,
                ps_lpen: bool = 8,
                ps_lpper: u8 as enum PsLpPeriod {
                    Period40ms = 0,
                    Period80ms = 1,
                    Period160ms = 2,
                    Period320ms = 3,
                } = 9..11,
            },
            register PS_DATA {
                type RWType = ReadOnly;
                const ADDRESS: u8 = 0xF8;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];
                value: u16 = 0..12,
            },
            register INT_FLAG {
                type RWType = ReadOnly;
                const ADDRESS: u8 = 0xF9;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];
                ps_if_away: bool = 8,
                ps_if_close: bool = 9,
                ps_spflag: bool = 12,
                ps_acflag: bool = 12,
            },
            register ID {
                type RWType = ReadOnly;
                const ADDRESS: u8 = 0xFA;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x26, 0x00];
                device_id: u16 = 0..12,
            },
            register PS_AC_DATA {
                type RWType = ReadOnly;
                const ADDRESS: u8 = 0xFB;
                const SIZE_BITS: usize = 16;
                const RESET_VALUE: [u8] = [0x00, 0x00];
                value: u16 = 0..12,
                ac_sun: bool = 14,
                ac_busy: bool = 15,
            },
        }
    );
}
