use std::fmt::{self, Write};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemCode {
    Revolution,
    WiiDisc,
    Gamecube,
    UtilityDisc,
    DemoDisc,
    PromotionalDisc,
    DiagnosticDisc,
    DiagnosticDisc2,
    WiiBackupDisc,
    WiiFitChanInstaller,
    Unknown(u8),
}

impl From<u8> for SystemCode {
    fn from(value: u8) -> Self {
        match value {
            b'R' => Self::Revolution,
            b'S' => Self::WiiDisc,
            b'G' => Self::Gamecube,
            b'U' => Self::UtilityDisc,
            b'D' => Self::DemoDisc,
            b'P' => Self::PromotionalDisc,
            b'0' => Self::DiagnosticDisc,
            b'1' => Self::DiagnosticDisc2,
            b'4' => Self::WiiBackupDisc,
            b'_' => Self::WiiFitChanInstaller,
            c => Self::Unknown(c),
        }
    }
}

impl From<SystemCode> for u8 {
    fn from(val: SystemCode) -> Self {
        match val {
            SystemCode::Revolution => b'R',
            SystemCode::WiiDisc => b'S',
            SystemCode::Gamecube => b'G',
            SystemCode::UtilityDisc => b'U',
            SystemCode::DemoDisc => b'D',
            SystemCode::PromotionalDisc => b'P',
            SystemCode::DiagnosticDisc => b'0',
            SystemCode::DiagnosticDisc2 => b'1',
            SystemCode::WiiBackupDisc => b'4',
            SystemCode::WiiFitChanInstaller => b'_',
            SystemCode::Unknown(c) => c,
        }
    }
}

impl fmt::Display for SystemCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded: u8 = (*self).into();
        f.write_char(encoded as char)
    }
}
