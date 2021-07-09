use std::{path::PathBuf, thread};

use crossbeam_channel::{Receiver, Sender};
use image::DynamicImage;
use img_hash::{HasherConfig, ImageHash};

pub fn calc_hashes(
    imgs_recv: Receiver<(PathBuf, DynamicImage)>,
    hashes_send: Sender<(PathBuf, ImageHash)>,
    thread_count: usize,
) {
    let join_handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let imgs_recv_local = imgs_recv.clone();
            let hashes_send_local = hashes_send.clone();
            thread::spawn(move || {
                let hasher = HasherConfig::new().to_hasher();
                // compute hash and send until empty and disconnected
                imgs_recv_local.iter().for_each(|(path, img)| {
                    let path_hash_pair = (path, hasher.hash_image(&img));
                    hashes_send_local
                        .send(path_hash_pair)
                        .expect("Hash receiver hung up unexpectedly");
                });
            })
        })
        .collect();

    // manually drop the implicitly held sender and receiver as per best practice
    drop(imgs_recv);
    drop(hashes_send);

    // wait for all workers to finish
    join_handles.into_iter().for_each(|h| {
        h.join().expect("A hash worker thread panicked unexpectedly");
    });
}
