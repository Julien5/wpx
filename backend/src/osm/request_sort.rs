use crate::bbox::BoundingBox;

pub type PartitionKey = Vec<(u64, u64, u64, u64)>;

pub fn key(v: &Vec<BoundingBox>) -> PartitionKey {
    let mut bits_vector: PartitionKey = v
        .iter()
        .map(|bbox| {
            // Normalize -0.0 to 0.0 on the fly to prevent silent negative-zero sorting mismatches
            let min_x = if bbox.get_min().x == 0.0 {
                0.0
            } else {
                bbox.get_min().x
            };
            let min_y = if bbox.get_min().y == 0.0 {
                0.0
            } else {
                bbox.get_min().y
            };
            let max_x = if bbox.get_max().x == 0.0 {
                0.0
            } else {
                bbox.get_max().x
            };
            let max_y = if bbox.get_max().y == 0.0 {
                0.0
            } else {
                bbox.get_max().y
            };

            (
                min_x.to_bits(),
                min_y.to_bits(),
                max_x.to_bits(),
                max_y.to_bits(),
            )
        })
        .collect();

    // 2. CRITICAL STEP: Sort the vector of bit-tuples.
    // This ensures that if your partitioning logic outputs the exact same boxes
    // in a different sequence order, they still resolve to the exact same map key.
    bits_vector.sort_unstable();

    bits_vector
}
