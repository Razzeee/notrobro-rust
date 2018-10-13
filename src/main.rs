use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

extern crate walkdir;
use walkdir::WalkDir;
extern crate clap;
use clap::{App, Arg};
extern crate tempfile;
use tempfile::tempdir;
extern crate pihash;
use pihash::PIHash;

#[macro_use]
extern crate lazy_static;
extern crate regex;
use regex::Regex;

struct Folder {
    folder_path: PathBuf,
    video_files: Vec<PathBuf>,
}

struct Video {
    path: PathBuf,
    intro: Vec<SceneChange>,
    outro: Vec<SceneChange>,
}

struct SceneChange {
    temp_picture_path: PathBuf,
    phash: u64,
    time: String,
}

#[derive(PartialEq, Eq)]
enum IntroOutro {
    Intro,
    Outro,
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

    let path_string = matches.value_of("PATH").unwrap(); // Todo remove trailing backslash from path
    let threshold = matches.value_of("THRESHOLD").unwrap_or("0.35");
    let force = matches.is_present("FORCE");

    println!("Value for path: {}", path_string);

    println!("Using threshold: {}", threshold);

    println!("Using force: {}", force); // Todo use this

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
        let mut hashed_videos = Vec::new();
        for folder in folders {
            folder
                .video_files
                .into_iter()
                .filter(|video| force || !video.has_edl())
                .for_each(|video| hashed_videos.push(call_ffmpeg(&video, threshold)))
        }

    // 2. compare edls ???
    for video in hashed_videos {
        println!("{:#?}", video.path);

        for intro in video.intro {
            println!("{:#?} {} {}", intro.temp_picture_path, intro.time, intro.phash);
        }
        for outro in video.outro {
            println!("{:#?} {} {}", outro.temp_picture_path, outro.time, outro.phash);
        }
        
    }
    println!("Done processing");
    // 3. Profit!
    } else {
        println!("Path {:#?} doesn't seem to exist. Did you mistype?", path);
    }
}

fn create_hashes(path: &PathBuf, threshold: &str, intro_outro: IntroOutro) -> Vec<SceneChange> {
    let dir = tempdir().unwrap();
    let concat_string: &Path = &dir.path().join("%04d.jpg");

    let mut command = Command::new("ffmpeg");

    if intro_outro == IntroOutro::Intro {
        command
            .arg("-i")
            .arg(path.to_str().unwrap())
            .arg("-ss")
            .arg("0")
            .arg("-to")
            .arg("360")
            .arg("-vf")
            .arg(format!("select='gt(scene,{})',showinfo", threshold))
            .arg("-vsync")
            .arg("vfr")
            .arg(concat_string);
    } else {
        command
            .arg("-i")
            .arg(path.to_str().unwrap())
            .arg("-sseof")
            .arg("-300")
            .arg("-vf")
            .arg(format!("select='gt(scene,{})',showinfo", threshold))
            .arg("-vsync")
            .arg("vfr")
            .arg(concat_string);
    }

    let output = command.output().expect("failed to execute process");
    println!("command: {:#?}", command);
    // println!("output: {:#?}", output);
    let mut scene_changes = find_timings(&format!("{:#?}", &output));

    for (i, path) in fs::read_dir(&dir.path()).unwrap().enumerate() {
        lazy_static! {
            static ref PIHASH: PIHash<'static> = PIHash::new(None);
        }

        let unwraped_path = &path.unwrap().path();
        scene_changes[i].temp_picture_path = unwraped_path.to_owned();
        scene_changes[i].phash = PIHASH.get_phash(unwraped_path);
    }

    println!("Done");

    dir.close().unwrap();

    scene_changes
}

fn call_ffmpeg(path: &PathBuf, threshold: &str) -> Video {
    Video {
        path: path.to_path_buf(),
        intro: create_hashes(path, threshold, IntroOutro::Intro),
        outro: create_hashes(path, threshold, IntroOutro::Outro),
    }
}

// Todo - compare hashes with get_hamming_distance

fn find_timings(output: &str) -> Vec<SceneChange> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r" pts_time:(\d+\.\d+) ").unwrap();
    }
    let mut vec = Vec::new();
    for caps in RE.captures_iter(output) {
        vec.push(SceneChange {
            time: caps[1].to_string(),
            temp_picture_path: PathBuf::new(),
            phash: 0,
        });
    }
    vec
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
