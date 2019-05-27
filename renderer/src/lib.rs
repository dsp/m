mod shader;

use gfx_hal as hal;
use hal::Primitive;
use hal::{device::Device, format as f, image as i, pass, pso};
use shader::*;

const ENTRY_NAME: &str = "main";

pub struct RenderTask<'a, B>
where
    B: hal::Backend,
{
    device: &'a B::Device,
    format: f::Format,
    pub pipeline: <B as hal::Backend>::GraphicsPipeline,
    pub render_pass: <B as hal::Backend>::RenderPass,
    pub pipeline_layout: <B as hal::Backend>::PipelineLayout,
    vs: Shader,
    fs: Shader,
}

impl<'a, B: hal::Backend> RenderTask<'a, B> {
    pub fn destroy(task: Self) {
        unsafe {
            task.device.destroy_graphics_pipeline(task.pipeline);
            task.device.destroy_render_pass(task.render_pass);
            task.device.destroy_pipeline_layout(task.pipeline_layout);
        }
    }

    pub fn new(device: &'a B::Device, format: f::Format) -> Self {
        let color_attachment = pass::Attachment {
            format: Some(format),
            samples: 1,
            ops: pass::AttachmentOps::new(
                pass::AttachmentLoadOp::Clear,
                pass::AttachmentStoreOp::Store,
            ),
            stencil_ops: pass::AttachmentOps::DONT_CARE,
            layouts: i::Layout::Undefined..i::Layout::Present,
        };

        let subpass = pass::SubpassDesc {
            colors: &[(0, i::Layout::ColorAttachmentOptimal)],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };

        let dependency = pass::SubpassDependency {
            passes: pass::SubpassRef::External..pass::SubpassRef::Pass(0),
            stages: pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT
                ..pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            accesses: i::Access::empty()
                ..(i::Access::COLOR_ATTACHMENT_READ | i::Access::COLOR_ATTACHMENT_WRITE),
        };

        let mut vs = Shader::from_path(
            "assets/shaders/simple_triangle/simple.vert.glsl".into(),
            ShaderType::Vertex,
        );
        let mut fs = Shader::from_path(
            "assets/shaders/simple_triangle/simple.frag.glsl".into(),
            ShaderType::Fragment,
        );

        let render_pass = unsafe {
            device
                .create_render_pass(&[color_attachment], &[subpass], &[dependency])
                .expect("Can't create render pass")
        };

        let pipeline_layout = unsafe { device.create_pipeline_layout(&[], &[]).unwrap() };

        let vs_module = unsafe {
            device
                .create_shader_module(&vs.spirv().expect("can't load shader"))
                .unwrap()
        };

        let fs_module = unsafe {
            device
                .create_shader_module(&fs.spirv().expect("can't load shader"))
                .unwrap()
        };

        let pipeline = {
            let (vs_entry, fs_entry) = (
                pso::EntryPoint {
                    entry: ENTRY_NAME,
                    module: &vs_module,
                    specialization: Default::default(),
                },
                pso::EntryPoint {
                    entry: ENTRY_NAME,
                    module: &fs_module,
                    specialization: Default::default(),
                },
            );

            let shader_entries = pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let subpass = pass::Subpass {
                index: 0,
                main_pass: &render_pass,
            };

            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                shader_entries,
                Primitive::TriangleList,
                pso::Rasterizer::FILL,
                &pipeline_layout,
                subpass,
            );

            pipeline_desc.blender.targets.push(pso::ColorBlendDesc(
                pso::ColorMask::ALL,
                pso::BlendState::ALPHA,
            ));

            unsafe {
                device
                    .create_graphics_pipeline(&pipeline_desc, None)
                    .unwrap()
            }
        };

        Self {
            device,
            format,
            render_pass,
            pipeline_layout,
            pipeline,
            fs,
            vs,
        }
    }
}

pub struct Renderer {
    // owns the swapchain
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
