use std::path::{Path, PathBuf};

extern crate walkdir;
use walkdir::WalkDir;
extern crate clap;
use clap::{App, Arg};

struct Folder {
    folder_path: PathBuf,
    video_files: Vec<PathBuf>,
}

fn main() {
    let matches = App::new("Notrobro")
        .version("1.0")
        .author("Team Kodi")
        .about("Finds intros and outros. Then creates files, so your videoplayers can skip those.")
        .arg(
            Arg::with_name("PATH")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("TV show directory path (mandatory argument)")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("THRESHOLD")
                .short("t")
                .long("threshold")
                .help("Threshold for scene change detection(default=0.35)")
                .takes_value(true),
        ).arg(
            Arg::with_name("FORCE")
                .short("f")
                .long("force")
                .help("Process all videos in the directory (default=False)"),
        ).get_matches();

    let path_string = matches.value_of("PATH").unwrap();
    let threshold = matches.value_of("THRESHOLD").unwrap_or("0.35");
    let force = matches.is_present("FORCE");

    println!("Value for path: {}", path_string);

    println!("Using threshold: {}", threshold);

    println!("Using force: {}", force);

    let path = Path::new(path_string);
    let mut folder_count = 0;

    if path.exists() {
        println!("Path exists");

        // find all folders with two or more video files
        let folders: Vec<Folder> = WalkDir::new(path_string)
            .into_iter()
            .filter_map(|result| result.ok())
            .filter(|entry| entry.file_type().is_dir())
            .map(|folder| {
                
                // find all videos in the folder
                folder_count += 1;

                let videos: Vec<PathBuf> = WalkDir::new(folder.path())
                    .max_depth(1)
                    .into_iter()
                    .filter_map(|result| result.ok())
                    .filter(|entry| entry.is_video())
                    .map(|entry| entry.path().into())
                    .collect();

                Folder {
                    folder_path: folder.path().into(),
                    video_files: videos,
                }
            }).filter(|folder| folder.video_files.len() >= 2)
            .collect();

        println!(
            "{} folders found, {} folders searched",
            folders.len(),
            folder_count
        );

        // 1. create edl-files
        for folder in folders {
            folder
                .video_files
                .into_iter()
                .filter(|video| force || !video.has_edl())
                .for_each(|video| println!("creating edl for {}", video.display()))
        }

    // 2. compare edls ???

    // 3. Profit!
    } else {
        println!("Path doesn't seem to exist. Did you mistype?");
    }
}

fn get_edl(path: &Path) -> Option<PathBuf> {
    if path.file_stem().is_some() {
        let mut edl_path: PathBuf = path.into();
        edl_path.set_extension("edl");

        if edl_path.exists() {
            println!("Edl does exist {}", edl_path.display());
            Some(edl_path)
        } else {
            println!("Edl does not exist {}", edl_path.display());
            None
        }
    } else {
        None
    }
}

trait Notrobro {
    fn is_video(&self) -> bool;
    fn has_edl(&self) -> bool;
}

impl Notrobro for Path {
    fn is_video(&self) -> bool {
        let ext = self.extension().and_then(|s| s.to_str());

        if let Some(ext) = ext {
            match ext {
                "mp4" | "mkv" | "avi" | "mov" | "wmv" => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn has_edl(&self) -> bool {
        if self.file_stem().is_some() {
            let mut edl_path: PathBuf = self.into();
            edl_path.set_extension("edl");

            if edl_path.exists() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Notrobro for walkdir::DirEntry {
    fn is_video(&self) -> bool {
        self.path().is_video()
    }
    fn has_edl(&self) -> bool {
        self.path().has_edl()
    }
}

impl Notrobro for PathBuf {
    fn is_video(&self) -> bool {
        let ext = self.extension().and_then(|s| s.to_str());

        if let Some(ext) = ext {
            match ext {
                "mp4" | "mkv" | "avi" | "mov" | "wmv" => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn has_edl(&self) -> bool {
        if self.file_stem().is_some() {
            let mut edl_path: PathBuf = self.into();
            edl_path.set_extension("edl");

            if edl_path.exists() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
