use std::{
    path::{Path, PathBuf},
    thread,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use image::DynamicImage;
use img_hash::{HashAlg, HasherConfig, ImageHash};
use itertools::Itertools;

pub fn calc_hashes(
    imgs_rx: Receiver<(PathBuf, DynamicImage)>,
    hashes_tx: Sender<(PathBuf, ImageHash)>,
    thread_count: usize,
    algorithm: HashAlg,
    hash_size: (u32, u32),
) {
    let join_handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let imgs_rx_local = imgs_rx.clone();
            let hashes_tx_local = hashes_tx.clone();
            thread::spawn(move || {
                let hasher = HasherConfig::new()
                    .hash_alg(algorithm)
                    .hash_size(hash_size.0, hash_size.1)
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

pub fn calc_pair_dist(img_hashes: &[(PathBuf, ImageHash)], thread_count: usize) -> Vec<(&Path, &Path, u32)> {
    use crossbeam::thread;

    // create pairs with Itertools
    let pairs: Vec<_> = img_hashes.iter().tuple_combinations::<(_, _)>().collect();

    // create channels
    let (pairs_tx, pairs_rx): (Sender<(&(PathBuf, ImageHash), &(PathBuf, ImageHash))>, _) = unbounded();
    let (dists_tx, dists_rx) = unbounded();

    // using scoped thread guarantees workers terminate before caller thread,
    // ... thereby satisfying lifetime constraints
    thread::scope(move |s| {
        let join_handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let pairs_rx_local = pairs_rx.clone();
                let dists_tx_local = dists_tx.clone();
                s.spawn(move |_| {
                    // compute distance and send until empty and disconnected
                    pairs_rx_local.iter().for_each(|((p0, h0), (p1, h1))| {
                        let dist = h0.dist(h1);
                        dists_tx_local
                            .send((p0, p1, dist))
                            .expect("Hash receiver hung up unexpectedly");
                    });
                })
            })
            .collect();

        // manually drop the implicitly held sender and receiver as per best practice
        drop(pairs_rx);
        drop(dists_tx);

        // send hash-pairs to workers
        pairs.into_iter().for_each(|pair| {
            pairs_tx
                .send(pair)
                .expect("All hash-pair receivers hung up unexpectedly");
        });
        // close pairs producer
        drop(pairs_tx);

        // wait for all workers to finish
        join_handles.into_iter().for_each(|h| {
            h.join().expect("A distance worker thread panicked unexpectedly");
        });
    })
    .unwrap(); // cannot be Err; panicked worker threads already caught by manual join

    dists_rx
        .into_iter()
        .map(|(p0, p1, d)| (p0.as_path(), p1.as_path(), d))
        .collect()
}
