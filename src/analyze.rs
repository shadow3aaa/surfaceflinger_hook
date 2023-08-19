use std::{
    collections::VecDeque,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use log::info;

const OTHERS_BUFFER_SIZE: usize = 128;
const TARGET_FPS_BUFFER_SIZE: usize = 20;

pub enum Message {
    Vsync,
    Soft,
}

pub fn jank(rx: &Receiver<Message>) {
    let mut vsync_buffer = VecDeque::with_capacity(OTHERS_BUFFER_SIZE);
    let mut composition_buffer = VecDeque::with_capacity(OTHERS_BUFFER_SIZE);
    let mut target_fps_buffer = Vec::with_capacity(TARGET_FPS_BUFFER_SIZE);

    let mut composition_fps: f64 = 0.0;
    let mut display_fps: f64 = 0.0;
    let mut target_fps: f64 = 0.0;
    let mut data_take: usize = 1;

    loop {
        match rx.recv().unwrap() {
            Message::Vsync => {
                vsync_buffer.push_front(Instant::now());
                if let Some(v) = analyze_screen(&vsync_buffer, data_take) {
                    display_fps = v;
                }

                let new_data_take = (display_fps / target_fps).round().max(1.0);

                if new_data_take.is_finite() {
                    data_take = new_data_take as usize;
                }
            }
            Message::Soft => {
                composition_buffer.push_front(Instant::now());
                if let Some(v) = analyze_soft(&composition_buffer, data_take) {
                    composition_fps = v;
                }

                target_fps_buffer.push(composition_fps);
            }
        }

        // keep buffer size
        if vsync_buffer.len() > OTHERS_BUFFER_SIZE {
            vsync_buffer.pop_back();
        }

        if composition_buffer.len() > OTHERS_BUFFER_SIZE {
            composition_buffer.pop_back();
        }

        if target_fps_buffer.len() > TARGET_FPS_BUFFER_SIZE {
            info!("debug: {target_fps_buffer:#?}");
            target_fps = target_fps_buffer.iter().fold(0.0, |max: f64, cur| {
                if *cur <= display_fps {
                    max.max(*cur)
                } else {
                    max
                }
            });
            target_fps_buffer.clear();
            info!("TARGET FPS: {target_fps:.2}");
        }

        // debug
        info!("SCREEN REFRESH RATE: {display_fps:.2}");
        info!("SOFTWARE FPS: {composition_fps:.2}");
    }
}

fn analyze_screen(b: &VecDeque<Instant>, t: usize) -> Option<f64> {
    // analyze buffer
    let (_, five_vsync_dur_max) = b
        .iter()
        .take(t + 1) // 5个间隔需要6个数据
        .fold(
            (None::<&Instant>, Duration::new(0, 0)),
            |(last, mut result), x| {
                if let Some(stamp) = last {
                    let dur = *stamp - *x;
                    result = result.max(dur);
                }
                (Some(x), result)
            },
        );
    let updated_display_fps =
        Duration::from_secs(1).as_secs_f64() / five_vsync_dur_max.as_secs_f64();

    if !updated_display_fps.is_finite() {
        return None;
    }

    Some(updated_display_fps)
}

fn analyze_soft(b: &VecDeque<Instant>, t: usize) -> Option<f64> {
    let (_, total_dur, count) = b.iter().take(t + 1).fold(
        (None::<&Instant>, Duration::new(0, 0), 0),
        |(last, mut total, mut count), cur| {
            if let Some(last) = last {
                let dur = *last - *cur;
                if dur > Duration::from_millis(1) {
                    total += dur;
                    count += 1;
                }
            }
            (Some(cur), total, count)
        },
    );

    let avg_dur = total_dur.checked_div(count).unwrap_or_default();
    let update_composition_fps = Duration::from_secs(1).as_secs_f64() / avg_dur.as_secs_f64();

    if update_composition_fps.is_finite() {
        Some(update_composition_fps)
    } else {
        None
    }
}
