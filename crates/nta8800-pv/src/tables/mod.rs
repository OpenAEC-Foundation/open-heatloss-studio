//! Norm-tabellen voor de PV-keten.
//!
//! Bevat de transcriptie van NTA 8800 Tabel 17.2 (opvallende zonnestraling per
//! hellingshoek + oriëntatie) die de tilt/azimut-afhankelijkheid van de
//! PV-opbrengst bepaalt — zie [`irradiation`].

pub mod irradiation;

pub use irradiation::{isol_w_per_m2, tilt_azimuth_factor};
