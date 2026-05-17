//! K-D Tree for nearest-point queries in O(log n)

use nalgebra::Point3;

/// Entry in the K-D tree
#[derive(Debug, Clone)]
struct KdNode {
    point: Point3<f64>,
    index: usize,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
    split_axis: usize,
}

/// 3D K-D Tree for fast nearest-neighbor and radius queries
#[derive(Debug)]
pub struct KdTree {
    root: Option<Box<KdNode>>,
    size: usize,
}

impl KdTree {
    /// Build from a set of points
    pub fn build(points: &[Point3<f64>]) -> Self {
        if points.is_empty() {
            return Self { root: None, size: 0 };
        }
        let mut entries: Vec<(Point3<f64>, usize)> = points.iter().cloned().zip(0..).collect();
        let root = Self::build_recursive(&mut entries, 0);
        Self { root: Some(root), size: points.len() }
    }

    fn build_recursive(entries: &mut [(Point3<f64>, usize)], depth: usize) -> Box<KdNode> {
        let axis = depth % 3;
        entries.sort_by(|a, b| a.0[axis].partial_cmp(&b.0[axis]).unwrap_or(std::cmp::Ordering::Equal));
        let mid = entries.len() / 2;

        let left = if mid > 0 {
            Some(Self::build_recursive(&mut entries[..mid], depth + 1))
        } else {
            None
        };
        let right = if mid + 1 < entries.len() {
            Some(Self::build_recursive(&mut entries[mid + 1..], depth + 1))
        } else {
            None
        };

        Box::new(KdNode {
            point: entries[mid].0,
            index: entries[mid].1,
            left,
            right,
            split_axis: axis,
        })
    }

    pub fn len(&self) -> usize { self.size }
    pub fn is_empty(&self) -> bool { self.size == 0 }

    /// Find the nearest point. Returns (index, distance²)
    pub fn nearest(&self, query: &Point3<f64>) -> Option<(usize, f64)> {
        let Some(ref root) = self.root else { return None };
        let mut best_idx = root.index;
        let mut best_dist2 = (root.point - query).norm_squared();
        Self::nearest_recursive(root, query, &mut best_idx, &mut best_dist2);
        Some((best_idx, best_dist2))
    }

    fn nearest_recursive(
        node: &KdNode,
        query: &Point3<f64>,
        best_idx: &mut usize,
        best_dist2: &mut f64,
    ) {
        let dist2 = (node.point - query).norm_squared();
        if dist2 < *best_dist2 {
            *best_dist2 = dist2;
            *best_idx = node.index;
        }

        let axis = node.split_axis;
        let diff = query[axis] - node.point[axis];

        let (first, second) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(ref child) = first {
            Self::nearest_recursive(child, query, best_idx, best_dist2);
        }
        // Check if the splitting plane could have closer points
        if diff * diff < *best_dist2 {
            if let Some(ref child) = second {
                Self::nearest_recursive(child, query, best_idx, best_dist2);
            }
        }
    }

    /// Find K nearest neighbors. Returns Vec<(index, distance²)> sorted by distance
    pub fn k_nearest(&self, query: &Point3<f64>, k: usize) -> Vec<(usize, f64)> {
        let mut heap = Vec::with_capacity(k + 1);
        if let Some(ref root) = self.root {
            Self::knn_recursive(root, query, k, &mut heap);
        }
        heap.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        heap
    }

    fn knn_recursive(
        node: &KdNode,
        query: &Point3<f64>,
        k: usize,
        heap: &mut Vec<(usize, f64)>,
    ) {
        let dist2 = (node.point - query).norm_squared();

        if heap.len() < k {
            heap.push((node.index, dist2));
            heap.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        } else if dist2 < heap[0].1 {
            heap[0] = (node.index, dist2);
            heap.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        let axis = node.split_axis;
        let diff = query[axis] - node.point[axis];
        let worst_dist2 = if heap.len() < k { f64::MAX } else { heap[0].1 };

        let (first, second) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(ref child) = first {
            Self::knn_recursive(child, query, k, heap);
        }
        if diff * diff < worst_dist2 {
            if let Some(ref child) = second {
                Self::knn_recursive(child, query, k, heap);
            }
        }
    }

    /// Find all points within radius. Returns Vec<(index, distance²)>
    pub fn radius_search(&self, query: &Point3<f64>, radius: f64) -> Vec<(usize, f64)> {
        let mut result = Vec::new();
        let r2 = radius * radius;
        if let Some(ref root) = self.root {
            Self::radius_recursive(root, query, r2, &mut result);
        }
        result
    }

    fn radius_recursive(
        node: &KdNode,
        query: &Point3<f64>,
        r2: f64,
        result: &mut Vec<(usize, f64)>,
    ) {
        let dist2 = (node.point - query).norm_squared();
        if dist2 <= r2 {
            result.push((node.index, dist2));
        }

        let axis = node.split_axis;
        let diff = query[axis] - node.point[axis];

        let (first, second) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(ref child) = first {
            Self::radius_recursive(child, query, r2, result);
        }
        if diff * diff <= r2 {
            if let Some(ref child) = second {
                Self::radius_recursive(child, query, r2, result);
            }
        }
    }
}
