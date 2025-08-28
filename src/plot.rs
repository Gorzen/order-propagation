use std::time::Duration;

use plotters::prelude::*;

pub fn plot_gossip_data(latencies: Vec<Duration>) -> Result<(), Box<dyn std::error::Error>> {
    const FILE_NAME: &str = "latency_histogram.png";

    let latencies_ms: Vec<u128> = latencies.iter().map(|d| d.as_millis()).collect();

    // Bin the data
    const NUM_BINS: usize = 50;
    let mut bins = [0u32; NUM_BINS];

    let max_latency = *latencies_ms.iter().max().unwrap_or(&0);

    // Ensures a bin_width of at least 1 and fixes the case where the integer division leads to the x-axis covering too little space (e.g. bin_width * NUM_BINS should be bigger than max_latency)
    let bin_width = max_latency / NUM_BINS as u128 + 1;

    for &latency in &latencies_ms {
        let bin_index = (latency / bin_width) as usize;
        bins[bin_index] += 1;
    }

    // Find the maximum frequency for scaling the y-axis
    let max_freq = *bins.iter().max().unwrap_or(&0);

    // Plot the histogram
    let root_area = BitMapBackend::new("latency_histogram.png", (800, 600)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root_area)
        .caption(
            "Gossip 95% Propagation Latency Distribution",
            ("sans-serif", 32).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(
            // X-axis: Latency in milliseconds (from smallest bin to biggest bin with some padding)
            (0..((NUM_BINS as u32 + 1) * bin_width as u32)).step(bin_width as u32),
            // Y-axis: Frequency (count)
            0u32..(max_freq + (max_freq / 10)), // Add a little padding to the top
        )?;

    chart
        .configure_mesh()
        .x_desc("Latency (milliseconds)")
        .y_desc("Frequency")
        .x_label_style(("sans-serif", 16).into_font())
        .y_label_style(("sans-serif", 16).into_font())
        .draw()?;

    chart.draw_series(Histogram::vertical(&chart).style(BLUE.filled()).data(
        // Map our binned data to what plotters expects: (bin_start, count)
        (0..NUM_BINS).map(|i| {
            let bin_start = i as u128 * bin_width;
            (bin_start as u32, bins[i])
        }),
    ))?;

    // Show count above each bin
    for (i, &count) in bins.iter().enumerate() {
        if count > 0 {
            let x_pos = i as u128 * bin_width;

            let label = Text::new(
                format!("{count}"),
                (x_pos as u32, count + 16),
                ("sans-serif", 16).into_font(),
            );

            // Draw the label directly onto the plotting area.
            chart.plotting_area().draw(&label)?;
        }
    }

    root_area.present()?;
    println!("Plot saved to {FILE_NAME}");
    Ok(())
}
