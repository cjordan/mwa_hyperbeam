/* automatically generated by rust-bindgen 0.68.1 */

pub const ANALYTIC_TYPE_MWA_PB: ANALYTIC_TYPE = 0;
pub const ANALYTIC_TYPE_RTS: ANALYTIC_TYPE = 1;
pub type ANALYTIC_TYPE = ::std::os::raw::c_uint;
extern "C" {
    pub fn gpu_analytic_calc_jones(
        at: ANALYTIC_TYPE,
        dipole_height_m: f32,
        d_azs: *const f32,
        d_zas: *const f32,
        num_directions: ::std::os::raw::c_int,
        d_freqs_hz: *const ::std::os::raw::c_uint,
        num_freqs: ::std::os::raw::c_int,
        d_delays: *const f32,
        d_amps: *const f32,
        num_tiles: ::std::os::raw::c_int,
        latitude_rad: f32,
        norm_to_zenith: u8,
        d_results: *mut ::std::os::raw::c_void,
    ) -> *const ::std::os::raw::c_char;
}
