// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Tests for CUDA FEE beam code.

use approx::{assert_abs_diff_eq, assert_abs_diff_ne};
use marlu::{constants::MWA_LAT_RAD, ndarray::prelude::*};
use serial_test::serial;

use super::*;

#[test]
#[serial]
fn test_cuda_calc_jones_no_norm() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    let freqs = [150e6 as u32];
    let delays = array![[3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0]];
    let amps =
        array![[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]];
    let norm_to_zenith = false;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 1);
    assert_eq!(cuda_beam.num_unique_tiles, 1);
    assert_eq!(cuda_beam.num_unique_freqs, 1);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = None;

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let jones_gpu = result.unwrap();

    // Compare with CPU results.
    let mut jones_cpu =
        Array3::from_elem((delays.dim().0, freqs.len(), az.len()), Jones::default());
    // Maybe need to regenerate the directions, depending on the CUDA precision.
    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| (0.45 + i as f64 / 10000.0, 0.45 + i as f64 / 10000.0))
        .unzip();
    for ((mut out, delays), amps) in jones_cpu
        .outer_iter_mut()
        .zip(delays.outer_iter())
        .zip(amps.outer_iter())
    {
        for (mut out, freq) in out.outer_iter_mut().zip(freqs) {
            let cpu_results = beam
                .calc_jones_array_pair(
                    &az,
                    &za,
                    freq,
                    delays.as_slice().unwrap(),
                    amps.as_slice().unwrap(),
                    norm_to_zenith,
                    None,
                    false,
                )
                .unwrap();

            // Demote the CPU results if we have to.
            #[cfg(feature = "cuda-single")]
            let cpu_results: Vec<Jones<f32>> = cpu_results.into_iter().map(|j| j.into()).collect();

            out.assign(&Array1::from(cpu_results));
        }
    }

    #[cfg(not(feature = "cuda-single"))]
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-15);

    #[cfg(feature = "cuda-single")]
    // The errors are heavily dependent on the directions.
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-7);
}

#[test]
#[serial]
fn test_cuda_calc_jones_w_norm() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    let freqs = [150e6 as u32];
    let delays = array![[3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0]];
    let amps =
        array![[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]];
    let norm_to_zenith = true;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 1);
    assert_eq!(cuda_beam.num_unique_tiles, 1);
    assert_eq!(cuda_beam.num_unique_freqs, 1);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = None;

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let jones_gpu = result.unwrap();

    // Compare with CPU results.
    let mut jones_cpu =
        Array3::from_elem((delays.dim().0, freqs.len(), az.len()), Jones::default());
    // Maybe need to regenerate the directions, depending on the CUDA precision.
    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| (0.45 + i as f64 / 10000.0, 0.45 + i as f64 / 10000.0))
        .unzip();
    for ((mut out, delays), amps) in jones_cpu
        .outer_iter_mut()
        .zip(delays.outer_iter())
        .zip(amps.outer_iter())
    {
        for (mut out, freq) in out.outer_iter_mut().zip(freqs) {
            let cpu_results = beam
                .calc_jones_array_pair(
                    &az,
                    &za,
                    freq,
                    delays.as_slice().unwrap(),
                    amps.as_slice().unwrap(),
                    norm_to_zenith,
                    array_latitude_rad,
                    false,
                )
                .unwrap();

            // Demote the CPU results if we have to.
            #[cfg(feature = "cuda-single")]
            let cpu_results: Vec<Jones<f32>> = cpu_results.into_iter().map(|j| j.into()).collect();

            out.assign(&Array1::from(cpu_results));
        }
    }

    #[cfg(not(feature = "cuda-single"))]
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-15);

    #[cfg(feature = "cuda-single")]
    // The errors are heavily dependent on the directions.
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-6);
}

#[test]
#[serial]
fn test_cuda_calc_jones_w_norm_and_parallactic() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    let freqs = [150e6 as u32];
    let delays = array![[3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0]];
    let amps =
        array![[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]];
    let norm_to_zenith = true;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 1);
    assert_eq!(cuda_beam.num_unique_tiles, 1);
    assert_eq!(cuda_beam.num_unique_freqs, 1);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = Some(MWA_LAT_RAD);

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, true);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let jones_gpu = result.unwrap();

    // Compare with CPU results.
    let mut jones_cpu =
        Array3::from_elem((delays.dim().0, freqs.len(), az.len()), Jones::default());
    // Maybe need to regenerate the directions, depending on the CUDA precision.
    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| (0.45 + i as f64 / 10000.0, 0.45 + i as f64 / 10000.0))
        .unzip();
    for ((mut out, delays), amps) in jones_cpu
        .outer_iter_mut()
        .zip(delays.outer_iter())
        .zip(amps.outer_iter())
    {
        for (mut out, freq) in out.outer_iter_mut().zip(freqs) {
            let cpu_results = beam
                .calc_jones_array_pair(
                    &az,
                    &za,
                    freq,
                    delays.as_slice().unwrap(),
                    amps.as_slice().unwrap(),
                    norm_to_zenith,
                    array_latitude_rad,
                    true,
                )
                .unwrap();

            // Demote the CPU results if we have to.
            #[cfg(feature = "cuda-single")]
            let cpu_results: Vec<Jones<f32>> = cpu_results.into_iter().map(|j| j.into()).collect();

            out.assign(&Array1::from(cpu_results));
        }
    }

    #[cfg(not(feature = "cuda-single"))]
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-15);

    #[cfg(feature = "cuda-single")]
    // The errors are heavily dependent on the directions.
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-6);
}

#[test]
#[serial]
fn test_cuda_calc_jones_with_and_without_parallactic() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    let freqs = [150e6 as u32];
    let delays = array![[3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0]];
    let amps =
        array![[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]];
    let norm_to_zenith = true;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 1);
    assert_eq!(cuda_beam.num_unique_tiles, 1);
    assert_eq!(cuda_beam.num_unique_freqs, 1);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = Some(MWA_LAT_RAD);

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let pa = result.unwrap();
    let result = cuda_beam.calc_jones_pair(&az, &za, None, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let not_pa = result.unwrap();

    assert_abs_diff_ne!(pa, not_pa);
}

#[test]
#[serial]
fn test_cuda_calc_jones_deduplication() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    // 6 freqs here, but only 3 unique ones.
    let freqs = [
        150e6 as u32,
        200e6 as _,
        250e6 as _,
        150e6 as _,
        200e6 as _,
        250000001,
    ];
    // Tiles 0 and 3 are the same; 3 unique tiles.
    let delays = array![
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
        [32, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0], // Delays of 32 are treated as distinct
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
    ];
    let amps = array![
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    ];
    let norm_to_zenith = false;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 9);
    assert_eq!(cuda_beam.num_unique_tiles, 3);
    assert_eq!(cuda_beam.num_unique_freqs, 3);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = None;

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let jones_gpu = result.unwrap();

    // Compare with CPU results.
    let mut jones_cpu =
        Array3::from_elem((delays.dim().0, freqs.len(), az.len()), Jones::default());
    // Maybe need to regenerate the directions, depending on the CUDA precision.
    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| (0.45 + i as f64 / 10000.0, 0.45 + i as f64 / 10000.0))
        .unzip();
    for ((mut out, delays), amps) in jones_cpu
        .outer_iter_mut()
        .zip(delays.outer_iter())
        .zip(amps.outer_iter())
    {
        for (mut out, freq) in out.outer_iter_mut().zip(freqs) {
            let cpu_results = beam
                .calc_jones_array_pair(
                    &az,
                    &za,
                    freq,
                    delays.as_slice().unwrap(),
                    amps.as_slice().unwrap(),
                    norm_to_zenith,
                    array_latitude_rad,
                    false,
                )
                .unwrap();

            // Demote the CPU results if we have to.
            #[cfg(feature = "cuda-single")]
            let cpu_results: Vec<Jones<f32>> = cpu_results.into_iter().map(|j| j.into()).collect();

            out.assign(&Array1::from(cpu_results));
        }
    }

    #[cfg(not(feature = "cuda-single"))]
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-15);

    #[cfg(feature = "cuda-single")]
    // The errors are heavily dependent on the directions.
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-6);
}

#[test]
#[serial]
fn test_cuda_calc_jones_deduplication_w_norm() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    // 6 freqs here, but only 3 unique ones.
    let freqs = [
        150e6 as u32,
        200e6 as _,
        250e6 as _,
        150e6 as _,
        200e6 as _,
        250000001,
    ];
    // Tiles 0 and 3 are the same; 3 unique tiles.
    let delays = array![
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
        [32, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0], // Delays of 32 are treated as distinct
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
    ];
    let amps = array![
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    ];
    let norm_to_zenith = true;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 9);
    assert_eq!(cuda_beam.num_unique_tiles, 3);
    assert_eq!(cuda_beam.num_unique_freqs, 3);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = None;

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let jones_gpu = result.unwrap();

    // Compare with CPU results.
    let mut jones_cpu =
        Array3::from_elem((delays.dim().0, freqs.len(), az.len()), Jones::default());
    // Maybe need to regenerate the directions, depending on the CUDA precision.
    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| (0.45 + i as f64 / 10000.0, 0.45 + i as f64 / 10000.0))
        .unzip();
    for ((mut out, delays), amps) in jones_cpu
        .outer_iter_mut()
        .zip(delays.outer_iter())
        .zip(amps.outer_iter())
    {
        for (mut out, freq) in out.outer_iter_mut().zip(freqs) {
            let cpu_results = beam
                .calc_jones_array_pair(
                    &az,
                    &za,
                    freq,
                    delays.as_slice().unwrap(),
                    amps.as_slice().unwrap(),
                    norm_to_zenith,
                    array_latitude_rad,
                    false,
                )
                .unwrap();

            // Demote the CPU results if we have to.
            #[cfg(feature = "cuda-single")]
            let cpu_results: Vec<Jones<f32>> = cpu_results.into_iter().map(|j| j.into()).collect();

            out.assign(&Array1::from(cpu_results));
        }
    }

    #[cfg(not(feature = "cuda-single"))]
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-15);

    #[cfg(feature = "cuda-single")]
    // The errors are heavily dependent on the directions.
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-6);
}

#[test]
#[serial]
fn test_cuda_calc_jones_no_amps() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    let freqs: Vec<u32> = [50e6, 75e6, 100e6, 125e6, 150e6, 175e6, 200e6]
        .into_iter()
        .map(|f| f as u32)
        .collect();
    let delays = array![
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0],
        [3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0]
    ];
    let amps = array![
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];
    let norm_to_zenith = false;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 14);
    assert_eq!(cuda_beam.num_unique_tiles, 2);
    assert_eq!(cuda_beam.num_unique_freqs, 7);

    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| {
            (
                0.45 + i as CudaFloat / 10000.0,
                0.45 + i as CudaFloat / 10000.0,
            )
        })
        .unzip();
    let array_latitude_rad = None;

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let jones_gpu = result.unwrap();

    // Compare with CPU results.
    let mut jones_cpu =
        Array3::from_elem((delays.dim().0, freqs.len(), az.len()), Jones::default());
    // Maybe need to regenerate the directions, depending on the CUDA precision.
    let (az, za): (Vec<_>, Vec<_>) = (0..1025)
        .map(|i| (0.45 + i as f64 / 10000.0, 0.45 + i as f64 / 10000.0))
        .unzip();
    for ((mut out, delays), amps) in jones_cpu
        .outer_iter_mut()
        .zip(delays.outer_iter())
        .zip(amps.outer_iter())
    {
        for (mut out, freq) in out.outer_iter_mut().zip(freqs.iter()) {
            let cpu_results = beam
                .calc_jones_array_pair(
                    &az,
                    &za,
                    *freq,
                    delays.as_slice().unwrap(),
                    amps.as_slice().unwrap(),
                    norm_to_zenith,
                    array_latitude_rad,
                    false,
                )
                .unwrap();

            // Demote the CPU results if we have to.
            #[cfg(feature = "cuda-single")]
            let cpu_results: Vec<Jones<f32>> = cpu_results.into_iter().map(|j| j.into()).collect();

            out.assign(&Array1::from(cpu_results));
        }
    }

    #[cfg(not(feature = "cuda-single"))]
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-15);

    #[cfg(feature = "cuda-single")]
    // The errors are heavily dependent on the directions.
    assert_abs_diff_eq!(jones_gpu, jones_cpu, epsilon = 1e-6);

    // The results for this tile are all zero.
    assert_abs_diff_eq!(
        jones_gpu.slice(s![1, .., ..]),
        Array2::from_elem((jones_gpu.dim().1, jones_gpu.dim().2), Jones::default())
    );

    // The results for this tile are at least some non-zero.
    assert_abs_diff_ne!(
        jones_gpu.slice(s![0, .., ..]),
        Array2::from_elem((jones_gpu.dim().1, jones_gpu.dim().2), Jones::default())
    );
}

#[test]
#[serial]
fn test_cuda_calc_jones_iau_order() {
    let beam = FEEBeam::new("mwa_full_embedded_element_pattern.h5").unwrap();
    let freqs = [150e6 as u32];
    let delays = array![[3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0]];
    let amps =
        array![[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]];
    let norm_to_zenith = false;
    let result = unsafe { beam.cuda_prepare(&freqs, delays.view(), amps.view(), norm_to_zenith) };
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let cuda_beam = result.unwrap();
    assert_eq!(cuda_beam.num_coeffs, 1);
    assert_eq!(cuda_beam.num_unique_tiles, 1);
    assert_eq!(cuda_beam.num_unique_freqs, 1);

    let (az, za): (Vec<_>, Vec<_>) = (vec![0.45 / 10000.0], vec![0.45 / 10000.0]);
    let array_latitude_rad = Some(MWA_LAT_RAD);

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, true);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let j_iau = result.unwrap();

    let result = cuda_beam.calc_jones_pair(&az, &za, array_latitude_rad, false);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    let j_not_iau = result.unwrap();

    assert_ne!(j_iau[(0, 0, 0)][0], j_not_iau[(0, 0, 0)][0]);
    assert_ne!(j_iau[(0, 0, 0)][1], j_not_iau[(0, 0, 0)][1]);
    assert_ne!(j_iau[(0, 0, 0)][2], j_not_iau[(0, 0, 0)][2]);
    assert_ne!(j_iau[(0, 0, 0)][3], j_not_iau[(0, 0, 0)][3]);
    assert_eq!(j_iau[(0, 0, 0)][0], j_not_iau[(0, 0, 0)][3]);
    assert_eq!(j_iau[(0, 0, 0)][1], j_not_iau[(0, 0, 0)][2]);
    assert_eq!(j_iau[(0, 0, 0)][2], j_not_iau[(0, 0, 0)][1]);
    assert_eq!(j_iau[(0, 0, 0)][3], j_not_iau[(0, 0, 0)][0]);
}
