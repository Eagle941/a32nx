use std::u64;

#[derive(Clone, Copy)]
pub struct Arinc429WordBetter {
    label: u8,
    sdi: u8,   // Real width is 2 bits
    data: u32, // Real width is 19 bits
    ssm: SignStatus,
    p: Parity,
}
impl Arinc429WordBetter {
    #[deprecated(note = "Prefer using `new_with_label` method.")]
    pub fn new<T: Into<u32>>(value: T, ssm: SignStatus) -> Self {
        let data_ext = value.into();
        let data = data_ext & ((0b1 << 19) - 1);
        debug_assert_eq!(data_ext, data); // `data` must fit in 19bits

        let mut word = Self {
            label: 0,
            sdi: 0,
            data,
            ssm,
            p: 0.into(),
        };
        word.p = Self::calculate_parity_bit(word.into());
        word
    }

    pub fn new_with_label<T: Into<u32>>(value: T, ssm: SignStatus, sdi: u8, label: u8) -> Self {
        let sdi_sat = sdi & ((0b1 << 2) - 1);
        debug_assert_eq!(sdi, sdi_sat); // `sdi` must fit in 2bits
        let sdi = sdi_sat;

        let data_ext = value.into();
        let data = data_ext & ((0b1 << 19) - 1);
        debug_assert_eq!(data_ext, data); // `data` must fit in 19bits

        let mut word = Self {
            label,
            sdi,
            data,
            ssm,
            p: 0.into(),
        };
        word.p = Self::calculate_parity_bit(word.into());
        word
    }

    fn calculate_parity_bit(value: u32) -> Parity {
        // Taken from https://graphics.stanford.edu/~seander/bithacks.html#ParityParallel
        let mut v = value;
        v ^= v >> 16;
        v ^= v >> 8;
        v ^= v >> 4;
        v &= 0xf;
        let parity: u32 = (0x6996 >> v) & 1;
        let parity: u32 = (parity + 1) % 2;
        return parity.into();
    }

    /// Returns `Some` value when the SSM indicates normal operation, `None` otherwise.
    pub fn normal_value<T: From<u32>>(&self) -> Option<T> {
        if self.is_normal_operation() && self.is_correct_parity() {
            Some(self.data.into())
        } else {
            None
        }
    }

    pub fn ssm(&self) -> SignStatus {
        self.ssm
    }

    pub fn sdi(&self) -> u8 {
        self.sdi
    }

    pub fn parity(&self) -> Parity {
        self.p
    }

    pub fn label(&self) -> u8 {
        self.label
    }

    pub fn is_failure_warning(&self) -> bool {
        self.ssm == SignStatus::FailureWarning
    }

    pub fn is_no_computed_data(&self) -> bool {
        self.ssm == SignStatus::NoComputedData
    }

    pub fn is_functional_test(&self) -> bool {
        self.ssm == SignStatus::FunctionalTest
    }

    pub fn is_normal_operation(&self) -> bool {
        self.ssm == SignStatus::NormalOperation
    }

    pub fn is_correct_parity(&self) -> bool {
        self.p == Self::calculate_parity_bit(Into::<u32>::into(*self))
    }

    pub fn set_bit(&mut self, bit: u8, value: bool) {
        debug_assert!((11..=29).contains(&bit));
        self.data = ((self.data) & !(1 << (bit - 1))) | ((value as u32) << (bit - 1));
        self.p = 0.into();
        self.p = Self::calculate_parity_bit(Into::<u32>::into(*self));
    }

    pub fn get_bit(&self, bit: u8) -> bool {
        debug_assert!((11..=29).contains(&bit));
        ((self.data >> (bit - 1)) & 1) != 0
    }
}

// All LVARs are 64bit. A429 signals need to be converted to f64 when written or read from
// the variables registry.
impl From<f64> for Arinc429WordBetter {
    fn from(simvar: f64) -> Arinc429WordBetter {
        let int_value: u64 = simvar as u64;
        let label: u64 = int_value & ((0b1 << 8) - 1);
        let sdi: u64 = (int_value >> 8) & ((0b1 << 2) - 1);
        let data: u64 = (int_value >> 10) & ((0b1 << 19) - 1);
        let ssm: u64 = (int_value >> 29) & ((0b1 << 2) - 1);
        let parity: u64 = (int_value >> 31) & ((0b1 << 1) - 1);

        // Creating the struct without the constructor to manually set the parity bit.
        let word = Arinc429WordBetter {
            label: label as u8,
            sdi: sdi as u8,
            data: data as u32,
            ssm: (ssm as u32).into(),
            p: (parity as u32).into(),
        };

        word
    }
}
impl From<Arinc429WordBetter> for u32 {
    fn from(value: Arinc429WordBetter) -> u32 {
        let label: u32 = value.label.into();
        let ssm: u32 = value.ssm.into();
        let sdi: u32 = value.sdi.into();
        let data: u32 = value.data.into();
        let parity: u32 = value.p.into();
        let int_value: u32 = label | (sdi << 8) | (data << 10) | (ssm << 29) | (parity << 31);
        int_value
    }
}
impl From<Arinc429WordBetter> for f64 {
    fn from(value: Arinc429WordBetter) -> f64 {
        let int_value: u32 = value.into();

        int_value as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignStatus {
    FailureWarning,
    NoComputedData,
    FunctionalTest,
    NormalOperation,
}

impl From<SignStatus> for u32 {
    fn from(value: SignStatus) -> Self {
        match value {
            SignStatus::FailureWarning => 0b00,
            SignStatus::NoComputedData => 0b01,
            SignStatus::FunctionalTest => 0b10,
            SignStatus::NormalOperation => 0b11,
        }
    }
}

impl From<u32> for SignStatus {
    fn from(value: u32) -> Self {
        match value {
            0b00 => SignStatus::FailureWarning,
            0b01 => SignStatus::NoComputedData,
            0b10 => SignStatus::FunctionalTest,
            0b11 => SignStatus::NormalOperation,
            _ => panic!("Unknown SSM value: {}.", value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    Odd,
    Even,
}

impl From<Parity> for u32 {
    fn from(value: Parity) -> Self {
        match value {
            Parity::Odd => 0b00,
            Parity::Even => 0b01,
        }
    }
}

impl From<u32> for Parity {
    fn from(value: u32) -> Self {
        match value {
            0b00 => Parity::Odd,
            0b01 => Parity::Even,
            _ => panic!("Unknown Parity value: {}.", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parity_count_ones(data: u32) -> Parity {
        let number_ones = data.count_ones() + 1;
        (number_ones % 2).into()
    }

    #[test]
    fn test_simple_word() {
        // let label = 0b10110000;
        // let sdi = 0b00;
        // let data = 0b1000110001100010001;
        // let ssm = 0b00;
        // let p = 0b1;
        let value = 0b1000110001100010001;
        let ssm = SignStatus::FailureWarning;
        let sdi = 0b00;
        let label = 0b10110000;
        let word = Arinc429WordBetter::new_with_label(value, ssm, sdi, label);
        let expected_parity = word.p;

        let lvar = f64::from(word);
        // 0x41E2318896000000 2441888944
        println!(
            "Arinc429WordBetter lvar arinc 0x{:X} {lvar}",
            lvar.to_bits(),
        );

        let result: Arinc429WordBetter = Arinc429WordBetter::from(lvar);

        assert_eq!(result.data, value);
        assert_eq!(result.ssm, ssm);
        assert_eq!(result.sdi, sdi);
        assert_eq!(result.label, label);
        assert_eq!(result.p, expected_parity);
    }

    #[test]
    fn test_calculate_parity_bit() {
        let data = 0b10001111111000100010010110000;
        assert_eq!(
            Arinc429WordBetter::calculate_parity_bit(data),
            parity_count_ones(data)
        );

        let data = 0b0;
        assert_eq!(
            Arinc429WordBetter::calculate_parity_bit(data),
            parity_count_ones(data)
        );

        let data = 0b1;
        assert_eq!(
            Arinc429WordBetter::calculate_parity_bit(data),
            parity_count_ones(data)
        );

        let data = 0b01111111111111111111111111111111;
        assert_eq!(
            Arinc429WordBetter::calculate_parity_bit(data),
            parity_count_ones(data)
        );

        let data = 0b01111111111111111111111111111110;
        assert_eq!(
            Arinc429WordBetter::calculate_parity_bit(data),
            parity_count_ones(data)
        );
    }

    #[ignore = "This test proves that no data is lost casting from u32 to f64"]
    #[test]
    fn test_casting() {
        for i in 0..u32::MAX {
            let i_f64 = i as f64;
            let i_u32 = i_f64 as u32;

            assert_eq!(i, i_u32);
        }
    }
}
