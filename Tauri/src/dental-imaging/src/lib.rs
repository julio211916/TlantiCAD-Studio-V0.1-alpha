//! TlantiStudio Dental Imaging
//!
//! Medical imaging crate supporting:
//! - DICOM file parsing and pixel-data extraction (X-Ray, CT, MRI)
//! - DICOM networking (SCP/SCU) via dicom-ul for C-ECHO, C-FIND, C-STORE, C-MOVE
//! - DICOMweb REST client (WADO-RS, STOW-RS, QIDO-RS)
//! - Orthanc PACS server client
//! - STL 3D mesh parsing for dental models
//! - Image gallery management per patient

pub mod dicom_viewer;
pub mod dicom_network;
pub mod dicom_web_client;
pub mod orthanc_client;
pub mod stl_viewer;
pub mod gallery;
pub mod error;

pub use error::ImagingError;
