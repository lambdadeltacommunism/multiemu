use arrayvec::ArrayVec;
use egui::FullOutput;
use egui::TextureId;
use nalgebra::{DMatrix, DMatrixViewMut, Point2, Vector2, Vector3};
use palette::{blend::Compose, LinSrgba, Srgba};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
struct EguiVertex {
    pos: Point2<f32>,
    uv: Point2<f32>,
    color: Srgba<u8>,
}

#[derive(Debug, Default)]
pub struct SoftwareEguiRenderer {
    textures: HashMap<TextureId, DMatrix<Srgba<u8>>>,
}

impl SoftwareEguiRenderer {
    pub fn render(
        &mut self,
        context: &egui::Context,
        mut render_buffer: DMatrixViewMut<Srgba<u8>>,
        full_output: FullOutput,
    ) {
        for (new_texture_id, new_texture) in full_output.textures_delta.set {
            tracing::debug!("Adding new egui texture {:?}", new_texture_id);

            if new_texture.pos.is_some() && !self.textures.contains_key(&new_texture_id) {
                panic!("Texture not found: {:?}", new_texture_id);
            }

            let texture = self.textures.entry(new_texture_id).or_insert_with(|| {
                let image_size = new_texture.image.size();
                DMatrix::from_element(image_size[0], image_size[1], Srgba::new(0, 0, 0, 0xff))
            });

            let source_texture_view = match &new_texture.image {
                egui::ImageData::Color(image) => {
                    let converted_image = image
                        .pixels
                        .clone()
                        .into_iter()
                        .map(|pixel| Srgba::from_components(pixel.to_tuple()))
                        .collect();

                    DMatrix::from_vec(image.size[0], image.size[1], converted_image)
                }
                egui::ImageData::Font(font_image) => {
                    let converted_image = font_image
                        .pixels
                        .clone()
                        .into_iter()
                        .map(|coverage| {
                            Srgba::from_linear(LinSrgba::new(
                                coverage, coverage, coverage, coverage,
                            ))
                        })
                        .collect();

                    DMatrix::from_vec(font_image.size[0], font_image.size[1], converted_image)
                }
            };

            let texture_update_offset = Vector2::from(new_texture.pos.unwrap_or([0, 0]));

            let mut destination_texture_view = texture.view_range_mut(
                texture_update_offset.x
                    ..(texture_update_offset.x + source_texture_view.nrows()).min(texture.nrows()),
                texture_update_offset.y
                    ..(texture_update_offset.y + source_texture_view.ncols()).min(texture.ncols()),
            );

            destination_texture_view.copy_from(&source_texture_view);
        }

        for remove_texture_id in full_output.textures_delta.free {
            tracing::trace!("Freeing egui texture {:?}", remove_texture_id);
            self.textures.remove(&remove_texture_id);
        }

        render_buffer.fill(Srgba::new(0, 0, 0, 0xff));

        for shape in context.tessellate(full_output.shapes, full_output.pixels_per_point) {
            match shape.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    let texture = self.textures.get(&mesh.texture_id).unwrap();

                    for vertex_indexes in mesh.indices.chunks(3) {
                        let vertexes: ArrayVec<_, 3> = vertex_indexes
                            .iter()
                            .map(|&index| {
                                let vertex = mesh.vertices[index as usize];

                                EguiVertex {
                                    pos: Point2::new(vertex.pos.x, vertex.pos.y),
                                    uv: Point2::new(vertex.uv.x, vertex.uv.y),
                                    color: Srgba::from_components(vertex.color.to_tuple()),
                                }
                            })
                            .collect();

                        if let [v0, v1, v2] = vertexes.as_slice() {
                            let min_x =
                                v0.pos.x.min(v1.pos.x).min(v2.pos.x).max(0.0).floor() as usize;
                            let min_y =
                                v0.pos.y.min(v1.pos.y).min(v2.pos.y).max(0.0).floor() as usize;
                            let max_x = v0
                                .pos
                                .x
                                .max(v1.pos.x)
                                .max(v2.pos.x)
                                .min(render_buffer.nrows() as f32 - 1.0)
                                .ceil() as usize;
                            let max_y = v0
                                .pos
                                .y
                                .max(v1.pos.y)
                                .max(v2.pos.y)
                                .min(render_buffer.ncols() as f32 - 1.0)
                                .ceil() as usize;

                            for x in min_x..=max_x {
                                for y in min_y..=max_y {
                                    let pixel_center = Point2::new(x as f32 + 0.5, y as f32 + 0.5);

                                    if is_point_in_triangle(pixel_center, [v0.pos, v1.pos, v2.pos])
                                    {
                                        // Interpolate colors based on barycentric coordinates
                                        let barycentric = barycentric_coordinates(
                                            pixel_center,
                                            [v0.pos, v1.pos, v2.pos],
                                        );

                                        if barycentric.iter().all(|b| b.is_sign_positive()) {
                                            let interpolated_color = v0.color.into_linear()
                                                * barycentric.x
                                                + v1.color.into_linear() * barycentric.y
                                                + v2.color.into_linear() * barycentric.z;

                                            let interpolated_uv = v0.uv.coords * barycentric.x
                                                + v1.uv.coords * barycentric.y
                                                + v2.uv.coords * barycentric.z;

                                            let pixel_coords = Point2::new(
                                                (texture.nrows() as f32 * interpolated_uv.x)
                                                    as usize,
                                                (texture.ncols() as f32 * interpolated_uv.y)
                                                    as usize,
                                            );

                                            // Inaccuraries that lead outside the texture we will read off with black
                                            let pixel = texture
                                                .get((pixel_coords.x, pixel_coords.y))
                                                .copied()
                                                .unwrap_or(Srgba::new(0, 0, 0, 0xff));

                                            render_buffer[(x, y)] = Srgba::from_linear(
                                                (interpolated_color * pixel.into_linear())
                                                    .over(render_buffer[(x, y)].into_linear()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                egui::epaint::Primitive::Callback(_) => {
                    unimplemented!()
                }
            }
        }
    }
}

#[inline]
fn triangle_area(v: [Point2<f32>; 3]) -> f32 {
    0.5 * ((v[1].x - v[0].x) * (v[2].y - v[0].y) - (v[2].x - v[0].x) * (v[1].y - v[0].y)).abs()
}

#[inline]
fn barycentric_coordinates(point: Point2<f32>, v: [Point2<f32>; 3]) -> Vector3<f32> {
    let area = Vector3::from_element(triangle_area(v));
    let area1 = triangle_area([point, v[1], v[2]]);
    let area2 = triangle_area([v[0], point, v[2]]);
    let area3 = triangle_area([v[0], v[1], point]);

    Vector3::new(area1, area2, area3).component_div(&area)
}

#[inline]
fn is_point_in_triangle(point: Point2<f32>, v: [Point2<f32>; 3]) -> bool {
    let b = Vector3::new(
        (v[1].x - v[0].x) * (point.y - v[0].y) - (v[1].y - v[0].y) * (point.x - v[0].x),
        (v[2].x - v[1].x) * (point.y - v[1].y) - (v[2].y - v[1].y) * (point.x - v[1].x),
        (v[0].x - v[2].x) * (point.y - v[2].y) - (v[0].y - v[2].y) * (point.x - v[2].x),
    );

    b.iter().all(|&p| p >= 0.0) || b.iter().all(|&p| p <= 0.0)
}
