use bevy::prelude::*;

type EarcutrIndices = Vec<usize>;
type EarcutrVertices<T> = Vec<T>;
type BevyIndices = Vec<u32>;
type BevyVertices = Vec<[f32; 3]>;

#[derive(Debug)]
pub struct EarcutrInput<T: num_traits::Float> {
    pub vertices: EarcutrVertices<T>,
    pub interior_indices: EarcutrIndices,
}

#[derive(Debug)]
pub struct EarcutrResult<T: num_traits::Float> {
    pub vertices: EarcutrVertices<T>,
    pub triangle_indices: EarcutrIndices,
}

impl<T: num_traits::Float> EarcutrResult<T> {
    fn merge(&mut self, mut other: EarcutrResult<T>) {
        let base_triangle_index = self.vertices.len() / 2;
        for other_triangle_index in other.triangle_indices {
            self.triangle_indices
                .push(other_triangle_index + base_triangle_index);
        }
        self.vertices.append(&mut other.vertices);
    }
}

pub struct PolygonMeshBuilder<T: num_traits::Float> {
    earcutr_inputs: Vec<EarcutrInput<T>>,
    z_index: T,
}

impl<T: num_traits::Float> PolygonMeshBuilder<T> {
    pub fn with_z_index(mut self, z_index: T) -> Self {
        self.z_index = z_index;
        self
    }

    /// Call for `add_earcutr_input` for each polygon you want to add to the mesh.
    pub fn add_earcutr_input(&mut self, earcutr_input: EarcutrInput<T>) {
        self.earcutr_inputs.push(earcutr_input);
    }

    pub fn build(self) -> Result<Mesh, Error> {
        let z_index = self.z_index;
        let result = self.run_earcutr()?;
        build_mesh_from_earcutr(result, z_index)
    }

    fn run_earcutr(self) -> Result<EarcutrResult<T>, Error> {
        let mut earcutr_inputs_iter = self.earcutr_inputs.into_iter();

        // Earcut the first polygon
        let first_input = earcutr_inputs_iter.next().ok_or(Error::EmptyInput)?;
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
                vertices,
            });
        }

        Ok(earcutr_result)
    }
}

impl<T: num_traits::Float> Default for PolygonMeshBuilder<T> {
    fn default() -> Self {
        PolygonMeshBuilder {
            earcutr_inputs: vec![],
            z_index: T::zero(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    EmptyInput,
    CouldNotConvertToF32,
}

pub fn build_mesh_from_earcutr<T: num_traits::Float>(
    earcutr_result: EarcutrResult<T>,
    z_index: T,
) -> Result<Mesh, Error> {
    let indices = earcutr_result
        .triangle_indices
        .into_iter()
        .map(|n| u32::try_from(n).unwrap())
        .collect::<Vec<_>>();
    let vertices = earcutr_result
        .vertices
        .chunks(2)
        .map(|n| {
            let x = n[0].to_f32().ok_or(Error::CouldNotConvertToF32)?;
            let y = n[1].to_f32().ok_or(Error::CouldNotConvertToF32)?;
            let z = z_index.to_f32().ok_or(Error::CouldNotConvertToF32)?;
            Ok([x, y, z])
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(build_mesh_from_bevy(indices, vertices))
}

fn build_mesh_from_bevy(triangle_indices: BevyIndices, vertices: BevyVertices) -> Mesh {
    let num_vertices = vertices.len();
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        Default::default(),
    );
    mesh.insert_indices(bevy::render::mesh::Indices::U32(triangle_indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    let normals = vec![[0.0, 0.0, 0.0]; num_vertices];
    let uvs = vec![[0.0, 0.0]; num_vertices];

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}
