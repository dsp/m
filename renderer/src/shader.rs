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
    pub fn from_path(path: PathBuf, shader_type: ShaderType) ->  Self {
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
            },
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

pub struct ShaderManager {
}

