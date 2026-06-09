mod color;
mod geometry;
mod gltf_loader;
mod pipeline;
mod pmx;
mod scene;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use glam::Vec3A;
use minifb::{Key, Window, WindowOptions};

use crate::color::Color;
use crate::pipeline::{Accumulator, PathTracerConfig, Sampler, TraceScene};
use crate::scene::{Camera, DirectionalLight, PointLight, Scene, SceneObject, Transform};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

struct Args {
    pmx_path: Option<String>,
    spp: u32,
    sampler: Sampler,
    rr_depth: u32,
    clamp: f32,
    max_depth: u32,
    output: Option<String>,
    reference: bool,
    width: usize,
    height: usize,
    seed: u64,
    gbuffer: bool,
}

fn parse_args() -> Result<Args> {
    let cfg = PathTracerConfig::default();
    let mut args = Args {
        pmx_path: None,
        spp: 16,
        sampler: cfg.sampler,
        rr_depth: cfg.rr_start,
        clamp: cfg.indirect_clamp,
        max_depth: cfg.max_depth,
        output: None,
        reference: false,
        width: 800,
        height: 600,
        seed: 0,
        gbuffer: false,
    };

    let raw: Vec<String> = env::args().collect();
    let mut i = 1;

    // Helper: advance past the current flag and return the next arg,
    // or error if none remains.
    fn value<'a>(i: &mut usize, raw: &'a [String], flag: &str) -> Result<&'a str> {
        *i += 1;
        raw.get(*i)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("{flag}: expected a value"))
    }

    while i < raw.len() {
        match raw[i].as_str() {
            p if p.ends_with(".pmx") && !p.starts_with('-') => {
                args.pmx_path = Some(p.to_string());
            }
            "--pmx" => {
                args.pmx_path = Some(value(&mut i, &raw, "--pmx")?.to_string());
            }
            "--spp" => {
                args.spp = value(&mut i, &raw, "--spp")?.parse()?;
            }
            "--sampler" => {
                args.sampler = match value(&mut i, &raw, "--sampler")? {
                    "halton" => Sampler::Halton,
                    "jitter" => Sampler::Jitter,
                    other => anyhow::bail!("unknown sampler: {other} (use halton or jitter)"),
                };
            }
            "--rr-depth" => {
                args.rr_depth = value(&mut i, &raw, "--rr-depth")?.parse()?;
            }
            "--clamp" => {
                args.clamp = value(&mut i, &raw, "--clamp")?.parse::<f32>()?;
            }
            "--max-depth" => {
                args.max_depth = value(&mut i, &raw, "--max-depth")?.parse()?;
            }
            "--output" => {
                args.output = Some(value(&mut i, &raw, "--output")?.to_string());
            }
            "--reference" => {
                args.reference = true;
            }
            "--width" => {
                args.width = value(&mut i, &raw, "--width")?.parse()?;
            }
            "--height" => {
                args.height = value(&mut i, &raw, "--height")?.parse()?;
            }
            "--seed" => {
                args.seed = value(&mut i, &raw, "--seed")?.parse()?;
            }
            "--gbuffer" => {
                args.gbuffer = true;
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
        i += 1;
    }

    // --reference overrides
    if args.reference {
        args.spp = 8192;
        args.sampler = Sampler::Jitter;
        args.rr_depth = 0;
        args.clamp = 0.0;
    }

    Ok(args)
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn save_png(width: usize, height: usize, pixels: &[Color], path: &std::path::Path) -> Result<()> {
    let mut buf = Vec::with_capacity(width * height * 3);
    for c in pixels {
        let [r, g, b] = c.to_srgb_bytes();
        buf.extend_from_slice(&[r, g, b]);
    }
    image::save_buffer(path, &buf, width as u32, height as u32, image::ColorType::Rgb8)
        .context("save png")?;
    Ok(())
}

fn save_bin(width: usize, height: usize, floats: &[f32], path: &std::path::Path) -> Result<()> {
    let header = [width as u32, height as u32];
    let mut data = Vec::with_capacity(8 + floats.len() * 4);
    data.extend_from_slice(&header[0].to_le_bytes());
    data.extend_from_slice(&header[1].to_le_bytes());
    for f in floats {
        data.extend_from_slice(&f.to_le_bytes());
    }
    fs::write(path, &data)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let args = parse_args()?;
    let (width, height) = (args.width, args.height);

    let mut scene: Scene;

    if let Some(ref path) = args.pmx_path {
        scene = Scene::new(Camera::default());
        let submeshes = pmx::load_pmx(path)?;
        let obj = SceneObject::new(
            submeshes,
            Transform::default()
                .with_translation(Vec3A::new(0.0, -1.5, 3.0))
                .with_scale(Vec3A::splat(0.2)),
        );
        scene.add_object(obj);
        scene.add_light(Arc::new(DirectionalLight::new(
            Vec3A::new(-0.4, -0.7, -0.8),
            Color::WHITE,
            1.6,
        )));
        scene.add_light(Arc::new(PointLight::new(
            Vec3A::new(2.0, 3.0, 3.5),
            Color::WHITE,
            25.0,
        )));
    } else {
        // ── Cornell Box: official data, handiness-adjusted ────────────
        // Conversion:  x = -(X_official - 278)   (see convert_cornell.py)
        // Camera:  (278, 273, -800)_official  →  (0, 273, -800)_ours
        // FOV = 2·atan(12.5 / 35) ≈ 39.3°
        let cb_camera = Camera::new(
            Vec3A::new(0.0, 273.0, -800.0),
            glam::Quat::IDENTITY,
            0.686,
        );
        scene = Scene::new(cb_camera);

        let glb_bytes = include_bytes!("../models/cornell_box/cornell_box.glb");
        for obj in gltf_loader::load_glb(glb_bytes)? {
            scene.add_object(obj);
        }
        // Light: panel centre (278, 548.8, 279.5)_official → (0, 548.8, 279.5)_ours
        scene.add_light(Arc::new(PointLight::new(
            Vec3A::new(0.0, 548.8, 279.5),
            Color::rgb(255, 200, 130),
            200_000.0,
        )));
    }

    let cfg = PathTracerConfig {
        max_depth: args.max_depth,
        rr_start: args.rr_depth,
        indirect_clamp: args.clamp,
        sampler: args.sampler,
        seed: args.seed,
    };

    let ts = TraceScene::from_scene(&scene, width, height);
    let mut acc = Accumulator::new(width, height, args.max_depth);

    // --- G-buffer capture (before rendering) ----------------------------
    if args.gbuffer {
        acc.capture_first_hit(&ts);
    }

    // --- Headless mode (--output) --------------------------------------
    if let Some(ref prefix) = args.output {
        let t0 = Instant::now();
        acc.accumulate_with_config(&ts, &scene, args.spp, cfg);
        let elapsed = t0.elapsed();

        let image = acc.as_image();
        let floats = acc.as_float_buffer();

        let png_path = PathBuf::from(format!("{prefix}.png"));
        let bin_path = PathBuf::from(format!("{prefix}.bin"));

        save_png(width, height, &image, &png_path)?;
        save_bin(width, height, &floats, &bin_path)?;

        // G-buffer outputs
        if args.gbuffer {
            let pos_path = PathBuf::from(format!("{prefix}_first_hit_pos.bin"));
            let norm_path = PathBuf::from(format!("{prefix}_first_hit_normal.bin"));
            save_bin(width, height, &acc.first_hit_pos_buffer(), &pos_path)?;
            save_bin(width, height, &acc.first_hit_normal_buffer(), &norm_path)?;
            println!("  → {}", pos_path.display());
            println!("  → {}", norm_path.display());
        }

        // Save path-depth histogram for RR geometric-distribution modeling
        let csv_path = PathBuf::from(format!("{prefix}_depth.csv"));
        let mut csv = String::from("depth,count\n");
        for (d, a) in acc.depth_histogram.iter().enumerate() {
            let n = a.load(std::sync::atomic::Ordering::Relaxed);
            csv.push_str(&format!("{d},{n}\n"));
        }
        fs::write(&csv_path, &csv)?;

        println!(
            "rendered {}×{}  spp={}  sampler={:?}  rr_depth={}  clamp={:.1}  in {:.1}s",
            width,
            height,
            args.spp,
            args.sampler,
            args.rr_depth,
            args.clamp,
            elapsed.as_secs_f32()
        );
        println!("  → {}", png_path.display());
        println!("  → {}", bin_path.display());
        println!("  → {}", csv_path.display());
        return Ok(());
    }

    // --- Interactive mode (no --output) ---------------------------------
    let mut window =
        Window::new("draft-render — path tracing", width, height, WindowOptions::default())?;
    let mut last_frame = Instant::now();
    let mut total_spp: u64 = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        acc.accumulate_with_config(&ts, &scene, args.spp, cfg);
        total_spp += args.spp as u64;

        let elapsed = last_frame.elapsed().as_secs_f32();
        let fps = 1.0 / elapsed.max(0.001);
        last_frame = Instant::now();

        let image = acc.as_image();
        let data: Vec<u32> = image.iter().map(|c| c.to_u32()).collect();
        window.update_with_buffer(&data, width, height)?;
        window.set_title(&format!(
            "{}×{}  {}spp  {:?}  rr={}  clamp={:.0}  {fps:.0}fps",
            width, height, total_spp, args.sampler, args.rr_depth, args.clamp,
        ));
    }

    Ok(())
}
