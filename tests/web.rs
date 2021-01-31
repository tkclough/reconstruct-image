//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
extern crate nalgebra as na;
use na::{DVector,DMatrix};

extern crate reconstruct_image;
use reconstruct_image::{project_onto_l1ball,project_onto_nuclear_ball};

use web_sys::console;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}

#[wasm_bindgen_test]
pub fn test_project_onto_l1ball() {
    let v_data = vec!(-1.0, 1.0);
    let expected_data = vec!(-0.5, 0.5);
    let expected = DVector::from_vec(expected_data);

    let v = DVector::from_vec(v_data);
    let u = project_onto_l1ball(&v, 1.0);
    
    assert_eq!(u, expected);
}

#[wasm_bindgen_test]
pub fn test_project_onto_l1ball_2() {
    let v_data = vec!(-1.0, 2.0);
    let v = DVector::from_vec(v_data);

    let expected_data = vec!(0.0, 1.0);
    let expected = DVector::from_vec(expected_data);

    let u = project_onto_l1ball(&v, 1.0);
    
    assert_eq!(u, expected);
}

#[wasm_bindgen_test]
pub fn test_project_onto_l1ball_3() {
    let v_data = vec!(-1.0, 2.0, 3.0);
    let v = DVector::from_vec(v_data);

    let expected_data = vec!(0.0, 0.5, 1.5);
    let expected = DVector::from_vec(expected_data);

    let u = project_onto_l1ball(&v, 2.0);
    
    assert_eq!(u, expected);
}

#[wasm_bindgen_test]
pub fn test_project_onto_l1ball_4() {
    let v_data = vec!(5.0, 6.0, 4.0);
    let v = DVector::from_vec(v_data);

    let expected_data = vec!(1.0, 2.0, 0.0);
    let expected = DVector::from_vec(expected_data);

    let u = project_onto_l1ball(&v, 3.0);
    
    assert_eq!(u, expected);
}



#[wasm_bindgen_test]
pub fn test_project_onto_nuclear_ball() {
    // Matrix with singular values 4, 5, 6
    let M_data = vec!(-3.97767621,  0.1910051,   3.1695218, 
                      -2.5267609,  -2.90601047, -3.87182264,
                       1.96896401, -2.84412429,  3.05113917);
    let M = DMatrix::from_vec(3, 3, M_data);

    let expected_singular_values = vec!(0.0, 1.0, 2.0);

    let P = project_onto_nuclear_ball(M, 3.0);
    let Psvd = P.svd(false, false);
    let mut svs = vec!(Psvd.singular_values[0], Psvd.singular_values[1], Psvd.singular_values[2]);
    svs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // need to account for floating point errors
    let diff = (DVector::from_vec(svs) - DVector::from_vec(expected_singular_values));
    assert!(diff.norm() < 1e-6);
}