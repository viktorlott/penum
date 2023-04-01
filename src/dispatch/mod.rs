pub use self::blueprint::Blueprint;
pub use self::blueprint::Blueprints;
pub use self::sig::VariantSig;

mod blueprint;
mod ret;
mod shm;
mod sig;
mod standard;

/// Storing token streams will cause "use after free" error, so we store them as Strings instead.
pub static T_SHM: self::shm::SharedMemory<String, String> = self::shm::SharedMemory::new();
