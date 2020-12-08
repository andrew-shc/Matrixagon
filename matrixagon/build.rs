use std::process::Command;
use std::fs;

#[allow(unused_must_use)]
fn main() {
    let glslc_path = "C:/VulkanSDK/1.2.154.1/Bin32/glslc.exe";
    let shaders_path = "./resource/shaders";
    let binary_path = "./resource/shaders-spirv";

    fs::create_dir(String::from(binary_path));
    let bpath = fs::canonicalize(String::from(binary_path)).unwrap();

    let paths = fs::read_dir(shaders_path).unwrap();
    for path in paths {
        let fpath_raw = path.unwrap().path();
        let ext = if let Some(ext) = fpath_raw.extension() {ext} else {continue};

        match ext.to_str().unwrap() {
            // file extension names for the GLSL compiler <google/glslc>
            // https://github.com/google/shaderc/tree/main/glslc#shader-stage-specification
            "vert" |  // GLSL - Vertex Shader
            "tesc" |  // GLSL - Tessellation Control Shader
            "tese" |  // GLSL - Tessellation Evaluation Shader
            "geom" |  // GLSL - Geometry Shader
            "frag" |  // GLSL - Fragment Shader
            "comp"    // GLSL - Compute Shader
            => {
                let fpath = fs::canonicalize(fpath_raw.clone()).unwrap();

                let mut new_fpath = bpath.clone();
                new_fpath.push(fpath_raw.file_name().unwrap());

                Command::new(glslc_path)
                    .arg(fpath.to_str().unwrap())
                    .arg("-o")
                    .arg(format!("{}.spv", new_fpath.to_str().unwrap()))
                    .output()
                    .expect("Failed to compile shader files");
            },
            _ => {},
        }
    }
}
