use std::fs::read;
use std::path::Path;
use pelite::PeFile;
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
    version_info.strings(Default::default(), |key, value| {
        match key {
            "CompanyName" => metadata.company_name = Some(value.to_string()),
            "FileDescription" => metadata.file_description = Some(value.to_string()),
            "FileVersion" => metadata.file_version = Some(value.to_string()),
            "InternalName" => metadata.internal_name = Some(value.to_string()),
            "LegalCopyright" => metadata.legal_copyright = Some(value.to_string()),
            "OriginalFilename" => metadata.original_filename = Some(value.to_string()),
            "ProductName" => metadata.product_name = Some(value.to_string()),
            "ProductVersion" => metadata.product_version = Some(value.to_string()),
            _ => {}
        }
    });

    Ok(metadata)
}

/// 修改PE文件的版本元数据（占位实现，功能待完善）
pub fn modify_pe_version_info(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    _metadata: &VersionMetadata,
) -> Result<(), PeModifyError> {
    // 先简单复制文件，修改功能后续用Windows API实现
    std::fs::copy(input_path, output_path)?;
    Ok(())
}
