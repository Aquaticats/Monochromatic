//! Feasibility of the GPU code view in the native test harness:
//!  the rendering notifier fires on a snapshot,
//!  a Vello scene rendered into a wgpu texture on the window's own device appears in that snapshot,
//!  and Vello's transparent output blends as straight alpha,
//! as Slint's import of a wgpu texture expects.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::{install_gpu_platform, snapshot};
/// Slint's own wgpu types,
/// the ones the window's device is made of.
use slint::wgpu_30::wgpu;
/// The notifier,
/// the image import,
/// and the test component's handle.
use slint::{ComponentHandle, GraphicsAPI, Image, RenderingState};
/// Vello draws the scene into the texture.
use vello::{
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene,
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};

// The fixture is test-only: one image over a mid-grey window.
slint::slint! {
    export component TextureProbe inherits Window {
        preferred-width: 64px;
        preferred-height: 64px;
        background: #808080;
        in property <image> picture;
        Image {
            x: 0px;
            y: 0px;
            width: 64px;
            height: 64px;
            source: root.picture;
        }
    }
}

/// A Vello scene rendered from the notifier shows in the snapshot,
///  and its alpha blends as straight alpha over the window's background.
#[test]
fn vello_texture_appears_in_the_snapshot_with_straight_alpha() {
    install_gpu_platform();
    let window = TextureProbe::new().expect("probe window");
    let weak = window.as_weak();
    window
        .window()
        .set_rendering_notifier(move |state, api| {
            let GraphicsAPI::WGPU30 { device, queue, .. } = api else {
                panic!("the harness window offers a wgpu 30 device to the notifier");
            };
            if !matches!(state, RenderingState::RenderingSetup) {
                return;
            }
            let mut vello = Renderer::new(
                device,
                RendererOptions {
                    use_cpu: false,
                    antialiasing_support: AaSupport::area_only(),
                    num_init_threads: None,
                    pipeline_cache: None,
                },
            )
            .expect("vello renderer on the harness device");
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("probe"),
                size: wgpu::Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let mut scene = Scene::new();
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::from_rgba8(255, 0, 0, 255),
                None,
                &Rect::new(0.0, 0.0, 16.0, 16.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::from_rgba8(0, 0, 255, 128),
                None,
                &Rect::new(16.0, 0.0, 32.0, 16.0),
            );
            vello
                .render_to_texture(
                    device,
                    queue,
                    &scene,
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    &RenderParams {
                        base_color: Color::from_rgba8(0, 0, 0, 0),
                        width: 64,
                        height: 64,
                        antialiasing_method: AaConfig::Area,
                    },
                )
                .expect("vello render");
            if let Some(active) = weak.upgrade() {
                active.set_picture(Image::try_from(texture).expect("import the texture"));
            }
        })
        .expect("rendering notifier");
    window.show().expect("show probe");
    let frame = snapshot(window.window());
    let pixels = frame.as_slice();
    let at = |x: usize, y: usize| {
        let pixel = pixels[y * 64 + x];
        return [pixel.r, pixel.g, pixel.b, pixel.a];
    };
    println!(
        "opaque {:?} half {:?} untouched {:?}",
        at(8, 8),
        at(24, 8),
        at(48, 48)
    );
    assert_eq!(at(8, 8), [255, 0, 0, 255], "an opaque rectangle is exact");
    let half = at(24, 8);
    assert!(
        half[0].abs_diff(64) <= 2 && half[1].abs_diff(64) <= 2 && half[2].abs_diff(192) <= 2,
        "half-transparent blue over grey must blend as straight alpha to about (64, 64, 192): {half:?}"
    );
    assert_eq!(
        at(48, 48),
        [128, 128, 128, 255],
        "transparent texels leave the window background"
    );
    window.hide().expect("close probe");
}

/// Premultiplied difference of two straight RGBA texels,
/// the largest over the four channels.
fn premultiplied_difference(left: slint::Rgba8Pixel, right: slint::Rgba8Pixel) -> u8 {
    let premultiply =
        |value: u8, alpha: u8| return (u32::from(value) * u32::from(alpha) / 255) as u8;
    return premultiply(left.r, left.a)
        .abs_diff(premultiply(right.r, right.a))
        .max(premultiply(left.g, left.a).abs_diff(premultiply(right.g, right.a)))
        .max(premultiply(left.b, left.a).abs_diff(premultiply(right.b, right.a)))
        .max(left.a.abs_diff(right.a));
}

/// Two renders of one scene are not bit-identical on the GPU:
///  how far apart can they be?
#[test]
fn repeated_renders_of_one_tile_spread() {
    let text =
        "let value = compute_total(first_item, second_item, third_item); // word word word\n"
            .repeat(60);
    let (_directory, reader) = super::annotation_paint_tests::fixture_reader(&text);
    let first = super::tile_reads::tile_pixels(&reader.window, &reader.source).expect("tile");
    let (mut worst, mut differing) = (0u8, 0usize);
    for _ in 0..40 {
        reader.source.borrow_mut().frame_stamp = None;
        super::render(&reader.window, &reader.source);
        let again = super::tile_reads::tile_pixels(&reader.window, &reader.source).expect("tile");
        for (left, right) in first.as_slice().iter().zip(again.as_slice()) {
            let difference = premultiplied_difference(*left, *right);
            if difference > 0 {
                differing += 1;
            }
            worst = worst.max(difference);
        }
    }
    println!(
        "SPREAD {} x {} texels, 40 renders: {differing} differing texel readings, worst {worst}",
        first.width(),
        first.height()
    );
    assert!(
        worst <= super::tile_reads::RENDER_NOISE,
        "two renders of one scene differ by {worst} levels, more than the {} level the tests accept",
        super::tile_reads::RENDER_NOISE
    );
}
