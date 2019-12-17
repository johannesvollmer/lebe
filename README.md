# LEBE
Dead simple endianness conversion on primitives, slices, Read, and Write.
As a consequence, this crate simplifies writing primitive types to
Read and Write objects, which only accept byte buffers per default.

# Usage

Convert slices in-place.
```rust
    use lebe::Endian;
    
    fn main(){
        let mut numbers: &[i32] = &[ 32, 102, 420, 594 ];
        numbers.make_le();
    }
```

Write slices.
```rust
    use lebe::io::WriteEndian;
    
    fn main(){
        let numbers: &[i32] = &[ 32, 102, 420, 594 ];
        
        let mut output_bytes: Vec<u8> = Vec::new();
        output_bytes.write_le(numbers).unwrap();
    }
```

Read numbers.
```rust
    use lebe::io::ReadEndian;
    
    fn main(){
        let mut input_bytes: &[u8] = &[ 3, 244 ];
        let number: u16 = input_bytes.read_le().unwrap();
    }
```

Read slices.
```rust
    use lebe::io::ReadEndian;
    
    fn main(){
        let mut numbers: &[i32] = &[ 0; 2 ];
        
        let mut input_bytes: &[u8] = &[ 0, 3, 244, 1, 0, 3, 244, 1 ];
        input_bytes.read_le_into(&mut numbers).unwrap();
    }
```


# Why not use [byteorder](https://crates.io/crates/byteorder)?
This crate supports batch-writing slices with native speed 
where the os has the matching endianness. Writing slices must be done
manually in `byteorder`, and may be slower than expected.

# Why not use [endianness](https://crates.io/crates/endianness)?
This crate has no runtim costs, just as byteorder,

# Why not use this crate?
This crate requires a fairly up-to-date rust version, 
which not all projects can support.



# Fun Facts
LEBE is made up from 'le' for little endian and 'be' for big endian.
If you say that word using english pronounciation, 
a german might think you said the german word for 'love'.