//! Módulo de handlers para File Relations (relaciones entre files y recursos)
//! Dividido por tipo de relación y operación

pub mod entradas;
pub mod guias;
pub mod tours;
pub mod pasajeros;
pub mod restaurantes;
pub mod vehiculos;

pub use entradas::*;
pub use guias::*;
pub use tours::*;
pub use pasajeros::*;
pub use restaurantes::*;
pub use vehiculos::*;
