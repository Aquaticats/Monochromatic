//! Reading a texture back from the GPU,
//! for the native tests:
//!  the tile alone with its alpha,
//!  and a scene rendered without a window around it.

/// The tile scene that is drawn into a texture,
///  the scenes of a stack,
///  and the uses a texture of the tile has.
use super::{Inner, TileScene, TileStack};
/// Slint's own wgpu types,
/// the ones the window's device is made of.
use slint::wgpu_30::wgpu;
/// Vello's renderer and its settings.
use vello::{
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, peniko::Color as VelloColor,
};

/// What:
///  Copy `texture`,
/// `size` physical pixels,
/// into a buffer on the GPU and read it as straight RGBA,
///  row by row from the top.
/// Why:
///  This is a GPU readback like the window snapshot:
///  the pixels are filled by the GPU and only copied out for the test to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTexture(device: GPUDevice, queue: GPUQueue, texture: GPUTexture, size: [number, number]): Pixels;
/// ```
pub(super) fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    let (width, height) = size;
    let unpadded = width * 4;
    let padded =
        unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ide tile readback"),
        size: u64::from(padded) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));
    let slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the GPU finishes the readback");
    receiver
        .recv()
        .expect("the readback callback ran")
        .expect("the readback buffer maps");
    let mapped = slice
        .get_mapped_range()
        .expect("the mapped readback buffer");
    let mut pixels = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let bytes = pixels.make_mut_bytes();
    for (row, source) in mapped.chunks(padded as usize).enumerate() {
        let start = row * unpadded as usize;
        bytes[start..start + unpadded as usize].copy_from_slice(&source[..unpadded as usize]);
    }
    return pixels;
}

/// What:
///  Render `scene` on the test thread's device,
/// without a window,
/// and read the texture back as straight RGBA.
/// Why:
///  The pictures of a layout at scales a window cannot be set to,
///  and the edges of a clip,
///  are looked at without a window around them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderScene(scene: TileScene): Pixels;
/// ```
pub(in crate::native) fn render_scene(
    scene: &TileScene,
) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    return render_scenes(&[scene]);
}

/// What:
///  Render the scenes of `stack` on the test thread's device,
/// without a window,
/// each into a texture of its own,
/// and read the textures back as straight RGBA put one below the other.
/// Why:
///  The stack a window shows is looked at without a window around it,
///  and it is compared with the one texture that holds the same rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderStack(stack: TileStack): Pixels;
/// ```
pub(in crate::native) fn render_stack(
    stack: &TileStack,
) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    let mut scenes = Vec::with_capacity(stack.textures.len());
    for texture in &stack.textures {
        scenes.push(texture);
    }
    return render_scenes(&scenes);
}

/// Render each of `scenes` into a texture of its own with one renderer,
/// and read them back one below the other;
/// the scenes are one width.
fn render_scenes(scenes: &[&TileScene]) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    let (device, queue) = crate::native::gpu_snapshot::device_handles();
    let mut renderer = Renderer::new(
        &device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    )
    .expect("the GPU text renderer starts");
    let width = scenes.first().map_or(0, |scene| return scene.width);
    let mut height = 0;
    for scene in scenes {
        assert_eq!(scene.width, width, "the scenes of a stack are one width");
        height += scene.height;
    }
    let mut stacked = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let mut row = 0;
    for scene in scenes {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ide scene probe"),
            size: wgpu::Extent3d {
                width: scene.width,
                height: scene.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: Inner::usage(),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        renderer
            .render_to_texture(
                &device,
                &queue,
                &scene.scene,
                &view,
                &RenderParams {
                    base_color: VelloColor::from_rgba8(0, 0, 0, 0),
                    width: scene.width,
                    height: scene.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .expect("the GPU renders the scene");
        let part = read_texture(&device, &queue, &texture, (scene.width, scene.height));
        let start = row as usize * width as usize;
        let slots = stacked.make_mut_slice();
        slots[start..start + part.as_slice().len()].copy_from_slice(part.as_slice());
        row += scene.height;
    }
    return stacked;
}
