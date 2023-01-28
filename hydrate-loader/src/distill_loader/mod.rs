
/// Handles provide automatic reference counting of assets, similar to [Rc](`std::rc::Rc`).
pub mod handle;

/// [`AssetStorage`](crate::storage::AssetStorage) is implemented by engines to store loaded asset data.
pub mod storage;

mod task_local;

pub use crossbeam_channel;
//pub use loader::Loader;
#[cfg(feature = "packfile_io")]
pub use packfile_io::PackfileReader;
#[cfg(feature = "rpc_io")]
pub use rpc_io::RpcIO;
pub use storage::LoadHandle;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + 'static>>;

#[cfg(feature = "handle")]
#[macro_export]
macro_rules! if_handle_enabled {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[cfg(not(feature = "handle"))]
#[macro_export]
#[doc(hidden)]
macro_rules! if_handle_enabled {
    ($($tt:tt)*) => {};
}
