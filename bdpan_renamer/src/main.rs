use clap::Parser;
use rusqlite::params;

#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    input: std::path::PathBuf,
    #[clap(short, long)]
    output: Option<std::path::PathBuf>,
}

#[derive(Debug)]
struct Cache {
    server_full_path: String,
    file_name: String,
}

fn main() {
    let args = Cli::parse();

    if !args.input.exists() {
        println!("{} is not exist", args.input.display());
        return;
    }

    let mut input_path = args.input.clone();
    input_path.push("Data");
    input_path.push("Documents");

    let user_dir = find_user_dir(&input_path).unwrap_or_else(|e| {
        println!("find user dir error: {}", e);
        std::process::exit(1);
    });
    let output_dir = args.output.unwrap_or_else(|| {
        let mut path = std::env::current_dir().unwrap();
        path.push("output");
        path
    });
    move_cache_file(&user_dir, &output_dir).unwrap_or_else(|e| {
        println!("move cache file error: {}", e);
        std::process::exit(1);
    });
}

fn find_user_dir(path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let cache_dir = path.join("Cache");
        let sqlite_filr = path.join("netdisk.sqlite");
        if cache_dir.exists() && sqlite_filr.exists() {
            return Ok(path);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "user document not found",
    ))
}

fn move_cache_file(path: &std::path::Path, output_dir: &std::path::Path) -> std::io::Result<()> {
    let cache_dir = path.join("Cache");
    let sqlite_file = path.join("netdisk.sqlite");
    for entry in std::fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.file_name().unwrap().to_str().unwrap().starts_with(".") {
            continue;
        }

        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let cache = read_sqlite_file(&sqlite_file, file_name)?;
        let server_path = std::path::Path::new(&cache.server_full_path);
        let output_path: std::path::PathBuf;
        if server_path.is_absolute() {
            output_path = output_dir.join(server_path.strip_prefix("/").unwrap());
        } else {
            output_path = output_dir.join(server_path);
        }
        // Move entry file to output_path
        if !output_path.parent().unwrap().exists() {
            std::fs::create_dir_all(output_path.parent().unwrap())?;
        }
        // std::fs::rename(&path, output_path)?;
        std::fs::copy(&path, output_path)?;
    }
    Ok(())
}

fn read_sqlite_file(path: &std::path::Path, file_md5: &str) -> std::io::Result<Cache> {
    let conn = rusqlite::Connection::open(path).unwrap();
    let mut stmt = conn.prepare("SELECT server_full_path, file_name FROM cachefilelist WHERE file_md5 = ?").unwrap();

    let mut rows = stmt.query(params![file_md5]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        Ok(Cache {
            server_full_path: row.get(0).unwrap(),
            file_name: row.get(1).unwrap(),
        })
    } else {
        println!("{} not found in sqlite", file_md5);
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ))
    }
}
