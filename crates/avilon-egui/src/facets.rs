//! Convex faceted previews for egui. Geometry, lighting and motion are independent.

use std::collections::BTreeMap;
use std::fmt;

use egui::Color32;
mod view;
pub use view::View;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Light {
    #[default]
    Studio,
    Sweep,
    Pearl,
    Neon,
    Wave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Prism,
    Shard,
    Crystal,
    Seal,
    Trigon,
    Lens,
    Obelisk,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormError {
    InvalidProfile,
    InvalidVertices,
    InvalidFace(usize),
    NonConvex,
    OpenMesh,
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfile => f.write_str("expected at least two ascending finite rings, with positive radii except at the poles"),
            Self::InvalidVertices => f.write_str("expected at least four distinct finite vertices, all used by the mesh"),
            Self::InvalidFace(i) => write!(f, "face {i} must contain distinct valid indices around a strictly convex planar polygon"),
            Self::NonConvex => f.write_str("expected a convex solid with nonzero volume"),
            Self::OpenMesh => f.write_str("each edge must belong to exactly two faces"),
        }
    }
}

impl std::error::Error for FormError {}

#[derive(Clone, Debug)]
pub struct Form {
    vertices: Vec<Point3>,
    faces: Vec<Vec<usize>>,
    pigments: Vec<Color32>,
}

const PIGMENTS: [Color32; 6] = [
    Color32::from_rgb(39, 216, 208),
    Color32::from_rgb(67, 116, 247),
    Color32::from_rgb(142, 114, 245),
    Color32::from_rgb(49, 162, 228),
    Color32::from_rgb(87, 236, 205),
    Color32::from_rgb(85, 94, 201),
];

impl Form {
    /// Face indices run around each polygon in either winding. Winding is corrected automatically.
    /// Coordinates are centered and uniformly scaled to fit the preview.
    pub fn from_mesh(vertices: &[Point3], faces: &[&[usize]]) -> Result<Self, FormError> {
        let pigments = (0..faces.len())
            .map(|i| PIGMENTS[i % PIGMENTS.len()])
            .collect();
        Self::validate(
            vertices.to_vec(),
            faces.iter().map(|face| face.to_vec()).collect(),
            pigments,
            true,
        )
    }

    /// Builds a solid around the Y axis from (height, radius) rings, bottom first.
    /// A zero radius at either end creates a tip. The profile must be convex.
    pub fn from_profile(sides: usize, rings: &[(f32, f32)]) -> Result<Self, FormError> {
        Self::profile(sides, rings, true)
    }

    pub fn preset(preset: Preset) -> Self {
        let (sides, rings): (_, &[(f32, f32)]) = match preset {
            Preset::Prism => (
                6,
                &[(-1.18, 0.54), (-0.79, 0.94), (0.68, 1.0), (1.10, 0.63)],
            ),
            Preset::Crystal => (5, &[(-1.30, 0.0), (0.0, 0.99), (1.30, 0.0)]),
            Preset::Seal => (
                8,
                &[(-0.40, 0.81), (-0.24, 1.18), (0.24, 1.18), (0.40, 0.81)],
            ),
            Preset::Trigon => (3, &[(-0.91, 1.03), (0.91, 1.03)]),
            Preset::Lens => (
                10,
                &[
                    (-0.34, 0.38),
                    (-0.19, 0.91),
                    (0.0, 1.16),
                    (0.19, 0.91),
                    (0.34, 0.38),
                ],
            ),
            Preset::Obelisk => (
                4,
                &[(-1.21, 0.59), (-0.90, 0.76), (0.67, 0.80), (1.30, 0.0)],
            ),
            Preset::Shard => {
                return Self::validate(
                    vec![
                        [0.92, 0.92, 0.92],
                        [-0.92, -0.92, 0.92],
                        [-0.92, 0.92, -0.92],
                        [0.92, -0.92, -0.92],
                    ],
                    vec![vec![0, 1, 2], vec![0, 1, 3], vec![0, 2, 3], vec![1, 2, 3]],
                    PIGMENTS[..4].to_vec(),
                    false,
                )
                .expect("shard is convex");
            }
        };
        Self::profile(sides, rings, false).expect("preset profile is convex")
    }

    fn profile(sides: usize, rings: &[(f32, f32)], fit: bool) -> Result<Self, FormError> {
        if sides < 3
            || rings.len() < 2
            || rings.windows(2).any(|pair| pair[0].0 >= pair[1].0)
            || rings.iter().enumerate().any(|(i, &(y, r))| {
                !y.is_finite()
                    || !r.is_finite()
                    || r < 0.0
                    || (r == 0.0 && i != 0 && i + 1 != rings.len())
            })
        {
            return Err(FormError::InvalidProfile);
        }
        let mut vertices = Vec::new();
        let indices: Vec<Vec<usize>> = rings
            .iter()
            .map(|&(y, r)| {
                let count = if r == 0.0 { 1 } else { sides };
                let base = vertices.len();
                for i in 0..count {
                    let (sin, cos) = (i as f32 * std::f32::consts::TAU / sides as f32).sin_cos();
                    vertices.push([cos * r, y, sin * r]);
                }
                (base..base + count).collect()
            })
            .collect();
        let mut faces = Vec::new();
        let mut pigments = Vec::new();
        for pair in indices.windows(2) {
            let (lower, upper) = (&pair[0], &pair[1]);
            if lower.len() == 1 && upper.len() == 1 {
                return Err(FormError::InvalidProfile);
            }
            for i in 0..sides {
                let next = (i + 1) % sides;
                let face = if lower.len() == 1 {
                    vec![lower[0], upper[i], upper[next]]
                } else if upper.len() == 1 {
                    vec![lower[i], upper[0], lower[next]]
                } else {
                    vec![lower[i], upper[i], upper[next], lower[next]]
                };
                faces.push(face);
                pigments.push(PIGMENTS[i % PIGMENTS.len()]);
            }
        }
        for (ring, color) in [
            (&indices[0], Color32::from_rgb(43, 89, 155)),
            (
                &indices[indices.len() - 1],
                Color32::from_rgb(109, 226, 228),
            ),
        ] {
            if ring.len() > 1 {
                faces.push(ring.clone());
                pigments.push(color);
            }
        }
        Self::validate(vertices, faces, pigments, fit)
    }

    fn validate(
        mut vertices: Vec<Point3>,
        mut faces: Vec<Vec<usize>>,
        pigments: Vec<Color32>,
        fit: bool,
    ) -> Result<Self, FormError> {
        const EPS: f32 = 1e-5;
        if vertices.len() < 4 || vertices.iter().flatten().any(|v| !v.is_finite()) {
            return Err(FormError::InvalidVertices);
        }
        let mean: [f64; 3] = std::array::from_fn(|i| {
            vertices
                .iter()
                .map(|point| f64::from(point[i]))
                .sum::<f64>()
                / vertices.len() as f64
        });
        if fit {
            let extent = vertices
                .iter()
                .flat_map(|p| (0..3).map(move |i| (f64::from(p[i]) - mean[i]).abs()))
                .fold(0.0, f64::max);
            if extent == 0.0 {
                return Err(FormError::InvalidVertices);
            }
            for p in &mut vertices {
                for i in 0..3 {
                    p[i] = ((f64::from(p[i]) - mean[i]) / extent * 1.15) as f32;
                }
            }
        }
        if vertices
            .iter()
            .enumerate()
            .any(|(i, p)| vertices[..i].contains(p))
        {
            return Err(FormError::InvalidVertices);
        }
        let center = if fit {
            [0.0; 3]
        } else {
            mean.map(|v| v as f32)
        };
        let mut used = vec![false; vertices.len()];
        let mut edges: BTreeMap<_, (usize, i32)> = BTreeMap::new();
        for (index, face) in faces.iter_mut().enumerate() {
            let invalid = FormError::InvalidFace(index);
            if face.len() < 3
                || face
                    .iter()
                    .enumerate()
                    .any(|(i, &v)| v >= vertices.len() || face[..i].contains(&v))
            {
                return Err(invalid);
            }
            let origin = vertices[face[0]];
            let cross_plane = cross(
                sub(vertices[face[1]], origin),
                sub(vertices[face[2]], origin),
            );
            if dot(cross_plane, cross_plane) <= EPS * EPS {
                return Err(invalid);
            }
            let mut normal = normalized(cross_plane);
            let center_side = dot(normal, sub(center, origin));
            if center_side.abs() <= EPS {
                return Err(FormError::NonConvex);
            }
            if center_side > 0.0 {
                face.reverse();
                normal = scale(normal, -1.0);
            }
            if face
                .iter()
                .any(|&i| dot(normal, sub(vertices[i], origin)).abs() > EPS)
            {
                return Err(invalid);
            }
            if vertices.iter().any(|&v| dot(normal, sub(v, origin)) > EPS) {
                return Err(FormError::NonConvex);
            }
            for (&a, &b) in face.iter().zip(face.iter().cycle().skip(1)) {
                let edge = sub(vertices[b], vertices[a]);
                if face.iter().any(|&v| {
                    v != a
                        && v != b
                        && dot(normal, cross(edge, sub(vertices[v], vertices[a]))) <= EPS
                }) {
                    return Err(invalid);
                }
                used[a] = true;
                let record = edges.entry((a.min(b), a.max(b))).or_default();
                record.0 += 1;
                record.1 += if a < b { 1 } else { -1 };
            }
        }
        if edges
            .values()
            .any(|&(count, direction)| count != 2 || direction != 0)
        {
            return Err(FormError::OpenMesh);
        }
        if used.contains(&false) {
            return Err(FormError::InvalidVertices);
        }
        Ok(Self {
            vertices,
            faces,
            pigments,
        })
    }
}

pub type Point3 = [f32; 3];

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Point3, factor: f32) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn dot(a: Point3, b: Point3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalized(a: Point3) -> Point3 {
    scale(a, dot(a, a).max(f32::MIN_POSITIVE).sqrt().recip())
}

#[cfg(test)]
mod tests {
    use super::*;

    const POINTS: [Point3; 4] = [
        [0.42, 1.4, 0.18],
        [-1.0, -0.42, -0.68],
        [0.84, -0.34, -0.35],
        [-0.2, -0.73, 1.1],
    ];
    const FACES: [&[usize]; 4] = [&[0, 1, 2], &[0, 2, 3], &[0, 3, 1], &[1, 2, 3]];

    #[test]
    fn accepts_asymmetric_mesh_and_corrects_mixed_winding() {
        let mut faces = FACES;
        faces[2] = &[3, 0, 1];
        let form = Form::from_mesh(&POINTS, &faces).unwrap();
        for face in &form.faces {
            let [a, b, c] = [face[0], face[1], face[2]].map(|i| form.vertices[i]);
            assert!(dot(a, cross(sub(b, a), sub(c, a))) > 0.0);
        }

        let mut view = View::new(form).with_light(Light::Pearl);
        let mut frame = egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
            view.show(ui, egui::vec2(200.0, 200.0));
        });
        frame.textures_delta.clear();
        assert!(frame.shapes.iter().any(|item| matches!(&item.shape,
            egui::Shape::Mesh(mesh) if !mesh.indices.is_empty()
                && mesh.vertices.iter().all(|v| v.pos.x.is_finite() && v.pos.y.is_finite())
        )));
    }

    #[test]
    fn rejects_missing_faces_and_invalid_indices() {
        assert_eq!(
            Form::from_mesh(&POINTS, &FACES[..3]).unwrap_err(),
            FormError::OpenMesh
        );
        let mut faces = FACES;
        faces[1] = &[0, 9, 2];
        assert_eq!(
            Form::from_mesh(&POINTS, &faces).unwrap_err(),
            FormError::InvalidFace(1)
        );
    }

    #[test]
    fn all_presets_use_the_same_validated_geometry() {
        for preset in [
            Preset::Prism,
            Preset::Shard,
            Preset::Crystal,
            Preset::Seal,
            Preset::Trigon,
            Preset::Lens,
            Preset::Obelisk,
        ] {
            assert!(!Form::preset(preset).faces.is_empty());
        }
        assert!(Form::from_profile(6, &[(-1.0, 0.0), (0.0, 0.5), (1.0, 0.0)]).is_ok());
        assert_eq!(
            Form::from_profile(6, &[(-1.0, 1.0), (0.0, 0.3), (1.0, 1.0)]).unwrap_err(),
            FormError::NonConvex
        );
    }
}
