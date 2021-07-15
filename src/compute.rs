use std::{path::PathBuf, thread};

use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use img_hash::{HasherConfig, ImageHash};

pub fn calc_hashes(
    imgs_rx: Receiver<(PathBuf, DynamicImage)>,
    hashes_tx: Sender<(PathBuf, ImageHash)>,
    thread_count: usize,
) {
    let join_handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let imgs_rx_local = imgs_rx.clone();
            let hashes_tx_local = hashes_tx.clone();
            thread::spawn(move || {
                let hasher = HasherConfig::new()
                    .hash_alg(img_hash::HashAlg::DoubleGradient)
                    .hash_size(32, 32)
                    .to_hasher();
                // compute hash and send until empty and disconnected
                imgs_rx_local.iter().for_each(|(path, img)| {
                    let path_hash_pair = (path, hasher.hash_image(&img));
                    hashes_tx_local
                        .send(path_hash_pair)
                        .expect("Hash receiver hung up unexpectedly");
                });
            })
        })
        .collect();

    // manually drop the implicitly held sender and receiver as per best practice
    drop(imgs_rx);
    drop(hashes_tx);

    // wait for all workers to finish
    join_handles.into_iter().for_each(|h| {
        h.join().expect("A hash worker thread panicked unexpectedly");
    });
}
