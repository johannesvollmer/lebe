# LEBE
Dead simple endianness conversion on primitives, slices, Read, and Write.
As a consequence, this crate simplifies writing primitive types to
Read and Write objects, which only accept byte buffers per default.

# Why not use [byteorder](https://crates.io/crates/byteorder)?
This crate supports batch-writing slices with native speed 
where the os has the matching endianness. Writing slices must be done
manually in `byteorder`, and may be slower than expected.

# Why not use [endianness](https://crates.io/crates/endianness)?
This crate has no runtim costs, just as byteorder,

# Why not use this crate?
This crate requires a fairly up-to-date rust version, 
which not all projects can support.

# Usage
```rust
    use lebe::io::{ ReadEndian, WriteEndian };
    
    fn main(){
        // a slice of 12 integers (zeroes for now)
        let mut numbers = [ 0_i32; 12 ];
        
        { // let's read those numbers from a file
        
            // a file to read from
            let mut read = std::fs::File::open("le-file").unwrap();
            
            // read 12 integers from a little-endian encoded file
            read.read_le_into(&mut numbers).unwrap();
        }
        
        println!("Mason, what do these numbers mean: {:?}!?", numbers);
        
        let mut output = std::fs::File::create("be-file").unwrap();
        output.write_be(&numbers).unwrap();
    }
```


# Fun Facts
LEBE is made up from 'le' for little endian and 'be' for big endian.
If you say that word using english pronounciation, 
a german might think you said the german word for 'love'.