# reference

What phient reads and is measured against. None is built.

Apple's interfaces are SDK files and stay where they are on disk; they are
listed here by path (from `gemphi/`) so there is one place to look.

- **Core ML** (model, configuration, compute plan and cost, state,
  update task, parameter keys, multi-array, batch and feature providers):
  `phi/crates/macos/src/core_ml/CoreML.framework/Versions/A/Headers/`
- **ML Compute** (graph, training graph, inference graph, device,
  optimizer descriptors): `phi/crates/macos/src/core_ml/MLCompute.framework/Versions/A/Headers/`
- **Create ML** (training sessions, checkpoints, resume):
  `phi/crates/macos/src/core_ml/CreateML.framework/`
- **Create ML Components** (estimator and transformer contracts, encode
  and decode with the optimizer): `phi/crates/macos/src/core_ml/CreateMLComponents.framework/`
- **Vision, Accelerate, Metal Performance Shaders:**
  `phi/apple-source/ruskit/{Vision,Accelerate,MetalPerformanceShaders}.framework`
- **The Rust transcriptions of those** (shapes only, nothing runs):
  `phikit/crates/apple/src/coreml`, `phi/crates/coreml`, `phi/crates/phi-oml`

The row-by-row mapping and the order of work are in
`phinet/docs/phient/apple.md`; the training session they lead to is drawn
in `phinet/docs/phient/session.md`, with its folders in `plan.md` and its
requirements in `specs.md` beside it; the view from the top of Apple's
stack is `phinet/docs/apple_vs_phient.md`.
