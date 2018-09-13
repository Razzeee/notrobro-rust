use std::path::{Path, PathBuf};

extern crate walkdir;
use walkdir::WalkDir;
extern crate clap;
use clap::{Arg, App};

fn main() {
   let matches = App::new("Notrobro")
                          .version("1.0")
                          .author("Team Kodi")
                          .about("Finds intros and outros. Then creates files, so your videoplayers can skip those.")
                          .arg(Arg::with_name("PATH")
                               .short("p")
                               .long("path")
                               .value_name("PATH")
                               .help("TV show directory path (mandatory argument)")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("THRESHOLD")
                               .short("t")
                               .long("threshold")
                               .help("Threshold for scene change detection(default=0.35)")
                               .takes_value(true))
                          .arg(Arg::with_name("FORCE")
                               .short("f")
                               .long("force")
                               .help("Process all videos in the directory (default=False)"))
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

        for entry in WalkDir::new(path_string).into_iter().filter_map(|e| e.ok()) {
            match entry.path().extension() {
                Some(extension) => 
                    match extension.to_str() {
                        Some("mp4")|Some("mkv")|Some("avi")|Some("mov")|Some("wmv")  => {
                            println!("{}", entry.path().display());
                             match entry.path().file_stem() {
                                 Some(_) => {
                                     let mut p = PathBuf::from(entry.path());
                                     p.set_extension("edl");
                                     let edl_path = Path::new(&p);
                                     match edl_path.exists() {
                                         true => {
                                             println!("Edl does exist {}", edl_path.display());
                                                if force {
                                                    // Proceed
                                                }
                                             },
                                         false => {
                                             println!("Edl does not exist {}", edl_path.display());
                                         }
                                     }
                                     },
                                 None => ()
                             }
                        },
                        _ => (),
                    },
                None => ()
            }
        }


    } else {
        println!("Path doesn't seem to exist. Did you mistype?");
    }

}

