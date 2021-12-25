# img_dedup
Image deduplicator written in Rust

## THIS WAS MY PERSONAL PLAYGROUND
The code is terrible and the effort was minimal.

If you are looking for an actually decent image deduplicator written in Rust, go check out [Czkawka](https://github.com/qarmin/czkawka). It has quite an unfriendly name (unless you speak Polish) IMO but its features are pretty neat.

### If you just want to play around with it
1. Clone this repository
2. Read help first: `cargo run --release -- --help`.
3. Run with correct parameters, e.g.:
  ```bash
  cargo run --release -- \
  "./test_imgs" \
  --in-filter "(\.jpe?g)|(.\png)" \
  --concurrency 8 \
  hash \
  --algorithm "blockhash" \
  --hash-size "24,24"
  ```

## Current status
- Specify an input directory and select specific files (via `regex`) on CLI
- Compute the perceptual hash of the selected image files
- Compute the pairwise hamming distance of images, thereby finding similar looking ones
- Move similar looking images into a user-specified directory for manual review
- All operations efficiently multithreaded using channels

## Planned objectives
- Nothing. This project is abandoned.
