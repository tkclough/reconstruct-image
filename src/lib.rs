mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

extern crate nalgebra as na;
use na::{DMatrix, Dynamic, DVector, U1};
use web_sys::{ImageData, CanvasRenderingContext2d};
use web_sys::console;
use js_sys::Math;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, reconstruct-image!");
}

#[wasm_bindgen]
pub struct ImageReconstruction {
    original_image: DMatrix<f64>,
    reconstructed_image: DMatrix<f64>,
    width: usize,
    height: usize,
    true_values: DVector<f64>,
    indices: Vec<(usize, usize)>,
    true_nuclear_norm: f64,
    matrix_buffer: DMatrix<f64>,
    vector_buffer: DVector<f64>,
    image_buffer: Vec<u8>,
}

#[wasm_bindgen]
impl ImageReconstruction {
    pub fn new(image: ImageData, p: f64) -> ImageReconstruction {
        utils::set_panic_hook();

        // Return an ImageReconstruction object with the given original image and
        // randomly sampled indices.
        
        let height = image.height() as usize;
        let width = image.width() as usize;
        let data = image.data();

        // interpolate to grayscale using luminosity method
        let mut grayscale: Vec<f64> = Vec::with_capacity(height * width);
        grayscale.resize(height * width, 0.0);
        for i in 0..height {
            for j in 0..width {
                let k = i * (width * 4) + j * 4;
                let (r, g, b) = (data[k], data[k + 1], data[k + 2]);
                grayscale[i + j * height] = (0.21 * (r as f64) + 0.72 * (g as f64) + 0.07 * (b as f64)) / 255.0;
            }
        }

        // make matrices for the images
        console::log_1(&"original image".into());
        let original_image: DMatrix<f64> = na::MatrixMN::<f64, Dynamic, Dynamic>::from_vec(height, width, grayscale);
        let reconstructed_image: DMatrix<f64> = na::MatrixMN::<f64, Dynamic, Dynamic>::zeros(height, width);

        // sample with probability p
        let mut indices: Vec<(usize, usize)> = Vec::new();
        let mut true_values = Vec::new();
        for i in 0..height {
            for j in 0..width {
                if Math::random() < p {
                    indices.push((i, j));
                    true_values.push(original_image[(i, j)]);
                }
            }
        }

        // the nuclear norm is the l1 norm of the singular values of the matrix
        let singular_values = original_image.clone_owned().svd(false, false).singular_values;
        let true_nuclear_norm = singular_values.abs().sum();

        let matrix_buffer = na::MatrixMN::<f64, Dynamic, Dynamic>::zeros(height, width);
        let vector_buffer = na::DVector::<f64>::zeros(true_values.len());
        let mut image_buffer = Vec::with_capacity(height * width * 4);
        image_buffer.resize(height * width * 4, 0);

        ImageReconstruction {
            original_image,
            reconstructed_image,
            width,
            height,
            true_values: DVector::from_vec(true_values),
            indices,
            true_nuclear_norm,
            matrix_buffer,
            vector_buffer,
            image_buffer,
        }
    }

    pub fn gradient_step(&mut self, eta: f64) {
        // compute the gradient
        // let diff = self.sample_known_indices(&self.reconstructed_image) - &self.true_values;
        // let g = self.unsample_known_indices(&diff);
        self.sample_known_indices_inplace();
        self.vector_buffer -= &self.true_values;
        self.unsample_known_indices_inplace();

        // move in that direction
        self.reconstructed_image -= eta * &self.matrix_buffer;

        // project back onto nuclear ball
        let projection = project_onto_nuclear_ball(self.reconstructed_image.clone_owned(), self.true_nuclear_norm);
        self.reconstructed_image = projection;

        // console::log_2(&"Error:".into(), &(&self.reconstructed_image - &self.original_image).norm().into());
    }

    fn sample_known_indices_inplace(&mut self) {
        for k in 0..self.indices.len() {
            let (i, j) = self.indices[k];
            self.vector_buffer[k] = self.reconstructed_image[(i, j)];
        }
    }

    fn unsample_known_indices_inplace(&mut self) {
        for k in 0..self.indices.len() {
            let (i, j) = self.indices[k];
            self.matrix_buffer[(i, j)] = self.vector_buffer[k];
        }
    }

    fn sample_known_indices(&self, mat: &DMatrix<f64>) -> DVector<f64> {
        let mut known: Vec<f64> = Vec::with_capacity(self.indices.len());
        known.resize(self.indices.len(), 0.0);
        for k in 0..known.len() {
            let (i, j) = self.indices[k];
            known[k] = mat[(i, j)];
        }

        DVector::from_vec(known)
    }

    fn unsample_known_indices(&self, vec: &DVector<f64>) -> DMatrix<f64> {
        let mut unsampled = Vec::with_capacity(self.original_image.len());
        unsampled.resize(self.original_image.len(), 0.0);
        for k in 0..vec.len() {
            let (i, j) = self.indices[k];
            unsampled[i + j*self.height] = vec[k];
        }

        DMatrix::from_vec(self.height, self.width, unsampled)
    }

    pub fn draw_original_image(
        &mut self,
        ctx: &CanvasRenderingContext2d
    ) {
        matrix_to_image_data(&self.original_image, &mut self.image_buffer);

        self.draw(ctx);
    }

    pub fn draw_reconstructed_image(
        &mut self,
        ctx: &CanvasRenderingContext2d
    ) {
        matrix_to_image_data(&self.reconstructed_image, &mut self.image_buffer);

        self.draw(ctx);
    }

    pub fn draw_corrupted_image(
        &mut self,
        ctx: &CanvasRenderingContext2d
    ) {
        let (r, c) = (self.height, self.width);
        for i in 0..self.height {
            for j in 0..self.width {
                self.image_buffer[i * (c*4) + j*4]     = 0;
                self.image_buffer[i * (c*4) + j*4 + 1] = 0;
                self.image_buffer[i * (c*4) + j*4 + 2] = 0;
                self.image_buffer[i * (c*4) + j*4 + 3] = 255;
            }
        }
        for (i,j) in &self.indices {
            let intensity = (255.0 * self.original_image[(*i, *j)]) as u8;
            self.image_buffer[i * (c*4) + j*4]     = intensity;
            self.image_buffer[i * (c*4) + j*4 + 1] = intensity;
            self.image_buffer[i * (c*4) + j*4 + 2] = intensity;
            self.image_buffer[i * (c*4) + j*4 + 3] = 255;
        }

        self.draw(ctx);
    }

    fn draw(
        &mut self,
        ctx: &CanvasRenderingContext2d
    ) {
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut self.image_buffer), self.width as u32, self.height as u32).unwrap();
        ctx.put_image_data(&image_data, 0.0, 0.0);
    }
}

fn matrix_to_image_data(mat: &DMatrix<f64>, image_data: &mut Vec<u8>) {
    let (r, c) = (mat.nrows(), mat.ncols());

    for i in 0..r {
        for j in 0..c {
            let intensity = (255.0 * mat[(i, j)]) as u8;
            image_data[i * (c * 4) + j * 4]     = intensity; // red
            image_data[i * (c * 4) + j * 4 + 1] = intensity; // blue
            image_data[i * (c * 4) + j * 4 + 2] = intensity; // green
            image_data[i * (c * 4) + j * 4 + 3] = 255;       // alpha
        }
    }
}

pub fn project_onto_simplex(v: &DVector<f64>, z: f64) -> DVector<f64> {
    // linear time algorithm described by Duchi et al. in
    // "Efficient projections onto the l 1-ball for learning in high dimensions."
    let mut v2 = v.clone_owned();
    let n = v.len();

    let (mut s, mut rho) = (0.0, 0 as isize);

    // U is elements in range [lo,hi)
    let (mut lo, mut hi) = (0, n);

    // U = {} once lo == hi
    while lo < hi {
        // pick k from U at random (i.e. from [lo, hi))
        let k = lo + ((hi - lo - 1) as f64 * Math::random()) as usize;
        let vk = v2[k];

        // partition U into L and G via inplace advancing
        // loop invariants:
        // - all elements in range [lo, lo2) are  < vk
        // - all elements in range (hi2, hi) are >= k
        let (mut lo2, mut hi2) = (lo, hi - 1);
        while lo2 < hi2 {
            let (vl, vh) = (v2[lo2], v2[hi2]);
            if vl < vk && vk <= vh {
                // both are in the right position; advance
                lo2 += 1;
                hi2 -= 1;
            } else if vl >= vk && vk <= vh {
                // vl should be swapped, vh is fine; advance hi2
                hi2 -= 1;
            } else if vl < vk && vk > vh {
                // vl is fine, vh should be swapped; advance lo2
                lo2 += 1;
            } else {
                // both are in wrong position; swap
                v2[lo2] = vh;
                v2[hi2] = vl;

                lo2 += 1;
                hi2 -= 1;
            }
        }

        // we know that all elements in positions < lo2 are < vk, but we don't know that 
        // v2[lo2] >= vk, so let's get to the first position that does satisfy that
        while v2[lo2] < vk {
            lo2 += 1;
        }

        // it must now be the case that:
        // - all elements in [lo, lo2) are less than vk (L = [lo, lo2))
        // - all elements in [lo2, hi) are greater than vk (G = [lo2, hi))

        // delta_s is the sum of values in G, i.e. values in [lo2, hi)
        // delta_rho is the number of values in G, i.e. hi - lo2
        let (mut delta_s, delta_rho) = (0.0,(hi - lo2) as isize);
        for i in lo2..hi {
            delta_s += v2[i];
        }

        if (s + delta_s) - (((rho + delta_rho) as f64) * vk) < z {
            s += delta_s;
            rho += delta_rho;

            // U <- L
            hi = lo2
        } else {
            // U <- G \ {k}
            // we don't know that v2[lo2] == vk, so we don't know that G \ {k} = [lo2 + 1, hi),
            // so we must find an index i in [lo2, hi) equal to vk and v2[i] with v2[lo2]
            if lo2 < hi {
                for i in lo2..hi {
                    if v2[i] == vk {
                        // swap
                        v2[lo2] = v2[i];
                        v2[i] = vk;

                        break
                    }
                }
            }
            lo = lo2 + 1; // lo2 points to k so G \ {k} = [lo2 + 1, hi)
        }
    }

    let theta = (s - z) / (rho as f64);
    for i in 0..n {
        v2[i] = if v[i] - theta > 0.0 {
            v[i] - theta
        } else {
            0.0
        };
    }

    v2
}

pub fn project_onto_l1ball(v: &DVector<f64>, z: f64) -> DVector<f64> {
    // compute signs
    let mut u = v.clone_owned();
    for i in 0..u.len() {
        u[i] = if u[i] > 0.0 {
            1.0
        } else if u[i] < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    let w = project_onto_simplex(&v.abs(), z);

    u.component_mul_assign(&w);

    u
}

pub fn project_onto_nuclear_ball(M: DMatrix<f64>, z: f64) -> DMatrix<f64> {
    let mut svd = M.svd(true, true);
    svd.singular_values = project_onto_l1ball(&svd.singular_values, z);
    svd.recompose().unwrap()
}