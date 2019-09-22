use apodize;
use hound;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FFTplanner;

type FrequencyBin = (f32, f32, f32);

fn get_freqs(input: &mut Vec<Complex<f32>>) -> Vec<FrequencyBin> {
    let bins: Vec<FrequencyBin> = input
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let amp = 20. * p.norm().log10();
            let freq = i as f32 * (44100f32 / input.len() as f32);
            let phase = p.re.atan2(p.im) * 180f32 / std::f32::consts::PI;
            (amp, freq, phase)
        })
        .collect();

    let max_freq = bins
        .iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(&(0f32, 0f32, 0f32));

	// A very scientific method to find frequency peaks: Just find all frequencies that have similar amplitude.
    let wiggle_room = max_freq.0 / 10f32;
    bins.iter()
        .cloned()
        .into_iter()
        .filter(|b| b.0 > max_freq.0 - wiggle_room && b.0 < max_freq.0 + wiggle_room)
        .collect()
}

fn generate_signal(input: Vec<FrequencyBin>, duration: usize) -> Vec<f32> {
    (0..duration)
        .into_iter()
        .map(|i| {
            let j = i as f32 / 44100f32;
            let mut amp = 0f32;

            input.iter().for_each(|(input_amp, freq, phase)| {
                let gen = (phase + (2f32 * freq * std::f32::consts::PI * j)).sin();
                amp = amp + gen * (100f32 / input_amp);
            });

            amp = amp * 0.25 * (std::i16::MAX as f32);
            let w: f32;

            if amp > std::i16::MAX as f32 {
                w = std::i16::MAX as f32;
            } else if amp < std::i16::MIN as f32 {
                w = std::i16::MIN as f32;
            } else {
                w = amp;
            }

            w
        })
        .collect()
}

fn apodize_signal(input: Vec<f32>) -> Vec<i16> {
    let window = apodize::hamming_iter(input.len()).collect::<Vec<f64>>();
    let mut windowed_data = vec![0i16; input.len()];

    for (windowed, (window, data)) in windowed_data
        .iter_mut()
        .zip(window.iter().zip(input.iter()))
    {
        *windowed = (*window * *data as f64) as i16;
    }

    windowed_data
}

/// If you want to tweak the params here's the gist:
/// reader: change the filename to the sound file you want to stretch (see readme for limitations)
/// writer: change the filename to the sound file you want to create (will be overwritten!)
/// fft_size: The amount of samples used to calculate the FFT.
///   Higher values = better resolution but worse performance
///   Lower values = worse resolution but better performance
/// .step_by(): How many samples are skipped ahead after each iteration. Should never be more than
///   `fft_size` to avoid artifacts like clicking
/// generate_signal(..., n): n is the param you want to tweak. It determines the duration of the generated
///   signal. If it is equal to `step_by` the sound won't be stretched. Higher numbers = more stretch.
///   The duration is measured as 1/sample_rate
/// 
/// How these values come together: Imagine an input file with 44100 samples and a sample rate of 44.1kHz. 
/// With a step_by value of 1470 you would have 30 iterations. You could use a fft_size of 2048 but not 1024. 
/// If you plug `44100` into the generate_signal function you'd get a resulting wave file that is 30 seconds long.
fn main() {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut reader = hound::WavReader::open("test2.wav").unwrap();
    let mut writer = hound::WavWriter::create("stretch2.wav", spec).unwrap();

    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    let itersamples = samples.clone();
	let fft_size = 2048;

    itersamples
        .into_iter()
        .enumerate()
        .step_by(1024)
        .for_each(|(i, sample)| {
            let mut frame = vec![sample];
            let mut framesamples = samples.clone().into_iter().skip(i + 1);

            (1..fft_size)
                .into_iter()
                .for_each(|_| frame.push(framesamples.next().unwrap_or(0)));

            let mut input: Vec<Complex<f32>> = frame
                .into_iter()
                .map(|s| Complex::new(s as f32, 0f32))
                .collect();
            let mut output: Vec<Complex<f32>> = vec![Complex::zero(); fft_size];

            let mut planner = FFTplanner::new(false);
            let fft = planner.plan_fft(fft_size);
            fft.process(&mut input, &mut output);

            let freq = get_freqs(&mut output);

            apodize_signal(generate_signal(freq, 2048))
                .iter()
                .for_each(|amp| {
                    writer.write_sample(*amp).unwrap();
                });
        });
}
