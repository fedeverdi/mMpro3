use crate::processing::Router;

/// Write processed audio frames to output buffer
/// Real-time safe - no allocations, just array indexing
#[inline]
pub fn write_output_frame(
    data: &mut [f32],
    frame_idx: usize,
    output_channels: usize,
    master_left: f32,
    master_right: f32,
    router: &Router,
) {
    let out_frame_start = frame_idx * output_channels;

    // Write master output to its channels (SWAPPED: left/right inverted for fix)
    let master_left_ch = router.master.output_channel_selection.left as usize;
    let master_right_ch = router.master.output_channel_selection.right as usize;
    if master_left_ch < output_channels {
        data[out_frame_start + master_left_ch] += master_right.clamp(-1.0, 1.0);
    }
    if master_right_ch < output_channels {
        data[out_frame_start + master_right_ch] += master_left.clamp(-1.0, 1.0);
    }
}

/// Write all subgroup outputs to their channels
#[inline]
pub fn write_subgroup_outputs(
    data: &mut [f32],
    frame_idx: usize,
    output_channels: usize,
    router: &Router,
) {
    let out_frame_start = frame_idx * output_channels;
    
    for (i, subgroup) in router.subgroups.iter().enumerate() {
        if subgroup.output_enabled && i < router.last_subgroup_outputs.len() {
            let (sg_l, sg_r) = router.last_subgroup_outputs[i];
            let sg_left_ch = subgroup.output_channel_selection.left as usize;
            let sg_right_ch = subgroup.output_channel_selection.right as usize;
            
            if sg_left_ch < output_channels {
                data[out_frame_start + sg_left_ch] += sg_l.clamp(-1.0, 1.0);
            }
            if sg_right_ch < output_channels {
                data[out_frame_start + sg_right_ch] += sg_r.clamp(-1.0, 1.0);
            }
        }
    }
}

/// Write all aux bus outputs to their channels (handles mono/stereo routing)
#[inline]
pub fn write_aux_outputs(
    data: &mut [f32],
    frame_idx: usize,
    output_channels: usize,
    router: &Router,
) {
    let out_frame_start = frame_idx * output_channels;
    
    for (i, aux_bus) in router.aux_buses.iter().enumerate() {
        if aux_bus.output_enabled && i < router.last_aux_outputs.len() {
            let (aux_l, aux_r) = router.last_aux_outputs[i];
            let aux_left_ch = aux_bus.output_channel_selection.left as usize;
            let aux_right_ch = aux_bus.output_channel_selection.right as usize;
            
            // If both channels are the same, send mono mix to single channel
            if aux_left_ch == aux_right_ch {
                let mono = (aux_l + aux_r) * 0.5;
                if aux_left_ch < output_channels {
                    data[out_frame_start + aux_left_ch] += mono.clamp(-1.0, 1.0);
                }
            } else {
                // Send stereo to different channels
                if aux_left_ch < output_channels {
                    data[out_frame_start + aux_left_ch] += aux_l.clamp(-1.0, 1.0);
                }
                if aux_right_ch < output_channels {
                    data[out_frame_start + aux_right_ch] += aux_r.clamp(-1.0, 1.0);
                }
            }
        }
    }
}
