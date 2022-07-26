use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

fn work_dir() -> PathBuf {
    env::current_dir().expect("Working dir is not found")
}

fn find_ffmpeg() -> Option<PathBuf> {
    let exe_file = env::current_exe().expect("Executable file is not found");
    let exe_dir = exe_file
        .parent()
        .expect("The parent of executable file is not found");
    let work_dir = work_dir();
    let mut work_dir_lib = work_dir.clone();
    work_dir_lib.push("lib");
    let work_dir_lib = work_dir_lib;
    let exe_dir_lib = exe_dir.join("lib");
    let dirs: Vec<&Path> = vec![
        exe_dir,
        exe_dir_lib.as_path(),
        work_dir.as_path(),
        work_dir_lib.as_path(),
    ];

    dirs.into_iter()
        .find_map(|i| {
            let ffmpeg_exe = i.join("ffmpeg");
            let ffmpeg_win_exe = i.join("ffmpeg.exe");
            vec![ffmpeg_exe, ffmpeg_win_exe]
                .into_iter()
                .find(|i| i.exists())
        })
        .or_else(|| {
            let program = if cfg!(windows) {
                "where.exe"
            } else {
                "which"
            };
            let output = Command::new(program)
                .arg("ffmpeg")
                .stderr(Stdio::inherit())
                .output();
            if let Ok(output) = output {
                let path = String::from_utf8_lossy(&*output.stdout).to_string();
                let path = path.trim_end();
                dbg!(&path);
                let buf = PathBuf::from(path.to_string());
                if !buf.exists() {
                    return None
                }
                Some(buf)
            } else {
                None
            }
        })
}

fn main() {
    std::panic::set_hook(Box::new(|info| eprintln!("{info}")));
    let ffmpeg = find_ffmpeg().expect("cannot find ffmpeg executable");
    println!("Found ffmpeg executable: {}", ffmpeg.to_string_lossy());
    let work_dir = work_dir();
    let video_regex = Regex::new(r#"(?x)\.(mp4|ts|flv|m4v)$"#).unwrap();
    let exclude_regex = Regex::new(r#".output\.(mp4|ts|flv|m4v)$"#).unwrap();

    let files = work_dir
        .read_dir()
        .expect(&*format!("Failed to read {}", work_dir.to_string_lossy()));

    let mut matches = Vec::new();

    for f in files {
        let file = f.unwrap();
        let file_name = file.file_name();
        let str = file_name.to_string_lossy();
        if !video_regex.is_match(&*str) || exclude_regex.is_match(&*str) {
            continue;
        }
        matches.push(file.path());
    }

    matches.sort();

    if matches.is_empty() {
        println!("Nothing to process");
        return;
    }
    println!("Inputs: [");
    matches
        .iter()
        .for_each(|i| println!("  {}", i.file_name().unwrap().to_string_lossy()));
    println!("]");

    let input_txt = work_dir.join(".inputs.txt");
    let delete_input_txt = || {
        let msg = format!("Failed to delete {}", input_txt.to_string_lossy());
        fs::remove_file(input_txt.clone()).expect(&*msg);
    };

    if input_txt.exists() {
        delete_input_txt();
    }

    println!(
        "Start writing inputs file to {}...",
        input_txt.to_string_lossy()
    );

    let mut input_txt_file =
        File::create(input_txt.clone()).expect("Failed to create `.inputs.txt`");
    matches.iter().for_each(|i| {
        input_txt_file
            .write(format!("file '{}'\n", i.to_string_lossy()).as_ref())
            .expect("Failed to write...");
    });

    let output = work_dir.join(format!(
        "{}.output.mp4",
        chrono::offset::Local::now().format("%Y%m%dT%H%M%S")
    ));

    if output.exists() {
        panic!(
            "Output file '{}' is already exists.",
            output.to_string_lossy()
        )
    }
    let mut command = Command::new(&*ffmpeg.to_string_lossy());
    command.args([
        "-hide_banner",
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        &input_txt.to_string_lossy(),
        "-c",
        "copy",
        &output.to_string_lossy(),
    ]);
    let args: Vec<_> = command.get_args().map(|i| i.to_string_lossy()).collect();
    let args = args.join(" ");

    println!("{} {}", command.get_program().to_string_lossy(), args);

    let child = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to combine video");

    let output = child.wait_with_output().expect("failed to wait on child");

    println!("Exit code of ffmpeg: {}", output.status);

    // clean
    delete_input_txt();
}
