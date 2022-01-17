use ndarray::{Array, Array3};
// use rayon::prelude::*;
// use rayon::vec::IntoIter;
use std::convert::From;
use std::ops::{Add, Mul};
use std::vec::IntoIter;
use crate::voxel::Voxel;
use crate::compose;
use crate::compose::compose_two;
use crate::iso_field_generator::ScalarField;

type Indexes = (usize, usize, usize);
type Mask = (usize, usize, usize);

const MASKS: [Mask; 8] = [
    (0, 0, 0),
    (1, 0, 0),
    (1, 0, 1),
    (0, 0, 1),
    (0, 1, 0),
    (1, 1, 0),
    (1, 1, 1),
    (0, 1, 1),
];

#[derive(Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl From<(Indexes, Mask)> for Vertex {
    fn from(i: (Indexes, Mask)) -> Self {
        Vertex {
            x: i.0.0.mul(2).add(i.1.0) as f32,
            y: i.0.1.mul(2).add(i.1.1) as f32,
            z: i.0.2.mul(2).add(i.1.2) as f32,
        }
    }
}

#[derive(Debug)]
pub struct Corner {
    vertex: Vertex,
    iso_value: Voxel,
}

pub type Cube = [Corner; 8];

pub fn collect_iso_volume(iso_field: &ScalarField) -> Array3<Array3<Voxel>> {
    let grid_size = iso_field.dim().0;

    iso_field
        .exact_chunks((2, 2, 2))
        .into_iter()
        .map(|c| c.into_owned())
        .collect::<Array<Array3<Voxel>, _>>()
        .into_shape((grid_size / 2, grid_size / 2, grid_size / 2))
        .unwrap()
}

pub fn cubes_iter(chunked_iso_field: Array3<Array3<Voxel>>) -> IntoIter<Cube> {
    //let chunked_iso_field = collect_iso_volume(&iso_field);

    let indexed_iter = chunked_iso_field.indexed_iter();

    indexed_iter
        .map(|(index, data)| {
            MASKS.map(|m| {
                Corner {
                    vertex: Vertex::from((index, m)),
                    iso_value: data[[m.0, m.1, m.2]],
                }
            })
        })
        .collect::<Vec<Cube>>()
        .into_iter()
}

pub fn cubes_iter2(iso_field: &ScalarField) -> IntoIter<Cube> {
    let chunked_iso_field = collect_iso_volume(&iso_field);

    let indexed_iter = chunked_iso_field.indexed_iter();

    indexed_iter
        .map(|(index, data)| {
            MASKS.map(|m| {
                Corner {
                    vertex: Vertex::from((index, m)),
                    iso_value: data[[m.0, m.1, m.2]],
                }
            })
        })
        .collect::<Vec<Cube>>()
        .into_iter()
}

fn t<'a>() -> impl Fn(&'a ScalarField) -> IntoIter<Cube> {
    let x = compose!(collect_iso_volume, cubes_iter);
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iso_field_generator::{generate_iso_field, ScalarField};
    use float_cmp::approx_eq;
    use pretty_assertions::assert_eq;
    use crate::compose;
    use crate::compose::compose_two;

    const BALL_POS: [(f32, f32, f32); 2] = [(8.5, 8.5, 8.5), (8.5, 17.0, 8.5)];

    const GRID_SIZE: usize = 32;

    fn assert_cube(voxel: &Cube, iso_cube: &ScalarField) {
        voxel.iter().for_each(|v| {
            let vertex_value = v.iso_value;
            let cube_value = iso_cube[[
                v.vertex.x as usize,
                v.vertex.y as usize,
                v.vertex.z as usize,
            ]];
            assert!(approx_eq!(Voxel, vertex_value, cube_value, ulps = 2));
        });
    }

    #[test]
    fn test_convert_iso_to_voxels() {
        let iso_cube = generate_iso_field(GRID_SIZE, &BALL_POS);

        //let mut c_iter = cubes_iter(&iso_cube);

        let c_iter = compose!(collect_iso_volume, cubes_iter)(&iso_cube);


        c_iter.for_each(|v| assert_cube(&v, &iso_cube));

        // println!();

        // let v0 = v_iter.nth(0).unwrap();
        // // assert_voxel(&v1, &iso_cube);
        //
        // println!("{:?}", v0);
        //
        // let v1 = v_iter.nth(1).unwrap();
        // // assert_voxel(&v1, &iso_cube);
        //
        // println!("{:?}", v1);
        //
        //
        // let v5 = v_iter.nth(5).unwrap();
        // // assert_voxel(&v1, &iso_cube);
        //
        // println!("{:?}", v5);

        // // println!("{:?}", iso_cube[[0, 0, 1]]);
        //
        // let v2 = v_iter.nth(256).unwrap();
        // assert_voxel(&v2, &iso_cube);
    }
}
