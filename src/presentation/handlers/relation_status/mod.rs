//! Módulo de handlers para actualización de status de file_relations
//! Estados válidos: pendiente, reservado, asignado, confirmado, en_curso, completado, cancelado, anulado

pub mod entrada_status;
pub mod guia_status;
pub mod pasajero_status;
pub mod restaurante_status;
pub mod vehiculo_status;
pub mod tour_status;

pub use entrada_status::*;
pub use guia_status::*;
pub use pasajero_status::*;
pub use restaurante_status::*;
pub use vehiculo_status::*;
pub use tour_status::*;
