use crate::processing::{Router, routing};

/// Collect FFT data from master analyzer
/// Returns (bins_left, bins_right) if available
#[inline]
pub fn collect_master_fft(router: &mut Router) -> Option<(Vec<f32>, Vec<f32>)> {
    router.fft_analyzer.analyze()
}

/// Collect FFT data from all active tracks
/// Returns vector of (track_id, bins_left, bins_right)
pub fn collect_track_fft(router: &mut Router) -> Vec<(usize, Vec<f32>, Vec<f32>)> {
    router.tracks.iter_mut()
        .filter(|t| t.source != routing::TrackSource::None)
        .filter_map(|t| {
            t.fft_analyzer.analyze().map(|(left, right)| (t.id, left, right))
        })
        .collect()
}
