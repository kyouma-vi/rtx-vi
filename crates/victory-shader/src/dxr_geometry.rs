//! Offline, bounded extraction of reachable triangles from a rehydrated
//! console BLAS. Foundation for future DXR geometry upload, NOT a renderer.
//! Do not use on packed data, TLAS, or memory being modified by the builder.
//! No guess about unused triangles: only pointers reached from the root count.
use crate::bvh;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    /// Original leaf pointer, including the triangle index (low three bits).
    pub pointer: u32,
    /// Position order matches the title's barycentric I/J order.
    pub vertices: [[f32; 3]; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractError {
    MissingNode(u32),
    UnsupportedKind(u32),
    InvalidTriangle(u32),
    RepeatedPointer(u32),
    BudgetExceeded,
}

/// `read` receives a 64-byte node's byte offset relative to the BLAS base.
/// The caller must enforce allocation boundaries and a stable snapshot.
/// `budget` bounds the total number of visited pointers (including boxes).
/// No partial geometry is returned after an error. A NONE root is empty.
pub fn extract_blas(
    root: u32,
    budget: usize,
    mut read: impl FnMut(u64) -> Option<[u32; 16]>,
) -> Result<Vec<Triangle>, ExtractError> {
    let mut seen = HashSet::new();
    let mut stack = vec![root];
    let mut out = Vec::new();
    while let Some(pointer) = stack.pop() {
        if pointer == bvh::NONE {
            continue;
        }
        if seen.len() >= budget {
            return Err(ExtractError::BudgetExceeded);
        }
        if !seen.insert(pointer) {
            return Err(ExtractError::RepeatedPointer(pointer));
        }
        let kind = pointer & 7;
        if kind >= 6 {
            return Err(ExtractError::UnsupportedKind(pointer));
        }
        let offset = (u64::from(pointer) >> 3) << 6;
        let node = read(offset).ok_or(ExtractError::MissingNode(pointer))?;
        if kind >= 4 {
            // Child pointers live in the first 16 bytes of both box formats.
            stack.extend(node[..4].iter().rev().copied());
            continue;
        }
        let id = node[15] >> (8 * kind);
        let i = id & 3;
        let j = (id >> 2) & 3;
        if i == j || i == 3 || j == 3 {
            return Err(ExtractError::InvalidTriangle(pointer));
        }
        let mut words = [0u32; 28];
        words[..16].copy_from_slice(&node);
        let vertices = bvh::triangle_vertices(&words, kind);
        if !vertices.iter().flatten().all(|v| v.is_finite()) {
            return Err(ExtractError::InvalidTriangle(pointer));
        }
        out.push(Triangle { pointer, vertices });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf() -> [u32; 16] {
        let mut node = [0; 16];
        for n in 0..15 { node[n] = (n as f32).to_bits(); }
        node[15] = 0x09090909; // I=1, J=2 for all four triangle slots.
        node
    }
    #[test]
    fn only_reachable_triangles_and_original_ids() {
        let mut box_node = [0; 16];
        box_node[..4].copy_from_slice(&[8, 10, bvh::NONE, bvh::NONE]);
        let result = extract_blas(4, 3, |offset| match offset {
            0 => Some(box_node), 64 => Some(leaf()), _ => None,
        }).unwrap();
        assert_eq!(result.iter().map(|t| t.pointer).collect::<Vec<_>>(), [8, 10]);
        assert_eq!(result[0].vertices, [[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]]);
        assert_eq!(result[1].vertices, [[6., 7., 8.], [9., 10., 11.], [12., 13., 14.]]);
    }
    #[test]
    fn refuses_cycles_bad_data_and_unsupported_nodes() {
        let mut node = [bvh::NONE; 16];
        node[0] = 4;
        assert_eq!(extract_blas(4, 8, |_| Some(node)), Err(ExtractError::RepeatedPointer(4)));
        assert_eq!(extract_blas(4, 0, |_| Some(node)), Err(ExtractError::BudgetExceeded));
        assert_eq!(extract_blas(0, 1, |_| None), Err(ExtractError::MissingNode(0)));
        assert_eq!(extract_blas(6, 1, |_| Some(node)), Err(ExtractError::UnsupportedKind(6)));
        assert_eq!(extract_blas(0, 1, |_| Some(node)), Err(ExtractError::InvalidTriangle(0)));
        let mut nonfinite = leaf();
        nonfinite[0] = f32::NAN.to_bits();
        assert_eq!(extract_blas(0, 1, |_| Some(nonfinite)), Err(ExtractError::InvalidTriangle(0)));
    }
}
