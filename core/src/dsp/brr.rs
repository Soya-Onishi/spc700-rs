#[derive(Clone)]
pub struct BRRInfo {
    pub shift_amount: u8,
    pub filter: FilterType,
    pub end: BRREnd,
}

#[derive(Copy, Clone)]
pub enum FilterType {
    NoFilter,
    UseOld,
    UseAll0,
    UseAll1,
}

#[derive(Copy, Clone, PartialEq)]
pub enum BRREnd {
    Normal,
    Mute,
    Loop,
}

impl BRRInfo {
    pub const fn new(format: u8) -> BRRInfo {
        let shift_amount = (format >> 4) & 0x0F;
        let filter = match (format >> 2) & 0b11 {
            0 => FilterType::NoFilter,
            1 => FilterType::UseOld,
            2 => FilterType::UseAll0,
            3 => FilterType::UseAll1,
            _ => panic!("filter value should be between 0 to 3"),
        };

        let end = match format & 0b11 {
            0 | 2 => BRREnd::Normal,
            1     => BRREnd::Mute,
            3     => BRREnd::Loop,
            _     => panic!("end range should be between 0 to 3"),
        };

        BRRInfo {
            shift_amount,
            filter,
            end,
        }
    }

    pub const fn empty() -> BRRInfo {
        BRRInfo::new(0)
    }
}