use pelite::PeFile;
use std::fs::read;
use std::path::Path;
#[cfg(windows)]
use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PeModifyError {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("PE解析错误: {0}")]
    Pelite(#[from] pelite::Error),
    #[error("资源查找错误: {0}")]
    FindError(#[from] pelite::resources::FindError),
    #[error("未找到版本信息资源")]
    VersionInfoNotFound,
    #[cfg(windows)]
    #[error("Windows API调用失败: {0}")]
    WindowsApi(&'static str),
}

/// PE版本元数据配置
#[derive(Debug, Default, Clone)]
pub struct VersionMetadata {
    /// 公司名称
    pub company_name: Option<String>,
    /// 文件描述
    pub file_description: Option<String>,
    /// 文件版本
    pub file_version: Option<String>,
    /// 内部名称
    pub internal_name: Option<String>,
    /// 法律版权
    pub legal_copyright: Option<String>,
    /// 原始文件名
    pub original_filename: Option<String>,
    /// 产品名称
    pub product_name: Option<String>,
    /// 产品版本
    pub product_version: Option<String>,
}

/// 读取PE文件的版本元数据
///
/// # Arguments
/// * `pe_path` - PE文件路径
/// * `lang_id` - 可选语言ID，默认取第一个找到的语言版本
///   常见语言ID：0x0804=中文(中国)，0x0409=英文(美国)
pub fn read_pe_version_info(
    pe_path: impl AsRef<Path>,
    _lang_id: Option<u16>,
) -> Result<VersionMetadata, PeModifyError> {
    // 读取PE文件
    let pe_data = read(pe_path)?;

    // 解析PE文件
    let pe = PeFile::from_bytes(&pe_data)?;

    // 获取资源目录
    let resources = pe.resources()?;

    // 查找版本信息资源
    let version_info = resources.version_info()?;

    let mut metadata = VersionMetadata::default();

    // pelite 0.10 简化读取，默认读取所有字符串
    version_info.strings(Default::default(), |key, value| match key {
        "CompanyName" => metadata.company_name = Some(value.to_string()),
        "FileDescription" => metadata.file_description = Some(value.to_string()),
        "FileVersion" => metadata.file_version = Some(value.to_string()),
        "InternalName" => metadata.internal_name = Some(value.to_string()),
        "LegalCopyright" => metadata.legal_copyright = Some(value.to_string()),
        "OriginalFilename" => metadata.original_filename = Some(value.to_string()),
        "ProductName" => metadata.product_name = Some(value.to_string()),
        "ProductVersion" => metadata.product_version = Some(value.to_string()),
        _ => {}
    });

    Ok(metadata)
}

/// 修改PE文件的版本元数据（占位实现，功能待完善）
pub fn modify_pe_version_info(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    _metadata: &VersionMetadata,
) -> Result<(), PeModifyError> {
    #[cfg(windows)]
    {
        update_version_resource_from_file(input_path.as_ref(), output_path.as_ref())?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        // 非 Windows 平台回退为复制
        std::fs::copy(input_path, output_path)?;
        return Ok(());
    }
}

#[cfg(windows)]
fn to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
fn update_version_resource_from_file(src_exe: &Path, dst_exe: &Path) -> Result<(), PeModifyError> {
    const RT_VERSION: usize = 16;
    const DEFAULT_LANG: u16 = 0x0409;

    let src_w = to_wide(src_exe);
    let dst_w = to_wide(dst_exe);

    let mut handle = 0u32;
    let size = unsafe { GetFileVersionInfoSizeW(src_w.as_ptr(), &mut handle) };
    if size == 0 {
        return Err(PeModifyError::WindowsApi("GetFileVersionInfoSizeW"));
    }

    let mut version_block = vec![0u8; size as usize];
    let ok = unsafe {
        GetFileVersionInfoW(
            src_w.as_ptr(),
            0,
            size,
            version_block.as_mut_ptr().cast::<c_void>(),
        )
    };
    if ok == 0 {
        return Err(PeModifyError::WindowsApi("GetFileVersionInfoW"));
    }

    let mut trans_ptr: *mut c_void = ptr::null_mut();
    let mut trans_len = 0u32;
    let trans_key: Vec<u16> = "\\VarFileInfo\\Translation\0".encode_utf16().collect();
    let lang = unsafe {
        if VerQueryValueW(
            version_block.as_mut_ptr().cast::<c_void>(),
            trans_key.as_ptr(),
            &mut trans_ptr,
            &mut trans_len,
        ) != 0
            && !trans_ptr.is_null()
            && trans_len >= 4
        {
            *(trans_ptr.cast::<u16>())
        } else {
            DEFAULT_LANG
        }
    };

    let update = unsafe { BeginUpdateResourceW(dst_w.as_ptr(), 0) };
    if update.is_null() {
        return Err(PeModifyError::WindowsApi("BeginUpdateResourceW"));
    }

    let update_ok = unsafe {
        UpdateResourceW(
            update,
            RT_VERSION as *const u16,
            1 as *const u16,
            lang,
            version_block.as_mut_ptr().cast::<c_void>(),
            size,
        )
    };
    if update_ok == 0 {
        unsafe {
            EndUpdateResourceW(update, 1);
        }
        return Err(PeModifyError::WindowsApi("UpdateResourceW"));
    }

    let end_ok = unsafe { EndUpdateResourceW(update, 0) };
    if end_ok == 0 {
        return Err(PeModifyError::WindowsApi("EndUpdateResourceW"));
    }

    Ok(())
}

#[cfg(windows)]
#[link(name = "version")]
unsafe extern "system" {
    fn GetFileVersionInfoSizeW(lptstrfilename: *const u16, lpdwhandle: *mut u32) -> u32;
    fn GetFileVersionInfoW(
        lptstrfilename: *const u16,
        dwhandle: u32,
        dwlen: u32,
        lpdata: *mut c_void,
    ) -> i32;
    fn VerQueryValueW(
        pblock: *mut c_void,
        lpsubblock: *const u16,
        lplpbuffer: *mut *mut c_void,
        pulen: *mut u32,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn BeginUpdateResourceW(pfilename: *const u16, bdeleteexistingresources: i32) -> *mut c_void;
    fn UpdateResourceW(
        hupdate: *mut c_void,
        lptype: *const u16,
        lpname: *const u16,
        wlanguage: u16,
        lpdata: *mut c_void,
        cbdata: u32,
    ) -> i32;
    fn EndUpdateResourceW(hupdate: *mut c_void, fdiscard: i32) -> i32;
}
