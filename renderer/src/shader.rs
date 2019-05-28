use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

type SpirvBytecode = Vec<u8>;

pub enum ShaderType {
    Vertex,
    Fragment,
}

struct PathInfo(PathBuf, SystemTime);

pub struct Shader {
    spirv: Option<SpirvBytecode>,
    shader_type: ShaderType,
    pathinfo: PathInfo,
}

impl Shader {
    pub fn from_path(path: PathBuf, shader_type: ShaderType) -> Self {
        Self {
            spirv: None,
            shader_type: shader_type,
            pathinfo: PathInfo(path, SystemTime::now()),
        }
    }

    pub fn spirv(&mut self) -> std::io::Result<SpirvBytecode> {
        match self.spirv {
            Some(_) => {
                if self.outdated()? {
                    let bytecode = Self::compile(&self.pathinfo.0, &self.shader_type)?;
                    self.spirv = Some(bytecode);
                }
            }
            None => {
                let bytecode = Self::compile(&self.pathinfo.0, &self.shader_type)?;
                self.spirv = Some(bytecode);
            }
        };

        Ok(self.spirv.as_ref().unwrap().clone())
    }

    pub fn outdated(&self) -> std::io::Result<bool> {
        let metadata = fs::metadata(&self.pathinfo.0)?;
        let last_modified = metadata.modified()?;
        let delta = SystemTime::now().duration_since(last_modified).unwrap();

        Ok(delta > Duration::from_millis(50))
    }

    // Helper functions

    fn compile(path: &Path, shader_type: &ShaderType) -> std::io::Result<SpirvBytecode> {
        let glsl = fs::read_to_string(path)?;
        let glsl_type = match shader_type {
            ShaderType::Vertex => glsl_to_spirv::ShaderType::Vertex,
            ShaderType::Fragment => glsl_to_spirv::ShaderType::Fragment,
        };

        let spirv: SpirvBytecode = glsl_to_spirv::compile(&glsl, glsl_type)
            .unwrap()
            .bytes()
            .map(|b| b.unwrap())
            .collect();

        Ok(spirv)
    }
}

struct ShaderPaths {
    vs: &'static str,
    fs: &'static str,
}

static SHADERS: &'static [(&'static str, ShaderPaths)] = &[(
    "simple_triangle",
    ShaderPaths {
        vs: "assets/shaders/simple_triangle/simple.vert.glsl",
        fs: "assets/shaders/simple_triangle/simple.frag.glsl",
    },
)];

pub struct ShaderManager {}

impl ShaderManager {
    pub fn from_name(name: &str, shader_type: ShaderType) -> Shader {
        let idx = SHADERS.binary_search_by(|e| e.0.cmp(name)).unwrap();
        let paths = &SHADERS[idx].1;
        match shader_type {
            ShaderType::Vertex => Shader::from_path(paths.vs.into(), ShaderType::Vertex),
            ShaderType::Fragment => Shader::from_path(paths.fs.into(), ShaderType::Fragment),
        }
    }
}
