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
#[command(about = "A toolkit for managing and creating Kingdom Come Deliverance mod files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Unpack a .pak file into a directory
    Unpack {
        input_pak: String,
        output_dir: String,
    },
    /// Pack a directory into a .pak file
    Pack {
        input_dir: String,
        output_pak: String,
    },
    /// Create a new mod (not yet implemented)
    Create {
        modname: String,
    },
}

fn pack(input_dir: &str, output_pak: &str) -> ZipResult<()> {
    let file = File::create(output_pak)?;
    let mut zip = ZipWriter::new(file);

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
    println!("Packed: {} -> {}", input_dir, output_pak);
    Ok(())
}

fn unpack(input_pak: &str, output_dir: &str) -> ZipResult<()> {
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

    println!("Unpacked: {} -> {}", input_pak, output_dir);
    Ok(())
}

fn create(modname: &str) {
    println!("Hello, world! Mod '{}' will be created here (not yet implemented).", modname);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Unpack { input_pak, output_dir } => {
            if let Err(e) = unpack(input_pak, output_dir) {
                eprintln!("Error unpacking: {}", e);
            }
        }
        Commands::Pack { input_dir, output_pak } => {
            if let Err(e) = pack(input_dir, output_pak) {
                eprintln!("Error packing: {}", e);
            }
        }
        Commands::Create { modname } => {
            create(modname);
        }
    }
}