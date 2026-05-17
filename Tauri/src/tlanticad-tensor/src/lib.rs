//! TlantiCAD Tensor Engine
//! ndarray-based tensor operations for dental ML inference

pub mod tensor;
pub mod inference;
pub mod ops;

pub use tensor::*;
pub use inference::*;
pub use ops::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_zeros_vector() {
        let t = DentalTensor::zeros(TensorShape::Vector(10));
        assert_eq!(t.numel(), 10);
    }

    #[test]
    fn test_tensor_zeros_matrix() {
        let t = DentalTensor::zeros(TensorShape::Matrix(3, 4));
        assert_eq!(t.numel(), 12);
    }

    #[test]
    fn test_tensor_zeros_volume() {
        let t = DentalTensor::zeros(TensorShape::Volume(2, 3, 4));
        assert_eq!(t.numel(), 24);
    }

    #[test]
    fn test_tensor_from_data() {
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let t = DentalTensor::from_data(TensorShape::Matrix(2, 3), data).unwrap();
        assert_eq!(t.numel(), 6);
    }

    #[test]
    fn test_tensor_normalize() {
        let data: Vec<f32> = vec![0.0, 50.0, 100.0, 200.0];
        let mut t = DentalTensor::from_data(TensorShape::Vector(4), data).unwrap();
        t.normalize();
        assert_eq!(t.numel(), 4);
    }

    #[test]
    fn test_from_cbct_slice() {
        let slice: Vec<u16> = vec![0, 100, 200, 500];
        let t = DentalTensor::from_cbct_slice(&slice, 2, 2);
        assert_eq!(t.numel(), 4);
    }

    #[test]
    fn test_stack_slices() {
        let s1 = DentalTensor::zeros(TensorShape::Matrix(2, 2));
        let s2 = DentalTensor::zeros(TensorShape::Matrix(2, 2));
        let stacked = DentalTensor::stack_slices(&[s1, s2]).unwrap();
        assert_eq!(stacked.numel(), 8);
    }

    #[test]
    fn test_threshold() {
        let data: Vec<f32> = vec![0.1, 0.5, 0.8, 0.3];
        let t = DentalTensor::from_data(TensorShape::Vector(4), data).unwrap();
        let r = threshold(&t, 0.4);
        assert_eq!(r.numel(), 4);
    }

    #[test]
    fn test_histogram() {
        let data: Vec<f32> = vec![0.0, 0.5, 1.0, 0.25, 0.75];
        let t = DentalTensor::from_data(TensorShape::Vector(5), data).unwrap();
        let h = histogram(&t, 10);
        assert_eq!(h.len(), 10);
    }

    #[test]
    fn test_tooth_feature_vector() {
        let fv = ToothFeatureVector::new_standard(11);
        assert_eq!(fv.fdi_number, 11);
    }

    #[test]
    fn test_bone_density_class() {
        let c = BoneDensityClass::from_hounsfield(500.0);
        assert!(!c.description().is_empty());
    }

    #[test]
    fn test_mean_hu() {
        let pixels: Vec<i16> = vec![100, 200, 300];
        let m = mean_hu(&pixels);
        assert!((m - 200.0).abs() < 0.01);
    }
}
