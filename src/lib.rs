#![warn(
    missing_docs,
    trivial_numeric_casts,
    unused_extern_crates, unused_import_braces,
    future_incompatible, rust_2018_compatibility,
    rust_2018_idioms, clippy::all
)]

// #![doc(html_root_url = "https://docs.rs/lebe/0.1.0")]

pub mod prelude {
    pub use super::Endian;
    pub use super::io::{ WriteEndian, ReadEndian, ReadPrimitives };
}

pub trait Endian {
    #[inline]
    fn swap_bytes(&mut self);

    #[inline] fn convert_current_to_little_endian(&mut self) {
        #[cfg(target_endian = "big")] {
            self.swap_bytes();
        }
    }

    #[inline] fn convert_current_to_big_endian(&mut self) {
        #[cfg(target_endian = "little")] {
            self.swap_bytes();
        }
    }

    #[inline] fn convert_little_endian_to_current(&mut self) {
        #[cfg(target_endian = "big")] {
            self.swap_bytes();
        }
    }

    #[inline] fn convert_big_endian_to_current(&mut self) {
        #[cfg(target_endian = "little")] {
            self.swap_bytes();
        }
    }

    #[inline] fn from_current_into_little_endian(mut self) -> Self where Self: Sized {
        self.convert_current_to_little_endian();
        self
    }

    #[inline] fn from_current_into_big_endian(mut self) -> Self where Self: Sized {
        self.convert_current_to_big_endian();
        self
    }

    #[inline] fn from_little_endian_into_current(mut self) -> Self where Self: Sized {
        self.convert_little_endian_to_current();
        self
    }

    #[inline] fn from_big_endian_into_current(mut self) -> Self where Self: Sized {
        self.convert_big_endian_to_current();
        self
    }
}


// call a macro for each argument
macro_rules! call_single_arg_macro_for_each {
    ($macro: ident, $( $arguments: ident ),* ) => {
        $( $macro! { $arguments }  )*
    };
}

// implement this interface for primitive signed and unsigned integers
macro_rules! implement_simple_primitive_endian {
    ($type: ident) => {
        impl Endian for $type {
            fn swap_bytes(&mut self) {
                *self = $type::swap_bytes(*self);
            }
        }
    };
}


call_single_arg_macro_for_each! {
    implement_simple_primitive_endian,
    u16, u32, u64, u128, i16, i32, i64, i128
}

// no-op implementations
impl Endian for u8 { fn swap_bytes(&mut self) {} }
impl Endian for i8 { fn swap_bytes(&mut self) {} }
impl Endian for [u8] { fn swap_bytes(&mut self) {} }
impl Endian for [i8] { fn swap_bytes(&mut self) {} }

// implement this interface for primitive floats, because they do not have a conversion in `std`
macro_rules! implement_float_primitive_by_transmute {
    ($type: ident, $proxy: ident) => {
        impl Endian for $type {
            fn swap_bytes(&mut self) {
                unsafe {
                    let proxy: &mut $proxy = &mut *(self as *mut Self as *mut $proxy);
                    proxy.swap_bytes();
                }
            }
        }
    };
}


implement_float_primitive_by_transmute!(f32, u32);
implement_float_primitive_by_transmute!(f64, u64);

macro_rules! implement_slice_by_element {
    ($type: ident) => {
        impl Endian for [$type] {
            fn swap_bytes(&mut self) {
                for number in self.iter_mut() { // TODO SIMD?
                    number.swap_bytes();
                }
            }
        }
    };
}

call_single_arg_macro_for_each! {
    implement_slice_by_element,
    u16, u32, u64, u128,
    i16, i32, i64, i128,
    f64, f32
}

pub mod io {
    use super::Endian;
    use std::io::{Read, Write, Result};

    pub mod bytes {
        use std::io::{Read, Write, Result};

        #[inline]
        pub unsafe fn slice_as_bytes<T>(value: &[T]) -> &[u8] {
            std::slice::from_raw_parts(
                value.as_ptr() as *const u8,
                value.len() * std::mem::size_of::<T>()
            )
        }

        #[inline]
        pub unsafe fn slice_as_bytes_mut<T>(value: &mut [T]) -> &mut [u8] {
            std::slice::from_raw_parts_mut(
                value.as_mut_ptr() as *mut u8,
                value.len() * std::mem::size_of::<T>()
            )
        }

        #[inline]
        pub unsafe fn value_as_bytes<T: Sized>(value: &T) -> &[u8] {
            std::slice::from_raw_parts(
                value as *const T as *const u8,
                std::mem::size_of::<T>()
            )
        }

        #[inline]
        pub unsafe fn value_as_bytes_mut<T: Sized>(value: &mut T) ->&mut [u8] {
            std::slice::from_raw_parts_mut(
                value as *mut T as *mut u8,
                std::mem::size_of::<T>()
            )
        }

        #[inline]
        pub unsafe fn write_slice<T>(write: &mut impl Write, value: &[T]) -> Result<()> {
            write.write_all(slice_as_bytes(value))
        }

        #[inline]
        pub unsafe fn read_slice<T>(read: &mut impl Read, value: &mut [T]) -> Result<()> {
            read.read_exact(slice_as_bytes_mut(value))
        }

        #[inline]
        pub unsafe fn write_value<T: Sized>(write: &mut impl Write, value: &T) -> Result<()> {
            write.write_all(value_as_bytes(value))
        }

        #[inline]
        pub unsafe fn read_value<T: Sized>(read: &mut impl Read, value: &mut T) -> Result<()> {
            read.read_exact(value_as_bytes_mut(value))
        }
    }

    pub trait WriteEndian<T: ?Sized> {
        #[inline]
        fn write_as_little_endian(&mut self, value: &T) -> Result<()>;

        #[inline]
        fn write_as_big_endian(&mut self, value: &T) -> Result<()>;
    }

    pub trait ReadEndian<T: ?Sized> {
        #[inline]
        fn read_from_little_endian_into(&mut self, value: &mut T) -> Result<()>;

        #[inline]
        fn read_from_big_endian_into(&mut self, value: &mut T) -> Result<()>;

        #[inline]
        fn read_from_little_endian(&mut self) -> Result<T> where T: Sized + Default {
            let mut value = T::default();
            self.read_from_little_endian_into(&mut value)?;
            Ok(value)
        }

        #[inline]
        fn read_from_big_endian(&mut self) -> Result<T> where T: Sized + Default {
            let mut value = T::default();
            self.read_from_big_endian_into(&mut value)?;
            Ok(value)
        }
    }

    macro_rules! primitive_read_fn {
        ($l_name: ident, $b_name: ident, $type: ident) => {
            fn $l_name (&mut self) -> Result<$type> where Self: ReadEndian<$type> { self.read_from_little_endian() }
            fn $b_name (&mut self) -> Result<$type> where Self: ReadEndian<$type>  { self.read_from_big_endian() }
        };
    }

    pub trait ReadPrimitives {
        primitive_read_fn! { read_u8_from_little_endian, read_u8_from_big_endian, u8 }
        primitive_read_fn! { read_i8_from_little_endian, read_i8_from_big_endian, i8 }

        primitive_read_fn! { read_u16_from_little_endian, read_u16_from_big_endian, u16 }
        primitive_read_fn! { read_i16_from_little_endian, read_i16_from_big_endian, i16 }

        primitive_read_fn! { read_u32_from_little_endian, read_u32_from_big_endian, u32 }
        primitive_read_fn! { read_i32_from_little_endian, read_i32_from_big_endian, i32 }

        primitive_read_fn! { read_u64_from_little_endian, read_u64_from_big_endian, u64 }
        primitive_read_fn! { read_i64_from_little_endian, read_i64_from_big_endian, i64 }

        primitive_read_fn! { read_u128_from_little_endian, read_u128_from_big_endian, u128 }
        primitive_read_fn! { read_i128_from_little_endian, read_i128_from_big_endian, i128 }

        primitive_read_fn! { read_f32_from_little_endian, read_f32_from_big_endian, f32 }
        primitive_read_fn! { read_f64_from_little_endian, read_f64_from_big_endian, f64 }
    }

    impl<R: Read> ReadPrimitives for R { }


    macro_rules! implement_simple_primitive_write {
        ($type: ident) => {
            impl<W: Write> WriteEndian<$type> for W {
                fn write_as_little_endian(&mut self, value: &$type) -> Result<()> {
                    unsafe { bytes::write_value(self, &value.from_current_into_little_endian()) }
                }

                fn write_as_big_endian(&mut self, value: &$type) -> Result<()> {
                    unsafe { bytes::write_value(self, &value.from_current_into_big_endian()) }
                }
            }

            impl<R: Read> ReadEndian<$type> for R {
                #[inline]
                fn read_from_little_endian_into(&mut self, value: &mut $type) -> Result<()> {
                    unsafe { bytes::read_value(self, value)?; }
                    value.convert_little_endian_to_current();
                    Ok(())
                }

                #[inline]
                fn read_from_big_endian_into(&mut self, value: &mut $type) -> Result<()> {
                    unsafe { bytes::read_value(self, value)?; }
                    value.convert_big_endian_to_current();
                    Ok(())
                }
            }
        };
    }

    call_single_arg_macro_for_each! {
        implement_simple_primitive_write,
        u8, u16, u32, u64, u128,
        i8, i16, i32, i64, i128,
        f32, f64
    }


    macro_rules! implement_slice_io {
        ($type: ident) => {
            impl<W: Write> WriteEndian<[$type]> for W {
                fn write_as_little_endian(&mut self, value: &[$type]) -> Result<()> {
                    #[cfg(target_endian = "big")] {
                        for number in value { // TODO SIMD!
                            self.write_as_little_endian(number)?;
                        }
                    }

                    // else write whole slice
                    #[cfg(target_endian = "little")]
                    unsafe { bytes::write_slice(self, value)?; }

                    Ok(())
                }

                fn write_as_big_endian(&mut self, value: &[$type]) -> Result<()> {
                    #[cfg(target_endian = "little")] {
                        for number in value { // TODO SIMD!
                            self.write_as_big_endian(number)?;
                        }
                    }

                    // else write whole slice
                    #[cfg(target_endian = "big")]
                    unsafe { bytes::write_slice(self, value)?; }

                    Ok(())
                }
            }

            impl<R: Read> ReadEndian<[$type]> for R {
                fn read_from_little_endian_into(&mut self, value: &mut [$type]) -> Result<()> {
                    unsafe { bytes::read_slice(self, value)? };
                    value.convert_little_endian_to_current();
                    Ok(())
                }

                fn read_from_big_endian_into(&mut self, value: &mut [$type]) -> Result<()> {
                    unsafe { bytes::read_slice(self, value)? };
                    value.convert_big_endian_to_current();
                    Ok(())
                }
            }
        };
    }

    call_single_arg_macro_for_each! {
        implement_slice_io,
        u8, u16, u32, u64, u128,
        i8, i16, i32, i64, i128,
        f64, f32
    }


    /*impl<R: Read> ReadEndian<[f32]> for R {
        fn read_from_little_endian_into(&mut self, value: &mut [f32]) -> Result<()> {
            unsafe { bytes::read_slice(self, value)? };
            value.convert_little_endian_to_current();
            Ok(())
        }

        fn read_from_big_endian_into(&mut self, value: &mut [f32]) -> Result<()> {
            unsafe { bytes::read_slice(self, value)? };
            value.convert_big_endian_to_current();
            Ok(())
        }
    }

    impl<W: Write> WriteEndian<[f32]> for W {
        fn write_as_big_endian(&mut self, value: &[f32]) -> Result<()> {
            if cfg!(target_endian = "little") {

                // FIX ME this SIMD optimization makes no difference ... why? like, ZERO difference, not even worse
//                #[cfg(feature = "simd")]
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                unsafe {
                    if is_x86_feature_detected!("avx2") {
                        write_bytes_avx(self, value);
                        return Ok(());
                    }
                }

                // otherwise (no avx2 available)
//                for number in value {
//                    self.write_as_little_endian(number);
//                }
//
//                return Ok(());
                unimplemented!();

                #[target_feature(enable = "avx2")]
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                unsafe fn write_bytes_avx(write: &mut impl Write, slice: &[f32]) -> Result<()> {
                    #[cfg(target_arch = "x86")] use std::arch::x86 as mm;
                    #[cfg(target_arch = "x86_64")] use std::arch::x86_64 as mm;

                    let bytes: &[u8] = crate::io::bytes::slice_as_bytes(slice);
                    let mut chunks = bytes.chunks_exact(32);

                    let indices = mm::_mm256_set_epi8(
                        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,
                        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15
//                        3,2,1,0, 7,6,5,4, 11,10,9,8, 15,14,13,12,
//                        3,2,1,0, 7,6,5,4, 11,10,9,8, 15,14,13,12
                    );

                    for chunk in &mut chunks {
                        let data = mm::_mm256_loadu_si256(chunk.as_ptr() as _);
                        let result = mm::_mm256_shuffle_epi8(data, indices);
                        let mut out = [0_u8; 32];
                        mm::_mm256_storeu_si256(out.as_mut_ptr() as _, result);
                        write.write_all(&out)?;
                    }

                    let remainder = chunks.remainder();

                    { // copy remainder into larger slice, with zeroes at the end
                        let mut last_chunk = [0_u8; 32];
                        last_chunk[0..remainder.len()].copy_from_slice(remainder);
                        let data = mm::_mm256_loadu_si256(last_chunk.as_ptr() as _);
                        let result = mm::_mm256_shuffle_epi8(data, indices);
                        mm::_mm256_storeu_si256(last_chunk.as_mut_ptr() as _, result);
                        write.write_all(&last_chunk[0..remainder.len()])?;
                    }

                    Ok(())
                }
            }

            else {
                unsafe { bytes::write_slice(self, value)?; }
                Ok(())
            }
        }

        fn write_as_little_endian(&mut self, value: &[f32]) -> Result<()> {
            for number in value {
                self.write_as_little_endian(number)?;
            }

            Ok(())
        }
    }*/
}

