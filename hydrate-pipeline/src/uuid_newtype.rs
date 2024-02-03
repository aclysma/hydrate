#[macro_export]
macro_rules! create_uuid_newtype {
    ($data_type:ident, $name:literal) => {
        #[derive(
            Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
        )]
        pub struct $data_type(u128);
        impl $data_type {
            pub const fn null() -> Self {
                Self(0)
            }

            pub fn from_bytes(bytes: [u8; 16]) -> Self {
                Self(uuid::Uuid::from_bytes(bytes).as_u128())
            }

            pub fn as_bytes(self) -> [u8; 16] {
                *uuid::Uuid::from_u128(self.0).as_bytes()
            }

            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid.as_u128())
            }

            pub fn as_uuid(self) -> uuid::Uuid {
                uuid::Uuid::from_u128(self.0)
            }

            pub fn from_u128(u: u128) -> Self {
                Self(u)
            }

            pub fn as_u128(self) -> u128 {
                self.0
            }

            pub fn is_null(&self) -> bool {
                return self.0 == 0;
            }
        }

        impl std::fmt::Debug for $data_type {
            fn fmt(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                f.debug_tuple($name)
                    .field(&uuid::Uuid::from_u128(self.0))
                    .finish()
            }
        }
    };
}
