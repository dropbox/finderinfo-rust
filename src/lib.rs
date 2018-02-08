#![allow(non_snake_case, non_upper_case_globals)]

//! Structs and functions to manipulate MacOS/HFS+ structs, e.g. com.apple.FinderInfo.
//!
//! Note that HFS+ is big-endian, and so all serialization/deserialization has to be byteswapped
//! appropriately. APFS isn't big-endian, but it pretends pretty hard internally (and does so
//! here).

extern crate byteorder;

use std::io;
use std::fmt;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct OSType(pub [u8; 4]);

impl fmt::Debug for OSType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = String::from_utf8(self.0.into_iter().cloned().collect());
        write!(f, "{:?}", s)
    }
}

#[allow(dead_code)]
pub mod constants {
    use super::OSType;

    // Finder flag constants
    /// Unused and reserved in System 7; set to 0.
    pub const kIsOnDesk: u16 = 0x0001;
    /// Three bits of color coding.
    pub const kColor: u16 = 0x000e;
    /// The file is an application that can be executed by multiple users simultaneously. Defined
    /// only for applications; otherwise, set to 0.
    pub const kIsShared: u16 = 0x0040;
    /// The file contains no 'INIT' resources; set to 0. Reserved for directories; set to 0.
    pub const kHasNoINITs: u16 = 0x0080;
    /// The Finder has recorded information from the file's bundle resource into the desktop
    /// database and given the file or folder a position on the desktop.
    pub const kHasBeenInited: u16 = 0x0100;
    /// The file or directory contains a customized icon.
    pub const kHasCustomIcon: u16 = 0x0400;
    /// For a file, this bit indicates that the file is a stationery pad. For directories, this bit
    /// is reserved--in which case, set to 0.
    pub const kIsStationery: u16 = 0x0800;
    /// The file or directory can't be renamed from the Finder, and the icon cannot be changed.
    pub const kNameLocked: u16 = 0x1000;
    /// For a file, this bit indicates that the file contains a bundle resource. For a directory,
    /// this bit indicates that the directory is a file package. Note that not all file packages
    /// have this bit set; many file packages are identified by other means, such as a recognized
    /// package extension in the name. The proper way to determine if an item is a package is
    /// through Launch Services.
    pub const kHasBundle: u16 = 0x2000;
    /// The file or directory is invisible from the Finder and from the Navigation Services dialogs.
    pub const kIsInvisible: u16 = 0x4000;
    /// For a file, this bit indicates that the file is an alias file. For directories, this bit is
    /// reserved--in which case, set to 0.
    pub const kIsAlias: u16 = 0x8000;

    // Extended finder flag constants
    /// If set the other extended flags are ignored.
    pub const kExtendedFlagsAreInvalid: u16 = 0x8000;
    /// Set if the file or folder has a badge resource.
    pub const kExtendedFlagHasCustomBadge: u16 = 0x0100;
    /// Set if the file contains routing info resource.
    pub const kExtendedFlagHasRoutingInfo: u16 = 0x0004;

    // File type constants
    /// File type for a symlink.
    pub const kSymLinkFileType: OSType = OSType([0x73, 0x6c, 0x6e, 0x6b]); /* 'slnk' */
    /// File type for the creator of a symlink.
    pub const kSymLinkCreator: OSType = OSType([0x72, 0x68, 0x61, 0x70]); /* 'rhap' */
}

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct Point {
    pub v: i16,
    pub h: i16,
}

impl Point {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<Point> {
        let v = r.read_i16::<BigEndian>()?;
        let h = r.read_i16::<BigEndian>()?;
        Ok(Point { v, h })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        w.write_i16::<BigEndian>(self.v)?;
        w.write_i16::<BigEndian>(self.h)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct Rect {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16,
}

impl Rect {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<Rect> {
        let top = r.read_i16::<BigEndian>()?;
        let left = r.read_i16::<BigEndian>()?;
        let bottom = r.read_i16::<BigEndian>()?;
        let right = r.read_i16::<BigEndian>()?;
        Ok(Rect {
            top,
            left,
            bottom,
            right,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        w.write_i16::<BigEndian>(self.top)?;
        w.write_i16::<BigEndian>(self.left)?;
        w.write_i16::<BigEndian>(self.bottom)?;
        w.write_i16::<BigEndian>(self.right)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct FinderFlags(u16);

impl FinderFlags {
    pub fn color(&self) -> Option<LabelColor> {
        LabelColor::from_u8((self.0 & constants::kColor) as u8)
    }

    pub fn set_color(&mut self, color: Option<LabelColor>) {
        self.0 &= !constants::kColor;
        self.0 |= u16::from(LabelColor::to_u8(color));
    }

    pub fn is_shared(&self) -> bool {
        self.0 & constants::kIsShared != 0
    }

    pub fn has_no_inits(&self) -> bool {
        self.0 & constants::kHasNoINITs != 0
    }

    pub fn has_been_inited(&self) -> bool {
        self.0 & constants::kHasBeenInited != 0
    }

    pub fn has_custom_icon(&self) -> bool {
        self.0 & constants::kHasCustomIcon != 0
    }

    pub fn set_has_custom_icon(&mut self, value: bool) {
        if value {
            self.0 |= constants::kHasCustomIcon;
        } else {
            self.0 &= !constants::kHasCustomIcon;
        }
    }

    pub fn is_stationery(&self) -> bool {
        self.0 & constants::kIsStationery != 0
    }

    pub fn name_locked(&self) -> bool {
        self.0 & constants::kNameLocked != 0
    }

    pub fn has_bundle(&self) -> bool {
        self.0 & constants::kHasBundle != 0
    }

    pub fn is_invisible(&self) -> bool {
        self.0 & constants::kIsInvisible != 0
    }

    pub fn is_alias(&self) -> bool {
        self.0 & constants::kIsAlias != 0
    }
}

impl fmt::Debug for FinderFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut flags = vec![];
        if let Some(color) = self.color() {
            flags.push(format!("{:?}", color));
        }
        if self.is_shared() {
            flags.push("kIsShared".to_string());
        }
        if self.has_no_inits() {
            flags.push("kHasNoINITs".to_string());
        }
        if self.has_been_inited() {
            flags.push("kHasBeenInited".to_string());
        }
        if self.has_custom_icon() {
            flags.push("kHasCustomIcon".to_string());
        }
        if self.is_stationery() {
            flags.push("kIsStationery".to_string());
        }
        if self.name_locked() {
            flags.push("kNameLocked".to_string());
        }
        if self.has_bundle() {
            flags.push("kHasBundle".to_string());
        }
        if self.is_invisible() {
            flags.push("kIsInvisible".to_string());
        }
        if self.is_alias() {
            flags.push("kIsAlias".to_string());
        }
        f.debug_struct("FinderFlags")
            .field("raw", &self.0)
            .field("flags", &flags)
            .finish()
    }
}

impl From<u16> for FinderFlags {
    fn from(s: u16) -> FinderFlags {
        FinderFlags(s)
    }
}
impl From<FinderFlags> for u16 {
    fn from(f: FinderFlags) -> u16 {
        f.0
    }
}

// TODO(robert): In MacOS 10.10 and above, the `LabelColor` is no longer stored in the
// `com.apple.FinderInfo` attribute but is instead stored in a `bplist` format. The last tag-string
// in the `bplist` which corresponds to a color is the one which we should set in the
// `com.apple.FinderInfo` attribute. We should synchronize these on write/read to be
// cross-compatible with MacOS 10.9.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LabelColor {
    Gray,
    Green,
    Purple,
    Blue,
    Yellow,
    Red,
    Orange,
}

impl LabelColor {
    pub fn from_u8(b: u8) -> Option<LabelColor> {
        match b {
            0x02 => Some(LabelColor::Gray),
            0x04 => Some(LabelColor::Green),
            0x06 => Some(LabelColor::Purple),
            0x08 => Some(LabelColor::Blue),
            0x0a => Some(LabelColor::Yellow),
            0x0c => Some(LabelColor::Red),
            0x0e => Some(LabelColor::Orange),
            _ => None,
        }
    }

    pub fn to_u8(c: Option<LabelColor>) -> u8 {
        match c {
            None => 0x00,
            Some(LabelColor::Gray) => 0x02,
            Some(LabelColor::Green) => 0x04,
            Some(LabelColor::Purple) => 0x06,
            Some(LabelColor::Blue) => 0x08,
            Some(LabelColor::Yellow) => 0x0a,
            Some(LabelColor::Red) => 0x0c,
            Some(LabelColor::Orange) => 0x0e,
        }
    }

    pub fn to_str(c: LabelColor) -> &'static str {
        match c {
            LabelColor::Gray => "Gray",
            LabelColor::Green => "Green",
            LabelColor::Purple => "Purple",
            LabelColor::Blue => "Blue",
            LabelColor::Yellow => "Yellow",
            LabelColor::Red => "Red",
            LabelColor::Orange => "Orange",
        }
    }

    pub fn from_str(s: &str) -> Option<LabelColor> {
        match s {
            "Gray" => Some(LabelColor::Gray),
            "Green" => Some(LabelColor::Green),
            "Purple" => Some(LabelColor::Purple),
            "Blue" => Some(LabelColor::Blue),
            "Yellow" => Some(LabelColor::Yellow),
            "Red" => Some(LabelColor::Red),
            "Orange" => Some(LabelColor::Orange),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct ExtendedFinderFlags(u16);

impl fmt::Debug for ExtendedFinderFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut flags = vec![];
        if self.are_invalid() {
            flags.push("kExtendedFlagsAreInvalid");
        }
        if self.has_custom_badge() {
            flags.push("kExtendedFlagHasCustomBadge");
        }
        if self.has_routing_info() {
            flags.push("kExtendedFlagHasCustomBadge");
        }
        f.debug_struct("ExtendedFinderFlags")
            .field("raw", &self.0)
            .field("flags", &flags)
            .finish()
    }
}

impl ExtendedFinderFlags {
    pub fn are_invalid(&self) -> bool {
        self.0 & constants::kExtendedFlagsAreInvalid != 0
    }

    pub fn has_custom_badge(&self) -> bool {
        self.0 & constants::kExtendedFlagHasCustomBadge != 0
    }

    pub fn has_routing_info(&self) -> bool {
        self.0 & constants::kExtendedFlagHasRoutingInfo != 0
    }
}

impl From<u16> for ExtendedFinderFlags {
    fn from(s: u16) -> ExtendedFinderFlags {
        ExtendedFinderFlags(s)
    }
}
impl From<ExtendedFinderFlags> for u16 {
    fn from(f: ExtendedFinderFlags) -> u16 {
        f.0
    }
}

/// Defines a file information structure.
///
/// The `FileInfo` structure is preferred over the FInfo structure.
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct FileInfo {
    /// File type.
    pub fileType: OSType,
    /// The signature of the application that created the file.
    pub fileCreator: OSType,
    /// Finder flags. See `FinderFlags`.
    pub finderFlags: FinderFlags,
    /// The location--specified in coordinates local to the window--of the file's icon within its window.
    pub location: Point,
    /// The window in which the file's icon appears; this information is meaningful only to the Finder.
    pub reservedField: u16,
}

impl FileInfo {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<FileInfo> {
        let mut fileType = [0u8; 4];
        r.read(&mut fileType)?;
        let mut fileCreator = [0u8; 4];
        r.read(&mut fileCreator)?;
        let finderFlags = r.read_u16::<BigEndian>()?.into();
        let location = Point::read(r)?;
        let reservedField = r.read_u16::<BigEndian>()?;
        Ok(FileInfo {
            fileType: OSType(fileType),
            fileCreator: OSType(fileCreator),
            finderFlags,
            location,
            reservedField,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        w.write(&self.fileType.0)?;
        w.write(&self.fileCreator.0)?;
        w.write_u16::<BigEndian>(self.finderFlags.into())?;
        self.location.write(w)?;
        w.write_u16::<BigEndian>(self.reservedField)?;
        Ok(())
    }
}

/// Defines an extended file information structure.
///
/// The `ExtendedFileInfo` structure is preferred over the FXInfo structure.
#[derive(Clone, Default)]
#[repr(C)]
pub struct ExtendedFileInfo {
    /// Reserved (set to 0).
    pub reserved1: [i16; 4],
    /// Extended flags. See `ExtendedFinderFlags`.
    pub extendedFinderFlags: ExtendedFinderFlags,
    /// Reserved (set to 0).
    pub reserved2: i16,
    /// If the user moves the file onto the desktop, the directory ID of the folder from which the
    /// user moves the file.
    pub putAwayFolderID: i32,
}

impl fmt::Debug for ExtendedFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.reserved1 == [0i16; 4] && self.reserved2 == 0 {
            f.debug_struct("ExtendedFileInfo")
                .field("extendedFinderFlags", &self.extendedFinderFlags)
                .field("putAwayFolderID", &self.putAwayFolderID)
                .finish()
        } else {
            f.debug_struct("ExtendedFileInfo")
                .field("reserved1", &self.reserved1)
                .field("extendedFinderFlags", &self.extendedFinderFlags)
                .field("reserved2", &self.reserved2)
                .field("putAwayFolderID", &self.putAwayFolderID)
                .finish()
        }
    }
}

impl ExtendedFileInfo {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<ExtendedFileInfo> {
        let mut reserved1 = [0i16; 4];
        r.read_i16_into::<BigEndian>(&mut reserved1)?;
        let extendedFinderFlags = r.read_u16::<BigEndian>()?.into();
        let reserved2 = r.read_i16::<BigEndian>()?;
        let putAwayFolderID = r.read_i32::<BigEndian>()?;
        Ok(ExtendedFileInfo {
            reserved1,
            extendedFinderFlags,
            reserved2,
            putAwayFolderID,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        for r in &self.reserved1 {
            w.write_i16::<BigEndian>(*r)?;
        }
        w.write_u16::<BigEndian>(self.extendedFinderFlags.into())?;
        w.write_i16::<BigEndian>(self.reserved2)?;
        w.write_i32::<BigEndian>(self.putAwayFolderID)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct FinderInfoFile {
    pub file_info: FileInfo,
    pub extended_file_info: ExtendedFileInfo,
}

impl FinderInfoFile {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<FinderInfoFile> {
        let file_info = FileInfo::read(r)?;
        let extended_file_info = ExtendedFileInfo::read(r)?;
        Ok(FinderInfoFile {
            file_info,
            extended_file_info,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        self.file_info.write(w)?;
        self.extended_file_info.write(w)?;
        Ok(())
    }
}

/// Defines a directory information structure.
///
/// The `FolderInfo` structure is preferred over the DInfo structure.
#[derive(Clone, Default)]
#[repr(C)]
pub struct FolderInfo {
    /// The rectangle for the window that the Finder displays when the user opens the folder.
    pub windowBounds: Rect,
    /// Finder flags. See `FinderFlags`.
    pub finderFlags: FinderFlags,
    /// Location of the folder in the parent window.
    pub location: Point,
    /// Reserved. Set to 0.
    pub reservedField: u16,
}

impl fmt::Debug for FolderInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.reservedField == 0 {
            f.debug_struct("FolderInfo")
                .field("windowBounds", &self.windowBounds)
                .field("finderFlags", &self.finderFlags)
                .field("location", &self.location)
                .finish()
        } else {
            f.debug_struct("FolderInfo")
                .field("windowBounds", &self.windowBounds)
                .field("finderFlags", &self.finderFlags)
                .field("location", &self.location)
                .field("reservedField", &self.reservedField)
                .finish()
        }
    }
}

impl FolderInfo {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<FolderInfo> {
        let windowBounds = Rect::read(r)?;
        let finderFlags = r.read_u16::<BigEndian>()?.into();
        let location = Point::read(r)?;
        let reservedField = r.read_u16::<BigEndian>()?;
        Ok(FolderInfo {
            windowBounds,
            finderFlags,
            location,
            reservedField,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        self.windowBounds.write(w)?;
        w.write_u16::<BigEndian>(self.finderFlags.into())?;
        self.location.write(w)?;
        w.write_u16::<BigEndian>(self.reservedField)?;
        Ok(())
    }
}

/// Defines an extended directory information structure.
///
/// The `ExtendedFolderInfo` structure is preferred over the DXInfo structure.
#[derive(Clone, Default)]
#[repr(C)]
pub struct ExtendedFolderInfo {
    /// Scroll position within the Finder window.
    /// The Finder does not necessarily save this position immediately upon user action.
    pub scrollPosition: Point,
    /// Reserved (set to 0).
    pub reserved1: i32,
    /// Extended Finder flags. See `ExtendedFinderFlags`.
    pub extendedFinderFlags: ExtendedFinderFlags,
    /// Reserved (set to 0).
    pub reserved2: i16,
    /// If the user moves the folder onto the desktop, the directory ID of the folder from which
    /// the user moves it.
    pub putAwayFolderID: i32,
}

impl fmt::Debug for ExtendedFolderInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.reserved1 == 0 && self.reserved2 == 0 {
            f.debug_struct("ExtendedFolderInfo")
                .field("scrollPosition", &self.scrollPosition)
                .field("extendedFinderFlags", &self.extendedFinderFlags)
                .field("putAwayFolderID", &self.putAwayFolderID)
                .finish()
        } else {
            f.debug_struct("ExtendedFolderInfo")
                .field("scrollPosition", &self.scrollPosition)
                .field("reserved1", &self.reserved1)
                .field("extendedFinderFlags", &self.extendedFinderFlags)
                .field("reserved2", &self.reserved2)
                .field("putAwayFolderID", &self.putAwayFolderID)
                .finish()
        }
    }
}

impl ExtendedFolderInfo {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<ExtendedFolderInfo> {
        let scrollPosition = Point::read(r)?;
        let reserved1 = r.read_i32::<BigEndian>()?;
        let extendedFinderFlags = r.read_u16::<BigEndian>()?.into();
        let reserved2 = r.read_i16::<BigEndian>()?;
        let putAwayFolderID = r.read_i32::<BigEndian>()?;
        Ok(ExtendedFolderInfo {
            scrollPosition,
            reserved1,
            extendedFinderFlags,
            reserved2,
            putAwayFolderID,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        self.scrollPosition.write(w)?;
        w.write_i32::<BigEndian>(self.reserved1)?;
        w.write_u16::<BigEndian>(self.extendedFinderFlags.into())?;
        w.write_i16::<BigEndian>(self.reserved2)?;
        w.write_i32::<BigEndian>(self.putAwayFolderID)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct FinderInfoFolder {
    pub folder_info: FolderInfo,
    pub extended_folder_info: ExtendedFolderInfo,
}

impl FinderInfoFolder {
    pub fn read<R: ReadBytesExt>(r: &mut R) -> io::Result<FinderInfoFolder> {
        let folder_info = FolderInfo::read(r)?;
        let extended_folder_info = ExtendedFolderInfo::read(r)?;
        Ok(FinderInfoFolder {
            folder_info,
            extended_folder_info,
        })
    }

    pub fn write<W: WriteBytesExt>(&self, w: &mut W) -> io::Result<()> {
        self.folder_info.write(w)?;
        self.extended_folder_info.write(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FinderInfo xattr with custom icon bit off.
    const DEFAULT_FINDERINFO_XATTR_VALUE: [u8; 32] = [0u8; 32];

    // FinderInfo xattr with the custom icon bit on.
    const FINDERINFO_XATTR_VALUE_ON: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    // FinderInfo xattr with label = Blue and custom icon bit set.
    const FINDERINFO_XATTR_RED_BLUE_FOO_ICON: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    // FinderInfo xattr with label = Red
    const FINDERINFO_XATTR_FOO_BLUE_RED: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    // FinderInfo xattr with label = Red with the the custom icon bit set.
    const FINDERINFO_XATTR_FOO_BLUE_RED_ICON: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    #[test]
    fn test_finderinfo_sizes() {
        assert_eq!(::std::mem::size_of::<FileInfo>(), 16);
        assert_eq!(::std::mem::size_of::<ExtendedFileInfo>(), 16);
        assert_eq!(::std::mem::size_of::<FinderInfoFile>(), 32);
        assert_eq!(::std::mem::size_of::<FolderInfo>(), 16);
        assert_eq!(::std::mem::size_of::<ExtendedFolderInfo>(), 16);
        assert_eq!(::std::mem::size_of::<FinderInfoFolder>(), 32);
    }

    #[test]
    fn test_set_get_finderinfo_file() {
        let mut finfo =
            FinderInfoFile::read(&mut io::Cursor::new(DEFAULT_FINDERINFO_XATTR_VALUE)).unwrap();
        assert!(!finfo.file_info.finderFlags.has_custom_icon());
        assert_eq!(finfo.file_info.finderFlags.color(), None);

        let mut cursor = io::Cursor::new(vec![]);
        finfo.write(&mut cursor).unwrap();
        let serialized = cursor.into_inner();
        assert_eq!(DEFAULT_FINDERINFO_XATTR_VALUE.to_vec(), serialized);

        finfo
            .file_info
            .finderFlags
            .set_color(Some(LabelColor::Blue));
        finfo.file_info.finderFlags.set_has_custom_icon(true);

        let mut cursor = io::Cursor::new(vec![]);
        finfo.write(&mut cursor).unwrap();
        let serialized = cursor.into_inner();
        assert_eq!(serialized.len(), 32);
        assert_eq!(serialized, FINDERINFO_XATTR_RED_BLUE_FOO_ICON);

        let finfo =
            FinderInfoFile::read(&mut io::Cursor::new(FINDERINFO_XATTR_FOO_BLUE_RED_ICON)).unwrap();
        assert!(finfo.file_info.finderFlags.has_custom_icon());
        assert_eq!(finfo.file_info.finderFlags.color(), Some(LabelColor::Red));
    }

    #[test]
    fn test_set_get_finderinfo_folder() {
        let mut finfo =
            FinderInfoFolder::read(&mut io::Cursor::new(DEFAULT_FINDERINFO_XATTR_VALUE)).unwrap();
        assert!(!finfo.folder_info.finderFlags.has_custom_icon());
        assert_eq!(finfo.folder_info.finderFlags.color(), None);

        let mut cursor = io::Cursor::new(vec![]);
        finfo.write(&mut cursor).unwrap();
        let serialized = cursor.into_inner();
        assert_eq!(DEFAULT_FINDERINFO_XATTR_VALUE.to_vec(), serialized);

        finfo.folder_info.finderFlags.set_has_custom_icon(true);

        let mut cursor = io::Cursor::new(vec![]);
        finfo.write(&mut cursor).unwrap();
        let serialized = cursor.into_inner();
        assert_eq!(serialized.len(), 32);
        assert_eq!(serialized, FINDERINFO_XATTR_VALUE_ON);

        finfo
            .folder_info
            .finderFlags
            .set_color(Some(LabelColor::Blue));

        let mut cursor = io::Cursor::new(vec![]);
        finfo.write(&mut cursor).unwrap();
        let serialized = cursor.into_inner();
        assert_eq!(serialized.len(), 32);
        assert_eq!(serialized, FINDERINFO_XATTR_RED_BLUE_FOO_ICON);

        let finfo =
            FinderInfoFolder::read(&mut io::Cursor::new(FINDERINFO_XATTR_FOO_BLUE_RED)).unwrap();
        assert!(!finfo.folder_info.finderFlags.has_custom_icon());
        assert_eq!(finfo.folder_info.finderFlags.color(), Some(LabelColor::Red));
    }
}
