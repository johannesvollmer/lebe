#[macro_use]
extern crate bencher;

use bencher::Bencher;
use lebe::prelude::*;
use byteorder::{ReadBytesExt, LittleEndian, BigEndian, WriteBytesExt};
use std::io::Read;

const COUNT_8:  usize = 2048 * 32;
const COUNT_16: usize = COUNT_8 / 2;
const COUNT_32: usize = COUNT_8 / 4;
const COUNT_64: usize = COUNT_8 / 8;


fn bytes(count: usize) -> impl Read {
    let vec: Vec<u8> = (0..count).map(|i| (i % 256) as u8).collect();
    std::io::Cursor::new(vec)
}

fn floats(count: usize) -> Vec<f32> {
    (0..count).map(|i| i as f32).collect()
}

fn read_slice_f32_le_crate(bench: &mut Bencher) {
    bench.iter(move ||{
        let mut target = vec![ 0_f32; COUNT_32 ];
        bencher::black_box(bytes(COUNT_8).read_le_into(&mut target)).unwrap();
        bencher::black_box(target);
    })
}

fn read_slice_f32_le_byteorder(bench: &mut Bencher) {
    bench.iter(move ||{
        let mut target = vec![ 0_f32; COUNT_32 ];
        bencher::black_box(bytes(COUNT_8).read_f32_into::<LittleEndian>(&mut target)).unwrap();
        bencher::black_box(target);
    })
}

fn read_slice_f32_be_crate(bench: &mut Bencher) {
    bench.iter(move ||{
        let mut target = vec![ 0_f32; COUNT_32 ];
        bencher::black_box(bytes(COUNT_8).read_be_into(&mut target)).unwrap();
        bencher::black_box(target);
    })
}

fn read_slice_f32_be_byteorder(bench: &mut Bencher) {
    bench.iter(move ||{
        let mut target = vec![ 0_f32; COUNT_32 ];
        bencher::black_box(bytes(COUNT_8).read_f32_into::<BigEndian>(&mut target)).unwrap();
        bencher::black_box(target);
    })
}


fn write_slice_f32_le_crate(bench: &mut Bencher) {
    bench.iter(move ||{
        let data = floats(COUNT_32);
        let mut output = Vec::with_capacity(COUNT_8);

        bencher::black_box(output.write_le(data.as_slice())).unwrap();
        bencher::black_box(output);
    })
}

fn write_slice_f32_le_byteorder(bench: &mut Bencher) {
    bench.iter(move ||{
        let data = floats(COUNT_32);
        let mut output = Vec::with_capacity(COUNT_8);

        for number in data {
            bencher::black_box(output.write_f32::<LittleEndian>(number)).unwrap();
        }

        bencher::black_box(output);
    })
}


fn write_slice_f32_be_crate(bench: &mut Bencher) {
    bench.iter(move ||{
        let data = floats(COUNT_32);
        let mut output = Vec::with_capacity(COUNT_8);

        bencher::black_box(output.write_be(data.as_slice())).unwrap();
        bencher::black_box(output);
    })
}

fn write_slice_f32_be_byteorder(bench: &mut Bencher) {
    bench.iter(move ||{
        let data = floats(COUNT_32);
        let mut output = Vec::with_capacity(COUNT_8);

        for number in data {
            bencher::black_box(output.write_f32::<BigEndian>(number)).unwrap();
        }

        bencher::black_box(output);
    })
}

benchmark_group!(
    benches,
    read_slice_f32_be_byteorder, read_slice_f32_be_crate, read_slice_f32_le_byteorder,
    read_slice_f32_le_crate, write_slice_f32_le_byteorder, write_slice_f32_le_crate,
    write_slice_f32_be_byteorder, write_slice_f32_be_crate
);

benchmark_main!(benches);