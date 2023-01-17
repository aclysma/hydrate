use std::cmp::max;
use std::path::PathBuf;
use hydrate_model::ObjectId;
use crossbeam_channel::{Sender, Receiver};

enum AssetIOThreadPoolCommand {
    Load(AssetRequest),
    Finish
}

struct AssetIOThreadPool {

    request_tx: Sender<AssetIOThreadPoolCommand>,
    //request_rx: Receiver<AssetRequest>,

    result_tx: Sender<AssetRequestResult>,
    result_rx: Receiver<AssetRequestResult>,
}

fn asset_io_load_job() {

}

fn asset_io_thread_pool_main_loop(max_requests_in_flight: usize, request_rx: Receiver<AssetIOThreadPoolCommand>) {
    loop {
        let mut worker_thread_count = 0;
        let mut worker_threads = Vec::default();
        worker_threads.resize(max_requests_in_flight, None);
        let mut available_worker_threads = Vec::default();
        for i in 0..64 {
            available_worker_threads.push(i);
        }

        while let Ok(command) = request_rx.recv() {
            match command {
                AssetIOThreadPoolCommand::Finish => {
                    for worker_thread in &worker_threads {
                        if let Some(worker_thread) = worker_thread {

                        }
                    }
                },
                AssetIOThreadPoolCommand::Load(request) => {
                    available_worker_threads.po

                    let new_thread = std::thread::spawn(move || asset_io_load_job());
                    worker_threads
                }
            }
        }
    }
}

impl AssetIOThreadPool {
    fn new(max_requests_in_flight: usize) {
        let (request_tx, request_rx) = crossbeam_channel::unbounded::<AssetIOThreadPoolCommand>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();

        std::thread::spawn(move || asset_io_thread_pool_main_loop(max_requests_in_flight, request_rx));

        AssetIOThreadPool {
            request_tx,
            result_tx,
            result_rx
        }
    }
}



struct AssetRequest {
    object_id: ObjectId,
    subresource: Option<u32>,
}

struct AssetRequestResult {
    result: std::io::Result<Vec<u8>>
}

struct AssetIO {
    build_data_root_path: PathBuf,
    result_tx: Sender<AssetRequestResult>,
    result_rx: Receiver<AssetRequestResult>,
}

impl AssetIO {
    pub fn new(build_data_root_path: PathBuf) -> Self {
        AssetIO {
            build_data_root_path
        }
    }

    pub fn request_data(&self, object_id: ObjectId, subresource: Option<u32>) {
        let path = hydrate_model::uuid_path::uuid_to_path(&self.build_data_root_path, object_id.as_uuid());
        std::thread::spawn(|| {
            let data = std::fs::read(&path);

        })
    }
}
