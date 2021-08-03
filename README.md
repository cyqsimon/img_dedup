# img_dedup
Image deduplicator written in Rust

## WORK IN PROGRESS
This is a work in progress. The program has **no testing whatsoever**, and there is **no guarantee whatsoever** on forward-compatibility.

If it's not obvious enough, that means **DO NOT** use in production.

## Current status
- Specify an input directory and select specific files (via `regex`) on CLI
- Compute the perceptual hash of the selected image files
- Compute the pairwise hamming distance of images, thereby finding similar looking ones
- Move similar looking images into a user-specified directory
- All operations efficiently multithreaded using channels

## Planned objectives
- Further optimise "similarity threshold"
- Migrate to `log` and `simple_logger`; add tiered logging
- Some (more) convenient mechanism for the user to review and determine whether and which image(s) to keep
- Automatically remove duplicate images, leaving just one (probably highest quality)
