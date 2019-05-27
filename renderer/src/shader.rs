use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

type SpirvBytecode = Vec<u8>;

pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct Shader {
    spirv: SpirvBytecode,
    shader_type: ShaderType,
    path: Option<PathBuf>,
}

impl Shader {
    pub fn new(spirv: SpirvBytecode, shader_type: ShaderType) -> Self {
        Self { spirv, shader_type, path: None }
    }

    pub fn from_path(path: PathBuf, shader_type: ShaderType) -> std::io::Result<Self> {
        Ok(Self {
            spirv: Self::compile(&path, &shader_type)?,
            shader_type: shader_type,
            path: Some(path),
        })
    }

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
               
    pub fn spirv(&mut self) -> std::io::Result<SpirvBytecode> {
        if self.must_reload() {
            if let Some(path) = &self.path {
                self.spirv = Self::compile(&path, &self.shader_type)?;
            }
        }

        Ok(self.spirv.clone())
    }

    fn must_reload(&self) -> bool {
        false
    }
}
