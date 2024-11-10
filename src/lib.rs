#![no_std]

use device_driver::{AsyncRegisterInterface, RegisterInterface};
use embedded_hal::i2c::{Error, ErrorKind, I2c, Operation};
use embedded_hal_async::i2c::I2c as I2cAsync;
use registers::{PsSt, Registers};

const VCNL36825T_ADDRESS: u8 = 0x60;
const VCNL36825T_ID: u16 = 0x0026;

#[derive(Debug)]
pub enum PSError {
    InvalidID,
    I2CError(ErrorKind),
}
pub struct Interface<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> RegisterInterface for Interface<I2C> {
    type Error = PSError;
    type AddressType = u8;

    fn write_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .transaction(
                VCNL36825T_ADDRESS,
                &mut [Operation::Write(&[address]), Operation::Write(data)],
            )
            .map_err(|e| PSError::I2CError(e.kind()))
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .write_read(VCNL36825T_ADDRESS, &[address], data)
            .map_err(|e| PSError::I2CError(e.kind()))
    }
}

impl<I2C: I2cAsync> AsyncRegisterInterface for Interface<I2C> {
    type Error = PSError;
    type AddressType = u8;

    async fn write_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .transaction(
                VCNL36825T_ADDRESS,
                &mut [Operation::Write(&[address]), Operation::Write(data)],
            )
            .await
            .map_err(|e| PSError::I2CError(e.kind()))
    }

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .write_read(VCNL36825T_ADDRESS, &[address], data)
            .await
            .map_err(|e| PSError::I2CError(e.kind()))
    }
}

pub struct VCNL36825T<I2C> {
    ll: registers::Registers<Interface<I2C>>,
}

impl<I2C: I2c + Default> Default for VCNL36825T<I2C> {
    fn default() -> Self {
        Self::new(I2C::default()).unwrap()
    }
}

impl<I2C: I2c> VCNL36825T<I2C> {
    pub fn new(i2c: I2C) -> Result<Self, PSError> {
        let mut vcnl36825t = VCNL36825T {
            ll: registers::Registers::new(Interface { i2c }),
        };
        if vcnl36825t.ll.id().read().unwrap().device_id() != VCNL36825T_ID {
            return Err(PSError::InvalidID);
        }
        vcnl36825t.ll.ps_thdl().write(|_| {})?;
        Ok(vcnl36825t)
    }

    pub fn power_on(&mut self) -> Result<(), PSError> {
        self.ll.ps_conf_1().write(|reg| {
            reg.set_res_1(1);
            reg.set_res_2(1)
        })?;
        self.ll.ps_conf_2().write(|reg| reg.set_ps_st(PsSt::Stop))?;
        self.ll.ps_conf_1().write(|reg| {
            reg.set_ps_on(true);
            reg.set_ps_cal(true);
            reg.set_res_1(1);
            reg.set_res_2(1)
        })?;
        self.ll.ps_conf_2().write(|reg| reg.set_ps_st(PsSt::Start))
    }
}

impl<I2C: I2cAsync> VCNL36825T<I2C> {
    pub async fn new_async(i2c: I2C) -> Result<Self, PSError> {
        let mut vcnl36825t = VCNL36825T {
            ll: registers::Registers::new(Interface { i2c }),
        };
        if vcnl36825t.ll.id().read_async().await.unwrap().device_id() != VCNL36825T_ID {
            return Err(PSError::InvalidID);
        }
        vcnl36825t.ll.ps_thdl().write_async(|_| {}).await?;
        Ok(vcnl36825t)
    }

    pub async fn power_on_async(&mut self) -> Result<(), PSError> {
        self.ll
            .ps_conf_1()
            .write_async(|reg| {
                reg.set_res_1(1);
                reg.set_res_2(1)
            })
            .await?;
        self.ll
            .ps_conf_2()
            .write_async(|reg| reg.set_ps_st(PsSt::Stop))
            .await?;
        self.ll
            .ps_conf_1()
            .write_async(|reg| {
                reg.set_ps_on(true);
                reg.set_ps_cal(true);
                reg.set_res_1(1);
                reg.set_res_2(1)
            })
            .await?;
        self.ll
            .ps_conf_2()
            .write_async(|reg| reg.set_ps_st(PsSt::Start))
            .await
    }
}

impl<I2C> VCNL36825T<I2C> {
    pub fn destroy(self) -> I2C {
        self.ll.interface.i2c
    }

    pub fn registers(&mut self) -> &mut Registers<Interface<I2C>> {
        &mut self.ll
    }
}

pub mod registers {
    device_driver::create_device!(
        device_name: Registers,
        dsl: {
            config {
                type DefaultByteOrder = LE;
                type RegisterAddressType = u8;
            }
            register PS_CONF1 {
                type Access = ReadWrite;
                const ADDRESS = 0x00;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x01, 0x00];

                res1: uint = 0..1,
                ps_on: bool = 1,
                ps_cal: bool = 7,
                res2: uint = 9..10,

            },
            register PS_CONF2 {
                type Access = ReadWrite;
                const ADDRESS = 0x03;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x01, 0x00];

                ps_st: uint as enum PsSt {
                    Start = 0,
                    Stop = 1,
                } = 0..1,
                ps_smart_pers: bool = 1,
                ps_int: uint as try enum PsInt {
                    IntOff = 0,
                    IntOn = 1,
                } = 2..4,
                ps_pers: uint as enum PsPers {
                    Pers1 = 0,
                    Pers2 = 1,
                    Pers3 = 2,
                    Pers4 = 3,
                } = 4..6,
                ps_period: uint as enum PsPeriod {
                    Period10ms = 0,
                    Period20ms = 1,
                    Period40ms = 2,
                    Period80ms = 3,
                } = 6..8,
                ps_hg: bool = 10,
                ps_itb: uint as enum PsItb {
                    Itb25us = 0,
                    Itb50us = 1,
                } = 11..12,
                ps_mps: uint as enum PsMps {
                    Mps1 = 0,
                    Mps2 = 1,
                    Mps4 = 2,
                    Mps8 = 3,
                } = 12..14,
                ps_it: uint as enum PsIt {
                    It1T = 0,
                    It2T = 1,
                    It4T = 2,
                    It8T = 3,
                } = 14..16,
            },
            register PS_CONF3 {
                type Access = ReadWrite;
                const ADDRESS = 0x04;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                ps_sp_int: bool = 2,
                res3: uint = 3..4,
                ps_forcenum: uint as enum PsForcenum {
                    OneCycle = 0,
                    TwoCycle = 1,
                } = 4..5,
                ps_trig: uint as enum PsTrig {
                    NoPSActive = 0,
                    OneTimeCycle = 1,
                } = 5..6,
                ps_af: uint as enum PsAF {
                    AutoMode = 0,
                    ForceMode = 1,
                } = 6..7,
                i_vcsel: uint as try enum IVcsel {
                    I10mA = 2,
                    I12mA = 3,
                    I14mA = 4,
                    I16mA = 5,
                    I18mA = 6,
                    I20mA = 7,
                } = 8..12,
                ps_hd: uint as enum PsHd {
                    HD12Bits = 0,
                    HD16Bits = 1,
                } = 12..13,
                ps_sc: uint as try enum PsSc {
                    SunlightCancellationDisable = 0,
                    SunlightCancellationEnable = 7,
                } = 13..16,
            },
            register PS_THDL {
                type Access = ReadWrite;
                const ADDRESS = 0x05;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                value: uint = 0..12,
            },
            register PS_THDH {
                type Access = ReadWrite;
                const ADDRESS = 0x06;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                value: uint = 0..12,
            },
            register PS_CANC {
                type Access = ReadWrite;
                const ADDRESS = 0x07;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                value: uint = 0..12,
            },
            register PS_CONF4 {
                type Access = ReadWrite;
                const ADDRESS = 0x08;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                ps_ac_int: bool = 0,
                ps_ac_trig: bool = 2,
                ps_ac: bool = 3,
                ps_ac_num: uint as enum PsAcNum {
                    Num1 = 0,
                    Num2 = 1,
                    Num4 = 2,
                    Num8 = 3,
                } = 4..6,
                ps_ac_period: uint as enum PsAcPeriod {
                    Period3ms = 0,
                    Period6ms = 1,
                    Period12ms = 2,
                    Period24ms = 3,
                } = 6..8,
                ps_lpen: bool = 8,
                ps_lpper: uint as enum PsLpPeriod {
                    Period40ms = 0,
                    Period80ms = 1,
                    Period160ms = 2,
                    Period320ms = 3,
                } = 9..11,
            },
            register PS_DATA {
                type Access = ReadOnly;
                const ADDRESS = 0xF8;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                value: uint = 0..12,
            },
            register INT_FLAG {
                type Access = ReadOnly;
                const ADDRESS = 0xF9;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                ps_if_away: bool = 8,
                ps_if_close: bool = 9,
                ps_spflag: bool = 12,
                ps_acflag: bool = 13,
            },
            register ID {
                type Access = ReadOnly;
                const ADDRESS = 0xFA;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x26, 0x00];

                device_id: uint = 0..12,
            },
            register PS_AC_DATA {
                type Access = ReadOnly;
                const ADDRESS = 0xFB;
                const SIZE_BITS = 16;
                const RESET_VALUE = [0x00, 0x00];

                value: uint = 0..12,
                ac_sun: bool = 14,
                ac_busy: bool = 15,
            },
        }
    );
}
