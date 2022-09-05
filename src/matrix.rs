//! Hardware pin switch matrix handling.

use embedded_hal::digital::v2::{InputPin, OutputPin};
use cortex_m::asm::delay;

/// Describes the hardware-level matrix of switches.
///
/// Generic parameters are in order: The type of column pins,
/// the type of row pins, the number of columns and rows.
/// **NOTE:** In order to be able to put different pin structs
/// in an array they have to be downgraded (stripped of their
/// numbers etc.). Most HAL-s have a method of downgrading pins
/// to a common (erased) struct. (for example see
/// [stm32f0xx_hal::gpio::PA0::downgrade](https://docs.rs/stm32f0xx-hal/0.17.1/stm32f0xx_hal/gpio/gpioa/struct.PA0.html#method.downgrade))
pub struct Matrix<C, R, const CS: usize, const RS: usize>
where
    C: InputPin,
    R: OutputPin,
{
    cols: [C; CS],
    rows: [R; RS],
}

impl<C, R, const CS: usize, const RS: usize> Matrix<C, R, CS, RS>
where
    C: InputPin,
    R: OutputPin,
{
    /// Creates a new Matrix.
    ///
    /// Assumes columns are pull-up inputs,
    /// and rows are output pins which are set high when not being scanned.
    pub fn new<E>(cols: [C; CS], rows: [R; RS]) -> Result<Self, E>
    where
        C: InputPin<Error = E>,
        R: OutputPin<Error = E>,
    {
        let mut res = Self { cols, rows };
        res.clear()?;
        Ok(res)
    }
    fn clear<E>(&mut self) -> Result<(), E>
    where
        C: InputPin<Error = E>,
        R: OutputPin<Error = E>,
    {
        for r in self.rows.iter_mut() {
            r.set_high()?;
        }
        Ok(())
    }


    fn delay_us(us: u32) {
        //self.timer.count_down().start(2000000_u32.microseconds());
        //self.timer.count_down().wait();
        let ticksPerSecond = 12_000_000u32;
    
        let ticksPerMicroSecond = ticksPerSecond/1000000;
    
        let iterations = us * 1;
    
        // Iterate rather than multiply to prevent buffer overflow
        for _ in 0..iterations{
           delay(ticksPerMicroSecond);
        }
    }
    /// Scans the matrix and checks which keys are pressed.
    ///
    /// Every row pin in order is pulled low, and then each column
    /// pin is tested; if it's low, the key is marked as pressed.
    pub fn get<E>(&mut self) -> Result<[[bool; CS]; RS], E>
    where
        C: InputPin<Error = E>,
        R: OutputPin<Error = E>,
    {
        let mut keys = [[false; CS]; RS];

        for (ri, row) in (&mut self.rows).iter_mut().enumerate() {
            row.set_low()?;
            // Hacked in delay to prevent multiple keys from being read
            // Real fix is to implement code outlined in this issue:
            // https://github.com/TeXitoi/keyberon/issues/97
            Self::delay_us(10);
            for (ci, col) in (&self.cols).iter().enumerate() {
                if col.is_low()? {
                    keys[ri][ci] = true;
                }
            }
            row.set_high()?;
        }
        Ok(keys)
        // let mut keys = [[false; CS]; RS];

        // for (ci, col) in (&mut self.cols).iter_mut().enumerate() {
        //     col.set_high()?;
        //     Self::delay_us(50);
        //     for (ri, row) in (&self.rows).iter().enumerate() {
        //         if row.is_high()? {
        //             keys[ri][ci] = true;
        //         }
        //     }
        //     Self::delay_us(50);
        //     col.set_low()?;
        // }
        // Ok(keys)
    }
}

/// Matrix-representation of switches directly attached to the pins ("diodeless").
///
/// Generic parameters are in order: The type of column pins,
/// the number of columns and rows.
pub struct DirectPinMatrix<P, const CS: usize, const RS: usize>
where
    P: InputPin,
{
    pins: [[Option<P>; CS]; RS],
}

impl<P, const CS: usize, const RS: usize> DirectPinMatrix<P, CS, RS>
where
    P: InputPin,
{
    /// Creates a new DirectPinMatrix.
    ///
    /// Assumes pins are pull-up inputs. Spots in the matrix that are
    /// not corresponding to any pins use ´None´.
    pub fn new<E>(pins: [[Option<P>; CS]; RS]) -> Result<Self, E>
    where
        P: InputPin<Error = E>,
    {
        let res = Self { pins };
        Ok(res)
    }

    /// Scans the pins and checks which keys are pressed (state is "low").
    pub fn get<E>(&mut self) -> Result<[[bool; CS]; RS], E>
    where
        P: InputPin<Error = E>,
    {
        let mut keys = [[false; CS]; RS];

        for (ri, row) in (&mut self.pins).iter_mut().enumerate() {
            for (ci, col_option) in row.iter().enumerate() {
                if let Some(col) = col_option {
                    if col.is_low()? {
                        keys[ri][ci] = true;
                    }
                }
            }
        }
        Ok(keys)
    }
}
