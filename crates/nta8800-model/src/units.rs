//! Dimensionele type-aliases voor NTA 8800 rekenkundige grootheden.
//!
//! Alle aliassen zijn puur documentair (`f64`) en dragen geen run-time
//! eenheidcheck. Ze bestaan om bij het lezen van signatures direct duidelijk
//! te maken welke fysische grootheid wordt bedoeld — conform NTA 8800:2025+C1:2026 §3.
//!
//! NTA 8800 rekent intermediate in **MJ** (niet kWh of J) voor energie, zie
//! §9 Energiestromen. Temperaturen zijn in **°C**, niet in Kelvin.

/// Oppervlakte in m².
pub type Area = f64;

/// Lengte in m.
pub type Length = f64;

/// Energie in MJ.
///
/// NTA 8800 rekent intermediate in megajoules. Conversies naar kWh gebeuren
/// pas op het eindresultaat-niveau (factor 3.6).
pub type Energy = f64;

/// Vermogen in W.
pub type Power = f64;

/// Warmteweerstand R in m²·K/W.
pub type ThermalResistance = f64;

/// Warmtegeleidingscoëfficiënt (soortelijke) of warmteoverdrachtcoëfficiënt
/// H in W/K (absolute, niet per oppervlakte of lengte).
pub type ThermalConductance = f64;

/// Warmtedoorgangscoëfficiënt U in W/(m²·K).
pub type ThermalTransmittance = f64;

/// Lineaire warmtedoorgangscoëfficiënt ψ in W/(m·K).
///
/// Gebruikt voor lineaire koudebruggen (NTA 8800 §8, bijlage I).
pub type LinearThermalTransmittance = f64;

/// Temperatuur in °C.
///
/// NTA 8800 gebruikt Celsius voor alle klimaat- en binnentemperaturen.
pub type Temperature = f64;

/// Zoninstraling in MJ/m² (cumulatief per maand of per etmaal).
///
/// Container-waarden worden gevoed vanuit `nta8800-tables` (bijlage E).
pub type SolarIrradiation = f64;

/// Windsnelheid op locatie in m/s.
///
/// NTA 8800 tabel 17.1 kolom `u_site;mi`. Wijkt licht af van de rekenkundige
/// maandgemiddelde windsnelheid `u_10` volgens NEN 5060 — uitschieters buiten
/// één standaarddeviatie zijn uit het gemiddelde gefilterd (zie §17.2
/// opmerking 2). Wordt gebruikt in het luchtstroommodel (§11.2) voor
/// winddruk-afhankelijke infiltratie.
pub type WindSpeed = f64;
