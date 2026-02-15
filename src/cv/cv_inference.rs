use image::{DynamicImage, ImageBuffer, Rgba};
use image::imageops::FilterType;
use ndarray::{Array4, Axis};
use ort::{inputs, session::Session, value::TensorRef};
use raqote::{
    DrawOptions, DrawTarget, LineJoin, PathBuilder,
    SolidSource, Source, StrokeStyle,
};

use super::InfType;

pub const INF_WIDTH: usize = 192;
pub const INF_HEIGHT: usize = 256;

const CONFIDENCE_THRESHOLD: f32 = 0.05;

// COCO skeleton
const SKELETON: &[(usize, usize)] = &[
    (0, 1), (0, 2), (1, 3), (2, 4),
    (0, 5), (0, 6),
    (5, 7), (7, 9),
    (6, 8), (8, 10),
    (5, 11), (6, 12),
    (11, 12),
    (11, 13), (13, 15),
    (12, 14), (14, 16),
];

const KEEP_KEYPOINTS: [usize; 5] = [0, 5, 6, 7, 8];

type Keypoints = [Option<(f32, f32, f32)>; 17];

#[derive(Debug)]
pub struct Model {
    session: Session,
}

impl Model {
    pub fn new<P: AsRef<std::path::Path>>(model_path: P) -> ort::Result<Self> {
        let session = Session::builder()?
            .commit_from_file(model_path)?;
        Ok(Self { session })
    }

    pub fn process_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        _data_type: InfType
    ) -> ort::Result<Vec<u8>> {
        // ----- Wrap RGBA bytes into an image -----
        let img = DynamicImage::ImageRgba8(
            ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba.to_vec())
                .expect("Invalid RGBA buffer"),
        );

        // ----- Preprocess -----
        let input = preprocess_image(&img);

        // ----- Inference -----
        let outputs = self.session.run(
            inputs!["input" => TensorRef::from_array_view(&input)?]
        )?;

        let heatmaps = outputs["output"]
            .try_extract_array::<f32>()?
            .into_owned()
            .into_dimensionality::<ndarray::Ix4>()
            .unwrap();

        // ----- Decode -----
        let keypoints = decode_pose_fast(&heatmaps, width, height);

        // ----- Draw -----
        let mut dt = DrawTarget::new(width as i32, height as i32);
        draw_skeleton(&mut dt, &keypoints);

        // ----- Extract RGBA back out -----
        let data = dt.get_data();
        let mut out = Vec::with_capacity((width * height * 4) as usize);

        for px in data {
            out.push((px >> 16) as u8); // R
            out.push((px >> 8) as u8);  // G
            out.push(*px as u8);        // B
            out.push((px >> 24) as u8); // A
        }

        Ok(out)
    }
}

/* ----------------- helpers ----------------- */

fn preprocess_image(img: &DynamicImage) -> Array4<f32> {
    let resized = img
        .resize_exact(INF_WIDTH as u32, INF_HEIGHT as u32, FilterType::CatmullRom)
        .to_rgb8();

    let raw = resized.as_raw();
    let mut input = Array4::<f32>::zeros((1, 3, INF_HEIGHT, INF_WIDTH));
    let out = input.as_slice_mut().unwrap();

    let hw = INF_HEIGHT * INF_WIDTH;
    let scale = 1.0 / 255.0;

    for i in 0..hw {
        out[i] = raw[i * 3] as f32 * scale;
        out[hw + i] = raw[i * 3 + 1] as f32 * scale;
        out[2 * hw + i] = raw[i * 3 + 2] as f32 * scale;
    }

    input
}

fn decode_pose_fast(
    heatmaps: &Array4<f32>,
    orig_w: u32,
    orig_h: u32,
) -> Keypoints {
    let mut keypoints: Keypoints = [None; 17];
    let maps = heatmaps.index_axis(Axis(0), 0);

    let hm_h = maps.len_of(Axis(1));
    let hm_w = maps.len_of(Axis(2));

    let scale_x = orig_w as f32 / INF_WIDTH as f32;
    let scale_y = orig_h as f32 / INF_HEIGHT as f32;

    for &k in &KEEP_KEYPOINTS {
        let map = maps.index_axis(Axis(0), k);
        let slice = map.as_slice().unwrap();

        let (mut max_val, mut max_idx) = (f32::MIN, 0);
        for (i, &v) in slice.iter().enumerate() {
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }

        let y = max_idx / hm_w;
        let x = max_idx % hm_w;

        let x_img = (x as f32 * INF_WIDTH as f32 / hm_w as f32) * scale_x;
        let y_img = (y as f32 * INF_HEIGHT as f32 / hm_h as f32) * scale_y;

        keypoints[k] = Some((x_img, y_img, max_val));
    }

    keypoints
}

fn draw_skeleton(dt: &mut DrawTarget, keypoints: &Keypoints) {
    for &(i, j) in SKELETON {
        if let (Some((x1, y1, c1)), Some((x2, y2, c2))) =
            (keypoints[i], keypoints[j])
        {
            if c1 < CONFIDENCE_THRESHOLD || c2 < CONFIDENCE_THRESHOLD {
                continue;
            }

            let mut pb = PathBuilder::new();
            pb.move_to(x1, y1);
            pb.line_to(x2, y2);

            dt.stroke(
                &pb.finish(),
                &Source::Solid(SolidSource { r: 255, g: 0, b: 0, a: 255 }),
                &StrokeStyle {
                    width: 2.0,
                    join: LineJoin::Round,
                    ..Default::default()
                },
                &DrawOptions::new(),
            );
        }
    }

    for &(x, y, c) in keypoints.iter().flatten() {
        if c < CONFIDENCE_THRESHOLD {
            continue;
        }

        let mut pb = PathBuilder::new();
        pb.arc(x, y, 4.0, 0.0, std::f32::consts::TAU);

        dt.fill(
            &pb.finish(),
            &Source::Solid(SolidSource { r: 0, g: 255, b: 0, a: 255 }),
            &DrawOptions::new(),
        );
    }
}