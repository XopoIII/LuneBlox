use std::{
    fmt,
    fs::{FileType as StdFileType, Metadata as StdMetadata, Permissions as StdPermissions},
    io::Result as IoResult,
    str::FromStr,
    time::SystemTime,
};

use mlua::prelude::*;

use lune_std_datetime::DateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsMetadataKind {
    None,
    File,
    Dir,
    Symlink,
}

impl fmt::Display for FsMetadataKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",
                Self::File => "file",
                Self::Dir => "dir",
                Self::Symlink => "symlink",
            }
        )
    }
}

impl FromStr for FsMetadataKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "none" => Ok(Self::None),
            "file" => Ok(Self::File),
            "dir" => Ok(Self::Dir),
            "symlink" => Ok(Self::Symlink),
            _ => Err("Invalid metadata kind"),
        }
    }
}

impl TryFrom<StdFileType> for FsMetadataKind {
    type Error = &'static str;

    /**
        A socket, a FIFO or a device file is none of the kinds a script
        can be told about, so it is an error rather than a guess.
    */
    fn try_from(value: StdFileType) -> Result<Self, Self::Error> {
        if value.is_file() {
            Ok(Self::File)
        } else if value.is_dir() {
            Ok(Self::Dir)
        } else if value.is_symlink() {
            Ok(Self::Symlink)
        } else {
            Err("not a file, a directory or a symlink")
        }
    }
}

impl IntoLua for FsMetadataKind {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        if self == Self::None {
            Ok(LuaValue::Nil)
        } else {
            self.to_string().into_lua(lua)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsPermissions {
    pub(crate) read_only: bool,
}

impl From<StdPermissions> for FsPermissions {
    fn from(value: StdPermissions) -> Self {
        Self {
            read_only: value.readonly(),
        }
    }
}

impl IntoLua for FsPermissions {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let tab = lua.create_table_with_capacity(0, 1)?;
        tab.set("readOnly", self.read_only)?;
        tab.set_readonly(true);
        Ok(LuaValue::Table(tab))
    }
}

#[derive(Debug, Clone)]
pub struct FsMetadata {
    pub(crate) kind: FsMetadataKind,
    pub(crate) exists: bool,
    pub(crate) size: Option<u64>,
    pub(crate) created_at: Option<DateTime>,
    pub(crate) modified_at: Option<DateTime>,
    pub(crate) accessed_at: Option<DateTime>,
    pub(crate) permissions: Option<FsPermissions>,
}

impl FsMetadata {
    pub fn not_found() -> Self {
        Self {
            kind: FsMetadataKind::None,
            exists: false,
            size: None,
            created_at: None,
            modified_at: None,
            accessed_at: None,
            permissions: None,
        }
    }
}

impl IntoLua for FsMetadata {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let tab = lua.create_table_with_capacity(0, 6)?;
        tab.set("kind", self.kind)?;
        tab.set("exists", self.exists)?;
        tab.set("size", self.size)?;
        tab.set("createdAt", self.created_at)?;
        tab.set("modifiedAt", self.modified_at)?;
        tab.set("accessedAt", self.accessed_at)?;
        tab.set("permissions", self.permissions)?;
        tab.set_readonly(true);
        Ok(LuaValue::Table(tab))
    }
}

impl TryFrom<StdMetadata> for FsMetadata {
    type Error = &'static str;

    fn try_from(value: StdMetadata) -> Result<Self, Self::Error> {
        let file_type = value.file_type();
        Ok(Self {
            kind: file_type.try_into()?,
            exists: true,
            size: file_type.is_file().then_some(value.len()),
            created_at: system_time_to_timestamp(value.created()),
            modified_at: system_time_to_timestamp(value.modified()),
            accessed_at: system_time_to_timestamp(value.accessed()),
            permissions: Some(FsPermissions::from(value.permissions())),
        })
    }
}

fn system_time_to_timestamp(res: IoResult<SystemTime>) -> Option<DateTime> {
    match res {
        Ok(t) => match t.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => DateTime::from_unix_timestamp_float(d.as_secs_f64()).ok(),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
