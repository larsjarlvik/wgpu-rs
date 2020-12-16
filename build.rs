use std::fs;
use glob::glob;
use std::process::Command;

fn main() {
    let output_dir = "src\\shaders-compiled";
    let shader_paths = glob("./src/shaders/**/*")
        .expect("Failed to list shaders!");

    fs::create_dir_all(output_dir).expect("Failed to create output directory!");

    for entry in shader_paths {
        match entry {
            Ok(path) => {
                println!("cargo:rerun-if-changed={}", path.display().to_string().as_str());

                let result = Command::new("./tools/glslangValidator")
                    .arg("-V")
                    .arg(path.display().to_string().as_str())
                    .arg("-o")
                    .arg(path.display().to_string().replace("src\\shaders", output_dir) + ".spv")
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
