use anyhow::*;
use glob::glob;
use rayon::prelude::*;
use std::fs::*;
use std::path::Path;
use std::path::PathBuf;

struct ShaderData {
    src: String,
    src_path: PathBuf,
    spv_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf, output_dir: &PathBuf) -> Result<Self> {
        let extension = src_path
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension cannot be converted to &str")?;
        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsupported shader: {}", src_path.display()),
        };

        let src = read_to_string(src_path.clone())?;

        let spv_name = src_path.with_extension(format!("{}.spv", extension));
        let spv_path = output_dir.join(spv_name.file_name().unwrap());

        eprintln!("{}", output_dir.join(spv_name.file_name().unwrap()).display());
        Ok(Self {
            src,
            src_path,
            spv_path,
            kind,
        })
    }
}

fn main() -> Result<()> {
    let output_dir = Path::new("src").join("shaders").join("compiled");
    create_dir_all(&output_dir).expect("Failed to create output directory!");

    let mut shader_paths = vec![];
    shader_paths.extend(glob("./src/shaders/**/*.vert")?);
    shader_paths.extend(glob("./src/shaders/**/*.frag")?);
    shader_paths.extend(glob("./src/shaders/**/*.comp")?);
    let shaders: Vec<ShaderData> = shader_paths
        .into_par_iter()
        .map(|glob_result| ShaderData::load(glob_result.unwrap(), &output_dir).unwrap())
        .collect();

    let mut compiler = shaderc::Compiler::new().context("Unable to create shader compiler")?;
    let mut options = shaderc::CompileOptions::new().unwrap();

    options.set_include_callback(|name, typ, from, _depth| {
        let mut path = match typ {
            shaderc::IncludeType::Standard => {
                std::env::var_os("CARGO_PKG_DIR").map_or_else(|| std::path::PathBuf::from("/"), std::path::PathBuf::from)
            }
            shaderc::IncludeType::Relative => std::path::PathBuf::from(from)
                .parent()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| std::path::PathBuf::from("/")),
        };
        path.push(name);
        let resolved_name = path
            .clone()
            .into_os_string()
            .into_string()
            .map_err(|e| format!("path contains invalid utf8: '{:?}'", e))?;

        println!("cargo:rerun-if-changed={}", resolved_name);
        Ok(shaderc::ResolvedInclude {
            resolved_name,
            content: std::fs::read_to_string(path).map_err(|e| e.to_string())?,
        })
    });

    for shader in shaders {
        println!("cargo:rerun-if-changed={}", shader.src_path.as_os_str().to_str().unwrap());
        let compiled = compiler.compile_into_spirv(&shader.src, shader.kind, &shader.src_path.to_str().unwrap(), "main", Some(&options))?;
        write(shader.spv_path, compiled.as_binary_u8())?;
    }

    Ok(())
}
