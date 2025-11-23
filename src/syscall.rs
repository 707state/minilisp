use crate::common::RegisterX;

#[cfg(target_os = "linux")]
pub const SYSCALL_REGISTER: RegisterX = RegisterX::X8;
#[cfg(target_os = "macos")]
pub const SYSCALL_REGISTER: RegisterX = RegisterX::X16;

#[repr(C)]
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Syscall {
    EXIT,
    WRITE,
}

#[cfg(target_os = "linux")]

const SYSCALL_TABLE: &[(Syscall, u16)] = &[(Syscall::WRITE, 64), (Syscall::EXIT, 93)];

#[cfg(target_os = "macos")]
const SYSCALL_TABLE: &[(Syscall, u16)] = &[(Syscall::WRITE, 4), (Syscall::EXIT, 1)];
impl Syscall {
    pub fn number(&self) -> u16 {
        #[cfg(target_os = "linux")]
        {
            SYSCALL_TABLE.iter().find(|(s, _)| s == self).unwrap().1
        }
        #[cfg(target_os = "macos")]
        {
            SYSCALL_TABLE.iter().find(|(s, _)| s == self).unwrap().1
        }
    }
}
