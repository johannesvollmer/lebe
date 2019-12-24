#![warn(
    missing_docs,
    trivial_numeric_casts,
    unused_extern_crates, unused_import_braces,
    future_incompatible, rust_2018_compatibility,
    rust_2018_idioms, clippy::all
)]

// #![doc(html_root_url = "https://docs.rs/lebe/0.1.0")]

pub mod tests;

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
        self.convert_current_to_big_endian();
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
    f64 // custom simd f32
}

impl Endian for [f32] {
    fn swap_bytes(&mut self) {
        #[cfg(target_endian = "little")]
        {
            /*
            with simd:
                test read_slice_baseline          ... bench:   2,626,792 ns/iter (+/- 816,840)
                test read_slice_f32_be_byteorder  ... bench:   2,633,617 ns/iter (+/- 571,687)
                test read_slice_f32_be_crate      ... bench:   2,515,297 ns/iter (+/- 1,650,939)
                test read_slice_f32_le_byteorder  ... bench:   2,262,485 ns/iter (+/- 1,460,366)
                test read_slice_f32_le_crate      ... bench:   2,328,342 ns/iter (+/- 851,733)
                test write_slice_baseline         ... bench:   2,064,107 ns/iter (+/- 230,779)
                test write_slice_f32_be_byteorder ... bench:   3,882,385 ns/iter (+/- 1,103,729)
                test write_slice_f32_be_crate     ... bench:   2,546,017 ns/iter (+/- 252,575)
                test write_slice_f32_le_byteorder ... bench:   3,550,125 ns/iter (+/- 388,437)
                test write_slice_f32_le_crate     ... bench:   1,891,302 ns/iter (+/- 594,722)

            without simd:
                test read_slice_baseline          ... bench:   2,337,835 ns/iter (+/- 645,782)
                test read_slice_f32_be_byteorder  ... bench:   2,299,995 ns/iter (+/- 288,691)
                test read_slice_f32_be_crate      ... bench:   2,315,692 ns/iter (+/- 248,317)
                test read_slice_f32_le_byteorder  ... bench:   2,055,587 ns/iter (+/- 377,853)
                test read_slice_f32_le_crate      ... bench:   2,424,600 ns/iter (+/- 949,144)
                test write_slice_baseline         ... bench:   2,284,342 ns/iter (+/- 883,520)
                test write_slice_f32_be_byteorder ... bench:   4,721,595 ns/iter (+/- 2,411,315)
                test write_slice_f32_be_crate     ... bench:   2,560,367 ns/iter (+/- 222,226)
                test write_slice_f32_le_byteorder ... bench:   3,483,840 ns/iter (+/- 508,871)
                test write_slice_f32_le_crate     ... bench:   1,754,372 ns/iter (+/- 141,829)
            */


            // FIX ME this SIMD optimization makes no difference ... why? like, ZERO difference, not even worse
            #[cfg(feature = "simd")]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                if is_x86_feature_detected!("avx2") {
                    swap_bytes_avx(self);
                    return;
                }
            }

            // otherwise (no avx2 available)
            for number in self.iter_mut() {
                number.swap_bytes();
            }

            #[target_feature(enable = "avx2")]
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe fn swap_bytes_avx(slice: &mut [f32]){
                #[cfg(target_arch = "x86")] use std::arch::x86 as mm;
                #[cfg(target_arch = "x86_64")] use std::arch::x86_64 as mm;

                let bytes: &mut [u8] = self::io::bytes::slice_as_bytes_mut(slice);
                let mut chunks = bytes.chunks_exact_mut(32);

                let indices = mm::_mm256_set_epi8(
                    3,2,1,0, 7,6,5,4, 11,10,9,8, 15,14,13,12,
                    3,2,1,0, 7,6,5,4, 11,10,9,8, 15,14,13,12
                );

                for chunk in &mut chunks {
                    let data = mm::_mm256_loadu_si256(chunk.as_ptr() as _);
                    let result = mm::_mm256_shuffle_epi8(data, indices);
                    mm::_mm256_storeu_si256(chunk.as_mut_ptr() as _, result);
                }

                let remainder = chunks.into_remainder();

                { // copy remainder into larger slice, with zeroes at the end
                    let mut last_chunk = [0_u8; 32];
                    last_chunk[0..remainder.len()].copy_from_slice(remainder);
                    let data = mm::_mm256_loadu_si256(last_chunk.as_ptr() as _);
                    let result = mm::_mm256_shuffle_epi8(data, indices);
                    mm::_mm256_storeu_si256(last_chunk.as_mut_ptr() as _, result);
                    remainder.copy_from_slice(&last_chunk[0..remainder.len()]);
                }
            }
        }
    }
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

    pub trait ReadPrimitives: Read {
        primitive_read_fn! { read_little_endian_u8, read_big_endian_u8, u8 }
        primitive_read_fn! { read_little_endian_i8, read_big_endian_i8, i8 }

        primitive_read_fn! { read_little_endian_i16, read_big_endian_i16, i16 }
        primitive_read_fn! { read_little_endian_u16, read_big_endian_u16, u16 }

        primitive_read_fn! { read_little_endian_i32, read_big_endian_i32, i32 }
        primitive_read_fn! { read_little_endian_u32, read_big_endian_u32, u32 }

        primitive_read_fn! { read_little_endian_i64, read_big_endian_i64, i64 }
        primitive_read_fn! { read_little_endian_u64, read_big_endian_u64, u64 }

        primitive_read_fn! { read_little_endian_i128, read_big_endian_i28, i128 }
        primitive_read_fn! { read_little_endian_u128, read_big_endian_u128, u128 }

        primitive_read_fn! { read_little_endian_f32, read_big_endian_f32, f32 }
        primitive_read_fn! { read_little_endian_f64, read_big_endian_f64, f64 }
    }


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
                    unsafe { bytes::read_value(self, value) }
                }

                #[inline]
                fn read_from_big_endian_into(&mut self, value: &mut $type) -> Result<()> {
                    unsafe { bytes::read_value(self, value) }
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
        f32, f64
    }
}

