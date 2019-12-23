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
    pub use super::{ MakeEndian, IntoEndian };
    pub use super::io::{ WriteEndian, ReadEndian, ReadEndianInto };
}

pub trait MakeEndian {
    /// Convert this object from little endian to big endian.
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    #[inline] fn make_be(&mut self);

    /// Convert this object from little endian to big endian.
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    #[inline] fn make_le(&mut self);
}

pub trait IntoEndian {
    #[inline] fn into_le(mut self) -> Self;
    #[inline] fn into_be(mut self) -> Self;
}


impl<T: IntoEndian> MakeEndian for T where T: Copy {
    fn make_be(&mut self) {
        *self = self.into_be();
    }

    fn make_le(&mut self) {
        *self = self.into_le();
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
        impl IntoEndian for $type {
            fn into_le(self) -> Self {
                self.to_le()
            }

            fn into_be(mut self) -> Self {
                self.to_be()
            }
        }
    };
}


call_single_arg_macro_for_each! {
    implement_simple_primitive_endian,
    u16, u32, u64, u128, i16, i32, i64, i128
}


// implement this interface for primitive floats, because they do not have a conversion in `std`
macro_rules! implement_float_primitive_by_transmute {
    ($type: ident, $proxy: ident) => {
        impl IntoEndian for $type {
            fn into_be(self) -> Self {
                unsafe {
                    let proxy: $proxy = *(&self as *const Self as *const $proxy);
                    let proxy_be = proxy.into_be();
                    *(&proxy_be as *const $proxy as *const Self)
                }
            }

            fn into_le(self) -> Self {
                unsafe {
                    let proxy: $proxy = *(&self as *const Self as *const $proxy);
                    let proxy_be = proxy.into_le();
                    *(&proxy_be as *const $proxy as *const Self)
                }
            }
        }
    };
}


implement_float_primitive_by_transmute!(f32, u32);
implement_float_primitive_by_transmute!(f64, u64);

macro_rules! implement_slice_by_element {
    ($type: ident) => {
        impl MakeEndian for [$type] {
            fn make_be(&mut self) {
                if cfg!(target_endian = "little") {
                    for number in self.iter_mut() { // TODO SIMD?
                        number.make_be();
                    }
                }
            }

            fn make_le(&mut self) {
                if cfg!(target_endian = "big") {
                    for number in self.iter_mut() { // TODO SIMD?
                        number.make_le();
                    }
                }
            }
        }
    };
}

call_single_arg_macro_for_each! {
    implement_slice_by_element,
    u16, u32, u64, u128, i16, i32, i64, i128, f64 // no f32
}

impl MakeEndian for [f32] {
    fn make_be(&mut self) {
        #[cfg(target_endian = "little")]
        {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                if is_x86_feature_detected!("avx2") {
                    swap_bytes_avx(self);
                    return;
                }
            }

            // otherwise (no avx2 available)
            for number in self.iter_mut() {
                number.make_be();
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

    fn make_le(&mut self) {
        if cfg!(target_endian = "big") {
            for number in self.iter_mut() { // TODO SIMD?
                number.make_le();
            }
        }
    }
}




pub mod io {
    use super::{ MakeEndian, IntoEndian };
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
        pub unsafe fn value_as_bytes<T>(value: &T) -> &[u8] {
            std::slice::from_raw_parts(
                value as *const T as *const u8,
                std::mem::size_of::<T>()
            )
        }

        #[inline]
        pub unsafe fn value_as_bytes_mut<T>(value: &mut T) ->&mut [u8] {
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
        pub unsafe fn write_value<T>(write: &mut impl Write, value: &T) -> Result<()> {
            write.write_all(value_as_bytes(value))
        }

        #[inline]
        pub unsafe fn read_value<T>(read: &mut impl Read, value: &mut T) -> Result<()> {
            read.read_exact(value_as_bytes_mut(value))
        }
    }

    pub trait WriteEndian<T> {
        #[inline]
        fn write_le(&mut self, value: T) -> Result<()>;

        #[inline]
        fn write_be(&mut self, value: T) -> Result<()>;
    }

    pub trait ReadEndianInto<T> {
        #[inline]
        fn read_le_into(&mut self, value: &mut [T]) -> Result<()>;

        #[inline]
        fn read_be_into(&mut self, value: &mut [T]) -> Result<()>;
    }

    pub trait ReadEndian<T> {
        #[inline] fn read_le(&mut self) -> Result<T>;
        #[inline] fn read_be(&mut self) -> Result<T>;
    }


    impl<W: Write> WriteEndian<i8> for W {
        fn write_le(&mut self, value: i8) -> Result<()> {
            unsafe { bytes::write_slice(self, &[value]) }
        }

        fn write_be(&mut self, value: i8) -> Result<()> {
            unsafe { bytes::write_slice(self, &[value]) }
        }
    }

    impl<W: Write> WriteEndian<&[i8]> for W {
        fn write_le(&mut self, value: &[i8]) -> Result<()> {
            unsafe { bytes::write_slice(self, value) }
        }

        fn write_be(&mut self, value: &[i8]) -> Result<()> {
            unsafe { bytes::write_slice(self, value) }
        }
    }

    impl<R: Read> ReadEndianInto<i8> for R {
        fn read_le_into(&mut self, value: &mut [i8]) -> Result<()> {
            unsafe { bytes::read_slice(self, value) }
        }

        fn read_be_into(&mut self, value: &mut [i8]) -> Result<()> {
            unsafe { bytes::read_slice( self, value) }
        }
    }


    macro_rules! implement_simple_primitive_write {
        ($type: ident) => {
            impl<W: Write> WriteEndian<$type> for W {
                fn write_le(&mut self, mut value: $type) -> Result<()> {
                    value.make_le();
                    unsafe { bytes::write_value(self, &value) }
                }

                fn write_be(&mut self, mut value: $type) -> Result<()> {
                    value.make_be();
                    unsafe { bytes::write_value(self, &value) }
                }
            }

            impl<W: Read> ReadEndian<$type> for W {
                #[inline]
                fn read_le(&mut self) -> Result<$type> {
                    let mut result = $type ::default();
                    unsafe { bytes::read_value(self, &mut result)?; }
                    Ok(result.into_le())
                }

                #[inline]
                fn read_be(&mut self) -> Result<$type> {
                    let mut result = $type ::default();
                    unsafe { bytes::read_value(self, &mut result)?; }
                    Ok(result.into_be())
                }
            }
        };
    }

    call_single_arg_macro_for_each! {
        implement_simple_primitive_write,
        u16, u32, u64, u128, i16, i32, i64, i128, f32, f64
    }



    macro_rules! implement_slice_io {
        ($type: ident) => {
            // this assumes buffered writers are used!
            impl<W: Write> WriteEndian<&[$type]> for W {
                fn write_le(&mut self, value: &[$type]) -> Result<()> {
                    if cfg!(target_endian = "big") {
                        for &number in value { // TODO SIMD?
                            self.write_le(number)?;
                        }

                        Ok(())
                    }
                    else {
                        unsafe { bytes::write_slice(self, value) }
                    }
                }

                fn write_be(&mut self, value: &[$type]) -> Result<()> {
                    if cfg!(target_endian = "little") {
                        for &number in value { // TODO SIMD?
                            self.write_be(number)?;
                        }

                        Ok(())
                    }
                    else {
                        unsafe { bytes::write_slice(self, value) }
                    }
                }
            }

            impl<R: Read> ReadEndianInto<$type> for R {
                fn read_le_into(&mut self, value: &mut [$type]) -> Result<()> {
                    unsafe { bytes::read_slice(self, value)? };
                    value.make_le();
                    Ok(())
                }

                fn read_be_into(&mut self, value: &mut [$type]) -> Result<()> {
                    unsafe { bytes::read_slice(self, value)? };
                    value.make_be();
                    Ok(())
                }
            }
        };
    }

    call_single_arg_macro_for_each! {
        implement_slice_io,
        u16, u32, u64, u128, i16, i32, i64, i128, f32, f64
    }

}

