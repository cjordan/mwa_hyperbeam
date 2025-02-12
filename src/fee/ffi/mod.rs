// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Code for allowing other languages to talk to this Rust library's FEE beam
//! code. See the examples directory for usage.

#[cfg(test)]
mod tests;

use std::{ffi::CStr, os::raw::c_char, panic, slice};

use rayon::iter::Either;

use super::FEEBeam;
use crate::ffi::{ffi_error, update_last_error};

cfg_if::cfg_if! {
    if #[cfg(any(feature = "cuda", feature = "hip"))] {
        use ndarray::prelude::*;

        use super::FEEBeamGpu;
        use crate::gpu::{DevicePointer, GpuFloat};
    }
}

/// Create a new MWA FEE beam.
///
/// # Arguments
///
/// * `hdf5_file` - the path to the MWA FEE beam file.
/// * `fee_beam` - a double pointer to the `FEEBeam` struct which is set by this
///   function. This struct must be freed by calling `free_fee_beam`.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[no_mangle]
pub unsafe extern "C" fn new_fee_beam(
    hdf5_file: *const c_char,
    fee_beam: *mut *mut FEEBeam,
) -> i32 {
    panic::set_hook(Box::new(|pi| {
        update_last_error(panic_message::panic_info_message(pi).to_string());
    }));

    let result = panic::catch_unwind(|| {
        let path = match CStr::from_ptr(hdf5_file).to_str() {
            Ok(p) => p,
            Err(e) => {
                update_last_error(e.to_string());
                return Either::Left(2);
            }
        };
        match FEEBeam::new(path) {
            Ok(b) => Either::Right(b),
            Err(e) => {
                update_last_error(e.to_string());
                Either::Left(1)
            }
        }
    });

    let _ = panic::take_hook();

    match result {
        Ok(Either::Right(b)) => {
            *fee_beam = Box::into_raw(Box::new(b));
            0
        }
        Ok(Either::Left(e)) => e,
        // For panics, the FFI error message is already updated.
        Err(_) => -1,
    }
}

/// Create a new MWA FEE beam. Requires the HDF5 beam file path to be specified
/// by the environment variable `MWA_BEAM_FILE`.
///
/// # Arguments
///
/// * `fee_beam` - a double pointer to the `FEEBeam` struct which is set by this
///   function. This struct must be freed by calling `free_fee_beam`.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[no_mangle]
pub unsafe extern "C" fn new_fee_beam_from_env(fee_beam: *mut *mut FEEBeam) -> i32 {
    panic::set_hook(Box::new(|pi| {
        update_last_error(panic_message::panic_info_message(pi).to_string());
    }));

    let result = panic::catch_unwind(|| match FEEBeam::new_from_env() {
        Ok(b) => Either::Right(b),
        Err(e) => {
            update_last_error(e.to_string());
            Either::Left(1)
        }
    });

    let _ = panic::take_hook();

    match result {
        Ok(Either::Right(b)) => {
            *fee_beam = Box::into_raw(Box::new(b));
            0
        }
        Ok(Either::Left(e)) => e,
        // For panics, the FFI error message is already updated.
        Err(_) => -1,
    }
}

/// Get the beam response Jones matrix for the given direction and pointing. Can
/// optionally re-define the X and Y polarisations and apply a parallactic-angle
/// correction; see
/// <https://github.com/MWATelescope/mwa_hyperbeam/blob/main/fee_pols.pdf>
///
/// `delays` and `amps` apply to each dipole in a given MWA tile, and *must*
/// have 16 elements (each corresponds to an MWA dipole in a tile, in the M&C
/// order; see
/// <https://wiki.mwatelescope.org/pages/viewpage.action?pageId=48005139>).
/// `amps` being dipole gains (usually 1 or 0), not digital gains.
///
/// 16 or 32 elements can be supplied for `amps`. If there are 16, then the
/// dipole gains apply to both X and Y elements of dipoles. If there are 32, the
/// first 16 amps are for the X elements, the next 16 the Y elements.
///
/// Note the type of `jones` (`*double`); we can't pass complex numbers across
/// the FFI boundary, so the real and imaginary components are unpacked into
/// doubles. The output contains 8 doubles, where the j00 is the first pair, j01
/// is the second pair, etc.
///
/// # Arguments
///
/// * `fee_beam` - A pointer to a `FEEBeam` struct created with the
///   `new_fee_beam` function
/// * `az_rad` - The azimuth direction to get the beam response (units of
///   radians)
/// * `za_rad` - The zenith angle direction to get the beam response (units of
///   radians)
/// * `freq_hz` - The frequency used for the beam response in Hertz
/// * `delays` - A pointer to a 16-element array of dipole delays for an MWA
///   tile
/// * `amps` - A pointer to a 16- or 32-element array of dipole gains for an MWA
///   tile. The number of elements is indicated by `num_amps`.
/// * `num_amps` - The number of dipole gains used (either 16 or 32).
/// * `norm_to_zenith` - A boolean indicating whether the beam response should
///   be normalised with respect to zenith.
/// * `latitude_rad` - A pointer to a telescope latitude to use for the
///   parallactic-angle correction. If the pointer is null, no correction is
///   done.
/// * `iau_order` - A boolean indicating whether the Jones matrix should be
///   arranged [NS-NS NS-EW EW-NS EW-EW] (true) or not (false).
/// * `jones` - A pointer to a buffer with at least `8 * sizeof(double)`
///   allocated. The Jones matrix beam response is written here.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[no_mangle]
pub unsafe extern "C" fn fee_calc_jones(
    fee_beam: *mut FEEBeam,
    az_rad: f64,
    za_rad: f64,
    freq_hz: u32,
    delays: *const u32,
    amps: *const f64,
    num_amps: u32,
    norm_to_zenith: u8,
    latitude_rad: *const f64,
    iau_order: u8,
    jones: *mut f64,
) -> i32 {
    match num_amps {
        16 | 32 => (),
        _ => {
            update_last_error("A value other than 16 or 32 was used for num_amps".to_string());
            return 1;
        }
    };
    let norm_bool = match norm_to_zenith {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for norm_to_zenith".to_string());
            return 1;
        }
    };
    let latitude_rad = latitude_rad.as_ref().copied();
    let iau_bool = match iau_order {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for iau_order".to_string());
            return 1;
        }
    };

    let beam = &*fee_beam;
    let delays_s = slice::from_raw_parts(delays, 16);
    let amps_s = slice::from_raw_parts(amps, num_amps as usize);

    // Using the passed-in beam, get the beam response (Jones matrix).
    match beam.calc_jones_pair(
        az_rad,
        za_rad,
        freq_hz,
        delays_s,
        amps_s,
        norm_bool,
        latitude_rad,
        iau_bool,
    ) {
        Ok(j) => {
            let jones_buf = slice::from_raw_parts_mut(jones, 8);
            jones_buf[..].copy_from_slice(&[
                j[0].re, j[0].im, j[1].re, j[1].im, j[2].re, j[2].im, j[3].re, j[3].im,
            ]);
            0
        }
        Err(e) => {
            update_last_error(e.to_string());
            1
        }
    }
}

/// Get the beam response Jones matrix for several az/za directions for the
/// given pointing. The Jones matrix elements for each direction are put into a
/// single array (made available with the output pointer `jones`). Can
/// optionally re-define the X and Y polarisations and apply a parallactic-angle
/// correction; see
/// <https://github.com/MWATelescope/mwa_hyperbeam/blob/main/fee_pols.pdf>
///
/// `delays` and `amps` apply to each dipole in a given MWA tile, and *must*
/// have 16 elements (each corresponds to an MWA dipole in a tile, in the M&C
/// order; see
/// <https://wiki.mwatelescope.org/pages/viewpage.action?pageId=48005139>).
/// `amps` being dipole gains (usually 1 or 0), not digital gains.
///
/// As there are 8 elements per Jones matrix, there must be at least `8 *
/// num_azza * sizeof(double)` allocated in the array. Rust will calculate the
/// Jones matrices in parallel. See the documentation for `calc_jones` for more
/// info.
///
/// # Arguments
///
/// * `fee_beam` - A pointer to a `FEEBeam` struct created with the
///   `new_fee_beam` function
/// * `num_azza` - The number of directions within `az_rad` and `za_rad`
/// * `az_rad` - The azimuth direction to get the beam response (units of
///   radians)
/// * `za_rad` - The zenith angle direction to get the beam response (units of
///   radians)
/// * `freq_hz` - The frequency used for the beam response in Hertz
/// * `delays` - A pointer to a 16-element array of dipole delays for an MWA
///   tile
/// * `amps` - A pointer to a 16- or 32-element array of dipole gains for an MWA
///   tile. The number of elements is indicated by `num_amps`.
/// * `num_amps` - The number of dipole gains used (either 16 or 32).
/// * `norm_to_zenith` - A boolean indicating whether the beam response should
///   be normalised with respect to zenith.
/// * `latitude_rad` - A pointer to a telescope latitude to use for the
///   parallactic-angle correction. If the pointer is null, no correction is
///   done.
/// * `iau_order` - A boolean indicating whether the Jones matrix should be
///   arranged [NS-NS NS-EW EW-NS EW-EW] (true) or not (false).
/// * `jones` - A pointer to a buffer with at least `8 * num_azza *
///   sizeof(double)` bytes allocated. The Jones matrix beam responses are
///   written here.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[no_mangle]
pub unsafe extern "C" fn fee_calc_jones_array(
    fee_beam: *mut FEEBeam,
    num_azza: u32,
    az_rad: *const f64,
    za_rad: *const f64,
    freq_hz: u32,
    delays: *const u32,
    amps: *const f64,
    num_amps: u32,
    norm_to_zenith: u8,
    latitude_rad: *const f64,
    iau_order: u8,
    jones: *mut f64,
) -> i32 {
    match num_amps {
        16 | 32 => (),
        _ => {
            update_last_error("A value other than 16 or 32 was used for num_amps".to_string());
            return 1;
        }
    };
    let norm_bool = match norm_to_zenith {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for norm_to_zenith".to_string());
            return 1;
        }
    };
    let latitude_rad = latitude_rad.as_ref().copied();
    let iau_bool = match iau_order {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for iau_order".to_string());
            return 1;
        }
    };

    let beam = &*fee_beam;
    let az = slice::from_raw_parts(az_rad, num_azza as usize);
    let za = slice::from_raw_parts(za_rad, num_azza as usize);
    let delays_s = slice::from_raw_parts(delays, 16);
    let amps_s = slice::from_raw_parts(amps, num_amps as usize);
    let results_s = slice::from_raw_parts_mut(jones.cast(), num_azza as usize);

    ffi_error!(beam.calc_jones_array_pair_inner(
        az,
        za,
        freq_hz,
        delays_s,
        amps_s,
        norm_bool,
        latitude_rad,
        iau_bool,
        results_s,
    ));
    0
}

/// Get the available frequencies inside the HDF5 file.
///
/// # Arguments
///
/// * `fee_beam` - the pointer to the `FEEBeam` struct.
/// * `freqs_ptr` - a double pointer to the FEE beam frequencies. The `const`
///   annotation is deliberate; the caller does not own the frequencies.
/// * `num_freqs` - a pointer to a `size_t` whose contents are set.
///
#[no_mangle]
pub unsafe extern "C" fn get_fee_beam_freqs(
    fee_beam: *mut FEEBeam,
    freqs_ptr: *mut *const u32,
    num_freqs: &mut usize,
) {
    let beam = &*fee_beam;
    let freqs = beam.get_freqs();
    *freqs_ptr = freqs.as_ptr();
    *num_freqs = freqs.len();
}

/// Given a frequency in Hz, get the closest available frequency inside the HDF5
/// file.
///
/// # Arguments
///
/// * `fee_beam` - the pointer to the `FEEBeam` struct.
///
/// # Returns
///
/// * The closest frequency to the specified frequency in Hz.
///
#[no_mangle]
pub unsafe extern "C" fn fee_closest_freq(fee_beam: *mut FEEBeam, freq: u32) -> u32 {
    let beam = &*fee_beam;
    beam.find_closest_freq(freq)
}

/// Free the memory associated with an `FEEBeam`.
///
/// # Arguments
///
/// * `fee_beam` - the pointer to the `FEEBeam` struct.
///
#[no_mangle]
pub unsafe extern "C" fn free_fee_beam(fee_beam: *mut FEEBeam) {
    drop(Box::from_raw(fee_beam));
}

/// Get a `FEEBeamGpu` struct, which is used to calculate beam responses on a
/// GPU (CUDA- or HIP-capable device).
///
/// # Arguments
///
/// * `fee_beam` - a pointer to a previously set `FEEBeam` struct.
/// * `freqs_hz` - a pointer to an array of frequencies (units of Hz) at which
///   the beam responses will be calculated.
/// * `delays` - a pointer to two-dimensional array of dipole delays. There must
///   be 16 delays per row; each row corresponds to a tile.
/// * `amps` - a pointer to two-dimensional array of dipole amplitudes. There
///   must be 16 or 32 amps per row; each row corresponds to a tile. The number
///   of amps per row is specified by `num_amps`.
/// * `num_freqs` - the number of frequencies in the array pointed to by
///   `freqs_hz`.
/// * `num_tiles` - the number of tiles in both `delays` and `amps`.
/// * `num_amps` - either 16 or 32. See the documentation for `calc_jones` for
///   more explanation.
/// * `norm_to_zenith` - A boolean indicating whether the beam responses should
///   be normalised with respect to zenith.
/// * `gpu_fee_beam` - a double pointer to the `FEEBeamGpu` struct which is set
///   by this function. This struct must be freed by calling
///   `free_gpu_fee_beam`.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn new_gpu_fee_beam(
    fee_beam: *mut FEEBeam,
    freqs_hz: *const u32,
    delays: *const u32,
    amps: *const f64,
    num_freqs: u32,
    num_tiles: u32,
    num_amps: u32,
    norm_to_zenith: u8,
    gpu_fee_beam: *mut *mut FEEBeamGpu,
) -> i32 {
    match num_amps {
        16 | 32 => (),
        _ => {
            update_last_error("A value other than 16 or 32 was used for num_amps".to_string());
            return 1;
        }
    };
    let norm_bool = match norm_to_zenith {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for norm_to_zenith".to_string());
            return 1;
        }
    };

    // Turn the pointers into slices and/or arrays.
    let freqs = slice::from_raw_parts(freqs_hz, num_freqs as usize);
    let amps = ArrayView2::from_shape_ptr((num_tiles as usize, num_amps as usize), amps);
    let delays = ArrayView2::from_shape_ptr((num_tiles as usize, 16), delays);

    let beam = &*fee_beam;
    let gpu_beam = ffi_error!(beam.gpu_prepare(freqs, delays, amps, norm_bool));
    *gpu_fee_beam = Box::into_raw(Box::new(gpu_beam));
    0
}

/// Get beam response Jones matrices for the given directions, using a GPU. The
/// Jones matrix elements for each direction are put into a host-memory buffer
/// `jones`. Can optionally re-define the X and Y polarisations and apply a
/// parallactic-angle correction; see
/// <https://github.com/MWATelescope/mwa_hyperbeam/blob/main/fee_pols.pdf>
///
/// # Arguments
///
/// * `gpu_fee_beam` - A pointer to a `FEEBeamGpu` struct created with the
///   `new_gpu_fee_beam` function
/// * `az_rad` - The azimuth directions to get the beam response (units of
///   radians)
/// * `za_rad` - The zenith angle directions to get the beam response (units of
///   radians)
/// * `latitude_rad` - A pointer to a telescope latitude to use for the
///   parallactic-angle correction. If the pointer is null, no correction is
///   done.
/// * `iau_order` - A boolean indicating whether the Jones matrix should be
///   arranged [NS-NS NS-EW EW-NS EW-EW] (true) or not (false).
/// * `jones` - A pointer to a buffer with at least `num_unique_tiles *
///   num_unique_fee_freqs * num_azza * 8 * sizeof(FLOAT)` bytes allocated.
///   `FLOAT` is either `float` or `double`, depending on how `hyperbeam` was
///   compiled. The Jones matrix beam responses are written here. This should be
///   set up with the `get_num_unique_tiles` and `get_num_unique_fee_freqs`
///   functions; see the examples for more help.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn fee_calc_jones_gpu(
    gpu_fee_beam: *mut FEEBeamGpu,
    num_azza: u32,
    az_rad: *const GpuFloat,
    za_rad: *const GpuFloat,
    latitude_rad: *const f64,
    iau_order: u8,
    jones: *mut GpuFloat,
) -> i32 {
    let iau_bool = match iau_order {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for iau_order".to_string());
            return 1;
        }
    };

    // Turn the pointers into slices and/or arrays.
    let beam = &*gpu_fee_beam;
    let az = slice::from_raw_parts(az_rad, num_azza as usize);
    let za = slice::from_raw_parts(za_rad, num_azza as usize);
    let results = ArrayViewMut3::from_shape_ptr(
        (
            beam.num_unique_tiles as usize,
            beam.num_unique_freqs as usize,
            num_azza as usize,
        ),
        jones.cast(),
    );
    let latitude_rad = latitude_rad.as_ref().copied();
    ffi_error!(beam.calc_jones_pair_inner(az, za, latitude_rad, iau_bool, results));
    0
}

/// Get beam response Jones matrices for the given directions, using a GPU. The
/// Jones matrix elements for each direction are left on the device (the device
/// pointer is communicated via `d_jones`). Can optionally re-define the X and Y
/// polarisations and apply a parallactic-angle correction; see
/// <https://github.com/MWATelescope/mwa_hyperbeam/blob/main/fee_pols.pdf>
///
/// # Arguments
///
/// * `gpu_fee_beam` - A pointer to a `FEEBeamGpu` struct created with the
///   `new_gpu_fee_beam` function
/// * `az_rad` - The azimuth directions to get the beam response (units of
///   radians)
/// * `za_rad` - The zenith angle directions to get the beam response (units of
///   radians)
/// * `latitude_rad` - A pointer to a telescope latitude to use for the
///   parallactic-angle correction. If the pointer is null, no correction is
///   done.
/// * `iau_order` - A boolean indicating whether the Jones matrix should be
///   arranged [NS-NS NS-EW EW-NS EW-EW] (true) or not (false).
/// * `d_jones` - A pointer to a device buffer with at least `8 *
///   num_unique_tiles * num_unique_fee_freqs * num_azza * sizeof(FLOAT)` bytes
///   allocated. `FLOAT` is either `float` or `double`, depending on how
///   `hyperbeam` was compiled. The Jones matrix beam responses are written
///   here. This should be set up with the `get_num_unique_tiles` and
///   `get_num_unique_fee_freqs` functions; see the examples for more help.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn fee_calc_jones_gpu_device(
    gpu_fee_beam: *mut FEEBeamGpu,
    num_azza: i32,
    az_rad: *const GpuFloat,
    za_rad: *const GpuFloat,
    latitude_rad: *const f64,
    iau_order: u8,
    d_jones: *mut GpuFloat,
) -> i32 {
    let iau_bool = match iau_order {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for iau_order".to_string());
            return 1;
        }
    };

    let beam = &*gpu_fee_beam;
    let az = slice::from_raw_parts(az_rad, num_azza as usize);
    let za = slice::from_raw_parts(za_rad, num_azza as usize);
    let d_az = ffi_error!(DevicePointer::copy_to_device(az));
    let d_za = ffi_error!(DevicePointer::copy_to_device(za));
    let d_latitude_rad = ffi_error!(latitude_rad
        .as_ref()
        .map(|f| DevicePointer::copy_to_device(&[*f as GpuFloat]))
        .transpose());
    ffi_error!(beam.calc_jones_device_pair_inner(
        d_az.get(),
        d_za.get(),
        num_azza,
        d_latitude_rad.map(|p| p.get()).unwrap_or(std::ptr::null()),
        iau_bool,
        d_jones.cast()
    ));
    0
}

/// The same as `calc_jones_gpu_device`, but with the directions already
/// allocated on the device. As with `d_jones`, the precision of the floats
/// depends on how `hyperbeam` was compiled.
///
/// # Arguments
///
/// * `gpu_fee_beam` - A pointer to a `FEEBeamGpu` struct created with the
///   `new_gpu_fee_beam` function
/// * `d_az_rad` - The azimuth directions to get the beam response (units of
///   radians)
/// * `d_za_rad` - The zenith angle directions to get the beam response (units
///   of radians)
/// * `latitude_rad` - A pointer to a telescope latitude to use for the
///   parallactic-angle correction. If the pointer is null, no correction is
///   done.
/// * `iau_order` - A boolean indicating whether the Jones matrix should be
///   arranged [NS-NS NS-EW EW-NS EW-EW] (true) or not (false).
/// * `d_jones` - A pointer to a device buffer with at least `8 *
///   num_unique_tiles * num_unique_fee_freqs * num_azza * sizeof(FLOAT)` bytes
///   allocated. `FLOAT` is either `float` or `double`, depending on how
///   `hyperbeam` was compiled. The Jones matrix beam responses are written
///   here. This should be set up with the `get_num_unique_tiles` and
///   `get_num_unique_fee_freqs` functions; see the examples for more help.
///
/// # Returns
///
/// * An exit code integer. If this is non-zero then an error occurred; the
///   details can be obtained by (1) getting the length of the error string by
///   calling `hb_last_error_length` and (2) calling `hb_last_error_message`
///   with a string buffer with a length at least equal to the error length.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn fee_calc_jones_gpu_device_inner(
    gpu_fee_beam: *mut FEEBeamGpu,
    num_azza: i32,
    d_az_rad: *const GpuFloat,
    d_za_rad: *const GpuFloat,
    d_latitude_rad: *const GpuFloat,
    iau_order: u8,
    d_jones: *mut GpuFloat,
) -> i32 {
    let iau_bool = match iau_order {
        0 => false,
        1 => true,
        _ => {
            update_last_error("A value other than 0 or 1 was used for iau_order".to_string());
            return 1;
        }
    };

    let beam = &*gpu_fee_beam;
    ffi_error!(beam.calc_jones_device_pair_inner(
        d_az_rad,
        d_za_rad,
        num_azza,
        d_latitude_rad,
        iau_bool,
        d_jones.cast()
    ));
    0
}

/// Get a pointer to the tile map. This is necessary to access de-duplicated
/// beam Jones matrices.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
/// # Returns
///
/// * A pointer to the tile map. The const annotation is deliberate; the caller
///   does not own the map.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn get_fee_tile_map(gpu_fee_beam: *mut FEEBeamGpu) -> *const i32 {
    let beam = &*gpu_fee_beam;
    beam.get_tile_map()
}

/// Get a pointer to the tile map. This is necessary to access de-duplicated
/// beam Jones matrices on the device.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
/// # Returns
///
/// * A pointer to the device tile map. The const annotation is deliberate; the
///   caller does not own the map.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn get_fee_device_tile_map(gpu_fee_beam: *mut FEEBeamGpu) -> *const i32 {
    let beam = &*gpu_fee_beam;
    beam.get_device_tile_map()
}

/// Get a pointer to the freq map. This is necessary to access de-duplicated
/// beam Jones matrices.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
/// # Returns
///
/// * A pointer to the freq map. The const annotation is deliberate; the caller
///   does not own the map.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn get_fee_freq_map(gpu_fee_beam: *mut FEEBeamGpu) -> *const i32 {
    let beam = &*gpu_fee_beam;
    beam.get_freq_map()
}

/// Get a pointer to the device freq map. This is necessary to access
/// de-duplicated beam Jones matrices on the device.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
/// # Returns
///
/// * A pointer to the device freq map. The const annotation is deliberate; the
///   caller does not own the map.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn get_fee_device_freq_map(gpu_fee_beam: *mut FEEBeamGpu) -> *const i32 {
    let beam = &*gpu_fee_beam;
    beam.get_device_freq_map()
}

/// Get the number of de-duplicated tiles associated with this `FEEBeamGpu`.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
/// # Returns
///
/// * The number of de-duplicated tiles associated with this `FEEBeamGpu`.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn get_num_unique_fee_tiles(gpu_fee_beam: *mut FEEBeamGpu) -> i32 {
    let beam = &*gpu_fee_beam;
    beam.num_unique_tiles
}

/// Get the number of de-duplicated frequencies associated with this
/// `FEEBeamGpu`.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
/// # Returns
///
/// * The number of de-duplicated frequencies associated with this
///   `FEEBeamGpu`.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn get_num_unique_fee_freqs(gpu_fee_beam: *mut FEEBeamGpu) -> i32 {
    let beam = &*gpu_fee_beam;
    beam.num_unique_freqs
}

/// Free the memory associated with an `FEEBeamGpu` beam.
///
/// # Arguments
///
/// * `gpu_fee_beam` - the pointer to the `FEEBeamGpu` struct.
///
#[cfg(any(feature = "cuda", feature = "hip"))]
#[no_mangle]
pub unsafe extern "C" fn free_gpu_fee_beam(fee_beam: *mut FEEBeamGpu) {
    drop(Box::from_raw(fee_beam));
}
