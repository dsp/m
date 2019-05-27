#![cfg_attr(
    not(any(
        feature = "vulkan",
        // feature = "dx11",
        // feature = "dx12",
        // feature = "metal",
        feature = "gl"
    )),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

use env_logger;
// #[cfg(feature = "dx11")]
// use gfx_backend_dx11 as back;
// #[cfg(feature = "dx12")]
// use gfx_backend_dx12 as back;
#[cfg(feature = "gl")]
use gfx_backend_gl as back;
// #[cfg(feature = "metal")]
// use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use gfx_hal as hal;

// use glsl_to_spirv;
// use image;
use log::debug;
use renderer;
use winit;

use hal::format::{ChannelType, Swizzle};
use hal::pso::PipelineStage;
use hal::queue::Submission;
use hal::{command, format as f, image as i, pool, pso, window::Extent2D};
use hal::{Device, Instance, Surface, Swapchain};
use hal::{SwapchainConfig};

#[cfg_attr(rustfmt, rustfmt_skip)]
const DIMS: Extent2D = Extent2D { width: 800, height: 600 };

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Vertex {
    a_Pos: [f32; 2],
    a_Uv: [f32; 2],
}

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: f::Aspects::COLOR,
    levels: 0..1,
    layers: 0..1,
};

fn get_dimensions(_window: &winit::Window) -> Extent2D {
    DIMS
    // let dpi_factor = window.get_available_monitors().last().unwrap().get_hidpi_factor();
    // debug!("{:?}", dpi_factor);
    // let logical_size = window.get_outer_size().unwrap();
    // let physical = logical_size.to_physical(dpi_factor);
    // debug!("{:?}", physical);
    // Extent2D {
    // 	width: physical.width as _,
    // 	height: physical.height as _,
    // }
}


#[cfg(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal",
    feature = "gl"
))]
fn main() {
    env_logger::init();

    let mut events_loop = winit::EventsLoop::new();
    let wb = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(
            DIMS.width as _,
            DIMS.height as _,
        ))
        .with_transparency(false)
        .with_title("evemap".to_string());

    // instantiate backend
    #[cfg(not(feature = "gl"))]
    let (window, _instance, mut adapters, mut surface) = {
        let window = wb.build(&events_loop).unwrap();
        let instance = back::Instance::create("gfx-rs quad", 1);
        let surface = instance.create_surface(&window);
        debug!("surface: {:?}", surface);
        debug!("instance: {:?}", instance);
        let adapters = instance.enumerate_adapters();
        (window, instance, adapters, surface)
    };
    #[cfg(feature = "gl")]
    let (mut adapters, mut surface) = {
        let window = {
            let builder =
                back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
                    .with_vsync(true);
            back::glutin::WindowedContext::new_windowed(wb, builder, &events_loop).unwrap()
        };

        let surface = back::Surface::from_window(window);
        let adapters = surface.enumerate_adapters();
        (adapters, surface)
    };

    for adapter in &adapters {
        debug!("{:?}", adapter.info);
    }

    let mut adapter = adapters.remove(0);
    // let memory_types = adapter.physical_device.memory_properties().memory_types;
    // let limits = adapter.physical_device.limits();

    // Build a new device and associated command queues
    let (device, mut queue_group) = adapter
        .open_with::<_, hal::queue::capability::Graphics>(1, |family| {
            surface.supports_queue_family(family)
        })
        .unwrap();

    let mut command_pool = unsafe {
        device.create_command_pool_typed(&queue_group, pool::CommandPoolCreateFlags::empty())
    }
    .expect("Can't create command pool");

    let (caps, formats, _present_modes) = surface.compatibility(&mut adapter.physical_device);

    debug!("formats: {:?}", formats);
    debug!("capabilities: {:?}", caps);

    let format = match formats {
        Some(choices) => choices
            .into_iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .unwrap(),
        None => f::Format::Rgba8Srgb,
    };

    let task = renderer::RenderTask::<gfx_backend_vulkan::Backend>::new(&device, format);

    // Initialize our swapchain, images, framebuffers, etc.
    // We expect to have to rebuild these when the window is resized -
    // however we're going to ignore that for this example.

    // A swapchain is effectively a chain of images (commonly two) that will be
    // displayed to the screen. While one is being displayed, we can draw to one
    // of the others.
    //
    // In a rare instance of the API creating resources for you, the backbuffer
    // contains the actual images that make up the swapchain. We'll create image
    // views and framebuffers from these next.
    //
    // We also want to store the swapchain's extent, which tells us how big each
    // image is.
    let swapconfig = SwapchainConfig::from_caps(&caps, format, get_dimensions(&window));
    let extent = swapconfig.extent.to_extent();
    debug!("extent: {:?}", extent);

    let (mut swapchain, backbuffer) =
        unsafe { device.create_swapchain(&mut surface, swapconfig, None) }
            .expect("Can't create swapchain");

    debug!("{:?}", backbuffer);

    // You can think of an image as just the raw binary of the literal image, with
    // additional metadata about the format.
    //
    // Accessing the image must be done through an image view - which is more or
    // less a sub-range of the base image. For example, it could be one 2D slice of
    // a 3D texture. In many cases, the view will just be of the whole image. You
    // can also use an image view to swizzle or reinterpret the image format, but
    // we don't need to do any of this right now.
    //
    // Framebuffers bind certain image views to certain attachments. So for example,
    // if your render pass requires one color, and one depth, attachment - the
    // framebuffer chooses specific image views for each one.
    //
    // Here we create an image view and a framebuffer for each image in our
    // swapchain.
    let (frameviews, framebuffers) = {
        let pairs = backbuffer
            .into_iter()
            .map(|image| unsafe {
                let rtv = device
                    .create_image_view(
                        &image,
                        i::ViewKind::D2,
                        format,
                        Swizzle::NO,
                        COLOR_RANGE.clone(),
                    )
                    .unwrap();
                (image, rtv)
            })
            .collect::<Vec<_>>();
        let fbos = pairs
            .iter()
            .map(|&(_, ref rtv)| unsafe {
                device
                    .create_framebuffer(&task.render_pass, Some(rtv), extent)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        (pairs, fbos)
    };

    // The frame semaphore is used to allow us to wait for an image to be ready
    // before attempting to draw on it,
    //
    // The frame fence is used to to allow us to wait until our draw commands have
    // finished before attempting to display the image.
    let frame_semaphore = device.create_semaphore().unwrap();
    let present_semaphore = device.create_semaphore().unwrap();

    loop {
        let mut quitting = false;

        // If the window is closed, or Escape is pressed, quit
        events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event, .. } = event {
                match event {
                    // IF RESIZE WE NEED TO RECREATE SWAPCHAIN, SEE:
                    // https://github.com/gfx-rs/gfx/blob/master/examples/quad/main.rs#L660
                    winit::WindowEvent::CloseRequested => quitting = true,
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => quitting = true,
                    _ => {}
                }
            }
        });

        if quitting {
            break;
        }

        unsafe {
            command_pool.reset();
        }

        // A swapchain contains multiple images - which one should we draw on? This
        // returns the index of the image we'll use. The image may not be ready for
        // rendering yet, but will signal frame_semaphore when it is.
        let frame_index = unsafe {
            match swapchain.acquire_image(!0, Some(&frame_semaphore), None) {
                Ok((i, _)) => i as usize,
                Err(_) => {
                    // recreate_swapchain = true;
                    continue;
                }
            }
        };

        // We have to build a command buffer before we send it off to draw.
        // We don't technically have to do this every frame, but if it needs to
        // change every frame, then we do.
        let finished_command_buffer = {
            let mut command_buffer = command_pool.acquire_command_buffer::<command::OneShot>();

            // Define a rectangle on screen to draw into.
            // In this case, the whole screen.
            let viewport = pso::Viewport {
                rect: pso::Rect {
                    x: 0,
                    y: 0,
                    w: extent.width as i16,
                    h: extent.height as i16,
                },
                depth: 0.0..1.0,
            };

            unsafe {
                command_buffer.begin();

                command_buffer.set_viewports(0, &[viewport.clone()]);
                command_buffer.set_scissors(0, &[viewport.rect]);

                // Choose a pipeline to use.
                command_buffer.bind_graphics_pipeline(&task.pipeline);

                {
                    // Clear the screen and begin the render pass.
                    let mut encoder = command_buffer.begin_render_pass_inline(
                        &task.render_pass,
                        &framebuffers[frame_index],
                        viewport.rect,
                        &[command::ClearValue::Color(command::ClearColor::Float([
                            0.0, 0.0, 0.0, 1.0,
                        ]))],
                    );

                    // Draw some geometry! In this case 0..3 means that we're drawing
                    // the range of vertices from 0 to 3. We have no vertex buffer so
                    // this really just tells our shader to draw one triangle. The
                    // specific vertices to draw are encoded in the vertex shader which
                    // you can see in `source_assets/shaders/part00.vert`.
                    //
                    // The 0..1 is the range of instances to draw. It's not relevant
                    // unless you're using instanced rendering.
                    encoder.draw(0..3, 0..1);
                }

                // Finish building the command buffer - it's now ready to send to the
                // GPU.
                command_buffer.finish()
            }

            command_buffer
        };

        // This is what we submit to the command queue. We wait until frame_semaphore
        // is signalled, at which point we know our chosen image is available to draw
        // on.
        let submission = Submission {
            command_buffers: Some(&finished_command_buffer),
            wait_semaphores: Some((&frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)),
            signal_semaphores: Some(&present_semaphore),
        };

        // We submit the submission to one of our command queues, which will signal
        // frame_fence once rendering is completed.
        unsafe {
            queue_group.queues[0].submit(submission, None);

            // We first wait for the rendering to complete...
            // TODO: Fix up for semaphores

            // ...and then present the image on screen!
            swapchain
                .present(
                    &mut queue_group.queues[0],
                    frame_index as u32,
                    vec![&present_semaphore],
                )
                .expect("Present failed");
        }
    }

    device.wait_idle().unwrap();
    unsafe {
        device.destroy_semaphore(frame_semaphore);
        device.destroy_semaphore(present_semaphore);
        device.destroy_command_pool(command_pool.into_raw());
        renderer::RenderTask::destroy(task);
        for framebuffer in framebuffers {
            device.destroy_framebuffer(framebuffer);
        }
        for (_, rtv) in frameviews {
            device.destroy_image_view(rtv);
        }

        device.destroy_swapchain(swapchain);
    }
}

#[cfg(not(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal",
    feature = "gl"
)))]
fn main() {
    println!("You need to enable the native API feature (vulkan/metal) in order to test the LL");
}
