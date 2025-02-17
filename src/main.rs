use clap::{Parser, Subcommand};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use zip::{
    read::ZipArchive,
    write::{FileOptions, ZipWriter},
    CompressionMethod,
    result::ZipResult,
};

#[derive(Parser)]
#[command(name = "kcd-toolkit")]
#[command(version = "1.0")]
#[command(about = "A tool to compress and extract CryEngine .pak files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compress {
        input_dir: String,
        output_pak: String,
    },
    Extract {
        input_pak: String,
        output_dir: String,
    },
}

fn compress_pak(input_dir: &str, output_pak: &str) -> ZipResult<()> {
    let file = File::create(output_pak)?;
    let mut zip = ZipWriter::new(file);

    // Explicitly specify the type of `options`
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(input_dir) {
        let entry = entry.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Walkdir error: {}", e),
            )
        })?;
        let path = entry.path();

        // Get relative path name
        let name = path
            .strip_prefix(input_dir)
            .map_err(|e| {
                zip::result::ZipError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Path prefix error: {}", e),
                ))
            })?
            .to_str()
            .ok_or_else(|| {
                zip::result::ZipError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid UTF-8 in file path",
                ))
            })?;

        let normalized_name = name.replace("\\", "/");

        if path.is_file() {
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.start_file(normalized_name, options)?;
            zip.write_all(&buffer)?;
        } else if path.is_dir() {
            zip.add_directory(normalized_name, options)?;
        }
    }

    zip.finish()?;
    println!("Compressed: {} -> {}", input_dir, output_pak);
    Ok(())
}

fn extract_pak(input_pak: &str, output_dir: &str) -> ZipResult<()> {
    let file = File::open(input_pak)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file
            .enclosed_name()
            .ok_or(zip::result::ZipError::InvalidArchive("Invalid file name"))?;

        let out_path = Path::new(output_dir).join(file_name);

        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("Extracted: {} -> {}", input_pak, output_dir);
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compress { input_dir, output_pak } => {
            if let Err(e) = compress_pak(input_dir, output_pak) {
                eprintln!("Error compressing: {}", e);
            }
        }
        Commands::Extract { input_pak, output_dir } => {
            if let Err(e) = extract_pak(input_pak, output_dir) {
                eprintln!("Error extracting: {}", e);
            }
        }
    }
}