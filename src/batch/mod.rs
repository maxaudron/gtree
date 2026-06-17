use itertools::Itertools;
use std::{sync::RwLockWriteGuard, thread};
use tracing::debug;

use crate::repo::{Repo, Repos};

pub fn batch(repos: Repos, f: fn(RwLockWriteGuard<Repo>)) {
    let cpus = num_cpus::get();
    let len = repos.len();

    let jobs = crate::GTREE.get().unwrap().args.jobs;

    let batch_size = if jobs != 0 {
        if len <= jobs { 1 } else { len / jobs }
    } else {
        if len <= cpus { 1 } else { len / cpus }
    };

    debug!(
        "got {} repos and {} threads, batch size is {}",
        len, cpus, batch_size
    );

    let chunks = repos.into_iter().chunks(batch_size);
    let mut handles = Vec::with_capacity(cpus);

    // This has to be a for loop because iterators are lazy
    // and will only start processing the closure in a map when used.
    // https://stackoverflow.com/q/34765967
    for (i, chunk) in chunks.into_iter().enumerate() {
        let repos: Repos = chunk.collect();

        let span = tracing::debug_span!("spawn_batch_thread", number = i);
        let t_span = tracing::debug_span!("batch_thread", number = i);
        let _enter = span.enter();
        let handle = thread::Builder::new()
            .name(format!("batch_{}", i))
            .spawn(move || {
                let _enter = t_span.enter();
                for (_name, repo) in repos {
                    let repo = repo.write().unwrap();

                    f(repo)
                }
            })
            .unwrap();

        handles.push(handle);
    }

    handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
}
