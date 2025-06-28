use anyhow::Result;
use baad::helpers::{ErrorContext, ErrorExt};
use baad::{debug, info, success};
use bacy::TableZipFile;
use std::path::PathBuf;
use tokio::fs;

pub async fn extract_media(input_path: PathBuf, output: PathBuf) -> Result<()> {
    info!("Extracting JP voices...");
    
    for entry in input_path.read_dir().handle_errors()? {
        let entry = entry.handle_errors()?;
        let path = entry.path();
        
        if path.extension().error_context("Failed to get file extension")? != "zip" {
            continue;
        }

        extract_zip(path, &output).await?;
    }
    
    Ok(())
}

pub async fn extract_zip(path: PathBuf, output: &PathBuf) -> Result<()> {
    let buf = fs::read(&path).await.handle_errors()?;
    let filename = path.file_name()
        .error_context("Failed to get filename from path")?
        .to_str()
        .error_context("Failed to convert filename to string")?;

    let mut zip = TableZipFile::new(buf, filename.to_lowercase()).handle_errors()?;
    let dir = output.join(filename.trim_end_matches(".zip"));

    debug!("Extracting <b><u><blue>{}</> to <b><u><bright-blue>{:?}</>", filename, dir);

    fs::create_dir_all(&dir).await?;

    for (name, buf) in zip.extract_all().handle_errors()? {
        fs::write(dir.join(name), buf).await.handle_errors()?;
    }

    success!("<b><u><green>{}</> extracted", filename);
    Ok(())
}