#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    future_incompatible,
    rust_2018_compatibility,
    rust_2018_idioms,
    clippy::all
)]

// #![doc(html_root_url = "https://docs.rs/half/1.4.0")]

pub mod tests;

pub trait Endian {
    /// Convert this object from little endian to big endian.
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    #[inline]
    fn make_be(&mut self);

    /// Convert this object from little endian to big endian.
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    #[inline]
    fn make_le(&mut self);

    #[inline]
    fn into_le(mut self) -> Self where Self: Sized {
        self.make_le();
        self
    }

    #[inline]
    fn into_be(mut self) -> Self where Self: Sized {
        self.make_be();
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
            fn make_be(&mut self) {
                *self = self.to_be();
            }

            fn make_le(&mut self) {
                *self = self.to_le();
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
        impl Endian for $type {
            fn make_be(&mut self) {
                unsafe { &mut * (self as *mut Self as *mut $proxy) }.make_be();
            }

            fn make_le(&mut self) {
                unsafe { &mut * (self as *mut Self as *mut $proxy) }.make_le();
            }
        }
    };
}


implement_float_primitive_by_transmute!(f32, u32);
implement_float_primitive_by_transmute!(f64, u64);



// implement this interface for slices, because they do not have a conversion in `std`
impl<T: Endian> Endian for [T] {
    fn make_be(&mut self) {
        if cfg!(target_endian = "little") {
            for number in self { // TODO SIMD?
                number.make_be();
            }
        }
    }

    fn make_le(&mut self) {
        if cfg!(target_endian = "big") {
            for number in self { // TODO SIMD?
                number.make_le();
            }
        }
    }
}




pub mod io {
    use super::Endian;
    use std::io::{Read, Write, Result};

    pub trait WriteEndian<T> {
        #[inline]
        fn write_le(&mut self, value: T) -> Result<()>;

        #[inline]
        fn write_be(&mut self, value: T) -> Result<()>;
    }

    pub trait ReadEndian<T> {

        #[inline]
        fn read_le_into(&mut self, value: &mut [T]) -> Result<()>;

        #[inline]
        fn read_be_into(&mut self, value: &mut [T]) -> Result<()>;

        #[inline]
        fn read_le(&mut self) -> Result<T> where T: Default {
            let mut result = [ T::default() ];
            self.read_le_into(&mut result)?;
            let [ result ] = result;
            Ok(result)
        }

        #[inline]
        fn read_be(&mut self) -> Result<T> where T: Default {
            let mut result = [ T::default() ];
            self.read_be_into(&mut result)?;
            let [ result ] = result;
            Ok(result)
        }
    }


    #[inline]
    unsafe fn write_transmuted_bytes<T>(write: &mut impl Write, value: &[T]) -> Result<()> {
        write.write_all(
            std::slice::from_raw_parts(
                value.as_ptr() as *const u8,
                value.len() * std::mem::size_of::<T>()
            )
        )
    }

    #[inline]
    unsafe fn read_transmuted_bytes<T>(read: &mut impl Read, value: &mut [T]) -> Result<()> {
        read.read_exact(
            std::slice::from_raw_parts_mut(
                value.as_mut_ptr() as *mut u8,
                value.len() * std::mem::size_of::<T>()
            )
        )
    }

    macro_rules! implement_simple_primitive_write {
        ($type: ident) => {
            impl<W: Write> WriteEndian<$type> for W {
                fn write_le(&mut self, mut value: $type) -> Result<()> {
                    value.make_le();
                    unsafe { write_transmuted_bytes(self, &[value]) }
                }

                fn write_be(&mut self, mut value: $type) -> Result<()> {
                    value.make_be();
                    unsafe { write_transmuted_bytes(self, &[value]) }
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
                        unsafe { write_transmuted_bytes(self, value) }
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
                        unsafe { write_transmuted_bytes(self, value) }
                    }
                }
            }

            impl<R: Read> ReadEndian<$type> for R {
                fn read_le_into(&mut self, value: &mut [$type]) -> Result<()> {
                    unsafe { read_transmuted_bytes(self, value)? };
                    value.make_le();
                    Ok(())
                }

                fn read_be_into(&mut self, value: &mut [$type]) -> Result<()> {
                    unsafe { read_transmuted_bytes(self, value)? };
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
