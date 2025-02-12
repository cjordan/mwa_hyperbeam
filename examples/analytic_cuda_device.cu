// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Get beam responses using CUDA, but leave the Jones matrices on the device.
//
// hyperbeam lets you run the CUDA code at single- or double-precision. The
// reason is that desktop GPUs have poor double-precision performance.
//
// Compile with something like:
// # Double precision
// cargo build --release --features=cuda
// # or single precision
// cargo build --release --features=cuda,gpu-single
//
// Note the precision you're using; if it's single then supply "-D SINGLE"
// below. Otherwise, no flag is needed.
//
// Compile and run this file with something like:

// nvcc -O3 -D SINGLE -I ../include/ -L ../target/release/ -l mwa_hyperbeam ./analytic_cuda_device.cu -o
// analytic_cuda_device
//
// LD_LIBRARY_PATH=../target/release ./analytic_cuda_device

#include <math.h>
#include <stdio.h>
#include <stdlib.h>

#include <cuComplex.h>

#include "mwa_hyperbeam.h"

#ifdef SINGLE
#define FLOAT     float
#define CREAL     crealf
#define CIMAG     cimagf
#define CUCOMPLEX cuFloatComplex
#define FABS      fabsf
#else
#define FLOAT     double
#define CREAL     creal
#define CIMAG     cimag
#define CUCOMPLEX cuDoubleComplex
#define FABS      fabs
#endif

typedef struct JONES {
    CUCOMPLEX j00;
    CUCOMPLEX j01;
    CUCOMPLEX j10;
    CUCOMPLEX j11;
} JONES;

void handle_hyperbeam_error(char file[], int line_num, const char function_name[]) {
    int err_length = hb_last_error_length();
    char *err = (char *)malloc(err_length * sizeof(char));
    int err_status = hb_last_error_message(err, err_length);
    if (err_status == -1) {
        printf("Something really bad happened!\n");
        exit(EXIT_FAILURE);
    }
    printf("File %s:%d: hyperbeam error in %s: %s\n", file, line_num, function_name, err);

    exit(EXIT_FAILURE);
}

__global__ void use_hyperbeam_values(JONES *d_jones, const int *d_tile_map, int num_tiles, int num_freqs,
                                     int num_directions) {
    int i_tile = blockIdx.y;
    int i_freq = blockIdx.z;
    if (i_tile >= num_tiles)
        return;
    if (i_freq >= num_freqs)
        return;
    int i_dir = blockDim.x * blockIdx.x + threadIdx.x;
    if (i_dir >= num_directions)
        return;

    // For *this tile* and *this frequency*, access the de-duplicated beam
    // response.
    int i_row = d_tile_map[i_tile];
    JONES jones = d_jones[((num_directions * num_freqs * i_row) + num_directions * i_freq) + i_dir];

    // To test that the right response is paired with the right tile, assert
    // here.
    if (i_tile == 0 || i_tile == 1) {
        // Tiles 0 and 1 should have non-zero responses.
        if (FABS(jones.j00.x) < 1e-8 && FABS(jones.j00.y) < 1e-8 && FABS(jones.j01.x) < 1e-8 &&
            FABS(jones.j01.y) < 1e-8 && FABS(jones.j10.x) < 1e-8 && FABS(jones.j10.y) < 1e-8 &&
            FABS(jones.j11.x) < 1e-8 && FABS(jones.j11.y) < 1e-8) {
            printf("uh oh, bad example for tile 0/1\n");
        }
    } else {
        // Tile 2 should *only* have zeros.
        if (FABS(jones.j00.x) > 1e-6 || FABS(jones.j00.y) > 1e-6) {
            printf("uh oh, bad example for tile 2\n");
        }
        if (FABS(jones.j01.x) > 1e-6 || FABS(jones.j01.y) > 1e-6) {
            printf("uh oh, bad example for tile 2\n");
        }
        if (FABS(jones.j10.x) > 1e-6 || FABS(jones.j10.y) > 1e-6) {
            printf("uh oh, bad example for tile 2\n");
        }
        if (FABS(jones.j11.x) > 1e-6 || FABS(jones.j11.y) > 1e-6) {
            printf("uh oh, bad example for tile 2\n");
        }
    }
}

int main(int argc, char *argv[]) {
    // Get a new beam object from hyperbeam.
    AnalyticBeam *beam;
    char rts_style = 1;                  // 1 or RTS style, 0 for mwa_pb
    double *dipole_height_metres = NULL; // Point to a valid float if you want a custom height
    if (new_analytic_beam(rts_style, dipole_height_metres, &beam))
        handle_hyperbeam_error(__FILE__, __LINE__, "new_analytic_beam");

    // Set up our telescope array. Here, we are using three tiles, but there are
    // two distinct types (one has all dipoles active, the other they're all
    // "dead"). The first 16 values are the first tile, the second 16 for the
    // second tile, etc. When giving 16 values per tile, each value is used for
    // the X and Y dipoles. It's possible to supply 32 values per tile; in that
    // case, the first 16 values are for the X dipoles and the last 16 are for
    // the Y dipoles.

    // Delays and amps correspond to dipoles in the "M&C order". See
    // https://wiki.mwatelescope.org/pages/viewpage.action?pageId=48005139) for
    // more info. Amps refer to dipole gains, and are usually set to 1 or 0 (if
    // a dipole is dead).
    int num_tiles = 3;
    unsigned delays[48] = {3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0,
                           3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0};
    double dip_amps[48] = {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                           1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
    // Number of specified amps per tile.
    int num_amps = 16;

    unsigned freqs_hz[2] = {(unsigned)150e6, (unsigned)200e6};
    int num_freqs = 2;

    // Should we normalise the beam response?
    int norm_to_zenith = 1;

    // Now get a new CUDA analytic beam object.
    AnalyticBeamGpu *gpu_beam;
    if (new_gpu_analytic_beam(beam, delays, dip_amps, num_tiles, num_amps, &gpu_beam))
        handle_hyperbeam_error(__FILE__, __LINE__, "new_gpu_analytic_beam");

    // Set up the directions to get the beam responses.
    size_t num_directions = 100000;
    FLOAT *az = (FLOAT *)malloc(num_directions * sizeof(FLOAT));
    FLOAT *za = (FLOAT *)malloc(num_directions * sizeof(FLOAT));
    for (int i = 0; i < num_directions; i++) {
        az[i] = (-170.0 + i * 340.0 / num_directions) * M_PI / 180.0;
        za[i] = (10.0 + i * 70.0 / num_directions) * M_PI / 180.0;
    }
    // MWA latitude
    double latitude_rad = -0.4660608448386394;

    // Allocate our device memory for the beam responses.
    JONES *d_jones;
    // We need the number of unique tiles and unique frequencies. hyperbeam
    // de-duplicates tiles and frequencies to go faster. Cast the returned ints
    // into size_t just in case we're hitting big numbers.
    size_t num_unique_tiles = (size_t)get_num_unique_analytic_tiles(gpu_beam);
    cudaMalloc(&d_jones, num_unique_tiles * num_freqs * num_directions * sizeof(JONES));
    // hyperbeam expects a pointer to our FLOAT macro. Casting the pointer works
    // fine.
    if (analytic_calc_jones_gpu_device(gpu_beam, num_directions, az, za, num_freqs, freqs_hz, latitude_rad,
                                       norm_to_zenith, (FLOAT *)d_jones))
        handle_hyperbeam_error(__FILE__, __LINE__, "analytic_calc_jones_gpu_device");

    // The beam responses are now on the device. Let's launch our own kernel and
    // interface with the values. This kernel prints messages if the values are
    // not what was expected. We need to have the tile map to interface with the
    // beam responses.
    const int *d_tile_map = get_analytic_device_tile_map(gpu_beam);

    dim3 gridDim, blockDim;
    blockDim.x = 128;
    gridDim.x = (int)ceil((double)num_directions / (double)blockDim.x);
    gridDim.y = num_tiles; // The total number of tiles, not the unique count.
    gridDim.z = num_freqs; // The total number of freqs, not the unique count.
    use_hyperbeam_values<<<gridDim, blockDim>>>(d_jones, d_tile_map, num_tiles, num_freqs, num_directions);
    // Check that our kernel had no issues.
    cudaError_t cuda_err_code = cudaGetLastError();
    if (cuda_err_code != cudaSuccess) {
        fprintf(stderr, "Error with use_hyperbeam_values kernel: %s\n", cudaGetErrorString(cuda_err_code));
        exit(cuda_err_code);
    }

    // // Copy the values to host and inspect them. N.B. There are Jones matrices
    // // for each *unique* tile (2), each frequency and direction.
    // size_t s = 2 * num_freqs * num_directions * sizeof(JONES);
    // JONES *host_jones = (JONES *)malloc(s);
    // cudaMemcpy(host_jones, d_jones, s, cudaMemcpyDeviceToHost);
    // for (int i = 0; i < s / sizeof(JONES); i++) {
    //     printf("% 4d [%f %f  %f %f\n", i, host_jones[i].j00.x, host_jones[i].j00.y, host_jones[i].j01.x,
    //            host_jones[i].j01.y);
    //     printf("      %f %f  %f %f]\n", host_jones[i].j10.x, host_jones[i].j10.y, host_jones[i].j11.x,
    //            host_jones[i].j11.y);
    // }
    // free(host_jones);

    // Free the device memory.
    cudaFree(d_jones);
    // Free the beam objects - we must use special functions to do this.
    free_gpu_analytic_beam(gpu_beam);
    free_analytic_beam(beam);

    printf("If there aren't any messages above, then all worked as expected.\n");

    return EXIT_SUCCESS;
}
