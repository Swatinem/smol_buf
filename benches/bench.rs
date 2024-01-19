use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smol_str::Str24;

fn inline_eq_inline(c: &mut Criterion) {
    let inline_1 = Str24::new_inline("some inline text");
    let inline_2 = inline_1.clone();
    c.bench_function("inline == inline", |b| {
        b.iter(|| {
            assert_eq!(black_box(&inline_1), black_box(&inline_2));
        })
    });
}

fn inline_ne_inline(c: &mut Criterion) {
    let inline_1 = Str24::new_inline("some inline text");
    let inline_2 = Str24::new_inline("another inline text");
    c.bench_function("inline != inline", |b| {
        b.iter(|| {
            assert_ne!(black_box(&inline_1), black_box(&inline_2));
        })
    });
}

fn static_ptr_eq_static(c: &mut Criterion) {
    let static_1 = Str24::new_static("some very long and even longer static text");
    let static_2 = static_1.clone();
    c.bench_function("static ptr::eq static", |b| {
        b.iter(|| {
            assert_eq!(black_box(&static_1), black_box(&static_2));
        })
    });
}

fn static_eq_static(c: &mut Criterion) {
    let static_1 = Str24::new_static("some very long and even longer static text");
    let static_2 =
        Str24::new_static(String::from("some very long and even longer static text").leak());
    c.bench_function("static == static", |b| {
        b.iter(|| {
            assert_eq!(black_box(&static_1), black_box(&static_2));
        })
    });
}

fn static_ne_static(c: &mut Criterion) {
    let static_1 = Str24::new_static("some very long and even longer static text");
    let static_2 = Str24::new_static("another very long and even longer static text");
    c.bench_function("static != static", |b| {
        b.iter(|| {
            assert_ne!(black_box(&static_1), black_box(&static_2));
        })
    });
}

fn heap_ptr_eq_heap(c: &mut Criterion) {
    let heap1: Str24 = std::iter::repeat('0').take(64).collect();
    let heap2 = heap1.clone();
    c.bench_function("heap ptr_eq heap", |b| {
        b.iter(|| {
            assert_eq!(black_box(&heap1), black_box(&heap2));
        })
    });
}

fn heap_eq_heap(c: &mut Criterion) {
    let heap1: Str24 = std::iter::repeat('0').take(64).collect();
    let heap2: Str24 = std::iter::repeat('0').take(64).collect();
    c.bench_function("heap == heap", |b| {
        b.iter(|| {
            assert_eq!(black_box(&heap1), black_box(&heap2));
        })
    });
}

fn heap_ne_heap(c: &mut Criterion) {
    let heap1: Str24 = std::iter::repeat('0').take(64).collect();
    let heap2: Str24 = std::iter::repeat('1').take(64).collect();
    c.bench_function("heap != heap", |b| {
        b.iter(|| {
            assert_ne!(black_box(&heap1), black_box(&heap2));
        })
    });
}

fn inline_ne_static(c: &mut Criterion) {
    let inline = Str24::new_inline("some text");
    let static_str = Str24::new_static("a very long and even longer static text");
    c.bench_function("inline != static", |b| {
        b.iter(|| {
            assert_ne!(black_box(&inline), black_box(&static_str));
        })
    });
}

fn inline_ne_heap(c: &mut Criterion) {
    let inline = Str24::new_inline("some text");
    let heap: Str24 = std::iter::repeat('0').take(64).collect();
    c.bench_function("inline != heap", |b| {
        b.iter(|| {
            assert_ne!(black_box(&inline), black_box(&heap));
        })
    });
}

criterion_group!(
    name = eq;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(2));
    targets = inline_eq_inline, static_ptr_eq_static, static_eq_static, heap_ptr_eq_heap, heap_eq_heap
);
criterion_group!(
    name = ne;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(2));
    targets = inline_ne_inline, static_ne_static, heap_ne_heap, inline_ne_static, inline_ne_heap
);
criterion_main!(eq, ne);

/*
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

#[divan::bench]
fn inline_eq_inline(bencher: Bencher) {
    let inline_1 = Str24::new_inline("some inline text");
    let inline_2 = inline_1.clone();
    bencher.bench(|| {
        assert_eq!(black_box(&inline_1), black_box(&inline_2));
    })
}

#[divan::bench]
fn inline_ne_inline(bencher: Bencher) {
    let inline_1 = Str24::new_inline("some inline text");
    let inline_2 = Str24::new_inline("another inline text");
    bencher.bench(|| {
        assert_ne!(black_box(&inline_1), black_box(&inline_2));
    })
}

#[divan::bench]
fn static_ptr_eq_static(bencher: Bencher) {
    let static_1 = Str24::new_static("some static text");
    let static_2 = static_1.clone();
    bencher.bench(|| {
        assert_eq!(black_box(&static_1), black_box(&static_2));
    })
}

#[divan::bench]
fn static_eq_static(bencher: Bencher) {
    let static_1 = Str24::new_static("some static text");
    let static_2 = Str24::new_static(String::from("some static text").leak());
    bencher.bench(|| {
        assert_eq!(black_box(&static_1), black_box(&static_2));
    })
}

#[divan::bench]
fn static_ne_static(bencher: Bencher) {
    let static_1 = Str24::new_static("some static text");
    let static_2 = Str24::new_static("another static text");
    bencher.bench(|| {
        assert_ne!(black_box(&static_1), black_box(&static_2));
    })
}

#[divan::bench]
fn heap_ptr_eq_heap(bencher: Bencher) {
    let heap1: Str24 = std::iter::repeat('0').take(64).collect();
    let heap2 = heap1.clone();
    bencher.bench(|| {
        assert_eq!(black_box(&heap1), black_box(&heap2));
    })
}

#[divan::bench]
fn heap_eq_heap(bencher: Bencher) {
    let heap1: Str24 = std::iter::repeat('0').take(64).collect();
    let heap2: Str24 = std::iter::repeat('0').take(64).collect();
    bencher.bench(|| {
        assert_eq!(black_box(&heap1), black_box(&heap2));
    })
}

#[divan::bench]
fn heap_ne_heap(bencher: Bencher) {
    let heap1: Str24 = std::iter::repeat('0').take(64).collect();
    let heap2: Str24 = std::iter::repeat('1').take(64).collect();
    bencher.bench(|| {
        assert_ne!(black_box(&heap1), black_box(&heap2));
    })
}

#[divan::bench]
fn inline_eq_static(bencher: Bencher) {
    let inline = Str24::new_inline("some text");
    let static_str = Str24::new_static("some text");
    bencher.bench(|| {
        assert_eq!(black_box(&inline), black_box(&static_str));
    })
}

#[divan::bench]
fn inline_ne_static(bencher: Bencher) {
    let inline = Str24::new_inline("some text");
    let static_str = Str24::new_static(
        "a very long and longer and longer and longer and even longer static text",
    );
    bencher.bench(|| {
        assert_ne!(black_box(&inline), black_box(&static_str));
    })
}

#[divan::bench]
fn inline_ne_heap(bencher: Bencher) {
    let inline = Str24::new_inline("some text");
    let heap: Str24 = std::iter::repeat('0').take(64).collect();
    bencher.bench(|| {
        assert_ne!(black_box(&inline), black_box(&heap));
    })
}
*/
