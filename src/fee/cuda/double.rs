/* automatically generated by rust-bindgen 0.59.1 */

pub type __int8_t = ::std::os::raw::c_schar;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FEECoeffs {
    pub x_q1_accum: *mut f64,
    pub x_q2_accum: *mut f64,
    pub x_m_accum: *mut i8,
    pub x_n_accum: *mut i8,
    pub x_m_signs: *mut i8,
    pub x_n_max: *mut ::std::os::raw::c_uchar,
    pub x_lengths: *mut ::std::os::raw::c_int,
    pub x_offsets: *mut ::std::os::raw::c_int,
    pub y_q1_accum: *mut f64,
    pub y_q2_accum: *mut f64,
    pub y_m_accum: *mut i8,
    pub y_n_accum: *mut i8,
    pub y_m_signs: *mut i8,
    pub y_n_max: *mut ::std::os::raw::c_uchar,
    pub y_lengths: *mut ::std::os::raw::c_int,
    pub y_offsets: *mut ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_FEECoeffs() {
    assert_eq!(
        ::std::mem::size_of::<FEECoeffs>(),
        128usize,
        concat!("Size of: ", stringify!(FEECoeffs))
    );
    assert_eq!(
        ::std::mem::align_of::<FEECoeffs>(),
        8usize,
        concat!("Alignment of ", stringify!(FEECoeffs))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_q1_accum as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_q1_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_q2_accum as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_q2_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_m_accum as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_m_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_n_accum as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_n_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_m_signs as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_m_signs)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_n_max as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_n_max)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_lengths as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_lengths)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).x_offsets as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(x_offsets)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_q1_accum as *const _ as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_q1_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_q2_accum as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_q2_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_m_accum as *const _ as usize },
        80usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_m_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_n_accum as *const _ as usize },
        88usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_n_accum)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_m_signs as *const _ as usize },
        96usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_m_signs)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_n_max as *const _ as usize },
        104usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_n_max)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_lengths as *const _ as usize },
        112usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_lengths)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FEECoeffs>())).y_offsets as *const _ as usize },
        120usize,
        concat!(
            "Offset of field: ",
            stringify!(FEECoeffs),
            "::",
            stringify!(y_offsets)
        )
    );
}
extern "C" {
    pub fn cuda_calc_jones(
        d_azs: *const f64,
        d_zas: *const f64,
        num_directions: ::std::os::raw::c_int,
        d_coeffs: *const FEECoeffs,
        num_coeffs: ::std::os::raw::c_int,
        norm_jones: *const ::std::os::raw::c_void,
        parallactic: i8,
        d_results: *mut ::std::os::raw::c_void,
        error_str: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
