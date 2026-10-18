use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use panta_core::project::DocumentWorkspace;
use std::hint::black_box;

fn benchmark_workspace(criterion: &mut Criterion) {
    criterion.bench_function("workspace/open_ready_import", |bencher| {
        bencher.iter_batched_ref(
            || DocumentWorkspace::new("Welcome"),
            |workspace| black_box(workspace.open_ready_import("asset", "model")),
            BatchSize::SmallInput,
        )
    });

    criterion.bench_function("workspace/activate_unloaded_import", |bencher| {
        bencher.iter_batched_ref(
            || {
                let mut workspace = DocumentWorkspace::new("Welcome");
                let _ = workspace.open_import("asset", "model");
                workspace
            },
            |workspace| black_box(workspace.activation_request("asset")),
            BatchSize::SmallInput,
        )
    });

    criterion.bench_function("workspace/snapshot_with_import", |bencher| {
        bencher.iter_batched_ref(
            || {
                let mut workspace = DocumentWorkspace::new("Welcome");
                let _ = workspace.open_ready_import("asset", "model");
                workspace
            },
            |workspace| black_box(workspace.snapshot()),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, benchmark_workspace);
criterion_main!(benches);
