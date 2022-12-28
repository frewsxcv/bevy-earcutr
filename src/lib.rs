use bevy_render::prelude::*;
use std::convert::TryFrom;

type EarcutrIndices = Vec<usize>;
type EarcutrVertices = Vec<f64>;
type BevyIndices = Vec<u32>;
type BevyVertices = Vec<[f32; 3]>;

#[derive(Debug)]
pub struct EarcutrInput {
    pub vertices: EarcutrVertices,
    pub interior_indices: EarcutrIndices,
}

#[derive(Debug)]
pub struct EarcutrResult {
    pub vertices: EarcutrVertices,
    pub triangle_indices: EarcutrIndices,
}

impl EarcutrResult {
    fn merge(&mut self, mut other: EarcutrResult) {
        let base_triangle_index = self.vertices.len() / 2;
        for other_triangle_index in other.triangle_indices {
            self.triangle_indices
                .push(other_triangle_index + base_triangle_index);
        }
        self.vertices.append(&mut other.vertices);
    }
}

pub struct PolygonMeshBuilder {
    earcutr_inputs: Vec<EarcutrInput>,
    z_index: f32,
}

impl PolygonMeshBuilder {
    pub fn new() -> Self {
        PolygonMeshBuilder {
            earcutr_inputs: vec![],
            z_index: 0.,
        }
    }

    pub fn with_z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Call for `add_earcutr_input` for each polygon you want to add to the mesh.
    pub fn add_earcutr_input(&mut self, earcutr_input: EarcutrInput) {
        self.earcutr_inputs.push(earcutr_input);
    }

    pub fn build(self) -> Option<Mesh> {
        let z_index = self.z_index;
        let result = self.run_earcutr()?;
        Some(build_mesh_from_earcutr(result, z_index))
    }

    fn run_earcutr(self) -> Option<EarcutrResult> {
        let mut earcutr_inputs_iter = self.earcutr_inputs.into_iter();

        // Earcut the first polygon
        let first_input = earcutr_inputs_iter.next()?;
        let first_triangle_indices =
            earcutr::earcut(&first_input.vertices, &first_input.interior_indices, 2).unwrap();
        let mut earcutr_result = EarcutrResult {
            triangle_indices: first_triangle_indices,
            vertices: first_input.vertices,
        };

        // Earcut any additional polygons and merge the results into the result of the first polygon
        for earcutr_input in earcutr_inputs_iter {
            let EarcutrInput {
                vertices,
                interior_indices,
            } = earcutr_input;
            let next_earcutr_result = earcutr::earcut(&vertices, &interior_indices, 2).unwrap();
            earcutr_result.merge(EarcutrResult {
                triangle_indices: next_earcutr_result,
                vertices: vertices,
            });
        }

        Some(earcutr_result)
    }
}

pub fn build_mesh_from_earcutr(earcutr_result: EarcutrResult, z_index: f32) -> Mesh {
    let indices = earcutr_result
        .triangle_indices
        .into_iter()
        .map(|n| u32::try_from(n).unwrap())
        .collect::<Vec<_>>();
    let vertices = earcutr_result
        .vertices
        .chunks(2)
        .map(|n| [n[0] as f32, n[1] as f32, z_index])
        .collect::<Vec<_>>();
    build_mesh_from_bevy(indices, vertices)
}

fn build_mesh_from_bevy(triangle_indices: BevyIndices, vertices: BevyVertices) -> Mesh {
    let num_vertices = vertices.len();
    let mut mesh = Mesh::new(bevy_render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy_render::mesh::Indices::U32(triangle_indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    let normals = vec![[0.0, 0.0, 0.0]; num_vertices];
    let uvs = vec![[0.0, 0.0]; num_vertices];

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}
