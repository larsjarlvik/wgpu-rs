use glob::glob;
use std::{env, process::Command};
use std::{fs, path::Path};

fn main() {
    let output_dir = Path::new("src").join("shaders-compiled");
    let shader_paths = glob("./src/shaders/**/*").expect("Failed to list shaders!");

    fs::create_dir_all(&output_dir).expect("Failed to create output directory!");

    for entry in shader_paths {
        match entry {
            Ok(path) => {
                if path.display().to_string().contains("include") {
                    continue;
                }

                println!("cargo:rerun-if-changed={}", path.display().to_string().as_str());

                let output_name = format!("{}.spv", path.file_name().unwrap().to_str().unwrap());
                let result = Command::new(format!("./tools/glslangValidator-{}", env::consts::OS))
                    .arg("-V")
                    .arg(path.display().to_string().as_str())
                    .arg("-o")
                    .arg(output_dir.join(output_name).display().to_string().as_str())
                    .status();

                if !result.unwrap().success() {
                    std::process::exit(-1);
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    println!("DONE!");
}
