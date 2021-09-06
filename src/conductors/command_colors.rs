pub(crate) const USER_CREATE: (u8, u8, u8) = (0xd5, 0xc4, 0xa1);
pub(crate) const USER_READ: (u8, u8, u8) = (0x83, 0xa5, 0x98);
pub(crate) const USER_UPDATE: (u8, u8, u8) = (0xb8, 0xb2, 0x26);
// pub(crate) const USER_DELETE: (u8, u8, u8) = (0x1d, 0x20, 0x21); <- disabled

pub(crate) const CONTENT_READ: (u8, u8, u8) = (0xfa, 0xdb, 0x2f);
pub(crate) const CONTENT_UPDATE: (u8, u8, u8) = (0x8e, 0xc0, 0x7c);
pub(crate) const CONTENT_DELETE: (u8, u8, u8) = (0x66, 0x5c, 0x54);

pub(crate) const POST: (u8, u8, u8) = (0xfb, 0xf1, 0xc7);
pub(crate) const LIKE: (u8, u8, u8) = (0xd3, 0x86, 0x9b);
pub(crate) const PIN: (u8, u8, u8) = (0xfb, 0x49, 0x34);
pub(crate) const BOOKMARK: (u8, u8, u8) = (0x83, 0xa5, 0x98);

pub(crate) const ERROR: (u8, u8, u8) = (0xfe, 0x80, 0x19);
