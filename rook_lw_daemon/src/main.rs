use std::fs;
use std::path;
use std::thread::sleep;

use rook_lw_daemon::error::RookLWResult;
use rook_lw_daemon::implementation::factory::FrameSourceFactory;
use rook_lw_daemon::image::fourcc::fourcc_to_string;
use rook_lw_daemon::image::yplane;

fn main() -> RookLWResult<()> {

    let image_dump_dir = "var/images";

    fs::create_dir_all(image_dump_dir)?;

    // Print available frame sources at compile time
    println!("Available frame sources: {:?}", FrameSourceFactory::available_sources());

    let mut frame_source = FrameSourceFactory::create()?;

    let sources = frame_source.list_sources()?;

    for (i, source) in sources.iter().enumerate() {
        println!("Source {}: {}", i, source);
    }

    if sources.is_empty() {
        return Err(rook_lw_daemon::error::RookLWError::Camera("No available frame sources found".to_owned()));
    }
    else {
        println!("Using source: {}", sources[0]);
        frame_source.set_source(&sources[0])?;
    }

    let pixel_format = fourcc_to_string(frame_source.get_pixel_format()?);
    println!("Pixel format: {}", pixel_format);

    if pixel_format != "MJPG" {
        return Err(rook_lw_daemon::error::RookLWError::Camera(format!("Expected MJPG pixel format, got {}", pixel_format)));
    }

    println!("Frame dimensions: {}x{}", frame_source.get_width()?, frame_source.get_height()?);

    println!("Starting frame source...");

    frame_source.start()?;

    sleep(std::time::Duration::from_secs(1));

    for run_id in 0..100 {
        let frame1 = frame_source.next_frame()?;
        let yplane1 = yplane::YPlane::from_frame(&*frame1)?;

        let plane_count1 = frame1.get_plane_count()?;
        let plane_data1 = frame1.get_plane_data(0)?;
        //println!("Acquired first frame: plane_count={}, plane_data_len={}", plane_count1, plane_data1.len());
        
        sleep(std::time::Duration::from_millis(50));

        let _frame2 = frame_source.next_frame()?;
        let yplane2 = yplane::YPlane::from_frame(&*_frame2)?;

        let plane_count2 = _frame2.get_plane_count()?;
        let plane_data2 = _frame2.get_plane_data(0)?;
        //println!("Acquired second frame: plane_count={}, plane_data_len={}", plane_count2, plane_data2.len());
        
        let score = yplane::normalized_mean_abs_diff(&yplane1, &yplane2, 1)?;
        println!("Luma difference score between frames: {}", score);

        // Write when difference is significant
        if score >= 0.04 {
            println!("Writing images for run_id {}", run_id);
            let image_path1 = path::Path::new(image_dump_dir).join(format!("frame-{}-{}.jpg", run_id, 1)); 
            fs::write(&image_path1, plane_data1)?;

            let image_path2 = path::Path::new(image_dump_dir).join(format!("frame-{}-{}.jpg", run_id, 2)); 
            fs::write(&image_path2, plane_data2)?;
        }

        sleep(std::time::Duration::from_millis(500));
    }

    println!("Complete...");
    Ok(())
}
