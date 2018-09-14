use std::path::{Path, PathBuf};

extern crate walkdir;
use walkdir::WalkDir;
extern crate clap;
use clap::{App, Arg};

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
        )
        .arg(
            Arg::with_name("THRESHOLD")
                .short("t")
                .long("threshold")
                .help("Threshold for scene change detection(default=0.35)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FORCE")
                .short("f")
                .long("force")
                .help("Process all videos in the directory (default=False)"),
        )
        .get_matches();

    let path_string = matches.value_of("PATH").unwrap();
    let threshold = matches.value_of("THRESHOLD").unwrap_or("0.35");
    let force = matches.is_present("FORCE");

    println!("Value for path: {}", path_string);

    println!("Using threshold: {}", threshold);

    println!("Using force: {}", force);

    let path = Path::new(path_string);

    if path.exists() {
        println!("Path exists");

        WalkDir::new(path_string)
            .into_iter()
            .filter_map(|e| e.ok())
            .for_each(|e| {
                do_something(e.path(), force);
            });
    } else {
        println!("Path doesn't seem to exist. Did you mistype?");
    }
}

fn do_something(path: &Path, force: bool) -> Option<()> {
    match path.extension()?.to_str()? {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" => {
            println!("{}", path.display());

            if path.file_stem().is_some() {
                let mut edl_path: PathBuf = path.into();
                edl_path.set_extension("edl");

                if edl_path.exists() {
                    println!("Edl does exist {}", edl_path.display());

                    if force {
                        // Proceed
                    }
                } else {
                    println!("Edl does not exist {}", edl_path.display());
                }
            }
        }

        _ => (),
    }

    Some(())
}
